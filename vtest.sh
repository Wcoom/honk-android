#!/bin/bash
# vtest.sh —— 只编 eBPF 对象，推到手机，用 --bpf-object 加载，看 verifier 结果
# （不重编 24MB 的 honk-core，几十秒一轮）
# 用法：./vtest.sh [config.dae on phone]
set -e
ROOT="$(cd "$(dirname "$0")" && pwd)"
CFG="${1:-/data/local/tmp/honk-min.dae}"
export PATH="$HOME/.cargo/bin:$PATH"
cd "$ROOT/src/crates/honk-ebpf"
env -u RUSTFLAGS CARGO_TARGET_DIR="$PWD/target" \
    cargo +nightly-2026-10-20 build --release -Zbuild-std=core --target bpfel-unknown-none 2>&1 \
    | grep -E "^(error|warning: unused)|Finished" || true
OBJ="$PWD/target/bpfel-unknown-none/release/honk-ebpf"
[ -f "$OBJ" ] || { echo "no object"; exit 1; }
adb push "$OBJ" /data/local/tmp/honk-ebpf.o >/dev/null
adb shell "su -c 'BASE=/data/local/tmp/honk; export HONK_RUN_DIR=\$BASE/run SSL_CERT_FILE=\$BASE/ssl SSL_CERT_DIR=\$BASE/ssl; cd \$BASE && timeout ${TIMEOUT:-60} /data/local/tmp/honk-core-a64 -c $CFG -b /data/local/tmp/honk-ebpf.o > \$BASE/v.log 2>&1; echo rc=\$?'"
adb shell "su -c 'cat /data/local/tmp/honk/v.log | grep -E \"ERROR|Error|stack|processed|is running|Attached|listening|insn\" | cut -c1-400 | tail -12'"
