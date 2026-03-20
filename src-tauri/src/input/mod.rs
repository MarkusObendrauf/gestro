use crate::config::Config;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

/// Message sent from the main thread to the input thread.
#[allow(dead_code)]
pub enum InputMessage {
    UpdateConfig(Config),
    /// Disable event interception while the settings window is open.
    Pause,
    /// Re-enable event interception after the settings window is closed/hidden.
    Resume,
    Stop,
}

#[cfg(target_os = "linux")]
pub use linux::run as run_platform;

#[cfg(target_os = "macos")]
pub use macos::run as run_platform;

#[cfg(target_os = "windows")]
pub use windows::run as run_platform;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn run_platform(
    _config: std::sync::Arc<std::sync::Mutex<Config>>,
    _rx: std::sync::mpsc::Receiver<InputMessage>,
) {
    log::warn!("gestro: no input backend available for this platform.");
}
