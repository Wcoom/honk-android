# honk-android：对上游 honk 的改动清单

基线：`daeuniverse/honk` `main` 分支（提交 `8919784a`，在 `v0.1.0-beta.6` / `8a7611e` 基础上同步上游至 `#518`）。

## 1. eBPF 数据面：适配 Android GKI 6.1+ / 6.6+ Verifier（1M 指令上限 + 512B 栈上限）

- `crates/honk-ebpf-common/src/conn.rs`、`crates/honk-ebpf-common/src/routing_policy.rs`：
  - 新增 `zero_in_place_volatile<T>(ptr: *mut T)`，按对齐字宽以 `write_volatile` 展开定长清零指令，彻底阻止 LLVM 将结构体清零合并成 `aya_ebpf::memset` 逐字节循环（verifier 对逐字节循环每次迭代派生新状态，在 L2/L3 多分支下会放大数十万条验证指令）。
  - `ParseTransportCtx::zero_in_place` 改用 `zero_in_place_volatile`。
  - `write_name_byte` 在 volatile 回读偏移后增加 `& 0x3f` 掩码，清除寄存器 `id` 别名传递，解决 Linux 6.1 Verifier 连续 UTF-8 替换写入时的死分支边界倒挂（`umin=48 > umax=45`）。
- `crates/honk-ebpf/src/egress.rs`：
  - 将 WAN egress 拆成平级子程序分派：`wan_egress_prelude`（64B 栈）、`do_tproxy_wan_egress_tcp_entry`（312B 栈）、`do_tproxy_wan_egress_udp_entry`（296B 栈），由 `wan_egress_dispatch` 平级调用；所有 `#[inline(never)]` 子程序统一返回单寄存器 `c_long`（避免 `Result<c_long, c_long>` 使用 `r2` 导致 `R2 !read_ok`）。
  - 新增 `fill_handoff_fresh` 逐字段 volatile 直写 `RoutingHandoffEntry`。
- `crates/honk-ebpf/src/maps.rs`、`contrack.rs`、`contrack/tcp.rs`、`contrack/udp.rs`：
  - `CONNTRACK_SNAPSHOT_MAP` 改为与 `FlowMapValue<ConnState>` 同布局的 `ConnSnapshotSlot`（80B），新增 `conntrack_output_flow_value` 与 `claim_udp_preparing_in`，使 `CONN_STATE_MAP.insert` 直接使用 per-CPU map 槽位而非在栈上构造 80B 的 `FlowMapValue` 临时量，将 `__mark_tcp_seen` / `__mark_udp_seen` 栈深压低 56–64B（总调用链栈深 ≤ 432B < 512B）。
- `crates/honk-ebpf/src/transport.rs`、`route.rs`、`auxiliary.rs`、`event.rs`、`ingress.rs`：
  - 将 `parse_slow`、`fill_tuples`、`build_input`、`evaluate_policy`、`honk_normalize_process_name`、`prep_redirect_to_control_plane`、`send_dae_event` / `send_flow_expired` 等处的 `write_bytes` / `mem::zeroed()` 全部替换为 `zero_in_place_volatile`，使 `wan_egress_l2`、`wan_egress_l3`、`lan_ingress_l2`、`lan_ingress_l3` 等全部 8 个 TC 程序在 6.6.118 上一次通过 verifier。

## 2. Android 策略路由与默认接口探测

- `crates/honk-ebpf-common/src/lib.rs`、`crates/honk-config/src/config.rs`：
  - 将 `DAE_BYPASS_MARK` 及 `GlobalConfig::effective_so_mark()` 默认值从 `0x100` 调整为 `0x1000_0000`。Android netd 的策略路由规则（`31000: from all fwmark 0x0/0xffff iif lo lookup <net_table>`）要求 `fwmark` 低 16 位为 0，否则落入 `32000: unreachable` 导致 `ENETUNREACH`。
- `crates/honk-config/src/config.rs`、`crates/honk-core/src/ebpf/interface.rs`：
  - 新增 `parse_android_netd_default_ifindex`（解析 `/proc/net/fib_trie` 中 Android netd 的 `Id <1000 + ifindex>:` 策略路由表里的 `0.0.0.0/0 universe UNICAST` 默认路由）以及 `default_route_interface_by_socket_probe` 二级兜底，使 `wan_interface: auto` 在 `/proc/net/route`（main 表）为空的 Android 上自动绑定当前移动数据/Wi-Fi 接口（如 `rmnet_data2` / `wlan0`）。

## 3. 内核版本门禁、cgroup 自动升级、Linux 6.1 clsact/BTF 回退与 Android 自包含运行

