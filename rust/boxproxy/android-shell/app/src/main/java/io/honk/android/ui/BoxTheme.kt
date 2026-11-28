package io.honk.android.ui

import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectHorizontalDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.composed
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import kotlinx.coroutines.delay

/** Canonical iOS interactive navigation easing curve `(0.32, 0.72, 0.0, 1.0)`. */
val IosNavEasing = CubicBezierEasing(0.32f, 0.72f, 0.0f, 1.0f)

val LocalLiquidDockBottomPadding = staticCompositionLocalOf<Dp> { 24.dp }
val LocalAppLang = staticCompositionLocalOf { "zh-CN" }

fun uiTr(lang: String, zhCn: String, zhTw: String = zhCn, en: String = zhCn, ru: String = en): String {
    val norm = lang.trim().lowercase()
    return when {
        norm.startsWith("zh-tw") || norm.startsWith("zh-hk") || norm.startsWith("zh-hant") -> zhTw
        norm.startsWith("en") -> en
        norm.startsWith("ru") -> ru
        else -> zhCn
    }
}

/**
 * Interactive iOS edge-swipe-to-pop gesture (`enable_swipe_back` + `enable_predictive_back_animation` + `follow_edge`).
 */
fun Modifier.iosInteractiveSwipeBack(
    enabled: Boolean,
    predictiveScale: Boolean = true,
    followEdge: Boolean = true,
    onPopBack: () -> Unit,
): Modifier = composed {
    if (!enabled) return@composed this
    val density = LocalDensity.current
    val edgeThresholdPx = with(density) { 42.dp.toPx() }
    var widthPx by remember { mutableFloatStateOf(1080f) }
    var rawDragPx by remember { mutableFloatStateOf(0f) }
    var trackingEdge by remember { mutableStateOf(0) } // 1 = left edge, -1 = right edge, 0 = inactive

    val animatedOffset by animateFloatAsState(
        targetValue = rawDragPx,
        animationSpec = if (trackingEdge != 0) spring(stiffness = Spring.StiffnessHigh) else spring(dampingRatio = 0.85f, stiffness = 460f),
        label = "iosSwipeBackTx",
    )
    val progress = (kotlin.math.abs(animatedOffset) / widthPx.coerceAtLeast(1f)).coerceIn(0f, 1f)

    this
        .onSizeChanged { sz ->
            if (sz.width > 0) widthPx = sz.width.toFloat()
        }
        .pointerInput(enabled, followEdge, edgeThresholdPx) {
            detectHorizontalDragGestures(
                onDragStart = { startOffset ->
                    trackingEdge = when {
                        startOffset.x <= edgeThresholdPx -> 1
                        followEdge && startOffset.x >= widthPx - edgeThresholdPx -> -1
                        else -> 0
                    }
                    rawDragPx = 0f
                },
                onHorizontalDrag = { change, dragAmount ->
                    if (trackingEdge == 1) {
                        val next = (rawDragPx + dragAmount).coerceAtLeast(0f)
                        rawDragPx = next
                        if (next > 4f) change.consume()
                    } else if (trackingEdge == -1) {
                        val next = (rawDragPx + dragAmount).coerceAtMost(0f)
                        rawDragPx = next
                        if (next < -4f) change.consume()
                    }
                },
                onDragEnd = {
                    val committed = kotlin.math.abs(rawDragPx) > widthPx * 0.25f
                    trackingEdge = 0
                    rawDragPx = 0f
                    if (committed) onPopBack()
                },
                onDragCancel = {
                    trackingEdge = 0
                    rawDragPx = 0f
                },
            )
        }
        .graphicsLayer {
            translationX = animatedOffset
            if (predictiveScale && progress > 0.001f) {
                val sc = 1f - 0.045f * progress
                scaleX = sc
                scaleY = sc
                alpha = 1f - 0.12f * progress
            }
        }
}

/**
 * Restrained iOS tactile press micro-interaction (`0.975x` spring settle).
 */
fun Modifier.iosPressScale(
    enabled: Boolean = true,
    pressedScale: Float = 0.975f,
    onClick: (() -> Unit)? = null,
): Modifier = composed {
    if (!enabled) return@composed this
    if (onClick != null) {
        val interaction = remember { MutableInteractionSource() }
        val pressed by interaction.collectIsPressedAsState()
        val scale by animateFloatAsState(
            targetValue = if (pressed) pressedScale else 1f,
            animationSpec = spring(dampingRatio = 0.82f, stiffness = 520f),
            label = "iosPressScale",
        )
        this
            .graphicsLayer {
                scaleX = scale
                scaleY = scale
            }
            .clickable(
                interactionSource = interaction,
                indication = null,
                onClick = onClick,
            )
    } else {
        var pressed by remember { mutableStateOf(false) }
        val scale by animateFloatAsState(
            targetValue = if (pressed) pressedScale else 1f,
            animationSpec = spring(dampingRatio = 0.82f, stiffness = 520f),
            label = "iosPressScalePassive",
        )
        this
            .graphicsLayer {
                scaleX = scale
                scaleY = scale
            }
            .pointerInput(Unit) {
                detectTapGestures(
                    onPress = {
                        pressed = true
                        tryAwaitRelease()
                        pressed = false
                    },
                )
            }
    }
}

