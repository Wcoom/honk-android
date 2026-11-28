# Rust migration plan: webui (TypeScript/JavaScript)

2 packages, 451 source files (82229 lines), 386 test files, 329 UI files, grouped into 98 units.

| Ready | Caution | Blocked | Keep in source language |
| ---: | ---: | ---: | ---: |
| 22 | 10 | 2 | 64 |

## Start here

Best first ports: safe to port (readiness) and worth porting (loops and string work, tests, size).

| # | Unit | Lines | Priority | Readiness | Tests | Bridge | Main risks |
| ---: | --- | ---: | ---: | ---: | ---: | --- | --- |
| 1 | `apps/lite/src/groupDraft.ts` | 114 | 89 | 79 | 2 | wasm-bindgen | String offsets and slicing (12); Insertion-ordered maps and object keys (2); Exceptions (10) |
| 2 | `apps/lite/src/ruleReview.ts` | 51 | 87 | 100 | 1 | wasm-bindgen | String offsets and slicing (1) |
| 3 | `apps/lite/src/searchIndex.ts` | 91 | 86 | 100 | 1 | wasm-bindgen | Insertion-ordered maps and object keys (2) |
| 4 | `apps/lite/src/config.ts` | 135 | 81 | 61 | 3 | wasm-bindgen | String offsets and slicing (16); Async and promises (12); Insertion-ordered maps and object keys (1) |
| 5 | `apps/lite/src/routingTest.ts` | 69 | 80 | 89 | 1 | wasm-bindgen | String offsets and slicing (2); Async and promises (9); Insertion-ordered maps and object keys (1) |
| 6 | `apps/lite/src/ruleDraft.ts` | 226 | 80 | 60 | 2 | wasm-bindgen | String offsets and slicing (36); Regex features the `regex` crate lacks (1); Insertion-ordered maps and object keys (1) |
| 7 | `apps/lite/src/resourceDraft.ts` | 46 | 79 | 100 | 1 | wasm-bindgen | String offsets and slicing (2); Exceptions (5) |
| 8 | `apps/lite/src/demo/` | 52 | 75 | 100 | 5 | wasm-bindgen | - |
| 9 | `apps/lite/src/groupBindings.ts` | 91 | 74 | 94 | 3 | wasm-bindgen | Insertion-ordered maps and object keys (2) |
| 10 | `apps/lite/src/operations.ts` | 54 | 68 | 93 | 1 | wasm-bindgen | Async and promises (5); Callback-shaped APIs (1); Exceptions (5) |

### `apps/lite/src/groupDraft.ts`

114 lines in 1 file, wave 5, priority 89, readiness 79, bridge **wasm-bindgen**.

- Import cycle with apps/lite/src/ui/motion/, apps/lite/src/ui.tsx, apps/lite/src/describe.ts, apps/lite/src/ui/, apps/lite/src/config.ts, apps/lite/src/api.ts: port them together.
- Imports `apps/lite/src/api.ts`, `apps/lite/src/config.ts`, `apps/lite/src/groupBindings.ts`, `src/dae/`: port them first, or port them together.
- 2 test files import it directly.
- Portable code: wasm-bindgen keeps it running in browsers and Node. It needs the wasm32-unknown-unknown target and wasm-bindgen; `bridge --kind napi` runs with what Node already has.
- String offsets and slicing x12: `apps/lite/src/groupDraft.ts:19`, `apps/lite/src/groupDraft.ts:27`, `apps/lite/src/groupDraft.ts:31`
- Insertion-ordered maps and object keys x2: `apps/lite/src/groupDraft.ts:23`, `apps/lite/src/groupDraft.ts:48`
- Exceptions x10: `apps/lite/src/groupDraft.ts:17`, `apps/lite/src/groupDraft.ts:20`, `apps/lite/src/groupDraft.ts:23`

### `apps/lite/src/ruleReview.ts`

51 lines in 1 file, wave 6, priority 87, readiness 100, bridge **wasm-bindgen**.

- Imports `apps/lite/src/describe.ts`, `apps/lite/src/i18n/`, `apps/lite/src/ruleDraft.ts`, `src/dae/`: port them first, or port them together.
- 1 test file imports it directly.
- Portable code: wasm-bindgen keeps it running in browsers and Node. It needs the wasm32-unknown-unknown target and wasm-bindgen; `bridge --kind napi` runs with what Node already has.
- String offsets and slicing x1: `apps/lite/src/ruleReview.ts:48`

### `apps/lite/src/searchIndex.ts`

91 lines in 1 file, wave 6, priority 86, readiness 100, bridge **wasm-bindgen**.

- Imports `apps/lite/src/regions.ts`, `apps/lite/src/ruleDraft.ts`, `apps/lite/src/ui/`, `src/api/`, `src/dae/`: port them first, or port them together.
- 1 test file imports it directly.
- Portable code: wasm-bindgen keeps it running in browsers and Node. It needs the wasm32-unknown-unknown target and wasm-bindgen; `bridge --kind napi` runs with what Node already has.
- Insertion-ordered maps and object keys x2: `apps/lite/src/searchIndex.ts:14`

### `apps/lite/src/config.ts`

135 lines in 1 file, wave 5, priority 81, readiness 61, bridge **wasm-bindgen**.

