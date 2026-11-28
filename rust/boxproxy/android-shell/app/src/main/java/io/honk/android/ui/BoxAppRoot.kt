package io.honk.android.ui

import android.content.Intent
import android.net.Uri
import android.os.Build
import androidx.activity.BackEventCompat
import androidx.activity.compose.PredictiveBackHandler
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.ContentTransform
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.slideOutVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.layer.drawLayer
import androidx.compose.ui.graphics.rememberGraphicsLayer
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import io.honk.android.bridge.BoxEngineHolder
import io.honk.android.webview.BoxWebViewHost
import org.json.JSONArray
import org.json.JSONObject
import java.util.Locale
import java.util.concurrent.Executors
import kotlin.math.roundToInt

internal val uiDispatchExecutor = Executors.newSingleThreadExecutor()

internal fun dispatchAsync(action: String, block: JSONObject.() -> Unit = {}) {
    uiDispatchExecutor.execute {
        BoxEngineHolder.dispatchAction(action, block)
    }
}

internal fun navigateTo(routeName: String) {
    dispatchAsync("navigate") {
        put("route", JSONObject().put("route", routeName))
    }
}

internal fun JSONObject.strOr(key: String, fallback: String = "-"): String {
    if (isNull(key)) return fallback
    val v = optString(key, "").trim()
    return if (v.isEmpty() || v.equals("null", ignoreCase = true)) fallback else v
}

/**
 * Pure stateless 1:1 Miuix Compose renderer driven exclusively by Rust `AppState` JSON snapshots.
 */