/**
 * Exact Miuix / HyperOS color tokens sampled 1:1 from `com.boxproxy.honk` v214.
 */
@Immutable
data class BoxPalette(
    val isDark: Boolean,
    val background: Color,
    val card: Color,
    val heroRunningBg: Color,
    val heroStoppedBg: Color,
    val accent: Color,
    val accentSoft: Color,
    val heroAccent: Color,
    val textPrimary: Color,
    val textSecondary: Color,
    val textMuted: Color,
    val divider: Color,
    val upGreen: Color,
    val upGreenSoft: Color,
    val stopRed: Color,
    val restartAmber: Color,
    val switchTrackOff: Color,
    val switchThumb: Color,
    val navBarBg: Color,
    val navInactive: Color,
    val uiMode: String = "miuix",
    val enableBlur: Boolean = true,
    val topBarGlass: Boolean = false,
)

private fun styleAdjustedAccent(
    base: Color,
    paletteStyle: String,
    isDark: Boolean,
): Color {
    val hsv = FloatArray(3)
    android.graphics.Color.colorToHSV(base.toArgb(), hsv)
    when (paletteStyle) {
        "Monochrome" -> {
            return if (isDark) Color(0xFFD4D8E2) else Color(0xFF333740)
        }
        "Neutral" -> {
            hsv[1] = (hsv[1] * 0.42f).coerceIn(0.06f, 0.32f)
        }
        "Vibrant" -> {
            hsv[1] = (hsv[1] * 1.35f).coerceIn(0.55f, 0.92f)
        }
        "Expressive" -> {
            hsv[0] = (hsv[0] + 24f) % 360f
            hsv[1] = (hsv[1] * 1.25f).coerceIn(0.48f, 0.88f)
        }
        "Rainbow" -> {
            hsv[0] = (hsv[0] + 48f) % 360f
            hsv[1] = (hsv[1] * 1.2f).coerceIn(0.45f, 0.85f)
        }
        "FruitSalad" -> {
            hsv[0] = (hsv[0] + 320f) % 360f
            hsv[1] = (hsv[1] * 1.3f).coerceIn(0.50f, 0.88f)
        }
        "Cmf" -> {
            return if (isDark) Color(0xFFFF6B4A) else Color(0xFFD94325)
        }
    }
    return Color(android.graphics.Color.HSVToColor(hsv))
}

fun boxPaletteFor(
    isDark: Boolean,
    pureBlack: Boolean,
    accentOverride: Color?,
    uiMode: String = "miuix",
    paletteStyle: String = "TonalSpot",
    colorSpec: String = "Spec2021",
    enableBlur: Boolean = true,
    topBarGlass: Boolean = false,
): BoxPalette {
    val highContrastSpec = colorSpec == "Spec2025" || colorSpec == "Spec2026"
    return if (!isDark) {
        val rawAccent = accentOverride ?: Color(0xFF455A8A)
        val accent = styleAdjustedAccent(rawAccent, paletteStyle, isDark = false)
        val bgBase = if (highContrastSpec) Color(0xFFEEEFF5) else Color(0xFFF3F4F8)
        val bg = if (uiMode == "material") lerp(bgBase, accent, 0.04f) else bgBase
        BoxPalette(
            isDark = false,
            background = bg,
            card = Color(0xFFFFFFFF),
            heroRunningBg = lerp(Color(0xFFFFFFFF), accent, 0.11f),
            heroStoppedBg = Color(0xFFFFFFFF),
            accent = accent,
            accentSoft = lerp(Color(0xFFFFFFFF), accent, 0.14f),
            heroAccent = accent,
            textPrimary = Color(0xFF1A1C1E),
            textSecondary = Color(0xFF5B616E),
            textMuted = Color(0xFF8C929F),
            divider = if (highContrastSpec) Color(0xFFDCE0EA) else Color(0xFFE6E8EF),
            upGreen = Color(0xFF1E9B55),
            upGreenSoft = Color(0xFFDFF3E7),
            stopRed = Color(0xFFC82829),
            restartAmber = Color(0xFF9E6B15),
            switchTrackOff = Color(0xFFC6C9D2),
            switchThumb = Color(0xFFFFFFFF),
            navBarBg = bg,
            navInactive = Color(0xFF8F949F),
            uiMode = uiMode,
            enableBlur = enableBlur,
            topBarGlass = topBarGlass,
        )
    } else {
        val rawAccent = accentOverride ?: Color(0xFF7D96D4)
        val accent = styleAdjustedAccent(rawAccent, paletteStyle, isDark = true)
        val baseBg = if (pureBlack) Color(0xFF000000) else Color(0xFF0D0F14)
        val baseCard = when {
            pureBlack && highContrastSpec -> Color(0xFF181B22)
            pureBlack -> Color(0xFF15171C)
            highContrastSpec -> Color(0xFF1C2029)
            else -> Color(0xFF181B22)
        }
        val card = if (uiMode == "material") lerp(baseCard, accent, 0.05f) else baseCard
        BoxPalette(
            isDark = true,
            background = baseBg,
            card = card,
            heroRunningBg = lerp(baseCard, accent, 0.16f),
            heroStoppedBg = card,
            accent = accent,
            accentSoft = lerp(baseCard, accent, 0.22f),
            heroAccent = accent,
            textPrimary = Color(0xFFF2F4F8),
            textSecondary = Color(0xFF9BA2B0),
            textMuted = Color(0xFF737987),
            divider = if (highContrastSpec) Color(0xFF323744) else Color(0xFF282C36),
            upGreen = Color(0xFF34D399),
            upGreenSoft = Color(0xFF173128),
            stopRed = Color(0xFFF87171),
            restartAmber = Color(0xFFFBBF24),
            switchTrackOff = Color(0xFF373B46),
            switchThumb = Color(0xFFFFFFFF),
            navBarBg = if (pureBlack) Color(0xFF0A0B0E) else Color(0xFF12141A),
            navInactive = Color(0xFF777C88),
            uiMode = uiMode,
            enableBlur = enableBlur,
            topBarGlass = topBarGlass,
        )
    }
}

