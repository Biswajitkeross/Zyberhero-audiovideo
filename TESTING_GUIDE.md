# 🧪 Complete Testing Guide - YouTube/Media Player Bad Word Detection

## ✅ Prerequisites (Already Done)

You have everything ready:
- ✅ VB-Cable installed and working
- ✅ VoiceMeeter installed and working
- ✅ OpenAI API key set in PowerShell
- ✅ All code compiled
- ✅ App ready to run

---

## 🚀 Step-by-Step Testing Process

### **STEP 1: Set API Key (If Not Already Set)**

**In PowerShell:**
```powershell
$env:OPENAI_API_KEY = "sk-your-actual-key-here"
```

Verify it's set:
```powershell
echo $env:OPENAI_API_KEY
```

You should see: `sk-...` (your key)

---

### **STEP 2: Start the Application**

**In PowerShell, run:**
```powershell
cd "C:\Users\USR-LPTP-81\Desktop\zybertest-desktop"
npx tauri dev
```

**Wait for these messages in terminal:**
```
VITE v7.3.1 ready in ... ms
...
✅ OpenAI Whisper API enabled
🎤 Initializing AudioMonitor with Whisper speech recognition...
```

✅ **If you see "✅ OpenAI Whisper API enabled", you're good to go!**

---

### **STEP 3: Open the App in Browser**

**URL:** `http://localhost:5173/`

**You should see:**
- Dashboard with status display
- "Start Monitoring All Audio" button
- Manual Detection section
- Activity Log section
- Detection Counter

---

### **STEP 4: Verify VB-Cable Routing in App**

Look at the app dashboard:
1. Check the audio device selector
2. Make sure "CABLE Output" is selected
3. If not, select it manually

**This ensures audio from your media is captured!**

---

### **STEP 5: Start Monitoring**

**In the app:**
1. Click: **"Start Monitoring All Audio"** button
2. Status should change to: **"MONITORING ACTIVE"** ✅
3. Check the terminal - you should see:
   ```
   🎤 Audio monitoring started
   ```

---

### **STEP 6: Test with YouTube (Method 1 - EASIEST)**

#### **A. Open YouTube**
1. In a NEW browser tab, go to: `https://www.youtube.com`
2. Search for a song with explicit/profanity content
3. **Good test songs:**
   - "Fuck This Shit" - explicit songs
   - Any rap song with profanity
   - Censored versions won't work (no audio profanity to detect)

#### **B. Play the Video**
1. Click play on the YouTube video
2. Let it play with volume at reasonable level
3. **DON'T play through speakers directly!**
4. Make sure **VoiceMeeter is routing YouTube audio to VB-Cable**

#### **C. Expected Result**
When a bad word is spoken:
```
Terminal Output:
📝 Whisper: fuck this shit
🚨 BAD WORDS DETECTED: fuck, shit (Count: 1)
BEEP! 🔊 (double beep sound)

App Dashboard:
✅ Activity Log updated with detection
✅ Counter incremented
✅ Status shows detection
```

---

### **STEP 7: Test with Media Player (Method 2)**

#### **A. Open Media Player**
Use any media player:
- Windows Media Player
- VLC Media Player
- Spotify (if it has explicit tracks)
- Any other audio application

#### **B. Play Audio with Profanity**
1. Play a song or audio clip with bad words
2. Ensure **VoiceMeeter is routing its audio to VB-Cable**
3. Volume should be normal (not muted)

#### **C. Expected Result**
Same as YouTube - beep alert + app update

---

### **STEP 8: Verify VoiceMeeter Routing**

**If no beep happens, check your routing:**

1. **Open VoiceMeeter**
2. Look for audio levels moving when YouTube/media plays
3. Make sure:
   - Input from media app is coming in
   - Virtual Input (A1) is receiving signal
   - Output is routing to VB-Cable

4. **In Windows Sound Settings:**
   - Right-click Volume icon
   - Select "Volume mixer"
   - For YouTube/media player:
     - Input device: "VoiceMeeter Input"
     - Output device: "CABLE Input (VB-Audio Virtual Cable)"

---

### **STEP 9: Test with Manual Detection**

**If YouTube/media test doesn't work yet, test manually:**

1. In app, go to: **"Manual Detection"** section
2. Type: `fuck this shit` (or any bad word)
3. Click: **"Test Detection"** button

**Expected Output:**
```
BAD WORDS DETECTED: fuck, shit
```

✅ If this works, your app is fine. Problem is with audio routing.

---

### **STEP 10: Troubleshoot If No Beep**

| Problem | Solution |
|---------|----------|
| **No beep at all** | Check Windows volume - not muted? |
| **No audio in app** | Verify CABLE Output selected in app |
| **No terminal messages** | Check if app is "MONITORING ACTIVE" |
| **Manual test works but YouTube doesn't** | VoiceMeeter routing issue - check Volume Mixer |
| **Beep but no text recognition** | API key issue or network problem |

---

## 🎯 Complete Testing Checklist

### **Setup Phase**
- [ ] API key set in PowerShell: `$env:OPENAI_API_KEY = "sk-..."`
- [ ] VB-Cable installed and visible in sound devices
- [ ] VoiceMeeter running and showing audio levels
- [ ] App code compiled (0 errors)

### **Launch Phase**
- [ ] Terminal shows: `✅ OpenAI Whisper API enabled`
- [ ] App opens at http://localhost:5173/
- [ ] Dashboard visible with all components
- [ ] CABLE Output device selected in app

### **Monitoring Phase**
- [ ] Click "Start Monitoring All Audio" button
- [ ] Status changes to "MONITORING ACTIVE"
- [ ] Terminal shows monitoring started

