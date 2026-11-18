#!/bin/bash
# measure.sh —— 快速测量 TC classifier 的 verifier 成本
# 用法：./measure.sh [说明]
set -e
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT"
CARGO_BUILD_JOBS=12 ./build.sh >/tmp/build.log 2>&1 || { echo "构建失败"; tail -20 /tmp/build.log; exit 1; }
adb push src/target/aarch64-unknown-linux-musl/release/honk-core /data/local/tmp/honk-core-a64 >/dev/null 2>&1
adb shell "su -c 'BASE=/data/local/tmp/honk; export HONK_RUN_DIR=\$BASE/run SSL_CERT_FILE=\$BASE/ssl SSL_CERT_DIR=\$BASE/ssl; cd \$BASE && timeout 90 /data/local/tmp/honk-core-a64 -c /data/local/tmp/honk-min.dae > \$BASE/m.log 2>&1; grep -oE \"processed [0-9]+ insns|Processed [0-9]+ insn|is running|fatal error[^/]*\" \$BASE/m.log | tail -3'" 2>&1
