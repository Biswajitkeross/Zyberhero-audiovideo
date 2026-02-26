# 🧪 Complete Testing Guide - Feb 18, 2026

## System Status: ✅ READY FOR TESTING

The application is now running at `http://localhost:5173/` with:
- ✅ SimpleMonitor ONLY (no old AudioMonitor interference)
- ✅ Clean startup with no background audio capture
- ✅ Proper stop/start mechanism with timeout protection
- ✅ 1.5 second Whisper chunks (no more 990ms errors)
- ✅ Guard against multiple simultaneous instances

---

## 🎬 Test Procedure

### **Test 1: Verify Clean Startup** (5 minutes)

**Steps:**
1. Open browser: `http://localhost:5173/`
2. Open DevTools Console (`F12`)
3. Check Terminal for startup messages

**Expected Output in Terminal:**
```
✅ [Startup] Audio alert system initialized - Using SimpleMonitor only
🔊 [AudioAlert] Starting persistent audio thread...
```

**NOT Expected (indicates old code still running):**
```
🚀 [AudioMonitor] Detected Output Sample Rate...
whisper_full_with_state: input is too short - 990 ms...  ← BAD!
```

**Result:** ✅ PASS if clean startup, ❌ FAIL if old monitor messages appear

---

### **Test 2: Start Monitoring** (2 minutes)

**Steps:**
1. In browser console, run:
   ```javascript
   await invoke('start_simple_monitoring')
   ```

2. Check terminal for response

**Expected:**
- Console returns: `"Monitoring started"`
- Terminal shows: `🎤 [SimpleMonitor] Started - Listening for bad words...`

**Result:** ✅ PASS

---

### **Test 3: Play Clean Content (No Profanity)** (3 minutes)

**Steps:**
1. While monitoring is active
2. Open YouTube in another tab
3. Play a clean video (no bad words)
4. Let it play for 30+ seconds

**Expected:**
- ✅ Video plays normally
- ✅ NO beeps
- ✅ NO error messages in terminal
- ✅ Audio is clean and clear
- ✅ Terminal shows no detection messages

**Result:** ✅ PASS (silence is good!)

---

### **Test 4: Play Content with Profanity** (5 minutes)

**Steps:**
1. While monitoring is active
2. Open YouTube
3. Find video with profanity (or use audio clip: "What the fuck")
4. Play the video

**Expected Behavior:**
```
Timeline:
00:00 - Start playing video
00:05 - Bad word spoken
00:05 - YOU HEAR 3 BEEPS (not the bad word)
00:06 - Video continues normally
00:07 - No more beeps (only one detection)
```

**Expected Terminal Output:**
```
🚨 BAD WORD DETECTED #1: 'fuck' at 0.00s-0.10s
🚨 BAD WORD DETECTED #2: 'shit' at 5.00s-5.10s  ← If multiple bad words
```

**NOT Expected:**
```
🚨 BAD WORD DETECTED #1: 'fuck' at 0.00s-0.10s
🚨 BAD WORD DETECTED #1: 'fuck' at 0.00s-0.10s  ← Duplicate detection (bad!)
🚨 BAD WORD DETECTED #1: 'fuck' at 0.00s-0.10s
```

