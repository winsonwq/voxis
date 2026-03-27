//! Voix — LLM text polishing module

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum LLMProvider {
    Ollama { base_url: String, model: String },
    OpenAI { api_key: String, model: String },
}

impl LLMProvider {
    pub fn ollama(model: &str) -> Self {
        Self::Ollama {
            base_url: "http://localhost:11434".to_string(),
            model: model.to_string(),
        }
    }

    pub fn openai(api_key: &str, model: &str) -> Self {
        Self::OpenAI {
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }
}

pub struct LLMPolisher {
    client: Client,
    provider: LLMProvider,
}

impl LLMPolisher {
    pub fn new(provider: LLMProvider) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();
        Self { client, provider }
    }

    pub fn polish(&self, text: &str, level: PolishLevel) -> Result<String> {
        let prompt = match level {
            PolishLevel::Light => format!(
                "Clean up this transcribed speech. Remove filler words (um, uh, like, you know). \
                Keep the original voice and meaning. Return only the cleaned text:\n\n{text}"
            ),
            PolishLevel::Medium => format!(
                "You are editing transcribed voice notes. Remove all filler words, \
                repetitions, and disfluencies (um, uh, like, you know, I mean, well, etc). \
                Fix any obvious transcription errors. Keep the original voice. \
                Return only the cleaned text:\n\n{text}"
            ),
            PolishLevel::Strong => format!(
                "You are a professional editor cleaning up voice transcription. Remove ALL filler \
                words, repetitions, false starts, and disfluencies (um, uh, like, you know, \
                I mean, well, actually, maybe, sort of, kind of, basically, literally, etc). \
                Fix grammar, capitalization, and punctuation. Preserve the original voice \
                and meaning. Return only the cleaned text:\n\n{text}"
            ),
        };

        match &self.provider {
            LLMProvider::Ollama { base_url, model } => self.ollama_polish(&prompt, base_url, model),
            LLMProvider::OpenAI { api_key, model } => {
                self.openai_polish(&prompt, api_key, model)
            }
        }
    }

    fn ollama_polish(&self, prompt: &str, base_url: &str, model: &str) -> Result<String> {
        #[derive(Serialize)]
        struct OllamaRequest {
            model: String,
            prompt: String,
            stream: bool,
        }

        #[derive(Deserialize)]
        struct OllamaResponse {
            response: String,
        }

        let url = format!("{base_url}/api/generate");
        let resp = self
            .client
            .post(&url)
            .json(&OllamaRequest {
                model: model.to_string(),
                prompt: prompt.to_string(),
                stream: false,
            })
            .send()
            .context("Ollama request failed")?;

        let data: OllamaResponse = resp.json().context("Failed to parse Ollama response")?;
        Ok(data.response.trim().to_string())
    }

    fn openai_polish(&self, prompt: &str, api_key: &str, model: &str) -> Result<String> {
        #[derive(Serialize)]
        struct OpenAIRequest {
            model: String,
            messages: Vec<serde_json::Value>,
            max_tokens: u32,
        }

        #[derive(Deserialize)]
        struct OpenAIChoice {
            message: OpenAIMessage,
        }

        #[derive(Deserialize)]
        struct OpenAIMessage {
            content: String,
        }

        #[derive(Deserialize)]
        struct OpenAIResponse {
            choices: Vec<OpenAIChoice>,
        }

        let body = OpenAIRequest {
            model: model.to_string(),
            messages: vec![serde_json::json!({"role": "user", "content": prompt})],
            max_tokens: 500,
        };

        let resp = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {api_key}"))
            .json(&body)
            .send()
            .context("OpenAI API request failed")?;

        let data: OpenAIResponse =
            resp.json().context("Failed to parse OpenAI response")?;
        Ok(data
            .choices
            .first()
            .map(|c| c.message.content.trim().to_string())
            .unwrap_or_default())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PolishLevel {
    Light,
    Medium,
    Strong,
}

impl Default for PolishLevel {
    fn default() -> Self {
        Self::Medium
    }
}
