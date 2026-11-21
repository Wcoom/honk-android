#!/bin/bash
# build-boxctl.sh - 交叉编译适配 honk 核心的 boxctl 到 Android (aarch64, musl 静态)
set -e
ROOT="$(cd "$(dirname "$0")" && pwd)"
TOOL="$ROOT/toolchain"

export PATH="$TOOL:$HOME/.cargo/bin:$PATH"
export CC_aarch64_unknown_linux_musl="$TOOL/aarch64-linux-musl-gcc"
export CXX_aarch64_unknown_linux_musl="$TOOL/aarch64-linux-musl-g++"
export AR_aarch64_unknown_linux_musl="${AR_aarch64_unknown_linux_musl:-/usr/bin/llvm-ar-18}"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER="$TOOL/zig-link"
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$HOME=/build --remap-path-prefix=$ROOT=/src"

cd "$ROOT/boxproxy"
cargo +1.99.1 build --release --manifest-path boxctl/Cargo.toml \
    --target aarch64-unknown-linux-musl "$@"

OUT="$ROOT/boxproxy/boxctl/target/aarch64-unknown-linux-musl/release/boxctl"
echo
echo "================ boxctl 构建完成 ================"
ls -la "$OUT"
file "$OUT" 2>/dev/null || true
