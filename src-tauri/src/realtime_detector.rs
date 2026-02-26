use crate::bad_word_detector::BadWordDetector;
use crate::audio_processor::AudioProcessor;
use crate::speech_recognizer::SpeechRecognizer;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Real-time bad word detector with minimal latency
/// Processes audio in 500ms chunks for ~500ms detection latency
#[derive(Clone)]
pub struct RealtimeDetector {
    recognizer: Arc<SpeechRecognizer>,
    detector: Arc<BadWordDetector>,
    processor: Arc<AudioProcessor>,
    buffer: Arc<Mutex<Vec<f32>>>,
}

#[derive(Debug, Clone)]
pub struct DetectionResult {
    pub word: String,
    pub is_bad: bool,
}

impl RealtimeDetector {
    pub fn new() -> Result<Self, String> {
        let recognizer = SpeechRecognizer::new().map_err(|e| e.to_string())?;

        Ok(Self {
            recognizer: Arc::new(recognizer),
            detector: Arc::new(BadWordDetector::new()),
            processor: Arc::new(AudioProcessor::new()),
            buffer: Arc::new(Mutex::new(Vec::new())),
        })
    }

    /// Process audio chunk and detect bad words
    /// Chunk size: 500ms at 16kHz = 8,000 samples
    /// Latency: ~200-500ms for Whisper + 0ms for detection = 200-500ms total
    pub async fn detect_bad_words(&self, chunk: &[f32]) -> Vec<DetectionResult> {
        if chunk.len() < 8_000 {
            return Vec::new(); // Too small, skip
        }

        // Run Whisper inference
        if let Some(transcript) = self.recognizer.recognize_speech_with_timestamps(chunk).await {
            let mut results = Vec::new();

            for word in transcript.words {
                if let Some(bad_word) = self.detector.contains_bad_word(&word.word) {
                    results.push(DetectionResult {
                        word: bad_word.clone(),
                        is_bad: true,
                    });
                    println!("🚨 DETECTED BAD WORD: '{}' at {:.2}s-{:.2}s", 
                             word.word, word.start, word.end);
                }
            }

            results
        } else {
            Vec::new()
        }
    }

    /// Add audio frame to processing buffer
    pub async fn add_frame(&self, samples: &[f32]) {
        let mut buf = self.buffer.lock().await;
        buf.extend_from_slice(samples);
    }

    /// Get and clear buffer when ready to process
    pub async fn get_buffer(&self, min_samples: usize) -> Option<Vec<f32>> {
        let mut buf = self.buffer.lock().await;

        if buf.len() >= min_samples {
            let chunk: Vec<f32> = buf.drain(0..min_samples).collect();
            Some(chunk)
        } else {
            None
        }
    }
}
