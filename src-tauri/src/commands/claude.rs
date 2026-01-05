use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

#[derive(Serialize, Clone)]
pub struct ClaudeResponse {
    pub content: String,
    pub session_id: Option<String>,
}

#[derive(Serialize, Clone)]
#[serde(tag = "type")]
pub enum ClaudeStreamEvent {
    #[serde(rename = "start")]
    Start,
    #[serde(rename = "text")]
    Text { content: String },
    #[serde(rename = "error")]
    Error { message: String },
    #[serde(rename = "done")]
    Done { content: String },
}

/// Represents a parsed event from Claude CLI stream-json output
#[derive(Deserialize, Debug)]
struct ClaudeJsonEvent {
    #[serde(rename = "type")]
    event_type: String,
    #[serde(default)]
    message: Option<ClaudeMessage>,
    #[serde(default)]
    tool: Option<String>,
    #[serde(default)]
    tool_input: Option<serde_json::Value>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    result: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
struct ClaudeMessage {
    #[serde(default)]
    content: Option<serde_json::Value>,
}

/// Parse a JSON event and return a human-readable string
fn parse_claude_event(json_str: &str) -> Option<String> {
    let event: ClaudeJsonEvent = serde_json::from_str(json_str).ok()?;

    match event.event_type.as_str() {
        "assistant" => {
            // Extract text content from assistant message
            if let Some(msg) = &event.message {
                if let Some(content) = &msg.content {
                    // Content can be a string or array of content blocks
                    if let Some(text) = content.as_str() {
                        return Some(text.to_string());
                    } else if let Some(arr) = content.as_array() {
                        let texts: Vec<String> = arr
                            .iter()
                            .filter_map(|item| {
                                if item.get("type")?.as_str()? == "text" {
                                    item.get("text")?.as_str().map(|s| s.to_string())
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if !texts.is_empty() {
                            return Some(texts.join("\n"));
                        }
                    }
                }
            }
            None
        }
        "tool_use" => {
            let tool = event.tool.as_deref().unwrap_or("unknown");
            match tool {
                "Read" => {
                    if let Some(input) = &event.tool_input {
                        let file = input.get("file_path")
                            .and_then(|v| v.as_str())
                            .unwrap_or("file");
                        Some(format!("📖 Reading: {}", file))
                    } else {
                        Some("📖 Reading file...".to_string())
                    }
                }
                "Edit" => {
                    if let Some(input) = &event.tool_input {
                        let file = input.get("file_path")
                            .and_then(|v| v.as_str())
                            .unwrap_or("file");
                        Some(format!("✏️  Editing: {}", file))
                    } else {
                        Some("✏️  Editing file...".to_string())
                    }
                }
                "Write" => {
                    if let Some(input) = &event.tool_input {
                        let file = input.get("file_path")
                            .and_then(|v| v.as_str())
                            .unwrap_or("file");
                        Some(format!("📝 Writing: {}", file))
                    } else {
                        Some("📝 Writing file...".to_string())
                    }
                }
                "Bash" => {
                    if let Some(input) = &event.tool_input {
                        let cmd = input.get("command")
                            .and_then(|v| v.as_str())
                            .unwrap_or("command");
                        // Truncate long commands
                        let display_cmd = if cmd.len() > 60 {
                            format!("{}...", &cmd[..60])
                        } else {
                            cmd.to_string()
                        };
                        Some(format!("💻 Running: {}", display_cmd))
                    } else {
                        Some("💻 Running command...".to_string())
                    }
                }
                _ => Some(format!("🔧 Using tool: {}", tool)),
            }
        }
        "tool_result" => {
            // Tool completed - could show result summary
            Some("   ✓ Done".to_string())
        }
        "result" => {
            // Final result
            if let Some(result) = &event.result {
                Some(format!("\n{}", result))
            } else {
                None
            }
        }
        "error" => {
            if let Some(content) = &event.content {
                Some(format!("❌ Error: {}", content))
            } else {
                Some("❌ An error occurred".to_string())
            }
        }
        _ => None,
    }
}

/// Build the system context for Claude when working on a plugin project
fn build_context(project_name: &str, description: &str) -> String {
    format!(
        r#"You are helping develop a VST audio plugin using nih-plug (Rust).

Project: {project_name}
Description: {description}

IMPORTANT: For nih-plug API reference, always check the official docs at:
https://nih-plug.robbertvanderhelm.nl/nih_plug/index.html

Key nih-plug conventions:
- Plugin struct holds params as Arc<PluginParams>
- Use #[derive(Params)] for parameter structs
- FloatParam for continuous values, IntParam for integers, BoolParam for toggles
- Process audio in the `process` method, iterate over buffer.iter_samples()
- Use .smoothed.next() for parameter smoothing
- Export with nih_export_vst3! and nih_export_clap! macros

When modifying the plugin:
1. Read src/lib.rs first to understand current state
2. Edit src/lib.rs with your changes
3. Keep code clean and well-commented
4. Use proper DSP practices (avoid denormals, handle edge cases)
5. Briefly summarize what you changed after making edits

The user will describe what they want. Make the changes directly to the code."#,
        project_name = project_name,
        description = description
    )
}

#[tauri::command]
pub async fn send_to_claude(
    project_path: String,
    project_name: String,
    description: String,
    message: String,
    window: tauri::Window,
) -> Result<ClaudeResponse, String> {
    // Build context
    let context = build_context(&project_name, &description);

    // Spawn Claude CLI process with stream-json for detailed output
    let mut child = Command::new("claude")
        .current_dir(&project_path)
        .args([
            "-p",
            &message,
            "--verbose",
            "--allowedTools",
            "Edit,Write,Read",
            "--output-format",
            "stream-json",
            "--append-system-prompt",
            &context,
            "--max-turns",
            "15",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn Claude CLI: {}", e))?;

    // Emit start event
    let _ = window.emit("claude-stream", ClaudeStreamEvent::Start);

    let stdout = child
        .stdout
        .take()
        .ok_or("Failed to capture stdout")?;

    let stderr = child
        .stderr
        .take()
        .ok_or("Failed to capture stderr")?;

    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();

    let mut full_output = String::new();
    let mut error_output = String::new();

    // Read stdout and stderr concurrently
    loop {
        tokio::select! {
            line = stdout_reader.next_line() => {
                match line {
                    Ok(Some(json_line)) => {
                        // Try to parse as JSON event
                        if let Some(display_text) = parse_claude_event(&json_line) {
                            full_output.push_str(&display_text);
                            full_output.push('\n');
                            let _ = window.emit("claude-stream", ClaudeStreamEvent::Text {
                                content: display_text,
                            });
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let _ = window.emit("claude-stream", ClaudeStreamEvent::Error {
                            message: e.to_string(),
                        });
                        break;
                    }
                }
            }
            line = stderr_reader.next_line() => {
                match line {
                    Ok(Some(text)) => {
                        error_output.push_str(&text);
                        error_output.push('\n');
                        // Also emit stderr as it may contain useful info
                        let _ = window.emit("claude-stream", ClaudeStreamEvent::Text {
                            content: format!("[stderr] {}", text),
                        });
                    }
                    Ok(None) => {}
                    Err(_) => {}
                }
            }
        }
    }

    // Wait for process to complete
    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed to wait for Claude CLI: {}", e))?;

    if !status.success() && !error_output.is_empty() {
        let _ = window.emit("claude-stream", ClaudeStreamEvent::Error {
            message: error_output.clone(),
        });
        return Err(format!("Claude CLI failed: {}", error_output));
    }

    // Emit done event
    let _ = window.emit("claude-stream", ClaudeStreamEvent::Done {
        content: full_output.clone(),
    });

    Ok(ClaudeResponse {
        content: full_output,
        session_id: None,
    })
}

/// Simple test to verify Claude CLI is accessible and working
#[tauri::command]
pub async fn test_claude_cli() -> Result<String, String> {
    let output = Command::new("claude")
        .args(["--version"])
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