val LocalBoxPalette = staticCompositionLocalOf { boxPaletteFor(false, false, null) }

/** Authentic iOS `UITableView.Style.insetGrouped` hairline separator. */
@Composable
fun MiuixInsetDivider(startIndent: Dp = 56.dp) {
    val p = LocalBoxPalette.current
    HorizontalDivider(
        modifier = Modifier.padding(start = startIndent, end = 16.dp),
        thickness = 0.5.dp,
        color = p.divider.copy(alpha = 0.65f),
    )
}

/** Subtle iOS staggered card entrance animation (`0.24s` `IosNavEasing` lift + fade). */
fun Modifier.iosCardEntrance(index: Int = 0): Modifier = composed {
    var appeared by remember { mutableStateOf(false) }
    LaunchedEffect(Unit) { appeared = true }
    val delayMs = (index * 22).coerceAtMost(132)
    val alphaAnim by animateFloatAsState(
        targetValue = if (appeared) 1f else 0f,
        animationSpec = tween(durationMillis = 240, delayMillis = delayMs, easing = IosNavEasing),
        label = "iosCardAlpha",
    )
    val offsetDp by animateDpAsState(
        targetValue = if (appeared) 0.dp else 14.dp,
        animationSpec = tween(durationMillis = 280, delayMillis = delayMs, easing = IosNavEasing),
        label = "iosCardOffset",
    )
    this.graphicsLayer {
        alpha = alphaAnim
        translationY = offsetDp.toPx()
    }
}

/**
 * Authentic iOS `UISegmentedControl` with spring-gliding pill indicator.
 */
@Composable
fun IosSegmentedControl(
    items: List<String>,
    selectedIndex: Int,
    onSelect: (Int) -> Unit,
    modifier: Modifier = Modifier,
    height: Dp = 42.dp,
    trackColor: Color = LocalBoxPalette.current.card,
) {
    val p = LocalBoxPalette.current
    val count = items.size.coerceAtLeast(1)
    val clamped = selectedIndex.coerceIn(0, count - 1)
    BoxWithConstraints(
        modifier = modifier
            .height(height)
            .clip(RoundedCornerShape(50))
            .background(trackColor)
            .padding(4.dp),
    ) {
        val segWidth = maxWidth / count
        val thumbOffset by animateDpAsState(
            targetValue = segWidth * clamped,
            animationSpec = spring(dampingRatio = 0.80f, stiffness = 460f),
            label = "iosSegThumb",
        )
        Box(
            modifier = Modifier
                .offset(x = thumbOffset)
                .width(segWidth)
                .height(height - 8.dp)
                .shadow(
                    elevation = if (p.isDark) 4.dp else 2.dp,
                    shape = RoundedCornerShape(50),
                    ambientColor = Color.Black.copy(alpha = 0.16f),
                )
                .clip(RoundedCornerShape(50))
                .background(p.accentSoft),
        )
        Row(modifier = Modifier.fillMaxWidth().height(height - 8.dp)) {
            items.forEachIndexed { idx, label ->
                val active = idx == clamped
                val textColor by animateColorAsState(
                    targetValue = if (active) p.accent else p.textPrimary,
                    animationSpec = tween(180, easing = IosNavEasing),
                    label = "iosSegText",
                )
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .height(height - 8.dp)
                        .clip(RoundedCornerShape(50))
                        .clickable(
                            interactionSource = remember { MutableInteractionSource() },
                            indication = null,
                        ) { onSelect(idx) },
                    contentAlignment = Alignment.Center,
                ) {
                    Text(
                        text = label,
                        fontSize = 14.sp,
                        fontWeight = if (active) FontWeight.SemiBold else FontWeight.Normal,
                        color = textColor,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
        }
    }
}

/**
 * Reusable iOS `UISheetPresentationController` modal bottom sheet with spring entrance & grabber pill.
 */
@Composable
fun IosBottomSheetDialog(
    onDismiss: () -> Unit,
    content: @Composable ColumnScope.() -> Unit,
) {
    val p = LocalBoxPalette.current
    var visible by remember { mutableStateOf(false) }
    var closing by remember { mutableStateOf(false) }
    LaunchedEffect(Unit) { visible = true }
    LaunchedEffect(closing) {
        if (closing) {
            visible = false
            kotlinx.coroutines.delay(175)
            onDismiss()
        }
    }
    val requestClose = { if (!closing) closing = true }
    val scrimAlpha by animateFloatAsState(
        targetValue = if (visible) 0.42f else 0f,
        animationSpec = tween(200, easing = IosNavEasing),
        label = "iosSheetScrim",
    )
    Dialog(
        onDismissRequest = requestClose,
        properties = DialogProperties(usePlatformDefaultWidth = false),
    ) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .background(Color.Black.copy(alpha = scrimAlpha))
                .clickable(
                    interactionSource = remember { MutableInteractionSource() },
                    indication = null,
                    onClick = requestClose,
                )
                .navigationBarsPadding()
                .padding(horizontal = 16.dp, vertical = 18.dp),
            contentAlignment = Alignment.BottomCenter,
        ) {
            androidx.compose.animation.AnimatedVisibility(
                visible = visible,
                enter = slideInVertically(
                    initialOffsetY = { (it * 0.55f).toInt() + 48 },
                    animationSpec = spring(dampingRatio = 0.84f, stiffness = 430f),
                ) + fadeIn(tween(190, easing = IosNavEasing)),
                exit = slideOutVertically(
                    targetOffsetY = { it / 2 },
                    animationSpec = tween(180, easing = IosNavEasing),
                ) + fadeOut(tween(150)),
            ) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .shadow(20.dp, RoundedCornerShape(28.dp), ambientColor = Color.Black.copy(alpha = 0.28f))
                        .clip(RoundedCornerShape(28.dp))
                        .background(p.card)
                        .clickable(
                            interactionSource = remember { MutableInteractionSource() },
                            indication = null,
                        ) {}
                        .padding(top = 10.dp, bottom = 22.dp, start = 22.dp, end = 22.dp),
                ) {
                    Box(
                        modifier = Modifier
                            .align(Alignment.CenterHorizontally)
                            .padding(bottom = 14.dp)
                            .width(36.dp)
                            .height(4.dp)
                            .clip(CircleShape)
                            .background(p.divider),
                    )
                    content()
                }
            }
        }
    }
}