@Composable
fun BoxAppRoot(
    onLaunchBackupExport: (JSONObject) -> Unit,
    onLaunchBackupRestore: (JSONObject) -> Unit,
    onLaunchImportCore: () -> Unit,
    onRequestNotificationPermission: () -> Unit,
) {
    val ctx = LocalContext.current
    val state by BoxEngineHolder.state.collectAsState()
    val prefs = state.optJSONObject("prefs") ?: JSONObject()
    val appSettings = prefs.optJSONObject("boxapp_settings") ?: JSONObject()

    val uiMode = appSettings.strOr("ui_mode", "miuix")
    val themeMode = appSettings.optInt("theme_mode", 0)
    val pureBlack = appSettings.optBoolean("pure_black_dark_theme", true)
    val monetEnabled = appSettings.optBoolean("miuix_monet_enabled", true)
    val paletteStyle = appSettings.strOr("theme_palette_style", "TonalSpot")
    val colorSpec = appSettings.strOr("theme_color_spec", "Spec2021")
    val enableBlur = appSettings.optBoolean("enable_blur", true)
    val topBarGlass = appSettings.strOr("top_bar_style", "Classic").equals("Glass", ignoreCase = true)
    val swipeBack = appSettings.optBoolean("enable_swipe_back", true)
    val predictiveBack = appSettings.optBoolean("enable_predictive_back_animation", true)
    val followEdge = appSettings.optBoolean("predictive_back_animation_follow_edge", true)
    val rawLang = appSettings.strOr("app_language", "system")
    val resolvedLang = remember(rawLang) {
        if (rawLang == "system") Locale.getDefault().toLanguageTag() else rawLang
    }

    val isDark = when (themeMode) {
        1 -> false
        2 -> true
        else -> isSystemInDarkTheme()
    }
    val accentUseDefault = appSettings.optBoolean("accent_use_default", true)
    val monetAccent = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S && monetEnabled && accentUseDefault) {
        if (isDark) dynamicDarkColorScheme(ctx).primary else dynamicLightColorScheme(ctx).primary
    } else null
    val accentOverride = when {
        !accentUseDefault -> Color(appSettings.optInt("accent_color", 0xFF455A8A.toInt()))
        monetAccent != null -> monetAccent
        else -> null
    }
    val palette = remember(isDark, pureBlack, accentOverride, uiMode, paletteStyle, colorSpec, enableBlur, topBarGlass) {
        boxPaletteFor(
            isDark = isDark,
            pureBlack = pureBlack,
            accentOverride = accentOverride,
            uiMode = uiMode,
            paletteStyle = paletteStyle,
            colorSpec = colorSpec,
            enableBlur = enableBlur,
            topBarGlass = topBarGlass,
        )
    }
    val pageScale = appSettings.optDouble("page_scale", 1.0).toFloat().coerceIn(0.8f, 1.2f)
    val baseDensity = LocalDensity.current
    val scaledDensity = remember(baseDensity, pageScale) {
        Density(baseDensity.density * pageScale, baseDensity.fontScale)
    }

    val colorScheme = if (isDark) {
        darkColorScheme(
            primary = palette.accent,
            background = palette.background,
            surface = palette.card,
            onBackground = palette.textPrimary,
            onSurface = palette.textPrimary,
        )
    } else {
        lightColorScheme(
            primary = palette.accent,
            background = palette.background,
            surface = palette.card,
            onBackground = palette.textPrimary,
            onSurface = palette.textPrimary,
        )
    }

    val routeObj = state.optJSONObject("current_route") ?: JSONObject().put("route", "main")
    val routeName = routeObj.optString("route", "main")
    val routeDepth = state.optJSONArray("back_stack")?.length() ?: if (routeName == "main") 0 else 1
    val activeTab = state.optInt("active_tab", 0)
    val showPanelTab = appSettings.optBoolean("show_proxy_selector_bottom_bar_item", true)
    val bottomBarStyle = appSettings.strOr("bottom_bar_style", "LiquidGlass")
    val useLiquidDock = !bottomBarStyle.equals("Classic", ignoreCase = true)

    val lastEffect by BoxEngineHolder.lastEffect.collectAsState()
    var bannerText by remember { mutableStateOf<String?>(null) }
    var bannerTone by remember { mutableStateOf("info") }
    LaunchedEffect(lastEffect) {
        val eff = lastEffect ?: return@LaunchedEffect
        val kind = eff.optString("effect", eff.optString("type", ""))
        val msg = eff.optString("message", eff.optString("text", "")).trim()
        if (msg.isNotEmpty()) {
            bannerText = msg
            bannerTone = when {
                kind.contains("error", true) || msg.contains("失败") || msg.contains("错误") -> "error"
                kind.contains("warn", true) -> "warn"
                msg.contains("成功") || msg.contains("已") || msg.contains("完成") -> "success"
                else -> "info"
            }
        }
    }

    val motionCfg = remember { LiquidGlassRuntime.config }
    var rawBackProgress by remember { mutableFloatStateOf(0f) }
    var backSwipeEdge by remember { mutableIntStateOf(0) }
    val smoothBackProgress by animateFloatAsState(
        targetValue = rawBackProgress,
        animationSpec = spring(dampingRatio = 0.86f, stiffness = 520f),
        label = "iosPredictiveBack",
    )

    LaunchedEffect(routeName) {
        rawBackProgress = 0f
    }

    PredictiveBackHandler(enabled = routeName != "main" && routeName != "onboarding") { progress ->
        try {
            progress.collect { event ->
                backSwipeEdge = event.swipeEdge
                rawBackProgress = if (predictiveBack && routeName != "web_ui" && routeName != "sub_store") {
                    event.progress
                } else {
                    0f
                }
            }
            rawBackProgress = 0f
            dispatchAsync("pop_back")
        } finally {
            rawBackProgress = 0f
        }
    }

    CompositionLocalProvider(
        LocalBoxPalette provides palette,
        LocalAppLang provides resolvedLang,
        LocalDensity provides scaledDensity,
        LocalLiquidDockBottomPadding provides if (useLiquidDock) 108.dp else 24.dp,
    ) {
        MaterialTheme(colorScheme = colorScheme) {
            if (routeName == "onboarding") {
                OnboardingScreen(state = state)
                return@MaterialTheme
            }

            val backdropLayer = rememberGraphicsLayer()

            Box(modifier = Modifier.fillMaxSize().background(palette.background)) {
                Scaffold(
                    containerColor = palette.background,
                    bottomBar = {
                        if (!useLiquidDock && routeName == "main") {
                            MiuixBottomBar(
                                activeTab = activeTab,
                                showPanelTab = showPanelTab,
                                onSelect = { tab -> dispatchAsync("switch_tab") { put("tab", tab) } },
                            )
                        }
                    },
                ) { padding ->
                    Box(
                        modifier = Modifier
                            .fillMaxSize()
                            .padding(bottom = if (useLiquidDock) 0.dp else padding.calculateBottomPadding())
                            .drawWithContent {
                                if (useLiquidDock) {
                                    backdropLayer.record {
                                        this@drawWithContent.drawContent()
                                    }
                                    drawLayer(backdropLayer)
                                } else {
                                    drawContent()
                                }
                            },
                    ) {
                        AnimatedContent(
                            targetState = Pair(routeName, routeDepth),
                            transitionSpec = {
                                val (fromRoute, fromDepth) = initialState
                                val (toRoute, toDepth) = targetState
                                val isPop = toRoute == "main" && fromRoute != "main" || toDepth < fromDepth
                                val isWebPortal = toRoute == "web_ui" || toRoute == "sub_store" || fromRoute == "web_ui" || fromRoute == "sub_store"
                                val parallax = motionCfg.navParallaxRatio
                                if (isWebPortal && !isPop) {
                                    // iOS App-Card / Safari Sheet expansion transition (zero flash, smooth zoom + rise)
                                    ContentTransform(
                                        targetContentEnter = scaleIn(
                                            initialScale = 0.93f,
                                            animationSpec = tween(340, easing = IosNavEasing),
                                        ) + slideInVertically(
                                            animationSpec = tween(340, easing = IosNavEasing),
                                            initialOffsetY = { h -> (h * 0.05f).roundToInt() },
                                        ) + fadeIn(
                                            animationSpec = tween(220, easing = IosNavEasing),
                                        ),
                                        initialContentExit = scaleOut(
                                            targetScale = 0.95f,
                                            animationSpec = tween(340, easing = IosNavEasing),
                                        ) + fadeOut(
                                            animationSpec = tween(300, easing = IosNavEasing),
                                            targetAlpha = 0.72f,
                                        ),
                                        targetContentZIndex = 1f,
                                    )
                                } else if (isWebPortal && isPop) {
                                    ContentTransform(
                                        targetContentEnter = scaleIn(
                                            initialScale = 0.95f,
                                            animationSpec = tween(310, easing = IosNavEasing),
                                        ) + fadeIn(
                                            animationSpec = tween(260, easing = IosNavEasing),
                                            initialAlpha = 0.75f,
                                        ),
                                        initialContentExit = scaleOut(
                                            targetScale = 0.93f,
                                            animationSpec = tween(290, easing = IosNavEasing),
                                        ) + slideOutVertically(
                                            animationSpec = tween(290, easing = IosNavEasing),
                                            targetOffsetY = { h -> (h * 0.05f).roundToInt() },
                                        ) + fadeOut(
                                            animationSpec = tween(220, easing = IosNavEasing),
                                        ),
                                        targetContentZIndex = -1f,
                                    )
                                } else if (isPop) {
                                    ContentTransform(
                                        targetContentEnter = slideInHorizontally(
                                            animationSpec = tween(motionCfg.navPopMs, easing = IosNavEasing),
                                            initialOffsetX = { w -> -(w * parallax).roundToInt() },
                                        ) + scaleIn(
                                            initialScale = 0.968f,
                                            animationSpec = tween(motionCfg.navPopMs, easing = IosNavEasing),
                                        ) + fadeIn(
                                            animationSpec = tween(motionCfg.navPopMs, easing = IosNavEasing),
                                            initialAlpha = 0.80f,
                                        ),
                                        initialContentExit = slideOutHorizontally(
                                            animationSpec = tween(motionCfg.navPopMs, easing = IosNavEasing),
                                            targetOffsetX = { w -> w },
                                        ),
                                        targetContentZIndex = -1f,
                                    )
                                } else {
                                    ContentTransform(
                                        targetContentEnter = slideInHorizontally(
                                            animationSpec = tween(motionCfg.navPushMs, easing = IosNavEasing),
                                            initialOffsetX = { w -> w },
                                        ),
                                        initialContentExit = slideOutHorizontally(
                                            animationSpec = tween(motionCfg.navPushMs, easing = IosNavEasing),
                                            targetOffsetX = { w -> -(w * parallax).roundToInt() },
                                        ) + scaleOut(
                                            targetScale = 0.968f,
                                            animationSpec = tween(motionCfg.navPushMs, easing = IosNavEasing),
                                        ) + fadeOut(
                                            animationSpec = tween(motionCfg.navPushMs, easing = IosNavEasing),
                                            targetAlpha = 0.80f,
                                        ),
                                        targetContentZIndex = 1f,
                                    )
                                }
                            },
                            contentKey = { it.first },
                            label = "iosRouteNav",
                        ) { (currentRoute, _) ->
                            val isPushedPage = currentRoute != "main" && currentRoute != "onboarding"
                            val enableEdgeSwipe = swipeBack &&
                                isPushedPage &&
                                currentRoute != "web_ui" &&
                                currentRoute != "sub_store"
                            val edgeSign = if (followEdge && backSwipeEdge == BackEventCompat.EDGE_RIGHT) -1f else 1f
                            Box(
                                modifier = Modifier
                                    .fillMaxSize()
                                    .graphicsLayer {
                                        if (isPushedPage && smoothBackProgress > 0.001f) {
                                            translationX = edgeSign * smoothBackProgress * size.width * 0.24f
                                            val sc = 1f - 0.065f * smoothBackProgress
                                            scaleX = sc
                                            scaleY = sc
                                            shape = RoundedCornerShape((28f * smoothBackProgress).dp)
                                            clip = true
                                        }
                                    }
                                    .drawWithContent {
                                        drawContent()
                                        if (isPushedPage) {
                                            // iOS UINavigationController leading-edge card shadow
                                            val shadowW = 14.dp.toPx()
                                            drawRect(
                                                brush = Brush.horizontalGradient(
                                                    colors = listOf(Color.Black.copy(alpha = 0.16f), Color.Transparent),
                                                    startX = 0f,
                                                    endX = shadowW,
                                                ),
                                                size = Size(shadowW, size.height),
                                            )
                                        }
                                    }
                                    .iosInteractiveSwipeBack(
                                        enabled = enableEdgeSwipe,
                                        predictiveScale = predictiveBack,
                                        followEdge = followEdge,
                                        onPopBack = { dispatchAsync("pop_back") },
                                    ),
                            ) {
                                when (currentRoute) {
                                    "main" -> AnimatedContent(
                                        targetState = activeTab,
                                        transitionSpec = {
                                            val dir = if (targetState >= initialState) 1 else -1
                                            val shift = 28
                                            (slideInHorizontally(
                                                animationSpec = tween(motionCfg.tabSwitchMs, easing = IosNavEasing),
                                                initialOffsetX = { dir * shift },
                                            ) + fadeIn(
                                                animationSpec = tween(motionCfg.tabSwitchMs, easing = IosNavEasing),
                                            ) + scaleIn(
                                                initialScale = 0.985f,
                                                animationSpec = tween(motionCfg.tabSwitchMs, easing = IosNavEasing),
                                            )) togetherWith (slideOutHorizontally(
                                                animationSpec = tween(motionCfg.tabSwitchMs - 40, easing = IosNavEasing),
                                                targetOffsetX = { -dir * (shift / 2) },
                                            ) + fadeOut(
                                                animationSpec = tween(motionCfg.tabSwitchMs - 40, easing = IosNavEasing),
                                            ))
                                        },
                                        label = "iosMainTab",
                                    ) { tab ->
                                        when (tab) {
                                            0 -> HomeTabScreen(state)
                                            1 -> PanelTabScreen(state)
                                            2 -> ToolsTabScreen(state, onLaunchImportCore = onLaunchImportCore)
                                            else -> SettingsTabScreen(
                                                state = state,
                                                onLaunchBackupExport = onLaunchBackupExport,
                                                onLaunchBackupRestore = onLaunchBackupRestore,
                                                onRequestNotificationPermission = onRequestNotificationPermission,
                                            )
                                        }
                                    }
                                    "web_ui" -> WebUiRouteScreen(state)
                                    "sub_store" -> SubStoreRouteScreen(state)
                                    "overview" -> OverviewDetailScreen(state)
                                    "proxy_selector" -> ProxySelectorDetailScreen(state)
                                    "connections_details" -> ConnectionsDetailScreen(state)
                                    "rules" -> RulesDetailScreen(state)
                                    "rule_providers" -> RuleProvidersDetailScreen(state)
                                    "subscription_details", "subscription_local_details" -> SubscriptionsDetailScreen(state)
                                    "proxy_config" -> ProxyConfigScreen(state)
                                    "other_proxy_config" -> OtherProxyConfigScreen(state)
                                    "app_management" -> AppManagementScreen(state)
                                    "cnip_settings" -> CnipSettingsScreen(state)
                                    "bypass_rules" -> BypassRulesScreen(state)
                                    "shared_network_config" -> SharedNetworkScreen(state)
                                    "network_match" -> NetworkMatchScreen(state)
                                    "shell_scripts", "js_scripts", "subscription_management" -> ScriptsHubScreen(state)
                                    "runtime_files" -> RuntimeFilesScreen(state)
                                    "runtime_file_editor" -> FileEditorScreen(state)
                                    "log_viewer" -> LogViewerScreen(state)
                                    "core_update_sources" -> CoreUpdateSourcesScreen(state)
                                    "network_test", "network_speed_history" -> NetworkDiagnosticsScreen(state)
                                    "theme_settings" -> ThemeSettingsScreen(state)
                                    "latency_targets" -> LatencyTargetsScreen(state)
                                    "notification_settings" -> NotificationSettingsScreen(state, onRequestNotificationPermission)
                                    "about", "about_libraries", "about_sponsorship" -> AboutScreen(state)
                                    else -> OverviewDetailScreen(state)
                                }
                            }
                        }
                    }
                }

                // Floating pure-Rust Liquid Glass Bottom Bar with iOS `hidesBottomBarWhenPushed` glide.
                if (useLiquidDock) {
                    AnimatedVisibility(
                        visible = routeName == "main",
                        enter = slideInVertically(
                            initialOffsetY = { it + 32 },
                            animationSpec = spring(dampingRatio = 0.84f, stiffness = 420f),
                        ) + fadeIn(tween(220, easing = IosNavEasing)),
                        exit = slideOutVertically(
                            targetOffsetY = { it + 32 },
                            animationSpec = tween(220, easing = IosNavEasing),
                        ) + fadeOut(tween(180)),
                        modifier = Modifier.align(Alignment.BottomCenter),
                    ) {
                        LiquidGlassBottomBar(
                            activeTab = activeTab,
                            showPanelTab = showPanelTab,
                            backdropLayer = backdropLayer,
                            onSelect = { tab -> dispatchAsync("switch_tab") { put("tab", tab) } },
                            barStyle = bottomBarStyle,
                            enableBlur = enableBlur,
                        )
                    }
                }

                // Global transient status feedback pill (loading / success / error).
                BoxStatusBannerOverlay(
                    message = bannerText,
                    tone = bannerTone,
                    onDismiss = { bannerText = null },
                    modifier = Modifier.align(Alignment.TopCenter),
                )
            }
        }
    }
}

