# 🔧 How to Find and Enable Stereo Mix - VISUAL GUIDE

## 🎯 What is Stereo Mix?

**Stereo Mix** (also called "Loopback" or "What U Hear") allows the app to capture audio playing ON YOUR COMPUTER (YouTube, Discord, Media Players, etc.).

**Why you need it:**
- Without it: App can't hear YouTube videos
- With it: App captures system audio and detects bad words ✅

---

## 📍 EXACT STEPS - Windows 11

### STEP 1: Open Volume Settings

**Method A: Via Volume Icon (Easiest)**
```
1. Look at the bottom right corner of your screen
   (where the clock is)

2. Find the VOLUME ICON:
   🔊 or 🔇 (looks like a speaker)

3. RIGHT-CLICK on it (don't left-click!)

4. You'll see a menu - click:
   "Open Sound settings"
```

**Method B: Via Settings App**
```
1. Press: Windows key + I (opens Settings)
2. Go to: Sound
3. Scroll down to: Advanced
4. Click: "Volume mixer" or "Recording devices"
```

---

### STEP 2: Find Recording Devices

After clicking "Open Sound settings", you should see:

```
┌─────────────────────────────────┐
│ Settings > Sound                │
├─────────────────────────────────┤
│                                 │
│ Output (speakers, headphones)   │
│ - Your speaker/headphone        │
│                                 │
│ Input (microphones)             │
│ - Microphone                    │
│                                 │
│ Advanced                        │
│ - Input devices                 │
│ - Output devices                │
│ - Volume mixer                  │
│                                 │
└─────────────────────────────────┘
```

**What to do:**
1. Scroll DOWN to find **"Advanced"** section
2. Click on **"Volume mixer"** OR **"Recording devices"**

---

### STEP 3: Find Stereo Mix in Recording Devices

**If you clicked "Volume mixer":**
```
1. You'll see the app window
2. Look for: "App volume and device preferences"
3. Scroll down to find your app (zybertest-desktop)
4. On the right side, change "Input" to "Stereo Mix"
5. Done! ✅
```

**If you clicked "Recording devices":**
```
You should see a window like this:

┌──────────────────────────────────┐
│ Recording Devices                │
├──────────────────────────────────┤
│ ✓ Microphone (default)           │ ← Already enabled
│ ○ Microphone Array (disabled)    │
│ ○ Stereo Mix (disabled)          │ ← FIND THIS ONE
│ ○ Loopback                       │
│ ○ Line In                        │
│                                  │
│ [Apply]  [OK]  [Cancel]          │
└──────────────────────────────────┘
```

**Look for one of these names:**
- ✅ "Stereo Mix"
- ✅ "Loopback"
- ✅ "What U Hear"
- ✅ "Microphone Stereo Mix"
- ✅ "Wave Out Mix"

---

## ⚙️ STEP 4: Enable Stereo Mix (If Disabled)

### If You See It DISABLED (Greyed Out):

```
1. RIGHT-CLICK on "Stereo Mix"

2. You'll see options:
   ├─ Enable
   ├─ Disable
   ├─ Set as Default Device
   ├─ Properties
   └─ More

3. Click: "Enable"

4. Wait 2 seconds

5. Stereo Mix should now show:
   ✓ Stereo Mix (enabled, green icon)
```

### If You See It ENABLED Already:

```
Great! ✅ 
Stereo Mix is already on.
You can skip to Step 5.
```

### If You DON'T See Stereo Mix At All:

This means your audio driver doesn't support it.

**Options:**
```
1. Update your audio driver:
   - Go to Dell/HP/Lenovo support page (depends on your laptop)
   - Download latest audio driver
   - Install and restart

2. Or use Virtual Audio Cable:
   - Download: VB-Audio Virtual Cable
   - Install it
   - This creates a virtual loopback device
   - Set it as input for your app

3. Or use OBS (free):
   - Download OBS Studio
   - Create virtual camera/audio
   - Route system audio through it
```

---

## 🎯 STEP 5: Set Stereo Mix as DEFAULT

**Important:** For the app to use it automatically:

```
1. In Recording Devices window:

2. RIGHT-CLICK on "Stereo Mix"

3. Click: "Set as Default Device"

4. You should see a green checkmark next to it:
   ✓ Stereo Mix (default device)
```

---

## ✅ STEP 6: Verify It's Working

```
1. Close the Settings window

2. Open the app: http://localhost:5173/

3. Go to: 📡 System Audio Monitoring

4. Click: "▶ Start Monitoring All Audio"

5. Wait 2 seconds

6. Check: Is the red indicator showing "🔴 MONITORING ACTIVE"?
   - YES → Stereo Mix is working! ✅
   - NO → Check troubleshooting below
```

