#!/usr/bin/env bash
# Debian package builder
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
cargo build --release -p rat-ui -p rat-signaling -p rat-agent
PKG=rat-remote-desktop_0.1.0_amd64
rm -rf "target/$PKG"
mkdir -p "target/$PKG/DEBIAN" "target/$PKG/usr/bin"
cp target/release/{rat,rat-signaling,rat-agent} "target/$PKG/usr/bin/"
cat > "target/$PKG/DEBIAN/control" <<EOF
Package: rat-remote-desktop
Version: 0.1.0
Architecture: amd64
Maintainer: RAT
Description: Native remote desktop (no Electron)
Depends: libxcb1, libglib2.0-0
EOF
dpkg-deb --build "target/$PKG"
echo "Built target/${PKG}.deb"
