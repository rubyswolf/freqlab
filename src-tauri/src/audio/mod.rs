//! Audio engine for VST preview system
//!
//! Provides real-time audio playback with:
//! - Test signal generation (sine, noise, sweep, etc.)
//! - Sample playback via Symphonia
//! - Live audio input capture
//! - CLAP plugin hosting with hot reload

pub mod buffer;
pub mod device;
pub mod engine;
pub mod input;
pub mod plugin;
pub mod samples;
pub mod signals;
pub mod spectrum;
