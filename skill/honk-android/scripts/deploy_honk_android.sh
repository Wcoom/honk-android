#!/usr/bin/env bash
# deploy_honk_android.sh - Idempotent deployment helper for honk-core + BoxProxy Honk on Android (via ADB + su).
# Operates strictly under /data/user/0/<pkg>/files/box (default com.boxproxy.honk, coexisting with com.boxproxy.box); never touches system/boot partitions.
set -euo pipefail

HONK_BIN=""
BOXCTL_BIN=""
CONFIG_FILE=""
GEOIP_FILE=""
GEOSITE_FILE=""
CONFIGURE_DB=0
START_SERVICE=0
BOOT_AUTOSTART=0
SERIAL=""
PKG="com.boxproxy.honk"

usage() {
  cat <<'EOF'
Usage: deploy_honk_android.sh [options]

Options:
  --honk <path>        Path to honk-android-arm64 binary (required unless already deployed)
  --boxctl <path>      Optional path to boxctl-android-arm64 binary
  --config <path>      Optional path to config.dae
  --geoip <path>       Optional path to geoip.dat
  --geosite <path>     Optional path to geosite.dat
  --pkg <package>      Target BoxProxy package name (default: com.boxproxy.honk)
  --configure-db       Update BoxProxy box.db to core=honk, mode=ebpf, config=config.dae
  --boot-autostart     Enable boot_auto_start in box.db and install /data/adb/service.d/boxproxy-honk.sh
  --start              Start/restart honk via boxctl after deployment and run health checks
  -s, --serial <id>    Target ADB device serial (when multiple devices are attached)
  -h, --help           Show this help message
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --honk) HONK_BIN="$2"; shift 2 ;;
    --boxctl) BOXCTL_BIN="$2"; shift 2 ;;
    --config) CONFIG_FILE="$2"; shift 2 ;;
    --geoip) GEOIP_FILE="$2"; shift 2 ;;
    --geosite) GEOSITE_FILE="$2"; shift 2 ;;
    --pkg) PKG="$2"; shift 2 ;;
    --configure-db) CONFIGURE_DB=1; shift ;;
    --boot-autostart) BOOT_AUTOSTART=1; shift ;;
    --start) START_SERVICE=1; shift ;;
    -s|--serial) SERIAL="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown option: $1" >&2; usage; exit 2 ;;
  esac
done

BOX="/data/user/0/$PKG/files/box"
ADB=(adb)
if [[ -n "$SERIAL" ]]; then
  ADB+=(-s "$SERIAL")
fi

echo "[1/5] Checking ADB connection and root environment (target package: $PKG)..."
"${ADB[@]}" get-state >/dev/null
"${ADB[@]}" shell su <<EOF
set -e
ARCH=\$(uname -m)
KVER=\$(uname -r)
echo "  Device arch=\$ARCH kernel=\$KVER uid=\$(id -u)"
if [ "\$ARCH" != "aarch64" ]; then
  echo "ERROR: Expected aarch64 device, got \$ARCH" >&2
  exit 1
fi
if [ ! -d "$BOX" ]; then
  echo "ERROR: $BOX not found. Install BoxProxy-honk-arm64.apk ($PKG) and open it once first." >&2
  exit 1
fi
EOF

echo "[2/5] Stopping active core if running..."
"${ADB[@]}" shell su <<EOF
if [ -x "$BOX/bin/boxctl" ] && [ -f "$BOX/box.db" ]; then
  "$BOX/bin/boxctl" --db "$BOX/box.db" down || true
fi
EOF

echo "[3/5] Pushing selected artifacts to /data/local/tmp..."
PUSH_HONK=0; PUSH_BOXCTL=0; PUSH_CONFIG=0; PUSH_GEOIP=0; PUSH_GEOSITE=0
if [[ -n "$HONK_BIN" ]]; then
  [[ -f "$HONK_BIN" ]] || { echo "Missing --honk file: $HONK_BIN" >&2; exit 1; }
  "${ADB[@]}" push "$HONK_BIN" /data/local/tmp/honk_deploy_bin
  PUSH_HONK=1
fi
if [[ -n "$BOXCTL_BIN" ]]; then
  [[ -f "$BOXCTL_BIN" ]] || { echo "Missing --boxctl file: $BOXCTL_BIN" >&2; exit 1; }
  "${ADB[@]}" push "$BOXCTL_BIN" /data/local/tmp/boxctl_deploy_bin
  PUSH_BOXCTL=1
fi
if [[ -n "$CONFIG_FILE" ]]; then
  [[ -f "$CONFIG_FILE" ]] || { echo "Missing --config file: $CONFIG_FILE" >&2; exit 1; }
  "${ADB[@]}" push "$CONFIG_FILE" /data/local/tmp/honk_deploy_config.dae
  PUSH_CONFIG=1
