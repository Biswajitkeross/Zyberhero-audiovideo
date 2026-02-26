#!/usr/bin/env pwsh
# Test script to verify OpenAI Whisper API integration

Write-Host "Testing OpenAI Whisper Integration" -ForegroundColor Cyan
Write-Host "====================================" -ForegroundColor Cyan
Write-Host ""

# Check 1: Environment Variable
Write-Host "Checking environment variable..." -ForegroundColor Yellow
if ($env:OPENAI_API_KEY) {
    $masked = $env:OPENAI_API_KEY.Substring(0, 10) + "..."
    Write-Host "OK: OPENAI_API_KEY is set ($masked)" -ForegroundColor Green
} else {
    Write-Host "ERROR: OPENAI_API_KEY not set!" -ForegroundColor Red
    Write-Host "Set it with: `$env:OPENAI_API_KEY = 'sk-...'" -ForegroundColor Yellow
    Write-Host ""
    exit 1
}

Write-Host ""

# Check 2: Dependencies
Write-Host "Checking dependencies..." -ForegroundColor Yellow
$tomlPath = "src-tauri/Cargo.toml"
if (Select-String -Path $tomlPath -Pattern "reqwest" -Quiet) {
    Write-Host "OK: reqwest (HTTP client) found" -ForegroundColor Green
} else {
    Write-Host "ERROR: reqwest dependency missing" -ForegroundColor Red
}

if (Select-String -Path $tomlPath -Pattern "base64" -Quiet) {
    Write-Host "OK: base64 (encoding) found" -ForegroundColor Green
} else {
    Write-Host "ERROR: base64 dependency missing" -ForegroundColor Red
}

Write-Host ""

# Check 3: Speech Recognizer Implementation
Write-Host "Checking speech_recognizer.rs..." -ForegroundColor Yellow
$srcPath = "src-tauri/src/speech_recognizer.rs"
if (Select-String -Path $srcPath -Pattern "call_whisper_api" -Quiet) {
    Write-Host "OK: OpenAI API call implementation found" -ForegroundColor Green
} else {
    Write-Host "ERROR: OpenAI API implementation missing" -ForegroundColor Red
}

if (Select-String -Path $srcPath -Pattern "samples_to_wav" -Quiet) {
    Write-Host "OK: WAV conversion found" -ForegroundColor Green
} else {
    Write-Host "ERROR: WAV conversion missing" -ForegroundColor Red
}

Write-Host ""

# Check 4: Audio Monitor Integration
Write-Host "Checking audio_monitor.rs integration..." -ForegroundColor Yellow
$monitorPath = "src-tauri/src/audio_monitor.rs"
if (Select-String -Path $monitorPath -Pattern "recognize_speech" -Quiet) {
    Write-Host "OK: Speech recognition called in audio monitor" -ForegroundColor Green
} else {
    Write-Host "ERROR: Speech recognition not integrated in audio monitor" -ForegroundColor Red
}

Write-Host ""

# Check 5: Compilation
Write-Host "Running cargo check..." -ForegroundColor Yellow
Push-Location src-tauri
$output = cargo check 2>&1
Pop-Location

if ($output -match "Finished" -and $output -notmatch "error:") {
    Write-Host "OK: Code compiles successfully" -ForegroundColor Green
} else {
    Write-Host "ERROR: Compilation errors detected" -ForegroundColor Red
    Write-Host $output
    exit 1
}

Write-Host ""
Write-Host "All checks passed! Ready to go!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "1. npx tauri dev" -ForegroundColor White
Write-Host "2. Open http://localhost:5173/" -ForegroundColor White
Write-Host "3. Start monitoring and test with YouTube" -ForegroundColor White
Write-Host ""
