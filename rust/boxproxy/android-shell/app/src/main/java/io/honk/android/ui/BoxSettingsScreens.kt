package io.honk.android.ui

import android.content.Intent
import android.net.Uri
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Checkbox
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import org.json.JSONArray
import org.json.JSONObject

private val LANGUAGE_OPTIONS = listOf(
    "system" to "跟随系统",
    "zh-CN" to "简体中文",
    "zh-TW" to "繁體中文",
    "en" to "English",
    "ru" to "Русский",
)

private val TILE_TARGET_OPTIONS = listOf(
    "panel" to "面板",
    "web_ui" to "WebUI",
    "app" to "应用主页",
)

private val PANEL_ENTRY_OPTIONS = listOf(
    "overview" to "概览",
    "proxy_selector" to "策略",
    "connections" to "连接",
    "subscription" to "订阅",
    "rules" to "规则",
    "rule_providers" to "规则集",
    "web_ui" to "WebUI",
)

private val CORE_OPTIONS = listOf(
    "honk" to "Honk",
    "mihomo" to "Mihomo",
    "meow" to "Meow",
    "sing-box" to "Sing-Box",
    "xray" to "Xray",
    "v2fly" to "V2Fly",
    "hysteria" to "Hysteria",
)

private val RUN_MODE_OPTIONS = listOf(
    "ebpf" to "eBPF",
    "tun" to "TUN",
    "tproxy" to "TProxy",
    "redirect" to "Redirect",
    "mixed" to "Mixed",
    "enhance" to "Enhance",
)

private val IPV6_OPTIONS = listOf(
    "enable" to "启用 IPv6",
    "bypass" to "绕过 IPv6",
    "disable" to "禁用 IPv6",
)

private val DNS_HIJACK_OPTIONS = listOf(
    "tproxy" to "TPROXY",
    "redirect" to "REDIRECT",
    "disabled" to "禁用",
)

private val PALETTE_STYLE_OPTIONS = listOf(
    "TonalSpot" to "Tonal Spot",
    "Neutral" to "Neutral",
    "Vibrant" to "Vibrant",
    "Expressive" to "Expressive",
    "Rainbow" to "Rainbow",
    "FruitSalad" to "Fruit Salad",
    "Monochrome" to "Monochrome",
    "Fidelity" to "Fidelity",
    "Content" to "Content",
    "Cmf" to "CMF",
)

private val COLOR_SPEC_OPTIONS = listOf(
    "Spec2021" to "Material 3 2021",
    "Spec2025" to "Material 3 2025",
    "Spec2026" to "Material 3 2026",
)

private val TOP_BAR_OPTIONS = listOf(
    "Classic" to "经典",
    "Glass" to "玻璃",
)

private val BOTTOM_BAR_OPTIONS = listOf(
    "Classic" to "经典",
    "Floating" to "悬浮",
    "LiquidGlass" to "液态玻璃",
    "Glass" to "玻璃",
)

private val ACCENT_PRESETS: List<Pair<String, Int>> = listOf(
    "默认" to 0xFF455A8A.toInt(),
    "靛青" to 0xFF3F51B5.toInt(),
    "钴蓝" to 0xFF2563EB.toInt(),
    "天蓝" to 0xFF0284C7.toInt(),
    "青翠" to 0xFF0D9488.toInt(),
    "森绿" to 0xFF16A34A.toInt(),
    "琥珀" to 0xFFD97706.toInt(),
    "珊瑚" to 0xFFEA580C.toInt(),
    "绯红" to 0xFFDC2626.toInt(),
    "玫红" to 0xFFDB2777.toInt(),
    "紫罗兰" to 0xFF7C3AED.toInt(),
    "石板灰" to 0xFF475569.toInt(),
)

private val CONFIG_OVERRIDE_KEYS: List<Pair<String, String>> = listOf(
    "override_enabled" to "启用核心自动覆写总开关",
    "override_tproxy" to "覆写 TProxy 透明代理",
    "override_redir" to "覆写 Redirect 端口",
    "mixed_port" to "覆写 Mixed 混合端口",
    "override_tun" to "覆写 TUN 接口与路由表",
    "override_dns" to "覆写 DNS 监听与上游",
    "fake_ip_range" to "覆写 Fake-IP 网段",
    "override_external_controller" to "覆写 External Controller API",
    "override_secret" to "覆写 API Secret",
    "override_external_ui" to "覆写本地 WebUI 路径",
    "override_ipv6" to "覆写 IPv6 配置",
    "sniffer_enable" to "覆写域名嗅探 Sniffer",
    "geodata_mode" to "覆写 GeoIP/GeoSite 模式",
    "override_unified_delay" to "启用统一延迟测速 (Unified Delay)",
    "scripts_enabled" to "合并覆写脚本清单",
)

private fun updateAppSetting(appSettings: JSONObject, mutate: JSONObject.() -> Unit) {
    val updated = JSONObject(appSettings.toString()).apply(mutate)
    dispatchAsync("update_box_app_settings") { put("settings", updated) }
}

private fun saveProfile(profile: JSONObject, applyLive: Boolean = false, mutate: JSONObject.() -> Unit) {
    val updated = JSONObject(profile.toString()).apply(mutate)
    dispatchAsync("save_runtime_profile") {
        put("profile", updated)
        put("apply_live", applyLive)
    }
}

/**
 * 1:1 Replica of original `com.boxproxy.honk` Settings Tab (`orig_settings_top.png` + `orig_settings_bot.png`).
 */