@Composable
fun MiuixCard(
    modifier: Modifier = Modifier,
    backgroundColor: Color = LocalBoxPalette.current.card,
    cornerRadius: Int = 20,
    onClick: (() -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    val p = LocalBoxPalette.current
    val effectiveCorner = if (p.uiMode == "material" && cornerRadius == 20) 24 else cornerRadius
    val shape = RoundedCornerShape(effectiveCorner.dp)
    val base = modifier
        .let { if (onClick != null) it.iosPressScale(pressedScale = 0.98f, onClick = onClick) else it }
        .clip(shape)
        .background(backgroundColor)
        .let {
            if (p.uiMode == "material") {
                it.border(0.8.dp, p.divider.copy(alpha = 0.85f), shape)
            } else {
                it
            }
        }
    Column(modifier = base, content = content)
}

@Composable
fun MiuixSwitch(
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    val p = LocalBoxPalette.current
    val trackColor by animateColorAsState(
        targetValue = if (checked) p.accent else p.switchTrackOff,
        animationSpec = tween(durationMillis = 220, easing = IosNavEasing),
        label = "switchTrack",
    )
    val thumbOffset by animateDpAsState(
        targetValue = if (checked) 22.dp else 0.dp,
        animationSpec = spring(dampingRatio = 0.78f, stiffness = 480f),
        label = "switchThumb",
    )
    Box(
        modifier = modifier
            .width(52.dp)
            .height(30.dp)
            .clip(RoundedCornerShape(50))
            .background(trackColor)
            .clickable(
                interactionSource = remember { MutableInteractionSource() },
                indication = null,
            ) { onCheckedChange(!checked) }
            .padding(3.dp),
        contentAlignment = Alignment.CenterStart,
    ) {
        Box(
            modifier = Modifier
                .offset(x = thumbOffset)
                .size(24.dp)
                .shadow(elevation = 2.dp, shape = CircleShape)
                .clip(CircleShape)
                .background(p.switchThumb),
        )
    }
}

@Composable
fun MiuixNavRow(
    title: String,
    icon: @Composable () -> Unit,
    trailingText: String? = null,
    isDropdown: Boolean = false,
    enabled: Boolean = true,
    onClick: () -> Unit,
) {
    val p = LocalBoxPalette.current
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(enabled = enabled, onClick = onClick)
            .padding(horizontal = 20.dp, vertical = 17.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(modifier = Modifier.size(26.dp), contentAlignment = Alignment.Center) {
            icon()
        }
        Spacer(Modifier.width(18.dp))
        Text(
            text = title,
            fontSize = 17.sp,
            fontWeight = FontWeight.Normal,
            color = if (enabled) p.textPrimary else p.textMuted,
            modifier = Modifier.weight(1f),
        )
        if (!trailingText.isNullOrEmpty()) {
            Text(
                text = trailingText,
                fontSize = 14.sp,
                color = p.textSecondary,
                modifier = Modifier.padding(end = 6.dp),
            )
        }
        if (isDropdown) {
            MiuixChevronUpDown(color = p.textSecondary)
        } else {
            MiuixChevronRight(color = p.textMuted)
        }
    }
}

@Composable
fun MiuixSwitchRow(
    title: String,
    icon: @Composable () -> Unit,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
    onRowClick: (() -> Unit)? = null,
) {
    val p = LocalBoxPalette.current
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onRowClick?.invoke() ?: onCheckedChange(!checked) }
            .padding(horizontal = 20.dp, vertical = 15.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(modifier = Modifier.size(26.dp), contentAlignment = Alignment.Center) {
            icon()
        }
        Spacer(Modifier.width(18.dp))
        Text(
            text = title,
            fontSize = 17.sp,
            color = p.textPrimary,
            modifier = Modifier.weight(1f),
        )
        MiuixSwitch(checked = checked, onCheckedChange = onCheckedChange)
    }
}

