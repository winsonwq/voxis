//! Voix — Global hotkey module
//! Detects Option+E (macOS) / Alt+E (Windows) for start/stop recording

use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// Start listening for global hotkey. Emits "hotkey-triggered" event when pressed.
pub fn start(app_handle: AppHandle) {
    #[cfg(target_os = "macos")]
    let shortcut_str = "Option+E";

    #[cfg(target_os = "windows")]
    let shortcut_str = "Alt+E";

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let shortcut_str = "Alt+E";

    let shortcut = shortcut_str.parse::<Shortcut>().unwrap();

    app_handle.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
        if event.state == ShortcutState::Pressed {
            let _ = _app.emit("hotkey-triggered", ());
            log::debug!("Hotkey triggered");
        }
    }).ok();

    log::info!("Global hotkey registered: {shortcut_str}");
}