@Composable
fun SettingsTabScreen(
    state: JSONObject,
    onLaunchBackupExport: (JSONObject) -> Unit,
    onLaunchBackupRestore: (JSONObject) -> Unit,
    onRequestNotificationPermission: () -> Unit,
) {
    val p = LocalBoxPalette.current
    val listState = rememberLazyListState()
    val collapsed by remember { derivedStateOf { listState.firstVisibleItemIndex > 0 || listState.firstVisibleItemScrollOffset > 60 } }

    val profile = state.optJSONObject("runtime_profile") ?: JSONObject()
    val bootEnabled = profile.optBoolean("boot_auto_start", false)
    val prefs = state.optJSONObject("prefs") ?: JSONObject()
    val appSettings = prefs.optJSONObject("boxapp_settings") ?: JSONObject()

    val appLang = appSettings.strOr("app_language", "system")
    val accelEnabled = appSettings.optBoolean("accelerated_download_enabled", false)
    val accelPrefix = appSettings.optString("accelerated_download_prefix", "")
    val ghTokenEnabled = appSettings.optBoolean("github_token_enabled", false)
    val ghToken = appSettings.optString("github_token", "")
    val tileTarget = appSettings.strOr("tile_long_click_target", "panel")
    val notifEnabled = appSettings.optBoolean("notifications_enabled", false)
    val showPanelTab = appSettings.optBoolean("show_proxy_selector_bottom_bar_item", true)
    val defaultPanel = appSettings.strOr("home_panel_entry_mode", "overview")
    val openPanelOnLaunch = appSettings.optBoolean("open_panel_on_launch", false)

    var showLangDialog by remember { mutableStateOf(false) }
    var showBackupDialog by remember { mutableStateOf(false) }
    var showMirrorDialog by remember { mutableStateOf(false) }
    var showTokenDialog by remember { mutableStateOf(false) }
    var showTileTargetDialog by remember { mutableStateOf(false) }
    var showPanelEntryDialog by remember { mutableStateOf(false) }

    val lang = LocalAppLang.current
    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixTabHeader(
            title = uiTr(lang, "设置", "設定", "Settings", "Настройки"),
            collapsed = collapsed,
        )

        LazyColumn(
            state = listState,
            contentPadding = PaddingValues(
                start = 16.dp,
                end = 16.dp,
                top = 8.dp,
                bottom = LocalLiquidDockBottomPadding.current,
            ),
            verticalArrangement = Arrangement.spacedBy(16.dp),
            modifier = Modifier.fillMaxSize(),
        ) {
            // Card 1: 基础代理配置 · 其他代理配置
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixNavRow(
                        title = "基础代理配置",
                        icon = { ToolRowGlyph("sliders", p.textPrimary) },
                        onClick = { navigateTo("proxy_config") },
                    )
                    MiuixInsetDivider()
                    MiuixNavRow(
                        title = "其他代理配置",
                        icon = { ToolRowGlyph("server", p.textPrimary) },
                        onClick = { navigateTo("other_proxy_config") },
                    )
                }
            }

            // Card 2: 语言 · 主题设置 · 备份与恢复
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixNavRow(
                        title = "语言",
                        icon = { ToolRowGlyph("language", p.textPrimary) },
                        trailingText = LANGUAGE_OPTIONS.firstOrNull { it.first == appLang }?.second ?: "跟随系统",
                        isDropdown = true,
                        onClick = { showLangDialog = true },
                    )
                    MiuixInsetDivider()
                    MiuixNavRow(
                        title = "主题设置",
                        icon = { ToolRowGlyph("brush", p.textPrimary) },
                        onClick = { navigateTo("theme_settings") },
                    )
                    MiuixInsetDivider()
                    MiuixNavRow(
                        title = "备份与恢复",
                        icon = { ToolRowGlyph("layers", p.textPrimary) },
                        onClick = { showBackupDialog = true },
                    )
                }
            }

            // Card 3: 开机自启 · 镜像下载 · GitHub Token
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixSwitchRow(
                        title = "开机自启",
                        icon = { ToolRowGlyph("power", p.textPrimary) },
                        checked = bootEnabled,
                        onCheckedChange = { enabled ->
                            dispatchAsync("set_boot_auto_start") { put("enabled", enabled) }
                        },
                    )
                    MiuixInsetDivider()
                    MiuixSwitchRow(
                        title = "镜像下载",
                        icon = { ToolRowGlyph("download", p.textPrimary) },
                        checked = accelEnabled,
                        onCheckedChange = { enabled ->
                            if (enabled && accelPrefix.isBlank()) {
                                showMirrorDialog = true
                            } else {
                                updateAppSetting(appSettings) { put("accelerated_download_enabled", enabled) }
                            }
                        },
                        onRowClick = { showMirrorDialog = true },
                    )
                    MiuixInsetDivider()
                    MiuixSwitchRow(
                        title = "GitHub Token",
                        icon = { ToolRowGlyph("lock", p.textPrimary) },
                        checked = ghTokenEnabled,
                        onCheckedChange = { enabled ->
                            if (enabled && ghToken.isBlank()) {
                                showTokenDialog = true
                            } else {
                                updateAppSetting(appSettings) { put("github_token_enabled", enabled) }
                            }
                        },
                        onRowClick = { showTokenDialog = true },
                    )
                }
            }

            // Card 4: 长按磁贴 · 通知 · 显示底栏面板入口 · 默认面板页面 · 启动时打开面板
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixNavRow(
                        title = "长按磁贴",
                        icon = { ToolRowGlyph("sparkle", p.textPrimary) },
                        trailingText = TILE_TARGET_OPTIONS.firstOrNull { it.first == tileTarget }?.second ?: "面板",
                        isDropdown = true,
                        onClick = { showTileTargetDialog = true },
                    )
                    MiuixInsetDivider()
                    MiuixSwitchRow(
                        title = "通知",
                        icon = { ToolRowGlyph("bell", p.textPrimary) },
                        checked = notifEnabled,
                        onCheckedChange = { enabled ->
                            if (enabled) onRequestNotificationPermission()
                            updateAppSetting(appSettings) { put("notifications_enabled", enabled) }
                        },
                        onRowClick = { navigateTo("notification_settings") },
                    )
                    MiuixInsetDivider()
                    MiuixSwitchRow(
                        title = "显示底栏面板入口",
                        icon = { ToolRowGlyph("link", p.textPrimary) },
                        checked = showPanelTab,
                        onCheckedChange = { enabled ->
                            updateAppSetting(appSettings) { put("show_proxy_selector_bottom_bar_item", enabled) }
                        },
                    )
                    MiuixInsetDivider()
                    MiuixNavRow(
                        title = "默认面板页面",
                        icon = { ToolRowGlyph("grid", p.textPrimary) },
                        trailingText = PANEL_ENTRY_OPTIONS.firstOrNull { it.first == defaultPanel }?.second ?: "概览",
                        isDropdown = true,
                        onClick = { showPanelEntryDialog = true },
                    )
                    MiuixInsetDivider()
                    MiuixSwitchRow(
                        title = "启动时打开面板",
                        icon = { ToolRowGlyph("book", p.textPrimary) },
                        checked = openPanelOnLaunch,
                        onCheckedChange = { enabled ->
                            updateAppSetting(appSettings) { put("open_panel_on_launch", enabled) }
                        },
                    )
                }
            }

            // Card 5: 关于
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixNavRow(
                        title = "关于",
                        icon = { ToolRowGlyph("info", p.textPrimary) },
                        onClick = { navigateTo("about") },
                    )
                }
            }
        }
    }

    if (showLangDialog) {
        MiuixOptionSheetDialog(
            title = "语言",
            options = LANGUAGE_OPTIONS,
            selectedValue = appLang,
            onSelect = { lang -> dispatchAsync("set_language") { put("language", lang) } },
            onDismiss = { showLangDialog = false },
        )
    }
    if (showTileTargetDialog) {
        MiuixOptionSheetDialog(
            title = "长按磁贴",
            options = TILE_TARGET_OPTIONS,
            selectedValue = tileTarget,
            onSelect = { target ->
                updateAppSetting(appSettings) { put("tile_long_click_target", target) }
            },
            onDismiss = { showTileTargetDialog = false },
        )
    }
    if (showPanelEntryDialog) {
        MiuixOptionSheetDialog(
            title = "默认面板页面",
            options = PANEL_ENTRY_OPTIONS,
            selectedValue = defaultPanel,
            onSelect = { entry ->
                updateAppSetting(appSettings) { put("home_panel_entry_mode", entry) }
            },
            onDismiss = { showPanelEntryDialog = false },
        )
    }
    if (showMirrorDialog) {
        MiuixTextInputDialog(
            title = "镜像下载前缀",
            subtitle = "用于加速 GitHub 核心与 WebUI 下载（如 https://gh-proxy.com/）",
            initialValue = accelPrefix,
            placeholder = "https://gh-proxy.com/",
            confirmLabel = "保存并启用",
            onConfirm = { prefix ->
                updateAppSetting(appSettings) {
                    put("accelerated_download_prefix", prefix)
                    put("accelerated_download_enabled", prefix.isNotEmpty())
                }
            },
            onDismiss = { showMirrorDialog = false },
        )
    }
    if (showTokenDialog) {
        MiuixTextInputDialog(
            title = "GitHub Token",
            subtitle = "仅存本地配置，用于提升 GitHub Release API 请求额度",
            initialValue = ghToken,
            placeholder = "ghp_... / github_pat_...",
            isPassword = true,
            confirmLabel = "保存",
            onConfirm = { token ->
                updateAppSetting(appSettings) {
                    put("github_token", token)
                    put("github_token_enabled", token.isNotEmpty())
                }
            },
            onDismiss = { showTokenDialog = false },
        )
    }
    if (showBackupDialog) {
        MiuixBackupRestoreSheet(
            onRestore = { scope ->
                showBackupDialog = false
                onLaunchBackupRestore(scope)
            },
            onBackup = { scope ->
                showBackupDialog = false
                onLaunchBackupExport(scope)
            },
            onDismiss = { showBackupDialog = false },
        )
    }
}

