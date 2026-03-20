use std::cell::RefCell;
use std::mem::MaybeUninit;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEINPUT, MOUSE_EVENT_FLAGS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, HHOOK, MSG, MSLLHOOKSTRUCT, PeekMessageW, PM_REMOVE,
    SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, WH_MOUSE_LL, WM_MOUSEMOVE,
    WM_RBUTTONDOWN, WM_RBUTTONUP,
};

use crate::config::Config;
use crate::gesture::{GestureEngine, GestureEvent, GestureOutcome, Point};
use crate::input::InputMessage;

/// LLMHF_INJECTED: set in MSLLHOOKSTRUCT.flags when the event was injected by SendInput.
const LLMHF_INJECTED: u32 = 0x1;

// ---------------------------------------------------------------------------
// Thread-local hook state
// Win32 low-level mouse hook callbacks don't carry a user-data pointer, so we
// store state in a thread-local instead.
// ---------------------------------------------------------------------------

struct HookState {
    engine: GestureEngine,
    config: Arc<Mutex<Config>>,
    paused: bool,
}

thread_local! {
    static HOOK_STATE: RefCell<Option<HookState>> = RefCell::new(None);
}

// ---------------------------------------------------------------------------
// Actions returned from the callback (avoids borrow across FFI boundary)
// ---------------------------------------------------------------------------

enum Action {
    PassThrough,
    Suppress,
    PassThroughRightClick { x: i32, y: i32 },
    EmitShortcut(crate::config::Shortcut),
}

// ---------------------------------------------------------------------------
// Low-level mouse hook callback
// ---------------------------------------------------------------------------

unsafe extern "system" fn low_level_mouse_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    // Per Win32 spec: if nCode < 0, must call next hook and return its value.
    if n_code < 0 {
        return CallNextHookEx(Some(HHOOK(std::ptr::null_mut())), n_code, w_param, l_param);
    }

    let action = HOOK_STATE.with(|cell| {
        let mut borrow = cell.borrow_mut();
        let state = match borrow.as_mut() {
            Some(s) => s,
            None => return Action::PassThrough,
        };

        if state.paused {
            return Action::PassThrough;
        }

        let info = &*(l_param.0 as *const MSLLHOOKSTRUCT);

        // Ignore events we injected ourselves to prevent feedback loops.
        if info.flags & LLMHF_INJECTED != 0 {
            return Action::PassThrough;
        }

        let msg = w_param.0 as u32;
        match msg {
            _ if msg == WM_RBUTTONDOWN => {
                let pos = Point::new(info.pt.x as f64, info.pt.y as f64);
                let _ = state.engine.process(GestureEvent::RightDown { pos });
                Action::Suppress
            }

            _ if msg == WM_RBUTTONUP => {
                let outcome = state.engine.process(GestureEvent::RightUp);
                match outcome {
                    GestureOutcome::Passthrough => Action::PassThroughRightClick {
                        x: info.pt.x,
                        y: info.pt.y,
                    },
                    GestureOutcome::Gesture(dir) => {
                        let shortcut = state
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

            _ if msg == WM_MOUSEMOVE => {
                let pos = Point::new(info.pt.x as f64, info.pt.y as f64);
                let _ = state.engine.process(GestureEvent::MouseMove { pos });
                Action::PassThrough
            }

            _ => Action::PassThrough,
        }
    });

    match action {
        Action::PassThrough => {
            CallNextHookEx(Some(HHOOK(std::ptr::null_mut())), n_code, w_param, l_param)
        }
        Action::Suppress => LRESULT(1),
        Action::PassThroughRightClick { x, y } => {
            inject_right_click(x, y);
            LRESULT(1)
        }
        Action::EmitShortcut(shortcut) => {
            log::info!("gestro: gesture → {:?}", shortcut.keys);
            crate::shortcut_windows::emit_shortcut(&shortcut);
            LRESULT(1)
        }
    }
}

/// Inject a synthetic right-click (down + up) at the current cursor position.
/// We do not move the cursor — it is already at the correct position when the
/// user releases the button, so adding MOUSEEVENTF_MOVE would only introduce
/// rounding errors from the 0-65535 normalisation and shift the cursor.
unsafe fn inject_right_click(_x: i32, _y: i32) {
    let inputs = [
        make_mouse_input(0, 0, MOUSEEVENTF_RIGHTDOWN),
        make_mouse_input(0, 0, MOUSEEVENTF_RIGHTUP),
    ];

    SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
}

fn make_mouse_input(x: i32, y: i32, flags: MOUSE_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_MOUSE,
        Anonymous: INPUT_0 {
            mi: MOUSEINPUT {
                dx: x,
                dy: y,
                mouseData: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn run(config: Arc<Mutex<Config>>, rx: std::sync::mpsc::Receiver<InputMessage>) {
    let threshold = config.lock().unwrap().threshold_px;

    HOOK_STATE.with(|cell| {
        *cell.borrow_mut() = Some(HookState {
            engine: GestureEngine::new(threshold),
            config: Arc::clone(&config),
            paused: false,
        });
    });

    let hook = unsafe {
        SetWindowsHookExW(WH_MOUSE_LL, Some(low_level_mouse_proc), None, 0)
    };

    let hook = match hook {
        Ok(h) => {
            log::info!("gestro: WH_MOUSE_LL hook installed");
            h
        }
        Err(e) => {
            log::error!("gestro: SetWindowsHookExW failed: {e}");
            return;
        }
    };

    // Message pump: required to deliver WH_MOUSE_LL callbacks.
    loop {
        unsafe {
            let mut msg = MaybeUninit::<MSG>::uninit();
            while PeekMessageW(msg.as_mut_ptr(), Some(HWND(std::ptr::null_mut())), 0, 0, PM_REMOVE)
                .as_bool()
            {
                TranslateMessage(msg.as_ptr());
                DispatchMessageW(msg.as_ptr());
            }
        }

        match rx.try_recv() {
            Ok(InputMessage::Stop) => break,

            Ok(InputMessage::Pause) => {
                HOOK_STATE.with(|cell| {
                    if let Some(s) = cell.borrow_mut().as_mut() {
                        if !s.paused {
                            s.engine = GestureEngine::new(s.engine.threshold_px);
                            s.paused = true;
                            log::info!("gestro: hook paused");
                        }
                    }
                });
            }

            Ok(InputMessage::Resume) => {
                HOOK_STATE.with(|cell| {
                    if let Some(s) = cell.borrow_mut().as_mut() {
                        if s.paused {
                            s.paused = false;
                            log::info!("gestro: hook resumed");
                        }
                    }
                });
            }

            Ok(InputMessage::UpdateConfig(new_cfg)) => {
                HOOK_STATE.with(|cell| {
                    if let Some(s) = cell.borrow_mut().as_mut() {
                        s.engine.update_threshold(new_cfg.threshold_px);
                        *s.config.lock().unwrap() = new_cfg;
                    }
                });
            }

            Err(std::sync::mpsc::TryRecvError::Empty) => {}
            Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
        }

        thread::sleep(Duration::from_millis(10));
    }

    unsafe {
        let _ = UnhookWindowsHookEx(hook);
    }

    log::info!("gestro: input thread exiting");
}