@Composable
fun MiuixBottomBar(
    activeTab: Int,
    showPanelTab: Boolean,
    onSelect: (Int) -> Unit,
) {
    val p = LocalBoxPalette.current
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .background(p.navBarBg)
            .navigationBarsPadding(),
    ) {
        HorizontalDivider(thickness = 0.6.dp, color = p.divider)
        Row(
            modifier = Modifier.fillMaxWidth().height(62.dp),
            horizontalArrangement = Arrangement.SpaceEvenly,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            val lang = LocalAppLang.current
            val items = buildList {
                add(Triple(0, uiTr(lang, "首页", "首頁", "Home", "Главная"), "home"))
                if (showPanelTab) add(Triple(1, uiTr(lang, "面板", "面板", "Panel", "Панель"), "panel"))
                add(Triple(2, uiTr(lang, "工具", "工具", "Tools", "Утилиты"), "tools"))
                add(Triple(3, uiTr(lang, "设置", "設定", "Settings", "Настройки"), "settings"))
            }
            items.forEach { (idx, label, kind) ->
                val active = activeTab == idx
                val tint = if (active) p.textPrimary else p.navInactive
                Column(
                    modifier = Modifier
                        .weight(1f)
                        .clickable { onSelect(idx) }
                        .padding(vertical = 6.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                    verticalArrangement = Arrangement.Center,
                ) {
                    NavGlyph(kind = kind, color = tint, active = active)
                    Spacer(Modifier.height(3.dp))
                    Text(
                        text = label,
                        fontSize = 11.sp,
                        fontWeight = if (active) FontWeight.Medium else FontWeight.Normal,
                        color = tint,
                    )
                }
            }
        }
    }
}

fun formatUptimeZh(secs: Long): String {
    if (secs <= 0L) return "刚刚启动"
    val hours = secs / 3600
    val mins = (secs % 3600) / 60
    return when {
        hours > 0 -> "$hours 小时 $mins 分钟"
        mins > 0 -> "$mins 分钟"
        else -> "$secs 秒"
    }
}

fun formatRateCompact(bps: Long): String {
    val f = bps.toDouble()
    return when {
        bps >= 1024L * 1024L * 1024L -> "${"%.2f".format(f / (1024.0 * 1024.0 * 1024.0))} GB/s"
        bps >= 1024L * 1024L -> "${"%.1f".format(f / (1024.0 * 1024.0))} MB/s"
        bps >= 1024L -> "${"%.1f".format(f / 1024.0)} KB/s"
        else -> "$bps B/s"
    }
}

fun formatBytesCompact(bytes: Long): String {
    val f = bytes.toDouble()
    return when {
        bytes >= 1024L * 1024L * 1024L -> "${"%.2f".format(f / (1024.0 * 1024.0 * 1024.0))} GB"
        bytes >= 1024L * 1024L -> "${"%.1f".format(f / (1024.0 * 1024.0))} MB"
        bytes >= 1024L -> "${"%.0f".format(f / 1024.0)} KB"
        else -> "$bytes B"
    }
}

fun coreDisplayName(raw: String): String = when (raw.lowercase()) {
    "honk" -> "Honk"
    "mihomo" -> "Mihomo"
    "meow" -> "Meow"
    "sing-box" -> "Sing-Box"
    "xray" -> "Xray"
    "v2fly" -> "V2Fly"
    "hysteria" -> "Hysteria"
    else -> raw
}

fun modeDisplayName(raw: String): String = when (raw.lowercase()) {
    "ebpf" -> "eBPF"
    "tun" -> "TUN"
    "tproxy" -> "TProxy"
    "redirect" -> "Redirect"
    "mixed" -> "Mixed"
    "enhance" -> "Enhance"
    else -> raw
}

