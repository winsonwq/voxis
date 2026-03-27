//! Voix — Audio capture module
//! Handles microphone input via cpal

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// AudioManager handles microphone capture
pub struct AudioManager {
    target_sample_rate: u32,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            target_sample_rate: 16000, // Whisper expects 16kHz
        }
    }

    /// List all available audio input devices
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

    /// Check if microphone is available
    pub fn is_available() -> bool {
        Self::list_devices()
            .map(|d| !d.is_empty())
            .unwrap_or(false)
    }

    /// Capture audio for given duration, returns 16kHz mono f32 samples
    pub fn capture(&self, duration_secs: f32) -> Result<Vec<f32>> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("No default input device")?;

        let config = device.default_input_config().context("Can't get input config")?;
        let source_sample_rate = config.sample_rate().0;
        let channels = config.channels();

        let samples = Arc::new(Mutex::new(Vec::<f32>::with_capacity(
            source_sample_rate as usize * duration_secs as usize,
        )));
        let samples_clone = samples.clone();

        let recording = Arc::new(AtomicBool::new(true));
        let recording_clone = recording.clone();

        let err_fn = move |err| {
            log::error!("Audio stream error: {}", err);
            recording_clone.store(false, Ordering::SeqCst);
        };

        let data_fn = move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mut buf = samples_clone.lock().unwrap();
            // Mix to mono if stereo
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

        std::thread::sleep(std::time::Duration::from_secs_f32(duration_secs));
        recording.store(false, Ordering::SeqCst);
        drop(stream);

        // Downsample to 16kHz if needed
        let raw = samples.lock().unwrap();
        if source_sample_rate == 16000 {
            Ok(raw.clone())
        } else {
            let ratio = source_sample_rate as f32 / 16000.0;
            let downsampled_len = (raw.len() as f32 / ratio) as usize;
            let mut result = Vec::with_capacity(downsampled_len);
            for i in 0..downsampled_len {
                let src_idx = (i as f32 * ratio) as usize;
                if src_idx < raw.len() {
                    result.push(raw[src_idx]);
                }
            }
            Ok(result)
        }
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}
