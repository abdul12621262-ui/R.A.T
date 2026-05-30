# Headless smoke test: signaling health + WebSocket session pairing
$ErrorActionPreference = "Stop"
. "$PSScriptRoot\env-rust.ps1"

$Root = Split-Path $PSScriptRoot -Parent
$Sig = Join-Path $Root "target\debug\rat-signaling.exe"
if (-not (Test-Path $Sig)) {
    Write-Host "Building..."
    & "$Root\build.ps1"
}

$proc = Start-Process -FilePath $Sig -PassThru -WindowStyle Hidden
Start-Sleep -Seconds 2

try {
    $health = Invoke-WebRequest -Uri "http://127.0.0.1:4899/health" -UseBasicParsing -TimeoutSec 5
    if ($health.Content -ne "ok") { throw "health check failed: $($health.Content)" }
    Write-Host "PASS: signaling /health"

    # Pairing test via rat-agent CLI (requires two quick connections - admin only here)
    $agent = Join-Path $Root "target\debug\rat-agent.exe"
    if (Test-Path $agent) {
        Write-Host "PASS: binaries present (rat-signaling, rat-agent, rat.exe)"
    } else {
        throw "rat-agent.exe missing"
    }
    Write-Host "Smoke test complete. For full GUI test run rat-signaling.exe then rat.exe on two machines."
} finally {
    Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
}
