---
name: honk-android
description: Deploy, configure, verify, troubleshoot, and migrate existing Clash/Mihomo configs to honk-core (ARM64 eBPF transparent proxy) with coexisting BoxProxy Honk (com.boxproxy.honk) and embedded full-feature native Doona WebUI on rooted Android (Linux 6.1+ / 6.6+ GKI) devices via ADB. Use when the user asks to install, configure, update, migrate to, or debug honk on Android.
---

# Honk for Android Automated Deployment & Operations Skill

This skill enables an AI agent to end-to-end deploy, configure, migrate, verify, and troubleshoot **honk (`honk-core`) + BoxProxy Honk (`com.boxproxy.honk`, coexisting alongside official `com.boxproxy.box`) + embedded native Doona WebUI** on a rooted ARM64 Android device connected over ADB.

## Hard Safety Rules (MUST FOLLOW)

1. **Never flash partitions or system images** (`fastboot`, `dd` to `/dev/block/*`, boot/vendor/system edits). Operate strictly under `/data/user/0/com.boxproxy.honk/files/box/`, `/data/local/tmp/`, and standard `adb install`.
2. **Never leak or commit user secrets**: Do not print, commit, or store user UUIDs, passwords, subscription URLs, private DNS tokens, or device serials into repository files, public docs, or memory stores.
3. **Coexistence with official `com.boxproxy.box`**: `BoxProxy-honk-arm64.apk` uses the standalone package name **`com.boxproxy.honk`** (`BoxProxy Honk`), so NEVER uninstall `com.boxproxy.box` unless explicitly asked. Before starting `honk` in `com.boxproxy.honk`, stop any running proxy core in `com.boxproxy.box` first so two transparent proxies do not run simultaneously.
4. **Always clean temporary files**: Remove temporary files pushed to `/data/local/tmp/` once deployment completes.

---

## Bundle Layout (Relative to Repo / Release Root)

- `bin/honk-android-arm64` — Static `aarch64-unknown-linux-musl` `honk-core` binary built with `ebpf,rprx,clash-api,native-ui,jemalloc` (includes embedded Doona UI `c7f29e3`, dedicated leaf cgroup `/sys/fs/cgroup/honk` memory telemetry, `/api/v1/x-honk/core/update` self-updater, ARM64 `kprobe`/`kretprobe` UDP receive trace, full `BPF_PROG_QUERY` datapath hook verification, RFC 1035 case-insensitive DNS validation, and Android UDP bootstrap retry + fallback resolver).
- `bin/boxctl-android-arm64` — Static `aarch64-unknown-linux-musl` `boxctl` supporting `honk` core, `ebpf` mode, `--store db`, `HONK_RECORD_ALWAYS=1`, dedicated `/honk` cgroup isolation, and `com.boxproxy.honk` home resolution.
- `apk/BoxProxy-honk-arm64.apk` — Coexisting BoxProxy ARM64 APK (`v214 / 26.11.23.aa98568`, `package="com.boxproxy.honk"`, label `BoxProxy Honk`, tile `Box Honk`, built-in `Honk` updater under `工具 -> 更新核心`, boot script `/data/adb/service.d/boxproxy-honk.sh`, Home `WebUI` -> `http://127.0.0.1:9527/ui/`).
- `config/config.example.dae` — Sanitized Android-ready configuration template.
- `skill/honk-android/scripts/deploy_honk_android.sh` — Idempotent deployment helper script (defaults to `--pkg com.boxproxy.honk`).
- `skill/honk-android/references/troubleshooting.md` — Diagnostic cookbook and Android GKI 6.6 runtime reference.

---

## Step-by-Step Agent Workflow

### Phase 1: Preflight Environment Check

Run a single shell check to verify device readiness before making any changes:

```bash
adb get-state
adb shell su <<'EOF'
echo "arch=$(uname -m)"
echo "kernel=$(uname -r)"
echo "uid=$(id -u)"
mount | grep -m1 cgroup2 || echo "NO_CGROUP2"
ls -d /sys/fs/bpf || echo "NO_BPF_FS"
pm list packages | grep boxproxy || echo "NO_BOXPROXY_INSTALLED"
EOF
```

