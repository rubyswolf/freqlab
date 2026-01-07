//! CLAUDE.md generation for VST plugin projects
//!
//! Generates project-specific guidance files that Claude reads when helping
//! users develop their plugins. Ensures Claude knows to update both DSP and UI.

/// Generate the complete CLAUDE.md content for a project
pub fn generate_claude_md(
    project_name: &str,
    template: &str,
    ui_framework: &str,
    components: Option<&Vec<String>>,
) -> String {
    let mut content = String::new();

    // Header and config
    content.push_str(&generate_header(project_name, template, ui_framework, components));

    // Critical guidelines (always included)
    content.push_str(&generate_critical_guidelines(ui_framework));

    // UI framework specific section
    content.push_str(&generate_ui_framework_section(ui_framework));

    // Plugin type specific section
    content.push_str(&generate_plugin_type_section(template));

    // Component-specific sections
    if let Some(comps) = components {
        content.push_str(&generate_component_sections(comps));
    }

    content
}

fn generate_header(
    project_name: &str,
    template: &str,
    ui_framework: &str,
    components: Option<&Vec<String>>,
) -> String {
    let components_str = components
        .map(|c| {
            if c.is_empty() {
                "None".to_string()
            } else {
                c.join(", ")
            }
        })
        .unwrap_or_else(|| "None".to_string());

    format!(
        r#"# CLAUDE.md - Project Guidelines for {project_name}

> This file provides Claude with project-specific context and best practices.
> It is read into context when you chat about this project.

## Important: Keep This File Updated

**Claude: You should update this file** as the plugin evolves. When you add significant features, parameters, or architectural changes, update the "Current Implementation" section below. This is critical because:
- Chat context may be compacted over long sessions
- This file persists and is re-read on each message
- It serves as the source of truth for what exists in the plugin

## Project Configuration

- **Type**: {template}
- **UI Framework**: {ui_framework}
- **Components**: {components_str}

## Current Implementation

<!-- Claude: Update this section as you build the plugin -->

**Parameters:**
- *(None yet - add parameters here as you implement them)*

**Key Features:**
- *(None yet - add features here as you implement them)*

**Architecture Notes:**
- *(Add important architectural decisions here)*

"#
    )
}

fn generate_critical_guidelines(ui_framework: &str) -> String {
    let ui_step = match ui_framework {
        "headless" => "3. *(Skip - headless plugin has no custom UI)*",
        "webview" => "3. **Add UI control** in `src/ui.html` (slider, knob, etc.)",
        "egui" => "3. **Add UI widget** in the `editor()` method",
        _ => "3. **Add UI control** for the parameter",
    };

    let ipc_step = match ui_framework {
        "webview" => "4. **Add IPC handling** - message variant in Rust, handler in JS",
        "egui" => "4. **Wire up setter** - use `widgets::ParamSlider::for_param()`",
        "headless" => "4. *(Skip - no UI to wire up)*",
        _ => "4. **Wire up the UI** to the parameter",
    };

    format!(
        r#"## Critical Guidelines

### ALWAYS Update Both DSP and UI Together

When adding or modifying any feature that affects user-controllable parameters, complete ALL steps:

1. **Add the parameter** to the `Params` struct with `#[id = "param_name"]`
2. **Add DSP logic** in `process()` that uses `self.params.param_name.smoothed.next()`
{ui_step}
{ipc_step}

**Checklist for adding a new parameter (e.g., "mix"):**
- [ ] `mix: FloatParam` added to `Params` struct with proper range and default
- [ ] `self.params.mix.smoothed.next()` used in `process()` loop
- [ ] UI control added (unless headless)
- [ ] Parameter changes sync between UI and host automation

### Parameter Best Practices

- **IDs**: lowercase snake_case - `#[id = "filter_cutoff"]`
- **Names**: Human-readable - `FloatParam::new("Filter Cutoff", ...)`
- **Grouping**: Use prefixes - `osc1_pitch`, `osc2_pitch`, `filter_cutoff`, `env_attack`
- **Smoothing**: Always smooth parameters that directly affect audio to avoid clicks

### Safety Requirements

- **Always limit output**: Use `.clamp(-1.0, 1.0)` or a safety limiter on final output
- **Handle edge cases**: Check for zero/negative values, NaN, Inf
- **Smooth transitions**: Use parameter smoothing for any audio-rate changes

"#
    )
}

fn generate_ui_framework_section(ui_framework: &str) -> String {
    match ui_framework {
        "webview" => generate_webview_section(),
        "egui" => generate_egui_section(),
        "headless" => generate_headless_section(),
        _ => String::new(),
    }
}

