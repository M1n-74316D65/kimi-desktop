//! Webview wrapper utilities for JS injection
//!
//! This module provides centralized configuration and utilities for injecting
//! JavaScript into the main webview window.

pub mod config;
pub mod error;
pub mod extension;
pub mod templates;
pub mod utils;

// Re-export commonly used items
pub use config::*;
pub use error::{WrapperError, WrapperResult};
pub use extension::{
    apply_all_wrappers, emit_launcher_shown, emit_settings_changed, set_offline_state,
    submit_chat_message, WrapperExt,
};
pub use templates::*;
pub use utils::{build_js, escape_js};
