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
