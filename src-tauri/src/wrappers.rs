//! Webview wrapper utilities for JS injection
//!
//! This module provides centralized configuration and utilities for injecting
//! JavaScript into the main webview window.

use tauri::{AppHandle, Emitter, WebviewWindow};

//============================================================================
// CONFIGURATION
//============================================================================

/// Centralized configuration for all wrapper functionality
pub mod config {
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
}

//============================================================================
// UTILITIES
//============================================================================

/// Escape a string for safe use in JavaScript template literals
///
/// Escapes: backslashes, backticks, dollar signs, newlines, carriage returns
pub fn escape_js(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace('$', "\\$")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// Build a JavaScript template by replacing placeholders
///
/// Placeholders are in the format `{{key}}`
pub fn build_js(template: &str, vars: &[(&str, &str)]) -> String {
    vars.iter().fold(template.to_string(), |acc, (key, val)| {
        acc.replace(&format!("{{{{{}}}}}", key), val)
    })
}

//============================================================================
// JAVASCRIPT TEMPLATES
//============================================================================

/// JavaScript template for hiding titlebar overlap on macOS
pub const TITLEBAR_OVERLAP_JS: &str = r#"
(function() {
    const STYLE_ID = '{{style_id}}';
    
    function injectStyles() {
        if (document.getElementById(STYLE_ID)) return;
        
        const style = document.createElement('style');
        style.id = STYLE_ID;
        style.textContent = `
            .app-header > button:first-child,
            header button:first-child {
                display: none !important;
            }
            .app-header,
            header {
                padding-top: {{header_padding}} !important;
            }
        `;
        document.head.appendChild(style);
    }
    
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', injectStyles);
    } else {
        injectStyles();
    }
    
    new MutationObserver(() => injectStyles()).observe(
        document.documentElement, 
        { childList: true, subtree: true }
    );
})();
"#;

/// JavaScript template for injecting messages into chat input
pub const INJECT_MESSAGE_JS: &str = r#"
(function() {
    const message = `{{message}}`;
    const maxRetries = {{max_retries}};
    const retryDelay = {{retry_delay}};
    const totalTimeout = {{total_timeout}};
    let retryCount = 0;
    let timedOut = false;
    
    function emitResult(success, error) {
        if (window.__TAURI__) {
            window.__TAURI__.event.emit('inject-result', { success, error: error || null });
        }
    }
    
    const timeoutId = setTimeout(() => {
        timedOut = true;
        const msg = 'Message injection timed out after ' + totalTimeout + 'ms';
        emitResult(false, msg);
    }}, totalTimeout);
    
    function findTextarea() {
        return document.querySelector('{{selector_chat_input}}')
            || document.querySelector('{{selector_contenteditable}}')
            || document.querySelector('{{selector_textarea_ask}}')
            || document.querySelector('{{selector_textarea_message}}')
            || document.querySelector('{{selector_textarea_lowercase_ask}}')
            || document.querySelector('{{selector_textarea_testid}}')
            || document.querySelector('{{selector_textarea_any}}');
    }
    
    async function injectMessage() {
        if (timedOut) return;
        
        if (document.readyState !== 'complete') {
            await new Promise(resolve => {
                window.addEventListener('load', resolve, { once: true });
                setTimeout(resolve, {{fallback_timeout}});
            });
        }
        
        await new Promise(r => setTimeout(r, {{react_init_delay}}));
        
        const textarea = findTextarea();
        
        if (!textarea) {
            retryCount++;
            if (retryCount < maxRetries) {
                setTimeout(injectMessage, retryDelay);
                return;
            }
            clearTimeout(timeoutId);
            emitResult(false, 'Could not find chat input after ' + maxRetries + ' attempts');
            return;
        }
        
        try {
            if (textarea.classList && textarea.classList.contains('chat-input-editor')) {
                textarea.contentEditable = 'true';
                textarea.focus();
                await new Promise(r => setTimeout(r, {{focus_delay}}));
                textarea.innerHTML = '';
                document.execCommand('insertText', false, message);
                textarea.dispatchEvent(new Event('input', { bubbles: true }));
                setTimeout(() => submitForm(textarea), {{submit_delay}});
                return;
            }
            
            if (textarea.contentEditable === 'true') {
                textarea.focus();
                textarea.innerHTML = '';
                const textNode = document.createTextNode(message);
                textarea.appendChild(textNode);
                
                const range = document.createRange();
                range.selectNodeContents(textarea);
                range.collapse(false);
                const selection = window.getSelection();
                selection.removeAllRanges();
                selection.addRange(range);
                
                textarea.dispatchEvent(new InputEvent('input', {
                    bubbles: true,
                    inputType: 'insertText',
                    data: message
                }));
                
                setTimeout(() => submitForm(textarea), {{submit_delay}});
                return;
            }
            
            const nativeInputValueSetter = Object.getOwnPropertyDescriptor(
                window.HTMLTextAreaElement.prototype, 'value'
            ).set;
            
            nativeInputValueSetter.call(textarea, message);
            textarea.dispatchEvent(new Event('input', { bubbles: true }));
            textarea.dispatchEvent(new Event('change', { bubbles: true }));
            textarea.focus();
            setTimeout(() => submitForm(textarea), {{submit_delay}});
        } catch (err) {
            clearTimeout(timeoutId);
            emitResult(false, 'Failed to set message: ' + err.message);
        }
    }
    
    let submitted = false;
    
    function submitForm(textarea) {
        if (timedOut || submitted) return;
        submitted = true;
        clearTimeout(timeoutId);
        
        const sendBtn = document.querySelector('{{selector_send_button}}')
            || document.querySelector('{{selector_send_button_fallback}}')
            || document.querySelector('{{selector_submit_button}}')
            || document.querySelector('{{selector_send_aria}}')
            || document.querySelector('{{selector_send_aria_cap}}')
            || document.querySelector('{{selector_send_data_testid}}')
            || document.querySelector('{{selector_form_button_last}}');
        
        if (sendBtn) {
            sendBtn.click();
            emitResult(true);
        } else {
            if (textarea) {
                textarea.dispatchEvent(new KeyboardEvent('keydown', {
                    key: 'Enter',
                    code: 'Enter',
                    keyCode: 13,
                    which: 13,
                    bubbles: true,
                    cancelable: true
                }));
            }
            emitResult(true);
        }
    }
    
    injectMessage().catch(err => {
        emitResult(false, err.message);
    });
})();
"#;