- Import cycle with apps/lite/src/ui/motion/, apps/lite/src/ui.tsx, apps/lite/src/groupDraft.ts, apps/lite/src/describe.ts, apps/lite/src/ui/, apps/lite/src/api.ts: port them together.
- Imports `apps/lite/src/api.ts`, `src/api/`, `src/dae/`: port them first, or port them together.
- 3 test files import it directly.
- Portable code: wasm-bindgen keeps it running in browsers and Node. It needs the wasm32-unknown-unknown target and wasm-bindgen; `bridge --kind napi` runs with what Node already has.
- String offsets and slicing x16: `apps/lite/src/config.ts:72`, `apps/lite/src/config.ts:73`, `apps/lite/src/config.ts:75`
- Async and promises x12: `apps/lite/src/config.ts:17`, `apps/lite/src/config.ts:21`, `apps/lite/src/config.ts:28`
- Insertion-ordered maps and object keys x1: `apps/lite/src/config.ts:100`
- Exceptions x9: `apps/lite/src/config.ts:41`, `apps/lite/src/config.ts:42`, `apps/lite/src/config.ts:44`
- JSON round-trips x2: `apps/lite/src/config.ts:51`, `apps/lite/src/config.ts:85`

### `apps/lite/src/routingTest.ts`

69 lines in 1 file, wave 6, priority 80, readiness 89, bridge **wasm-bindgen**.

- Imports `apps/lite/src/api.ts`, `apps/lite/src/ruleDraft.ts`, `src/api/`, `src/dae/`: port them first, or port them together.
- 1 test file imports it directly.
- Portable code: wasm-bindgen keeps it running in browsers and Node. It needs the wasm32-unknown-unknown target and wasm-bindgen; `bridge --kind napi` runs with what Node already has.
- String offsets and slicing x2: `apps/lite/src/routingTest.ts:42`, `apps/lite/src/routingTest.ts:67`
- Async and promises x9: `apps/lite/src/routingTest.ts:27`, `apps/lite/src/routingTest.ts:33`, `apps/lite/src/routingTest.ts:34`
- Insertion-ordered maps and object keys x1: `apps/lite/src/routingTest.ts:39`
- Exceptions x7: `apps/lite/src/routingTest.ts:9`, `apps/lite/src/routingTest.ts:11`, `apps/lite/src/routingTest.ts:14`

## Order by package

Packages come after the workspace packages they use. Inside a package, port by wave: a unit's imports are in earlier waves.

### 1. doona (`.`, portable, 367 test files)

| Wave | Unit | Status | Lines | Readiness | Bridge | Main risks |
| ---: | --- | --- | ---: | ---: | --- | --- |
| 0 | `tools/e2e-shards-fixture/` | ready | 10 | 100 | wasm-bindgen | - |
| 0 | `src/fonts.ts` | blocked | 18 | 93 | wasm-bindgen | Dynamic code (eval, new Function, computed import) (1); Async and promises (2) |
| 0 | `public/` | ready | 74 | 85 | wasm-bindgen | String offsets and slicing (2); Async and promises (20) |
| 1 | `src/api/engines/` | caution | 387 | 70 | wasm-bindgen | String offsets and slicing (2); Callback-shaped APIs (7) |
| 1 | `src/i18n/` | keep | 619 | 43 | keep | String offsets and slicing (4); Insertion-ordered maps and object keys (24) |
| 1 | `mock/` | caution | 3383 | 0 | wasm-bindgen | String offsets and slicing (52); Regexes built at runtime (1) |
| 1 | `src/api/` | caution | 7099 | 0 | wasm-bindgen | String offsets and slicing (26); Regex features the `regex` crate lacks (5) |
| 1 | `src/dae/` | caution | 3525 | 0 | wasm-bindgen | String offsets and slicing (276); Regex features the `regex` crate lacks (7) |
| 2 | `src/ui/icons/` | keep | 711 | 71 | keep | - |
| 2 | `src/ui/charts/` | keep | 1868 | 17 | keep | String offsets and slicing (5); Callback-shaped APIs (17) |
| 2 | `src/ui/code/` | keep | 624 | 16 | keep | String offsets and slicing (10); Regex features the `regex` crate lacks (3) |
| 2 | `src/store/` | keep | 2687 | 0 | keep | String offsets and slicing (4); Async and promises (138) |
| 2 | `src/ui/` | keep | 7309 | 0 | keep | String offsets and slicing (13); Regex features the `regex` crate lacks (1) |
| 2 | `tools/` | blocked | 2713 | 0 | napi-rs | String offsets and slicing (43); Regexes built at runtime (4) |
| 3 | `src/features/events/` | keep | 239 | 95 | keep | String offsets and slicing (1); Callback-shaped APIs (1) |
| 3 | `src/features/overview/` | keep | 632 | 88 | keep | Insertion-ordered maps and object keys (2); JSON round-trips (1) |
| 3 | `src/features/logs/` | keep | 453 | 80 | keep | String offsets and slicing (4); Insertion-ordered maps and object keys (4) |
| 3 | `src/shell/search/` | keep | 427 | 77 | keep | String offsets and slicing (2); Insertion-ordered maps and object keys (5) |
| 3 | `src/features/activity/` | keep | 1348 | 49 | keep | String offsets and slicing (1); Callback-shaped APIs (16) |
| 3 | `src/features/policies/group/` | keep | 1186 | 47 | keep | String offsets and slicing (4); Callback-shaped APIs (33) |
| 3 | `src/features/flows/` | keep | 1155 | 43 | keep | String offsets and slicing (7); Insertion-ordered maps and object keys (28) |
| 3 | `src/features/connections/` | keep | 1273 | 42 | keep | String offsets and slicing (2); Insertion-ordered maps and object keys (9) |
| 3 | `src/features/policies/` | keep | 1601 | 37 | keep | Callback-shaped APIs (20); Insertion-ordered maps and object keys (14) |
| 3 | `src/features/dns/` | keep | 1421 | 33 | keep | String offsets and slicing (4); Regexes built at runtime (1) |
| 3 | `src/features/config/` | keep | 2071 | 14 | keep | String offsets and slicing (6); Callback-shaped APIs (17) |
| 3 | `src/features/nodes/` | keep | 3487 | 0 | keep | String offsets and slicing (4); Callback-shaped APIs (58) |
| 3 | `src/features/rules/` | keep | 3288 | 0 | keep | String offsets and slicing (12); Regex features the `regex` crate lacks (1) |
| 3 | `src/features/settings/` | keep | 2762 | 0 | keep | String offsets and slicing (4); Async and promises (29) |
| 3 | `src/features/shared/` | keep | 2674 | 0 | keep | String offsets and slicing (16); Async and promises (41) |
| 3 | `src/shell/` | keep | 3154 | 0 | keep | String offsets and slicing (9); Callback-shaped APIs (78) |
| 3 | `src/shell/widgets/` | keep | 2760 | 0 | keep | String offsets and slicing (13); Callback-shaped APIs (39) |
| 4 | `src/main.tsx` | keep | 135 | 89 | keep | Async and promises (5); Dynamic typing (1) |