**Gate checks:**
- `arch` MUST be `aarch64`.
- `kernel` MUST be Linux `6.1` or newer (`6.6+` uses native TCX; `6.1–6.5` automatically falls back to Netlink `clsact`/`cls_bpf`; warn and stop if `< 6.1`).
- `uid` MUST be `0` (`su` root works).

### Phase 2: Install `BoxProxy-honk-arm64.apk` (`com.boxproxy.honk`)

1. Install directly (coexists cleanly even if official `com.boxproxy.box` is installed):
   ```bash
   adb install -r apk/BoxProxy-honk-arm64.apk
   ```
2. Launch once so it initializes `/data/user/0/com.boxproxy.honk/files/box`:
   ```bash
   adb shell monkey -p com.boxproxy.honk -c android.intent.category.LAUNCHER 1
   ```
   Prompt the user to grant Root permission to **`BoxProxy Honk`** in KernelSU / Magisk / APatch if not yet granted.

### Phase 3: Prepare `config.dae` & Geodata

1. **Locate or build `config.dae`**:
   - If the user wants to migrate from an existing Mihomo/Clash YAML (e.g. from `/data/user/0/com.boxproxy.box/files/box/mihomo/config.yaml`), translate proxies/groups/rules into `config.dae` syntax using `config/config.example.dae` as the structural template:
     - `global`: `tproxy_port: 12345`, `wan_interface: auto`, `auto_config_kernel_parameter: true`, `dial_mode: domain++`, `nfqueue_enable: false`, `tcp_check_url: 'https://cp.cloudflare.com/generate_204', 'http://connectivitycheck.platform.hicloud.com/generate_204'` (HTTPS first to enable Score QUIC probing), `bootstrap_resolver: '223.5.5.5:53'`, `fallback_resolver: '119.29.29.29:53'`, `data_dir: '/data/user/0/com.boxproxy.honk/files/box/honk/data'`, `preconnect_node_count: 2`, `udp_warm_node_count: 2`.
     - `dns`: add any custom DoH (`https://...`), DoH3 (`h3://...`), or DoT (`tls://...:853`) upstreams, and route their own parent domains (plus proxy server domains) to `bootstrap_udp` to prevent bootstrap loops.
     - `routing`:
       - Keep `pname('dnsmasq') -> direct(must)` and `sip(192.168.42.0/24) -> direct(must)` at the top to avoid hijacking Android USB tethering (`rndis0`/`ncm0`/`usb0`).
       - Keep `dip(geoip:private, 100.64.0.0/10, 224.0.0.0/4, 255.255.255.255/32) && !dport(53) -> direct(must)` (excluding port 53 so `honk` DNS interception remains active).
       - Truncate all `pname(...)` process-name matchers to **15 ASCII bytes max** (eBPF `argv[0]` basename limit on Linux/Android).
       - Place `domain(...)` rules BEFORE `dip(geoip:...)` rules so `domain++` sniffing wins over shared CDN IPs.
     - `experimental`:
       - `clash_api { external_controller: '127.0.0.1:9090' secret: '' default_mode: Rule }` (powers BoxProxy Honk's built-in Panel tab).
       - `native_api { enabled: true listen: '127.0.0.1:9527' allow_anonymous_loopback: true config_write: true ui: 'embedded' }` (powers BoxProxy Honk's Home WebUI -> full-feature native Doona panel).
       - `cache_file { enabled: true store_dns: true }`.
     - `assets`:
       - `geodata { geosite: 'https://raw.githubusercontent.com/Wcoom/honk-rules-dat/release/geosite.dat' geoip: 'https://raw.githubusercontent.com/Wcoom/honk-rules-dat/release/geoip.dat' route: 总模式 auto_update: true cron: '20 4,16 * * *' checksum: true }` (note: `honk` intentionally does not follow HTTP 302 redirects, so use `raw.githubusercontent.com/.../release/...` direct HTTP 200 URLs rather than GitHub `/releases/download/...`; Android `honk` evaluates cron in UTC so `20 4,16 * * *` equals 12:20 / 00:20 CST).
2. **Locate `geoip.dat` & `geosite.dat`**:
   - Download the matching 7-source merged geodata files from `https://raw.githubusercontent.com/Wcoom/honk-rules-dat/release/{geosite,geoip}.dat` (required for `merged-*` rules in `config/config.example.dae`), or copy existing compatible files into `/data/user/0/com.boxproxy.honk/files/box/honk/data/`.

### Phase 4: Deploy & Start `honk`

If official `com.boxproxy.box` is currently running a proxy core, stop it cleanly first (`boxctl --db /data/user/0/com.boxproxy.box/files/box/box.db down || true`), then run the bundled helper script:

```bash
bash skill/honk-android/scripts/deploy_honk_android.sh \
  --honk bin/honk-android-arm64 \
  --boxctl bin/boxctl-android-arm64 \
  --config /path/to/config.dae \
  --geoip /path/to/geoip.dat \
  --geosite /path/to/geosite.dat \
  --configure-db \
  --boot-autostart \
  --start
```

> Pass `--boot-autostart` (or toggle **设置 → 开机自启** inside `BoxProxy Honk`) to install `/data/adb/service.d/boxproxy-honk.sh` (isolated from official `/data/adb/service.d/boxproxy-box.sh`). Ensure official `BoxProxy`'s boot auto-start is disabled if enabling `BoxProxy Honk` boot auto-start so both proxies do not race at boot.

### Phase 5: Automated End-to-End Verification

ALWAYS run this verification suite after deployment and report only directly observed results:

```bash
adb shell su <<'EOF'
BOX=/data/user/0/com.boxproxy.honk/files/box
echo "=== 1. SERVICE STATUS ==="
"$BOX/bin/boxctl" --db "$BOX/box.db" status
echo "=== 2. CLASH API (127.0.0.1:9090) ==="
curl -sS --max-time 3 http://127.0.0.1:9090/version && echo ""
echo "=== 3. NATIVE DOONA WEBUI & DATAPATH (127.0.0.1:9527) ==="
curl -sI --max-time 3 http://127.0.0.1:9527/ui/ | head -n 5
curl -sS --max-time 3 http://127.0.0.1:9527/api/v1/datapath && echo ""
echo "=== 4. TRAFFIC & MIXED-CASE DNS CHECK ==="
curl -sS --max-time 8 -o /dev/null -w "cn_direct=%{http_code} %{time_total}s\n" https://WWW.BaiDu.COM
curl -sS --max-time 8 -o /dev/null -w "proxy_foreign=%{http_code} %{time_total}s\n" https://WWW.GooGle.COM/generate_204
echo "=== 5. LOG HEALTH AUDIT ==="
echo "errors=$(grep -c ' ERROR ' "$BOX/run/honk.log" || true) warns=$(grep -c ' WARN ' "$BOX/run/honk.log" || true)"
grep -E "Bootstrap|Fallback|clash API|native API|WARN|ERROR" "$BOX/run/honk.log" | tail -n 20
EOF
```

**Pass Criteria:**
- `boxctl status` reports `honk 正在运行: PID <pid>` and `模式 ebpf`.
- `127.0.0.1:9090/version` returns `{"meta":true,"premium":true,"version":"honk ..."}`.
- `127.0.0.1:9527/ui/` returns `HTTP/1.1 200 OK` and `/api/v1/datapath` reports `"state":"active"`, `"hooks":"attached"`, `"health":"healthy"`.
- `cn_direct=200` and `proxy_foreign=204`.
- `errors=0`, `warns=0`, and BOTH `Bootstrap resolver:` and `Fallback bootstrap resolver:` appear in `honk.log`.