/// JavaScript template for watching AI response completion
pub const RESPONSE_WATCHER_JS: &str = r#"
(function() {
    if (window.__kimiResponseWatcher) return;
    window.__kimiResponseWatcher = true;
    
    const CHECK_INTERVAL = {{check_interval}};
    const INITIAL_DELAY = {{initial_delay}};
    let wasStreaming = false;
    let checkCount = 0;
    const MAX_CHECKS = {{max_checks}};
    
    function isStreaming() {
        const stopBtn = document.querySelector('{{selector_stop_aria}}')
            || document.querySelector('{{selector_stop_aria_cap}}')
            || document.querySelector('{{selector_cancel_aria}}')
            || document.querySelector('{{selector_stop_data_testid}}');
        return !!stopBtn;
    }
    
    setTimeout(() => {
        const intervalId = setInterval(() => {
            checkCount++;
            
            if (checkCount > MAX_CHECKS) {
                clearInterval(intervalId);
                window.__kimiResponseWatcher = false;
                return;
            }
            
            const streaming = isStreaming();
            
            if (streaming) {
                wasStreaming = true;
            }
            
            if (wasStreaming && !streaming) {
                clearInterval(intervalId);
                window.__kimiResponseWatcher = false;
                if (window.__TAURI__) {
                    window.__TAURI__.event.emit('response-complete', {});
                }
            }
        }, CHECK_INTERVAL);
    }, INITIAL_DELAY);
})();
"#;

/// JavaScript template for PWA connectivity monitoring
pub const CONNECTIVITY_JS: &str = r#"
(function() {
    const CHAT_URL = '{{chat_url}}';
    const RELOAD_KEY = '{{reload_key}}';
    
    async function checkServiceWorker() {
        if (!('serviceWorker' in navigator)) {
            return { supported: false, registered: false };
        }
        try {
            const registrations = await navigator.serviceWorker.getRegistrations();
            const hasSW = registrations.length > 0;
            return { supported: true, registered: hasSW };
        } catch (e) {
            return { supported: true, registered: false };
        }
    }
    
    window.addEventListener('offline', () => {
        console.log('[Kimi] Browser went offline');
    });
    
    window.addEventListener('online', () => {
        if (window.location.href.includes('www.kimi.com')) {
            window.location.reload();
        } else {
            window.location.href = CHAT_URL;
        }
    });
    
    setTimeout(async () => {
        const isErrorPage = !navigator.onLine
            || document.title.toLowerCase().includes('error')
            || document.title.toLowerCase().includes('not found')
            || document.title === ''
            || (document.body && document.body.innerText.length < 50
                && !document.querySelector('{{selector_data_sidebar}}'));
        
        if (!isErrorPage || window.location.href.includes('tauri')) {
            await checkServiceWorker();
            return;
        }
        
        const sw = await checkServiceWorker();
        if (sw.registered) {
            if (sessionStorage.getItem(RELOAD_KEY)) {
                sessionStorage.removeItem(RELOAD_KEY);
                if (window.__TAURI__) {
                    window.__TAURI__.core.invoke('navigate_to_offline').catch(() => {});
                }
            } else {
                sessionStorage.setItem(RELOAD_KEY, '1');
                window.location.reload();
            }
        } else {
            if (window.__TAURI__) {
                window.__TAURI__.core.invoke('navigate_to_offline').catch(() => {});
            }
        }
    }, {{connectivity_check_delay}});
})();
"#;

