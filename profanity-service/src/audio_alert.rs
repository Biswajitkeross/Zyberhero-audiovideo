//! Audio Alert - Plays beep sound on detection

use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::Duration;
use rodio::{OutputStream, Sink, Source};

pub struct AudioAlert {
    sender: Sender<()>,
}

impl AudioAlert {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel::<()>();
        
        // Start persistent audio thread
        thread::spawn(move || {
            let (_stream, handle) = match OutputStream::try_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("❌ Failed to open audio output: {}", e);
                    return;
                }
            };
            
            let sink = match Sink::try_new(&handle) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("❌ Failed to create audio sink: {}", e);
                    return;
                }
            };
            
            println!("🔊 Audio alert thread ready");
            
            loop {
                match receiver.recv() {
                    Ok(()) => {
                        // Generate 800Hz beep for 300ms
                        let source = rodio::source::SineWave::new(800.0)
                            .take_duration(Duration::from_millis(300))
                            .amplify(0.5);
                        sink.append(source);
                        sink.sleep_until_end();
                    }
                    Err(_) => break, // Channel closed
                }
            }
        });
        
        Self { sender }
    }
    
    pub fn beep(&self) {
        let _ = self.sender.send(());
    }
}