/**
 * 1:1 Replica of `orig_set_backup.png` floating Miuix bottom backup/restore dialog with M49 scope toggles.
 */
@Composable
private fun MiuixBackupRestoreSheet(
    onRestore: (JSONObject) -> Unit,
    onBackup: (JSONObject) -> Unit,
    onDismiss: () -> Unit,
) {
    val p = LocalBoxPalette.current
    var appSettingsScope by remember { mutableStateOf(true) }
    var runtimeDataScope by remember { mutableStateOf(true) }
    var userScriptsScope by remember { mutableStateOf(true) }
    var coreBinariesScope by remember { mutableStateOf(false) }

    fun buildScope() = JSONObject()
        .put("app_settings", appSettingsScope)
        .put("runtime_data", runtimeDataScope)
        .put("user_scripts", userScriptsScope)
        .put("core_binaries", coreBinariesScope)

    IosBottomSheetDialog(onDismiss = onDismiss) {
        Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Text(
                text = "备份与恢复",
                fontSize = 20.sp,
                fontWeight = FontWeight.SemiBold,
                color = p.textPrimary,
                textAlign = TextAlign.Center,
                modifier = Modifier.fillMaxWidth(),
            )
            Text(
                text = "备份或恢复应用数据，恢复会覆盖当前数据",
                fontSize = 15.sp,
                color = p.textSecondary,
            )
            Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                ScopeCheckRow("应用设置与 box.db 数据库", appSettingsScope) { appSettingsScope = it }
                ScopeCheckRow("运行配置与规则数据", runtimeDataScope) { runtimeDataScope = it }
                ScopeCheckRow("用户 Shell / JS 脚本", userScriptsScope) { userScriptsScope = it }
                ScopeCheckRow("核心二进制文件", coreBinariesScope) { coreBinariesScope = it }
            }
            Row(
                modifier = Modifier.fillMaxWidth().padding(top = 6.dp),
                horizontalArrangement = Arrangement.spacedBy(14.dp),
            ) {
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .height(50.dp)
                        .iosPressScale { onRestore(buildScope()) }
                        .clip(RoundedCornerShape(16.dp))
                        .background(p.background),
                    contentAlignment = Alignment.Center,
                ) {
                    Text("恢复", fontSize = 17.sp, fontWeight = FontWeight.SemiBold, color = p.accent)
                }
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .height(50.dp)
                        .iosPressScale { onBackup(buildScope()) }
                        .clip(RoundedCornerShape(16.dp))
                        .background(p.accentSoft),
                    contentAlignment = Alignment.Center,
                ) {
                    Text("备份", fontSize = 17.sp, fontWeight = FontWeight.SemiBold, color = p.accent)
                }
            }
        }
    }
}

@Composable
private fun ScopeCheckRow(label: String, checked: Boolean, onToggle: (Boolean) -> Unit) {
    val p = LocalBoxPalette.current
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onToggle(!checked) }
            .padding(vertical = 2.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Checkbox(checked = checked, onCheckedChange = onToggle)
        Text(label, fontSize = 14.sp, color = p.textPrimary)
    }
}

/**
 * 1:1 Replica of `基础代理配置` (`orig_set_proxy1.png`, M21 + M22).
 */
