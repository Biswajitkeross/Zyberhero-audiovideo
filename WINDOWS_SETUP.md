# 🪟 Windows OpenAI API Setup Guide

## Method 1: PowerShell (EASIEST - 30 seconds)

### Step 1: Open PowerShell
1. Press `Windows Key + R`
2. Type: `powershell`
3. Press Enter

### Step 2: Set the API Key (ONE command)
Copy and paste this into PowerShell:

```powershell
$env:OPENAI_API_KEY = "sk-your-api-key-here"
```

**Replace** `sk-your-api-key-here` with your actual OpenAI API key.

**Example:**
```powershell
$env:OPENAI_API_KEY = "sk-proj-abcdef123456xyz789..."
```

### Step 3: Verify It's Set
```powershell
echo $env:OPENAI_API_KEY
```

You should see your key displayed.

### Step 4: Start the App (same PowerShell window)
```powershell
cd "C:\Users\USR-LPTP-81\Desktop\zybertest-desktop"
npx tauri dev
```

✅ **Done!** The app will now use OpenAI Whisper API.

---

## Method 2: Permanent Setup (if you want it to persist)

### Option A: Add to PowerShell Profile

1. **Open PowerShell as Administrator**
   - Press `Windows Key`
   - Type: `powershell`
   - Right-click → "Run as administrator"

2. **Open your profile:**
   ```powershell
   notepad $PROFILE
   ```

3. **Add this line to the file:**
   ```powershell
   $env:OPENAI_API_KEY = "sk-your-api-key-here"
   ```

4. **Save the file** (Ctrl+S, then close)

5. **Restart PowerShell** and it will auto-set your API key every time!

### Option B: Windows Environment Variables (More Permanent)

1. **Open Environment Variables**
   - Press `Windows Key + R`
   - Type: `sysdm.cpl`
   - Press Enter

2. **Click "Environment Variables..." button**
   - At bottom right of window

3. **Click "New..." under "User variables"**
   - Variable name: `OPENAI_API_KEY`
   - Variable value: `sk-your-api-key-here`

4. **Click OK** (several times to close windows)

5. **Restart PowerShell/CMD**
   - API key will now persist permanently!

---

## ✅ Verification Checklist

### Check 1: Environment Variable is Set
```powershell
echo $env:OPENAI_API_KEY
```
Expected: `sk-...` (your API key)

### Check 2: App Recognizes It
When you start the app:
```powershell
npx tauri dev
```

Look in the terminal for:
```
✅ OpenAI Whisper API enabled
```

### Check 3: Test Manual Detection
1. Open http://localhost:5173/
2. In "Manual Detection" section
3. Type: `fuck this shit`
4. Click "Test Detection"
5. Should show: **"BAD WORDS DETECTED: fuck, shit"** ✅

### Check 4: Test YouTube Audio
1. Click "Start Monitoring All Audio"
2. Open YouTube tab
3. Play a song with profanity
4. App should **BEEP** ✅

---

## 🆘 Troubleshooting

### Problem: "OPENAI_API_KEY not set. Speech recognition disabled."

**Solution:** You forgot step 2!

Run this in PowerShell:
```powershell
$env:OPENAI_API_KEY = "sk-your-api-key-here"
```

Then restart the app:
```powershell
npx tauri dev
```

---

### Problem: "Invalid API Key"

**Possible causes:**
1. Key is wrong (typo)
2. Key has expired
3. Key doesn't start with `sk-`

**Fix:**
1. Go to: https://platform.openai.com/api-keys
2. Create a NEW key (click "Create new secret key")
3. Copy the full key
4. Set it in PowerShell:
   ```powershell
   $env:OPENAI_API_KEY = "sk-new-key-here"
   ```

---

### Problem: API Key works locally but I want it permanent

**Use Method 2 (above):**
- **Quick:** Add to PowerShell profile
- **Permanent:** Add to Windows Environment Variables

---

## 📋 Complete Step-by-Step Example

### Here's exactly what to type:

```powershell
# 1. Open PowerShell and run:
$env:OPENAI_API_KEY = "sk-proj-abc123def456xyz789..."

# 2. Verify it's set:
echo $env:OPENAI_API_KEY
# Output: sk-proj-abc123def456xyz789...

# 3. Go to project directory:
cd "C:\Users\USR-LPTP-81\Desktop\zybertest-desktop"

# 4. Start the app:
npx tauri dev

# 5. Open browser:
# http://localhost:5173/

# 6. Test it!
```

---

## 🎯 Quick Reference

| Task | Command |
|------|---------|
| Set API key | `$env:OPENAI_API_KEY = "sk-..."` |
| Check if set | `echo $env:OPENAI_API_KEY` |
| Start app | `npx tauri dev` |
| Open app | http://localhost:5173/ |
| Check status | Look for "✅ OpenAI Whisper API enabled" in terminal |

---

## 💡 Tips

✅ **Keep your API key private!** Don't share it with anyone.

✅ **API key works with BOTH commands:**
```powershell
# Either of these work
echo $env:OPENAI_API_KEY
$env:OPENAI_API_KEY
```

✅ **If you're in CMD (not PowerShell), use:**
```cmd
set OPENAI_API_KEY=sk-your-key-here
```

✅ **Multiple projects?** Set the API key once in Windows Environment Variables (Method 2, Option B) and all projects will use it!

---

## 🚀 You're Ready!

**Just 2 things to do:**

1. Set the environment variable (30 seconds)
2. Start the app (instant)

**That's it!** The rest is automatic. 🎉

---

## 📞 Still Stuck?

- **API key not working?** Check https://platform.openai.com/account/usage - make sure account has credits
- **Port 5173 already in use?** App will auto-try 5174, 5175, etc. Check browser console
- **No audio detection?** Check VB-Cable routing in Windows Sound settings
- **Beep not working?** Check system volume is NOT muted

Everything should work now! Good luck! 🎵