- `crates/honk-core/src/ebpf/real/{attach.rs,mod.rs,bindings.rs,observation.rs,routing.rs}`、`crates/honk-core/src/ebpf/interface.rs`、`crates/honk-core/src/netlink.rs`：
  - 修复 `kernel_version()` 对带 `-android` 后缀补丁号的解析，将默认内核版本门禁从 `6.12.0` 放宽至 `6.1.0`（设 `HONK_MIN_KERNEL=1` 可恢复上游 `6.12.0` 门禁）。
  - 新增 `BindingHandle(Tcx | Netlink)` 与 Netlink `clsact` qdisc / `cls_bpf` 过滤器管理：Linux 6.6+ 保持原生 TCX 链路，Linux 6.1–6.5 无 TCX 时自动回退至 `clsact`/`cls_bpf` 并同步接入 `/api/v1/datapath` 钩子健康校验。
  - `routing.rs` 创建 `ROUTING_POLICY_ROOT` 内层 `RoutingPolicyDescriptor` 数组时附带匹配结构体 BTF（含无 BTF 回退及单测），解决 Linux 6.1 `bpf_map_meta_equal()` 的 `timer_off` 冲突。
  - 新增 `upgrade_android_cgroup_legacy_hooks`：启动时自动通过 `BPF_PROG_QUERY` 检查根 cgroup 上 honk 所需的 9 类 hook；若存在 `attach_flags == 0` 的 Android legacy BPF 程序，自动通过 `BPF_PROG_GET_FD_BY_ID` → `BPF_PROG_DETACH` → `BPF_PROG_ATTACH(BPF_F_ALLOW_MULTI)` 原地无损升级为 multi 模式。
- `crates/honk-core/src/lib.rs`、`crates/honk-core/src/daens.rs`：
  - 自动探测 `/apex/com.android.conscrypt/cacerts` 与 `/system/etc/security/cacerts` 设置 `SSL_CERT_DIR`；
  - 实例锁路径支持 `HONK_RUN_DIR`，无 `/run` 时自动回退至 `/data/local/tmp/honk-core.lock`；
  - 无 `/var/run` 时静默跳过 `ip netns` 兼容 bind-mount；
  - 新增 `-t`（测试配置并退出）、`-d <data_dir>`、`-f <config>` 命令行兼容别名，直接对接 BoxProxy (`boxctl`)。

## 4. DNS 健壮性与 Android Bootstrap 双栈容错

- `crates/honk-core/src/dns/response.rs`、`crates/honk-core/src/dns/response/tests.rs`：
  - 修复上游 DNS 响应 QNAME 大小写敏感比较缺陷（改为按 RFC 1035 / RFC 4343 使用 `eq_ignore_ascii_case`），避免 DoH 上游将大写查询名规范化为小写时误触发 `QuestionMismatch` 并返回 `SERVFAIL`。
- `crates/honk-outbound/src/bootstrap.rs`、`crates/honk-core/src/lib.rs`、`crates/honk-core/src/control/reload/transaction.rs`：
  - 新增 Bootstrap UDP 1 秒超时自动重传（总预算 3 秒）与 `GLOBAL_FALLBACK`（`set_global_fallback`），在启动与热重载时同步接入 `config.global.fallback_resolver`，彻底解决 Android 无 `/etc/resolv.conf` 时切网瞬间单包丢失导致上游加密 DNS 失联的问题。

## 5. Android 6.6 GKI 内核 UDP 接收跟踪与数据路径钩子全量健康校验

- `crates/honk-ebpf/src/receive_trace.rs`、`crates/honk-core/src/ebpf/real/receive_trace.rs`：
  - 新增 `RECEIVE_ACTIVE_TASK` BPF Map 与 5 个 ARM64 `#[kprobe]` / `#[kretprobe]` 回退程序（挂载 `udp_recvmsg`、`udpv6_recvmsg`、`__skb_recv_udp`）；当 Android 6.6 GKI 内核对 `fentry`/`fexit` 返回 `ENOTSUPP (524)` 时自动回退到 kprobe 路径，消除 `udp_trace_unavailable` 降级并保留完整内核 UDP 流追踪能力。
- `crates/honk-core/src/ebpf/real/bindings.rs`、`crates/honk-core/src/ebpf/real/observation.rs`：
  - 新增 `query_target_progs`（通过 `BPF_PROG_QUERY` 实时核查根 cgroup 与 `daens` netns `sk_lookup` 钩子）及 `observe_named_tc`（进入 `daens` 核查 `dae0peer` TCX 链），使 `/api/v1/datapath` 在全部钩子正常时准确上报 `state="active"`、`hooks="attached"`、`health="healthy"`。

## 6. 原生 Doona 面板全功能解锁与双 API 共存

- `crates/honk-config/src/experimental.rs`、`crates/honk-core/src/native_api/{config.rs,settings.rs,store/startup.rs}`、`crates/honk-core/src/lib.rs`：
  - 新增 `NativeApiConfig::admin_authorized()`：允许仅监听本机回环且开启 `allow_anonymous_loopback: true` 的实例直接启用 `config_write: true` 与 `--store db`，在手机内置 WebView 免登录即可使用配置编辑、导入、节点/规则/订阅管理、修订历史与地理数据更新。
  - 允许 `--store db` 与本机回环 `clash_api`（`127.0.0.1:9090`）共存，同时驱动 BoxProxy 原生「面板」与「WebUI（Doona）」。
  - Android 平台（非 `/var/lib` 数据目录）在满足管理授权时自动提升为 `ConfigStore::Db` 并默认启用常驻后台流程记录（`RecorderMode::On`）。