fun countryBadge(countryCode: String, fallbackCountry: String): String {
    val cc = countryCode.trim().uppercase()
    if (cc.length != 2 || !cc.all { it in 'A'..'Z' }) {
        return fallbackCountry.ifEmpty { "-" }
    }
    val flag = String(Character.toChars(0x1F1E6 + (cc[0] - 'A'))) +
        String(Character.toChars(0x1F1E6 + (cc[1] - 'A')))
    val zh = when (cc) {
        "HK" -> "香港"
        "TW" -> "台湾"
        "SG" -> "新加坡"
        "JP" -> "日本"
        "US" -> "美国"
        "CN" -> "中国"
        "KR" -> "韩国"
        "GB" -> "英国"
        "DE" -> "德国"
        "FR" -> "法国"
        "NL" -> "荷兰"
        "CA" -> "加拿大"
        "AU" -> "澳大利亚"
        else -> fallbackCountry.ifEmpty { cc }
    }
    return "$flag $zh"
}

@Composable
fun MiuixTabHeader(
    title: String,
    collapsed: Boolean,
    trailing: (@Composable () -> Unit)? = null,
) {
    val p = LocalBoxPalette.current
    val headerBg = if (p.topBarGlass && collapsed) p.card.copy(alpha = 0.84f) else p.background
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .background(headerBg)
            .statusBarsPadding(),
    ) {
        AnimatedContent(
            targetState = collapsed,
            transitionSpec = {
                (fadeIn(tween(200, easing = IosNavEasing))) togetherWith
                    (fadeOut(tween(150, easing = IosNavEasing)))
            },
            label = "tabHeaderCollapse",
        ) { isCollapsed ->
            if (isCollapsed) {
                Column(modifier = Modifier.fillMaxWidth()) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(52.dp)
                            .padding(horizontal = 16.dp),
                    ) {
                        Text(
                            text = title,
                            fontSize = 18.sp,
                            fontWeight = FontWeight.SemiBold,
                            letterSpacing = (-0.2).sp,
                            color = p.textPrimary,
                            modifier = Modifier.align(Alignment.Center),
                        )
                        if (trailing != null) {
                            Box(modifier = Modifier.align(Alignment.CenterEnd)) { trailing() }
                        }
                    }
                    HorizontalDivider(thickness = 0.5.dp, color = p.divider.copy(alpha = 0.65f))
                }
            } else {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(start = 20.dp, end = 16.dp, top = 12.dp, bottom = 10.dp),
                ) {
                    if (trailing != null) {
                        Box(modifier = Modifier.align(Alignment.TopEnd)) { trailing() }
                    }
                    Text(
                        text = title,
                        fontSize = 31.sp,
                        fontWeight = FontWeight.SemiBold,
                        letterSpacing = (-0.6).sp,
                        color = p.textPrimary,
                        modifier = Modifier.padding(top = if (trailing != null) 28.dp else 16.dp),
                    )
                }
            }
        }
    }
}

@Composable
fun MiuixSubPageTopBar(
    title: String,
    collapsed: Boolean,
    onBack: () -> Unit,
    trailing: (@Composable () -> Unit)? = null,
) {
    val p = LocalBoxPalette.current
    val barBg = if (p.topBarGlass) p.card.copy(alpha = 0.84f) else p.background
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .background(barBg)
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
                    .iosPressScale(pressedScale = 0.9f, onClick = onBack)
                    .clip(CircleShape),
                contentAlignment = Alignment.Center,
            ) {
                ToolRowGlyph("back", p.textPrimary, size = 22.dp)
            }
            androidx.compose.animation.AnimatedVisibility(
                visible = collapsed,
                enter = fadeIn(tween(180, easing = IosNavEasing)),
                exit = fadeOut(tween(140)),
                modifier = Modifier.align(Alignment.Center),
            ) {
                Text(
                    text = title,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.SemiBold,
                    letterSpacing = (-0.2).sp,
                    color = p.textPrimary,
                )
            }
            if (trailing != null) {
                Box(modifier = Modifier.align(Alignment.CenterEnd)) { trailing() }
            }
        }
        AnimatedVisibility(
            visible = !collapsed,
            enter = fadeIn(tween(180, easing = IosNavEasing)),
            exit = fadeOut(tween(140)),
        ) {
            Text(
                text = title,
                fontSize = 30.sp,
                fontWeight = FontWeight.SemiBold,
                letterSpacing = (-0.5).sp,
                color = p.textPrimary,
                modifier = Modifier.padding(start = 20.dp, end = 20.dp, top = 2.dp, bottom = 12.dp),
            )
        }
    }
}

/**
 * Clean iOS-style confirmation sheet for critical/destructive actions.
 */
@Composable
fun MiuixConfirmDialog(
    title: String,
    message: String,
    confirmLabel: String,
    destructive: Boolean = true,
    onConfirm: () -> Unit,
    onDismiss: () -> Unit,
) {
    val p = LocalBoxPalette.current
    val confirmBg = if (destructive) p.stopRed.copy(alpha = if (p.isDark) 0.22f else 0.12f) else p.accent
    val confirmFg = if (destructive) p.stopRed else Color.White
    IosBottomSheetDialog(onDismiss = onDismiss) {
        Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(
                    text = title,
                    fontSize = 19.sp,
                    fontWeight = FontWeight.SemiBold,
                    letterSpacing = (-0.2).sp,
                    color = p.textPrimary,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.fillMaxWidth(),
                )
                Text(
                    text = message,
                    fontSize = 14.sp,
                    lineHeight = 20.sp,
                    color = p.textSecondary,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .height(48.dp)
                        .iosPressScale(onClick = onDismiss)
                        .clip(RoundedCornerShape(16.dp))
                        .background(p.background),
                    contentAlignment = Alignment.Center,
                ) {
                    Text("取消", fontSize = 16.sp, fontWeight = FontWeight.Medium, color = p.textSecondary)
                }
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .height(48.dp)
                        .iosPressScale {
                            onConfirm()
                            onDismiss()
                        }
                        .clip(RoundedCornerShape(16.dp))
                        .background(confirmBg),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(confirmLabel, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = confirmFg)
                }
            }
        }
    }
}