fn generate_webview_section() -> String {
    r#"## WebView UI Framework

### File Structure
- **DSP/Plugin Logic**: `src/lib.rs`
- **UI**: `src/ui.html` (HTML + CSS + JavaScript)

### IPC Communication Pattern

The plugin and UI communicate via JSON messages:

**Rust → JavaScript** (parameter updates from host automation):
```rust
ctx.send_json(json!({
    "type": "param_change",
    "param": "filter_cutoff",
    "value": params.filter_cutoff.unmodulated_normalized_value(),
    "text": params.filter_cutoff.to_string()
}));
```

**JavaScript → Rust** (user adjusts UI control):
```javascript
sendToPlugin({ type: 'SetFilterCutoff', value: normalizedValue });
```

### Adding a New Parameter (Complete Flow)

**1. Rust side - Add message variant:**
```rust
#[derive(Deserialize)]
#[serde(tag = "type")]
enum UIMessage {
    // ... existing variants ...
    SetMyParam { value: f32 },  // ADD THIS
}
```

**2. Rust side - Handle the message:**
```rust
UIMessage::SetMyParam { value } => {
    setter.begin_set_parameter(&params.my_param);
    setter.set_parameter_normalized(&params.my_param, value);
    setter.end_set_parameter(&params.my_param);
}
```

**3. Rust side - Sync from host automation (add AtomicBool flag):**
```rust
// In struct: my_param_changed: Arc<AtomicBool>,
if my_param_changed.swap(false, Ordering::Relaxed) {
    ctx.send_json(json!({
        "type": "param_change",
        "param": "my_param",
        "value": params.my_param.unmodulated_normalized_value()
    }));
}
```

**4. JavaScript side - Add control and handlers:**
```html
<input type="range" id="my-param" min="0" max="1" step="0.001">
<script>
document.getElementById('my-param').addEventListener('input', (e) => {
    sendToPlugin({ type: 'SetMyParam', value: parseFloat(e.target.value) });
});
// In onPluginMessage handler:
if (msg.param === 'my_param') {
    document.getElementById('my-param').value = msg.value;
}
</script>
```

"#
    .to_string()
}

fn generate_egui_section() -> String {
    r#"## egui UI Framework

### File Structure
- **Everything in**: `src/lib.rs`
- UI code lives in the `editor()` method

### Adding Parameter Controls

Use the built-in `ParamSlider` widget for most parameters:

```rust
fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    let params = self.params.clone();
    create_egui_editor(
        self.params.editor_state.clone(),
        (),
        |_, _| {},
        move |egui_ctx, setter, _state| {
            egui::CentralPanel::default().show(egui_ctx, |ui| {
                ui.heading("My Plugin");

                // Simple parameter slider
                ui.label("Filter Cutoff");
                ui.add(widgets::ParamSlider::for_param(&params.filter_cutoff, setter));

                // Slider with custom width
                ui.add(widgets::ParamSlider::for_param(&params.resonance, setter)
                    .with_width(150.0));
            });
        },
    )
}
```

### Custom Displays

Read parameter values directly for custom visualizations:
```rust
let cutoff_hz = params.filter_cutoff.value();
ui.label(format!("Cutoff: {:.0} Hz", cutoff_hz));
```

### Layout Tips

- Use `ui.horizontal()` for side-by-side controls
- Use `ui.group()` to visually group related parameters
- Use `ui.separator()` between sections

"#
    .to_string()
}

fn generate_headless_section() -> String {
    r#"## Headless Plugin (No Custom UI)

This plugin has no custom graphical interface. Users interact through:
- DAW's generic parameter interface
- Automation lanes
- MIDI CC mapping (if enabled)

### Best Practices for Headless Plugins

1. **Clear parameter names**: Users only see the name in their DAW
2. **Sensible ranges**: Make defaults useful, ranges intuitive
3. **Good presets**: Consider adding factory presets for common use cases
4. **Proper units**: Use `formatters::v2s_f32_hz_then_khz` etc. for display

### Consider Adding

- Preset system for quick configuration
- Parameter groups for organization in DAW

"#
    .to_string()
}

fn generate_plugin_type_section(template: &str) -> String {
    match template {
        "effect" => generate_effect_section(),
        "instrument" => generate_instrument_section(),
        _ => String::new(),
    }
}

