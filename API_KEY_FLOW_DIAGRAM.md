# 🔑 API Key Access Flow - Visual Guide

## Simple Version: Where Your API Key Goes

```
STEP 1: You set in PowerShell
┌─────────────────────────────────────┐
│ $env:OPENAI_API_KEY = "sk-proj..." │
└──────────────┬──────────────────────┘
               │ (stored in Windows environment)
               ↓
STEP 2: App starts (npx tauri dev)
┌──────────────────────────────────────────┐
│ speech_recognizer.rs line 24 runs:      │
│ let api_key = std::env::var(...)        │
│                   ↓ READS YOUR KEY      │
└──────────────┬───────────────────────────┘
               │
               ↓
STEP 3: Key is stored
┌──────────────────────────────────────────┐
│ SpeechRecognizer struct:                 │
│ api_key: Option<String> = Some("sk...") │
└──────────────┬───────────────────────────┘
               │
               ↓
STEP 4: Audio detected, key is used
┌──────────────────────────────────────────┐
│ call_whisper_api() function receives:    │
│ - client (HTTP)                          │
│ - api_key (your key!)                    │
│ - samples (audio frames)                 │
└──────────────┬───────────────────────────┘
               │
               ↓
STEP 5: API authentication
┌──────────────────────────────────────────┐
│ .bearer_auth(api_key)                    │
│ Adds: Authorization: Bearer sk-proj...   │
└──────────────┬───────────────────────────┘
               │
               ↓
STEP 6: HTTPS POST to OpenAI
┌──────────────────────────────────────────┐
│ POST https://api.openai.com/.../trans... │
│ Headers: Authorization: Bearer sk-...    │
│ Body: audio.wav file                     │
└──────────────┬───────────────────────────┘
               │
               ↓
STEP 7: OpenAI responds with text
┌──────────────────────────────────────────┐
│ { "text": "fuck this shit" }             │
└──────────────┬───────────────────────────┘
               │
               ↓
STEP 8: Bad word detection
┌──────────────────────────────────────────┐
│ "fuck" and "shit" found!                 │
│ ↓ BEEP! + Log + Counter++                │
└──────────────────────────────────────────┘
```

---

## Code Locations

### **Location 1: Reading the API Key**

**File:** `src-tauri/src/speech_recognizer.rs`
**Lines:** 23-24

```rust
pub fn new() -> Self {
    let api_key = std::env::var("OPENAI_API_KEY").ok();  // ← READS HERE
    let enabled = api_key.is_some();
    
    if enabled {
        println!("✅ OpenAI Whisper API enabled");  // Shows if successful
    }
}
```

**What happens:**
- `std::env::var("OPENAI_API_KEY")` reads the environment variable
- `.ok()` converts Result to Option
- If found: `Some("sk-proj-...")`
- If not found: `None`

---

### **Location 2: Passing the API Key to Function**

**File:** `src-tauri/src/speech_recognizer.rs`
**Lines:** 55-60

```rust
pub fn recognize_speech(&self, samples: &[f32]) -> Option<String> {
    if !self.enabled || samples.is_empty() {
        return None;
    }

    let api_key = self.api_key.as_ref()?.clone();  // ← RETRIEVES FROM STRUCT
    let samples = samples.to_vec();
    let client = Arc::clone(&self.client);

    let rt = tokio::runtime::Runtime::new().ok()?;
    rt.block_on(async {
        Self::call_whisper_api(&client, &api_key, &samples).await
        //                                    ↑ PASSES HERE
    })
}
```

**What happens:**
- Gets the API key from the struct
- Clones it (makes a copy)
- Passes it to the async function

---

### **Location 3: Using API Key for Authentication**

**File:** `src-tauri/src/speech_recognizer.rs`
**Lines:** 62-84

```rust
async fn call_whisper_api(
    client: &Client,
    api_key: &str,  // ← RECEIVES HERE
    samples: &[f32],
) -> Option<String> {
    // Convert audio to WAV
    let wav_data = Self::samples_to_wav(samples)?;

    // Create form
    let form = reqwest::multipart::Form::new()
        .part("file", part)
        .text("model", "whisper-1")
        .text("language", "en");

    // ← API KEY USED HERE FOR AUTHENTICATION
    let response = client
        .post("https://api.openai.com/v1/audio/transcriptions")
        .bearer_auth(api_key)  // ← AUTHENTICATES WITH YOUR KEY
        .multipart(form)
        .send()
        .await
        .ok()?;
    
    // Parse response...
}
```

**What happens:**
- `.bearer_auth(api_key)` adds Authorization header
- Header sent to OpenAI: `Authorization: Bearer sk-proj-...`
- OpenAI validates your key
- If valid: Returns recognized text
- If invalid: Returns error

---

## 🔍 The Complete Journey

