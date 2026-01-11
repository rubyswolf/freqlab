//! Optional component skills
//!
//! These skills are generated only when the component is selected during project creation.

/// Returns the skill content for a component, or None if not found
pub fn get_component_skill(component: &str) -> Option<&'static str> {
    match component {
        "preset_system" => Some(PRESET_SYSTEM),
        "param_smoothing" => Some(PARAM_SMOOTHING),
        "sidechain_input" => Some(SIDECHAIN_INPUT),
        "oversampling" => Some(OVERSAMPLING),
        "polyphony" => Some(POLYPHONY),
        "velocity_layers" => Some(VELOCITY_LAYERS),
        "adsr_envelope" => Some(ADSR_ENVELOPE),
        "lfo" => Some(LFO),
        _ => None,
    }
}

/// Preset system skill - State persistence, factory presets, user presets
pub const PRESET_SYSTEM: &str = r#"---
name: preset-system
description: Preset system implementation. State persistence, factory presets, user preset management. Invoke when implementing preset save/load functionality.
---

# Preset System Implementation

## State Persistence

Use `#[persist = "key"]` for non-parameter state that should save with presets:

```rust
#[derive(Params)]
struct MyParams {
    #[id = "gain"]
    pub gain: FloatParam,

    // Non-parameter state persisted with presets
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    // For UI change tracking
    #[persist = "gain-dirty"]
    gain_changed: Arc<AtomicBool>,
}
```

**CRITICAL: Persist Key Rules**
- Every `#[persist = "key"]` MUST have a **unique, non-empty key**
- Using `#[persist = ""]` (empty string) for multiple fields causes **compile/runtime errors**
- Keys must be unique across the entire Params struct

## Factory Presets

Embed presets in the binary for instant access:

```rust
const FACTORY_PRESETS: &[(&str, &str)] = &[
    ("Init", include_str!("../presets/init.json")),
    ("Warm", include_str!("../presets/warm.json")),
    ("Aggressive", include_str!("../presets/aggressive.json")),
];

impl Default for MyParams {
    fn default() -> Self {
        // Load init preset or use hardcoded defaults
        Self { /* ... */ }
    }
}
```

## User Presets

### Storage Locations

Platform-specific preset storage:
- **macOS**: `~/Library/Application Support/{PluginName}/Presets/`
- **Windows**: `%APPDATA%/{PluginName}/Presets/`
- **Linux**: `~/.config/{PluginName}/Presets/`

### Preset File Format

Use JSON for human-readable, versionable presets:

```json
{
    "name": "Warm Pad",
    "version": "1.0",
    "parameters": {
        "gain": 0.75,
        "cutoff": 2000.0,
        "resonance": 0.3
    }
}
```

### Error Handling

Handle missing or corrupted preset files gracefully:

```rust
fn load_preset(&mut self, path: &Path) -> Result<(), PresetError> {
    let content = std::fs::read_to_string(path)
        .map_err(|_| PresetError::FileNotFound)?;

    let preset: PresetData = serde_json::from_str(&content)
        .map_err(|_| PresetError::InvalidFormat)?;

    // Validate version compatibility
    if !self.is_compatible_version(&preset.version) {
        return Err(PresetError::IncompatibleVersion);
    }

    self.apply_preset(&preset);
    Ok(())
}
```

## UI Integration

### WebView Presets

```javascript
// Request preset list on init
sendToPlugin({ type: 'GetPresets' });

// Handle preset list response
window.onPluginMessage = function(msg) {
    if (msg.type === 'presets') {
        populatePresetDropdown(msg.factory, msg.user);
    }
};

// Load preset
function loadPreset(name, isFactory) {
    sendToPlugin({ type: 'LoadPreset', name, isFactory });
}
```

### egui Presets

```rust
egui::ComboBox::from_label("Preset")
    .selected_text(&current_preset_name)
    .show_ui(ui, |ui| {
        ui.label("Factory");
        for preset in &factory_presets {
            if ui.selectable_label(false, preset.name).clicked() {
                // Load preset
            }
        }
        ui.separator();
        ui.label("User");
        for preset in &user_presets {
            if ui.selectable_label(false, preset.name).clicked() {
                // Load preset
            }
        }
    });
```
"#;