@Composable
fun ProxyConfigScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val listState = rememberLazyListState()
    val collapsed by remember { derivedStateOf { listState.firstVisibleItemIndex > 0 || listState.firstVisibleItemScrollOffset > 50 } }

    val profile = state.optJSONObject("runtime_profile") ?: JSONObject()
    val overrides = state.optJSONObject("config_overrides") ?: JSONObject()
    val currentCore = profile.strOr("core_name", "honk")
    val currentMode = profile.strOr("mode", "ebpf")
    val ipv6Mode = profile.strOr("ipv6_mode", "enable")
    val autoSync = profile.optBoolean("auto_sync_config", true)
    val configName = profile.strOr("config_name", "config.dae")
    val cfgFilesArr = state.optJSONArray("core_config_files") ?: JSONArray()
    val cfgFiles = remember(cfgFilesArr.toString(), configName) {
        val list = mutableListOf<String>()
        for (i in 0 until cfgFilesArr.length()) {
            val s = cfgFilesArr.optString(i).trim()
            if (s.isNotEmpty()) list.add(s)
        }
        if (configName.isNotEmpty() && configName !in list) list.add(0, configName)
        list
    }

    var showCorePicker by remember { mutableStateOf(false) }
    var showModePicker by remember { mutableStateOf(false) }
    var showIpv6Picker by remember { mutableStateOf(false) }
    var showOverrideDialog by remember { mutableStateOf(false) }
    var showNewConfigDialog by remember { mutableStateOf(false) }

    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(
            title = "基础代理配置",
            collapsed = collapsed,
            onBack = { dispatchAsync("pop_back") },
        )

        LazyColumn(
            state = listState,
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 28.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
            modifier = Modifier.fillMaxSize(),
        ) {
            // Card 1: 核心选择 / 运行模式 / IPv6 / 自动覆写 / 覆写设置
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixSettingTextRow(
                        title = "核心选择",
                        trailingText = coreDisplayName(currentCore),
                        isDropdown = true,
                        onClick = { showCorePicker = true },
                    )
                    MiuixSettingTextRow(
                        title = "运行模式",
                        trailingText = modeDisplayName(currentMode),
                        isDropdown = true,
                        onClick = { showModePicker = true },
                    )
                    MiuixSettingTextRow(
                        title = "IPv6",
                        trailingText = IPV6_OPTIONS.firstOrNull { it.first == ipv6Mode }?.second ?: "启用 IPv6",
                        isDropdown = true,
                        onClick = { showIpv6Picker = true },
                    )
                    MiuixSettingSwitchRow(
                        title = "自动覆写",
                        checked = autoSync,
                        onCheckedChange = { checked ->
                            saveProfile(profile) { put("auto_sync_config", checked) }
                        },
                    )
                    MiuixSettingTextRow(
                        title = "覆写设置",
                        onClick = { showOverrideDialog = true },
                    )
                }
            }

            // Card 2: 查看启动配置
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixSettingTextRow(
                        title = "查看启动配置",
                        onClick = {
                            dispatchAsync("open_editor_file") {
                                put("relative_path", "run/state/config/startup-config")
                            }
                        },
                    )
                }
            }

            // Card 3: 配置选择 + (+) + file list with checkmark
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .padding(horizontal = 20.dp, vertical = 16.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            text = "配置选择",
                            fontSize = 17.sp,
                            fontWeight = FontWeight.Medium,
                            color = p.textPrimary,
                            modifier = Modifier.weight(1f),
                        )
                        Box(
                            modifier = Modifier
                                .size(32.dp)
                                .clip(CircleShape)
                                .clickable { showNewConfigDialog = true },
                            contentAlignment = Alignment.Center,
                        ) {
                            ToolRowGlyph("plus_circle", p.textPrimary, size = 24.dp)
                        }
                    }
                    cfgFiles.forEach { file ->
                        val selected = file == configName
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clickable {
                                    saveProfile(profile, applyLive = false) { put("config_name", file) }
                                }
                                .padding(horizontal = 20.dp, vertical = 16.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Text(
                                text = file,
                                fontSize = 17.sp,
                                color = p.textPrimary,
                                modifier = Modifier.weight(1f),
                            )
                            if (selected) {
                                ToolRowGlyph("check", p.accent, size = 20.dp)
                            }
                        }
                    }
                }
            }
        }
    }

    if (showCorePicker) {
        MiuixOptionSheetDialog(
            title = "核心选择",
            options = CORE_OPTIONS,
            selectedValue = currentCore,
            onSelect = { core -> saveProfile(profile) { put("core_name", core) } },
            onDismiss = { showCorePicker = false },
        )
    }
    if (showModePicker) {
        MiuixOptionSheetDialog(
            title = "运行模式",
            options = RUN_MODE_OPTIONS,
            selectedValue = currentMode,
            onSelect = { mode -> saveProfile(profile, applyLive = true) { put("mode", mode) } },
            onDismiss = { showModePicker = false },
        )
    }
    if (showIpv6Picker) {
        MiuixOptionSheetDialog(
            title = "IPv6",
            options = IPV6_OPTIONS,
            selectedValue = ipv6Mode,
            onSelect = { v -> saveProfile(profile, applyLive = true) { put("ipv6_mode", v) } },
            onDismiss = { showIpv6Picker = false },
        )
    }
    if (showNewConfigDialog) {
        val defaultExt = if (currentCore == "honk") ".dae" else if (currentCore in listOf("sing-box", "xray", "v2fly")) ".json" else ".yaml"
        MiuixTextInputDialog(
            title = "新建配置文件",
            subtitle = "将在 $currentCore/ 目录下创建空配置并打开编辑器",
            initialValue = "custom$defaultExt",
            placeholder = "config$defaultExt",
            confirmLabel = "创建",
            onConfirm = { filename ->
                if (filename.isNotEmpty()) {
                    dispatchAsync("save_editor_file") {
                        put("relative_path", "$currentCore/$filename")
                        put("content", "# $filename\n")
                    }
                    dispatchAsync("open_editor_file") {
                        put("relative_path", "$currentCore/$filename")
                    }
                }
            },
            onDismiss = { showNewConfigDialog = false },
        )
    }
    if (showOverrideDialog) {
        Dialog(
            onDismissRequest = { showOverrideDialog = false },
            properties = DialogProperties(usePlatformDefaultWidth = false),
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 32.dp)
                    .clip(RoundedCornerShape(28.dp))
                    .background(p.card)
                    .padding(vertical = 18.dp),
            ) {
                Text(
                    text = "覆写设置 (${coreDisplayName(currentCore)})",
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Medium,
                    color = p.textPrimary,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.fillMaxWidth().padding(bottom = 8.dp),
                )
                LazyColumn(modifier = Modifier.weight(1f, fill = false).height(380.dp)) {
                    items(CONFIG_OVERRIDE_KEYS.size) { idx ->
                        val (optKey, optLabel) = CONFIG_OVERRIDE_KEYS[idx]
                        val on = overrides.optString(optKey, "0") == "1"
                        MiuixSettingSwitchRow(
                            title = optLabel,
                            checked = on,
                            onCheckedChange = { checked ->
                                dispatchAsync("set_config_override") {
                                    put("core", currentCore)
                                    put("option", optKey)
                                    put("value", if (checked) "1" else "0")
                                }
                            },
                        )
                    }
                }
                Row(
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 10.dp),
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    Box(
                        modifier = Modifier
                            .weight(1f)
                            .height(46.dp)
                            .clip(RoundedCornerShape(14.dp))
                            .background(p.accentSoft)
                            .clickable {
                                dispatchAsync("config_sync") { put("manual", true) }
                                showOverrideDialog = false
                            },
                        contentAlignment = Alignment.Center,
                    ) {
                        Text("重新生成启动配置", fontSize = 15.sp, fontWeight = FontWeight.Medium, color = p.accent)
                    }
                    Box(
                        modifier = Modifier
                            .width(88.dp)
                            .height(46.dp)
                            .clip(RoundedCornerShape(14.dp))
                            .background(p.background)
                            .clickable { showOverrideDialog = false },
                        contentAlignment = Alignment.Center,
                    ) {
                        Text("关闭", fontSize = 15.sp, color = p.textSecondary)
                    }
                }
            }
        }
    }
}

/**
 * 1:1 Replica of `其他代理配置` (`orig_set_other1.png` + `orig_set_other2.png`, M23).
 */
