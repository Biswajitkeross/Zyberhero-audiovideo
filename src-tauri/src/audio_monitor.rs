use crate::audio_capture::{start_audio_capture, AudioFrame};
use crate::audio_processor::AudioProcessor;
use crate::bad_word_detector::BadWordDetector;
use crate::speech_recognizer::SpeechRecognizer;
use crate::delay_buffer::DelayBuffer;
use crate::audio_render::start_audio_render;

use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};
use tokio::sync::mpsc;
use tauri::{AppHandle, Emitter};

// Settings
const PLAYBACK_LATENCY_MS: u32 = 12000; // Increased to 12s for slow machines for safety
const WHISPER_SAMPLE_RATE: u32 = 16_000;
// const PLAYBACK_SAMPLE_RATE: u32 = 48_000; // REMOVED

pub struct AudioMonitor {
    state: Arc<tokio::sync::Mutex<State>>,
    stop_flag: Arc<AtomicBool>,
    task: Option<tokio::task::JoinHandle<()>>,
    active_tasks: Arc<AtomicUsize>,
}

#[derive(Default)]
struct State {
    is_monitoring: bool,
    detection_count: u32,
    last_detected_word: String,
    last_detection_time: String,
}

impl AudioMonitor {
    pub fn new() -> Self {
        Self {
            state: Arc::new(tokio::sync::Mutex::new(State::default())),
            stop_flag: Arc::new(AtomicBool::new(false)),
            task: None,
            active_tasks: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub async fn start(&mut self, app: AppHandle, output_device_id: Option<String>) -> Result<String, String> {
        let mut s = self.state.lock().await;
        if s.is_monitoring { return Err("Already running".into()); }
        s.is_monitoring = true;
        drop(s);

        self.stop_flag.store(false, Ordering::Relaxed);
        let stop_flag = self.stop_flag.clone();
        
        // Reset active tasks count
        self.active_tasks.store(0, Ordering::SeqCst);
        let active_tasks_handle = self.active_tasks.clone();
        
        // --- NEW: Atomic flag to track if we are currently beeping ---
        let is_beeping = Arc::new(AtomicBool::new(false));
        let is_beeping_capture = is_beeping.clone();
        
        let state_handle = self.state.clone();

        // DYNAMIC SAMPLE RATE DETECTION
        let playback_sample_rate = crate::audio_render::get_default_device_sample_rate().unwrap_or(48_000);
        println!("🚀 [AudioMonitor] Detected Output Sample Rate: {} Hz", playback_sample_rate);

        let delay_buffer = Arc::new(DelayBuffer::new(playback_sample_rate, PLAYBACK_LATENCY_MS));
        let buf_capture = delay_buffer.clone();
        let buf_render = delay_buffer.clone();

        // 1. Playback Thread
        std::thread::spawn(move || {
            let _ = start_audio_render(buf_render, stop_flag, playback_sample_rate, output_device_id);
        });

        // 2. Monitoring Logic
        let recognizer = Arc::new(SpeechRecognizer::new().map_err(|e| e.to_string())?);
        let detector = Arc::new(BadWordDetector::new());
        let processor = Arc::new(AudioProcessor::new());
        let stop_capture = self.stop_flag.clone();

        self.task = Some(tokio::spawn(async move {
            let (tx, mut rx) = mpsc::unbounded_channel::<AudioFrame>();
            
            // --- FIXED: Passing all 3 required arguments ---
            std::thread::spawn(move || { 
                let _ = start_audio_capture(tx, stop_capture, is_beeping_capture); 
            });

            let mut whisper_accumulator: Vec<f32> = Vec::new();

            while let Some(frame) = rx.recv().await {
                // 1. DRIFT CORRECTION: Resample Raw Audio if Capture Rate != Playback Rate
                let raw_samples = if frame._raw_sample_rate != playback_sample_rate {
                   // println!("🔄 Resampling {} -> {}", frame._raw_sample_rate, playback_sample_rate);
                   processor.resample(&frame.raw_samples, frame._raw_sample_rate, playback_sample_rate)
                } else {
                   frame.raw_samples.clone()
                };

                // HQ audio push (to DelayBuffer)
                buf_capture.push(&processor.stereo_to_mono(&raw_samples, 2));

                // Whisper accumulator (Already 16kHz)
                whisper_accumulator.extend_from_slice(&frame.samples);

                // OPTIMIZED FOR REAL-TIME DETECTION:
                // Whisper REQUIRES minimum 1.0s (16,000 samples at 16kHz)
                // Accumulate 1.5s (24,000 samples) for safety margin.
                // Process.
                // Drain 0.8s (12,800 samples) - keep 0.7s overlap.
                // This gives ~500-700ms latency for detection + beeping.
                if whisper_accumulator.len() >= 16_000 { 
                    let chunk = whisper_accumulator.clone();
                    
                    whisper_accumulator.drain(0..12_800);  
                    if processor.calculate_energy(&chunk) < 0.01 { continue; }

                    // --- CONCURRENCY CONTROL (OPTIMIZED) ---
                    // Allow up to 2 concurrent Whisper tasks.
                    // Fast 1.0s chunks should process in ~300-500ms.
                    // This keeps detection frequent while avoiding overload.
                    let current_tasks = active_tasks_handle.load(Ordering::SeqCst);
                    // Allow up to 2 tasks - if 3+ active, we're overloaded, skip.
                    if current_tasks > 1 {
                        println!("⏳ Overloaded (Active Tasks: {}), skipping to maintain sync", current_tasks);
                        continue;
                    }

                    let rec = recognizer.clone();
                    let det = detector.clone();
                    let buf = buf_capture.clone();
                    let app_emit = app.clone();
                    let st = state_handle.clone();
                    let is_beeping_trigger = is_beeping.clone();
                    let active_tasks = active_tasks_handle.clone();

                    let current_write_pos = buf_capture.get_write_pos(); // SNAPSHOT POSITION
                    let capacity = buf_capture.get_capacity();

                    // Increment task count
                    active_tasks.fetch_add(1, Ordering::SeqCst);

                    tokio::spawn(async move {
                        // RAII Guard to ensure decrement happens even on panic/early return
                        struct TaskGuard(Arc<AtomicUsize>);
                        impl Drop for TaskGuard {
                            fn drop(&mut self) {
                                self.0.fetch_sub(1, Ordering::SeqCst);
                            }
                        }
                        let _guard = TaskGuard(active_tasks.clone());

                        // --- STALE CHECK BEFORE INFERENCE ---
                        // Check if the chunk we agreed to process is already ancient history.
                        let check_read_pos = buf.get_read_pos();
                        // If ReadPos has passed WritePos (snapshot), we are too late.
                        // dist_remaining = (SnapshotWrite - CurrentRead + Cap) % Cap
                        let dist_remaining = if current_write_pos >= check_read_pos {
                            current_write_pos - check_read_pos
                        } else {
                            capacity - (check_read_pos - current_write_pos)
                        };

                        if dist_remaining > capacity / 2 {
                             println!("🗑️ Dropping OLD chunk. Read {} passed WriteSnapshot {}", check_read_pos, current_write_pos);
                             return;
                        }

                        if let Some(transcript) = rec.recognize_speech_with_timestamps(&chunk).await {
                            for w in transcript.words {
                                if let Some(bad) = det.contains_bad_word(&w.word) {
                                    // Calculate precise time from the END of the audio chunk to the START of the bad word
                                    let chunk_duration_s = chunk.len() as f32 / WHISPER_SAMPLE_RATE as f32;
                                    let time_from_chunk_end_s = chunk_duration_s - w.start;
                                    
                                    // Convert to samples (Dynamic Rate)
                                    let samples_ago = (time_from_chunk_end_s * playback_sample_rate as f32) as usize;
                                    let duration_samples = ((w.end - w.start) * playback_sample_rate as f32) as usize;
                                    
                                    // PADDING & OFFSETS:
                                    // 1. Shift start 250ms EARLIER (Whisper timestamps are often slightly late)
                                    let start_padding = (0.25 * playback_sample_rate as f32) as usize;
                                    // 2. Add extra duration (600ms) to ensure the word is fully silenced
                                    let duration_padding = (0.60 * playback_sample_rate as f32) as usize;
                                    
                                    let samples_ago_adjusted = samples_ago + start_padding;
                                    let duration_adjusted = duration_samples + duration_padding;
                                    // PROCESSING OFFSET:
                                    // The 'current_write_pos' snapshot was taken AFTER the chunk was pushed.
                                    // So 'current_write_pos' corresponds to the END of the captured chunk.
                                    // Therefore, we just subtract 'samples_ago' from 'current_write_pos'.
                                    
                                    // Handle Ring Buffer Wrap-around
                                    let start_index = if current_write_pos >= samples_ago_adjusted {
                                        current_write_pos - samples_ago_adjusted
                                    } else {
                                        capacity - (samples_ago_adjusted - current_write_pos)
                                    };
                                    
                                    // No delay needed - target exact start time
                                    let delay_samples = 0;
                                    let adjusted_start = (start_index + delay_samples) % capacity;

                                    println!("🤬 BEEPING: '{}' | WritePos: {} | StartIdx: {}", bad, current_write_pos, adjusted_start);

                                    let current_read_pos = buf.get_read_pos();
                                    let buffer_distance = buf.get_distance();
                                    
                                    // Fallback: If we missed the window, beep immediately (at ReadPos)
                                    let mut final_start = adjusted_start;
                                    // Removed unused missed_deadline variable

                                    // Valid Future Window: From ReadPos to (ReadPos + Distance)
                                    // We need to check if adjusted_start falls in this window.
                                    // Circular Range Check:
                                    let end_valid = (current_read_pos + buffer_distance) % capacity;
                                    
                                    let is_valid = if end_valid >= current_read_pos {
                                        adjusted_start >= current_read_pos && adjusted_start < end_valid
                                    } else {
                                        // Wrap around case: [Read ... End] OR [0 ... Write]
                                        adjusted_start >= current_read_pos || adjusted_start < end_valid
                                    };

                                    if !is_valid {
                                        println!("⚠️ MISSED DEADLINE (Drift?): Start {} is not in valid window [Read:{} .. Write:{}] (Dist: {}). Beeping NOW.", 
                                            adjusted_start, current_read_pos, end_valid, buffer_distance);
                                        final_start = (current_read_pos + 480) % capacity; // +10ms leeway
                                    } else {
                                        println!("✅ ON TIME: Start {} is ahead of Read {} by {}", adjusted_start, current_read_pos, 
                                            if adjusted_start >= current_read_pos { adjusted_start - current_read_pos } else { capacity - (current_read_pos - adjusted_start) }
                                        );
                                    }

                                    // Mark as beeping to prevent feedback
                                    is_beeping_trigger.store(true, Ordering::SeqCst);
                                    buf.duck_and_beep_at_pos(final_start, duration_adjusted);
                                    // Reset after small delay (approximation)
                                    is_beeping_trigger.store(false, Ordering::SeqCst);

                                    let mut s = st.lock().await;
                                    s.detection_count += 1;
                                    s.last_detected_word = bad;
                                    s.last_detection_time = chrono::Local::now().to_rfc3339();
                                    app_emit.emit("bad-word-detected", vec![w.word]).ok();
                                }
                            }
                        }
                    });
                }
            }
        }));

        Ok("Monitoring started".into())
    }

    pub async fn stop(&mut self) -> Result<String, String> {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(h) = self.task.take() { h.await.ok(); }
        let mut s = self.state.lock().await;
        s.is_monitoring = false;
        Ok("Stopped".into())
    }

    pub async fn get_status(&self) -> MonitorStatus {
        let s = self.state.lock().await;
        MonitorStatus {
            is_monitoring: s.is_monitoring,
            detection_count: s.detection_count,
            last_detected_word: s.last_detected_word.clone(),
            last_detection_time: s.last_detection_time.clone(),
        }
    }
}

#[derive(Clone, serde::Serialize)]
pub struct MonitorStatus {
    pub is_monitoring: bool,
    pub detection_count: u32,
    pub last_detected_word: String,
    pub last_detection_time: String,
}