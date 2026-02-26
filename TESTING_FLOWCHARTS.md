# 🎬 VISUAL TESTING FLOWCHART

## THE COMPLETE TESTING JOURNEY

```
┌─────────────────────────────────────────────────────────────────┐
│                    TESTING YOUR APP                             │
└─────────────────────────────────────────────────────────────────┘

                              START
                                │
                                ▼
                ┌─────────────────────────────┐
                │  1. SET API KEY             │
                │  $env:OPENAI_API_KEY=       │
                │     "sk-your-key"           │
                │  ✓ Verify with echo         │
                └────────────┬────────────────┘
                             │
                             ▼
                ┌─────────────────────────────┐
                │  2. START APP               │
                │  npx tauri dev              │
                │  ✓ Wait for startup         │
                └────────────┬────────────────┘
                             │
                             ▼
                ┌─────────────────────────────┐
                │  SEE MESSAGE?               │
                │  "✅ OpenAI Whisper         │
                │   API enabled"              │
                └────────┬────────────┬───────┘
                   YES │             │ NO
                       ▼             ▼
                    ✅              ❌ PROBLEM
              Continue          Check API key
                       │
                       ▼
                ┌─────────────────────────────┐
                │  3. OPEN BROWSER            │
                │  http://localhost:5173/     │
                │  ✓ See dashboard            │
                └────────────┬────────────────┘
                             │
                             ▼
                ┌─────────────────────────────┐
                │  4. OPTIONAL: TEST MANUAL   │
                │  Type bad word              │
                │  Click "Test Detection"     │
                │  ✓ Should show detection    │
                └────────────┬────────────────┘
                             │
                             ▼
                ┌─────────────────────────────┐
                │  5. CLICK "START MONITORING"│
                │  ✓ Status changes to        │
                │    "MONITORING ACTIVE"      │
                └────────────┬────────────────┘
                             │
                             ▼
                ┌─────────────────────────────┐
                │  6. OPEN NEW BROWSER TAB    │
                │  Go to YouTube              │
                │  Search: "explicit rap"     │
                │  ✓ Play video with audio    │
                └────────────┬────────────────┘
                             │
                             ▼
                ┌─────────────────────────────┐
                │  WAIT FOR BAD WORD          │
                │  TO BE SPOKEN IN VIDEO      │
                └────────────┬────────────────┘
                             │
                             ▼
                ┌─────────────────────────────┐
                │  7. DID YOU HEAR BEEP?      │
                │  🔊 🔊 (double beep)        │
                └────────┬────────────┬───────┘
                   YES │             │ NO
                       ▼             ▼
                    ✅              Check:
                SUCCESS!       • VoiceMeeter
                               • CABLE Output
                               • Volume
                       │
                       ▼
                ┌─────────────────────────────┐
                │  8. CHECK APP DASHBOARD     │
                │  ✓ Activity Log updated     │
                │  ✓ Counter incremented      │
                │  ✓ Timestamp visible        │
                └────────────┬────────────────┘
                             │
                             ▼
                ┌─────────────────────────────┐
                │  9. CHECK TERMINAL OUTPUT   │
                │  📝 Whisper: [text heard]   │
                │  🚨 BAD WORDS: [list]       │
                │  ✓ API key worked!          │
                └────────────┬────────────────┘
                             │
                             ▼
                        🎉 SUCCESS! 🎉
                   Your app is working!

```

---

## DATA FLOW DIAGRAM

