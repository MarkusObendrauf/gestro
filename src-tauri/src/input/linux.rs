use std::sync::{Arc, Mutex};

use evdev::{Device, EventType, InputEventKind, Key, RelativeAxisType};
use uinput::event::controller::{Controller, Mouse};
use uinput::event::relative::{Position, Wheel};

use crate::config::Config;
use crate::gesture::{GestureEngine, GestureEvent, GestureOutcome, Point};
use crate::input::InputMessage;
use crate::shortcut::{keys_to_uinput, AnyKey};

/// Find the first device that reports BTN_RIGHT and REL_X (i.e. a mouse).
fn find_mouse() -> Option<Device> {
    for (_, device) in evdev::enumerate() {
        let supported = device.supported_events();
        let has_rel = supported.contains(EventType::RELATIVE);
        let has_key = supported.contains(EventType::KEY);
        if !has_rel || !has_key {
            continue;
        }
        if let Some(keys) = device.supported_keys() {
            if !keys.contains(Key::BTN_RIGHT) {
                continue;
            }
        } else {
            continue;
        }
        if let Some(axes) = device.supported_relative_axes() {
            if axes.contains(RelativeAxisType::REL_X) {
                return Some(device);
            }
        }
    }
    None
}

/// Build a uinput virtual keyboard + mouse for re-emitting events.
fn create_virtual_device(name: &str, id: &evdev::InputId) -> Result<uinput::Device, uinput::Error> {
    uinput::default()?
        .name(name)?
        .bus(id.bus_type().0)
        .vendor(id.vendor())
        .product(id.product())
        .event(uinput::event::Keyboard::All)?
        .event(Controller::Mouse(Mouse::Left))?
        .event(Controller::Mouse(Mouse::Right))?
        .event(Controller::Mouse(Mouse::Middle))?
        .event(Controller::Mouse(Mouse::Side))?
        .event(Controller::Mouse(Mouse::Extra))?
        .event(uinput::event::relative::Relative::Position(Position::X))?
        .event(uinput::event::relative::Relative::Position(Position::Y))?
        .event(uinput::event::relative::Relative::Wheel(Wheel::Vertical))?
        .event(uinput::event::relative::Relative::Wheel(Wheel::Horizontal))?
        .create()
}

/// Emit a right-click (press + release) on the virtual device.
/// Uses raw write since uinput's high-level API targets keyboard keys only.
fn passthrough_right_click(vdev: &mut uinput::Device) {
    // EV_KEY = 1, BTN_RIGHT = 0x111 = 273
    let _ = vdev.write(1, 273, 1); // press
    let _ = vdev.synchronize();
    let _ = vdev.write(1, 273, 0); // release
    let _ = vdev.synchronize();
}

/// Press all keys in a shortcut, then release them in reverse.
fn emit_shortcut(vdev: &mut uinput::Device, shortcut: &crate::config::Shortcut) {
    let codes = keys_to_uinput(&shortcut.keys);
    for key in &codes {
        match key {
            AnyKey::Key(k) => { let _ = vdev.press(k); }
            AnyKey::Misc(m) => { let _ = vdev.press(m); }
        }
    }
    let _ = vdev.synchronize();
    for key in codes.iter().rev() {
        match key {
            AnyKey::Key(k) => { let _ = vdev.release(k); }
            AnyKey::Misc(m) => { let _ = vdev.release(m); }
        }
    }
    let _ = vdev.synchronize();
}

