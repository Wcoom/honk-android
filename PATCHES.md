# honk-android：对 honk v0.1.0-beta.6 的改动清单

基线：`daeuniverse/honk` tag `v0.1.0-beta.6`（提交 `8a7611e`）。

## 1. `crates/honk-core/src/ebpf/real/attach.rs` — 放宽内核版本门禁

上游硬性要求 `>= 6.12.0`（理由是「compiled routing 需要同步的 map-in-map 发布」）。
实测 Android GKI 6.6.118 已经具备该语义（`kernel/bpf/syscall.c` 的
`maybe_wait_bpf_programs()` → `synchronize_rcu_expedited()`，专门针对
`ARRAY_OF_MAPS`/`HASH_OF_MAPS`；`BPF_MAP_FREEZE`、`BPF_LINK_CREATE` +
`BPF_PROG_TYPE_EXT` 也都在）。因此把下限降到 6.6，并把真正的能力判定交给
后续的 BTF / freplace / TCX / cgroup attach 检查。
设 `HONK_MIN_KERNEL=1` 可恢复上游的 6.12 门禁。

## 2. `crates/honk-core/src/lib.rs` — `/run` 可配置

Android 没有 `/run`（根分区 erofs 只读，`mkdir /run` 直接 `Read-only file system`），
而 honk 把实例锁写死在 `/run/honk-core.lock`。
改为读 `HONK_RUN_DIR`，未设时仍是 `/run`。

## 3. `crates/honk-core/src/daens.rs` — 新增 `netns_dir()`

同上原因：`/run/netns`、`/var/run/netns` 在 Android 不存在。
新增 `netns_dir()` 返回 `$HONK_RUN_DIR/netns`（未设时 `/run/netns`）。
注：daens 命名空间本身是 **FD 持有**的，这个 bind-mount 只是给 `ip netns exec`
用的兼容设施，挂不上不影响数据面（日志里那两条 WARN 可以忽略）。

## 4. `crates/honk-core/src/ebpf/real/attach.rs` + `tcp_binding.rs` — 错误信息带 OS 错误码

aya 的 `ProgramError::SyscallError` 只打印 `bpf_link_create failed`，
没有 errno。加上 `{error:?}` 后才看得到 `code: 22, InvalidInput`。
（这次排查 cgroup attach 失败全靠它。）

## 5. 构建：`crates/honk-ebpf/.cargo/config.toml` 的实验（已还原）

试过 `opt-level=3` + `inline-threshold=1000` 想压 verifier 成本，**无效**，
已还原为上游原值。

## 未解决：TC 分类器过不了 6.6 的 verifier

`classifier` 段在 Android 6.6.118 上 `BPF program is too large. Processed 1000001 insn`
（撞 `BPF_COMPLEXITY_LIMIT_INSNS = 1000000`）。上游自己的设计文档
`doc/en/design/routing.md` 记录同类策略在 6.12.107 上是 53,741–368,874 条指令，
说明 6.6 的 verifier 状态合并能力明显弱于 6.12，同一份字节码成本翻了几十倍。
cgroup/TCX 部分都能挂上，卡点只在这一个程序。
