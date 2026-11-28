# Port map: boxbpf

2 of 5 units have a Rust port; no original changed since it was recorded.

## Ports

| Unit | Crate | In Rust | Kept in the original | Cases | Last diff | Original |
| --- | --- | --- | --- | ---: | ---: | --- |
| `config.c` | `rust/config-rs` | `load_cidr_file`, `load_uid_file`, `uid_file_has_entries` | `config_defaults`, `read_config_json` | 152 | 152/152 | unchanged |
| `loader.c` | `rust/loader-rs` | `create_map`, `clear_map`, `get_pinned`, `pin_replace`, `close_fd`, `remove_known_pins`, `load_program` | `bpf_call`, `update_elem` | 313 | 313/313 | unchanged |

## Whole app

| Glue | Runs |
| --- | --- |
| `rust/app/src/units/config.rs` | Rust only |
| `rust/app/src/units/loader.rs` | Rust only |

## Not ported yet

In the order `analyze` suggests.

| Unit | Status | Lines | Exports | Readiness |
| --- | --- | ---: | ---: | ---: |
| `compat.h` | ready | 31 | 0 | 94 |
| `common.h` | ready | 87 | 0 | 73 |
| `main.c` | caution | 220 | 0 | 47 |

Regenerate with `rust-anything map --write`. Commit this file: when the original changes upstream, the next map marks every port whose recording it outdated.
