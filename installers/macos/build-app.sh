#!/usr/bin/env bash
# macOS .app bundle builder
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
cargo build --release -p rat-ui -p rat-signaling -p rat-agent
APP="$ROOT/target/RAT.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/rat "$APP/Contents/MacOS/"
cp target/release/rat-signaling "$APP/Contents/MacOS/"
cp target/release/rat-agent "$APP/Contents/MacOS/"
cat > "$APP/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>RAT</string>
  <key>CFBundleExecutable</key><string>rat</string>
  <key>CFBundleIdentifier</key><string>com.rat.desktop</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSScreenCaptureDescription</key><string>RAT needs screen recording to share your display with a trusted admin.</string>
</dict></plist>
PLIST
echo "Built $APP — grant Screen Recording + Accessibility in System Settings."