/**
 * 1:1 Replica of original `com.boxproxy.honk` Home Tab (`orig_home_run.png` + `orig_home_bot.png`).
 */
@Composable
fun HomeTabScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val listState = rememberLazyListState()
    val collapsed by remember { derivedStateOf { listState.firstVisibleItemIndex > 0 || listState.firstVisibleItemScrollOffset > 60 } }

    val profile = state.optJSONObject("runtime_profile") ?: JSONObject()
    val statusObj = state.optJSONObject("service_status") ?: JSONObject()
    val statusStr = statusObj.strOr("status", "stopped")
    val isRunning = statusStr == "running"
    val isBusy = statusStr in listOf("starting", "stopping", "restarting")
    val proc = state.optJSONObject("process") ?: JSONObject()
    val prefs = state.optJSONObject("prefs") ?: JSONObject()
    val ipMode = prefs.optJSONObject("home_ui")?.strOr("home_ip_mode", "wan") ?: "wan"
    val geo = state.optJSONObject("ip_geo") ?: JSONObject()
    val latencies = state.optJSONArray("latency_results")
    val defaultTargets = prefs.optJSONObject("latency_targets")?.optJSONArray("targets")
    val pendingUpdate = state.optJSONObject("pending_core_update")

    var showSystemDetails by remember { mutableStateOf(false) }
    var showStopConfirm by remember { mutableStateOf(false) }

    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixTabHeader(title = "BoxProxy Honk", collapsed = collapsed)

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
            if (pendingUpdate != null && pendingUpdate.optBoolean("update_available", false)) {
                item {
                    MiuixCard(
                        modifier = Modifier.fillMaxWidth(),
                        backgroundColor = p.accentSoft,
                    ) {
                        Row(
                            modifier = Modifier.fillMaxWidth().padding(horizontal = 18.dp, vertical = 14.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Column(modifier = Modifier.weight(1f)) {
                                Text(
                                    "发现新版本 ${pendingUpdate.strOr("remote_version")}",
                                    fontSize = 15.sp,
                                    fontWeight = FontWeight.Medium,
                                    color = p.accent,
                                )
                                Text(
                                    "点击立即更新 ${coreDisplayName(pendingUpdate.strOr("target_core", "honk"))}",
                                    fontSize = 12.sp,
                                    color = p.textSecondary,
                                )
                            }
                            Text(
                                "更新",
                                fontSize = 14.sp,
                                fontWeight = FontWeight.Medium,
                                color = p.accent,
                                modifier = Modifier
                                    .clip(RoundedCornerShape(12.dp))
                                    .background(p.card)
                                    .clickable {
                                        dispatchAsync("install_core_update") {
                                            put("source_id", pendingUpdate.strOr("source_id", "honk"))
                                        }
                                    }
                                    .padding(horizontal = 14.dp, vertical = 6.dp),
                            )
                        }
                    }
                }
            }

            // 1. Hero Status Card (exact 1:1 `orig_home_run.png`)
            item {
                val heroBg = if (isRunning) p.heroRunningBg else p.heroStoppedBg
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(22.dp))
                        .background(heroBg)
                        .clickable { showSystemDetails = true },
                ) {
                    Box(
                        modifier = Modifier
                            .align(Alignment.BottomEnd)
                            .offset(x = 18.dp, y = 26.dp),
                    ) {
                        HeroStatusWatermark(
                            running = isRunning,
                            color = if (isRunning) p.heroAccent.copy(alpha = 0.85f) else p.divider,
                            size = 138.dp,
                        )
                    }
                    Column(
                        modifier = Modifier.padding(horizontal = 22.dp, vertical = 20.dp),
                        verticalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        Row(
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(10.dp),
                        ) {
                            Box(
                                modifier = Modifier
                                    .size(9.dp)
                                    .clip(CircleShape)
                                    .background(if (isRunning) p.heroAccent else p.textMuted),
                            )
                            Text(
                                text = when {
                                    isRunning -> "运行中"
                                    isBusy -> "切换中…"
                                    else -> "已停止"
                                },
                                fontSize = 23.sp,
                                fontWeight = FontWeight.Medium,
                                color = if (isRunning) p.heroAccent else p.textPrimary,
                            )
                        }
                        Spacer(Modifier.height(2.dp))
                        Text(
                            text = if (isRunning) formatUptimeZh(proc.optLong("uptime_secs", 0L)) else "未启动",
                            fontSize = 15.sp,
                            color = p.textPrimary,
                        )
                        Text(
                            text = "${coreDisplayName(profile.strOr("core_name", "honk"))} · ${modeDisplayName(profile.strOr("mode", "ebpf"))}",
                            fontSize = 15.sp,
                            color = p.textPrimary,
                        )
                        Text(
                            text = profile.strOr("config_name", "config.dae"),
                            fontSize = 15.sp,
                            color = p.textPrimary,
                        )
                    }
                }
            }

            // 2. Service Action Pill Bar (single clear primary CTA when stopped; guarded Stop when running)
            item {
                if (!isRunning) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(56.dp)
                            .iosPressScale(enabled = !isBusy, pressedScale = 0.975f) {
                                dispatchAsync("service_up")
                            }
                            .clip(RoundedCornerShape(20.dp))
                            .background(if (isBusy) p.accent.copy(alpha = 0.65f) else p.accent),
                        contentAlignment = Alignment.Center,
                    ) {
                        Text(
                            text = if (isBusy) "启动中…" else "启动服务",
                            fontSize = 17.sp,
                            fontWeight = FontWeight.SemiBold,
                            letterSpacing = 0.2.sp,
                            color = Color.White,
                        )
                    }
                } else {
                    MiuixCard(modifier = Modifier.fillMaxWidth()) {
                        Row(
                            modifier = Modifier.fillMaxWidth().height(56.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Box(
                                modifier = Modifier
                                    .weight(1f)
                                    .fillMaxSize()
                                    .iosPressScale { dispatchAsync("service_reload") },
                                contentAlignment = Alignment.Center,
                            ) {
                                Text("重载", fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = p.accent)
                            }
                            Box(modifier = Modifier.width(1.dp).height(22.dp).background(p.divider))
                            Box(
                                modifier = Modifier
                                    .weight(1f)
                                    .fillMaxSize()
                                    .iosPressScale { showStopConfirm = true },
                                contentAlignment = Alignment.Center,
                            ) {
                                Text("停止", fontSize = 16.sp, fontWeight = FontWeight.Medium, color = p.stopRed)
                            }
                            Box(modifier = Modifier.width(1.dp).height(22.dp).background(p.divider))
                            Box(
                                modifier = Modifier
                                    .weight(1f)
                                    .fillMaxSize()
                                    .iosPressScale { dispatchAsync("service_restart") },
                                contentAlignment = Alignment.Center,
                            ) {
                                Text("重启", fontSize = 16.sp, fontWeight = FontWeight.Medium, color = p.restartAmber)
                            }
                        }
                    }
                }
            }

            // 3. 2-col Quick Row: WebUI | 日志
            item {
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    MiuixCard(
                        modifier = Modifier.weight(1f),
                        onClick = { navigateTo("web_ui") },
                    ) {
                        Column(modifier = Modifier.padding(horizontal = 20.dp, vertical = 18.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                            Text("WebUI", fontSize = 17.sp, color = p.textPrimary)
                            Text("Web 界面", fontSize = 14.sp, color = p.textSecondary)
                        }
                    }
                    MiuixCard(
                        modifier = Modifier.weight(1f),
                        onClick = {
                            dispatchAsync("list_logs")
                            navigateTo("log_viewer")
                        },
                    ) {
                        Column(modifier = Modifier.padding(horizontal = 20.dp, vertical = 18.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                            Text("日志", fontSize = 17.sp, color = p.textPrimary)
                            Text("查看", fontSize = 14.sp, color = p.textSecondary)
                        }
                    }
                }
            }

            // 4. 延迟 Card (3-column Baidu / Cloudflare / Google)
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(horizontal = 20.dp, vertical = 18.dp)) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Text("延迟", fontSize = 17.sp, color = p.textPrimary, modifier = Modifier.weight(1f))
                            Box(
                                modifier = Modifier
                                    .size(32.dp)
                                    .clip(CircleShape)
                                    .clickable { navigateTo("latency_targets") },
                                contentAlignment = Alignment.Center,
                            ) {
                                ToolRowGlyph("sliders", p.textPrimary, size = 20.dp)
                            }
                            Spacer(Modifier.width(8.dp))
                            Box(
                                modifier = Modifier
                                    .size(32.dp)
                                    .clip(CircleShape)
                                    .clickable { dispatchAsync("probe_latency_targets") },
                                contentAlignment = Alignment.Center,
                            ) {
                                ToolRowGlyph("refresh", p.textPrimary, size = 20.dp)
                            }
                        }
                        Spacer(Modifier.height(14.dp))
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            val fallbackNames = listOf("Baidu", "Cloudflare", "Google")
                            for (i in 0 until 3) {
                                val resObj = latencies?.optJSONObject(i)
                                val cfgObj = defaultTargets?.optJSONObject(i)
                                val label = resObj?.strOr("name", "")?.ifEmpty { null }
                                    ?: cfgObj?.strOr("name", "")?.ifEmpty { null }
                                    ?: fallbackNames[i]
                                val valueText = when {
                                    resObj == null -> "-"
                                    resObj.optBoolean("ok", false) -> "${resObj.optLong("latency_ms", 0)} ms"
                                    else -> "超时"
                                }
                                val valueColor = when {
                                    resObj == null -> p.textMuted
                                    resObj.optBoolean("ok", false) -> p.accent
                                    else -> p.stopRed
                                }
                                Column(
                                    modifier = Modifier.weight(1f),
                                    horizontalAlignment = Alignment.CenterHorizontally,
                                    verticalArrangement = Arrangement.spacedBy(6.dp),
                                ) {
                                    Text(label, fontSize = 13.sp, color = p.textSecondary, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                    Text(valueText, fontSize = 17.sp, fontWeight = FontWeight.Normal, color = valueColor)
                                }
                                if (i < 2) {
                                    Box(modifier = Modifier.width(1.dp).height(34.dp).background(p.divider))
                                }
                            }
                        }
                    }
                }
            }

            // 5. 2-col Telemetry Row: WAN/LAN | 网速
            item {
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    MiuixCard(
                        modifier = Modifier.weight(1f),
                        onClick = { dispatchAsync("toggle_home_ip_mode") },
                    ) {
                        Column(modifier = Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                            Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                                Text(
                                    text = if (ipMode == "lan") "LAN" else "WAN",
                                    fontSize = 16.sp,
                                    color = p.textPrimary,
                                    modifier = Modifier.weight(1f),
                                )
                                Box(
                                    modifier = Modifier
                                        .clip(RoundedCornerShape(50))
                                        .background(p.accentSoft)
                                        .clickable {
                                            dispatchAsync("refresh_ip_geo")
                                            navigateTo("network_test")
                                        }
                                        .padding(horizontal = 10.dp, vertical = 3.dp),
                                ) {
                                    Text("详情", fontSize = 12.sp, color = p.accent)
                                }
                            }
                            Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                                Text("IP", fontSize = 14.sp, color = p.textSecondary)
                                Spacer(Modifier.width(6.dp))
                                Text(
                                    text = geo.strOr("ipv4", "-"),
                                    fontSize = 15.sp,
                                    color = p.textPrimary,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                    textAlign = TextAlign.End,
                                    modifier = Modifier.weight(1f),
                                )
                            }
                            Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                                Text("地区", fontSize = 14.sp, color = p.textSecondary)
                                Spacer(Modifier.width(6.dp))
                                Text(
                                    text = countryBadge(geo.optString("country_code", ""), geo.optString("country", "")),
                                    fontSize = 15.sp,
                                    color = p.textPrimary,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                    textAlign = TextAlign.End,
                                    modifier = Modifier.weight(1f),
                                )
                            }
                        }
                    }

                    MiuixCard(
                        modifier = Modifier.weight(1f),
                        onClick = {
                            dispatchAsync("refresh_connections")
                            navigateTo("connections_details")
                        },
                    ) {
                        Column(modifier = Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                            Text("网速", fontSize = 16.sp, color = p.textPrimary)
                            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                                Text("上行", fontSize = 14.sp, color = p.textSecondary)
                                Text(formatRateCompact(state.optLong("upload_bps", 0L)), fontSize = 16.sp, color = p.textPrimary)
                            }
                            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                                Text("下行", fontSize = 14.sp, color = p.textSecondary)
                                Text(formatRateCompact(state.optLong("download_bps", 0L)), fontSize = 16.sp, color = p.textPrimary)
                            }
                        }
                    }
                }
            }

            // 6. 2-col Bottom Row: 订阅 | 资源占用
            item {
                val pssMb = proc.optDouble("pss_mb", 0.0)
                val cpuPct = proc.optDouble("cpu_percent", 0.0)
                val subData = state.opt("proxy_providers_data")
                val hasSub = subData != null && subData.toString() != "null" && subData.toString().length > 4

                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    MiuixCard(
                        modifier = Modifier.weight(1f),
                        onClick = {
                            dispatchAsync("refresh_subscriptions")
                            navigateTo("subscription_details")
                        },
                    ) {
                        Column(modifier = Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
                            Text("订阅", fontSize = 16.sp, color = p.textPrimary)
                            Text(
                                text = if (hasSub) "点击查看订阅流量详情" else "刷新后显示数据",
                                fontSize = 14.sp,
                                color = p.textSecondary,
                                textAlign = TextAlign.Center,
                                modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp),
                            )
                        }
                    }

                    MiuixCard(
                        modifier = Modifier.weight(1f),
                        onClick = { showSystemDetails = true },
                    ) {
                        Column(modifier = Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                            Text("资源占用", fontSize = 16.sp, color = p.textPrimary)
                            if (!isRunning) {
                                Text(
                                    text = "启动后显示数据",
                                    fontSize = 14.sp,
                                    color = p.textSecondary,
                                    textAlign = TextAlign.Center,
                                    modifier = Modifier.fillMaxWidth().padding(vertical = 10.dp),
                                )
                            } else {
                                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                                    Text("内存", fontSize = 14.sp, color = p.textSecondary)
                                    Text("${pssMb.toInt()}MB", fontSize = 15.sp, color = p.textPrimary)
                                }
                                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                                    Text("CPU", fontSize = 14.sp, color = p.textSecondary)
                                    Text("${"%.1f".format(cpuPct)}%", fontSize = 15.sp, color = p.textPrimary)
                                }
                                Box(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .height(4.dp)
                                        .clip(RoundedCornerShape(50))
                                        .background(p.accentSoft),
                                ) {
                                    Box(
                                        modifier = Modifier
                                            .fillMaxWidth((pssMb / 256.0).toFloat().coerceIn(0.08f, 1f))
                                            .height(4.dp)
                                            .clip(RoundedCornerShape(50))
                                            .background(p.accent),
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if (showSystemDetails) {
        HomeSystemDetailsDialog(
            state = state,
            onDismiss = { showSystemDetails = false },
        )
    }

    if (showStopConfirm) {
        MiuixConfirmDialog(
            title = "停止代理服务？",
            message = "停止后将断开透明代理接管，你可以随时再次启动服务。",
            confirmLabel = "确认停止",
            destructive = true,
            onConfirm = { dispatchAsync("service_down") },
            onDismiss = { showStopConfirm = false },
        )
    }
}

@Composable
private fun HomeSystemDetailsDialog(state: JSONObject, onDismiss: () -> Unit) {
    val p = LocalBoxPalette.current
    val profile = state.optJSONObject("runtime_profile") ?: JSONObject()
    val proc = state.optJSONObject("process") ?: JSONObject()
    val api = state.optJSONObject("active_api") ?: JSONObject()
    val mode = state.strOr("outbound_mode", "rule")
    val rootOk = state.optBoolean("root_granted", false)

    IosBottomSheetDialog(onDismiss = onDismiss) {
        Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Text(
                "运行系统详情",
                fontSize = 19.sp,
                fontWeight = FontWeight.SemiBold,
                color = p.textPrimary,
                textAlign = TextAlign.Center,
                modifier = Modifier.fillMaxWidth(),
            )
            DetailKvRow("核心 / 模式", "${coreDisplayName(profile.strOr("core_name", "honk"))} · ${modeDisplayName(profile.strOr("mode", "ebpf"))}")
            DetailKvRow("配置文件", profile.strOr("config_name", "config.dae"))
            DetailKvRow("进程 PID", proc.optInt("pid", 0).let { if (it > 0) it.toString() else "-" })
            DetailKvRow("运行时长", formatUptimeZh(proc.optLong("uptime_secs", 0L)))
            DetailKvRow("CPU / 内存", "${"%.1f".format(proc.optDouble("cpu_percent", 0.0))}% · ${proc.optDouble("pss_mb", 0.0).toInt()} MB")
            DetailKvRow("API 接口", "${api.strOr("kind", "honk_native")} (${api.strOr("base_url")})")
            DetailKvRow("Root 授权", if (rootOk) "已授权 (uid=0)" else "未授权")

            HorizontalDivider(color = p.divider, modifier = Modifier.padding(vertical = 4.dp))
            Text("出站模式快速切换", fontSize = 14.sp, color = p.textSecondary)
            val modes = listOf("rule", "direct", "global")
            IosSegmentedControl(
                items = listOf("规则", "直连", "全局"),
                selectedIndex = modes.indexOfFirst { it.equals(mode, true) }.coerceAtLeast(0),
                onSelect = { idx -> dispatchAsync("switch_outbound_mode") { put("mode", modes[idx]) } },
                modifier = Modifier.fillMaxWidth(),
                trackColor = p.background,
            )
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 4.dp)
                    .height(46.dp)
                    .iosPressScale(onClick = onDismiss)
                    .clip(RoundedCornerShape(16.dp))
                    .background(p.background),
                contentAlignment = Alignment.Center,
            ) {
                Text("关闭", fontSize = 16.sp, color = p.textSecondary)
            }
        }
    }
}

