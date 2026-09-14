#!/usr/bin/env bash
# Builds the dev-flavoured Android APK reproducibly: application id
# `org.voxlabs.core.dev`, launcher label "VoxLabs (dev)", release profile,
# 16 KB page alignment, verified and copied to dist/ under the
# `voxlabs-dev-<feature>-<sha>.apk` convention (docs/STATUS.md §5).
#
# cargo-apk has no per-profile application id, so the id and label are
# patched into Cargo.toml for the duration of the build and restored
# afterwards — on success, failure, or Ctrl-C — from a byte copy, so any
# uncommitted edits to Cargo.toml survive.
#
# Usage:
#   scripts/build-dev-apk.sh [--debug] [<feature>]
#     <feature>   tag for the output name (default: current branch's last
#                 commit subject prefix, e.g. "phase1"); letters, digits, -, _
#     --debug     dev profile (no release keystore needed; unoptimized —
#                 not for the Phase 1 timing gate)
#
# Environment (release builds):
#   ANDROID_HOME        SDK root            default: $HOME/android-sdk
#   ANDROID_NDK_ROOT    NDK root            default: $ANDROID_HOME/ndk/26.3.11579264
#   CARGO_APK_RELEASE_KEYSTORE, CARGO_APK_RELEASE_KEYSTORE_PASSWORD
#                       the signing key (docs/android-build.md §2)
#   BUILD_TOOLS_VERSION build-tools to verify with   default: 34.0.0
set -euo pipefail

readonly BASE_ID="org.voxlabs.core"
readonly DEV_ID="org.voxlabs.core.dev"
readonly BASE_LABEL="Voice Harmonic Engine"
readonly DEV_LABEL="VoxLabs (dev)"
# Pixel 8+ / Android 15+ run with 16 KB pages; NDK 26 links 4 KB-aligned by
# default and such a .so cannot be mapped (docs/android-build.md §3).
readonly PAGE_SIZE_BYTES=16384
readonly PAGE_SIZE_HEX="0x4000"

profile="release"
feature=""
for arg in "$@"; do
  case "$arg" in
    --debug) profile="dev" ;;
    -h|--help) sed -n '2,24p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
    -*) echo "unknown option: $arg" >&2; exit 2 ;;
    *) feature="$arg" ;;
  esac
done

repo="$(cd "$(dirname "$0")/.." && pwd)"
cd "$repo"

export ANDROID_HOME="${ANDROID_HOME:-$HOME/android-sdk}"
export ANDROID_SDK_ROOT="${ANDROID_SDK_ROOT:-$ANDROID_HOME}"
export ANDROID_NDK_ROOT="${ANDROID_NDK_ROOT:-$ANDROID_HOME/ndk/26.3.11579264}"
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-$ANDROID_NDK_ROOT}"
case "${RUSTFLAGS:-}" in
  *max-page-size*) ;;   # caller already set it; keep the flag string stable for cargo's cache
  *) export RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }-C link-arg=-Wl,-z,max-page-size=${PAGE_SIZE_BYTES}" ;;
esac
build_tools="$ANDROID_HOME/build-tools/${BUILD_TOOLS_VERSION:-34.0.0}"
readelf="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-readelf"

fail() { echo "build-dev-apk: $*" >&2; exit 1; }

command -v cargo-apk >/dev/null || fail "cargo-apk not installed (cargo install cargo-apk)"
[ -x "$build_tools/aapt" ] || fail "aapt not found under $build_tools (set BUILD_TOOLS_VERSION / ANDROID_HOME)"
[ -x "$readelf" ] || fail "llvm-readelf not found under $ANDROID_NDK_ROOT"
if [ "$profile" = "release" ]; then
  [ -n "${CARGO_APK_RELEASE_KEYSTORE:-}" ] && [ -n "${CARGO_APK_RELEASE_KEYSTORE_PASSWORD:-}" ] \
    || fail "release build needs CARGO_APK_RELEASE_KEYSTORE and _PASSWORD (or pass --debug)"
fi

grep -q "^package = \"$BASE_ID\"" Cargo.toml || fail "Cargo.toml does not declare package = \"$BASE_ID\""
grep -q "^label = \"$BASE_LABEL\"" Cargo.toml || fail "Cargo.toml does not declare label = \"$BASE_LABEL\""

if [ -z "$feature" ]; then
  feature="$(git log -1 --format=%s | sed -E 's/^[a-z]+(\([^)]*\))?: //; s/[^A-Za-z0-9]+.*//' | tr 'A-Z' 'a-z')"
  feature="${feature:-build}"
fi
[[ "$feature" =~ ^[A-Za-z0-9_-]+$ ]] || fail "feature tag must be letters, digits, - or _: '$feature'"
sha="$(git rev-parse --short HEAD)"

# Patch, and guarantee the restore.
backup="$(mktemp "${TMPDIR:-/tmp}/Cargo.toml.XXXXXX")"
cp Cargo.toml "$backup"
restore() { cp "$backup" Cargo.toml; rm -f "$backup"; }
trap restore EXIT
sed -i "s/^package = \"$BASE_ID\"/package = \"$DEV_ID\"/; s/^label = \"$BASE_LABEL\"/label = \"$DEV_LABEL\"/" Cargo.toml
grep -q "^package = \"$DEV_ID\"" Cargo.toml && grep -q "^label = \"$DEV_LABEL\"" Cargo.toml \
  || fail "patching Cargo.toml failed"

echo "==> building $profile APK as $DEV_ID (\"$DEV_LABEL\")"
if [ "$profile" = "release" ]; then
  cargo apk build --lib --release
  apk="target/release/apk/vox-core.apk"
  so="target/aarch64-linux-android/release/libvox_core.so"
else
  cargo apk build --lib
  apk="target/debug/apk/vox-core.apk"
  so="target/aarch64-linux-android/debug/libvox_core.so"
fi
[ -f "$apk" ] || fail "expected $apk after the build"

echo "==> verifying"
badging="$("$build_tools/aapt" dump badging "$apk")"
grep -q "^package: name='$DEV_ID'" <<<"$badging" || fail "APK package id is not $DEV_ID"
# The label is a raw manifest attribute here (no resources), which badging
# prints as empty; read the manifest tree instead.
"$build_tools/aapt" dump xmltree "$apk" AndroidManifest.xml | grep -q "android:label.*\"$DEV_LABEL\"" \
  || fail "APK label is not \"$DEV_LABEL\""
grep -q "native-code: 'arm64-v8a'" <<<"$badging" || fail "APK carries no arm64-v8a library"
loads="$("$readelf" -l "$so" | awk '/LOAD/ {print $NF}' | sort -u)"
[ "$loads" = "$PAGE_SIZE_HEX" ] || fail "LOAD segments not all $PAGE_SIZE_HEX-aligned: $loads"
# apksigner is silent on success; only its exit status matters.
"$build_tools/apksigner" verify "$apk" >/dev/null 2>&1 || fail "signature does not verify"

mkdir -p dist
out="dist/voxlabs-dev-${feature}-${sha}.apk"
cp "$apk" "$out"
echo "==> $out"
echo "    id $DEV_ID · label \"$DEV_LABEL\" · $profile · LOAD align $loads · $(stat -c %s "$out") bytes"
echo "    install:  adb install -r $out"
echo "    permit:   adb shell pm grant $DEV_ID android.permission.RECORD_AUDIO"
