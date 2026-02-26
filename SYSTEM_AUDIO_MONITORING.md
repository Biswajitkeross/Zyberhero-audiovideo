# System Audio Monitoring - Complete Guide

## Overview

Your application now includes **real-time system audio monitoring** that can detect bad words from any audio source on your computer:

- 🎬 YouTube videos
- 🎵 Music and media players (Spotify, VLC, Windows Media Player, etc.)
- 💬 Discord and video calls
- 🎮 Gaming streams and audio
- 🎙️ Live recordings
- 📺 Any system audio output

When a bad word is detected, the application automatically plays a **double beep alert** sound to notify you.

---

## How It Works

### Architecture

```
System Audio (YouTube, Discord, Media Player, etc.)
           ↓
    WASAPI Loopback (Stereo Mix)
           ↓
    Audio Capture Module (CPAL)
           ↓
    Real-time Audio Processing (Speech Detection, Energy Analysis)
           ↓
    Bad Word Detection Engine
           ↓
    Alert Sound (Double Beep) ← Automatic response
           ↓
    Activity Log (UI)
```

### Components

1. **Audio Capture**: Uses Windows WASAPI loopback recording to capture system audio
2. **Audio Processing**: Analyzes audio frames for speech patterns and energy
3. **Bad Word Detection**: Checks captured audio against your configured bad words list
4. **Alert System**: Plays a double beep sound when violations are detected
5. **Monitoring Service**: Runs continuously in the background as an async service

---

## Getting Started

### Step 1: Enable Stereo Mix on Windows

For system audio monitoring to work, you must enable **Stereo Mix** (also called "What U Hear"):

**Windows 10/11:**

1. Right-click the **Volume icon** in your system tray
2. Select **Open Sound settings**
3. Scroll down to **Advanced** section
4. Click **Volume mixer**
5. Click **App volume and device preferences**
6. Look for your app in the list
7. In the top right, click **Device** dropdown and select your speakers
8. Go back to Sound settings
9. Scroll down to **Input devices**
10. Look for **Stereo Mix** or **WASAPI loopback** device
11. If it's disabled (greyed out), right-click it and select **Enable**
12. Set it as your default recording device (Optional)

**Alternative method:**
1. Right-click the **Volume icon**
2. Select **Open Sound settings**
3. Go to **Recording devices** tab
4. Look for "Stereo Mix"
5. If disabled, right-click → **Enable**

### Step 2: Start Monitoring

1. Open the application UI
2. Go to the **"📡 System Audio Monitoring"** section
3. Click the **"▶ Start Monitoring All Audio"** button
4. The status will change to:
   - 🔴 **MONITORING ACTIVE** (red indicator)
   - With live detection stats (if any)

### Step 3: Test It

#### Test with YouTube:
1. Start monitoring
2. Open YouTube in your browser
3. Play a video
4. Speak or play audio with bad words
5. You should hear a **double beep** automatically

#### Test with Media Player:
1. Start monitoring
2. Open Windows Media Player or any audio app
3. Play content with detected bad words
4. The app will alert with double beep

#### Test with Text:
1. Use the **"🧪 Test Text for Bad Words"** section
2. Type text with bad words
3. Click **"Check Text"** button
4. You'll see:
   - Bad words highlighted in the activity log
   - Single beep alert plays automatically

---

## Understanding the UI

### System Audio Monitoring Section

```
📡 System Audio Monitoring

Monitor YouTube, media players, Discord, and all system audio 
in real-time. Automatically plays a double beep when bad words 
are detected.

[▶ Start Monitoring All Audio] ← Click to activate

🔴 MONITORING ACTIVE              ← Shows live status
Detections: 5                      ← Count of violations
Last Detected: badword, profanity  ← What was detected
Time: 14:32:45                     ← When it happened
```

### Status Indicators

- **🔴 MONITORING ACTIVE** (Red with pulse): System is listening to all audio
- **⚫ Monitoring Inactive** (Grey): Monitoring is not running

### Real-time Stats

- **Detections**: Total number of bad words detected
- **Last Detected**: The actual bad words found
- **Time**: When the detection occurred

---

## Managing Bad Words

### Add Custom Bad Words

1. Go to **"🚫 Bad Word Management"** section
2. Type a word in the **"Enter bad word to add..."** field
3. Click **"Add Word"** or press Enter
4. The word appears in the **"Current Bad Words"** list
5. These are monitored in both text and audio

### Remove Bad Words

1. In the **"Current Bad Words"** list
2. Click the **"×"** button next to a word
3. It's immediately removed from monitoring

### Clear All Bad Words

1. Click **"Clear All"** button
2. Confirms you want to remove all words
3. All monitoring stops for bad words

### Toggle Detection

1. Click **"✓ Detection ON/OFF"** button
2. Turns monitoring on (✓) or off (✗)
3. Good for pausing without stopping the system audio monitor

---

## Activity Log

The **"📋 Activity Log"** shows a real-time history of events:

```
14:48:40 pm - Alert played: double - Alert played
14:48:38 pm - Alert played: single - Alert played
14:47:36 pm - Checked text: "bad word here" - [bad, word]
14:47:34 pm - Added bad word: curse - Success
```

Each log entry shows:
- **Time**: When the event occurred
- **Message**: What happened
- **Bad Words**: Which words were detected (if applicable)

Logs are kept for the last 50 events. Older logs are automatically removed.

---

## How to Test Each Feature

### Test 1: Alert Sounds
**Section**: Alert Test
**Steps**:
1. Click "Single Beep (1000Hz)" → Should hear 1000Hz beep
2. Click "Double Beep" → Should hear 2 beeps (urgent alert)
3. Click "Ascending Alert" → Should hear ascending tone

