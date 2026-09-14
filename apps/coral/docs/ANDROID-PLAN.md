# Coral on Android: review, plan, and the first APK

Status of this pass (September 2026):

- The app was reviewed for how it behaves inside the Android WebView shell
  that Tauri v2 wraps around it. Findings are below, ordered by impact.
- One blocker was fixed in code: the desktop-only layout. Phone-width
  viewports now stack the waterfall above the controls (`src/App.tsx`,
  `src/components/Sidebar.tsx`). Desktop and tablet layouts are unchanged.
- A debug arm64 APK was built from a bare Linux box using only the
  command-line SDK, and the exact recipe is now in `docs/PACKAGING.md`.
- `.github/workflows/android.yml` builds the same APK on GitHub Actions and
  uploads it as an artifact, so nobody needs a local SDK to get a phone build.
- Everything that needs a physical device is written up as a checklist in
  section 4, with the fix already sketched for each expected failure.

## 1. What is already right

The Android scaffolding from PR #1 is sound and needed no changes:

- `src-tauri/gen/android` is a complete Gradle project (AGP 8.11, Kotlin
  1.9.25, Gradle 8.14.3) with `compileSdk`/`targetSdk` 36, `minSdk` 24 and
  `buildToolsVersion` pinned to the one published stable 36.x.
- The manifest declares `RECORD_AUDIO` and `MODIFY_AUDIO_SETTINGS`, marks the
  microphone feature `required="false"`, and leaves `usesCleartextTraffic`
  off in release builds.
- `applicationId` and the `.debug` suffix agree between `tauri.conf.json` and
  `build.gradle.kts`, so debug and release builds install side by side.
- Release signing reads a gitignored `keystore.properties`; nothing secret is
  in the tree.
- The Linux-only `webkit2gtk` crate is gated on `cfg(target_os = "linux")`, so
  the Android library build does not pull GTK. `lib.rs` carries the
  `mobile_entry_point` attribute and the crate type includes `cdylib`.
- `vite.config.ts` lowers the JS target to `safari13` for Tauri builds and
  exposes `TAURI_ENV_*`; the plain web build is untouched.
- The frontend uses only web APIs (getUserMedia, AudioWorklet, Web Worker,
  Canvas 2D). No Tauri command or plugin is called, so the capability file
  only needs `core:default`, and there is nothing to port.
- The AudioWorklet is loaded from the absolute path `/mic-worklet.js`, and the
  DSP worker is bundled through `new URL(..., import.meta.url)`. Both resolve
  against the app origin inside the WebView.
- Cargo's lockfile already resolves the Android-only dependency graph
  (`jni`, `ndk`, `android-activity`), so the first Android build did not
  change `Cargo.lock`.

## 2. Findings

Severity: **P0** unusable, **P1** feature broken, **P2** degraded, **P3** polish.

### P0. Layout was desktop-only. Fixed in this PR.

`App.tsx` rendered a fixed 288 px sidebar beside a 1400:760 canvas. On a
390 px-wide phone the canvas was an 80 px sliver and the controls ran off the
bottom of the screen. Now, below Tailwind's `md` breakpoint (768 px) the root
flex container is a column, the canvas comes first, and the sidebar is
full-width underneath it. On a 390×844 viewport the waterfall, keyboard band,
mode toggle and Start mic button are all visible without scrolling. From
768 px up the classes resolve to the previous layout, and screenshots at
844×390, 1024×768 and 1400×900 are pixel-identical to before the change.

Follow-ups, not done here: landscape on a phone still gives the sidebar a
full-width block under a short canvas. A compact top bar (mode toggle, Start,
Freeze, Save) with the parameter sections in a sheet would be the right mobile
control surface. See Phase 2.

### P1. Save PNG did nothing on Android. Fixed.

`LiveSpectrogramView.exportPng` created a blob URL and clicked an
`<a download>`. Android WebView only honours downloads when the host app
installs a `DownloadListener`; wry does not install one, so the tap was
silent on the first device build.

Fix (`src/utils/save-png.ts`): inside the Tauri shell the frontend asks the
OS for a destination with `@tauri-apps/plugin-dialog`'s `save` (on Android
that is the system "Save as" sheet, `ACTION_CREATE_DOCUMENT`, which returns a
`content://` URI) and writes the PNG bytes there with
`@tauri-apps/plugin-fs`'s `writeFile`, which accepts that URI directly. The
plain browser build keeps the anchor download. Both plugins are registered in
`lib.rs`; the capability grants only `dialog:allow-save` and
`fs:allow-write-file`, because the dialog adds the picked location to the fs
scope at runtime. A dismissed sheet is reported as cancelled, and a failed
write now surfaces in the error banner instead of vanishing. The desktop
Tauri build takes the same path, so it shows a native save dialog rather than
relying on the platform webview's download behaviour.