/// Main loop for the Linux input backend.
/// Runs on a dedicated thread; blocks on evdev events.
pub fn run(config: Arc<Mutex<Config>>, rx: std::sync::mpsc::Receiver<InputMessage>) {
    let mut mouse = match find_mouse() {
        Some(d) => d,
        None => {
            log::error!("gestro: no mouse device found in /dev/input — make sure you are in the 'input' group");
            return;
        }
    };

    if let Err(e) = mouse.grab() {
        log::error!("gestro: failed to grab mouse device: {e}. Make sure you are in the 'input' group.");
        return;
    }
    log::info!("gestro: mouse grabbed successfully");

    let mouse_name = mouse.name().unwrap_or("gestro-virtual").to_string();
    let mouse_id = mouse.input_id();
    let mut vdev = match create_virtual_device(&mouse_name, &mouse_id) {
        Ok(d) => d,
        Err(e) => {
            log::error!("gestro: failed to create uinput virtual device: {e}. Make sure /dev/uinput is accessible.");
            // Release grab before returning
            let _ = mouse.ungrab();
            return;
        }
    };
    // Give the compositor time to discover and register the new uinput device
    // before any events are emitted from it.
    std::thread::sleep(std::time::Duration::from_millis(200));

    let threshold = {
        let cfg = config.lock().unwrap();
        cfg.threshold_px
    };
    let mut engine = GestureEngine::new(threshold);
    let mut cursor = Point::new(0.0, 0.0);
    let mut paused = false;

    loop {
        // Check for config updates / control signals (non-blocking)
        match rx.try_recv() {
            Ok(InputMessage::Stop) => break,
            Ok(InputMessage::UpdateConfig(new_cfg)) => {
                engine.update_threshold(new_cfg.threshold_px);
                let mut cfg = config.lock().unwrap();
                *cfg = new_cfg;
            }
            Ok(InputMessage::Pause) if !paused => {
                let _ = mouse.ungrab();
                engine = GestureEngine::new(engine.threshold_px);
                paused = true;
                log::info!("gestro: mouse ungrabbed (settings open)");
            }
            Ok(InputMessage::Resume) if paused => {
                if let Err(e) = mouse.grab() {
                    log::error!("gestro: failed to re-grab mouse: {e}");
                } else {
                    paused = false;
                    log::info!("gestro: mouse re-grabbed (settings closed)");
                }
            }
            _ => {}
        }

        // Fetch events (blocking)
        let events = match mouse.fetch_events() {
            Ok(e) => e,
            Err(e) => {
                log::error!("gestro: evdev error: {e}");
                break;
            }
        };

        for ev in events {
            // While paused the evdev grab is released; libinput handles events
            // normally. We drain them here so the buffer doesn't fill up.
            if paused { continue; }

            match ev.kind() {
                InputEventKind::Key(Key::BTN_RIGHT) => {
                    let outcome = if ev.value() == 1 {
                        engine.process(GestureEvent::RightDown {
                            pos: cursor,
                        })
                    } else if ev.value() == 0 {
                        engine.process(GestureEvent::RightUp)
                    } else {
                        GestureOutcome::None
                    };

                    handle_outcome(outcome, &config, &mut vdev);
                }

                InputEventKind::RelAxis(RelativeAxisType::REL_X) => {
                    let _ = vdev.write(ev.event_type().0.into(), ev.code().into(), ev.value());
                    cursor.x += ev.value() as f64;
                    let outcome = engine.process(GestureEvent::MouseMove { pos: cursor });
                    handle_outcome(outcome, &config, &mut vdev);
                }

                InputEventKind::RelAxis(RelativeAxisType::REL_Y) => {
                    let _ = vdev.write(ev.event_type().0.into(), ev.code().into(), ev.value());
                    cursor.y += ev.value() as f64;
                    let outcome = engine.process(GestureEvent::MouseMove { pos: cursor });
                    handle_outcome(outcome, &config, &mut vdev);
                }

                // Pass through all other events (left click, scroll, etc.)
                _ => {
                    // Re-emit raw event via uinput so the system sees it normally.
                    // We use a small delay to avoid kernel buffer collisions.
                    let _ = vdev.write(ev.event_type().0.into(), ev.code().into(), ev.value());
                }
            }
        }
    }

    let _ = mouse.ungrab();
    log::info!("gestro: mouse ungrabbed, input thread exiting");
}

fn handle_outcome(
    outcome: GestureOutcome,
    config: &Arc<Mutex<Config>>,
    vdev: &mut uinput::Device,
) {
    match outcome {
        GestureOutcome::None => {}
        GestureOutcome::Passthrough => {
            passthrough_right_click(vdev);
        }
        GestureOutcome::Gesture(dir) => {
            let cfg = config.lock().unwrap();
            if let Some(Some(shortcut)) = cfg.directions.get(&dir) {
                log::info!("gestro: gesture {:?} → {:?}", dir, shortcut.keys);
                emit_shortcut(vdev, shortcut);
            } else {
                log::debug!("gestro: gesture {:?} — no shortcut bound", dir);
            }
        }
    }
}