/// Parameter smoothing skill - Smoothing styles, when to smooth
pub const PARAM_SMOOTHING: &str = r#"---
name: param-smoothing
description: Advanced parameter smoothing techniques. Smoothing styles, when to smooth, and avoiding artifacts. Invoke when fine-tuning parameter behavior.
---

# Parameter Smoothing

## When to Smooth

| Parameter Type | Smooth? | Style | Time |
|---------------|---------|-------|------|
| Gain/Volume | Yes | Logarithmic | 50ms |
| Filter Cutoff | Yes | Exponential | 50ms |
| Pan | Yes | Linear | 20ms |
| Mix (Dry/Wet) | Yes | Linear | 50ms |
| Waveform Select | No | None | - |
| Bypass Toggle | No | None | - |
| Tempo/BPM | No | None | - |

## Smoothing Styles

```rust
use nih_plug::prelude::*;

// Linear - good for most parameters
FloatParam::new("Gain", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
    .with_smoother(SmoothingStyle::Linear(50.0))  // 50ms

// Logarithmic - better for gain (perceptually linear)
// WARNING: Cannot cross zero! Don't use for bipolar params (-1 to +1)
FloatParam::new("Output", util::db_to_gain(0.0), FloatRange::Skewed {
    min: util::db_to_gain(-60.0),
    max: util::db_to_gain(12.0),
    factor: FloatRange::gain_skew_factor(-60.0, 12.0),
})
.with_smoother(SmoothingStyle::Logarithmic(50.0))

// Exponential - better for frequencies
FloatParam::new("Cutoff", 1000.0, FloatRange::Skewed {
    min: 20.0,
    max: 20000.0,
    factor: FloatRange::skew_factor(-2.0),
})
.with_smoother(SmoothingStyle::Exponential(50.0))
```

## Using Smoothed Values

```rust
fn process(&mut self, buffer: &mut Buffer, ...) -> ProcessStatus {
    for channel_samples in buffer.iter_samples() {
        // Call smoothed.next() ONCE per sample
        let gain = self.params.gain.smoothed.next();
        let cutoff = self.params.cutoff.smoothed.next();

        // Update filter only if cutoff changed significantly
        if self.params.cutoff.smoothed.is_smoothing() {
            self.update_filter_coefficients(cutoff);
        }

        for sample in channel_samples {
            *sample *= gain;
            *sample = self.filter.process(*sample);
        }
    }
    ProcessStatus::Normal
}
```

## Avoiding Clicks (Bipolar Parameters)

For parameters that cross zero (like pan -1 to +1):

```rust
// WRONG - Logarithmic can't cross zero
FloatParam::new("Pan", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 })
    .with_smoother(SmoothingStyle::Logarithmic(50.0))  // BROKEN!

// CORRECT - Use Linear for bipolar
FloatParam::new("Pan", 0.0, FloatRange::Linear { min: -1.0, max: 1.0 })
    .with_smoother(SmoothingStyle::Linear(20.0))  // Works correctly
```

## Manual Smoothing (When Needed)

For special cases where built-in smoothing isn't enough:

```rust
struct ManualSmoother {
    current: f32,
    target: f32,
    coeff: f32,  // Smoothing coefficient
}

impl ManualSmoother {
    fn new(initial: f32, time_ms: f32, sample_rate: f32) -> Self {
        Self {
            current: initial,
            target: initial,
            coeff: 1.0 - (-1.0 / (time_ms * 0.001 * sample_rate)).exp(),
        }
    }

    fn set_target(&mut self, target: f32) {
        self.target = target;
    }

    fn next(&mut self) -> f32 {
        self.current += self.coeff * (self.target - self.current);
        self.current
    }
}
```
"#;

