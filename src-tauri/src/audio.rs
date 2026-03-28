//! Voix — Audio capture module
//! Handles microphone input via cpal with start/stop control

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Shared state between audio thread and caller
pub struct AudioState {
    pub samples: Arc<Mutex<Vec<f32>>>,
    pub recording: Arc<AtomicBool>,
    pub sample_rate: u32,
}

impl AudioState {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            recording: Arc::new(AtomicBool::new(false)),
            sample_rate,
        }
    }
}

/// AudioManager handles microphone capture with start/stop
pub struct AudioManager {
    target_sample_rate: u32,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            target_sample_rate: 16000,
        }
    }

    pub fn list_devices() -> Result<Vec<String>> {
        let host = cpal::default_host();
        let mut devices = Vec::new();
        for device in host.input_devices().into_iter().flatten() {
            if let Ok(name) = device.name() {
                devices.push(name);
            }
        }
        Ok(devices)
    }

    pub fn is_available() -> bool {
        Self::list_devices().map(|d| !d.is_empty()).unwrap_or(false)
    }

    /// Start capturing audio. Returns a shared state object.
    /// The stream is kept alive inside the state.
    pub fn start_capture(&self) -> Result<(AudioState, cpal::Stream)> {
        let host = cpal::default_host();
        let device = host.default_input_device().context("No default input device")?;
        let config = device.default_input_config().context("Can't get input config")?;
        let source_sample_rate = config.sample_rate().0;
        let channels = config.channels();

        let state = AudioState::new(source_sample_rate);
        let samples = state.samples.clone();
        let recording = state.recording.clone();
        recording.store(true, Ordering::SeqCst);

        let samples_clone = samples.clone();
        let recording_clone = recording.clone();

        let err_fn = move |err| {
            log::error!("Audio stream error: {}", err);
            recording_clone.store(false, Ordering::SeqCst);
        };

        let data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            if !recording.load(Ordering::SeqCst) {
                return;
            }
            let mut buf = samples_clone.lock().unwrap();
            if channels == 1 {
                buf.extend_from_slice(data);
            } else {
                for chunk in data.chunks(channels as usize) {
                    let avg: f32 = chunk.iter().sum::<f32>() / channels as f32;
                    buf.push(avg);
                }
            }
        };

        let stream = device.build_input_stream(&config.into(), data_fn, err_fn, None)?;
        stream.play()?;

        Ok((state, stream))
    }

    /// Stop capture and return downsampled 16kHz mono samples
    pub fn stop_capture(state: &AudioState) -> Vec<f32> {
        state.recording.store(false, Ordering::SeqCst);
        std::thread::sleep(std::time::Duration::from_millis(100));

        let raw = state.samples.lock().unwrap();
        let source_sample_rate = state.sample_rate;

        if source_sample_rate == 16000 || raw.is_empty() {
            return raw.clone();
        }

        // Downsample to 16kHz
        let ratio = source_sample_rate as f32 / 16000.0;
        let downsampled_len = (raw.len() as f32 / ratio) as usize;
        let mut result = Vec::with_capacity(downsampled_len);
        for i in 0..downsampled_len {
            let src_idx = (i as f32 * ratio) as usize;
            if src_idx < raw.len() {
                result.push(raw[src_idx]);
            }
        }
        result
    }

    /// Get current audio level (0.0 - 1.0) for visualization
    pub fn get_level(state: &AudioState) -> f32 {
        let samples = state.samples.lock().unwrap();
        if samples.is_empty() {
            return 0.0;
        }
        let last = samples.iter().rev().take(1600);
        let sum: f32 = last.map(|s| s.abs()).sum();
        let avg = sum / samples.len().min(1600) as f32;
        (avg * 10.0).min(1.0)
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}
