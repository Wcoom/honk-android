# BoxProxy Honk (`com.boxproxy.honk`) 逆向工程与纯 Rust 重写技术规范

> **基线样本**：`com.boxproxy.honk` · `versionCode = 214` · `versionName = 26.11.23.aa98568` · `minSdk = 29` · `targetSdk = 37` · `compileSdk = 37`  
> **主要证据源**：
> - 反编译源码与资源：`/tmp/honk-rev/boxproxy-jadx/{sources,resources}/`
> - 伴随 Rust 守护/控制程序：`boxproxy/boxctl/src/`
> - Honk 核心原生 API 实现：`src/crates/honk-core/src/native_api/`
>
> 本文档为 `honk-android` 纯 Rust 重写（Wave 1 研究交付物）的唯一行为基准，覆盖 Manifest 与系统入口、持久化存储契约、Root/`boxctl`/多核心执行协议、全部 58 个功能模块清单，以及面向 Wave 3 的"纯 Rust 引擎 + 超薄 Android DEX 桥接壳"架构蓝图。

---

## 目录

1. [Section 1：Android Manifest 与 OS 入口点](#section-1android-manifest-与-os-入口点)
2. [Section 2：数据存储契约（SQLite `box.db`、`shared_prefs` 与文件布局）](#section-2数据存储契约sqlite-boxdbshared_prefs-与文件布局)
3. [Section 3：Root / `boxctl` / 核心执行协议](#section-3root--boxctl--核心执行协议)
4. [Section 4：完整 58 模块功能清单与 API 协议矩阵](#section-4完整-58-模块功能清单与-api-协议矩阵)
5. [Section 5：Wave 3 纯 Rust + 超薄 Android DEX 桥接架构蓝图](#section-5wave-3-纯-rust--超薄-android-dex-桥接架构蓝图)

---

## Section 1：Android Manifest 与 OS 入口点

主要证据：`resources/AndroidManifest.xml`、`resources/res/xml/{backup_rules,data_extraction_rules,locales_config}.xml`、`sources/com/box/app/{AppApplication,MainActivity,PanelSheetActivity,CoreImportActivity}.java`、`sources/com/box/app/tile/BoxServiceTileService.java`、`sources/com/box/app/notification/BoxPersistentNotificationService.java`。

### 1.1 应用清单元信息与构建属性

| 属性 | 值 | 证据位置 |
| :--- | :--- | :--- |
| `package` | `com.boxproxy.honk` | `resources/AndroidManifest.xml:2` |
| `versionCode` / `versionName` | `214` / `26.11.23.aa98568` | `resources/AndroidManifest.xml:3-4` |
| `minSdk` / `targetSdk` / `compileSdk` | `29` / `37` / `37` | `resources/AndroidManifest.xml:5-7` |
| `android:name` | `com.box.app.AppApplication` | `resources/AndroidManifest.xml:25` |
| `android:label` | `@string/app_name`（`"BoxProxy Honk"`） | `resources/AndroidManifest.xml:28`, `values-zh-rCN/strings.xml` |
| `android:usesCleartextTraffic` | `true`（本地 `127.0.0.1` Clash/Native API 与远端 HTTP 面板/测速需要） | `resources/AndroidManifest.xml:34` |
| `android:extractNativeLibs` | `true` | `resources/AndroidManifest.xml:31` |
| `android:enableOnBackInvokedCallback` | `true`（Android 13+ 预测性返回手势） | `resources/AndroidManifest.xml:37` |
| `android:localeConfig` | `@xml/locales_config`（支持 `en`、`zh-CN`、`zh-TW`、`ru`） | `resources/res/xml/locales_config.xml` |
| `allowBackup` / `backup_rules` | `true`；仅保留默认全包含规则，无敏感排除 | `resources/res/xml/{backup_rules,data_extraction_rules}.xml` |
| 内置原生库（`arm64-v8a`） | `libandroidx.graphics.path.so`、`libksvgfilters.so` | `resources/lib/arm64-v8a/` |
| 内置关键 Assets | `box/mihomo/config.yaml`、`boxctl/arm64-v8a/{boxctl,boxbpf}`、`network_platforms/*.svg`（29 个平台图标）、`schema/sing-box-official.json`、`textmate/{darcula,light,languages,languages-js}.json` + `textmate/languages/{javascript,json,shell,yaml}*.json`、`PublicSuffixDatabase.list`、`res/raw/aboutlibraries.json` | `resources/assets/`、`resources/res/raw/` |

### 1.2 声明权限清单（共 14 项）

| 权限 | 类型 / 约束 | 用途与源码证据 |
| :--- | :--- | :--- |
| `android.permission.INTERNET` | Normal | Clash API、Honk Native API、订阅拉取、GitHub 核心/WebUI 更新、网络测速与连通性探测 |
| `android.permission.POST_NOTIFICATIONS` | Runtime（API 33+） | 常驻状态通知（`BoxPersistentNotificationService.java:1040` 检查 `SDK_INT >= 33`） |
| `android.permission.VIBRATE` | Normal | 界面触觉反馈 / 通知辅助 |
| `android.permission.QUERY_ALL_PACKAGES` | Special | 应用管理（`AppManagement`）、CNIP 强制代理应用选择、智能分流应用扫描（`lu.java`） |
| `android.permission.ACCESS_WIFI_STATE` | Normal | 网络匹配（读取当前 Wi-Fi SSID/BSSID，连接状态检测） |
| `android.permission.CHANGE_WIFI_STATE` | Normal | Wi-Fi 状态/连接信息查询辅助 |
| `android.permission.ACCESS_FINE_LOCATION` | Runtime | Android 10+ 读取 Wi-Fi SSID/BSSID 所需定位权限 |
| `android.permission.ACCESS_COARSE_LOCATION` | Runtime | 定位回退权限 |
| `android.permission.ACCESS_NETWORK_STATE` | Normal | 监听网络切换、LAN/WAN IP 卡片判断网络类型 |
| `android.permission.READ_PHONE_STATE` | Runtime | 网络匹配读取蜂窝运营商 `MCC/MNC` 与双卡 `SIM1/SIM2` 槽位（`lp4.java`） |
| `android.permission.FOREGROUND_SERVICE` | Normal | 启动前台常驻通知服务 |
| `android.permission.FOREGROUND_SERVICE_SPECIAL_USE` | Normal（API 34+） | `BoxPersistentNotificationService` 声明 `foregroundServiceType="specialUse"`（`0x40000000`） |
| `android.permission.NEARBY_WIFI_DEVICES` | Runtime（`usesPermissionFlags="neverForLocation"`，API 33+） | Android 13+ 附近 Wi-Fi/热点状态读取 |
| `com.boxproxy.honk.DYNAMIC_RECEIVER_NOT_EXPORTED_PERMISSION` | Signature | AndroidX 动态广播接收器内部隔离权限 |

### 1.3 Application 与全部组件清单（3 Activity + 6 Alias + 2 Service + 1 Provider + 1 Receiver）

#### 1.3.1 `AppApplication`（`sources/com/box/app/AppApplication.java`）
- 持有全局 `Application` 单例引用 `AppApplication.h`（通过 `ob3.B()` / `ob3.q()` 访问）。
- `onCreate()` 执行：
  1. 同步应用图标与磁贴长按 `activity-alias` 状态（`ao.java`，读取 `app_icon_preferences` 中的 `app_icon_variant` 与 `boxapp_settings` 中的 `tile_long_click_target`）。
  2. 初始化 `aw`（`AppSettingsRepository`）与后台协程域（`es.a`），触发常驻通知状态对齐（`kd0.c`）及核心更新检查器（`bc0.java` case 0：延迟 `1500 ms` 后加载 `ki1.f()` 源并检查更新）。
  3. 同步开机自启状态检测与 `box.db` 初始化（延迟按需打开）。

#### 1.3.2 Activity 与 Activity-Alias 清单

| 组件名 | 类型 | `exported` / `enabled` | `launchMode` / 窗口特性 | Intent Filter 与路由目标 | 精确生命周期行为 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `com.box.app.MainActivity` | Activity | `exported=true` | `adjustResize`、`resizeableActivity=true`、Edge-to-Edge | 无直接 `LAUNCHER`（由 alias 代理）；接收通知点击 Intent（`EXTRA_TARGET`: `"app"` / `"web_ui"` / `"panel"`） | `onCreate`/`onNewIntent`：若来自 `BoxTilePreferencesWebUiActivity` 或 extra=`"web_ui"` 则打开 WebUI；若 extra=`"panel"` 则立即以 `action=com.boxproxy.honk.action.OPEN_PANEL_SHEET`、`flags=0x30000000`（`FLAG_ACTIVITY_NEW_TASK \| FLAG_ACTIVITY_CLEAR_TASK`）启动 `PanelSheetActivity` 并 `finish()`；`onStart`/`onStop` 向 `mz2.a` 注册/注销前台在线状态，驱动首页轮询与 Clash API WebSocket 开关（`sources/com/box/app/MainActivity.java`、`p000/v.java`、`p000/h94.java` case 16、`p000/lx.java`）。 |
| `com.box.app.MainLauncherDefault` | `activity-alias` -> `MainActivity` | `exported=true` / `enabled=true` | 默认桌面图标 `@mipmap/ic_launcher` | `MAIN` + `LAUNCHER` | 通过 `PackageManager.setComponentEnabledSetting` 与 `MainLauncherAlternate` 互斥切换（`p000/y6.java`、`R.string.about_app_icon_default`）。 |
| `com.box.app.MainLauncherAlternate` | `activity-alias` -> `MainActivity` | `exported=true` / `enabled=false` | 备用桌面图标 `@mipmap/ic_launcher1` | `MAIN` + `LAUNCHER` | 对应 `app_icon_variant = "alternate"`（`R.string.about_app_icon_alternate`）。 |
| `com.box.app.PanelSheetActivity` | Activity | `exported=false`（通过 alias 暴露） | `singleInstance`、`excludeFromRecents=true`、`finishOnTaskLaunch=true`、`taskAffinity=""`、`Theme.BoxApp.Translucent.Popup`、`dimAmount=0.18f`、`setFinishOnTouchOutside(true)`、API 31+ `windowBlurBehind` | 响应内部 `com.boxproxy.honk.action.OPEN_PANEL_SHEET` 及磁贴长按 alias | 渲染独立底部半透明快捷面板（`p000/h94.java` case 24 -> `ck3.x`）：直接展示 API 联动面板（概览/节点/连接/订阅/规则/规则集/WebUI，遵循 `home_panel_entry_mode`），提供"打开主应用"与无动画关闭回调（`sources/com/box/app/PanelSheetActivity.java`）。 |
| `com.box.app.BoxTilePreferencesDefaultActivity` | `activity-alias` -> `PanelSheetActivity` | `exported=true` / `enabled=true` | `@mipmap/ic_launcher` | `android.service.quicksettings.action.QS_TILE_PREFERENCES` | 长按快捷磁贴默认动作：打开 `PanelSheetActivity`（当 `app_icon_variant == "default"` 且 `tile_long_click_target == "panel"`）。 |
| `com.box.app.BoxTilePreferencesAlternateActivity` | `activity-alias` -> `PanelSheetActivity` | `exported=true` / `enabled=false` | `@mipmap/ic_launcher1` | `QS_TILE_PREFERENCES` | 当切换到备用图标且长按磁贴设为 `"panel"` 时启用，使系统设置页显示匹配图标。 |
| `com.box.app.BoxTilePreferencesWebUiActivity` | `activity-alias` -> `MainActivity` | `exported=true` / `enabled=false` | — | `QS_TILE_PREFERENCES` | 长按磁贴设为 `"web_ui"` 时启用；`MainActivity` 识别 component 名直接跳入 `WebUi` 路由。 |
| `com.box.app.BoxTilePreferencesAppActivity` | `activity-alias` -> `MainActivity` | `exported=true` / `enabled=false` | — | `QS_TILE_PREFERENCES` | 长按磁贴设为 `"app"` 时启用；`MainActivity` 识别 component 名打开主界面首页。 |
| `com.box.app.CoreImportActivity` | Activity | `exported=true` | `Theme.BoxApp.Translucent.Popup`、`excludeFromRecents=true`、`setFinishOnTouchOutside(false)`、`dimAmount=0.18f` | 3 组过滤器：① `ACTION_VIEW`（`content`/`file` + MIME `application/gzip`, `application/x-gzip`, `application/zip`, `application/x-zip-compressed`, `application/x-tar`, `application/octet-stream`）；② `ACTION_VIEW`（`content` + `pathPattern` `.*\.gz`/`.*\.tar\.gz`/`.*\.tgz`/`.*\.tar`/`.*\.zip`）；③ `ACTION_SEND`（上述 6 种 MIME，读取 `EXTRA_STREAM`） | 弹出核心导入对话框（`sources/com/box/app/CoreImportActivity.java`、`p000/qb0.java` default case）：选择目标核心（7 选 1），校验压缩包（上限 192 MiB）与 ELF64 AArch64（`e_machine=183`）头部，通过 root 原子安装至 `files/box/bin/<core>`，若该核心当前正在运行则提示或自动重启。 |

#### 1.3.3 Service、Provider 与 Receiver

1. **`com.box.app.tile.BoxServiceTileService`**（`sources/com/box/app/tile/BoxServiceTileService.java`）
   - 权限：`android.permission.BIND_QUICK_SETTINGS_TILE`；`exported=true`；Action：`android.service.quicksettings.action.QS_TILE`。
   - `onStartListening()`：异步检测核心运行态（`oe0`），刷新磁贴状态（`Tile.STATE_ACTIVE = 2` / `Tile.STATE_INACTIVE = 1`），图标固定为 `R.drawable.ic_stat_box`，主标签 `R.string.tile_service_label`（`"Box Honk"`），副标签展示运行中/已停止/处理中（`R.string.tile_service_working`）。
   - `onClick()`：调用 `unlockAndRun`，在操作期间置为过渡态，若当前运行则执行 `fz2` case 2（`boxctl down`），否则执行 `fz2` case 1（`boxctl up`），完成后自动刷新磁贴与常驻通知。

2. **`com.box.app.notification.BoxPersistentNotificationService`**（`sources/com/box/app/notification/BoxPersistentNotificationService.java`、`p000/hd0.java`、`p000/od0.java`）
   - 属性：`exported=false`、`stopWithTask=false`、`foregroundServiceType="specialUse"`，声明 `<property android:name="android.app.PROPERTY_SPECIAL_USE_FGS_SUBTYPE" android:value="persistent_box_service_status_notification" />`。
   - 通知渠道：ID = `"box_persistent_status"`，重要性 `IMPORTANCE_LOW (2)`，静音、无震动、无角标（`setSound(null, null)`、`enableVibration(false)`、`setShowBadge(false)`）。
   - 前台通知 ID：`1001`；API 34+ 传入 `FOREGROUND_SERVICE_TYPE_SPECIAL_USE (0x40000000)`，否则 `0`；设置 `extras.putBoolean("android.app.preferSmallIcon", true)`。
   - 接收的 Intent Action 与 Extra：
     - `com.boxproxy.honk.notification.ACTION_RELOAD` -> 触发 Clash API `PUT /configs?force=true` 热重载（`od0` case 0 -> `jy` case 8）。
     - `com.boxproxy.honk.notification.ACTION_RESTART` -> 触发 `boxctl restart`（`od0` case 1 -> `fz2` case 0）。
     - `com.boxproxy.honk.notification.ACTION_STOP` -> 触发 `boxctl down`（`od0` case 2 -> `fz2` case 2）。
     - `com.boxproxy.honk.notification.EXTRA_ICON_VARIANT` -> 覆盖图标变体（`"default"` / `"alternate"`）。
   - 通知点击目标（requestCode `100`）：`panel` -> `PanelSheetActivity`（`OPEN_PANEL_SHEET`），`web_ui` -> `MainActivity(EXTRA_TARGET="web_ui")`，`app` -> `MainActivity(EXTRA_TARGET="app")`。
   - 最多 3 个自定义操作按钮（requestCode `101` / `102` / `103`），支持 5 种动作枚举 `xt4`：`open_panel`、`open_web_ui`、`reload`、`stop`、`restart`。
   - 轮询刷新周期：读取 `boxapp_settings.notification_refresh_interval_seconds`（默认 `5` 秒），读取核心运行态、`/proc/<pid>` CPU/内存、`runtime_profile`（仅模板含 `{core}`/`{mode}`/`{config}` 时读库）和 Clash API `/connections`（或系统 `TrafficStats`）实时速率；当核心已停止或权限关闭时自动 `stopForeground(STOP_FOREGROUND_REMOVE)` + `stopSelf()`。
   - 12 个模板变量（`hd0.java`、`BoxPersistentNotificationService.java:806`）：`{status}`、`{pid}`、`{core}`、`{mode}`、`{config}`、`{uptime}`、`{cpu}`、`{memory}`、`{upload}`、`{download}`、`{totalUpload}`、`{totalDownload}`；默认标题 `"BoxProxy Honk"`，默认正文 `"{status} · {uptime}\n↑ {upload} · ↓ {download}\nCPU {cpu} · RAM {memory}"`。

3. **系统基础设施组件**
   - `androidx.startup.InitializationProvider`（`authorities="com.boxproxy.honk.androidx-startup"`，`exported=false`）：初始化 `EmojiCompatInitializer`、`ProcessLifecycleInitializer`、`ProfileInstallerInitializer`、`okhttp3.internal.platform.PlatformInitializer`。
   - `androidx.profileinstaller.ProfileInstallReceiver`（`permission="android.permission.DUMP"`，`exported=true`）：AOT baseline profile 安装/转储。

---

## Section 2：数据存储契约（SQLite `box.db`、`shared_prefs` 与文件布局）

### 2.1 SQLite 数据库：`box.db`（APK Schema v15，共 21 张表）

- **默认路径**：`/data/user/0/com.boxproxy.honk/files/box/box.db`（`ge0.k().e`，由 `Context.getFilesDir()/box/box.db` 解析）。
- **版本**：APK `p000/wc0.java` 将 `PRAGMA user_version` 升级至 **`15`**（开启 WAL / 事务包装，支持从历史旧表 `wifi_match_*` 与 `hotspot_*` 自动迁移）。

#### 2.1.1 全部 21 张表精确定义（证据：`sources/p000/wc0.java:65-390`、`tc0.java`、`qc0.java`、`u61.java`、`w61.java`）

| # | 表名 | 精确列定义（类型与默认值） | 语义与默认种子数据 |
| :-: | :--- | :--- | :--- |
| 1 | `runtime_profile` | `id INTEGER PRIMARY KEY CHECK (id = 1)`<br>`core TEXT NOT NULL DEFAULT 'mihomo'`<br>`network_mode TEXT NOT NULL DEFAULT 'tun'`<br>`proxy_mode TEXT NOT NULL DEFAULT 'blacklist'`<br>`ipv6_mode TEXT NOT NULL DEFAULT 'enable'`<br>`block_quic INTEGER NOT NULL DEFAULT 0`<br>`enable_udp INTEGER NOT NULL DEFAULT 1`<br>`bypass_intranet INTEGER NOT NULL DEFAULT 1`<br>`config_file TEXT NOT NULL DEFAULT ''`<br>`use_custom_proxy_port INTEGER NOT NULL DEFAULT 0`<br>`custom_proxy_port TEXT NOT NULL DEFAULT ''`<br>`use_custom_dns_port INTEGER NOT NULL DEFAULT 0`<br>`custom_dns_port TEXT NOT NULL DEFAULT ''`<br>`use_custom_tun_device INTEGER NOT NULL DEFAULT 0`<br>`custom_tun_device TEXT NOT NULL DEFAULT ''`<br>`use_custom_fakeip_range INTEGER NOT NULL DEFAULT 0`<br>`custom_fakeip_range TEXT NOT NULL DEFAULT ''`<br>`use_custom_fakeip6_range INTEGER NOT NULL DEFAULT 0`<br>`custom_fakeip6_range TEXT NOT NULL DEFAULT ''`<br>`cgroup_memcg INTEGER NOT NULL DEFAULT 0`<br>`cgroup_memcg_limit TEXT NOT NULL DEFAULT ''`<br>`cgroup_cpuset INTEGER NOT NULL DEFAULT 0`<br>`cgroup_cpuset_cpus TEXT NOT NULL DEFAULT ''`<br>`cgroup_blkio INTEGER NOT NULL DEFAULT 0`<br>`cgroup_blkio_weight TEXT NOT NULL DEFAULT ''`<br>`taskset_cpu TEXT NOT NULL DEFAULT ''`<br>`clean_vendor_firewall TEXT NOT NULL DEFAULT 'enable'`<br>`boot_auto_start INTEGER NOT NULL DEFAULT 0` *(兼容迁移旧列：`tproxy_port`/`redir_port`/`mihomo_dns_port`/`tun_device`/`fake_ip_range`/`fake_ip6_range`)* | 单行核心运行时配置。`core` ∈ `{mihomo, meow, sing-box, xray, v2fly, hysteria, honk}`；`network_mode` ∈ `{tun, tproxy, ebpf, redirect, mixed, enhance}`；`proxy_mode` ∈ `{blacklist, whitelist}`；`ipv6_mode` ∈ `{enable, bypass, disable}`；`clean_vendor_firewall` ∈ `{enable, disable}`。 |
| 2 | `app_selection` | `uid INTEGER PRIMARY KEY` | 应用分流黑/白名单 UID 集合（含多用户 `userId * 100000 + appId` 编码）。 |
| 3 | `app_settings` | `key TEXT PRIMARY KEY`<br>`value TEXT NOT NULL` | 通用键值配置表，承载：① `app_language`（`"system"`/`"zh-CN"`/`"zh-TW"`/`"en"`/`"ru"`）；② `clashapi_history_enabled`（`"0"`/`"1"`）、`clashapi_history_interval_seconds`、`clashapi_history_retention_days`、`clashapi_history_max_database_mib`（默认 `"128"`）；③ `shell_scripts_pre_start`、`shell_scripts_post_stop`、`shell_scripts_last_run:<name>`（JSON `{exitCode, atMillis}`）；④ 30 个多核心配置覆写键 `config_override.<core>.<option>`（见下表 2.1.2）。 |
| 4 | `app_gid_list` | `id INTEGER PRIMARY KEY AUTOINCREMENT`<br>`value TEXT NOT NULL UNIQUE` | GID 分流豁免/纳入列表。 |
| 5 | `cnip_settings` | `id INTEGER PRIMARY KEY CHECK (id = 1)`<br>`bypass_cnip INTEGER NOT NULL DEFAULT 0`<br>`cnip_mode TEXT NOT NULL DEFAULT 'ipset'`<br>`bypass_ipv4 INTEGER NOT NULL DEFAULT 1`<br>`bypass_ipv6 INTEGER NOT NULL DEFAULT 0`<br>`ipv4_url TEXT NOT NULL DEFAULT 'https://github.com/Hackl0us/GeoIP2-CN/raw/release/CN-ip-cidr.txt'`<br>`ipv4_file TEXT NOT NULL DEFAULT '<files>/box/run/state/cnip/cn_ip.txt'`<br>`ipv4_source TEXT NOT NULL DEFAULT 'url'`<br>`ipv6_url TEXT NOT NULL DEFAULT 'https://ispip.clang.cn/all_cn_ipv6.txt'`<br>`ipv6_file TEXT NOT NULL DEFAULT '<files>/box/run/state/cnip/cn_ipv6.txt'`<br>`ipv6_source TEXT NOT NULL DEFAULT 'url'`<br>`ipv4_additions TEXT NOT NULL DEFAULT ''`<br>`ipv6_additions TEXT NOT NULL DEFAULT ''` | CNIP 绕过设置。`cnip_mode` ∈ `{ipset, ebpf}`；`ipv4_source`/`ipv6_source` ∈ `{url, file}`。 |
| 6 | `cnip_force_uids` | `uid INTEGER PRIMARY KEY` | CNIP 绕过例外（强制走代理）的 UID 列表。 |
| 7 | `cnip_force_apps` | `id INTEGER PRIMARY KEY AUTOINCREMENT`<br>`value TEXT NOT NULL UNIQUE` | CNIP 强制代理的应用包名（格式 `userId:packageName` 或纯 UID 字符串，保存时解析同步至 `cnip_force_uids`，`y3.java:404-430`）。 |
| 8 | `network_match_settings` | `id INTEGER PRIMARY KEY CHECK (id = 1)`<br>`network_control_enabled INTEGER NOT NULL DEFAULT 0`<br>`use_on_wifi_disconnect INTEGER NOT NULL DEFAULT 0`<br>`use_on_wifi_connect INTEGER NOT NULL DEFAULT 0`<br>`enable_ssid_matching INTEGER NOT NULL DEFAULT 0`<br>`enable_network_control_log INTEGER NOT NULL DEFAULT 0`<br>`list_mode TEXT NOT NULL DEFAULT 'blacklist'`<br>`matched_action TEXT NOT NULL DEFAULT 'stop'`<br>`unmatched_action TEXT NOT NULL DEFAULT 'start'`<br>`matched_config_name TEXT NOT NULL DEFAULT ''`<br>`unmatched_config_name TEXT NOT NULL DEFAULT ''`<br>`matched_script_name TEXT NOT NULL DEFAULT ''`<br>`unmatched_script_name TEXT NOT NULL DEFAULT ''`<br>`switch_config INTEGER NOT NULL DEFAULT 0`<br>`matched_proxy_mode TEXT NOT NULL DEFAULT ''`<br>`unmatched_proxy_mode TEXT NOT NULL DEFAULT ''` | 网络匹配全局开关与旧版匹配/不匹配动作兼容字段（由旧表 `wifi_match_settings` 迁移而来）。 |
| 9 | `network_match_ssids` | `id INTEGER PRIMARY KEY AUTOINCREMENT`<br>`value TEXT NOT NULL UNIQUE` | 匹配 Wi-Fi SSID 列表（从 `wifi_match_ssids` 迁移）。 |
| 10 | `network_match_bssids` | `id INTEGER PRIMARY KEY AUTOINCREMENT`<br>`value TEXT NOT NULL UNIQUE` | 匹配 Wi-Fi BSSID 列表（从 `wifi_match_bssids` 迁移）。 |
| 11 | `network_match_conditions` | `id INTEGER PRIMARY KEY AUTOINCREMENT`<br>`type TEXT NOT NULL`<br>`value TEXT NOT NULL DEFAULT ''`<br>`enabled INTEGER NOT NULL DEFAULT 1`<br>`sort_order INTEGER NOT NULL DEFAULT 0` | 结构化网络条件表。`type` ∈ `{wifi, wifi_ssid, wifi_bssid, cellular, mcc_mnc, sim1, sim2}`（`lp4.java`）。 |
| 12 | `network_rule_groups` | `id INTEGER PRIMARY KEY AUTOINCREMENT`<br>`name TEXT NOT NULL`<br>`enabled INTEGER NOT NULL DEFAULT 1`<br>`priority INTEGER NOT NULL DEFAULT 0`<br>`expression TEXT NOT NULL DEFAULT ''` | 高级网络规则组（按 `priority ASC, id ASC` 求值）。`expression` 为 JSON 布尔表达式 AST，节点含 `kind`（`leaf`/`op`）、`op`（`and`/`or`/`not`）、`children`、`type`（7 种条件）、`value`、`disabled`（`tn4.java`、`tc0.java`）。 |
| 13 | `network_rule_actions` | `id INTEGER PRIMARY KEY AUTOINCREMENT`<br>`group_id INTEGER NOT NULL`<br>`sort_order INTEGER NOT NULL DEFAULT 0`<br>`type TEXT NOT NULL`<br>`target TEXT NOT NULL DEFAULT ''`<br>`continue_on_error INTEGER NOT NULL DEFAULT 0` | 关联规则组（`group_id`）的动作流水线（启动/停止/重启/切换配置/执行脚本等）。 |
| 14 | `shared_network_settings` | `id INTEGER PRIMARY KEY CHECK (id = 1)`<br>`mac_filter INTEGER NOT NULL DEFAULT 0`<br>`mac_mode TEXT NOT NULL DEFAULT 'blacklist'` | 热点/共享网络 MAC 过滤模式（从 `hotspot_settings` 迁移）。 |
| 15 | `shared_network_interface_settings` | `id INTEGER PRIMARY KEY CHECK (id = 1)`<br>`initialized INTEGER NOT NULL DEFAULT 0` | 标记共享网络接口列表是否已初始化默认接口（如 `ap0`/`wlan+`/`rndis+`）。 |
| 16 | `shared_network_interfaces` | `id INTEGER PRIMARY KEY AUTOINCREMENT`<br>`value TEXT NOT NULL UNIQUE` | 纳入共享代理的网卡名列表（从 `hotspot_ap_interfaces` 迁移）。 |
| 17 | `blocked_interfaces` | `id INTEGER PRIMARY KEY AUTOINCREMENT`<br>`value TEXT NOT NULL UNIQUE` | 显式禁止代理/绕过的网卡名列表。 |
| 18 | `shared_network_macs` | `id INTEGER PRIMARY KEY AUTOINCREMENT`<br>`value TEXT NOT NULL UNIQUE` | MAC 黑/白名单（从 `hotspot_macs` 迁移）。 |
| 19 | `intranet_bypass_settings` | `id INTEGER PRIMARY KEY CHECK (id = 1)`<br>`initialized INTEGER NOT NULL DEFAULT 0` | 标记默认内网绕过 CIDR 列表是否已填充。 |
| 20 | `intranet_ipv4_cidrs` | `id INTEGER PRIMARY KEY AUTOINCREMENT`<br>`value TEXT NOT NULL UNIQUE` | 自定义 IPv4 内网绕过网段。默认种子 16 条：`0.0.0.0/8`, `10.0.0.0/8`, `100.64.0.0/10`, `127.0.0.0/8`, `169.254.0.0/16`, `172.16.0.0/12`, `192.0.0.0/24`, `192.0.2.0/24`, `192.88.99.0/24`, `192.168.0.0/16`, `198.18.0.0/15`, `198.51.100.0/24`, `203.0.113.0/24`, `224.0.0.0/4`, `240.0.0.0/4`, `255.255.255.255/32`。 |
| 21 | `intranet_ipv6_cidrs` | `id INTEGER PRIMARY KEY AUTOINCREMENT`<br>`value TEXT NOT NULL UNIQUE` | 自定义 IPv6 内网绕过网段。默认种子 12 条：`::/128`, `::1/128`, `::ffff:0:0/96`, `64:ff9b::/96`, `100::/64`, `2001::/32`, `2001:10::/28`, `2001:20::/28`, `2001:db8::/32`, `2002::/16`, `fc00::/7`, `fe80::/10`（外加组播 `ff00::/8` 由 `boxctl` 补充）。 |

#### 2.1.2 `app_settings` 中的 `config_override.<core>.<option>` 覆写键（30 项，证据：`p000/u61.java`、`w61.java`）

支持核心：`mihomo`、`meow`、`sing-box`、`honk`（按核心支持项过滤）；键格式为 `config_override.<core>.<option>`，覆盖 7 大类：
1. **基础与模式**：`override_enabled`、`override_dns`、`override_tun`、`override_tproxy`、`override_redir`、`override_external_controller`、`override_secret`、`override_external_ui`、`override_ipv6`、`override_unified_delay`、`override_tcp_concurrent`、`override_find_process_mode`、`override_log_level`
2. **端口覆写**：`tproxy_port`、`redir_port`、`mixed_port`、`dns_listen`
3. **TUN 参数**：`tun_stack`、`tun_device`、`tun_auto_route`、`tun_strict_route`、`tun_auto_detect_interface`、`tun_mtu`
4. **DNS / FakeIP**：`dns_enhanced_mode`、`fake_ip_range`、`fake_ip6_range`、`dns_prefer_h3`
5. **嗅探 / Geo / 脚本**：`sniffer_enable`、`geodata_mode`、`scripts_enabled`

#### 2.1.3 关键发现：APK (v15) 与仓库/内置 `boxctl` (v3) 的 Schema 差异（Wave 3 必解项）

交叉比对 `sources/p000/wc0.java` 与 `boxproxy/boxctl/src/db/{schema,read}.rs` 及 APK 资产内嵌二进制 `resources/assets/boxctl/arm64-v8a/boxctl` 的字符串审计，发现**严重的契约漂移**：

| 维度 | APK (`wc0.java`, v214) | 仓库与内嵌 `boxctl` (`boxctl/src/db/schema.rs`) | 影响与 Wave 3 对齐策略 |
| :--- | :--- | :--- | :--- |
| `PRAGMA user_version` | `15`（21 张表） | `3`（16 张表） | 两者共享同一 `box.db` 文件时，`boxctl` 忽略了 APK v4–v15 新增表与重命名表！ |
| 网络匹配表 | `network_match_settings` + `network_match_{ssids,bssids,conditions}` + `network_rule_{groups,actions}` | 仅读写旧表 `wifi_match_settings` + `wifi_match_{ssids,bssids}` | APK 新建规则组与 `mcc_mnc`/`sim1`/`sim2` 条件不会自动生效，除非 `boxctl` 同步升级至 v15 读取逻辑。 |
| 热点/共享网络表 | `shared_network_settings` + `shared_network_{interface_settings,interfaces,macs}` + `blocked_interfaces` | 仅读写旧表 `hotspot_settings` + `hotspot_ap_interfaces` + `hotspot_macs` | 同上；Wave 3 必须在 Rust 统一存储 crate (`boxproxy-store`) 中以 v15 为唯一真源并兼容旧表迁移。 |
| `meow` 核心 | 完整枚举支持（`ar5.java`、`hi1.java`、`cr0.java`） | `boxctl/src/config/core_values.rs` 未注册 `meow` | Wave 3 在 Rust 后端统一核心枚举（含 `honk`/`mihomo`/`meow`/`sing-box`/`xray`/`v2fly`/`hysteria`）。 |
| Clash API 历史采集与 Socket | 调用 `boxctl clashapi init`/`clear`，并连接 Unix Domain Socket `files/box/run/state/runtime/clashapi-history.sock`（`nm0.java:26`, `rs0.java`, `ps0.java`） | `boxctl/src/main.rs` 中 `Command::Clashapi => Ok(())` 为空实现，内嵌二进制无 `clashapi-history.sock` | 概览页"热门数据/范围流量历史"在当前内嵌 `boxctl` 下不可用；Wave 3 可由 `boxproxy-engine` 或 `honk-core` 原生 API `/api/v1/runtime/traffic/history` 直接驱动！ |

---

### 2.2 `shared_prefs/*.xml` 全量契约（共 24 个文件）

23 个纳入 `bo.java` ZIP 备份范围，外加 1 个运行缓存 `core_update_monitor`（不备份）。证据：`sources/p000/bo.java:6`、`ar.java:903-1097` 及全局 `getSharedPreferences` 静态扫描。

| # | 文件名（不含 `.xml`） | 备份 | 完整 Key / 类型 / 默认值 | 归属模块 |
| :-: | :--- | :-: | :--- | :--- |
| 1 | `boxapp_settings` | 是 | `ui_mode` (String, `"material"`\|`"miuix"`, 默认按 ROM 或 `"material"`)；`app_language` (String, `"system"`)；`home_panel_entry_mode` (String, `"overview"`)；`auto_start_enabled` (Bool, `false`)；`accelerated_download_enabled` (Bool, `false`)；`accelerated_download_prefix` (String, `null`)；`github_token_enabled` (Bool, `false`)；`github_token` (String, `null`)；`notifications_enabled` (Bool, `false`)；`notification_refresh_interval_seconds` (Int, `5`)；`notification_title_template` (String, `"BoxProxy Honk"`)；`notification_content_template` (String, `"{status} · {uptime}\n↑ {upload} · ↓ {download}\nCPU {cpu} · RAM {memory}"`)；`notification_click_target` (String, `"panel"`)；`notification_action_reload` (Bool, `true`)；`notification_action_reload_target` (String, `"reload"`)；`notification_action_reload_label` (String, `""`)；`notification_action_stop` (Bool, `true`)；`notification_action_stop_target` (String, `"stop"`)；`notification_action_stop_label` (String, `""`)；`notification_action_restart` (Bool, `true`)；`notification_action_restart_target` (String, `"restart"`)；`notification_action_restart_label` (String, `""`)；`open_panel_on_launch` (Bool, `false`)；`show_proxy_selector_bottom_bar_item` (Bool, `true`)；`tile_long_click_target` (String, `"panel"`)；`theme_mode` (Int, `0`=System/`1`=Light/`2`=Dark)；`pure_black_dark_theme` (Bool, `false`)；`miuix_monet_enabled` (Bool, `false`)；`accent_color` (Int, `-11554992` = `#FF4FB1D0` 派生)；`accent_use_default` (Bool, `true`)；`theme_palette_style` (String, `null` -> `TonalSpot`)；`theme_color_spec` (String, `null` -> `Spec2021`/`Spec2025`)；`enable_blur` (Bool, `true`)；`top_bar_style` (String, `null` -> `Classic`/`Glass`)；`bottom_bar_style` (String, `null` -> `Classic`/`Floating`/`LiquidGlass`/`Glass`)；`floating_bottom_bar` (Bool, `false`)；`floating_bottom_bar_blur` (Bool, `false`)；`landscape_glass_navigation` (Bool, 依系统)；`enable_predictive_back_animation` (Bool, `true`)；`predictive_back_animation_follow_edge` (Bool, 依系统)；`enable_swipe_back` (Bool, 依系统)；`use_net_speed_clash_api` (Bool, 默认 `false`/`true`)；`speed_test_source` (String, `null` -> `"cloudflare"`)；`speed_test_use_megabytes` (Bool, `false`)；`use_subscription_detail_clash_api` (Bool, `false`)；`page_scale` (Float, `1.0f`)；`welcome_onboarding_shown` (Bool, `false`) | 全局设置（`aw.java`/`vv.java`/`ar.java` 共 45+ 项） |
| 2 | `web_panel_preferences` | 是 | `webui_local_panel_mode` (String, `"auto"`\|`"clash"`\|`"sing_box"`)；`webui_selected_panel` (String, `"local"`)；`substore_selected_panel` (String, `"local"`)；`webui_custom_panels` (JSON Array String, `"[]"`)；`substore_custom_panels` (JSON Array String, `"[]"`) | WebUI 与 Sub-Store 面板（`fz7.java`, `p97.java`, `rj7.java`） |
| 3 | `proxy_selector_preferences` | 是 | `custom_api_enabled` (Bool, `false`)；`custom_api_endpoint` (String, `""`)；`custom_api_secret` (String, `""`)；`custom_latency_test_url_enabled` (Bool, `false`)；`custom_latency_test_url` (String, `"https://connectivitycheck.platform.hicloud.com/generate_204"`)；`policy_icon_pack_urls` (JSON Array String, `null`)；`policy_icon_pack_url` (String, `""`，旧版迁移)；`policy_icon_overrides` (JSON Object String, `null`)；`node_sort_mode` (String, `"defaultsort"`)；`show_hidden_groups` (Bool, `true`)；`show_unavailable_nodes` (Bool, `true`)；`group_by_provider` (Bool, `false`)；`detect_ipv6_on_latency_test` (Bool, `false`)；`name_overflow_mode` (String, `null` -> `"clip"`\|`"scroll"`\|`"wrap"`)；`scroll_overflowing_node_names` (Bool, `false`)；`display_global_by_mode` (Bool, `false`)；`disconnect_on_select` (Bool, `false`)；`collapse_previous_group_on_expand` (Bool, `false`)；`expand_selected_policy_in_bottom_sheet` (Bool, `false`)；`node_column_mode` (String, `"auto"`\|`"fixed"`)；`node_column_count` (Int, `1`，范围 `1..12`)；`group_column_mode` (String, `"auto"`\|`"fixed"`)；`group_column_count` (Int, `1`，范围 `1..12`)；`node_density` (String, fallback `density` -> `"standard"`)；`group_density` (String, fallback `density` -> `"standard"`) | 策略页与自定义 API（`sp5.java`, `kq5.java`, `ml3.java`, `vk.java`） |
| 4 | `api_config_reveal_preferences` | 是 | `api_config_endpoint_revealed` (Bool, `false`)；`api_config_secret_revealed` (Bool, `false`) | API 配置页敏感字段显隐状态（`vk.java`, `bk.java`） |
| 5 | `latency_settings` | 是 | `latency_name_1` (String, `"Baidu"`)；`latency_url_1` (String, `"https://baidu.com"`)；`latency_name_2` (String, `"Google"`)；`latency_url_2` (String, `"https://google.com"`)；`latency_name_3` (String, `"Cloudflare"`)；`latency_url_3` (String, `"https://cloudflare.com"`) | 首页三目标延迟测试（`cc0.java`, `wh3.java`） |
| 6 | `app_icon_preferences` | 是 | `app_icon_variant` (String, `"default"`\|`"alternate"`) | 应用桌面图标变体（`y6.java`, `BoxPersistentNotificationService.java`） |
| 7 | `tools_app_management` | 是 | 应用列表筛选/排序偏好（系统应用显示、分组筛选等，`lu.java`） | 工具-应用管理 |
| 8 | `tools_core_update` | 是 | `selected_core_id` (String, `null` -> 索引 0 `"mihomo"`)；`sources_v1` (JSON Array String，存储自定义/内置更新源对象列表 `{id, name, target, kind, location, ...}`)；`meow_source_added` (Bool, `false`)；`honk_source_added` (Bool, `false`) | 核心更新源管理（`ki1.java`, `hm7.java`） |
| 9 | `core_update_settings` | 是 | `mihomo_stable` (Bool, `true`)；`sing_box_stable` (Bool, `true`) | Mihomo / Sing-Box 稳定版与测试版通道开关（`hi1.java`, `gm7.java`） |
| 10 | `core_update_monitor` | **否** | `cache` (JSON Object String, `"{}"`，各核心远端检查结果缓存)；`ignored_versions` (JSON Object String, `"{}"`，被用户忽略提示的版本映射) | 后台更新提示缓存（`of1.java`, `z21.java`） |
| 11 | `connection_preferences` | 是 | `sort_mode` (String, `"Host"`)；`reverse_sort` (Bool, `false`)；`source_ip_filter` (String, `null`)；`filters_<scope>` (JSON Object String `{version:1, sourceIp, rules:[{id,name,pattern,mode,fields,ignoreCase,enabled}]}`) | 连接详情筛选与排序（`lx.java:121-148, 586-629`） |
| 12 | `net_speed_connection_preferences` | 是 | 旧版连接偏好（启动时若 `connection_preferences` 为空则自动全量迁移，`lx.java:320-359`） | 连接页历史兼容 |
| 13 | `home_ui_preferences` | 是 | `home_ip_mode` (String, `"wan"`\|`"lan"`) | 首页 IP 卡片 LAN/WAN 切换（`hw.java`, `zy2.java`） |
| 14 | `home_subscription_cache` | 是 | `subscription_settings_json` (JSON Object String，订阅展示配置)；`subscription_cache_local_json` (JSON String，本地订阅流量解析缓存) | 首页订阅流量卡片与缓存（`ao.java`, `h13.java`, `m5.java`） |
| 15 | `tools_logs` | 是 | 日志查看器选中文件、级别过滤、自动滚动偏好（`zy7.java`） | 工具-日志查看 |
| 16 | `runtime_file_editor_prefs` | 是 | `word_wrap` (Bool, `true`)；`sticky_scroll` (Bool, `false`) | Sora 代码编辑器（运行时文件，`dw.java`） |
| 17 | `module_file_editor_prefs` | 是 | `word_wrap` (Bool, `true`)；`sticky_scroll` (Bool, `false`) | Sora 代码编辑器（模块/脚本文件，`dw.java`） |
| 18 | `runtime_files_download_cache` | 是 | 键为相对路径，值为 JSON `{url, userAgent}`，用于"从原 URL 一键更新文件"（`rl3.java`, `e64.java:636`） | 运行时文件/脚本下载来源记录 |
| 19 | `download_preferences` | 是 | `user_agent` (String, `"boxproxy"`) | 下载文件默认 User-Agent（`nb3.java`, `c54.java`, `y96.java`） |
| 20 | `shell_scripts` | 是 | `pre_start_script` / `post_stop_script`（旧版迁移源，`wc0.java`）；`script_<base64>_at` (Long, `0L`)；`script_<base64>_code` (Int, `-1`)（`wr6.java:74-81`） | Shell 脚本上次运行记录 |
| 21 | `webui_update_cache` | 是 | `target_path` (String, 默认 `<files>/box/mihomo/dashboard`)；`download_url` (String, 默认 `"https://github.com/Zephyruso/zashboard/releases/latest/download/dist.zip"`) | WebUI 更新工具缓存（`hm7.java:27-35`） |
| 22 | `overview_preferences` | 是 | `auto_expand_top_traffic` (Bool, `false`)；`top_traffic_window` (String, `"30m"`)；`top_traffic_sort_descending` (Bool, `true`)；`top_traffic_limit` (Int, `10`, `1..100`)；`top_traffic_sort` (String, `"download"`)；`realtime_rank_dimension` (String, `null` -> `"outbound"`)；`realtime_rank_limit` (Int, `5`, `1..100`)；`realtime_rank_sort` (String, `"connections"`)；`realtime_rank_sort_descending` (Bool, `true`) | 概览页实时排行与热门历史（`ck3.java`, `t35.java`, `a35.java`） |
| 23 | `network_speed_history` | 是 | `entries` (JSON Array String，网络测速历史记录列表，`mq4.java`) | 测速历史页 |
| 24 | `zashboard_monet_panels` | 是 | `urls` (StringSet，已自动识别为 Zashboard 且需注入动态 Monet CSS 的 URL 集合，`j87.java`, `t50.java`) | WebUI Monet 动态取色白名单 |

### 2.3 运行时文件目录布局（`files/box/` 与外部路径）

证据：`sources/p000/ge0.java`、`wd0.java`、`bo.java`、`xo.java`、`z27.java`、`e64.java`、`wr6.java`。

```text
/data/user/0/com.boxproxy.honk/files/box/          # wd0.a (homeDir)
├── box.db                                         # wd0.e (SQLite v15，含 -wal/-shm)
├── bin/                                           # wd0.b (chown root:net_admin, chmod 0755)
│   ├── boxctl                                     # wd0.f (从 assets/boxctl/arm64-v8a/boxctl 按 SHA-256 同步)
│   ├── boxbpf                                     # wd0.g (从 assets/boxctl/arm64-v8a/boxbpf 按 SHA-256 同步)
│   ├── honk                                       # Honk 核心二进制
│   ├── mihomo                                     # Mihomo 核心二进制
│   ├── meow                                       # Meow 核心二进制
│   ├── sing-box                                   # Sing-Box 核心二进制
│   ├── xray                                       # Xray 核心二进制
│   ├── v2fly                                      # V2Fly 核心二进制
│   └── hysteria                                   # Hysteria 核心二进制
├── run/                                           # wd0.c (runDir)
│   ├── box.pid                                    # 当前核心进程 PID
│   ├── box.identity                               # 进程校验指纹（cmdline/exe/start_time）
│   ├── runs.log                                   # boxctl 运行主日志
│   ├── boot.log                                   # 开机自启脚本日志
│   ├── net.log                                    # 网络监控/匹配日志
│   ├── <core>.log                                 # 各核心 stdout/stderr 日志
│   ├── rule-mirror/<core>/                        # 规则集本地镜像缓存（z46.java:47）
│   └── state/                                     # wd0.d (stateDir)
│       ├── locks/                                 # control.lock 等并发互斥锁
│       ├── config/
│       │   ├── startup-config                     # wd0.d() 当前生成的启动配置（Clash API 发现读取点）
│       │   ├── mihomo-proxy-providers.yaml        # 订阅管理覆写文件
│       │   ├── mihomo-scripts.yaml                # Mihomo 覆写 JS 脚本执行清单
│       │   ├── meow-scripts.yaml                  # Meow 覆写 JS 脚本执行清单
│       │   └── sing-box-scripts.yaml              # Sing-Box 覆写 JS 脚本执行清单
│       ├── cache/                                 # 临时下载/校验缓存
│       ├── history/                               # 历史统计数据目录
│       ├── monitor/                               # 网络监听状态
│       ├── runtime/
│       │   ├── clashapi-history.sock              # Clash API 历史采集 Unix Domain Socket（nm0.java:26）
│       │   └── runtime.iptables.{env,json}        # iptables 规则运行态快照
│       ├── cnip/
│       │   ├── cn_ip.txt                          # IPv4 CNIP CIDR 列表
│       │   └── cn_ipv6.txt                        # IPv6 CNIP CIDR 列表
│       ├── ebpf/                                  # boxbpf 挂载与映射状态
│       └── ipset.stamp                            # ipset 规则更新戳
├── honk/                                          # Honk 配置目录（支持 *.dae，运行数据在 honk/data）
├── mihomo/                                        # Mihomo 配置目录（自动释放 assets/box/mihomo/config.yaml）
│   ├── config.yaml
│   ├── dashboard/                                 # 默认本地 WebUI 目录（hm7.java:32）
│   └── scripts/*.js                               # Mihomo JS 覆写脚本（e64.java）
├── meow/                                          # Meow 配置目录（含 meow/scripts/*.js）
├── sing-box/                                      # Sing-Box 配置目录（含 sing-box/scripts/*.js）
├── xray/                                          # Xray 配置目录（*.json）
├── v2fly/                                         # V2Fly 配置目录（*.json）
├── hysteria/                                      # Hysteria 配置目录（*.yaml/*.yml）
└── scripts/                                       # 用户 Shell 钩子脚本（*.sh，wr6.java:49）

外部系统路径（经 root 访问）：
├── /data/adb/service.d/boxproxy-honk.sh           # Magisk / KernelSU / APatch 开机自启脚本（ge0.java:530）
├── /data/adb/modules/sub_store                    # Sub-Store 模块安装探测目录（sn.java:505）
└── /data/local/sub_store/scripts/sub_store.env    # Sub-Store 本地端口与后端路径配置（z27.java:38）
```

---

## Section 3：Root / `boxctl` / 核心执行协议

### 3.1 Root Shell 连接池与命令安全转义（`sources/p000/v16.java`、`ge0.java`）

- **底层机制**：使用 `libsu` 持久 Root Shell 池（`v16.java`），最多维护 **2 个复用会话**；单个 Shell 达到 **120 条命令** 或 **空闲 5 分钟** 后自动回收重建。
- **Root 权限检测**：`v16.i()` 执行 `id -u`，返回 stdout 去空白后精确等于 `"0"` 即视为已授权。
- **Shell 转义**：所有路径与参数通过 `ge0.s(str)` 包裹 POSIX 单引号：`"'" + str.replace("'", "'\\''") + "'"`。
- **超时分级**：
  - 常规文件/系统探测：`5s`–`60s`（默认 `v16.f(60000L, ...)`）
  - 轻量 `boxctl` 操作：`clashapi init`/`clear` `8s`、`monitor`/`resource apply` `10s`、`config sync`/`mode renew`/`network-match apply` `15s`（手动重新生成启动配置为 `60s`）
  - 服务生命周期：`down` `50s`、`up` `75s`、`restart` `125s`

### 3.2 `boxctl` 与 `boxbpf` 资产同步协议（`ge0.java:548-799, 1070-1252`）

每次调用 `boxctl` 前由互斥锁 `ge0.c`（`Mutex`）串行执行 `ge0.l()`：
1. 依次同步 `assets/boxctl/arm64-v8a/boxctl` -> `files/box/bin/boxctl` 与 `assets/boxctl/arm64-v8a/boxbpf` -> `files/box/bin/boxbpf`。
2. 计算资产字节数组 SHA-256 与目标文件 SHA-256；若文件存在、字节长度一致且 SHA-256 相同，则跳过写入，仅确保 `chmod 755`。
3. 若需更新：先写入同目录随机临时文件 `.<name>.<uuid>.tmp` -> 回读校验 SHA-256 -> `File.renameTo` 原子替换（权限不足时回退 root `mv -f <tmp> <dst>`）-> root `chmod 755 <dst>`。

### 3.3 `boxctl` 子命令完整映射表

命令构造器 `ge0.c(subcmd)` 统一生成：`'<files>/box/bin/boxctl' --db '<files>/box/box.db' <subcmd>`。下表将 APK 全部调用与 `boxproxy/boxctl/src/cli.rs` 逐一交叉对照：

| APK 调用子命令 | 超时 | APK 证据位置 | `boxctl` 实现（`boxctl/src/cli.rs`） | 功能与退出行为 |
| :--- | :-: | :--- | :--- | :--- |
| `up` | `75s` | `fz2.java:283`、`hz2.java:133` | `Command::Up` -> `control::up` | 前置检查核心二进制与配置文件（可无配置仅靠脚本的核心：`mihomo`/`meow`/`sing-box`）；启动核心并下发路由/iptables/eBPF；APK 随后轮询运行态 `8 × 400 ms`，刷新 Tile/通知并执行 `ge0.u()`。 |
| `down` | `50s` | `fz2.java:433` | `Command::Down` -> `control::down` | 停止核心进程并清理路由/规则；APK 轮询停止态 `30 × 400 ms`，刷新 Tile/通知。 |
| `restart` | `125s` | `fz2.java:133`、`bc0.java:348,655`、`hi1.java:656` | `Command::Restart` -> `control::restart` | 原子重启核心与规则（eBPF 模式下配置变更必须走 `restart` 而非 `mode renew`，`bc0.java:346`）；APK 轮询运行态 `8 × 400 ms` 并调用 `ge0.u()`。 |
| `boot` | 无（后台） | `ge0.java:522`（开机脚本内） | `Command::Boot` -> `control::boot` | 仅由 `/data/adb/service.d/boxproxy-honk.sh` 调用；读取 `runtime_profile.boot_auto_start`，为 `1` 时启动核心与网络监听。 |
| `config sync` | `15s` / `60s` | `bc0.java:369,616` | `Command::Config { ConfigCommand::Sync }` | 根据 `box.db` 与覆写选项重新生成 `run/state/config/startup-config`（`xray`/`v2fly` 不支持）。 |
| `mode renew` | `15s` | `bc0.java:306,870` | `Command::Mode { ModeCommand::Renew }` | 热刷新透明代理路由与防火墙规则（绕过规则/应用管理/共享网络/CNIP 保存后若服务运行中且非 eBPF 模式调用）。 |
| `resource apply` | `10s` | `fr5.java:57` | `Command::Resource { ResourceCommand::Apply }` | 热应用 cgroup（`memcg`/`cpuset`/`blkio`）与 `taskset_cpu` 亲和性到运行中核心进程。 |
| `network-match apply` | `15s` | `bc0.java:1057` | `Command::NetworkMatch { NetworkMatchCommand::Apply }` | 立即按当前网络状态评估网络匹配策略；随后 APK 自动调用 `ge0.u()` 确保监听守护进程存活。 |
| `monitor` | `10s` | `ge0.java:1026,1054` | `Command::Monitor` -> `monitor::ensure` | 启动/保活 `boxctl` 后台网络变化监听守护进程（监听 Wi-Fi/蜂窝切换并记录 `net.log`）。 |
| `clashapi init` | `8s` | `ge0.java:1047` | `Command::Clashapi`（**当前仓内为空实现 `Ok(())`**） | 当 `app_settings.clashapi_history_enabled == "1"` 时，在 `ge0.u()` 中先于 `monitor` 调用，预期拉起监听 `clashapi-history.sock` 的历史采集服务。 |
| `clashapi clear` | `8s` | `bc0.java:535` | `Command::Clashapi`（**当前仓内为空实现**） | 清理 Clash API 历史缓存数据库。 |
| *(仓库保留 CLI)* | — | — | `service {start,stop,restart,status,check}`、`iptables {enable,disable,renew}`、`cnip {update,enable,disable,status}`、`status`、`check`、`print-config`、`version` | `boxctl/src/cli.rs` 供内部与调试使用的子命令；返回码 `0` 表示成功，非 `0` 时错误输出至 `stderr`（APK 取 `stderr`，为空则取 `stdout`，并过滤 `R.array.home_service_failure_ignored_prefixes`）。 |

### 3.4 开机自启脚本生成协议（`ge0.java:450-538, 803-969`）

当用户在设置页切换"开机自启"（`ge0.r(enabled)`）：
1. 先将 `runtime_profile.boot_auto_start` 更新为 `1`/`0`。
2. **开启（`ge0.f`）**：校验 `boxctl` 可用后，以 root 原子写入 `/data/adb/service.d/boxproxy-honk.sh`（先写 `.boxproxy-honk.sh.tmp` -> `chmod 755` -> `chown root:root` -> `mv` -> `test -x`，超时 `10s`）；若写盘失败则回滚数据库字段。脚本精确结构如下：
   - 导出 `PATH="/data/adb/magisk:/data/adb/ksu/bin:/data/adb/ap/bin:/system/bin:/system/xbin:$PATH"` 与 `BOXCTL_STDOUT_LOG_LINE=1`。
   - 循环等待 `getprop init.svc.bootanim == "stopped"`（每 `10s` 轮询）。
   - 循环等待 `[ -x "$boxctl" ] && [ -f "$db" ]`（每 `2s` 轮询）。
   - 清空并写入 `files/box/run/boot.log`，执行 `"$boxctl" --db "$db" boot >> "$log_file" 2>&1`，后台子 shell 异步返回 `exit 0`。
3. **关闭（`ge0.o`）**：执行 `rm -f '/data/adb/service.d/boxproxy-honk.sh'`（超时 `5s`），失败则回滚数据库字段。

### 3.5 核心二进制校验、更新、版本识别与多核心运行协议

#### 3.5.1 9 个内置核心更新源与包校验（`ki1.java`、`hi1.java`、`de1.java`、`qi1.java`）

| 源 ID | 安装目标文件名 | 上游 GitHub 仓库 | 通道 / 资产正则 | 包格式 (`ce1`) | 压缩内二进制名 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `mihomo` | `mihomo` | `MetaCubeX/mihomo` | `mihomo_stable=true`: latest (`mihomo-android-arm64-v8-<tag>\.gz`); `false`: tag `Prerelease-Alpha` (`mihomo-android-arm64-v8-alpha-[A-Za-z0-9._-]+\.gz`) | `Gzip` | `mihomo` |
| `mihomo_smart` | `mihomo` | `vernesong/mihomo` | tag `Prerelease-Alpha` (`mihomo-android-arm64-v8-alpha-smart-[A-Fa-f0-9]+\.gz`) | `Gzip` | `mihomo` |
| `meow` | `meow` | `meow-rs/meow-rs` | latest (`meow-<tag>-aarch64-unknown-linux-musl\.tar\.gz`) | `TarGz` | `meow` |
| `sing-box` | `sing-box` | `SagerNet/sing-box` | `sing_box_stable` 控稳定/预发布 (`sing-box-<ver>-android-arm64\.tar\.gz`) | `TarGz` | `sing-box` |
| `sing-box_ref1nd` | `sing-box` | `reF1nd/sing-box-releases` | `sing_box_stable` 控稳定/预发布 (`sing-box-<ver>-android-arm64\.tar\.gz`) | `TarGz` | `sing-box` |
| `xray` | `xray` | `XTLS/Xray-core` | 含预发布 (`Xray-android-arm64-v8a\.zip`) | `Zip` | `xray` |
| `v2fly` | `v2fly` | `v2fly/v2ray-core` | 含预发布 (`v2ray-android-arm64-v8a\.zip`) | `Zip` | `v2ray` |
| `hysteria` | `hysteria` | `apernet/hysteria` | 含预发布 (`hysteria-android-arm64$`) | `Direct` | `hysteria` |
| `honk` | `honk` | `Wcoom/honk-android` | latest (`^honk-android-arm64$`) | `Direct` | `honk` |

- **下载与镜像**：支持自定义加速下载前缀（`accelerated_download_prefix`）与 GitHub PAT（`github_token`）。
- **解包与安全校验**：压缩包/二进制大小上限 **192 MiB**；若 Release 附带 checksum 则校验 SHA-256（不匹配报错 `"SHA-256 does not match the release checksum"`，`n9.java:1156`）；通过 `de1.b(file)` 解析 ELF 头，强制要求 **64-bit (`EI_CLASS=2`) + 小端 (`EI_DATA=1`) + `e_machine == 183` (`EM_AARCH64`)**（`hi1.java:90-99`）。
- **原子安装（`hi1.java:102-155`）**：root 执行 `mkdir -p '<bin>' && cp -f '<src>' '<bin>/.<core>.new' && chown root:net_admin '<bin>/.<core>.new' 2>/dev/null || true && chmod 0755 '<bin>/.<core>.new' && mv -f '<bin>/.<core>.new' '<bin>/<core>'`（超时 `30s`）。若该核心当前正在运行则自动调用 `boxctl restart`（`hi1.h`，超时 `120s`）。
- **本地版本探测与对比**：
  - `qi1.a` / `mx0.java:2848` 依次尝试 `"$core_path" --version` -> `-v` -> `version` -> `-version`（按核心名优先序，如 `xray`/`v2fly` 用 `version`，`mihomo`/`meow`/`honk` 用 `-v`/`--version`），剥离 ANSI 转义序列并取 `version:` 行或首非空行。
  - 版本比较器 `li1` 支持 4 种模式：`VERSION`（SemVer + prerelease 比较）、`NUMBER`（大整数）、`BUILD`（精确 build-id/hash 相等匹配）、`DISPLAY`（仅展示不自动比对）。

#### 3.5.2 7 大核心配置检查、运行参数与配置覆写流水线（交叉比对 `boxctl/src/service/command.rs` 与 `boxctl/src/core_config/`）

| 核心 | 配置文件后缀 (`ar5.java`) | 校验命令 (`boxctl`) | 运行命令 (`boxctl`) | 特殊环境变量 / 行为 |
| :--- | :--- | :--- | :--- | :--- |
| **`honk`** | `.dae` | `-t -d <honk_dir> -f <source-config>` | `-d <honk_dir>/data -c <source-config> --store db` | `HONK_RUN_DIR=<run>`、`HONK_RECORD_ALWAYS=1`、`SSL_CERT_DIR=/apex/com.android.conscrypt/cacerts`；启动前复制源码配置至 `startup-config`；启动存活探测延迟 `1500ms`（其余核心 `300ms`）；当 `cgroup_memcg=0` 时挂入无内存限制的 `/sys/fs/cgroup/honk`。 |
| **`mihomo`** | `.yaml`, `.yml` | `-t -d <mihomo_dir> -f <startup-config>` | `-d <mihomo_dir> -f <startup-config>` | `boxctl/src/core_config/mihomo.rs` 支持覆写端口/TUN/DNS/FakeIP/外部控制器/Secret，合并 `mihomo-proxy-providers.yaml` 及 `mihomo-scripts.yaml` JS 脚本清单。 |
| **`meow`** | `.yaml`, `.yml` | 同 Mihomo 兼容格式（APK 已支持；`boxctl` 仓待补齐） | 同 Mihomo 兼容格式 | APK 支持 Clash API 发现（`cr0.java:272`）与 `meow/scripts/*.js` + `meow-scripts.yaml`（`e64.java`）。 |
| **`sing-box`** | `.json`, `.jsonc`, `.yaml`, `.yml` | `check -c <startup-config> -D <sing-box_dir>` | `run -c <startup-config> -D <sing-box_dir>` | `boxctl/src/core_config/sing_box.rs` 覆写 `experimental.clash_api`、inbounds（TUN/TProxy/Redirect）、DNS；支持 `sing-box-scripts.yaml` 与 `sing-box-official.json` / `sing-box-ref1nd.json` JSON Schema 校验。 |
| **`xray`** | `.json` | `-test -confdir <xray_dir>` | `run -confdir <xray_dir>` | `XRAY_LOCATION_ASSET=<xray_dir>`；不支持 `startup-config` 自动覆写（`bc0.java:605`）。 |
| **`v2fly`** | `.json` | `test -d <v2fly_dir>` | `run -d <v2fly_dir>` | `V2RAY_LOCATION_ASSET=<v2fly_dir>`；不支持 `startup-config` 自动覆写。 |
| **`hysteria`** | `.yaml`, `.yml` | 无内置 check | `-c <source-config>` | 直接使用用户选定配置文件启动。 |

所有核心进程在 `boxctl` 中统一以 **`root:net_admin`** 身份启动，并按 `runtime_profile.taskset_cpu`（过滤当前在线且允许 CPU 掩码）通过 `taskset <hexmask>` 绑定核心，以及按 `cgroup_{memcg,cpuset,blkio}` 写入对应 cgroup。

#### 3.5.3 进程状态与性能采样（`y6.java`、`oe0.java`、`mx0.java`）
- 读取 `files/box/run/box.pid` 与 `box.identity`，经 `/proc/<pid>/cmdline`、`/proc/<pid>/exe`、`/proc/<pid>/stat` 核验进程存活与启动时间戳，避免 PID 复用误判。
- 内存读取 `/proc/<pid>/smaps_rollup` 的 `Pss`（回退 `/proc/<pid>/status` `VmRSS`），换算 MB；CPU 占用由相邻两次采样 `/proc/<pid>/stat` `utime+stime` 差值结合运行核心数计算；当前所在 CPU 核读取 `awk '{print $39}' /proc/<pid>/stat`（回退 `ps -o psr= -p <pid>`，`mx0.java:2848-2852`）。

#### 3.5.4 Shell 生命周期钩子环境变量契约（`wr6.java:8-45`）
执行 `shell_scripts_pre_start`、`shell_scripts_post_stop` 或手动运行 `files/box/scripts/*.sh` 时，向子进程注入 12 个标准环境变量：
`BOX_EVENT`、`BOX_HOME`、`BOX_CORE`、`BOX_CORE_PATH`、`BOX_CONFIG`、`BOX_CONFIG_PATH`、`BOX_STARTUP_CONFIG_PATH`、`BOX_MODE`、`BOX_PROXY_MODE`、`BOX_SCRIPTS_DIR`、`BOX_RUN_DIR`、`BOX_STATE_DIR`。

---

## Section 4：完整 58 模块功能清单与 API 协议矩阵

### 4.1 导航架构与全部 32 条路由（`BoxAppRoute` / `kb0`）

主界面入口 `lx.a`（`sources/p000/lx.java:927`、`m90.java`、`ck4.java`）承载根路由 `Main`（`la0`）及 31 条子页面路由（共 32 个 `kb0` 实现类）：
- **底部栏 / 侧边导航栏主标签页**（受 `show_proxy_selector_bottom_bar_item` 与 `home_panel_entry_mode` 控制）：
  1. **首页（Home）**
  2. **面板 / 策略（Panel / Proxy Selector）**：可按 `home_panel_entry_mode`（`iy2`：`overview` / `proxy_selector` / `connections` / `subscription` / `rules` / `rule_providers` / `web_ui`）作为独立底栏入口展示（`show_proxy_selector_bottom_bar_item = true` 时显示）
  3. **工具（Tools）**
  4. **设置（Settings）**
- **启动直达逻辑**：若 `welcome_onboarding_shown == false`，先渲染首次引导（Onboarding）；若 `open_panel_on_launch == true`（且无特定磁贴/通知目标覆盖），启动后直接切至 `home_panel_entry_mode` 对应面板页。

### 4.2 协议与接口总表（Clash API / WebSocket / Unix Socket / Honk Native API / WebView Bridge）

#### 4.2.1 Clash API HTTP 与 WebSocket 端点（APK 实际调用）

API 发现顺序（`cr0.java:245-455`、`xe0.java:218-277`）：
1. 优先检查 `proxy_selector_preferences.custom_api_enabled`（自定义 endpoint + secret）。
2. 否则按 `runtime_profile.core` 读取 `files/box/run/state/config/startup-config`：
   - `mihomo` / `honk` / `meow`：root `grep -m1 -E '^[[:space:]]*external[-_]controller:'` 与 `secret:`，绑定 `127.0.0.1:<port>`。
   - `sing-box`：root `cat startup-config`，JSON 解析 `experimental.clash_api.external_controller` 与 `secret`。
   - `xray` / `v2fly` / `hysteria`：返回 `unsupported`。
3. 认证头：`Authorization: Bearer <secret>`；WebSocket 额外附加查询参数 `?token=<secret>`（`y3.java:357`）。

| 协议 / 方法 | 路径与参数 | 调用类与行号 | 用途与超时 |
| :--- | :--- | :--- | :--- |
| `GET` | `/configs` | `sn.java:381`、`zi7.java:514` | 获取运行态配置、`mode` 与 `mode-list`/`mode_list`/`modeList`（超时 5s/7s） |
| `PATCH` | `/configs`（JSON `{"mode":"rule"\|"global"\|"direct"}`） | `n9.java:1402-1418` | 切换出站模式（Rule / Global / Direct） |
| `PUT` | `/configs?force=true`（JSON `{}`） | `jy.java:1075-1085` | 热重载当前配置（首页/常驻通知"重载"按钮） |
| `GET` | `/proxies` | `sn.java:359,411`、`ex0.java` | 获取全部策略组与代理节点拓扑、历史延迟 |
| `PUT` | `/proxies/<group>`（JSON `{"name":"<node>"}`） | `zi7.java`、`cr0.j` | 在指定 Selector 策略组中切换节点 |
| `GET` | `/proxies/<node>/delay?timeout=<ms>&url=<url>` | `wk5.java:122-125`、`zi7.java:656,714` | 单节点延迟测速；开启 `detect_ipv6_on_latency_test` 时额外向 `https://ipv6.google.com/generate_204`（`timeout=2000`）探测 IPv6 能力 |
| `GET` | `/group/<group>/delay?timeout=3000&url=<url>` | `wk5.java:36` | 策略组并发延迟测试 |
| `GET` | `/providers/proxies` | `sn.java:337,396`、`eq2.java:581` | 获取全部代理集（含 `subscriptionInfo` 流量配额与更新时间，超时 20s/30s） |
| `GET` | `/providers/proxies/<name>` | `eq2.java:448` | 获取单个代理集详情 |
| `PUT` | `/providers/proxies/<name>` | `zi7.java`、`cr0.j` | 触发代理集远端更新（超时 45s/60s） |
| `GET` | `/providers/proxies/<name>/healthcheck` | `eq2.java:709` | 代理集全节点健康检查（超时 45s/60s） |
| `GET` | `/rules` | `sn.java:441` | 获取分流规则列表 |
| `GET` | `/providers/rules` | `sn.java:426` | 获取规则集列表 |
| `PUT` | `/providers/rules/<name>` | `k46.java:168` | 触发单条/批量（并发 4）规则集更新 |
| `GET` | `/connections` | `bc0.java:789`、`xe0.java` | 常驻通知周期拉取 `downloadTotal`/`uploadTotal`（或累加活跃连接字节数）计算速率 |
| `WS` | `/connections?token=<secret>` | `by2.java:55`、`y3.java:367` | 首页网速曲线（60 点滑动窗口）、实时连接列表、单连接速率与出站统计（仅前台且服务运行时连接） |
| `DELETE` | `/connections` / `/connections/<id>` | `qb2.java:267`、`n9.java:1578-1580` | 断开全部连接或按 ID 断开指定连接（`disconnect_on_select=true` 切换节点时也触发） |

#### 4.2.2 Unix Domain Socket 协议：`clashapi-history.sock`（`nm0.java`、`rs0.java`、`ps0.java`、`os0.java`、`sr0.java`）

- **套接字路径**：`files/box/run/state/runtime/clashapi-history.sock`（`LocalSocketAddress.Namespace.FILESYSTEM`）。
- **请求帧**（连接后发送单行 JSON + `\n`，`os0.java:21`）：
  ```json
  {"dimension":"outbound|domain|destination_ip|rule|process","window":"30m|1h|6h|24h|7d|30d","limit":10,"sort":"connections|download|upload|name","descending":true}
  ```
- **响应流**：NDJSON 逐行推送快照（聚合连接摘要 `dr0`、时间范围流量 `kr0` 与热门排行列表），断线后指数退避重连（`500ms << min(retry, 5)`，上限 `10s`，`rs0.java:76`）。

#### 4.2.3 Honk 原生 API 端点目录（`src/crates/honk-core/src/native_api/handlers.rs`、`core_update.rs`、`ui.rs`）

> **重要现状区分**：当前反编译的 APK v214 对 `honk` 核心**仅在两处使用了原生端口**：① 本地 WebUI 默认直连 **`http://127.0.0.1:9527/ui/`**（`n9.java:726`，即 Honk 内置 Doona WebUI）；② 核心更新源指向 `Wcoom/honk-android`。其余页面（策略/连接/规则/速率）仍走 Clash 兼容 API。Wave 3 纯 Rust 客户端可直接对接以下全部 35 个 Honk Native API 端点：

| 分组 | 端点与方法 | 功能说明 |
| :--- | :--- | :--- |
| 发现与认证 | `GET /api`、`POST /api/v1/auth/{setup,login,logout}`、`GET /api/v1/version`、`GET /api/v1/capabilities` | 能力自发现、密码/Token 双模认证、版本与 revision 查询 |
| 运行态与遥测 | `GET /api/v1/runtime`、`GET /api/v1/runtime/memory`、`GET /api/v1/runtime/outbounds`、`GET /api/v1/runtime/traffic/history`、`GET /api/v1/runtime/memory/history`、`GET/PATCH /api/v1/runtime/settings`、`GET /api/v1/datapath` | 生命周期/代际状态、CPU%、内存与流量历史（直接替代缺失的 `clashapi-history.sock`）、eBPF 数据面观测、降级告警 |
| 节点、策略组与探测 | `GET/POST /api/v1/nodes`、`GET/DELETE /api/v1/nodes/{id}`、`GET /api/v1/groups`、`GET /api/v1/groups/{groupId}`、`GET/PATCH /api/v1/groups/{groupId}/config`、`PUT/DELETE /api/v1/groups/{groupId}/selection`、`POST /api/v1/probes` | 节点/策略组查询与增删、节点选择与覆盖清除、延迟探测 |
| 订阅与 GeoData | `GET/POST /api/v1/providers`、`GET/DELETE /api/v1/providers/{id}`、`POST /api/v1/providers/{id}/refresh`、`GET /api/v1/geodata`、`POST /api/v1/geodata/update` | 订阅源管理与异步刷新、GeoIP/GeoSite 状态与更新 |
| 连接、流与路由 | `GET/DELETE /api/v1/connections`、`DELETE /api/v1/connections/{connection_id}`、`GET /api/v1/flows`、`GET /api/v1/flows/{flow_id}`、`GET /api/v1/rules`、`POST /api/v1/routing/trace` | 活跃连接管理、内核/用户态 Flow 轨迹取证、规则字典、服务端路由追踪 |
| DNS | `POST /api/v1/dns/query`、`GET /api/v1/dns/rules`、`GET /api/v1/dns/log`、`GET/DELETE /api/v1/dns/cache`、`DELETE /api/v1/dns/cache/{entry_id}`、`POST /api/v1/dns/cache/flush` | DNS 实时探测、DNS 规则与日志、缓存查询与清理 |
| 配置与热重载 | `GET /api/v1/config`、`POST /api/v1/config/validate`、`POST /api/v1/config/sources`、`GET/PUT /api/v1/config/sources/{source_id}`、`POST /api/v1/operations/reload`、`GET /api/v1/operations/{id}` | 配置校验、分片配置源编辑、原子热重载与异步 Operation 状态追踪 |
| 实时事件与日志 | `GET /api/v1/events`（SSE/流式事件）、`GET /api/v1/logs` | 实时状态变更广播（`runtime.updated` 等）与日志流 |
| Honk 扩展（`x-honk`） | `GET /api/v1/x-honk/config/export`、`POST /api/v1/x-honk/config/import`、`GET /api/v1/x-honk/config/revisions`、`POST /api/v1/x-honk/config/revisions/{number}/activate`、**`GET/POST /api/v1/x-honk/core/update`** | 配置导入导出与历史修订回滚；核心自检（`?refresh=1`）与带 ELF64 AArch64 + `SHA256SUMS.txt` 校验的原子自更新（`{force: bool}`） |
| 内置 UI 托管 | `GET /`、`GET /ui`、`GET /ui/*` | 托管 Honk 内置 Doona 完整版/Lite WebUI（`ui_fallback`） |

#### 4.2.4 WebView 双向桥接协议（`ZashboardKsuBridge.java`、`t50.java`、`pm0.java`、`t08.java`、`id7.java`）

1. **`window.ksu`（`ZashboardKsuBridge` via `addJavascriptInterface(..., "ksu")`）**：
   - `fullScreen(boolean)`：空操作兼容桩（防止 Zashboard 报错）。
   - `enableInsets(boolean)`：空操作兼容桩。
   - `moduleInfo()`：返回常量 `"{}"`。
   - *(安全设计：有意不暴露 KernelSU 的 `exec`/`spawn` 任意命令执行接口！)*
2. **`window.BoxAndroid`（AndroidX `WebViewCompat.addWebMessageListener`）**：
   - 页面加载后由 `pm0.java:234` 注入检测脚本，识别 `#app` 且 `document.title === 'zashboard'` 或 `meta[name=description] === 'a dashboard using clash api'`，通过 `window.BoxAndroid.postMessage(JSON.stringify({type: 'zashboard-detected', pageUrl, token}))` 回传（带随机一次性 `token` 防伪造，`t08.java:106`），命中后将规范化 URL 写入 `zashboard_monet_panels.urls` 并触发动态取色。
   - 导出文件保存：`window.BoxAndroid.postMessage(JSON.stringify({type: 'save-file', base64, fileName, mimeType}))`（`t08.java:87-105`），由宿主解码 Base64 并弹出系统 SAF 保存文件对话框。
3. **动态 Monet CSS 拦截（`id7.java:181-195`）**：
   - 在 `shouldInterceptRequest` 中拦截 `https://mui.kernelsu.org/internal/colors.css`（大小写不敏感），当当前面板属于 Zashboard 白名单时，返回 `200 OK` + `Cache-Control: no-store` 的动态生成 Material You CSS 变量表（由当前系统/应用配色方案实时生成）。

---

### 4.3 完整 58 模块功能清单（按 8 大功能域分组）

#### 域 A：首页（Home，共 8 个模块 `M01`–`M08`）

| 编号 | 模块名称 | 路由 / 源码证据 / `R.string.*` | 界面交互、调用接口与持久化状态 |
| :-: | :--- | :--- | :--- |
| **M01** | 服务状态主控卡片 | `Main` (`la0`), `hz2.java`, `fz2.java`, `jy.java`, `R.string.home_service_*`, `R.string.home_action_*` | 展示运行/停止/过渡/错误状态、核心名、网络模式、配置文件名、运行时长、CPU%、内存 PSS/RSS；提供启动（`boxctl up`）、停止（`boxctl down`）、重启（`boxctl restart`）、热重载（`PUT /configs?force=true`）按钮；持久化：`runtime_profile`。 |
| **M02** | 运行系统详情弹窗 | `mx0.java:2830-2919`, `R.string.home_system_detail_*` | 点击状态卡片弹出：展示 PID、核心实测版本（`--version`/`-v`/`version`）、配置路径、当前运行 CPU 核心编号（`/proc/<pid>/stat` 第 39 列）。 |
| **M03** | 实时网速与流量卡片 | `by2.java`, `R.string.home_net_speed_*` | 支持双源切换（`use_net_speed_clash_api`）：① Clash API `WS /connections`（实时上/下行速率、累计总流量、活跃连接数、60 点实时折线图）；② 系统 `TrafficStats` / `/proc/net/dev` 速率统计。点击进入 `ConnectionsDetails`。 |
| **M04** | 出站模式快速切换器 | `zi7.java:492-568`, `n9.java:1402`, `R.string.proxy_selector_mode_*` | 读取 `GET /configs`（`mode` 与 `mode-list`），支持一键切换 `Rule` / `Global` / `Direct`（`PATCH /configs`）。 |
| **M05** | 三目标延迟测速卡片 | `av2.java`, `cc0.java`, `wh3.java`, `R.string.home_latency_*` | 并发 HTTPS 握手/请求探测 3 个自定义目标（默认 Baidu/Google/Cloudflare），点击刷新或进入 `LatencyTargets` 编辑；持久化：`latency_settings`。 |
| **M06** | LAN / WAN IP 与归属地卡片 | `py2.java`, `ty2.java`, `hw.java`, `zy2.java`, `R.string.home_ip_*` | 一键切换 `wan` / `lan` 视图（持久化 `home_ui_preferences.home_ip_mode`）；WAN 模式请求 `https://api-ipv4.ip.sb/geoip`、`https://api-ipv6.ip.sb/geoip` 与 `https://ipwho.is/` 展示公网 IPv4/IPv6、国旗、地区与 ASN；LAN 模式列举本机网卡内网 IP。 |
| **M07** | 订阅流量汇总卡片 | `h13.java`, `ao.java`, `eq2.java`, `R.string.subscription_detail_*` | 支持双源（`use_subscription_detail_clash_api`）：Clash API `GET /providers/proxies` 或本地配置文件解析 + HTTP `Subscription-Userinfo` 响应头抓取，展示已用/总量/剩余百分比与到期日；缓存：`home_subscription_cache`；点击进入 `SubscriptionDetails`/`SubscriptionLocalDetails`。 |
| **M08** | 首页快捷面板入口与更新横幅 | `of1.java`, `z21.java`, `iy2.java`, `R.string.home_panel_*`, `R.string.core_update_*` | 启动延迟 1.5s 静默检查当前核心远端更新（结果缓存于 `core_update_monitor`），有新版本时展示提示条（支持"更新"或"忽略此版本"）；提供快速进入 `home_panel_entry_mode` 对应页面的入口卡片。 |

#### 域 B：面板（Panel / WebUI & API 联动页，共 8 个模块 `M09`–`M16`）

| 编号 | 模块名称 | 路由 / 源码证据 / `R.string.*` | 界面交互、调用接口与持久化状态 |
| :-: | :--- | :--- | :--- |
| **M09** | 概览页（Overview） | `iy2.overview`, `sr0.java`, `rs0.java`, `t35.java`, `ck3.java`, `R.string.panel_overview`, `R.string.clashapi_*` | 聚合展示实时速率/流量、实时连接维度排行（`outbound`/`domain`/`destination_ip`/`rule`/`process`，条数 `1..100`、排序 `connections`/`download`/`upload`/`name`）及基于 `clashapi-history.sock` 的历史时间窗口（`30m` 等）范围流量与热门数据排行（含清空历史 `boxctl clashapi clear`）；持久化：`overview_preferences` + `app_settings.clashapi_history_*`。 |
| **M10** | 实时连接管理页（Connections） | `ConnectionsDetails` (`ga0`), `by2.java`, `lx.java`, `qb2.java`, `R.string.connection_*` | 消费 `WS /connections`，支持按 9 种方式排序（`Host`/`Rule`/`Type`/`ConnectedAt`/`Download`/`Upload`/`DownloadSpeed`/`UploadSpeed`/`SourceIp` + 升降序）、按来源 IP 筛选、自定义正则/字段过滤规则组（7 字段：`Domain`/`Destination`/`Source`/`Process`/`Rule`/`Chain`/`Inbound`）、暂停刷新、单连接详情弹窗（概览 + 格式化原始 JSON 元数据）、一键关闭单连接/全部连接（`DELETE /connections[/{id}]`）；持久化：`connection_preferences`。 |
| **M11** | 分流规则与路由追踪页（Rules） | `Rules` (`xa0`), `k46.java`, `h94.java:14-71`, `R.string.rules_*` | 调用 `GET /rules` 展示全量分流规则，支持搜索、按类型/策略筛选，并提供路由追踪测试（输入域名/IP/端口模拟命中规则与分步骤耗时/跳过/复用轨迹展示）。 |
| **M12** | 规则集管理页（Rule Providers） | `iy2.rule_providers`, `k46.java:148-233`, `sn.java:426`, `R.string.settings_home_panel_entry_rule_providers` | 调用 `GET /providers/rules` 展示规则集名称、行为、格式、规则数与更新时间；支持单条更新（`PUT /providers/rules/<name>`）及 4 并发全量更新。 |
| **M13** | 订阅详情页（API 模式） | `SubscriptionDetails` (`fb0`), `eq2.java`, `R.string.subscription_detail_*` | 调用 `GET /providers/proxies` 展示各远程订阅代理集节点数、上/下行已用流量、总配额、到期时间、更新时间，支持单个/全部触发 `PUT /providers/proxies/<name>` 更新与 `GET /providers/proxies/<name>/healthcheck`。 |
| **M14** | 订阅详情页（本地解析模式） | `SubscriptionLocalDetails` (`gb0`), `h13.java`, `qb2.java:318-341` | 在不依赖 Clash API 时解析配置内订阅 URL 并并发 HEAD/GET 获取 `Subscription-Userinfo`，结果缓存至 `home_subscription_cache.subscription_cache_local_json`。 |
| **M15** | 内置 WebUI 面板浏览器 | `WebUi` (`jb0`), `t50.java`, `fz7.java`, `n9.java:664-870`, `dy3.java:2920`, `R.string.web_ui_*` | 内置 4 个预设面板：① **本地面板**（`honk` -> **`http://127.0.0.1:9527/ui/`**；`mihomo`/`meow`/`sing-box` -> `http://127.0.0.1:<port>/ui/`，支持 `zz7` `auto`/`clash`/`sing_box` 模式）；② **Zashboard**（`http://board.zash.run.place`）；③ **MetaCubeXD**（`http://metacubex.github.io/metacubexd`）；④ **Sing-Box**（`http://sing-box-dashboard.sagernet.org`）。支持新增/删除自定义面板、刷新、清缓存、文件上传/下载、`ZashboardKsuBridge`、Zashboard 自动识别与动态 Monet CSS 注入；持久化：`web_panel_preferences` + `zashboard_monet_panels`。 |
| **M16** | API 配置与联动抽屉 | `vk.java`, `bk.java`, `R.string.api_panel_*` | 允许覆盖 Clash API 端点与 Secret（独立显隐切换保护隐私）、开启/配置 Clash API 历史采集（开关/保留天数/最大数据库 MiB）及配置策略页自定义测速 URL；持久化：`proxy_selector_preferences` + `api_config_reveal_preferences` + `app_settings`。 |

#### 域 C：策略（Proxy Selector，共 4 个模块 `M17`–`M20`）

| 编号 | 模块名称 | 路由 / 源码证据 / `R.string.*` | 界面交互、调用接口与持久化状态 |
| :-: | :--- | :--- | :--- |
| **M17** | 策略组与节点选择器 | `ProxySelector` (`wa0`), `zi7.java`, `wk5.java`, `sp5.java`, `R.string.proxy_selector_*` | 消费 `GET /proxies` 与 `GET /providers/proxies`；支持折叠/展开策略组（可配置手风琴模式 `collapse_previous_group_on_expand` 或底部弹窗展开 `expand_selected_policy_in_bottom_sheet`）、点击切换节点（`PUT /proxies/<group>`，可选 `disconnect_on_select` 自动断开旧连接）、按代理集分组（`group_by_provider`）、按出站模式显隐 Global 组（`display_global_by_mode`）、显示隐藏组/不可用节点开关。 |
| **M18** | 节点与组延迟测速及 IPv6 探测 | `wk5.java`, `zi7.java:629-739`, `R.string.proxy_selector_*` | 支持单节点测速（`GET /proxies/<name>/delay`）、整组测速（`GET /group/<group>/delay`）、代理集健康检查（`GET /providers/proxies/<name>/healthcheck`）；自定义测速 URL（默认 `https://connectivitycheck.platform.hicloud.com/generate_204`）；可选同时用 `https://ipv6.google.com/generate_204`（2000ms）标记节点 IPv6 支持状态。 |
| **M19** | 策略页布局、排序与排版定制 | `kq5.java`, `sp5.java` | 支持 5 种节点排序（`defaultsort`/`nameasc`/`namedesc`/`latencyasc`/`latencydesc`）、组与节点独立列数（`auto`/`fixed` 1–12 列）、密度（`standard`/`compact`）、长节点名溢出模式（`clip`/`scroll`/`wrap`）；持久化：`proxy_selector_preferences`。 |
| **M20** | 自定义策略组图标与图标包管理 | `CustomPolicyIcons` (`ia0`), `ml3.java`, `cd5.java`, `h94.java:619`, `R.string.proxy_selector_policy_icon_*` | 支持导入多个远程图标包 URL（`policy_icon_pack_urls`）及按策略组名覆写指定图标（`policy_icon_overrides`），含 URL 合法性校验；持久化：`proxy_selector_preferences`。 |

#### 域 D：工具（Tools，共 22 个模块 `M21`–`M42`）

| 编号 | 模块名称 | 路由 / 源码证据 / `R.string.*` | 界面交互、调用接口与持久化状态 |
| :-: | :--- | :--- | :--- |
| **M21** | 核心切换与主代理配置 | `ProxyConfig` (`va0`), `ar5.java`, `ge0.java`, `bc0.java`, `R.string.proxy_core_*`, `R.string.proxy_mode_*` | 切换 7 大核心（`mihomo`/`meow`/`sing-box`/`xray`/`v2fly`/`hysteria`/`honk`）、6 种透明代理模式（`tun`/`tproxy`/`ebpf`/`redirect`/`mixed`/`enhance`）、配置文件选择、IPv6 策略（`enable`/`bypass`/`disable`）、UDP 开关、阻断 QUIC、清理厂商防火墙（`clean_vendor_firewall`）、重启/同步应用；持久化：`runtime_profile`。 |
| **M22** | 核心配置自动覆写面板 | `ProxyConfig` / `u61.java`, `w61.java`, `bc0.java:551-634`, `R.string.config_override_*`, `R.string.startup_config_*` | 管理 30 项 `config_override.<core>.<option>` 覆写开关（DNS/TUN/TProxy/Redir/FakeIP/External Controller/Secret/Sniffer/Geo/Scripts 等）及一键"重新生成启动配置"（`boxctl config sync`）；持久化：`app_settings`。 |
| **M23** | 高级代理与资源控制配置 | `OtherProxyConfig` (`ua0`), `fr5.java`, `R.string.proxy_mode_*` | 自定义代理端口、DNS 端口、TUN 设备名、IPv4/IPv6 FakeIP 网段；Linux cgroup（`memcg` 内存上限、`cpuset` 绑定、`blkio` 权重）与 `taskset_cpu` 绑核，支持热执行 `boxctl resource apply`；持久化：`runtime_profile`。 |
| **M24** | 应用分流管理（黑白名单 + GID） | `AppManagement` (`ca0`), `lu.java`, `wc0.java`, `R.string.tools_app_management_title`, `R.string.app_management_*` | 扫描本机全量应用（含多用户/分身 UID），切换黑/白名单模式（`runtime_profile.proxy_mode`），勾选 UID（`app_selection`）与管理 GID 列表（`app_gid_list`），保存后热执行 `boxctl mode renew`（eBPF 下 `restart`）；视图筛选持久化：`tools_app_management`。 |
| **M25** | 智能分流选应用（Smart Select） | `AppManagement` 子功能, `R.string.app_management_smart_select*` | 一键按应用特征（国内常用/系统/无网络权限等规则）批量勾选分流应用，简化黑白名单配置。 |
| **M26** | CNIP 绕过与强制代理设置 | `CnipSettings` (`ea0`), `y3.java:370-484`, `fx0.java`, `qc0.java`, `R.string.tools_cnip_*`, `R.string.cnip_settings_*` | 开关 CNIP 绕过、选择模式（`ipset`/`ebpf`）、独立开关 IPv4/IPv6、选择在线源或本地文件导入（写至 `run/state/cnip/cn_ip{,v6}.txt`）、自定义补充 CIDR（带合法性校验）、强制走代理应用选择（`cnip_force_apps` -> 同步解析 UID 写入 `cnip_force_uids`）；持久化：`cnip_settings` + `cnip_force_*`。 |
| **M27** | 绕过规则与内网 CIDR 管理 | `BypassRules` (`da0`), `bc0.java:851-940`, `R.string.tools_bypass_rules_title`, `R.string.bypass_rules_*` | 开关绕过内网（`runtime_profile.bypass_intranet`），增删改及重置默认 IPv4（16 条）与 IPv6（12 条）内网绕过 CIDR，配置禁止代理网卡（`blocked_interfaces`）；保存后热执行 `boxctl mode renew`；持久化：`intranet_*` + `blocked_interfaces`。 |
| **M28** | 共享网络（热点代理与客户端 MAC 过滤） | `SharedNetworkConfig` (`cb0`), `wc0.java`, `R.string.tools_shared_network_config_title`, `R.string.shared_network_*` | 管理共享代理网卡（`shared_network_interfaces`），扫描已连热点客户端（ARP/邻接表），配置 MAC 黑/白名单过滤（`shared_network_settings` + `shared_network_macs`）；保存后热应用规则。 |
| **M29** | 网络匹配与布尔规则组引擎 | `NetworkMatch` (`pa0`), `tn4.java`, `lp4.java`, `tc0.java`, `bc0.java:996-1088`, `R.string.tools_network_*`, `R.string.wifi_match_*` | 全局启用网络控制与监听日志；配置优先级排序的规则组（`network_rule_groups`），以可视化布尔表达式树（`and`/`or`/`not` 组合 `wifi`/`wifi_ssid`/`wifi_bssid`/`cellular`/`mcc_mnc`/`sim1`/`sim2`）触发多步动作流（`network_rule_actions`：启停服务/切配置/切模式/跑脚本）；支持选择当前网络候选 SSID/BSSID/MCC-MNC；保存后执行 `boxctl network-match apply` + `ge0.u()`。 |
| **M30** | 订阅配置覆写管理（Mihomo / Meow） | `SubscriptionManagement` (`hb0`), `br7.java`, `R.string.subscription_*`, `R.string.tools_subscription_*` | 管理 `run/state/config/mihomo-proxy-providers.yaml`，支持 3 种来源类型（`http` 远程订阅、`file` 本地文件、`inline` 内嵌 YAML 节点），配置过滤/排除正则、健康检查与间隔、自定义请求头、节点参数覆写 YAML，支持"拉取到本地"缓存。 |
| **M31** | JS 覆写脚本管理（Mihomo / Meow / Sing-Box） | `MihomoScripts(coreName)` (`oa0`), `e64.java`, `R.string.mihomo_scripts_*` | 管理 `<core>/scripts/*.js`（上限 512 KiB，模板 `function main(config, profileName) { return config; }`），按序勾选启用写入 `run/state/config/<core>-scripts.yaml`，支持导入、新建、重命名、从原 URL 更新及跳转 Sora 编辑器编辑。 |
| **M32** | Shell 生命周期钩子脚本管理 | `ShellScripts` (`db0`), `wr6.java`, `jv1.java`, `R.string.tools_shell_scripts_title`, `R.string.shell_scripts_*` | 管理 `files/box/scripts/*.sh`（拒绝二进制导入），绑定 `pre_start` / `post_stop` 事件（存 `app_settings`），支持手动 Root 执行（注入 12 个 `BOX_*` 环境变量）并查看实时输出与上次退出码/时间戳（存 `app_settings` + `shell_scripts`）。 |
| **M33** | 运行时文件管理器 | `RuntimeFiles` (`bb0`), `lc6.java`, `y96.java`, `rl3.java`, `R.string.tools_runtime_files_title`, `R.string.runtime_files_*` | Root 浏览/管理 `files/box/` 下全部目录与配置（支持新建、重命名、删除、导入本地文件、自定义 User-Agent 从 URL 下载与一键重新拉取、导出分享、切换为当前配置文件）；下载源记录存 `runtime_files_download_cache` + `download_preferences`。 |
| **M34** | Sora 高亮代码编辑器与 Schema 校验 | `RuntimeFileEditor(relativePath)` (`ab0`), `dw.java`, `zs6.java`, `zi7.java:334-468`, `R.string.common_undo/redo` | 集成 `io.github.rosemoe.sora` 编辑器，加载 `assets/textmate/{darcula,light}.json` 主题与 YAML/JSON/JS/Shell TextMate 语法，支持自动换行（`word_wrap`）、吸顶滚动（`sticky_scroll`）、撤销/重做、全文查找替换、JSON5 校验与 Sing-Box 官方/reF1nd JSON Schema 在线更新及校验（`https://sing-box.sagernet.org/schema.json`、`https://raw.githubusercontent.com/reF1nd/sing-box/reF1nd-testing/docs/schema.json`）；偏好存 `runtime_file_editor_prefs` / `module_file_editor_prefs`。 |
| **M35** | 运行日志查看器 | `LogViewer` (`ka0`), `sn.java:260-309`, `zy7.java`, `R.string.tools_log_viewer_title`, `R.string.log_viewer_*` | 枚举 `files/box/run/*.log`（`runs.log`/`boot.log`/`net.log`/`<core>.log`），支持文件切换、关键字搜索、级别着色过滤、清空/导出/分享；偏好存 `tools_logs`。 |
| **M36** | 核心在线更新器 | Tools 内置弹窗 + `hm7.java`, `hi1.java`, `ki1.java`, `R.string.tools_update_*`, `R.string.core_update_*` | 选择 9 个内置源或自定义源，切换 Mihomo/Sing-Box 稳定/测试版通道（`core_update_settings`），检查远端版本与本地版本对比，走加速镜像/GitHub Token 下载，验证大小/SHA-256/ELF64 AArch64 并原子替换重启。 |
| **M37** | 核心自定义更新源管理 | `CoreUpdateSources` (`ha0`), `ki1.java`, `R.string.core_compare_*` | 增删改自定义核心更新源（GitHub Release / 直接 URL / 正则资产匹配 / 4 种版本比较模式 `VERSION`/`NUMBER`/`BUILD`/`DISPLAY`）；持久化：`tools_core_update.sources_v1`。 |
| **M38** | 本地文件导入安装核心 | Tools 入口 + `CoreImportActivity`, `de1.java`, `hi1.java`, `R.string.tools_import_core_title`, `R.string.core_import_*` | 通过 SAF 文件选择器或外部应用分享（`ACTION_VIEW`/`ACTION_SEND`）导入 `.gz`/`.tar.gz`/`.tgz`/`.tar`/`.zip`/原始 ELF 二进制，校验 AArch64 架构后原子安装到选定核心路径。 |
| **M39** | 本地 WebUI 面板更新器 | `hm7.java`, `g08.java`, `R.string.webui_update_*` | 下载并解压 WebUI 静态资源包（默认 `https://github.com/Zephyruso/zashboard/releases/latest/download/dist.zip`）至目标目录（默认 `files/box/mihomo/dashboard`），支持加速镜像；缓存存 `webui_update_cache`。 |
| **M40** | Sub-Store 本地模块集成 | `SubStore` (`eb0`), `z27.java`, `bc0.java:1089-1203`, `sn.java:500-520`, `R.string.substore_*` | 探测 `/data/adb/modules/sub_store` 是否安装，解析 `/data/local/sub_store/scripts/sub_store.env`（`SUB_STORE_BACKEND_API_PORT`、`SUB_STORE_BACKEND_MERGE`、`SUB_STORE_FRONTEND_PORT`、`SUB_STORE_FRONTEND_BACKEND_PATH`），拼接 `http://127.0.0.1:<port>?api=...` 在 WebView 中打开，并支持新增自定义远程 Sub-Store 面板（`web_panel_preferences.substore_*`）。 |
| **M41** | 网络诊断：29 平台连通性 + Net.Coffee IP 质量 | `NetworkTest` (`ra0`), `ol4.java`, `dc3.java`, `y3.java:759-787`, `R.string.network_test_*` | ① **29 平台并发连通性/解锁探测**（4 大类，配 29 个 `assets/network_platforms/*.svg`）：AI（ChatGPT/Claude/Grok/Doubao/Gemini）、社交（X/Reddit/Discord/TikTok/Douyin）、流媒体（Netflix/Disney+/YouTube/Spotify/Apple TV+/Hulu/Twitch/MissAV/E-Hentai/Bilibili）、通用（Apple/Cloudflare/Steam/GitHub/Wikipedia/PayPal/Tencent/Alibaba/NetEase），含区域/解锁精准探测 URL；② **IP 质量检测**：先请求 `https://ip.net.coffee/cdn-cgi/trace` 获取出口 IP，再解析数据中心/住宅/商宽/移动/教育/ISP 标签与风险 Verdict（`dc3.java`）。 |
| **M42** | 网络测速与测速历史 | `NetworkTest` (`ra0`) + `NetworkSpeedHistory` (`qa0`), `ao.java:492`, `cr4.java`, `mq4.java`, `R.string.network_test_speed_*` | 支持 3 个测速源（`cy6`：`cloudflare` [`speed.cloudflare.com/__down?bytes=1000|25000000`, `/__up`, `/cdn-cgi/trace`]、`apple` [`mensura.cdn-apple.com/api/v1/gm/config`]、`ookla` [`speedtest.net/api/js/servers?engine=js&limit=5`]），支持 Mbps/MB/s 单位切换（`speed_test_use_megabytes`），记录并管理历史测速条目（`network_speed_history.entries`）。 |

#### 域 E：设置（Settings，共 10 个模块 `M43`–`M52`）

| 编号 | 模块名称 | 路由 / 源码证据 / `R.string.*` | 界面交互、调用接口与持久化状态 |
| :-: | :--- | :--- | :--- |
| **M43** | 开机自启设置 | Settings 主页, `ge0.java:897-969`, `R.string.service_auto_start_title` | 切换 `runtime_profile.boot_auto_start` 并同步写/删 `/data/adb/service.d/boxproxy-honk.sh`，含失败回滚。 |
| **M44** | 应用语言切换 | Settings 主页, `os.java`, `wc0.java`, `resources/res/xml/locales_config.xml` | 切换 `system` / `zh-CN` / `zh-TW` / `en` / `ru`（Android 13+ `LocaleManager` / AppCompat），双写 `boxapp_settings.app_language` 与 `box.db` `app_settings.app_language`。 |
| **M45** | 加速下载镜像与 GitHub Token | Settings 主页, `ar.java`, `mt1.java`, `R.string.service_accelerated_*`, `R.string.settings_github_token_title` | 配置加速代理前缀（应用于 GitHub 下载与 API）与 GitHub PAT（提升 GitHub Release API 限额）；持久化：`boxapp_settings`。 |
| **M46** | 启动与导航入口设置 | Settings 主页, `ar.java`, `y6.java`, `R.string.settings_launch_*`, `R.string.settings_tile_*` | 配置"启动时打开面板"（`open_panel_on_launch`）、"默认面板页面"（`home_panel_entry_mode` 7 选 1）、"显示底栏面板入口"（`show_proxy_selector_bottom_bar_item`）、"长按磁贴行为"（`tile_long_click_target`：`panel`/`web_ui`/`app`，同步切换 4 个 `BoxTilePreferences*` alias）。 |
| **M47** | 主题、UI 风格与 OS4 玻璃导航定制 | `ThemeSettings` (`ib0`), `ss7.java`, `vi7.java`, `xi7.java`, `ri7.java`, `mn7.java`, `h60.java`, `R.string.theme_*`, `R.string.ui_mode_*`, `R.string.top_bar_*`, `R.string.bottom_bar_*` | 支持双设计系统切换（`UiMode`: `material` [Material 3] / `miuix` [HyperOS Miuix]）、深浅色模式（`System`/`Light`/`Dark` + `pure_black_dark_theme`）、Miuix Monet 开关、20 种预设强调色 + 自定义强调色、10 种动态调色板风格（`TonalSpot`/`Neutral`/`Vibrant`/`Expressive`/`Rainbow`/`FruitSalad`/`Monochrome`/`Fidelity`/`Content`/`Cmf`）、3 代色标规范（`Spec2021`/`Spec2025`/`Spec2026`）、模糊开关（`enable_blur`）、顶栏样式（`Classic`/`Glass`）、底栏样式（`Classic`/`Floating`/`LiquidGlass`/`Glass` + `floating_bottom_bar_blur` + `landscape_glass_navigation`）、预测性返回/边缘跟随/横滑返回开关及全局页面缩放（`page_scale`）；持久化：`boxapp_settings`。 |
| **M48** | 首页三目标延迟设置页 | `LatencyTargets` (`ja0`), `lx.java:535-576`, `wh3.java`, `R.string.home_latency_*` | 编辑 3 组延迟探测目标名称与 URL，实时同步至首页卡片；持久化：`latency_settings`。 |
| **M49** | 备份与恢复（ZIP 原子快照 + 回滚） | Settings 主页, `bo.java`, `xo.java`, `m00.java`, `R.string.settings_backup_*`, `R.string.settings_restore_*` | 导出/恢复格式为 `boxapp.backup.zip`（含 `manifest.json`、`database/box.db`、`files/...`，限制 20,000 条目、单文件 512 MiB、总计 2 GiB，防 Zip Slip 路径穿越）；支持 4 类可选勾选（`m00`：`app_settings` [含 23 个 SP + `box.db`]、`runtime_data`、`user_scripts`、`core_binaries`）；恢复前自动创建 `restore_<nano>` 回滚快照（`xo.a`），任何阶段失败自动原子回滚！ |
| **M50** | 应用图标切换与关于页 | `About` (`w90`), `y6.java`, `ns.java`, `R.string.about_*` | 展示应用版本、仓库链接（`Wcoom/honk-android`、`boxproxy/com.boxproxy.box` 与 7 大核心仓库），支持在默认图标与备用图标间切换（`MainLauncherDefault` <-> `MainLauncherAlternate`，同步通知/磁贴图标变体）；持久化：`app_icon_preferences`。 |
| **M51** | 开源引用库列表与详情 | `AboutLibraries` (`x90`) + `AboutLibraryDetail(libraryId)` (`aa0`), `qb0.java:34-53`, `R.string.about_libraries_title` | 解析 `res/raw/aboutlibraries.json` 展示全部第三方依赖许可证及详情。 |
| **M52** | 赞助支持页 | `AboutSponsorship` (`ba0`), `R.string.about_sponsorship_*`, `R.string.about_link_*` | 展示项目支持渠道与复制提示（遵循规范不在此记录具体个人地址）。 |

#### 域 F：首次引导（Onboarding，共 2 个模块 `M53`–`M54`）

| 编号 | 模块名称 | 路由 / 源码证据 / `R.string.*` | 界面交互、调用接口与持久化状态 |
| :-: | :--- | :--- | :--- |
| **M53** | 首次启动欢迎与权限/Root 检查引导 | `lx.java`, `ar.java`, `v16.i`, `R.string.welcome_*` | 首次启动（`boxapp_settings.welcome_onboarding_shown == false`）展示多步引导页：检测 Root 权限（`id -u`）、请求通知/定位/电话状态等运行时权限、选择默认 UI 风格（Material3/Miuix）与主题。 |
| **M54** | 初始核心与配置快速初始化向导 | `ge0.d`, `ge0.l`, `hi1.java` | 自动创建 `files/box/` 目录树，释放内置 `assets/box/mihomo/config.yaml` 与 `boxctl`/`boxbpf`，引导用户下载或导入首个核心（默认含 `honk` 等 7 核心可选）并标记 `welcome_onboarding_shown = true`。 |

#### 域 G：快捷磁贴与悬浮面板（PanelSheet & Tile，共 2 个模块 `M55`–`M56`）

| 编号 | 模块名称 | 路由 / 源码证据 / `R.string.*` | 界面交互、调用接口与持久化状态 |
| :-: | :--- | :--- | :--- |
| **M55** | 系统快捷设置磁贴（Quick Settings Tile） | `BoxServiceTileService.java`, `R.string.tile_service_*` | 下拉通知栏一键启停代理服务（`boxctl up`/`down`），实时反映运行/停止/过渡状态；长按按 `tile_long_click_target` 路由至 `BoxTilePreferences{Default,Alternate,WebUi,App}Activity`。 |
| **M56** | 半透明快捷悬浮面板（PanelSheet） | `PanelSheetActivity.java`, `h94.java:570-617`, `ck3.x` | 独立 `singleInstance`、不入最近任务（`excludeFromRecents=true`）、背景暗化 `0.18f` + Android 12+ 背景模糊（`windowBlurBehind`）；直接承载 7 种 API 联动面板并支持一键跳转主应用或点击外部无动画关闭。 |

#### 域 H：常驻通知（Persistent Notification，共 2 个模块 `M57`–`M58`）

| 编号 | 模块名称 | 路由 / 源码证据 / `R.string.*` | 界面交互、调用接口与持久化状态 |
| :-: | :--- | :--- | :--- |
| **M57** | 前台常驻状态通知服务 | `BoxPersistentNotificationService.java`, `hd0.java`, `od0.java`, `R.string.notification_channel_status_*` | Android 14+ `specialUse` 前台服务（通知 ID `1001`、渠道 `box_persistent_status`），按设定间隔轮询服务状态、`/proc/<pid>` CPU/内存、Clash API `/connections` 速率与累计流量，渲染 12 个占位符模板并响应 3 个快捷操作按钮（热重载/停止/重启/打开面板/打开 WebUI）。 |
| **M58** | 常驻通知定制设置页 | `NotificationSettings` (`sa0`), `ar.java`, `yt4.java`, `xt4.java`, `R.string.settings_notifications_title` | 开关常驻通知（申请 `POST_NOTIFICATIONS`）、配置刷新间隔（秒）、编辑标题与多行正文模板（带 12 个占位符快捷插入与实时预览）、自定义点击动作（`panel`/`web_ui`/`app`）及 3 个操作按钮的启用开关、自定义标签与动作绑定；持久化：`boxapp_settings`。 |

---

## Section 5：Wave 3 纯 Rust + 超薄 Android DEX 桥接架构蓝图

遵循项目既定 ADR（`rust/DECISIONS.md`：Rust-first、`boxproxy/boxctl` + `honk-core` 同仓协同、无多套真相源），Wave 3 目标是将除 Android OS 强绑定壳外的**全部业务逻辑、数据存储、Root 编排、网络探测、API 客户端与状态机 100% 收敛至单一 Rust 动态库 `libboxproxy_app.so`**，并同时统一 `boxctl` 的数据库 Schema（v3 -> v15），彻底消除现存架构撕裂。

### 5.1 Rust Workspace Crate 划分

建议在仓库 `rust/` 下建立独立 App 引擎 Workspace（与 `boxproxy/boxctl` 共享基础存储与类型 crate）：

```text
rust/
├── crates/
│   ├── boxproxy-schema/       # [共享唯一真源] box.db v15 SQLite DDL/迁移/强类型 Model（同时被 boxctl 与 App 复用！）
│   ├── boxproxy-store/        # rusqlite v15 事务仓储 + AppConfig（兼容导入旧 24 个 shared_prefs XML）+ ZIP 原子备份/回滚引擎
│   ├── boxproxy-root/         # 持久 Root Shell 池（复用 su 进程、命令上限/TTL 回收、POSIX 转义）+ boxctl/boxbpf SHA-256 资产同步 + /proc/<pid> 采样
│   ├── boxproxy-api/          # 统一核心控制面客户端：
│   │                          #   (a) Honk Native API (/api/v1/*, /api/v1/x-honk/*, SSE /events, /runtime/traffic/history)
│   │                          #   (b) Clash 兼容 API HTTP + WS (/connections, /proxies, /configs, /rules, /providers)
│   │                          #   (c) clashapi-history.sock Unix Domain Socket 客户端（用于兼容旧版 mihomo/sing-box 历史采集）
│   ├── boxproxy-core-mgr/     # 多核心适配器（7 核心配置覆写/JS 清单/Shell 钩子/Sub-Store 解析）+ ELF64 AArch64 校验 + SHA-256 核心/WebUI 更新器 + 开机脚本生成
│   ├── boxproxy-tools/        # 29 平台连通性/解锁探测 + Net.Coffee IP 质量 + IP Geo + 3 源测速 + 3 目标延迟 + 路由规则追踪 + Network Match JSON AST 引擎 + Sing-Box Schema 校验
│   ├── boxproxy-app-core/     # 单向数据流 MVI 状态机（AppState / AppAction / AppEffect）、58 模块 Reducer 与异步任务调度、i18n 模板与常驻通知文本渲染器
│   └── boxproxy-jni/          # cdylib (`libboxproxy_app.so`)：Tokio 运行时托管、JNI 导出函数、二进制帧（MessagePack / JSON）状态快照与事件流推送
└── android-shell/             # 超薄 Android DEX 壳（最小化 Kotlin/Java，零业务逻辑）
```

> **核心协同收益**：`boxproxy/boxctl` 改为直接依赖 `boxproxy-schema`，一次性解决 Section 2.1.3 发现的 v15 vs v3 表漂移问题；同时对 `honk` 核心优先走 `boxproxy-api` 的 **Honk Native API**（`/api/v1/runtime/traffic/history`、`/api/v1/routing/trace`、`/api/v1/x-honk/core/update`、`GET /api/v1/events`），无需依赖未实现的 `clashapi-history.sock`！

### 5.2 职责边界：超薄 DEX 壳 vs 100% Rust 引擎

| 边界划分 | 必须留在超薄 Android DEX 壳（不可由纯 Rust 替代的 Android Framework 绑定） | 100% 下沉至 Rust (`libboxproxy_app.so`) |
| :--- | :--- | :--- |
| **OS 入口组件** | `AppApplication`、`MainActivity`、`PanelSheetActivity`、`CoreImportActivity`、`BoxServiceTileService`、`BoxPersistentNotificationService`、`AndroidManifest.xml` 中全部 6 个 `activity-alias` | 所有生命周期决策（启动目标路由、磁贴状态机、通知内容与按钮渲染文本、轮询节拍控制） |
| **系统服务调用** | `PackageManager`（切换 6 个 alias、枚举已安装应用包名/UID/图标位图）、`NotificationManager`/`NotificationChannel`、`WifiManager`/`TelephonyManager`/`ConnectivityManager`（采集当前 SSID/BSSID/MCC-MNC/SIM 槽位/本机网卡 IP）、`TileService.requestListeningState` | 应用黑白名单与智能分流规则、CNIP `userId:pkg`->UID 映射、网络匹配 AST 求值、12 占位符通知文案渲染 |
| **平台 View 与 I/O** | `WebView` + `ZashboardKsuBridge` (`"ksu"`) + `BoxAndroid` `WebMessageListener` + `shouldInterceptRequest`（Monet CSS 拦截，CSS 内容由 Rust 生成）、SAF `ACTION_OPEN_DOCUMENT`/`CREATE_DOCUMENT`、系统剪贴板/Toast/Share、API 31+ `Window.setBackgroundBlurRadius`、可选 Sora `CodeEditor` View | 本地 WebUI/Sub-Store URL 解析、Monet CSS 样式表文本生成、Base64 `save-file` 解码、JSON5/YAML/Sing-Box Schema 语法诊断 |
| **数据、网络与 Root** | 仅向 Rust 传递 `filesDir` 路径、`AssetManager` 资产字节与 SAF `ParcelFileDescriptor` (`fd`) | 全部 `box.db` v15 读写与迁移、配置持久化、ZIP 原子备份/回滚、Root Shell 池、`boxctl`/`boxbpf` 同步、`/proc/<pid>` 采样、全部 HTTP/WS/SSE/Unix Socket 客户端、核心 ELF/SHA-256 校验与安装、29 平台/IP 质量/测速引擎 |

### 5.3 精确 JNI FFI 接口契约（`libboxproxy_app.so`）

由超薄 Kotlin 桥接对象 `com.box.app.bridge.RustBridge` 声明，保持极简、强边界、无细粒度跨 JNI 对象引用：

```java
package com.box.app.bridge;

public final class RustBridge {
    static { System.loadLibrary("boxproxy_app"); }

    // 1. 运行时初始化与生命周期（在 AppApplication.onCreate 调用一次）
    public static native long nativeInit(
        String filesDir,           // Context.getFilesDir().getAbsolutePath()
        String cacheDir,           // Context.getCacheDir().getAbsolutePath()
        String packageName,        // "com.boxproxy.honk"
        String appVersion,         // BuildConfig.VERSION_NAME
        int sdkInt,                // Build.VERSION.SDK_INT
        String initialLocaleTag,   // "zh-CN" | "en" | ...
        PlatformCallback callback  // 反向调用 DEX 壳系统能力 + 推送状态更新
    );
    public static native void nativeShutdown(long handle);

    // 2. 资产注入（由 DEX 壳读取 AssetManager 字节后喂给 Rust 做 SHA-256 同步）
    public static native void nativeRegisterAsset(long handle, String assetPath, byte[] bytes);

    // 3. 单向 MVI 命令派发（JSON/MessagePack 编码的 AppAction，返回可选同步结果或由回调异步推送）
    public static native byte[] nativeDispatch(long handle, byte[] actionBytes);

    // 4. 订阅界面状态切片（仅推当前可见页面的差量/快照，降低序列化开销）
    public static native long nativeSubscribeView(long handle, String viewId, StateListener listener);
    public static native void nativeUnsubscribeView(long handle, long subscriptionId);

    // 5. OS 入口专用同步/轻异步快捷接口（供 TileService / NotificationService / CoreImportActivity / WebView 拦截器调用）
    public static native byte[] nativeQueryTileState(long handle);
    public static native byte[] nativeRenderNotification(long handle, boolean forceRefresh);
    public static native String nativeResolveMonetCss(long handle, byte[] themePaletteJson);
    public static native byte[] nativeImportCoreFromFd(long handle, String targetCore, int fd, String displayName);
    public static native byte[] nativeExportBackupToFd(long handle, int fd, byte[] scopeMaskJson);
    public static native byte[] nativeRestoreBackupFromFd(long handle, int fd, byte[] scopeMaskJson);

    // 6. Rust -> DEX 壳最小反向平台回调接口（仅 7 个原语）
    public interface PlatformCallback {
        void onEffect(byte[] effectBytes);                  // Toast / 打开 SAF / 打开 PanelSheet / 刷新 Tile / 启停前台通知服务
        void setComponentAlias(String aliasName, boolean enabled);
        byte[] queryInstalledApps();                        // 返回包名/UID/标签列表 JSON（图标按需懒加载）
        byte[] queryNetworkSnapshot();                      // 当前 Wi-Fi SSID/BSSID、蜂窝 MCC/MNC、SIM 槽位、本机网卡列表
        boolean hasPermission(String permission);
        byte[] readAssetBytes(String assetPath);            // 按需拉取 boxctl/boxbpf/config.yaml/schema 等资产
        void openExternalUrl(String url);
    }

    public interface StateListener {
        void onStateSnapshot(long version, byte[] stateBytes);
    }
}
```

- **线程模型**：Rust 侧持有单一多线程 `tokio::runtime::Runtime`（`worker_threads = 2..4`），所有阻塞 Root Shell / SQLite / 压缩解包均跑在 `spawn_blocking`，**绝不阻塞 Android 主线程（Main Looper）**；`StateListener.onStateSnapshot` 附带递增 `version` 号，DEX 侧直接切换至主线程更新 UI。
- **文件描述符零拷贝**：SAF 备份/恢复与核心文件导入直接传入 `ParcelFileDescriptor.detachFd()` 整数 `fd`，Rust 通过 `std::os::fd::FromRawFd` 流式读写，避免大文件（最大 2 GiB 备份或 192 MiB 核心包）在 JVM 堆上 OOM。

---

### 5.4 Wave 1 检查点：UI 渲染方案开放技术决策（需拍板选型）

业务逻辑（约占原 APK 75% 复杂度）已 100% 明确下沉至 Rust；而**界面渲染层**在保真还原 **Material 3 + Miuix（HyperOS 风格）+ OS4 液态玻璃模糊（`LiquidGlass`/`Glass`）+ Sora TextMate 代码编辑器** 上存在 3 条截然不同的技术路线，需在 Wave 1 检查点确认 ADR：

| 评估维度 | 方案 A：超薄 Compose / DEX UI 渲染层 + 100% Rust 状态机（**推荐**） | 方案 B：纯 Rust GPU 渲染框架（`Slint` / `egui` / `iced` + `android-activity`） | 方案 C：WebView（Doona / Leptos WASM）为主 UI + 原生 Tile/Sheet 壳 |
| :--- | :--- | :--- | :--- |
| **Material 3 / Miuix / OS4 Glass 还原度** | **100% 原生保真**：可直接复用 Jetpack Compose + Miuix Compose + Android 12+ `RenderEffect`/Haze 液态玻璃模糊与系统预测性返回动画 | **低~中**：需从零手写全部 Material 3 动态取色、Miuix 弹簧物理、中文输入法 IME、系统无障碍与液态玻璃特效，工作量极大 | **中**：CSS `backdrop-filter` 可模拟玻璃态，但缺乏 Miuix 原生质感与系统级预测性返回/边缘手势 |
| **Sora TextMate 代码编辑器** | **零损耗复用**：直接嵌入 `sora-editor` Android View，100% 保留 TextMate 语法高亮、吸顶滚动、大文件虚拟化与 IME 组合输入 | **极难**：纯 Rust Canvas 编辑器在 Android 软键盘 IME（中文拼音候选、光标放大镜、选区手柄）兼容性极差，且需嵌入平台 `AndroidView` 叠层 | **中~高**：可用 CodeMirror 6 / Monaco（WebView 内），但冷启动慢、内存占用高（额外 +60~100 MB） |
| **APK 体积与内存占用** | `libboxproxy_app.so`（~4–6 MB）+ 超薄 Compose DEX（~5–8 MB），无 Retrofit/OkHttp/Gson/Room/SQLite Java 依赖，总体积显著小于原版 | 体积最小（~8–12 MB），但引入平台 `WebView` + `Sora` 叠层后混合窗口焦点管理极复杂 | 若内嵌 WASM + CodeMirror + WebUI，内存峰值偏高；与已有的 Doona WebUI 协同度高 |
| **Rust 化比例与可测试性** | **逻辑 100% Rust**（可直接在 Linux host 跑 `cargo test` 全覆盖 58 模块逻辑与状态机），DEX 壳仅为无状态视图绑定 | **逻辑+UI 100% Rust**，但 Android 端 IME/手势/焦点/多窗口缺陷风险极高 | **后端+前端可全 Rust（Leptos/Dioxus）**，但原生手感弱于方案 A |
| **推荐结论** | **Wave 3 首选主方案**：最稳健地同时满足"业务逻辑 100% 纯 Rust + 完整 58 模块体验零降级（含 Miuix/OS4 Glass/Sora/预测性返回）" | 不推荐用于全量 58 模块（适合极简工具 App，不适合含复杂表单、代码编辑器、多主题与 IME 重度交互的管理器） | 可作为 **Honk 专属轻量化 Web 控制台（Doona）** 持续演进，在方案 A 中通过 `WebUi` 标签页与 `PanelSheet` 无缝内嵌 |

#### 建议在 Wave 1 检查点同步确认的 3 个关键问题
1. **UI 渲染路线**：是否锁定 **方案 A（超薄 Compose DEX 渲染层 + 100% Rust MVI 引擎）**，或在 Wave 3 中裁剪掉 Miuix / OS4 玻璃风格以换取更极简的纯 Rust UI（方案 B/C）？
2. **`boxctl` Schema 统一策略**：是否将 `boxproxy/boxctl` 升级至与 APK 完全一致的 **`box.db` v15**（共享 `boxproxy-schema` crate，补齐 `network_rule_groups`、`shared_network_*`、`meow` 核心），还是让 `boxproxy-app` 直接接管 `clashapi-history` 与配置同步以瘦身 `boxctl`？
3. **Honk 原生 API 优先策略**：当活跃核心为 `honk` 时，Wave 3 的首页速率/概览历史/策略选择/规则追踪/核心更新是否直接优先切换至 **Honk Native API (`/api/v1/*`, `/api/v1/x-honk/core/update`)**，仅在切换到 `mihomo`/`meow`/`sing-box` 时回退 Clash 兼容 API？
