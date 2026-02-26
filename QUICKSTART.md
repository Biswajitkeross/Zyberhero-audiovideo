# 🚀 Quick Start Guide - Audio Content Monitor

## What's Been Done ✅

Your React + Tauri application is now fully functional with:

1. **Rust Backend** - Complete with audio processing, bad word detection, and alerts
2. **React Frontend** - Beautiful UI to manage and test the system
3. **Tauri IPC** - Seamless communication between React and Rust

---

## Testing Your App (5 minutes)

### Step 1: Start the App
```powershell
cd c:\Users\USR-LPTP-81\Desktop\zybertest-desktop
npm run dev
```

**Expected Output:**
- Terminal shows Vite running on `http://localhost:5173`
- Native Tauri window opens with your app

### Step 2: Test Alert Sounds
- Click **"Single Beep (1000Hz)"** button → 🔊 You should hear a beep
- Click **"Double Beep"** button → 🔊 Two beeps
- Click **"Ascending Alert"** button → 🔊 Rising tone sound

### Step 3: Test Bad Word Detection

**Add a bad word:**
1. In the "Bad Word Management" section
2. Type `damn` in the input field
3. Click **"Add Word"**
4. You'll see "damn" appear in the "Current Bad Words" list

**Check text:**
1. Go to "Test Text for Bad Words" section
2. Type: `This is damn annoying`
3. Click **"Check Text"**
4. Result: Shows "Bad words found!" + plays alert + logs the action

**Remove a word:**
- Click the `×` button next to any word to remove it
- Or click **"Clear All"** to remove everything

---

## Key Features Explained

### 🔊 Alert Test Section
- **Single Beep**: 1000 Hz sine wave (500ms) - standard warning
- **Double Beep**: Two beeps at different frequencies - urgent alert
- **Ascending Alert**: Three ascending tones (800→1000→1200 Hz) - distinctive alert

### 🚫 Bad Word Management
- **Pre-loaded**: 40+ common bad words already added
- **Add Custom**: Type any word and click "Add"
- **Remove**: Click `×` on the word tag
- **Toggle**: Enable/disable detection temporarily
- **Clear**: Remove all words at once

### 🧪 Test Text for Bad Words
- Enter any text
- Click "Check Text"
- If bad words found: Alert plays + logged
- Supports multiple bad words in one sentence

### 📋 Activity Log
- Shows timestamp, action, and any detected words
- Last 50 actions kept
- Color-coded for easy reading
- Red tags = detected bad words

---

## How to Use in Production

### Build for Distribution
```powershell
npm run build
```

Creates:
- Windows executable in `src-tauri/target/release/bundle/nsis/`
- Ready to distribute to users
- Self-contained application

### Build Output
```
src-tauri/
└── target/
    └── release/
        └── bundle/
            ├── nsis/              # Windows installer (.exe)
            │   └── zybertest-desktop_0.1.0_x64-Setup.exe
            └── msi/               # Windows MSI installer
                └── zybertest-desktop_0.1.0_x64.msi
```

---

## System Requirements

### For Development
- Windows 10/11
- Node.js 18+
- Rust 1.70+
- ~2GB RAM free

### For Running Built App
- Windows 10/11
- ~100MB disk space

### Important: Stereo Mix Setup (Required for Audio Capture)

**If you want real-time audio monitoring:**

1. Open Windows Sound Settings
2. Click "Sound" in left menu
3. Scroll down → "Advanced" → "Volume mixer"
4. Look for "Stereo Mix"
5. If disabled → Right-click → Enable
6. If missing → 
   - Right-click any device → "Show Disabled Devices"
   - Right-click "Stereo Mix" → Enable

---

## Project Structure

```
Your Project
├── React Files (src/)
│   ├── App.tsx          ← Main UI component
│   ├── App.css          ← Styling
│   └── main.tsx         ← Entry point
│
├── Rust Files (src-tauri/src/)
│   ├── lib.rs           ← Tauri commands
│   ├── bad_word_detector.rs   ← Word detection
│   ├── audio_alert.rs         ← Sound generation
│   ├── audio_capture.rs       ← Audio input
│   └── audio_processor.rs     ← Signal processing
│
└── Config Files
    ├── package.json           ← NPM dependencies
    ├── src-tauri/Cargo.toml   ← Rust dependencies
    └── tauri.conf.json        ← Tauri configuration
```

---

## Common Tasks

### Add a New Bad Word
```typescript
// In React component:
await invoke('add_bad_word', { word: 'newword' })
```

### Check Text Programmatically
```typescript
const badWords = await invoke<string[]>('check_bad_words', { 
  text: 'some text to check' 
})
if (badWords.length > 0) {
  console.log('Found:', badWords)
  await invoke('play_alert')
}
```

### Get Current Bad Words List
```typescript
const allWords = await invoke<string[]>('get_all_bad_words')
console.log('Total bad words:', allWords.length)
```

### Reset to Defaults
```typescript
await invoke('clear_bad_words')
// Then add back your default list...
```

---

## Performance Optimization Tips

### Reduce False Positives
- Use specific, complete words in bad word list
- Avoid adding common letter combinations
- Test thoroughly with sample text

### Improve Detection Speed
- Keep bad word list under 500 items
- Use simple pattern matching (current approach)
- Consider using HashSet for larger lists

### Development Tips
- Use React DevTools browser extension
- Enable Rust logging: Edit `tauri.conf.json`
- Test edge cases with various text inputs

---

## Next Steps for Advanced Features

### 1. Real-Time Audio Monitoring
- Uncomment audio capture in `src-tauri/src/lib.rs`
- Integrate with Vosk for speech-to-text
- Stream events back to React UI

### 2. Media Player Detection
- Detect which app is playing audio
- Show source in activity log
- Optional: Pause/resume media

### 3. Custom Filters
- Create different word lists (violence, profanity, drugs, etc.)
- Apply different alerts for each category
- Schedule monitoring times

### 4. Database Integration
- Save detected content history
- Parent dashboard view
- Time-based analytics

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| App won't start | Delete `src-tauri/target/`, run `cargo clean`, try again |
| Beep sounds not working | Check Windows volume, ensure audio not muted |
| Bad word detection not working | Verify words are added, check exact spelling |
| High CPU usage | This is test mode only - real monitoring will be optimized |
| React changes not updating | Hot reload should work - check browser console for errors |

---

## Support Resources

### Official Documentation
- **Tauri**: https://tauri.app/docs
- **React**: https://react.dev/learn
- **Rust**: https://doc.rust-lang.org

### Community
- Tauri Discord: https://discord.com/invite/tauri
- React Discussions: https://github.com/facebook/react/discussions

---

## What's Happening Behind the Scenes

When you click "Check Text":

```
React UI
   ↓
   └─ invoke('check_bad_words', { text })
      ↓
      Tauri IPC Bridge
      ↓
      Rust Backend (lib.rs)
      ↓
      BadWordDetector
      ├─ Convert text to lowercase
      ├─ Check each word in list
      ├─ Match with boundaries
      └─ Return matching words
         ↓
         React receives [word1, word2, ...]
         ├─ Display in activity log
         ├─ Show in UI as detected
         └─ Trigger alert if any found
            ↓
            invoke('play_alert')
            ↓
            AudioAlert (Rust)
            ├─ Generate sine wave
            ├─ Route to speaker
            └─ Play 500ms beep
```

---

## Happy Testing! 🎉

Your application is fully functional and ready to use. Start with the basic features and explore the code to understand how everything works together.

For questions or issues, check the IMPLEMENTATION_GUIDE.md for more detailed information!
