package io.honk.android.ui

import android.graphics.ColorMatrix
import android.graphics.ColorMatrixColorFilter
import android.graphics.RenderEffect
import android.graphics.RuntimeShader
import android.graphics.Shader
import android.os.Build
import androidx.annotation.RequiresApi
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import kotlinx.coroutines.channels.Channel
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.BlendMode
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ShaderBrush
import androidx.compose.ui.graphics.asComposeRenderEffect
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.layer.GraphicsLayer
import androidx.compose.ui.graphics.layer.drawLayer
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.positionInRoot
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import io.honk.android.bridge.RustBridge
import org.json.JSONObject
import kotlin.math.PI

/**
 * Pure-Rust-backed optical shader & iOS motion preset bundle loaded once from `libboxproxy_app.so`.
 */
internal data class RustLiquidGlassConfig(
    val refractionAgsl: String,
    val dispersionAgsl: String,
    val defaultHighlightAgsl: String,
    val ambientHighlightAgsl: String,
    val vibrancyMatrix: FloatArray,
    val navPushMs: Int,
    val navPopMs: Int,
    val navParallaxRatio: Float,
    val tabSwitchMs: Int,
    val sheetEnterMs: Int,
    val sheetExitMs: Int,
    val pressScale: Float,
)

internal object LiquidGlassRuntime {
    val config: RustLiquidGlassConfig by lazy {
        val raw = runCatching { RustBridge.nativeLiquidGlassShadersJson() }.getOrDefault("{}")
        val obj = runCatching { JSONObject(raw) }.getOrDefault(JSONObject())
        val matArr = obj.optJSONArray("vibrancy_matrix")
        val matrix = FloatArray(20) { i ->
            matArr?.optDouble(i, if (i % 6 == 0) 1.0 else 0.0)?.toFloat()
                ?: if (i % 6 == 0) 1.0f else 0.0f
        }
        val motion = obj.optJSONObject("ios_motion") ?: JSONObject()
        RustLiquidGlassConfig(
            refractionAgsl = obj.optString("refraction", ""),
            dispersionAgsl = obj.optString("refraction_dispersion", ""),
            defaultHighlightAgsl = obj.optString("default_highlight", ""),
            ambientHighlightAgsl = obj.optString("ambient_highlight", ""),
            vibrancyMatrix = matrix,
            navPushMs = motion.optInt("nav_push_ms", 340),
            navPopMs = motion.optInt("nav_pop_ms", 300),
            navParallaxRatio = motion.optDouble("nav_parallax_ratio", 0.28).toFloat(),
            tabSwitchMs = motion.optInt("tab_switch_ms", 230),
            sheetEnterMs = motion.optInt("sheet_enter_ms", 280),
            sheetExitMs = motion.optInt("sheet_exit_ms", 180),
            pressScale = motion.optDouble("press_scale", 0.975).toFloat(),
        )
    }
}

@RequiresApi(Build.VERSION_CODES.TIRAMISU)
private fun buildBarRenderEffect(
    cfg: RustLiquidGlassConfig,
    shader: RuntimeShader,
    widthPx: Float,
    heightPx: Float,
    blurPx: Float,
    refractionHeightPx: Float,
    refractionAmountPx: Float,
): androidx.compose.ui.graphics.RenderEffect {
    val vibrancy = RenderEffect.createColorFilterEffect(
        ColorMatrixColorFilter(ColorMatrix(cfg.vibrancyMatrix)),
    )
    val blurred = RenderEffect.createBlurEffect(
        blurPx.coerceAtLeast(1f),
        blurPx.coerceAtLeast(1f),
        vibrancy,
        Shader.TileMode.CLAMP,
    )
    val r = (minOf(widthPx, heightPx) * 0.5f).coerceAtLeast(1f)
    shader.setFloatUniform("size", widthPx.coerceAtLeast(1f), heightPx.coerceAtLeast(1f))
    shader.setFloatUniform("offset", 0f, 0f)
    shader.setFloatUniform("cornerRadii", floatArrayOf(r, r, r, r))
    shader.setFloatUniform("refractionHeight", refractionHeightPx.coerceAtLeast(1f))
    shader.setFloatUniform("refractionAmount", -refractionAmountPx)
    shader.setFloatUniform("depthEffect", 0.2f)
    val lens = RenderEffect.createRuntimeShaderEffect(shader, "content")
    return RenderEffect.createChainEffect(lens, blurred).asComposeRenderEffect()
}

