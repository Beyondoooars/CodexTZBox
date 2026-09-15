use serde_json::{json, Value};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};

#[path = "../../backend/common.rs"]
mod common;
#[path = "../../backend/win/mod.rs"]
mod windows;

fn call_backend_sync(_app: AppHandle, command: String, payload: Value) -> Result<Value, String> {
    static REQUEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = REQUEST_LOCK.lock().map_err(|_| "系统功能锁不可用。".to_string())?;
    windows::execute(&command, payload)
}

#[tauri::command]
async fn backend(app: AppHandle, command: String, payload: Option<Value>) -> Result<Value, String> {
    let cmd = command.clone();
    let app_handle = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        call_backend_sync(app, command, payload.unwrap_or_else(|| json!({})))
    })
    .await
    .map_err(|e| e.to_string())??;

    if cmd == "launch" {
        if let Some(window) = app_handle.get_webview_window("main") {
            let _ = window.hide();
        }
    }

    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let launch_item = MenuItem::with_id(app, "launch", "保存并启动 Codex", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出程序", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &launch_item, &sep, &quit_item])?;

            let mut builder = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Codex 时区启动器")
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                        "launch" => {
                            let app_clone = app.clone();
                            tauri::async_runtime::spawn_blocking(move || {
                                match call_backend_sync(app_clone.clone(), "launch".into(), json!({})) {
                                    Ok(_) => {
                                        if let Some(window) = app_clone.get_webview_window("main") {
                                            let _ = window.hide();
                                        }
                                    }
                                    Err(_) => {
                                        if let Some(window) = app_clone.get_webview_window("main") {
                                            let _ = window.show();
                                            let _ = window.unminimize();
                                            let _ = window.set_focus();
                                        }
                                    }
                                }
                            });
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {}
                    }
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
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                });

            if let Some(icon) = app.default_window_icon() {
                builder = builder.icon(icon.clone());
            }

            builder.build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![backend])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
