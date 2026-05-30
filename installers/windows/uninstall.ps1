param([string]$InstallDir = "$env:LOCALAPPDATA\RAT")

$RunKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Run"
Remove-ItemProperty -Path $RunKey -Name "RATSignaling" -ErrorAction SilentlyContinue
Remove-ItemProperty -Path $RunKey -Name "RATDesktop" -ErrorAction SilentlyContinue
Remove-Item "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\RAT Remote Desktop.lnk" -ErrorAction SilentlyContinue
Remove-Item -Recurse -Force $InstallDir -ErrorAction SilentlyContinue
netsh advfirewall firewall delete rule name="RAT Signaling" | Out-Null
netsh advfirewall firewall delete rule name="RAT Relay" | Out-Null
Write-Host "R.A.T uninstalled."
