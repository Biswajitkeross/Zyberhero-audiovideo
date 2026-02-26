# 🔑 How Your API Key is Accessed - Complete Guide

## ✅ Good News!

**Your API key is AUTOMATICALLY accessed from the environment!** You don't need to do anything special. The code reads it automatically.

---

## 📍 Where Your API Key is Read (Automatic)

### **Main Entry Point: `speech_recognizer.rs` (Line 24)**

When the app starts, this line automatically reads your API key from the environment:

```rust
let api_key = std::env::var("OPENAI_API_KEY").ok();
```

**This is in the `new()` function:**

```rust
pub fn new() -> Self {
    // This line reads YOUR API key from: $env:OPENAI_API_KEY
    let api_key = std::env::var("OPENAI_API_KEY").ok();
    let enabled = api_key.is_some();
    
    if enabled {
        println!("✅ OpenAI Whisper API enabled");  // You'll see this!
    } else {
        println!("⚠️  OPENAI_API_KEY not set. Speech recognition disabled.");
    }

    SpeechRecognizer {
        enabled,
        api_key,  // Your key is stored here
        client: Arc::new(Client::new()),
        last_recognition_time: Arc::new(Mutex::new(0.0)),
        recognition_buffer: Arc::new(Mutex::new(Vec::new())),
    }
}
```

---

## 🔄 How It Flows Through Your Project

```
1. You set in PowerShell:
   $env:OPENAI_API_KEY = "sk-proj-abc123..."
   
2. App starts: npx tauri dev
   
3. Rust code runs: std::env::var("OPENAI_API_KEY")
   ↓ Reads from your environment variable
   
4. API key stored in: SpeechRecognizer struct
   ↓ api_key: Option<String>
   
5. Used in API call: Self::call_whisper_api(&client, &api_key, &samples)
   ↓ Passed to the API function
   
6. API authentication: .bearer_auth(api_key)
   ↓ Sent to OpenAI with HTTPS POST
   
7. OpenAI Whisper API responds with text
   ↓ Success!
```

---

## 📄 File by File: Where API Key is Used

### **File 1: `src-tauri/src/speech_recognizer.rs`**

**Line 24 - Reading the API Key:**
```rust
pub fn new() -> Self {
    let api_key = std::env::var("OPENAI_API_KEY").ok();  // ← Reads here
    let enabled = api_key.is_some();
    // ... rest of initialization
}
```

**Line 64 - Using the API Key in API Call:**
```rust
let api_key = self.api_key.as_ref()?.clone();
let rt = tokio::runtime::Runtime::new().ok()?;
rt.block_on(async {
    Self::call_whisper_api(&client, &api_key, &samples).await
    //                                    ↑ Passed here
})
```

**Line 84 - Bearer Token Authentication:**
```rust
let response = client
    .post("https://api.openai.com/v1/audio/transcriptions")
    .bearer_auth(api_key)  // ← Used here to authenticate
    .multipart(form)
    .send()
    .await
    .ok()?;
```

### **File 2: `src-tauri/src/audio_monitor.rs`**

**Where speech recognizer is initialized:**
```rust
let recognizer = Arc::new(SpeechRecognizer::new());
// ↑ Creates SpeechRecognizer which reads API key automatically
```

**Where it's used:**
```rust
if let Some(recognized_text) = recognizer.recognize_speech(&frame.samples) {
    // ↑ This calls the function that uses your API key
}
```

### **File 3: `src-tauri/src/lib.rs`**

**Initialization:**
```rust
pub struct RecognizerState(Mutex<SpeechRecognizer>);
// ↑ SpeechRecognizer automatically reads API key on creation
```

---

## ✅ How to Verify It's Working

### **Step 1: Start the App**
```powershell
npx tauri dev
```

### **Step 2: Look in Terminal for This Message**
```
✅ OpenAI Whisper API enabled
```

If you see that, your API key was successfully read! ✅

### **Step 3: You'll Also See**
```
🎤 Initializing AudioMonitor with Whisper speech recognition...
```

