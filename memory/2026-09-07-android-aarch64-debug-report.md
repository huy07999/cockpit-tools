# Android aarch64 debug report

- **Symptom:** The `Build Cockpit CLI - Termux Android ARM64` workflow failed three times while cross-compiling `cockpit-cli` for `aarch64-linux-android`.
- **Root cause:** `reqwest` originally enabled native TLS and required a target OpenSSL installation; after switching to rustls, the desktop-only `trash` crate was still compiled for Android; the attempted target-specific dependency fix was then placed in the virtual workspace manifest, which Cargo rejects. Once those failures were fixed, `cockpit-cli` still compiled every desktop module in `cockpit-core`, exposing Linux/macOS/Windows-only code paths on Android.
- **Fix:** Keep shared dependency versions in the workspace, inherit `trash` only for non-Android targets, permanently remove instance directories on Android because Termux has no desktop trash service, and add a minimal `cli` feature that excludes desktop modules and optional Tauri dependencies.
- **Evidence:** `cargo metadata --no-deps --format-version 1`, `cargo check --package cockpit-cli`, and `cargo run --package cockpit-cli -- --help` pass on an `aarch64-linux-android` Termux host.
- **Regression test:** `.github/workflows/android-termux.yaml` builds the release CLI with the Android NDK for `aarch64-linux-android` on every push to `main` and on manual dispatch.
- **Related:** GitHub Actions runs `34070395926`, `34070718007`, `34071309320`, and `34074169336` captured the OpenSSL, `trash`, invalid virtual-manifest, and desktop-module coupling failures respectively.
- **Status:** DONE
