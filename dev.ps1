$backend = Start-Process -NoNewWindow -PassThru pwsh -ArgumentList '-NoProfile', '-Command', 'cargo watch -x ''run --bin docx-jats-backend'''
$frontend = Start-Process -NoNewWindow -PassThru pwsh -ArgumentList '-NoProfile', '-Command', 'cd frontend; npm run dev'

Write-Host "Backend PID $($backend.Id) | Frontend PID $($frontend.Id)"
Write-Host "Press Ctrl+C to stop both."

try { Wait-Process -Id $backend.Id, $frontend.Id }
finally {
    Stop-Process -Id $backend.Id, $frontend.Id -ErrorAction SilentlyContinue
}
