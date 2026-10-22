use serde_json::{json, Value};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
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

use ::windows::{
    core::w,
    Win32::{
        Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE},
        System::Threading::{
            CreateEventW, CreateMutexW, OpenEventW, SetEvent, WaitForSingleObject,
            EVENT_MODIFY_STATE, INFINITE,
        },
    },
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Single instance protection:
    // If another instance of CodexTZBox is already running,
    // signal its wake event to bring its window to front, then exit immediately.
    unsafe {
        let mutex = CreateMutexW(None, true, w!("Local\\CodexTZBox_SingleInstance_Mutex"));
        if GetLastError() == ERROR_ALREADY_EXISTS {
            if let Ok(event) = OpenEventW(EVENT_MODIFY_STATE, false, w!("Local\\CodexTZBox_WakeUp_Event")) {
                let _ = SetEvent(event);
                let _ = CloseHandle(event);
            }
            if let Ok(m) = mutex {
                let _ = CloseHandle(m);
            }
            std::process::exit(0);
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Background listener for single-instance wake event
            if let Ok(wake_event) = unsafe { CreateEventW(None, false, false, w!("Local\\CodexTZBox_WakeUp_Event")) } {
                let handle_raw = wake_event.0 as usize;
                let app_handle = app.handle().clone();
                std::thread::spawn(move || {
                    let event = HANDLE(handle_raw as *mut _);
                    loop {
                        let wait_res = unsafe { WaitForSingleObject(event, INFINITE) };
                        if wait_res.0 == 0 {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        } else {
                            break;
                        }
                    }
                    unsafe { let _ = CloseHandle(event); }
                });
            }

            let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let launch_item = MenuItem::with_id(app, "launch", "保存并启动 Codex", true, None::<&str>)?;
            let sep = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出程序", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &launch_item, &sep, &quit_item])?;

            let mut builder = TrayIconBuilder::new()
                .menu(&menu)
                .tooltip("Codex 时区沙盒")
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
                                    Ok(result) => {
                                        if result["launched"].as_bool().unwrap_or(false) {
                                            if let Some(window) = app_clone.get_webview_window("main") {
                                                let _ = window.hide();
                                            }
                                        } else {
                                            // 客户端正在运行：把窗口带到前台，交由前端询问是否重启。
                                            let message = result["message"].as_str().unwrap_or("检测到 Codex 正在运行。").to_string();
                                            if let Some(window) = app_clone.get_webview_window("main") {
                                                let _ = window.show();
                                                let _ = window.unminimize();
                                                let _ = window.set_focus();
                                            }
                                            let _ = app_clone.emit("tray-launch-running", message);
                                        }
                                    }
                                    Err(err) => {
                                        if let Some(window) = app_clone.get_webview_window("main") {
                                            let _ = window.show();
                                            let _ = window.unminimize();
                                            let _ = window.set_focus();
                                            let _ = window.emit("tray-launch-error", err);
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
