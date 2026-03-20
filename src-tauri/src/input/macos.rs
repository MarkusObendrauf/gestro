use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::config::Config;
use crate::gesture::{GestureEngine, GestureEvent, GestureOutcome, Point};
use crate::input::InputMessage;

// ---------------------------------------------------------------------------
// Raw CGPoint (CGFloat = f64 on all supported 64-bit macOS targets)
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct CGPoint {
    x: f64,
    y: f64,
}

// ---------------------------------------------------------------------------
// CoreGraphics + CoreFoundation raw FFI
// ---------------------------------------------------------------------------

type CGEventTapProxy = *mut c_void;
type CGEventRef = *mut c_void;
type CFMachPortRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;
type CFRunLoopRef = *mut c_void;

/// kCGHIDEventTap — intercepts at the HID level, before any application sees events.
const HID_TAP: u32 = 0;
/// kCGHeadInsertEventTap — insert at the head of the handler list.
const HEAD_INSERT: u32 = 0;
/// kCGEventTapOptionDefault — active tap; can suppress or modify events.
const TAP_DEFAULT: u32 = 0;
/// kCGSessionEventTap — used for re-posting events so they bypass our HID-level tap.
const SESSION_TAP: u32 = 1;

/// Event type constants
const ET_RIGHT_DOWN: u32 = 3; // kCGEventRightMouseDown
const ET_RIGHT_UP: u32 = 4; // kCGEventRightMouseUp
const ET_RIGHT_DRAGGED: u32 = 7; // kCGEventRightMouseDragged

/// kCGMouseButtonRight
const MOUSE_BTN_RIGHT: u32 = 2;

/// Special pseudo-types sent to the callback when macOS auto-disables the tap.
const ET_TAP_DISABLED_TIMEOUT: u32 = 0xFFFFFFFE; // kCGEventTapDisabledByTimeout
const ET_TAP_DISABLED_USER: u32 = 0xFFFFFFFF; // kCGEventTapDisabledByUserInput

/// Event mask: right-button events + right-drag movement.
/// We intentionally omit kCGEventMouseMoved (type 5) — that fires on every mouse
/// movement even when no button is held, flooding the callback and causing macOS
/// to auto-disable the tap.  Right-drag movement arrives as ET_RIGHT_DRAGGED.
const EVENT_MASK: u64 =
    (1 << ET_RIGHT_DOWN) | (1 << ET_RIGHT_UP) | (1 << ET_RIGHT_DRAGGED);

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: unsafe extern "C" fn(
            proxy: CGEventTapProxy,
            event_type: u32,
            event: CGEventRef,
            user_info: *mut c_void,
        ) -> CGEventRef,
        user_info: *mut c_void,
    ) -> CFMachPortRef;

    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
    fn CGEventCreateMouseEvent(
        source: *mut c_void,
        mouse_type: u32,
        mouse_cursor_position: CGPoint,
        mouse_button: u32,
    ) -> CGEventRef;
    fn CGEventPost(tap_location: u32, event: CGEventRef);
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(
        allocator: *mut c_void,
        port: CFMachPortRef,
        order: i64,
    ) -> CFRunLoopSourceRef;
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: *const c_void);
    fn CFRunLoopRunInMode(
        mode: *const c_void,
        seconds: f64,
        return_after_source_handled: bool,
    ) -> i32;
    fn CFRelease(cf: *const c_void);
    static kCFRunLoopDefaultMode: *const c_void;
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    /// Returns non-zero if the process is trusted for Accessibility API access.
    fn AXIsProcessTrusted() -> u8;
}

// ---------------------------------------------------------------------------
// Shared state between the run loop and the event-tap callback
// ---------------------------------------------------------------------------

struct SharedState {
    engine: GestureEngine,
    config: Arc<Mutex<Config>>,
    paused: bool,
    /// Stored so the callback can immediately re-enable the tap on timeout.
    tap: CFMachPortRef,
}

// SharedState is only ever accessed on the input thread (single-threaded CFRunLoop),
// but we wrap it in Arc<Mutex<>> to satisfy Rust's Send requirement for the raw pointer
// passed through CGEventTap's userInfo.
unsafe impl Send for SharedState {}