/// Sidechain input skill - Aux input configuration, sidechain processing
pub const SIDECHAIN_INPUT: &str = r#"---
name: sidechain-input
description: Sidechain input implementation. Aux input configuration, accessing sidechain signal, ducking/gating. Invoke when adding sidechain functionality.
---

# Sidechain Input

## Audio I/O Configuration

Configure auxiliary inputs in the Plugin trait:

```rust
const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
    AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[
            // Sidechain input (stereo)
            AuxPort {
                name: "Sidechain",
                num_channels: 2,
            },
        ],
        ..AudioIOLayout::const_default()
    }
];
```

## Accessing Sidechain in process()

```rust
fn process(
    &mut self,
    buffer: &mut Buffer,
    aux: &mut AuxiliaryBuffers,
    _context: &mut impl ProcessContext<Self>,
) -> ProcessStatus {
    // Get sidechain buffer (may not be connected!)
    let sidechain = aux.inputs.first();

    for (sample_idx, mut channel_samples) in buffer.iter_samples().enumerate() {
        // Get sidechain level (handle disconnected case)
        let sidechain_level = if let Some(sc) = sidechain {
            let sc_samples = sc.as_slice();
            // Average left and right channels
            let sc_left = sc_samples.get(0).map(|c| c[sample_idx]).unwrap_or(0.0);
            let sc_right = sc_samples.get(1).map(|c| c[sample_idx]).unwrap_or(0.0);
            (sc_left.abs() + sc_right.abs()) * 0.5
        } else {
            0.0  // No sidechain connected
        };

        // Use sidechain_level for ducking, gating, etc.
        let duck_amount = self.calculate_duck(sidechain_level);

        for sample in channel_samples.iter_mut() {
            *sample *= duck_amount;
        }
    }

    ProcessStatus::Normal
}
```

## Sidechain Ducking (Compressor Style)

```rust
struct SidechainDucker {
    envelope: f32,
    attack_coeff: f32,
    release_coeff: f32,
    threshold: f32,
    ratio: f32,
}

impl SidechainDucker {
    fn process(&mut self, sidechain_level: f32) -> f32 {
        // Envelope follower
        let coeff = if sidechain_level > self.envelope {
            self.attack_coeff
        } else {
            self.release_coeff
        };
        self.envelope += coeff * (sidechain_level - self.envelope);

        // Calculate gain reduction
        if self.envelope > self.threshold {
            let over = self.envelope / self.threshold;
            over.powf(1.0 / self.ratio - 1.0)
        } else {
            1.0
        }
    }
}
```

## Sidechain Gate

```rust
struct SidechainGate {
    envelope: f32,
    attack_coeff: f32,
    release_coeff: f32,
    threshold: f32,
    hold_samples: usize,
    hold_counter: usize,
}

impl SidechainGate {
    fn process(&mut self, sidechain_level: f32) -> f32 {
        // Update envelope
        let coeff = if sidechain_level > self.envelope {
            self.attack_coeff
        } else {
            self.release_coeff
        };
        self.envelope += coeff * (sidechain_level - self.envelope);

        // Gate logic with hold
        if self.envelope > self.threshold {
            self.hold_counter = self.hold_samples;
            1.0  // Gate open
        } else if self.hold_counter > 0 {
            self.hold_counter -= 1;
            1.0  // Hold period
        } else {
            0.0  // Gate closed
        }
    }
}
```

## DAW Compatibility Notes

- Most DAWs support sidechain routing, but UI varies
- Test in multiple DAWs (Logic, Ableton, FL Studio, etc.)
- Consider adding a "Listen to Sidechain" toggle for debugging
"#;

/// Oversampling skill - When to oversample, implementation patterns
pub const OVERSAMPLING: &str = r#"---
name: oversampling
description: Oversampling implementation for nonlinear processing. When to oversample, quality vs performance tradeoffs. Invoke when implementing distortion or saturation effects.
---

# Oversampling

## When to Oversample

| Processing Type | Oversample? | Why |
|----------------|-------------|-----|
| Distortion/Saturation | Yes | Creates harmonics that alias |
| Waveshaping | Yes | Nonlinear = new frequencies |
| Clipping | Yes | Hard edges = infinite harmonics |
| Linear filters | No | No new frequencies created |
| Delay/Reverb | No | Just copying/mixing samples |
| Gain/Pan | No | Linear operations |

