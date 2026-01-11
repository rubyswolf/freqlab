//! UI Framework skills - WebView, egui, and Native patterns
//!
//! Only one of these is generated per project based on UI framework selection.

/// WebView UI skill - IPC patterns, AtomicBool sync, HTML/JS integration
pub const WEBVIEW_UI: &str = r#"---
name: webview-ui
description: WebView UI patterns for nih-plug-webview. IPC messaging, AtomicBool sync, HTML/JS integration. Invoke when working on UI code in webview projects.
---

# WebView UI Framework

## TWO FILES MUST BE MODIFIED FOR EVERY FEATURE

| File | Purpose | What to Add |
|------|---------|-------------|
| `src/lib.rs` | Rust DSP + IPC | Parameter, process() logic, UIMessage variant, handler |
| `src/ui.html` | User Interface | HTML control (slider/knob), JS event handlers |

**If you only edit `src/lib.rs`, the feature is INCOMPLETE.** Users need UI to control parameters.

## Required Imports for WebView

```rust
use nih_plug_webview::{WebViewEditor, HTMLSource, EventStatus};
use serde::Deserialize;
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
```

## Message Enum Pattern

Define typed messages for UI -> Plugin communication:

```rust
#[derive(Deserialize)]
#[serde(tag = "type")]
enum UIMessage {
    Init,                        // UI requests initial state
    SetGain { value: f32 },      // User adjusted gain slider
    SetFilterCutoff { value: f32 },
    // Add more as needed...
}
```

## AtomicBool Pattern for Host Automation Sync

When the DAW automates a parameter, you need to notify the UI:

```rust
#[derive(Params)]
struct MyParams {
    #[id = "gain"]
    pub gain: FloatParam,

    // For change tracking (must have unique key)
    #[persist = "gain-dirty"]
    gain_changed: Arc<AtomicBool>,
}

// In Default impl, add callback to the parameter:
impl Default for MyParams {
    fn default() -> Self {
        let gain_changed = Arc::new(AtomicBool::new(false));
        let gain_changed_clone = gain_changed.clone();

        Self {
            gain: FloatParam::new("Gain", util::db_to_gain(0.0), FloatRange::Skewed { ... })
                .with_callback(Arc::new(move |_| {
                    gain_changed_clone.store(true, Ordering::Relaxed);
                })),
            gain_changed,
        }
    }
}
```

## Editor Creation Pattern

```rust
fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    let params = self.params.clone();
    let gain_changed = self.params.gain_changed.clone();

    Some(Box::new(
        WebViewEditor::new(HTMLSource::String(include_str!("ui.html")), (400, 300))
            .with_background_color((26, 26, 46, 255))  // Match your UI background
            .with_developer_mode(cfg!(debug_assertions)) // DevTools in debug builds
            .with_event_loop(move |ctx, setter, _window| {
                // Handle messages from JavaScript
                while let Ok(msg) = ctx.next_event() {
                    match msg {
                        UIMessage::Init => {
                            // Send initial parameter values to UI
                            ctx.send_json(json!({
                                "type": "init",
                                "gain": params.gain.unmodulated_normalized_value(),
                            }));
                        }
                        UIMessage::SetGain { value } => {
                            setter.begin_set_parameter(&params.gain);
                            setter.set_parameter_normalized(&params.gain, value);
                            setter.end_set_parameter(&params.gain);
                        }
                        // Handle other messages...
                    }
                }

                // Check for host automation changes and notify UI
                if gain_changed.swap(false, Ordering::Relaxed) {
                    ctx.send_json(json!({
                        "type": "param_change",
                        "param": "gain",
                        "value": params.gain.unmodulated_normalized_value()
                    }));
                }

                EventStatus::Ignored
            })
    ))
}
```

## JavaScript Side (ui.html)

```html
<!DOCTYPE html>
<html>
<head>
    <style>
        body { background: #1a1a2e; color: #eee; font-family: sans-serif; }
        .slider { width: 200px; }
    </style>
</head>
<body>
    <h1>My Plugin</h1>
    <label>Gain: <input type="range" id="gain" class="slider" min="0" max="1" step="0.001"></label>

    <script>
        // Send message to plugin
        function sendToPlugin(msg) {
            window.ipc.postMessage(JSON.stringify(msg));
        }

        // Receive messages from plugin
        window.onPluginMessage = function(msg) {
            if (msg.type === 'init') {
                document.getElementById('gain').value = msg.gain;
            } else if (msg.type === 'param_change') {
                if (msg.param === 'gain') {
                    document.getElementById('gain').value = msg.value;
                }
            }
        };

        // Request initial state when page loads
        window.addEventListener('DOMContentLoaded', () => {
            sendToPlugin({ type: 'Init' });
        });

        // Handle user input
        document.getElementById('gain').addEventListener('input', (e) => {
            sendToPlugin({ type: 'SetGain', value: parseFloat(e.target.value) });
        });
    </script>
</body>
</html>
```

