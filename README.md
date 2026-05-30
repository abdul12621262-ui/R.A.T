# R.A.T — Native Remote Desktop

Installable AnyDesk-style remote desktop: **admin generates a session code**, **joiner** links with code + signaling host. **No Electron** — Rust core + Slint UI.

## Prerequisites

- [Rust](https://rustup.rs/) (toolchain on **D:** recommended if C: is low on space)
- Visual Studio **C++ build tools** (for Slint)
- Windows 10/11 for the current MVP

### Environment (important if C: is full)

```powershell
. .\scripts\env-rust.ps1
```

Or set permanently:

- `CARGO_HOME=D:\rust\cargo`
- `RUSTUP_HOME=D:\rust\rustup`

## Build

```powershell
cd d:\R.A.T
.\build.ps1              # debug
.\build.ps1 -Release      # release
```

Binaries: `target\debug\rat.exe`, `rat-signaling.exe`, `rat-agent.exe`, `rat-relay.exe`

## Quick smoke test

```powershell
.\scripts\smoke-test.ps1
```

## Two-machine LAN test

### Machine A (admin + signaling)

1. Allow firewall: **TCP 4899** inbound (signaling).
2. Run signaling: `target\debug\rat-signaling.exe`
3. Run UI: `target\debug\rat.exe`
4. Click **Create Connection Code** — share `###-###` and your LAN IP (e.g. `192.168.1.10:4899`).

### Machine B (joiner / controlled PC)

1. Run UI: `target\debug\rat.exe`
2. Signaling host: `192.168.1.10:4899` (admin's IP)
3. Session code: the code from admin
4. Click **Link Device** → **Accept** on consent screen
5. Admin should see the remote screen; mouse/keyboard control the joiner PC
6. Joiner: **Stop Sharing** to disconnect immediately

## Architecture

| Component | Role |
|-----------|------|
| `rat-signaling` | WebSocket rooms, codes, consent relay (port **4899**) |
| `rat` (UI) | Slint native app — admin or joiner |
| `rat-agent` | Session engine (capture, input, crypto) |
| `rat-relay` | UDP media relay (port **4900**, optional) |
| `legacy/` | Original Node.js + browser prototype |

## Install (per-user)

```powershell
.\installers\windows\install.ps1 -AddFirewallRule
```

Uninstall: `.\installers\windows\uninstall.ps1`

## Legal

For **authorized support** and devices you own only. Joiner must **consent** before control. Do not use for unauthorized access.
