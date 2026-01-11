# nih-plug Quick Reference

## Plugin Trait Implementation

```rust
impl Plugin for MyPlugin {
    const NAME: &'static str = "Plugin Name";
    const VENDOR: &'static str = "Vendor";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        }
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;  // or MidiConfig::Basic
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> { self.params.clone() }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Process audio here
        ProcessStatus::Normal
    }
}
```

## Parameter Types

### FloatParam
```rust
FloatParam::new("Gain", 0.0, FloatRange::Linear { min: -30.0, max: 30.0 })
    .with_unit(" dB")
    .with_smoother(SmoothingStyle::Logarithmic(50.0))
    .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
    .with_string_to_value(formatters::s2v_f32_gain_to_db())

// Gain-specific range with proper skew
FloatRange::Skewed {
    min: util::db_to_gain(-30.0),
    max: util::db_to_gain(30.0),
    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
}
```

### IntParam
```rust
IntParam::new("Voices", 4, IntRange::Linear { min: 1, max: 16 })
```

### BoolParam
```rust
BoolParam::new("Bypass", false)
```

### EnumParam
```rust
#[derive(Enum, PartialEq)]
enum Mode {
    #[name = "Clean"]
    Clean,
    #[name = "Distorted"]
    Distorted,
}
EnumParam::new("Mode", Mode::Clean)
```

## Params Derive
```rust
#[derive(Params)]
struct PluginParams {
    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "bypass"]
    pub bypass: BoolParam,
}
```

## Buffer Processing

### Per-Sample (Most Common)
```rust
for channel_samples in buffer.iter_samples() {
    let gain = self.params.gain.smoothed.next();
    for sample in channel_samples {
        *sample = process_sample(*sample, gain);
    }
}
```

### Per-Channel
```rust
for (channel_idx, channel) in buffer.as_slice().iter_mut().enumerate() {
    for sample in channel.iter_mut() {
        *sample = process_sample(*sample);
    }
}
```

## Getting Context Info
```rust
// In process():
let sample_rate = _context.transport().sample_rate;
let tempo = _context.transport().tempo;
let playing = _context.transport().playing;
```

## MIDI Processing (Instruments)
```rust
while let Some(event) = context.next_event() {
    match event {
        NoteEvent::NoteOn { note, velocity, .. } => {
            let freq = 440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0);
            // Start note
        }
        NoteEvent::NoteOff { note, .. } => {
            // Stop note
        }
        _ => {}
    }
}
```

## Common DSP Functions

### Soft Clipping (tanh)
```rust
fn soft_clip(x: f32) -> f32 { x.tanh() }
```

### Hard Clipping
```rust
fn hard_clip(x: f32, threshold: f32) -> f32 {
    x.clamp(-threshold, threshold)
}
```

### NaN/Inf Protection
```rust
// Check AFTER all DSP - do NOT use clamp() as a limiter
if !sample.is_finite() { *sample = 0.0; }
```

### dB Conversions
```rust
fn db_to_linear(db: f32) -> f32 { 10.0_f32.powf(db / 20.0) }
fn linear_to_db(linear: f32) -> f32 { 20.0 * linear.log10() }
// Or use: util::db_to_gain(), util::gain_to_db()
```

### One-Pole Lowpass Filter
```rust
struct OnePole { z1: f32, a: f32 }

impl OnePole {
    fn new(cutoff_hz: f32, sample_rate: f32) -> Self {
        let a = (-2.0 * std::f32::consts::PI * cutoff_hz / sample_rate).exp();
        Self { z1: 0.0, a }
    }

    fn process(&mut self, x: f32) -> f32 {
        self.z1 = x * (1.0 - self.a) + self.z1 * self.a;
        self.z1
    }
}
```

### Biquad Filter
```rust
struct Biquad {
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
    x1: f32, x2: f32, y1: f32, y2: f32,
}

impl Biquad {
    fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
              - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}
```

## Export Macros
```rust
nih_export_clap!(MyPlugin);
nih_export_vst3!(MyPlugin);
```

## CLAP Plugin Trait
```rust
impl ClapPlugin for MyPlugin {
    const CLAP_ID: &'static str = "com.vendor.plugin-name";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("Description");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,  // or ClapFeature::Instrument
        ClapFeature::Stereo,
    ];
}
```

## VST3 Plugin Trait
```rust
impl Vst3Plugin for MyPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"UniqueId16Chars!";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,  // or Vst3SubCategory::Synth
    ];
}
```

## Smoothing Styles
```rust
SmoothingStyle::None           // No smoothing
SmoothingStyle::Linear(ms)     // Linear interpolation over ms
SmoothingStyle::Logarithmic(ms) // Log interpolation (better for gain)
SmoothingStyle::Exponential(ms) // Exponential decay
```

## Common Imports
```rust
use nih_plug::prelude::*;
use std::sync::Arc;
```
