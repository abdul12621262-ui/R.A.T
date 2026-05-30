# Source before building:  . .\scripts\env-rust.ps1
$env:CARGO_HOME = "D:\rust\cargo"
$env:RUSTUP_HOME = "D:\rust\rustup"
$env:TEMP = "D:\temp"
$env:TMP = "D:\temp"
$env:Path = "$env:CARGO_HOME\bin;" + $env:Path
Write-Host "CARGO_HOME=$env:CARGO_HOME"
Write-Host "RUSTUP_HOME=$env:RUSTUP_HOME"
Write-Host "TEMP=$env:TEMP"
Write-Host "rustc: $(rustc --version 2>&1)"
Write-Host "cargo: $(cargo --version 2>&1)"
