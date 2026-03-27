mod audio;
mod inject;
mod polish;
mod settings;
mod whisper;

use anyhow::Result;
use audio::AudioManager;
use polish::{LLMPolisher, LLMProvider, PolishLevel};
use serde::{Deserialize, Serialize};
use settings::{HistoryEntry, Settings, SettingsStore};
use std::sync::Mutex;
use std::time::Instant;
use tauri::{Manager, State};
use whisper::{ModelSize, WhisperEngine};

pub struct AppState {
    settings: Mutex<SettingsStore>,
    whisper: Mutex<Option<WhisperEngine>>,
    llm: Mutex<Option<LLMPolisher>>,
    is_recording: Mutex<bool>,
}

impl AppState {
    fn new() -> Result<Self> {
        Ok(Self {
            settings: Mutex::new(SettingsStore::new()?),
            whisper: Mutex::new(None),
            llm: Mutex::new(None),
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
    let provider = if settings.llm_provider == "openai" {
        LLMProvider::openai(&"", &settings.llm_model) // Will be checked differently
    } else {
        LLMProvider::ollama(&settings.llm_model)
    };
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .unwrap_or_default();

    let url = match &provider {
        LLMProvider::Ollama { base_url, .. } => format!("{base_url}/api/tags"),
        LLMProvider::OpenAI { .. } => "https://api.openai.com/v1/models".to_string(),
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
        // For OpenAI, we don't store the API key in settings, use env var
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
        LLMProvider::openai(&api_key, &settings.llm_model)
    } else {
        LLMProvider::ollama(&settings.llm_model)
    };

    log::info!("Initializing LLM provider");
    *state.llm.lock().unwrap() = Some(LLMPolisher::new(provider));
    Ok(())
}

#[derive(Debug, Serialize)]
struct TranscribeResult {
    original: String,
    polished: String,
    latency_ms: i64,
    language: String,
}

#[tauri::command]
fn transcribe_and_polish(duration_secs: f32, state: State<AppState>) -> Result<TranscribeResult, String> {
    let start = Instant::now();

    // 1. Capture audio
    let audio = AudioManager::new();
    let samples = audio.capture(duration_secs).map_err(|e| e.to_string())?;

    // 2. Transcribe
    let mut whisper_guard = state.whisper.lock().unwrap();
    let whisper = whisper_guard.as_mut().ok_or("Whisper not initialized")?;
    let original = whisper.transcribe(&samples).map_err(|e| e.to_string())?;
    drop(whisper_guard);

    // 3. Polish
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

    // 4. Save to history
    let _ = state.settings.lock().unwrap().add_history(&original, &polished, &settings.language, total_ms);

    Ok(TranscribeResult {
        original,
        polished,
        latency_ms: total_ms,
        language: settings.language,
    })
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
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_setting,
            get_devices,
            check_mic,
            check_llm,
            init_whisper,
            init_llm,
            transcribe_and_polish,
            inject,
            get_history,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
