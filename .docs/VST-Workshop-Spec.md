# VST Workshop — Technical Specification

## Overview

VST Workshop is a desktop application for macOS that enables users to create, iterate on, and manage VST audio plugins using AI assistance. The app integrates with Claude Code CLI to provide a conversational interface for plugin development.

**Core Value Proposition:** Build VST plugins by describing what you want in natural language, iterate quickly with AI assistance, and manage versions — all without leaving the app.

---

## Tech Stack

| Layer | Technology | Rationale |
|-------|------------|-----------|
| Framework | Tauri 2.x | Lightweight, Rust backend pairs with nih-plug |
| Frontend | React 18 + TypeScript | Component-based, good ecosystem |
| Styling | Tailwind CSS | Rapid UI development, consistent design |
| State Management | Zustand | Simple, minimal boilerplate |
| VST Framework | nih-plug (Rust) | Fast compiles, modern, ISC + GPLv3 license |
| Versioning | Git (via CLI) | Built into each project |
| AI Integration | Claude Code CLI | Uses existing subscription, no API key needed |

---

## Directory Structure

### Application Data

```
~/VSTWorkshop/
├── config.json                    # App settings
├── projects/
│   └── {plugin-name}/
│       ├── .git/                  # Version control
│       ├── src/
│       │   └── lib.rs             # Main plugin code
│       ├── Cargo.toml
│       ├── CHANGELOG.md
│       ├── README.md
│       ├── .vstworkshop/
│       │   ├── metadata.json      # Plugin metadata (name, description, icon, etc.)
│       │   └── conversations.json # Chat history with Claude
│       └── xtask/                 # Build helper (nih-plug convention)
└── output/                        # Compiled plugins (DAW scans this)
    ├── PluginName.vst3
    └── PluginName.clap
```

### Tauri App Structure

```
vst-workshop/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs                # Tauri entry point
│   │   ├── lib.rs                 # Command handlers
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── projects.rs        # Project CRUD operations
│   │   │   ├── claude.rs          # Claude Code integration
│   │   │   ├── build.rs           # Build system
│   │   │   ├── git.rs             # Version control
│   │   │   └── prerequisites.rs   # System checks
│   │   ├── templates/
│   │   │   ├── mod.rs
│   │   │   ├── gain.rs            # Gain plugin template
│   │   │   ├── reverb.rs          # Reverb template
│   │   │   ├── distortion.rs      # Distortion template
│   │   │   └── synth.rs           # Synth template
│   │   └── utils/
│   │       ├── mod.rs
│   │       └── paths.rs           # Path helpers
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/
│   ├── App.tsx
│   ├── main.tsx
│   ├── components/
│   │   ├── Layout/
│   │   │   ├── Sidebar.tsx
│   │   │   ├── Header.tsx
│   │   │   └── OutputPanel.tsx
│   │   ├── Projects/
│   │   │   ├── ProjectList.tsx
│   │   │   ├── ProjectCard.tsx
│   │   │   ├── NewProjectModal.tsx
│   │   │   └── TemplateSelector.tsx
│   │   ├── Chat/
│   │   │   ├── ChatPanel.tsx
│   │   │   ├── ChatMessage.tsx
│   │   │   └── ChatInput.tsx
│   │   ├── Build/
│   │   │   ├── BuildButton.tsx
│   │   │   ├── BuildOutput.tsx
│   │   │   └── BuildStatus.tsx
│   │   ├── Version/
│   │   │   ├── VersionBump.tsx
│   │   │   ├── Changelog.tsx
│   │   │   └── VersionHistory.tsx
│   │   ├── Setup/
│   │   │   ├── PrerequisitesCheck.tsx
│   │   │   ├── DawGuide.tsx
│   │   │   └── WelcomeWizard.tsx
│   │   └── Common/
│   │       ├── Button.tsx
│   │       ├── Modal.tsx
│   │       ├── Input.tsx
│   │       └── Spinner.tsx
│   ├── stores/
│   │   ├── projectStore.ts
│   │   ├── chatStore.ts
│   │   ├── buildStore.ts
│   │   └── settingsStore.ts
│   ├── hooks/
│   │   ├── useProjects.ts
│   │   ├── useClaude.ts
│   │   ├── useBuild.ts
│   │   └── usePrerequisites.ts
│   ├── types/
│   │   └── index.ts
│   └── lib/
│       ├── tauri.ts               # Tauri invoke wrappers
│       └── utils.ts
├── package.json
├── tsconfig.json
├── tailwind.config.js
└── vite.config.ts
```

---

