# Termux WebUI implementation report

- **Goal:** Reuse the Cockpit visual language in a self-hosted browser dashboard similar to local router management tools.
- **Backend:** Added `cockpit serve` with Axum, a registry covering all 18 desktop navigation providers, safe account metadata scanning, tag updates, soft deletion, Cursor switching, static WebUI hosting, SPA fallback, graceful shutdown, and configurable host/port/web root.
- **Security:** The server binds to `127.0.0.1` by default. Non-loopback binds require a bearer token. Account access and refresh tokens are never serialized by the Web API.
- **Frontend:** Added a responsive React dashboard modeled after the desktop floating sidebar and card layout. Provider navigation is generated from `/api/providers`; account screens include search, tags, status, soft deletion, dark mode, health state, token authentication, refresh, and honest Android capability labels.
- **Packaging:** The Android workflow builds the WebUI and packages `webui/` beside the ARM64 binary in `cockpit-termux-aarch64.tar.gz`.
- **Evidence:** WebUI TypeScript check and Vite production build pass; `cargo check --package cockpit-cli` passes; four Rust server tests pass. Live HTTP smoke coverage includes health, auth rejection, all-provider discovery/listing, secret redaction, tag updates, soft deletion, unsupported switch behavior, static assets, and SPA fallback.
- **Status:** DONE