## Oversampling Factor Guide

| Factor | Quality | CPU Cost | Use Case |
|--------|---------|----------|----------|
| 2x | Good | 2x | Light saturation |
| 4x | Better | 4x | Moderate distortion |
| 8x | Excellent | 8x | Heavy distortion |
| 16x | Overkill | 16x | Extreme processing |

## Basic Pattern

```rust
use rubato::{FftFixedIn, Resampler};

struct OversampledProcessor {
    upsampler: FftFixedIn<f32>,
    downsampler: FftFixedIn<f32>,
    oversampling_factor: usize,
    upsampled_buffer: Vec<f32>,
}

impl OversampledProcessor {
    fn process(&mut self, input: &[f32], output: &mut [f32]) {
        // 1. Upsample
        let upsampled = self.upsampler.process(&[input], None).unwrap();

        // 2. Process at higher sample rate
        for sample in &mut upsampled[0] {
            *sample = self.apply_distortion(*sample);
        }

        // 3. Downsample (includes anti-aliasing filter)
        let downsampled = self.downsampler.process(&upsampled, None).unwrap();

        output.copy_from_slice(&downsampled[0]);
    }

    fn apply_distortion(&self, x: f32) -> f32 {
        x.tanh()  // Or your distortion function
    }
}
```

## Manual Oversampling (Without External Crate)

For simple 2x oversampling:

```rust
struct Simple2xOversampler {
    // Upsampling filter (interpolation)
    up_filter: [f32; 4],
    up_history: [f32; 4],

    // Downsampling filter (anti-aliasing)
    down_filter: [f32; 4],
    down_history: [f32; 8],
}

impl Simple2xOversampler {
    fn process_sample(&mut self, input: f32) -> f32 {
        // Upsample: insert input and zero
        let up1 = self.upsample_filter(input);
        let up2 = self.upsample_filter(0.0);

        // Process both samples at 2x rate
        let processed1 = self.apply_distortion(up1);
        let processed2 = self.apply_distortion(up2);

        // Downsample: filter and decimate
        self.downsample_filter(processed1);
        self.downsample_filter(processed2)
    }
}
```

## Performance Optimization

Make oversampling quality adjustable:

```rust
#[derive(Enum, PartialEq, Clone)]
pub enum OversampleQuality {
    #[name = "Off (Fastest)"]
    Off,
    #[name = "2x (Good)"]
    X2,
    #[name = "4x (Better)"]
    X4,
    #[name = "8x (Best)"]
    X8,
}

// In params
#[id = "oversample"]
pub oversample: EnumParam<OversampleQuality>,

// In process
let factor = match self.params.oversample.value() {
    OversampleQuality::Off => 1,
    OversampleQuality::X2 => 2,
    OversampleQuality::X4 => 4,
    OversampleQuality::X8 => 8,
};
```

## Latency Reporting

Oversampling adds latency. Report it to the DAW:

```rust
fn latency_samples(&self) -> u32 {
    // FIR filter delay from oversampling
    self.oversampler.latency() as u32
}
```
"#;

/// Polyphony skill - Voice management, allocation, rendering
pub const POLYPHONY: &str = r#"---
name: polyphony
description: Polyphony implementation. Voice structure, allocation, stealing, and efficient rendering. Invoke when implementing polyphonic instruments.
---

# Polyphony

## Voice Structure

