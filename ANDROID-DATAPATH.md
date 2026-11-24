# 为什么 honk 的 TC 数据面过不了 6.6 的 verifier —— 根因与解法

## 1. 现象

上游 honk v0.1.0-beta.6 的 TC 程序（`wan_egress_l2` / `wan_egress_l3` 等）在 Android GKI **6.6.118** 上加载失败：

```
load TC program 'wan_egress_l2':
  BPF_PROG_LOAD syscall returned Argument list too long (os error 7)
BPF program is too large. Processed 1000001 insn
processed 1000001 insns (limit 1000000) max_states_per_insn 50
  total_states 44810 peak_states 4483 mark_read 116
```

而在 6.12+ 内核上，同一份字节码只需 53K–368K 条验证指令。

## 2. 两大根因

### 根因 A：单一大函数内联导致分支状态叠加，而纵向拆子程序又撞 512B 栈上限

`do_tproxy_wan_egress` 把包解析、前置过滤、TCP conntrack + 编译路由 + handoff、UDP conntrack + 编译路由 + handoff 全部内联在一个函数内，6.6 verifier 状态合并能力较弱，所有路径状态在主函数内笛卡尔积叠加。

但如果简单把 `do_tproxy_wan_egress` → `do_tproxy_wan_egress_tcp` → `__mark_tcp_seen` → `send_flow_expired` 纵向拆成嵌套子程序，4 层调用链栈深达到 `352 + 296 + 56 = 704B`（或 `544B`），超过 BPF `MAX_BPF_STACK = 512B` 硬上限。

### 根因 B：`aya_ebpf::memset` 逐字节循环在多分支下的状态乘数爆炸

即使把 `wan_egress_l2` 压到 ~57 万指令，`wan_egress_l3` 依然会撞 `1000001 insns`。检查 verifier 日志末尾发现，验证器死在 `.text` 中的 `aya_ebpf::memset`：

```asm
0000000000001778 <memset>:
  if r3 == 0x0 goto +0x6
  r4 = 0x0
loop:
  r5 = r1
  r5 += r4
  *(u8 *)(r5 + 0x0) = r2
  r4 += 0x1
  if r3 > r4 goto loop
  exit
```

- 在Rust `no_std` eBPF 目标下，无论是 `ptr::write_bytes(p, 0, 1)`、`mem::zeroed()`、`*p = T::default()`，还是未加 `write_volatile` 的逐字段清零，都会被 LLVM `MemCpyOpt` Pass 合并成 `call memset(ptr, 0, N)`。
- 6.6 verifier 验证这个逐字节循环时，循环计数器 `r4` 从 `0` 递增到 `N`，每一轮都是不同的精确标量，**完全无法状态剪枝**，一次 `memset(72)` 就要原地展开 72 轮（~360 条验证指令）。
- 在 `wan_egress_l3` 中，入口先调 `l3_link_h_len(&ctx)` 返回 `{0, 14}` 两种可能值，导致后续整个调用树的状态数翻倍；路径上残余的 `ConnState`（72B）、`ConntrackArgs`（56B）、`RedirectEntry`（64B）、`RoutingInput`（96B）、`Tuples`（48B）等各处 `memset` 循环叠加起来，轻松吞掉数百万条验证指令预算。

## 3. 解法（已实测全部 8 个 L2/L3 TC 程序一次通过）

1. **平级子程序分派（Sibling Subprogram Dispatch）**：
   - 将 `do_tproxy_wan_egress` 拆为三个**平级** `#[inline(never)]` 子程序：
     - `wan_egress_prelude(ctx, link_h_len) -> c_long`（栈 64B）：只做包解析与前置过滤，返回 `WAN_CONTINUE_TCP` / `WAN_CONTINUE_UDP` 或直接判决；
     - `do_tproxy_wan_egress_tcp_entry(ctx, link_h_len) -> c_long`（栈 312B）：从 `PKT_SCRATCH_KEY` 取出已解析包，独立完成 TCP 路径；
     - `do_tproxy_wan_egress_udp_entry(ctx, link_h_len) -> c_long`（栈 296B）：从 `PKT_SCRATCH_KEY` 取出已解析包，独立完成 UDP 路径。
   - 三个子程序平级调用、栈帧互不叠加；且全部返回单寄存器 `c_long`（避免 Rust `Result<c_long, c_long>` 用 `r0+r2` 双寄存器返回触发 verifier `R2 !read_ok`）。
2. **`ConnSnapshotSlot` 消除 conntrack 插入时的 80B 栈临时量**：
   - 将 `CONNTRACK_SNAPSHOT_MAP` 的元素类型改为与 `FlowMapValue<ConnState>` 同布局的 `ConnSnapshotSlot { lock_val: u32, padding: u32, value: ConnState }`（80B）。
   - `CONN_STATE_MAP.insert` 直接传入该 per-CPU map 槽位指针，不再在栈上构造 `FlowMapValue::new(*new_state)`，使 `__mark_tcp_seen` 栈深从 144B 降到 80B，`__mark_udp_seen` 从 128B 降到 72B，全链最大栈深降至 **432B <= 512B**。
