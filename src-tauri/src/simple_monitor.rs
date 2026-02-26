/// Simple Audio Monitor - Using VOSK for Low-Latency Bad Word Detection
/// 
/// Features:
/// - Real-time WASAPI loopback audio capture (no VB-Cable needed)
/// - Vosk streaming speech recognition (~50-100ms latency vs 1-2s for Whisper)
/// - Bad word detection with instant beep alert
/// - 3-STRIKE SYSTEM: Beep first 3 times, then PAUSE + CLOSE APP
/// - Minimal dependencies, maximum reliability

use crate::audio_capture::{start_audio_capture, AudioFrame};
use crate::audio_processor::AudioProcessor;
use crate::bad_word_detector::BadWordDetector;
use crate::vosk_recognizer::VoskSpeechRecognizer;
use crate::audio_alert::AudioAlert;

use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::os::windows::process::CommandExt;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{sleep, Duration};
use tauri::{AppHandle, Emitter};
use windows::Win32::UI::Input::KeyboardAndMouse::*;

const VOSK_SAMPLE_RATE: u32 = 16_000;

// Helper to pause system audio using media play/pause key
fn pause_system_audio() {
    unsafe {
        // ⚠️ FIX: Removed VK_MEDIA_STOP because sending both Stop + Play/Pause 
        // often causes players to glitch and RESUME immediately.
        let key = VK_MEDIA_PLAY_PAUSE; 
        let inputs = [INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: key,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(0),
                    time: 0,
                    dwExtraInfo: 0,
                }
            }
        }, INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: key,
                    wScan: 0,
                    dwFlags: KEYBD_EVENT_FLAGS(2), // KEYEVENTF_KEYUP
                    time: 0,
                    dwExtraInfo: 0,
                }
            }
        }];
        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
        println!("⌨️ [SimpleMonitor] Sent Global Media Pause (Play/Pause Toggle)");
    }
}

// Close media apps (browsers with YouTube, media players)
fn close_media_apps() {
    println!("🔪 [SimpleMonitor] Closing media applications...");
    
    // List of common media apps/browsers to close
    let apps_to_close = [
        "chrome.exe",       // Google Chrome (YouTube)
        "msedge.exe",       // Microsoft Edge
        "firefox.exe",      // Firefox
        "brave.exe",        // Brave browser
        "opera.exe",        // Opera browser
        "spotify.exe",      // Spotify
        "vlc.exe",          // VLC Media Player
        "wmplayer.exe",     // Windows Media Player
        "groove.exe",       // Groove Music
        "iTunes.exe",       // iTunes
    ];
    
    for app in &apps_to_close {
        let result = std::process::Command::new("taskkill")
            .args(["/F", "/IM", app])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW - hide console
            .output();
        
        match result {
            Ok(output) => {
                if output.status.success() {
                    println!("   ✅ Closed: {}", app);
                }
            }
            Err(_) => {}
        }
    }
}

pub struct SimpleAudioMonitor {
    stop_flag: Arc<AtomicBool>,
    task: Option<tokio::task::JoinHandle<()>>,
    detection_count: Arc<AtomicUsize>,
    is_monitoring: Arc<AtomicBool>,
}