## Data Models

### Project Metadata (`metadata.json`)

```json
{
  "id": "uuid-v4",
  "name": "RetroVerb",
  "description": "A warm vintage reverb with shimmer effect",
  "version": "0.3.0",
  "author": "Your Name",
  "category": "reverb",
  "createdAt": "2026-01-02T10:30:00Z",
  "updatedAt": "2026-01-04T15:45:00Z",
  "icon": "🎛️",
  "tags": ["reverb", "vintage", "shimmer"],
  "buildFormats": ["vst3", "clap"]
}
```

### Conversation History (`conversations.json`)

```json
{
  "messages": [
    {
      "id": "uuid-v4",
      "role": "user",
      "content": "Add a wet/dry mix knob",
      "timestamp": "2026-01-04T15:30:00Z"
    },
    {
      "id": "uuid-v4",
      "role": "assistant",
      "content": "I'll add a wet/dry mix parameter...",
      "timestamp": "2026-01-04T15:30:05Z",
      "filesModified": ["src/lib.rs"],
      "summary": "Added mix parameter (0-100%)"
    }
  ]
}
```

### App Config (`config.json`)

```json
{
  "workspacePath": "~/VSTWorkshop",
  "outputPath": "~/VSTWorkshop/output",
  "buildFormats": ["vst3", "clap"],
  "autoOpenOutput": true,
  "showNotifications": true,
  "theme": "dark",
  "setupComplete": true
}
```

---

## Feature Specifications

### 1. Prerequisites Check

**Purpose:** Ensure the user's system is ready for VST development.

**Checks Required:**
| Prerequisite | Check Command | Install Command |
|--------------|---------------|-----------------|
| macOS 12+ | `sw_vers -productVersion` | N/A (manual) |
| Xcode CLI Tools | `xcode-select -p` | `xcode-select --install` |
| Rust & Cargo | `rustc --version` | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Claude Code CLI | `claude --version` | `npm install -g @anthropic-ai/claude-code` |
| Claude Auth | `claude auth status` | `claude login` (opens browser) |

**UI States:**
- ✅ Installed (green check)
- ❌ Not installed (red X with Install button)
- ⚠️ Needs configuration (yellow warning)
- ⏳ Installing... (spinner)

**Flow:**
1. On first launch, show prerequisites screen
2. Auto-detect installed items
3. Provide one-click install where possible
4. Show manual instructions when auto-install isn't possible
5. Store `setupComplete: true` in config once all green

### 2. Project Management

**Create New Project:**
1. User clicks "+ New Plugin"
2. Modal appears with options:
   - **From description:** Text area to describe the plugin
   - **From template:** Grid of template cards (Gain, Reverb, Distortion, Synth)
3. User provides:
   - Name (validated: lowercase, no spaces, becomes folder name)
   - Display Name (shown in DAW)
   - Description
   - Category (effect, instrument, analyzer)
4. On submit:
   - Create project folder
   - Initialize git repo
   - Generate nih-plug project files
   - If from description, invoke Claude Code to generate initial code
   - If from template, copy template files

**Project Templates:**

Each template is a minimal working plugin:

```rust
// Gain template (simplest)
// - Input gain parameter (-24 to +24 dB)
// - Output gain parameter (-24 to +24 dB)

// Reverb template
// - Room size parameter
// - Decay time parameter
// - Wet/dry mix parameter
// - Simple delay-based reverb algorithm

// Distortion template
// - Drive parameter
// - Tone parameter
// - Output level parameter
// - Waveshaping algorithm

// Synth template
// - Basic oscillator (sine, saw, square)
// - ADSR envelope
// - Cutoff filter
// - MIDI input handling
```

**Project List:**
- Show all projects from `~/VSTWorkshop/projects/`
- Display: icon, name, version, last modified
- Sort by: last modified (default), name, date created
- Actions: Open, Duplicate, Archive, Delete

### 3. Claude Code Integration

**How It Works:**

The app spawns Claude Code CLI as a subprocess and streams output.

```rust
// Tauri command (Rust side)
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{BufReader, AsyncBufReadExt};

#[tauri::command]
async fn send_to_claude(
    project_path: String,
    message: String,
    window: tauri::Window
) -> Result<String, String> {
    let mut child = Command::new("claude")
        .current_dir(&project_path)
        .arg(&message)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);
    let mut lines = reader.lines();

    let mut full_output = String::new();

    while let Some(line) = lines.next_line().await.map_err(|e| e.to_string())? {
        full_output.push_str(&line);
        full_output.push('\n');
        // Emit real-time updates to frontend
        window.emit("claude-output", &line).unwrap();
    }

    Ok(full_output)
}
```