@RequiresApi(Build.VERSION_CODES.TIRAMISU)
private fun buildDropletRenderEffect(
    shader: RuntimeShader,
    widthPx: Float,
    heightPx: Float,
    refractionHeightPx: Float,
    refractionAmountPx: Float,
    chromaticStrength: Float,
): androidx.compose.ui.graphics.RenderEffect {
    val r = (minOf(widthPx, heightPx) * 0.5f).coerceAtLeast(1f)
    shader.setFloatUniform("size", widthPx.coerceAtLeast(1f), heightPx.coerceAtLeast(1f))
    shader.setFloatUniform("offset", 0f, 0f)
    shader.setFloatUniform("cornerRadii", floatArrayOf(r, r, r, r))
    shader.setFloatUniform("refractionHeight", refractionHeightPx.coerceAtLeast(1f))
    shader.setFloatUniform("refractionAmount", -refractionAmountPx)
    shader.setFloatUniform("depthEffect", 0.35f)
    shader.setFloatUniform("chromaticAberration", chromaticStrength)
    return RenderEffect.createRuntimeShaderEffect(shader, "content").asComposeRenderEffect()
}

/**
 * Floating Liquid Glass Bottom Tab Bar inspired by `@Kyant0/AndroidLiquidGlass` (`LiquidBottomTabs`),
 * driven frame-by-frame by our pure-Rust `liquid-glass` spring physics & AGSL refraction shaders.
 */
