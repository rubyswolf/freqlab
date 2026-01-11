//! Claude Code skills for per-project agents
//!
//! Each skill is a markdown file with frontmatter that Claude Code recognizes
//! as a slash command. Skills are generated per-project based on configuration.

pub mod components;
pub mod core;
pub mod plugin_types;
pub mod ui_frameworks;

// Re-export skill constants for easy access
pub use components::get_component_skill;
pub use core::{DSP_SAFETY, NIH_PLUG_BASICS};
pub use plugin_types::{EFFECT_PATTERNS, INSTRUMENT_PATTERNS};
pub use ui_frameworks::{EGUI_UI, NATIVE_UI, WEBVIEW_UI};