/// JavaScript template for intercepting external links
pub const LINK_INTERCEPTOR_JS: &str = r#"
(function() {
    if (window.__kimiLinkInterceptor) return;
    window.__kimiLinkInterceptor = true;
    
    document.addEventListener('click', function(e) {
        const link = e.target.closest('a[href]');
        if (!link) return;
        
        const href = link.getAttribute('href');
        if (!href) return;
        
        const isExternal = href.startsWith('http') && !href.includes('kimi.com') && !href.includes('moonshot.cn');
        const isMailto = href.startsWith('mailto:');
        
        if (isExternal || isMailto) {
            e.preventDefault();
            e.stopPropagation();
            if (window.__TAURI__) {
                window.__TAURI__.core.invoke('open_external_link', { url: href }).catch(() => {});
            }
        }
    }, true);
})();
"#;

//============================================================================
// WINDOW EXTENSION TRAIT
//============================================================================

/// Extension trait for WebviewWindow to provide wrapper injection methods
pub trait WrapperExt {
    /// Inject titlebar overlap styles (macOS only)
    fn inject_titlebar_styles(&self) -> Result<(), ()>;

    /// Inject a message into the chat input
    fn inject_message(&self, message: &str) -> Result<(), ()>;

    /// Inject response watcher for notification handling
    fn inject_response_watcher(&self) -> Result<(), ()>;

    /// Inject connectivity monitoring
    fn inject_connectivity(&self) -> Result<(), ()>;

    /// Inject external link interceptor
    fn inject_link_interceptor(&self) -> Result<(), ()>;
}

impl WrapperExt for WebviewWindow {
    fn inject_titlebar_styles(&self) -> Result<(), ()> {
        let js = build_js(
            TITLEBAR_OVERLAP_JS,
            &[
                ("style_id", config::Styles::STYLE_ID),
                ("header_padding", config::Styles::HEADER_PADDING),
            ],
        );
        self.eval(&js).map_err(|_| ())
    }

    fn inject_message(&self, message: &str) -> Result<(), ()> {
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
        self.eval(&js).map_err(|_| ())
    }

    fn inject_response_watcher(&self) -> Result<(), ()> {
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
        self.eval(&js).map_err(|_| ())
    }

    fn inject_connectivity(&self) -> Result<(), ()> {
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
        self.eval(&js).map_err(|_| ())
    }

    fn inject_link_interceptor(&self) -> Result<(), ()> {
        self.eval(LINK_INTERCEPTOR_JS).map_err(|_| ())
    }
}

//============================================================================
// HIGH-LEVEL WRAPPER API
//============================================================================

/// Apply all standard wrappers to the main window
pub fn apply_all_wrappers(window: &WebviewWindow) {
    // These all fail silently as requested
    let _ = window.inject_connectivity();
    let _ = window.inject_link_interceptor();
}

/// Submit a message to the chat window with all necessary injections
pub fn submit_chat_message(window: &WebviewWindow, message: &str) -> Result<(), ()> {
    window.inject_message(message)?;
    window.inject_response_watcher()
}

/// Set offline state in the main window
pub fn set_offline_state(window: &WebviewWindow) -> Result<(), ()> {
    window
        .eval("document.getElementById('main-container').className = 'container offline';")
        .map_err(|_| ())
}

/// Emit launcher shown event to the launcher window
pub fn emit_launcher_shown(app: &AppHandle) {
    let _ = app.emit("launcher-shown", ());
}

/// Emit settings changed event
pub fn emit_settings_changed(app: &AppHandle, settings: &crate::AppSettings) {
    let _ = app.emit("settings-changed", settings);
}
