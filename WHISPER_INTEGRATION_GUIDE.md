# Whisper Speech-to-Text Integration Guide

## Overview

The Audio Content Monitor now has a framework for real-time speech-to-text recognition. This guide explains:

1. **Current Status** - What's implemented now
2. **Integration Options** - How to add actual speech recognition
3. **Setup Instructions** - For each platform

---

## Current Status

✅ **Implemented:**
- Speech recognizer module structure (`speech_recognizer.rs`)
- Audio detection thresholds (energy & peak)
- Bad word detection from recognized text
- Integration points in audio monitor

❌ **Not Yet Connected:**
- Actual speech-to-text API
- Audio encoding to WAV/FLAC format
- API authentication

---

## Why We Need Speech-to-Text?

The current system detects "strong audio" (energy > 0.02, peak > 0.35) but **cannot identify specific words** without converting audio to text first.

**Solution:** Convert audio → text → check for bad words

---

## Integration Option 1: OpenAI Whisper API (RECOMMENDED)

### Pros:
- ✅ Most accurate speech recognition
- ✅ Handles accents and background noise well
- ✅ No complex build dependencies
- ✅ Works globally

### Cons:
- ❌ Requires API key
- ❌ Cloud-based (needs internet)
- ❌ Costs per request ($0.001 per minute)

### Setup Steps:

1. **Get OpenAI API Key:**
   ```
   https://platform.openai.com/api-keys
   ```

2. **Add Dependencies to Cargo.toml:**
   ```toml
   [dependencies]
   reqwest = { version = "0.11", features = ["json", "multipart"] }
   tokio-util = "0.7"
   ```

3. **Update `speech_recognizer.rs`:**

```rust
use reqwest::multipart;
use std::io::Cursor;

pub struct SpeechRecognizer {
    api_key: String,
}

impl SpeechRecognizer {
    pub fn new() -> Self {
        let api_key = std::env::var("OPENAI_API_KEY")
            .expect("OPENAI_API_KEY environment variable not set");
        
        println!("✅ OpenAI Whisper API initialized");
        SpeechRecognizer { api_key }
    }

    pub fn recognize_speech(&self, samples: &[f32]) -> Option<String> {
        // Convert samples to WAV format
        let wav_data = Self::samples_to_wav(samples);
        
        // Send to OpenAI API
        Self::call_whisper_api(&self.api_key, wav_data)
    }

    fn samples_to_wav(samples: &[f32]) -> Vec<u8> {
        // Convert f32 samples to WAV format
        // ... implementation ...
        vec![]
    }

    fn call_whisper_api(api_key: &str, audio_data: Vec<u8>) -> Option<String> {
        // Make POST request to OpenAI API
        // ... implementation ...
        None
    }
}
```

4. **Environment Variable:**
   ```bash
   $env:OPENAI_API_KEY = "sk-..."
   ```

---

## Integration Option 2: Google Cloud Speech-to-Text

### Setup:
1. Create Google Cloud project
2. Enable Speech-to-Text API
3. Create service account credentials
4. Add `google-cloud1` crate

```toml
google-speech1 = "5.0"
google-authz = "0.5"
```

---

## Integration Option 3: Local Whisper (Advanced)

For offline, free speech recognition:

1. Install Whisper binary:
   ```bash
   pip install openai-whisper
   ```

2. Use `whisper-rs` crate (requires C++ build tools):
   ```toml
   whisper-rs = "0.2"
   ```

3. Modify `speech_recognizer.rs` to use `whisper_rs::WhisperContext`

---

## Current Architecture

```
Audio Frames (48kHz, f32)
        ↓
    [Energy/Peak Check]
    (> 0.02 energy, > 0.35 peak)
        ↓
    [Speech Recognizer]
    (TODO: Connect to API/local model)
        ↓
    [Bad Word Detection]
    (Check recognized text)
        ↓
    [Beep Alert]
    (Only if bad words found)
```

