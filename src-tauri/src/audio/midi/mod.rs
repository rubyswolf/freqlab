//! MIDI handling for instrument plugins
//!
//! Provides MIDI event types, queuing, pattern playback, and MIDI file support.

mod events;
pub mod file;
pub mod patterns;
mod player;

pub use events::{MidiEvent, MidiEventQueue};
pub use file::{MidiFileInfo, MidiFileNote, MidiTrackInfo, ParsedMidiFile, TempoEvent, parse_midi_file, get_midi_file_info};
pub use patterns::{PatternCategory, PatternInfo, list_patterns, get_pattern};
pub use player::{MidiPlayer, PlaybackSource};