impl SimpleAudioMonitor {
    pub fn new() -> Self {
        Self {
            stop_flag: Arc::new(AtomicBool::new(false)),
            task: None,
            detection_count: Arc::new(AtomicUsize::new(0)),
            is_monitoring: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn start(
        &mut self, 
        app: AppHandle, 
        detector_state: Arc<Mutex<BadWordDetector>>
    ) -> Result<String, String> {
        // ⚠️ CRITICAL FIX: Check if already monitoring
        if self.is_monitoring.load(Ordering::Relaxed) {
            return Err("Monitoring already active. Stop it first before starting again.".to_string());
        }

        // ⚠️ CRITICAL FIX: Stop any existing task before starting new one
        if self.task.is_some() {
            self.stop_flag.store(true, Ordering::Relaxed);
            if let Some(task) = self.task.take() {
                let _ = task.await;
            }
        }

        self.is_monitoring.store(true, Ordering::Relaxed);
        self.stop_flag.store(false, Ordering::Relaxed);
        self.detection_count.store(0, Ordering::Relaxed);

        let stop_flag = self.stop_flag.clone();
        let detection_count = self.detection_count.clone();

        println!("🎧 [Vosk] Starting real-time streaming recognition...");

        // 1. Setup channel and start capture
        let (tx, mut rx) = mpsc::unbounded_channel::<AudioFrame>();
        let is_beeping = Arc::new(AtomicBool::new(false));
        
        // Start WASAPI capture in a dedicated thread
        let capture_stop_flag = stop_flag.clone();
        let capture_is_beeping = is_beeping.clone();
        
        std::thread::spawn(move || {
            match start_audio_capture(tx, capture_stop_flag, capture_is_beeping) {
                Ok(_) => println!("🔈 [SimpleMonitor] Capture thread stopped cleanly"),
                Err(e) => eprintln!("❌ [SimpleMonitor] Capture thread error: {}", e),
            }
        });

        // 2. Spawn processing task
        let is_monitoring = self.is_monitoring.clone();
        let is_beeping_inner = is_beeping.clone();
        let app_handle = app.clone();

        self.task = Some(tokio::spawn(async move {
            let processor = AudioProcessor::new();
            
            // Initialize Vosk
            let recognizer_res = VoskSpeechRecognizer::new();
            if let Err(e) = recognizer_res {
                eprintln!("❌ [SimpleMonitor] Failed to load Vosk: {}", e);
                is_monitoring.store(false, Ordering::Relaxed);
                return;
            }
            let recognizer = recognizer_res.unwrap();
            
            // Create streaming recognizer
            let stream_res = recognizer.create_stream();
            if let Err(e) = stream_res {
                eprintln!("❌ [SimpleMonitor] Failed to create Vosk stream: {}", e);
                is_monitoring.store(false, Ordering::Relaxed);
                return;
            }
            let mut vosk_stream = stream_res.unwrap();

            // 3. Main detection loop - STREAMING with Vosk
            let mut pending_samples: Vec<f32> = Vec::new();
            let mut last_detected_word = String::new();
            let mut last_detect_time = std::time::Instant::now();
            const CHUNK_SIZE: usize = 800; // 50ms at 16kHz - FASTER processing for lower latency!

            println!("✅ [Vosk] Streaming recognition active - listening for bad words...");

            let mut frame_count = 0u64;
            
            while !stop_flag.load(Ordering::Relaxed) {
                // 1. Receive audio frames
                let mut new_samples = Vec::new();
                if pending_samples.is_empty() {
                    if let Some(frame) = rx.recv().await {
                        frame_count += 1;
                        if frame_count % 100 == 1 {
                            println!("📥 [Vosk] Received frame #{}, samples: {}, rate: {}", 
                                frame_count, frame.samples.len(), frame._sample_rate);
                        }
                        // Note: frame.samples is already at 16kHz mono from wasapi_capture
                        // Only resample if _sample_rate != 16000 (should rarely happen)
                        new_samples.extend_from_slice(&if frame._sample_rate != VOSK_SAMPLE_RATE {
                            processor.resample(&frame.samples, frame._sample_rate, VOSK_SAMPLE_RATE)
                        } else {
                            frame.samples.clone()
                        });
                    } else { 
                        println!("⚠️ [Vosk] Channel closed, no more frames");
                        break; 
                    }
                }

                // Non-blocking drain to stay real-time
                while let Ok(f) = rx.try_recv() {
                    new_samples.extend_from_slice(&if f._sample_rate != VOSK_SAMPLE_RATE {
                        processor.resample(&f.samples, f._sample_rate, VOSK_SAMPLE_RATE)
                    } else {
                        f.samples.clone()
                    });
                }

                pending_samples.extend_from_slice(&new_samples);

                // 2. Process in 100ms chunks for real-time streaming
                static CHUNK_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                
                while pending_samples.len() >= CHUNK_SIZE {
                    // Skip if we're currently beeping
                    if is_beeping_inner.load(Ordering::SeqCst) {
                        pending_samples.drain(0..CHUNK_SIZE);
                        vosk_stream.reset();
                        continue;
                    }

                    let chunk: Vec<f32> = pending_samples.drain(0..CHUNK_SIZE).collect();
                    
                    // Check energy - but STILL feed to Vosk even if low energy
                    // Just use energy for logging
                    let energy = processor.calculate_energy(&chunk);
                    
                    let chunk_num = CHUNK_COUNT.fetch_add(1, Ordering::Relaxed);
                    if chunk_num % 50 == 0 {
                        println!("📊 [Vosk] Chunk #{}, energy: {:.6}", chunk_num, energy);
                    }
                    
                    // Skip only very silent chunks (energy < 0.0001)
                    if energy < 0.0001 { continue; }

                    // Feed to Vosk and get partial result
                    if let Some(partial_text) = vosk_stream.accept_waveform(&chunk) {
                        // Log every partial result for debugging
                        if !partial_text.is_empty() {
                            println!("🎤 [Vosk] Partial: '{}'", partial_text);
                        }
                        
                        // Check partial result for bad words in real-time
                        let detector = detector_state.lock().await;
                        
                        for word in partial_text.split_whitespace() {
                            let cleaned = word.trim().to_lowercase();
                            if cleaned.is_empty() { continue; }
                            
                            // Deduplication: Don't re-alert the same word within 500ms
                            if cleaned == last_detected_word && last_detect_time.elapsed() < Duration::from_millis(500) {
                                continue;
                            }

                            if let Some(bad_word) = detector.contains_bad_word(&cleaned) {
                                last_detected_word = cleaned.clone();
                                last_detect_time = std::time::Instant::now();
                                
                                let count = detection_count.fetch_add(1, Ordering::SeqCst) + 1;
                                
                                println!("🚨 [PROFANITY #{}/3] Detected: '{}'", count, bad_word);
                                let _ = app_handle.emit("bad-word-detected", serde_json::json!({
                                    "word": bad_word,
                                    "count": count,
                                    "max": 3
                                }));

                                is_beeping_inner.store(true, Ordering::SeqCst);
                                
                                if count < 3 {
                                    // First 2 detections: INSTANT beep (no waiting)
                                    let _ = AudioAlert::play_warning_beep();
                                    // Reduced delay for faster response
                                    sleep(Duration::from_millis(300)).await;
                                } else {
                                    // 3rd detection: Double beep + PAUSE + CLOSE MEDIA APPS
                                    let _ = AudioAlert::play_double_beep();
                                    sleep(Duration::from_millis(200)).await;
                                    
                                    // First pause the audio
                                    pause_system_audio();
                                    sleep(Duration::from_millis(100)).await;
                                    
                                    // Close all media applications (browsers, players)
                                    close_media_apps();
                                    
                                    println!("🛑 [3 STRIKES] Media apps closed! Exiting monitor app...");
                                    let _ = app_handle.emit("app-closing", "3 bad words detected - media apps closed");
                                    
                                    // Give UI time to show message, then exit
                                    sleep(Duration::from_millis(1000)).await;
                                    std::process::exit(0);
                                }
                                
                                is_beeping_inner.store(false, Ordering::SeqCst);
                                vosk_stream.reset(); // Reset recognizer after detection
                                break;
                            }
                        }
                    }
                }
            }

            is_monitoring.store(false, Ordering::Relaxed);
            println!("✅ [SimpleMonitor] Stopped");
        }));

        Ok("Monitoring started with Vosk (low-latency streaming)".to_string())
    }

    pub async fn stop(&mut self) -> Result<String, String> {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
        self.is_monitoring.store(false, Ordering::Relaxed);
        Ok("Monitoring stopped".to_string())
    }

    pub fn get_detection_count(&self) -> usize {
        self.detection_count.load(Ordering::Relaxed)
    }

    pub fn is_running(&self) -> bool {
        self.is_monitoring.load(Ordering::Relaxed)
    }
}
