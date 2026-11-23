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

# 脱敏编译路径：避免二进制 panic 信息中携带本机用户名/工作目录绝对路径
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$HOME=/build --remap-path-prefix=$ROOT=/src"

cd "$SRC"
# 本机 12 核 / 14 GiB，尽量占满并行
# feature 组合照抄官方 release.yml：
#   full 变体 = clash-api,native-ui，再拼 ebpf,rprx，静态目标加 jemalloc
# 当 HONK_DOONA_DIR 未显式指定时，若 src/target/doona-* 已拉取则自动复用；
# 若设置 FETCH_DOONA=1 则自动调用 ci/fetch-doona.sh 拉取官方预构建 Doona 资源。
if [ -z "${HONK_DOONA_DIR:-}" ]; then
    if [ "${FETCH_DOONA:-0}" = "1" ] && [ -x "$SRC/ci/fetch-doona.sh" ]; then
        HONK_DOONA_DIR="$("$SRC/ci/fetch-doona.sh")"
        export HONK_DOONA_DIR
    elif [ -f "$(dirname "$SRC")/webui/package.json" ]; then
        WEBUI_DIR="$(cd "$(dirname "$SRC")/webui" && pwd)"
        PNPM_BIN="$(command -v pnpm || true)"
        if [ -z "$PNPM_BIN" ] && [ -x "$HOME/.local/bin/pnpm" ]; then
            PNPM_BIN="$HOME/.local/bin/pnpm"
        fi
        if [ -n "$PNPM_BIN" ] && command -v node >/dev/null 2>&1; then
            if [ ! -d "$WEBUI_DIR/node_modules" ]; then
                "$PNPM_BIN" -C "$WEBUI_DIR" install --frozen-lockfile
            fi
            if [ ! -d "$WEBUI_DIR/apps/lite/node_modules" ]; then
                "$PNPM_BIN" -C "$WEBUI_DIR/apps/lite" install --frozen-lockfile
            fi
            if [ "${REBUILD_WEBUI:-0}" = "1" ] || [ ! -f "$WEBUI_DIR/dist/index.html" ] || [ ! -f "$WEBUI_DIR/dist/.vite/bundled-packages.json" ]; then
                "$PNPM_BIN" -C "$WEBUI_DIR" build
            fi
            if [ "${REBUILD_WEBUI:-0}" = "1" ] || [ ! -f "$WEBUI_DIR/apps/lite/dist/index.html" ] || [ ! -f "$WEBUI_DIR/apps/lite/dist/.vite/bundled-packages.json" ]; then
                "$PNPM_BIN" -C "$WEBUI_DIR/apps/lite" build
            fi
            STAGE_DOONA="$SRC/target/doona-embedded"
            for edition in main lite; do
                if [ "$edition" = "main" ]; then
                    DIST_DIR="$WEBUI_DIR/dist"
                    OUT_DIR="$STAGE_DOONA"
                else
                    DIST_DIR="$WEBUI_DIR/apps/lite/dist"
                    OUT_DIR="${STAGE_DOONA}-lite"
                fi
                rm -rf "$OUT_DIR"
                mkdir -p "$OUT_DIR"
                cp -a "$DIST_DIR"/. "$OUT_DIR"/
                rm -rf "$OUT_DIR/fonts" "$OUT_DIR/.vite"
                cp "$WEBUI_DIR/LICENSE" "$WEBUI_DIR/NOTICE" "$WEBUI_DIR/README.md" "$WEBUI_DIR/README.en.md" "$OUT_DIR"/
                (cd "$WEBUI_DIR" && node tools/notices.mjs "$OUT_DIR" "$DIST_DIR/.vite/bundled-packages.json")
            done
            HONK_DOONA_DIR="$STAGE_DOONA"
            export HONK_DOONA_DIR
        fi
    fi
    if [ -z "${HONK_DOONA_DIR:-}" ]; then
        for cand in "$SRC"/target/doona-[0-9]*; do
            if [ -d "$cand" ] && [ -d "${cand}-lite" ]; then
                HONK_DOONA_DIR="$cand"
                export HONK_DOONA_DIR
                break
            fi
        done
    fi
fi

# 当 HONK_DOONA_DIR 指向已拉取的 doona 资源（及 -lite 同级目录）时启用 native-ui（内嵌 Doona），
# 否则启用 native-api（仍支持通过 experimental.native_api.ui 指定本地目录加载 Doona）。
if [ -n "${HONK_DOONA_DIR:-}" ] && [ -d "$HONK_DOONA_DIR" ] && [ -d "${HONK_DOONA_DIR}-lite" ]; then
    FEATURES="ebpf,rprx,clash-api,native-ui,jemalloc"
else
    FEATURES="ebpf,rprx,clash-api,native-api,jemalloc"
fi
cargo build --release -p honk-core --no-default-features --features "$FEATURES" \
    --target aarch64-unknown-linux-musl "$@"

OUT="$SRC/target/aarch64-unknown-linux-musl/release/honk-core"
echo
echo "================ 构建完成 ================"
ls -la "$OUT"
file "$OUT" 2>/dev/null || true