```typescript
// Frontend (React side)
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const sendMessage = async (message: string) => {
  // Listen for streaming output
  const unlisten = await listen('claude-output', (event) => {
    appendToChat(event.payload as string);
  });

  try {
    const result = await invoke('send_to_claude', {
      projectPath: currentProject.path,
      message: message
    });
    return result;
  } finally {
    unlisten();
  }
};
```

**Context Management:**

Before sending to Claude, the app prepends context:

```
You are helping develop a VST audio plugin using nih-plug (Rust).

Project: {name}
Description: {description}
Current Version: {version}

The user will ask you to modify the plugin. Make changes directly to the source files.
After making changes, briefly summarize what you did.

Focus on:
- Clean, efficient DSP code
- Proper parameter handling
- Good UI/UX for the plugin interface
```

**Conversation Persistence:**
- Save all messages to `conversations.json`
- Load history when opening project
- "Clear conversation" button (keeps files, clears chat)

### 4. Build System

**Build Command:**
```bash
cd {project_path}
cargo xtask bundle {plugin_name} --release
```

**Build Output Parsing:**
- Stream stdout/stderr to Output Panel
- Detect success: "Finished release" or ".vst3" appears
- Detect failure: "error[E" pattern
- On success: Copy artifacts to output folder

**Post-Build Actions:**
```bash
# Copy VST3
cp -r target/bundled/{PluginName}.vst3 ~/VSTWorkshop/output/

# Copy CLAP (if enabled)
cp -r target/bundled/{PluginName}.clap ~/VSTWorkshop/output/
```

**Build States:**
- Idle: "Ready to build"
- Building: Spinner + streaming output
- Success: Green check + "Build successful"
- Failed: Red X + "Build failed" + "Ask Claude to fix?" button

**"Fix with Claude" Flow:**
1. Extract error message from build output
2. Auto-send to Claude: "The build failed with this error: {error}. Please fix it."
3. Claude makes fixes
4. User can rebuild

### 5. Version Management

**Semantic Versioning:**
- MAJOR.MINOR.PATCH (e.g., 0.3.1)
- Stored in `Cargo.toml` and `metadata.json`

**Version Bump Flow:**
1. User clicks "Bump Version"
2. Modal shows:
   - Current version
   - Radio buttons: Patch / Minor / Major
   - Auto-generated changelog based on conversation since last version
   - Editable release notes textarea
3. On confirm:
   ```bash
   # Update Cargo.toml version
   # Update metadata.json version
   # Prepend to CHANGELOG.md
   git add -A
   git commit -m "Release v{new_version}"
   git tag v{new_version}
   ```

**Changelog Format (CHANGELOG.md):**
```markdown
# Changelog

## [0.3.0] - 2026-01-04

### Added
- High-pass filter with cutoff control (20-2000Hz)
- Freeze button for infinite reverb tails

### Fixed
- Audio click when toggling freeze

## [0.2.0] - 2026-01-03

### Added
- Shimmer effect
- Room size control
```

**Version History View:**
- List all git tags
- Show date and changelog entry for each
- "Restore" button: `git checkout v{version}` (with confirmation)
- "Compare" button: Show diff between versions

### 6. DAW Setup Guides

**Content for Each DAW:**

```markdown
# Ableton Live Setup

1. Open Ableton Live
2. Go to **Preferences** → **Plug-ins**
3. Under "VST3 Custom Folder", click **Browse**
4. Navigate to: `~/VSTWorkshop/output`
5. Enable **"Use VST3 Custom Folder"**
6. Click **"Rescan"**

Your plugins will appear under "Plug-ins" in the browser.

## Reloading After Changes

After building a new version:
1. Remove the plugin from your track
2. Re-add it from the browser

The updated version will load automatically.
```

```markdown
# Reaper Setup

1. Open Reaper
2. Go to **Preferences** → **Plug-ins** → **VST**
3. Add path: `~/VSTWorkshop/output`
4. Click **"Re-scan"**

## Reloading After Changes

After building a new version:
1. Right-click the plugin → **"Reload"**
   (Reaper supports hot-reloading!)
```

```markdown
# FL Studio Setup

1. Open FL Studio
2. Go to **Options** → **Manage plugins**
3. Click **"Add path"** under VST3
4. Add: `~/VSTWorkshop/output`
5. Click **"Start scan"**
```

---

## UI Specifications

### Theme

