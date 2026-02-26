use crate::audio_capture::{start_audio_capture, AudioFrame};
use crate::realtime_detector::RealtimeDetector;
use crate::audio_alert::AudioAlert;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::sync::mpsc;
use tauri::{AppHandle, Emitter};

const WHISPER_SAMPLE_RATE: u32 = 16_000;
const CHUNK_SIZE_MS: u32 = 500; // Process every 500ms
const CHUNK_SIZE_SAMPLES: usize = (WHISPER_SAMPLE_RATE as usize * CHUNK_SIZE_MS as usize) / 1000; // 8,000 samples

pub struct AudioMonitorSimple {
    stop_flag: Arc<AtomicBool>,
    task: Option<tokio::task::JoinHandle<()>>,
}

impl AudioMonitorSimple {
    pub fn new() -> Self {
        Self {
            stop_flag: Arc::new(AtomicBool::new(false)),
            task: None,
        }
    }

    pub async fn start(&mut self, app: AppHandle) -> Result<String, String> {
        self.stop_flag.store(false, Ordering::Relaxed);
        let stop_flag = self.stop_flag.clone();

        let detector = RealtimeDetector::new()?;
        let detector_clone = detector.clone();

        // Start audio capture
        let (tx, mut rx) = mpsc::unbounded_channel::<AudioFrame>();
        let stop_capture = stop_flag.clone();

        std::thread::spawn(move || {
            let is_beeping = Arc::new(AtomicBool::new(false));
            let _ = start_audio_capture(tx, stop_capture, is_beeping);
        });

        self.task = Some(tokio::spawn(async move {
            let mut audio_buffer: Vec<f32> = Vec::new();
            let audio_alert = AudioAlert::new();

            while let Some(frame) = rx.recv().await {
                // Resample to 16kHz if needed (Whisper requirement)
                let samples_16k = if frame._raw_sample_rate != WHISPER_SAMPLE_RATE {
                    // Simple linear interpolation resampling
                    resample_to_16k(&frame.samples, frame._raw_sample_rate)
                } else {
                    frame.samples.clone()
                };

                audio_buffer.extend_from_slice(&samples_16k);

                // Process in 500ms chunks (8,000 samples at 16kHz)
                while audio_buffer.len() >= CHUNK_SIZE_SAMPLES {
                    let chunk: Vec<f32> = audio_buffer.drain(0..CHUNK_SIZE_SAMPLES).collect();

                    // Detect bad words in this chunk
                    let results = detector_clone.detect_bad_words(&chunk).await;

                    for result in results {
                        if result.is_bad {
                            // 🔊 BEEP + MUTE happens HERE, in real-time
                            println!("🔔 BEEPING for bad word: {}", result.word);
                            audio_alert.play_alert();
                            app.emit("bad-word-detected", &result.word).ok();
                        }
                    }
                }
            }
        }));

        Ok("Monitoring started".to_string())
    }

    pub async fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
    }
}

fn resample_to_16k(samples: &[f32], original_rate: u32) -> Vec<f32> {
    if original_rate == 16_000 {
        return samples.to_vec();
    }

    let ratio = 16_000.0 / original_rate as f32;
    let new_len = (samples.len() as f32 * ratio) as usize;
    let mut resampled = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_pos = i as f32 / ratio;
        let idx = src_pos as usize;

        if idx >= samples.len() - 1 {
            if idx < samples.len() {
                resampled.push(samples[idx]);
            }
            break;
        }

        let frac = src_pos - idx as f32;
        let interpolated = samples[idx] * (1.0 - frac) + samples[idx + 1] * frac;
        resampled.push(interpolated);
    }

    resampled
}

impl Drop for AudioMonitorSimple {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}