@Composable
private fun DetailKvRow(k: String, v: String) {
    val p = LocalBoxPalette.current
    Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
        Text(k, fontSize = 14.sp, color = p.textSecondary)
        Text(v, fontSize = 14.sp, color = p.textPrimary, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

/**
 * 1:1 Replica of original `com.boxproxy.honk` Panel Tab (`orig_panel.png`).
 */
@Composable
fun PanelTabScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val prefs = state.optJSONObject("prefs") ?: JSONObject()
    val defaultEntry = prefs.optJSONObject("boxapp_settings")?.strOr("home_panel_entry_mode", "overview") ?: "overview"
    var segment by remember {
        mutableIntStateOf(
            when (defaultEntry) {
                "proxy_selector" -> 1
                "subscription" -> 2
                "connections" -> 3
                else -> 0
            },
        )
    }
    var menuExpanded by remember { mutableStateOf(false) }
    var showApiConfigDialog by remember { mutableStateOf(false) }

    LaunchedEffect(segment) {
        when (segment) {
            0 -> dispatchAsync("refresh_status")
            1 -> dispatchAsync("refresh_proxies")
            2 -> dispatchAsync("refresh_subscriptions")
            3 -> dispatchAsync("refresh_connections")
        }
    }

    val listState = rememberLazyListState()
    val collapsed by remember { derivedStateOf { listState.firstVisibleItemIndex > 0 || listState.firstVisibleItemScrollOffset > 50 } }

    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixTabHeader(
            title = "面板",
            collapsed = collapsed,
            trailing = {
                Box {
                    Box(
                        modifier = Modifier
                            .size(38.dp)
                            .clip(CircleShape)
                            .clickable { menuExpanded = true },
                        contentAlignment = Alignment.Center,
                    ) {
                        ToolRowGlyph("more_vert", p.textPrimary, size = 22.dp)
                    }
                    DropdownMenu(expanded = menuExpanded, onDismissRequest = { menuExpanded = false }) {
                        DropdownMenuItem(
                            text = { Text("内置 WebUI") },
                            onClick = { menuExpanded = false; navigateTo("web_ui") },
                        )
                        DropdownMenuItem(
                            text = { Text("分流规则与追踪") },
                            onClick = { menuExpanded = false; dispatchAsync("refresh_rules"); navigateTo("rules") },
                        )
                        DropdownMenuItem(
                            text = { Text("规则集管理") },
                            onClick = { menuExpanded = false; dispatchAsync("refresh_rule_providers"); navigateTo("rule_providers") },
                        )
                        DropdownMenuItem(
                            text = { Text("API 配置") },
                            onClick = { menuExpanded = false; showApiConfigDialog = true },
                        )
                        DropdownMenuItem(
                            text = { Text("刷新数据") },
                            onClick = { menuExpanded = false; dispatchAsync("refresh_status"); dispatchAsync("refresh_overview") },
                        )
                    }
                }
            },
        )

        // 4-segment spring-gliding iOS pill switcher: 概览 | 策略 | 订阅 | 连接
        Box(modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 6.dp)) {
            IosSegmentedControl(
                items = listOf("概览", "策略", "订阅", "连接"),
                selectedIndex = segment,
                onSelect = { segment = it },
                modifier = Modifier.fillMaxWidth(),
                height = 46.dp,
            )
        }

        AnimatedContent(
            targetState = segment,
            transitionSpec = {
                val dir = if (targetState >= initialState) 1 else -1
                (slideInHorizontally(
                    animationSpec = tween(220, easing = IosNavEasing),
                    initialOffsetX = { dir * 24 },
                ) + fadeIn(tween(220, easing = IosNavEasing))) togetherWith (slideOutHorizontally(
                    animationSpec = tween(180, easing = IosNavEasing),
                    targetOffsetX = { -dir * 14 },
                ) + fadeOut(tween(180)))
            },
            label = "panelSegment",
        ) { seg ->
            when (seg) {
                0 -> PanelOverviewContent(
                    state = state,
                    listState = listState,
                    onJumpSegment = { segment = it },
                )
                1 -> ProxySelectorInlineBody(state = state)
                2 -> SubscriptionsInlineBody(state = state)
                else -> ConnectionsInlineBody(state = state)
            }
        }
    }

    if (showApiConfigDialog) {
        val ps = prefs.optJSONObject("proxy_selector") ?: JSONObject()
        CustomApiConfigDialog(
            enabled = ps.optBoolean("custom_api_enabled", false),
            endpoint = ps.optString("custom_api_endpoint", "http://127.0.0.1:9527"),
            secret = ps.optString("custom_api_secret", ""),
            onSave = { en, ep, sec ->
                dispatchAsync("update_custom_api") {
                    put("enabled", en)
                    put("endpoint", ep)
                    put("secret", sec)
                }
            },
            onDismiss = { showApiConfigDialog = false },
        )
    }
}

