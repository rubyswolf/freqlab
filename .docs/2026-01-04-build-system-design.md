# Phase 3: Build System Design

## Overview

Add build functionality to freqlab - compile nih-plug projects into VST3/CLAP plugins with fast incremental builds via a shared Cargo workspace.

## Architecture

### Shared Cargo Workspace

All projects share a single workspace for fast builds:

```
~/VSTWorkshop/
  Cargo.toml          # Workspace root
  target/             # Shared build artifacts (compiled deps)
  projects/
    my-plugin/
      Cargo.toml      # Workspace member
      src/lib.rs
    another-plugin/
      Cargo.toml
      src/lib.rs
  output/             # Built .vst3/.clap files
```

Workspace root `Cargo.toml`:
```toml
[workspace]
members = ["projects/*"]
resolver = "2"
```

Benefits:
- First build compiles nih-plug deps (~5-15 min)
- Subsequent builds reuse cached deps (seconds)
- Glob pattern auto-includes new projects

## UI Design

### Build Button

Located in header bar next to "Open Folder":

```
┌─────────────────────────────────────────────────────────┐
│  ← MyPlugin                    [Open Folder] [Build]    │
└─────────────────────────────────────────────────────────┘
```

States:
- Default: "Build" with icon
- Building: Spinner + "Building..." (disabled)
- No project: Disabled

### Build Flow

1. User clicks "Build"
2. Output panel auto-expands, streams build output
3. On success:
   - Copy artifacts to `~/VSTWorkshop/output/`
   - Green toast: "Build successful!" with "Open Output" button
4. On failure:
   - Red toast: "Build failed" with "Fix with Claude" button
   - Button sends error to chat

### Toast Notifications

- Position: Bottom-right, stacked vertically
- Types: success (green), error (red), info (blue)
- Success: Auto-dismiss after 5s
- Error: Persist until dismissed
- Optional action buttons

## Backend Implementation

### New File: `src-tauri/src/commands/build.rs`

```rust
#[derive(Serialize, Clone)]
pub struct BuildResult {
    pub success: bool,
    pub output_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum BuildStreamEvent {
    Start,
    Output { line: String },
    Done { success: bool, output_path: Option<String> },
    Error { message: String },
}

#[tauri::command]
pub async fn build_project(
    project_name: String,
    window: tauri::Window,
) -> Result<BuildResult, String>
```

### Build Process

1. Run from workspace root (`~/VSTWorkshop/`):
   ```bash
   cargo xtask bundle {project_name} --release
   ```
2. Stream stdout/stderr via "build-stream" events
3. On success, copy artifacts:
   - `target/bundled/{name}.vst3` → `output/{name}.vst3`
   - `target/bundled/{name}.clap` → `output/{name}.clap`
4. Return BuildResult

### Workspace Initialization

Modify `create_project` in `projects.rs`:
- Check if `~/VSTWorkshop/Cargo.toml` exists
- If not, create workspace root config
- Projects auto-included via glob pattern

## Frontend Implementation

### New Files

| File | Purpose |
|------|---------|
| `src/components/Common/Toast.tsx` | Toast notification component |
| `src/stores/toastStore.ts` | Zustand store for toasts |
| `src/hooks/useBuild.ts` | Build command + streaming hook |

### MainLayout Changes

- Add "Build" button in header
- Wire to `useBuild` hook
- Listen for build-stream events → outputStore
- Handle toast on completion

## Future Enhancements (Phase 5)

- Configurable output folder in settings
- DAW plugin folder auto-install option
