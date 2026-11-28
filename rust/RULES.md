# Port Rules (`honk-android`)

- 严禁将任何密码、PIN、Token、Cookie、私钥或真实订阅节点写入代码、测试用例、文档、日志或提交中。
- `boxbpf` 的 CLI 参数、退出码（`0`/`1`/`2`/`3`）、stdout `key=value` 格式以及被 `boxctl/src/rules/cnip.rs` 解析的 stderr 错误关键词（`eBPF program load rejected`、`Permission denied`、`Operation not permitted`、`No such file` 等）必须与原 C 版保持完全一致。
- `boxbpf` 生成的 `BPF_PROG_TYPE_SOCKET_FILTER` 指令序列、栈偏移、LPM_TRIE key 布局与 `READ_DIRECT -> READ_HELPER -> READ_LDABS` 回退顺序必须与原 C 版等价，确保在 Android GKI 6.1/6.6 内核 verifier 与 `xt_bpf` 下零回归。
- 严禁触碰、暂存、还原或删除工作区既有用户文件：`.gitignore`、`build-cf-opt.sh`、`cf-opt/`。
- `rust/DECISIONS.md` 中的已接受 ADR 为唯一架构真源；变更决策须新增 ADR 并标记 `superseded by`。
- 所有单元移植必须以 `rust-anything diff` 全量通过与 `rust-anything gate` 通过为完成标准，严禁篡改 `cases.jsonl` 或跳过失败用例。
