//! Agent Provider abstraction layer
//!
//! Provides a common interface for different coding agent CLIs (Claude Code, OpenCode, etc.)

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Supported agent providers
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AgentProviderType {
    #[default]
    Claude,
    OpenCode,
}

/// Status of an agent provider installation
#[derive(Clone, Debug, Serialize, Default)]
pub struct ProviderStatus {
    pub installed: bool,
    pub authenticated: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl AgentProviderType {
    /// Get the display name for this provider
    pub fn display_name(&self) -> &'static str {
        match self {
            AgentProviderType::Claude => "Claude Code CLI",
            AgentProviderType::OpenCode => "OpenCode",
        }
    }

    /// Get the CLI binary name
    pub fn cli_binary(&self) -> &'static str {
        match self {
            AgentProviderType::Claude => "claude",
            AgentProviderType::OpenCode => "opencode",
        }
    }

    /// Get install command for this provider
    pub fn install_command(&self) -> &'static str {
        match self {
            AgentProviderType::Claude => "npm i -g @anthropic-ai/claude-code",
            AgentProviderType::OpenCode => "curl -fsSL https://opencode.ai/install | bash",
        }
    }

    /// Get auth command for this provider
    pub fn auth_command(&self) -> &'static str {
        match self {
            AgentProviderType::Claude => "claude auth",
            AgentProviderType::OpenCode => "opencode auth login",
        }
    }

    /// Get available models for this provider
    pub fn available_models(&self) -> Vec<(&'static str, &'static str)> {
        match self {
            AgentProviderType::Claude => vec![
                ("sonnet", "Claude Sonnet (Default)"),
                ("opus", "Claude Opus (Most Capable)"),
                ("haiku", "Claude Haiku (Fastest)"),
            ],
            AgentProviderType::OpenCode => vec![
                ("anthropic/claude-sonnet-4", "Claude Sonnet 4 (Anthropic)"),
                ("anthropic/claude-opus-4", "Claude Opus 4 (Anthropic)"),
                ("openai/gpt-4o", "GPT-4o (OpenAI)"),
                ("openai/o1", "o1 (OpenAI)"),
                ("google/gemini-2.0-flash", "Gemini 2.0 Flash (Google)"),
                ("ollama/llama3", "Llama 3 (Local/Ollama)"),
            ],
        }
    }

    /// Get session file path for a project
    pub fn session_file_path(&self, project_path: &str) -> PathBuf {
        let filename = match self {
            AgentProviderType::Claude => "claude_session.txt",
            AgentProviderType::OpenCode => "opencode_session.txt",
        };
        PathBuf::from(project_path)
            .join(".vstworkshop")
            .join(filename)
    }
}

/// Normalized stream events emitted to frontend
///
/// All providers normalize their output to these event types
#[derive(Clone, Serialize)]
#[serde(tag = "type")]
pub enum AgentStreamEvent {
    #[serde(rename = "start")]
    Start { project_path: String },

    #[serde(rename = "text")]
    Text { project_path: String, content: String },

    #[serde(rename = "tool_use")]
    ToolUse {
        project_path: String,
        tool: String,
        file: Option<String>,
        description: String,
    },

    #[serde(rename = "tool_result")]
    ToolResult { project_path: String, success: bool },

    #[serde(rename = "error")]
    Error { project_path: String, message: String },

    #[serde(rename = "done")]
    Done { project_path: String, content: String },
}

/// Response from sending a message to an agent provider
#[derive(Serialize, Clone)]
pub struct AgentResponse {
    pub content: String,
    pub session_id: Option<String>,
    pub commit_hash: Option<String>,
}

/// Configuration for a provider call
#[derive(Clone, Debug)]
pub struct ProviderCallConfig {
    pub project_path: String,
    pub project_name: String,
    pub description: String,
    pub message: String,
    pub model: Option<String>,
    pub custom_instructions: Option<String>,
    pub agent_verbosity: Option<String>,
    pub session_id: Option<String>,
    pub is_first_message: bool,
    pub components: Option<Vec<String>>,
    pub ui_framework: Option<String>,
}

/// Trait for agent provider implementations
///
/// Each provider (Claude, OpenCode, etc.) implements this trait to handle
/// their specific CLI arguments, JSON parsing, and session management.
pub trait AgentProvider: Send + Sync {
    /// Get the provider type
    fn provider_type(&self) -> AgentProviderType;

    /// Build CLI arguments for the provider
    fn build_args(&self, config: &ProviderCallConfig, context: &str) -> Vec<String>;

    /// Parse a line of streaming JSON output into a normalized event
    fn parse_stream_line(&self, line: &str, project_path: &str) -> ParsedLine;

    /// Extract session ID from a line if present
    fn extract_session_id(&self, line: &str) -> Option<String>;
}

/// Result of parsing a provider's output line
#[derive(Default)]
pub struct ParsedLine {
    /// Event to emit to frontend (if any)
    pub event: Option<AgentStreamEvent>,
    /// Assistant content for final message extraction
    pub assistant_content: Option<String>,
    /// Human-readable display text for streaming
    pub display_text: Option<String>,
}