```
YOUTUBE VIDEO (with profanity)
    │
    │ Audio Output
    ▼
┌────────────────────────────┐
│  VoiceMeeter               │
│  - Receives YouTube audio  │
│  - Routes to Virtual Cable │
└────────────┬───────────────┘
    │
    │ Virtual Audio Stream
    ▼
┌────────────────────────────┐
│  VB-Cable (Virtual Device) │
│  - Input: YouTube audio    │
│  - Output: To App          │
└────────────┬───────────────┘
    │
    │ Audio Samples (48kHz)
    ▼
┌────────────────────────────┐
│  App's Audio Capture       │
│  (CPAL Library)            │
│  - Reads: 48000 Hz         │
│  - 2 seconds samples       │
└────────────┬───────────────┘
    │
    │ Raw Audio Frame
    ▼
┌────────────────────────────┐
│  Energy Detection          │
│  - Energy > 0.02?          │
│  - Peak > 0.35?            │
│  - Is it speech?           │
└────────────┬───────────────┘
    │
    ├─ NO  → Skip (not speech)
    │
    └─ YES → Send to API
         │
         ▼
┌────────────────────────────┐
│  OpenAI Whisper API        │
│  - Convert audio to text   │
│  - Accuracy: 99%           │
│  - Get: "fuck this shit"   │
└────────────┬───────────────┘
    │
    │ Recognized Text
    ▼
┌────────────────────────────┐
│  Bad Word Detector         │
│  - Check: 22 words list    │
│  - Match found?            │
│  - fuck = YES              │
│  - shit = YES              │
└────────────┬───────────────┘
    │
    ├─ NO MATCH   → Silent (clean)
    │
    └─ MATCH FOUND → Alert!
         │
         ▼
┌────────────────────────────┐
│  Audio Alert System        │
│  - Generate beep tone      │
│  - BEEP! 🔊                │
│  - BEEP! 🔊 (double)       │
└────────────┬───────────────┘
    │
    ├──────────────┬──────────────┐
    │              │              │
    ▼              ▼              ▼
   LOG         DASHBOARD      TERMINAL
   │           │              │
   │           ▼              ▼
   │    Activity Log     [15:30:45]
   │    [15:30:45]    Detected:
   │    Detected:     fuck, shit
   │    fuck, shit
   │                Terminal Output:
   │                📝 Whisper: 
   │                   fuck this shit
   │                🚨 BAD WORDS
   │                   DETECTED
   │
   └──► Counter Incremented (1 → 2)
```

---

## TIMING DIAGRAM

```
TIME    ACTION              EXPECTED RESULT
────────────────────────────────────────────────────────────
0:00    Run: npx tauri dev  App starts

0:30    Terminal shows:     ✅ API key loaded
        "✅ OpenAI Whisper
         API enabled"

1:00    Browser opens       Dashboard visible
        http://localhost:
        5173/

1:30    Click "Start        Button changes
        Monitoring All      Status: "MONITORING
        Audio"              ACTIVE"

2:00    YouTube starts      
        playing             

2:15    [Waiting for bad    Terminal shows:
        word to be said]    🎵 Strong audio
                           detected

2:45    Bad word spoken     Terminal shows:
        "fuck"              📝 Whisper: fuck
                           🚨 BAD WORDS
                           DETECTED: fuck

2:46    BEEP SOUND!         Dashboard updates:
                           Activity Log:
                             [time] fuck
                           Counter: 1

2:48    Next bad word       Same process
        spoken              repeats...

3:00    Another beep!       Counter: 2

3:30    Stop Monitoring     Button changes
        (click button)      Status: "STOPPED"

```

---

## STATE DIAGRAM

```
                        ┌─────────────────┐
                        │  APP STARTUP    │
                        └────────┬────────┘
                                 │
                                 ▼
                    ┌─────────────────────────┐
                    │  READ API KEY FROM ENV  │
                    │  std::env::var()        │
                    └────────┬────┬───────────┘
                    Found   │    │    Not Found
                            ▼    ▼
                    ✅ ENABLED  ❌ DISABLED
                            │    │
                            └──┬─┘
                               ▼
                    ┌─────────────────────────┐
                    │  WAITING FOR USER TO    │
                    │  CLICK "START"          │
                    └────────────┬────────────┘
                                 │
                                 ▼
                    ┌─────────────────────────┐
                    │  MONITORING ACTIVE      │
                    │  (listening for audio)  │
                    └────────────┬────────────┘
                                 │
                    ┌────────────┴────────────┐
                    │                        │
                    ▼                        ▼
            [Audio Detected]        [No Audio]
                    │                    │
                    ▼                    ▼
        ┌──────────────────────┐     (stay in
        │ ANALYZING AUDIO      │      MONITORING)
        │ • Energy check       │
        │ • Peak check         │
        └──┬─────────────┬─────┘
       WEAK│            │STRONG
           ▼            ▼
        ┌──────┐   ┌─────────────────┐
        │SKIP  │   │ SEND TO WHISPER │
        │      │   │ API             │
        └──────┘   └────────┬────────┘
                            │
                            ▼
                ┌────────────────────────┐
                │ TEXT RECEIVED FROM API │
                │ "fuck this shit"       │
                └────────┬───────────────┘
                         │
                         ▼
                ┌────────────────────────┐
                │ CHECK BAD WORD LIST    │
                │ (22 words)             │
                └────────┬───────────────┘
                         │
                ┌────────┴────────┐
                │                 │
             MATCH            NO MATCH
                │                 │
                ▼                 ▼
        ┌──────────────┐    ┌──────────┐
        │ PLAY BEEP    │    │ SILENT   │
        │ BEEP! 🔊 🔊  │    │ (clean)  │
        │ UPDATE LOG   │    └──────────┘
        │ INCREMENT    │
        │ COUNTER      │
        └──────┬───────┘
               │
               ▼
        ┌──────────────────────┐
        │ BACK TO MONITORING   │
        │ (continue listening) │
        └──────────────────────┘

```