### Test 2: Manual Text Checking
**Section**: Test Text for Bad Words
**Steps**:
1. Type: "This is a damn test with bad words"
2. Click "Check Text"
3. Should detect: `damn, bad` (if in your list)
4. Single beep plays automatically
5. Activity log shows what was detected

### Test 3: System Audio Monitoring with YouTube
**Steps**:
1. Start Monitoring
2. Open YouTube: youtube.com
3. Play any video
4. Status should show: "🔴 MONITORING ACTIVE"
5. The app listens to everything from the video

**To trigger detection**:
- Find a video with profanity
- Play it
- When bad words are spoken, you'll hear the double beep
- Detection count increases in the UI

### Test 4: System Audio with Media Player
**Steps**:
1. Start monitoring
2. Open Windows Media Player
3. Play a music or audio file with detected words
4. Double beep will play when words are detected

### Test 5: Discord/Video Call
**Steps**:
1. Start monitoring
2. Open Discord or any video conferencing app
3. Join a call
4. When someone speaks bad words, double beep plays
5. Application automatically detects and alerts

---

## Troubleshooting

### Issue: Monitoring won't start

**Solution 1: Enable Stereo Mix**
- Windows Settings → Sound → Recording devices
- Right-click "Stereo Mix" → Enable
- Make it the default recording device

**Solution 2: Check permissions**
- Make sure the app has permission to access microphone
- Windows Settings → Privacy → Microphone
- Enable microphone access for the app

**Solution 3: Check audio device**
- Windows Settings → Sound → Volume mixer
- Ensure your audio output device is configured

### Issue: No audio detected

**Possible causes**:
1. Stereo Mix not enabled
2. Wrong audio output device selected
3. Audio is not playing (volume too low, muted)

**Fix**:
```
1. Enable Stereo Mix (see section above)
2. Make sure audio is actually playing
3. Check volume levels are adequate
4. Check if app is recording from correct device
```

### Issue: False positives (detecting words that aren't bad)

**Solution**:
- The detector is very sensitive
- You can remove sensitive words from the list
- Click "×" next to words in "Current Bad Words"
- Add only words you really want to monitor

### Issue: No beep sound when bad words are detected

**Check**:
1. System volume is not muted
2. Speaker/headphone volume is up
3. Bad words list is not empty
4. Detection is enabled (green toggle button)

**Test**:
1. Click "Double Beep" button
2. If you hear it, monitoring will work
3. If you don't, check audio settings

---

## Performance Tips

1. **Use Selectively**: Only enable monitoring when you need it
2. **Minimize Bad Words List**: Fewer words = better performance
3. **Close Unnecessary Apps**: Reduces system audio noise
4. **Use Headphones**: Better audio quality = better detection

---

## Advanced Features

### How Detection Works

1. **Audio Capture**: System audio is captured in small frames
2. **Speech Detection**: Audio is analyzed for human speech patterns
3. **Energy Analysis**: Checks if audio has sufficient energy (not silence)
4. **Word Matching**: Audio content is checked against bad words list
5. **Alert Trigger**: When match found → double beep plays
6. **Logging**: Event is recorded in activity log

### Customization

You can customize:
- **Bad Words List**: Add/remove words anytime
- **Alert Sound**: Can add custom alert logic
- **Detection Sensitivity**: Modify energy thresholds in code
- **Monitoring Interval**: Adjust how often audio is checked

---

## Keyboard Shortcuts

| Action | Shortcut |
|--------|----------|
| Add Bad Word | Type word + Press Enter |
| Check Text | Type text + Click "Check Text" or Ctrl+Enter |
| Start Monitoring | Click button or use command |
| Test Beep | Click "Double Beep" button |

---

## Command Line Usage (For Developers)

The Tauri commands available are:

```typescript
// Start monitoring all system audio
await invoke('start_monitoring')

// Stop monitoring
await invoke('stop_monitoring')

// Get current monitoring status
await invoke('get_monitoring_status')
// Returns: { is_monitoring, last_detected_word, detection_count, last_detection_time }

// Check text for bad words
await invoke('check_bad_words', { text: "user input" })

// Add a bad word
await invoke('add_bad_word', { word: "newword" })

// Remove a bad word
await invoke('remove_bad_word', { word: "oldword" })

// Get all bad words
await invoke('get_all_bad_words')

// Clear all bad words
await invoke('clear_bad_words')

// Toggle detection
await invoke('set_detection_enabled', { enabled: true/false })
```

---

## Example Use Cases

### Parental Control
Monitor your child's online activities (YouTube, Discord, Gaming) and get instant alerts when inappropriate language is used.

### Workplace Monitoring
In professional environments, monitor streaming content or calls to ensure compliance with company policies.

### Content Moderation
For content creators, monitor live streams and get alerts when bad words are used on air.

### Accessibility
Help non-native speakers learn appropriate language by providing real-time feedback.

---

## Privacy & Security

- **Local Processing**: All audio detection happens on your computer
- **No Cloud**: No data is sent to external servers
- **No Recording**: Audio is not recorded, only analyzed in real-time
- **No Transmission**: Bad words are processed locally only

---

## System Requirements

- Windows 10 or Windows 11
- WASAPI Loopback (Stereo Mix) support
- ~50-100MB RAM for monitoring
- 2+ GHz CPU recommended

---

## Next Steps

1. **Enable Stereo Mix** (Windows Settings)
2. **Start Monitoring** (Click button in app)
3. **Test with YouTube** (Play a video)
4. **Listen for Alerts** (Double beep when words detected)
5. **Check Activity Log** (See what was detected)

Enjoy your audio monitoring! 🎉

