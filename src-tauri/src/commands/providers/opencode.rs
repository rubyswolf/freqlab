//! OpenCode CLI provider implementation

use serde::Deserialize;

use super::super::agent_provider::{
    AgentProvider, AgentProviderType, AgentStreamEvent, ParsedLine, ProviderCallConfig,
};

/// OpenCode CLI provider
pub struct OpenCodeProvider;

/// Represents a parsed event from OpenCode CLI --format json output
#[derive(Deserialize, Debug)]
pub struct OpenCodeJsonEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    #[serde(default)]
    pub part: Option<OpenCodePart>,
}

#[derive(Deserialize, Debug)]
pub struct OpenCodePart {
    #[serde(rename = "type")]
    pub part_type: String,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub state: Option<OpenCodeToolState>,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OpenCodeToolState {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub input: Option<serde_json::Value>,
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
}

impl AgentProvider for OpenCodeProvider {
    fn provider_type(&self) -> AgentProviderType {
        AgentProviderType::OpenCode
    }

    fn build_args(&self, config: &ProviderCallConfig, _context: &str) -> Vec<String> {
        let mut args = vec![
            "run".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ];

        // Add model flag if specified
        if let Some(ref m) = config.model {
            args.push("--model".to_string());
            args.push(m.clone());
        }

        // Add session flag if resuming
        if let Some(ref session_id) = config.session_id {
            args.push("--session".to_string());
            args.push(session_id.clone());
        }

        // Get verbosity style (default to balanced)
        let verbosity = config.agent_verbosity.as_deref().unwrap_or("balanced");

        // Build the message with style hint
        // OpenCode automatically reads AGENTS.md (same content as CLAUDE.md), so we
        // only need to add custom user instructions and verbosity hints to the message
        let mut message_parts = Vec::new();

        // Add verbosity hint
        let verbosity_hint = match verbosity {
            "direct" => "[Response Style: Direct - minimal questions, implement immediately, 1-3 sentences max]",
            "thorough" => "[Response Style: Thorough - ask clarifying questions, explore options before implementing]",
            _ => "[Response Style: Balanced - ask 1-2 key questions if needed, then implement]",
        };
        message_parts.push(verbosity_hint.to_string());

        // Add custom instructions if present (for new sessions only)
        if config.session_id.is_none() {
            if let Some(ref instructions) = config.custom_instructions {
                if !instructions.trim().is_empty() {
                    message_parts.push(format!("User preferences: {}", instructions.trim()));
                }
            }
        }

        // Add the actual message
        message_parts.push(config.message.clone());

        args.push(message_parts.join("\n\n"));
        args
    }

    fn parse_stream_line(&self, line: &str, project_path: &str) -> ParsedLine {
        let event: OpenCodeJsonEvent = match serde_json::from_str(line) {
            Ok(e) => e,
            Err(_) => return ParsedLine::default(),
        };

        let part = match &event.part {
            Some(p) => p,
            None => return ParsedLine::default(),
        };

        match event.event_type.as_str() {
            "text" => {
                if let Some(text) = &part.text {
                    if !text.is_empty() {
                        return ParsedLine {
                            event: Some(AgentStreamEvent::Text {
                                project_path: project_path.to_string(),
                                content: text.clone(),
                            }),
                            assistant_content: Some(text.clone()),
                            display_text: Some(text.clone()),
                        };
                    }
                }
                ParsedLine::default()
            }
            "tool_use" => {
                let tool = part.tool.as_deref().unwrap_or("unknown");
                let (description, file) = format_tool_description(tool, &part.state);

                ParsedLine {
                    event: Some(AgentStreamEvent::ToolUse {
                        project_path: project_path.to_string(),
                        tool: tool.to_string(),
                        file,
                        description: description.clone(),
                    }),
                    assistant_content: None,
                    display_text: Some(description),
                }
            }
            "step_finish" => {
                // Check if this was a tool-calls step (tool completed)
                if part.reason.as_deref() == Some("tool-calls") {
                    ParsedLine {
                        event: Some(AgentStreamEvent::ToolResult {
                            project_path: project_path.to_string(),
                            success: true,
                        }),
                        assistant_content: None,
                        display_text: Some("   ✓ Done".to_string()),
                    }
                } else {
                    ParsedLine::default()
                }
            }
            "step_start" => {
                // Silent event, just marks step beginning
                ParsedLine::default()
            }
            "error" => {
                let message = part
                    .text
                    .clone()
                    .unwrap_or_else(|| "An error occurred".to_string());
                let display = format!("❌ Error: {}", message);

                ParsedLine {
                    event: Some(AgentStreamEvent::Error {
                        project_path: project_path.to_string(),
                        message,
                    }),
                    assistant_content: None,
                    display_text: Some(display),
                }
            }
            _ => ParsedLine::default(),
        }
    }

    fn extract_session_id(&self, line: &str) -> Option<String> {
        let event: OpenCodeJsonEvent = serde_json::from_str(line).ok()?;
        event.session_id
    }
}

/// Format tool use description for display
fn format_tool_description(
    tool: &str,
    state: &Option<OpenCodeToolState>,
) -> (String, Option<String>) {
    let input = state.as_ref().and_then(|s| s.input.as_ref());
    let title = state.as_ref().and_then(|s| s.title.clone());

    match tool {
        "read" => {
            let file = input
                .and_then(|i| i.get("filePath"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let display_name = title.unwrap_or_else(|| file.clone().unwrap_or_else(|| "file".to_string()));
            let display = format!("📖 Reading: {}", display_name);
            (display, file)
        }
        "edit" | "write" => {
            let file = input
                .and_then(|i| i.get("filePath"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let display_name = title.unwrap_or_else(|| file.clone().unwrap_or_else(|| "file".to_string()));
            let emoji = if tool == "edit" { "✏️ " } else { "📝" };
            let action = if tool == "edit" { "Editing" } else { "Writing" };
            let display = format!("{} {}: {}", emoji, action, display_name);
            (display, file)
        }
        "bash" => {
            let cmd = input
                .and_then(|i| i.get("command"))
                .and_then(|v| v.as_str())
                .unwrap_or("command");
            let display_cmd = if cmd.len() > 60 {
                format!("{}...", &cmd[..60])
            } else {
                cmd.to_string()
            };
            (format!("💻 Running: {}", display_cmd), None)
        }
        "grep" | "glob" => {
            let pattern = input
                .and_then(|i| i.get("pattern"))
                .and_then(|v| v.as_str())
                .unwrap_or("...");
            let emoji = if tool == "grep" { "🔍" } else { "📂" };
            (format!("{} Searching: {}", emoji, pattern), None)
        }
        _ => (format!("🔧 Using tool: {}", tool), None),
    }
}