@Composable
fun OtherProxyConfigScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val listState = rememberLazyListState()
    val collapsed by remember { derivedStateOf { listState.firstVisibleItemIndex > 0 || listState.firstVisibleItemScrollOffset > 50 } }

    val profile = state.optJSONObject("runtime_profile") ?: JSONObject()
    val perfMode = profile.optBoolean("performance_mode", false)
    val quicOn = profile.strOr("quic", "enable").equals("enable", true)
    val dnsForwardOn = profile.strOr("mihomo_dns_forward", "enable").equals("enable", true)
    val proxyTcp = profile.optBoolean("proxy_tcp", true)
    val proxyUdp = profile.optBoolean("proxy_udp", true)
    val dnsHijackTcp = profile.optBoolean("dns_hijack_tcp", true)
    val dnsHijackUdp = profile.optBoolean("dns_hijack_udp", true)
    val dnsHijackMode = profile.strOr("dns_hijack_mode", "tproxy")
    val tasksetCpu = profile.optBoolean("taskset_cpu", false)
    val cgroupMemcg = profile.optBoolean("cgroup_memcg", false)
    val cgroupBlkio = profile.optBoolean("cgroup_blkio", false)
    val cleanFirewall = profile.optBoolean("clean_vendor_firewall", false)

    var allowCpu by remember(profile.optString("allow_cpu")) { mutableStateOf(profile.strOr("allow_cpu", "0-7")) }
    var memcgLimit by remember(profile.optString("memcg_limit")) { mutableStateOf(profile.strOr("memcg_limit", "100M")) }
    var blkioWeight by remember(profile.optString("weight")) { mutableStateOf(profile.optString("weight", "")) }
    var showDnsHijackPicker by remember { mutableStateOf(false) }

    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(
            title = "其他代理配置",
            collapsed = collapsed,
            onBack = { dispatchAsync("pop_back") },
        )

        LazyColumn(
            state = listState,
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 28.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
            modifier = Modifier.fillMaxSize(),
        ) {
            // Card 1: 代理能力
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        text = "代理能力",
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Medium,
                        color = p.textPrimary,
                        modifier = Modifier.padding(start = 20.dp, top = 16.dp, bottom = 4.dp),
                    )
                    MiuixSettingSwitchRow("性能模式", perfMode) { v ->
                        saveProfile(profile) { put("performance_mode", v) }
                    }
                    MiuixSettingSwitchRow("QUIC", quicOn) { v ->
                        saveProfile(profile) { put("quic", if (v) "enable" else "disable") }
                    }
                    MiuixSettingSwitchRow("DNS 转发", dnsForwardOn) { v ->
                        saveProfile(profile) { put("mihomo_dns_forward", if (v) "enable" else "disable") }
                    }
                    MiuixSettingSwitchRow("代理 TCP", proxyTcp) { v ->
                        saveProfile(profile) { put("proxy_tcp", v) }
                    }
                    MiuixSettingSwitchRow("代理 UDP", proxyUdp) { v ->
                        saveProfile(profile) { put("proxy_udp", v) }
                    }
                }
            }

            // Card 2: DNS 劫持
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        text = "DNS 劫持",
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Medium,
                        color = p.textPrimary,
                        modifier = Modifier.padding(start = 20.dp, top = 16.dp, bottom = 4.dp),
                    )
                    MiuixSettingSwitchRow("DNS 劫持 TCP", dnsHijackTcp) { v ->
                        saveProfile(profile) { put("dns_hijack_tcp", v) }
                    }
                    MiuixSettingSwitchRow("DNS 劫持 UDP", dnsHijackUdp) { v ->
                        saveProfile(profile) { put("dns_hijack_udp", v) }
                    }
                    MiuixSettingTextRow(
                        title = "DNS 劫持策略",
                        trailingText = DNS_HIJACK_OPTIONS.firstOrNull { it.first.equals(dnsHijackMode, true) }?.second ?: "TPROXY",
                        isDropdown = true,
                        onClick = { showDnsHijackPicker = true },
                    )
                }
            }

            // Card 3: 资源限制
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        text = "资源限制",
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Medium,
                        color = p.textPrimary,
                        modifier = Modifier.padding(start = 20.dp, top = 16.dp, bottom = 4.dp),
                    )
                    MiuixSettingSwitchRow("CPU 核心分配", tasksetCpu) { v ->
                        saveProfile(profile) { put("taskset_cpu", v); put("allow_cpu", allowCpu) }
                    }
                    MiuixPillTextField(value = allowCpu, placeholder = "0-7", onValueChange = { allowCpu = it })

                    MiuixSettingSwitchRow("内存限制", cgroupMemcg) { v ->
                        saveProfile(profile) { put("cgroup_memcg", v); put("memcg_limit", memcgLimit) }
                    }
                    MiuixPillTextField(value = memcgLimit, placeholder = "100M", onValueChange = { memcgLimit = it })

                    MiuixSettingSwitchRow("磁盘 I/O 权重", cgroupBlkio) { v ->
                        saveProfile(profile) { put("cgroup_blkio", v); put("weight", blkioWeight) }
                    }
                    MiuixPillTextField(value = blkioWeight, placeholder = "权重", onValueChange = { blkioWeight = it })

                    Row(
                        modifier = Modifier.fillMaxWidth().padding(horizontal = 18.dp, vertical = 14.dp),
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Box(
                            modifier = Modifier
                                .weight(1f)
                                .height(44.dp)
                                .clip(RoundedCornerShape(14.dp))
                                .background(p.accentSoft)
                                .clickable {
                                    saveProfile(profile) {
                                        put("allow_cpu", allowCpu)
                                        put("memcg_limit", memcgLimit)
                                        put("weight", blkioWeight)
                                    }
                                },
                            contentAlignment = Alignment.Center,
                        ) {
                            Text("保存资源配置", fontSize = 14.sp, fontWeight = FontWeight.Medium, color = p.accent)
                        }
                        Box(
                            modifier = Modifier
                                .weight(1f)
                                .height(44.dp)
                                .clip(RoundedCornerShape(14.dp))
                                .background(p.background)
                                .clickable { dispatchAsync("apply_resources") },
                            contentAlignment = Alignment.Center,
                        ) {
                            Text("热应用限制", fontSize = 14.sp, color = p.textPrimary)
                        }
                    }
                }
            }

            // Card 4: 厂商防火墙
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        text = "厂商防火墙",
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Medium,
                        color = p.textPrimary,
                        modifier = Modifier.padding(start = 20.dp, top = 16.dp, bottom = 4.dp),
                    )
                    MiuixSettingSwitchRow("启动时清理", cleanFirewall) { v ->
                        saveProfile(profile) { put("clean_vendor_firewall", v) }
                    }
                }
            }
        }
    }

    if (showDnsHijackPicker) {
        MiuixOptionSheetDialog(
            title = "DNS 劫持策略",
            options = DNS_HIJACK_OPTIONS,
            selectedValue = dnsHijackMode.lowercase(),
            onSelect = { mode -> saveProfile(profile) { put("dns_hijack_mode", mode) } },
            onDismiss = { showDnsHijackPicker = false },
        )
    }
}

/**
 * 1:1 Replica of `主题设置` (`orig_set_theme1.png` + `orig_set_theme2.png`, M47).
 */
