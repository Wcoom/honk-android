# honk-android：对上游 honk 的改动清单

基线：`daeuniverse/honk` `main` 分支（提交 `467cfa77`，位于上游 `v0.1.0-beta.7` 之后 13 个提交）。

## 1. eBPF 数据面：适配 Android GKI 6.1+ / 6.6+ Verifier（1M 指令上限 + 512B 栈上限）

- `crates/honk-ebpf-common/src/conn.rs`、`crates/honk-ebpf-common/src/lib.rs`、`crates/honk-ebpf-common/src/routing_policy.rs`：
  - 新增 `zero_in_place_volatile<T>(ptr: *mut T)`，按对齐字宽以 `write_volatile` 展开定长清零指令，彻底阻止 LLVM 将结构体清零合并成 `aya_ebpf::memset` 逐字节循环（verifier 对逐字节循环每次迭代派生新状态，在 L2/L3 多分支下会放大数十万条验证指令）。
  - 新增 `RedirectTuple::write_from_tuples` 逐字段 volatile 直写 40B 重定向键（含 `[u8; 3]` 填充），配合 `flow::redirect_{identity,incarnation}_for_tuples` 消除栈临时量与 `memset(3)`。
  - `ParseTransportCtx::zero_in_place` 改用 `zero_in_place_volatile`。
  - `write_name_byte` 在 volatile 回读偏移后增加 `& 0x3f` 掩码，清除寄存器 `id` 别名传递，解决 Linux 6.1 Verifier 连续 UTF-8 替换写入时的死分支边界倒挂（`umin=48 > umax=45`）。
- `crates/honk-ebpf/src/egress.rs`、`crates/honk-ebpf/src/ingress.rs`、`crates/honk-ebpf/src/fragment.rs`：
  - **平级分派 + `BTF_FUNC_GLOBAL` 独立验证（Fixes #2）**：将 WAN egress 与 LAN ingress 重构为平级分派入口并导出为 `BTF_FUNC_GLOBAL` 全局 BPF 函数——`honk_wan_egress_{prelude,tcp,udp}`（64B / 288B / 264B 栈）、`honk_lan_ingress_{prelude,tcp,udp_route}`（88B / 224B / 160B 栈）、`honk_lan_ingress_udp_pre`（200B 栈，单参数 `ctx`）以及 `honk_fragment_record`（56B 栈）。在保持总调用链栈深 ≤ 416B（`wan_egress_*`）/ ≤ 384B（`lan_ingress_*`）的同时，让 Linux 6.1/6.6 Verifier 对每个全局函数仅验证一次并复用摘要，彻底消除 L3/L2 入口状态乘法爆炸（真机 GKI 6.6.118 上 `wan_egress_l3` 验证指令从 `328,888` 降至 **`128,040`**，降幅 **-61.1%**）。
  - **合并 `fast_path_decision` 双份内联**：在 `do_tproxy_wan_egress_udp` 中统一缓存命中与路由未命中后的出口，将 `honk_wan_egress_udp` 由 2,835 指令压降至 1,860 指令。
  - 新增 `fill_handoff_fresh`、`zero_mac()` 逐字段 volatile 直写 `RoutingHandoffEntry` 与 MAC 数组。
- `crates/honk-ebpf/src/maps.rs`、`contrack.rs`、`contrack/tcp.rs`、`contrack/udp.rs`、`tcp_binding.rs`：
  - `CONNTRACK_SNAPSHOT_MAP` 改为与 `FlowMapValue<ConnState>` 同布局的 `ConnSnapshotSlot`（80B），新增 `conntrack_output_flow_value`、`claim_udp_preparing_in` 与 `claim_udp_preparing_scratch`，使 `CONN_STATE_MAP.insert` 直接使用 per-CPU map 槽位而非在栈上构造 80B 的 `FlowMapValue` 临时量。