fn generate_effect_section() -> String {
    r#"## Effect Plugin Patterns

### Audio Flow
```
Input Buffer → Read Samples → Process/Transform → Safety Limit → Output Buffer
```

### Typical Process Loop

```rust
fn process(&mut self, buffer: &mut Buffer, ...) -> ProcessStatus {
    for channel_samples in buffer.iter_samples() {
        // Get smoothed parameter values (once per sample)
        let gain = self.params.gain.smoothed.next();
        let mix = self.params.mix.smoothed.next();

        for sample in channel_samples {
            let dry = *sample;
            let wet = self.process_sample(*sample);

            // Mix dry/wet
            *sample = dry * (1.0 - mix) + wet * mix;

            // Apply output gain and safety limit
            *sample *= gain;
            *sample = sample.clamp(-1.0, 1.0);
        }
    }
    ProcessStatus::Normal
}
```

### Common Effect Patterns

- **Filters**: Maintain state (previous samples) per channel
- **Delays**: Use ring buffers, handle sample rate changes
- **Distortion**: Apply nonlinear waveshaping, consider oversampling
- **Dynamics**: Track envelope, apply gain reduction

"#
    .to_string()
}

fn generate_instrument_section() -> String {
    r#"## Instrument Plugin Patterns

### MIDI Note Handling

```rust
fn process(&mut self, buffer: &mut Buffer, ..., context: &mut impl ProcessContext<Self>) -> ProcessStatus {
    // Process MIDI events
    while let Some(event) = context.next_event() {
        match event {
            NoteEvent::NoteOn { note, velocity, timing, .. } => {
                // Start/trigger a voice
                self.trigger_voice(note, velocity, timing);
            }
            NoteEvent::NoteOff { note, timing, .. } => {
                // Release the voice
                self.release_voice(note, timing);
            }
            _ => {}
        }
    }

    // Generate audio from active voices
    for channel_samples in buffer.iter_samples() {
        let output = self.render_voices();
        for sample in channel_samples {
            *sample = output.clamp(-1.0, 1.0);
        }
    }

    ProcessStatus::Normal
}
```

### Voice Management Considerations

- **Polyphony**: Track multiple simultaneous notes
- **Voice stealing**: When max voices reached, steal oldest/quietest
- **Note tracking**: Map note numbers to voice instances
- **Sample-accurate timing**: Use the `timing` field for precise triggering

### Common Instrument Patterns

- **Oscillators**: Phase accumulator, anti-aliasing for non-sine waves
- **Envelopes**: ADSR with configurable curves
- **Filters**: Per-voice or global filtering
- **LFOs**: Free-running or tempo-synced modulation

"#
    .to_string()
}

fn generate_component_sections(components: &[String]) -> String {
    let mut content = String::new();

    for component in components {
        let section = match component.as_str() {
            "preset_system" => generate_preset_system_section(),
            "param_smoothing" => generate_param_smoothing_section(),
            "sidechain_input" => generate_sidechain_section(),
            "oversampling" => generate_oversampling_section(),
            "polyphony" => generate_polyphony_section(),
            "velocity_layers" => generate_velocity_layers_section(),
            "adsr_envelope" => generate_adsr_section(),
            "lfo" => generate_lfo_section(),
            "custom_gui" => String::new(), // Covered by UI framework section
            _ => String::new(),
        };
        content.push_str(&section);
    }

    content
}

fn generate_preset_system_section() -> String {
    r#"## Preset System Guidelines

### State Persistence

Use `#[persist = "key"]` for non-parameter state that should save with presets:
```rust
#[persist = "editor_state"]
editor_state: Arc<EguiState>,
```

### Factory Presets

Embed presets in the binary for instant access:
```rust
const FACTORY_PRESETS: &[(&str, &str)] = &[
    ("Init", include_str!("../presets/init.json")),
    ("Warm", include_str!("../presets/warm.json")),
];
```

### User Presets

- Platform-specific storage: `~/Library/Application Support/` (macOS)
- Use plugin name in path to avoid conflicts
- Handle missing/corrupted preset files gracefully

"#
    .to_string()
}

fn generate_param_smoothing_section() -> String {
    r#"## Parameter Smoothing Guidelines

### When to Smooth

- **Always smooth**: Gain, filter cutoff, any parameter affecting audio directly
- **Don't smooth**: Discrete choices (waveform type), triggers

### Smoothing Styles

```rust
// Linear smoothing (good for most parameters)
FloatParam::new("Gain", 0.5, FloatRange::Linear { min: 0.0, max: 1.0 })
    .with_smoother(SmoothingStyle::Linear(50.0))  // 50ms

// Exponential (better for frequencies)
FloatParam::new("Cutoff", 1000.0, FloatRange::Skewed { min: 20.0, max: 20000.0, factor: 0.3 })
    .with_smoother(SmoothingStyle::Exponential(50.0))
```

### Using Smoothed Values

```rust
// In process() - call once per sample
let gain = self.params.gain.smoothed.next();
```

"#
    .to_string()
}

