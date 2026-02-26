//! ZeroMQ Alert Sender for Profanity/NSFW Alerts
//! Uses pure Rust zeromq crate (no native library required)

use serde::Serialize;
use chrono::Local;
use parking_lot::Mutex;
use tokio::runtime::Runtime;
use zeromq::{PubSocket, Socket, SocketSend, ZmqMessage};

#[derive(Serialize, Clone)]
pub struct AudioBlockedAlert {
    pub r#type: String,
    pub timestamp: String,
    #[serde(rename = "badWords")]
    pub bad_words: String,
    pub reason: String,
    pub strike: u32,
    pub max_strikes: u32,
}

#[derive(Serialize, Clone)]
pub struct VideoBlockedAlert {
    pub r#type: String,
    pub timestamp: String,
    #[serde(rename = "className")]
    pub class_name: String,
    pub category: String,
    pub confidence: String,
    pub reason: String,
    pub strike: u32,
    pub max_strikes: u32,
}

struct ZmqInner {
    socket: PubSocket,
    runtime: Runtime,
}

pub struct ZmqAlertSender {
    inner: Mutex<Option<ZmqInner>>,
    endpoint: String,
}

impl ZmqAlertSender {
    pub fn new(endpoint: &str) -> Self {
        println!("[ZMQ] Initializing PUB socket on {}", endpoint);
        
        let inner = match Runtime::new() {
            Ok(runtime) => {
                let mut socket = PubSocket::new();
                match runtime.block_on(socket.bind(endpoint)) {
                    Ok(_endpoint) => {
                        println!("[ZMQ] ✓ PUB socket bound to {}", endpoint);
                        // Give subscribers time to connect
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        Some(ZmqInner { socket, runtime })
                    }
                    Err(e) => {
                        println!("[ZMQ] Bind failed: {} - alerts will be logged only", e);
                        None
                    }
                }
            }
            Err(e) => {
                println!("[ZMQ] Runtime creation failed: {}", e);
                None
            }
        };
        
        Self {
            inner: Mutex::new(inner),
            endpoint: endpoint.to_string(),
        }
    }
    
    fn send_message(&self, topic: &str, json: &str) {
        let mut guard = self.inner.lock();
        if let Some(ref mut zmq_inner) = *guard {
            let message = format!("{} {}", topic, json);
            let msg: ZmqMessage = message.into();
            
            match zmq_inner.runtime.block_on(zmq_inner.socket.send(msg)) {
                Ok(()) => {
                    // Sent successfully
                }
                Err(e) => {
                    println!("[ZMQ] Send failed: {}", e);
                }
            }
        } else {
            // ZMQ not available, message will be logged but not sent
        }
    }

    pub fn send_audio_blocked_alert(&self, bad_word: &str, strike: u32, max_strikes: u32) {
        let alert = AudioBlockedAlert {
            r#type: "AUDIO_BLOCKED".to_string(),
            timestamp: Local::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            bad_words: bad_word.to_string(),
            reason: format!("Profanity detected (Strike {}/{})", strike, max_strikes),
            strike,
            max_strikes,
        };
        if let Ok(json) = serde_json::to_string(&alert) {
            println!("[ZMQ] AUDIO_BLOCKED: '{}' strike {}/{}", bad_word, strike, max_strikes);
            self.send_message("ALERT", &json);
        }
    }

    pub fn send_video_blocked_alert(&self, class_name: &str, category: &str, confidence: f32, strike: u32, max_strikes: u32) {
        let alert = VideoBlockedAlert {
            r#type: "VIDEO_BLOCKED".to_string(),
            timestamp: Local::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
            class_name: class_name.to_string(),
            category: category.to_string(),
            confidence: format!("{:.2}", confidence),
            reason: format!("NSFW content detected (Strike {}/{})", strike, max_strikes),
            strike,
            max_strikes,
        };
        if let Ok(json) = serde_json::to_string(&alert) {
            println!("[ZMQ] VIDEO_BLOCKED: '{}' ({}) strike {}/{}", class_name, category, strike, max_strikes);
            self.send_message("ALERT", &json);
        }
    }
}

// Make it thread-safe
unsafe impl Send for ZmqAlertSender {}
unsafe impl Sync for ZmqAlertSender {}