### 2. doona-lite (`apps/lite`, portable, 19 test files)

| Wave | Unit | Status | Lines | Readiness | Bridge | Main risks |
| ---: | --- | --- | ---: | ---: | --- | --- |
| 0 | `apps/lite/src/systemFonts.ts` | ready | 1 | 100 | wasm-bindgen | Async and promises (1) |
| 0 | `apps/lite/src/connectionStore.ts` | ready | 19 | 94 | wasm-bindgen | Dynamic typing (2); JSON round-trips (2) |
| 0 | `apps/lite/src/ui/icons/` | keep | 561 | 76 | keep | - |
| 0 | `apps/lite/tools/` | ready | 150 | 76 | napi-rs | String offsets and slicing (4); Regex features the `regex` crate lacks (2) |
| 0 | `apps/lite/src/i18n/` | keep | 2231 | 39 | keep | - |
| 0 | `apps/lite/scripts/` | caution | 4702 | 0 | napi-rs | String offsets and slicing (17); Regex features the `regex` crate lacks (2) |
| 1 | `apps/lite/src/i18n/settings/` | ready | 133 | 99 | wasm-bindgen | - |
| 1 | `apps/lite/src/app/` | keep | 10 | 94 | keep | Callback-shaped APIs (2) |
| 1 | `apps/lite/src/themes/shared/` | keep | 187 | 94 | keep | - |
| 1 | `apps/lite/src/state.ts` | keep | 107 | 78 | keep | Async and promises (4); Callback-shaped APIs (4) |
| 2 | `apps/lite/src/demo/` | ready | 52 | 100 | wasm-bindgen | - |
| 2 | `apps/lite/src/fetchFailure.ts` | ready | 18 | 100 | wasm-bindgen | - |
| 2 | `apps/lite/src/groupBindings.ts` | ready | 91 | 94 | wasm-bindgen | Insertion-ordered maps and object keys (2) |
| 2 | `apps/lite/src/operations.ts` | ready | 54 | 93 | wasm-bindgen | Async and promises (5); Callback-shaped APIs (1) |
| 2 | `apps/lite/src/themes/site/` | ready | 786 | 77 | wasm-bindgen | - |
| 4 | `apps/lite/src/regions.ts` | ready | 5 | 100 | wasm-bindgen | - |
| 4 | `apps/lite/src/groupHealth.ts` | ready | 32 | 98 | wasm-bindgen | Insertion-ordered maps and object keys (1) |
| 4 | `apps/lite/src/docs.ts` | ready | 6 | 95 | wasm-bindgen | - |
| 4 | `apps/lite/src/ruleDraft.ts` | ready | 226 | 60 | wasm-bindgen | String offsets and slicing (36); Regex features the `regex` crate lacks (1) |
| 5 | `apps/lite/src/groupDraft.ts` | ready | 114 | 79 | wasm-bindgen | String offsets and slicing (12); Insertion-ordered maps and object keys (2) |
| 5 | `apps/lite/src/ui.tsx` | keep | 128 | 77 | keep | Dynamic typing (5); Insertion-ordered maps and object keys (2) |
| 5 | `apps/lite/src/describe.ts` | ready | 255 | 72 | wasm-bindgen | String offsets and slicing (5); Dynamic typing (4) |
| 5 | `apps/lite/src/ui/motion/` | keep | 350 | 69 | keep | Callback-shaped APIs (5); Insertion-ordered maps and object keys (1) |
| 5 | `apps/lite/src/api.ts` | caution | 196 | 65 | wasm-bindgen | String offsets and slicing (1); Async and promises (28) |
| 5 | `apps/lite/src/config.ts` | ready | 135 | 61 | wasm-bindgen | String offsets and slicing (16); Async and promises (12) |
| 5 | `apps/lite/src/ui/` | keep | 1761 | 0 | keep | String offsets and slicing (6); Callback-shaped APIs (30) |
| 6 | `apps/lite/src/connectionStartup.ts` | caution | 23 | 100 | wasm-bindgen | Async and promises (5); Exceptions (2) |
| 6 | `apps/lite/src/groupMembers.ts` | caution | 54 | 100 | wasm-bindgen | - |
| 6 | `apps/lite/src/resourceDraft.ts` | ready | 46 | 100 | wasm-bindgen | String offsets and slicing (2); Exceptions (5) |
| 6 | `apps/lite/src/ruleReview.ts` | ready | 51 | 100 | wasm-bindgen | String offsets and slicing (1) |
| 6 | `apps/lite/src/searchIndex.ts` | ready | 91 | 100 | wasm-bindgen | Insertion-ordered maps and object keys (2) |
| 6 | `apps/lite/src/subscriptionDraft.ts` | caution | 47 | 95 | wasm-bindgen | Async and promises (4); Dynamic typing (1) |
| 6 | `apps/lite/src/CopyMembers.tsx` | keep | 64 | 94 | keep | Callback-shaped APIs (2) |
| 6 | `apps/lite/src/Demo.tsx` | keep | 36 | 93 | keep | Async and promises (1); Callback-shaped APIs (1) |
| 6 | `apps/lite/src/SubscriptionFeatureNodes.tsx` | keep | 101 | 93 | keep | Callback-shaped APIs (2) |
| 6 | `apps/lite/src/nodeProbeState.ts` | caution | 96 | 92 | wasm-bindgen | Async and promises (6); Insertion-ordered maps and object keys (3) |
| 6 | `apps/lite/src/useFeatureDraft.tsx` | keep | 82 | 92 | keep | Async and promises (5); Insertion-ordered maps and object keys (2) |
| 6 | `apps/lite/src/AddNodeToGroup.tsx` | keep | 120 | 91 | keep | Callback-shaped APIs (2) |
| 6 | `apps/lite/src/GroupEditor.tsx` | keep | 70 | 91 | keep | Callback-shaped APIs (2); Insertion-ordered maps and object keys (1) |
| 6 | `apps/lite/src/SubscriptionFormInterval.tsx` | keep | 103 | 90 | keep | String offsets and slicing (1); Callback-shaped APIs (1) |
| 6 | `apps/lite/src/GroupTextDialog.tsx` | keep | 67 | 89 | keep | String offsets and slicing (1); Callback-shaped APIs (2) |
| 6 | `apps/lite/src/ShareDialog.tsx` | keep | 67 | 89 | keep | Callback-shaped APIs (1) |
| 6 | `apps/lite/src/routingTest.ts` | ready | 69 | 89 | wasm-bindgen | String offsets and slicing (2); Async and promises (9) |
| 6 | `apps/lite/src/ConfigExport.tsx` | keep | 88 | 87 | keep | String offsets and slicing (1); Async and promises (2) |
| 6 | `apps/lite/src/AboutPanel.tsx` | keep | 219 | 84 | keep | Async and promises (10); Exceptions (3) |
| 6 | `apps/lite/src/NodeForm.tsx` | keep | 364 | 78 | keep | String offsets and slicing (1); Insertion-ordered maps and object keys (2) |
| 6 | `apps/lite/src/DnsUpstreams.tsx` | keep | 292 | 70 | keep | String offsets and slicing (15); Callback-shaped APIs (3) |
| 6 | `apps/lite/src/GroupInspector.tsx` | keep | 616 | 63 | keep | String offsets and slicing (1); Callback-shaped APIs (6) |
| 6 | `apps/lite/src/RuleList.tsx` | keep | 576 | 61 | keep | String offsets and slicing (5); Callback-shaped APIs (7) |
| 7 | `apps/lite/src/NodeProbes.tsx` | keep | 72 | 95 | keep | Number semantics (1) |
| 7 | `apps/lite/src/TemplatePicker.tsx` | keep | 71 | 91 | keep | Callback-shaped APIs (2); Insertion-ordered maps and object keys (1) |
| 7 | `apps/lite/src/GroupBindPopover.tsx` | keep | 165 | 87 | keep | Insertion-ordered maps and object keys (5); Callback-shaped APIs (1) |
| 7 | `apps/lite/src/RuleTester.tsx` | keep | 200 | 86 | keep | Async and promises (2); Callback-shaped APIs (2) |
| 7 | `apps/lite/src/Commands.tsx` | keep | 284 | 75 | keep | Async and promises (11); Callback-shaped APIs (4) |
| 7 | `apps/lite/src/Connect.tsx` | keep | 305 | 70 | keep | Async and promises (11); Insertion-ordered maps and object keys (3) |
| 7 | `apps/lite/src/SettingsDialog.tsx` | keep | 532 | 56 | keep | String offsets and slicing (2); Async and promises (7) |
| 8 | `apps/lite/src/GroupEditPanel.tsx` | keep | 471 | 63 | keep | String offsets and slicing (4); Insertion-ordered maps and object keys (4) |
| 8 | `apps/lite/src/Subscriptions.tsx` | keep | 599 | 61 | keep | Async and promises (19); Dynamic typing (4) |
| 8 | `apps/lite/src/Nodes.tsx` | keep | 786 | 49 | keep | String offsets and slicing (1); Async and promises (15) |
| 9 | `apps/lite/src/Groups.tsx` | keep | 608 | 57 | keep | String offsets and slicing (2); Async and promises (22) |
| 10 | `apps/lite/src/preload.ts` | keep | 75 | 92 | keep | Async and promises (2); Dynamic typing (1) |
| 10 | `apps/lite/src/Dns.tsx` | keep | 231 | 82 | keep | Async and promises (5); Dynamic typing (1) |
| 10 | `apps/lite/src/Routing.tsx` | keep | 405 | 77 | keep | Async and promises (9); Dynamic typing (1) |
| 11 | `apps/lite/src/Shell.tsx` | keep | 348 | 78 | keep | Async and promises (6); Callback-shaped APIs (4) |
| 12 | `apps/lite/src/App.tsx` | keep | 319 | 79 | keep | String offsets and slicing (2); Insertion-ordered maps and object keys (2) |
| 13 | `apps/lite/src/main.tsx` | keep | 13 | 96 | keep | - |

