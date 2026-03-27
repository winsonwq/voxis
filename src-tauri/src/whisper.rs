//! Voix — Whisper transcription module

use anyhow::{Context, Result};
use std::path::PathBuf;
use whisper_rs::{FullParams, WhisperContext};

/// Transcription engine using whisper.cpp
pub struct WhisperEngine {
    ctx: WhisperContext,
    model_path: PathBuf,
}

impl WhisperEngine {
    /// Download and initialize the whisper model
    pub fn new(model_size: ModelSize) -> Result<Self> {
        let model_path = WhisperEngine::ensure_model(model_size)?;
        let ctx = WhisperContext::new(&model_path)
            .context("Failed to load Whisper model")?;
        Ok(Self { ctx, model_path })
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

        for chunk in response.bytes() {
            let chunk = chunk.context("Download failed")?;
            std::io::Write::write_all(&mut file, &chunk)?;
            downloaded += chunk.len() as u64;
            if total_size > 0 {
                let pct = (downloaded * 100) / total_size;
                log::info!("Download progress: {pct}%");
            }
        }

        Ok(path)
    }

    /// Transcribe audio samples (16kHz mono f32) → text
    pub fn transcribe(&mut self, samples: &[f32]) -> Result<String> {
        let mut params = FullParams::new();
        params.set_language(Some("en"));
        params.set_translate(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        self.ctx.full(params, samples)?;

        // Collect all results
        let mut result = String::new();
        let n_segments = self.ctx.n_segments();
        for i in 0..n_segments {
            let segment = self.ctx.full_get_segment_text(i);
            result.push_str(&segment);
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
        Self::Small // Balance of speed and accuracy
    }
}
