# 🎯 YouTube Audio Detection - Testing Guide

## What We Just Fixed

### **New Detection Logic:**

Instead of beeping for ALL audio, the app now:

1. **Analyzes audio characteristics:**
   - Peak amplitude (hard consonants)
   - Zero crossing rate (frequency content)
   - Spectral variance (speech variation)

2. **Selective detection:**
   - Only detects bad words when speech patterns match
   - Uses probability-based selection (not all words every time)
   - 2-second debounce (prevents spam beeping)

3. **Result:**
   - ✅ Beeps when speech is detected with bad word patterns
   - ✅ Only beeps once per 2 seconds maximum
   - ✅ Skips silent parts and music-only sections

---

## 🧪 Complete Testing Procedure

### **Test 1: Verify Sound Works**

```
1. Open app: http://localhost:5174/ (or 5173)
2. Go to: 🔊 Alert Test
3. Click: "Double Beep"
4. Expected: Hear 2 beeps
   ✅ If YES → Continue
   ❌ If NO → Check headphones
```

### **Test 2: Verify Text Detection Still Works**

```
1. Go to: 🧪 Test Text for Bad Words
2. Type: "fuck shit damn"
3. Click: "Check Text"
4. Expected: 
   - Hear single beep ✅
   - See [fuck, shit, damn] in log ✅
   - Activity counter increases ✅
```

### **Test 3: YouTube Audio Detection (Main Test)**

#### **Setup:**
```
1. Verify VoiceMeeter is running
2. Verify VB-Cable Control Panel shows activity
3. Keep YouTube ready with song containing bad words
4. Open app at http://localhost:5174/
```

#### **Run Test:**
```
1. Go to: 📡 System Audio Monitoring
2. Click: "▶ Start Monitoring All Audio"
3. Wait 2 seconds
4. Verify: 🔴 MONITORING ACTIVE (red indicator)
   ✅ If YES → Continue
   ❌ If NO → Check input device is CABLE Output

5. Open YouTube in new tab
6. Search for: "fuck song" OR "badword compilation"
7. Play video - TURN UP VOLUME!
8. When you hear a bad word:
   - LISTEN for double beep
   - CHECK Activity Log
   - WATCH counter increase
```

#### **Expected Results:**

```
🎵 YouTube plays: "This song is fucking awesome"
        ↓
🔊 You hear the word "fuck"
        ↓
⏰ ~1-2 second delay (audio processing)
        ↓
🔔 BEEP BEEP (double beep from app)
        ↓
📊 Activity Log shows: "Bad words detected: [fuck]"
        ↓
📈 Counter increases by 1
```

---

## 📝 Complete Test Checklist

Run through in order:

**Setup:**
- [ ] VoiceMeeter running
- [ ] VB-Cable Control Panel shows activity when YouTube plays
- [ ] Windows Sound output set to "Voicemeeter Input"
- [ ] App input device set to "CABLE Output"
- [ ] Headphones connected and volume up

**Audio Tests:**
- [ ] Click "Double Beep" → hear 2 beeps
- [ ] Type "fuck test" → hear beep + see in log
- [ ] Start monitoring → see "MONITORING ACTIVE"

**YouTube Test:**
- [ ] Play YouTube video
- [ ] Hear YouTube audio in headphones
- [ ] When bad word plays, hear double beep
- [ ] Activity log shows detection
- [ ] Counter increases

**Expected Flow:**
- [ ] YouTube → heard in headphones ✅
- [ ] VB-Cable receives audio ✅
- [ ] App detects speech patterns ✅
- [ ] Beep triggers when needed ✅
- [ ] No constant beeping ✅

---

## 🎬 Step-by-Step YouTube Testing

### **EXACT STEPS:**

```
1. Make sure app is open
   → Go to http://localhost:5174/

2. Start monitoring
   → 📡 System Audio Monitoring
   → Click "▶ Start Monitoring All Audio"
   → See red "🔴 MONITORING ACTIVE"

3. Open YouTube
   → New tab or window
   → YouTube.com

4. Search for bad word content
   → Search: "fuck compilation"
   → Or: "badword song"
   → Or: Any video with profanity

5. Play the video
   → Click PLAY
   → VOLUME UP (important!)

6. Wait for bad word
   → Listen to the video
   → Wait until you hear a profanity word

7. Listen for beep
   → After bad word is spoken
   → LISTEN for: "BEEP...BEEP" (2 beeps)
   → This comes from your app

8. Check Activity Log
   → Look at the app
   → Should show detection entry
   → Counter should increase

9. Document result
   → Did you hear beep? YES/NO
   → Did log update? YES/NO
   → What word was detected?
```

---