// ---------------------------------------------------------------------------
// Actions returned from handle_event (avoids holding the mutex during I/O)
// ---------------------------------------------------------------------------

enum Action {
    PassThrough,
    Suppress,
    PassThroughRightClick,
    EmitShortcut(crate::config::Shortcut),
}

// ---------------------------------------------------------------------------
// Event tap callback — must be a named `extern "C"` fn
// ---------------------------------------------------------------------------

unsafe extern "C" fn tap_callback(
    _proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    // Borrow the Arc without taking ownership: from_raw + mem::forget.
    let state: Arc<Mutex<SharedState>> = Arc::from_raw(user_info as *const Mutex<SharedState>);
    let result = handle_event(&state, event_type, event);
    std::mem::forget(state);
    result
}

unsafe fn handle_event(
    state: &Arc<Mutex<SharedState>>,
    event_type: u32,
    event: CGEventRef,
) -> CGEventRef {
    let action = {
        let mut s = match state.try_lock() {
            Ok(g) => g,
            Err(_) => return event, // lock contention: pass through
        };

        if s.paused {
            return event;
        }

        match event_type {
            // macOS auto-disabled our tap (callback was too slow or the system
            // decided to revoke it).  Re-enable immediately.
            ET_TAP_DISABLED_TIMEOUT | ET_TAP_DISABLED_USER => {
                if !s.paused {
                    unsafe { CGEventTapEnable(s.tap, true) };
                    log::warn!("gestro: event tap auto-disabled by macOS; re-enabled");
                }
                return std::ptr::null_mut();
            }

            ET_RIGHT_DOWN => {
                let pos = CGEventGetLocation(event);
                let _ = s.engine.process(GestureEvent::RightDown {
                    pos: Point::new(pos.x, pos.y),
                });
                // Suppress the real right-down; we re-post it on right-up if needed.
                Action::Suppress
            }

            ET_RIGHT_UP => {
                let outcome = s.engine.process(GestureEvent::RightUp);
                match outcome {
                    GestureOutcome::Passthrough => Action::PassThroughRightClick,
                    GestureOutcome::Gesture(dir) => {
                        let shortcut = s
                            .config
                            .lock()
                            .unwrap()
                            .directions
                            .get(&dir)
                            .and_then(|v| v.clone());
                        match shortcut {
                            Some(sc) => Action::EmitShortcut(sc),
                            None => {
                                log::debug!("gestro: gesture {:?} — no shortcut bound", dir);
                                Action::Suppress
                            }
                        }
                    }
                    GestureOutcome::None => Action::Suppress,
                }
            }

            ET_RIGHT_DRAGGED => {
                let pos = CGEventGetLocation(event);
                let _ = s.engine.process(GestureEvent::MouseMove {
                    pos: Point::new(pos.x, pos.y),
                });
                Action::PassThrough
            }

            _ => Action::PassThrough,
        }
        // MutexGuard dropped here
    };

    match action {
        Action::PassThrough => event,
        Action::Suppress => std::ptr::null_mut(),
        Action::PassThroughRightClick => {
            // Re-post right-down + right-up at session level so they bypass our tap.
            let pos = CGEventGetLocation(event);
            post_right_click(pos);
            std::ptr::null_mut()
        }
        Action::EmitShortcut(shortcut) => {
            log::info!("gestro: gesture → {:?}", shortcut.keys);
            crate::shortcut_macos::emit_shortcut(&shortcut);
            std::ptr::null_mut()
        }
    }
}

