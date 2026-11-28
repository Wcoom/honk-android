# Architecture Decision Records (ADRs)

## ADR-001: Git 迁移分支与黄金基准策略
- **Status**: accepted (2026-11-28)
- **Context**: `rust-anything` 的 `record`、`diff`、`gate` 与 `app replay` 需要原版 `webui/`、`boxproxy/` 与仓库脚本保留在仓库中作为行为黄金基准，直至 `app lift` 完成。此前工作区存在未提交的实验性删除。
- **Decision**: 在新分支 `feat/pure-rust-rewrite` 上从 `HEAD` (`e58099d`, `v1.0.10`) 完整恢复原始 `webui/`、`boxproxy/`、构建/校验脚本与补丁；严禁修改用户自有未提交文件（`.gitignore`、`build-cf-opt.sh`、`cf-opt/`）。所有迁移文档、研究笔记、用例与 Rust 迁移产物统一存放在 `rust/` 及对应子目录 `rust/` 下；原版代码在 `app lift` 及真机对比验收通过前作为参照保留。
- **Consequences**: 任何波次都可随时与原版实现做自动化 `diff`/`app replay` 与真机 A/B 对比，并保证主力机可随时回退至 `v1.0.10`。

## ADR-002: `boxproxy/boxbpf`（C → Rust）纯 Rust 整应用迁移
- **Status**: accepted (2026-11-28)
- **Context**: `boxproxy/boxbpf` 共 5 个 C 文件（951 LOC：`compat.h`、`common.h`、`config.c`、`loader.c`、`main.c`），为 `boxctl` 提供 `xt_bpf` 规则匹配器的 BPF map 创建/更新/pin 与 `BPF_PROG_TYPE_SOCKET_FILTER` 指令生成加载；`rust-anything analyze --whole-app` 显示 `ready: 2, caution: 3, blocked: 0, keep: 0`。`boxctl/src/rules/cnip.rs` 强依赖其命令行参数、退出码（`0/1/2/3`）、stdout 格式与 stderr 错误关键词。
- **Decision**: 使用 `rust-anything` 整应用流程（`app record` → `app front-door` → 单元 `bridge`/`record`/`diff`/`gate` → `app wire` → `app lift`）将 `boxbpf` 完整迁移为纯 Rust 静态二进制，直接通过 `libc::syscall(SYS_bpf, ...)` 与等价 `#[repr(C)]` `bpf_insn`/`bpf_attr` 结构保持与 Android GKI 6.1/6.6 内核 ABI 字节级兼容，交叉编译至 `aarch64-unknown-linux-musl` 并在 OnePlus 13 实测验证。
- **Consequences**: 消除仓库内全部 C 源码及 C 编译器依赖（`boxproxy/boxbpf`），同时 100% 保持与 `boxctl` 的 CLI、JSON 配置、退出码和 stderr 契约。

## ADR-003: 仓库 Shell/Python 构建与运维脚本迁移至纯 Rust `xtask`
- **Status**: accepted (2026-11-28)
- **Context**: 仓库根目录及 `skill/` 存在 7 个非 Rust 脚本：`build.sh`、`build-boxctl.sh`、`setup-cgroup.sh`、`measure.sh`、`vtest.sh`、`stackmap.py`、`skill/honk-android/scripts/deploy_honk_android.sh`。
- **Decision**: 在 `rust/xtask/` 建立纯 Rust 工作区工具，以子命令等价替代上述脚本（`build-core`、`build-boxctl`、`build-boxbpf`、`stackmap`、`setup-cgroup`、`vtest`、`measure`、`deploy`），并以 `stackmap.py` 等原脚本输出做逐行行为一致性校验。
- **Consequences**: 构建、eBPF 栈深分析、ADB 部署与自测全部由纯 Rust 驱动，不再依赖 bash/python 运行时（手机端 `service.d` 开机自启脚本仍按 Android KernelSU/Magisk `service.d` 规范生成或由 Rust CLI 执行）。

