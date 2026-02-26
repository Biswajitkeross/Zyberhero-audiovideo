use tokio::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

pub type AudioBuffer = Vec<f32>;

#[derive(Clone, Debug)]
pub struct AudioFrame {
    pub samples: AudioBuffer,
    pub _sample_rate: u32,
    pub raw_samples: AudioBuffer,
    pub _raw_sample_rate: u32,
}

pub fn start_audio_capture(
    tx: mpsc::UnboundedSender<AudioFrame>,
    should_stop: Arc<AtomicBool>,
    is_beeping: Arc<AtomicBool>, // <--- Add this
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔌 [AudioCapture] Starting native WASAPI Loopback...");
    crate::wasapi_capture::start_wasapi_loopback(tx, should_stop, is_beeping)
}