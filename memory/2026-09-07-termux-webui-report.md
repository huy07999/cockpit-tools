# Termux WebUI implementation report

- **Goal:** Reuse the Cockpit visual language in a self-hosted browser dashboard similar to local router management tools.
- **Backend:** Added `cockpit serve` with Axum, safe account summary endpoints, Cursor switching, static WebUI hosting, SPA fallback, graceful shutdown, and configurable host/port/web root.
- **Security:** The server binds to `127.0.0.1` by default. Non-loopback binds require a bearer token. Account access and refresh tokens are never serialized by the Web API.
- **Frontend:** Added a responsive React dashboard for Cursor and GitHub Copilot account pools, dark mode, health state, token authentication, refresh, and Cursor switching.
- **Packaging:** The Android workflow builds the WebUI and packages `webui/` beside the ARM64 binary in `cockpit-termux-aarch64.tar.gz`.
- **Evidence:** WebUI TypeScript check and Vite production build pass; `cargo check --package cockpit-cli` passes; three Rust server tests pass; live HTTP smoke tests returned health `200`, unauthorized account access `401`, authenticated account lists `200`, Copilot switch `501`, valid HTML, and working SPA fallback.
- **Status:** DONE
