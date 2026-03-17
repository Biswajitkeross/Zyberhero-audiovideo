//! Vosk Speech Recognizer — Bad words + filler grammar.
//!
//! Grammar contains bad words + common filler words + [unk].
//! Filler words give Vosk clean options so it doesn't force normal music into
//! bad words. NO sound-alike words (no fog, frog, ship, beach, etc.).

use vosk::{Model, Recognizer, DecodingState};
use std::path::Path;

/// Streaming recognizer that processes audio chunks in real-time
pub struct VoskStream {
    recognizer: Recognizer,
}

impl VoskStream {
    pub fn new(model_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        const SAMPLE_RATE: f32 = 16000.0;
        
        if !Path::new(model_path).exists() {
            return Err(format!(
                "Vosk model not found at '{}'. Download from https://alphacephei.com/vosk/models",
                model_path
            ).into());
        }

        println!("🎤 Loading Vosk model from: {}", model_path);
        let model = Model::new(model_path)
            .ok_or("Failed to load Vosk model")?;
        
        // Bad words + filler words + [unk].
        // Only removed nigga/nigger — they cause false positives on
        // non-English music (Hindi kids' songs etc.).
        let grammar: &[&str] = &[
            // === BAD WORDS ===
            "fuck", "fucking", "fucker", "fucked",
            "shit",
            "bitch",
            "suck",
            "slut", "asshole",
            "dick",
            // Sound-alike traps (Vosk may hear "fuck" as these)
            "folk", "fog",

            // === FILLER WORDS (~70 — absorb normal music vocals) ===
            "the", "a", "i", "you", "he", "she", "we", "they", "it",
            "me", "my", "your", "his", "her",
            "is", "are", "was", "be", "have", "has", "had",
            "do", "did", "will", "would", "can", "could",
            "not", "no", "yes",
            "and", "or", "but", "so", "if",
            "in", "on", "at", "to", "of", "with", "by", "up", "out",
            "for", "from",
            "this", "that", "what", "how", "all", "just", "like",
            "know", "go", "get", "got", "come", "make", "take",
            "see", "say", "want", "need",
            "love", "baby", "yeah", "oh",
            "here", "there", "now", "time", "day", "way",
            "right", "good", "back", "down",
            "don't", "it's", "i'm",
            "let", "man",

            "[unk]"
        ];
        
        let recognizer = Recognizer::new_with_grammar(&model, SAMPLE_RATE, grammar)
            .or_else(|| {
                println!("⚠️ Grammar mode failed, falling back to full vocabulary");
                Recognizer::new(&model, SAMPLE_RATE)
            })
            .ok_or("Failed to create Vosk recognizer")?;
        
        println!("✅ Vosk recognizer ready ({} grammar words)", grammar.len());
        
        Ok(Self { recognizer })
    }

    /// Feed audio samples and return text from BOTH partial and finalized results.
    /// Partial results give instant detection; finalized results confirm.
    /// Returns (text, is_final).
    pub fn process_audio(&mut self, samples: &[i16]) -> Option<(String, bool)> {
        let state = self.recognizer.accept_waveform(samples);
        
        match state {
            DecodingState::Finalized => {
                let result = self.recognizer.result();
                let text = match &result {
                    vosk::CompleteResult::Single(s) => s.text.to_string(),
                    vosk::CompleteResult::Multiple(m) => {
                        m.alternatives.first()
                            .map(|a| a.text.to_string())
                            .unwrap_or_default()
                    }
                };
                let text = text.trim().to_string();
                if text.is_empty() || text == "[unk]" {
                    return None;
                }
                Some((text, true))
            },
            DecodingState::Running => {
                // Partial result — check for bad words instantly
                let partial = self.recognizer.partial_result();
                let text = partial.partial.trim().to_string();
                if text.is_empty() || text == "[unk]" {
                    return None;
                }
                Some((text, false))
            },
            DecodingState::Failed => None,
        }
    }

    /// Reset the recognizer state
    pub fn reset(&mut self) {
        let _ = self.recognizer.final_result();
    }
}
