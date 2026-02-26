use std::sync::{Mutex, Condvar};
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};

pub struct DelayBuffer {
    buffer: Mutex<Vec<f32>>,
    cvar: Condvar,
    capacity: usize,
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
    is_buffering: AtomicBool,
    target_latency_samples: usize,
}

impl DelayBuffer {
    pub fn new(sample_rate: u32, latency_ms: u32) -> Self {
        let latency_samples = (sample_rate as f32 * (latency_ms as f32 / 1000.0)) as usize;
        // Capacity MUST be > 2x Latency.
        // Latency 12s -> Capacity 30s.
        let capacity = sample_rate as usize * 30; 

        Self {
            buffer: Mutex::new(vec![0.0; capacity]),
            cvar: Condvar::new(),
            capacity,
            // Lock the recorder exactly 'latency' samples ahead of the player
            write_pos: AtomicUsize::new(latency_samples), 
            read_pos: AtomicUsize::new(0),
            is_buffering: AtomicBool::new(false),
            target_latency_samples: latency_samples,
        }
    }

    pub fn push(&self, samples: &[f32]) {
        let mut buf = self.buffer.lock().unwrap();
        let mut w = self.write_pos.load(Ordering::SeqCst);
        let r = self.read_pos.load(Ordering::SeqCst);

        for &sample in samples {
            buf[w] = sample;
            let next_w = (w + 1) % self.capacity;
            
            // IF OVERLAP DETECTED: Pause the recorder until the player moves
            if next_w == r { break; }
            w = next_w;
        }
        self.write_pos.store(w, Ordering::SeqCst);
        self.cvar.notify_all(); // Wake up the Render thread
    }

    pub fn get_write_pos(&self) -> usize {
        self.write_pos.load(Ordering::SeqCst)
    }

    pub fn get_capacity(&self) -> usize {
        self.capacity
    }

    pub fn get_read_pos(&self) -> usize {
        self.read_pos.load(Ordering::SeqCst)
    }

    pub fn get_distance(&self) -> usize {
        self.available_to_read()
    }

    pub fn pop(&self, count: usize) -> Vec<f32> {
        let mut buf = self.buffer.lock().unwrap();
        
        let available = self.available_to_read_internal(&buf);
        let was_buffering = self.is_buffering.load(Ordering::SeqCst);
        
        // DYNAMIC THRESHOLD:
        // We want to maintain at least (Target - 0.5s) of audio.
        // If we drop below this, we force a refill to avoid missing deadlines.
        // Since Target is ~4.0s, we use 1/8th as the 0.5s margin approx.
        let margin = self.target_latency_samples / 8; 
        let underrun_threshold = self.target_latency_samples.saturating_sub(margin);

        // HYSTERESIS LOGIC
        if was_buffering {
            // If buffering, waiting for FULL latency target (3s)
            if available >= self.target_latency_samples {
                println!("✅ Buffer refilled ({} samples). Resuming playback.", available);
                self.is_buffering.store(false, Ordering::SeqCst);
            } else {
                 // Still buffering: output silence
                 return vec![0.0; count];
            }
        } else {
            // If playing, check for underrun
            if available < underrun_threshold {
                println!("⚠️ Buffer Underrun ({} < {}). Entering BUFFERING mode.", available, underrun_threshold);
                self.is_buffering.store(true, Ordering::SeqCst);
                return vec![0.0; count];
            }
        }

        // --- NORMAL PLAYBACK ---

        // Wait if we temporarily ran dry (rare due to hysteresis, but possible)
        let mut attempts = 0;
        while self.available_to_read_internal(&buf) < count {
            buf = self.cvar.wait(buf).unwrap();
            attempts += 1;
            if attempts > 5 { break; } 
        }

        let mut out = Vec::with_capacity(count);
        let mut r = self.read_pos.load(Ordering::SeqCst);

        for _ in 0..count {
            out.push(buf[r]);
            // CRITICAL: Clear memory after playing. 
            // This prevents old audio from repeating if the buffer wraps.
            buf[r] = 0.0; 
            r = (r + 1) % self.capacity;
        }
        
        self.read_pos.store(r, Ordering::SeqCst);
        out
    }

    fn available_to_read_internal(&self, _buf: &Vec<f32>) -> usize {
        let w = self.write_pos.load(Ordering::SeqCst);
        let r = self.read_pos.load(Ordering::SeqCst);
        if w >= r { w - r } else { self.capacity - (r - w) }
    }

    fn available_to_read(&self) -> usize {
        let w = self.write_pos.load(Ordering::SeqCst);
        let r = self.read_pos.load(Ordering::SeqCst);
        if w >= r { w - r } else { self.capacity - (r - w) }
    }

    pub fn duck_and_beep_at_pos(&self, start_index: usize, duration: usize) {
        let mut buf = self.buffer.lock().unwrap();
        
        for i in 0..duration {
            let idx = (start_index + i) % self.capacity;
            
            // DUCKING: Mute original volume completely (0.0)
            let original = 0.0;
            
            // BEEP: Generate 1kHz sine wave slightly louder
            let beep = (2.0 * std::f32::consts::PI * 1000.0 * (i as f32 / 48000.0)).sin() * 0.8; 
            
            // MIX: Only beep, no original audio
            buf[idx] = original + beep;
        }
    }
}