## npm dependencies

| Package | Units | Rust | Fit | Note |
| --- | ---: | --- | --- | --- |
| `@playwright/test` | 3 | ? | unknown |  |
| `@astryxdesign/core` | 1 | ? | unknown |  |
| `ajv-formats` | 1 | ? | unknown |  |
| `openapi-fetch` | 1 | ? | unknown |  |
| `typescript` | 2 | oxc | partial | Parsing is covered. Type checking has no Rust equivalent. |
| `yaml` | 1 | saphyr, serde_yaml_ng | partial | Parsing is covered. No crate rewrites YAML while keeping comments and layout. |
| `ajv` | 1 | jsonschema | good |  |

## Blocked

- `src/fonts.ts`: Uses dynamic code (eval, new Function or computed imports).
- `tools/`: Uses dynamic code (eval, new Function or computed imports).

## Risk guide

- **String offsets and slicing** (High): Rust slices strings at UTF-8 byte offsets that fall on character boundaries. Convert offsets from the source's indexing units as needed and test with non-ASCII text.
- **Regex features the `regex` crate lacks** (High): Lookaround, backreferences and the `y` flag are not in the `regex` crate. Use `fancy-regex` for those patterns, or rewrite them, and test them on non-ASCII text: `\w`, `\b` and case-insensitive matching differ between engines even where a pattern compiles. JS `/i` without `u` never matches a non-ASCII letter to an ASCII one, Rust `(?i)` does (`ſ` matches `s`, the Kelvin sign `K` matches `k`): write `(?i-u)` or compare with `eq_ignore_ascii_case`.
- **Regexes built at runtime** (High): Patterns come from data or config in JS syntax. Rust regex syntax differs, so user-visible behavior can change. Pin it with tests on real patterns.
- **Dynamic code (eval, new Function, computed import)** (High): eval, new Function and computed imports have no Rust equivalent. Keep this part in JS or replace it with an explicit registry.
- **Dynamic typing** (Medium): Values whose shape is checked at runtime need an explicit Rust enum or `serde_json::Value`.
- **Insertion-ordered maps and object keys** (Medium): JS Map, Set and object keys keep insertion order. `HashMap` does not: use `indexmap` wherever order reaches output. With serde_json's `preserve_order`, `Map::remove` swaps the last key into the hole: use `shift_remove` for JS `delete`. Sort with `js::sort` when a comparator can return NaN: the order then depends on V8's algorithm.
- **Exceptions** (Low): Represent source exceptions with `Result`. Preserve the original exception type and message when errors cross the bridge.
- **Async and promises** (Medium): Promises become futures. Bridge with napi-rs `async fn` or `wasm-bindgen-futures`, or keep the async shell in JS and port the sync core.
- **Class inheritance** (Medium): Class hierarchies become traits plus composition; `instanceof` checks become enums.
- **Reflection and prototypes** (Medium): Proxy, Reflect, Symbol and prototype tricks need a redesign, not a translation.
- **Callback-shaped APIs** (Medium): Function-typed parameters and methods cross the bridge on every call. Keep boundaries coarse: pass data in, get data out.
- **Number semantics** (Low): JS numbers are f64; bitwise ops truncate to i32; parseInt is lenient. Pick Rust integer types deliberately. Printing differs: `${n}` gives `1e+21`, `1e-7` and `0` for -0 where Rust's `{}` does not, so use `js::number_to_string`. `Math.round` rounds halves up (-2.5 gives -2), Rust's `round` away from zero: use `js::math_round`. `toFixed` rounds exact halves up ((2.5).toFixed(0) is "3"), Rust's `{:.0}` to even.
- **JSON round-trips** (Low): JSON.stringify key order and number formatting must match `serde_json` output if the result is user-visible.
- **Clock and random numbers** (Low): Results that depend on the time or on random numbers differ on every run, so a plain recording cannot pin them down. Record with `record --pin-clock` (TS/JS with the napi bridge, Python): `Date.now()`, `new Date()` and `Math.random` (`time.time`, `datetime.now` and `random`) then return fixed values during record, diff and gate, and the port reads the same ones from `src/port/pinned.rs` (`now_ms()`, `random()`, a documented mulberry32). Or pass the time and the seed in as parameters.