---

## EXPECTED OUTPUT SAMPLES

### **Terminal Output During Test**

```
[APP STARTUP]
✅ OpenAI Whisper API enabled
🎤 Initializing AudioMonitor with Whisper speech recognition...

[MONITORING STARTED]
🎤 Audio monitoring started

[AUDIO DETECTED FROM YOUTUBE]
🎵 Strong audio detected (energy: 0.0234, peak: 0.67)
🎵 Strong audio detected (energy: 0.0456, peak: 0.85)
🎵 Strong audio detected (energy: 0.0389, peak: 0.72)

[BAD WORD DETECTED]
📝 Whisper: fuck this shit
🚨 BAD WORDS DETECTED: fuck, shit (Count: 1)

[ANOTHER WORD]
📝 Whisper: damn
🚨 BAD WORDS DETECTED: damn (Count: 2)

[MORE AUDIO]
🎵 Strong audio detected (energy: 0.0445, peak: 0.81)

[CLEAN AUDIO - NO BEEP]
📝 Whisper: hello there
✅ No bad words detected

[MORE DETECTIONS]
🎵 Strong audio detected (energy: 0.0567, peak: 0.92)
📝 Whisper: what the hell
🚨 BAD WORDS DETECTED: hell (Count: 3)

[MONITORING STOPPED]
🎤 Audio monitoring stopped
```

### **Dashboard Activity Log**

```
Activity Log:
  [15:32:45] Detected: fuck, shit
  [15:33:12] Detected: damn
  [15:33:45] Detected: hell
  [15:34:18] Detected: bitch, ass
  
Detection Counter: 4
Status: MONITORING ACTIVE
Last Detection: 15:34:18
```

---

## 🎯 SUCCESS INDICATORS

✅ **You will know it's working when you see:**

1. **Terminal Message on Startup:**
   ```
   ✅ OpenAI Whisper API enabled
   ```

2. **Audio Detection Message:**
   ```
   🎵 Strong audio detected (energy: 0.0234, peak: 0.67)
   ```

3. **Recognition Message:**
   ```
   📝 Whisper: [words that were heard]
   ```

4. **Bad Word Detection Message:**
   ```
   🚨 BAD WORDS DETECTED: [list of bad words]
   ```

5. **Physical Beep Sound:**
   ```
   🔊 🔊 (double beep - you should HEAR this!)
   ```

6. **Dashboard Update:**
   - Activity log has new entry
   - Counter incremented
   - Timestamp added

---

## ❌ FAILURE INDICATORS

❌ **If you see these, something's wrong:**

| Problem | Cause | Fix |
|---------|-------|-----|
| No beep sound | System audio muted | Check volume |
| No terminal messages | Monitoring not started | Click "Start" button |
| "API key error" | Wrong or missing key | Check PowerShell var |
| "CABLE Output not found" | VB-Cable not set up | Check VB-Cable install |
| Manual test works, YouTube doesn't | VoiceMeeter routing | Check Volume Mixer |
| "No modules named..." | Python issue | Restart app |
| Slow detection | Normal (2-3 sec) | Don't worry |

---

## 📋 CHECKLIST BEFORE TESTING

- [ ] VB-Cable installed (in Sound devices)
- [ ] VoiceMeeter running (showing levels)
- [ ] API key set in PowerShell
- [ ] App compiled (0 errors)
- [ ] Browser can access localhost:5173
- [ ] Volume not muted in Windows
- [ ] System sounds enabled

---

## 🚀 GO TIME!

**You're ready to test!**

```
Step 1: npx tauri dev
Step 2: Open http://localhost:5173/
Step 3: Click "Start Monitoring All Audio"
Step 4: Open YouTube
Step 5: Search "explicit rap"
Step 6: Play video
Step 7: Listen for BEEP! 🔊
Step 8: SUCCESS! 🎉
```

Total time: ~5 minutes

Good luck! 🎵
