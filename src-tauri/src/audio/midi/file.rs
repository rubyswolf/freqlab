//! MIDI file parsing and playback
//!
//! Loads standard MIDI files (.mid) and extracts track information
//! for playback through instrument plugins.

use midly::{MetaMessage, MidiMessage, Smf, TrackEventKind};
use std::path::Path;

/// Information about a single track in a MIDI file
#[derive(Debug, Clone, serde::Serialize)]
pub struct MidiTrackInfo {
    /// Track index (0-based)
    pub index: usize,
    /// Track name from MIDI meta event (if present)
    pub name: Option<String>,
    /// Number of note events in this track
    pub note_count: usize,
    /// Primary MIDI channel used (0-15), if consistent
    pub channel: Option<u8>,
    /// Duration in beats
    pub duration_beats: f32,
}

/// A tempo change event in a MIDI file
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct TempoEvent {
    /// Position in beats where this tempo starts
    pub beat: f32,
    /// Tempo in BPM
    pub bpm: f32,
}

/// A note extracted from a MIDI file
#[derive(Debug, Clone, Copy)]
pub struct MidiFileNote {
    /// When to play, in beats (0.0 = start)
    pub beat: f32,
    /// MIDI note number (0-127)
    pub note: u8,
    /// Velocity (0-127)
    pub velocity: u8,
    /// Duration in beats
    pub duration: f32,
    /// MIDI channel (0-15)
    pub channel: u8,
}

/// A parsed MIDI file with track information
#[derive(Debug)]
pub struct ParsedMidiFile {
    /// File path
    pub path: String,
    /// Ticks per beat (from MIDI header)
    pub ticks_per_beat: u16,
    /// Initial BPM (tempo at beat 0)
    pub bpm: f32,
    /// Tempo map (list of tempo changes, always has at least one entry at beat 0)
    pub tempo_map: Vec<TempoEvent>,
    /// Track information
    pub tracks: Vec<MidiTrackInfo>,
    /// Notes extracted from each track (index matches tracks)
    pub track_notes: Vec<Vec<MidiFileNote>>,
}

impl ParsedMidiFile {
    /// Get notes for a specific track
    pub fn get_track_notes(&self, track_index: usize) -> Option<&[MidiFileNote]> {
        self.track_notes.get(track_index).map(|v| v.as_slice())
    }

    /// Check if the file has tempo automation (more than one tempo event)
    pub fn has_tempo_automation(&self) -> bool {
        self.tempo_map.len() > 1
    }
}

/// Information returned to frontend about a loaded MIDI file
#[derive(Debug, Clone, serde::Serialize)]
pub struct MidiFileInfo {
    /// File name (without path)
    pub filename: String,
    /// Default BPM from file
    pub bpm: f32,
    /// Total duration in beats (longest track)
    pub duration_beats: f32,
    /// Track information
    pub tracks: Vec<MidiTrackInfo>,
    /// Tempo map for automation
    pub tempo_map: Vec<TempoEvent>,
    /// Whether the file has tempo changes
    pub has_tempo_automation: bool,
}

