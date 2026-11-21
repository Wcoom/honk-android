---
name: honk-android
description: Deploy, configure, verify, troubleshoot, and migrate existing Clash/Mihomo configs to honk-core (ARM64 eBPF transparent proxy) with BoxProxy and embedded full-feature native Doona WebUI on rooted Android (Linux 6.6+ GKI) devices via ADB. Use when the user asks to install, configure, update, migrate to, or debug honk on Android.
---

# Honk for Android Automated Deployment & Operations Skill

This skill enables an AI agent to end-to-end deploy, configure, migrate, verify, and troubleshoot **honk (`honk-core`) + BoxProxy + embedded native Doona WebUI** on a rooted ARM64 Android device connected over ADB.

## Hard Safety Rules (MUST FOLLOW)

1. **Never flash partitions or system images** (`fastboot`, `dd` to `/dev/block/*`, boot/vendor/system edits). Operate strictly under `/data/user/0/com.boxproxy.box/files/box/`, `/data/local/tmp/`, and standard `adb install`.
2. **Never leak or commit user secrets**: Do not print, commit, or store user UUIDs, passwords, subscription URLs, private DNS tokens, or device serials into repository files, public docs, or memory stores.
3. **Preserve user data on APK upgrade**: Before uninstalling an existing `com.boxproxy.box` with a different signature, always back up `/data/user/0/com.boxproxy.box/files/box/` (configs, `box.db`, geodata) to a temporary staging directory and restore afterwards.
4. **Always clean temporary files**: Remove temporary files pushed to `/data/local/tmp/` once deployment completes.

---

## Bundle Layout (Relative to Repo / Release Root)

