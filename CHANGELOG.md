# Changelog

## v1.0.5 (2026-11-23)

### 合并上游 `daeuniverse/honk` 主线至 `7cd8b13c`
- **同步上游提交 `7cd8b13c`（`fix(relay): bound copy drain from the first EOF (#526)`）**：
  - 用户态 TCP 复制中继自首次读到 EOF 起以单调时钟严格约束 30 秒空闲排空超时（覆盖最终 flush 与写端半关闭），避免延迟调度无限延长空闲排空，并在超时截断时仅统计目的端实际接受的字节数。
  - 同步更新 `relay`、`vision` 与 `dns::transport::retry` 单元测试（40/40 relay、4/4 dns retry、130/130 `--features ebpf` 全部通过），并基于 `7cd8b13c` 重新生成与校验 `patches/honk-android.patch`。

### 升级共存客户端至官方 BoxProxy `v204` (`26.11.21.5e4c0b7`) 并完整对接开机自启动脚本
- **升级底座至官方 BoxProxy `v204` (`26.11.21.5e4c0b7`)**：
  - 完整移植 `com.boxproxy.honk`（`BoxProxy Honk` / 磁贴 `Box Honk`）独立包名与全组件共存隔离机制，可与官方原版 `com.boxproxy.box` (`v204`) 免卸载直接并存。
  - 完整对接 `Honk` 核心（`proxy_core_honk`）、锁定 `eBPF` 模式、`.dae` 配置选择（默认 `config.dae`）、`box.identity` 进程白名单、全链路 Clash API `external[-_]controller:` 兼容（首页实时速率/状态与原生「面板」页），以及首页「WebUI」一键直连内嵌 Doona 原生面板（`http://127.0.0.1:9527/ui/`）；完整保留新版 `boxbpf` 并内嵌静态 `honk` 版 `boxctl`。
- **完整对接并验证 BoxProxy 开机自启动脚本 (`/data/adb/service.d/boxproxy-honk.sh`)**：
  - 在 `BoxProxy Honk` 内置 **设置 → 开机自启** 开关中将自启脚本路径、临时写入文件与 Logcat 标签完整隔离为 `/data/adb/service.d/boxproxy-honk.sh`、`.boxproxy-honk.sh.tmp` 与 `BoxProxyHonkBoot`，开启/关闭互不干扰官方 `/data/adb/service.d/boxproxy-box.sh`。
  - 真机端到端实测执行 `/data/adb/service.d/boxproxy-honk.sh` 调用 `boxctl --db "$db" boot` 拉起 `honk-core v1.0.5`，确认 `boot.log` 全流程记录正常、数据路径健康且应用状态自动同步。
  - 一键部署脚本 `skill/honk-android/scripts/deploy_honk_android.sh` 新增 `--boot-autostart` 选项，并同步完善 `GUIDE.md` 与 `SKILL.md` 开机自启章节。

## v1.0.4 (2026-11-21)

### 修复配置 `lan_interface`（热点 / LAN 透明代理）时 `lan_ingress_l2/l3` 栈超限（Fixes #1）
- **LAN ingress 平级子程序拆分**：参考 `wan_egress_dispatch`，将 `crates/honk-ebpf/src/ingress.rs` 中的单一 `do_tproxy_lan_ingress`（304B 栈 + 嵌套 `stage_udp_decision` 152B + `send_flow_expired` 16B，在 GKI 6.6 上叠加至 `544B > 512B`）重构为平级子程序分派：
  - `lan_ingress_prelude`（56B，最大链深 128B）：包解析、VLAN/标记/广播过滤与协议分发；
  - `do_tproxy_lan_ingress_tcp_entry`（208B，最大链深 352B）：LAN TCP 完整流处理；
  - `lan_ingress_udp_pre_entry`（200B，最大链深 352B）：LAN UDP 缓存查找与 `bpf_sock_tuple` / `BpfFibLookup` 本地 socket 探测；
  - `do_tproxy_lan_ingress_udp_route_entry`（232B，最大链深 448B）：LAN UDP 路由、流状态发布与扁平化 `c_long` 返回的 `stage_udp_decision`。
- **全量 TC 栈深校验与真机验证**：全部 10 个 TC 入口程序（含 `lan_ingress_l2`、`lan_ingress_l3`、`wan_egress_l2`、`wan_egress_l3`）调用链最大栈深均控制在 `≤ 448B`（低于 512B 限制）；在 OnePlus 13 Android GKI 6.6 上使用 L2 `veth` 接口复现原 544B 报错并验证修复后 `lan_ingress_l2` 一次通过 Verifier 并成功绑定。