@Composable
fun LiquidGlassBottomBar(
    activeTab: Int,
    showPanelTab: Boolean,
    backdropLayer: GraphicsLayer,
    onSelect: (Int) -> Unit,
    modifier: Modifier = Modifier,
    barStyle: String = "LiquidGlass",
    enableBlur: Boolean = true,
) {
    val p = LocalBoxPalette.current
    val lang = LocalAppLang.current
    val density = LocalDensity.current
    val haptic = LocalHapticFeedback.current
    val cfg = remember { LiquidGlassRuntime.config }
    val isLiquid = barStyle.equals("LiquidGlass", ignoreCase = true)
    val isDockedGlass = barStyle.equals("Glass", ignoreCase = true)

    val items = remember(showPanelTab, lang) {
        buildList {
            add(Triple(0, uiTr(lang, "首页", "首頁", "Home", "Главная"), "home"))
            if (showPanelTab) add(Triple(1, uiTr(lang, "面板", "面板", "Panel", "Панель"), "panel"))
            add(Triple(2, uiTr(lang, "工具", "工具", "Tools", "Утилиты"), "tools"))
            add(Triple(3, uiTr(lang, "设置", "設定", "Settings", "Настройки"), "settings"))
        }
    }
    val tabsCount = items.size
    val visualSelectedIndex = remember(activeTab, items) {
        items.indexOfFirst { it.first == activeTab }.coerceAtLeast(0)
    }

    val frameBuf = remember { FloatArray(15) }
    val physicsHandle = remember {
        val h = RustBridge.nativeLiquidTabCreate(tabsCount, visualSelectedIndex)
        RustBridge.nativeLiquidTabStep(h, 0f, 900f, density.density, frameBuf)
        h
    }
    DisposableEffect(physicsHandle) {
        onDispose { RustBridge.nativeLiquidTabDestroy(physicsHandle) }
    }

    val wakeChannel = remember { Channel<Unit>(Channel.CONFLATED) }
    var frameTick by remember { mutableLongStateOf(0L) }
    var barWidthPx by remember { mutableStateOf(900f) }
    var barRootOffset by remember { mutableStateOf(Offset.Zero) }
    var lastHapticIdx by remember { mutableIntStateOf(visualSelectedIndex) }

    LaunchedEffect(tabsCount, visualSelectedIndex) {
        RustBridge.nativeLiquidTabEvent(physicsHandle, 0, tabsCount.toFloat(), visualSelectedIndex.toFloat())
        RustBridge.nativeLiquidTabEvent(physicsHandle, 2, visualSelectedIndex.toFloat(), 0f)
        lastHapticIdx = visualSelectedIndex
        wakeChannel.trySend(Unit)
    }

    // Single resident 120Hz VSync physics stepper: zero coroutine churn on 240Hz/360Hz touch drag,
    // exact delta-time pacing, and automatic 0% CPU sleep once spring converges!
    LaunchedEffect(physicsHandle) {
        wakeChannel.trySend(Unit)
        for (unused in wakeChannel) {
            var lastNanos = 0L
            while (true) {
                val settled = withFrameNanos { now ->
                    val dt = if (lastNanos == 0L) 1f / 120f else ((now - lastNanos) / 1_000_000_000f).coerceIn(0.001f, 0.05f)
                    lastNanos = now
                    val done = RustBridge.nativeLiquidTabStep(
                        physicsHandle,
                        dt,
                        barWidthPx,
                        density.density,
                        frameBuf,
                    )
                    frameTick++
                    done
                }
                if (settled) break
            }
        }
    }

    // Only recomposes tab labels when the active index actually crosses a midpoint (never 120fps)!
    val activeVisualIdx by remember(tabsCount) {
        derivedStateOf {
            @Suppress("UNUSED_VARIABLE")
            val t = frameTick
            kotlin.math.round(frameBuf[0].coerceIn(0f, (tabsCount - 1).coerceAtLeast(0).toFloat())).toInt()
        }
    }
    val defaultBarRefractH = with(density) { 20.dp.toPx() }
    val defaultBarRefractAmt = with(density) { 22.dp.toPx() }
    val defaultDropRefractH = with(density) { 6.dp.toPx() }
    val defaultDropRefractAmt = with(density) { 8.dp.toPx() }

    // Pre-compile AGSL RuntimeShaders once on Android 13+ (API 33+).
    val barRefractionShader = remember(cfg.refractionAgsl) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU && cfg.refractionAgsl.isNotBlank()) {
            runCatching { RuntimeShader(cfg.refractionAgsl) }.getOrNull()
        } else null
    }
    val dropletDispersionShader = remember(cfg.dispersionAgsl) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU && cfg.dispersionAgsl.isNotBlank()) {
            runCatching { RuntimeShader(cfg.dispersionAgsl) }.getOrNull()
        } else null
    }
    val highlightShader = remember(cfg.defaultHighlightAgsl) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU && cfg.defaultHighlightAgsl.isNotBlank()) {
            runCatching { RuntimeShader(cfg.defaultHighlightAgsl) }.getOrNull()
        } else null
    }

    val barHeightDp = 64.dp
    val dropletHeightDp = 56.dp
    val innerPadDp = 4.dp
    val barShape = if (isDockedGlass) androidx.compose.foundation.shape.RoundedCornerShape(topStart = 24.dp, topEnd = 24.dp) else CircleShape
    val containerTint = when {
        !enableBlur || barStyle.equals("Floating", ignoreCase = true) -> {
            if (p.isDark) Color(0xFF191D26).copy(alpha = 0.96f) else Color(0xFFFFFFFF).copy(alpha = 0.96f)
        }
        p.isDark -> Color(0xFF171A23).copy(alpha = if (isDockedGlass) 0.78f else 0.64f)
        else -> Color(0xFFFCFDFF).copy(alpha = if (isDockedGlass) 0.82f else 0.68f)
    }
    val rimLightBase = if (p.isDark) 0.34f else 0.88f
    val rimBottomColor = if (p.isDark) {
        Color.White.copy(alpha = 0.08f)
    } else {
        Color(0xFF1A1C1E).copy(alpha = 0.10f)
    }
    val dropletBaseAlpha = if (p.isDark) 0.22f else 0.13f

    Box(
        modifier = modifier
            .fillMaxWidth()
            .navigationBarsPadding()
            .padding(
                horizontal = if (isDockedGlass) 0.dp else 16.dp,
                vertical = if (isDockedGlass) 0.dp else 10.dp,
            ),
        contentAlignment = Alignment.Center,
    ) {
        BoxWithConstraints(
            modifier = Modifier
                .fillMaxWidth()
                .height(barHeightDp)
                .onGloballyPositioned { coords ->
                    barWidthPx = coords.size.width.toFloat().coerceAtLeast(120f)
                    barRootOffset = coords.positionInRoot()
                }
                .graphicsLayer {
                    @Suppress("UNUSED_VARIABLE")
                    val t = frameTick
                    val pOffsetX = frameBuf[2]
                    val pScale = if (frameBuf[3] > 0.1f) frameBuf[3] else 1f
                    translationX = if (isDockedGlass) 0f else pOffsetX
                    scaleX = if (isDockedGlass) 1f else pScale
                    scaleY = if (isDockedGlass) 1f else pScale
                }
                .shadow(
                    elevation = if (isDockedGlass) 8.dp else if (p.isDark) 18.dp else 14.dp,
                    shape = barShape,
                    ambientColor = Color.Black.copy(alpha = 0.08f),
                    spotColor = Color.Black.copy(alpha = 0.14f),
                ),
            contentAlignment = Alignment.CenterStart,
        ) {
            val totalWidthPx = with(density) { maxWidth.toPx() }
            val barHeightPx = with(density) { barHeightDp.toPx() }
            val innerPadPx = with(density) { innerPadDp.toPx() }
            val tabWidthPx = ((totalWidthPx - innerPadPx * 2f) / tabsCount).coerceAtLeast(1f)
            val tabWidthDp = with(density) { tabWidthPx.toDp() }
            val blurPx = with(density) { 10.dp.toPx() }

            // Cache static bar RenderEffect once per layout size (0 allocations per VSync frame!).
            val cachedBarRenderEffect = remember(totalWidthPx, barHeightPx, blurPx, isLiquid, barRefractionShader, cfg) {
                when {
                    Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU && isLiquid && barRefractionShader != null -> {
                        buildBarRenderEffect(
                            cfg = cfg,
                            shader = barRefractionShader,
                            widthPx = totalWidthPx,
                            heightPx = barHeightPx,
                            blurPx = blurPx,
                            refractionHeightPx = defaultBarRefractH,
                            refractionAmountPx = defaultBarRefractAmt,
                        )
                    }
                    Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
                        RenderEffect.createBlurEffect(
                            blurPx,
                            blurPx,
                            Shader.TileMode.CLAMP,
                        ).asComposeRenderEffect()
                    }
                    else -> null
                }
            }

            // Layer 1: Live GPU-refracted / frosted backdrop inside the bar (only on API 31+ where blur is hardware-supported).
            if (enableBlur && (isLiquid || isDockedGlass) && cachedBarRenderEffect != null) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .clip(barShape)
                        .graphicsLayer {
                            renderEffect = cachedBarRenderEffect
                        }
                        .drawBehind {
                            clipRect {
                                translate(left = -barRootOffset.x, top = -barRootOffset.y) {
                                    drawLayer(backdropLayer)
                                }
                            }
                        },
                )
            }

            // Layer 2: Translucent glass veil + interactive radial sheen + Kyant0 AGSL Fresnel rim.
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .clip(barShape)
                    .background(containerTint)
                    .drawWithContent {
                        @Suppress("UNUSED_VARIABLE")
                        val t = frameTick
                        val pressProg = frameBuf[1].coerceIn(0f, 1f)
                        val hlAlpha = if (frameBuf[10] > 0f) frameBuf[10] else 0.6f
                        val hlCenterFrac = frameBuf[14].coerceIn(0f, 1f)
                        val rimLightColor = Color.White.copy(alpha = rimLightBase * hlAlpha)
                        drawContent()
                        // Subtle interactive specular glow following the gliding droplet.
                        val cx = size.width * hlCenterFrac
                        drawRect(
                            brush = Brush.radialGradient(
                                colors = listOf(
                                    (if (p.isDark) Color.White else p.accent).copy(alpha = 0.10f + 0.08f * pressProg),
                                    Color.Transparent,
                                ),
                                center = Offset(cx, size.height * 0.45f),
                                radius = size.height * 1.65f,
                            ),
                            blendMode = BlendMode.Plus,
                        )
                        // Top meniscus light sheen.
                        drawRect(
                            brush = Brush.verticalGradient(
                                colors = listOf(
                                    Color.White.copy(alpha = if (p.isDark) 0.10f else 0.32f),
                                    Color.Transparent,
                                ),
                                startY = 0f,
                                endY = size.height * 0.48f,
                            ),
                        )
                        // Directional Fresnel edge highlight via Rust AGSL shader (API 33+) or gradient stroke fallback.
                        val strokeW = 1.1.dp.toPx()
                        val cornerR = if (isDockedGlass) 24.dp.toPx() else size.height * 0.5f
                        if (!isDockedGlass && Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU && highlightShader != null) {
                            highlightShader.setFloatUniform("size", size.width, size.height)
                            highlightShader.setFloatUniform(
                                "cornerRadii",
                                floatArrayOf(cornerR, cornerR, cornerR, cornerR),
                            )
                            highlightShader.setColorUniform("color", rimLightColor.toArgb())
                            highlightShader.setFloatUniform("angle", (45.0 * PI / 180.0).toFloat())
                            highlightShader.setFloatUniform("falloff", 1.35f)
                            drawRoundRect(
                                brush = ShaderBrush(highlightShader),
                                topLeft = Offset(strokeW * 0.5f, strokeW * 0.5f),
                                size = Size(size.width - strokeW, size.height - strokeW),
                                cornerRadius = CornerRadius(cornerR, cornerR),
                                style = Stroke(width = strokeW),
                                blendMode = BlendMode.Plus,
                            )
                        } else {
                            drawRoundRect(
                                brush = Brush.linearGradient(
                                    colors = listOf(rimLightColor, rimBottomColor, rimLightColor.copy(alpha = 0.35f)),
                                    start = Offset.Zero,
                                    end = Offset(size.width, size.height),
                                ),
                                topLeft = Offset(strokeW * 0.5f, strokeW * 0.5f),
                                size = Size(size.width - strokeW, size.height - strokeW),
                                cornerRadius = CornerRadius(cornerR, cornerR),
                                style = Stroke(width = strokeW),
                            )
                        }
                    },
            )

            // Layer 3: Gliding Liquid Glass Droplet Pill with 7-band chromatic dispersion & velocity stretch.
            val dropletWidthPx = tabWidthPx
            val dropletHeightPx = with(density) { dropletHeightDp.toPx() }
            val maxTabIdxF = (tabsCount - 1).coerceAtLeast(0).toFloat()
            Box(
                modifier = Modifier
                    .graphicsLayer {
                        @Suppress("UNUSED_VARIABLE")
                        val t = frameTick
                        val curVal = frameBuf[0].coerceIn(0f, maxTabIdxF)
                        val dScaleX = if (frameBuf[4] > 0.1f) frameBuf[4] else 1f
                        val dScaleY = if (frameBuf[5] > 0.1f) frameBuf[5] else 1f
                        translationX = innerPadPx + curVal * tabWidthPx
                        scaleX = dScaleX
                        scaleY = dScaleY
                    }
                    .width(tabWidthDp)
                    .height(dropletHeightDp)
                    .clip(CircleShape)
                    .then(
                        if (isLiquid && enableBlur && Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU && dropletDispersionShader != null) {
                            Modifier
                                .graphicsLayer {
                                    @Suppress("UNUSED_VARIABLE")
                                    val t = frameTick
                                    val pressProg = frameBuf[1].coerceIn(0f, 1f)
                                    val dRefH = if (frameBuf[6] > 0.1f) frameBuf[6] else defaultDropRefractH
                                    val dRefAmt = if (frameBuf[7] > 0.1f) frameBuf[7] else defaultDropRefractAmt
                                    renderEffect = buildDropletRenderEffect(
                                        shader = dropletDispersionShader,
                                        widthPx = dropletWidthPx,
                                        heightPx = dropletHeightPx,
                                        refractionHeightPx = dRefH,
                                        refractionAmountPx = dRefAmt,
                                        chromaticStrength = 0.65f + 0.35f * pressProg,
                                    )
                                }
                                .drawBehind {
                                    @Suppress("UNUSED_VARIABLE")
                                    val t = frameTick
                                    val curVal = frameBuf[0].coerceIn(0f, maxTabIdxF)
                                    val curTx = innerPadPx + curVal * tabWidthPx
                                    clipRect {
                                        translate(
                                            left = -(barRootOffset.x + curTx),
                                            top = -(barRootOffset.y + innerPadPx),
                                        ) {
                                            drawLayer(backdropLayer)
                                        }
                                    }
                                }
                        } else {
                            Modifier
                        },
                    )
                    .drawBehind {
                        @Suppress("UNUSED_VARIABLE")
                        val t = frameTick
                        val pressProg = frameBuf[1].coerceIn(0f, 1f)
                        drawRect(p.accent.copy(alpha = dropletBaseAlpha * (1f - 0.35f * pressProg)))
                    }
                    .border(
                        width = 0.9.dp,
                        brush = Brush.verticalGradient(
                            colors = listOf(
                                Color.White.copy(alpha = if (p.isDark) 0.36f else 0.92f),
                                Color.White.copy(alpha = if (p.isDark) 0.08f else 0.25f),
                            ),
                        ),
                        shape = CircleShape,
                    ),
            )

            // Layer 4: Tab items + horizontal liquid drag gesture handler.
            Row(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(innerPadDp)
                    .pointerInput(tabsCount, tabWidthPx, items) {
                        detectHorizontalDragGestures(
                            onDragStart = {
                                RustBridge.nativeLiquidTabEvent(physicsHandle, 3, 0f, 0f)
                                wakeChannel.trySend(Unit)
                            },
                            onDragEnd = {
                                val chosen = RustBridge.nativeLiquidTabEvent(physicsHandle, 5, 0f, 0f)
                                wakeChannel.trySend(Unit)
                                items.getOrNull(chosen)?.let { (tabId, _, _) ->
                                    if (tabId != activeTab) onSelect(tabId)
                                }
                            },
                            onDragCancel = {
                                val chosen = RustBridge.nativeLiquidTabEvent(physicsHandle, 5, 0f, 0f)
                                wakeChannel.trySend(Unit)
                                items.getOrNull(chosen)?.let { (tabId, _, _) ->
                                    if (tabId != activeTab) onSelect(tabId)
                                }
                            },
                            onHorizontalDrag = { _, dragAmount ->
                                val previewIdx = RustBridge.nativeLiquidTabEvent(
                                    physicsHandle,
                                    4,
                                    dragAmount,
                                    tabWidthPx,
                                )
                                wakeChannel.trySend(Unit)
                                if (previewIdx != lastHapticIdx && previewIdx in items.indices) {
                                    lastHapticIdx = previewIdx
                                    haptic.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                                }
                            },
                        )
                    },
                verticalAlignment = Alignment.CenterVertically,
            ) {
                items.forEachIndexed { visualIdx, (tabId, label, kind) ->
                    val active = visualIdx == activeVisualIdx
                    val tint = if (active) p.accent else p.navInactive
                    Column(
                        modifier = Modifier
                            .weight(1f)
                            .fillMaxHeight()
                            .clip(CircleShape)
                            .clickable(
                                interactionSource = remember { MutableInteractionSource() },
                                indication = null,
                            ) {
                                if (visualIdx != visualSelectedIndex) {
                                    haptic.performHapticFeedback(HapticFeedbackType.TextHandleMove)
                                    RustBridge.nativeLiquidTabEvent(physicsHandle, 1, visualIdx.toFloat(), 0f)
                                    wakeChannel.trySend(Unit)
                                    onSelect(tabId)
                                }
                            }
                            .graphicsLayer {
                                @Suppress("UNUSED_VARIABLE")
                                val t = frameTick
                                val curVal = frameBuf[0].coerceIn(0f, maxTabIdxF)
                                val pressProg = frameBuf[1].coerceIn(0f, 1f)
                                val focus = (1f - kotlin.math.abs(curVal - visualIdx.toFloat())).coerceIn(0f, 1f)
                                val itemScale = 1f + 0.06f * focus + 0.06f * focus * pressProg
                                scaleX = itemScale
                                scaleY = itemScale
                            },
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = androidx.compose.foundation.layout.Arrangement.Center,
                    ) {
                        NavGlyph(kind = kind, color = tint, active = active)
                        Spacer(Modifier.height(2.dp))
                        Text(
                            text = label,
                            fontSize = 11.sp,
                            fontWeight = if (active) FontWeight.SemiBold else FontWeight.Normal,
                            letterSpacing = 0.1.sp,
                            color = tint,
                        )
                    }
                }
            }
        }
    }
}
