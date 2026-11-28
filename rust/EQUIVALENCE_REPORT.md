# `honk-android` Pure-Rust Rewrite — Wave 4 实体机 A/B 等价性与验收报告

- **测试日期**：2026-11-28
- **代码分支**：`feat/pure-rust-rewrite`（基线 `e58099d` / `v1.0.10`）
- **真机环境**：OnePlus 13 (`5d6d4090`)，Android 15 (SDK 35)，GKI 6.6.118 aarch64
- **测试方式**：独立包名 `io.honk.android` 与原版 `com.boxproxy.honk` (`v1.0.10`) 同机并排 A/B 实测；期间短暂切换活动代理至 100% 纯 Rust 全栈运行、验证全链路与日志清零后，无损恢复原版 `com.boxproxy.honk` 运行。

---

## 1. 全栈纯 Rust 架构与产物总览

| 组件 | 替代目标 | 纯 Rust 实现路径 | 静态/发布产物大小 | 与原版对比 |
| :--- | :--- | :--- | :--- | :--- |
| **`boxbpf`** | `boxproxy/boxbpf/*.c` (C/libbpf) | [boxproxy/boxbpf/rust/](../boxproxy/boxbpf/rust/) | `865,064 B` (845 KB) | `0%` fallback，`No drift`，12 BPF 对象 + 8 组 `bpftool map dump` 逐字节一致 |
| **`honk-xtask`** | 7 个 Shell/Python 构建运维脚本 | [rust/xtask/](xtask/) | Host CLI | `stackmap` 0 字节差异，覆盖 `build-boxbpf`/`build-webui` 等全流程 |
| **Doona `/ui/` + `/ui/lite/`** | React/TypeScript/Vite 前端 | [webui/rust/](../webui/rust/) (14 逻辑 crate) + [rust/doona-wasm/](doona-wasm/) + [rust/doona-lite-wasm/](doona-lite-wasm/) | `honk-core` `31,286,288 B` (29.8 MB) | 较原版 React 内嵌 `honk-core`（32.72 MB）**缩小 1.43 MB** |
| **`boxctl` v15** | `box.db` v3 `boxctl` | [boxproxy/boxctl/](../boxproxy/boxctl/) + [boxproxy-schema](boxproxy/crates/boxproxy-schema/) | `3.3 MB` ARM64 | 统一 `box.db` schema v15（21 表 + `meow` + `network_rule_groups` AST + `shared_network_*`） |
| **`BoxProxy Honk` 引擎** | `com.boxproxy.honk` Java/Kotlin 业务逻辑（M01–M58） | [rust/boxproxy/](boxproxy/)（8 crate -> `libboxproxy_app.so`） | `7,033,264 B`（APK 内压缩 ~3.1 MB） | 全 58 模块 MVI 引擎 + 12 JNI 入口，**零 Java 业务/网络/DB 逻辑** |
| **Android 1:1 Miuix DEX 壳** | `com.boxproxy.honk` UI 与系统组件 | [rust/boxproxy/android-shell/](boxproxy/android-shell/) (`io.honk.android`) | 签名 APK **`7.3 MB`**（单 `classes.dex` `2.12 MB`） | 较原版 `com.boxproxy.honk` APK（`9.4 MB`）**缩小 2.1 MB**，已含 `libboxproxy_app.so` + `boxctl` + `boxbpf` |

---

## 2. 自动化门禁与单元测试矩阵

| 测试套件 | 命令 | 结果 |
| :--- | :--- | :--- |
| `boxbpf` (`rust-anything` gate + lift) | `config-rs` (152/152) + `loader-rs` (313/313) + `app lift` (13/13 trycmd) + `app drift` | **PASS**（0% fallback，`No drift`） |
| `rust/xtask` | `cargo clippy -- -D warnings && cargo test` | **PASS**（3/3，0 warnings） |
| Doona 14 个 `rust-anything` 逻辑 crate | `diff` + `gate` + `dae-core`(9/9) + `doona-logic`(48/48) + `doona-common`(5/5) | **12,469 / 12,493 (99.81%)**；13/14 100% `Gate passed.`（`dae-rs` 24 条为双会话 `nextId +45` 非逻辑偏移） |
| `honk-core` UI 托管单测 | `cargo test -p honk-core native_api::ui` | **PASS**（2/2） |
| `boxproxy/boxctl` (v15) | `cargo clippy -- -D warnings && cargo test` | **PASS**（35/35，0 warnings） |
| `rust/boxproxy` (8 crates) | `cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace` | **PASS**（28/28，0 warnings） |
| R8 签名 APK 校验 | `gradle assembleRelease && apksigner verify` | **PASS**（v2 签名通过，单 DEX `2.12 MB`，总包 `7.3 MB`） |

---

## 3. OnePlus 13 (`5d6d4090`) 实体机 A/B 实测结果