/**
 * Floating iOS-style top status banner displaying transient loading / success / error feedback.
 */
@Composable
fun BoxStatusBannerOverlay(
    message: String?,
    tone: String,
    onDismiss: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val p = LocalBoxPalette.current
    LaunchedEffect(message, tone) {
        if (!message.isNullOrBlank()) {
            delay(2600)
            onDismiss()
        }
    }
    AnimatedVisibility(
        visible = !message.isNullOrBlank(),
        enter = slideInVertically(
            initialOffsetY = { -it - 24 },
            animationSpec = spring(dampingRatio = 0.82f, stiffness = Spring.StiffnessMediumLow),
        ) + fadeIn(tween(180, easing = IosNavEasing)),
        exit = slideOutVertically(
            targetOffsetY = { -it - 24 },
            animationSpec = tween(200, easing = IosNavEasing),
        ) + fadeOut(tween(160)),
        modifier = modifier
            .fillMaxWidth()
            .statusBarsPadding()
            .padding(horizontal = 20.dp, vertical = 10.dp),
    ) {
        val dotColor = when (tone) {
            "error" -> p.stopRed
            "warn" -> p.restartAmber
            "success" -> p.upGreen
            else -> p.accent
        }
        val bgColor = if (p.isDark) Color(0xE81C202A) else Color(0xF2FFFFFF)
        val borderColor = if (p.isDark) Color.White.copy(alpha = 0.14f) else p.divider
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .shadow(12.dp, CircleShape, ambientColor = Color.Black.copy(alpha = 0.18f))
                .clip(CircleShape)
                .background(bgColor)
                .border(0.8.dp, borderColor, CircleShape)
                .clickable(onClick = onDismiss)
                .padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            Box(modifier = Modifier.size(8.dp).clip(CircleShape).background(dotColor))
            Text(
                text = message.orEmpty(),
                fontSize = 14.sp,
                fontWeight = FontWeight.Medium,
                color = p.textPrimary,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(1f),
            )
        }
    }
}

@Composable
fun MiuixSettingTextRow(
    title: String,
    trailingText: String? = null,
    isDropdown: Boolean = false,
    showChevron: Boolean = true,
    enabled: Boolean = true,
    swatchColors: List<Color>? = null,
    onClick: () -> Unit,
) {
    val p = LocalBoxPalette.current
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .alpha(if (enabled) 1f else 0.45f)
            .clickable(enabled = enabled, onClick = onClick)
            .padding(horizontal = 20.dp, vertical = 17.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = title,
            fontSize = 17.sp,
            color = p.textPrimary,
            modifier = Modifier.weight(1f),
        )
        if (!swatchColors.isNullOrEmpty()) {
            Row(
                horizontalArrangement = Arrangement.spacedBy(5.dp),
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.padding(end = 10.dp),
            ) {
                swatchColors.forEach { c ->
                    Box(modifier = Modifier.size(10.dp).clip(CircleShape).background(c))
                }
            }
        }
        if (!trailingText.isNullOrEmpty()) {
            Text(
                text = trailingText,
                fontSize = 14.sp,
                color = p.textSecondary,
                modifier = Modifier.padding(end = 6.dp),
            )
        }
        if (isDropdown) {
            MiuixChevronUpDown(color = p.textSecondary)
        } else if (showChevron) {
            MiuixChevronRight(color = p.textMuted)
        }
    }
}

@Composable
fun MiuixSettingSwitchRow(
    title: String,
    checked: Boolean,
    enabled: Boolean = true,
    onCheckedChange: (Boolean) -> Unit,
) {
    val p = LocalBoxPalette.current
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .alpha(if (enabled) 1f else 0.48f)
            .clickable(enabled = enabled) { onCheckedChange(!checked) }
            .padding(horizontal = 20.dp, vertical = 15.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            text = title,
            fontSize = 17.sp,
            color = p.textPrimary,
            modifier = Modifier.weight(1f),
        )
        MiuixSwitch(checked = checked, onCheckedChange = { if (enabled) onCheckedChange(it) })
    }
}

@Composable
fun MiuixPillTextField(
    value: String,
    placeholder: String,
    enabled: Boolean = true,
    onValueChange: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val p = LocalBoxPalette.current
    OutlinedTextField(
        value = value,
        onValueChange = onValueChange,
        enabled = enabled,
        singleLine = true,
        placeholder = { Text(placeholder, color = p.textMuted, fontSize = 15.sp) },
        shape = RoundedCornerShape(14.dp),
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 18.dp, vertical = 6.dp),
    )
}

