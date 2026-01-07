# freqlab

Describe it. Build it. Hear it.

A macOS app for creating VST3/CLAP audio plugins through conversation.

> **Note**: This is a personal side project and is not consistently maintained. Use as-is.

## About

freqlab is a desktop application that combines the [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) with [nih-plug](https://github.com/robbert-vdh/nih-plug) to let you build audio plugins through conversation. Instead of writing boilerplate Rust code, describe your plugin idea and iterate through chat.

**How it works:**
1. Create a new plugin project (effect or instrument)
2. Chat with Claude to describe what you want
3. Claude modifies the code, freqlab builds it
4. Preview the result in real-time with the built-in audio engine
5. Publish to your DAW

## Features

**AI-Powered Development**
- Natural language plugin creation via Claude Code CLI
- Streaming responses with real-time feedback
- Automatic git commits after each change
- Revert to any previous version with one click
- File attachments for context

**Plugin Templates**
- Effect or Instrument starting points
- WebView, egui, or Headless UI options
- All templates work cross-platform

**Audio Preview**
- Real-time CLAP plugin hosting
- Hot reload on code changes
- Test signals: sine, noise, sweep, impulse, chirp
- Sample file playback (WAV, MP3, AAC)
- Level metering

**Build & Publish**
- One-click builds with streaming output
- Versioned output folders
- Copy plugins directly to DAW folders
- Export/import projects as ZIP

## Prerequisites

- **macOS 12+** (Monterey or later) - the app currently runs on macOS only
- **Xcode Command Line Tools** - `xcode-select --install`
- **Rust** - via [rustup](https://rustup.rs/)
- **Claude Code CLI** - with an active Anthropic subscription

freqlab will check these requirements on first launch.

## Installation

```bash
# Clone the repository
git clone https://github.com/jamesontucker/freqlab.git
cd freqlab

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Or build a release
npm run tauri build
```

## Quick Start

1. **Launch freqlab** and complete the prerequisites check
2. **Create a new project** - give it a name and choose a template
3. **Start chatting** - describe what you want your plugin to do
4. **Build** - click the build button to compile your plugin
5. **Preview** - use the audio preview panel to hear your plugin in action
6. **Iterate** - keep chatting to refine your plugin
7. **Publish** - copy the built plugin to your DAW's plugin folder

## Known Issues

### macOS Only (Currently)

freqlab currently runs on macOS only. Cross-platform support may come in the future.

### Unsigned Plugins & Gatekeeper

Plugins built with freqlab are unsigned. Your DAW may block them or show security warnings.

**To remove the quarantine flag:**
```bash
xattr -cr /path/to/YourPlugin.clap
xattr -cr /path/to/YourPlugin.vst3
```

This is a limitation of unsigned code distribution, not specific to nih-plug.

### AI-Generated Code

Claude generates the plugin code. While it follows safety practices (like output limiting), always review generated code before distributing plugins. freqlab is provided as-is without warranty.

## License

freqlab is licensed under **GPL-3.0**. See [LICENSE](LICENSE) for details.

### Plugin Licensing

Plugins created with freqlab use [nih-plug](https://github.com/robbert-vdh/nih-plug):

- The **nih-plug framework** is licensed under the permissive **ISC license**
- The **VST3 bindings** are licensed under **GPL-3.0**

This means:
- **VST3 plugins** must comply with GPL-3.0 (source must be available on request)
- **CLAP-only plugins** are not subject to this requirement
- You **can sell** your plugins, but must provide source code if requested

## Contributing

Contributions are welcome, but keep in mind this is a side project with irregular maintenance. Feel free to open issues or PRs on [GitHub](https://github.com/jamesontucker/freqlab).

---

Built with [Tauri](https://tauri.app/), [nih-plug](https://github.com/robbert-vdh/nih-plug), and [Claude Code](https://docs.anthropic.com/en/docs/claude-code).
