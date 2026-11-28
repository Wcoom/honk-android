# Research: `boxbpf` Linux Kernel BPF Syscall & ABI Contract

Primary sources verified against host Linux UAPI headers (`/usr/include/linux/bpf.h`, `/usr/include/x86_64-linux-gnu/sys/syscall.h`, `/usr/include/x86_64-linux-gnu/sys/resource.h`) and Android musl/NDK kernel headers (`boxproxy/boxbpf/{compat.h,common.h,loader.c,main.c}`).

## 1. `<linux/bpf.h>` → Rust `#[repr(C)]` UAPI Structures (`libc` + zero-C-dep ABI)

| Symbol / Struct | Verified C Value / Size / Offset | Rust Strategy |
| --- | --- | --- |
| `sizeof(struct bpf_insn)` | `8` bytes (`code: u8, regs: u8, off: i16, imm: i32`) | `#[repr(C)] struct BpfInsn` with `regs = (src_reg << 4) | (dst_reg & 0x0f)` (little-endian bitfield order on both `x86_64` and `aarch64` Android!). |
| `sizeof(union bpf_attr)` | `168` bytes on Linux 6.x (`>= 120` on 5.x/6.1) | Fixed `[u8; 168]` zero-initialized buffer or `#[repr(C)]` per-command sub-structs sized to 168 bytes (kernel `bpf_attr` copies up to `size` and checks trailing bytes are zero — passing exact command struct size or 168-byte zero-padded buffer works on Linux 4.14–6.12+). |
| `BPF_MAP_CREATE` | `0` (`map_type@0`, `key_size@4`, `value_size@8`, `max_entries@12`, `map_flags@16`) | `#[repr(C)] struct BpfMapCreateAttr` |
| `BPF_MAP_UPDATE_ELEM` | `2` (`map_fd@0`, `key@8`, `value@16`, `flags@24`) | `#[repr(C)] struct BpfMapElemAttr` |
| `BPF_MAP_DELETE_ELEM` | `3` (`map_fd@0`, `key@8`) | `#[repr(C)] struct BpfMapElemAttr` |
| `BPF_MAP_GET_NEXT_KEY` | `4` (`map_fd@0`, `key@8`, `next_key@16`) | `#[repr(C)] struct BpfMapElemAttr` |
| `BPF_PROG_LOAD` | `5` (`prog_type@0`, `insn_cnt@4`, `insns@8`, `license@16`, `log_level@24`, `log_size@28`, `log_buf@32`, `prog_name@48..64`) | `#[repr(C)] struct BpfProgLoadAttr` |
| `BPF_OBJ_PIN` / `BPF_OBJ_GET` | `6` / `7` (`pathname@0`, `bpf_fd@8`) | `#[repr(C)] struct BpfObjAttr` |
| `BPF_MAP_TYPE_HASH` / `BPF_MAP_TYPE_LPM_TRIE` | `1` / `11` | `const BPF_MAP_TYPE_HASH: u32 = 1; const BPF_MAP_TYPE_LPM_TRIE: u32 = 11;` |
| `BPF_PROG_TYPE_SOCKET_FILTER` | `1` | `const BPF_PROG_TYPE_SOCKET_FILTER: u32 = 1;` |
| `BPF_ANY` / `BPF_F_NO_PREALLOC` | `0` / `1` | `const BPF_ANY: u64 = 0; const BPF_F_NO_PREALLOC: u32 = 1;` |
| `BPF_FUNC_map_lookup_elem` | `1` | Helper ID `1` |
| `BPF_FUNC_skb_load_bytes` | `26` | Helper ID `26` |
| `BPF_FUNC_get_socket_uid` | `47` | Helper ID `47` |
| `offsetof(struct __sk_buff, data)` | `76` | Exact constant `76` (UAPI `u32` array independent of arch) |
| `offsetof(struct __sk_buff, data_end)` | `80` | Exact constant `80` |
| `struct lpm4_key` / `struct lpm6_key` | `8` B (`prefixlen: u32, addr: [u8; 4]`) / `20` B (`prefixlen: u32, addr: [u8; 16]`) | `#[repr(C)]` exact layout |

### Semantic Traps Found
1. **Little-endian bitfield packing in `struct bpf_insn`**: In Linux `<linux/bpf.h>`, `__u8 dst_reg:4; __u8 src_reg:4;` puts `dst_reg` in the **low** 4 bits (`0x0f`) and `src_reg` in the **high** 4 bits (`0xf0`) on little-endian (`x86_64`, `aarch64`). Swapping nibbles will cause immediate BPF verifier rejection.
2. **64-bit `BPF_LD | BPF_DW | BPF_IMM` (`emit_map_fd`)**: Takes **two** 8-byte `bpf_insn` slots (`src_reg = BPF_PSEUDO_MAP_FD (1)`, `imm = map_fd`, followed by an all-zero second `bpf_insn`). Jump offsets (`off = count - jump_index - 1`) count 8-byte instruction slots including the second slot!
3. **`prog_name` truncation**: C `snprintf(attr.prog_name, 16, "%s", name)` copies at most 15 bytes + NUL terminator (`BPF_OBJ_NAME_LEN = 16`).
4. **3-tier read fallback (`READ_DIRECT` → `READ_HELPER` → `READ_LDABS`)**: Only logs the verifier rejection message on the **final** attempt (`i + 1 == count`), preserving the exact stderr phrase `eBPF program load rejected: program=... errno=...` parsed by `boxctl/src/rules/cnip.rs:611`.

## 2. `<sys/syscall.h>` → `libc::syscall(libc::SYS_bpf, ...)`
- `libc::SYS_bpf` resolves to `321` on `x86_64-unknown-linux-gnu` and `280` on `aarch64-unknown-linux-musl` / `aarch64-linux-android` automatically.
- All raw syscalls are isolated in one thin module with explicit `// SAFETY:` comments, returning `std::io::Error::last_os_error()` (whose `raw_os_error()` gives exact `errno` and `libc::strerror` / `io::Error` formatting matches C).

## 3. `<sys/resource.h>` → `libc::setrlimit(libc::RLIMIT_MEMLOCK, ...)`
- `libc::rlimit { rlim_cur: libc::RLIM_INFINITY, rlim_max: libc::RLIM_INFINITY }` passed to `libc::setrlimit(libc::RLIMIT_MEMLOCK, &unlimited)`. Failure is ignored (identical to C `raise_memlock()` in `main.c:13-16`).