## v1.0.3 (2026-11-21)

### Android GKI Linux 6.1+ 全栈适配（兼容 6.1–6.5 与 6.6+）
- **Linux 6.1 eBPF Verifier 适配**：`crates/honk-ebpf-common/src/routing_policy.rs` 的 `write_name_byte` 在 volatile 回读偏移后增加 `& 0x3f` 掩码，清除寄存器 `id` 别名传递，消除 Linux 6.1 Verifier 在连续 UTF-8 替换写入中的死分支边界倒挂（`umin=48 > umax=45`）。
- **Linux < 6.6 `clsact` Netlink TC 自动回退与健康校验**：在 `netlink.rs`、`interface.rs`、`bindings.rs`、`attach.rs`、`observation.rs` 中新增 Netlink `clsact` qdisc / `cls_bpf` 过滤器管理与 `BindingHandle(Tcx | Netlink)` 句柄；Linux 6.6+ 仍保持原生 TCX 链路，Linux 6.1–6.5 在内核无 TCX 支持时自动回退至 `clsact`/`cls_bpf` 并完整接入 Doona `/api/v1/datapath` 健康校验。
- **Linux 6.1 `ROUTING_POLICY_ROOT` map-in-map BTF 适配**：`crates/honk-core/src/ebpf/real/routing.rs` 创建内层 `RoutingPolicyDescriptor` 数组时自动附带匹配结构体 BTF（含无 BTF 回退及单元测试），解决 Linux 6.1 `bpf_map_meta_equal()` 的 `timer_off` 冲突（`-ENOENT` vs `-EINVAL`）。
- **内核版本解析修复与默认门槛放宽至 6.1+**：`crates/honk-core/src/ebpf/real/mod.rs` 修复 `kernel_version()` 对带 `-android` 后缀补丁号的解析，`attach.rs` 将 Android 默认内核门槛放宽至 `6.1.0`（设置 `HONK_MIN_KERNEL=1` 仍恢复上游 `6.12.0`）。
- **测试与补丁同步**：同步补齐 `--features ebpf` 下 `BindingHandle` 与 `start_interfaces` 单元测试签名（130/130 通过），重新生成 `patches/honk-android.patch`。

## v1.0.2 (2026-11-21)

### 新版 BoxProxy (`v199 / 26.11.20.7fc0984`) 共存 APK 全面对接
- **升级底座至官方 BoxProxy `v199` (`26.11.20.7fc0984`)**：基于最新官方版本完整移植 `Honk` 核心适配与 `com.boxproxy.honk` 共存隔离机制，支持直接覆盖升级安装，并与官方原版 `com.boxproxy.box` (`v199`) 完美并存。
- **原生对接 `Honk` 核心与 `eBPF` 模式**：
  - 在客户端核心枚举与资源表新增 `Honk`（`proxy_core_honk`），自动锁定并仅允许 `eBPF` 运行模式，配置文件选择器原生支持 `.dae` 扩展名（默认 `config.dae`）。
  - 完整适配 `box.identity` 进程校验、首页实时状态与网速/流量卡片（兼容 `.dae` 中 `external_controller` 下划线键）、原生「面板」页 Clash API 仪表板（策略/规则/连接/趋势），以及首页「WebUI」一键直连内嵌 Doona 原生面板（`http://127.0.0.1:9527/ui/`）。
  - 完整保留新版 `boxbpf` 资源并内嵌静态 `honk` 版 `boxctl`，同步覆盖手动核心导入、备份/恢复白名单及全组件隔离（独立包名、应用名 `BoxProxy Honk`、磁贴 `Box Honk`、权限、Authority、通知广播与自启脚本 `/data/adb/service.d/boxproxy-honk.sh`）。

### 同步上游 `daeuniverse/honk` 主线与配置模板修复
- **合并上游提交 `8a7611e..8919784a`（5 commits / 52 files）**：同步上游协议栈与隧道层重构（`honk-protocol` / `honk-outbound` 等），重新生成 `patches/honk-android.patch` 并完成单元测试、ARM64 静态编译与真机端到端验证。
- **修复 `config.example.dae` 策略组名称引号问题**：移除 `group { ... }` 块头及路由目标 `default:` 中的多余单引号，修复初次使用模板启动时报错 `routing target is not a declared group or builtin` 的问题。

