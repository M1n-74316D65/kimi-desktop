//! Wrapper configuration constants

/// DOM selectors used across wrapper injections
pub struct Selectors;

impl Selectors {
    pub const CHAT_INPUT: &str = ".chat-input-editor";
    pub const CONTENTEDITABLE: &str = "div[contenteditable=\"true\"]";
    pub const TEXTAREA_ASK: &str = "textarea[placeholder*=\"Ask\"]";
    pub const TEXTAREA_MESSAGE: &str = "textarea[placeholder*=\"Message\"]";
    pub const TEXTAREA_LOWERCASE_ASK: &str = "textarea[placeholder*=\"ask\"]";
    pub const TEXTAREA_TESTID: &str = "textarea[data-testid]";
    pub const TEXTAREA_ANY: &str = "textarea";
    pub const SEND_BUTTON: &str = ".send-button-container:not(.disabled)";
    pub const SEND_BUTTON_FALLBACK: &str = ".send-button-container";
    pub const SUBMIT_BUTTON: &str = "button[type=\"submit\"]";
    pub const SEND_ARIA_LABEL: &str = "button[aria-label*=\"send\" i]";
    pub const SEND_ARIA_LABEL_CAP: &str = "button[aria-label*=\"Send\" i]";
    pub const SEND_DATA_TESTID: &str = "button[data-testid*=\"send\" i]";
    pub const FORM_BUTTON_LAST: &str = "form button:last-of-type";
    pub const STOP_BUTTON_ARIA: &str = "button[aria-label*=\"stop\" i]";
    pub const STOP_BUTTON_ARIA_CAP: &str = "button[aria-label*=\"Stop\" i]";
    pub const CANCEL_BUTTON_ARIA: &str = "button[aria-label*=\"cancel\" i]";
    pub const STOP_BUTTON_DATA_TESTID: &str = "button[data-testid*=\"stop\" i]";
    pub const DATA_SIDEBAR: &str = "[data-sidebar]";
}

/// Timeout values in milliseconds
pub struct Timeouts;

impl Timeouts {
    pub const INJECTION_TOTAL: u64 = 8000;
    pub const RETRY_DELAY: u64 = 300;
    pub const PAGE_LOAD_WAIT: u64 = 800;
    pub const BOT_PAGE_LOAD_WAIT: u64 = 1500;
    pub const WINDOW_VISIBLE_DELAY: u64 = 100;
    pub const REACT_INIT_DELAY: u64 = 200;
    pub const FOCUS_DELAY: u64 = 50;
    pub const SUBMIT_DELAY: u64 = 300;
    pub const FALLBACK_LOAD_TIMEOUT: u64 = 500;
    pub const CONNECTIVITY_CHECK_DELAY: u64 = 5000;
    pub const RESPONSE_WATCHER_INTERVAL: u64 = 500;
    pub const RESPONSE_WATCHER_INITIAL_DELAY: u64 = 2000;
    pub const RESPONSE_WATCHER_MAX_CHECKS: u32 = 600;
    pub const INJECTION_MAX_RETRIES: u32 = 15;
}

/// Application URLs
pub struct Urls;

impl Urls {
    pub const CHAT: &str = "https://www.kimi.com/";
    pub const BOT: &str = "https://www.kimi.com/bot";
}

/// Style injection configuration
pub struct Styles;

impl Styles {
    pub const STYLE_ID: &str = "kimi-custom-styles";
    pub const HEADER_PADDING: &str = "2.5rem";
}

/// Storage keys
pub struct Storage;

impl Storage {
    pub const RELOAD_GUARD_KEY: &str = "__kimi_sw_reload";
}
