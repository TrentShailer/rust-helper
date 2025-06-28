//! Various helper functions, structures, and traits for working on my Rust projects.
//!

#[cfg(feature = "command")]
pub mod basic_command;
#[cfg(feature = "config")]
pub mod config;
#[cfg(feature = "config-command")]
pub mod config_command;
pub mod error;
#[cfg(feature = "json")]
pub mod json;
