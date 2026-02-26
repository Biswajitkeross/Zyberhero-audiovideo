//! Windows Named Pipe Logger
//! Sends log messages to C++ application via Named Pipe

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use chrono::Local;
use serde::Serialize;

#[derive(Serialize)]
pub struct LogMessage {
    pub timestamp: String,
    pub event_type: String,
    pub message: String,
}

pub struct PipeLogger {
    pipe_name: String,
    pipe: Mutex<Option<std::fs::File>>,
}

impl PipeLogger {
    pub fn new(pipe_name: &str) -> Self {
        let logger = Self {
            pipe_name: pipe_name.to_string(),
            pipe: Mutex::new(None),
        };
        
        // Try to connect to pipe (C++ app may not be running yet)
        logger.try_connect();
        
        logger
    }
    
    fn try_connect(&self) -> bool {
        let mut pipe_guard = self.pipe.lock().unwrap();
        
        if pipe_guard.is_some() {
            return true;
        }
        
        // Try to open the named pipe
        match OpenOptions::new()
            .write(true)
            .open(&self.pipe_name)
        {
            Ok(file) => {
                *pipe_guard = Some(file);
                println!("📡 Connected to Named Pipe: {}", self.pipe_name);
                true
            }
            Err(_) => {
                // Pipe not available - C++ app may not be running
                // This is OK - we'll just log to console
                false
            }
        }
    }
    
    pub fn log(&self, event_type: &str, message: &str) {
        let log_msg = LogMessage {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            event_type: event_type.to_string(),
            message: message.to_string(),
        };
        
        // Always print to console
        println!("[{}] {}: {}", log_msg.timestamp, event_type, message);
        
        // Try to send to pipe
        if self.try_connect() {
            let mut pipe_guard = self.pipe.lock().unwrap();
            if let Some(ref mut pipe) = *pipe_guard {
                // Send JSON message followed by newline
                if let Ok(json) = serde_json::to_string(&log_msg) {
                    let msg = format!("{}\n", json);
                    if pipe.write_all(msg.as_bytes()).is_err() {
                        // Pipe disconnected - clear it so we try reconnecting
                        *pipe_guard = None;
                    }
                }
            }
        }
    }
}

impl Drop for PipeLogger {
    fn drop(&mut self) {
        self.log("DISCONNECTED", "Pipe logger shutting down");
    }
}