## Common Pitfalls to Avoid

| Pitfall | Problem | Solution |
|---------|---------|----------|
| Missing Init message | UI shows wrong values on open | Always handle `Init` and send current state |
| No AtomicBool callback | UI doesn't update from host automation | Add `.with_callback()` to each parameter |
| Feedback loop | UI -> Plugin -> UI infinite loop | JavaScript should ignore updates while dragging |
| Missing begin/end_set_parameter | Undo/redo doesn't work properly | Always wrap set_parameter_normalized |
| Empty persist keys | Compile errors | Use unique descriptive keys like `"gain-dirty"` |

## Feature Completion Checklist

Before saying a feature is "done", verify ALL boxes are checked:

- [ ] Parameter added to `Params` struct with `#[id = "..."]`
- [ ] DSP code uses the parameter via `.smoothed.next()` or `.value()`
- [ ] **UI CONTROL EXISTS in src/ui.html** - slider/knob/button in the UI
- [ ] UIMessage variant added for this parameter
- [ ] UI sends parameter changes to plugin (IPC)
- [ ] AtomicBool callback notifies UI of host automation changes
"#;

/// egui UI skill - widget patterns, ParamSlider, EguiState
pub const EGUI_UI: &str = r#"---
name: egui-ui
description: egui UI patterns for nih-plug-egui. ParamSlider widgets, EguiState, layout patterns. Invoke when working on UI in egui projects.
---

# egui UI Framework

## BOTH DSP AND UI ARE IN THE SAME FILE

| Location | Purpose | What to Add |
|----------|---------|-------------|
| `src/lib.rs` - `Params` struct | Parameters | `FloatParam`, `IntParam`, etc. |
| `src/lib.rs` - `process()` | DSP Logic | Use `params.x.smoothed.next()` |
| `src/lib.rs` - `editor()` | **UI Widgets** | `ParamSlider::for_param()` |

**If you only add the parameter and DSP, the feature is INCOMPLETE.** You MUST also add a widget in `editor()`.

## Required Imports for egui

```rust
use nih_plug_egui::{create_egui_editor, egui, widgets, EguiState};
```

## EguiState Setup (Required for Window Persistence)

```rust
#[derive(Params)]
struct MyPluginParams {
    #[persist = "editor-state"]  // Saves window size with presets
    editor_state: Arc<EguiState>,

    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for MyPluginParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(400, 300),  // Width x Height in logical pixels
            gain: FloatParam::new(...),
        }
    }
}
```

## Complete Editor Pattern

```rust
fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
    let params = self.params.clone();
    let peak_meter = self.peak_meter.clone();  // For visualizations

    create_egui_editor(
        self.params.editor_state.clone(),
        (),                              // User state (use () if not needed)
        |ctx, _| {                       // Build function (one-time setup)
            // Optional: Customize styling
            let mut style = (*ctx.style()).clone();
            style.visuals.window_fill = egui::Color32::from_rgb(26, 26, 46);
            ctx.set_style(style);
        },
        move |egui_ctx, setter, _state| {  // Update function (called every frame)
            egui::CentralPanel::default().show(egui_ctx, |ui| {
                ui.heading("My Plugin");
                ui.add_space(10.0);

                // Parameter slider with label
                ui.horizontal(|ui| {
                    ui.label("Gain:");
                    ui.add(widgets::ParamSlider::for_param(&params.gain, setter));
                });

                ui.add_space(10.0);

                // Peak meter visualization
                let peak = f32::from_bits(peak_meter.load(std::sync::atomic::Ordering::Relaxed));
                ui.add(egui::ProgressBar::new(peak).text(format!("{:.1} dB", util::gain_to_db(peak))));
            });
        },
    )
}
```

## ParamSlider Widget

The built-in `ParamSlider` handles all parameter binding automatically:

```rust
// Basic usage
ui.add(widgets::ParamSlider::for_param(&params.gain, setter));

// With custom width
ui.add(widgets::ParamSlider::for_param(&params.cutoff, setter).with_width(200.0));
```

This automatically handles:
- Displaying current value with proper formatting
- begin_set_parameter / end_set_parameter calls
- Drag interaction
- Value display using parameter's formatters

## Peak Meter Pattern (Audio -> GUI Communication)

For real-time visualizations, use `AtomicU32` to safely pass data from audio thread:

```rust
use std::sync::atomic::{AtomicU32, Ordering};

struct MyPlugin {
    params: Arc<MyPluginParams>,
    peak_meter: Arc<AtomicU32>,  // Store peak as bits (f32 -> u32)
}

// In process():
let peak = buffer.iter_samples()
    .map(|s| s.iter().map(|x| x.abs()).fold(0.0f32, f32::max))
    .fold(0.0f32, f32::max);
self.peak_meter.store(peak.to_bits(), Ordering::Relaxed);

// In editor - only compute when UI is visible:
if self.params.editor_state.is_open() {
    // Compute expensive visualizations
}
```

## Layout Patterns

```rust
// Horizontal layout
ui.horizontal(|ui| {
    ui.label("Cutoff:");
    ui.add(widgets::ParamSlider::for_param(&params.cutoff, setter));
});

// Grouped parameters
ui.group(|ui| {
    ui.label("Filter");
    ui.add(widgets::ParamSlider::for_param(&params.cutoff, setter));
    ui.add(widgets::ParamSlider::for_param(&params.resonance, setter));
});

// Sections with separators
ui.separator();
ui.heading("Modulation");

// Vertical centering
ui.vertical_centered(|ui| {
    ui.heading("My Plugin");
});

// Custom spacing
ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);
```

## Custom Parameter Controls

For more control than `ParamSlider`:

```rust
// Get current normalized value (0.0 to 1.0)
let mut value = params.cutoff.unmodulated_normalized_value();

let response = ui.add(egui::Slider::new(&mut value, 0.0..=1.0).text("Cutoff"));

if response.drag_started() {
    setter.begin_set_parameter(&params.cutoff);
}
if response.changed() {
    setter.set_parameter_normalized(&params.cutoff, value);
}
if response.drag_stopped() {
    setter.end_set_parameter(&params.cutoff);
}
```

## Common Pitfalls to Avoid

| Pitfall | Problem | Solution |
|---------|---------|----------|
| Missing `editor_state` | Window size not saved | Add `#[persist = "editor-state"]` field |
| Heavy computation in UI | UI lag, audio glitches | Use `is_open()` check, pre-compute values |
| Not using `ParamSlider` | Missing begin/end calls | Use built-in widget or call manually |
| Forgetting UI for new param | Parameter not controllable | Always add widget in `editor()` |

## Feature Completion Checklist

Before saying a feature is "done", verify ALL boxes are checked:

- [ ] Parameter added to `Params` struct with `#[id = "..."]`
- [ ] DSP code uses the parameter via `.smoothed.next()` or `.value()`
- [ ] **UI WIDGET EXISTS in editor()** - ParamSlider or custom control
- [ ] Widget is wired up to the parameter via setter
"#;

/// Native UI skill - no custom UI, DAW controls only
pub const NATIVE_UI: &str = r#"---
name: native-ui
description: Native plugin patterns (no custom GUI). DAW generic interface, automation, parameter naming. Invoke when working on native/no-UI plugins.
---

# Native Plugin (No Custom UI)

This plugin has no custom graphical interface. Users interact through:
- DAW's generic parameter interface
- Automation lanes
- MIDI CC mapping (if enabled)

## Best Practices for Native Plugins

1. **Clear parameter names**: Users only see the name in their DAW
2. **Sensible ranges**: Make defaults useful, ranges intuitive
3. **Good presets**: Consider adding factory presets for common use cases
4. **Proper units**: Use `formatters::v2s_f32_hz_then_khz` etc. for display

## Parameter Naming Guidelines

Since users can't see a custom UI, parameter names are critical:

```rust
// Good - clear, descriptive names
FloatParam::new("Filter Cutoff", ...)
FloatParam::new("Attack Time", ...)
FloatParam::new("Output Gain", ...)

// Bad - unclear names
FloatParam::new("Param1", ...)
FloatParam::new("Freq", ...)
FloatParam::new("Amt", ...)
```

## Using Formatters for Display

```rust
use nih_plug::prelude::formatters;

FloatParam::new("Cutoff", 1000.0, FloatRange::Skewed { ... })
    .with_value_to_string(formatters::v2s_f32_hz_then_khz(0))
    .with_string_to_value(formatters::s2v_f32_hz_then_khz())
    .with_unit("")  // Formatter already includes Hz/kHz
```

## Consider Adding

- Preset system for quick configuration
- Parameter groups for organization in DAW

## Feature Completion Checklist (Native)

- [ ] Parameter added to `Params` struct with `#[id = "..."]`
- [ ] DSP code uses the parameter via `.smoothed.next()` or `.value()`
- [ ] Parameter has clear, descriptive name
- [ ] Parameter has appropriate unit and formatter
- [ ] Default value is sensible
"#;
