pub struct AudioProcessor;

impl AudioProcessor {
    pub fn new() -> Self {
        AudioProcessor
    }

    /// Calculate RMS energy (volume) to detect silence
    /// Returns a value between 0.0 (silence) and ~1.0 (loud)
    pub fn calculate_energy(&self, samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }

        let sum: f32 = samples.iter().map(|s| s * s).sum();
        (sum / samples.len() as f32).sqrt()
    }

    /// Convert stereo samples to mono
    pub fn stereo_to_mono(&self, samples: &[f32], channels: usize) -> Vec<f32> {
        if channels == 1 || samples.is_empty() {
            return samples.to_vec();
        }

        samples
            .chunks(channels)
            .map(|chunk| chunk.iter().sum::<f32>() / channels as f32)
            .collect()
    }

    /// Resample audio from one rate to another using linear interpolation
    pub fn resample(&self, samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
        if from_rate == to_rate {
            return samples.to_vec();
        }

        let ratio = from_rate as f32 / to_rate as f32;
        let new_len = (samples.len() as f32 / ratio).ceil() as usize;
        let mut out = Vec::with_capacity(new_len);
        
        let last_idx = samples.len() - 1;

        for i in 0..new_len {
            let src_idx_f = i as f32 * ratio;
            let idx = src_idx_f as usize;
            
            if idx >= last_idx {
                out.push(samples[last_idx]);
                continue;
            }

            let frac = src_idx_f - idx as f32;
            let sample = samples[idx] * (1.0 - frac) + samples[idx + 1] * frac;
            out.push(sample);
        }
        out
    }
}