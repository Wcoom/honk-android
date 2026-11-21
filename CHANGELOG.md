# Changelog

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
