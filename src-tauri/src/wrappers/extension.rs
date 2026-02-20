//! WebviewWindow extension trait for wrapper operations

use tauri::{AppHandle, Emitter, WebviewWindow};

use crate::wrappers::{
    config,
    error::{WrapperError, WrapperResult},
    templates::{
        CONNECTIVITY_JS, INJECT_MESSAGE_JS, LINK_INTERCEPTOR_JS, RESPONSE_WATCHER_JS,
        TITLEBAR_OVERLAP_JS,
    },
    utils::build_js,
};

/// Extension trait for WebviewWindow to provide wrapper injection methods
pub trait WrapperExt {
    /// Inject titlebar overlap styles (macOS only)
    fn inject_titlebar_styles(&self) -> WrapperResult<()>;

    /// Inject a message into the chat input
    fn inject_message(&self, message: &str) -> WrapperResult<()>;

    /// Inject response watcher for notification handling
    fn inject_response_watcher(&self) -> WrapperResult<()>;

    /// Inject connectivity monitoring
    fn inject_connectivity(&self) -> WrapperResult<()>;

    /// Inject external link interceptor
    fn inject_link_interceptor(&self) -> WrapperResult<()>;
}

impl WrapperExt for WebviewWindow {
    fn inject_titlebar_styles(&self) -> WrapperResult<()> {
        let js = build_js(
            TITLEBAR_OVERLAP_JS,
            &[
                ("style_id", config::Styles::STYLE_ID),
                ("header_padding", config::Styles::HEADER_PADDING),
            ],
        );
        self.eval(&js).map_err(|e| WrapperError::InjectionFailed {
            context: "titlebar styles".to_string(),
            source: e.to_string(),
        })
    }

    fn inject_message(&self, message: &str) -> WrapperResult<()> {
        use crate::wrappers::utils::escape_js;
        use config::*;

        let js = build_js(
            INJECT_MESSAGE_JS,
            &[
                ("message", &escape_js(message)),
                ("max_retries", &Timeouts::INJECTION_MAX_RETRIES.to_string()),
                ("retry_delay", &Timeouts::RETRY_DELAY.to_string()),
                ("total_timeout", &Timeouts::INJECTION_TOTAL.to_string()),
                (
                    "fallback_timeout",
                    &Timeouts::FALLBACK_LOAD_TIMEOUT.to_string(),
                ),
                ("react_init_delay", &Timeouts::REACT_INIT_DELAY.to_string()),
                ("focus_delay", &Timeouts::FOCUS_DELAY.to_string()),
                ("submit_delay", &Timeouts::SUBMIT_DELAY.to_string()),
                ("selector_chat_input", Selectors::CHAT_INPUT),
                ("selector_contenteditable", Selectors::CONTENTEDITABLE),
                ("selector_textarea_ask", Selectors::TEXTAREA_ASK),
                ("selector_textarea_message", Selectors::TEXTAREA_MESSAGE),
                (
                    "selector_textarea_lowercase_ask",
                    Selectors::TEXTAREA_LOWERCASE_ASK,
                ),
                ("selector_textarea_testid", Selectors::TEXTAREA_TESTID),
                ("selector_textarea_any", Selectors::TEXTAREA_ANY),
                ("selector_send_button", Selectors::SEND_BUTTON),
                (
                    "selector_send_button_fallback",
                    Selectors::SEND_BUTTON_FALLBACK,
                ),
                ("selector_submit_button", Selectors::SUBMIT_BUTTON),
                ("selector_send_aria", Selectors::SEND_ARIA_LABEL),
                ("selector_send_aria_cap", Selectors::SEND_ARIA_LABEL_CAP),
                ("selector_send_data_testid", Selectors::SEND_DATA_TESTID),
                ("selector_form_button_last", Selectors::FORM_BUTTON_LAST),
            ],
        );
        self.eval(&js).map_err(|e| WrapperError::InjectionFailed {
            context: "chat message".to_string(),
            source: e.to_string(),
        })
    }

    fn inject_response_watcher(&self) -> WrapperResult<()> {
        use config::*;

        let js = build_js(
            RESPONSE_WATCHER_JS,
            &[
                (
                    "check_interval",
                    &Timeouts::RESPONSE_WATCHER_INTERVAL.to_string(),
                ),
                (
                    "initial_delay",
                    &Timeouts::RESPONSE_WATCHER_INITIAL_DELAY.to_string(),
                ),
                (
                    "max_checks",
                    &Timeouts::RESPONSE_WATCHER_MAX_CHECKS.to_string(),
                ),
                ("selector_stop_aria", Selectors::STOP_BUTTON_ARIA),
                ("selector_stop_aria_cap", Selectors::STOP_BUTTON_ARIA_CAP),
                ("selector_cancel_aria", Selectors::CANCEL_BUTTON_ARIA),
                (
                    "selector_stop_data_testid",
                    Selectors::STOP_BUTTON_DATA_TESTID,
                ),
            ],
        );
        self.eval(&js).map_err(|e| WrapperError::InjectionFailed {
            context: "response watcher".to_string(),
            source: e.to_string(),
        })
    }

    fn inject_connectivity(&self) -> WrapperResult<()> {
        use config::*;

        let js = build_js(
            CONNECTIVITY_JS,
            &[
                ("chat_url", Urls::CHAT),
                ("reload_key", Storage::RELOAD_GUARD_KEY),
                ("selector_data_sidebar", Selectors::DATA_SIDEBAR),
                (
                    "connectivity_check_delay",
                    &Timeouts::CONNECTIVITY_CHECK_DELAY.to_string(),
                ),
            ],
        );
        self.eval(&js).map_err(|e| WrapperError::InjectionFailed {
            context: "connectivity monitor".to_string(),
            source: e.to_string(),
        })
    }

    fn inject_link_interceptor(&self) -> WrapperResult<()> {
        self.eval(LINK_INTERCEPTOR_JS)
            .map_err(|e| WrapperError::InjectionFailed {
                context: "link interceptor".to_string(),
                source: e.to_string(),
            })
    }
}

/// Apply all standard wrappers to the main window
pub fn apply_all_wrappers(window: &WebviewWindow) {
    // These all fail silently as requested
    let _ = window.inject_connectivity();
    let _ = window.inject_link_interceptor();
}

/// Submit a message to the chat window with all necessary injections
pub fn submit_chat_message(window: &WebviewWindow, message: &str) -> WrapperResult<()> {
    window.inject_message(message)?;
    window.inject_response_watcher()
}

/// Set offline state in the main window
pub fn set_offline_state(window: &WebviewWindow) -> WrapperResult<()> {
    window
        .eval("document.getElementById('main-container').className = 'container offline';")
        .map_err(|e| WrapperError::InjectionFailed {
            context: "offline state".to_string(),
            source: e.to_string(),
        })
}

/// Emit launcher shown event to the launcher window
pub fn emit_launcher_shown(app: &AppHandle) {
    let _ = app.emit("launcher-shown", ());
}

/// Emit settings changed event
pub fn emit_settings_changed(app: &AppHandle, settings: &crate::AppSettings) {
    let _ = app.emit("settings-changed", settings);
}