```rust
const MAX_VOICES: usize = 16;

#[derive(Clone)]
struct Voice {
    note: u8,
    velocity: f32,
    phase: f32,
    frequency: f32,
    envelope: AdsrEnvelope,
    active: bool,
    age: u32,  // For voice stealing
}

impl Voice {
    fn new() -> Self {
        Self {
            note: 0,
            velocity: 0.0,
            phase: 0.0,
            frequency: 440.0,
            envelope: AdsrEnvelope::new(),
            active: false,
            age: 0,
        }
    }

    fn trigger(&mut self, note: u8, velocity: f32, sample_rate: f32) {
        self.note = note;
        self.velocity = velocity / 127.0;
        self.frequency = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
        self.phase = 0.0;
        self.envelope.trigger();
        self.active = true;
        self.age = 0;
    }

    fn release(&mut self) {
        self.envelope.release();
    }

    fn render(&mut self, sample_rate: f32) -> f32 {
        if !self.active {
            return 0.0;
        }

        self.age += 1;

        // Generate oscillator output
        let osc = (self.phase * std::f32::consts::TAU).sin();
        self.phase += self.frequency / sample_rate;
        if self.phase >= 1.0 { self.phase -= 1.0; }

        // Apply envelope
        let env = self.envelope.process();

        // Mark inactive when envelope finishes
        if self.envelope.is_idle() {
            self.active = false;
        }

        osc * env * self.velocity
    }
}
```

## Voice Allocation

```rust
struct VoiceAllocator {
    voices: [Voice; MAX_VOICES],
}

impl VoiceAllocator {
    fn allocate(&mut self, note: u8, velocity: f32, sample_rate: f32) {
        // Try to find free voice
        if let Some(voice) = self.voices.iter_mut().find(|v| !v.active) {
            voice.trigger(note, velocity, sample_rate);
            return;
        }

        // Voice stealing: find best candidate
        let steal_idx = self.find_voice_to_steal();
        self.voices[steal_idx].trigger(note, velocity, sample_rate);
    }

    fn find_voice_to_steal(&self) -> usize {
        // Priority: released voices first, then oldest
        let mut best_idx = 0;
        let mut best_score = 0u64;

        for (i, voice) in self.voices.iter().enumerate() {
            let score = if voice.envelope.is_releasing() {
                // Prefer releasing voices
                (1u64 << 32) + voice.age as u64
            } else {
                voice.age as u64
            };

            if score > best_score {
                best_score = score;
                best_idx = i;
            }
        }

        best_idx
    }

    fn release(&mut self, note: u8) {
        // Release all voices playing this note
        for voice in &mut self.voices {
            if voice.active && voice.note == note {
                voice.release();
            }
        }
    }

    fn release_all(&mut self) {
        for voice in &mut self.voices {
            voice.release();
        }
    }
}
```

## Efficient Rendering

```rust
fn render_all_voices(&mut self, sample_rate: f32) -> f32 {
    let mut output = 0.0;

    for voice in &mut self.voices {
        if voice.active {
            output += voice.render(sample_rate);
        }
    }

    // Prevent clipping with many voices
    output * (1.0 / (MAX_VOICES as f32).sqrt())
}
```

## Mono Mode (Single Voice)

```rust
fn trigger_mono(&mut self, note: u8, velocity: f32, sample_rate: f32) {
    // Always use voice 0 for mono
    let voice = &mut self.voices[0];

    // Legato: don't reset envelope if already playing
    let legato = voice.active;

    voice.note = note;
    voice.velocity = velocity / 127.0;
    voice.frequency = midi_to_freq(note);

    if !legato {
        voice.envelope.trigger();
    }

    voice.active = true;
}
```
"#;

/// Velocity layers skill - Sample layer selection, crossfading
pub const VELOCITY_LAYERS: &str = r#"---
name: velocity-layers
description: Velocity layers for samplers. Layer selection, crossfading between layers, realistic dynamics. Invoke when implementing velocity-sensitive sample playback.
---

# Velocity Layers

## Basic Layer Selection

Map MIDI velocity (0-127) to sample layers:

```rust
struct VelocityMappedSampler {
    layers: Vec<Vec<f32>>,  // Multiple samples per note
}

impl VelocityMappedSampler {
    fn select_layer(&self, velocity: u8) -> usize {
        let num_layers = self.layers.len();

        match num_layers {
            1 => 0,
            2 => if velocity < 64 { 0 } else { 1 },
            3 => match velocity {
                0..=42 => 0,    // Soft
                43..=84 => 1,   // Medium
                85..=127 => 2,  // Hard
            },
            4 => match velocity {
                0..=31 => 0,    // pp
                32..=63 => 1,   // p
                64..=95 => 2,   // f
                96..=127 => 3,  // ff
            },
            _ => ((velocity as usize * num_layers) / 128).min(num_layers - 1),
        }
    }
}
```

