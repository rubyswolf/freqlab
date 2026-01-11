//! Core skills that are always generated for every plugin project
//!
//! These contain critical safety rules and framework essentials.

/// DSP Safety skill - anti-hallucination rules and safety guardrails
/// Invoke when implementing audio processing, filters, or any DSP code.
pub const DSP_SAFETY: &str = r#"---
name: dsp-safety
description: Critical DSP safety rules and anti-hallucination guardrails. Invoke when implementing audio processing, filters, effects, or any DSP code.
---

# DSP Safety Rules & Anti-Hallucination Guardrails

## Never Invent Filter Coefficients

**DO NOT** generate filter coefficient formulas from memory. Filter math is precise and errors cause broken audio. Always use:

- The `biquad` crate (implements Audio EQ Cookbook correctly)
- The `fundsp` crate for pre-built filters and DSP
- Reference: https://webaudio.github.io/Audio-EQ-Cookbook/audio-eq-cookbook.html

> **WARNING:** `synfx-dsp` requires **nightly Rust**. Use `biquad` or `fundsp` for stable Rust builds.

**Correct approach - use a crate:**
```rust
use biquad::{Biquad, Coefficients, DirectForm1, ToHertz, Type, Q_BUTTERWORTH_F32};

// Create filter
let coeffs = Coefficients::<f32>::from_params(
    Type::LowPass,
    sample_rate.hz(),
    cutoff_freq.hz(),
    Q_BUTTERWORTH_F32,
).unwrap();
let mut filter = DirectForm1::<f32>::new(coeffs);

// In process loop:
let filtered = filter.run(input_sample);
```