@Composable
private fun PanelOverviewContent(
    state: JSONObject,
    listState: androidx.compose.foundation.lazy.LazyListState,
    onJumpSegment: (Int) -> Unit,
) {
    val p = LocalBoxPalette.current
    val groupsCount = state.optInt("groups_count", 0)
    val rulesCount = state.optInt("rules_count", 0)
    val connsCount = state.optInt("active_connections_count", 0)
    val upBps = state.optLong("upload_bps", 0L)
    val downBps = state.optLong("download_bps", 0L)
    val totUp = state.optLong("total_upload", 0L)
    val totDown = state.optLong("total_download", 0L)
    val histArr = state.optJSONArray("net_speed_history") ?: JSONArray()
    val upSeries = remember(histArr.toString()) {
        (0 until histArr.length()).map { histArr.optJSONObject(it)?.optLong("upload_bps", 0L) ?: 0L }
    }
    val downSeries = remember(histArr.toString()) {
        (0 until histArr.length()).map { histArr.optJSONObject(it)?.optLong("download_bps", 0L) ?: 0L }
    }

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
        // 1. 运行概况 (策略 / 规则 / 连接)
        item {
            MiuixCard(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
                    Text("运行概况", fontSize = 17.sp, color = p.textPrimary)
                    Row(modifier = Modifier.fillMaxWidth()) {
                        Column(
                            modifier = Modifier.weight(1f).clickable { onJumpSegment(1) },
                            horizontalAlignment = Alignment.CenterHorizontally,
                            verticalArrangement = Arrangement.spacedBy(4.dp),
                        ) {
                            Text("$groupsCount", fontSize = 24.sp, fontWeight = FontWeight.Medium, color = p.textPrimary)
                            Text("策略", fontSize = 13.sp, color = p.textSecondary)
                        }
                        Column(
                            modifier = Modifier.weight(1f).clickable {
                                dispatchAsync("refresh_rules")
                                navigateTo("rules")
                            },
                            horizontalAlignment = Alignment.CenterHorizontally,
                            verticalArrangement = Arrangement.spacedBy(4.dp),
                        ) {
                            Text("$rulesCount", fontSize = 24.sp, fontWeight = FontWeight.Medium, color = p.textPrimary)
                            Text("规则", fontSize = 13.sp, color = p.textSecondary)
                        }
                        Column(
                            modifier = Modifier.weight(1f).clickable { onJumpSegment(3) },
                            horizontalAlignment = Alignment.CenterHorizontally,
                            verticalArrangement = Arrangement.spacedBy(4.dp),
                        ) {
                            Text("$connsCount", fontSize = 24.sp, fontWeight = FontWeight.Medium, color = p.textPrimary)
                            Text("连接", fontSize = 13.sp, color = p.textSecondary)
                        }
                    }
                }
            }
        }

        // 2. 订阅 card
        item {
            MiuixCard(
                modifier = Modifier.fillMaxWidth(),
                onClick = { onJumpSegment(2) },
            ) {
                Column(modifier = Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    Text("订阅", fontSize = 17.sp, color = p.textPrimary)
                    Text("刷新后显示数据", fontSize = 15.sp, color = p.textSecondary)
                }
            }
        }

        // 3. 2-col 上行速度 / 下行速度
        item {
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                MiuixCard(modifier = Modifier.weight(1f)) {
                    Row(
                        modifier = Modifier.padding(horizontal = 14.dp, vertical = 15.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Box(
                            modifier = Modifier.size(36.dp).clip(RoundedCornerShape(11.dp)).background(p.upGreenSoft),
                            contentAlignment = Alignment.Center,
                        ) {
                            ToolRowGlyph("arrow_up", p.upGreen, size = 18.dp)
                        }
                        Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                            Text("上行速度", fontSize = 12.sp, color = p.textSecondary, maxLines = 1)
                            Text(
                                formatRateCompact(upBps),
                                fontSize = 16.sp,
                                fontWeight = FontWeight.SemiBold,
                                color = p.upGreen,
                                maxLines = 1,
                                softWrap = false,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                    }
                }
                MiuixCard(modifier = Modifier.weight(1f)) {
                    Row(
                        modifier = Modifier.padding(horizontal = 14.dp, vertical = 15.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Box(
                            modifier = Modifier.size(36.dp).clip(RoundedCornerShape(11.dp)).background(p.accentSoft),
                            contentAlignment = Alignment.Center,
                        ) {
                            ToolRowGlyph("arrow_down", p.accent, size = 18.dp)
                        }
                        Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                            Text("下行速度", fontSize = 12.sp, color = p.textSecondary, maxLines = 1)
                            Text(
                                formatRateCompact(downBps),
                                fontSize = 16.sp,
                                fontWeight = FontWeight.SemiBold,
                                color = p.accent,
                                maxLines = 1,
                                softWrap = false,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                    }
                }
            }
        }

        // 4. 总流量
        item {
            MiuixCard(modifier = Modifier.fillMaxWidth()) {
                Row(
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 18.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text("总流量", fontSize = 17.sp, color = p.textPrimary, modifier = Modifier.weight(1f))
                    Text("上行 ${formatBytesCompact(totUp)}", fontSize = 15.sp, color = p.upGreen)
                    Text("  /  ", fontSize = 15.sp, color = p.textSecondary)
                    Text("下行 ${formatBytesCompact(totDown)}", fontSize = 15.sp, color = p.accent)
                }
            }
        }

        // 5. 近期趋势 (60s smooth Cubic Bézier area waveform)
        item {
            MiuixCard(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text("近期趋势", fontSize = 17.sp, color = p.textPrimary)
                            Text("最近 60 秒", fontSize = 13.sp, color = p.textSecondary)
                        }
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            Box(
                                modifier = Modifier
                                    .clip(RoundedCornerShape(50))
                                    .background(p.upGreenSoft)
                                    .padding(horizontal = 12.dp, vertical = 5.dp),
                            ) {
                                Text("● 上行", fontSize = 12.sp, color = p.upGreen, fontWeight = FontWeight.Medium)
                            }
                            Box(
                                modifier = Modifier
                                    .clip(RoundedCornerShape(50))
                                    .background(p.accentSoft)
                                    .padding(horizontal = 12.dp, vertical = 5.dp),
                            ) {
                                Text("● 下行", fontSize = 12.sp, color = p.accent, fontWeight = FontWeight.Medium)
                            }
                        }
                    }
                    TrafficTrendChart(
                        uploadSeries = upSeries,
                        downloadSeries = downSeries,
                        upColor = p.upGreen,
                        downColor = p.accent,
                        modifier = Modifier.fillMaxWidth().height(164.dp),
                    )
                }
            }
        }
    }
}

