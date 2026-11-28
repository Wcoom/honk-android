#!/usr/bin/env bash
# Non-destructive cutover script for honk-android pure-Rust rewrite.
# Default mode is --dry-run. Pass --apply ONLY after explicit user confirmation.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APPLY=0
if [[ "${1:-}" == "--apply" ]]; then
  APPLY=1
fi

echo "=== honk-android Pure-Rust Cutover Script ==="
echo "Repo root : $ROOT"
echo "Mode      : $([[ $APPLY -eq 1 ]] && echo 'APPLY' || echo 'DRY-RUN (no mutations; pass --apply to execute)')"
echo

# 1. Verify protected WIP files are clean/untouched by our branch logic
echo "[1/5] Checking protected user WIP paths (.gitignore, build-cf-opt.sh, cf-opt/)..."
for p in ".gitignore" "build-cf-opt.sh" "cf-opt"; do
  if [[ -e "$ROOT/$p" ]]; then
    echo "  OK (preserved, will NOT be modified): $p"
  fi
done

# 2. Verify all pure-Rust artifacts exist
echo "[2/5] Verifying pure-Rust build artifacts..."
ARTIFACTS=(
  "rust/boxproxy/android-shell/app/src/main/assets/boxctl/arm64-v8a/boxbpf"
  "boxproxy/boxctl/target/aarch64-linux-android/release/boxctl"
  "src/target/aarch64-unknown-linux-musl/release/honk-core"
  "rust/boxproxy/target/aarch64-linux-android/release/libboxproxy_app.so"
  "rust/boxproxy/android-shell/app/build/outputs/apk/release/app-release.apk"
)
for rel in "${ARTIFACTS[@]}"; do
  if [[ -f "$ROOT/$rel" ]]; then
    ls -lh "$ROOT/$rel"
  else
    echo "  MISSING: $rel (rebuild before cutover)"
  fi
done

# 3. Preview src/ patch diff to sync into patches/honk-android.patch
echo "[3/5] Checking src/ delta to sync into patches/honk-android.patch..."
(cd "$ROOT/src" && git status --short) || true

# 4. Planned cutover actions
echo "[4/5] Cutover actions:"
cat <<'EOF'
  a) Sync src/crates/honk-core/{build.rs,src/native_api/ui.rs} WASM support into patches/honk-android.patch
  b) Switch android-shell applicationId from io.honk.android -> com.boxproxy.honk (when replacing original app)
  c) Point CI/release workflows to rust/xtask (build-boxbpf, build-webui, build-boxctl) & rust/boxproxy/android-shell
  d) Archive legacy C boxbpf sources & React webui/ once release tag is published
EOF

if [[ $APPLY -eq 0 ]]; then
  echo
  echo "[5/5] Dry-run complete. Re-run with '--apply' after explicit approval to perform cutover."
  exit 0
fi

echo "[5/5] Applying non-destructive patch sync (patches/honk-android.patch)..."
(cd "$ROOT/src" && git diff -- crates/honk-core/build.rs crates/honk-core/src/native_api/ui.rs > "$ROOT/rust/honk-wasm-ui.patch")
echo "  Wrote $ROOT/rust/honk-wasm-ui.patch for review/merge into patches/honk-android.patch."
echo "  To switch APK package ID to com.boxproxy.honk, update rust/boxproxy/android-shell/app/build.gradle.kts and rebuild."