## v1.0.1 (2026-11-21)

### 与官方原版 BoxProxy 共存升级 (`com.boxproxy.honk`)
- **独立包名与应用标识**：将 APK 包名更改为 **`com.boxproxy.honk`**，桌面应用名称设为 **`BoxProxy Honk`**，通知栏快捷开关磁贴设为 **`Box Honk`**，支持与官方原版 `com.boxproxy.box` **免卸载直接并存安装**。
- **全组件隔离**：同步隔离 `AndroidManifest.xml` 签名权限、`androidx-startup` Provider Authority、通知栏控制广播 Action/Extra、备份文件名前缀，以及开机自启脚本路径（`/data/adb/service.d/boxproxy-honk.sh`），彻底避免与原版配置、数据库或自启脚本产生冲突。
- **工具链与配置同步**：更新 `boxctl` 默认目录探测、推荐配置模板 `config.example.dae`、一键部署脚本 `deploy_honk_android.sh`（默认指向 `com.boxproxy.honk`，支持 `--pkg` 参数自定义且在宿主机直接完成 SQLite `runtime_profile` 预配置）及全部文档。

## v1.0.0 (2026-11-20)

### Android 平台核心移植 (`honk-core` / `honk-ebpf`)
- 完整适配 Android ARM64 GKI Linux 6.6 内核 eBPF 透明代理数据面（`cgroup` + `tc` + `daens` + `sk_lookup`），静态链接 `aarch64-unknown-linux-musl` + `jemalloc`，支持蜂窝/Wi-Fi 网卡动态切换自动解绑与重挂。
- 启用 `ebpf,rprx,clash-api,native-ui,jemalloc` 全特性构建，内置原生 **Doona** 面板资源（`ui: 'embedded'`），单文件即可同时提供 Clash API 与原生 Doona 面板。
- **Android 6.6 GKI 内核 UDP 接收跟踪支持**：为 `honk-ebpf` / `honk-core` 增加 ARM64 `kprobe`/`kretprobe` 自动回退（挂载 `__skb_recv_udp`、`udp_recvmsg`、`udpv6_recvmsg`），消除 `udp_trace_unavailable` 降级告警并完整保留内核 UDP 流追踪证据。
- **完整 eBPF 数据路径内核钩子校验**：通过 `BPF_PROG_QUERY` 对 cgroup、`daens` `sk_lookup` 与 `dae0peer` TCX 链进行实时内核态校验，使 Doona 数据路径状态完整显示为 `活动 (active)` / `挂钩: 已附加 (attached)` / `健康: 健康 (healthy)`。
- **原生 Doona 全功能解锁 + UDP/QUIC 全开**：支持本机环回免密管理授权、`--store db` 配置库修订历史与导入（与本机 Clash API 共存）、后台常驻流程/日志记录（`HONK_RECORD_ALWAYS=1`）、完整 UDP/QUIC 转发及节点评分 QUIC 探测。
- 优化编译参数，加入 `--remap-path-prefix` 脱敏构建路径，确保发布二进制不含本机路径信息。

### Bug 修复（基于 22 小时实机长稳压测日志）
- **修复 DNS 响应 QNAME 大小写敏感校验 Bug**：`crates/honk-core/src/dns/response.rs` 改为按 RFC 1035/4343 使用 `eq_ignore_ascii_case` 校验响应问题段域名，解决 DoH 上游将大写域名规范化为小写时被误判 `QuestionMismatch` 并返回 `SERVFAIL` 的问题。
- **修复 Android 平台 Bootstrap DNS 单点丢包与回退失效**：`crates/honk-outbound/src/bootstrap.rs` 新增 UDP 1 秒自动重传与 `fallback_resolver` 自动兜底（启动与热重载同步接入），避免 Android 缺少 `/etc/resolv.conf` 时在切网瞬间触发 `SERVFAIL`。
- **消除 Clash API 启动绑定告警**：默认配置模板将 `clash_api.external_controller` 收紧为 `127.0.0.1:9090`。

### BoxProxy 与 `boxctl` 集成
- 升级 `boxproxy/boxctl` 支持 `honk` 核心、`ebpf` 模式、`startup-config` 同步、过期 `box.identity` 清理及新版 BoxProxy CLI 子命令。
- 适配 BoxProxy ARM64 APK 原生支持 `Honk` 核心选择、首页运行状态卡片、Clash API 仪表板以及 **WebUI 直连 `http://127.0.0.1:9527/ui/`（原生 Doona 面板）**。