/**
 * 1:1 Replica of original `com.boxproxy.honk` Tools Tab (exact 7 grouped white 20dp cards).
 */
@Composable
fun ToolsTabScreen(
    state: JSONObject,
    onLaunchImportCore: () -> Unit,
) {
    val p = LocalBoxPalette.current
    val listState = rememberLazyListState()
    val collapsed by remember { derivedStateOf { listState.firstVisibleItemIndex > 0 || listState.firstVisibleItemScrollOffset > 60 } }
    var showWebUiUpdateDialog by remember { mutableStateOf(false) }

    val prefs = state.optJSONObject("prefs") ?: JSONObject()
    val webUiPrefs = prefs.optJSONObject("webui_update") ?: JSONObject()

    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixTabHeader(title = "工具", collapsed = collapsed)

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
            // Card 1: 网络测试
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixNavRow(
                        title = "网络测试",
                        icon = { ToolRowGlyph("pin", p.textPrimary) },
                        onClick = { navigateTo("network_test") },
                    )
                }
            }

            // Card 2: 文件管理 · 脚本
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixNavRow(
                        title = "文件管理",
                        icon = { ToolRowGlyph("folder", p.textPrimary) },
                        onClick = {
                            dispatchAsync("list_runtime_files") { put("dir", "") }
                            navigateTo("runtime_files")
                        },
                    )
                    MiuixInsetDivider()
                    MiuixNavRow(
                        title = "脚本",
                        icon = { ToolRowGlyph("script", p.textPrimary) },
                        onClick = {
                            dispatchAsync("list_shell_scripts")
                            dispatchAsync("list_js_scripts") { put("core", "mihomo") }
                            navigateTo("shell_scripts")
                        },
                    )
                }
            }

            // Card 3: 日志查看
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixNavRow(
                        title = "日志查看",
                        icon = { ToolRowGlyph("log", p.textPrimary) },
                        onClick = {
                            dispatchAsync("list_logs")
                            navigateTo("log_viewer")
                        },
                    )
                }
            }

            // Card 4: 应用管理
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixNavRow(
                        title = "应用管理",
                        icon = { ToolRowGlyph("apps", p.textPrimary) },
                        onClick = {
                            dispatchAsync("refresh_installed_apps")
                            navigateTo("app_management")
                        },
                    )
                }
            }

            // Card 5: 网络匹配 · 共享网络 · 绕过规则
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixNavRow(
                        title = "网络匹配",
                        icon = { ToolRowGlyph("wifi", p.textPrimary) },
                        onClick = { navigateTo("network_match") },
                    )
                    MiuixInsetDivider()
                    MiuixNavRow(
                        title = "共享网络",
                        icon = { ToolRowGlyph("hotspot", p.textPrimary) },
                        onClick = {
                            dispatchAsync("scan_arp_clients")
                            navigateTo("shared_network_config")
                        },
                    )
                    MiuixInsetDivider()
                    MiuixNavRow(
                        title = "绕过规则",
                        icon = { ToolRowGlyph("branch", p.textPrimary) },
                        onClick = { navigateTo("bypass_rules") },
                    )
                }
            }

            // Card 6: 订阅管理 · CNIP 设置
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixNavRow(
                        title = "订阅管理",
                        icon = { ToolRowGlyph("link", p.textPrimary) },
                        onClick = { navigateTo("subscription_management") },
                    )
                    MiuixInsetDivider()
                    MiuixNavRow(
                        title = "CNIP 设置",
                        icon = { ToolRowGlyph("pin", p.textPrimary) },
                        onClick = { navigateTo("cnip_settings") },
                    )
                }
            }

            // Card 7: 更新 WebUI · 更新核心 · 从文件安装核心 · Sub-Store
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixNavRow(
                        title = "更新 WebUI",
                        icon = { ToolRowGlyph("globe", p.textPrimary) },
                        onClick = { showWebUiUpdateDialog = true },
                    )
                    MiuixInsetDivider()
                    MiuixNavRow(
                        title = "更新核心",
                        icon = { ToolRowGlyph("download", p.textPrimary) },
                        onClick = { navigateTo("core_update_sources") },
                    )
                    MiuixInsetDivider()
                    MiuixNavRow(
                        title = "从文件安装核心",
                        icon = { ToolRowGlyph("folder", p.textPrimary) },
                        onClick = onLaunchImportCore,
                    )
                    MiuixInsetDivider()
                    MiuixNavRow(
                        title = "Sub-Store",
                        icon = { ToolRowGlyph("layers", p.textPrimary) },
                        onClick = {
                            dispatchAsync("refresh_sub_store")
                            navigateTo("sub_store")
                        },
                    )
                }
            }
        }
    }

    if (showWebUiUpdateDialog) {
        MiuixTextInputDialog(
            title = "更新本地 WebUI (M39)",
            subtitle = "目标目录: ${webUiPrefs.strOr("target_path", "mihomo/dashboard")}",
            initialValue = webUiPrefs.strOr("download_url", "https://github.com/Zephyruso/zashboard/releases/latest/download/dist.zip"),
            placeholder = "https://.../dist.zip",
            confirmLabel = "下载并解压",
            onConfirm = { url ->
                if (url.isNotEmpty()) {
                    dispatchAsync("update_local_web_ui") {
                        put("target_path", webUiPrefs.strOr("target_path", "mihomo/dashboard"))
                        put("download_url", url)
                    }
                }
            },
            onDismiss = { showWebUiUpdateDialog = false },
        )
    }
}

