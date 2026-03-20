use crate::config::Config;

pub mod linux;

/// Message sent from the main thread to the input thread.
#[allow(dead_code)]
pub enum InputMessage {
    UpdateConfig(Config),
    /// Release the evdev grab so the compositor gets native mouse input (settings window open).
    Pause,
    /// Re-acquire the evdev grab (settings window closed/hidden).
    Resume,
    Stop,
}

#[cfg(target_os = "linux")]
pub use linux::run as run_platform;

#[cfg(not(target_os = "linux"))]
pub fn run_platform(
    _config: std::sync::Arc<std::sync::Mutex<Config>>,
    _rx: std::sync::mpsc::Receiver<InputMessage>,
) {
    log::warn!("No input backend available for this platform.");
}