### P1. Secure-context APIs. Verify on device.

`getUserMedia` and `AudioWorklet` require a secure context. Tauri serves the
Android frontend from `http://tauri.localhost`. Chromium treats
`*.localhost` as potentially trustworthy, so this should pass, but it is the
single assumption everything else rests on. If Start mic fails with a
"failed to load audio worklet" banner, set `app.windows[0].useHttpsScheme`
to `true` in `tauri.conf.json`. Coral persists nothing, so the origin change
that option causes on Windows has no cost.

### P1. Backgrounding kills capture without telling the UI. Planned.

Android stops microphone access for a backgrounded activity (no foreground
service is declared, and one would be inappropriate for this app). The
`MediaStreamTrack` mutes or ends, but `MicPipeline` never listens for
`ended`/`mute`, so the UI keeps saying "Live capture in progress" with a
frozen waterfall. Fix: subscribe to `track.onended` and `document
visibilitychange`, stop the pipeline on hide, and show "Capture paused, tap
Start mic to resume". `FilePlayback` already re-anchors after a background
gap and needs nothing.

### P2. Edge-to-edge insets. Verify on device.

`MainActivity` calls `enableEdgeToEdge()` (mandatory behaviour from
targetSdk 35). The WebView may extend under the status bar and gesture
navigation bar, putting the top of the canvas and the bottom controls under
system chrome. If so, add `viewport-fit=cover` to `index.html` and pad the root
with `env(safe-area-inset-*)`.

### P2. Performance on mid-range phones is unmeasured.

The worker runs a 2048-point display STFT plus the decoupled 8192-point
detection STFT with harmonic cancellation at roughly 90 frames/s at 48 kHz.
The renderer is already throttled to 30 rows/s and max-pools frames, so a
slow worker degrades to skipped rows rather than lag. Measure with Chrome
remote debugging (`chrome://inspect`) on the debug build before deciding
whether a "battery" preset (detection FFT 4096, hop 1024) is needed.

### P2. Orientation. Owner decision.

The waterfall puts frequency across 1400 px, so landscape is the natural
phone orientation. `configChanges` already includes `orientation|screenSize`,
so rotating does not recreate the activity and capture survives rotation.
Whether to lock phones to `sensorLandscape` is a product call; the responsive
layout works either way.

### P3. Android TV launcher category.

The manifest carries `LEANBACK_LAUNCHER` and the leanback feature from the
Tauri template. It is harmless for Play (TV distribution is opt-in) but a mic
app has little use on a TV; drop both lines before the first Play upload to
avoid a TV listing review.

### P3. No Content-Security-Policy.

`tauri.conf.json` sets `csp: null`. Nothing remote is loaded, so risk is low,
but Tauri recommends a CSP. One that keeps the worker, worklet, blob PNG and
inline styles working:
`default-src 'self'; worker-src 'self' blob:; img-src 'self' blob: data:; style-src 'self' 'unsafe-inline'`.
Test it on desktop first; a wrong CSP breaks the app silently.

### P3. Version numbers disagree.

`package.json` says 0.0.1 while `Cargo.toml` and `tauri.conf.json` say 0.1.0.
Tauri derives `versionCode` from its own config, so only the Tauri version
matters for Play, but aligning them avoids confusion.

### P3. Template leftovers.

`res/layout/activity_main.xml` is the "Hello World" layout from the template
and is never inflated. Safe to delete or ignore.

## 3. Plan

### Phase 0. This PR

- Responsive layout (done).
- Local build recipe from a bare machine (done, `docs/PACKAGING.md`).
- CI workflow producing the arm64 debug APK artifact (done).
- This document.

### Phase 1. Device verification

Install the debug APK on one modern phone (Android 13+) and one older device
near `minSdk` if available, then walk the checklist in section 4. Capture
`adb logcat` for every failure. Budget: one afternoon.

### Phase 2. Fix what Phase 1 finds

Expected work, in priority order:

1. ~~PNG export via `plugin-fs`/`plugin-dialog` on Tauri~~ (done).
2. Track-ended and visibility handling in `MicPipeline` plus a "paused" state
   in `App`.
3. Safe-area insets if edge-to-edge overlaps content.
4. A compact mobile control bar; keep the full sidebar for tablets/desktop.
5. Optional battery preset if the worker cannot hold 90 frames/s.

### Phase 3. Signed release

1. Generate the upload keystore (`keytool`, see `docs/PACKAGING.md`), back it
   up offline, and enrol in Play App Signing so Google holds the app key.
