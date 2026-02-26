# Clear Corrupted Audio State
# Run this script as Administrator to clear corrupted WASAPI state

Write-Host "🔄 Stopping Windows Audio Service..." -ForegroundColor Yellow
Stop-Service -Name "Audiosrv" -Force

Write-Host "⏳ Waiting 3 seconds..." -ForegroundColor Yellow
Start-Sleep -Seconds 3

Write-Host "▶️ Starting Windows Audio Service..." -ForegroundColor Green
Start-Service -Name "Audiosrv"

Write-Host "✅ Audio service restarted successfully!" -ForegroundColor Green
Write-Host "💡 Now restart 'npx tauri dev' and it should work." -ForegroundColor Cyan
