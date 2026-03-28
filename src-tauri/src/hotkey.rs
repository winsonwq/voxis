//! Voix — Global hotkey module
//! Detects Option+E (macOS) / Alt+E (Windows) for start/stop recording

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};

static CALLBACK: std::sync::OnceLock<Box<dyn Fn() + Send + Sync>> = std::sync::OnceLock::new();
static RECORDING: AtomicBool = AtomicBool::new(false);

pub fn is_recording() -> bool {
    RECORDING.load(Ordering::SeqCst)
}

pub fn set_recording(v: bool) {
    RECORDING.store(v, Ordering::SeqCst);
}

/// Start listening for global hotkey. Callback fires on each trigger.
pub fn start(app_handle: AppHandle, callback: impl Fn() + Send + Sync + 'static) {
    let _ = CALLBACK.set(Box::new(callback));

    #[cfg(target_os = "macos")]
    {
        use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

        let shortcut = "Option+E".parse::<Shortcut>().unwrap();
        let app = app_handle.clone();

        app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                if let Some(cb) = CALLBACK.get() {
                    cb();
                }
            }
        }).ok();
    }

    #[cfg(target_os = "windows")]
    {
        use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

        let shortcut = "Alt+E".parse::<Shortcut>().unwrap();
        let app = app_handle.clone();

        app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
            if event.state == ShortcutState::Pressed {
                if let Some(cb) = CALLBACK.get() {
                    cb();
                }
            }
        }).ok();
    }

    log::info!("Global hotkey registered: Option+E (macOS) / Alt+E (Windows)");
}