---

## 📸 VISUAL SCREENSHOTS (Text Descriptions)

### What You're Looking For:

**BEFORE (Disabled):**
```
❌ Stereo Mix ━━━━━━━━━━━━━ GREY/FADED TEXT
(Might say "disabled" underneath)
```

**AFTER (Enabled):**
```
✓ Stereo Mix ━━━━━━━━━━━━━━ BLACK/CLEAR TEXT
   (Ready to use)
```

---

## 🆘 TROUBLESHOOTING

### Problem 1: I Don't See Stereo Mix Anywhere

**This is VERY COMMON** on laptops. Windows might be hiding it.

**Solution:**
```
1. In Recording Devices window, right-click empty space

2. Check BOTH boxes:
   ☑ Show Disabled Devices
   ☑ Show Disconnected Devices

3. Now you should see Stereo Mix!

4. If it's greyed out: Right-click > Enable
```

### Problem 2: Stereo Mix is Disabled and Won't Enable

**Reason:** Your audio driver doesn't support it.

**Solutions:**
```
Option A: Update Audio Driver
- Go to: Dell/HP/Lenovo support page
- Search for: "Audio Driver" for your model
- Download latest version
- Install and restart
- Try enabling Stereo Mix again

Option B: Use Virtual Audio Cable
- Download: VB-Cable Virtual Audio Device
- Install it (might need restart)
- Use it as input device instead

Option C: Use Different Audio App
- Try: Audacity (free, has loopback)
- Or: VoiceMeeter (free audio router)
```

### Problem 3: I Enabled It But Monitoring Still Won't Start

```
1. Make sure Stereo Mix is the DEFAULT:
   - Right-click it
   - Click "Set as Default Device"

2. Restart the app:
   - Close browser tab
   - Close dev server (Ctrl+C)
   - Run: npm run dev
   - Open http://localhost:5173/ again

3. Try monitoring again
```

### Problem 4: I See Stereo Mix But My App Can't Use It

```
Reason: App might need permission to access audio

Solution:
1. Go to: Settings > Privacy > Microphone
2. Make sure your app has permission
3. Or try: Settings > Apps > Permissions > Recording
4. Check that the app is allowed to record
```

---

## 🎬 QUICK REFERENCE

| Step | Action | What You'll See |
|------|--------|-----------------|
| 1 | Right-click Volume icon | Menu appears |
| 2 | Click "Sound settings" | Settings window opens |
| 3 | Find "Recording devices" | List of audio devices |
| 4 | Find "Stereo Mix" | One item in the list |
| 5 | Right-click > Enable | No longer greyed out |
| 6 | Right-click > Set Default | Checkmark appears |
| 7 | Test in app | "🔴 MONITORING ACTIVE" |

---

## 🎯 EXACTLY WHERE TO CLICK

### For Windows 11:

```
1. Bottom right corner of screen
   ↓
2. Right-click Volume icon (🔊)
   ↓
3. Click "Open Sound settings"
   ↓
4. Settings app opens
   ↓
5. Scroll to "Advanced" section
   ↓
6. Click "Recording devices" (or "Volume mixer")
   ↓
7. Find "Stereo Mix" in the list
   ↓
8. Right-click it
   ↓
9. Click "Enable" (if greyed out)
   ↓
10. Right-click again
   ↓
11. Click "Set as Default Device"
   ↓
12. Close Settings
   ↓
13. Test in app at http://localhost:5173/
```

---

## ✨ SUCCESS INDICATORS

You'll know it's working when:

✅ Stereo Mix shows in Recording devices (not greyed out)
✅ You see a checkmark next to "Stereo Mix"
✅ App shows "🔴 MONITORING ACTIVE" (red indicator)
✅ YouTube/Discord audio plays normally
✅ Double beep triggers when bad word is heard

---

## 🚀 NEXT STEPS

Once Stereo Mix is enabled and working:

1. Go to app: http://localhost:5173/
2. Click "▶ Start Monitoring"
3. Open YouTube
4. Play a video with bad words
5. **YOU SHOULD HEAR THE DOUBLE BEEP!** 🔊

---

## 📞 STILL STUCK?

If you've done all these steps and it still doesn't work:

```
1. Tell me:
   - Can you see "Stereo Mix" in Recording devices?
   - Is it enabled (not greyed out)?
   - Does the app show "🔴 MONITORING ACTIVE"?

2. Tell me your computer:
   - Windows 10 or Windows 11?
   - Laptop brand? (Dell, HP, Lenovo, etc.)
   - Audio device? (Built-in, USB headset, etc.)

3. I can help debug with more specific steps!
```

---

**Now you know where to find Stereo Mix!** 🎧

Go enable it and test your YouTube videos with bad words! 🚀

