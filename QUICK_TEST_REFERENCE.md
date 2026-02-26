# ⚡ Quick Testing Reference - 5 Minutes to Results

## 🚀 THE FASTEST PATH TO TESTING

### **Step 1: API Key** (30 seconds)
```powershell
$env:OPENAI_API_KEY = "sk-your-key-here"
echo $env:OPENAI_API_KEY  # Verify it shows your key
```

### **Step 2: Start App** (30 seconds)
```powershell
cd "C:\Users\USR-LPTP-81\Desktop\zybertest-desktop"
npx tauri dev
```

**Wait for this message:**
```
✅ OpenAI Whisper API enabled
```

### **Step 3: Open Browser** (30 seconds)
- URL: `http://localhost:5173/`
- See the dashboard

### **Step 4: Start Monitoring** (30 seconds)
- Click: **"Start Monitoring All Audio"**
- Status should show: **"MONITORING ACTIVE"** ✅

### **Step 5: Test with YouTube** (2 minutes)
1. Open NEW browser tab
2. Go to YouTube: `youtube.com`
3. Search: `explicit rap songs` or `bad words`
4. Click play on any video
5. **Volume on reasonable level**
6. Wait for bad words to be spoken...

### **EXPECTED RESULT**
- Terminal shows: `📝 Whisper: [words heard]`
- Terminal shows: `🚨 BAD WORDS DETECTED: [word list]`
- **App BEEPS** 🔊🔊 (double beep)
- Dashboard updates with detection

---

## 🎯 VISUAL FLOW

```
YouTube Audio
    ↓
VoiceMeeter (routing)
    ↓
VB-Cable (virtual input)
    ↓
App (listens)
    ↓
Energy Check (strong enough?)
    ↓
Whisper API (what words?)
    ↓
Bad Word Detector (is it bad?)
    ↓
BEEP! 🔊 (alert!)
    ↓
Dashboard (show detection)
```

---

## ✅ VERIFICATION CHECKLIST

Before testing:
- [ ] VB-Cable showing in Windows Sound Settings
- [ ] VoiceMeeter running (showing audio levels)
- [ ] API key set in PowerShell
- [ ] App showing "✅ OpenAI Whisper API enabled"
- [ ] Monitoring button says "Start Monitoring All Audio"

During testing:
- [ ] Click "Start Monitoring"
- [ ] Status shows "MONITORING ACTIVE"
- [ ] YouTube video playing
- [ ] Terminal shows "🎵 Strong audio detected"
- [ ] Beep sound plays

After detection:
- [ ] Activity log shows entry
- [ ] Counter incremented
- [ ] Timestamp present

---

## 🎵 TEST SOURCES

### **WORKS (has actual bad words in audio):**
✅ Explicit rap songs
✅ Comedy sketches with profanity
✅ Movie clips with bad words
✅ Documentary with curse words

### **WON'T WORK (no audio profanity):**
❌ Clean/radio versions (bad words censored out)
❌ Instrumental music (no words)
❌ Background music (no speech)
❌ Songs with only music (no vocals)

---

## 🔧 TROUBLESHOOTING (2 Minute Fixes)

| See This | Do This |
|----------|---------|
| **No beep at all** | 1. Check Windows volume<br>2. Check system sounds enabled<br>3. Restart app |
| **Terminal blank** | 1. Click "Start Monitoring" button<br>2. Check status shows "MONITORING ACTIVE"<br>3. Check YouTube is playing |
| **"CABLE Output not found"** | 1. Restart app<br>2. Check VB-Cable in Sound Settings<br>3. Restart VoiceMeeter |
| **Manual test works, YouTube doesn't** | VoiceMeeter routing wrong - check Volume Mixer |
| **App crashes** | Check terminal for error message |
| **API key error** | Verify key at: https://platform.openai.com/api-keys |

---

## 📊 WHAT EACH OUTPUT MEANS

### **Terminal - App Startup**
```
✅ OpenAI Whisper API enabled
→ Your API key was found and loaded! ✅
```

### **Terminal - Audio Detected**
```
🎵 Strong audio detected (energy: 0.0234, peak: 0.67)
→ App is hearing YouTube audio! ✅
```

### **Terminal - Bad Word Found**
```
📝 Whisper: fuck this shit
🚨 BAD WORDS DETECTED: fuck, shit (Count: 1)
→ App recognized the bad words! ✅
```

### **App Dashboard - Beep**
```
🔊 Double beep sound plays
→ Alert system working! ✅
```

### **App Dashboard - Activity Log**
```
[15:30:45] Detected: fuck, shit
→ History recorded! ✅
```

---

## 🎬 REAL EXAMPLE SESSION

**Time: 0:00 - Start**
```powershell
PS> npx tauri dev
✅ OpenAI Whisper API enabled
```

**Time: 0:30 - Browser Open**
- Navigate to http://localhost:5173/
- See dashboard

**Time: 1:00 - Monitoring Started**
- Click "Start Monitoring All Audio"
- Status: "MONITORING ACTIVE"

**Time: 1:30 - YouTube Playing**
- New tab, YouTube open
- Playing explicit song
- Volume normal

**Time: 2:00 - First Detection**
```
🎵 Strong audio detected (energy: 0.0456, peak: 0.85)
📝 Whisper: yo fuck this shit up
🚨 BAD WORDS DETECTED: fuck, shit (Count: 1)
BEEP! 🔊 🔊
```

**Dashboard Shows:**
```
Activity Log:
  [15:32:18] Detected: fuck, shit

Detection Counter: 1
Status: MONITORING ACTIVE
```

**Time: 2:30 - Success** ✅ System working!

---

## 🚀 NEXT STEPS

1. **Test Manual Detection First** (optional, quick verification)
   - Type bad word in manual section
   - Click "Test Detection"
   - Should immediately show detection

2. **Test YouTube** (main test)
   - Play video with profanity
   - Wait for bad words
   - Listen for beep

3. **Test Multiple Detections** (optional, verify stability)
   - Play longer video
   - Let app detect multiple bad words
   - Check counter increments

4. **Adjust Sensitivity** (if needed)
   - Too many false positives? → Increase thresholds
   - Missing detections? → Decrease thresholds
   - See TESTING_GUIDE.md for details

---

## ✨ YOU'RE READY!

**Just run:**
```powershell
npx tauri dev
```

**Then go to:** `http://localhost:5173/`

**And test with:** YouTube + explicit music

**System will:** Beep on bad words! 🔊

---

**Total time to first successful test: ~5 minutes**

Good luck! 🎉