**Critical Checks:**
- [ ] Exactly 3 beeps per bad word (not 6, 9, 12, etc.)
- [ ] Beeps NOT overlapping or distorted
- [ ] Video continues, NOT repeated
- [ ] One detection per word spoken (not duplicated)
- [ ] Beeps play OVER the bad audio (user doesn't hear the word)

**Result:** ✅ PASS if all checks pass

---

### **Test 5: Stop Monitoring** (2 minutes)

**Steps:**
1. While monitoring is active
2. In console, run:
   ```javascript
   await invoke('stop_simple_monitoring')
   ```

**Expected:**
- Console returns: `"Monitoring stopped"`
- Terminal shows: `✅ [SimpleMonitor] Task stopped cleanly`

**Result:** ✅ PASS

---

### **Test 6: Continue Playing After Stop** (2 minutes)

**Steps:**
1. After stopping monitoring
2. Keep YouTube playing (or play new video)
3. If video has profanity, no beeps should occur

**Expected:**
- ✅ Video continues normally
- ✅ Audio is clean (no interference)
- ✅ NO MORE BEEPS (monitoring is off)
- ✅ Multiple bad words can be spoken without any beeps

**Result:** ✅ PASS if NO beeps occur

---

### **Test 7: Restart Monitoring** (3 minutes)

**Steps:**
1. Click "Start Monitoring" again in console:
   ```javascript
   await invoke('start_simple_monitoring')
   ```

2. Play YouTube with profanity

**Expected:**
- ✅ Beeps work again
- ✅ Single beep (not duplicated)
- ✅ No issues from first session
- ✅ Fresh detection count (resets to 0)

**Terminal should show:**
```
🎤 [SimpleMonitor] Started - Listening for bad words...
🚨 BAD WORD DETECTED #1: 'fuck' at X.XXs-X.XXs  ← Starts from #1 again
```

**Result:** ✅ PASS

---

### **Test 8: Multiple Start Attempts (Safety Check)** (1 minute)

**Steps:**
1. While monitoring is active
2. Try to start again:
   ```javascript
   await invoke('start_simple_monitoring')
   ```

**Expected:**
- Returns error: `"Monitoring already active. Stop it first before starting again."`
- Monitoring continues (not restarted)
- No duplicate instances created

**Result:** ✅ PASS (safety guard works!)

---

## 📊 Summary Checklist

Use this to track all tests:

```
Test 1: Clean Startup           [ ] ✅ PASS  [ ] ❌ FAIL  [ ] ⏭️ SKIP
Test 2: Start Monitoring         [ ] ✅ PASS  [ ] ❌ FAIL  [ ] ⏭️ SKIP
Test 3: Play Clean Content       [ ] ✅ PASS  [ ] ❌ FAIL  [ ] ⏭️ SKIP
Test 4: Play Profanity          [ ] ✅ PASS  [ ] ❌ FAIL  [ ] ⏭️ SKIP
Test 5: Stop Monitoring          [ ] ✅ PASS  [ ] ❌ FAIL  [ ] ⏭️ SKIP
Test 6: Continue After Stop      [ ] ✅ PASS  [ ] ❌ FAIL  [ ] ⏭️ SKIP
Test 7: Restart Monitoring       [ ] ✅ PASS  [ ] ❌ FAIL  [ ] ⏭️ SKIP
Test 8: Multiple Start Guard     [ ] ✅ PASS  [ ] ❌ FAIL  [ ] ⏭️ SKIP

Overall Result: [ ] ✅ ALL PASS  [ ] ⚠️ SOME FAIL  [ ] ❌ CRITICAL FAIL
```

---

## 🔧 Debugging Hints

### If beeps repeat (6, 9, 12 times instead of 3):

1. Check lib.rs line 195 - verify old AudioMonitor is commented out
2. Restart dev server: `Ctrl+C` then `npx tauri dev`
3. Verify startup message says "Using SimpleMonitor only"

### If songs repeat/loop:

1. Means old delay buffer is still active
2. Delete `src-tauri/target` folder and rebuild:
   ```bash
   rm -r src-tauri/target
   npx tauri dev
   ```

### If 990ms errors appear in terminal:

1. Old WASAPI capture is still running
2. Check if old monitor code is still being used
3. Verify lib.rs line 195 has old monitor commented out
4. Full rebuild might be needed

### If stop button doesn't work:

1. Check if timeout waiting (up to 5 seconds)
2. Manually test in console: `await invoke('stop_simple_monitoring')`
3. Verify terminal shows: `✅ [SimpleMonitor] Task stopped cleanly`

---

## 📞 Quick Commands Reference

```javascript
// Start monitoring
await invoke('start_simple_monitoring')

// Stop monitoring  
await invoke('stop_simple_monitoring')

// Check status
const status = await invoke('get_simple_monitoring_status')
console.log(status)
// Output: { is_monitoring: true, detection_count: 2 }

// Test alert sound
await invoke('play_alert')

// Check bad words in text
const detected = await invoke('check_bad_words', { text: 'hello fuck world' })
console.log(detected)  // Output: ['fuck']
```

---

## ✅ Success Criteria

**System is working correctly if:**
- ✅ Startup is clean (no 990ms errors)
- ✅ Beeps play exactly 3 times per bad word (not 6+)
- ✅ Beeps are clean (not distorted/overlapping)
- ✅ Stop button works instantly
- ✅ No repeated/looped songs
- ✅ Can start/stop multiple times without issues
- ✅ Guard prevents multiple instances

**System needs more work if:**
- ❌ Startup shows 990ms Whisper errors
- ❌ Beeps repeat (6+, 9+, 12+ times)
- ❌ Beeps are garbled/distorted
- ❌ Stop button freezes the app
- ❌ Songs repeat or loop
- ❌ App crashes after stop/start cycle

---

## 🎯 Next Phase (After Testing)

Once tests pass, implement:

1. **UI Buttons for Monitoring**
   - "Start Monitoring" button → calls `start_simple_monitoring`
   - "Stop Monitoring" button → calls `stop_simple_monitoring`
   - Status display → calls `get_simple_monitoring_status`

2. **Auto-Pause on Bad Word**
   - Listen for `pause-audio-app` event
   - Pause YouTube playback automatically
   - Or show popup to pause manually

3. **Detection Log**
   - Display detected words and timestamps
   - Show how many bad words detected

4. **Cleanup** (Optional)
   - Delete old monitor files
   - Remove unused code
   - Update documentation

---

**Happy Testing!** 🎉