3. **`zero_in_place_volatile<T>` 彻底消灭大块 `memset` 循环**：
   - 在 `honk-ebpf-common::conn` 提供 `zero_in_place_volatile<T>(ptr: *mut T)`：在编译期按对齐字宽展开为定长 `u64`/`u32`/`u16`/`u8` 的 `write_volatile(0)` 指令序列。
   - `volatile` 阻止 LLVM 将连续 store 重新合并为 `call memset`，同时不在栈上产生任何结构体字面量临时对象。

## 4. `v1.0.7` 进一步根治 6.1 GKI 无 TCX 设备 `wan_egress_l3` 1M 指令超限（Issue #2）

尽管 `v1.0.6` 在 6.6.118 GKI 上仅耗用 `328,888` 指令，部分 **Linux 6.1 GKI**（缺少 TCX 支持、回退到 Netlink `clsact` 且状态剪枝能力更弱）用户反馈在蜂窝 `rmnet`（L3）接口挂载 `wan_egress_l3` 时仍触发：

```text
processed 1000001 insns (limit 1000000) max_states_per_insn 53 total_states 60216 peak_states 4193
stack depth 0+2+64+0+40+16+312+0+80+24+88+0+0+288+56+56
```

### 进一步定位的三大放大因子
1. **静态（`LOCAL`）子程序导致按调用状态重复验证**：`wan_egress_prelude`、`do_tproxy_wan_egress_tcp_entry`、`do_tproxy_wan_egress_udp_entry` 与 `fragment::record` 为 static 子程序，6.1 Verifier 对每个 `(l3_link_h_len ∈ {0, 14})` 及分支组合重复 symbolic execution，`total_states` 膨胀至 60,216。
2. **UDP 路径双份内联 `fast_path_decision`**：`do_tproxy_wan_egress_udp` 在缓存命中与未命中分支各内联了一整份 `fast_path_decision`，使单个 UDP 子程序膨胀至 2,835 条指令。
3. **残余小尺寸 `memset` / `memmove` 循环**：`RedirectTuple` `[u8; 3]` 填充、`[0u8; 6]` MAC 初始化、跨 PerCpuArray 的 ETH/TCP/IPv6 拷贝（14B/20B/16B）以及 `local_udp_reply`/`BpfFibLookup`/`bpf_sock_tuple` 清零仍在热点函数内保留数次 `memset`/`memmove`。

### `v1.0.7` 彻底方案与真机对比
1. **`BTF_FUNC_GLOBAL` 函数级独立验证**：
   - 将 `honk_wan_egress_{prelude,tcp,udp}`、`honk_lan_ingress_{prelude,tcp,udp_route}`（严格以 `(ctx: *mut __sk_buff, is_l2: u32) -> i32` 导出为 `pub unsafe extern "C" fn`）、无 `is_l2` 依赖的 `honk_lan_ingress_udp_pre(ctx: *mut __sk_buff) -> i32`（避免死参数被 LLVM 优化导致 `R2 is not a scalar`），以及 `honk_fragment_record(verdict: i32) -> i32` 升级为 `BTF_FUNC_GLOBAL`。
   - 内核 `btf_prepare_func_args()` 对每个全局函数只独立验证一次并复用签名边界，将复杂度从**乘积**降为**线性加和**。
2. **收敛 `do_tproxy_wan_egress_udp`**：合并缓存命中与路由未命中后的后半程调用，整条函数仅保留 1 份 `fast_path_decision`（2,835 → 1,860 指令）。
3. **路由热路径 `memset`/`memmove` 归零**：通过 `RedirectTuple::write_from_tuples`、`flow::redirect_{identity,incarnation}_for_tuples`、`claim_udp_preparing_scratch`、定长非对齐字拷贝 `copy_unaligned_bytes` 与 `zero_mac()`，使 `wan_egress_l2/l3` 与 `lan_ingress_l2/l3` 全调用链 `memset`/`memmove`/`memcpy` 调用全部清零，最大调用链栈深进一步压低至 **416B**（`wan_egress_*`）/ **384B**（`lan_ingress_*`）。

在真机 Android 16 GKI `6.6.118` 下实测全部 8 个 TC 程序的 `BPF_LOG_STATS` 对比如下：

| TC 程序 | 修复前指令数 (`v1.0.6`) | 修复后指令数 (`v1.0.7`) | 降幅 | 修复前→修复后总状态 |
| --- | ---: | ---: | ---: | ---: |
| `wan_egress_l3` | 328,888 | **128,040** | **-61.1%** | 18,990 → **7,072** |
| `wan_egress_l2` | 174,634 | **127,958** | -26.7% | 9,910 → **7,065** |
| `lan_ingress_l3` | 64,031 | **40,420** | **-36.9%** | 3,419 → **2,199** |
| `lan_ingress_l2` | 42,646 | **40,350** | -5.4% | 2,359 → **2,193** |
| `wan_ingress_l3` | 14,807 | **13,069** | -11.7% | 664 → **608** |
| `wan_ingress_l2` | 10,939 | **9,180** | -16.1% | 506 → **449** |
| `lan_egress_l3` | 16,576 | **14,639** | -11.7% | 754 → **684** |
| `lan_egress_l2` | 10,479 | **9,073** | -13.4% | 514 → **463** |

可见优化后 `wan_egress_l3` 与 `wan_egress_l2` 仅相差 **82 条验证指令**（`0.06%`），L3 链路头长度分叉已不再对下游状态产生任何乘法放大效应。
