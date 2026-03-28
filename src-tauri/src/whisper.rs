//! Voix — Whisper transcription module
//! Handles local speech-to-text using whisper.cpp

use anyhow::{Context, Result};
use std::path::PathBuf;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState};

/// Transcription engine using whisper.cpp
pub struct WhisperEngine {
    ctx: WhisperContext,
    state: WhisperState,
    model_path: PathBuf,
}

impl WhisperEngine {
    /// Download and initialize the whisper model
    pub fn new(model_size: ModelSize) -> Result<Self> {
        let model_path = WhisperEngine::ensure_model(model_size)?;
        let ctx = WhisperContext::new_with_params(
            &model_path,
            WhisperContextParameters::new(),
        )
        .context("Failed to load Whisper model")?;
        let state = ctx.create_state().context("Failed to create Whisper state")?;
        Ok(Self { ctx, state, model_path })
    }

    /// Download model if not cached
    fn ensure_model(size: ModelSize) -> Result<PathBuf> {
        let cache_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("voxis")
            .join("models");

        std::fs::create_dir_all(&cache_dir)?;

        let filename = match size {
            ModelSize::Tiny => "ggml-tiny.en.bin",
            ModelSize::Base => "ggml-base.en.bin",
            ModelSize::Small => "ggml-small.en.bin",
            ModelSize::Medium => "ggml-medium.bin",
        };

        let path = cache_dir.join(filename);
        if path.exists() {
            return Ok(path);
        }

        // Download model
        let url = format!(
            "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{filename}"
        );
        log::info!("Downloading Whisper model from {url}");

        let response = reqwest::blocking::Client::new()
            .get(&url)
            .send()
            .context("Failed to download model")?;

        let total_size = response.content_length().unwrap_or(0);
        let mut file = std::fs::File::create(&path).context("Failed to create model file")?;
        let mut downloaded: u64 = 0;

        let bytes = response.bytes().context("Download failed")?;
        let bytes_total = bytes.len() as u64;

        for chunk in bytes.chunks(65536) {
            std::io::Write::write_all(&mut file, chunk)?;
            downloaded += chunk.len() as u64;
            if bytes_total > 0 {
                let pct = (downloaded * 100) / bytes_total;
                log::info!("Download progress: {pct}%");
            }
        }

        log::info!("Whisper model downloaded: {}", path.display());
        Ok(path)
    }

    /// Transcribe audio samples (16kHz mono f32) → text
    pub fn transcribe(&mut self, samples: &[f32]) -> Result<String> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_translate(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        self.state.full(params, samples)?;

        // Collect all results
        let mut result = String::new();
        let n_segments = self.state.full_n_segments();
        for i in 0..n_segments {
            if let Some(segment) = self.state.get_segment(i) {
                if let Ok(text) = segment.to_str() {
                    result.push_str(text);
                }
            }
        }

        Ok(result.trim().to_string())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ModelSize {
    Tiny,
    Base,
    Small,
    Medium,
}

impl Default for ModelSize {
    fn default() -> Self {
        Self::Small
    }
}
