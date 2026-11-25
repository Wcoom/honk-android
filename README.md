# Honk for Android (`honk-android`)

将基于 Linux eBPF 的新一代透明代理核心 **[daeuniverse/honk](https://github.com/daeuniverse/honk)** 完整移植至 **Android ARM64（GKI Linux 6.1+ / 6.6+）** 平台，并深度集成 **[boxproxy/boxproxy](https://github.com/boxproxy/boxproxy)** 客户端与内嵌 **[Zakkaus/doona](https://github.com/Zakkaus/doona)** 原生面板。

> **免刷机、零环境变量、单二进制开箱即用**：全程仅在 `/data` 用户空间与内核 eBPF 子系统运行，无需刷写任何系统/boot 分区镜像。

---

## 核心特性

| 模块 | 状态与说明 |
| --- | --- |
| **Android GKI 6.1+ / 6.6+ eBPF 数据面** | 全部 8 个 L2/L3 TC 分类器 + 6 个根 cgroup 程序 + `daens` `sk_lookup` + `freplace` 路由扩展通过 GKI 6.1+ / 6.6+ Verifier；6.6+ 保持原生 TCX，6.1–6.5 自动回退 Netlink `clsact`/`cls_bpf` 并支持 BTF map-in-map 路由描述符（详见 [`ANDROID-DATAPATH.md`](./ANDROID-DATAPATH.md)） |
| **Android 网络栈自适配** | 启动时自动将根 cgroup legacy BPF 钩子无损升级为 `BPF_F_ALLOW_MULTI`；`DAE_BYPASS_MARK=0x10000000` 避开 Android `netd` 策略路由冲突；`wan_interface: auto` 解析 `/proc/net/fib_trie` 动态跟踪 5G 蜂窝 / Wi-Fi 切换 |
| **完整 UDP / QUIC 透明代理 & 内核追踪** | 支持完整 UDP / HTTP3 (QUIC) 透明转发、节点评分 QUIC 探测，并新增 ARM64 `kprobe`/`kretprobe` 内核 UDP 接收跟踪回退（解决 GKI 6.1+ / 6.6+ `fentry` `ENOTSUPP` 问题） |
| **数据路径实时健康校验** | 通过 `BPF_PROG_QUERY` 与 Netlink `cls_bpf` 实时核查 cgroup、`daens` `sk_lookup` 与 TC 链，Doona 面板数据路径完整显示 **`活动 / 已附加 / 健康`** |
| **与官方原版 BoxProxy 完美共存** | 独立包名 **`com.boxproxy.honk`**（桌面名称 **`BoxProxy Honk`**，快捷磁贴 **`Box Honk`**，独立自启脚本 `/data/adb/service.d/boxproxy-honk.sh` 与独立沙盒目录 `/data/user/0/com.boxproxy.honk/files/box`），**无需卸载官方原版 `com.boxproxy.box`** |
| **双面板无缝协同** | 同时启用本机 `Clash API`（`127.0.0.1:9090`，驱动 BoxProxy 原生「面板」页）与内嵌 **Doona 原生面板**（`0ed10b6`，`http://127.0.0.1:9527/ui/`，BoxProxy 首页点击「WebUI」免密直连；支持配置读写、`--store db` 修订历史、代理链与组出口节点编辑、全部 8 项内存与独立 cgroup 遥测指标、地理数据更新及 **核心在线检测更新与热重启**） |
| **双端核心在线更新** | 内核 Native API `/api/v1/x-honk/core/update`（Doona **设置 → 核心更新**）与 `BoxProxy Honk` **工具 → 更新核心** 均原生对接 `Wcoom/honk-android` GitHub Releases，支持 SHA-256 与 ELF64 校验后原子替换并分离自重启 |
| **DNS 双栈容错与规范兼容** | 修复 DNS 响应 QNAME 大小写敏感校验缺陷（RFC 1035/4343）；新增 Bootstrap UDP 1s 自动重传与 `fallback_resolver` 兜底，消除 Android 无 `/etc/resolv.conf` 切网失联 |
| **USB 共享网络免劫持** | 内核 `wan_egress_l3` 自动放行 `rndis0`/`ncm0`/`usb0` 转发包，默认配置放行 `dnsmasq`，开启 USB 热点不劫持宿主电脑流量 |

---

## 快速安装（正式版 Release）

请前往 **[GitHub Releases](https://github.com/Wcoom/honk-android/releases/latest)** 下载正式版预编译成品：

- **人类零基础图文级分步教程**：请阅读 **[`GUIDE.md`](./GUIDE.md)**
- **AI 助手全自动部署 Skill**：请将 **[`skill/honk-android/SKILL.md`](./skill/honk-android/SKILL.md)** 提供给 Antigravity / Claude Code / Cursor / Codex / Gemini CLI，即可自动完成环境体检、配置迁移、推送安装与全链路验证。

### Release 资产说明

| 文件 | 说明 |
| --- | --- |
| `honk-android-release-v1.0.8-arm64.tar.gz` | 完整正式版一体包（含二进制、共存版 APK、配置模板、指南、AI Skill、补丁与校验和） |
| `honk-android-arm64` | 静态链接 `aarch64-unknown-linux-musl` 的 `honk-core`（支持 GKI 6.1+ / 6.6+，合并上游 `4ce5272f`，含全策略组空闲探活暂停、Hysteria2 端口跳跃别名合并、独立 cgroup 内存遥测、核心在线更新与 Doona `0ed10b6` 前端） |
| `boxctl-android-arm64` | 适配 `honk` 核心、`ebpf` 模式、`boot` 自启、独立 `/honk` cgroup 与 `com.boxproxy.honk` 共存目录的静态 `boxctl` |
| `BoxProxy-honk-arm64.apk` | 基于官方新版 BoxProxy (`26.11.23.aa98568` / `v214`) 适配，独立包名 **`com.boxproxy.honk`**（`BoxProxy Honk`），内置 `Honk` 核心更新源并完整对接独立开机自启脚本 `/data/adb/service.d/boxproxy-honk.sh`，可与官方原版 BoxProxy 直接共存安装 |
| `config.example.dae` | 脱敏后的 Android 全功能 7 源融合分流配置模板（含去广告、WebRTC 防泄漏、大陆 App/域名/IPv4+IPv6 直连及 `Wcoom/honk-rules-dat` 云端自动拉取） |
| `SHA256SUMS.txt` | 所有发布产物的 SHA-256 校验值 |

---

## 从源码交叉编译

### 环境依赖（Linux x86_64 宿主）

- Rust 工具链 `1.100.0` + target `aarch64-unknown-linux-musl`（eBPF 重编需 `nightly` + `rust-src` + `bpf-linker`）
- `zig`（默认读取 `/usr/lib/zig0.16/lib` 头文件，可通过环境变量 `ZIG_LIB` 覆盖）
- `cmake`、`llvm-ar-18`、`curl`

### 1. 一键编译 `honk-core`（含内嵌 Doona 原生面板）

```bash
# 自动编译内置 webui/（标准版 + Lite）并静态链接 honk-core；如需改用远端预构建资源可设 FETCH_DOONA=1
./build.sh
# 产物：src/target/aarch64-unknown-linux-musl/release/honk-core
```

### 2. 一键编译 `boxctl`

```bash
./build-boxctl.sh
# 产物：boxproxy/boxctl/target/aarch64-unknown-linux-musl/release/boxctl
```

### 3. 重新编译 eBPF 对象（仅修改 `src/crates/honk-ebpf` 时需要）

```bash
cd src/crates/honk-ebpf
env -u RUSTFLAGS CARGO_TARGET_DIR="$PWD/target" \
  cargo +nightly build --release -Zbuild-std=core --target bpfel-unknown-none
cp target/bpfel-unknown-none/release/honk-ebpf ../../target/honk-core.o
```

---

## 仓库目录结构

```text
honk-android/
├── src/                  honk 完整源码（同步上游 main 4ce5272f）+ Android 全功能适配
├── webui/                嵌入式 Doona 原生前端（同步上游 Zakkaus/doona 0ed10b6 + 内存/更新扩展）
├── boxproxy/             boxproxy / boxctl 源码（新增 honk 核心与 ebpf 模式生命周期管理）
├── toolchain/            zig + rust-lld 的 aarch64-linux-musl 交叉编译包装器
├── lib/                  静态链接所需的 aarch64 libc++ / libc++abi / compiler_rt 归档
├── config/               脱敏后的 Android 推荐配置模板（config.example.dae）
├── patches/              相对上游基线导出的独立补丁文件（honk-android.patch / boxctl-honk.patch）
├── skill/honk-android/   标准化 AI Agent 自动部署与排障 Skill（含一键部署脚本）
├── build.sh              一键交叉编译 honk-core
├── build-boxctl.sh       一键交叉编译 boxctl
├── vtest.sh              手机端快速验证 eBPF Verifier 辅助脚本
├── stackmap.py           静态分析 eBPF 调用链栈深与指令数工具
├── setup-cgroup.sh       手工升级根 cgroup legacy BPF 钩子备用脚本（已内置进 honk-core）
├── GUIDE.md              人类使用与部署完全指南
├── ANDROID-DATAPATH.md   Android GKI 6.6 eBPF Verifier 卡点根因与解法分析
├── PATCHES.md            对上游 honk 的改动技术清单
├── PROGRESS.md           完整研发、压测与验证记录
└── CHANGELOG.md          版本发布日志
```

---

## 致谢与开源协议

- 上游透明代理核心：[daeuniverse/honk](https://github.com/daeuniverse/honk)
- 原生 Web 控制面板：[Zakkaus/doona](https://github.com/Zakkaus/doona)
- Android 客户端与控制框架：[boxproxy/boxproxy](https://github.com/boxproxy/boxproxy)

本项目遵循上游 **[GNU General Public License v3.0 (GPL-3.0)](./LICENSE)** 开源协议。
