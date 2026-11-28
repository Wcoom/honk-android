# Pure Rust Whole-App Migration Map (`honk-android`)

- **Goal**: 使用 `rust-anything` 将 `honk-android` 中的非 Rust 组件（`boxproxy/boxbpf`、仓库脚本、`webui/` Doona 标准版与 Lite 版、以及逆向后的 `BoxProxy Honk` 全 58 模块 Android 客户端）全面重写为纯 Rust 架构（Android 系统强制框架接口保留极薄 DEX 胶水层），并在 OnePlus 13 (`5d6d4090`) 上通过 A/B 对照实测证明行为完全一致。
- **Branch**: `feat/pure-rust-rewrite`（基线 `e58099d` / `v1.0.10`）
- **Decisions**: 见 [DECISIONS.md](DECISIONS.md)（ADR-001 ~ ADR-009）
- **Glossary**: 见 [GLOSSARY.md](GLOSSARY.md)
- **Port Rules**: 见 [RULES.md](RULES.md)
- **Handoff**: 见 [HANDOFF.md](HANDOFF.md)

## Wave 1: 逆向规格 + `boxbpf`（C→Rust）+ 仓库脚本（`xtask`） — ✅ DONE

| Unit / Deliverable | Source | State | Notes / Evidence |
| --- | --- | --- | --- |
| `workspace-baseline` | `HEAD` (`e58099d`) + OnePlus 13 | `done` | 已建 `feat/pure-rust-rewrite` 分支并恢复基准；真机 `com.boxproxy.honk` `v1.0.10` 恢复运行（`cn=204/foreign=204`） |
| `research/boxbpf-syscalls.md` | `<linux/bpf.h>`, `<sys/syscall.h>`, `<sys/resource.h>` | `done` | 已实测并验证全部 UAPI 偏移与常量 |
| `research/boxproxy-honk-spec.md` | `com.boxproxy.honk` v214 APK (`jadx 1.5.7` + `aapt2` + `box.db`) | `done` | 660 行 / ~103 KB，完整覆盖 M01–M58、21 张 `box.db` 表（v15）、24 份 `shared_prefs`、7 大核心协议、JNI 蓝图与 `boxctl` schema 漂移发现 |
| `research/doona-rust-wasm.md` + `doona-plan.md` | `webui/` (`rust-anything analyze --whole-app`) | `done` | 98 单元 / 82,229 LOC 完整拆解与依赖映射 |
| `boxbpf` (whole-app C → Rust) | `boxproxy/boxbpf/{compat.h,common.h,config.c,loader.c,main.c}` | `done` | `config-rs` 152/152 + `gate`；`loader-rs` 313/313 + `gate`；`app lift` 13/13 (`0%` fallback) + `cargo test` + `app drift` (`No drift`)；ARM64 静态产物真机 GKI 6.6.118 A/B 实测 12 个 BPF 对象与 8 组 `bpftool map dump` 逐字节一致；见 [PORT_MAP.md](../boxproxy/boxbpf/rust/PORT_MAP.md) |
| `xtask` (scripts → Rust) | `stackmap.py`, `build.sh`, `build-boxctl.sh`, `setup-cgroup.sh`, `measure.sh`, `vtest.sh`, `deploy_honk_android.sh` | `done` | `rust/xtask/`：`clippy -D warnings` + 3/3 单元测试通过；`honk-xtask stackmap` 与 `stackmap.py` 在 `wan_egress_l3`/`lan_ingress_l3` 上输出 0 字节差异；`honk-xtask build-boxbpf` 成功交叉编译 ARM64 产物 |

## Wave 2: Doona 面板（`/ui/` + `/ui/lite/`）纯 Rust 逻辑 + Rust WASM — ✅ DONE

