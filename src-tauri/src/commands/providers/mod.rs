//! Agent provider implementations
//!
//! Each provider module implements the AgentProvider trait for a specific CLI tool.

pub mod claude_cli;
pub mod opencode;

use super::agent_provider::{AgentProvider, AgentProviderType};

/// Get the provider implementation for a given type
pub fn get_provider(provider_type: AgentProviderType) -> Box<dyn AgentProvider> {
    match provider_type {
        AgentProviderType::Claude => Box::new(claude_cli::ClaudeProvider),
        AgentProviderType::OpenCode => Box::new(opencode::OpenCodeProvider),
    }
}