## ADR-004: Doona 面板（`/ui/` 与 `/ui/lite/`）纯 Rust WASM 架构
- **Status**: accepted (2026-11-28)
- **Context**: `webui/` 共 451 源文件（82,229 LOC，98 单元），含标准版 `/ui/` 与 Lite 版 `/ui/lite/`。`rust-anything analyze --whole-app` 识别出 32 个可移植 TS 逻辑单元（`src/dae/`、`src/api/`、`apps/lite/src/*.ts` 等）与 64 个 React/TSX `keep` UI 单元。用户确认采用方案 A（纯 Rust WASM 全量重写，同时保留标准版与 Lite 版）。
- **Decision**:
  1. 先使用 `rust-anything`（`bridge` → `record --from-tests` → `port.rs` → `build` → `diff` N/N → `gate`）将全部核心逻辑单元迁入经验证的纯 Rust crate；
  2. 使用 **Leptos CSR（`wasm32-unknown-unknown`）** 重写 `/ui/` 与 `/ui/lite/` 全部视图层，复用原版 CSS/配色体系/i18n 文案与 DOM 交互契约（含「节点策略选择」、8 项内存指标、核心检测更新）；
  3. 以原版 Playwright E2E 用例与 OnePlus 13 真机 `/api/v1/*` 实测对照证明行为完全一致。
- **Consequences**: 最终产物不含 TypeScript/React 源码，所有浏览器端逻辑与视图均由 Rust 编译为 WASM 并继续内嵌至 `honk-core`。

## ADR-005: BoxProxy Honk Android 客户端纯 Rust 架构与全模块范围
- **Status**: accepted (2026-11-28)
- **Context**: 逆向 `com.boxproxy.honk` v214 APK（9.4 MB，8,752 个 R8 混淆类，1,193 条本地化文案覆盖 58 个功能模块前缀，20 张 `box.db` SQLite 表，9 份 `shared_prefs`）发现，其重度依赖 Android 框架组件（`TileService`、`ForegroundService`、`WebView`、`PackageManager`、`WifiManager`、`CoreImportActivity`）并支持多核心管理与全套工具模块。用户确认 Q3=A（Rust 100% 核心逻辑 + 极薄 Android DEX 系统桥接壳）与 Q4=B（1:1 完整复刻全部 58 个模块，含多核与主题）。
- **Decision**:
  1. 在 Wave 1 使用 `jadx` 完整反编译 `com.boxproxy.honk` 并对照 `boxctl` 源码与真机 `box.db`/`shared_prefs`，形成 58 模块完整行为规格（`rust/research/boxproxy-honk-spec.md`）；
  2. 将全部业务逻辑、SQLite `box.db` 读写、多核心配置/覆写、`boxctl`/`boxbpf` 控制、Honk/Clash API 客户端、规则查询、应用分流、绕过规则、CNIP、网络匹配引擎、核心更新、订阅/Sub-Store、脚本管理、备份恢复、日志与状态机全部用纯 Rust（`cdylib` + `jni`）实现，核心纯逻辑单元通过 `rust-anything` 录制回放校验；
  3. 仅保留极薄的 Android DEX 胶水层承载 JVM 强制接口（`TileService`、`ForegroundService`、`WebView`、`PackageManager`、`WifiManager`/IME），所有状态决策下沉至 Rust。
- **Consequences**: 兼顾「业务与控制面 100% 纯 Rust」与「Android 原生能力与原版 BoxProxy Honk 1:1 完整一致（含磁贴、常驻通知、内置 Doona、分应用、多核、主题）」。