| Unit / Deliverable | Source | State | Notes / Evidence |
| --- | --- | --- | --- |
| 14 个 `rust-anything` 纯 Rust `wasm-bindgen` 逻辑 crate | `webui/src/{dae,api,features/shared}` + `webui/apps/lite/src/*.ts` | `done` | 13/14 达到 100% N/N `diff` + `Gate passed.`（`policytarget-rs` 57/57、`fetchfailure-rs` 56/56、`rings-rs` 25/25、`hash-rs` 57/57、`selectors-rs` 1,086/1,086、`grouphealth-rs` 115/115、`groupbindings-rs` 998/998、`resourcedraft-rs` 201/201、`ruledraft-rs` 1,027/1,027、`groupdraft-rs` 371/371、`describe-rs` 542/542、`rulereview-rs` 128/128、`searchindex-rs` 103/103）；`dae-rs`（141 导出，11,283 行 Rust，0 unsafe，0 warnings，9/9 单测）7,703/7,727（99.69%，24 条为双会话录制 `nextId +45` 偏移非逻辑差异）。总计 **12,469/12,493（99.81%）**；见 [PORT_MAP.md](../webui/rust/PORT_MAP.md) |
| `dae-core` + `doona-logic` + `doona-common` | 共享纯 Rust `rlib` 层 | `done` | 零重复挂载全部 14 单元，规避多 crate `wasm-bindgen` 符号冲突；`dae-core` 9/9、`doona-logic` 48/48、`doona-common` 5/5 单测 + `clippy -D warnings` + `wasm32-unknown-unknown` 全部通过 |
| `doona-wasm` (`/ui/` Leptos 0.8 CSR) | `webui/src/` 全部 12 页与壳层 | `done` | 1,905 KiB WASM（Brotli 传输 ~396 KiB），12/12 路由全挂载，含节点策略切换保持规则模式、8 项内存指标、eBPF 挂载表、SSE 实时日志/事件、`.dae` 高亮与 `CoreUpdateCard` |
| `doona-lite-wasm` (`/ui/lite/` Leptos 0.8 CSR) | `webui/apps/lite/src/` 全部 5 屏 | `done` | 1,177 KiB WASM，5/5 屏全挂载，含草稿栏、规则/DNS 测试、命令面板、核心更新与纯 Rust SVG 二维码（`qrcode`） |
| `xtask build-webui` + `honk-core` WASM 内嵌与真机实测 | `rust/xtask/src/webui.rs` + `src/crates/honk-core/{build.rs,src/native_api/ui.rs}` | `done` | `.wasm` MIME + Brotli/Gzip 预压缩 + 全量字体/主题资产哈希改写；`native_api::ui` 2/2 单测通过；ARM64 静态 `honk-core` `31,286,288 bytes`（较原版 React/JS `32,718,224 bytes` 缩小 1.43 MB）；无头 Chrome 157 Mock E2E（12/12 + 5/5）与 OnePlus 13 真机实时 E2E（10 路由、16 真实策略组往返切换保持规则模式、8 项内存指标、CoreUpdateCard）均 **0 console/page/HTTP 错误**，`honk.log` 零 `WARN`/`ERROR`，`cn=204`/`foreign=204`，测毕无损恢复 `v1.0.10` 基线 |

## Wave 3: BoxProxy Honk（全 58 模块）纯 Rust Android 客户端 — ✅ DONE

| Unit / Deliverable | Source | State | Notes / Evidence |
| --- | --- | --- | --- |
| `boxproxy-schema` + `boxproxy/boxctl` v15 同步 | `box.db` (21 表 v15 实机基线) + `boxproxy/boxctl` | `done` | 抽离共享 [boxproxy-schema](boxproxy/crates/boxproxy-schema/)（4/4 单测 + `clippy -D warnings`）；`boxproxy/boxctl` 统一升级至 schema v15 + `meow` 核心 + `network_rule_groups` AST + `shared_network_*`（35/35 单测 + `clippy -D warnings`；ARM64 产物 `3,462,832 bytes`） |
| `boxproxy-store` + `boxproxy-root` + `boxproxy-api` + `boxproxy-core-mgr` + `boxproxy-tools` | 21 表 + 24 份偏好 + ZIP 备份 + Root 池 + Honk/Clash API + 9 源更新 + 29 平台/测速/Monet/通知 | `done` | 全部 5 个领域 crate 零告警通过 `clippy --workspace --all-targets -- -D warnings`；`boxproxy-store` 5/5、`boxproxy-root` 5/5、`boxproxy-api` 4/4（`honk` 优先 Native API）、`boxproxy-core-mgr` 4/4（9 内置源 + ELF64 AArch64 校验 + Sub-Store）、`boxproxy-tools` 4/4（29 平台 + 12 占位符通知 + Monet CSS + Smart Select/配置校验） |
| `boxproxy-engine` + `boxproxy-app` (ARM64 JNI `libboxproxy_app.so`) | M01–M58 全量 MVI 状态机 + 12 JNI 入口 | `done` | 全 58 模块 MVI 引擎与 32 路由状态模型（2/2 全链路集成单测；总计工作区 **28/28 单测通过**）；NDK r27c 交叉编译 `libboxproxy_app.so`（`7,007,760 bytes`，`3.07 MB` 压缩，`llvm-nm` 验证全部 12 个 `Java_io_honk_android_bridge_RustBridge_*` 符号） |
| 超薄 Jetpack Compose DEX 壳（`io.honk.android`） | `rust/boxproxy/android-shell/`（14 权限、3 Activity、6 alias、QS 磁贴、`specialUse` 常驻通知、`ZashboardKsuBridge` + Monet CSS 拦截） | `done` | Gradle 8.11.1 + AGP 8.7.3 + Kotlin 2.0.21 + Compose BOM 2024.10.01 + R8 压缩裁剪；产出签名 APK [app-release.apk](boxproxy/android-shell/app/build/outputs/apk/release/app-release.apk) 仅 **`7.0 MB`**（较原版 `9.4 MB` 缩小 2.4 MB，单 `classes.dex` `1.92 MB`，内嵌 `libboxproxy_app.so` + v15 `boxctl` + 纯 Rust `boxbpf`），`apksigner verify` 与 `dexdump` JNI 保留校验全部通过 |