---

## 🔍 Manual Check (Optional)

If you want to verify your API key is accessible, you can add this temporary code:

**In `speech_recognizer.rs` line 24, add this:**
```rust
let api_key = std::env::var("OPENAI_API_KEY").ok();
if let Some(ref key) = api_key {
    println!("✅ API Key found: {}", &key[..10]); // Show first 10 chars
} else {
    println!("❌ API Key NOT found");
}
```

But **you don't need to do this** - the code already works!

---

## 📊 What Gets Stored Where

| Variable | Location | Contains | Purpose |
|----------|----------|----------|---------|
| `$env:OPENAI_API_KEY` | Windows Environment | `sk-proj-abc123...` | Your API key in PowerShell |
| `api_key` | `SpeechRecognizer::new()` | Option<String> | Read from environment |
| `api_key` (in struct) | `SpeechRecognizer` struct field | Option<String> | Stored for later use |
| `api_key` (in function) | `call_whisper_api()` parameter | &str | Passed to API call |

---

## 🚀 The Beautiful Part

**You don't need to hardcode the API key anywhere!**

The way it's designed:
- ✅ Secure: Key stays in environment, not in code
- ✅ Automatic: Read on startup
- ✅ Simple: One line of code does it all
- ✅ Portable: Works across all projects with that env var set

---

## 🧪 Real Example Flow

### What Happens When You Run: `npx tauri dev`

```
1. App starts
2. speech_recognizer.rs loads
3. Line 24 runs: let api_key = std::env::var("OPENAI_API_KEY").ok();
4. Your API key is read from $env:OPENAI_API_KEY
5. Terminal shows: ✅ OpenAI Whisper API enabled
6. Your key is stored in the SpeechRecognizer struct
7. When audio is detected, your key is used to authenticate with OpenAI
8. OpenAI recognizes the speech
9. Text is returned to your app
10. Bad word detector checks it
11. If bad words found → BEEP! ✅
```

---

## 🎯 Summary

### How Your API Key is Accessed

| Question | Answer |
|----------|--------|
| Where is the API key set? | In PowerShell: `$env:OPENAI_API_KEY` |
| Where is it read in code? | `src-tauri/src/speech_recognizer.rs` line 24 |
| How is it read? | `std::env::var("OPENAI_API_KEY")` |
| How is it used? | Passed to OpenAI Whisper API for authentication |
| Do you need to hardcode it? | NO! Environment variable is enough |
| Is it secure? | YES! Key never appears in source code |
| Will it work in production? | YES! Just set the env var in production server |

---

## ✨ You're All Set!

Your API key is **already being accessed automatically** when the app starts.

### Just Start the App:
```powershell
npx tauri dev
```

### Look for:
```
✅ OpenAI Whisper API enabled
```

### That's it! The rest is automatic! 🎉

---

## 🔒 Security Best Practices (You're Already Doing It!)

✅ **Correct:** Using environment variable (what you're doing)
```powershell
$env:OPENAI_API_KEY = "sk-..."  # Secure
```

❌ **Never do this:** Hardcoding in code
```rust
let api_key = "sk-...";  // INSECURE! Don't do this
```

You're doing it right! 💪

---

## 📞 If Something Goes Wrong

### "API Key not set" message
- You forgot to run: `$env:OPENAI_API_KEY = "sk-..."`
- Run it now and restart the app

### "Invalid API Key" message
- Check your key at: https://platform.openai.com/api-keys
- Make sure you copied the full key (should start with `sk-`)
- Make sure there are no extra spaces

### "Connection timeout"
- Check internet connection
- Try again (API might be slow)
- Check OpenAI status: https://status.openai.com/

---

## 🎬 Next Step

Just run:
```powershell
npx tauri dev
```

The code will automatically:
1. ✅ Read your API key from environment
2. ✅ Initialize OpenAI Whisper integration
3. ✅ Start monitoring audio
4. ✅ Show: `✅ OpenAI Whisper API enabled`

**You're done with setup!** Everything works automatically! 🚀
