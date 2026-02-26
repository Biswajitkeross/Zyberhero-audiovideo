//! Vosk Speech Recognizer - Streaming Recognition

use vosk::{Model, Recognizer};
use std::path::Path;

/// Streaming recognizer that processes audio chunks in real-time
pub struct VoskStream {
    recognizer: Recognizer,
}

impl VoskStream {
    pub fn new(model_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        const SAMPLE_RATE: f32 = 16000.0;
        
        // Check model exists
        if !Path::new(model_path).exists() {
            return Err(format!(
                "Vosk model not found at '{}'. Please download from https://alphacephei.com/vosk/models",
                model_path
            ).into());
        }

        println!("🎤 Loading Vosk model from: {}", model_path);
        let model = Model::new(model_path)
            .ok_or("Failed to load Vosk model")?;
        
        // HYBRID APPROACH: Grammar with bad words + common similar-sounding words
        let grammar: &[&str] = &[
            // === BAD WORDS (what we want to detect) ===
            "fuck", "fucking", "fucker", "fucked", "fucks", "motherfucker",
            "shit", "shitting", "shitty", "bullshit",
            "bitch", "bitches", "bitchy", "bitching",
            "ass", "asshole", "dumbass", "badass", "jackass",
            "damn", "dammit", "goddamn", "damned",
            "crap", "crappy", "hell", "bastard", "bastards",
            "dick", "dicks", "pussy", "cunt",
            "idiot", "idiots", "stupid", "dumb",
            
            // === SIMILAR SOUNDING CLEAN WORDS (to reduce false positives) ===
            "frog", "flog", "flood", "fog", "folk", "for", "four", "floor",
            "fork", "fort", "form", "force", "ford", "fore",
            "ship", "shift", "sheet", "shoot", "shot", "shop", "short", "show",
            "shut", "shed", "shell", "shelf", "shield",
            "beach", "bench", "batch", "pitch", "witch", "ditch", "rich", "which",
            "hitch", "switch", "stitch", "twitch",
            "as", "ask", "asked", "class", "glass", "pass", "fast", "last", "past",
            
            // Common filler words for context
            "i", "i'm", "you", "your", "the", "a", "an", "this", "that", "is", "are",
            "my", "me", "we", "on", "in", "to", "of", "and", "or", "but", "so",
            "way", "day", "game", "time", "go", "get", "got", "know", "no", "yes",
            "oh", "what", "how", "why", "when", "where", "who", "all", "just",
            "like", "it", "it's", "be", "been", "being", "have", "has", "had",
            "do", "don't", "does", "did", "will", "would", "could", "should",
            "can", "can't", "not", "with", "from", "at", "by", "about", "up",
            "out", "down", "off", "over", "under", "back", "here", "there",
            "now", "then", "good", "bad", "right", "wrong", "new", "old",
            
            "[unk]"  // Unknown words fallback
        ];
        
        let recognizer = Recognizer::new_with_grammar(&model, SAMPLE_RATE, grammar)
            .or_else(|| {
                println!("⚠️ Grammar mode failed, falling back to full vocabulary");
                Recognizer::new(&model, SAMPLE_RATE)
            })
            .ok_or("Failed to create Vosk recognizer")?;
        
        println!("✅ Vosk recognizer ready (hybrid grammar mode)");
        
        Ok(Self { recognizer })
    }

    /// Feed audio samples and get partial result
    pub fn process_audio(&mut self, samples: &[i16]) -> Option<String> {
        // Feed to recognizer
        self.recognizer.accept_waveform(samples);
        
        // Get partial result (real-time hypothesis)
        let partial_result = self.recognizer.partial_result();
        let partial_text = partial_result.partial;
        
        if !partial_text.is_empty() {
            return Some(partial_text.to_string());
        }
        
        None
    }

    /// Reset the recognizer
    pub fn reset(&mut self) {
        let _ = self.recognizer.final_result();
    }
}
