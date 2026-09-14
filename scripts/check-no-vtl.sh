#!/usr/bin/env bash
# Plan v3 Phase 6 gate, the GPL boundary: the production library (vox-core,
# what the APK loads) depends on no `vox-tract-vtl` crate and carries no
# VocalTractLab symbol. Run in CI after the tests.
set -euo pipefail
cd "$(dirname "$0")/.."
echo "==> cargo tree: vox-core must not depend on vox-tract-vtl"
if cargo tree -p vox-core -e normal --prefix none | grep -qi 'vox-tract-vtl\|vocaltractlab'; then
  echo "vox-core depends on a VTL crate" >&2; exit 1
fi
echo "==> building the production library"
cargo build --release -p vox-core --lib >/dev/null
lib="$(ls target/release/libvox_core.so target/release/libvox_core.dylib 2>/dev/null | head -1 || true)"
if [ -z "$lib" ]; then
  echo "no cdylib produced (target/release/libvox_core.*)" >&2; exit 1
fi
echo "==> symbol check on $lib"
if command -v nm >/dev/null 2>&1; then
  if nm -D --defined-only "$lib" 2>/dev/null | grep -qi 'vtl[A-Z]\|VocalTractLab'; then
    echo "VTL symbols found in the production library" >&2; exit 1
  fi
fi
if strings "$lib" | grep -qi 'libVocalTractLabApi\|vtlInitialize\|vtlTractToTube'; then
  echo "VTL strings found in the production library" >&2; exit 1
fi
echo "OK: no VTL dependency, no VTL symbols"