## Velocity Crossfading

For smoother transitions between layers:

```rust
struct CrossfadeSampler {
    layers: Vec<Vec<f32>>,
    crossfade_range: u8,  // Velocity range for crossfade
}

impl CrossfadeSampler {
    fn get_layers_with_mix(&self, velocity: u8) -> Vec<(usize, f32)> {
        let num_layers = self.layers.len();
        if num_layers == 1 {
            return vec![(0, 1.0)];
        }

        // Calculate layer boundaries
        let layer_size = 128.0 / num_layers as f32;
        let position = velocity as f32 / layer_size;

        let lower_layer = (position.floor() as usize).min(num_layers - 1);
        let upper_layer = (lower_layer + 1).min(num_layers - 1);

        if lower_layer == upper_layer {
            return vec![(lower_layer, 1.0)];
        }

        // Crossfade between layers
        let crossfade_pos = position - position.floor();
        let lower_gain = 1.0 - crossfade_pos;
        let upper_gain = crossfade_pos;

        vec![
            (lower_layer, lower_gain),
            (upper_layer, upper_gain),
        ]
    }

    fn render(&mut self, voice: &Voice) -> f32 {
        let layers_mix = self.get_layers_with_mix((voice.velocity * 127.0) as u8);

        let mut output = 0.0;
        for (layer_idx, gain) in layers_mix {
            let sample = self.read_sample(&self.layers[layer_idx], voice.position);
            output += sample * gain;
        }

        output
    }
}
```

## Velocity Scaling

Apply velocity to amplitude (not just layer selection):

```rust
fn velocity_to_amplitude(velocity: f32, curve: VelocityCurve) -> f32 {
    match curve {
        // Linear: Direct mapping
        VelocityCurve::Linear => velocity,

        // Soft: More dynamic range in soft playing
        VelocityCurve::Soft => velocity.powf(0.5),

        // Hard: More dynamic range in loud playing
        VelocityCurve::Hard => velocity.powf(2.0),

        // Fixed: Ignore velocity
        VelocityCurve::Fixed => 1.0,
    }
}
```

## Round Robin

Prevent "machine gun" effect with repeated notes:

```rust
struct RoundRobinSampler {
    layers: Vec<Vec<Vec<f32>>>,  // [velocity_layer][round_robin_variant]
    round_robin_index: Vec<usize>,  // Per velocity layer
}

impl RoundRobinSampler {
    fn get_sample(&mut self, velocity: u8) -> &[f32] {
        let layer = self.select_velocity_layer(velocity);

        // Get next round robin variant
        let rr_idx = self.round_robin_index[layer];
        let num_variants = self.layers[layer].len();

        // Advance round robin
        self.round_robin_index[layer] = (rr_idx + 1) % num_variants;

        &self.layers[layer][rr_idx]
    }
}
```
"#;

/// ADSR envelope skill - Envelope stages, coefficient calculation
pub const ADSR_ENVELOPE: &str = r#"---
name: adsr-envelope
description: ADSR envelope implementation. Exponential curves, stage transitions, retriggering. Invoke when implementing or debugging envelopes.
---

# ADSR Envelope

## Complete Implementation

