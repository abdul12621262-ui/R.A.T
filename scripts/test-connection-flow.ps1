# Test script: verify admin/joiner connection flow
param([string]$SignalingHost = "127.0.0.1:4899")
$ErrorActionPreference = "Stop"

# Check signaling is alive
try {
    $health = Invoke-WebRequest -Uri "http://$SignalingHost/health" -UseBasicParsing -TimeoutSec 5
} catch {
    Write-Host "FAIL: Signaling server not responding at $SignalingHost"
    exit 1
}

if ($health.Content -ne "ok") {
    Write-Host "FAIL: Health check returned: $($health.Content)"
    exit 1
}

Write-Host "PASS: Signaling server /health is ok"
Write-Host "Ready for interactive test with rat.exe"