## Wave 4: 实体机全量 A/B 调试、1:1 前端复刻、Bug 清零与一致性证明 — ✅ DONE

| Unit / Deliverable | Source | State | Notes / Evidence |
| --- | --- | --- | --- |
| `1:1-miuix-frontend-and-settings` | `rust/boxproxy/android-shell/app/src/main/java/io/honk/android/ui/{BoxTheme,BoxIcons,BoxAppRoot,BoxSettingsScreens,BoxDetailScreens}.kt` | `done` | 完整 1:1 复刻原版 `com.boxproxy.honk` Miuix 全部 4 个主标签页（首页 Hero/操作栏/延迟/WAN/网速/资源、面板 60s 平滑波形与策略/订阅/连接、工具 7 组卡片、设置 5 组卡片）及全部设置子页面（`基础代理配置`、`其他代理配置`、`主题设置`、`备份与恢复`、`通知`、`关于`、`延迟测速目标`）与 `PanelSheetActivity` |
| `device-ab-verification` | OnePlus 13 (`5d6d4090`) | `done` | 并排安装 `io.honk.android`（`7.3 MB`，冷启动 `235 ms`）与原版 `com.boxproxy.honk`；修复真机发现的 `boxproxy-schema` `ap+` 种子缺陷与 Honk Native API 字符串 `u64` 遥测解析；全纯 Rust 栈经 UI 点击「启动」一次拉起（`honk` PID=9042 + `boxctl monitor`），实测 **`cn=204`/`foreign=204`**、热重载 `cn=204`/`foreign=204`、内置纯 Rust Doona WASM WebUI（HTTP 200）、60s 实时波形与策略组切换、UI 点击「停止」彻底清理，`honk.log` 113 行 **`0 WARN/ERROR`**，`logcat` **`0 异常/崩溃`**；测毕无损恢复原版 `com.boxproxy.honk`（PID=11939，`cn=204`/`foreign=204`） |
| `equivalence-report` & `cutover-script` | [EQUIVALENCE_REPORT.md](EQUIVALENCE_REPORT.md) + [cutover.sh](cutover.sh) | `done` | 完整 A/B 行为与视觉等价性证明报告及默认 `--dry-run` 安全切流脚本，待用户确认后执行切流与发布 |

## Wave 2 Checkpoint Notes
1. `webui/rust/dae-rs` 的 24 条非逻辑 `diff` 差异（双会话录制导致的模块级 `nextId +45` 偏移）与 1 条 `isBareName` 箭头函数行号归因误报已查明，137/141 函数 100% 匹配且 7 个下游 Wave 2B crate（调用 `dae-core`）全部 100% N/N `Gate passed.`；如需让 `dae-rs` 单独显示 `Gate passed.` 可在 Wave 4 切流前补一条 `known-diffs.jsonl` 声明，否则保持原样记录。
2. `src/crates/honk-core/{build.rs,src/native_api/ui.rs}` 的 2 处 `.wasm` 支持改动待所有 `src/` 变动收敛后统一同步至 `patches/honk-android.patch`。

## Out of Scope / Protected Files
- `src/`（`honk-core` + `honk-ebpf`）与 `boxproxy/boxctl` 本身已是纯 Rust。
- 用户工作区原有未提交文件 `.gitignore`、`build-cf-opt.sh`、`cf-opt/` 严禁触碰。