@Composable
fun ThemeSettingsScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val listState = rememberLazyListState()
    val collapsed by remember { derivedStateOf { listState.firstVisibleItemIndex > 0 || listState.firstVisibleItemScrollOffset > 50 } }

    val prefs = state.optJSONObject("prefs") ?: JSONObject()
    val appSettings = prefs.optJSONObject("boxapp_settings") ?: JSONObject()

    val uiMode = appSettings.strOr("ui_mode", "miuix")
    val themeMode = appSettings.optInt("theme_mode", 0)
    val pureBlack = appSettings.optBoolean("pure_black_dark_theme", true)
    val monetEnabled = appSettings.optBoolean("miuix_monet_enabled", true)
    val paletteStyle = appSettings.strOr("theme_palette_style", "TonalSpot")
    val colorSpec = appSettings.strOr("theme_color_spec", "Spec2021")
    val accentUseDefault = appSettings.optBoolean("accent_use_default", true)
    val accentColorInt = appSettings.optInt("accent_color", 0xFF455A8A.toInt())
    val enableBlur = appSettings.optBoolean("enable_blur", true)
    val topBarStyle = appSettings.strOr("top_bar_style", "Classic")
    val bottomBarStyle = appSettings.strOr("bottom_bar_style", "LiquidGlass")
    val swipeBack = appSettings.optBoolean("enable_swipe_back", true)
    val predictiveBack = appSettings.optBoolean("enable_predictive_back_animation", true)
    val followEdge = appSettings.optBoolean("predictive_back_animation_follow_edge", true)
    val pageScale = appSettings.optDouble("page_scale", 1.0).toFloat().coerceIn(0.8f, 1.2f)

    var showUiModePicker by remember { mutableStateOf(false) }
    var showThemeModePicker by remember { mutableStateOf(false) }
    var showPalettePicker by remember { mutableStateOf(false) }
    var showSpecPicker by remember { mutableStateOf(false) }
    var showAccentPicker by remember { mutableStateOf(false) }
    var showTopBarPicker by remember { mutableStateOf(false) }
    var showBottomBarPicker by remember { mutableStateOf(false) }

    val swatchDots = listOf(p.accent, Color(0xFFC5C7CE), Color(0xFFF3C6E2))
    val accentName = if (accentUseDefault) {
        "默认"
    } else {
        ACCENT_PRESETS.firstOrNull { it.second == accentColorInt }?.first ?: "自定义"
    }

    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(
            title = "主题设置",
            collapsed = collapsed,
            onBack = { dispatchAsync("pop_back") },
        )

        LazyColumn(
            state = listState,
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 28.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
            modifier = Modifier.fillMaxSize(),
        ) {
            // Card 1: 界面风格 / 主题模式
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixSettingTextRow(
                        title = "界面风格",
                        trailingText = if (uiMode == "miuix") "Miuix" else "Material 3",
                        isDropdown = true,
                        onClick = { showUiModePicker = true },
                    )
                    MiuixInsetDivider(startIndent = 20.dp)
                    MiuixSettingTextRow(
                        title = "主题模式",
                        trailingText = when (themeMode) {
                            1 -> "浅色"
                            2 -> "深色"
                            else -> "跟随系统"
                        },
                        isDropdown = true,
                        onClick = { showThemeModePicker = true },
                    )
                }
            }

            // Card 2: 深色使用纯黑背景
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixSettingSwitchRow("深色使用纯黑背景", pureBlack) { v ->
                        updateAppSetting(appSettings) { put("pure_black_dark_theme", v) }
                    }
                }
            }

            // Card 3: 使用 Monet 取色 / 色彩风格 / 色彩标准 / 强调色
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixSettingSwitchRow("使用 Monet 取色", monetEnabled) { v ->
                        updateAppSetting(appSettings) { put("miuix_monet_enabled", v) }
                    }
                    MiuixInsetDivider(startIndent = 20.dp)
                    MiuixSettingTextRow(
                        title = "色彩风格",
                        trailingText = PALETTE_STYLE_OPTIONS.firstOrNull { it.first == paletteStyle }?.second ?: paletteStyle,
                        isDropdown = true,
                        swatchColors = swatchDots,
                        onClick = { showPalettePicker = true },
                    )
                    MiuixInsetDivider(startIndent = 20.dp)
                    MiuixSettingTextRow(
                        title = "色彩标准",
                        trailingText = COLOR_SPEC_OPTIONS.firstOrNull { it.first == colorSpec }?.second ?: colorSpec,
                        isDropdown = true,
                        onClick = { showSpecPicker = true },
                    )
                    MiuixInsetDivider(startIndent = 20.dp)
                    MiuixSettingTextRow(
                        title = "强调色",
                        trailingText = accentName,
                        isDropdown = true,
                        swatchColors = swatchDots,
                        onClick = { showAccentPicker = true },
                    )
                }
            }

            // Card 4: 模糊效果 / 顶栏样式 / 底栏样式
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixSettingSwitchRow("模糊效果", enableBlur) { v ->
                        updateAppSetting(appSettings) { put("enable_blur", v) }
                    }
                    MiuixInsetDivider(startIndent = 20.dp)
                    MiuixSettingTextRow(
                        title = "顶栏样式",
                        trailingText = TOP_BAR_OPTIONS.firstOrNull { it.first == topBarStyle }?.second ?: "经典",
                        isDropdown = true,
                        onClick = { showTopBarPicker = true },
                    )
                    MiuixInsetDivider(startIndent = 20.dp)
                    MiuixSettingTextRow(
                        title = "底栏样式",
                        trailingText = BOTTOM_BAR_OPTIONS.firstOrNull { it.first == bottomBarStyle }?.second ?: "经典",
                        isDropdown = true,
                        onClick = { showBottomBarPicker = true },
                    )
                }
            }

            // Card 5: 横滑返回 / 预测性返回动画 / 动画方向跟随滑动边缘
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixSettingSwitchRow("横滑返回", swipeBack) { v ->
                        updateAppSetting(appSettings) { put("enable_swipe_back", v) }
                    }
                    MiuixInsetDivider(startIndent = 20.dp)
                    MiuixSettingSwitchRow("预测性返回动画", predictiveBack) { v ->
                        updateAppSetting(appSettings) { put("enable_predictive_back_animation", v) }
                    }
                    MiuixInsetDivider(startIndent = 20.dp)
                    MiuixSettingSwitchRow("动画方向跟随滑动边缘", followEdge) { v ->
                        updateAppSetting(appSettings) { put("predictive_back_animation_follow_edge", v) }
                    }
                }
            }

            // Card 6: 界面缩放 100% > + 5-stop Miuix pill slider
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixSettingTextRow(
                        title = "界面缩放",
                        trailingText = "${(pageScale * 100).toInt()}%",
                        onClick = {
                            updateAppSetting(appSettings) { put("page_scale", 1.0) }
                        },
                    )
                    MiuixDiscreteScaleSlider(
                        currentScale = pageScale,
                        onScaleSelected = { next ->
                            updateAppSetting(appSettings) { put("page_scale", next.toDouble()) }
                        },
                    )
                    Spacer(Modifier.height(6.dp))
                }
            }
        }
    }

    if (showUiModePicker) {
        MiuixOptionSheetDialog(
            title = "界面风格",
            options = listOf("miuix" to "Miuix", "material" to "Material 3"),
            selectedValue = uiMode,
            onSelect = { v -> updateAppSetting(appSettings) { put("ui_mode", v) } },
            onDismiss = { showUiModePicker = false },
        )
    }
    if (showThemeModePicker) {
        MiuixOptionSheetDialog(
            title = "主题模式",
            options = listOf("0" to "跟随系统", "1" to "浅色", "2" to "深色"),
            selectedValue = themeMode.toString(),
            onSelect = { v -> updateAppSetting(appSettings) { put("theme_mode", v.toIntOrNull() ?: 0) } },
            onDismiss = { showThemeModePicker = false },
        )
    }
    if (showPalettePicker) {
        MiuixOptionSheetDialog(
            title = "色彩风格",
            options = PALETTE_STYLE_OPTIONS,
            selectedValue = paletteStyle,
            onSelect = { v -> updateAppSetting(appSettings) { put("theme_palette_style", v) } },
            onDismiss = { showPalettePicker = false },
        )
    }
    if (showSpecPicker) {
        MiuixOptionSheetDialog(
            title = "色彩标准",
            options = COLOR_SPEC_OPTIONS,
            selectedValue = colorSpec,
            onSelect = { v -> updateAppSetting(appSettings) { put("theme_color_spec", v) } },
            onDismiss = { showSpecPicker = false },
        )
    }
    if (showTopBarPicker) {
        MiuixOptionSheetDialog(
            title = "顶栏样式",
            options = TOP_BAR_OPTIONS,
            selectedValue = topBarStyle,
            onSelect = { v -> updateAppSetting(appSettings) { put("top_bar_style", v) } },
            onDismiss = { showTopBarPicker = false },
        )
    }
    if (showBottomBarPicker) {
        MiuixOptionSheetDialog(
            title = "底栏样式",
            options = BOTTOM_BAR_OPTIONS,
            selectedValue = bottomBarStyle,
            onSelect = { v -> updateAppSetting(appSettings) { put("bottom_bar_style", v) } },
            onDismiss = { showBottomBarPicker = false },
        )
    }
    if (showAccentPicker) {
        MiuixOptionSheetDialog(
            title = "强调色",
            options = ACCENT_PRESETS.map { (label, colorInt) -> colorInt.toString() to label },
            selectedValue = if (accentUseDefault) ACCENT_PRESETS.first().second.toString() else accentColorInt.toString(),
            onSelect = { raw ->
                val cInt = raw.toIntOrNull() ?: ACCENT_PRESETS.first().second
                val isDef = cInt == ACCENT_PRESETS.first().second
                updateAppSetting(appSettings) {
                    put("accent_use_default", isDef)
                    put("accent_color", cInt)
                }
            },
            onDismiss = { showAccentPicker = false },
        )
    }
}