/// Parse a MIDI file and extract track information
pub fn parse_midi_file(path: &Path) -> Result<ParsedMidiFile, String> {
    // Read file
    let data = std::fs::read(path)
        .map_err(|e| format!("Failed to read MIDI file: {}", e))?;

    // Parse MIDI
    let smf = Smf::parse(&data)
        .map_err(|e| format!("Failed to parse MIDI file: {}", e))?;

    // Get ticks per beat from header
    let ticks_per_beat = match smf.header.timing {
        midly::Timing::Metrical(tpb) => tpb.as_int(),
        midly::Timing::Timecode(fps, tpf) => {
            // Convert timecode to approximate ticks per beat
            // This is a rough approximation for SMPTE timing
            (fps.as_f32() * tpf as f32 / 2.0) as u16
        }
    };

    // Collect ALL tempo events from all tracks to build tempo map
    let mut tempo_map: Vec<TempoEvent> = Vec::new();

    for track in &smf.tracks {
        let mut current_tick: u32 = 0;
        for event in track {
            current_tick += event.delta.as_int();
            if let TrackEventKind::Meta(MetaMessage::Tempo(tempo)) = event.kind {
                let beat = current_tick as f32 / ticks_per_beat as f32;
                let event_bpm = 60_000_000.0 / tempo.as_int() as f32;
                tempo_map.push(TempoEvent { beat, bpm: event_bpm });
            }
        }
    }

    // Sort tempo map by beat position
    tempo_map.sort_by(|a, b| a.beat.partial_cmp(&b.beat).unwrap_or(std::cmp::Ordering::Equal));

    // Ensure there's always a tempo at beat 0 (MIDI spec default is 120 BPM)
    if tempo_map.is_empty() || tempo_map[0].beat > 0.0 {
        tempo_map.insert(0, TempoEvent { beat: 0.0, bpm: 120.0 });
    }

    // Initial BPM is the tempo at beat 0 (after sorting and adding default)
    let bpm = tempo_map[0].bpm;

    // Parse each track - collect all notes first
    let mut all_track_data: Vec<(usize, Option<String>, Vec<MidiFileNote>)> = Vec::new();

    for (track_idx, track) in smf.tracks.iter().enumerate() {
        let mut track_name: Option<String> = None;
        let mut notes: Vec<MidiFileNote> = Vec::new();
        let mut active_notes: std::collections::HashMap<(u8, u8), (f32, u8)> = std::collections::HashMap::new();
        let mut current_tick: u32 = 0;

        for event in track {
            current_tick += event.delta.as_int();
            let current_beat = current_tick as f32 / ticks_per_beat as f32;

            match event.kind {
                TrackEventKind::Meta(MetaMessage::TrackName(name_bytes)) => {
                    if let Ok(name) = std::str::from_utf8(name_bytes) {
                        track_name = Some(name.to_string());
                    }
                }
                TrackEventKind::Midi { channel, message } => {
                    let ch = channel.as_int();

                    match message {
                        MidiMessage::NoteOn { key, vel } => {
                            let note = key.as_int();
                            let velocity = vel.as_int();

                            if velocity > 0 {
                                active_notes.insert((ch, note), (current_beat, velocity));
                            } else {
                                if let Some((start_beat, vel)) = active_notes.remove(&(ch, note)) {
                                    let duration = (current_beat - start_beat).max(0.01);
                                    notes.push(MidiFileNote {
                                        beat: start_beat,
                                        note,
                                        velocity: vel,
                                        duration,
                                        channel: ch,
                                    });
                                }
                            }
                        }
                        MidiMessage::NoteOff { key, .. } => {
                            let note = key.as_int();
                            if let Some((start_beat, vel)) = active_notes.remove(&(ch, note)) {
                                let duration = (current_beat - start_beat).max(0.01);
                                notes.push(MidiFileNote {
                                    beat: start_beat,
                                    note,
                                    velocity: vel,
                                    duration,
                                    channel: ch,
                                });
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Close any remaining active notes
        let end_beat = current_tick as f32 / ticks_per_beat as f32;
        for ((ch, note), (start_beat, vel)) in active_notes {
            let duration = (end_beat - start_beat).max(0.01);
            notes.push(MidiFileNote {
                beat: start_beat,
                note,
                velocity: vel,
                duration,
                channel: ch,
            });
        }

        if !notes.is_empty() {
            all_track_data.push((track_idx, track_name, notes));
        }
    }

    // Now process tracks - split by channel if a track uses multiple channels
    let mut tracks = Vec::new();
    let mut track_notes = Vec::new();

    for (orig_idx, track_name, notes) in all_track_data {
        // Find unique channels in this track
        let mut channels: Vec<u8> = notes.iter().map(|n| n.channel).collect();
        channels.sort();
        channels.dedup();

        if channels.len() <= 1 {
            // Single channel (or no notes) - keep as one track
            let channel = channels.first().copied();
            let duration_beats = notes.iter()
                .map(|n| n.beat + n.duration)
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap_or(0.0);

            tracks.push(MidiTrackInfo {
                index: orig_idx,
                name: track_name,
                note_count: notes.len(),
                channel,
                duration_beats,
            });
            track_notes.push(notes);
        } else {
            // Multiple channels - split into virtual tracks per channel
            for ch in channels {
                let ch_notes: Vec<MidiFileNote> = notes.iter()
                    .filter(|n| n.channel == ch)
                    .copied()
                    .collect();

                if ch_notes.is_empty() {
                    continue;
                }

                let duration_beats = ch_notes.iter()
                    .map(|n| n.beat + n.duration)
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .unwrap_or(0.0);

                // Generate name based on channel (GM standard names for common channels)
                let ch_name = match ch {
                    9 => "Drums".to_string(),
                    _ => format!("Channel {}", ch + 1),
                };

                tracks.push(MidiTrackInfo {
                    index: orig_idx,
                    name: Some(ch_name),
                    note_count: ch_notes.len(),
                    channel: Some(ch),
                    duration_beats,
                });
                track_notes.push(ch_notes);
            }
        }
    }

    // Sort tracks by channel for consistent ordering
    let mut combined: Vec<_> = tracks.into_iter().zip(track_notes.into_iter()).collect();
    combined.sort_by_key(|(info, _)| info.channel.unwrap_or(255));
    let (tracks, track_notes): (Vec<_>, Vec<_>) = combined.into_iter().unzip();

    // If no tracks with notes found, return error
    if tracks.is_empty() {
        return Err("MIDI file contains no note data".to_string());
    }

    Ok(ParsedMidiFile {
        path: path.to_string_lossy().to_string(),
        ticks_per_beat,
        bpm,
        tempo_map,
        tracks,
        track_notes,
    })
}

/// Convert parsed MIDI file to info struct for frontend
pub fn get_midi_file_info(parsed: &ParsedMidiFile) -> MidiFileInfo {
    let filename = Path::new(&parsed.path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let duration_beats = parsed.tracks.iter()
        .map(|t| t.duration_beats)
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(0.0);

    MidiFileInfo {
        filename,
        bpm: parsed.bpm,
        duration_beats,
        tracks: parsed.tracks.clone(),
        tempo_map: parsed.tempo_map.clone(),
        has_tempo_automation: parsed.has_tempo_automation(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tempo_calculation() {
        // 500000 microseconds per beat = 120 BPM
        let bpm: f32 = 60_000_000.0 / 500_000.0;
        assert!((bpm - 120.0).abs() < 0.01);

        // 600000 microseconds per beat = 100 BPM
        let bpm: f32 = 60_000_000.0 / 600_000.0;
        assert!((bpm - 100.0).abs() < 0.01);
    }

}
