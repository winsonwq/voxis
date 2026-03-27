//! Voix — Text injection module
//! Injects text into the active application via keyboard simulation

use anyhow::Result;

#[cfg(target_os = "macos")]
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};

#[cfg(target_os = "macos")]
pub fn inject_text(text: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())?;
    enigo.text(text)?;
    Ok(())
}

#[cfg(target_os = "windows")]
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Settings,
};

#[cfg(target_os = "windows")]
pub fn inject_text(text: &str) -> Result<()> {
    let mut enigo = Enigo::new(&Settings::default())?;
    enigo.text(text)?;
    Ok(())
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn inject_text(_text: &str) -> Result<()> {
    anyhow::bail!("Text injection not supported on this platform")
}
