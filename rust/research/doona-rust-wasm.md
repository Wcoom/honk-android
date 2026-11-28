# Research: Doona WebUI (`/ui/` & `/ui/lite/`) Pure Rust WASM Port

Primary sources: `webui/package.json`, `webui/apps/lite/package.json`, `webui/vite.config.ts`, `rust-anything analyze --whole-app webui`.

## 1. Unit Landscape (`rust-anything analyze@1`)
- **Total**: 451 source files, 386 test files, 82,229 LOC, 98 units across 2 packages (`doona` + `doona-lite`).
- **Portable Logic Units (32: 22 `ready`, 10 `caution`)**:
  - Wave 0–1 Core: `src/dae/` (3,525 LOC — DAE config tokenizer/parser/AST/formatter/outboundMode), `src/api/` + `src/api/engines/` (7,486 LOC — Honk/Clash API client, SSE/WS normalization, error handling), `mock/` (3,383 LOC), `apps/lite/src/i18n/settings/` (133 LOC).
  - Wave 2–6 Domain Logic: `groupBindings`, `operations`, `ruleDraft`, `config`, `describe`, `groupDraft`, `connectionStartup`, `groupMembers`, `nodeProbeState`, `resourceDraft`, `routingTest`, `ruleReview`, `searchIndex`, `subscriptionDraft`, plus `src/features/shared/policyTarget.ts` & `modeView.ts` (Node Policy Selector).
- **UI Units (64 `keep`, 329 `.tsx` files)**:
  - Full Doona (`src/features/{activity,config,connections,dns,events,flows,logs,nodes,overview,policies,rules,settings,shared}`, `src/shell/`, `src/store/`, `src/ui/`).
  - Doona Lite (`apps/lite/src/*.tsx` — 40 UI units).

## 2. Dependency Mapping (Flagged `fit = unknown / partial`)

| Original npm Dependency | Role in Doona | Rust / WASM Equivalent | Semantic Traps & Notes |
| --- | --- | --- | --- |
| `react` / `react-dom` / `react-aria-components` | Reactive DOM UI, accessible focus/dialogs/menus/tabs | `leptos` (CSR on `wasm32-unknown-unknown`) + `web-sys` + native `<dialog>` / ARIA attributes | Keep exact `role`, `aria-*`, keyboard roving tabindex, and `data-testid` hooks so Doona's Playwright E2E + axe-core a11y specs pass identically. |
| `@astryxdesign/core` & `@stylexjs/stylex` | Design tokens & atomic CSS in Doona Lite | Static CSS custom properties compiled once from existing stylesheets / design tokens, bundled via Rust asset embed | Zero runtime JS needed; static CSS guarantees pixel-accurate visual parity. |
| `@codemirror/*` + `@lezer/highlight` | `.dae` config syntax highlighting, lint markers, search | Pure Rust lexer/highlighter (built directly atop the ported `src/dae/` tokenizer crate!) rendering synchronized highlighted lines + `<textarea>` overlay or contenteditable editor with lint gutters & Ctrl+F search | Avoids 200KB+ JS CodeMirror bundle; guarantees tokenizer and syntax highlighting never drift from `honk-config` `.dae` grammar. |
| `d3-shape` | Sparkline / throughput area & line SVG paths | Pure Rust SVG path generator (`format!("M{x},{y} L...")` / Catmull-Rom or monotone-X cubic Bezier matching `d3.curveMonotoneX`) | Verify curve control points with float tolerance `1e-6` against `d3-shape` recorded cases. |
| `openapi-fetch` | Typed HTTP client over `/api/v1/*` | `gloo-net` / `web-sys::fetch` + `serde` structs generated/verified against `contract/api-standardize/openapi.yaml` | Preserve exact ETag/If-Match, error envelope `{error:{code,message,details}}`, and SSE/WebSocket reconnect backoff. |
| `qrcode.react` | Subscription / share QR SVG rendering | `qrcode` crate (`QrCode::new(...)` -> `<svg>`) | Pure Rust, zero-dep WASM compatible. |
| `yaml` (partial fit) | Parsing & comment-preserving YAML inspection/edit | `serde_yaml_ng` / `saphyr` (for parsing) + line-preserving span patcher (for comments) | Doona primarily edits `.dae` configs (custom parser in `src/dae/`); YAML is used for contract/subscription inspection. |
| `ajv` / `ajv-formats` | Build/test JSON Schema validation | `jsonschema` crate (host tests only) | Not needed in browser runtime bundle. |

## 3. Build & Embedding Pipeline (`honk-core` `HONK_DOONA_DIR`)
- `honk-core` embeds `HONK_DOONA_DIR` (standard `/ui/`) and `${HONK_DOONA_DIR}-lite` (`/ui/lite/`) at compile time via `include_dir` / build script.
- Compiling `doona-wasm` and `doona-lite-wasm` with `cargo +1.100.0 build --target wasm32-unknown-unknown --release` + `wasm-bindgen --target web --no-typescript` + optional `wasm-opt -Oz` produces static `index.html`, `.css`, `.wasm`, and minimal ES glue loader directly in the exact directory layout `honk-core` expects — zero Node/Vite/pnpm required at build time after Wave 2 lift!
- **Note**: Browsers require a ~2–4 KB `wasm-bindgen` JS loader snippet to instantiate `WebAssembly.instantiateStreaming` and bridge DOM calls; 100% of application source code (logic + UI + styling) is pure Rust + CSS + HTML.
