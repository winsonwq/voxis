mod audio;
mod hotkey;
mod inject;
mod polish;
mod settings;
mod whisper;

use anyhow::Result;
use audio::{AudioManager, AudioState};
use polish::{LLMPolisher, LLMProvider, PolishLevel};
use serde::{Deserialize, Serialize};
use settings::{HistoryEntry, Settings, SettingsStore};
use std::sync::Mutex;
use std::time::Instant;
use tauri::{AppHandle, Manager, State};
use whisper::{ModelSize, WhisperEngine};

pub struct AppState {
    settings: Mutex<SettingsStore>,
    whisper: Mutex<Option<WhisperEngine>>,
    llm: Mutex<Option<LLMPolisher>>,
    audio_state: Mutex<Option<AudioState>>,
    is_recording: Mutex<bool>,
}

impl AppState {
    fn new() -> Result<Self> {
        Ok(Self {
            settings: Mutex::new(SettingsStore::new()?),
            whisper: Mutex::new(None),
            llm: Mutex::new(None),
            audio_state: Mutex::new(None),
            is_recording: Mutex::new(false),
        })
    }
}

// --- Tauri Commands ---

#[tauri::command]
fn get_settings(state: State<AppState>) -> Result<Settings, String> {
    state
        .settings
        .lock()
        .unwrap()
        .get_settings()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn save_setting(key: String, value: String, state: State<AppState>) -> Result<(), String> {
    state
        .settings
        .lock()
        .unwrap()
        .set_setting(&key, &value)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_devices() -> Result<Vec<String>, String> {
    AudioManager::list_devices().map_err(|e| e.to_string())
}

#[tauri::command]
fn check_mic() -> bool {
    AudioManager::is_available()
}

#[tauri::command]
fn check_llm(state: State<AppState>) -> Result<bool, String> {
    let settings = state.settings.lock().unwrap().get_settings().map_err(|e| e.to_string())?;
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .unwrap_or_default();

    let url = if settings.llm_provider == "openai" {
        "https://api.openai.com/v1/models".to_string()
    } else {
        format!("http://localhost:11434/api/tags")
    };

    Ok(client.get(&url).send().is_ok())
}

#[tauri::command]
fn init_whisper(state: State<AppState>) -> Result<(), String> {
    let settings = state.settings.lock().unwrap().get_settings().map_err(|e| e.to_string())?;
    let model_size = match settings.whisper_model.as_str() {
        "tiny" => ModelSize::Tiny,
        "base" => ModelSize::Base,
        "small" => ModelSize::Small,
        "medium" => ModelSize::Medium,
        _ => ModelSize::Small,
    };

    log::info!("Initializing Whisper model: {:?}", model_size);
    let engine = WhisperEngine::new(model_size).map_err(|e| e.to_string())?;
    *state.whisper.lock().unwrap() = Some(engine);
    Ok(())
}

#[tauri::command]
fn init_llm(state: State<AppState>) -> Result<(), String> {
    let settings = state.settings.lock().unwrap().get_settings().map_err(|e| e.to_string())?;
    let provider = if settings.llm_provider == "openai" {
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
        LLMProvider::openai(&api_key, &settings.llm_model)
    } else {
        LLMProvider::ollama(&settings.llm_model)
    };

    log::info!("Initializing LLM provider");
    *state.llm.lock().unwrap() = Some(LLMPolisher::new(provider));
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TranscribeResult {
    pub original: String,
    pub polished: String,
    pub latency_ms: i64,
    pub language: String,
}

/// Start recording audio. Call stop_and_process to finish.
#[tauri::command]
fn start_recording(state: State<AppState>) -> Result<(), String> {
    let mut is_recording = state.is_recording.lock().unwrap();
    if *is_recording {
        return Err("Already recording".into());
    }

    let manager = AudioManager::new();
    let audio_state = manager.start_capture().map_err(|e| e.to_string())?;
    *state.audio_state.lock().unwrap() = Some(audio_state);
    *is_recording = true;
    log::info!("Recording started");
    Ok(())
}

/// Get current audio level (0.0 - 1.0)
#[tauri::command]
fn get_audio_level(state: State<AppState>) -> f32 {
    let audio_state = state.audio_state.lock().unwrap();
    if let Some(ref s) = *audio_state {
        AudioManager::get_level(s)
    } else {
        0.0
    }
}

/// Stop recording and process: transcribe → polish → inject
#[tauri::command]
fn stop_and_process(app_handle: AppHandle, state: State<AppState>) -> Result<TranscribeResult, String> {
    let is_recording = {
        let mut r = state.is_recording.lock().unwrap();
        if !*r {
            return Err("Not recording".into());
        }
        *r = false;
        true
    };

    if !is_recording {
        return Err("Not recording".into());
    }

    let start = Instant::now();

    // Stop capture and get samples
    let samples = {
        let audio_state = state.audio_state.lock().unwrap();
        audio_state
            .as_ref()
            .map(|s| AudioManager::stop_capture(s))
            .unwrap_or_default()
    };

    // Transcribe
    let original = {
        let mut whisper_guard = state.whisper.lock().unwrap();
        let whisper = whisper_guard.as_mut().ok_or("Whisper not initialized")?;
        whisper.transcribe(&samples).map_err(|e| e.to_string())?
    };

    // Polish
    let settings = state.settings.lock().unwrap().get_settings().map_err(|e| e.to_string())?;
    let level = match settings.polish_level.as_str() {
        "light" => PolishLevel::Light,
        "strong" => PolishLevel::Strong,
        _ => PolishLevel::Medium,
    };

    let polished = if original.trim().is_empty() {
        String::new()
    } else {
        let llm_guard = state.llm.lock().unwrap();
        if let Some(llm) = llm_guard.as_ref() {
            match llm.polish(&original, level) {
                Ok(p) => p,
                Err(e) => {
                    log::warn!("LLM polish failed, using original: {}", e);
                    original.clone()
                }
            }
        } else {
            original.clone()
        }
    };

    let total_ms = start.elapsed().as_millis() as i64;

    // Save to history
    let _ = state.settings
        .lock()
        .unwrap()
        .add_history(&original, &polished, &settings.language, total_ms);

    // Emit event to frontend
    let _ = app_handle.emit("recording-stopped", &polished);

    log::info!("Processing done in {}ms", total_ms);

    Ok(TranscribeResult {
        original,
        polished,
        latency_ms: total_ms,
        language: settings.language,
    })
}

/// Toggle recording - called by hotkey
#[tauri::command]
fn toggle_recording(app_handle: AppHandle, state: State<AppState>) -> Result<String, String> {
    let is_recording = *state.is_recording.lock().unwrap();

    if is_recording {
        // Stop and process
        let result = stop_and_process(app_handle.clone(), state.clone())?;
        Ok(format!("stopped:{}", result.polished))
    } else {
        // Start recording
        start_recording(state)?;
        let _ = app_handle.emit("recording-started", ());
        Ok("started".to_string())
    }
}

#[tauri::command]
fn inject(result: TranscribeResult, state: State<AppState>) -> Result<(), String> {
    inject::inject_text(&result.polished).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn get_history(limit: usize, state: State<AppState>) -> Result<Vec<HistoryEntry>, String> {
    state
        .settings
        .lock()
        .unwrap()
        .get_history(limit)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn is_currently_recording(state: State<AppState>) -> bool {
    *state.is_recording.lock().unwrap()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("Voix starting...");

    let app_state = AppState::new().expect("Failed to initialize app state");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .manage(app_state)
        .setup(|app| {
            let handle = app.handle().clone();
            let state: State<AppState> = handle.state();

            // Initialize whisper and LLM on startup
            let settings = state.settings.lock().unwrap().get_settings().ok();
            drop(settings);

            // Register global hotkey
            hotkey::start(handle.clone(), move || {
                let handle2 = handle.clone();
                let state: State<AppState> = handle2.state();
                match toggle_recording(handle2, state) {
                    Ok(msg) => log::info!("Hotkey triggered: {}", msg),
                    Err(e) => log::error!("Hotkey error: {}", e),
                }
            });

            log::info!("Voix ready");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_setting,
            get_devices,
            check_mic,
            check_llm,
            init_whisper,
            init_llm,
            start_recording,
            stop_and_process,
            toggle_recording,
            get_audio_level,
            inject,
            get_history,
            is_currently_recording,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
