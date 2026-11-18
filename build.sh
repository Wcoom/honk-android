#!/bin/bash
# build.sh - 交叉编译 honk-core 到 Android (aarch64, musl 静态)
#
# 依赖（均为本机已装）：
#   - rustup 工具链 1.99.1 + target aarch64-unknown-linux-musl
#   - zig（充当 musl 交叉 C/C++ 编译器，见 toolchain/）
#   - cmake（boring-sys 用）
#   - 预编译的 eBPF 对象（从官方 aarch64 release 提取，见 extract-ebpf.sh）
set -e
ROOT="$(cd "$(dirname "$0")" && pwd)"
SRC="$ROOT/src"
TOOL="$ROOT/toolchain"

export PATH="$TOOL:$HOME/.cargo/bin:$PATH"
export CC_aarch64_unknown_linux_musl="$TOOL/aarch64-linux-musl-gcc"
export CXX_aarch64_unknown_linux_musl="$TOOL/aarch64-linux-musl-g++"
export AR_aarch64_unknown_linux_musl=/usr/bin/llvm-ar-18
# 链接器用 zig-link 包装器：关掉 zig 自带 crt 避免 _start 重复，
# 并把 -lstdc++ 改写成 -lc++（musl 目标没有 libstdc++）。
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER="$TOOL/zig-link"
# rust-lld 支持 --fix-cortex-a53-843419；-nostartfiles 保留 rust 自带 crt

export CMAKE_C_COMPILER_LAUNCHER=""
export CMAKE_CXX_COMPILER_LAUNCHER=""

# boring-sys 的 bindgen 不经过我们的 CC 包装器，自己拼 clang 参数；
# 交叉时它默认看宿主 glibc 头，找不到 bits/libc-header-start.h。
# 用 BINDGEN_EXTRA_CLANG_ARGS 把 zig 自带的 musl 头路径塞进去。
ZIG_LIB="${ZIG_LIB:-/usr/lib/zig0.16/lib}"
# ⚠️ 顺序必须 aarch64 在前：generic-musl 的 bits/alltypes.h 把 _Addr 定义成 int，
# aarch64-linux-musl 的定义成 long；反过来 include 会让 size_t=4，bindgen 直接断言崩。
MUSL_INC="-isystem $ZIG_LIB/libc/include/aarch64-linux-musl -isystem $ZIG_LIB/libc/include/generic-musl -isystem $ZIG_LIB/libc/include/any-linux-any"
export BINDGEN_EXTRA_CLANG_ARGS_aarch64_unknown_linux_musl="$MUSL_INC"
export BINDGEN_EXTRA_CLANG_ARGS="$MUSL_INC"

cd "$SRC"
# 本机 12 核 / 14 GiB，尽量占满并行
# feature 组合照抄官方 release.yml：
#   full 变体 = clash-api,native-ui，再拼 ebpf,rprx，静态目标加 jemalloc
# 去掉 native-ui：它要求联网拉取 doona 前端资源（HONK_DOONA_DIR），
# 本机装不了；clash-api 足够（BoxProxy 走的就是 Clash API）。
FEATURES="ebpf,rprx,clash-api,jemalloc"
cargo build --release -p honk-core --no-default-features --features "$FEATURES" \
    --target aarch64-unknown-linux-musl "$@"

OUT="$SRC/target/aarch64-unknown-linux-musl/release/honk-core"
echo
echo "================ 构建完成 ================"
ls -la "$OUT"
file "$OUT" 2>/dev/null || true