## ADR-006: 真机 A/B 并行调试与切流策略
- **Status**: accepted (2026-11-28)
- **Context**: OnePlus 13 (`5d6d4090`) 为主力设备，当前运行原版 `com.boxproxy.honk` + `honk-core v1.0.10`。
- **Decision**: 开发与验证期间保持原版 `com.boxproxy.honk` 不受破坏作为对照组，新版纯 Rust 客户端先以独立包名 `io.honk.android` 并排安装测试，逐项核对 `box.db`、`boxctl`/`boxbpf`、eBPF、路由分流、WebUI、`logcat` 与 `honk.log`（要求零 `WARN`/`ERROR` 与零行为差异），最终是否替换 `com.boxproxy.honk` 由用户确认后执行。
- **Consequences**: 零断网风险，可随时进行同机 A/B 对比。

## ADR-007: 等价判定门槛与 4 波次检查点
- **Status**: accepted (2026-11-28)
- **Context**: 整仓库迁移规模庞大，需要可度量的完成标准与跨会话断点恢复能力。
- **Decision**:
  - **Wave 1**：ADRs/Map/Glossary + `com.boxproxy.honk` 全量逆向规格 + `boxbpf`（C→Rust `rust-anything gate` & `app lift` + 真机验证）+ `xtask` 纯 Rust 替代脚本。
  - **Wave 2**：Doona 核心逻辑单元 `rust-anything gate`（`diff` 100%）+ 纯 Rust WASM（Leptos）重写 `/ui/` 与 `/ui/lite/` + Playwright E2E 与真机验证。
  - **Wave 3**：按逆向规格以纯 Rust 重写 BoxProxy Honk 全部 58 模块客户端（Rust JNI 核心 + 极薄 Android DEX 桥接）+ 行为录制回放。
  - **Wave 4**：OnePlus 13 实体机全量 A/B 对比、日志与 Bug 清零、行为一致性证明报告，由用户决策切流与发布。
  - 每个波次结束必须更新 `rust/MAP.md`、`rust/HANDOFF.md`、执行 `rust-anything map --write` 并向用户汇报等待确认。
- **Consequences**: 每一步有机器可验证证据，绝不以“编译通过”替代行为等价证明。

## ADR-008: Wave 3 BoxProxy Honk 客户端 UI 渲染层选型（Q7=A）
- **Status**: accepted (2026-11-28)
- **Context**: Wave 1 逆向发现 `com.boxproxy.honk` 全 58 模块深度依赖 Material 3、Miuix、OS4 液态玻璃模糊、Sora TextMate 编辑器与 Android 中文 IME/无障碍体验。
- **Decision**: 采用 **100% Rust MVI 业务与状态引擎（`libboxproxy_app.so`，~8 个 JNI 入口，覆盖全部状态机、SQLite、API、Root/`boxctl`、更新、备份、脚本与网络匹配引擎）+ 超薄 Jetpack Compose DEX 渲染桥接层**（不含业务逻辑，仅负责声明式渲染与系统框架桥接）。
- **Consequences**: 兼顾业务与控制面 100% 纯 Rust 可验证性，以及视觉/交互/中文输入法/系统磁贴/通知与原版 BoxProxy Honk 的 1:1 完全一致。

## ADR-009: `box.db` Schema v15 统一与 Honk Native API 优先策略（Q8=是）
- **Status**: accepted (2026-11-28)
- **Context**: Wave 1 逆向发现 APK 已使用 `box.db` schema v15（21 表，含 `network_rule_*`、`shared_network_*`、`blocked_interfaces`、`meow` 核心），而仓库 `boxctl` 仍停留在 schema v3（16 表）且 `clashapi-history.sock` 为空桩；同时 `honk-core` 已原生提供 35 个 `/api/v1/*` 端点。
- **Decision**: 在 Wave 3 抽离共享 Rust crate `boxproxy-schema` 将 App 与 `boxctl` 同步升级至 `box.db` schema v15；在 `boxproxy-api` 中对 `honk` 核心优先使用 Honk Native API（`/api/v1/*` 与 `/api/v1/x-honk/core/update`），对 `mihomo`/`meow`/`sing-box` 保留完整 Clash API 支持。
- **Consequences**: 彻底消除 App 与 `boxctl` 的数据库 schema 漂移，并解锁 `honk` 原生历史流量、路由追踪与核心自更新能力。