## Whole app

Shape: **library**.

| Entry point | Kind | Found from |
| --- | --- | --- |
| doona: `.` | library | no main and no server: the package's public API |
| doona-lite: `apps/lite` | library | no main and no server: the package's public API |

### Every dependency

| Dependency | Declared | Used by | Rust | Fit | Note |
| --- | --- | --- | --- | --- | --- |
| `@astryxdesign/core` | yes | 35 | ? | unknown |  |
| `react-aria-components` | yes | 7 | ? | unknown |  |
| `@stylexjs/stylex` | yes | 6 | ? | unknown |  |
| `@playwright/test` | no | 3 | ? | unknown |  |
| `react-aria` | yes | 3 | ? | unknown |  |
| `@codemirror/autocomplete` | yes | 2 | ? | unknown |  |
| `@codemirror/commands` | yes | 2 | ? | unknown |  |
| `@codemirror/language` | yes | 2 | ? | unknown |  |
| `@codemirror/lint` | yes | 2 | ? | unknown |  |
| `@codemirror/search` | yes | 2 | ? | unknown |  |
| `@codemirror/state` | yes | 2 | ? | unknown |  |
| `@codemirror/view` | yes | 2 | ? | unknown |  |
| `@lezer/highlight` | yes | 2 | ? | unknown |  |
| `react-dom` | yes | 2 | ? | unknown |  |
| `ajv-formats` | no | 1 | ? | unknown |  |
| `d3-shape` | yes | 1 | ? | unknown |  |
| `openapi-fetch` | yes | 1 | ? | unknown |  |
| `qrcode.react` | yes | 1 | ? | unknown |  |
| `vitest` | no | 1 | ? | unknown |  |
| `@astryxdesign/theme-neutral` | yes | 0 | ? | unknown |  |
| `@fontsource-variable/figtree` | yes | 0 | ? | unknown |  |
| `@fontsource-variable/noto-sans-sc` | yes | 0 | ? | unknown |  |
| `@fontsource-variable/noto-sans-tc` | yes | 0 | ? | unknown |  |
| `react` | yes | 0 | ? | unknown |  |
| `typescript` | no | 2 | oxc | partial | Parsing is covered. Type checking has no Rust equivalent. |
| `yaml` | no | 1 | saphyr, serde_yaml_ng | partial | Parsing is covered. No crate rewrites YAML while keeping comments and layout. |
| `ajv` | no | 1 | jsonschema | good |  |

