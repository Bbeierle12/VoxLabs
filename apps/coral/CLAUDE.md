# Coral — project notes for Claude sessions

## Android release signing (read before any `tauri android build`)

- There is exactly ONE Coral release key. Never run `keytool -genkeypair` in this
  repo; never sign a release with a debug key or a fresh key "just for now".
- The key and its `keystore.properties` live in `src-tauri/gen/android/` (both
  gitignored). If they are not present, ask Brandon for the keystore — do not
  build a release. The gradle release build now fails without them by design.
- Key registry with certificate fingerprints: docs/PACKAGING.md, "Release
  signing". The one installed on Brandon's phone as 0.1.0 is the ORIGINAL
  (`CN=Coral local test build`, SHA-256 `197ccd33…`). 0.2.0 was signed with an
  interim key (`coral-release.jks`, alias `coral`, SHA-256 `d5397edb…`) because
  the original was not available in that session. Which one is canonical is
  Brandon's call; until he says otherwise assume the ORIGINAL.
- Verify before delivering an APK: `apksigner verify --print-certs <apk>` and
  compare the digest with the registry.

## Build facts

- Toolchain pins: NDK 28.2.13676358, build-tools 36.1.0, platform 36, JDK 21,
  Gradle 8.14.3 (wrapper). Rust targets: at least `aarch64-linux-android`.
- `versionCode` is derived by the Tauri CLI from `tauri.conf.json` version
  (major·1e6 + minor·1e3 + patch): 0.1.0 → 1000, 0.2.0 → 2000. Bump the version
  in `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml` (and
  both lockfiles) together.
- Quick arm64 release: `npx tauri android build --apk --target aarch64`; output
  `src-tauri/gen/android/app/build/outputs/apk/universal/release/`.