### **Testing Phase**
- [ ] Manual text test works (type bad word, click test)
- [ ] Open YouTube in new tab
- [ ] Play song with profanity
- [ ] Terminal shows: `📝 Whisper: [recognized text]`
- [ ] Terminal shows: `🚨 BAD WORDS DETECTED: [words]`
- [ ] App beeps (double beep sound) 🔊
- [ ] Activity log updates
- [ ] Detection counter increments

### **Verification Phase**
- [ ] Stop monitoring - status shows "STOPPED"
- [ ] Activity log has at least 1 entry
- [ ] Counter shows total detections
- [ ] Each detection has timestamp

---

## 📝 Expected Terminal Output

### **When App Starts:**
```
✅ OpenAI Whisper API enabled
🎤 Initializing AudioMonitor with Whisper speech recognition...
```

### **When YouTube Plays:**
```
🎵 Strong audio detected (energy: 0.0234, peak: 0.67)
```

### **When Bad Word Found:**
```
📝 Whisper: fuck this shit
🚨 BAD WORDS DETECTED: fuck, shit (Count: 1)
BEEP SOUND PLAYS ✓
```

### **App Dashboard Updates:**
```
Activity Log:
  [15:30:45] Detected: fuck, shit
  
Counter: 1
Status: MONITORING ACTIVE
```

---

## 🎵 Test Videos to Use

### **YouTube Search Terms:**
- "Explicit rap songs" - Most have profanity
- "Comedy sketches with bad words"
- "Movie clips with profanity"
- "Music with censored bad words won't work!" ❌

### **Important Notes:**
- ✅ **Censored songs WON'T work** (audio is removed, nothing to detect)
- ✅ **Explicit/Uncensored songs WILL work** (audio contains bad words)
- ✅ **Background music without speech won't trigger** (Whisper needs speech)
- ✅ **Quiet audio won't trigger** (detection threshold: energy > 0.02, peak > 0.35)

---

## 🔧 Configuration During Testing

### **If Detection Too Sensitive (Too Many False Positives)**
Edit: `src-tauri/src/audio_monitor.rs` Line 160:
```rust
// Increase these values (make it less sensitive)
if energy > 0.03 && max_sample > 0.40 {  // Was: 0.02 and 0.35
```

Then rebuild:
```powershell
cd src-tauri
cargo build
cd ..
npx tauri dev
```

### **If Detection Too Weak (Misses Bad Words)**
Edit: `src-tauri/src/audio_monitor.rs` Line 160:
```rust
// Decrease these values (make it more sensitive)
if energy > 0.01 && max_sample > 0.30 {  // Was: 0.02 and 0.35
```

Then rebuild.

### **Add More Bad Words**
Edit: `src-tauri/src/bad_word_detector.rs`:
```rust
pub fn get_all_words(&self) -> Vec<String> {
    vec![
        "fuck", "shit", "damn", "hell", "bitch",
        "newbadword",  // ← Add here
        "anotherbadword",
    ]
}
```

Then rebuild.

---

## 📊 Real-World Example

### **Complete Testing Session:**

**Terminal:**
```
PS> $env:OPENAI_API_KEY = "sk-proj-abc123..."
PS> npx tauri dev

✅ OpenAI Whisper API enabled
🎤 Initializing AudioMonitor...
```

**Browser:**
- Open http://localhost:5173/
- Click "Start Monitoring All Audio"
- Status: "MONITORING ACTIVE"

**YouTube (New Tab):**
- Search "explicit rap"
- Play song
- When profanity starts:

**Terminal Shows:**
```
🎵 Strong audio detected (energy: 0.0456, peak: 0.85)
📝 Whisper: yo fuck this shit up
🚨 BAD WORDS DETECTED: fuck, shit (Count: 1)
```

**App Shows:**
```
BEEP! 🔊 (double beep)
Activity Log:
  [15:32:18] Detected: fuck, shit

Counter: 1
```

**Browser (Next Bad Word):**
```
🎵 Strong audio detected
📝 Whisper: hell yeah
🚨 BAD WORDS DETECTED: hell (Count: 2)
BEEP! 🔊

Activity Log:
  [15:32:25] Detected: hell
  [15:32:18] Detected: fuck, shit

Counter: 2
```

---

## ✅ Success Criteria

Your system is working perfectly when:

1. ✅ Manual text detection works
2. ✅ YouTube plays without app crash
3. ✅ When bad word spoken, app beeps
4. ✅ Terminal shows recognized text
5. ✅ Activity log updates
6. ✅ Counter increments
7. ✅ Multiple detections work
8. ✅ No false positives on clean audio

---

## 🚀 Quick Start (Summary)

```powershell
# 1. Set API key
$env:OPENAI_API_KEY = "sk-your-key"

# 2. Start app
npx tauri dev

# 3. Open browser
# http://localhost:5173/

# 4. Click "Start Monitoring All Audio"

# 5. Play YouTube video with bad words

# 6. App should beep! ✅
```

---

## 📞 If Something Goes Wrong

| Issue | Fix |
|-------|-----|
| **No beep sound** | Check Windows volume, system sounds enabled |
| **No terminal messages** | Check if "Start Monitoring" was clicked |
| **Manual test works but YouTube doesn't** | VoiceMeeter routing issue |
| **App crashes** | Check terminal for error messages |
| **API key error** | Verify key is correct at https://platform.openai.com/api-keys |
| **"CABLE Output not found"** | Restart app, check VB-Cable in Sound settings |

---

## 🎉 You're Ready!

Your system is **completely set up and ready to test!**

Just follow the steps above and watch it detect profanity in real-time! 🎵

**Start with:** `npx tauri dev`

**Test with:** YouTube + explicit music

**Result:** Beep alerts on bad words! ✅