### Pieces, in port order

- wave 0: `apps/lite/scripts/` (4702 lines)
- wave 0: `apps/lite/src/connectionStore.ts` (19 lines)
- wave 0: `apps/lite/src/systemFonts.ts` (1 line)
- wave 0: `apps/lite/tools/` (150 lines)
- wave 0: `public/` (74 lines)
- wave 0: `src/fonts.ts` (18 lines)
- wave 0: `tools/e2e-shards-fixture/` (10 lines)
- wave 1: `apps/lite/src/i18n/settings/` (133 lines)
- wave 1: `mock/`, `src/api/`, `src/api/engines/`, `src/dae/`, tangled: one piece (14394 lines)
- wave 2: `apps/lite/src/demo/` (52 lines)
- wave 2: `apps/lite/src/fetchFailure.ts` (18 lines)
- wave 2: `apps/lite/src/groupBindings.ts` (91 lines)
- wave 2: `apps/lite/src/operations.ts` (54 lines)
- wave 2: `apps/lite/src/themes/site/` (786 lines)
- wave 2: `tools/` (2713 lines)
- wave 4: `apps/lite/src/docs.ts` (6 lines)
- wave 4: `apps/lite/src/groupHealth.ts` (32 lines)
- wave 4: `apps/lite/src/regions.ts` (5 lines)
- wave 4: `apps/lite/src/ruleDraft.ts` (226 lines)
- wave 5: `apps/lite/src/api.ts`, `apps/lite/src/config.ts`, tangled: one piece (331 lines)
- wave 5: `apps/lite/src/describe.ts` (255 lines)
- wave 5: `apps/lite/src/groupDraft.ts` (114 lines)
- wave 6: `apps/lite/src/connectionStartup.ts` (23 lines)
- wave 6: `apps/lite/src/groupMembers.ts` (54 lines)
- wave 6: `apps/lite/src/nodeProbeState.ts` (96 lines)
- wave 6: `apps/lite/src/resourceDraft.ts` (46 lines)
- wave 6: `apps/lite/src/routingTest.ts` (69 lines)
- wave 6: `apps/lite/src/ruleReview.ts` (51 lines)
- wave 6: `apps/lite/src/searchIndex.ts` (91 lines)
- wave 6: `apps/lite/src/subscriptionDraft.ts` (47 lines)

### Decisions for the user

**D1** (shape): The app is a library, entered through doona (.), doona-lite (apps/lite). Port every entry point, in that shape?

> Yes: the Rust crate's public API becomes the library's front door

