# freqlab

A Tauri 2.x desktop app for creating VST audio plugins with AI assistance.

## Tech Stack

- **Frontend**: React 18 + TypeScript + Tailwind CSS + Zustand
- **Backend**: Tauri 2.x (Rust)
- **AI**: Claude Code CLI integration
- **Audio**: nih-plug (Rust VST/CLAP framework)

## Project Structure

```
src/                     # React frontend
  components/
    Chat/               # ChatPanel, ChatMessage, ChatInput
    Common/             # Button, Modal, Spinner
    Layout/             # Header, Sidebar, OutputPanel, MainLayout
    Projects/           # ProjectList, ProjectCard, NewProjectModal
    Setup/              # WelcomeWizard, PrerequisitesCheck
  stores/               # Zustand stores (settings, project, output)
  types/                # TypeScript interfaces

src-tauri/              # Rust backend
  src/
    commands/
      prerequisites.rs  # System requirement checks
      projects.rs       # Project CRUD operations
      claude.rs         # Claude Code CLI integration
    lib.rs              # Tauri app setup
```

## Key Features

1. **Prerequisites Check**: Verifies Xcode CLI, Rust, Claude CLI are installed
2. **Project Management**: Create/list/delete VST plugin projects
3. **Claude Integration**: Chat with Claude to build/modify plugins
4. **Output Panel**: Streams Claude's work in real-time

## How It Works

1. User creates a new plugin (name + description)
2. App generates nih-plug project skeleton at `~/VSTWorkshop/projects/{name}/`
3. User chats with Claude to describe features
4. Claude modifies `src/lib.rs` directly
5. User builds with `cargo xtask bundle` (Phase 4)

## Claude CLI Integration

Uses non-interactive mode with streaming:
```bash
claude -p "message" \
  --output-format stream-json \
  --allowedTools "Edit,Write,Read" \
  --append-system-prompt "..." \
  --max-turns 15
```

## Implementation Phases

### Phase 1: Foundation ✅
- Tauri + React + TypeScript scaffold
- Tailwind CSS with dark theme
- Prerequisites check system
- Welcome wizard flow

### Phase 2: Project Management + Claude Integration ✅
- Project creation with nih-plug templates
- Project list in sidebar
- Claude Code CLI integration
- Chat interface with streaming output

### Phase 3: Build System ✅
- Shared Cargo workspace at `~/VSTWorkshop/` for fast incremental builds
- `cargo xtask bundle` execution from workspace root
- Build output streaming to output panel
- Toast notifications for success/failure
- "Fix with Claude" button sends build errors to chat
- Copy artifacts to `~/VSTWorkshop/output/`

### Phase 4: Version Control
- Git integration for projects
- Version bump modal
- Changelog generation from chat history
- **Persistent chat history storage**

### Phase 5: Polish
- DAW setup guides
- Settings panel
- Keyboard shortcuts
- Error handling improvements

## Useful Commands

```bash
# Development
npm run tauri dev

# Build
npm run tauri build

# Check Rust code
cd src-tauri && cargo check
```

## File Locations

- Projects: `~/VSTWorkshop/projects/{name}/`
- Built plugins: `~/VSTWorkshop/output/`
- App config: Zustand persisted to localStorage
