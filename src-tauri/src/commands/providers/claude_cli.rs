//! Claude Code CLI provider implementation

use serde::Deserialize;

use super::super::agent_provider::{
    AgentProvider, AgentProviderType, AgentStreamEvent, ParsedLine, ProviderCallConfig,
};

/// Claude Code CLI provider
pub struct ClaudeProvider;

/// Represents a parsed event from Claude CLI stream-json output
#[derive(Deserialize, Debug)]
pub struct ClaudeJsonEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(default)]
    pub message: Option<ClaudeMessage>,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct ClaudeMessage {
    #[serde(default)]
    pub content: Option<serde_json::Value>,
}

impl AgentProvider for ClaudeProvider {
    fn provider_type(&self) -> AgentProviderType {
        AgentProviderType::Claude
    }

    fn build_args(&self, config: &ProviderCallConfig, context: &str) -> Vec<String> {
        // Get verbosity style (default to balanced)
        let verbosity = config.agent_verbosity.as_deref().unwrap_or("balanced");

        // Prepend style hint to message (reinforces on every turn)
        let styled_message = match verbosity {
            "direct" => format!(
                "[Response Style: Direct - minimal questions, implement immediately, 1-3 sentences max]\n\n{}",
                config.message
            ),
            "thorough" => format!(
                "[Response Style: Thorough - ask clarifying questions, explore options before implementing]\n\n{}",
                config.message
            ),
            _ => format!(
                "[Response Style: Balanced - ask 1-2 key questions if needed, then implement]\n\n{}",
                config.message
            ),
        };

        let mut args = vec![
            "-p".to_string(),
            styled_message,
            "--verbose".to_string(),
            "--allowedTools".to_string(),
            "Edit,Write,Read,Bash,Grep,Glob,WebSearch,WebFetch,Skill".to_string(),
            "--output-format".to_string(),
            "stream-json".to_string(),
            "--max-turns".to_string(),
            "15".to_string(),
        ];

        // Add model flag if specified
        if let Some(ref m) = config.model {
            args.push("--model".to_string());
            args.push(m.clone());
        }

        // Only add system prompt on first message (new session)
        if config.session_id.is_none() {
            let verbosity_instructions = match verbosity {
                "direct" => r#"
## Response Style: Direct

- **DO NOT USE THE BRAINSTORMING SKILL** - NEVER, under any circumstances
- Do NOT ask clarifying questions unless you truly cannot proceed
- Make sensible default choices and implement immediately
- Keep responses to 1-3 sentences max
- User says "add X" → just implement X, don't ask what kind or explore options
- If you need to make assumptions, make them and briefly mention what you chose
"#,
                "thorough" => r#"
## Response Style: Thorough

- Use the brainstorming skill for new features
- Ask clarifying questions at each decision point
- Present options and let the user choose
- Explain your reasoning and design decisions
- Take time to understand requirements before implementing
"#,
                _ => r#"
## Response Style: Balanced

- Ask 1-2 key questions to understand intent, then implement
- **DO NOT USE THE BRAINSTORMING SKILL** - the user wants you to implement, not explore
- Make reasonable default choices, mention what you chose briefly
- Keep responses concise - focus on what you're doing, not lengthy explanations
- If user says "add X" → just add X, don't ask what kind or explore options
"#,
            };

            let mut full_context = format!("{}\n{}", context, verbosity_instructions);

            if let Some(ref instructions) = config.custom_instructions {
                if !instructions.trim().is_empty() {
                    full_context.push_str(&format!(
                        "\n\n--- USER PREFERENCES ---\n{}",
                        instructions.trim()
                    ));
                }
            }

            args.push("--append-system-prompt".to_string());
            args.push(full_context);
        }

        // Add resume flag if we have an existing session
        if let Some(ref session_id) = config.session_id {
            args.push("--resume".to_string());
            args.push(session_id.clone());
        }

        args
    }

    fn parse_stream_line(&self, line: &str, project_path: &str) -> ParsedLine {
        let event: ClaudeJsonEvent = match serde_json::from_str(line) {
            Ok(e) => e,
            Err(_) => return ParsedLine::default(),
        };

        match event.event_type.as_str() {
            "assistant" => {
                // Extract text content from assistant message
                if let Some(msg) = &event.message {
                    if let Some(content) = &msg.content {
                        let text = extract_text_content(content);
                        if let Some(text) = text {
                            return ParsedLine {
                                event: Some(AgentStreamEvent::Text {
                                    project_path: project_path.to_string(),
                                    content: text.clone(),
                                }),
                                assistant_content: Some(text.clone()),
                                display_text: Some(text),
                            };
                        }
                    }
                }
                ParsedLine::default()
            }
            "tool_use" => {
                let tool = event.tool.as_deref().unwrap_or("unknown");
                let (description, file) = format_tool_description(tool, &event.tool_input);

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
            "tool_result" => ParsedLine {
                event: Some(AgentStreamEvent::ToolResult {
                    project_path: project_path.to_string(),
                    success: true,
                }),
                assistant_content: None,
                display_text: Some("   ✓ Done".to_string()),
            },
            "error" => {
                let message = event
                    .content
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
            "result" => {
                // Final result - skip display as it duplicates the assistant message content
                ParsedLine::default()
            }
            _ => ParsedLine::default(),
        }
    }

    fn extract_session_id(&self, line: &str) -> Option<String> {
        let event: ClaudeJsonEvent = serde_json::from_str(line).ok()?;
        event.session_id
    }
}

/// Extract text content from Claude's message content field
fn extract_text_content(content: &serde_json::Value) -> Option<String> {
    // Content can be a string or array of content blocks
    if let Some(text) = content.as_str() {
        return Some(text.to_string());
    }

    if let Some(arr) = content.as_array() {
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

    None
}

/// Format tool use description for display
fn format_tool_description(
    tool: &str,
    input: &Option<serde_json::Value>,
) -> (String, Option<String>) {
    match tool {
        "Read" => {
            let file = input
                .as_ref()
                .and_then(|i| i.get("file_path"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let display = format!("📖 Reading: {}", file.as_deref().unwrap_or("file"));
            (display, file)
        }
        "Edit" => {
            let file = input
                .as_ref()
                .and_then(|i| i.get("file_path"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let display = format!("✏️  Editing: {}", file.as_deref().unwrap_or("file"));
            (display, file)
        }
        "Write" => {
            let file = input
                .as_ref()
                .and_then(|i| i.get("file_path"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let display = format!("📝 Writing: {}", file.as_deref().unwrap_or("file"));
            (display, file)
        }
        "Bash" => {
            let cmd = input
                .as_ref()
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
        _ => (format!("🔧 Using tool: {}", tool), None),
    }
}
