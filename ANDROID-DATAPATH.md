# 为什么 honk 的 TC 数据面过不了 6.6 的 verifier

## 现象

```
load TC program 'wan_egress_l2':
  BPF_PROG_LOAD syscall returned Argument list too long (os error 7)
BPF program is too large. Processed 1000001 insn
processed 1000001 insns (limit 1000000) max_states_per_insn 50
  total_states 44810 peak_states 4483 mark_read 116
verification time 3403809 usec
```

cgroup 六个程序、TCX 附着、BTF、freplace、map-in-map 全部正常。
只死在这一个程序上。`BPF_COMPLEXITY_LIMIT_INSNS` 在 6.6 和 7.3 里
**都是 1000000**，所以不是"版本号"的差别。

## 已经试过、都无效的优化

| 改动 | 结果 |
|---|---|
| `opt-level=3` + `inline-threshold=1000` | 无变化（仍 1M） |
| `write_bytes(handoff, 0, 1)` → 结构体赋值 | 无变化 |
| `ParseTransportCtx` 的 `write_bytes` → 逐字段 `write_volatile` | 无变化 |
| 两处 handoff 写入改成逐字段直写（去掉栈临时量） | 无变化 |
| `do_tproxy_wan_egress` 改 `inline(never)` | 1M → **611,514**（有效！但…） |
| 再把 tcp/udp 也 `inline(never)` | 611K 但 **栈超限**：`combined stack size of 4 calls is 544. Too large` |
| 只拆 udp | 又回到 1M |

`inline(never)` 那一档确实把成本砍了 **39%**（1M → 611K），进到了
上限以内 —— 但拆出子程序后 4 层调用链栈和 544 字节，超过 BPF 的
512 字节硬上限。而 BPF 没有"只压栈不拆函数"的开关。

## 为什么 6.12 能过

上游 `doc/en/design/routing.md` 记录，同类策略在 **6.12.107** 上
verifier 只走 53,741–368,874 条指令。同样的字节码、同一个 LLVM，
在 6.6 上要 1M+。差别在 verifier 的状态合并能力：

- 6.6 `regsafe()` 对 `PTR_TO_MAP_VALUE` 只比 `memcmp(rold, rcur, offsetof(id))`
  + `range_within` + `tnum_in` + `check_ids`；
- 6.12/7.3 增加了标量/tnum 的更细粒度子集判定，以及状态剪枝的启发式改进。

结果是同一个带大循环 + 多层条件的数据面程序，6.6 的 verifier 要多走
一个数量级的状态。

## 可行的三条路

1. **写一个 6.6 能过的数据面**（工作量最大，但正路）
   honk 的 TC 程序承担：包解析 → conntrack → 编译路由扩展（freplace）→
   handoff 发布 → 重定向。可以把「编译路由扩展」这块从 TC 程序里挪走：
   TC 只做「解析 + 查表 + 重定向」，路由由用户态在 accept 后算（honk 已有
   用户态 `Router::route_full`，`handoff=None` 时就走它）。

2. **不用 TC，走 iptables TPROXY + 用户态透明 socket**
   实测本内核支持：`iptables -t mangle -j TPROXY --on-port --tproxy-mark`
   rc=0，`ip rule fwmark` / `ip route local table` 都可用。
   honk 已经是 TPROXY 架构（dae0peer + sk_lookup 把包投给 TPROXY 监听），
   把「选哪些包」从 TC 换成 iptables 规则即可。这需要改 honk：
   - 新增 `datapath: iptables` 模式
   - 启动时用 iptables 代替 TC 挂载
   - 路由决策全走用户态

3. **抬 verifier 上限**（不推荐）
   `BPF_COMPLEXITY_LIMIT_INSNS` 是编译期常量，改它等于绕过验证器的
   安全边界。而且 6.6 verifier 本来就慢，抬上去只会让加载时间更长。