**D2** (keep apps/lite/src/AboutPanel.tsx): apps/lite/src/AboutPanel.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D3** (keep apps/lite/src/AddNodeToGroup.tsx): apps/lite/src/AddNodeToGroup.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D4** (keep apps/lite/src/App.tsx): apps/lite/src/App.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D5** (keep apps/lite/src/Commands.tsx): apps/lite/src/Commands.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D6** (keep apps/lite/src/ConfigExport.tsx): apps/lite/src/ConfigExport.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D7** (keep apps/lite/src/Connect.tsx): apps/lite/src/Connect.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D8** (keep apps/lite/src/CopyMembers.tsx): apps/lite/src/CopyMembers.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D9** (keep apps/lite/src/Demo.tsx): apps/lite/src/Demo.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D10** (keep apps/lite/src/Dns.tsx): apps/lite/src/Dns.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D11** (keep apps/lite/src/DnsUpstreams.tsx): apps/lite/src/DnsUpstreams.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D12** (keep apps/lite/src/GroupBindPopover.tsx): apps/lite/src/GroupBindPopover.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D13** (keep apps/lite/src/GroupEditPanel.tsx): apps/lite/src/GroupEditPanel.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D14** (keep apps/lite/src/GroupEditor.tsx): apps/lite/src/GroupEditor.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D15** (keep apps/lite/src/GroupInspector.tsx): apps/lite/src/GroupInspector.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D16** (keep apps/lite/src/GroupTextDialog.tsx): apps/lite/src/GroupTextDialog.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D17** (keep apps/lite/src/Groups.tsx): apps/lite/src/Groups.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D18** (keep apps/lite/src/NodeForm.tsx): apps/lite/src/NodeForm.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D19** (keep apps/lite/src/NodeProbes.tsx): apps/lite/src/NodeProbes.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D20** (keep apps/lite/src/Nodes.tsx): apps/lite/src/Nodes.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D21** (keep apps/lite/src/Routing.tsx): apps/lite/src/Routing.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D22** (keep apps/lite/src/RuleList.tsx): apps/lite/src/RuleList.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D23** (keep apps/lite/src/RuleTester.tsx): apps/lite/src/RuleTester.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D24** (keep apps/lite/src/SettingsDialog.tsx): apps/lite/src/SettingsDialog.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D25** (keep apps/lite/src/ShareDialog.tsx): apps/lite/src/ShareDialog.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D26** (keep apps/lite/src/Shell.tsx): apps/lite/src/Shell.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D27** (keep apps/lite/src/SubscriptionFeatureNodes.tsx): apps/lite/src/SubscriptionFeatureNodes.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D28** (keep apps/lite/src/SubscriptionFormInterval.tsx): apps/lite/src/SubscriptionFormInterval.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D29** (keep apps/lite/src/Subscriptions.tsx): apps/lite/src/Subscriptions.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D30** (keep apps/lite/src/TemplatePicker.tsx): apps/lite/src/TemplatePicker.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D31** (keep apps/lite/src/app/): apps/lite/src/app/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D32** (keep apps/lite/src/i18n/): apps/lite/src/i18n/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D33** (keep apps/lite/src/main.tsx): apps/lite/src/main.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D34** (keep apps/lite/src/preload.ts): apps/lite/src/preload.ts stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D35** (keep apps/lite/src/state.ts): apps/lite/src/state.ts stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D36** (keep apps/lite/src/themes/shared/): apps/lite/src/themes/shared/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D37** (keep apps/lite/src/ui.tsx): apps/lite/src/ui.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D38** (keep apps/lite/src/ui/): apps/lite/src/ui/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D39** (keep apps/lite/src/ui/icons/): apps/lite/src/ui/icons/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D40** (keep apps/lite/src/ui/motion/): apps/lite/src/ui/motion/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D41** (keep apps/lite/src/useFeatureDraft.tsx): apps/lite/src/useFeatureDraft.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D42** (keep src/features/activity/): src/features/activity/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D43** (keep src/features/config/): src/features/config/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D44** (keep src/features/connections/): src/features/connections/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D45** (keep src/features/dns/): src/features/dns/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D46** (keep src/features/events/): src/features/events/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D47** (keep src/features/flows/): src/features/flows/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D48** (keep src/features/logs/): src/features/logs/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D49** (keep src/features/nodes/): src/features/nodes/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D50** (keep src/features/overview/): src/features/overview/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D51** (keep src/features/policies/): src/features/policies/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D52** (keep src/features/policies/group/): src/features/policies/group/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D53** (keep src/features/rules/): src/features/rules/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D54** (keep src/features/settings/): src/features/settings/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D55** (keep src/features/shared/): src/features/shared/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D56** (keep src/i18n/): src/i18n/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D57** (keep src/main.tsx): src/main.tsx stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D58** (keep src/shell/): src/shell/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D59** (keep src/shell/search/): src/shell/search/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D60** (keep src/shell/widgets/): src/shell/widgets/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D61** (keep src/store/): src/store/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D62** (keep src/ui/): src/ui/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D63** (keep src/ui/charts/): src/ui/charts/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D64** (keep src/ui/code/): src/ui/code/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D65** (keep src/ui/icons/): src/ui/icons/ stays in the source language (UI code: keep in the original language). What replaces it in the Rust app, or does it stay outside?

> Keep it outside the Rust app, behind the front door, and decide its replacement once the rest is Rust

**D66** (blocked src/fonts.ts): src/fonts.ts is blocked: Uses dynamic code (eval, new Function or computed imports). Rework that part in Rust, or keep the unit outside?

> Research the blocker first; then port it with the piece that uses it

**D67** (blocked tools/): tools/ is blocked: Uses dynamic code (eval, new Function or computed imports). Rework that part in Rust, or keep the unit outside?

> Research the blocker first; then port it with the piece that uses it