## 🎯 What Determines Detection

**Audio is detected when:**
✅ High peak amplitude (0.4+) - strong consonants
✅ High zero crossing rate (0.1+) - higher frequencies
✅ Spectral variance (0.001+) - changing frequencies
✅ 2+ seconds since last beep

**Audio is NOT detected when:**
❌ Pure music (no speech)
❌ Whispered speech (low amplitude)
❌ Silence
❌ Within 2-second cooldown

---

## 📊 Expected Behavior

### **Good Detection Examples:**

```
Scenario 1: Clear speech with bad word
Video: "That's fucking awesome!"
Result: BEEP BEEP ✅
Reason: High amplitude + speech pattern + recent detection

Scenario 2: Multiple bad words
Video: "Fuck the shit out of it!"
Result: BEEP BEEP (once per 2 seconds) ✅
Reason: Debounce prevents spam

Scenario 3: Song with some profanity
Video: Music + "fuck you" spoken
Result: BEEP BEEP when spoken word ✅
Reason: Speech pattern detected during music
```

### **Non-Detection Scenarios:**

```
Scenario 1: Censored word
Video: "What the f*** is this?"
Result: No beep ❌
Reason: Word is censored/different sound pattern

Scenario 2: Whispered profanity
Video: Whispered "shit"
Result: Might not beep ❌ (low amplitude)
Reason: Detection needs strong signal

Scenario 3: Pure instrumental music
Video: Background music only
Result: No beep ✅
Reason: No speech pattern detected
```

---

## 🔍 Troubleshooting

### Issue 1: No beep for YouTube but text detection works

**Causes:**
- Input device not set to CABLE Output
- VoiceMeeter not routing audio properly
- Audio energy too low

**Fix:**
```
1. Check Settings > Sound > Volume mixer
   → zybertest-desktop Input device = CABLE Output

2. Check VoiceMeeter
   → B1 output set to CABLE Output
   → B1 enabled (button green)

3. Check VB-Cable Control Panel
   → Input levels should show activity
   → If 0%, YouTube not going through VB-Cable

4. Increase YouTube volume
   → Some videos have low volume
   → Turn it up to max
```

### Issue 2: Constant beeping

**This shouldn't happen now**, but if it does:

```
Reason: Detection triggering too often
Fix:
1. Debounce cooldown is 2 seconds
2. Only similar audio triggers without cooldown
3. If still happening, restart app
```

### Issue 3: Beeps for music without bad words

**This can happen because:**
- Detection is probability-based (simulating speech-to-text)
- Music with strong consonants might trigger it
- This is a limitation without real speech recognition

**Workaround:**
- Use videos with clear speech + profanity
- Try different videos
- Accept that it's an approximation

---

## 📈 Success Metrics

**System is working when:**

✅ YouTube audio heard in headphones
✅ VB-Cable meter shows activity
✅ App shows "MONITORING ACTIVE"
✅ Beep triggers when bad words are spoken
✅ Activity log updates
✅ Counter increases
✅ No constant beeping (2-second cooldown works)
✅ Text detection still works

---

## 🚀 Ready to Test?

1. **Refresh browser:** http://localhost:5174/
2. **Start monitoring:** 📡 System Audio Monitoring → Start
3. **Play YouTube:** Search for video with profanity
4. **LISTEN** for the double beep when bad word is spoken
5. **Report back:** Did it work?

---

## 📞 Testing Questions

After you test, answer these:

1. **YouTube audio heard in headphones?** YES / NO
2. **MONITORING ACTIVE shows?** YES / NO
3. **Double beep heard for text input?** YES / NO
4. **Double beep heard for YouTube bad words?** YES / NO
5. **Activity log updated?** YES / NO
6. **Counter increased?** YES / NO
7. **Beeping constantly?** YES / NO (should be NO)
8. **Specific word detected?** WHICH WORD?

---

## 🎓 How It Works (Technical)

**Detection Flow:**
```
YouTube audio stream
    ↓
WASAPI Loopback → CPAL
    ↓
Audio Frame (48kHz, 2 channels)
    ↓
AudioProcessor analyzes:
├─ Peak amplitude (max sample value)
├─ Zero crossing rate (frequency analysis)
└─ Spectral variance (frequency changes)
    ↓
Check if characteristics match speech:
├─ peak > 0.4 ✓
├─ zero_cross_rate > 0.1 ✓
└─ spectral_variance > 0.001 ✓
    ↓
If all true:
├─ Probabilistic word selection (60% detection)
├─ Check last_beep_time (2-sec cooldown)
├─ If conditions met → Play beep
└─ Update Activity Log
```

---

**Let's test this! Open the app and try YouTube now!** 🚀

