# R.A.T Windows installer script (per-user, no admin required)
param(
    [string]$InstallDir = "$env:LOCALAPPDATA\RAT",
    [switch]$AddFirewallRule
)

$ErrorActionPreference = "Stop"
$Root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent

Write-Host "Building R.A.T release..."
Push-Location $Root
$env:CARGO_HOME = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { "D:\rust\cargo" }
$env:RUSTUP_HOME = if ($env:RUSTUP_HOME) { $env:RUSTUP_HOME } else { "D:\rust\rustup" }
$env:Path = "$env:CARGO_HOME\bin;" + $env:Path
& "$env:CARGO_HOME\bin\cargo.exe" build --release -p rat-ui -p rat-signaling -p rat-agent -p rat-relay
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
Pop-Location

$Bin = Join-Path $Root "target\release"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
Copy-Item (Join-Path $Bin "rat.exe") (Join-Path $InstallDir "rat.exe") -Force
Copy-Item (Join-Path $Bin "rat-signaling.exe") (Join-Path $InstallDir "rat-signaling.exe") -Force
Copy-Item (Join-Path $Bin "rat-agent.exe") (Join-Path $InstallDir "rat-agent.exe") -Force

# Start menu shortcut
$Wsh = New-Object -ComObject WScript.Shell
$Shortcut = $Wsh.CreateShortcut("$env:APPDATA\Microsoft\Windows\Start Menu\Programs\RAT Remote Desktop.lnk")
$Shortcut.TargetPath = Join-Path $InstallDir "rat.exe"
$Shortcut.WorkingDirectory = $InstallDir
$Shortcut.Save()

# Autostart signaling + tray app at login
$RunKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
New-ItemProperty -Path $RunKey -Name "RATSignaling" -Value "`"$InstallDir\rat-signaling.exe`"" -PropertyType String -Force | Out-Null
New-ItemProperty -Path $RunKey -Name "RATDesktop" -Value "`"$InstallDir\rat.exe`"" -PropertyType String -Force | Out-Null

if ($AddFirewallRule) {
    netsh advfirewall firewall add rule name="RAT Signaling" dir=in action=allow protocol=TCP localport=4899 | Out-Null
    netsh advfirewall firewall add rule name="RAT Relay" dir=in action=allow protocol=UDP localport=4900 | Out-Null
}

Write-Host "Installed to $InstallDir"
Write-Host "Run 'rat.exe' from Start Menu."
