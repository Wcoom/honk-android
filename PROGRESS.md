# honk 移植到 Android —— 项目状态与进度

> 最后更新：2026-11-18
> 目标设备：OnePlus 13（PJZ110），Android 16，内核 **6.6.118** GKI（`abogki537459655`），
> bootloader 已回锁，root 来自 KernelSU **LKM** 模式（`phone root` 一键恢复，仅本 boot 有效）。

---

## 1. 项目目标

把 [daeuniverse/honk](https://github.com/daeuniverse/honk)（Rust + eBPF 透明代理）
跑在 Android 上，**替代现有的 mihomo / BoxProxy**。允许的改动范围（用户明确约束）：

- ✅ 只能加载 `.ko` 模块、只能在 shell 里跑脚本 —— **不许动分区和镜像**（否则变砖）
- ✅ 可以改 honk 源码
- ✅ 可以改 [boxproxy/boxproxy](https://github.com/boxproxy/boxproxy) 来对接 honk
- ✅ 若重启导致 root 丢失，重跑 `phone root` 再继续

---

## 2. 目录结构

```
/home/wcoom/桌面/oplus13/honk-android/
├── src/                  honk 源码（基于 v0.1.0-beta.6 / 8a7611e）+ 本项目补丁
├── toolchain/
│   ├── aarch64-linux-musl-gcc    cc-rs 找的 C 编译器（zig 冒充 musl gcc）
│   ├── aarch64-linux-musl-g++    同上（C++）
│   └── zig-link                  rustc 用的链接器包装器
├── lib/                  zig 预构建的 aarch64 运行时归档
│   ├── libc++.a / libc++abi.a    boring-sys 需要（musl 无 libstdc++）
│   ├── libubsan_rt.a             aws-lc-sys 的 jitterentropy 带 UBSan 插桩
│   └── libcompiler_rt.a          上面两者依赖的 __extendxftf2 等
├── build.sh              一键交叉编译 → aarch64 静态 honk-core
├── measure.sh            快速量 verifier 成本
├── setup-cgroup.sh       手机侧：把根 cgroup 的 legacy hook 升级到 multi
├── boxproxy/             boxproxy 源码（待对接，尚未改动）
├── README.md             构建与部署说明
├── PATCHES.md            对上游 honk 的改动清单
├── ANDROID-DATAPATH.md   为什么 TC 数据面过不了 6.6 verifier（技术分析）
└── PROGRESS.md           本文件
```

Git 历史（3 个提交）：
```
1b476d3 eBPF 去除 memset 循环（6.6 verifier 优化尝试）
2f833ad 加 setup-cgroup.sh：把根 cgroup 的 legacy hook 升级到 multi 模式
f9af855 honk 移植到 Android：交叉编译工具链 + Android 适配补丁
```
⚠️ `src/` 之后还有未提交的改动（见 §7），`boxproxy/` 是嵌套 git 仓库（未纳入）。

---

## 3. 已完成

### 3.1 交叉编译链路（✅ 完全打通）

产物：`src/target/aarch64-unknown-linux-musl/release/honk-core`，**约 24 MB，aarch64 静态链接**。

踩过的坑与解法（都在脚本注释里写清楚了）：

| 问题 | 原因 | 解法 |
|---|---|---|
| 找不到 `aarch64-linux-musl-gcc` | `boring-sys`（BoringSSL C++）的 cc-rs 按名字找 musl 交叉 gcc | 写包装器用 `zig cc` 冒充 |
| `unable to parse target query 'aarch64-unknown-linux-musl'` | zig 不认 Rust 三元组拼写 | 包装器剥掉所有 `--target=` 形式，自己用 `aarch64-linux-musl` 重新锚定 |
| `bits/libc-header-start.h` 找不到 | clang 交叉时默认看宿主 glibc 头 | 补 zig 自带 musl 头路径 |
| bindgen 断言 `size_t` 是 4 字节 | **头文件顺序错**：`generic-musl/bits/alltypes.h` 把 `_Addr` 定义成 `int`，`aarch64-linux-musl/` 的定义成 `long` | **aarch64 目录必须排在 generic-musl 之前** |
| 链接报 `duplicate symbol: _start` | zig 自带 crt1.o 与 rust 的 self-contained crt 冲突 | 改直接调 `rust-lld`（`-flavor gnu`），彻底绕开 zig 的启动逻辑 |
| `-Wl,--fix-cortex-a53-843419` 不被识别 | rustc 给 aarch64-musl 默认注入的 GNU ld 专有选项 | 包装器里剥掉 |
| `undefined symbol: __ubsan_handle_*` | aws-lc-sys 的 jitterentropy 被编进 UBSan 插桩 | 从 zig 缓存取 aarch64 的 `libubsan_rt.a` 显式链入 |
| `undefined symbol: __extendxftf2` | 上面的 ubsan 运行时依赖 compiler_rt | 再把 `libcompiler_rt.a` 链进去 |
| rustc 默认传 `-lstdc++` 但 musl 没有 | boring-sys 的 build script 塞的 | 链接器包装器里改写成 zig 预构建的 libc++ 归档 |
| `running: cmake` 失败 | 本机没装 cmake | `apt install cmake` |

构建命令：`./build.sh`（照抄官方 `release.yml` 的 feature 组合 `ebpf,rprx,clash-api,jemalloc`；
**去掉 `native-ui`**，因为它要求联网拉 doona 前端资源）。

工具链版本：rustup `1.99.1` + target `aarch64-unknown-linux-musl`，
nightly `2026-10-20`（含 `rust-src`/`llvm-tools`，用于编 eBPF），`bpf-linker 0.11.0`，
`clang-18` / `llvm-18` / `cmake` / `zig 0.16`。

### 3.2 honk 源码的 Android 适配（✅ 已改并验证）

详见 `PATCHES.md`，要点：

1. **内核版本门禁降到 6.6**（`ebpf/real/attach.rs`）
   上游硬性 `>= 6.12.0`。逐项核查 GKI 6.6.118 源码后确认它已具备：
   `maybe_wait_bpf_programs()` → `synchronize_rcu_expedited()` 对 `ARRAY_OF_MAPS`/`HASH_OF_MAPS`
   的同步语义、`BPF_MAP_FREEZE`、`BPF_LINK_CREATE` + `BPF_PROG_TYPE_EXT`。
   设 `HONK_MIN_KERNEL=1` 可恢复上游 6.12 门禁。

2. **`/run` 路径可配置**（`lib.rs` / `daens.rs`）
   Android 没有 `/run`（根分区 erofs 只读，`mkdir /run` 直接 `Read-only file system`）。
   改成读 `HONK_RUN_DIR`。daens 命名空间本身是 FD 持有的，那个 bind-mount 只是给
   `ip netns exec` 用的兼容设施，挂不上不影响数据面。

3. **cgroup attach 错误带 errno**（`attach.rs` / `tcp_binding.rs`）
   aya 的 `ProgramError::SyscallError` 只打印 `bpf_link_create failed`，
   加 `{error:?}` 后才看得到 `code: 22, InvalidInput`。排查全靠这个。

4. **`--host-tproxy` 新模式**（`lib.rs` / `interface_startup.rs` / `routing.rs` / `bootstrap.rs`）
   不加载任何 TC 程序、不做接口挂载、不发布内核侧路由表，路由全走用户态
   `Router::route_full`。（这是 §6 方向 B 的产物，目前**部分生效**。）

### 3.3 内核侧能力验证（✅ 全部实测通过）

手机上逐项实测（`adb shell su -c ...`）：

| 检查项 | 结果 |
|---|---|
| `CONFIG_BPF/SYSCALL/JIT/CGROUP_BPF` | 全 `=y` |
| `CONFIG_NET_SCHED/CLS_ACT/XGRESS/NET_NS` | 全 `=y`（TCX 可用） |
| bpffs `/sys/fs/bpf` | 已挂载可写，`map create` 成功 |
| cgroup v2 | `/sys/fs/cgroup` 是 cgroup2 |
| `/sys/kernel/btf/vmlinux` | 6,136,300 字节，ksu 域可读 |
| map-in-map | `array_of_maps` 创建 + `freeze` 都 rc=0 |
| freplace | `bpf_check_attach_target` 符号在 |
| veth / netns / sysctl / iptables | 都能建能写 |
| `iptables -j NFQUEUE --queue-num 320` | 规则能下（`CONFIG_NETFILTER_XT_TARGET_NFQUEUE=y`） |
| `iptables -j TPROXY --on-port --tproxy-mark` | 规则能下，`ip rule`/`ip route local` 都可用 |
| `CONFIG_NF_TABLES` | **`is not set`**（一个 `NFT_*` 都没有）→ honk 的 NFQUEUE 会自动降级（只 warn，不中断） |
| `CONFIG_FUNCTION_TRACER` / `DYNAMIC_FTRACE` | `n` / 不存在 → UDP receive-trace 兜底降级 |

⚠️ **调试坑**：`u:r:shell:s0` 下 `/sys/kernel/btf/vmlinux`、`/proc/kallsyms`、
`/proc/sys/kernel/unprivileged_bpf_disabled` 全是 Permission denied，`dmesg` 也读不了。
**必须确认在 `u:r:ksu:s0`**（`su -c` 里再套一层会掉回 shell 域）。

### 3.4 cgroup hook 冲突（✅ 已解决，`setup-cgroup.sh`）

Android 内核在根 cgroup 上预挂了 15 个 BPF 程序，**全部是 legacy 模式**
（`attach_flags=0`）。内核规则（`kernel/bpf/cgroup.c:__cgroup_bpf_attach`）：
一旦某 hook 上有 legacy 程序，就再不许叠加任何程序（`BPF_F_ALLOW_MULTI` 也会 EPERM）。
而 honk 要往根 cgroup 挂 6 个必需程序，`CgroupAttachMode::Single` 等价于 flags=0。

**解法**：detach 再以 `multi` 挂回**同一个程序** —— 功能不变，只是把该 hook 变成可叠加。
需要的 8 个类型：`cgroup_sock_ops`(19)、`cgroup_inet_sock_create`(29)、
`cgroup_inet4_connect`(33)、`cgroup_inet6_connect`(34)、`cgroup_udp4_sendmsg`(37)、
`cgroup_udp6_sendmsg`(38)、`cgroup_inet4_bind`(31)、`cgroup_inet6_bind`(32)、
`cgroup_inet_sock_release`(30)。

实测结果：升级后 honk 成功挂上 —— 日志 `Attached 6 required cgroup programs to /sys/fs/cgroup`。
**网络、mihomo 均不受影响**（已验证）。

⚠️ **本 boot 有效**，真重启后恢复 legacy，需要重新跑 `setup-cgroup.sh`。

---

## 4. 卡点：TC 分类器过不了 6.6 的 verifier（❌ 未解决）

### 4.1 现象

```
load TC program 'wan_egress_l2':
  BPF_PROG_LOAD syscall returned Argument list too long (os error 7)
BPF program is too large. Processed 1000001 insn
processed 1000001 insns (limit 1000000) max_states_per_insn 50
  total_states 44810 peak_states 4483 mark_read 116
verification time 3403809 usec
```

cgroup 6 个程序全挂上、TCX 可用、BTF/freplace/map-in-map 都正常，
**只死在这一个程序上**。`BPF_COMPLEXITY_LIMIT_INSNS` 在 6.6 和 7.3 里**都是 1000000**，
所以不是版本号差异。

### 4.2 已试过、都无效的优化

| 改动 | 结果 |
|---|---|
| `opt-level=3` + `inline-threshold=1000` | 无变化（仍 1M） |
| `write_bytes(handoff, 0, 1)` → 结构体整体赋值 | 无变化 |
| `ParseTransportCtx` 的 `write_bytes` → 逐字段 `write_volatile`（`zero_in_place`） | 无变化 |
| 两处 handoff 写入改成逐字段直写（`fill_handoff_fresh`，去掉栈临时量） | 无变化 |
| `do_tproxy_wan_egress` 改 `inline(never)` | 1M → **611,514**（有效！） |
| 再把 tcp + udp 也 `inline(never)` | 611K 但 **栈超限**：`combined stack size of 4 calls is 544` |
| **只拆 tcp**（当前代码状态） | **947,936 指令**（进上限内了！）但 **栈超限 704**：`combined stack size of 3 calls is 704. Too large` |

### 4.3 根因

上游 `doc/en/design/routing.md` 记录：同类策略在 **6.12.107** 上 verifier 只走
**53,741–368,874** 条指令。同样字节码、同一 LLVM，6.6 上要 1M+。
差别在 verifier 的状态合并能力：6.6 的 `regsafe()` 对 `PTR_TO_MAP_VALUE` 只比
`memcmp` + `range_within` + `tnum_in` + `check_ids`，6.12+ 有更细粒度的标量剪枝。

**两个维度同时吃紧**：指令数（1M 上限）和栈（512 字节硬上限）。
拆子程序能降指令数，但每层调用各占栈，3 层就 704 字节超限；BPF 没有
「只压栈不拆函数」的开关。

---

## 5. 已验证能跑通的部分

`--host-tproxy` 模式（见 §3.2 第 4 条）在手机上**能启动并常驻**（`timeout 60` 后被 kill，退出码 124）：

```
INFO  honk_core::ebpf::real::attach: Attached 6 required cgroup programs to /sys/fs/cgroup
INFO  honk_core: no TC routing target (host_tproxy datapath); all routing decisions are made in userspace
INFO  honk_core: honk-core is running.
INFO  honk_core::control::lifecycle: Control plane listening for TPROXY TCPv4 on 0.0.0.0:12345
INFO  honk_core::control::lifecycle: Control plane listening for TPROXY UDPv4 x4 on 0.0.0.0:12345
```

但**此时没有数据面**：没有 TC 程序往外捞流量，也没有 iptables 规则，
所以不会有任何流量被代理。要真正接管需要二选一：
- 修好 TC（§4，栈是指令数之外的第二个硬墙）
- 或走 iptables 选包（用户已明确**不要**这条路）

---

## 6. 下一步方向

### 方向 A：继续攻 TC verifier（当前主线）

指令数已经压进上限（947K/1M），现在**卡在栈 704 > 512**。可尝试：
1. 把 `do_tproxy_wan_egress_tcp` 里的大栈对象挪到 map（scratch 里已有
   `UDP_DECISION_SCRATCH_MAP`，可以再加一块或复用）；
2. 看 `parsed_packet` 相关的大结构（`RoutingInput` 128B / `KernelRouteOutput` 272B）
   是否在某个子程序的栈上；
3. 用 `bpftool prog load ... log_level=2` 拿到每层子程序的确切栈用量再针对性处理。

### 方向 B：~~用 iptables TPROXY 代替 TC~~ —— 用户已否决

用户明确指示：「不必使用 iptables TPROXY 模式，
[Wcoom/mihomo-ebpf-smart-export](https://github.com/Wcoom/mihomo-ebpf-smart-export)
这条路已经完美使用 eBPF」。该仓库是 mihomo 的 fork，`common/ebpf/native/` 下有
**3027 行手写 C**（`tc.bpf.c` 905 行 + `cgroup.bpf.c` 900 行 + `shared_network.bpf.c` 644 行
+ `fakeip_icmp.bpf.c` 568 行），用 `bpf2go` 生成内嵌对象，**在 6.6 上能过 verifier**。
它的做法（值得借鉴）：
- 程序按职责拆成多个**独立的小程序**（`local_egress_ethernet_mark`、
  `local_egress_raw_ip_process`、`shared_ingress_ethernet` 等），不是一个大程序
- 用 legacy `struct bpf_map_def SEC("maps")` 而非 BTF map
- 用 `CiliumEBPF` + `link` 库，TCX 与 tc 双栈（`tcxSupport` 探测）

### 方向 C：与 boxproxy 对接

`boxproxy/` 已克隆（`boxctl` + `boxbpf` 两个 Rust 组件）。`boxbpf` 是加载
socket-filter 给 `xt_bpf` 用的，`boxctl` 负责 iptables/eBPF 规则、核心配置、
SQLite 规则库、Wi-Fi 监控、服务生命周期。**尚未开始改动**。

---

## 7. 当前代码状态（未提交的改动）

`src/` 相对 `1b476d3` 的 diff（10 文件，+252 −53）：

```
crates/honk-core/src/control/bootstrap.rs          |  19 ++-   容忍无 TC 的路由发布
crates/honk-core/src/daens.rs                      |   9 ++    netns_dir()
crates/honk-core/src/ebpf/real/attach.rs           |   4 +-    版本门禁 + errno
crates/honk-core/src/ebpf/real/interface_startup.rs|  14 +-    host_tproxy 跳过 TC
crates/honk-core/src/ebpf/real/routing.rs          |   6 +    无 target 时优雅失败
crates/honk-core/src/ebpf/real/tcp_binding.rs      |   7 +-    errno
crates/honk-core/src/lib.rs                        |  28 ++-   --host-tproxy + HONK_RUN_DIR
crates/honk-ebpf-common/src/conn.rs                |  58 ++++  zero_in_place()
crates/honk-ebpf/src/egress.rs                     | 148 ++++  fill_handoff_fresh + 子程序拆分
crates/honk-ebpf/src/transport.rs                  |  12 +-   parse_fast 用 zero_in_place
```

eBPF 子程序的当前 `inline` 标注（`egress.rs`）：
- `do_tproxy_lan_egress` — `inline(never)`（上游原有）
- `do_tproxy_wan_egress` — `inline(never)`（本项目改）
- `do_tproxy_wan_egress_tcp` — `inline(never)`（本项目改，**导致栈 704**）
- `do_tproxy_wan_egress_udp` — `inline(always)`（试过拆，栈更差，已回退）

⚠️ 如果继续调，**先把 `do_tproxy_wan_egress_tcp` 改回 `inline(always)`** 可以回到
1M 指令/无栈问题的基线状态。

---

## 8. 设备当前状态

- **mihomo 正在运行**（BoxProxy eBPF 模式），网络正常
- 8 个 cgroup hook 停在 **multi 模式**（为 honk 升级的），真重启会恢复 legacy
- 无 `dae0` 残留、无 honk 进程
- 手机侧临时文件：`/data/local/tmp/honk-core-a64`（二进製）、
  `/data/local/tmp/honk/`（run/ssl/data 目录 + 各种 log）、
  `/data/local/tmp/setup-cgroup.sh`、`/data/local/tmp/honk-min.dae`
- KernelSU root 有效（`u:r:ksu:s0`），模块版本三方一致（32716）

---

## 9. 复现步骤

```bash
# 1. 构建
cd /home/wcoom/桌面/oplus13/honk-android
./build.sh
# → src/target/aarch64-unknown-linux-musl/release/honk-core

# 2. 推送到手机
adb push src/target/aarch64-unknown-linux-musl/release/honk-core /data/local/tmp/honk-core-a64

# 3. 手机侧准备（需 root）
BASE=/data/local/tmp/honk
mkdir -p $BASE/run $BASE/ssl $BASE/data
mount --bind /apex/com.android.conscrypt/cacerts $BASE/ssl

# 4. 升级 cgroup hook（每次真重启后都要做）
su -c 'sh /data/local/tmp/setup-cgroup.sh'

# 5. 停 mihomo（两者抢同一批 hook）
su -c 'kill $(pidof mihomo)'

# 6. 运行
su -c "BASE=$BASE; export HONK_RUN_DIR=\$BASE/run SSL_CERT_FILE=\$BASE/ssl SSL_CERT_DIR=\$BASE/ssl; \
       cd \$BASE && /data/local/tmp/honk-core-a64 -c /path/to/config.dae"
```

⚠️ honk 硬编码 `/run/honk-core.lock`，必须设 `HONK_RUN_DIR`。
⚠️ 没有 CA 证书会直接退出（`No CA certificates were loaded from the system`），
所以要把 `/apex/com.android.conscrypt/cacerts` bind 到 `$BASE/ssl`。
⚠️ `SSL_CERT_FILE` 指向目录时 rustls 会 warn（`failed to read PEM from file: Is a directory`），
但不影响启动 —— 真正起作用的是 `rustls-platform-verifier` 扫目录。

---

## 10. 关键结论速查

| 结论 | 依据 |
|---|---|
| 6.6 内核有 honk 需要的全部 eBPF 能力 | 逐项实测 + 6.6.118 源码核对 |
| 版本门禁是「审计门槛」不是「能力门槛」 | `maybe_wait_bpf_programs` 等能力在 6.6 已存在 |
| cgroup legacy hook 可以无损升级到 multi | 实测 detach→multi 重挂，网络与 mihomo 正常 |
| 真正的拦路虎是 verifier 的**指令数 + 栈**双约束 | 1M 上限与 512 栈都是硬墙 |
| `BPF_COMPLEXITY_LIMIT_INSNS` 6.6 与 7.3 相同 | 两版 `include/linux/bpf.h` 都是 1000000 |
| 同项目在 6.12 上只需 53K–368K 指令 | 上游 `doc/en/design/routing.md` |
| mihomo fork 的 eBPF 能在 6.6 跑 | 3027 行 C，按职责拆成多个小程序 |
