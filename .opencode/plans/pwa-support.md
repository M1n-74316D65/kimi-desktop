# Plan: Enable Le Chat PWA Support

## Goal
Make the Tauri webview properly support Le Chat's PWA (service worker, offline caching) instead of fighting against it with custom offline handling.

## Context
- Le Chat (chat.mistral.ai) is a full PWA with a web manifest, service worker, and offline capabilities
- The current app injects JS that detects load failures and forcibly navigates to a local offline page (`index.html`)
- This overrides the service worker's ability to serve cached content when offline
- Since the main window loads a real HTTPS URL (`https://chat.mistral.ai/chat`), WKWebView should support service workers natively

## Changes

### 1. Replace connectivity monitoring JS in `lib.rs` (lines 884-926)

**File**: `src-tauri/src/lib.rs`

Replace the existing connectivity JS block (the `connectivity_js` variable and its injection) with a PWA-aware version that:

1. Checks `navigator.serviceWorker.getRegistrations()` to see if Le Chat's SW is active
2. Logs the service worker status for debugging
3. If the page fails to load AND a service worker IS registered → reload (let SW serve cache)
4. If the page fails to load AND NO service worker → fall back to local offline page (first launch scenario)
5. Keep the online/offline event listeners for reload behavior

**Old code** (lines 884-926):
```rust
            // Inject connectivity monitoring and offline detection into the main window.
            // This JS checks if we're on an error page and handles online/offline events.
            if let Some(main_window) = app.get_webview_window("main") {
                let connectivity_js = r#"
                    (function() {
                        const CHAT_URL = 'https://chat.mistral.ai/chat';
                        
                        // Monitor online/offline events
                        window.addEventListener('offline', () => {
                            console.log('[Le Chat] Browser went offline');
                        });
                        window.addEventListener('online', () => {
                            console.log('[Le Chat] Browser came online — reloading');
                            if (window.location.href.includes('chat.mistral.ai')) {
                                window.location.reload();
                            } else {
                                window.location.href = CHAT_URL;
                            }
                        });
                        
                        // Check if page loaded successfully after a delay.
                        // If the page title contains error indicators or the body has no
                        // meaningful content, it likely failed to load.
                        setTimeout(() => {
                            const isErrorPage = !navigator.onLine 
                                || document.title.toLowerCase().includes('error')
                                || document.title.toLowerCase().includes('not found')
                                || document.title === ''
                                || (document.body && document.body.innerText.length < 50 
                                    && !document.querySelector('[data-sidebar]'));
                            
                            if (isErrorPage && !window.location.href.includes('tauri')) {
                                console.log('[Le Chat] Page appears to have failed loading, showing offline page');
                                // Navigate to local offline fallback
                                if (window.__TAURI__) {
                                    window.__TAURI__.core.invoke('navigate_to_offline').catch(() => {});
                                }
                            }
                        }, 5000);
                    })();
                "#;
                let _ = main_window.eval(connectivity_js);
            }
```

**New code**:
```rust
            // Inject PWA-aware connectivity monitoring into the main window.
            // Respects Le Chat's service worker for offline caching — only falls back
            // to the local offline page when no service worker is registered (first launch).
            if let Some(main_window) = app.get_webview_window("main") {
                let connectivity_js = r#"
                    (function() {
                        const CHAT_URL = 'https://chat.mistral.ai/chat';

                        // Log service worker status for debugging
                        async function checkServiceWorker() {
                            if (!('serviceWorker' in navigator)) {
                                console.log('[Le Chat] Service workers not supported in this webview');
                                return { supported: false, registered: false };
                            }
                            try {
                                const registrations = await navigator.serviceWorker.getRegistrations();
                                const hasSW = registrations.length > 0;
                                console.log('[Le Chat] Service worker supported: true, registered:', hasSW,
                                    hasSW ? '(PWA active — offline handled by service worker)' : '(no PWA cache yet)');
                                for (const reg of registrations) {
                                    console.log('[Le Chat]   SW scope:', reg.scope, 'state:', 
                                        reg.active ? 'active' : reg.installing ? 'installing' : reg.waiting ? 'waiting' : 'unknown');
                                }
                                return { supported: true, registered: hasSW };
                            } catch (e) {
                                console.log('[Le Chat] Service worker check failed:', e.message);
                                return { supported: true, registered: false };
                            }
                        }

                        // Monitor online/offline events
                        window.addEventListener('offline', () => {
                            console.log('[Le Chat] Browser went offline');
                        });
                        window.addEventListener('online', () => {
                            console.log('[Le Chat] Browser came online — reloading');
                            if (window.location.href.includes('chat.mistral.ai')) {
                                window.location.reload();
                            } else {
                                window.location.href = CHAT_URL;
                            }
                        });

                        // Check if page loaded successfully after a delay.
                        // Only fall back to the local offline page if the PWA service worker
                        // is NOT registered (i.e., first launch with no cached content).
                        // If the service worker IS registered, let it handle offline gracefully.
                        setTimeout(async () => {
                            const isErrorPage = !navigator.onLine
                                || document.title.toLowerCase().includes('error')
                                || document.title.toLowerCase().includes('not found')
                                || document.title === ''
                                || (document.body && document.body.innerText.length < 50
                                    && !document.querySelector('[data-sidebar]'));

                            if (!isErrorPage || window.location.href.includes('tauri')) {
                                // Page loaded fine or we're already on a local page
                                await checkServiceWorker();
                                return;
                            }

                            // Page failed to load — check if the PWA can handle it
                            const sw = await checkServiceWorker();
                            if (sw.registered) {
                                // Service worker is active — let the PWA handle offline.
                                // Just reload to give the service worker a chance to serve cached content.
                                console.log('[Le Chat] Page failed but PWA service worker is active — reloading to use cache');
                                window.location.reload();
                            } else {
                                // No service worker yet (first launch or SW not installed).
                                // Fall back to local offline page.
                                console.log('[Le Chat] Page failed and no service worker — showing offline page');
                                if (window.__TAURI__) {
                                    window.__TAURI__.core.invoke('navigate_to_offline').catch(() => {});
                                }
                            }
                        }, 5000);
                    })();
                "#;
                let _ = main_window.eval(connectivity_js);
            }
```

### 2. No changes to `navigate_to_offline` command

The command at lines 554-568 stays as-is. It's still needed as a fallback for first-launch-without-internet scenarios.

### 3. No changes to `navigate_to_chat` command

Stays as-is at lines 570-577.

### 4. No changes to `index.html`

The offline fallback page stays as-is. It's still useful for the first-launch-no-internet case.

### 5. Verification

After making changes:
```bash
cd src-tauri && cargo check
cd src-tauri && cargo test
cd src-tauri && cargo clippy
```

## What this achieves

- **PWA service worker is respected**: After the first successful load of chat.mistral.ai, its service worker will cache assets. Subsequent offline launches will be served from the SW cache instead of being forcibly redirected to a bare offline page.
- **First-launch safety**: If the user has never loaded Le Chat before (no SW cache), the local offline page still kicks in as a fallback.
- **Debug visibility**: Console logs show whether the service worker is active, making it easy to verify PWA behavior in dev tools.
- **No breaking changes**: All existing commands and the invoke_handler stay the same.

## Risks

- **Infinite reload loop**: If the service worker IS registered but its cache is stale/broken, the new code reloads once. To prevent a loop, we should add a reload guard (e.g., a sessionStorage flag). **This should be added to the implementation.**
- **WKWebView SW support**: Service workers for external HTTPS sites should work in WKWebView, but if they don't, the behavior degrades gracefully to the old offline page.