```rust
#[derive(Clone, Copy, PartialEq)]
enum EnvelopeStage {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

#[derive(Clone)]
struct AdsrEnvelope {
    stage: EnvelopeStage,
    level: f32,

    // Coefficients (calculated from times)
    attack_coeff: f32,
    decay_coeff: f32,
    release_coeff: f32,

    sustain_level: f32,
}

impl AdsrEnvelope {
    fn new() -> Self {
        Self {
            stage: EnvelopeStage::Idle,
            level: 0.0,
            attack_coeff: 0.01,
            decay_coeff: 0.001,
            release_coeff: 0.001,
            sustain_level: 0.7,
        }
    }

    /// Set envelope times (in seconds)
    fn set_params(&mut self, attack: f32, decay: f32, sustain: f32, release: f32, sample_rate: f32) {
        // Time constant: reach ~99.3% of target in given time
        // coefficient = 1 - e^(-5/samples)
        let time_to_coeff = |time_s: f32| -> f32 {
            if time_s <= 0.0 {
                1.0  // Instant
            } else {
                let samples = time_s * sample_rate;
                1.0 - (-5.0 / samples).exp()
            }
        };

        self.attack_coeff = time_to_coeff(attack);
        self.decay_coeff = time_to_coeff(decay);
        self.release_coeff = time_to_coeff(release);
        self.sustain_level = sustain.clamp(0.0, 1.0);
    }

    fn trigger(&mut self) {
        self.stage = EnvelopeStage::Attack;
        // DON'T reset level - allows smooth retriggering
    }

    fn release(&mut self) {
        if self.stage != EnvelopeStage::Idle {
            self.stage = EnvelopeStage::Release;
        }
    }

    fn process(&mut self) -> f32 {
        match self.stage {
            EnvelopeStage::Idle => {
                self.level = 0.0;
            }

            EnvelopeStage::Attack => {
                // Exponential approach to 1.0
                self.level += self.attack_coeff * (1.0 - self.level);

                if self.level >= 0.999 {
                    self.level = 1.0;
                    self.stage = EnvelopeStage::Decay;
                }
            }

            EnvelopeStage::Decay => {
                // Exponential approach to sustain
                self.level += self.decay_coeff * (self.sustain_level - self.level);

                if (self.level - self.sustain_level).abs() < 0.001 {
                    self.level = self.sustain_level;
                    self.stage = EnvelopeStage::Sustain;
                }
            }

            EnvelopeStage::Sustain => {
                self.level = self.sustain_level;
            }

            EnvelopeStage::Release => {
                // Exponential approach to 0
                self.level += self.release_coeff * (0.0 - self.level);

                if self.level < 0.001 {
                    self.level = 0.0;
                    self.stage = EnvelopeStage::Idle;
                }
            }
        }

        self.level
    }

    fn is_idle(&self) -> bool {
        self.stage == EnvelopeStage::Idle
    }

    fn is_releasing(&self) -> bool {
        self.stage == EnvelopeStage::Release
    }
}
```

## Common Mistakes to Avoid

| Mistake | Problem | Fix |
|---------|---------|-----|
| Resetting level on retrigger | Click when retriggering | Don't reset `level` in `trigger()` |
| Linear attack | Unnatural sound | Use exponential curves |
| Instant release | Clicks on note-off | Minimum 5-10ms release |
| Wrong sample rate | Envelope times off | Recalculate in `initialize()` |
| Level > 1.0 | Clipping | Clamp sustain, check math |

## Modulation Envelope (Bipolar)

For filter cutoff modulation:

```rust
struct ModEnvelope {
    env: AdsrEnvelope,
    amount: f32,  // -1.0 to +1.0
}

impl ModEnvelope {
    fn process(&mut self) -> f32 {
        // Returns value in range [-amount, +amount]
        self.env.process() * self.amount
    }
}
```
"#;

/// LFO skill - Waveforms, tempo sync, modulation routing
pub const LFO: &str = r#"---
name: lfo
description: LFO implementation. Waveforms, tempo sync, modulation routing. Invoke when implementing low-frequency oscillators for modulation.
---

# LFO (Low Frequency Oscillator)

## Basic Implementation