@Composable
fun MiuixOptionSheetDialog(
    title: String,
    options: List<Pair<String, String>>,
    selectedValue: String,
    onSelect: (String) -> Unit,
    onDismiss: () -> Unit,
) {
    val p = LocalBoxPalette.current
    IosBottomSheetDialog(onDismiss = onDismiss) {
        Text(
            text = title,
            fontSize = 18.sp,
            fontWeight = FontWeight.SemiBold,
            color = p.textPrimary,
            textAlign = TextAlign.Center,
            modifier = Modifier.fillMaxWidth().padding(bottom = 12.dp),
        )
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            options.forEach { (value, label) ->
                val selected = value == selectedValue
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(14.dp))
                        .background(if (selected) p.accentSoft.copy(alpha = 0.72f) else Color.Transparent)
                        .iosPressScale(pressedScale = 0.985f) {
                            onSelect(value)
                            onDismiss()
                        }
                        .padding(horizontal = 16.dp, vertical = 14.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        text = label,
                        fontSize = 16.sp,
                        fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
                        color = if (selected) p.accent else p.textPrimary,
                        modifier = Modifier.weight(1f),
                    )
                    if (selected) {
                        ToolRowGlyph("check", p.accent, size = 18.dp)
                    }
                }
            }
        }
    }
}

@Composable
fun MiuixTextInputDialog(
    title: String,
    subtitle: String? = null,
    initialValue: String,
    placeholder: String,
    isPassword: Boolean = false,
    confirmLabel: String = "确定",
    onConfirm: (String) -> Unit,
    onDismiss: () -> Unit,
) {
    val p = LocalBoxPalette.current
    var text by remember(initialValue) { mutableStateOf(initialValue) }
    IosBottomSheetDialog(onDismiss = onDismiss) {
        Column(verticalArrangement = Arrangement.spacedBy(14.dp)) {
            Text(
                text = title,
                fontSize = 19.sp,
                fontWeight = FontWeight.SemiBold,
                color = p.textPrimary,
                textAlign = TextAlign.Center,
                modifier = Modifier.fillMaxWidth(),
            )
            if (!subtitle.isNullOrEmpty()) {
                Text(
                    text = subtitle,
                    fontSize = 14.sp,
                    color = p.textSecondary,
                )
            }
            OutlinedTextField(
                value = text,
                onValueChange = { text = it },
                placeholder = { Text(placeholder, color = p.textMuted) },
                singleLine = true,
                visualTransformation = if (isPassword) PasswordVisualTransformation() else VisualTransformation.None,
                shape = RoundedCornerShape(14.dp),
                modifier = Modifier.fillMaxWidth(),
            )
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .height(48.dp)
                        .iosPressScale(onClick = onDismiss)
                        .clip(RoundedCornerShape(16.dp))
                        .background(p.background),
                    contentAlignment = Alignment.Center,
                ) {
                    Text("取消", fontSize = 16.sp, color = p.textSecondary)
                }
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .height(48.dp)
                        .iosPressScale {
                            onConfirm(text.trim())
                            onDismiss()
                        }
                        .clip(RoundedCornerShape(16.dp))
                        .background(p.accentSoft),
                    contentAlignment = Alignment.Center,
                ) {
                    Text(confirmLabel, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = p.accent)
                }
            }
        }
    }
}

@Composable
fun MiuixDiscreteScaleSlider(
    currentScale: Float,
    onScaleSelected: (Float) -> Unit,
) {
    val p = LocalBoxPalette.current
    val stops = listOf(0.8f, 0.9f, 1.0f, 1.1f, 1.2f)
    val selectedIdx = stops.indices.minByOrNull { kotlin.math.abs(stops[it] - currentScale) } ?: 2
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(horizontal = 18.dp, vertical = 10.dp)
            .height(34.dp)
            .clip(RoundedCornerShape(50))
            .background(p.accentSoft),
    ) {
        val fillFrac = ((selectedIdx + 0.5f) / stops.size.toFloat()).coerceIn(0.18f, 1f)
        Box(
            modifier = Modifier
                .fillMaxWidth(fillFrac)
                .height(34.dp)
                .clip(RoundedCornerShape(50))
                .background(p.accent),
        )
        Row(
            modifier = Modifier.fillMaxWidth().height(34.dp).padding(horizontal = 6.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            stops.forEachIndexed { idx, value ->
                val isThumb = idx == selectedIdx
                Box(
                    modifier = Modifier
                        .size(28.dp)
                        .clip(CircleShape)
                        .clickable { onScaleSelected(value) },
                    contentAlignment = Alignment.Center,
                ) {
                    if (isThumb) {
                        Box(
                            modifier = Modifier
                                .size(24.dp)
                                .clip(CircleShape)
                                .background(Color.White),
                        )
                    } else {
                        Box(
                            modifier = Modifier
                                .size(6.dp)
                                .clip(CircleShape)
                                .background(if (idx < selectedIdx) Color.White.copy(alpha = 0.85f) else p.accent.copy(alpha = 0.85f)),
                        )
                    }
                }
            }
        }
    }
}