```
Your Terminal:
┌────────────────────────────────────────────────────────────┐
│ PS C:\zybertest-desktop>                                   │
│ $env:OPENAI_API_KEY = "sk-proj-abc123def456xyz789..."     │
│                                                             │
│ npx tauri dev                                              │
└────────────────────────────────────────────────────────────┘
                          ↓
Your Computer (Windows Environment):
┌────────────────────────────────────────────────────────────┐
│ Environment Variables:                                     │
│ ├─ OPENAI_API_KEY = "sk-proj-abc123def456xyz789..."      │
│ ├─ PATH = ...                                              │
│ └─ ...                                                      │
└────────────────────────────────────────────────────────────┘
                          ↓
Rust Code (speech_recognizer.rs):
┌────────────────────────────────────────────────────────────┐
│ Line 24: let api_key = std::env::var("OPENAI_API_KEY")   │
│          ├─ Reads from environment                         │
│          ├─ Gets: "sk-proj-abc123def456xyz789..."         │
│          └─ Stores in: api_key: Option<String>            │
└────────────────────────────────────────────────────────────┘
                          ↓
SpeechRecognizer Struct:
┌────────────────────────────────────────────────────────────┐
│ pub struct SpeechRecognizer {                              │
│     enabled: bool,                    ← true              │
│     api_key: Option<String>,          ← "sk-proj-..."    │
│     client: Arc<Client>,              ← HTTP client      │
│     last_recognition_time: ...,       ← timing            │
│     recognition_buffer: ...,          ← buffer            │
│ }                                                          │
└────────────────────────────────────────────────────────────┘
                          ↓
When Audio Detected:
┌────────────────────────────────────────────────────────────┐
│ recognize_speech(&samples)                                 │
│ ├─ Extracts: api_key = "sk-proj-..."                      │
│ └─ Calls: call_whisper_api(..., api_key, ...)             │
└────────────────────────────────────────────────────────────┘
                          ↓
OpenAI API Call:
┌────────────────────────────────────────────────────────────┐
│ POST https://api.openai.com/v1/audio/transcriptions       │
│ Headers:                                                   │
│ ├─ Authorization: Bearer sk-proj-abc123...                │
│ └─ Content-Type: multipart/form-data                      │
│                                                             │
│ Body:                                                      │
│ ├─ file: audio.wav (your audio)                           │
│ ├─ model: whisper-1                                       │
│ └─ language: en                                            │
└────────────────────────────────────────────────────────────┘
                          ↓
OpenAI Response:
┌────────────────────────────────────────────────────────────┐
│ {                                                          │
│   "text": "fuck this shit"                                │
│ }                                                          │
└────────────────────────────────────────────────────────────┘
                          ↓
Bad Word Detection:
┌────────────────────────────────────────────────────────────┐
│ "fuck" → Found in database ✓                             │
│ "shit" → Found in database ✓                             │
│                                                             │
│ 🔊 BEEP!                                                   │
│ 📝 Activity Log: "fuck, shit" detected at 14:32:15        │
│ 🔢 Counter: 1                                              │
└────────────────────────────────────────────────────────────┘
```

---

## ✅ Verification Checklist

When you run `npx tauri dev`, verify these steps:

- [ ] **Step 1:** App starts
- [ ] **Step 2:** Terminal shows:
  ```
  🎤 Initializing AudioMonitor with Whisper speech recognition...
  ```
- [ ] **Step 3:** Terminal shows:
  ```
  ✅ OpenAI Whisper API enabled
  ```
  (If you see this, your API key was successfully read!)
  
- [ ] **Step 4:** Open browser: `http://localhost:5173/`
  
- [ ] **Step 5:** Click "Start Monitoring All Audio"
  
- [ ] **Step 6:** Play YouTube video with profanity
  
- [ ] **Step 7:** Listen for: 🔊 BEEP!
  
- [ ] **Step 8:** Check Activity Log shows detection

If all steps work, your API key is properly integrated! ✅

---

## 🎯 Summary: How Your API Key Works

| Aspect | How It Works |
|--------|-------------|
| **Set** | PowerShell: `$env:OPENAI_API_KEY = "sk-..."` |
| **Read** | Rust: `std::env::var("OPENAI_API_KEY")` at line 24 |
| **Stored** | In `SpeechRecognizer` struct field |
| **Passed** | To `call_whisper_api()` function as parameter |
| **Used** | In `.bearer_auth(api_key)` for HTTPS authentication |
| **Sent** | To OpenAI with: `Authorization: Bearer sk-...` |
| **Validated** | OpenAI checks if key is valid |
| **Response** | If valid: recognized text. If invalid: error |

---

## 🚀 Your Next Step

Just run this and watch the magic happen:

```powershell
npx tauri dev
```

Look for:
```
✅ OpenAI Whisper API enabled
```

When you see that, your API key is working! 🎉

Then open `http://localhost:5173/` and test with YouTube!

---

## 📞 Quick Troubleshooting

| Error | Cause | Fix |
|-------|-------|-----|
| "API Key not set" | Forgot to set env var | `$env:OPENAI_API_KEY = "sk-..."` |
| "Invalid API Key" | Wrong key | Check at https://platform.openai.com/api-keys |
| "Connection timeout" | Network issue | Check internet, try again |
| No beep on YouTube | VB-Cable routing | Check Windows Sound mixer |

---

**Your API key is already being used! No changes needed to the code!** ✅
