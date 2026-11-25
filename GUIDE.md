# Honk for Android 人类使用与部署完全指南

这份指南面向普通玩家与进阶用户，手把手带你把 **honk（基于 Linux eBPF 的新一代透明代理核心）** + **BoxProxy** + **Doona 原生面板** 部署到一台已 Root 的 Android ARM64 手机上。全程仅在 `/data` 目录操作，**不会刷写任何系统分区或镜像**。

---

## 一、开始前确认你的手机是否满足条件

满足以下 3 个条件即可使用：

1. **架构**：ARM64（绝大多数近几年的安卓手机均为 `aarch64`）。
2. **内核版本**：Android GKI **Linux 6.1 及以上**（可在手机终端或电脑执行 `adb shell uname -r` 查看，例如 `6.1.x-android14-...` / `6.6.x-android15-...`）。
3. **Root 权限**：已安装 KernelSU / Magisk / APatch，且可正常授予 Root 权限。

> 电脑端仅需安装 `adb` 并打开手机「开发者选项 → USB 调试」即可。

---

## 二、准备所需文件

正式版发布包已包含所有核心成品（`honk-android-arm64`、`boxctl-android-arm64`、`BoxProxy-honk-arm64.apk`、`config.example.dae`），你只需要额外准备以下 2 样东西：

