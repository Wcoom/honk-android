package io.honk.android.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties

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
)

fun boxPaletteFor(isDark: Boolean, pureBlack: Boolean, accentOverride: Color?): BoxPalette {
    return if (!isDark) {
        val accent = accentOverride ?: Color(0xFF455A8A)
        BoxPalette(
            isDark = false,
            background = Color(0xFFF3F4F8),
            card = Color(0xFFFFFFFF),
            heroRunningBg = Color(0xFFECEFF6),
            heroStoppedBg = Color(0xFFFFFFFF),
            accent = accent,
            accentSoft = Color(0xFFE2E7F4),
            heroAccent = Color(0xFF5E719B),
            textPrimary = Color(0xFF1A1C1E),
            textSecondary = Color(0xFF6B707B),
            textMuted = Color(0xFF9EA3AE),
            divider = Color(0xFFE6E8EF),
            upGreen = Color(0xFF1E9B55),
            upGreenSoft = Color(0xFFDFF3E7),
            stopRed = Color(0xFFC82829),
            restartAmber = Color(0xFF9E6B15),
            switchTrackOff = Color(0xFFC6C9D2),
            switchThumb = Color(0xFFFFFFFF),
            navBarBg = Color(0xFFF3F4F8),
            navInactive = Color(0xFF8F949F),
        )
    } else {
        val accent = accentOverride ?: Color(0xFF7D96D4)
        BoxPalette(
            isDark = true,
            background = if (pureBlack) Color(0xFF000000) else Color(0xFF0D0F14),
            card = if (pureBlack) Color(0xFF15171C) else Color(0xFF181B22),
            heroRunningBg = Color(0xFF1B2130),
            heroStoppedBg = if (pureBlack) Color(0xFF15171C) else Color(0xFF181B22),
            accent = accent,
            accentSoft = Color(0xFF242D42),
            heroAccent = Color(0xFF8AA2DB),
            textPrimary = Color(0xFFF2F4F8),
            textSecondary = Color(0xFF969CA8),
            textMuted = Color(0xFF6B707D),
            divider = Color(0xFF282C36),
            upGreen = Color(0xFF34D399),
            upGreenSoft = Color(0xFF173128),
            stopRed = Color(0xFFF87171),
            restartAmber = Color(0xFFFBBF24),
            switchTrackOff = Color(0xFF373B46),
            switchThumb = Color(0xFFFFFFFF),
            navBarBg = if (pureBlack) Color(0xFF0A0B0E) else Color(0xFF12141A),
            navInactive = Color(0xFF777C88),
        )
    }
}

val LocalBoxPalette = staticCompositionLocalOf { boxPaletteFor(false, false, null) }

@Composable
fun MiuixCard(
    modifier: Modifier = Modifier,
    backgroundColor: Color = LocalBoxPalette.current.card,
    cornerRadius: Int = 20,
    onClick: (() -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    val shape = RoundedCornerShape(cornerRadius.dp)
    val base = modifier
        .clip(shape)
        .background(backgroundColor)
        .let { if (onClick != null) it.clickable(onClick = onClick) else it }
    Column(modifier = base, content = content)
}

@Composable
fun MiuixSwitch(
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    val p = LocalBoxPalette.current
    Box(
        modifier = modifier
            .width(52.dp)
            .height(30.dp)
            .clip(RoundedCornerShape(50))
            .background(if (checked) p.accent else p.switchTrackOff)
            .clickable { onCheckedChange(!checked) }
            .padding(3.dp),
        contentAlignment = if (checked) Alignment.CenterEnd else Alignment.CenterStart,
    ) {
        Box(
            modifier = Modifier
                .size(24.dp)
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
            val items = buildList {
                add(Triple(0, "首页", "home"))
                if (showPanelTab) add(Triple(1, "面板", "panel"))
                add(Triple(2, "工具", "tools"))
                add(Triple(3, "设置", "settings"))
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
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .background(p.background)
            .statusBarsPadding(),
    ) {
        if (collapsed) {
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(52.dp)
                    .padding(horizontal = 16.dp),
            ) {
                Text(
                    text = title,
                    fontSize = 19.sp,
                    fontWeight = FontWeight.Medium,
                    color = p.textPrimary,
                    modifier = Modifier.align(Alignment.Center),
                )
                if (trailing != null) {
                    Box(modifier = Modifier.align(Alignment.CenterEnd)) { trailing() }
                }
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
                    fontWeight = FontWeight.Normal,
                    color = p.textPrimary,
                    modifier = Modifier.padding(top = if (trailing != null) 28.dp else 18.dp),
                )
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
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .background(p.background)
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
                    .clickable(onClick = onBack),
                contentAlignment = Alignment.Center,
            ) {
                ToolRowGlyph("back", p.textPrimary, size = 22.dp)
            }
            if (collapsed) {
                Text(
                    text = title,
                    fontSize = 19.sp,
                    fontWeight = FontWeight.Medium,
                    color = p.textPrimary,
                    modifier = Modifier.align(Alignment.Center),
                )
            }
            if (trailing != null) {
                Box(modifier = Modifier.align(Alignment.CenterEnd)) { trailing() }
            }
        }
        if (!collapsed) {
            Text(
                text = title,
                fontSize = 31.sp,
                fontWeight = FontWeight.Normal,
                color = p.textPrimary,
                modifier = Modifier.padding(start = 20.dp, end = 20.dp, top = 2.dp, bottom = 12.dp),
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
    Dialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(usePlatformDefaultWidth = false),
    ) {
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 20.dp),
            contentAlignment = Alignment.BottomCenter,
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(28.dp))
                    .background(p.card)
                    .padding(vertical = 18.dp),
            ) {
                Text(
                    text = title,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Medium,
                    color = p.textPrimary,
                    textAlign = TextAlign.Center,
                    modifier = Modifier.fillMaxWidth().padding(bottom = 12.dp),
                )
                options.forEach { (value, label) ->
                    val selected = value == selectedValue
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .background(if (selected) p.accentSoft.copy(alpha = 0.65f) else Color.Transparent)
                            .clickable {
                                onSelect(value)
                                onDismiss()
                            }
                            .padding(horizontal = 24.dp, vertical = 15.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            text = label,
                            fontSize = 16.sp,
                            fontWeight = if (selected) FontWeight.Medium else FontWeight.Normal,
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
    Dialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(usePlatformDefaultWidth = false),
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 18.dp)
                .clip(RoundedCornerShape(28.dp))
                .background(p.card)
                .padding(22.dp),
            verticalArrangement = Arrangement.spacedBy(14.dp),
        ) {
            Text(
                text = title,
                fontSize = 19.sp,
                fontWeight = FontWeight.Medium,
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
                        .clip(RoundedCornerShape(16.dp))
                        .background(p.background)
                        .clickable(onClick = onDismiss),
                    contentAlignment = Alignment.Center,
                ) {
                    Text("取消", fontSize = 16.sp, color = p.textSecondary)
                }
                Box(
                    modifier = Modifier
                        .weight(1f)
                        .height(48.dp)
                        .clip(RoundedCornerShape(16.dp))
                        .background(p.accentSoft)
                        .clickable {
                            onConfirm(text.trim())
                            onDismiss()
                        },
                    contentAlignment = Alignment.Center,
                ) {
                    Text(confirmLabel, fontSize = 16.sp, fontWeight = FontWeight.Medium, color = p.accent)
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
