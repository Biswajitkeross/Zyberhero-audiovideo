//! TCP Socket Logger
//! Sends log messages to C++ application via TCP Socket (works across network)

use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use chrono::Local;
use serde::Serialize;

const DEFAULT_PORT: u16 = 9999;

#[derive(Serialize, Clone)]
pub struct LogMessage {
    pub timestamp: String,
    pub event_type: String,
    pub message: String,
}

pub struct TcpLogger {
    port: u16,
    clients: Arc<Mutex<Vec<TcpStream>>>,
}

impl TcpLogger {
    pub fn new(port: Option<u16>) -> Self {
        let port = port.unwrap_or(DEFAULT_PORT);
        let clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
        
        let logger = Self {
            port,
            clients: clients.clone(),                                           
        };
        
        // Start TCP server in background thread
        let listener_clients = clients.clone();
        thread::spawn(move || {
            Self::start_server(port, listener_clients);
        });
        
        // Give server time to start
        thread::sleep(std::time::Duration::from_millis(100));
        
        logger
    }
    
    fn start_server(port: u16, clients: Arc<Mutex<Vec<TcpStream>>>) {
        // Bind to all interfaces so remote computers can connect
        let bind_addr = format!("0.0.0.0:{}", port);
        
        match TcpListener::bind(&bind_addr) {
            Ok(listener) => {
                println!("🌐 TCP Server listening on port {}", port);
                println!("📡 C++ app can connect to: <YOUR_IP>:{}", port);
                
                // Print local IP addresses for convenience
                if let Ok(hostname) = hostname::get() {
                    println!("   Hostname: {:?}", hostname);
                }
                
                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => {
                            let peer_addr = stream.peer_addr().ok();
                            println!("✅ Client connected: {:?}", peer_addr);
                            
                            // Clone stream for the clients list
                            if let Ok(stream_clone) = stream.try_clone() {
                                let mut clients_guard = clients.lock().unwrap();
                                clients_guard.push(stream_clone);
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Connection failed: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to start TCP server on port {}: {}", port, e);
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
        
        // Send to all connected TCP clients
        self.send_to_clients(&log_msg);
    }
    
    fn send_to_clients(&self, log_msg: &LogMessage) {
        let mut clients_guard = self.clients.lock().unwrap();
        
        // Remove disconnected clients
        let mut to_remove = Vec::new();
        
        if let Ok(json) = serde_json::to_string(&log_msg) {
            let msg = format!("{}\n", json);
            
            for (i, client) in clients_guard.iter_mut().enumerate() {
                if client.write_all(msg.as_bytes()).is_err() {
                    to_remove.push(i);
                }
            }
        }
        
        // Remove disconnected clients (in reverse to maintain indices)
        for i in to_remove.into_iter().rev() {
            println!("📤 Client disconnected");
            clients_guard.remove(i);
        }
    }
    
    pub fn get_port(&self) -> u16 {
        self.port
    }
    
    pub fn client_count(&self) -> usize {
        self.clients.lock().unwrap().len()
    }
}

impl Drop for TcpLogger {
    fn drop(&mut self) {
        // Send shutdown message to all clients
        let log_msg = LogMessage {
            timestamp: Local::now().format("%Y-%m-%d %H:%M:%S%.3f").to_string(),
            event_type: "SERVICE_STOPPED".to_string(),
            message: "Profanity filter service shutting down".to_string(),
        };
        self.send_to_clients(&log_msg);
    }
}
