use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use windows::Win32::Media::Audio::*;
use windows::Win32::System::Com::*; 
use crate::audio_capture::AudioFrame;

pub fn start_wasapi_loopback(
    tx: mpsc::UnboundedSender<AudioFrame>,
    should_stop: Arc<AtomicBool>,
    is_beeping: Arc<AtomicBool>, // <--- NEW PARAMETER
) -> Result<(), Box<dyn std::error::Error>> {
    const MAX_RETRIES: usize = 5;
    const RETRY_DELAYS_MS: [u64; 5] = [500, 1000, 2000, 5000, 10000];
    
    let mut retry_count = 0;
    
    loop {
        // Pass is_beeping to attempt_capture
        match attempt_capture(&tx, &should_stop, &is_beeping) {
            Ok(_) => {
                println!("🔈 [WASAPI] Capture stopped cleanly");
                return Ok(());
            }
            Err(e) => {
                // ... (Keep existing retry logic exactly as is) ...
                let error_msg = e.to_string();
                if error_msg.contains("0x88890004") {
                    retry_count += 1;
                    if retry_count > MAX_RETRIES {
                        println!("❌ [WASAPI] Max retries exceeded. Giving up.");
                        return Err(e);
                    }
                    let delay = RETRY_DELAYS_MS[retry_count - 1];
                    println!("⚠️ [WASAPI] Device invalidated. Retrying in {}ms...", delay);
                    std::thread::sleep(std::time::Duration::from_millis(delay));
                } else {
                    return Err(e);
                }
            }
        }
        
        if should_stop.load(Ordering::Relaxed) {
            return Ok(());
        }
    }
}

fn attempt_capture(
    tx: &mpsc::UnboundedSender<AudioFrame>,
    should_stop: &Arc<AtomicBool>,
    is_beeping: &Arc<AtomicBool>, // <--- NEW PARAMETER
) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED).ok();

        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            println!("🎧 [WASAPI] Initializing audio device...");

            let enumerator: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let device: IMMDevice = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
            let audio_client: IAudioClient = device.Activate(CLSCTX_ALL, None)?;

            let mix_format_ptr = audio_client.GetMixFormat()?;
            let mix_format = &*mix_format_ptr;
            let sample_rate = mix_format.nSamplesPerSec; // Likely 48000
            let channels = mix_format.nChannels as usize; // Likely 2

            // Initialize in Loopback mode
            audio_client.Initialize(
                AUDCLNT_SHAREMODE_SHARED,
                AUDCLNT_STREAMFLAGS_LOOPBACK, // Loopback capture
                0,
                0,
                mix_format_ptr,
                None,
            )?;

            let capture_client: IAudioCaptureClient = audio_client.GetService()?;
            audio_client.Start()?;

            println!("🎧 [WASAPI] Capture Loop Started ({}Hz, {}ch)", sample_rate, channels);

            while !should_stop.load(Ordering::Relaxed) {
                // 1. Get next packet size
                let packet_length_result = capture_client.GetNextPacketSize();
                if let Err(_) = packet_length_result {
                    return Err("Device invalidated".into());
                }
                let mut packet_length = packet_length_result.unwrap();

                while packet_length > 0 {
                    let mut p_data: *mut u8 = std::ptr::null_mut();
                    let mut num_frames_to_read: u32 = 0;
                    let mut flags: u32 = 0;

                    // 2. Get Buffer
                    capture_client.GetBuffer(&mut p_data, &mut num_frames_to_read, &mut flags, None, None)?;

                    // 3. CRITICAL: Check checks
                    // Check for AUDCLNT_BUFFERFLAGS_SILENT (0x2)
                    // Check if we are currently beeping (Feedback Protection)
                    let is_silent = (flags & 2) != 0;
                    let beeping = is_beeping.load(Ordering::SeqCst); // 🛡️ HIGH PRIORITY SYNC

                    // Debug: Log frame info
                    static FRAME_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
                    let frame_num = FRAME_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if frame_num % 500 == 0 {
                        println!("🎙️ [WASAPI] Frame #{}, frames: {}, flags: {}, silent: {}, beeping: {}", 
                            frame_num, num_frames_to_read, flags, is_silent, beeping);
                    }

                    if !is_silent && !beeping && num_frames_to_read > 0 {
                        // Process valid audio
                        let raw_slice = std::slice::from_raw_parts(p_data as *const f32, num_frames_to_read as usize * channels);
                        
                        // Debug: Check if raw audio has data
                        let max_val = raw_slice.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
                        if frame_num % 500 == 0 {
                            println!("🔊 [WASAPI] Raw audio max amplitude: {:.6}", max_val);
                        }
                        
                        // Optimized processing inline
                        process_and_send(raw_slice, channels, sample_rate, tx);
                    } 
                    
                    // 4. Release Buffer immediately
                    capture_client.ReleaseBuffer(num_frames_to_read)?;
                    
                    packet_length = capture_client.GetNextPacketSize()?;
                }
                
                // Small sleep to prevent CPU spin when idle
                std::thread::sleep(std::time::Duration::from_millis(5));
            }

            audio_client.Stop()?;
            Ok(())
        })();

        CoUninitialize();
        result
    }
}

// Improved Downsampler
fn process_and_send(
    raw_data: &[f32],
    channels: usize,
    input_sample_rate: u32,
    tx: &mpsc::UnboundedSender<AudioFrame>,
) {
    // 1. Downmix to Mono (Average)
    let mono_samples: Vec<f32> = raw_data
        .chunks(channels)
        .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
        .collect();

    // 2. Resample to 16kHz
    // Simple decimation ratio (e.g., 48000 / 16000 = 3.0)
    let ratio = input_sample_rate as f32 / 16_000.0;
    let mut downsampled = Vec::with_capacity((mono_samples.len() as f32 / ratio) as usize + 1);
    
    let mut index = 0.0;
    while (index as usize) < mono_samples.len() {
        downsampled.push(mono_samples[index as usize]);
        index += ratio;
    }

    // 3. Send
    if !downsampled.is_empty() {
        let _ = tx.send(AudioFrame {
            samples: downsampled,         // 16k Mono for Whisper
            _sample_rate: 16_000,
            raw_samples: raw_data.to_vec(), // 48k Stereo for Loopback/Playback
            _raw_sample_rate: input_sample_rate,
        });
    }
}