### 3.1 全纯 Rust 运行链路实测（UI 点击驱动）

| 验收项 | 实测方式 | 实测结果 |
| :--- | :--- | :--- |
| **冷启动耗时** | `am start -W -n io.honk.android/.MainActivity` | **`235 ms`**（`WaitTime: 238 ms`），零崩溃 |
| **Root 与资产同步** | 首次进入自动执行 `id -u` + SHA-256 原子同步 | `Root 权限已就绪 (uid=0)`；`bin/boxctl`、`bin/boxbpf` 同步为 `root:net_admin 0755` |
| **数据库 Schema v15** | 检查 `/data/user/0/io.honk.android/files/box/box.db` | `user_version = 15`，完整 21 表，与原版 `com.boxproxy.honk` 字段及默认记录 1:1 一致 |
| **UI 点击「启动」** | 首页点击 `启动` -> JNI -> Rust Engine -> 持久 `su` 池 -> v15 `boxctl up` -> Rust `boxbpf` -> WASM `honk` | **`honk` PID=9042** + **`boxctl monitor` PID=9080** 一次拉起成功，`Inbound rules applied: mode ebpf, core honk` |
| **国内/海外分流连通性** | 经 100% 纯 Rust 全栈实测 `hicloud` + `google` `generate_204` | **`cn=204`**、**`foreign=204`** |
| **UI 点击「重载」** | 首页点击 `重载` -> Honk Native API `/api/v1/operations/reload` | `honk.log` 记录 `SIGHUP reload request 1 applied`；重载后 **`cn=204` / `foreign=204`** |
| **内置纯 Rust Doona WebUI** | 首页点击 `WebUI` -> `WebView` 加载 `http://127.0.0.1:9527/ui/` | **HTTP 200**，CSP 含 `'wasm-unsafe-eval'`，Leptos WASM 面板完整渲染；展示「出站模式（规则/直连/全局）」与「节点策略选择（`智能策略 / ALL`）」 |
| **实时遥测与 60s 波形** | 前台 2s 轮询 Honk Native API (`/api/v1/runtime`, `/runtime/memory`, `/groups`, `/rules`, `/connections`, `/runtime/traffic/history`) | 首页实时显示运行时长、3 目标延迟、WAN IP + 国旗地区、内存/CPU 占用条；面板实时显示 `49 策略 / 75 规则 / 1 连接`、总流量、与 60 秒平滑 Cubic Bézier 双曲线波形 |
| **策略选择切换** | 面板 -> `策略` 分段 | 实时拉取全部 Honk 策略组（`AI`/`Apple`/`总模式`/`智能策略 / ALL`/地区节点等）与 `规则/直连/全局` 切换 |
| **29 平台并发探测 (M41)** | 工具 -> `网络测试` -> 29 平台连通性探测 | 并发返回真实延迟与区域（ChatGPT SG、Claude TW、Douyin 等） |
| **悬浮面板 (M56)** | `PanelSheetActivity` | Android 15 真实 `windowBlurBehind` 高斯模糊 + 底部 28dp 悬浮 Miuix 面板 |
| **UI 点击「停止」** | 首页点击 `停止` -> `boxctl down` | `PURE_RUST_HONK_AFTER_UI_STOP=STOPPED`，进程停止且 eBPF 规则清理干净 |
| **原版无损恢复** | 恢复启动原版 `com.boxproxy.honk` `v1.0.10` | `orig_pid=11939`，**`cn=204` / `foreign=204`**，双包安全共存 |

### 3.2 1:1 前端视觉与设置功能逐页核对

已将 `io.honk.android` 与原版 `com.boxproxy.honk` 在 OnePlus 13 上逐屏截屏对照确认 1:1 复刻：

