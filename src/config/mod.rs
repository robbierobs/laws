//! Configuration module
//!
//! Contains centralized configuration management and CLI argument parsing.

pub mod configuration;
pub mod args;

pub use configuration::AppConfig;
pub use args::Args;
