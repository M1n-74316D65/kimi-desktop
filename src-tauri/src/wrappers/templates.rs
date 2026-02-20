//! JavaScript template strings for wrapper injections

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