- `crates/honk-ebpf/src/transport.rs`、`route.rs`、`auxiliary.rs`、`event.rs`：
  - 在 `transport.rs` 新增编译期展开的 `copy_unaligned_bytes` 替换跨 map 的 ETH/TCP/UDP/IPv6 拷贝，消除 `memmove`；并将 `parse_slow`、`fill_tuples`、`build_input`、`evaluate_policy`、`honk_normalize_process_name`、`prep_redirect_to_control_plane`、`send_dae_event` / `send_flow_expired` 等处的 `write_bytes` / `mem::zeroed()` 全部替换为 `zero_in_place_volatile`，实现全部路由 TC 函数 **0 个 `memset`/`memmove`/`memcpy` 调用**，全部 8/8 TC 程序在真机上 `LOAD OK`。

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
  - 基于上游 `#550` 阶梯 UDP 重传（`400ms` / `1200ms`）与截止时间保护，保留 `GLOBAL_FALLBACK`（`set_global_fallback`）并在启动与热重载时同步接入 `config.global.fallback_resolver`，彻底解决 Android 无 `/etc/resolv.conf` 时切网瞬间主上游失效导致加密 DNS 失联的问题。

## 5. Android 6.6 GKI 内核 UDP 接收跟踪与数据路径钩子全量健康校验

- `crates/honk-ebpf/src/receive_trace.rs`、`crates/honk-core/src/ebpf/real/receive_trace.rs`：
  - 新增 `RECEIVE_ACTIVE_TASK` BPF Map 与 5 个 ARM64 `#[kprobe]` / `#[kretprobe]` 回退程序（挂载 `udp_recvmsg`、`udpv6_recvmsg`、`__skb_recv_udp`）；当 Android 6.6 GKI 内核对 `fentry`/`fexit` 返回 `ENOTSUPP (524)` 时自动回退到 kprobe 路径，消除 `udp_trace_unavailable` 降级并保留完整内核 UDP 流追踪能力。
- `crates/honk-core/src/ebpf/real/bindings.rs`、`crates/honk-core/src/ebpf/real/observation.rs`：
  - 新增 `query_target_progs`（通过 `BPF_PROG_QUERY` 实时核查根 cgroup 与 `daens` netns `sk_lookup` 钩子）及 `observe_named_tc`（进入 `daens` 核查 `dae0peer` TCX 链），使 `/api/v1/datapath` 在全部钩子正常时准确上报 `state="active"`、`hooks="attached"`、`health="healthy"`。

## 6. 原生 Doona 面板全功能解锁与双 API 共存

- `crates/honk-config/src/experimental.rs`、`crates/honk-core/src/native_api/{config.rs,settings.rs,store/startup.rs}`、`crates/honk-core/src/lib.rs`：
  - 新增 `NativeApiConfig::admin_authorized()`：允许仅监听本机回环且开启 `allow_anonymous_loopback: true` 的实例直接启用 `config_write: true` 与 `--store db`，在手机内置 WebView 免登录即可使用配置编辑、导入、节点/规则/订阅管理、修订历史与地理数据更新。
  - 允许 `--store db` 与本机回环 `clash_api`（`127.0.0.1:9090`）共存，同时驱动 BoxProxy 原生「面板」与「WebUI（Doona）」。
  - Android 平台（非 `/var/lib` 数据目录）在满足管理授权时自动提升为 `ConfigStore::Db`，并采用按需流程记录（`RecorderMode::Auto`）兼顾面板观测与后台低功耗。

## 7. Android 独立叶子 cgroup 内存遥测、并发观测防抖与双端核心在线更新（v1.0.6）

- `crates/honk-core/src/lib.rs`（`ensure_dedicated_cgroup()`）、`crates/honk-core/src/native_api/telemetry.rs` 与 `boxproxy/boxctl/src/resource.rs`：
  - 当 `honk-core` 运行于 Android 根 cgroup 时，自动开启 `cgroup.subtree_control` `+memory` 并将其迁入独立叶子 cgroup `/sys/fs/cgroup/honk`；`boxctl` 同步新增 `ensure_honk_unbounded_cgroup()` 为 `honk` 创建并维持独立的无上限或显式受限 `/honk` cgroup。
  - `/api/v1/capabilities` 在 `resources.runtime_memory.metrics` 中明确声明 `cgroup.limit_bytes`（即使当前配额为 `max` 无上限），使内嵌 Doona 完整显示「内存」板块全部 8 项指标（含 `cgroup 上限 = 未设上限` 与 `cgroup 范围 = 本服务`）。
