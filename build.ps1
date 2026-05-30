param([switch]$Release)
$env:CARGO_HOME = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { "D:\rust\cargo" }
$env:RUSTUP_HOME = if ($env:RUSTUP_HOME) { $env:RUSTUP_HOME } else { "D:\rust\rustup" }
$env:TEMP = if ($env:TEMP -and $env:TEMP.StartsWith("D:")) { $env:TEMP } else { "D:\temp" }
$env:TMP = if ($env:TMP -and $env:TMP.StartsWith("D:")) { $env:TMP } else { "D:\temp" }
$env:Path = "$env:CARGO_HOME\bin;" + $env:Path
$cargo = "$env:CARGO_HOME\bin\cargo.exe"
if (-not (Test-Path $cargo)) { $cargo = "cargo" }
$args = @("build")
if ($Release) { $args += "--release" }
& $cargo @args -p rat-ui -p rat-signaling -p rat-agent -p rat-relay
exit $LASTEXITCODE