**Available biquad filter types** (use crate, don't calculate):
- `Type::LowPass`, `Type::HighPass`, `Type::BandPass`
- `Type::Notch`, `Type::AllPass`
- `Type::PeakingEQ`, `Type::LowShelf`, `Type::HighShelf`

## Parameter Smoothing (MANDATORY)

Every parameter that directly affects audio MUST be smoothed:

```rust
// nih-plug built-in smoothing
gain: FloatParam::new("Gain", 0.0, FloatRange::Linear { min: -30.0, max: 6.0 })
    .with_smoother(SmoothingStyle::Logarithmic(50.0))  // 50ms smoothing time
```

**Smoothing style guide:**
- `SmoothingStyle::Linear(ms)` - Good for most parameters
- `SmoothingStyle::Logarithmic(ms)` - Better for gain/volume (**WARNING: cannot cross zero!**)
- `SmoothingStyle::Exponential(ms)` - Better for frequencies

**Logarithmic Smoothing Limitation:**
`SmoothingStyle::Logarithmic` **cannot handle parameters that cross zero** (e.g., pan -1 to +1, bipolar modulation).
Use `SmoothingStyle::Linear` for bipolar parameters instead.

## Sample Rate Independence

**ALWAYS recalculate** time-based values when sample rate changes:

```rust
fn initialize(
    &mut self,
    _audio_io: &AudioIOLayout,
    buffer_config: &BufferConfig,
    _context: &mut impl InitContext<Self>
) -> bool {
    self.sample_rate = buffer_config.sample_rate;
    self.recalculate_time_constants();
    true
}

fn recalculate_time_constants(&mut self) {
    // Delay times - convert ms to samples
    self.delay_samples = (self.delay_ms * self.sample_rate / 1000.0) as usize;

    // LFO phase increment
    self.lfo_phase_inc = self.lfo_rate_hz / self.sample_rate;

    // Envelope coefficients
    self.attack_coeff = calc_coeff(self.attack_time, self.sample_rate);
}
```

## Realtime Safety (CRITICAL)

The audio thread (`process()`) must NEVER:

| Forbidden | Why | Alternative |
|-----------|-----|-------------|
| `Vec::push()`, `String::new()` | Memory allocation blocks | Pre-allocate in `initialize()` |
| `Mutex::lock()` | Can block indefinitely | Use `AtomicBool`, lock-free queues |
| File I/O | Blocks for disk | Load in background thread |
| `println!()`, `dbg!()` | I/O and allocation | Use `nih_log!()` sparingly |
| System calls | Unpredictable latency | Avoid entirely |

**Enable allocation detection in development:**
```toml
# Cargo.toml [features]
assert_process_allocs = ["nih_plug/assert_process_allocs"]
```

**Pre-allocate everything in initialize():**
```rust
fn initialize(&mut self, ...) -> bool {
    // Pre-allocate buffers at max expected size
    self.delay_buffer = vec![0.0; MAX_DELAY_SAMPLES];
    self.temp_buffer = vec![0.0; MAX_BLOCK_SIZE];
    true
}
```

## NaN/Inf Protection (MANDATORY)

Every plugin must protect against NaN/Inf values (which crash DAWs):

```rust
// In process() - after all DSP processing:
if !sample.is_finite() {
    *sample = 0.0;
}
```

**Note:** Do NOT use `sample.clamp(-1.0, 1.0)` as a safety limiter - this masks problems and breaks gain staging. The preview engine has its own output limiter for speaker protection. Let plugins output their true levels so users can see accurate metering.

## Anti-Hallucination Checklist

Before generating DSP code, verify:

- [ ] **Am I using a known algorithm?** Don't invent math - use established techniques
- [ ] **Are filter coefficients from a crate or cookbook?** Never calculate biquad coefficients from memory
- [ ] **Is sample rate used in ALL time-based calculations?** Delays, LFOs, envelopes all depend on it
- [ ] **Are parameters being smoothed?** Any audio-rate parameter change needs smoothing
- [ ] **Is NaN/Inf protected?** Output must be finite to prevent DAW crashes

## When Uncertain About DSP Math

If you're unsure about a DSP algorithm:

1. **Say so explicitly** - "I'm not certain about the exact coefficients for..."
2. **Recommend a crate** - `biquad` or `fundsp` handle most cases (stable Rust)
3. **Link to reference** - Audio EQ Cookbook, DAFX book, musicdsp.org
4. **Don't guess** - Wrong DSP math = broken audio or crashes

## Common Pitfalls to Avoid

| Pitfall | Problem | Solution |
|---------|---------|----------|
| Naive waveforms | Aliasing artifacts | Use PolyBLEP or wavetables |
| Instant parameter changes | Clicks and pops | Use smoothing (50ms typical) |
| Hardcoded sample rate | Broken at different rates | Always use `buffer_config.sample_rate` |
| Allocations in process() | Audio glitches | Pre-allocate in initialize() |
| NaN/Inf in output | DAW crash | Check `is_finite()`, set to 0.0 |
| Hand-rolled filter math | Wrong coefficients | Use `biquad` crate |
| Division by zero | NaN/Inf propagation | Guard all divisions |
| Unbounded feedback | Runaway levels | Limit feedback to < 1.0 or use tanh() |

## Implementing reset() (Important!)

The `reset()` method is called when playback stops or the plugin is bypassed. **Always implement this** to clear state:

```rust
fn reset(&mut self) {
    // Clear delay buffers to prevent old audio from playing
    self.delay_buffer.fill(0.0);

    // Reset filter state
    self.filter = DirectForm1::<f32>::new(self.current_coeffs.clone());

    // Reset envelopes to idle
    self.envelope.reset();
}
```

**When to implement reset():**
- Any plugin with delay lines (delay, reverb, chorus)
- Any plugin with filters (they have internal state)
- Instruments with envelopes
- Any effect that accumulates state over time
"#;

/// nih-plug Basics skill - framework essentials
/// Invoke when you need reference for plugin structure, parameters, or lifecycle.
pub const NIH_PLUG_BASICS: &str = r#"---
name: nih-plug-basics
description: Core nih-plug framework patterns, parameter setup, plugin lifecycle, and buffer processing. Invoke when setting up plugin structure or working with parameters.
---

# nih-plug Framework Essentials

This plugin uses [nih-plug](https://github.com/robbert-vdh/nih-plug), a Rust VST3/CLAP plugin framework.

## Plugin Trait Implementation

Every plugin implements the `Plugin` trait:

```rust
use nih_plug::prelude::*;
use std::sync::Arc;

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

    const MIDI_INPUT: MidiConfig = MidiConfig::None;   // or MidiConfig::Basic for instruments
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        // Audio processing here
        ProcessStatus::Normal
    }
}
```

## Parameter Types

Use the `#[derive(Params)]` macro with parameter fields:

```rust
#[derive(Params)]
struct MyPluginParams {
    #[id = "gain"]  // Stable ID for automation/presets
    pub gain: FloatParam,

    #[id = "bypass"]
    pub bypass: BoolParam,

    #[id = "mode"]
    pub mode: EnumParam<MyMode>,

    // For non-parameter persisted state (e.g., editor state)
    #[persist = "editor-state"]  // MUST be unique, non-empty string
    pub editor_state: Arc<SomeState>,
}
```

**CRITICAL: Persist Key Rules**
- Every `#[persist = "key"]` MUST have a **unique, non-empty key**
- Using `#[persist = ""]` (empty string) for multiple fields causes **compile/runtime errors**
- Keys must be unique across the entire Params struct
- Use descriptive keys: `"editor-state"`, `"gain-changed"`, not empty strings

```rust
// BAD - empty keys cause conflicts:
#[persist = ""]
gain_changed: Arc<AtomicBool>,
#[persist = ""]
filter_changed: Arc<AtomicBool>,  // ERROR: duplicate key!

// GOOD - unique descriptive keys:
#[persist = "gain-dirty"]
gain_changed: Arc<AtomicBool>,
#[persist = "filter-dirty"]
filter_changed: Arc<AtomicBool>,
```

### FloatParam
```rust
FloatParam::new("Gain", 0.0, FloatRange::Linear { min: -30.0, max: 30.0 })
    .with_unit(" dB")
    .with_smoother(SmoothingStyle::Logarithmic(50.0))
    .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
    .with_string_to_value(formatters::s2v_f32_gain_to_db())

// For gain parameters with proper skew:
FloatParam::new("Gain", util::db_to_gain(0.0), FloatRange::Skewed {
    min: util::db_to_gain(-30.0),
    max: util::db_to_gain(30.0),
    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
})
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
enum MyMode {
    #[name = "Clean"]
    Clean,
    #[name = "Warm"]
    Warm,
    #[name = "Aggressive"]
    Aggressive,
}

EnumParam::new("Mode", MyMode::Clean)
```

## Buffer Processing

### Per-Sample Processing (Most Common)
```rust
fn process(&mut self, buffer: &mut Buffer, ...) -> ProcessStatus {
    for channel_samples in buffer.iter_samples() {
        // Get smoothed parameter value (call once per sample)
        let gain = self.params.gain.smoothed.next();

        for sample in channel_samples {
            *sample = process_sample(*sample, gain);
            // Protect against NaN/Inf (crashes DAWs)
            if !sample.is_finite() { *sample = 0.0; }
        }
    }
    ProcessStatus::Normal
}
```

### Per-Channel Processing
```rust
for (channel_idx, channel) in buffer.as_slice().iter_mut().enumerate() {
    for sample in channel.iter_mut() {
        *sample = process_sample(*sample);
    }
}
```

## Getting Context Info

```rust
fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, context: &mut impl ProcessContext<Self>) -> ProcessStatus {
    let sample_rate = context.transport().sample_rate;
    let tempo = context.transport().tempo;           // BPM (Option<f64>)
    let playing = context.transport().playing;       // Is DAW playing?
    let pos_samples = context.transport().pos_samples(); // Position in samples
    // ...
}
```

## Smoothing Styles

```rust
SmoothingStyle::None              // No smoothing (for discrete values)
SmoothingStyle::Linear(50.0)      // 50ms linear interpolation
SmoothingStyle::Logarithmic(50.0) // 50ms log (better for gain) - CANNOT cross zero!
SmoothingStyle::Exponential(50.0) // 50ms exponential (better for frequencies)
```

**WARNING:** `SmoothingStyle::Logarithmic` cannot handle parameters that cross zero (e.g., pan -1 to +1). Use `Linear` for bipolar parameters.

## Reading Parameter Values

```rust
// In process() - use smoothed values for audio
let gain = self.params.gain.smoothed.next();

// In UI code - use unsmoothed for display
let gain_display = self.params.gain.value();

// Normalized value (0.0 to 1.0)
let normalized = self.params.gain.unmodulated_normalized_value();
```

## Plugin Format Traits

### CLAP Plugin
```rust
impl ClapPlugin for MyPlugin {
    const CLAP_ID: &'static str = "com.vendor.plugin-name";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("My audio plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,  // or ClapFeature::Instrument
        ClapFeature::Stereo,
    ];
}
```

### VST3 Plugin
```rust
impl Vst3Plugin for MyPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"MyPlugin16Chars!";  // Must be exactly 16 bytes
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Fx,  // or Vst3SubCategory::Synth
    ];
}
```

## Export Macros

At the end of lib.rs:
```rust
nih_export_clap!(MyPlugin);
nih_export_vst3!(MyPlugin);
```

## Lifecycle Methods

```rust
impl Plugin for MyPlugin {
    // Called when plugin loads or sample rate changes
    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        // Pre-allocate buffers, recalculate coefficients
        true  // Return false to indicate initialization failure
    }

    // Called when playback stops or plugin is bypassed
    fn reset(&mut self) {
        // Clear delay buffers, reset filters, reset envelopes
        self.delay_buffer.fill(0.0);
    }

    // Called before process() - can update latency
    fn latency_samples(&self) -> u32 {
        // Return latency in samples (for lookahead limiters, etc.)
        0
    }
}
```

## Common Imports

```rust
use nih_plug::prelude::*;
use std::sync::Arc;
```

## dB/Gain Conversions

```rust
// Using nih_plug utilities (preferred)
let linear = util::db_to_gain(-6.0);  // 0.5
let db = util::gain_to_db(0.5);       // -6.0

// Manual (if needed)
fn db_to_linear(db: f32) -> f32 { 10.0_f32.powf(db / 20.0) }
fn linear_to_db(linear: f32) -> f32 { 20.0 * linear.log10() }
```
"#;
