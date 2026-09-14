# Packaging Coral (desktop & Android)

Coral ships as a [Tauri v2](https://v2.tauri.app) app from the same Vite/React
frontend used on the web. One `src-tauri/` drives **desktop** (Linux/macOS/
Windows) and **Android** (for Google Play). iOS is intentionally not configured.

The web build is unchanged: `npm run build` still produces `dist/` exactly as
before. Tauri only consumes that output.

## Prerequisites

- **Rust** (stable) + **Node 22**.
- Desktop Linux: WebKitGTK 4.1 dev libs (`libwebkit2gtk-4.1-dev`, `libsoup-3.0`,
  GTK3, `librsvg2-dev`, `libayatana-appindicator3-dev`, `build-essential`).
- Android only:
  - JDK 17+ (JDK 21 works).
  - Android SDK with `platforms;android-36`, `build-tools;36.1.0`,
    `platform-tools`, and `ndk;28.2.13676358`. From a bare machine (no
    Android Studio) the command-line tools are enough:

    ```bash
    export ANDROID_HOME="$HOME/Android/Sdk"
    mkdir -p "$ANDROID_HOME/cmdline-tools"
    curl -sSL -o /tmp/clt.zip \
      https://dl.google.com/android/repository/commandlinetools-linux-13114758_latest.zip
    unzip -q /tmp/clt.zip -d /tmp/clt && mv /tmp/clt/cmdline-tools "$ANDROID_HOME/cmdline-tools/latest"
    SDKM="$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager"
    yes | "$SDKM" --sdk_root="$ANDROID_HOME" --licenses >/dev/null
    "$SDKM" --sdk_root="$ANDROID_HOME" "platform-tools" "platforms;android-36" \
      "build-tools;36.1.0" "ndk;28.2.13676358"
    ```

  - Env: `ANDROID_HOME=~/Android/Sdk`, `NDK_HOME=$ANDROID_HOME/ndk/28.2.13676358`,
    `JAVA_HOME` set.
  - Rust targets: `rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android`.

## Desktop

```bash
npm run tauri dev          # dev: Vite dev server + native window, hot reload
npm run tauri build        # release: bundles (.deb/.AppImage/.rpm on Linux)
npm run tauri build -- --no-bundle   # just the binary, skip OS packaging
```

**Microphone on Linux.** WebKitGTK denies `getUserMedia` by default — the
webview emits a `permission-request` that nothing answers, so a browser-working
mic app silently fails. `src-tauri/src/lib.rs` fixes this on Linux only: it
enables the media-stream setting and auto-grants the permission request via
`with_webview`. No change is needed on macOS/Windows (their webviews prompt
natively). In some headless/VM display stacks you may also need
`WEBKIT_DISABLE_DMABUF_RENDERER=1` at runtime.

## Android (Google Play)

```bash
export ANDROID_HOME="$HOME/Android/Sdk"
export NDK_HOME="$ANDROID_HOME/ndk/28.2.13676358"
export JAVA_HOME="/usr/lib/jvm/java-21-openjdk-amd64"

npm run tauri android build -- --aab            # release AAB for Play (all ABIs)
npm run tauri android build -- --apk --debug -t aarch64   # quick local check
```

Outputs land under `src-tauri/gen/android/app/build/outputs/`; the arm64 debug
APK is `apk/universal/debug/app-universal-debug.apk` (the CLI builds the
"universal" flavor containing only the requested ABI). It is signed with the standard
Android debug key, installs as `com.bbeierle.coral.debug`, and can be
sideloaded (`adb install -r app-universal-debug.apk`).

**CI.** `.github/workflows/android.yml` builds that same arm64 debug APK on
GitHub Actions (on demand via *Run workflow*, on every push to `master`, and on
pull requests that touch `src-tauri/` or the workflow) and uploads it as the
`coral-android-arm64-debug` artifact. See `docs/ANDROID-PLAN.md` for the
device-verification checklist and the road to a signed Play release.

**Microphone on Android.** `getUserMedia({audio})` works because the manifest
(`src-tauri/gen/android/app/src/main/AndroidManifest.xml`) declares
`RECORD_AUDIO` + `MODIFY_AUDIO_SETTINGS`. wry's `WebChromeClient` does the rest:
it requests the runtime permission and grants the WebView capture request. The
mic feature is declared `required="false"` so mic-less devices can still install
and use file-playback mode.

**Save PNG on Android.** The WebView has no download handler, so
`src/utils/save-png.ts` routes exports in the Tauri shell through the dialog
plugin's system save sheet and the fs plugin's `writeFile` (capabilities:
`dialog:allow-save`, `fs:allow-write-file`). The web build keeps the plain
browser download.

**Play config** (`src-tauri/gen/android/app/build.gradle.kts`):

- `applicationId = "com.bbeierle.coral"` — **permanent once published**. Change
  it to a reverse-DNS of a domain you control before the first upload.
- `targetSdk = 36`, `minSdk = 24`. API 36 satisfies Play's target-API rule
  through the 2026 deadline.
- `versionCode` / `versionName` come from `tauri.properties`; bump `versionCode`
  on every upload.

### Release signing — ONE key, never regenerate

Android only installs an update over an app signed with the **same** key. Coral
has already been signed with two different keys by accident, and every extra key
means "uninstall, lose settings, reinstall". The rule from here on:

**Do not run `keytool -genkeypair` for this project. Ever.** If the key is not on
the machine, stop and fetch it from the backup — do not build a release with a
fresh key "just to test".

Key registry (SHA-256 of the signing certificate — check an APK with
`apksigner verify --print-certs app.apk`, a keystore with `keytool -list -v`):

| Key | Signed | Cert SHA-256 |
|---|---|---|
| **Original** — `CN=Coral local test build, OU=dev, O=Coral` (Brandon's local keystore) | 0.1.0 (versionCode 1000), the copy installed on the phone | `197ccd33…07f989a` |
| Interim — `coral-release.jks`, alias `coral` (created 2026-09-05 in a cloud session because the original was not available there) | 0.2.0 (versionCode 2000) | `d5397edb…1371aa1a` |

Whichever of these becomes canonical, the other is retired. The canonical key
lives at `src-tauri/gen/android/<name>.jks` with its `keystore.properties`
beside it (both gitignored) **and** in an offline backup; the build below refuses
to assemble a release without them.

```properties
# src-tauri/gen/android/keystore.properties
storeFile=../coral-release.jks
storePassword=…
keyAlias=coral
keyPassword=…
```

For Play, enrol in Play App Signing with this as the upload key; Google then
holds the app-signing key and a lost upload key is recoverable.

### Upload (manual, your account)

`src-tauri/gen/android/.../*.aab` → Google Play Console → create app → upload to
a testing track → complete the store listing / data-safety form. These steps
require your Play Developer account and can't be automated here.
