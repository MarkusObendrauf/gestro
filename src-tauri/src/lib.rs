mod config;
mod gesture;
mod input;
mod shortcut;

use std::sync::{Arc, Mutex};

use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Manager;

use config::Config;
use input::InputMessage;

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
fn get_config(state: tauri::State<Arc<Mutex<Config>>>) -> Config {
    state.lock().unwrap().clone()
}

#[tauri::command]
fn save_config(
    new_config: Config,
    state: tauri::State<Arc<Mutex<Config>>>,
    tx: tauri::State<std::sync::mpsc::SyncSender<InputMessage>>,
) -> Result<(), String> {
    config::save(&new_config)?;
    let _ = tx.try_send(InputMessage::UpdateConfig(new_config.clone()));
    *state.lock().unwrap() = new_config;
    Ok(())
}

// ---------------------------------------------------------------------------
// App entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let cfg = config::load();
    let shared_config: Arc<Mutex<Config>> = Arc::new(Mutex::new(cfg));

    // Channel for sending messages to the input thread.
    let (tx, rx) = std::sync::mpsc::sync_channel::<InputMessage>(8);

    // Spawn the input backend on a dedicated OS thread.
    #[cfg(target_os = "linux")]
    {
        let config_clone = Arc::clone(&shared_config);
        std::thread::spawn(move || {
            input::run_platform(config_clone, rx);
        });
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(shared_config)
        .manage(tx)
        .setup(|app| {
            // System tray
            let settings_item = MenuItemBuilder::new("Settings")
                .id("settings")
                .build(app)?;
            let quit_item = MenuItemBuilder::new("Quit").id("quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&settings_item, &quit_item])
                .build()?;

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("pie")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "settings" => {
                        if let Some(win) = app.get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_config, save_config])
        .run(tauri::generate_context!())
        .expect("error while running pie");
}