- **Tab 1 首页**：可折叠大标题 `BoxProxy Honk`、运行态淡蓝灰 Hero 卡片 + 裁剪大水印（点击弹出 M02 运行系统详情与 M04 出站模式弹窗）、`启动` / `重载|停止|重启` 操作栏、`WebUI | 日志` 双卡、`延迟` 三列卡片（`Baidu`/`Cloudflare`/`Google` + 设置/刷新图标）、`WAN [详情] | 网速`、`订阅 | 资源占用`（含内存进度条）、无药丸指示器的经典 Miuix 底栏。
- **Tab 2 面板**：右上 `⋮` 菜单（WebUI / 分流规则与追踪 / 规则集 / M16 API 配置 / 刷新）、四段胶囊（`概览 | 策略 | 订阅 | 连接`）、`运行概况` 三数卡片、`订阅`、`上行速度`/`下行速度` 彩色箭头卡、`总流量`、`近期趋势（最近 60 秒）` 平滑双曲线面积图。
- **Tab 3 工具**：完整 7 组白色 20dp 卡片与原版逐行一致（`网络测试`；`文件管理`/`脚本`；`日志查看`；`应用管理`；`网络匹配`/`共享网络`/`绕过规则`；`订阅管理`/`CNIP 设置`；`更新 WebUI`/`更新核心`/`从文件安装核心`/`Sub-Store`）。
- **Tab 4 设置与全部子页面**：
  - 主设置页 5 组卡片：`基础代理配置` / `其他代理配置`；`语言`（5 语言选择弹窗） / `主题设置` / `备份与恢复`（底部 28dp 浮窗 + 4 类 M49 范围勾选 + SAF ZIP 原子备份恢复）；`开机自启` / `镜像下载`（前缀编辑弹窗） / `GitHub Token`（密文编辑弹窗）；`长按磁贴`（面板/WebUI/主页） / `通知`（权限申请 + 12 占位符模板/间隔/3 按钮子页） / `显示底栏面板入口` / `默认面板页面`（7 入口） / `启动时打开面板`；`关于`（粉紫蓝渐变头图 + BoxCat 徽标 + `赞助支持` + `GitHub 项目` + `开源引用库` + M50 桌面图标切换）。
  - **`基础代理配置`**：`核心选择`（7 核心）、`运行模式`（6 模式）、`IPv6`（3 模式）、`自动覆写` 开关、`覆写设置`（M22 覆写开关弹窗 + `boxctl config sync`）、`查看启动配置`（直接打开 `startup-config`）、`配置选择 (+)` 列表与蓝色勾选标记。
  - **`其他代理配置`**：`代理能力`（性能模式 / QUIC / DNS 转发 / 代理 TCP / 代理 UDP）、`DNS 劫持`（TCP / UDP / 策略下拉）、`资源限制`（CPU 核心分配 `0-7` / 内存限制 `100M` / 磁盘 I/O 权重 + 保存与热应用）、`厂商防火墙`（启动时清理）。
  - **`主题设置`**：`界面风格`（Miuix / Material 3）、`主题模式`（跟随系统 / 浅色 / 深色）、`深色使用纯黑背景`、`使用 Monet 取色`、`色彩风格`（10 种带 3 色点预览）、`色彩标准`（Material 3 2021/2025/2026）、`强调色`（默认 + 12 预设）、`模糊效果`、`顶栏样式`、`底栏样式`、`横滑返回`、`预测性返回动画`、`动画方向跟随滑动边缘`、`界面缩放`（80%–120% 5 档 Miuix 药丸滑块，实时驱动 `LocalDensity`）。

---

## 4. 真机调试中发现并彻底清零的缺陷清单

1. **`boxproxy-schema` 默认种子写入 `ap+` 通配符导致 `boxctl up` 报错**：原版 `com.boxproxy.honk` 的 `shared_network_interfaces` 默认为空，而 `boxctl` 严格校验网卡名不含 `+`。已将 `DEFAULT_SHARED_NETWORK_INTERFACES` 修正为 `&[]` 并在 `seed_default_rows` 中自动清理历史通配符条目，重编 `boxctl` 与 `libboxproxy_app.so` 后一次性拉起成功。
2. **Honk Native API 字符串型 `u64` 遥测解析**：真机实测 `/api/v1/runtime` 与 `/api/v1/runtime/memory` 将 `uptime_seconds`、`upload_bytes_per_second`、`download_bytes_per_second`、`rss_bytes`、`current_bytes` 序列化为十进制字符串。已在 `boxproxy-engine` 新增 `json_u64`/`json_f64` 兼容解析，并首帧拉取 `/api/v1/runtime/traffic/history` 预热 60 秒波形。
3. **`PanelSheetActivity` 定位与主题配色**：修正为底部对齐浮窗 + `navigationBarsPadding()` + 同步 `LocalBoxPalette` 明暗主题。
4. **R8 裁剪体积优化**：移除冗余 `material-icons-extended`，开启 `isMinifyEnabled = true` 与 `isShrinkResources = true`，配合 `proguard-rules.pro` 保护全部 12 JNI 与 WebView 桥接符号，APK 由 45 MB 压缩至 **`7.3 MB`**（单 `classes.dex`）。
5. **Android 系统返回栈与空字符串兜底**：补齐 Compose `BackHandler` 对接 Rust `pop_back`，修复 `JSONObject.optString` 空串覆盖 fallback 导致 `"WAN:   "` 与 `"Grok ()"` 的问题。
6. **最终日志审计结果**：
   - `honk.log`（113 行）：**`HONK_LOG_WARN_ERR = 0`**（0 WARN、0 ERROR、0 PANIC）。
   - `runs.log`：完整记录 Config check passed -> Inbound rules applied -> Service started -> Startup completed，**0 错误**。
   - `logcat`（`io.honk.android` PID）：**0 FATAL、0 Exception、0 UnsatisfiedLinkError、0 Rust panic**。
