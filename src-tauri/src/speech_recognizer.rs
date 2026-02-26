use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};
use std::path::Path;
use std::sync::Arc;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct WhisperWord {
    pub word: String,
    pub start: f32,
    pub end: f32,
}

#[derive(Debug, Clone)]
pub struct WhisperTranscript {
    #[allow(dead_code)]
    pub text: String,
    pub words: Vec<WhisperWord>,
}

pub struct SpeechRecognizer {
    ctx: Arc<WhisperContext>,
}

impl SpeechRecognizer {
    pub fn new() -> Result<Self> {
        let model_path = "resources/ggml-base.en.bin";

        if !Path::new(model_path).exists() {
            return Err(anyhow::anyhow!("Model file not found"));
        }

        let ctx = WhisperContext::new_with_params(
            model_path,
            WhisperContextParameters::default()
        ).map_err(|e| anyhow::anyhow!("Failed to load model: {}", e))?;

        Ok(Self {
            ctx: Arc::new(ctx),
        })
    }

    pub async fn recognize_speech_with_timestamps(&self, samples: &[f32]) -> Option<WhisperTranscript> {
        // 🚀 Reverting to Beam Search (size 2) as requested for better sensitivity
        let mut params = FullParams::new(SamplingStrategy::BeamSearch { 
            beam_size: 2, 
            patience: -1.0 
        });
        
        // � Improved: Using a more natural prompt instead of a keyword list to reduce hallucinations
        // Neutral prompt and stricter thresholds to prevent hallucinations in noise
        params.set_initial_prompt("Wait, I hear music and lyrics.");

        params.set_print_special(false);
        params.set_language(Some("en"));
        params.set_suppress_non_speech_tokens(true);
        
        params.set_no_speech_thold(0.6); // Higher = stricter for noise
        params.set_logprob_thold(-0.5); // Closer to 0.0 = much stricter for low-confidence words like "bitch"
        params.set_entropy_thold(2.4);
        
        params.set_max_tokens(64);
        params.set_n_threads(6); 
        params.set_temperature(0.0);
        
        let mut state = self.ctx.create_state().ok()?;

        if let Err(e) = state.full(params, samples) {
            eprintln!("⚠️ [Whisper] Skipped frame (inference error): {}", e);
            return None;
        }

        let num_segments = state.full_n_segments().ok()?;
        let mut words = Vec::new();
        let mut full_text = String::new();

        for i in 0..num_segments {
            if let Ok(segment_text) = state.full_get_segment_text(i) {
                full_text.push_str(&segment_text);
            }

            let num_tokens = state.full_n_tokens(i).unwrap_or(0);
            for j in 0..num_tokens {
                if let Ok(token_text) = state.full_get_token_text(i, j) {
                    if token_text.starts_with("[_") { continue; }

                    let t0 = state.full_get_token_data(i, j).ok()?.t0;
                    let t1 = state.full_get_token_data(i, j).ok()?.t1;

                    words.push(WhisperWord {
                        word: token_text,
                        start: t0 as f32 / 100.0,
                        end: t1 as f32 / 100.0,
                    });
                }
            }
        }

        if full_text.trim().is_empty() {
            return None;
        }

        Some(WhisperTranscript {
            text: full_text,
            words,
        })
    }
}