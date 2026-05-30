# Optional auto-update check (point RELEASE_URL at your GitHub releases API)
param([string]$ReleaseUrl = "")
if (-not $ReleaseUrl) {
    Write-Host "Set RELEASE_URL to enable update checks."
    exit 0
}
$latest = Invoke-RestMethod -Uri $ReleaseUrl
Write-Host "Latest release: $($latest.tag_name)"