1. **地理数据库**（路由分流必需，与默认模板 `config.example.dae` 的 7 源规则分组配套）：
   - `geosite.dat`：https://raw.githubusercontent.com/Wcoom/honk-rules-dat/release/geosite.dat
   - `geoip.dat`：https://raw.githubusercontent.com/Wcoom/honk-rules-dat/release/geoip.dat
   - 来自配套开源自动构建仓库 **[Wcoom/honk-rules-dat](https://github.com/Wcoom/honk-rules-dat)**（每天北京时间 00:00 / 12:00 自动融合 MetaCubeX + blackmatrix7 + Surfing/AWAvenue + Hackl0us + clang IPv6 + X-Shelby + IPinfo Lite CN 并去重发布；模板已默认开启云端直链自动拉取与 `.sha256sum` 校验）。
2. **你的代理节点或订阅链接**（支持 VLESS / AnyTLS / Trojan / Shadowsocks / Hysteria2 / TUIC / VMess 等常见协议）。

---

## 三、分步安装教程

### 第 1 步：安装共存版 BoxProxy Honk APK

本包内的 APK 已采用独立包名 **`com.boxproxy.honk`**（桌面名称显示为 **`BoxProxy Honk`**，快捷开关磁贴为 **`Box Honk`**），**可与手机上原有的官方版 BoxProxy (`com.boxproxy.box`) 直接并存安装，无需卸载原版、互不干扰**。

将手机连接电脑并执行：

```bash
adb install apk/BoxProxy-honk-arm64.apk
```

安装完成后：
1. 在手机桌面上找到并**打开一次 `BoxProxy Honk`**；
2. 在 KernelSU / Magisk / APatch 弹出 Root 授权提示时点击**允许**；
3. 等待应用初始化完成（此时会在 `/data/user/0/com.boxproxy.honk/files/box/` 自动生成独立目录与数据库）。

---

### 第 2 步：编写你的 `config.dae` 配置文件

复制 `config/config.example.dae` 为 `config.dae`，用任意文本编辑器打开修改 **3 处**：

1. **填入节点或订阅**：
   - 单节点写法（放在 `node { ... }` 块中）：
     ```dae
     node {
       '我的香港节点': 'vless://你的UUID@服务器域名:443?encryption=none&security=tls&type=ws&sni=服务器域名&path=%2Fws'
       '我的日本节点': 'anytls://你的密码@服务器域名:443/?sni=服务器域名'
     }
     ```
   - 订阅写法（取消 `subscription { ... }` 注释）：
     ```dae
     subscription {
       my_sub: 'https://你的订阅链接'
     }
     ```
2. **（可选）替换 DNS 上游**：
   - 模板默认使用阿里 DoH（国内直连）+ Google DoH（海外经代理），并默认开启去广告、WebRTC 拒连防泄漏、大陆 App/域名/IPv4+IPv6 直连与海外精细分组。
   - 如果你有自定义 DoH / DoH3 (`h3://...`) / DoT (`tls://...:853`)，直接修改 `dns.upstream` 块即可，并记得把你的 DNS 服务器主域名加入 `qname(...) -> bootstrap_udp` 与 `routing` 中的直连规则，避免回环。
3. **保留 Android 关键项（不要删）**：
   - `wan_interface: auto`：自动跟踪 5G 蜂窝与 Wi-Fi 切换，自动重挂 eBPF。
   - `bootstrap_resolver: '223.5.5.5:53'` + `fallback_resolver: '119.29.29.29:53'`：Android 没有 `/etc/resolv.conf`，主备直连 DNS 可确保切网时不失联。
   - `tcp_check_url` 首项保持 HTTPS URL（如 `https://cp.cloudflare.com/generate_204`），配合 `preconnect_node_count: 2` 与 `udp_warm_node_count: 2`，开启完整 UDP / QUIC 预热与节点评分 QUIC 探测。
   - `pname('dnsmasq') -> direct(must)` 与 `sip(192.168.42.0/24) -> direct(must)`：防止手机开 USB 共享网络时误劫持电脑流量。
   - `experimental.clash_api`（`127.0.0.1:9090`）与 `experimental.native_api`（`127.0.0.1:9527`，`config_write: true`，`ui: 'embedded'`）：同时驱动 BoxProxy 内置面板与全功能 Doona 原生面板。

---

### 第 3 步：一键推送并部署到手机

方式 A（推荐）：直接运行随包附带的一键部署脚本：

```bash
bash skill/honk-android/scripts/deploy_honk_android.sh \
  --honk bin/honk-android-arm64 \
  --boxctl bin/boxctl-android-arm64 \
  --config /路径/到你的/config.dae \
  --geoip /路径/到你的/geoip.dat \
  --geosite /路径/到你的/geosite.dat \
  --configure-db \
  --start
```

方式 B（手动命令部署）：

```bash
# 1. 推送到手机临时目录
adb push bin/honk-android-arm64 /data/local/tmp/honk
adb push bin/boxctl-android-arm64 /data/local/tmp/boxctl
adb push /路径/到你的/config.dae /data/local/tmp/config.dae
adb push /路径/到你的/geoip.dat /data/local/tmp/geoip.dat
adb push /路径/到你的/geosite.dat /data/local/tmp/geosite.dat

# 2. 复制到 BoxProxy Honk 目录并修正属主与权限
adb shell su <<'EOF'
BOX=/data/user/0/com.boxproxy.honk/files/box
OWNER=$(stat -c "%u:%g" "$BOX")
mkdir -p "$BOX/bin" "$BOX/honk/data"
cp /data/local/tmp/honk "$BOX/bin/honk"
cp /data/local/tmp/boxctl "$BOX/bin/boxctl"
cp /data/local/tmp/config.dae "$BOX/honk/config.dae"
cp /data/local/tmp/geoip.dat "$BOX/honk/data/geoip.dat"
cp /data/local/tmp/geosite.dat "$BOX/honk/data/geosite.dat"
chown -R "$OWNER" "$BOX/bin/honk" "$BOX/bin/boxctl" "$BOX/honk"
chmod 755 "$BOX/bin/honk" "$BOX/bin/boxctl"
chmod 644 "$BOX/honk/config.dae" "$BOX/honk/data/geoip.dat" "$BOX/honk/data/geosite.dat"
rm -f /data/local/tmp/honk /data/local/tmp/boxctl /data/local/tmp/config.dae /data/local/tmp/geoip.dat /data/local/tmp/geosite.dat
EOF
```

---

### 第 4 步：在 BoxProxy Honk 中选择核心并启动

> **注意**：共存安装时请勿与原版 BoxProxy 同时开启代理服务（启动其中一个前先停止另一个）。

1. 打开手机上的 **`BoxProxy Honk`**。
2. 进入底部 **设置 → 基础代理配置**：
   - **代理核心**：选择 **`Honk`**
   - **代理模式**：选择 **`eBPF`**
   - **配置文件**：选择 **`config.dae`**
3. 返回底部 **首页**，点击 **启动**。
4. 等待 2~3 秒，首页应显示：
   - 状态：**运行中**
   - 核心与模式：**`Honk · eBPF`**
   - 配置：**`config.dae`**
   - 下方自动刷新延迟、WAN IP、实时网速与内存占用。

### 第 5 步（可选）：开启开机自启动

`BoxProxy Honk` 已完整对接独立开机自启脚本 **`/data/adb/service.d/boxproxy-honk.sh`**（与官方原版 `/data/adb/service.d/boxproxy-box.sh` 完全隔离，支持 Magisk / KernelSU / APatch）：

1. 在 `BoxProxy Honk` 底部进入 **设置**，打开 **开机自启** 开关（或部署脚本追加 `--boot-autostart`）。
2. 应用会自动写入 `/data/adb/service.d/boxproxy-honk.sh`（权限 `0755`，`root:root`，Logcat 标签 `BoxProxyHonkBoot`）。
3. 手机重启并解锁屏幕后，脚本会自动等待开机动画结束及 `/data/user/0/com.boxproxy.honk/files/box/` 就绪，再调用 `boxctl --db "$db" boot` 拉起 `honk-core`；`honk-core` 会自动升级根 cgroup BPF 钩子为 `multi` 并通过 `iface_watch` 在网络就绪后自动挂接 WAN 网卡。
4. 排查自启日志可查看 `/data/user/0/com.boxproxy.honk/files/box/run/boot.log` 或 `adb logcat -d -s BoxProxyHonkBoot`。

> **提示**：如已开启 `BoxProxy Honk` 的开机自启，请关闭官方原版 `BoxProxy` 内的「开机自启」（移除 `/data/adb/service.d/boxproxy-box.sh`），避免两个透明代理在开机时同时拉起产生冲突。

---

## 四、使用双面板：BoxProxy 内置面板 + 原生 Doona 全功能面板

本方案同时为你保留了两套面板，互不冲突：

1. **BoxProxy 底部「面板」标签页**（走 `127.0.0.1:9090` Clash API）：
   - 原生 Android 界面，适合日常快速切节点、看策略组、查连接和规则。
2. **BoxProxy 首页「WebUI（Web 界面）」卡片**（直连 `http://127.0.0.1:9527/ui/`）：
   - 直接打开 **`honk` 官方原生 Doona 面板**（已内嵌在 `honk` 二进制内，免密码直接进入）。
   - **全功能已解锁**：支持 **TCP / UDP 分别选节点**、QUIC 评分探测、在线编辑配置/节点/策略组/路由规则、配置导入与修订历史回滚（`honk.db`）、地理数据一键更新、常驻后台连接与内核 UDP 追踪观测、eBPF 数据路径健康度实时监控（`运行中 / 已挂载 / 健康`）、独立 cgroup 全 8 项内存遥测指标，以及在 **设置 → 核心更新** 一键在线检查并升级 `honk-core`。

---

## 五、常见问题排查（FAQ）

### 1. 启动失败怎么办？怎么看日志？
在电脑执行以下命令查看 `honk` 最近日志与配置检查结果：
```bash
adb shell su <<'EOF'
BOX=/data/user/0/com.boxproxy.honk/files/box
"$BOX/bin/honk" -t -d "$BOX/honk/data" -c "$BOX/honk/config.dae"
tail -n 40 "$BOX/run/honk.log"
EOF
```
常见原因：
- `geoip.dat` 或 `geosite.dat` 缺失或路径不对（须位于 `$BOX/honk/data/`）。
- `config.dae` 语法错误（注意：`group { ... }` 中的**策略组名不要加单引号**，如 `Proxy { ... }` 而非 `'Proxy' { ... }`，否则 `routing` 会报 `routing target is not a declared group or builtin`；`node { ... }` 中含空格或特殊字符的节点名才加单引号）。
- 手机内核低于 Linux 6.1 或未开启 eBPF。

### 2. 在 Doona 面板里修改了节点或规则后，为什么直接改 `config.dae` 没变化？
启用全功能模式后，`honk` 默认使用 SQLite 配置库（`$BOX/honk/data/honk.db`）以支持在线编辑与修订历史回滚。首次启动时会自动导入 `config.dae`；之后既可直接在 Doona 面板中可视化编辑或点击「导入」重新导入配置文件，也可在停止服务后删除 `honk.db` 让下次启动重新从 `config.dae` 导入。

### 3. 为什么应用分流规则里的 `pname(...)` 对长包名不生效？
Linux 内核任务名 / `argv[0]` basename 截取上限为 **15 字节**。写 `pname(...)` 规则时，若包名超过 15 个字符，请只取前 15 个字符（例如 `com.instagram.android` 写成 `pname('com.instagram.a')`）。

### 4. 开启 USB 共享网络给电脑后，电脑会被代理吗？
**不会**。`honk` 仅挂接广域网接口（如 `rmnet_data*` / `wlan0`），内核 eBPF 自动放行所有来自 `rndis0`/`ncm0`/`usb0` 的转发流量，且配置模板默认放行了 `dnsmasq`，确保电脑端网络完全独立不受劫持。

### 5. 如何更新或彻底卸载？
- **WebUI 一键在线更新核心（推荐）**：进入 Doona 原生面板的 **设置 → 核心更新**（或在 **活动 → 系统状态** 点击「检查核心更新」），即可向 `Wcoom/honk-android` GitHub Releases 检查新版并自动下载 `honk-android-arm64`，经 SHA-256 与 ELF64 校验后原子替换二进制并自动分离重启。
- **BoxProxy Honk 客户端内更新核心**：进入 `BoxProxy Honk` 底部 **工具 → 更新核心**，在 **Honk** 卡片点击 **检查更新** 即可对比本地与远端版本并直接下载升级。
- **手动替换核心**：停止服务 → 替换 `$BOX/bin/honk`（保持 `0755` 权限）→ 启动服务即可，无需重新配置。
- **彻底卸载**：先在 BoxProxy 首页点击「停止」清理 eBPF 钩子，再正常卸载 BoxProxy 应用即可（无需重启或恢复镜像）。

---

> **给 AI 助手用户的提示**：如果你正在使用 Claude Code / Antigravity / Cursor / Codex / Gemini CLI 等 AI 编程助手，可直接把本目录中的 `skill/honk-android/SKILL.md` 加载为 Skill，让 AI 自动完成环境检查、配置转换、推送部署与全链路验证。