2. Store the keystore (base64) and its passwords as repository secrets; add a
   `release` job to `android.yml` that runs only on tags, writes
   `keystore.properties` from the secrets, and builds
   `npm run tauri android build -- --aab`.
3. Confirm `applicationId` is a domain the owner controls; it cannot change
   after the first upload.
4. Remove the TV launcher category, set the CSP, bump the version in
   `tauri.conf.json` (which drives `versionCode`).
5. Upload to an internal testing track; complete the data-safety form (mic
   audio is processed on-device and never leaves it).

### Phase 4. Later

The WebGL2 renderer, device-pixel-ratio handling and retroactive recolour
already on the README roadmap all benefit Android as much as desktop.

## 4. Device checklist

```bash
adb install -r app-arm64-debug.apk
adb logcat -c && adb logcat -s Tauri RustStdoutStderr chromium AndroidRuntime
```

Then, in order:

1. App opens, canvas shows the purple floor and the keyboard band, controls
   are below it (portrait) with Start mic visible.
2. Tap Start mic: the OS microphone prompt appears once. Deny it, confirm the
   red banner explains the failure, re-launch and allow.
3. With permission: level meter moves, waterfall scrolls, note labels and the
   cents readout appear when you sing a steady tone.
4. Check the banner for "Browser kept ... enabled": Android often forces
   noise suppression on the default mic. If it does, try the device picker.
5. Rotate the phone: capture continues, layout switches.
6. Press Home for ten seconds and return. Note whether the waterfall resumes
   or stays frozen (expected: frozen, see P1 backgrounding).
7. Switch to File mode, pick a WAV and an MP3 from the picker, confirm both
   decode and play at real-time pace. Try FLAC and OGG.
8. Change FFT size and the frequency range while running.
9. Freeze, then Save PNG. The system "Save as" sheet opens; pick a folder
   and confirm the PNG opens from the Files app. Cancel the sheet once and
   confirm no error banner appears.
10. Inspect with `chrome://inspect` from a desktop Chrome: look for worker
    frame timing and any console errors.

## 5. How the APK in this pass was built

Environment: Ubuntu container, Rust 1.94.1, Node 22.22, OpenJDK 21,
Gradle 8.14.3 via the wrapper, Android command-line tools 13114758,
`platforms;android-36`, `build-tools;36.1.0`, `ndk;28.2.13676358`,
Rust target `aarch64-linux-android`.

```bash
export ANDROID_HOME=$HOME/Android/Sdk NDK_HOME=$ANDROID_HOME/ndk/28.2.13676358
export JAVA_HOME=/usr/lib/jvm/java-21-openjdk-amd64
npm ci
npm run tauri android build -- --apk --debug --target aarch64 --ci
```

Results (both invoked with `--ci`, both verified with `aapt dump badging` and
`apksigner verify`):

| build | extra flag | output under `app/build/outputs/` | size | signed by |
| --- | --- | --- | --- | --- |
| debug | `--debug` | `apk/universal/debug/app-universal-debug.apk` | 122 MB | Android debug key |
| release | none | `apk/universal/release/app-universal-release.apk` | 11 MB | throwaway local key |

- Package `com.bbeierle.coral.debug` and `com.bbeierle.coral`, `versionCode`
  1000 (derived from 0.1.0), `versionName` 0.1.0, minSdk 24, targetSdk 36,
  `native-code: arm64-v8a`, permissions INTERNET, RECORD_AUDIO and
  MODIFY_AUDIO_SETTINGS.
- Wall time on 4 cores from an empty Cargo registry: 408 s debug (including
  the Gradle distribution and dependency downloads), 263 s release.
- The debug APK is large because the unoptimised Rust library keeps its debug
  info (115 MB of the 122 MB). The release library is 8.4 MB.
- The release build passed R8 minification with the CLI-generated
  `proguard-tauri.pro`, so the Play-bound path is known to build, not only
  the debug one.
- The release APK was signed with a keystore generated for this pass and then
  discarded. It installs and runs for testing, but a build signed with the
  real upload key cannot update it in place; uninstall first. A release build
  without `keystore.properties` comes out unsigned and does not install.
- Neither build modified a tracked file.
- Pitfall seen while iterating: after the frontend changed, an incremental
  debug rebuild produced a 237 MB APK. The zip's entries still totalled
  122 MB; the extra bytes were the replaced Rust library left as dead space
  by AGP's incremental packager. The APK still verified and would install,
  but if a size looks doubled, delete `src-tauri/gen/android/app/build` and
  rebuild. CI always starts clean, so it never sees this.

Only arm64 was built. It covers every phone sold since 2017; add `armv7`,
`x86_64` and `i686` targets (and `rustup target add` for each) for the
universal AAB Play wants, or use `--split-per-abi`.