/**
 * 1:1 Replica of `常驻通知设置` (M57 / M58) with 12 placeholder chips, interval, click action, and 3 buttons.
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun NotificationSettingsScreen(
    state: JSONObject,
    onRequestNotificationPermission: () -> Unit,
) {
    val p = LocalBoxPalette.current
    val listState = rememberLazyListState()
    val collapsed by remember { derivedStateOf { listState.firstVisibleItemIndex > 0 || listState.firstVisibleItemScrollOffset > 50 } }

    val prefs = state.optJSONObject("prefs") ?: JSONObject()
    val appSettings = prefs.optJSONObject("boxapp_settings") ?: JSONObject()
    val enabled = appSettings.optBoolean("notifications_enabled", false)
    val intervalSec = appSettings.optInt("notification_refresh_interval_seconds", 5)
    val clickTarget = appSettings.strOr("notification_click_target", "panel")

    var titleTpl by remember(appSettings.optString("notification_title_template")) {
        mutableStateOf(appSettings.optString("notification_title_template", "BoxProxy Honk"))
    }
    var contentTpl by remember(appSettings.optString("notification_content_template")) {
        mutableStateOf(appSettings.optString("notification_content_template", "{status} · {uptime}\n↑ {upload} · ↓ {download}"))
    }
    var showIntervalPicker by remember { mutableStateOf(false) }
    var showClickTargetPicker by remember { mutableStateOf(false) }
    var editingBtnSlot by remember { mutableStateOf<Int?>(null) }

    val placeholders = listOf(
        "{status}", "{pid}", "{core}", "{mode}", "{config}", "{uptime}",
        "{cpu}", "{memory}", "{upload}", "{download}", "{total_upload}", "{total_download}",
    )
    val btnTargets = listOf(
        "reload" to "重载",
        "stop" to "停止",
        "restart" to "重启",
        "panel" to "打开面板",
        "web_ui" to "打开 WebUI",
    )

    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(
            title = "常驻通知设置",
            collapsed = collapsed,
            onBack = { dispatchAsync("pop_back") },
        )

        LazyColumn(
            state = listState,
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 28.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
            modifier = Modifier.fillMaxSize(),
        ) {
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixSettingSwitchRow("开启前台常驻状态通知", enabled) { checked ->
                        if (checked) onRequestNotificationPermission()
                        updateAppSetting(appSettings) { put("notifications_enabled", checked) }
                    }
                    MiuixSettingTextRow(
                        title = "刷新间隔",
                        trailingText = "${intervalSec} 秒",
                        isDropdown = true,
                        onClick = { showIntervalPicker = true },
                    )
                    MiuixSettingTextRow(
                        title = "点击通知动作",
                        trailingText = TILE_TARGET_OPTIONS.firstOrNull { it.first == clickTarget }?.second ?: "面板",
                        isDropdown = true,
                        onClick = { showClickTargetPicker = true },
                    )
                }
            }

            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                        Text("通知文案模板（12 占位符）", fontSize = 16.sp, fontWeight = FontWeight.Medium, color = p.textPrimary)
                        OutlinedTextField(
                            value = titleTpl,
                            onValueChange = { titleTpl = it },
                            label = { Text("标题模板") },
                            singleLine = true,
                            shape = RoundedCornerShape(14.dp),
                            modifier = Modifier.fillMaxWidth(),
                        )
                        OutlinedTextField(
                            value = contentTpl,
                            onValueChange = { contentTpl = it },
                            label = { Text("正文模板") },
                            minLines = 3,
                            shape = RoundedCornerShape(14.dp),
                            modifier = Modifier.fillMaxWidth(),
                        )
                        FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                            placeholders.forEach { ph ->
                                Box(
                                    modifier = Modifier
                                        .clip(RoundedCornerShape(10.dp))
                                        .background(p.accentSoft)
                                        .clickable { contentTpl += ph }
                                        .padding(horizontal = 10.dp, vertical = 5.dp),
                                ) {
                                    Text(ph, fontSize = 12.sp, fontFamily = FontFamily.Monospace, color = p.accent)
                                }
                            }
                        }
                        Box(
                            modifier = Modifier
                                .fillMaxWidth()
                                .height(46.dp)
                                .clip(RoundedCornerShape(14.dp))
                                .background(p.accentSoft)
                                .clickable {
                                    updateAppSetting(appSettings) {
                                        put("notification_title_template", titleTpl)
                                        put("notification_content_template", contentTpl)
                                    }
                                },
                            contentAlignment = Alignment.Center,
                        ) {
                            Text("保存通知模板", fontSize = 15.sp, fontWeight = FontWeight.Medium, color = p.accent)
                        }
                    }
                }
            }

            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        text = "快捷操作按钮（最多 3 个）",
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Medium,
                        color = p.textPrimary,
                        modifier = Modifier.padding(start = 20.dp, top = 16.dp, bottom = 4.dp),
                    )
                    listOf(1, 2, 3).forEach { slot ->
                        val key = "notification_action_$slot"
                        val btnObj = appSettings.optJSONObject(key) ?: JSONObject()
                        val bEnabled = btnObj.optBoolean("enabled", true)
                        val bTarget = btnObj.strOr("target", if (slot == 1) "reload" else if (slot == 2) "stop" else "restart")
                        val bLabel = btnTargets.firstOrNull { it.first == bTarget }?.second ?: bTarget
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .clickable { editingBtnSlot = slot }
                                .padding(horizontal = 20.dp, vertical = 14.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Text("按钮 $slot ($bLabel)", fontSize = 16.sp, color = p.textPrimary, modifier = Modifier.weight(1f))
                            MiuixSwitch(
                                checked = bEnabled,
                                onCheckedChange = { checked ->
                                    updateAppSetting(appSettings) {
                                        val nextBtn = JSONObject(btnObj.toString()).put("enabled", checked)
                                        put(key, nextBtn)
                                    }
                                },
                            )
                        }
                    }
                }
            }
        }
    }

    if (showIntervalPicker) {
        MiuixOptionSheetDialog(
            title = "刷新间隔",
            options = listOf("1" to "1 秒", "2" to "2 秒", "3" to "3 秒", "5" to "5 秒", "10" to "10 秒", "15" to "15 秒", "30" to "30 秒"),
            selectedValue = intervalSec.toString(),
            onSelect = { v -> updateAppSetting(appSettings) { put("notification_refresh_interval_seconds", v.toIntOrNull() ?: 5) } },
            onDismiss = { showIntervalPicker = false },
        )
    }
    if (showClickTargetPicker) {
        MiuixOptionSheetDialog(
            title = "点击通知动作",
            options = TILE_TARGET_OPTIONS,
            selectedValue = clickTarget,
            onSelect = { v -> updateAppSetting(appSettings) { put("notification_click_target", v) } },
            onDismiss = { showClickTargetPicker = false },
        )
    }
    editingBtnSlot?.let { slot ->
        val key = "notification_action_$slot"
        val btnObj = appSettings.optJSONObject(key) ?: JSONObject()
        MiuixOptionSheetDialog(
            title = "按钮 $slot 动作",
            options = btnTargets,
            selectedValue = btnObj.strOr("target", "reload"),
            onSelect = { target ->
                updateAppSetting(appSettings) {
                    val nextBtn = JSONObject(btnObj.toString()).put("target", target)
                    put(key, nextBtn)
                }
            },
            onDismiss = { editingBtnSlot = null },
        )
    }
}

/**
 * 1:1 Replica of `关于` (`orig_set_about.png`, M50 / M51 / M52).
 */