**Dark Mode (Default):**
```css
--bg-primary: #1a1a1a;
--bg-secondary: #242424;
--bg-tertiary: #2e2e2e;
--text-primary: #ffffff;
--text-secondary: #a0a0a0;
--accent: #6366f1;        /* Indigo */
--accent-hover: #818cf8;
--success: #22c55e;
--warning: #f59e0b;
--error: #ef4444;
--border: #3a3a3a;
```

### Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  Header (48px)                                                  │
│  App title / Current project name                    Settings   │
├──────────────┬──────────────────────────────────────────────────┤
│              │                                                  │
│  Sidebar     │  Main Content Area                               │
│  (240px)     │                                                  │
│              │  - Chat Panel (primary)                          │
│  - Projects  │  - Or Welcome/Setup screens                      │
│  - Setup     │                                                  │
│  - Help      │                                                  │
│              │                                                  │
│              ├──────────────────────────────────────────────────┤
│              │  Action Bar                                      │
│              │  [Build] [Bump Version] [Changelog] [Files]      │
├──────────────┴──────────────────────────────────────────────────┤
│  Output Panel (collapsible, 150px default)                      │
│  Build output, logs, errors                                     │
└─────────────────────────────────────────────────────────────────┘
```

### Key Components

**Project Card:**
```
┌──────────────────────────────┐
│ 🎛️ RetroVerb                 │
│ v0.3.0 • Modified 5 min ago  │
│ ●  (green dot = clean)       │
│ ◐  (half = uncommitted)      │
└──────────────────────────────┘
```

**Chat Message:**
```
┌──────────────────────────────────────────┐
│ You                           10:30 AM   │
│ Add a high-pass filter before the reverb │
└──────────────────────────────────────────┘

┌──────────────────────────────────────────┐
│ Claude                        10:30 AM   │
│ I'll add a high-pass filter with a       │
│ cutoff frequency parameter.              │
│                                          │
│ 📄 Modified: src/lib.rs                  │
│   • Added highpass_cutoff parameter      │
│   • Inserted filter before reverb        │
│                                          │
│ Ready to build when you are.             │
└──────────────────────────────────────────┘
```

**Build Button States:**
```
[ 🔨 Build ]           - Default (idle)
[ ⏳ Building... ]     - In progress (disabled)
[ ✅ Built ]           - Success (shows for 3s, then resets)
[ ❌ Failed ]          - Error (red, shows "Fix?" option)
```

---

## nih-plug Project Template

### Cargo.toml

```toml
[package]
name = "{plugin_name_snake_case}"
version = "0.1.0"
edition = "2021"
authors = ["{author}"]
license = "GPL-3.0-only"
description = "{description}"

[lib]
crate-type = ["cdylib"]

[dependencies]
nih_plug = { git = "https://github.com/robbert-vdh/nih-plug.git", features = ["assert_process_allocs"] }

[profile.release]
lto = "thin"
strip = "symbols"

[workspace]
members = ["xtask"]
```

### src/lib.rs (Gain Template)

```rust
use nih_plug::prelude::*;
use std::sync::Arc;

struct {PluginName} {
    params: Arc<{PluginName}Params>,
}

#[derive(Params)]
struct {PluginName}Params {
    #[id = "gain"]
    pub gain: FloatParam,
}

impl Default for {PluginName} {
    fn default() -> Self {
        Self {
            params: Arc::new({PluginName}Params::default()),
        }
    }
}

impl Default for {PluginName}Params {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
        }
    }
}

impl Plugin for {PluginName} {
    const NAME: &'static str = "{Display Name}";
    const VENDOR: &'static str = "{author}";
    const URL: &'static str = "";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
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
        for channel_samples in buffer.iter_samples() {
            let gain = self.params.gain.smoothed.next();
            for sample in channel_samples {
                *sample *= gain;
            }
        }
        ProcessStatus::Normal
    }
}

impl ClapPlugin for {PluginName} {
    const CLAP_ID: &'static str = "com.vstworkshop.{plugin_name_snake_case}";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("{description}");
    const CLAP_MANUAL_URL: Option<&'static str> = None;
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for {PluginName} {
    const VST3_CLASS_ID: [u8; 16] = *b"{16_byte_unique}";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Fx];
}

nih_export_clap!({PluginName});
nih_export_vst3!({PluginName});
```

### xtask/src/main.rs

```rust
fn main() -> nih_plug_xtask::Result<()> {
    nih_plug_xtask::main()
}
```

### xtask/Cargo.toml

```toml
[package]
name = "xtask"
version = "0.1.0"
edition = "2021"

