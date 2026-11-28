# Session Handoff (`feat/pure-rust-rewrite` — All 4 Waves Complete, Ready for Cutover)

- **Skill to load next**: `/home/wcoom/.gemini/config/skills/rust-anything/SKILL.md`
- **Current Milestone**: **Wave 1–4 全部验收通过（待用户明确确认后执行切流与发布）**
  - **Wave 1 (`done`)**: `boxproxy/boxbpf` C→Rust（`0.0%` fallback，`No drift`，OnePlus 13 GKI 6.6.118 12 BPF 对象 + 8 组 map dump 逐字节一致）；`rust/xtask` 替代 7 脚本；`com.boxproxy.honk` v214 逆向规格（660 行 / 58 模块）。
  - **Wave 2 (`done`)**: Doona 14 个 `rust-anything` 逻辑 crate（12,469/12,493 = 99.81%，13/14 `Gate passed.`）+ Leptos 0.8 CSR WASM `/ui/` & `/ui/lite/` + `honk-core` ARM64 静态产物（`31,286,288 B`，比 React 版小 1.43 MB）+ 真机 0 错误 E2E。
  - **Wave 3 (`done`)**: 8-crate 纯 Rust 引擎 [rust/boxproxy/](boxproxy/)（28/28 单测 + `clippy -D warnings` 0 告警）；`boxproxy/boxctl` 统一接入 `boxproxy-schema` v15（35/35 单测）；ARM64 `libboxproxy_app.so`（12 JNI 符号）。
  - **Wave 4 (`done`)**:
    - 完整 1:1 复刻原版 `com.boxproxy.honk` Miuix 前端（含全部 4 个主标签页及全部设置子页面：`基础代理配置`/`其他代理配置`/`主题设置`/`备份与恢复`/`通知`/`关于`/`延迟测速目标` + `PanelSheetActivity` 底部高斯模糊悬浮窗）。
    - R8 签名 APK [app-release.apk](boxproxy/android-shell/app/build/outputs/apk/release/app-release.apk) 体积 **`7.3 MB`**（较原版 `9.4 MB` 小 2.1 MB，单 `classes.dex` `2.12 MB`）。
    - 真机修复 `boxproxy-schema` `ap+` 种子问题与 Honk Native API 字符串 `u64` 遥测解析，全纯 Rust 栈通过 `io.honk.android` 界面点击「启动」/「重载」/「停止」实测通过（冷启动 `235 ms`，`cn=204`/`foreign=204`，`honk.log` 113 行 `0 WARN/ERROR`，`logcat` `0 异常/崩溃`）。
    - 原版 `com.boxproxy.honk` (`v1.0.10`) 已无损恢复运行（PID=11939，`cn=204`/`foreign=204`），`io.honk.android` 并排保留在真机。
- **Next Action**: 等待用户确认是否执行切流（同步 `patches/honk-android.patch`、切换包名、打包发布），切流脚本见 [cutover.sh](cutover.sh)（默认 `--dry-run`）。
- **Protected Files**: `.gitignore`、`build-cf-opt.sh`、`cf-opt/` 未触碰。

## Key Pointers
- Equivalence Report: [EQUIVALENCE_REPORT.md](EQUIVALENCE_REPORT.md)
- Cutover Script: [cutover.sh](cutover.sh)
- Migration Map: [MAP.md](MAP.md)
- Decisions (ADR-001 ~ ADR-009): [DECISIONS.md](DECISIONS.md)
- Signed Pure-Rust APK: [app-release.apk](boxproxy/android-shell/app/build/outputs/apk/release/app-release.apk)