- `bin/honk-android-arm64` — Static `aarch64-unknown-linux-musl` `honk-core` binary built with `ebpf,rprx,clash-api,native-ui,jemalloc` (includes embedded Doona UI, ARM64 `kprobe`/`kretprobe` UDP receive trace, full `BPF_PROG_QUERY` datapath hook verification, RFC 1035 case-insensitive DNS validation, and Android UDP bootstrap retry + fallback resolver).
- `bin/boxctl-android-arm64` — Static `aarch64-unknown-linux-musl` `boxctl` supporting `honk` core, `ebpf` mode, `--store db`, and `HONK_RECORD_ALWAYS=1`.
- `apk/BoxProxy-honk-arm64.apk` — BoxProxy ARM64 APK patched with native `Honk` core support, `eBPF` mode, Clash API dashboard parsing (`external_controller`), bundled `boxctl`, and Home `WebUI` pointing directly to `http://127.0.0.1:9527/ui/` (Doona).
- `config/config.example.dae` — Sanitized Android-ready configuration template.
- `skill/honk-android/scripts/deploy_honk_android.sh` — Idempotent deployment helper script.
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
pm path com.boxproxy.box || echo "BOXPROXY_NOT_INSTALLED"
EOF
```

**Gate checks:**
- `arch` MUST be `aarch64`.
- `kernel` MUST be Linux `6.6` or newer (warn and stop if `< 6.6` unless the user explicitly overrides).
- `uid` MUST be `0` (`su` root works).
- Note whether `com.boxproxy.box` is already installed.

### Phase 2: Install or Upgrade `BoxProxy-honk-arm64.apk`

1. Try in-place install first:
   ```bash
   adb install -r apk/BoxProxy-honk-arm64.apk
   ```
2. If `adb install -r` fails with `INSTALL_FAILED_UPDATE_INCOMPATIBLE` (different signature from upstream BoxProxy):
   - Back up existing `/data/user/0/com.boxproxy.box/files/box` to `/data/local/tmp/box_backup.tar.gz` (root shell).
   - Uninstall old package: `adb uninstall com.boxproxy.box`.
   - Install new APK: `adb install apk/BoxProxy-honk-arm64.apk`.
   - Launch once (`adb shell monkey -p com.boxproxy.box -c android.intent.category.LAUNCHER 1`), prompt the user if Root permission dialog requires tapping **Allow**.
   - Restore `/data/local/tmp/box_backup.tar.gz` into `/data/user/0/com.boxproxy.box/files/box`, fixing `chown -R "$(stat -c '%u:%g' "$BOX")"` (the new UID changes after reinstall!), then delete `/data/local/tmp/box_backup.tar.gz`.
   - Ensure `$BOX/bin/boxctl` is updated from `bin/boxctl-android-arm64` (mode `0755`, owned by `$OWNER`).

### Phase 3: Prepare `config.dae` & Geodata

1. **Locate or build `config.dae`**:
   - If the user already has a `.dae` file, validate it and ensure all Android-critical settings below are present.
   - If the user wants to migrate from an existing Mihomo/Clash YAML (e.g., `/data/user/0/com.boxproxy.box/files/box/mihomo/config.yaml`), translate proxies/groups/rules into `config.dae` syntax using `config/config.example.dae` as the structural template:
     - `global`: `tproxy_port: 12345`, `wan_interface: auto`, `auto_config_kernel_parameter: true`, `dial_mode: domain++`, `nfqueue_enable: false`, `tcp_check_url: 'https://cp.cloudflare.com/generate_204', 'http://connectivitycheck.platform.hicloud.com/generate_204'` (HTTPS first to enable Score QUIC probing), `bootstrap_resolver: '223.5.5.5:53'`, `fallback_resolver: '119.29.29.29:53'`, `data_dir: '/data/user/0/com.boxproxy.box/files/box/honk/data'`, `preconnect_node_count: 2`, `udp_warm_node_count: 2`.
     - `dns`: add any custom DoH (`https://...`), DoH3 (`h3://...`), or DoT (`tls://...:853`) upstreams, and route their own parent domains (plus proxy server domains) to `bootstrap_udp` to prevent bootstrap loops.
     - `routing`:
       - Keep `pname('dnsmasq') -> direct(must)` and `sip(192.168.42.0/24) -> direct(must)` at the top to avoid hijacking Android USB tethering (`rndis0`/`ncm0`/`usb0`).
       - Keep `dip(geoip:private, 100.64.0.0/10, 224.0.0.0/4, 255.255.255.255/32) && !dport(53) -> direct(must)` (excluding port 53 so `honk` DNS interception remains active).
       - Truncate all `pname(...)` process-name matchers to **15 ASCII bytes max** (eBPF `argv[0]` basename limit on Linux/Android).
       - Place `domain(...)` rules BEFORE `dip(geoip:...)` rules so `domain++` sniffing wins over shared CDN IPs.
     - `experimental`:
       - `clash_api { external_controller: '127.0.0.1:9090' secret: '' default_mode: Rule }` (powers BoxProxy's built-in Panel tab).
       - `native_api { enabled: true listen: '127.0.0.1:9527' allow_anonymous_loopback: true config_write: true ui: 'embedded' }` (powers BoxProxy's Home WebUI -> full-feature native Doona panel).
       - `cache_file { enabled: true store_dns: true }`.
     - `assets`:
       - `geodata { geosite: 'https://github.com/Loyalsoldier/v2ray-rules-dat/releases/latest/download/geosite.dat' geoip: 'https://github.com/Loyalsoldier/v2ray-rules-dat/releases/latest/download/geoip.dat' }`.
2. **Locate `geoip.dat` & `geosite.dat`**:
   - Check if they already exist on the device (e.g., `$BOX/honk/data/` or `$BOX/mihomo/`) or on the host; copy them into `$BOX/honk/data/`.

### Phase 4: Deploy & Start `honk`

Use the bundled helper script (from the release or repo root):

```bash
bash skill/honk-android/scripts/deploy_honk_android.sh \
  --honk bin/honk-android-arm64 \
  --boxctl bin/boxctl-android-arm64 \
  --config /path/to/config.dae \
  --geoip /path/to/geoip.dat \
  --geosite /path/to/geosite.dat \
  --configure-db \
  --start
```

### Phase 5: Automated End-to-End Verification

ALWAYS run this verification suite after deployment and report only directly observed results:

```bash
adb shell su <<'EOF'
BOX=/data/user/0/com.boxproxy.box/files/box
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