**D68** (dependency @astryxdesign/core): Which crate replaces @astryxdesign/core (used by 35 units)?

> Research it against primary sources into rust/research/@astryxdesign-core.md before porting what uses it

**D69** (dependency react-aria-components): Which crate replaces react-aria-components (used by 7 units)?

> Research it against primary sources into rust/research/react-aria-components.md before porting what uses it

**D70** (dependency @stylexjs/stylex): Which crate replaces @stylexjs/stylex (used by 6 units)?

> Research it against primary sources into rust/research/@stylexjs-stylex.md before porting what uses it

**D71** (dependency @playwright/test): Which crate replaces @playwright/test (used by 3 units)?

> Research it against primary sources into rust/research/@playwright-test.md before porting what uses it

**D72** (dependency react-aria): Which crate replaces react-aria (used by 3 units)?

> Research it against primary sources into rust/research/react-aria.md before porting what uses it

**D73** (dependency @codemirror/autocomplete): Which crate replaces @codemirror/autocomplete (used by 2 units)?

> Research it against primary sources into rust/research/@codemirror-autocomplete.md before porting what uses it

**D74** (dependency @codemirror/commands): Which crate replaces @codemirror/commands (used by 2 units)?

> Research it against primary sources into rust/research/@codemirror-commands.md before porting what uses it

**D75** (dependency @codemirror/language): Which crate replaces @codemirror/language (used by 2 units)?

> Research it against primary sources into rust/research/@codemirror-language.md before porting what uses it

**D76** (dependency @codemirror/lint): Which crate replaces @codemirror/lint (used by 2 units)?

> Research it against primary sources into rust/research/@codemirror-lint.md before porting what uses it

**D77** (dependency @codemirror/search): Which crate replaces @codemirror/search (used by 2 units)?

> Research it against primary sources into rust/research/@codemirror-search.md before porting what uses it

**D78** (dependency @codemirror/state): Which crate replaces @codemirror/state (used by 2 units)?

> Research it against primary sources into rust/research/@codemirror-state.md before porting what uses it

**D79** (dependency @codemirror/view): Which crate replaces @codemirror/view (used by 2 units)?

> Research it against primary sources into rust/research/@codemirror-view.md before porting what uses it

**D80** (dependency @lezer/highlight): Which crate replaces @lezer/highlight (used by 2 units)?

> Research it against primary sources into rust/research/@lezer-highlight.md before porting what uses it

**D81** (dependency react-dom): Which crate replaces react-dom (used by 2 units)?

> Research it against primary sources into rust/research/react-dom.md before porting what uses it

**D82** (dependency ajv-formats): Which crate replaces ajv-formats (used by 1 unit)?

> Research it against primary sources into rust/research/ajv-formats.md before porting what uses it

**D83** (dependency d3-shape): Which crate replaces d3-shape (used by 1 unit)?

> Research it against primary sources into rust/research/d3-shape.md before porting what uses it

**D84** (dependency openapi-fetch): Which crate replaces openapi-fetch (used by 1 unit)?

> Research it against primary sources into rust/research/openapi-fetch.md before porting what uses it

**D85** (dependency qrcode.react): Which crate replaces qrcode.react (used by 1 unit)?

> Research it against primary sources into rust/research/qrcode.react.md before porting what uses it

**D86** (dependency vitest): Which crate replaces vitest (used by 1 unit)?

> Research it against primary sources into rust/research/vitest.md before porting what uses it

**D87** (dependency @astryxdesign/theme-neutral): Which crate replaces @astryxdesign/theme-neutral (declared, no unit imports it)?

> Research it against primary sources into rust/research/@astryxdesign-theme-neutral.md before porting what uses it

**D88** (dependency @fontsource-variable/figtree): Which crate replaces @fontsource-variable/figtree (declared, no unit imports it)?

> Research it against primary sources into rust/research/@fontsource-variable-figtree.md before porting what uses it

**D89** (dependency @fontsource-variable/noto-sans-sc): Which crate replaces @fontsource-variable/noto-sans-sc (declared, no unit imports it)?

> Research it against primary sources into rust/research/@fontsource-variable-noto-sans-sc.md before porting what uses it

**D90** (dependency @fontsource-variable/noto-sans-tc): Which crate replaces @fontsource-variable/noto-sans-tc (declared, no unit imports it)?

> Research it against primary sources into rust/research/@fontsource-variable-noto-sans-tc.md before porting what uses it

**D91** (dependency react): Which crate replaces react (declared, no unit imports it)?

> Research it against primary sources into rust/research/react.md before porting what uses it

**D92** (dependency typescript): typescript maps to oxc only in part (used by 2 units; Parsing is covered. Type checking has no Rust equivalent). Use it?

> Use oxc, and record cases for the behavior it does not cover

**D93** (dependency yaml): yaml maps to saphyr, serde_yaml_ng only in part (used by 1 unit; Parsing is covered. No crate rewrites YAML while keeping comments and layout). Use it?

> Use saphyr, serde_yaml_ng, and record cases for the behavior it does not cover

**D94** (tangled units): mock/, src/api/, src/api/engines/, src/dae/ import each other (14394 lines). Port them as one piece?

> Yes: one piece in wave 1, no glue between its parts

**D95** (tangled units): apps/lite/src/api.ts, apps/lite/src/config.ts import each other (331 lines). Port them as one piece?

> Yes: one piece in wave 5, no glue between its parts

