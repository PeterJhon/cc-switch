# OpenCharm Windows extension for CC Switch (P1)

## Goal
CC Switch remains the upstream desktop base. OpenCharm is isolated as a
Windows-only extension. Existing CC Switch provider, proxy, OAuth, skills,
sessions, database and routing code are not modified.

## Files and boundary
- src-tauri/src/opencharm/mod.rs — trusted Windows process owner.
- src-tauri/src/commands/opencharm.rs — desktop-only Tauri start/stop/status.
- src-tauri/src/commands/mod.rs — isolated command registration.
- src-tauri/src/lib.rs — one app-state registration, one command registration,
  one module declaration, one explicit-exit cleanup hook.

This first PR contains no network or ESP32 API. It must not expose Tauri IPC,
CC Switch credentials, or launch_session_terminal to remote devices.

## Status semantics
The status is intentionally conservative:
- installed: only indicates that a trusted launcher file exists.
- processRunning: only indicates the owned child process has not exited.
- protocolReady: ALWAYS false in P1. P2 must add authenticated health checks.

P1 does NOT yet bundle a Node + charmd runtime. Opening the UI will not
silently launch a fake service. When unbundled, start reports a meaningful
error. Explicit exit stops the child; minimize-to-tray does not.

The provisional trusted Windows launcher contract is:
- resource path: opencharm/opencharm-runtime.exe
- fixed argv: serve
- environment: OPENCHARM_DATA set to the app-local OpenCharm data directory
- Windows development builds only: CCSWITCH_OPENCHARM_DEV_EXECUTABLE override

Before shipping, verify the contract against the pinned OpenCharm build.
The current P1 stop uses process termination and must be replaced with graceful
authenticated admin-IPC shutdown before long-running remote development tasks.

## Windows tests to run in a checkout
    cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
    cargo test --manifest-path src-tauri/Cargo.toml opencharm
    pnpm install --frozen-lockfile
    pnpm build

These tests have NOT been run by the current GitHub-only authoring session.
A Windows executable and real charmd connection have not been verified.

## P1b–P4
P1b: package pinned Windows Node + charmd, verify launch/health and graceful exit.
P2: authenticated local bridge, session/provider summaries only.
P3: real Agent Resume, task ownership, permission and event streaming.
P4: MimiDesk text-only OpenCharm UI; XiaoZhi stays separate.

## Upstream updates
Keep feature logic in OpenCharm paths. Merge changes from
farion1231/cc-switch into a clean integration branch, run regression tests,
and release via a separate signed Windows update channel; never install the
official upstream CC Switch updater over a customized build.