[dependencies]
nih_plug_xtask = { git = "https://github.com/robbert-vdh/nih-plug.git" }
```

---

## Implementation Phases

### Phase 1: Foundation
- [ ] Initialize Tauri + React + TypeScript project
- [ ] Set up Tailwind CSS with dark theme
- [ ] Create basic layout (sidebar, main area, output panel)
- [ ] Implement prerequisites check system
- [ ] Create welcome/setup wizard flow

### Phase 2: Project Management
- [ ] Implement project creation (from template)
- [ ] Create nih-plug project templates (gain, reverb, distortion, synth)
- [ ] Build project list component
- [ ] Implement project metadata storage
- [ ] Add duplicate/archive/delete functionality

### Phase 3: Claude Integration
- [ ] Implement Claude Code CLI subprocess spawning
- [ ] Add real-time output streaming to frontend
- [ ] Build chat interface (messages, input)
- [ ] Implement conversation persistence
- [ ] Add context injection for plugin development

### Phase 4: Build System
- [ ] Implement cargo xtask bundle command execution
- [ ] Stream build output to panel
- [ ] Parse build results (success/failure)
- [ ] Auto-copy to output folder
- [ ] Add "Fix with Claude" for build errors

### Phase 5: Version Control
- [ ] Implement git init for new projects
- [ ] Add version bump modal
- [ ] Auto-generate changelog from conversation
- [ ] Create git commits and tags
- [ ] Build version history view
- [ ] Implement restore to previous version

### Phase 6: Polish
- [ ] Add DAW setup guides
- [ ] Implement settings panel
- [ ] Add keyboard shortcuts
- [ ] Error handling and edge cases
- [ ] Testing and bug fixes
- [ ] App icon and branding

---

## Commands Reference

### Tauri Commands (Rust → JS)

```rust
// Prerequisites
#[tauri::command] fn check_prerequisites() -> PrerequisiteStatus;
#[tauri::command] fn install_prerequisite(name: String) -> Result<(), String>;

// Projects
#[tauri::command] fn list_projects() -> Vec<ProjectMeta>;
#[tauri::command] fn create_project(name: String, template: String, description: String) -> Result<ProjectMeta, String>;
#[tauri::command] fn delete_project(id: String) -> Result<(), String>;
#[tauri::command] fn duplicate_project(id: String, new_name: String) -> Result<ProjectMeta, String>;

// Claude
#[tauri::command] fn send_to_claude(project_path: String, message: String) -> Result<String, String>;
#[tauri::command] fn get_conversation(project_id: String) -> Vec<ChatMessage>;
#[tauri::command] fn clear_conversation(project_id: String) -> Result<(), String>;

// Build
#[tauri::command] fn build_project(project_path: String) -> Result<BuildResult, String>;
#[tauri::command] fn cancel_build() -> Result<(), String>;

// Git
#[tauri::command] fn bump_version(project_path: String, bump_type: String, notes: String) -> Result<String, String>;
#[tauri::command] fn get_version_history(project_path: String) -> Vec<VersionEntry>;
#[tauri::command] fn restore_version(project_path: String, version: String) -> Result<(), String>;

// Config
#[tauri::command] fn get_config() -> AppConfig;
#[tauri::command] fn set_config(config: AppConfig) -> Result<(), String>;
```

### Events (Rust → JS)

```rust
// Real-time streaming
window.emit("claude-output", line: String);
window.emit("build-output", line: String);
window.emit("build-complete", result: BuildResult);
```

---

## Success Criteria

The app is complete when a user can:

1. ✅ Launch app and complete prerequisite setup
2. ✅ Create a new plugin from description or template
3. ✅ Have a conversation with Claude to modify the plugin
4. ✅ Build the plugin with one click
5. ✅ See build errors and ask Claude to fix them
6. ✅ Load the plugin in their DAW
7. ✅ Bump version and see changelog
8. ✅ Manage multiple plugins
9. ✅ Restore a previous version if needed

---

## Notes for Claude Code

When building this app:

1. **Start with Phase 1** - Get the Tauri + React foundation working first
2. **Test incrementally** - Build and test each feature before moving on
3. **Use Tauri 2.x** - Latest version, check docs at tauri.app
4. **Handle errors gracefully** - Every Tauri command should have proper error handling
5. **Keep UI responsive** - Use streaming for Claude and build output
6. **Follow the directory structure** - Consistency matters for maintainability

For nih-plug specifics, refer to:
- https://github.com/robbert-vdh/nih-plug
- Examples in the nih-plug repo's `plugins/examples/` folder