- `crates/honk-core/src/ebpf/real/{programs.rs,observation.rs}`：
  - `ProgramLoader::try_object()` 改为 `try_lock_for(5ms)`，`observe_backend()` 在 `programs` 仍为 `DatapathCheck::Unknown` 时保持未确认而不误判为 `DatapathObservationError::Hooks`，消除并发轮询 `/api/v1/runtime` 与 `/api/v1/datapath` 时偶发的 `hook_observation_incomplete`。
- `crates/honk-core/src/native_api/{core_update.rs,handlers.rs,types.rs}`、`crates/honk-core/src/{main.rs,native_api.rs}` 与 `webui/`：
  - 同步 `Zakkaus/doona` 功能模型（含代理链管理、策略组出口节点编辑、「节点策略选择」规则分流中枢联动及内存/核心更新扩展）并接入 [`rust/xtask`](./rust/xtask/src/main.rs) 自动构建内嵌标准版与 Lite 版纯 Rust WASM WebUI。
  - 新增 `GET/POST /api/v1/x-honk/core/update`，支持检测 `Wcoom/honk-android` GitHub Releases、下载 `honk-android-arm64` 与 `SHA256SUMS.txt`、校验 SHA-256 与 ARM64 ELF64 头并原子替换后分离重启（复用 `record_startup_args()`）；同步适配 Doona 标准版/Lite 版核心更新卡片与 `BoxProxy Honk` 客户端内更新源。

## 8. 内嵌纯 Rust WASM Doona WebUI 静态托管与预压缩支持（v1.1.0 – v1.2.0）

- `crates/honk-core/build.rs`、`crates/honk-core/src/native_api/ui.rs`、`crates/honk-core/src/native_api.rs`：
  - 将 `wasm` 扩展名纳入 `build.rs` 静态资源 Brotli/Gzip 预压缩白名单，并在 `embedded_response()` 补齐 `"wasm" => "application/wasm"` MIME 类型映射；
  - `ui: "embedded"` 同时挂载标准版（`/ui/`）与轻量版（`/ui/lite/` 及 `/lite` 重定向），实现双面板零额外配置无缝切换。
- **手机端低功耗与双栈网络加固（v1.2.0）**：
  - `crates/honk-core/src/native_api/settings.rs`、`crates/honk-core/src/native_api.rs`：恢复默认按需录制（`RecorderMode::Auto`），无面板观察时自动休眠内核流追踪与事件广播，同时 `/api/v1/connections` 直接回退读取 `ConnectionEntry` 链路与规则；
  - `crates/honk-core/src/native_api/telemetry.rs`、`clash_api.rs`、`control/runtime.rs`、`subscription/supervisor.rs`：缓存 `mountinfo` 与 BPF `fdinfo` 扫描结果、零订阅者自动退出 Clash 1Hz 采样协程、收敛空闲 FD 扫描与订阅调度唤醒频率；
  - `crates/honk-config/src/config.rs`、`crates/honk-core/src/ebpf/real/attach.rs`、`crates/honk-outbound/src/bootstrap.rs`：升级双栈 `AF_INET`/`AF_INET6` 默认 WAN 接口探测（穿透 464XLAT `v4-*` 并解除 `ifindex < 1000` 限制）、动态扩容 ColorOS/OPlus 根 cgroup BPF 升级槽位与失败回滚、前置 `/etc/hosts` 并为主/备 Bootstrap DNS 分配独立超时预算。
  - 改动已同步写入 [`patches/honk-android.patch`](./patches/honk-android.patch) 与 [`patches/boxctl-honk.patch`](./patches/boxctl-honk.patch) 并通过 `git apply --check -R` 校验。