/// Inject a synthetic right-click (down + up) at `pos` via `kCGSessionEventTap`.
unsafe fn post_right_click(pos: CGPoint) {
    let down = CGEventCreateMouseEvent(std::ptr::null_mut(), ET_RIGHT_DOWN, pos, MOUSE_BTN_RIGHT);
    if !down.is_null() {
        CGEventPost(SESSION_TAP, down);
        CFRelease(down as *const c_void);
    }
    let up = CGEventCreateMouseEvent(std::ptr::null_mut(), ET_RIGHT_UP, pos, MOUSE_BTN_RIGHT);
    if !up.is_null() {
        CGEventPost(SESSION_TAP, up);
        CFRelease(up as *const c_void);
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn run(config: Arc<Mutex<Config>>, rx: std::sync::mpsc::Receiver<InputMessage>) {
    let threshold = config.lock().unwrap().threshold_px;
    let state = Arc::new(Mutex::new(SharedState {
        engine: GestureEngine::new(threshold),
        config: Arc::clone(&config),
        paused: false,
        tap: std::ptr::null_mut(),
    }));

    // Leak one Arc refcount into the raw pointer passed as userInfo.
    // It is reclaimed via Arc::from_raw at the end of this function.
    let state_ptr = Arc::into_raw(Arc::clone(&state)) as *mut c_void;

    // Retry creating the event tap until it succeeds.  When launched at login
    // the Accessibility permission check can return false until the user grants
    // access in System Settings, or until the security subsystem finishes
    // initialising — so we wait and try again every 5 seconds rather than
    // giving up immediately.
    let tap = loop {
        if unsafe { AXIsProcessTrusted() } == 0 {
            log::warn!(
                "gestro: Accessibility permission not granted – retrying in 5 s. \
                 Open System Settings → Privacy & Security → Accessibility and enable gestro."
            );
        } else {
            let t = unsafe {
                CGEventTapCreate(
                    HID_TAP,
                    HEAD_INSERT,
                    TAP_DEFAULT,
                    EVENT_MASK,
                    tap_callback,
                    state_ptr,
                )
            };
            if !t.is_null() {
                break t;
            }
            log::warn!("gestro: CGEventTapCreate failed – retrying in 5 s");
        }

        // Check for a Stop message while waiting so we don't block shutdown.
        match rx.try_recv() {
            Ok(InputMessage::Stop) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                unsafe { drop(Arc::from_raw(state_ptr as *const Mutex<SharedState>)) };
                return;
            }
            _ => {}
        }
        std::thread::sleep(Duration::from_secs(5));
    };

    // Store tap in SharedState so the callback can re-enable it on timeout.
    state.lock().unwrap().tap = tap;

    let source = unsafe { CFMachPortCreateRunLoopSource(std::ptr::null_mut(), tap, 0) };
    let rl = unsafe { CFRunLoopGetCurrent() };
    unsafe { CFRunLoopAddSource(rl, source, kCFRunLoopDefaultMode) };
    unsafe { CGEventTapEnable(tap, true) };

    log::info!("gestro: CGEventTap active");

    // Re-arm counter: macOS can auto-disable an event tap if it considers the
    // callback too slow.  Re-enable the tap every ~5 s so it recovers.
    let mut rearm_ticks: u32 = 0;

    // Run the CFRunLoop in 50 ms slices, checking the mpsc channel between iterations.
    loop {
        unsafe { CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.05, false) };

        rearm_ticks += 1;
        if rearm_ticks >= 100 {
            rearm_ticks = 0;
            let paused = state.lock().unwrap().paused;
            if !paused {
                unsafe { CGEventTapEnable(tap, true) };
            }
        }

        match rx.try_recv() {
            Ok(InputMessage::Stop) => break,

            Ok(InputMessage::Pause) => {
                let mut s = state.lock().unwrap();
                if !s.paused {
                    unsafe { CGEventTapEnable(tap, false) };
                    s.engine = GestureEngine::new(s.engine.threshold_px);
                    s.paused = true;
                    log::info!("gestro: event tap disabled (settings open)");
                }
            }

            Ok(InputMessage::Resume) => {
                let mut s = state.lock().unwrap();
                if s.paused {
                    unsafe { CGEventTapEnable(tap, true) };
                    s.paused = false;
                    log::info!("gestro: event tap enabled (settings closed)");
                }
            }

            Ok(InputMessage::UpdateConfig(new_cfg)) => {
                let mut s = state.lock().unwrap();
                s.engine.update_threshold(new_cfg.threshold_px);
                *s.config.lock().unwrap() = new_cfg;
            }

            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
        }
    }

    unsafe {
        CGEventTapEnable(tap, false);
        CFRelease(source as *const c_void);
        CFRelease(tap as *const c_void);
        // Reclaim the Arc we leaked into userInfo.
        drop(Arc::from_raw(state_ptr as *const Mutex<SharedState>));
    }

    log::info!("gestro: input thread exiting");
}
