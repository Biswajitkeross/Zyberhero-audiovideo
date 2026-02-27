//! Standalone Profanity and NSFW Detection Service

mod audio_capture;
mod vosk_recognizer;
mod bad_word_detector;
mod audio_alert;
mod pipe_logger;
mod tcp_logger;
mod video_capture;
mod nsfw_detector;
mod blur_overlay;
mod zmq_alert_sender;

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::process::Command;
use std::os::windows::process::CommandExt;

use audio_capture::WasapiCapture;
use vosk_recognizer::VoskStream;
use bad_word_detector::BadWordDetector;
use audio_alert::AudioAlert;
use tcp_logger::TcpLogger;
use video_capture::{ScreenCapture, CaptureConfig};
use nsfw_detector::NsfwDetector;
use blur_overlay::BlurOverlay;
use zmq_alert_sender::ZmqAlertSender;

const CREATE_NO_WINDOW: u32 = 0x08000000;

static RUNNING: AtomicBool = AtomicBool::new(true);
static STRIKE_COUNT: AtomicU32 = AtomicU32::new(0);
static RESET_VOSK: AtomicBool = AtomicBool::new(false);

const MAX_STRIKES: u32 = 3;
const VOSK_MODEL_PATH: &str = "vosk-model-small-en-us-0.15";
const NSFW_MODEL_PATH: &str = "nsfw-model.onnx";
const TCP_PORT: u16 = 5561;  // TCP logger on 5561

const ENABLE_AUDIO: bool = true;
const ENABLE_VIDEO: bool = true;  // NSFW detection enabled
const ZMQ_ENDPOINT: &str = "tcp://127.0.0.1:5559"; // ZMQ PUB endpoint on 5559

