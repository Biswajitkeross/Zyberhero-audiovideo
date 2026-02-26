# 🔧 FIXING API KEY NOT DETECTED - SOLUTION GUIDE

## ⚠️ The Problem

You're seeing:
```
⚠️  OPENAI_API_KEY not set. Speech recognition disabled.
   Set it with: $env:OPENAI_API_KEY = 'sk-...'
```

This means the app isn't reading your API key even though you set it in PowerShell.

---

## ✅ THE SOLUTION (Try These 3 Methods)

### **METHOD 1: Restart PowerShell (QUICKEST)**

1. **Close ALL PowerShell windows**
   - Close every PowerShell window completely
   
2. **Open a NEW PowerShell window**
   - Right-click on desktop → "Open PowerShell window here"
   - OR Press: `Win + X` → Select `Terminal` or `PowerShell`

3. **Set API key in the NEW window:**
   ```powershell
   $env:OPENAI_API_KEY = "sk-proj-your-actual-key-here"
   ```

4. **Verify it's set:**
   ```powershell
   echo $env:OPENAI_API_KEY
   ```
   Should show: `sk-proj-...` (not blank!)

5. **In THE SAME PowerShell window, run:**
   ```powershell
   cd "C:\Users\USR-LPTP-81\Desktop\zybertest-desktop"
   npx tauri dev
   ```

✅ **This should now show:** `✅ OpenAI Whisper API enabled`

---

### **METHOD 2: Set as System Environment Variable (PERMANENT)**

This makes it work even after restart!

#### **Step 1: Open Environment Variables**
1. Press: `Win + X` → Select `System`
2. Click: **"Advanced system settings"** (on the left)
3. Click: **"Environment Variables"** button

#### **Step 2: Create New Variable**
1. Under **"User variables"**, click: **"New..."**
2. Variable name: `OPENAI_API_KEY`
3. Variable value: `sk-proj-your-actual-key-here`
4. Click: **OK**
5. Click: **OK** again

#### **Step 3: Restart Everything**
1. Close ALL PowerShell windows
2. Restart your computer (or at least close terminal)
3. Open NEW PowerShell
4. Run: `npx tauri dev`

✅ **This will work permanently!**

---

### **METHOD 3: Set API Key Inline (ONE-LINE SOLUTION)**

In PowerShell, run this single command:

```powershell
$env:OPENAI_API_KEY = "sk-proj-your-key-here"; npx tauri dev
```

This sets the variable AND runs the app in one command!

---

## 🎯 STEP-BY-STEP SOLUTION (RECOMMENDED)

### **Quick Fix - Try This Now:**

```powershell
# 1. Close all PowerShell windows completely
# (Use Alt+F4 or the X button)

# 2. Open NEW PowerShell window
# (Right-click desktop or Win+X → Terminal)

# 3. Set API key
$env:OPENAI_API_KEY = "sk-proj-YOUR-ACTUAL-KEY-HERE"

# 4. Verify
echo $env:OPENAI_API_KEY

# 5. Navigate to project
cd "C:\Users\USR-LPTP-81\Desktop\zybertest-desktop"

# 6. Start app
npx tauri dev
```

### **Expected Output:**

If the fix worked, you'll see:
```
✅ OpenAI Whisper API enabled
🎤 Initializing AudioMonitor with Whisper speech recognition...
```

If still not working, you'll see:
```
⚠️  OPENAI_API_KEY not set...
```

---

## 🔍 VERIFY YOUR API KEY IS CORRECT

### **Check if API Key is Valid**

1. Visit: https://platform.openai.com/api-keys
2. Login with your OpenAI account
3. Check your API key starts with: `sk-proj-`
4. Make sure it's not expired
5. Copy the FULL key (sometimes it gets cut off)

### **Check if API Key is Set in PowerShell**

```powershell
# Run this command
echo $env:OPENAI_API_KEY
```

Should show: `sk-proj-abc123xyz...`

If it shows **nothing** or is **blank**, the key is not set!

---

## ❌ COMMON MISTAKES TO AVOID

| Mistake | Problem | Solution |
|---------|---------|----------|
| **Key not copied fully** | App says "not set" | Copy entire key from website |
| **Copy-paste error** | Key is wrong | Try again, double-check |
| **Using same PowerShell** | Old session has no key | Close and open NEW window |
| **Key in quotes** | Variable includes quotes | Use: `$env:VAR = "value"` not `$env:VAR = '"value"'` |
| **Typo in variable name** | App can't find it | Must be exactly: `OPENAI_API_KEY` |
| **Running old terminal** | Doesn't see new key | Close all, open new terminal |

---

## 📝 COMPLETE WORKING EXAMPLE

### **Copy-Paste This Exact Sequence:**