---

## Files to Modify

### 1. `src-tauri/Cargo.toml`
Add your chosen API dependencies:
```toml
# For OpenAI Whisper
reqwest = { version = "0.11", features = ["json", "multipart"] }
tokio-util = "0.7"

# OR for Google Cloud
google-speech1 = "5.0"

# OR for Local Whisper
whisper-rs = "0.2"
```

### 2. `src-tauri/src/speech_recognizer.rs`
Implement the `recognize_speech()` method based on your chosen platform.

### 3. Update `process_audio_frame()` in `audio_monitor.rs`
Currently it calls `recognizer.recognize_speech()` which returns `None`.
Once you implement it, it will automatically start working!

---

## Testing the Integration

```rust
#[tokio::test]
async fn test_whisper_detection() {
    let recognizer = SpeechRecognizer::new();
    let detector = BadWordDetector::new();
    
    // Get some audio samples
    let samples = vec![0.1f32; 48000]; // 1 second of audio
    
    // Recognize speech
    if let Some(text) = recognizer.recognize_speech(&samples) {
        println!("Recognized: {}", text);
        
        // Detect bad words
        let bad_words = detector.detect_all_bad_words(&text);
        println!("Bad words: {:?}", bad_words);
    }
}
```

---

## Quick Start for OpenAI (Most Popular)

### 1. Install OpenAI CLI:
```bash
pip install openai
```

### 2. Set API Key:
```powershell
$env:OPENAI_API_KEY = "sk-YOUR-KEY-HERE"
```

### 3. Test Whisper:
```bash
whisper audio.wav --model tiny.en
```

### 4. Use in Rust:
See "Integration Option 1" above.

---

## Performance Considerations

### Current System (Without Speech-to-Text):
- ✅ Very fast (just audio analysis)
- ❌ Can't identify specific words
- ❌ High false positives

### With Speech-to-Text:
- ⚠️ Slower (API calls or local model)
- ✅ Accurate word detection
- ✅ Low false positives
- ℹ️ Typical latency: 1-3 seconds per request

### Optimization Tips:
1. **Batch processing** - Collect 2-3 seconds of audio before sending
2. **Async processing** - Don't block UI while waiting for API
3. **Caching** - Don't re-process the same audio
4. **Local model** - Use offline Whisper for faster response

---

## Recommended Path Forward

1. **Start with OpenAI Whisper API** (easiest, most reliable)
2. **Get 1,000 free minutes** in first 3 months
3. **Test with your YouTube videos**
4. **Monitor costs** ($0.001 per minute = $60 for 1000 hours)
5. **Migrate to local Whisper** if costs are high

---

## Troubleshooting

### "API Key not found"
```bash
# Set environment variable
$env:OPENAI_API_KEY = "sk-..."
echo $env:OPENAI_API_KEY  # Verify
```

### "Audio format not supported"
Ensure you're converting f32 samples to proper WAV format.

### "API returns empty text"
Try sending longer audio clips (at least 1 second).

### "Too slow / high latency"
Use local Whisper model instead of API.

---

## Next Steps

1. Choose integration option (OpenAI recommended)
2. Update Cargo.toml with dependencies
3. Implement `recognize_speech()` in `speech_recognizer.rs`
4. Test with YouTube videos
5. Monitor API costs or local model performance

---

## Resources

- **OpenAI Whisper**: https://platform.openai.com/docs/guides/speech-to-text
- **Google Cloud Speech**: https://cloud.google.com/speech-to-text/docs
- **Whisper GitHub**: https://github.com/openai/whisper
- **Local Whisper**: https://github.com/openai/whisper/blob/main/README.md#installation

---

## Support

For implementation help:
1. Check OpenAI documentation
2. Review Tauri async patterns
3. Test with curl first (before Rust)
4. Use VS Code Copilot for code generation

Happy detecting! 🎵
