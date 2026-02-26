/// Audio Alert Module (Low Latency)
/// Uses a persistent background thread to keep the audio stream open
/// This avoids the 100ms+ delay of opening the output device for every beep.

use rodio::{OutputStream, Sink, Source};
use std::sync::mpsc;
use std::time::Duration;
use std::sync::OnceLock;

static AUDIO_TX: OnceLock<mpsc::Sender<AudioCommand>> = OnceLock::new();

enum AudioCommand {
    PlayBeep(f32, u64),
    PlaySilence(u64),
}

pub struct AudioAlert;

impl AudioAlert {
    /// Initialize the persistent audio thread (Call once at app startup)
    pub fn init() {
        if AUDIO_TX.get().is_some() {
            return;
        }

        let (tx, rx) = mpsc::channel::<AudioCommand>();
        let _ = AUDIO_TX.set(tx);

        std::thread::spawn(move || {
            println!("🔊 [AudioAlert] Starting persistent audio thread...");
            
            // Keep stream and handle alive for the life of the thread
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("❌ [AudioAlert] Failed to open audio stream: {}", e);
                    return;
                }
            };

            let sink = match Sink::try_new(&stream_handle) {
                Ok(s) => {
                    s.set_volume(2.0); // 🔊 MAX VOLUME BOOST
                    s
                },
                Err(e) => {
                    eprintln!("❌ [AudioAlert] Failed to create Sink: {}", e);
                    return;
                }
            };

            while let Ok(cmd) = rx.recv() {
                match cmd {
                    AudioCommand::PlayBeep(freq, dur_ms) => {
                        println!("🔊 [AudioAlert] Internal: Playing {}Hz beep for {}ms", freq, dur_ms);
                        let source = rodio::source::SineWave::new(freq)
                            .take_duration(Duration::from_millis(dur_ms))
                            .amplify(2.0); // 🔊 Push individual beep volume even more
                        
                        sink.append(source);
                    }
                    AudioCommand::PlaySilence(dur_ms) => {
                        let source = rodio::source::Zero::<f32>::new(1, 44100)
                            .take_duration(Duration::from_millis(dur_ms));
                        sink.append(source);
                    }
                }
            }
        });
    }

    /// Internal helper: sends a beep command to the background thread
    fn send_beep(frequency: f32, duration_ms: u64) {
        if let Some(tx) = AUDIO_TX.get() {
            if let Err(e) = tx.send(AudioCommand::PlayBeep(frequency, duration_ms)) {
                eprintln!("❌ [AudioAlert] Failed to send beep command: {}", e);
            }
        } else {
            // Lazy init if not already called (safety fallback)
            Self::init();
            if let Some(tx) = AUDIO_TX.get() {
                let _ = tx.send(AudioCommand::PlayBeep(frequency, duration_ms));
            }
        }
    }

    fn send_silence(duration_ms: u64) {
        if let Some(tx) = AUDIO_TX.get() {
            let _ = tx.send(AudioCommand::PlaySilence(duration_ms));
        }
    }

    pub fn play_warning_beep() -> Result<(), Box<dyn std::error::Error>> {
        // High frequency beep
        Self::send_beep(1000.0, 400); 
        Ok(())
    }

    pub fn play_double_beep() -> Result<(), Box<dyn std::error::Error>> {
        // Sequential pulses with gaps, no spawning needed since Sink queues them
        Self::send_beep(800.0, 200);
        Self::send_silence(100);
        Self::send_beep(1200.0, 250);
        // Action pulses
        Self::send_silence(150);
        Self::send_beep(1500.0, 300);
        Ok(())
    }

    pub fn play_alert_sound() -> Result<(), Box<dyn std::error::Error>> {
        Self::send_beep(800.0, 150);
        Self::send_silence(50);
        Self::send_beep(1100.0, 150);
        Self::send_silence(50);
        Self::send_beep(1400.0, 200);
        Ok(())
    }
}