```powershell
# Close all PowerShell windows first!
# Then open NEW window and paste:

$env:OPENAI_API_KEY = "sk-proj-your-key-here"
cd "C:\Users\USR-LPTP-81\Desktop\zybertest-desktop"
npx tauri dev
```

Replace `sk-proj-your-key-here` with your actual key!

---

## 🚀 IF IT STILL DOESN'T WORK

### **Try Troubleshooting Steps:**

**Step 1: Verify Key Format**
```powershell
# Check the key starts with sk-proj-
echo $env:OPENAI_API_KEY
# Should show: sk-proj-abc123...
```

**Step 2: Try Method 2 (System Variable)**
Set it as a system environment variable (permanent):
1. Win + X → System
2. Advanced system settings
3. Environment Variables
4. New → Name: `OPENAI_API_KEY` → Value: `sk-proj-...`
5. OK → OK
6. Restart computer

**Step 3: Manual Test**
Check if Rust can read the environment variable:
```powershell
cd "C:\Users\USR-LPTP-81\Desktop\zybertest-desktop\src-tauri"
cargo run 2>&1 | grep -i "openai\|api\|key"
```

---

## ✅ SUCCESS VERIFICATION

Once the API key is detected, you should see:

### **In Terminal:**
```
✅ OpenAI Whisper API enabled
🎤 Initializing AudioMonitor with Whisper speech recognition...
```

### **In Browser (http://localhost:5173/):**
Dashboard loads and "Start Monitoring All Audio" button appears

### **When Testing:**
```
🎵 Strong audio detected
📝 Whisper: [words from YouTube]
🚨 BAD WORDS DETECTED: [words]
BEEP! 🔊
```

If you see all of this, **congratulations! API key is working!** 🎉

---

## 📊 QUICK REFERENCE - WHICH METHOD TO USE

| Situation | Use This |
|-----------|----------|
| **Just want to test now** | METHOD 1 (Restart PowerShell) |
| **Want it to work forever** | METHOD 2 (System Environment Variable) |
| **Want one-liner command** | METHOD 3 (Inline) |
| **Nothing works** | Try all 3 in order |

---

## 🎯 RECOMMENDED APPROACH

**Best Solution (Permanent & Reliable):**

1. **METHOD 2: Set as System Environment Variable**
   - Works permanently
   - Works across all applications
   - Survives computer restarts
   - Takes 2 minutes to set up

2. **Then immediately test:**
   ```powershell
   cd "C:\Users\USR-LPTP-81\Desktop\zybertest-desktop"
   npx tauri dev
   ```

---

## 💡 PRO TIPS

### **Tip 1: Keep Terminal Open**
Once you set the API key in a PowerShell window, keep that window OPEN while running `npx tauri dev`. Don't close it!

### **Tip 2: Verify Before Running App**
Always check API key is set BEFORE starting app:
```powershell
echo $env:OPENAI_API_KEY  # Should show your key
npx tauri dev              # Then start app
```

### **Tip 3: Use Same Terminal Session**
Set the key and run the app in the SAME terminal window without closing it!

---

## 🔑 YOUR API KEY

**Where to find it:**
1. Visit: https://platform.openai.com/api-keys
2. Login with OpenAI account
3. Copy your key (starts with: `sk-proj-`)
4. Use in command: `$env:OPENAI_API_KEY = "sk-proj-..."`

**Format check:**
- ✅ Should start with: `sk-proj-`
- ✅ Should be ~48 characters long
- ✅ Should be alphanumeric + dashes

---

## 🎉 NEXT STEPS AFTER FIX

Once you see: **"✅ OpenAI Whisper API enabled"**

1. Keep terminal open
2. Open browser: `http://localhost:5173/`
3. Click "Start Monitoring All Audio"
4. Play YouTube with bad words
5. **Listen for BEEP! 🔊🔊**

---

## ❓ STILL HAVING ISSUES?

Check these in order:

1. **Is API key copied correctly?**
   - Visit: https://platform.openai.com/api-keys
   - Copy full key (not cut off)
   - Check it starts with: `sk-proj-`

2. **Is PowerShell new?**
   - Close all PowerShell windows
   - Open fresh window
   - Set key in new window

3. **Did you use exact variable name?**
   - Must be: `OPENAI_API_KEY` (exactly this)
   - Not: `API_KEY` or `OPENAI_KEY` or anything else

4. **Is key set before app starts?**
   - Set key FIRST
   - THEN run: `npx tauri dev`
   - Don't close the window

5. **Still stuck?**
   - Try METHOD 2 (System environment variable)
   - This is the most reliable method

---

**Created:** February 4, 2026
**Status:** Solution guide for API key not detected
**Recommended:** Try METHOD 1 first, then METHOD 2 if needed

🔑 Let's get your API key working! 🚀