@Composable
fun PanelQuickContent(
    state: JSONObject,
    onRouteSelected: (() -> Unit)? = null,
) {
    val p = LocalBoxPalette.current
    val go: (String, String?) -> Unit = { route, refreshAction ->
        if (refreshAction != null) dispatchAsync(refreshAction)
        navigateTo(route)
        onRouteSelected?.invoke()
    }
    Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
        Text("快捷面板", fontSize = 18.sp, fontWeight = FontWeight.Medium, color = p.textPrimary)
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            QuickSheetPill("Doona WebUI", true, Modifier.weight(1f)) { go("web_ui", null) }
            QuickSheetPill("概览", false, Modifier.weight(1f)) { go("overview", "refresh_overview") }
            QuickSheetPill("策略选择", false, Modifier.weight(1f)) { go("proxy_selector", "refresh_proxies") }
        }
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            QuickSheetPill("实时连接", false, Modifier.weight(1f)) { go("connections_details", "refresh_connections") }
            QuickSheetPill("规则追踪", false, Modifier.weight(1f)) { go("rules", "refresh_rules") }
            QuickSheetPill("订阅详情", false, Modifier.weight(1f)) { go("subscription_details", "refresh_subscriptions") }
        }
        Text(
            "面板地址: ${state.strOr("webui_resolved_url", "http://127.0.0.1:9527/ui/")}",
            fontSize = 12.sp,
            color = p.textSecondary,
        )
    }
}

@Composable
private fun QuickSheetPill(label: String, primary: Boolean, modifier: Modifier, onClick: () -> Unit) {
    val p = LocalBoxPalette.current
    Box(
        modifier = modifier
            .height(42.dp)
            .clip(RoundedCornerShape(14.dp))
            .background(if (primary) p.accentSoft else p.background)
            .clickable(onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = label,
            fontSize = 13.sp,
            fontWeight = if (primary) FontWeight.Medium else FontWeight.Normal,
            color = if (primary) p.accent else p.textPrimary,
            maxLines = 1,
        )
    }
}

@Composable
private fun CollapsibleGlassWebCapsule(
    label: String,
    expanded: Boolean,
    onBack: () -> Unit,
    onToggleExpand: () -> Unit,
    onReload: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val p = LocalBoxPalette.current
    val stripBg = if (p.isDark) {
        Color(0xFF10131A).copy(alpha = if (p.enableBlur) 0.88f else 0.96f)
    } else {
        Color(0xFFF6F7FA).copy(alpha = if (p.enableBlur) 0.90f else 0.98f)
    }
    val pillBg = if (p.isDark) Color.White.copy(alpha = 0.08f) else Color.Black.copy(alpha = 0.05f)
    val borderCol = if (p.isDark) Color.White.copy(alpha = 0.12f) else p.divider
    Column(modifier = modifier.fillMaxWidth().background(stripBg)) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .height(38.dp)
                .padding(horizontal = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Box(
                modifier = Modifier
                    .size(34.dp)
                    .iosPressScale(pressedScale = 0.9f, onClick = onBack)
                    .clip(CircleShape),
                contentAlignment = Alignment.Center,
            ) {
                ToolRowGlyph("back", p.textPrimary, size = 18.dp)
            }

            Row(
                modifier = Modifier
                    .height(28.dp)
                    .iosPressScale(pressedScale = 0.96f, onClick = onToggleExpand)
                    .clip(CircleShape)
                    .background(if (expanded) p.accentSoft else pillBg)
                    .border(0.7.dp, if (expanded) p.accent.copy(alpha = 0.4f) else borderCol, CircleShape)
                    .padding(horizontal = 13.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(5.dp),
            ) {
                Text(
                    text = label,
                    fontSize = 12.sp,
                    fontWeight = FontWeight.Medium,
                    color = if (expanded) p.accent else p.textPrimary,
                    maxLines = 1,
                )
                Text(
                    text = if (expanded) "▴" else "▾",
                    fontSize = 11.sp,
                    color = p.accent,
                )
            }

            Box(
                modifier = Modifier
                    .size(34.dp)
                    .iosPressScale(pressedScale = 0.9f, onClick = onReload)
                    .clip(CircleShape),
                contentAlignment = Alignment.Center,
            ) {
                ToolRowGlyph("refresh", p.textSecondary, size = 16.dp)
            }
        }
        if (!expanded) {
            HorizontalDivider(thickness = 0.5.dp, color = p.divider.copy(alpha = 0.55f))
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun WebUiRouteScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val ctx = LocalContext.current
    val prefs = state.optJSONObject("prefs") ?: JSONObject()
    val webPanel = prefs.optJSONObject("web_panel") ?: JSONObject()
    val selected = webPanel.strOr("webui_selected_panel", "local")
    val localMode = webPanel.strOr("webui_local_panel_mode", "full")
    val customArr = webPanel.optJSONArray("webui_custom_panels") ?: JSONArray()
    val url = state.strOr("webui_resolved_url", "http://127.0.0.1:9527/ui/")
    val token = state.optString("webui_one_time_token", "")

    var topExpanded by remember { mutableStateOf(false) }
    var reloadNonce by remember { mutableIntStateOf(0) }
    var showAddCustomDialog by remember { mutableStateOf(false) }

    val panelItems = remember(selected, localMode, customArr.toString()) {
        buildList {
            add(Triple("local_full", "本地 Doona", selected == "local" && localMode != "lite"))
            add(Triple("local_lite", "Doona Lite", selected == "local" && localMode == "lite"))
            add(Triple("zashboard", "Zashboard", selected == "zashboard"))
            add(Triple("metacubexd", "MetaCubeXD", selected == "metacubexd"))
            add(Triple("sing_box", "Sing-Box", selected == "sing_box"))
            for (i in 0 until customArr.length()) {
                val obj = customArr.optJSONObject(i) ?: continue
                val cid = obj.optString("id", "").trim()
                val cname = obj.optString("name", cid).trim()
                if (cid.isNotEmpty()) {
                    add(Triple("custom:$cid", cname, selected == cid))
                }
            }
        }
    }
    val activeLabel = panelItems.firstOrNull { it.third }?.second ?: "本地 Doona"

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(p.background)
            .statusBarsPadding()
            .navigationBarsPadding(),
    ) {
        CollapsibleGlassWebCapsule(
            label = activeLabel,
            expanded = topExpanded,
            onBack = { dispatchAsync("pop_back") },
            onToggleExpand = { topExpanded = !topExpanded },
            onReload = { reloadNonce++ },
        )

        // Collapsible iOS Glass Panel Switcher Drawer (collapsed by default; zero overlap with WebView!)
        AnimatedVisibility(
            visible = topExpanded,
            enter = expandVertically(tween(240, easing = IosNavEasing)) + fadeIn(tween(180)),
            exit = shrinkVertically(tween(200, easing = IosNavEasing)) + fadeOut(tween(140)),
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(p.card.copy(alpha = 0.95f)),
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .horizontalScroll(rememberScrollState())
                        .padding(start = 12.dp, end = 12.dp, bottom = 8.dp, top = 8.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    panelItems.forEach { (key, label, active) ->
                        Box(
                            modifier = Modifier
                                .clip(RoundedCornerShape(50))
                                .background(if (active) p.accentSoft else p.background)
                                .clickable {
                                    when {
                                        key == "local_full" -> {
                                            dispatchAsync("select_web_panel") { put("panel_id", "local") }
                                            dispatchAsync("set_web_ui_local_mode") { put("mode", "full") }
                                        }
                                        key == "local_lite" -> {
                                            dispatchAsync("select_web_panel") { put("panel_id", "local") }
                                            dispatchAsync("set_web_ui_local_mode") { put("mode", "lite") }
                                        }
                                        key.startsWith("custom:") -> {
                                            val cid = key.removePrefix("custom:")
                                            dispatchAsync("select_web_panel") { put("panel_id", cid) }
                                        }
                                        else -> {
                                            dispatchAsync("select_web_panel") { put("panel_id", key) }
                                        }
                                    }
                                    topExpanded = false
                                }
                                .padding(horizontal = 13.dp, vertical = 6.dp),
                        ) {
                            Text(
                                text = label,
                                fontSize = 12.sp,
                                color = if (active) p.accent else p.textPrimary,
                                fontWeight = if (active) FontWeight.SemiBold else FontWeight.Normal,
                                maxLines = 1,
                            )
                        }
                    }

                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(50))
                            .background(p.background)
                            .clickable { showAddCustomDialog = true }
                            .padding(horizontal = 11.dp, vertical = 6.dp),
                    ) {
                        Text("+ 面板", fontSize = 12.sp, color = p.textSecondary, maxLines = 1)
                    }
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(50))
                            .background(p.background)
                            .clickable {
                                runCatching {
                                    ctx.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)))
                                }
                            }
                            .padding(horizontal = 11.dp, vertical = 6.dp),
                    ) {
                        Text("浏览器 ↗", fontSize = 12.sp, color = p.textSecondary, maxLines = 1)
                    }
                }
                HorizontalDivider(thickness = 0.5.dp, color = p.divider.copy(alpha = 0.7f))
            }
        }

        Box(modifier = Modifier.weight(1f).fillMaxWidth()) {
            BoxWebViewHost(
                url = url,
                oneTimeToken = token,
                reloadNonce = reloadNonce,
                onScrollCollapse = { if (topExpanded) topExpanded = false },
                modifier = Modifier.fillMaxSize(),
            )
        }
    }

    if (showAddCustomDialog) {
        MiuixTextInputDialog(
            title = "添加自定义面板 URL",
            subtitle = "输入任意本地或远程 WebUI 地址（例如 http://127.0.0.1:9090/ui/）",
            initialValue = "http://127.0.0.1:9527/ui/",
            placeholder = "https://...",
            confirmLabel = "添加并切换",
            onConfirm = { customUrl ->
                if (customUrl.isNotBlank()) {
                    val id = "p_${System.currentTimeMillis()}"
                    val host = runCatching { Uri.parse(customUrl).host ?: "自定义" }.getOrDefault("自定义")
                    dispatchAsync("add_custom_web_panel") {
                        put("is_substore", false)
                        put(
                            "panel",
                            JSONObject()
                                .put("id", id)
                                .put("name", host)
                                .put("url", customUrl.trim()),
                        )
                    }
                    dispatchAsync("select_web_panel") { put("panel_id", id) }
                }
            },
            onDismiss = { showAddCustomDialog = false },
        )
    }
}

