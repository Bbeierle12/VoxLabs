# Android APK — build & install runbook

Native Android build of **Voice Harmonic Engine**: the same egui `DashboardApp`
as desktop, packaged as a `NativeActivity` APK. Real-time mic capture + additive
synthesis via cpal's AAudio backend; pitch/formant DSP runs on the **CPU**
(`math::yin_pitch` + LPC) — the wgpu GPU-compute path is desktop-only and is not
compiled into the APK.

- **Package:** `com.voiceharmonic.engine`
- **App label:** Voice Harmonic Engine
- **minSdk 26 (Android 8.0), target 34.** AAudio (the low-latency audio path)
  requires API 26, so 26 is the floor.
- **ABI:** `arm64-v8a` only (Pixel and every modern phone). Add more in
  `Cargo.toml` → `[package.metadata.android].build_targets` if needed.
- **Entry point:** `android_main` in `src/android.rs` (behind
  `#[cfg(target_os = "android")]`), called by android-activity's NativeActivity
  glue. Desktop (`src/lib.rs::run`) and web (`src/main.rs` wasm) are untouched.

---

## 1. One-time toolchain setup (build host, not the phone)

Already provisioned in this environment; reproduce elsewhere with:

```bash
# Rust target (already installed here)
rustup target add aarch64-linux-android

# Android SDK command-line tools -> $ANDROID_HOME
export ANDROID_HOME="$HOME/android-sdk"
# ... unzip commandlinetools into $ANDROID_HOME/cmdline-tools/latest/ ...
yes | "$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" --licenses
"$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" \
  "platform-tools" "platforms;android-34" "build-tools;34.0.0" "ndk;26.3.11579264"

# Rust -> Android packaging tool
cargo install cargo-apk        # v0.10.0 used here
```

Java 21 is required (present) for `keytool` / `apksigner`.

> Note: on hosts whose `/tmp` is mounted `noexec`, build scripts fail with
> `Permission denied (os error 13)`. Point temp + cargo at `$HOME`:
> `export TMPDIR="$HOME/tmp"` (and, for `cargo install`,
> `CARGO_TARGET_DIR="$HOME/.cache/cargo-apk-build"`).

## 2. Release signing keystore (secret — never commit)

A self-signed release keystore was generated **outside the repo**:

| field | value |
|-------|-------|
| path | `~/.android-keystores/voice_harmonic_engine-release.jks` |
| alias | `voiceharmonic` |
| store password | in `~/.android-keystores/voice_harmonic_engine-release.password` (not in the repo) |
| key password | same as store password |
| cert | `CN=Voice Harmonic Engine, OU=Development, O=VoiceHarmonic, C=US` |
| SHA-256 | `05:09:86:79:47:E7:EF:D8:C5:6E:1A:95:F9:8C:77:AD:75:CA:D8:1D:35:89:3D:E5:21:E9:5C:FB:EF:E8:9E:4E` |

Regenerate (e.g. on another machine) with:

```bash
STOREPASS="$(cat ~/.android-keystores/voice_harmonic_engine-release.password)"
keytool -genkeypair -v \
  -keystore ~/.android-keystores/voice_harmonic_engine-release.jks \
  -alias voiceharmonic -keyalg RSA -keysize 2048 -validity 10000 \
  -storepass "$STOREPASS" -keypass "$STOREPASS" \
  -dname "CN=Voice Harmonic Engine, OU=Development, O=VoiceHarmonic, L=Unknown, ST=Unknown, C=US"
```

> This is a throwaway dev key — fine for sideloading. Its original password was
> committed in earlier revisions of this file and remains in git history, so
> treat this key as public knowledge; generate a real, strong-password key
> before any Play Store distribution, and keep it (Play requires all updates be
> signed by the same key). `.gitignore` blocks `*.apk`, `*.keystore`, `*.jks`,
> and `dist/`.

## 3. Build the signed release APK

`cargo-apk` reads the keystore path + password from env vars, so no secret is
written into `Cargo.toml`:

```bash
export ANDROID_HOME="$HOME/android-sdk"
export ANDROID_SDK_ROOT="$HOME/android-sdk"
export ANDROID_NDK_ROOT="$HOME/android-sdk/ndk/26.3.11579264"
export ANDROID_NDK_HOME="$ANDROID_NDK_ROOT"
export CARGO_APK_RELEASE_KEYSTORE="$HOME/.android-keystores/voice_harmonic_engine-release.jks"
export CARGO_APK_RELEASE_KEYSTORE_PASSWORD="$(cat "$HOME/.android-keystores/voice_harmonic_engine-release.password")"
export TMPDIR="$HOME/tmp"

cargo apk build --lib --release
```

Output (already produced here):

```
target/release/apk/voice_harmonic_engine.apk      # cargo-apk output
dist/voice_harmonic_engine-release.apk            # copy kept as the deliverable
```

`--lib` is required: the app is a `cdylib` (`libvoice_harmonic_engine.so`) loaded
by NativeActivity; the `[[bin]]` target is only the desktop/web entry.

Verify the signature and manifest:

```bash
BT="$ANDROID_HOME/build-tools/34.0.0"
"$BT/apksigner" verify --print-certs target/release/apk/voice_harmonic_engine.apk
"$BT/aapt" dump badging     target/release/apk/voice_harmonic_engine.apk
```

Expected: v2 + v3 signature schemes verified, signer SHA-256 matching the table
above, `sdkVersion:'26'`, `native-code:'arm64-v8a'`, RECORD_AUDIO permission.

---

## 4. Install on your Pixel (your steps — not done here)

This build host has **no device access**; do the following on your phone.

1. **Enable Developer Options:** Settings → About phone → tap **Build number**
   7 times.
2. **Enable USB debugging:** Settings → System → Developer options → **USB
   debugging** = on. Plug the Pixel into the machine that has the APK and accept
   the "Allow USB debugging?" RSA-key prompt on the phone.
3. **Confirm the device is visible:**
   ```bash
   "$ANDROID_HOME/platform-tools/adb" devices     # your Pixel should be "device"
   ```
4. **Install (replace if already installed):**
   ```bash
   "$ANDROID_HOME/platform-tools/adb" install -r dist/voice_harmonic_engine-release.apk
   ```
   If a previous install with a different signature blocks it:
   `adb uninstall com.voiceharmonic.engine` first.
5. **Launch** "Voice Harmonic Engine" from the app drawer (default system icon —
   no custom launcher icon is bundled yet; see Known rough edges).
6. **Grant the microphone permission.** This build uses a plain `NativeActivity`
   and does **not** pop the runtime permission dialog itself, so grant it
   manually — either:
   - Settings → Apps → Voice Harmonic Engine → Permissions → Microphone → Allow, **or**
   ```bash
   "$ANDROID_HOME/platform-tools/adb" shell pm grant com.voiceharmonic.engine android.permission.RECORD_AUDIO
   ```
   then fully close and reopen the app. Until RECORD_AUDIO is granted the UI runs
   but the audio input stream can't open, so the dashboard stays in "SEARCHING".
7. **Watch logs while testing:**
   ```bash
   "$ANDROID_HOME/platform-tools/adb" logcat -s VoiceHarmonicEngine RustStdoutStderr '*:E'
   ```

---

## Known rough edges / unverified

- **Not run on hardware.** The APK builds, assembles, and signs cleanly, but no
  device was available here — on-device launch, egui/glow rendering, live mic
  capture, synthesis output, and **audio latency** are all unverified.
- **Mic permission is manual** (step 6). A proper JNI runtime-permission request
  (or an `androidx`/`RustActivity` shell) is a follow-up.
- **No custom launcher icon.** The app uses the Android default icon. To add one:
  drop `res/mipmap-*/ic_launcher.png` in the crate, set
  `[package.metadata.android] resources = "res"` and
  `[package.metadata.android.application] icon = "@mipmap/ic_launcher"`.
- **arm64-v8a only.** No 32-bit or x86_64-emulator ABI; add to `build_targets`
  if you need them.
- **CPU DSP only on Android.** The wgpu YIN compute path is desktop-only by
  design here; formant/pitch analysis on Android is the CPU reference path.
