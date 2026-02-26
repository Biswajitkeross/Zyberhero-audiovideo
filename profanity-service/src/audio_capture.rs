//! WASAPI Loopback Audio Capture
//! Captures system audio output for speech recognition

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, StreamConfig};

pub struct WasapiCapture {
    _stream: cpal::Stream,
    should_stop: Arc<AtomicBool>,
}

impl WasapiCapture {
    pub fn start<F>(mut callback: F) -> Result<Self, Box<dyn std::error::Error>>
    where
        F: FnMut(Vec<i16>) + Send + 'static,
    {
        let host = cpal::host_from_id(cpal::HostId::Wasapi)?;
        
        // Get default output device for loopback capture
        let device = host
            .default_output_device()
            .ok_or("No output device found")?;
        
        println!("🎧 Using device: {}", device.name().unwrap_or_default());
        
        // Get supported config
        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        
        println!("🎧 Config: {}Hz, {} channels", sample_rate, channels);
        
        let should_stop = Arc::new(AtomicBool::new(false));
        let should_stop_clone = should_stop.clone();
        
        // Buffer for collecting samples
        let buffer_size = (sample_rate / 20) as usize; // 50ms chunks
        let mut buffer: Vec<f32> = Vec::with_capacity(buffer_size * channels);
        
        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                let config: StreamConfig = config.into();
                device.build_input_stream(
                    &config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if should_stop_clone.load(Ordering::Relaxed) {
                            return;
                        }
                        
                        buffer.extend_from_slice(data);
                        
                        // Process when we have enough samples
                        while buffer.len() >= buffer_size * channels {
                            // Extract chunk
                            let chunk: Vec<f32> = buffer.drain(..buffer_size * channels).collect();
                            
                            // Downmix to mono and resample to 16kHz
                            let samples_16k = downsample_to_16k_mono(&chunk, channels, sample_rate);
                            
                            // Call the callback with i16 samples
                            callback(samples_16k);
                        }
                    },
                    |err| eprintln!("❌ Stream error: {}", err),
                    None,
                )?
            }
            _ => return Err("Unsupported sample format".into()),
        };
        
        stream.play()?;
        
        Ok(Self {
            _stream: stream,
            should_stop,
        })
    }
}

impl Drop for WasapiCapture {
    fn drop(&mut self) {
        self.should_stop.store(true, Ordering::SeqCst);
    }
}

/// Downsample from source rate stereo to 16kHz mono i16
fn downsample_to_16k_mono(samples: &[f32], channels: usize, src_rate: u32) -> Vec<i16> {
    // 1. Convert to mono by averaging channels
    let mono: Vec<f32> = samples
        .chunks(channels)
        .map(|chunk| {
            let sum: f32 = chunk.iter().sum();
            sum / channels as f32
        })
        .collect();
    
    // 2. Resample to 16kHz
    let ratio = src_rate as f32 / 16000.0;
    let output_len = (mono.len() as f32 / ratio) as usize;
    let mut resampled = Vec::with_capacity(output_len);
    
    for i in 0..output_len {
        let src_idx = (i as f32 * ratio) as usize;
        if src_idx < mono.len() {
            // Apply gain boost (2x) and convert to i16
            let sample = mono[src_idx] * 2.0;
            // Clamp to prevent overflow
            let clamped = sample.clamp(-1.0, 1.0);
            let as_i16 = (clamped * 32767.0) as i16;
            resampled.push(as_i16);
        }
    }
    
    resampled
}