@Composable
fun SubStoreRouteScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val ctx = LocalContext.current
    val env = state.optJSONObject("substore_env") ?: JSONObject()
    val url = env.strOr("resolved_url", "http://127.0.0.1:3001/")
    var topExpanded by remember { mutableStateOf(false) }
    var reloadNonce by remember { mutableIntStateOf(0) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(p.background)
            .statusBarsPadding()
            .navigationBarsPadding(),
    ) {
        CollapsibleGlassWebCapsule(
            label = "Sub-Store",
            expanded = topExpanded,
            onBack = { dispatchAsync("pop_back") },
            onToggleExpand = { topExpanded = !topExpanded },
            onReload = { reloadNonce++ },
        )

        AnimatedVisibility(
            visible = topExpanded,
            enter = expandVertically(tween(240, easing = IosNavEasing)) + fadeIn(tween(180)),
            exit = shrinkVertically(tween(200, easing = IosNavEasing)) + fadeOut(tween(140)),
        ) {
            Column(modifier = Modifier.fillMaxWidth().background(p.card.copy(alpha = 0.95f))) {
                Row(
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Text(
                        url,
                        fontSize = 12.sp,
                        color = p.textSecondary,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(1f),
                    )
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(50))
                            .background(p.background)
                            .clickable {
                                runCatching { ctx.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url))) }
                            }
                            .padding(horizontal = 11.dp, vertical = 6.dp),
                    ) {
                        Text("浏览器 ↗", fontSize = 12.sp, color = p.textSecondary)
                    }
                }
                HorizontalDivider(thickness = 0.5.dp, color = p.divider.copy(alpha = 0.7f))
            }
        }

        Box(modifier = Modifier.weight(1f).fillMaxWidth()) {
            BoxWebViewHost(
                url = url,
                oneTimeToken = "",
                reloadNonce = reloadNonce,
                onScrollCollapse = { if (topExpanded) topExpanded = false },
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}

@Composable
fun OnboardingScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val rootGranted = state.optBoolean("root_granted", false)
    Column(
        modifier = Modifier.fillMaxSize().background(p.background).padding(24.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        BoxCatAboutLogo(color = p.accent, size = 104.dp)
        Spacer(Modifier.height(16.dp))
        Text("BoxProxy Honk", fontSize = 28.sp, fontWeight = FontWeight.Bold, color = p.textPrimary)
        Spacer(Modifier.height(8.dp))
        Text(
            "纯 Rust MVI 引擎 (libboxproxy_app.so) · 58 模块 · box.db v15",
            fontSize = 14.sp,
            color = p.textSecondary,
            textAlign = TextAlign.Center,
        )
        Spacer(Modifier.height(18.dp))
        MiuixCard(modifier = Modifier.fillMaxWidth()) {
            Column(modifier = Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(
                    if (rootGranted) "Root 权限已就绪 (uid=0)" else "点击下方按钮检测 Root 权限并初始化目录",
                    fontSize = 15.sp,
                    color = if (rootGranted) p.upGreen else p.textPrimary,
                )
            }
        }
        Spacer(Modifier.height(20.dp))
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Box(
                modifier = Modifier
                    .weight(1f)
                    .height(50.dp)
                    .clip(RoundedCornerShape(16.dp))
                    .background(p.card)
                    .clickable { dispatchAsync("refresh_status") },
                contentAlignment = Alignment.Center,
            ) {
                Text("检测 Root", fontSize = 16.sp, color = p.textPrimary)
            }
            Box(
                modifier = Modifier
                    .weight(1f)
                    .height(50.dp)
                    .clip(RoundedCornerShape(16.dp))
                    .background(p.accentSoft)
                    .clickable { dispatchAsync("complete_onboarding") },
                contentAlignment = Alignment.Center,
            ) {
                Text("开始使用", fontSize = 16.sp, fontWeight = FontWeight.Medium, color = p.accent)
            }
        }
    }
}

@Composable
private fun CustomApiConfigDialog(
    enabled: Boolean,
    endpoint: String,
    secret: String,
    onSave: (Boolean, String, String) -> Unit,
    onDismiss: () -> Unit,
) {
    val p = LocalBoxPalette.current
    var en by remember { mutableStateOf(enabled) }
    var ep by remember { mutableStateOf(endpoint) }
    var sec by remember { mutableStateOf(secret) }
    IosBottomSheetDialog(onDismiss = onDismiss) {
        Column(
            modifier = Modifier.fillMaxWidth(),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text("自定义 API 配置", fontSize = 19.sp, fontWeight = FontWeight.SemiBold, color = p.textPrimary, textAlign = TextAlign.Center, modifier = Modifier.fillMaxWidth())
            MiuixSettingSwitchRow("启用自定义 API 覆盖", en) { en = it }
            OutlinedTextField(value = ep, onValueChange = { ep = it }, label = { Text("API Endpoint") }, singleLine = true, shape = RoundedCornerShape(14.dp), modifier = Modifier.fillMaxWidth())
            OutlinedTextField(value = sec, onValueChange = { sec = it }, label = { Text("Secret") }, singleLine = true, shape = RoundedCornerShape(14.dp), modifier = Modifier.fillMaxWidth())
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Box(
                    modifier = Modifier.weight(1f).height(46.dp).clip(RoundedCornerShape(14.dp)).background(p.background).clickable(onClick = onDismiss),
                    contentAlignment = Alignment.Center,
                ) { Text("取消", color = p.textSecondary) }
                Box(
                    modifier = Modifier.weight(1f).height(46.dp).clip(RoundedCornerShape(14.dp)).background(p.accentSoft).clickable {
                        onSave(en, ep.trim(), sec.trim())
                        onDismiss()
                    },
                    contentAlignment = Alignment.Center,
                ) { Text("保存", color = p.accent, fontWeight = FontWeight.SemiBold) }
            }
        }
    }
}
