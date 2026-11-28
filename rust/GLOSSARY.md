# Domain Glossary (`honk-android` Pure Rust Port)

| 原始术语 / 组件 | 含义与行为契约 | Rust 目标命名（Crate / Module / Type） |
| --- | --- | --- |
| `honk-core` | 透明代理核心（eBPF + DNS + 路由 + Clash/Native API + 内嵌 WebUI） | `src/crates/honk-core`（已纯 Rust） |
| `boxctl` | BoxProxy 核心生命周期、规则、CNIP、网络监控与 `box.db` 控制 CLI | `boxproxy/boxctl`（已纯 Rust） |
| `boxbpf` | `xt_bpf` eBPF 规则匹配器加载程序（C，951 LOC，5 文件） | `boxproxy/boxbpf/rust/app`（crate `boxbpf`，纯 Rust 二进制） |
| `STATUS_OK / UNSUPPORTED / CONFIG_ERROR / UPDATE_ERROR` | `boxbpf` 进程退出码 `0 / 1 / 2 / 3` | `StatusCode::{Ok, Unsupported, ConfigError, UpdateError}` |
| `lpm4_key / lpm6_key` | Linux `BPF_MAP_TYPE_LPM_TRIE` 键（`prefixlen: u32` + `addr: [u8; 4/16]`） | `#[repr(C)] struct Lpm4Key / Lpm6Key` |
| `READ_DIRECT / READ_HELPER / READ_LDABS` | `boxbpf` 读取 `sk_buff` 目的 IP 的 3 级回退模式 | `enum ReadMode { Direct, Helper, LdAbs }` |
| `Doona` (`/ui/`) | 内嵌主管理面板（React/TS -> Rust WASM） | `webui/rust/*`（逻辑 crate）+ `rust/doona-wasm`（Leptos CSR） |
| `Doona Lite` (`/ui/lite/`) | 轻量管理面板（React/TS -> Rust WASM） | `webui/rust/*` + `rust/doona-lite-wasm` |
| `节点策略选择` (`policyTarget`) | 规则模式下通过中心选择器（如 `总模式`）切换代理策略组且保持规则分流 | `policy_target::find_policy_hub` / `resolve_policy_state` |
| `出站模式` (`outboundMode`) | 规则 / 直连 / 全局 三态模式（`# doona: outbound mode` 标记） | `outbound_mode::{OutboundMode, read_rule_fallback, write_rule_fallback}` |
| `BoxProxy Honk` (`com.boxproxy.honk`) | Android 管理端 App（58 模块，20 张 `box.db` 表，9 份 `shared_prefs`） | `rust/boxproxy-app`（Rust 核心 + JNI `cdylib`）+ 极薄 Android DEX 桥接 |
| `box.db` | BoxProxy 与 `boxctl` 共享的 SQLite 运行时配置数据库（20 表） | `boxproxy_store::BoxDb`（schema 100% 兼容 `boxctl/src/db/schema.rs`） |
| `ZashboardKsuBridge` | Android WebView 注入给 WebUI 的 JS Bridge | `web_bridge`（JNI -> Rust 控制器） |
| `BoxServiceTileService` | Android 快速设置磁贴服务（切换核心启停） | DEX `BoxServiceTileService` -> JNI `box_tile_*` |
| `BoxPersistentNotificationService` | Android 常驻前台通知服务（展示状态、快捷重载/停止） | DEX `BoxPersistentNotificationService` -> JNI `box_notif_*` |
| `stackmap.py` | 解析 `honk-ebpf` ELF 符号与反汇编以统计每条链最大栈深的脚本 | `rust/xtask` 子命令 `stackmap` |
| `build.sh` / `build-boxctl.sh` / `setup-cgroup.sh` / `vtest.sh` / `measure.sh` / `deploy_honk_android.sh` | 仓库 Shell 构建、eBPF 验证、cgroup 升级与 ADB 部署脚本 | `rust/xtask` 对应子命令 |