fn generate_sidechain_section() -> String {
    r#"## Sidechain Input Guidelines

### Configuration

```rust
fn audio_io_layout(&self) -> AudioIOLayout {
    AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[PortConfig::Stereo],  // Sidechain input
        ..Default::default()
    }
}
```

### Accessing Sidechain

```rust
fn process(&mut self, buffer: &mut Buffer, aux: &mut AuxiliaryBuffers, ...) {
    let sidechain = aux.inputs.first();
    // Use sidechain signal for ducking, gating, etc.
}
```

"#
    .to_string()
}

fn generate_oversampling_section() -> String {
    r#"## Oversampling Guidelines

### When to Oversample

- Distortion/saturation effects (reduces aliasing)
- Nonlinear processing with harmonics above Nyquist
- NOT needed for: linear filters, delays, simple gain

### Implementation Pattern

1. Upsample input (2x, 4x, or 8x)
2. Process at higher sample rate
3. Apply anti-aliasing lowpass filter
4. Downsample back to original rate

### Performance Considerations

- 2x oversampling doubles CPU usage
- Consider making it optional/adjustable
- Quality vs. performance tradeoff

"#
    .to_string()
}

fn generate_polyphony_section() -> String {
    r#"## Polyphony Guidelines

### Voice Structure

```rust
struct Voice {
    note: u8,
    velocity: f32,
    phase: f32,
    envelope: AdsrEnvelope,
    active: bool,
}

struct Synth {
    voices: [Voice; MAX_VOICES],
    // ...
}
```

### Voice Allocation

```rust
fn allocate_voice(&mut self, note: u8) -> Option<&mut Voice> {
    // First try: find free voice
    if let Some(v) = self.voices.iter_mut().find(|v| !v.active) {
        return Some(v);
    }
    // Voice stealing: take oldest/quietest
    self.voices.iter_mut().min_by_key(|v| /* priority */)
}
```

### Voice Rendering

```rust
fn render_voices(&mut self) -> f32 {
    self.voices
        .iter_mut()
        .filter(|v| v.active)
        .map(|v| v.render())
        .sum()
}
```

"#
    .to_string()
}

fn generate_velocity_layers_section() -> String {
    r#"## Velocity Layers Guidelines

### Layer Selection

Map MIDI velocity (0-127) to sample layers:
```rust
fn select_layer(&self, velocity: u8) -> usize {
    match velocity {
        0..=42 => 0,    // Soft
        43..=84 => 1,   // Medium
        85..=127 => 2,  // Hard
    }
}
```

### Crossfading Between Layers

For smoother transitions, crossfade adjacent layers based on velocity.

"#
    .to_string()
}

fn generate_adsr_section() -> String {
    r#"## ADSR Envelope Guidelines

### State Machine

```rust
enum EnvelopeStage { Attack, Decay, Sustain, Release, Idle }

struct AdsrEnvelope {
    stage: EnvelopeStage,
    level: f32,
    // Coefficients calculated from times
}
```

### Coefficient Calculation

```rust
// For exponential curves
let attack_coeff = 1.0 - (-1.0 / (attack_time * sample_rate)).exp();
```

### Per-Sample Processing

```rust
fn process(&mut self) -> f32 {
    match self.stage {
        EnvelopeStage::Attack => {
            self.level += self.attack_coeff * (1.0 - self.level);
            if self.level >= 0.999 { self.stage = EnvelopeStage::Decay; }
        }
        // ... other stages
    }
    self.level
}
```

"#
    .to_string()
}

fn generate_lfo_section() -> String {
    r#"## LFO Guidelines

### Basic Structure

```rust
struct Lfo {
    phase: f32,
    frequency: f32,
    waveform: LfoWaveform,
}

impl Lfo {
    fn process(&mut self, sample_rate: f32) -> f32 {
        let output = match self.waveform {
            LfoWaveform::Sine => (self.phase * TAU).sin(),
            LfoWaveform::Triangle => 1.0 - 4.0 * (self.phase - 0.5).abs(),
            LfoWaveform::Square => if self.phase < 0.5 { 1.0 } else { -1.0 },
            LfoWaveform::Saw => 2.0 * self.phase - 1.0,
        };

        self.phase += self.frequency / sample_rate;
        if self.phase >= 1.0 { self.phase -= 1.0; }

        output
    }
}
```

### Tempo Sync

Convert BPM to frequency: `frequency = bpm / 60.0 * note_division`

### Modulation Routing

Use LFO output to modulate other parameters (filter cutoff, amplitude, pitch).

"#
    .to_string()
}
