mod assets;
mod bag_manager;
mod db;
mod game_state_machine;
mod models;
mod net;
mod parser;
mod pricing;
mod state_store;
mod tailer;
mod utils;

use models::AppSettings;
use tauri::{Emitter, Manager, State};
use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};
use tokio::sync::watch;
use tokio::task;

struct SettingsState(watch::Sender<AppSettings>);

#[tauri::command]
fn set_click_through(window: tauri::Window, ignore: bool) {
    if window.label() == "main" {
        let _ = window.set_ignore_cursor_events(ignore);
    }
}

#[tauri::command]
fn save_settings(
    state: State<'_, SettingsState>,
    ignore_auction: bool,
    apply_tax: bool,
) -> Result<(), String> {
    let settings = AppSettings {
        ignore_auction,
        apply_tax,
    };

    state.0.send(settings.clone()).map_err(|e| e.to_string())?;

    let json = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    let config_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("config.json")))
        .unwrap_or_else(|| std::path::PathBuf::from("config.json"));
    std::fs::write(&config_path, json).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn load_settings(state: State<'_, SettingsState>) -> Result<AppSettings, String> {
    Ok(state.0.borrow().clone())
}

#[tauri::command]
fn get_cursor_position(window: tauri::Window) -> (f64, f64) {
    match window.cursor_position() {
        Ok(pos) => (pos.x, pos.y),
        Err(_) => (0.0, 0.0),
    }
}

#[derive(serde::Serialize)]
struct PriceEntry {
    item_id: String,
    item_name: String,
    price: f64,
}

#[tauri::command]
fn get_all_prices() -> Vec<PriceEntry> {
    let mut entries: Vec<PriceEntry> = crate::pricing::get_all_cached_prices()
        .into_iter()
        .filter(|(_, p)| *p > 0.0)
        .map(|(id, price)| {
            let item_name = crate::db::DbManager::get_item_name(&id);
            PriceEntry {
                item_id: id,
                item_name,
                price,
            }
        })
        .collect();
    entries.sort_by(|a, b| {
        b.price
            .partial_cmp(&a.price)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    entries
}

#[tauri::command]
fn get_log_path() -> String {
    match utils::get_game_log_path() {
        Some(p) => p.to_string_lossy().to_string(),
        None => "Log file not found".to_string(),
    }
}

fn load_settings_from_disk() -> AppSettings {
    let config_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("config.json")))
        .unwrap_or_else(|| std::path::PathBuf::from("config.json"));
    std::fs::read_to_string(&config_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let initial_settings = load_settings_from_disk();
    let (settings_tx, settings_rx) = watch::channel(initial_settings.clone());

    tauri::Builder::default()
        .manage(SettingsState(settings_tx))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    if shortcut.matches(Modifiers::empty(), Code::F8) {
                        let _ = app.emit("toggle_edit_mode", ());
                    }
                    if shortcut.matches(Modifiers::empty(), Code::F7) {
                        if let Some(window) = app.get_webview_window("main") {
                            let visible = window.is_visible().unwrap_or(true);
                            if visible {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(),
        )
        .setup(move |app| {
            use std::str::FromStr;
            use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::TrayIconBuilder;

            let quit_i = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().cloned().unwrap())
                .menu(&menu)
                .on_menu_event(|app, event| {
                    if event.id.as_ref() == "quit" {
                        app.exit(0);
                    }
                })
                .build(app)?;

            if let Ok(shortcut) = Shortcut::from_str("F8") {
                let _ = app.global_shortcut().register(shortcut);
            }
            if let Ok(shortcut) = Shortcut::from_str("F7") {
                let _ = app.global_shortcut().register(shortcut);
            }

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_decorations(false);
                let _ = window.set_skip_taskbar(true);
                let _ = window.set_ignore_cursor_events(true);
                if let Ok(Some(monitor)) = window.primary_monitor() {
                    let size = monitor.size();
                    let pos = monitor.position();
                    let _ =
                        window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                            x: pos.x,
                            y: pos.y,
                        }));
                    let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                        width: size.width,
                        height: size.height,
                    }));
                }
                let _ = window.show();
            }

            let app_handle = app.handle().clone();

            task::spawn(async move {
                match utils::get_game_log_path() {
                    Some(log_path) => {
                        let _ =
                            app_handle.emit("log_status", format!("Watching log: {:?}", log_path));
                        tailer::start_tailing(app_handle, log_path, settings_rx).await;
                    }
                    None => {
                        let _ =
                            app_handle.emit("log_status", "Error: Game log not found!".to_string());
                        let _ =
                            app_handle.emit("game_event", crate::models::GameEvent::LoadingDone);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            assets::check_assets_update,
            assets::download_assets,
            assets::get_assets_path,
            set_click_through,
            save_settings,
            load_settings,
            get_log_path,
            get_cursor_position,
            get_all_prices,
            state_store::save_state,
            state_store::load_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
