# honk-android：把 honk 移植到 OnePlus 13（Android 16 / GKI 6.6.118）

目标：用 honk 替代手机上的 mihomo（BoxProxy eBPF 模式）。

## 目录

```
honk-android/
├── src/             honk 源码（v0.1.0-beta.6 + 本项目的 Android 补丁）
├── toolchain/       aarch64-linux-musl 交叉工具链包装器（基于 zig）
│   ├── aarch64-linux-musl-gcc / -g++   cc-rs 找的 C/C++ 编译器
│   ├── zig-link                        rustc 用的链接器
├── lib/             zig 预构建的 aarch64 libc++ / libc++abi / ubsan / compiler_rt
├── build.sh         一键交叉编译
└── PATCHES.md       对上游源码的改动清单（含每个改动的理由）
```

## 构建

```bash
./build.sh
# 产物：src/target/aarch64-unknown-linux-musl/release/honk-core（aarch64 静态）
```

前置（本机已装）：`clang-18`、`llvm-18`、`cmake`、`zig 0.16`、
`rustup` 的 `1.99.1`（含 `aarch64-unknown-linux-musl` target）、
`nightly-2026-10-20`（含 `rust-src`/`llvm-tools`）、`bpf-linker 0.11.0`。

工具链为什么要这么多包装：`boring-sys`（BoringSSL，C++）的 cc-rs 按名字找
`aarch64-linux-musl-gcc`，本机没有 musl 交叉 gcc；用 zig 冒充时又要剥掉
Rust 三元组拼写的 `--target`、补 zig 自带的 musl 头（且**aarch64 头必须在
generic-musl 之前**，否则 `size_t` 变 4 字节）、把 `-lstdc++` 换成 zig 的 libc++。

## 手机上跑（临时目录验证用）

```bash
adb push src/target/aarch64-unknown-linux-musl/release/honk-core /data/local/tmp/honk-core-a64
adb push <你的 config.dae> /data/local/tmp/honk.dae

# 手机侧（需 root）
BASE=/data/local/tmp/honk
mkdir -p $BASE/run $BASE/ssl $BASE/data
mount --bind /apex/com.android.conscrypt/cacerts $BASE/ssl   # honk 要 CA
export HONK_RUN_DIR=$BASE/run SSL_CERT_FILE=$BASE/ssl SSL_CERT_DIR=$BASE/ssl
/data/local/tmp/honk-core-a64 -c /data/local/tmp/honk.dae
```

⚠️ 先停 mihomo：`kill $(pidof mihomo)`，两者抢同一批 cgroup/TC hook。

## 当前状态

| 环节 | 状态 |
|---|---|
| aarch64 静态二进制 | ✅ 能构建、能在手机上跑 |
| 内核版本门禁 | ✅ 已放宽，实测通过 |
| BTF / freplace / map-in-map | ✅ 都可用（实测） |
| cgroup hook 挂载 | ✅ 6 个必需程序全部挂上（需先把系统的 legacy 占用升到 multi） |
| **TC 分类器** | ❌ **过不了 6.6 的 verifier**（1M 指令上限），见 PATCHES.md |

## 换行

`classifier` 段是 honk 的主数据面（L2/L3 入口出口 8 个变体）。它过不了 verifier
就等于没有数据面，所以目前**还不能替代 mihomo**。详因与后续方向见 PATCHES.md。
