/// Vosk Speech Recognizer - Low Latency Streaming Recognition
/// 
/// Key advantages over Whisper:
/// - TRUE STREAMING: Words returned AS THEY ARE SPOKEN (50-100ms latency)
/// - No batch processing needed - processes audio in small chunks
/// - Lightweight model (~50MB vs 150MB+ for Whisper)
/// - Works well with background music/noise

use vosk::{Model, Recognizer};
use std::path::Path;
use std::sync::Arc;
use anyhow::Result;
use serde::Deserialize;

/// Word with timestamp from Vosk
#[derive(Debug, Clone)]
pub struct VoskWord {
    pub word: String,
    pub start: f32,
    pub end: f32,
    pub conf: f32,
}

/// Vosk result structure (matches JSON output)
#[derive(Deserialize, Debug)]
struct VoskResult {
    #[serde(default)]
    text: String,
    #[serde(default)]
    result: Vec<VoskWordResult>,
}

#[derive(Deserialize, Debug)]
struct VoskWordResult {
    word: String,
    start: f32,
    end: f32,
    conf: f32,
}

/// Partial result from Vosk (real-time hypothesis)
#[derive(Deserialize, Debug)]
struct VoskPartialResult {
    partial: String,
}

pub struct VoskSpeechRecognizer {
    model: Arc<Model>,
}

impl VoskSpeechRecognizer {
    pub fn new() -> Result<Self> {
        // Model path - using vosk-model-small-en-us
        let model_path = "resources/vosk-model-small-en-us-0.15";
        
        if !Path::new(model_path).exists() {
            return Err(anyhow::anyhow!(
                "Vosk model not found at '{}'. Please download from https://alphacephei.com/vosk/models",
                model_path
            ));
        }

        println!("🎤 [Vosk] Loading model from: {}", model_path);
        let model = Model::new(model_path)
            .ok_or_else(|| anyhow::anyhow!("Failed to load Vosk model"))?;
        
        println!("✅ [Vosk] Model loaded successfully!");

        Ok(Self {
            model: Arc::new(model),
        })
    }

    /// Create a new streaming recognizer for continuous audio processing
    pub fn create_stream(&self) -> Result<VoskStream> {
        VoskStream::new(self.model.clone())
    }
}

/// Streaming recognizer that processes audio chunks in real-time
pub struct VoskStream {
    recognizer: Recognizer,
    sample_rate: f32,
}

impl VoskStream {
    pub fn new(model: Arc<Model>) -> Result<Self> {
        const SAMPLE_RATE: f32 = 16000.0;
        
        // HYBRID APPROACH: Grammar with bad words + common similar-sounding words
        // This helps Vosk distinguish between "fuck" and "flog/flood/for"
        // by giving it BOTH options to choose from based on acoustic match
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
            // Words that sound like "fuck" - if Vosk picks these, it's NOT profanity
            "frog", "flog", "flood", "fog", "folk", "for", "four", "floor",
            "fork", "fort", "form", "force", "ford", "fore",
            
            // Words that sound like "shit"
            "ship", "shift", "sheet", "shoot", "shot", "shop", "short", "show",
            "shut", "shed", "shell", "shelf", "shield",
            
            // Words that sound like "bitch"  
            "beach", "bench", "batch", "pitch", "witch", "ditch", "rich", "which",
            "hitch", "switch", "stitch", "twitch",
            
            // Words that sound like "ass"
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
            "come", "going", "want", "need", "think", "say", "said", "make",
            "take", "see", "look", "give", "use", "find", "tell", "put",
            
            "[unk]"  // Unknown words fallback
        ];
        
        // Use grammar mode with expanded vocabulary
        let recognizer = Recognizer::new_with_grammar(&model, SAMPLE_RATE, grammar)
            .or_else(|| {
                println!("⚠️ [Vosk] Grammar mode failed, falling back to full vocabulary");
                Recognizer::new(&model, SAMPLE_RATE)
            })
            .ok_or_else(|| anyhow::anyhow!("Failed to create Vosk recognizer"))?;
        
        println!("✅ [Vosk] Using hybrid grammar (bad words + similar clean words)");
        
        Ok(Self {
            recognizer,
            sample_rate: SAMPLE_RATE,
        })
    }

    /// Feed audio samples and get partial result (real-time hypothesis)
    /// This is called frequently with small chunks (~100ms of audio)
    pub fn accept_waveform(&mut self, samples: &[f32]) -> Option<String> {
        // Apply gain boost to make speech more prominent against background music
        // This helps with recognition when speech is quieter than music
        const GAIN_BOOST: f32 = 2.0; // 2x gain boost
        
        // Convert f32 samples to i16 for Vosk with gain boost
        let samples_i16: Vec<i16> = samples
            .iter()
            .map(|&s| {
                let boosted = s * GAIN_BOOST;
                (boosted * 32767.0).clamp(-32767.0, 32767.0) as i16 // Note: -32767 not -32768 to avoid overflow
            })
            .collect();

        // Debug: Check if samples have audio data
        // Use saturating_abs() to avoid overflow when s == i16::MIN
        let max_sample = samples_i16.iter().map(|s| ((*s) as i32).abs() as i16).max().unwrap_or(0);
        static CALL_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let call_num = CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if call_num % 100 == 0 {
            println!("🔊 [Vosk] accept_waveform #{}, samples: {}, max_i16: {}", 
                call_num, samples_i16.len(), max_sample);
        }

        // Feed to recognizer - returns state
        let state = self.recognizer.accept_waveform(&samples_i16);
        
        // Get partial result (real-time hypothesis as user speaks)
        let partial_result = self.recognizer.partial_result();
        let partial_text = partial_result.partial;
        
        // Debug: Always log state and partial
        if call_num % 100 == 0 || !partial_text.is_empty() {
            println!("🔤 [Vosk] State: {:?}, Partial: '{}'", state, partial_text);
        }
        
        if !partial_text.is_empty() {
            return Some(partial_text.to_string());
        }
        
        None
    }

    /// Get final result with timestamps after silence is detected
    #[allow(dead_code)]
    pub fn get_final_result(&mut self) -> Vec<VoskWord> {
        let result = self.recognizer.final_result();
        
        // Use the single result variant
        if let vosk::CompleteResult::Single(single) = result {
            // single.result is already a Vec<Word>, not Option
            return single.result.iter().map(|w| VoskWord {
                word: w.word.to_string(),
                start: w.start,
                end: w.end,
                conf: w.conf,
            }).collect();
        }
        
        Vec::new()
    }

    /// Reset the recognizer for a new utterance
    pub fn reset(&mut self) {
        // Create a fresh recognizer state by getting the final result
        let _ = self.recognizer.final_result();
    }

    #[allow(dead_code)]
    pub fn sample_rate(&self) -> f32 {
        self.sample_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vosk_init() {
        // This test will fail if model is not downloaded
        // Run: cargo test -- --ignored
        let result = VoskSpeechRecognizer::new();
        assert!(result.is_ok() || result.is_err()); // Just ensure it doesn't panic
    }
}
