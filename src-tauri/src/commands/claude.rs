use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Mutex;
use std::time::Duration;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time::timeout;

use super::agent_provider::{AgentProvider, AgentProviderType, AgentStreamEvent, ProviderCallConfig};
use super::providers::claude_cli::ClaudeProvider;
use super::providers::opencode::OpenCodeProvider;

// Track active agent processes by project path so we can interrupt them
// Key format: "project_path" (we interrupt all providers for a project when requested)
// Value: (provider_type, pid) to know which provider is running
static ACTIVE_PROCESSES: Mutex<Option<HashMap<String, (AgentProviderType, u32)>>> = Mutex::new(None);

fn register_process(project_path: &str, provider: AgentProviderType, pid: u32) {
    let mut guard = ACTIVE_PROCESSES.lock().unwrap();
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }
    if let Some(ref mut map) = *guard {
        map.insert(project_path.to_string(), (provider, pid));
    }
}

fn unregister_process(project_path: &str) {
    let mut guard = ACTIVE_PROCESSES.lock().unwrap();
    if let Some(ref mut map) = *guard {
        map.remove(project_path);
    }
}

fn get_process_info(project_path: &str) -> Option<(AgentProviderType, u32)> {
    let guard = ACTIVE_PROCESSES.lock().unwrap();
    guard.as_ref().and_then(|map| map.get(project_path).copied())
}

fn get_process_pid(project_path: &str) -> Option<u32> {
    get_process_info(project_path).map(|(_, pid)| pid)
}

/// Response from Claude - kept for backwards compatibility with frontend
#[derive(Serialize, Clone)]
pub struct ClaudeResponse {
    pub content: String,
    pub session_id: Option<String>,
    pub commit_hash: Option<String>,
}

/// Get the session file path for a project (default: Claude)
fn get_session_file(project_path: &str) -> PathBuf {
    get_session_file_for_provider(project_path, &AgentProviderType::Claude)
}

/// Get the session file path for a specific provider
fn get_session_file_for_provider(project_path: &str, provider: &AgentProviderType) -> PathBuf {
    let filename = match provider {
        AgentProviderType::Claude => "claude_session.txt",
        AgentProviderType::OpenCode => "opencode_session.txt",
    };
    PathBuf::from(project_path)
        .join(".vstworkshop")
        .join(filename)
}

/// Load session ID for a project (if exists)
fn load_session_id(project_path: &str) -> Option<String> {
    load_session_id_for_provider(project_path, &AgentProviderType::Claude)
}

/// Load session ID for a specific provider
fn load_session_id_for_provider(project_path: &str, provider: &AgentProviderType) -> Option<String> {
    let session_file = get_session_file_for_provider(project_path, provider);
    fs::read_to_string(session_file)
        .ok()
        .map(|s| s.trim().to_string())
}

/// Save session ID for a project
fn save_session_id(project_path: &str, session_id: &str) -> Result<(), String> {
    save_session_id_for_provider(project_path, session_id, &AgentProviderType::Claude)
}

/// Save session ID for a specific provider
fn save_session_id_for_provider(project_path: &str, session_id: &str, provider: &AgentProviderType) -> Result<(), String> {
    let session_file = get_session_file_for_provider(project_path, provider);
    fs::write(&session_file, session_id)
        .map_err(|e| format!("Failed to save session ID: {}", e))
}

/// Get a boxed provider instance based on type
fn get_provider(provider_type: &AgentProviderType) -> Box<dyn AgentProvider> {
    match provider_type {
        AgentProviderType::Claude => Box::new(ClaudeProvider),
        AgentProviderType::OpenCode => Box::new(OpenCodeProvider),
    }
}