@Composable
fun AboutScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val ctx = LocalContext.current
    val prefs = state.optJSONObject("prefs") ?: JSONObject()
    val iconVariant = prefs.optJSONObject("app_icon")?.strOr("variant", "default") ?: "default"
    val routeName = state.optJSONObject("current_route")?.strOr("route", "about") ?: "about"

    if (routeName == "about_libraries") {
        AboutLibrariesScreen()
        return
    }
    if (routeName == "about_sponsorship") {
        AboutSponsorshipScreen()
        return
    }

    val heroBg = if (!p.isDark) {
        Brush.verticalGradient(
            colors = listOf(Color(0xFFE7E8FA), Color(0xFFFCE6F0), Color(0xFFE6F0FB), p.background),
        )
    } else {
        Brush.verticalGradient(
            colors = listOf(Color(0xFF1F1D30), Color(0xFF281A24), p.background),
        )
    }
    val brandColor = if (!p.isDark) Color(0xFF7B1F56) else Color(0xFFE69AC4)

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(heroBg)
            .statusBarsPadding(),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(50.dp)
                .padding(horizontal = 10.dp),
        ) {
            Box(
                modifier = Modifier
                    .align(Alignment.CenterStart)
                    .size(40.dp)
                    .clip(CircleShape)
                    .clickable { dispatchAsync("pop_back") },
                contentAlignment = Alignment.Center,
            ) {
                ToolRowGlyph("back", p.textPrimary, size = 22.dp)
            }
        }

        LazyColumn(
            contentPadding = PaddingValues(start = 16.dp, end = 16.dp, top = 28.dp, bottom = 28.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier.fillMaxSize(),
        ) {
            item {
                Column(
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.spacedBy(10.dp),
                    modifier = Modifier.padding(top = 18.dp, bottom = 28.dp),
                ) {
                    BoxCatAboutLogo(color = brandColor, size = 114.dp)
                    Text(
                        text = "BoxProxy Honk",
                        fontSize = 30.sp,
                        fontWeight = FontWeight.Bold,
                        color = brandColor,
                    )
                    Text(
                        text = state.strOr("app_version", "26.11.28.pure-rust"),
                        fontSize = 15.sp,
                        color = p.textSecondary,
                    )
                    Text(
                        text = "纯 Rust MVI 引擎 (libboxproxy_app.so) · 58/58 模块 · box.db v15",
                        fontSize = 12.sp,
                        color = p.textMuted,
                    )
                }
            }

            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixSettingTextRow(
                        title = "赞助支持",
                        onClick = { navigateTo("about_sponsorship") },
                    )
                }
            }

            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixSettingTextRow(
                        title = "GitHub 项目",
                        trailingText = "Wcoom/honk-android",
                        onClick = {
                            runCatching {
                                ctx.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse("https://github.com/Wcoom/honk-android")))
                            }
                        },
                    )
                    MiuixSettingTextRow(
                        title = "开源引用库",
                        onClick = { navigateTo("about_libraries") },
                    )
                }
            }

            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixSettingTextRow(
                        title = "桌面应用图标",
                        trailingText = if (iconVariant == "alternate") "备用图标" else "默认图标",
                        isDropdown = true,
                        onClick = {
                            val next = if (iconVariant == "alternate") "default" else "alternate"
                            dispatchAsync("set_app_icon_variant") { put("variant", next) }
                        },
                    )
                }
            }
        }
    }
}

@Composable
private fun AboutLibrariesScreen() {
    val p = LocalBoxPalette.current
    val libs = listOf(
        "honk-core (Pure Rust eBPF Core)" to "MIT / Apache-2.0",
        "boxctl & boxbpf (Pure Rust Control Plane)" to "MIT",
        "libboxproxy_app.so (8-Crate Rust MVI Engine)" to "MIT",
        "Leptos 0.8 + wasm-bindgen (Doona WebUI)" to "MIT",
        "rusqlite / tokio / reqwest / axum / zip" to "MIT / Apache-2.0",
        "Jetpack Compose + AndroidX Activity" to "Apache-2.0",
    )
    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "开源引用库", collapsed = false, onBack = { dispatchAsync("pop_back") })
        LazyColumn(
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            items(libs.size) { idx ->
                val (name, license) = libs[idx]
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        Text(name, fontSize = 16.sp, fontWeight = FontWeight.Medium, color = p.textPrimary)
                        Text(license, fontSize = 13.sp, color = p.textSecondary)
                    }
                }
            }
        }
    }
}

@Composable
private fun AboutSponsorshipScreen() {
    val p = LocalBoxPalette.current
    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "赞助支持", collapsed = false, onBack = { dispatchAsync("pop_back") })
        Column(modifier = Modifier.padding(16.dp)) {
            MiuixCard(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Text("感谢使用 BoxProxy Honk (Pure Rust)", fontSize = 17.sp, fontWeight = FontWeight.Medium, color = p.textPrimary)
                    Text(
                        "本项目在开源社区支持下持续演进。欢迎前往 GitHub 仓库 Star、反馈 Issue 或贡献 PR。",
                        fontSize = 14.sp,
                        color = p.textSecondary,
                    )
                }
            }
        }
    }
}

/**
 * 1:1 Miuix `延迟设置` (M48) opened from Home latency card sliders icon.
 */
@Composable
fun LatencyTargetsScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val prefs = state.optJSONObject("prefs") ?: JSONObject()
    val lt = prefs.optJSONObject("latency_targets")?.optJSONArray("targets") ?: JSONArray()
    var n1 by remember { mutableStateOf(lt.optJSONObject(0)?.strOr("name", "Baidu") ?: "Baidu") }
    var u1 by remember { mutableStateOf(lt.optJSONObject(0)?.strOr("url", "https://baidu.com") ?: "https://baidu.com") }
    var n2 by remember { mutableStateOf(lt.optJSONObject(1)?.strOr("name", "Cloudflare") ?: "Cloudflare") }
    var u2 by remember { mutableStateOf(lt.optJSONObject(1)?.strOr("url", "https://cloudflare.com") ?: "https://cloudflare.com") }
    var n3 by remember { mutableStateOf(lt.optJSONObject(2)?.strOr("name", "Google") ?: "Google") }
    var u3 by remember { mutableStateOf(lt.optJSONObject(2)?.strOr("url", "https://google.com") ?: "https://google.com") }

    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "延迟测速目标", collapsed = false, onBack = { dispatchAsync("pop_back") })
        LazyColumn(
            contentPadding = PaddingValues(16.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                        OutlinedTextField(value = n1, onValueChange = { n1 = it }, label = { Text("目标 1 名称") }, singleLine = true, shape = RoundedCornerShape(14.dp), modifier = Modifier.fillMaxWidth())
                        OutlinedTextField(value = u1, onValueChange = { u1 = it }, label = { Text("目标 1 URL") }, singleLine = true, shape = RoundedCornerShape(14.dp), modifier = Modifier.fillMaxWidth())
                        HorizontalDivider(color = p.divider)
                        OutlinedTextField(value = n2, onValueChange = { n2 = it }, label = { Text("目标 2 名称") }, singleLine = true, shape = RoundedCornerShape(14.dp), modifier = Modifier.fillMaxWidth())
                        OutlinedTextField(value = u2, onValueChange = { u2 = it }, label = { Text("目标 2 URL") }, singleLine = true, shape = RoundedCornerShape(14.dp), modifier = Modifier.fillMaxWidth())
                        HorizontalDivider(color = p.divider)
                        OutlinedTextField(value = n3, onValueChange = { n3 = it }, label = { Text("目标 3 名称") }, singleLine = true, shape = RoundedCornerShape(14.dp), modifier = Modifier.fillMaxWidth())
                        OutlinedTextField(value = u3, onValueChange = { u3 = it }, label = { Text("目标 3 URL") }, singleLine = true, shape = RoundedCornerShape(14.dp), modifier = Modifier.fillMaxWidth())
                        Box(
                            modifier = Modifier
                                .fillMaxWidth()
                                .padding(top = 6.dp)
                                .height(48.dp)
                                .clip(RoundedCornerShape(14.dp))
                                .background(p.accentSoft)
                                .clickable {
                                    val arr = JSONArray()
                                        .put(JSONObject().put("name", n1).put("url", u1))
                                        .put(JSONObject().put("name", n2).put("url", u2))
                                        .put(JSONObject().put("name", n3).put("url", u3))
                                    dispatchAsync("save_latency_targets") { put("targets", arr) }
                                    dispatchAsync("probe_latency_targets")
                                    dispatchAsync("pop_back")
                                },
                            contentAlignment = Alignment.Center,
                        ) {
                            Text("保存并重新测速", fontSize = 16.sp, fontWeight = FontWeight.Medium, color = p.accent)
                        }
                    }
                }
            }
        }
    }
}