fi
if [[ -n "$GEOIP_FILE" ]]; then
  [[ -f "$GEOIP_FILE" ]] || { echo "Missing --geoip file: $GEOIP_FILE" >&2; exit 1; }
  "${ADB[@]}" push "$GEOIP_FILE" /data/local/tmp/honk_deploy_geoip.dat
  PUSH_GEOIP=1
fi
if [[ -n "$GEOSITE_FILE" ]]; then
  [[ -f "$GEOSITE_FILE" ]] || { echo "Missing --geosite file: $GEOSITE_FILE" >&2; exit 1; }
  "${ADB[@]}" push "$GEOSITE_FILE" /data/local/tmp/honk_deploy_geosite.dat
  PUSH_GEOSITE=1
fi

echo "[4/5] Installing files, fixing ownership/permissions, and validating config..."
"${ADB[@]}" shell su <<EOF
set -e
BOX="$BOX"
OWNER=\$(stat -c "%u:%g" "\$BOX")
mkdir -p "\$BOX/bin" "\$BOX/honk/data"

if [ "$PUSH_HONK" = "1" ]; then
  cp /data/local/tmp/honk_deploy_bin "\$BOX/bin/honk"
  chmod 755 "\$BOX/bin/honk"
  chown "\$OWNER" "\$BOX/bin/honk"
fi
if [ "$PUSH_BOXCTL" = "1" ]; then
  cp /data/local/tmp/boxctl_deploy_bin "\$BOX/bin/boxctl"
  chmod 755 "\$BOX/bin/boxctl"
  chown "\$OWNER" "\$BOX/bin/boxctl"
fi
if [ "$PUSH_CONFIG" = "1" ]; then
  cp /data/local/tmp/honk_deploy_config.dae "\$BOX/honk/config.dae"
  chmod 644 "\$BOX/honk/config.dae"
  chown "\$OWNER" "\$BOX/honk/config.dae"
fi
if [ "$PUSH_GEOIP" = "1" ]; then
  cp /data/local/tmp/honk_deploy_geoip.dat "\$BOX/honk/data/geoip.dat"
  chmod 644 "\$BOX/honk/data/geoip.dat"
  chown "\$OWNER" "\$BOX/honk/data/geoip.dat"
fi
if [ "$PUSH_GEOSITE" = "1" ]; then
  cp /data/local/tmp/honk_deploy_geosite.dat "\$BOX/honk/data/geosite.dat"
  chmod 644 "\$BOX/honk/data/geosite.dat"
  chown "\$OWNER" "\$BOX/honk/data/geosite.dat"
fi

chown -R "\$OWNER" "\$BOX/honk"
rm -f /data/local/tmp/honk_deploy_bin /data/local/tmp/boxctl_deploy_bin \
      /data/local/tmp/honk_deploy_config.dae /data/local/tmp/honk_deploy_geoip.dat \
      /data/local/tmp/honk_deploy_geosite.dat

if [ -x "\$BOX/bin/honk" ] && [ -f "\$BOX/honk/config.dae" ]; then
  echo "  Validating config.dae..."
  "\$BOX/bin/honk" -t -d "\$BOX/honk/data" -c "\$BOX/honk/config.dae"
fi
EOF

if [[ "$CONFIGURE_DB" == "1" || "$BOOT_AUTOSTART" == "1" ]]; then
  echo "  Updating box.db (configure_db=$CONFIGURE_DB, boot_autostart=$BOOT_AUTOSTART)..."
  TMP_DB="$(mktemp)"
  "${ADB[@]}" shell su <<EOF
if [ -f "$BOX/box.db" ]; then
  cp "$BOX/box.db" /data/local/tmp/honk_deploy_box.db
  chmod 644 /data/local/tmp/honk_deploy_box.db
fi
EOF
  if "${ADB[@]}" pull /data/local/tmp/honk_deploy_box.db "$TMP_DB" >/dev/null 2>&1; then
    python3 - "$TMP_DB" "$CONFIGURE_DB" "$BOOT_AUTOSTART" <<'PY'
import sqlite3, sys
conn = sqlite3.connect(sys.argv[1])
cfg_db = sys.argv[2] == "1"
boot_auto = sys.argv[3] == "1"
cur = conn.cursor()
tables = {r[0] for r in cur.execute("SELECT name FROM sqlite_master WHERE type='table'").fetchall()}
if "runtime_profile" in tables:
    if cfg_db:
        cur.execute("UPDATE runtime_profile SET core_name='honk', mode='ebpf', config_name='config.dae' WHERE id=1")
    if boot_auto:
        cur.execute("UPDATE runtime_profile SET boot_auto_start=1 WHERE id=1")
