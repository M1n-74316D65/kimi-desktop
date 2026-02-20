#[cfg(target_os = "macos")]
use tauri::TitleBarStyle;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Listener, Manager, RunEvent,
};

#[cfg(target_os = "macos")]
#[allow(deprecated)]
use cocoa::appkit::{NSColor, NSWindow};
#[cfg(target_os = "macos")]
#[allow(deprecated)]
use cocoa::base::{id, nil, NO};
#[cfg(target_os = "macos")]
use objc::{msg_send, sel, sel_impl};

use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

mod wrappers;
use wrappers::{WrapperExt, apply_all_wrappers, submit_chat_message, set_offline_state, emit_launcher_shown, emit_settings_changed};
use wrappers::config::Urls;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppSettings {
    pub new_chat_default: bool,
    pub notifications_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            new_chat_default: true,
            notifications_enabled: true,
        }
    }
}

#[tauri::command]
async fn hide_launcher(app: AppHandle) -> Result<(), String> {
    if let Some(launcher) = app.get_webview_window("launcher") {
        launcher.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn show_launcher(app: AppHandle) -> Result<(), String> {
    if let Some(launcher) = app.get_webview_window("launcher") {
        launcher.center().map_err(|e| e.to_string())?;
        launcher.show().map_err(|e| e.to_string())?;
        launcher.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn toggle_launcher(app: AppHandle) -> Result<(), String> {
    if let Some(launcher) = app.get_webview_window("launcher") {
        let is_visible = launcher.is_visible().unwrap_or(false);
        if is_visible {
            launcher.hide().map_err(|e| e.to_string())?;
        } else {
            launcher.center().map_err(|e| e.to_string())?;
            launcher.show().map_err(|e| e.to_string())?;
            launcher.set_focus().map_err(|e| e.to_string())?;
            emit_launcher_shown(&app);
        }
    }
    Ok(())
}

#[tauri::command]
async fn show_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(main_window) = app.get_webview_window("main") {
        main_window.show().map_err(|e| e.to_string())?;
        main_window.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn navigate_to_offline(app: AppHandle) -> Result<(), String> {
    if let Some(main_window) = app.get_webview_window("main") {
        let url = "tauri://localhost/index.html"
            .parse::<tauri::Url>()
            .map_err(|e| e.to_string())?;
        main_window.navigate(url).map_err(|e| e.to_string())?;
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        let _ = set_offline_state(&main_window);
    }
    Ok(())
}

#[tauri::command]
async fn navigate_to_chat(app: AppHandle) -> Result<(), String> {
    if let Some(main_window) = app.get_webview_window("main") {
        let url = Urls::CHAT.parse::<tauri::Url>().map_err(|e| e.to_string())?;
        main_window.navigate(url).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
async fn submit_message(app: AppHandle, message: String, new_chat: bool, bot_mode: bool) -> Result<(), String> {
    use wrappers::config::Timeouts;
    
    if let Some(launcher) = app.get_webview_window("launcher") {
        launcher.hide().map_err(|e| e.to_string())?;
    }

    if let Some(main_window) = app.get_webview_window("main") {
        main_window.show().map_err(|e| e.to_string())?;
        main_window.set_focus().map_err(|e| e.to_string())?;

        if bot_mode {
            let url = Urls::BOT.parse::<tauri::Url>().map_err(|e| e.to_string())?;
            main_window.navigate(url).map_err(|e| e.to_string())?;
            tokio::time::sleep(std::time::Duration::from_millis(Timeouts::BOT_PAGE_LOAD_WAIT)).await;
        } else if new_chat {
            let url = Urls::CHAT.parse::<tauri::Url>().map_err(|e| e.to_string())?;
            main_window.navigate(url).map_err(|e| e.to_string())?;
            tokio::time::sleep(std::time::Duration::from_millis(Timeouts::PAGE_LOAD_WAIT)).await;
        } else {
            tokio::time::sleep(std::time::Duration::from_millis(Timeouts::WINDOW_VISIBLE_DELAY)).await;
        }

        let _ = submit_chat_message(&main_window, &message);
    }

    Ok(())
}

#[tauri::command]
async fn get_settings(app: AppHandle) -> Result<AppSettings, String> {
    use tauri_plugin_store::StoreExt;
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let settings = match store.get("app_settings") {
        Some(value) => serde_json::from_value(value).unwrap_or_default(),
        None => AppSettings::default(),
    };
    Ok(settings)
}

#[tauri::command]
async fn save_settings(app: AppHandle, settings: AppSettings) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let value = serde_json::to_value(&settings).map_err(|e| e.to_string())?;
    store.set("app_settings", value);
    store.save().map_err(|e| e.to_string())?;
    emit_settings_changed(&app, &settings);
    Ok(())
}

#[tauri::command]
async fn show_settings(app: AppHandle) -> Result<(), String> {
    if let Some(settings) = app.get_webview_window("settings") {
        settings.show().map_err(|e| e.to_string())?;
        settings.set_focus().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Validates that a URL is safe to open externally
/// Only allows http, https, and mailto schemes
fn validate_external_url(url: &str) -> Result<(), String> {
    // Check for javascript: URLs (XSS prevention)
    if url.trim().to_lowercase().starts_with("javascript:") {
        return Err("JavaScript URLs are not allowed".to_string());
    }

    // Check for data: URLs (potential security risk)
    if url.trim().to_lowercase().starts_with("data:") {
        return Err("Data URLs are not allowed".to_string());
    }

    // Check for file: URLs (local file access)
    if url.trim().to_lowercase().starts_with("file:") {
        return Err("File URLs are not allowed".to_string());
    }

    // Check for other dangerous schemes
    let dangerous_schemes = ["vbscript:", "mhtml:", "x-javascript:"];
    let url_lower = url.trim().to_lowercase();
    for scheme in &dangerous_schemes {
        if url_lower.starts_with(scheme) {
            return Err(format!("{} URLs are not allowed", scheme.trim_end_matches(':')));
        }
    }

    // Allow http, https, and mailto
    let allowed_schemes = ["http://", "https://", "mailto:"];
    let has_allowed_scheme = allowed_schemes
        .iter()
        .any(|scheme| url_lower.starts_with(scheme));

    if !has_allowed_scheme {
        return Err(
            "URL must use http://, https://, or mailto: scheme".to_string(),
        );
    }

    Ok(())
}

#[tauri::command]
async fn open_external_link(url: String) -> Result<(), String> {
    // Validate URL before opening
    validate_external_url(&url)?;

    tauri_plugin_opener::open_url(&url, None::<&str>).map_err(|e| e.to_string())
}

fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "Show Kimi", true, None::<&str>)?;
    let launcher_item = MenuItem::with_id(app, "launcher", "Quick Ask...", true, None::<&str>)?;
    let separator1 = PredefinedMenuItem::separator(app)?;
    let settings_item = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    let separator2 = PredefinedMenuItem::separator(app)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &show_item,
            &launcher_item,
            &separator1,
            &settings_item,
            &separator2,
            &quit_item,
        ],
    )?;

    let _tray = TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "launcher" => {
                if let Some(launcher) = app.get_webview_window("launcher") {
                    let _ = launcher.center();
                    let _ = launcher.show();
                    let _ = launcher.set_focus();
                }
            }
            "settings" => {
                if let Some(settings) = app.get_webview_window("settings") {
                    let _ = settings.show();
                    let _ = settings.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn setup_global_shortcut(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let shortcut = Shortcut::new(Some(Modifiers::ALT), Code::Space);
    let app_handle = app.clone();

    app.global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                if let Some(launcher) = app_handle.get_webview_window("launcher") {
                    let is_visible = launcher.is_visible().unwrap_or(false);
                    if is_visible {
                        let _ = launcher.hide();
                    } else {
                        let _ = launcher.center();
                        let _ = launcher.show();
                        let _ = launcher.set_focus();
                    }
                }
            }
        })?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(main_window) = app.get_webview_window("main") {
                let _ = main_window.show();
                let _ = main_window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            hide_launcher,
            show_launcher,
            toggle_launcher,
            show_main_window,
            submit_message,
            navigate_to_chat,
            navigate_to_offline,
            get_settings,
            save_settings,
            show_settings,
            open_external_link,
        ])
        .setup(|app| {
            if let Err(e) = setup_tray(app.handle()) {
                eprintln!("Failed to setup tray: {}", e);
            }

            if let Err(e) = setup_global_shortcut(app.handle()) {
                eprintln!("Failed to setup global shortcut: {}", e);
            }

            if let Some(main_window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                main_window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.hide();
                        }
                    }
                });
            }

            {
                let app_handle = app.handle().clone();
                app.listen("response-complete", move |_event| {
                    let notifications_enabled = {
                        use tauri_plugin_store::StoreExt;
                        app_handle
                            .store("settings.json")
                            .ok()
                            .and_then(|store| store.get("app_settings"))
                            .and_then(|v| serde_json::from_value::<AppSettings>(v).ok())
                            .map(|s| s.notifications_enabled)
                            .unwrap_or(true)
                    };

                    if !notifications_enabled {
                        return;
                    }

                    let is_focused = app_handle
                        .get_webview_window("main")
                        .and_then(|w| w.is_focused().ok())
                        .unwrap_or(false);

                    if !is_focused {
                        use tauri_plugin_notification::NotificationExt;
                        let _ = app_handle
                            .notification()
                            .builder()
                            .title("Kimi")
                            .body("Response ready")
                            .show();

                        if let Some(window) = app_handle.get_webview_window("main") {
                            let _ = window.show();
                        }
                    }
                });
            }

            if let Some(launcher) = app.get_webview_window("launcher") {
                let app_handle = app.handle().clone();
                launcher.on_window_event(move |event| {
                    if let tauri::WindowEvent::Focused(false) = event {
                        if let Some(launcher_window) = app_handle.get_webview_window("launcher") {
                            let _ = launcher_window.hide();
                        }
                    }
                });
            }

            #[cfg(target_os = "macos")]
            #[allow(deprecated)]
            {
                if let Some(launcher) = app.get_webview_window("launcher") {
                    if let Ok(ns_window) = launcher.ns_window() {
                        let ns_window = ns_window as id;
                        unsafe {
                            let clear_color = NSColor::clearColor(nil);
                            ns_window.setBackgroundColor_(clear_color);

                            let content_view: id = msg_send![ns_window, contentView];
                            if !content_view.is_null() {
                                let subviews: id = msg_send![content_view, subviews];
                                let count: usize = msg_send![subviews, count];
                                for i in 0..count {
                                    let subview: id = msg_send![subviews, objectAtIndex:i];
                                    let _: () = msg_send![subview, _setDrawsBackground:NO];
                                }
                            }
                        }
                    }
                }
            }

            if let Some(main_window) = app.get_webview_window("main") {
                apply_all_wrappers(&main_window);
                
                #[cfg(target_os = "macos")]
                {
                    let _ = main_window.set_title_bar_style(TitleBarStyle::Overlay);
                    let _ = main_window.inject_titlebar_styles();
                }
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, _event| {
            #[cfg(target_os = "macos")]
            if let RunEvent::Reopen { .. } = _event {
                if let Some(window) = _app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use wrappers::{escape_js, build_js};
    use wrappers::config::{Selectors, Timeouts, Urls};

    #[test]
    fn test_app_settings_default() {
        let settings = AppSettings::default();
        assert!(settings.new_chat_default, "new_chat_default should be true");
        assert!(
            settings.notifications_enabled,
            "notifications_enabled should be true"
        );
    }

    #[test]
    fn test_app_settings_serialization_roundtrip() {
        let settings = AppSettings {
            new_chat_default: false,
            notifications_enabled: true,
        };
        let json = serde_json::to_value(&settings).unwrap();
        let deserialized: AppSettings = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.new_chat_default, false);
        assert_eq!(deserialized.notifications_enabled, true);
    }

    #[test]
    fn test_app_settings_deserialize_missing_fields_uses_defaults() {
        let json = serde_json::json!({ "new_chat_default": false });
        let result: Result<AppSettings, _> = serde_json::from_value(json);
        assert!(
            result.is_err(),
            "Missing fields should fail without #[serde(default)]"
        );
    }

    #[test]
    fn test_escape_js_escapes_backticks() {
        let escaped = escape_js("code `inline` here");
        assert!(escaped.contains(r"\`inline\`"));
        assert!(!escaped.contains("code `inline` here"));
    }

    #[test]
    fn test_escape_js_escapes_backslashes() {
        let escaped = escape_js(r"path\to\file");
        assert!(escaped.contains(r"path\\to\\file"));
    }

    #[test]
    fn test_escape_js_escapes_dollar_signs() {
        let escaped = escape_js("cost is $100");
        assert!(escaped.contains(r"cost is \$100"));
    }

    #[test]
    fn test_escape_js_escapes_newlines() {
        let escaped = escape_js("line1\nline2\rline3");
        assert!(escaped.contains(r"line1\nline2\rline3"));
    }

    #[test]
    fn test_escape_js_handles_empty_string() {
        let escaped = escape_js("");
        assert_eq!(escaped, "");
    }

    #[test]
    fn test_build_js_replaces_placeholders() {
        let template = "Hello {{name}}, you are {{age}} years old";
        let result = build_js(template, &[("name", "Alice"), ("age", "30")]);
        assert_eq!(result, "Hello Alice, you are 30 years old");
    }

    #[test]
    fn test_build_js_handles_missing_placeholders() {
        let template = "Hello {{name}}";
        let result = build_js(template, &[]);
        assert_eq!(result, "Hello {{name}}");
    }

    #[test]
    fn test_inject_message_js_simple() {
        let js = build_js(wrappers::INJECT_MESSAGE_JS, &[("message", "Hello world")]);
        assert!(js.contains("Hello world"));
        assert!(js.contains("emitResult"));
        assert!(js.contains("findTextarea"));
    }

    #[test]
    fn test_response_watcher_js_is_valid() {
        assert!(!wrappers::RESPONSE_WATCHER_JS.is_empty());
        assert!(wrappers::RESPONSE_WATCHER_JS.contains("__kimiResponseWatcher"));
        assert!(wrappers::RESPONSE_WATCHER_JS.contains("response-complete"));
    }

    #[test]
    fn test_titlebar_overlap_js_is_valid() {
        assert!(!wrappers::TITLEBAR_OVERLAP_JS.is_empty());
        assert!(wrappers::TITLEBAR_OVERLAP_JS.contains("{{style_id}}"));
    }

    #[test]
    fn test_connectivity_js_is_valid() {
        assert!(!wrappers::CONNECTIVITY_JS.is_empty());
        assert!(wrappers::CONNECTIVITY_JS.contains("serviceWorker"));
    }

    #[test]
    fn test_link_interceptor_js_is_valid() {
        assert!(!wrappers::LINK_INTERCEPTOR_JS.is_empty());
        assert!(wrappers::LINK_INTERCEPTOR_JS.contains("__kimiLinkInterceptor"));
    }

    #[test]
    fn test_config_selectors() {
        assert_eq!(Selectors::CHAT_INPUT, ".chat-input-editor");
        assert_eq!(Selectors::SEND_BUTTON, ".send-button-container:not(.disabled)");
    }

    #[test]
    fn test_config_timeouts() {
        assert_eq!(Timeouts::INJECTION_TOTAL, 8000);
        assert_eq!(Timeouts::INJECTION_MAX_RETRIES, 15);
    }

    #[test]
    fn test_config_urls() {
        assert_eq!(Urls::CHAT, "https://www.kimi.com/");
        assert!(Urls::CHAT.starts_with("https://"));
    }

    #[test]
    fn test_validate_external_url_accepts_https() {
        assert!(validate_external_url("https://example.com").is_ok());
    }

    #[test]
    fn test_validate_external_url_accepts_http() {
        assert!(validate_external_url("http://example.com").is_ok());
    }

    #[test]
    fn test_validate_external_url_accepts_mailto() {
        assert!(validate_external_url("mailto:test@example.com").is_ok());
    }

    #[test]
    fn test_validate_external_url_rejects_javascript() {
        let result = validate_external_url("javascript:alert('xss')");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("JavaScript"));
    }

    #[test]
    fn test_validate_external_url_rejects_data_url() {
        let result = validate_external_url("data:text/html,<script>alert('xss')</script>");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Data URLs"));
    }

    #[test]
    fn test_validate_external_url_rejects_file_url() {
        let result = validate_external_url("file:///etc/passwd");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("File URLs"));
    }

    #[test]
    fn test_validate_external_url_rejects_vbscript() {
        let result = validate_external_url("vbscript:msgbox('test')");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_external_url_rejects_relative_url() {
        let result = validate_external_url("/path/to/page");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_external_url_case_insensitive() {
        assert!(validate_external_url("HTTPS://EXAMPLE.COM").is_ok());
        assert!(validate_external_url("JAVASCRIPT:alert(1)").is_err());
    }
}
