mod config;
mod gesture;
mod input;
#[cfg(target_os = "linux")]
mod shortcut;
#[cfg(target_os = "macos")]
mod shortcut_macos;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::{Manager, WindowEvent};

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

    let (tx, rx) = std::sync::mpsc::sync_channel::<InputMessage>(8);
    // Clone before moving tx into Tauri's managed state so we can use it in
    // the window-event closure below.
    let tx_for_window = tx.clone();

    // Spawn the input backend on a dedicated OS thread.
    #[cfg(any(target_os = "linux", target_os = "macos"))]
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
        .setup(move |app| {
            // Hide the window immediately so the app starts as a tray-only process.
            // We still let it initialize as `visible: true` in tauri.conf.json so
            // that the Wayland xdg_toplevel configure/ack handshake completes at
            // startup; without this, WM decoration buttons (close/min/max) are
            // unresponsive until the user triggers a maximize cycle themselves.
            let win = app.get_webview_window("main").unwrap();
            let _ = win.hide();
            let win2 = win.clone();
            let tx_win = tx_for_window.clone();
            // Flag set just before maximize(); cleared in Resized to trigger
            // the deferred unmaximize once the compositor has applied the state.
            let pending_unmax = Arc::new(AtomicBool::new(false));
            let pending_unmax_event = Arc::clone(&pending_unmax);
            win.on_window_event(move |event| match event {
                // Intercept the OS close button: hide to tray instead of closing.
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = win2.hide();
                    let _ = tx_win.try_send(InputMessage::Resume);
                }
                // Window lost focus (minimized, another window clicked, etc.):
                // re-enable gesture detection. Also hide to tray if minimized.
                WindowEvent::Focused(false) => {
                    let _ = tx_win.try_send(InputMessage::Resume);
                    if win2.is_minimized().unwrap_or(false) {
                        let _ = win2.hide();
                    }
                }
                // Window gained focus: pause gesture detection so WM decorations
                // receive native mouse events from the compositor.
                WindowEvent::Focused(true) => {
                    let _ = tx_win.try_send(InputMessage::Pause);
                }
                // Wayland: after the compositor applies the maximize configure,
                // a Resized event fires. Use that as the signal to restore.
                WindowEvent::Resized(_) => {
                    if pending_unmax_event.swap(false, Ordering::SeqCst) {
                        let _ = win2.unmaximize();
                        // Request center again after unmaximize so the compositor
                        // positions the restored window at center, not top-left.
                        let _ = win2.center();
                    }
                }
                _ => {}
            });

            // ---------------------------------------------------------------
            // System tray
            // ---------------------------------------------------------------
            let toggle_item = MenuItemBuilder::new("Disable")
                .id("toggle")
                .build(app)?;
            let settings_item = MenuItemBuilder::new("Settings")
                .id("settings")
                .build(app)?;
            let quit_item = MenuItemBuilder::new("Quit").id("quit").build(app)?;
            let menu = MenuBuilder::new(app)
                .items(&[&toggle_item, &settings_item, &quit_item])
                .build()?;

            // Track whether gesture detection is enabled (starts enabled).
            let gesture_enabled = Arc::new(AtomicBool::new(true));

            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("gestro")
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "toggle" => {
                        let tx = app.state::<std::sync::mpsc::SyncSender<InputMessage>>();
                        let enabled = gesture_enabled.fetch_xor(true, Ordering::SeqCst);
                        if enabled {
                            // Was enabled → now disabled
                            let _ = tx.try_send(InputMessage::Pause);
                            let _ = toggle_item.set_text("Enable");
                        } else {
                            // Was disabled → now enabled
                            let _ = tx.try_send(InputMessage::Resume);
                            let _ = toggle_item.set_text("Disable");
                        }
                    }
                    "settings" => {
                        if let Some(win) = app.get_webview_window("main") {
                            // Wayland: each hide() unmaps the xdg_toplevel surface,
                            // so re-showing needs a fresh configure/ack handshake.
                            // We maximize immediately after show; the Resized event
                            // handler above will unmaximize once the compositor has
                            // applied the state, leaving the window at normal size.
                            pending_unmax.store(true, Ordering::SeqCst);
                            let _ = win.show();
                            // center() before maximize so the compositor saves a
                            // centered restore geometry instead of top-left.
                            let _ = win.center();
                            let _ = win.set_focus();
                            let _ = win.maximize();
                            // Release the evdev grab while settings is visible so
                            // the compositor receives native mouse events and WM
                            // decorations (close/minimize/maximize) work normally.
                            let tx = app.state::<std::sync::mpsc::SyncSender<InputMessage>>();
                            let _ = tx.try_send(InputMessage::Pause);
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
        .expect("error while running gestro");
}