```rust
use std::f32::consts::TAU;

#[derive(Clone, Copy, PartialEq)]
enum LfoWaveform {
    Sine,
    Triangle,
    Square,
    Saw,
    SawDown,
    SampleAndHold,
}

struct Lfo {
    phase: f32,
    frequency: f32,
    waveform: LfoWaveform,
    sample_hold_value: f32,
    last_phase: f32,
}

impl Lfo {
    fn new() -> Self {
        Self {
            phase: 0.0,
            frequency: 1.0,
            waveform: LfoWaveform::Sine,
            sample_hold_value: 0.0,
            last_phase: 0.0,
        }
    }

    fn set_frequency(&mut self, hz: f32) {
        self.frequency = hz.max(0.001);
    }

    fn process(&mut self, sample_rate: f32) -> f32 {
        let output = match self.waveform {
            LfoWaveform::Sine => (self.phase * TAU).sin(),

            LfoWaveform::Triangle => {
                let p = self.phase;
                if p < 0.5 {
                    4.0 * p - 1.0
                } else {
                    -4.0 * p + 3.0
                }
            }

            LfoWaveform::Square => {
                if self.phase < 0.5 { 1.0 } else { -1.0 }
            }

            LfoWaveform::Saw => {
                2.0 * self.phase - 1.0
            }

            LfoWaveform::SawDown => {
                1.0 - 2.0 * self.phase
            }

            LfoWaveform::SampleAndHold => {
                // Update value when phase wraps
                if self.phase < self.last_phase {
                    self.sample_hold_value = fastrand::f32() * 2.0 - 1.0;
                }
                self.last_phase = self.phase;
                self.sample_hold_value
            }
        };

        // Advance phase
        self.phase += self.frequency / sample_rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        output  // Returns -1.0 to +1.0
    }

    fn reset(&mut self) {
        self.phase = 0.0;
    }
}
```

## Tempo Sync

Convert tempo divisions to frequency:

```rust
#[derive(Clone, Copy)]
enum TempoDiv {
    Whole,      // 1/1
    Half,       // 1/2
    Quarter,    // 1/4
    Eighth,     // 1/8
    Sixteenth,  // 1/16
    DottedHalf,
    DottedQuarter,
    Triplet8th,
}

fn tempo_div_to_freq(bpm: f64, div: TempoDiv) -> f32 {
    let beats_per_sec = bpm / 60.0;

    let multiplier = match div {
        TempoDiv::Whole => 0.25,
        TempoDiv::Half => 0.5,
        TempoDiv::Quarter => 1.0,
        TempoDiv::Eighth => 2.0,
        TempoDiv::Sixteenth => 4.0,
        TempoDiv::DottedHalf => 1.0 / 3.0,
        TempoDiv::DottedQuarter => 2.0 / 3.0,
        TempoDiv::Triplet8th => 3.0,
    };

    (beats_per_sec * multiplier) as f32
}

// In process():
if self.params.lfo_sync.value() {
    if let Some(tempo) = context.transport().tempo {
        let freq = tempo_div_to_freq(tempo, self.params.lfo_div.value());
        self.lfo.set_frequency(freq);
    }
} else {
    self.lfo.set_frequency(self.params.lfo_rate.value());
}
```

## Modulation Routing

Apply LFO to parameters:

```rust
struct ModulationTarget {
    base_value: f32,
    lfo_amount: f32,  // -1.0 to +1.0
    lfo_value: f32,
}

impl ModulationTarget {
    fn get_modulated(&self) -> f32 {
        self.base_value + (self.lfo_value * self.lfo_amount * self.base_value)
    }
}

// In process:
let lfo_out = self.lfo.process(sample_rate);

// Modulate filter cutoff
let base_cutoff = self.params.cutoff.smoothed.next();
let lfo_amount = self.params.lfo_to_cutoff.value();
let modulated_cutoff = base_cutoff * (1.0 + lfo_out * lfo_amount);

// Clamp to valid range
let final_cutoff = modulated_cutoff.clamp(20.0, 20000.0);
```

## Per-Voice vs Global LFO

```rust
// Global LFO: Same phase for all voices (classic synth behavior)
struct Synth {
    lfo: Lfo,  // Single LFO
    voices: [Voice; MAX_VOICES],
}

// Per-voice LFO: Each voice has own phase (more organic)
struct Voice {
    lfo: Lfo,  // Each voice has its own
    // ...
}
```
"#;
