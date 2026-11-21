# Honk for Android Troubleshooting Reference

## 1. Config Store (`honk.db` vs `config.dae`)

- **Why direct edits to `config.dae` do not immediately appear in Doona**:
  - When `native_api.config_write: true` and `allow_anonymous_loopback: true` (loopback listen) are active on Android, `honk` runs in `db` store mode (`$BOX/honk/data/honk.db`) to unlock Doona's revision history, config import, and live node/rule/group editing.
  - On first startup (when `honk.db` has no stored config revision), `honk` imports `config.dae` automatically.
  - To re-import an updated `config.dae`: either use Doona's **Import** button in the UI, call `POST http://127.0.0.1:9527/api/v1/config/import`, or stop `honk`, remove `$BOX/honk/data/honk.db`, and start `honk` again.

## 2. Config Validation Failures (`honk -t`)

- **Unquoted non-ASCII or space-containing identifiers**: Wrap node names, group names, and file paths containing spaces, emoji, `/`, or special characters in single quotes (`'...'`).
- **Missing geodata**: `honk` reads `geoip.dat` and `geosite.dat` from `global.data_dir` (`/data/user/0/com.boxproxy.honk/files/box/honk/data`). Ensure both files exist and are mode `0644`.
- **Geodata download URLs**: Place `geosite` and `geoip` URLs under top-level `assets { geodata { ... } }` (not under `experimental.native_api`, which emits a `legacy-assets-key` deprecation warning).

## 3. BoxProxy Honk (`com.boxproxy.honk`) Coexistence & UI Checks

- **Coexistence with official `com.boxproxy.box`**:
  - `BoxProxy Honk` (`com.boxproxy.honk`) has its own data directory (`/data/user/0/com.boxproxy.honk/files/box`), its own Quick Settings tile (`Box Honk`), and its own boot script (`/data/adb/service.d/boxproxy-honk.sh`).
  - Do not run both `com.boxproxy.box` and `com.boxproxy.honk` proxy services at the exact same time; stop one before starting the other.
- **BoxProxy Honk Home shows `停止` or wrong core**:
  - Verify `box.db` (`runtime_profile` table or `boxctl --db $BOX/box.db status`) has `core_name='honk'`, `mode='ebpf'`, `config_name='config.dae'`.
  - Ensure `$BOX/run/state/config/startup-config` contains `external_controller: '127.0.0.1:9090'` (`boxctl` syncs `$BOX/honk/config.dae` -> `startup-config` automatically on `up`/`config sync`).
  - Grant Root access to `BoxProxy Honk` (`com.boxproxy.honk`) in KernelSU / Magisk / APatch Manager so the UI can query `boxctl`.
- **BoxProxy `WebUI` fails to load Doona**:
  - Confirm `experimental.native_api.enabled: true`, `listen: '127.0.0.1:9527'`, `allow_anonymous_loopback: true`, `ui: 'embedded'`.
  - Test `curl -sI http://127.0.0.1:9527/ui/` directly in root shell (must return `HTTP/1.1 200 OK`).
  - Confirm `BoxProxy-honk-arm64.apk` is installed (upstream BoxProxy APK does not route `honk` WebUI to port `9527`).
- **Crash after upgrading APK**:
  - Collect stacktrace via `adb shell su -c 'logcat -b crash -d'`.
  - Confirm `chown -R "$(stat -c '%u:%g' "$BOX")" "$BOX"` was run after any uninstall/reinstall (Android assigns a new app UID on fresh install).

## 4. Network, DNS, and USB Tethering Diagnostics

- **DNS `SERVFAIL` during cellular/Wi-Fi handoff**:
  - Verify both `bootstrap_resolver: '223.5.5.5:53'` and `fallback_resolver: '119.29.29.29:53'` are configured under `global`.
  - Verify your custom DoH/DoT hostnames and proxy server hostnames are routed to `bootstrap_udp` under `dns.routing.request` and `direct` under `routing` to prevent resolution loops.
- **USB Tethering (`rndis0` / `ncm0` / `usb0`) traffic accidentally hijacked**:
  - Do **NOT** set `lan_interface` to `rndis0` in `global`.
  - Keep `pname('dnsmasq') -> direct(must)` and `sip(192.168.42.0/24) -> direct(must)` at the top of `routing`.
  - Kernel eBPF `wan_egress_l3` automatically bypasses forwarded packets (`skb_ingress_ifindex != 0`).
- **Process-name (`pname`) rule not matching an Android app**:
  - eBPF captures at most **15 bytes** of `argv[0]` basename. Shorten package names longer than 15 characters to their 15-character prefix (e.g., `com.ss.android.ugc.aweme` -> `pname('com.ss.android.')`).

## 5. Useful Runtime Inspection Commands

```bash
# Live process RSS, threads, and FD breakdown
adb shell su <<'EOF'
PID=$(cat /data/user/0/com.boxproxy.honk/files/box/run/box.pid)
grep -E "VmRSS|VmHWM|Threads" /proc/$PID/status
echo "FDs=$(ls /proc/$PID/fd | wc -l)"
EOF

# Native API runtime summary & eBPF datapath health (expect active / attached / healthy)
adb shell su -c 'curl -sS http://127.0.0.1:9527/api/v1/runtime'
adb shell su -c 'curl -sS http://127.0.0.1:9527/api/v1/datapath'

# Check active WAN interface attachments in honk.log
adb shell su -c 'grep "iface_watch" /data/user/0/com.boxproxy.honk/files/box/run/honk.log | tail -n 20'
```