/// Get human-readable description for a component ID
fn get_component_description(component_id: &str) -> &'static str {
    match component_id {
        // Effect components
        "custom_gui" => "Custom GUI using nih_plug_vizia with knobs, sliders, and visual feedback",
        "preset_system" => "Preset save/load system with factory presets and user preset management",
        "param_smoothing" => "Advanced parameter smoothing with configurable interpolation curves",
        "sidechain_input" => "Sidechain audio input for ducking, keying, or modulation sources",
        "oversampling" => "2x/4x oversampling for reduced aliasing in nonlinear processing",
        // Instrument components
        "polyphony" => "Polyphonic voice architecture with 8 voices and voice stealing",
        "velocity_layers" => "Velocity-sensitive response with multiple sample/synthesis layers",
        "adsr_envelope" => "ADSR amplitude envelope with attack, decay, sustain, release controls",
        "lfo" => "LFO modulation source with multiple waveforms and tempo sync",
        _ => "Additional plugin feature",
    }
}

/// Local nih-plug API reference bundled at compile time (fallback)
const NIH_PLUG_REFERENCE: &str = include_str!("../../resources/nih-plug-reference.md");

/// Build the system context for Claude when working on a plugin project
fn build_context(
    project_name: &str,
    description: &str,
    project_path: &str,
    components: Option<&Vec<String>>,
    is_first_message: bool,
    ui_framework: Option<&str>,
) -> String {
    // Get path to local nih-plug repo for documentation
    let nih_plug_docs_path = super::projects::get_nih_plug_docs_path();
    let docs_path_str = nih_plug_docs_path.to_string_lossy();

    // Read project-specific CLAUDE.md if it exists
    let claude_md_path = PathBuf::from(project_path).join("CLAUDE.md");
    let claude_md_content = fs::read_to_string(&claude_md_path).unwrap_or_default();

    let mut context = String::new();

    // FIRST: Add starter components at the VERY TOP if this is first message
    if is_first_message {
        if let Some(comps) = components {
            if !comps.is_empty() {
                context.push_str("# ⚠️ STOP - READ THIS FIRST ⚠️\n\n");
                context.push_str("## YOU MUST IMPLEMENT THESE STARTER COMPONENTS\n\n");
                context.push_str("The user selected these components when creating the plugin. ");
                context.push_str("**You MUST implement ALL of them on the first feature request.**\n\n");

                for comp_id in comps {
                    let desc = get_component_description(comp_id);
                    context.push_str(&format!("- **{}**: {}\n", comp_id, desc));
                }

                context.push_str("\n### Implementation Order:\n");
                context.push_str("1. Implement ALL starter components above FIRST\n");
                context.push_str("2. THEN implement whatever feature the user requested\n");
                context.push_str("3. Invoke the relevant skill for each component (e.g., `/preset-system`, `/param-smoothing`)\n\n");

                context.push_str("### DO NOT SKIP THIS. The user paid for these features.\n\n");
                context.push_str("---\n\n");
            }
        }

        // Add UI requirement for webview plugins
        if ui_framework == Some("webview") {
            context.push_str("## YOU MUST UPDATE THE UI\n\n");
            context.push_str("This is a WebView plugin. When you add parameters/features:\n");
            context.push_str("1. Add the parameter in lib.rs\n");
            context.push_str("2. **ALSO add a UI control in ui.html** (slider, knob, button, etc.)\n");
            context.push_str("3. Connect the UI control via IPC messages\n\n");
            context
                .push_str("A plugin with no UI controls is BROKEN. Always update both lib.rs AND ui.html.\n\n");
            context.push_str("---\n\n");
        }
    }

    context.push_str(&format!(
        r#"You are helping develop a VST audio plugin using nih-plug (Rust).

Project: {project_name}
Description: {description}

# CRITICAL RULES - READ FIRST"#,
        project_name = project_name,
        description = description,
    ));

    context.push_str(
        r#"

## 1. NEVER SAY "BUILD"
The app has a Build button. Using "build" confuses users into thinking you're doing their build.
- ❌ "Let me build..." / "I'll build..." / "Building..." → ✅ "Let me implement..."
- ❌ "Build successful" / "Build complete" → ✅ "Done!" or "Ready for you to try"
- ❌ "The build passed" → ✅ "The code compiles" (only if user asks)
You are implementing code. The USER clicks Build. NEVER say "build" in any form.

## 2. TALK ABOUT FEATURES, NOT CODE (unless they specifically ask)
The user is a producer/sound designer, not a programmer.
- ❌ "I modified the Params struct..." → ✅ "I added a new control for..."
- ❌ "The process() function now..." → ✅ "The audio processing now..."
- ❌ "I added a UIMessage variant..." → ✅ "The knob is connected."
- ❌ "Let me rewrite lib.rs..." → Just do it silently, then say what feature changed
Talk about SOUND, not code. filters/oscillators/gain/etc = good. structs/functions/Rust = bad.
If user asks "how does this work" or "show me the code" → then explain code. Otherwise, features only.

## 3. BE CONCISE
Say what you did in 1-2 sentences max. Don't narrate your process.
- ❌ "First let me read the file... now I'll add... now let me check..."
- ✅ [Do the work silently, then] "Added the filter with cutoff and resonance controls."

## 4. INTERNAL FILES ARE SECRET
Never mention CLAUDE.md, .vstworkshop/, or metadata files to the user. Update them silently.

## 5. ALWAYS CHECK YOUR SKILLS
Before implementing ANY audio feature, you MUST check if a relevant skill exists in `.claude/commands/`.

**Skill check is MANDATORY for:**
- DSP/audio processing → invoke `/dsp-safety` first
- Filters, EQ, dynamics → `/dsp-safety` has anti-hallucination rules
- Effects (reverb, delay, chorus, etc.) → `/effect-patterns`
- Instruments (synths, samplers) → `/instrument-patterns`
- UI work → `/webview-ui` or `/egui-ui` (whichever this project uses)
- Presets → `/preset-system`
- Polyphony → `/polyphony`
- Envelopes → `/adsr-envelope`
- LFOs → `/lfo`
- Oversampling → `/oversampling`
- Sidechain → `/sidechain-input`

**The skill contains patterns that prevent common mistakes.** Skipping skills = bugs.

---

"#,
    );

    context.push_str(&format!(
        r#"## nih-plug Documentation

A local clone of the nih-plug repository is available at: {}
- Use Grep/Read to search the repo for API examples and syntax
- Key directories: src/lib.rs (main exports), src/params/ (parameter types), src/buffer.rs (audio buffers)
- The plugins/ directory contains example plugins you can reference

## Quick Reference
{}

## Workflow (Follow This Order)

When the user requests a feature:
1. **Read src/lib.rs** to understand current state
2. **INVOKE THE RELEVANT SKILL** - this is NOT optional (see rule #5 above)
3. Implement using patterns from the skill
4. Protect against NaN/Inf: `if !sample.is_finite() {{ *sample = 0.0; }}`
5. Briefly summarize what you added (feature terms, not code terms)

The user will describe what they want. Make the changes directly to the code."#,
        docs_path_str, NIH_PLUG_REFERENCE
    ));

    // Append project-specific CLAUDE.md guidelines if present
    if !claude_md_content.is_empty() {
        context.push_str("\n\n--- PROJECT-SPECIFIC GUIDELINES (from CLAUDE.md) ---\n\n");
        context.push_str(&claude_md_content);
    }

    context
}

/// Load project metadata to get components and other info
fn load_project_metadata(project_path: &str) -> Option<super::projects::ProjectMeta> {
    let metadata_path = PathBuf::from(project_path)
        .join(".vstworkshop")
        .join("metadata.json");

    let content = fs::read_to_string(metadata_path).ok()?;
    serde_json::from_str(&content).ok()
}

#[tauri::command]
pub async fn send_to_claude(
    project_path: String,
    project_name: String,
    description: String,
    message: String,
    model: Option<String>,
    custom_instructions: Option<String>,
    agent_verbosity: Option<String>,
    window: tauri::Window,
) -> Result<ClaudeResponse, String> {
    // Ensure git is initialized for this project (handles existing projects)
    if !super::git::is_git_repo(&project_path) {
        super::git::init_repo(&project_path).await?;
        super::git::create_gitignore(&project_path)?;
        super::git::commit_changes(&project_path, "Initialize git for version control").await?;
    }

    // Ensure .vstworkshop/ is not tracked by git (fixes existing projects)
    if let Err(e) = super::git::ensure_vstworkshop_ignored(&project_path) {
        eprintln!("[WARN] Failed to update gitignore: {}", e);
    }

    // Check for existing session to resume
    let existing_session = load_session_id(&project_path);
    let is_first_message = existing_session.is_none();

    // Load project metadata to get components and UI framework
    let metadata = load_project_metadata(&project_path);
    let components = metadata.as_ref().and_then(|m| m.components.clone());
    let ui_framework = metadata.as_ref().and_then(|m| m.ui_framework.clone());

    // Build context
    let context = build_context(
        &project_name,
        &description,
        &project_path,
        components.as_ref(),
        is_first_message,
        ui_framework.as_deref(),
    );

    // Create provider and config
    let provider = ClaudeProvider;
    let config = ProviderCallConfig {
        project_path: project_path.clone(),
        project_name: project_name.clone(),
        description: description.clone(),
        message: message.clone(),
        model,
        custom_instructions,
        agent_verbosity,
        session_id: existing_session.clone(),
        is_first_message,
        components,
        ui_framework,
    };

    // Build args using provider
    let args = provider.build_args(&config, &context);

    if existing_session.is_some() {
        eprintln!(
            "[DEBUG] Resuming Claude session: {}",
            existing_session.as_ref().unwrap()
        );
    } else {
        eprintln!("[DEBUG] Starting new Claude session");
    }

    // Spawn Claude CLI process
    let mut child = Command::new("claude")
        .current_dir(&project_path)
        .args(&args)
        .env("PATH", super::get_extended_path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn Claude CLI: {}", e))?;

    // Register process for potential interruption (legacy send_to_claude uses Claude provider)
    if let Some(pid) = child.id() {
        register_process(&project_path, AgentProviderType::Claude, pid);
    }

    // Emit start event
    let _ = window.emit(
        "agent-stream",
        AgentStreamEvent::Start {
            project_path: project_path.clone(),
        },
    );

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut full_output = String::new();
    let mut error_output = String::new();
    let mut captured_session_id: Option<String> = None;
    let mut last_substantial_content: Option<String> = None;
    let mut last_nonempty_content: Option<String> = None;

    let read_timeout = Duration::from_secs(900); // 15 minutes
    let mut consecutive_timeouts = 0;
    let max_consecutive_timeouts = 2;

    // Read stdout and stderr concurrently with timeout protection
    loop {
        let read_result = timeout(read_timeout, async {
            tokio::select! {
                line = stdout_reader.next_line() => ("stdout", line),
                line = stderr_reader.next_line() => ("stderr", line),
            }
        })
        .await;

        match read_result {
            Ok(("stdout", line)) => {
                consecutive_timeouts = 0;
                match line {
                    Ok(Some(json_line)) => {
                        // Try to extract session_id if present
                        if let Some(sid) = provider.extract_session_id(&json_line) {
                            captured_session_id = Some(sid);
                        }

                        // Parse using provider
                        let parsed = provider.parse_stream_line(&json_line, &project_path);

                        // Track assistant messages for final content extraction
                        if let Some(ref content) = parsed.assistant_content {
                            let trimmed = content.trim();
                            if !trimmed.is_empty() {
                                last_nonempty_content = Some(content.clone());
                                if trimmed.len() > 10 {
                                    last_substantial_content = Some(content.clone());
                                }
                            }
                        }

                        // Display text during streaming
                        if let Some(display_text) = parsed.display_text {
                            full_output.push_str(&display_text);
                            full_output.push('\n');
                            let _ = window.emit(
                                "agent-stream",
                                AgentStreamEvent::Text {
                                    project_path: project_path.clone(),
                                    content: display_text,
                                },
                            );
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let _ = window.emit(
                            "agent-stream",
                            AgentStreamEvent::Error {
                                project_path: project_path.clone(),
                                message: e.to_string(),
                            },
                        );
                        break;
                    }
                }
            }
            Ok(("stderr", line)) => {
                consecutive_timeouts = 0;
                match line {
                    Ok(Some(text)) => {
                        error_output.push_str(&text);
                        error_output.push('\n');
                        let _ = window.emit(
                            "agent-stream",
                            AgentStreamEvent::Text {
                                project_path: project_path.clone(),
                                content: format!("[stderr] {}", text),
                            },
                        );
                    }
                    Ok(None) => {}
                    Err(_) => {}
                }
            }
            Ok(_) => {}
            Err(_) => {
                consecutive_timeouts += 1;
                let timeout_mins = read_timeout.as_secs() / 60;
                eprintln!(
                    "[WARN] Claude CLI read timeout ({}/{})",
                    consecutive_timeouts, max_consecutive_timeouts
                );

                let _ = window.emit(
                    "agent-stream",
                    AgentStreamEvent::Text {
                        project_path: project_path.clone(),
                        content: format!("[Warning] No output for {} minutes...", timeout_mins),
                    },
                );

                if consecutive_timeouts >= max_consecutive_timeouts {
                    eprintln!("[ERROR] Claude CLI appears stalled, terminating process");
                    let _ = window.emit(
                        "agent-stream",
                        AgentStreamEvent::Error {
                            project_path: project_path.clone(),
                            message: "Claude CLI stalled (no output for 30 minutes). Session terminated.".to_string(),
                        },
                    );
                    let _ = child.kill().await;
                    break;
                }
            }
        }
    }

    // Wait for process to complete
    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed to wait for Claude CLI: {}", e))?;

    let was_interrupted = get_process_pid(&project_path).is_none();
    unregister_process(&project_path);

    // Handle non-success exit
    if !status.success() {
        if !error_output.is_empty() {
            let _ = window.emit(
                "agent-stream",
                AgentStreamEvent::Error {
                    project_path: project_path.clone(),
                    message: error_output.clone(),
                },
            );
            return Err(format!("Claude CLI failed: {}", error_output));
        } else if was_interrupted {
            return Err("Session interrupted".to_string());
        } else {
            let _ = window.emit(
                "agent-stream",
                AgentStreamEvent::Error {
                    project_path: project_path.clone(),
                    message: "Claude CLI terminated unexpectedly".to_string(),
                },
            );
            return Err("Claude CLI terminated unexpectedly".to_string());
        }
    }

    // Helper to check if text looks like a "done" message
    let is_done_like = |text: &str| -> bool {
        let trimmed = text.trim().to_lowercase();
        trimmed.len() <= 15
            && (trimmed == "done"
                || trimmed == "done!"
                || trimmed == "done."
                || trimmed == "finished"
                || trimmed == "finished!"
                || trimmed == "complete"
                || trimmed == "complete!"
                || trimmed.starts_with("all done")
                || trimmed.starts_with("that's it")
                || trimmed.starts_with("thats it")
                || trimmed.contains("✓ done")
                || trimmed.contains("✓done")
                || (trimmed.len() < 15 && trimmed.contains("done")))
    };

    let streaming_ends_with_done = full_output
        .lines()
        .last()
        .map(|line| is_done_like(line))
        .unwrap_or(false);

    let final_content = if streaming_ends_with_done {
        "All done! What would you like to do next?".to_string()
    } else if let Some(ref last) = last_nonempty_content {
        if is_done_like(last) {
            "All done! What would you like to do next?".to_string()
        } else if last.trim().len() > 10 {
            last.clone()
        } else {
            last_substantial_content.unwrap_or_else(|| last.clone())
        }
    } else {
        full_output.clone()
    };

    // Emit done event
    let _ = window.emit(
        "agent-stream",
        AgentStreamEvent::Done {
            project_path: project_path.clone(),
            content: final_content.clone(),
        },
    );

    // Save session ID for next conversation
    if let Some(ref sid) = captured_session_id {
        if let Err(e) = save_session_id(&project_path, sid) {
            eprintln!("[WARN] Failed to save session ID: {}", e);
        } else {
            eprintln!("[DEBUG] Saved session ID: {}", sid);
        }
    }

    // Commit changes after Claude finishes
    let commit_msg = if message.len() > 50 {
        format!("{}...", &message[..47])
    } else {
        message.clone()
    };
    let commit_hash = super::git::commit_changes(&project_path, &commit_msg)
        .await
        .ok();

    Ok(ClaudeResponse {
        content: final_content,
        session_id: captured_session_id,
        commit_hash,
    })
}

/// Simple test to verify Claude CLI is accessible and working
#[tauri::command]
pub async fn test_claude_cli() -> Result<String, String> {
    let output = Command::new("claude")
        .args(["--version"])
        .env("PATH", super::get_extended_path())
        .output()
        .await
        .map_err(|e| format!("Failed to run Claude CLI: {}", e))?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    } else {
        Err("Claude CLI not available".to_string())
    }
}

/// Interrupt a running agent session for a specific project
/// Works with any provider (Claude, OpenCode, etc.)
#[tauri::command]
pub async fn interrupt_agent(project_path: String, window: tauri::Window) -> Result<(), String> {
    if let Some((provider_type, pid)) = get_process_info(&project_path) {
        let provider_name = match provider_type {
            AgentProviderType::Claude => "Claude",
            AgentProviderType::OpenCode => "OpenCode",
        };

        eprintln!(
            "[DEBUG] Interrupting {} process {} for {}",
            provider_name, pid, project_path
        );

        #[cfg(unix)]
        {
            use std::process::Command as StdCommand;
            let _ = StdCommand::new("kill")
                .args(["-TERM", &pid.to_string()])
                .output();
        }

        #[cfg(windows)]
        {
            use std::process::Command as StdCommand;
            let _ = StdCommand::new("taskkill")
                .args(["/PID", &pid.to_string(), "/F"])
                .output();
        }

        unregister_process(&project_path);

        let _ = window.emit(
            "agent-stream",
            AgentStreamEvent::Text {
                project_path: project_path.clone(),
                content: "Session stopped. Ready for your next message.".to_string(),
            },
        );

        Ok(())
    } else {
        Err("No active agent session for this project".to_string())
    }
}

// ============================================================================
// Unified Agent Command - Works with any provider
// ============================================================================

/// Unified command to send messages to any AI agent provider
///
/// This command dispatches to the configured provider (Claude, OpenCode, etc.)
/// and handles streaming output, session management, and git commits.
#[tauri::command]
pub async fn send_to_agent(
    provider_type: Option<AgentProviderType>,
    project_path: String,
    project_name: String,
    description: String,
    message: String,
    model: Option<String>,
    custom_instructions: Option<String>,
    agent_verbosity: Option<String>,
    window: tauri::Window,
) -> Result<ClaudeResponse, String> {
    // Default to Claude if no provider specified
    let provider_type = provider_type.unwrap_or_default();
    let provider = get_provider(&provider_type);

    // Ensure git is initialized for this project (handles existing projects)
    if !super::git::is_git_repo(&project_path) {
        super::git::init_repo(&project_path).await?;
        super::git::create_gitignore(&project_path)?;
        super::git::commit_changes(&project_path, "Initialize git for version control").await?;
    }

    // Ensure .vstworkshop/ is not tracked by git (fixes existing projects)
    if let Err(e) = super::git::ensure_vstworkshop_ignored(&project_path) {
        eprintln!("[WARN] Failed to update gitignore: {}", e);
    }

    // Check for existing session to resume (provider-specific)
    let existing_session = load_session_id_for_provider(&project_path, &provider_type);
    let is_first_message = existing_session.is_none();

    // Load project metadata to get components and UI framework
    let metadata = load_project_metadata(&project_path);
    let components = metadata.as_ref().and_then(|m| m.components.clone());
    let ui_framework = metadata.as_ref().and_then(|m| m.ui_framework.clone());

    // Build context
    let context = build_context(
        &project_name,
        &description,
        &project_path,
        components.as_ref(),
        is_first_message,
        ui_framework.as_deref(),
    );

    // Create provider config
    let config = ProviderCallConfig {
        project_path: project_path.clone(),
        project_name: project_name.clone(),
        description: description.clone(),
        message: message.clone(),
        model,
        custom_instructions,
        agent_verbosity,
        session_id: existing_session.clone(),
        is_first_message,
        components,
        ui_framework,
    };

    // Build args using provider
    let args = provider.build_args(&config, &context);

    let cli_binary = provider_type.cli_binary();
    let provider_name = provider_type.display_name();

    if existing_session.is_some() {
        eprintln!(
            "[DEBUG] Resuming {} session: {}",
            provider_name,
            existing_session.as_ref().unwrap()
        );
    } else {
        eprintln!("[DEBUG] Starting new {} session", provider_name);
    }

    // Spawn CLI process
    let mut child = Command::new(cli_binary)
        .current_dir(&project_path)
        .args(&args)
        .env("PATH", super::get_extended_path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn {} CLI: {}", provider_name, e))?;

    // Register process for potential interruption (with provider type for accurate tracking)
    if let Some(pid) = child.id() {
        register_process(&project_path, provider_type, pid);
    }

    // Emit start event (use same event name for frontend compatibility)
    let _ = window.emit(
        "agent-stream",
        AgentStreamEvent::Start {
            project_path: project_path.clone(),
        },
    );

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut full_output = String::new();
    let mut error_output = String::new();
    let mut captured_session_id: Option<String> = None;
    let mut last_substantial_content: Option<String> = None;
    let mut last_nonempty_content: Option<String> = None;

    let read_timeout = Duration::from_secs(900); // 15 minutes
    let mut consecutive_timeouts = 0;
    let max_consecutive_timeouts = 2;

    // Read stdout and stderr concurrently with timeout protection
    loop {
        let read_result = timeout(read_timeout, async {
            tokio::select! {
                line = stdout_reader.next_line() => ("stdout", line),
                line = stderr_reader.next_line() => ("stderr", line),
            }
        })
        .await;

        match read_result {
            Ok(("stdout", line)) => {
                consecutive_timeouts = 0;
                match line {
                    Ok(Some(json_line)) => {
                        // Try to extract session_id if present
                        if let Some(sid) = provider.extract_session_id(&json_line) {
                            captured_session_id = Some(sid);
                        }

                        // Parse using provider
                        let parsed = provider.parse_stream_line(&json_line, &project_path);

                        // Track assistant messages for final content extraction
                        if let Some(ref content) = parsed.assistant_content {
                            let trimmed = content.trim();
                            if !trimmed.is_empty() {
                                last_nonempty_content = Some(content.clone());
                                if trimmed.len() > 10 {
                                    last_substantial_content = Some(content.clone());
                                }
                            }
                        }

                        // Display text during streaming
                        if let Some(display_text) = parsed.display_text {
                            full_output.push_str(&display_text);
                            full_output.push('\n');
                            let _ = window.emit(
                                "agent-stream",
                                AgentStreamEvent::Text {
                                    project_path: project_path.clone(),
                                    content: display_text,
                                },
                            );
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let _ = window.emit(
                            "agent-stream",
                            AgentStreamEvent::Error {
                                project_path: project_path.clone(),
                                message: e.to_string(),
                            },
                        );
                        break;
                    }
                }
            }
            Ok(("stderr", line)) => {
                consecutive_timeouts = 0;
                match line {
                    Ok(Some(text)) => {
                        error_output.push_str(&text);
                        error_output.push('\n');
                        let _ = window.emit(
                            "agent-stream",
                            AgentStreamEvent::Text {
                                project_path: project_path.clone(),
                                content: format!("[stderr] {}", text),
                            },
                        );
                    }
                    Ok(None) => {}
                    Err(_) => {}
                }
            }
            Ok(_) => {}
            Err(_) => {
                consecutive_timeouts += 1;
                let timeout_mins = read_timeout.as_secs() / 60;
                eprintln!(
                    "[WARN] {} CLI read timeout ({}/{})",
                    provider_name, consecutive_timeouts, max_consecutive_timeouts
                );

                let _ = window.emit(
                    "agent-stream",
                    AgentStreamEvent::Text {
                        project_path: project_path.clone(),
                        content: format!("[Warning] No output for {} minutes...", timeout_mins),
                    },
                );

                if consecutive_timeouts >= max_consecutive_timeouts {
                    eprintln!("[ERROR] {} CLI appears stalled, terminating process", provider_name);
                    let _ = window.emit(
                        "agent-stream",
                        AgentStreamEvent::Error {
                            project_path: project_path.clone(),
                            message: format!("{} CLI stalled (no output for 30 minutes). Session terminated.", provider_name),
                        },
                    );
                    let _ = child.kill().await;
                    break;
                }
            }
        }
    }

    // Wait for process to complete
    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed to wait for {} CLI: {}", provider_name, e))?;

    let was_interrupted = get_process_pid(&project_path).is_none();
    unregister_process(&project_path);

    // Handle non-success exit
    if !status.success() {
        if !error_output.is_empty() {
            let _ = window.emit(
                "agent-stream",
                AgentStreamEvent::Error {
                    project_path: project_path.clone(),
                    message: error_output.clone(),
                },
            );
            return Err(format!("{} CLI failed: {}", provider_name, error_output));
        } else if was_interrupted {
            return Err("Session interrupted".to_string());
        } else {
            let _ = window.emit(
                "agent-stream",
                AgentStreamEvent::Error {
                    project_path: project_path.clone(),
                    message: format!("{} CLI terminated unexpectedly", provider_name),
                },
            );
            return Err(format!("{} CLI terminated unexpectedly", provider_name));
        }
    }

    // Helper to check if text looks like a "done" message
    let is_done_like = |text: &str| -> bool {
        let trimmed = text.trim().to_lowercase();
        trimmed.len() <= 15
            && (trimmed == "done"
                || trimmed == "done!"
                || trimmed == "done."
                || trimmed == "finished"
                || trimmed == "finished!"
                || trimmed == "complete"
                || trimmed == "complete!"
                || trimmed.starts_with("all done")
                || trimmed.starts_with("that's it")
                || trimmed.starts_with("thats it")
                || trimmed.contains("✓ done")
                || trimmed.contains("✓done")
                || (trimmed.len() < 15 && trimmed.contains("done")))
    };

    let streaming_ends_with_done = full_output
        .lines()
        .last()
        .map(|line| is_done_like(line))
        .unwrap_or(false);

    let final_content = if streaming_ends_with_done {
        "All done! What would you like to do next?".to_string()
    } else if let Some(ref last) = last_nonempty_content {
        if is_done_like(last) {
            "All done! What would you like to do next?".to_string()
        } else if last.trim().len() > 10 {
            last.clone()
        } else {
            last_substantial_content.unwrap_or_else(|| last.clone())
        }
    } else {
        full_output.clone()
    };

    // Emit done event
    let _ = window.emit(
        "agent-stream",
        AgentStreamEvent::Done {
            project_path: project_path.clone(),
            content: final_content.clone(),
        },
    );

    // Save session ID for next conversation (provider-specific)
    if let Some(ref sid) = captured_session_id {
        if let Err(e) = save_session_id_for_provider(&project_path, sid, &provider_type) {
            eprintln!("[WARN] Failed to save session ID: {}", e);
        } else {
            eprintln!("[DEBUG] Saved {} session ID: {}", provider_name, sid);
        }
    }

    // Commit changes after agent finishes
    let commit_msg = if message.len() > 50 {
        format!("{}...", &message[..47])
    } else {
        message.clone()
    };
    let commit_hash = super::git::commit_changes(&project_path, &commit_msg)
        .await
        .ok();

    Ok(ClaudeResponse {
        content: final_content,
        session_id: captured_session_id,
        commit_hash,
    })
}