if cfg_db and "settings" in tables:
    cur.execute("UPDATE settings SET value='honk' WHERE key IN ('core','proxy_core')")
    cur.execute("UPDATE settings SET value='ebpf' WHERE key IN ('mode','proxy_mode')")
    cur.execute("UPDATE settings SET value='config.dae' WHERE key IN ('config_file','honk_config_file')")
conn.commit()
conn.close()
PY
    "${ADB[@]}" push "$TMP_DB" /data/local/tmp/honk_deploy_box.db >/dev/null
    "${ADB[@]}" shell su <<EOF
OWNER=\$(stat -c "%u:%g" "$BOX")
am force-stop "$PKG" >/dev/null 2>&1 || true
rm -f "$BOX/box.db-wal" "$BOX/box.db-shm" "$BOX/box.db-journal"
cp /data/local/tmp/honk_deploy_box.db "$BOX/box.db"
chown "\$OWNER" "$BOX/box.db"
chmod 600 "$BOX/box.db"
rm -f /data/local/tmp/honk_deploy_box.db
EOF
  fi
  rm -f "$TMP_DB"
fi

if [[ "$BOOT_AUTOSTART" == "1" ]]; then
  if [[ "$PKG" == "com.boxproxy.honk" ]]; then
    BOOT_SCRIPT="/data/adb/service.d/boxproxy-honk.sh"
    BOOT_TAG="BoxProxyHonkBoot"
  else
    BOOT_SCRIPT="/data/adb/service.d/boxproxy-box.sh"
    BOOT_TAG="BoxProxyBoot"
  fi
  echo "  Installing boot auto-start script: $BOOT_SCRIPT..."
  "${ADB[@]}" shell su <<EOF
set -e
mkdir -p /data/adb/service.d
tmp="${BOOT_SCRIPT}.tmp"
cat > "\$tmp" <<'BOOTEOF'
#!/system/bin/sh
(
  export PATH="/data/adb/magisk:/data/adb/ksu/bin:/data/adb/ap/bin:/system/bin:/system/xbin:\$PATH"
  export BOXCTL_STDOUT_LOG_LINE=1
  run_dir='$BOX/run'
  log_file='$BOX/run/boot.log'
  boxctl='$BOX/bin/boxctl'
  db='$BOX/box.db'

  timestamp() {
    date '+%Y-%m-%d %H:%M:%S' 2>/dev/null || date 2>/dev/null || echo "-"
  }

  boot_log() {
    line="\$(timestamp) [Info] \$*"
    echo "\$line" >> "\$log_file" 2>/dev/null || true
  }

  log -t $BOOT_TAG '开机自启：准备启动服务' 2>/dev/null || true
  log -t $BOOT_TAG 'waiting for boot animation to stop' 2>/dev/null || true

  while [ "\$(getprop init.svc.bootanim 2>/dev/null)" != "stopped" ]; do
    sleep 10
  done

  log -t $BOOT_TAG 'waiting for runtime files' 2>/dev/null || true
  while [ ! -x "\$boxctl" ] || [ ! -f "\$db" ]; do
    sleep 2
  done
  mkdir -p "\$run_dir" 2>/dev/null || exit 1
  : > "\$log_file" 2>/dev/null || exit 1
  boot_log '开机自启：准备启动服务'
  log -t $BOOT_TAG 'runtime files are ready' 2>/dev/null || true

  "\$boxctl" --db "\$db" boot >> "\$log_file" 2>&1
  start_code=\$?
  if [ "\$start_code" -eq 0 ]; then
    boot_log '开机自启：服务启动命令已完成'
  else
    boot_log "\$(printf '%s' '启动失败: exit code __CODE__' | sed "s/__CODE__/\$start_code/g")"
  fi
) >/dev/null 2>&1 &
exit 0
BOOTEOF
chmod 755 "\$tmp"
chown root:root "\$tmp" 2>/dev/null || true
mv "\$tmp" "$BOOT_SCRIPT"
test -x "$BOOT_SCRIPT"
EOF
fi

if [[ "$START_SERVICE" = "1" ]]; then
  echo "[5/5] Starting honk via boxctl and running quick health check..."
  "${ADB[@]}" shell su <<EOF
set -e
BOX="$BOX"
"\$BOX/bin/boxctl" --db "\$BOX/box.db" up
sleep 3
"\$BOX/bin/boxctl" --db "\$BOX/box.db" status
echo "--- Clash API (127.0.0.1:9090) ---"
curl -sS --max-time 3 http://127.0.0.1:9090/version && echo "" || true
echo "--- Native Doona WebUI (127.0.0.1:9527/ui/) ---"
curl -sI --max-time 3 http://127.0.0.1:9527/ui/ | head -n 5 || true
EOF
else
  echo "[5/5] Deployment complete (pass --start to start service automatically)."
fi
