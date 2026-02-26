//! Screen Capture Module using scrap

use scrap::{Capturer, Display};
use std::thread;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::io::ErrorKind;

pub struct CaptureConfig {
    pub fps: u32,
    pub scale: f32,
}

pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub struct ScreenCapture {
    running: Arc<AtomicBool>,
    _handle: thread::JoinHandle<()>,
}

impl ScreenCapture {
    pub fn start<F>(config: CaptureConfig, mut callback: F) -> Result<Self, String>
    where
        F: FnMut(CapturedFrame) + Send + 'static,
    {
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();
        
        let frame_interval = Duration::from_millis(1000 / config.fps as u64);
        let scale = config.scale;

        // Create display and capturer inside the thread since they're not Send
        let handle = thread::spawn(move || {
            let display = match Display::primary() {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Display error: {}", e);
                    return;
                }
            };
            
            let width = display.width();
            let height = display.height();
            
            let mut capturer = match Capturer::new(display) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to create capturer: {}", e);
                    return;
                }
            };

            while running_clone.load(Ordering::SeqCst) {
                let start = Instant::now();

                match capturer.frame() {
                    Ok(frame) => {
                        // Scale down if needed
                        let (out_w, out_h, data) = if scale < 1.0 {
                            scale_frame(&frame, width, height, scale)
                        } else {
                            (width as u32, height as u32, frame.to_vec())
                        };

                        callback(CapturedFrame {
                            width: out_w,
                            height: out_h,
                            data,
                        });
                    }
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        // Screen not ready, wait a bit
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(e) => {
                        eprintln!("Capture error: {}", e);
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                }

                let elapsed = start.elapsed();
                if elapsed < frame_interval {
                    thread::sleep(frame_interval - elapsed);
                }
            }
        });

        Ok(Self {
            running,
            _handle: handle,
        })
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl Drop for ScreenCapture {
    fn drop(&mut self) {
        self.stop();
    }
}

fn scale_frame(frame: &[u8], width: usize, height: usize, scale: f32) -> (u32, u32, Vec<u8>) {
    let new_w = (width as f32 * scale) as usize;
    let new_h = (height as f32 * scale) as usize;
    let mut scaled = vec![0u8; new_w * new_h * 4];

    let x_ratio = width as f32 / new_w as f32;
    let y_ratio = height as f32 / new_h as f32;

    for y in 0..new_h {
        for x in 0..new_w {
            let src_x = (x as f32 * x_ratio) as usize;
            let src_y = (y as f32 * y_ratio) as usize;
            let src_idx = (src_y * width + src_x) * 4;
            let dst_idx = (y * new_w + x) * 4;

            if src_idx + 3 < frame.len() && dst_idx + 3 < scaled.len() {
                scaled[dst_idx..dst_idx + 4].copy_from_slice(&frame[src_idx..src_idx + 4]);
            }
        }
    }

    (new_w as u32, new_h as u32, scaled)
}
