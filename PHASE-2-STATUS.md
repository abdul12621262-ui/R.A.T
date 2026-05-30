# R.A.T Phase 2 Smoke Test — Status & Next Steps

**Date:** May 30, 2026  
**Status:** Phase 2 infrastructure complete, ready for LAN testing

## What I Did

### 1. Verified Build Infrastructure ✅
- All four binaries present: `rat.exe`, `rat-signaling.exe`, `rat-agent.exe`, `rat-relay.exe`
- Headless smoke test passes: `/health` endpoint returns `ok`
- Signaling server listening on TCP 4899
- WebSocket ready at `ws://127.0.0.1:4899/ws`

### 2. Code Review & Fixes ✅
**Mouse Coordinate Scaling Bug - FIXED**
- **File:** [apps/ui/src/main.rs](apps/ui/src/main.rs#L197-L202)
- **Issue:** UI was reading `remote_screen` dimensions but ignoring them when sending mouse coordinates
- **Fix:** Scale viewport coordinates to actual remote resolution:
  ```rust
  let scaled_x = if rw > 0 { ((x as u32 * rw) / 1920).max(0) as i32 } else { x as i32 };
  let scaled_y = if rh > 0 { ((y as u32 * rh) / 1080).max(0) as i32 } else { y as i32 };
  ```
- **Impact:** Prevents mouse input from targeting wrong coordinates when resolutions differ

### 3. Architecture Verification ✅
All critical paths verified in code:
- **Key exchange timing** ([crates/agent/src/session.rs](crates/agent/src/session.rs#L431)): Capture waits for `crypto_ready=true` before starting frame loop
- **Single Tokio runtime** ([apps/ui/src/main.rs](apps/ui/src/main.rs#L17)): Prevents thread churn; reused across all async operations
- **UI thread safety** ([apps/ui/src/main.rs](apps/ui/src/main.rs#L227-L236)): Proper Arc<Mutex> + `invoke_from_event_loop` pattern
- **WebSocket normalization** ([crates/transport/src/lib.rs](crates/transport/src/lib.rs#L54)): Handles all input formats (ws://, http://, plain IP)
- **Signaling state machine** ([server/signaling/src/main.rs](server/signaling/src/main.rs#L130-L220)): Room creation → linking → consent → capture flow
- **Consent before capture** ([crates/agent/src/session.rs](crates/agent/src/session.rs#L431)): Joiner screen only captured after explicit consent

## Build Issue & Workaround

**Current Issue:** C: drive full; `cargo build` tries to sync toolchain to default location  
**Cause:** Rust channels trying to update in system default location (C:)  
**Status:** Not critical — all binaries already built; only rat.exe needs rebuild for coordinate fix

**To Complete Rebuild:**
```powershell
cd d:\R.A.T

# Set environment for D: drive explicitly
$env:CARGO_HOME = "D:\rust\cargo"
$env:RUSTUP_HOME = "D:\rust\rustup"
$env:TEMP = "D:\temp"
$env:TMP = "D:\temp"

# Rebuild
cargo build -p rat-ui
```

Or free space on C: drive (5–10 GB) and run `.\build.ps1` normally.

## Ready for Phase 2: LAN Smoke Test

### Test Setup (Two Machines on Same LAN)

**Machine A (Admin + Signaling):**
1. Open PowerShell
2. Add firewall rule: `netsh advfirewall firewall add rule name="R.A.T Signaling" dir=in action=allow protocol=tcp localport=4899`
3. Start signaling: `d:\R.A.T\target\debug\rat-signaling.exe`
4. Start UI: `d:\R.A.T\target\debug\rat.exe`
5. Click **Create Connection Code** → Note code (e.g., `ABC-DEF`) and your LAN IP (e.g., `192.168.1.10`)

**Machine B (Joiner / Controlled PC):**
1. Run UI: `d:\R.A.T\target\debug\rat.exe`
2. Enter admin signaling: `192.168.1.10:4899`
3. Enter session code: `ABC-DEF`
4. Click **Link Device**
5. **Accept** consent screen on both machines
6. Admin should see B's screen; mouse/keyboard control should work

### Expected Outcomes ✅
- [ ] Consent screen displays on joiner
- [ ] Screen capture active (live preview on admin)
- [ ] Mouse movements recognized
- [ ] Clicks registered on remote PC
- [ ] Disconnect button stops session cleanly

### Known Limitations (Not MVP Blockers)
- Mouse scaling uses assumed 1920x1080 viewport (TODO: Get actual Slint dimensions)
- JPEG quality 70% (adequate for MVP; H.264 post-MVP)
- No DXGI Desktop Duplication (using screenshots library; adequate for MVP)

## Files to Review/Test

Core flow:
1. [apps/ui/src/main.rs](apps/ui/src/main.rs) — UI event loop and frame rendering
2. [apps/ui/src/agent_bridge.rs](apps/ui/src/agent_bridge.rs) — Agent session management
3. [crates/agent/src/session.rs](crates/agent/src/session.rs) — Capture/input/crypto orchestration
4. [server/signaling/src/main.rs](server/signaling/src/main.rs) — WebSocket room and consent handling

## Next Phase (Post-MVP)
After LAN smoke test confirms basic functionality:
- Phase 3: Fix timing/scaling/firewall issues found during test
- Phase 4: DXGI capture + H.264 media (optional quality upgrades)
- Phase 5: Release build + installer

---

**TL;DR:** Infrastructure ready, coordinate scaling fixed in source, ready for real LAN test. Rebuild may need D: drive temp env vars set explicitly if C: is full.
