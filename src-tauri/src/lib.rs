// ✅ REGISTER ALL MODULES
mod audio_capture;
mod wasapi_capture;
mod audio_alert;
mod audio_processor;
mod bad_word_detector;
// OLD: audio_monitor uses Whisper - disabled
// mod audio_monitor;
mod simple_monitor;
mod vosk_recognizer;  // NEW: Vosk for low-latency streaming
mod delay_buffer;
mod audio_render;

// OLD: Whisper-based recognizer - disabled since whisper_rs is removed from dependencies
// We now use vosk_recognizer instead for lower latency
// #[allow(dead_code)]
// mod speech_recognizer;

use audio_alert::AudioAlert;
// OLD: AudioMonitor uses Whisper
// use audio_monitor::AudioMonitor;
use simple_monitor::SimpleAudioMonitor;
use bad_word_detector::BadWordDetector;

use std::sync::Arc;
use tauri::{AppHandle, State};
use tokio::sync::Mutex;

// =========================
// Shared Application State
// =========================

pub struct DetectorState(pub Arc<Mutex<BadWordDetector>>);
// OLD: MonitorState uses Whisper-based AudioMonitor
// pub struct MonitorState(pub Mutex<AudioMonitor>);
pub struct SimpleMonitorState(pub Mutex<SimpleAudioMonitor>);

// =========================
// Audio Alert Commands
// =========================

#[tauri::command]
fn play_alert() -> Result<String, String> {
    AudioAlert::play_warning_beep().map_err(|e| e.to_string())?;
    Ok("Alert played".to_string())
}

#[tauri::command]
fn play_double_alert() -> Result<String, String> {
    AudioAlert::play_double_beep().map_err(|e| e.to_string())?;
    Ok("Double alert played".to_string())
}

#[tauri::command]
fn play_alert_sound() -> Result<String, String> {
    AudioAlert::play_alert_sound().map_err(|e| e.to_string())?;
    Ok("Alert sound played".to_string())
}

// =========================
// Bad Word Detector Commands
// =========================

#[tauri::command]
async fn check_bad_words(
    text: String,
    state: State<'_, DetectorState>,
) -> Result<Vec<String>, String> {
    let detector = state.0.lock().await;
    Ok(detector.detect_all_bad_words(&text))
}

#[tauri::command]
async fn add_bad_word(
    word: String,
    state: State<'_, DetectorState>,
) -> Result<String, String> {
    let mut detector = state.0.lock().await;
    detector.add_word(word.clone());
    Ok(format!("Added bad word: {}", word))
}

#[tauri::command]
async fn remove_bad_word(
    word: String,
    state: State<'_, DetectorState>,
) -> Result<String, String> {
    let mut detector = state.0.lock().await;
    detector.remove_word(&word);
    Ok(format!("Removed bad word: {}", word))
}

#[tauri::command]
async fn get_all_bad_words(state: State<'_, DetectorState>) -> Result<Vec<String>, String> {
    Ok(state.0.lock().await.get_all_bad_words())
}

#[tauri::command]
async fn set_detection_enabled(
    enabled: bool,
    state: State<'_, DetectorState>,
) -> Result<String, String> {
    let mut detector = state.0.lock().await;
    detector.set_enabled(enabled);
    Ok(format!("Detection enabled: {}", enabled))
}

#[tauri::command]
async fn clear_bad_words(
    state: State<'_, DetectorState>,
) -> Result<String, String> {
    let mut detector = state.0.lock().await;
    detector.clear();
    Ok("All bad words cleared".to_string())
}

#[tauri::command]
fn get_status() -> String {
    "Ready".to_string()
}

#[tauri::command]
fn get_output_devices() -> Result<Vec<(String, String)>, String> {
    Ok(audio_render::enumerate_output_devices())
}

// =========================
// Monitoring Commands (Legacy - Deprecated)
// =========================
// (Removed unused monitoring commands to reduce dead code and warnings)

// =========================
// Simple Monitor Commands (NEW - Better for direct audio detection)
// =========================

#[tauri::command]
async fn start_simple_monitoring(
    app: AppHandle,
    state: State<'_, SimpleMonitorState>,
    detector: State<'_, DetectorState>,
) -> Result<String, String> {
    let mut monitor = state.0.lock().await;
    let detector_state = detector.0.clone();
    monitor.start(app, detector_state).await
}

#[tauri::command]
async fn stop_simple_monitoring(
    state: State<'_, SimpleMonitorState>,
) -> Result<String, String> {
    let mut monitor = state.0.lock().await;
    monitor.stop().await
}

#[tauri::command]
async fn get_simple_monitoring_status(
    state: State<'_, SimpleMonitorState>,
) -> Result<serde_json::Value, String> {
    let monitor = state.0.lock().await;
    let count = monitor.get_detection_count();
    let running = monitor.is_running();

    Ok(serde_json::json!({
        "is_monitoring": running,
        "detection_count": count,
    }))
}

// =========================
// Tauri Entry Point
// =========================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(DetectorState(Arc::new(Mutex::new(BadWordDetector::new()))))
        // Only use SimpleAudioMonitor - it has the fixes!
        .manage(SimpleMonitorState(Mutex::new(SimpleAudioMonitor::new())))
        .setup(|app| {
            // Initialize audio alert system on startup
            AudioAlert::init();
            println!("✅ [Startup] Audio alert system initialized - Using SimpleMonitor only");
            
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            play_alert,
            play_double_alert,
            play_alert_sound,
            check_bad_words,
            add_bad_word,
            remove_bad_word,
            get_all_bad_words,
            set_detection_enabled,
            clear_bad_words,
            get_output_devices,
            // Use ONLY SimpleMonitor commands:
            start_simple_monitoring,
            stop_simple_monitoring,
            get_simple_monitoring_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}