fn main() {
    println!("Profanity and NSFW Detection Service Starting...");
    
    let logger = Arc::new(TcpLogger::new(Some(TCP_PORT)));
    logger.log("SERVICE_STARTED", "Service initialized");
    
    // Initialize ZMQ Alert Sender
    let zmq_sender = Arc::new(ZmqAlertSender::new(ZMQ_ENDPOINT));
    
    let logger_clone = logger.clone();
    ctrlc::set_handler(move || {
        println!("Shutting down...");
        logger_clone.log("SERVICE_STOPPED", "Shutting down");
        RUNNING.store(false, Ordering::SeqCst);
    }).ok();
    
    let blur_overlay = if ENABLE_VIDEO {
        match BlurOverlay::new() {
            Ok(o) => {
                println!("🔲 Blur overlay ready");
                Some(Arc::new(o))
            },
            Err(e) => {
                eprintln!("⚠️ Blur overlay failed: {}", e);
                None
            }
        }
    } else { None };
    
    let last_text = Arc::new(parking_lot::Mutex::new(String::new()));
    
    let _audio = if ENABLE_AUDIO {
        start_audio_monitoring(logger.clone(), last_text.clone(), zmq_sender.clone())
    } else { None };
    
    let _video = if ENABLE_VIDEO {
        start_video_monitoring(logger.clone(), blur_overlay.clone(), last_text.clone(), zmq_sender.clone())
    } else { None };
    
    let vosk_reset: Option<Arc<parking_lot::Mutex<VoskStream>>> = if ENABLE_AUDIO {
        VoskStream::new(VOSK_MODEL_PATH).ok().map(|v| Arc::new(parking_lot::Mutex::new(v)))
    } else { None };
    
    println!("Service running...");
    
    while RUNNING.load(Ordering::SeqCst) {
        if RESET_VOSK.load(Ordering::SeqCst) {
            if let Some(ref v) = vosk_reset {
                v.lock().reset();
            }
            RESET_VOSK.store(false, Ordering::SeqCst);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    
    println!("Service stopped");
}

fn start_audio_monitoring(
    logger: Arc<TcpLogger>,
    last_text: Arc<parking_lot::Mutex<String>>,
    zmq_sender: Arc<ZmqAlertSender>,
) -> Option<WasapiCapture> {
    let vosk = match VoskStream::new(VOSK_MODEL_PATH) {
        Ok(v) => Arc::new(parking_lot::Mutex::new(v)),
        Err(e) => { eprintln!("Vosk error: {}", e); return None; }
    };
    
    let detector = Arc::new(BadWordDetector::new());
    let alert = Arc::new(AudioAlert::new());
    let last_det = Arc::new(parking_lot::Mutex::new(Instant::now()));
    
    let v = vosk.clone();
    let d = detector.clone();
    let a = alert.clone();
    let l = logger.clone();
    let ld = last_det.clone();
    let lt = last_text.clone();
    let zmq = zmq_sender.clone();
    
    let cb = move |samples: Vec<i16>| {
        if !RUNNING.load(Ordering::SeqCst) { return; }
        
        let mut vg = v.lock();
        if let Some(text) = vg.process_audio(&samples) {
            if text.is_empty() { return; }
            
            {
                let mut last = lt.lock();
                if *last == text { return; }
                *last = text.clone();
            }
            
            let bw = d.detect_all_bad_words(&text);
            if !bw.is_empty() {
                let mut ldt = ld.lock();
                if ldt.elapsed() < Duration::from_millis(500) { return; }
                *ldt = Instant::now();
                drop(ldt);
                
                let s = STRIKE_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
                let w = bw.first().unwrap();
                println!("AUDIO: '{}' Strike {}/{}", w, s, MAX_STRIKES);
                l.log("BAD_WORD", &format!("{}|{}|{}", w, s, MAX_STRIKES));
                a.beep();
                
                // Send ZMQ alert to C++ agent
                zmq.send_audio_blocked_alert(w, s, MAX_STRIKES);
                
                if s >= MAX_STRIKES { handle_max_strikes(&l, &lt, "AUDIO"); }
            }
        }
    };
    
    WasapiCapture::start(cb).ok()
}

fn start_video_monitoring(
    logger: Arc<TcpLogger>,
    blur: Option<Arc<BlurOverlay>>,
    last_text: Arc<parking_lot::Mutex<String>>,
    zmq_sender: Arc<ZmqAlertSender>,
) -> Option<ScreenCapture> {
    println!("📹 Loading NSFW model from: {}", NSFW_MODEL_PATH);
    let nsfw = match NsfwDetector::new(NSFW_MODEL_PATH) {
        Ok(n) => {
            println!("✅ NSFW detector ready (NudeNet 320n)");
            Arc::new(parking_lot::Mutex::new(n))
        },
        Err(e) => { eprintln!("❌ NSFW model error: {}", e); return None; }
    };
    
    let last_det = Arc::new(parking_lot::Mutex::new(Instant::now()));
    let config = CaptureConfig { fps: 2, scale: 0.5 };
    println!("📺 Screen capture starting ({}fps, {:.0}% scale)", config.fps, config.scale * 100.0);
    
    let n = nsfw.clone();
    let l = logger.clone();
    let ld = last_det.clone();
    let b = blur.clone();
    let lt = last_text.clone();
    let zmq = zmq_sender.clone();
    
    let cb = move |frame: video_capture::CapturedFrame| {
        if !RUNNING.load(Ordering::SeqCst) { return; }
        
        let mut ng = n.lock();
        if let Ok(result) = ng.detect(&frame.data, frame.width, frame.height) {
            if result.is_nsfw {
                let mut ldt = ld.lock();
                if ldt.elapsed() < Duration::from_secs(2) { return; }
                *ldt = Instant::now();
                drop(ldt);
                
                let s = STRIKE_COUNT.fetch_add(1, Ordering::SeqCst) + 1;
                println!("🚨 VIDEO: {} ({:?}) {:.0}% Strike {}/{}", result.class_name, result.category, result.confidence * 100.0, s, MAX_STRIKES);
                l.log("NSFW_DETECTED", &format!("{}|{:?}|{:.2}|{}|{}", result.class_name, result.category, result.confidence, s, MAX_STRIKES));
                
                // Send ZMQ alert to C++ agent for video detection
                zmq.send_video_blocked_alert(&result.class_name, &format!("{:?}", result.category), result.confidence, s, MAX_STRIKES);
                
                if let Some(ref ov) = b {
                    ov.show();
                    let ovh = ov.clone();
                    std::thread::spawn(move || { std::thread::sleep(Duration::from_secs(3)); ovh.hide(); });
                }
                
                if s >= MAX_STRIKES { handle_max_strikes(&l, &lt, "VIDEO"); }
            }
        }
    };
    
    ScreenCapture::start(config, cb).ok()
}

fn handle_max_strikes(logger: &Arc<TcpLogger>, last_text: &Arc<parking_lot::Mutex<String>>, source: &str) {
    println!("MAX STRIKES ({}) - Closing apps...", source);
    logger.log("MAX_STRIKES", &format!("Triggered by {}", source));
    close_media_apps();
    logger.log("APPS_CLOSED", "Media apps terminated");
    STRIKE_COUNT.store(0, Ordering::SeqCst);
    RESET_VOSK.store(true, Ordering::SeqCst);
    { let mut l = last_text.lock(); l.clear(); }
    println!("State reset");
    logger.log("STATE_RESET", "Ready for new session");
}

fn close_media_apps() {
    let apps = [
        ("chrome.exe", "Chrome"), ("msedge.exe", "Edge"), ("firefox.exe", "Firefox"),
        ("spotify.exe", "Spotify"), ("vlc.exe", "VLC"), ("wmplayer.exe", "WMP"),
        ("potplayer.exe", "PotPlayer"), ("mpc-hc64.exe", "MPC"),
    ];
    for (p, n) in apps {
        if let Ok(o) = Command::new("taskkill").args(["/F", "/IM", p]).creation_flags(CREATE_NO_WINDOW).output() {
            if o.status.success() { println!("  Closed {}", n); }
        }
    }
}
