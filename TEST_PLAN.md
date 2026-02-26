# Audio Monitor - Final Test Plan

## Current Status
✅ **System IS WORKING** - Detected "slut" word in YouTube song and played beep!

**Last Detection Output:**
```
🎵 STRONG speech/singing detected (energy: 0.03573, peak: 0.35)
🚨 Bad words detected: ..., slut, ... (Count: 1)
✓ Double beep alert played
```

## Final Thresholds (ACTIVE NOW)
- **Energy > 0.02** - Catches YouTube/strong speech
- **Peak > 0.35** - Catches sung lyrics and speech
- **Cooldown: 1 second** - Prevents spam beeping

---

## Test Instructions

### Test 1: Silence Test (Should NOT beep)
1. Open http://localhost:5173/
2. Click **"Start Monitoring All Audio"**
3. **DO NOT play any music** - just silence for 10 seconds
4. **Expected:** No beep, no "Bad words detected" in terminal
5. Click **"Stop Monitoring"**

### Test 2: YouTube Music WITH BAD WORDS (Should beep 🔊)
1. Open http://localhost:5173/
2. Click **"Start Monitoring All Audio"**
3. Play YouTube video with profanity (like the song you used: slut, fuck, shit, etc.)
4. **Expected:** 
   - App beeps (double beep sound)
   - Terminal shows: `🎵 STRONG speech/singing detected`
   - Terminal shows: `🚨 Bad words detected: ... (Count: X)`
   - Counter in UI increments

### Test 3: YouTube INSTRUMENTAL/No Words (Should NOT beep)
1. Start monitoring
2. Play YouTube instrumental music (no lyrics/words)
3. **Expected:** No beep, no detection messages

### Test 4: Background Noise (Should NOT beep)
1. Start monitoring
2. Create background noise (typing, papers, talking quietly in background)
3. **Expected:** No beep, no detection messages

### Test 5: Manual Text Detection (Always works)
1. In "Test Text for Bad Words" section
2. Type: "This is fucking shit"
3. Click "Check Text"
4. **Expected:** Red badges show detected words (fuck, shit)

---

## What to Report

If it works:
✅ Detected bad word from YouTube with beep
✅ Did NOT beep on silence
✅ Did NOT beep on background noise
✅ Counter incremented correctly
✅ Activity log shows detection

If it doesn't work:
❌ Show me the terminal output when you:
   1. Start monitoring
   2. Play YouTube song with bad words
   3. Wait 10 seconds
   4. Stop monitoring
   
   Copy-paste everything you see in the terminal!

---

## Audio Setup Checklist

✅ VB-Cable installed (virtual audio device)
✅ VoiceMeeter running (audio router)
✅ Windows Volume Mixer configured:
   - System Output → VoiceMeeter Input
   - System Input → VoiceMeeter Out B3
   - App Input → CABLE Output

If beep still doesn't work:
- Check VB-Cable is working: VB-Cable Control Panel should show activity
- Check VoiceMeeter: A1 → Headphones (ON), B1 → CABLE Output (ON)
- Try increasing system volume to maximum

---

## Expected Behavior Summary

| Scenario | Should Beep? | Count Increase? |
|----------|-------------|-----------------|
| Silence | ❌ No | ❌ No |
| YouTube with bad words | ✅ Yes | ✅ Yes |
| YouTube instrumental | ❌ No | ❌ No |
| Background noise | ❌ No | ❌ No |
| Manual text (bad words) | N/A | ✅ Yes |
| Music played in app | ✅ Yes (if has bad words) | ✅ Yes |

---

## System Thresholds Explained

**Why these specific thresholds?**

From actual YouTube audio data we captured:
- Silence: energy 0.000-0.005, peak 0.00-0.15
- Background noise: energy 0.005-0.01, peak 0.15-0.25
- YouTube music: energy 0.03-0.04, peak 0.35-0.40 ← **OUR TARGET**

By setting `energy > 0.02` and `peak > 0.35`, we:
- ✅ Catch all YouTube songs (energy 0.03-0.04)
- ✅ Catch clear speech (peak > 0.35)
- ✅ Ignore silence (energy < 0.005)
- ✅ Ignore background noise (energy < 0.01)

---

## Support

If detection still not working:
1. Check terminal for `🎵 STRONG speech/singing detected` messages
2. If you see that but no beep → Audio output problem
3. If you don't see that → Audio input problem (routing)
4. Show me the terminal output from the test!
