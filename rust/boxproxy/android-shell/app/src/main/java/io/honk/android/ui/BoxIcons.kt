package io.honk.android.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.sin

/**
 * Custom crisp vector glyphs matching `com.boxproxy.honk` Miuix iconography 1:1.
 */
@Composable
fun NavGlyph(kind: String, color: Color, active: Boolean, size: Dp = 24.dp) {
    Canvas(modifier = Modifier.size(size)) {
        val w = this.size.width
        val h = this.size.height
        val stroke = Stroke(width = if (active) w * 0.085f else w * 0.072f, cap = StrokeCap.Round, join = StrokeJoin.Round)
        when (kind) {
            "home" -> {
                val path = Path().apply {
                    moveTo(w * 0.16f, h * 0.44f)
                    lineTo(w * 0.50f, h * 0.15f)
                    lineTo(w * 0.84f, h * 0.44f)
                    lineTo(w * 0.84f, h * 0.84f)
                    lineTo(w * 0.62f, h * 0.84f)
                    lineTo(w * 0.62f, h * 0.60f)
                    lineTo(w * 0.38f, h * 0.60f)
                    lineTo(w * 0.38f, h * 0.84f)
                    lineTo(w * 0.16f, h * 0.84f)
                    close()
                }
                drawPath(path, color = color, style = stroke)
            }
            "panel" -> {
                // Open book icon matching real BoxProxy Honk bottom bar
                val left = Path().apply {
                    moveTo(w * 0.50f, h * 0.28f)
                    cubicTo(w * 0.38f, h * 0.18f, w * 0.22f, h * 0.18f, w * 0.14f, h * 0.24f)
                    lineTo(w * 0.14f, h * 0.78f)
                    cubicTo(w * 0.24f, h * 0.72f, w * 0.38f, h * 0.73f, w * 0.50f, h * 0.82f)
                }
                val right = Path().apply {
                    moveTo(w * 0.50f, h * 0.28f)
                    cubicTo(w * 0.62f, h * 0.18f, w * 0.78f, h * 0.18f, w * 0.86f, h * 0.24f)
                    lineTo(w * 0.86f, h * 0.78f)
                    cubicTo(w * 0.76f, h * 0.72f, w * 0.62f, h * 0.73f, w * 0.50f, h * 0.82f)
                }
                drawPath(left, color = color, style = stroke)
                drawPath(right, color = color, style = stroke)
                drawLine(color, Offset(w * 0.50f, h * 0.28f), Offset(w * 0.50f, h * 0.82f), strokeWidth = stroke.width, cap = StrokeCap.Round)
            }
            "tools" -> {
                val cell = w * 0.32f
                val r = CornerRadius(w * 0.08f, w * 0.08f)
                val gap = w * 0.12f
                val off = w * 0.12f
                drawRoundRect(color, Offset(off, off), Size(cell, cell), r, style = stroke)
                drawRoundRect(color, Offset(off + cell + gap, off), Size(cell, cell), r, style = stroke)
                drawRoundRect(color, Offset(off, off + cell + gap), Size(cell, cell), r, style = stroke)
                drawRoundRect(color, Offset(off + cell + gap, off + cell + gap), Size(cell, cell), r, style = stroke)
            }
            else -> {
                // Hexagon settings nut icon
                val cx = w * 0.5f
                val cy = h * 0.5f
                val rad = w * 0.38f
                val hex = Path().apply {
                    for (i in 0 until 6) {
                        val angle = (PI / 3.0 * i - PI / 2.0)
                        val px = cx + (rad * cos(angle)).toFloat()
                        val py = cy + (rad * sin(angle)).toFloat()
                        if (i == 0) moveTo(px, py) else lineTo(px, py)
                    }
                    close()
                }
                drawPath(hex, color = color, style = stroke)
                drawCircle(color = color, radius = w * 0.13f, center = Offset(cx, cy), style = stroke)
            }
        }
    }
}

@Composable
fun HeroStatusWatermark(running: Boolean, color: Color, size: Dp = 142.dp) {
    Canvas(modifier = Modifier.size(size)) {
        val w = this.size.width
        val h = this.size.height
        val strokeW = w * 0.095f
        val radius = w * 0.42f
        drawCircle(color = color, radius = radius, center = Offset(w * 0.5f, h * 0.5f), style = Stroke(width = strokeW))
        if (running) {
            val check = Path().apply {
                moveTo(w * 0.31f, h * 0.51f)
                lineTo(w * 0.45f, h * 0.64f)
                lineTo(w * 0.70f, h * 0.38f)
            }
            drawPath(
                check,
                color = color,
                style = Stroke(width = strokeW * 1.05f, cap = StrokeCap.Round, join = StrokeJoin.Round),
            )
        } else {
            drawLine(
                color = color,
                start = Offset(w * 0.32f, h * 0.50f),
                end = Offset(w * 0.68f, h * 0.50f),
                strokeWidth = strokeW * 1.05f,
                cap = StrokeCap.Round,
            )
        }
    }
}

@Composable
fun MiuixChevronRight(color: Color, size: Dp = 16.dp) {
    Canvas(modifier = Modifier.size(size)) {
        val w = this.size.width
        val h = this.size.height
        val path = Path().apply {
            moveTo(w * 0.34f, h * 0.20f)
            lineTo(w * 0.68f, h * 0.50f)
            lineTo(w * 0.34f, h * 0.80f)
        }
        drawPath(path, color, style = Stroke(w * 0.11f, cap = StrokeCap.Round, join = StrokeJoin.Round))
    }
}

@Composable
fun MiuixChevronUpDown(color: Color, size: Dp = 16.dp) {
    Canvas(modifier = Modifier.size(size)) {
        val w = this.size.width
        val h = this.size.height
        val st = Stroke(w * 0.11f, cap = StrokeCap.Round, join = StrokeJoin.Round)
        val up = Path().apply {
            moveTo(w * 0.26f, h * 0.38f)
            lineTo(w * 0.50f, h * 0.16f)
            lineTo(w * 0.74f, h * 0.38f)
        }
        val down = Path().apply {
            moveTo(w * 0.26f, h * 0.62f)
            lineTo(w * 0.50f, h * 0.84f)
            lineTo(w * 0.74f, h * 0.62f)
        }
        drawPath(up, color, style = st)
        drawPath(down, color, style = st)
    }
}

@Composable
fun ToolRowGlyph(kind: String, color: Color, size: Dp = 22.dp) {
    Canvas(modifier = Modifier.size(size)) {
        val w = this.size.width
        val h = this.size.height
        val st = Stroke(w * 0.08f, cap = StrokeCap.Round, join = StrokeJoin.Round)
        when (kind) {
            "sliders" -> {
                drawLine(color, Offset(w * 0.12f, h * 0.32f), Offset(w * 0.88f, h * 0.32f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.12f, h * 0.68f), Offset(w * 0.88f, h * 0.68f), st.width, StrokeCap.Round)
                drawCircle(color, w * 0.11f, Offset(w * 0.65f, h * 0.32f))
                drawCircle(color, w * 0.11f, Offset(w * 0.35f, h * 0.68f))
            }
            "refresh" -> {
                drawArc(color, 30f, 290f, false, Offset(w * 0.16f, h * 0.16f), Size(w * 0.68f, h * 0.68f), style = st)
                drawLine(color, Offset(w * 0.76f, h * 0.16f), Offset(w * 0.84f, h * 0.36f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.62f, h * 0.34f), Offset(w * 0.84f, h * 0.36f), st.width, StrokeCap.Round)
            }
            "pin" -> {
                drawCircle(color, w * 0.22f, Offset(w * 0.50f, h * 0.36f), style = st)
                drawCircle(color, w * 0.07f, Offset(w * 0.50f, h * 0.36f))
                val base = Path().apply {
                    moveTo(w * 0.30f, h * 0.45f)
                    lineTo(w * 0.50f, h * 0.80f)
                    lineTo(w * 0.70f, h * 0.45f)
                }
                drawPath(base, color, style = st)
                drawLine(color, Offset(w * 0.24f, h * 0.86f), Offset(w * 0.76f, h * 0.86f), st.width, StrokeCap.Round)
            }
            "folder" -> {
                val p = Path().apply {
                    moveTo(w * 0.12f, h * 0.26f)
                    lineTo(w * 0.40f, h * 0.26f)
                    lineTo(w * 0.50f, h * 0.36f)
                    lineTo(w * 0.88f, h * 0.36f)
                    lineTo(w * 0.88f, h * 0.80f)
                    lineTo(w * 0.12f, h * 0.80f)
                    close()
                }
                drawPath(p, color, style = st)
            }
            "script" -> {
                drawRoundRect(color, Offset(w * 0.20f, h * 0.16f), Size(w * 0.60f, h * 0.54f), CornerRadius(w * 0.18f), style = st)
                drawLine(color, Offset(w * 0.50f, h * 0.70f), Offset(w * 0.50f, h * 0.88f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.34f, h * 0.86f), Offset(w * 0.66f, h * 0.86f), st.width, StrokeCap.Round)
            }
            "log" -> {
                drawRoundRect(color, Offset(w * 0.18f, h * 0.14f), Size(w * 0.64f, h * 0.72f), CornerRadius(w * 0.10f), style = st)
                drawLine(color, Offset(w * 0.32f, h * 0.36f), Offset(w * 0.68f, h * 0.36f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.32f, h * 0.52f), Offset(w * 0.68f, h * 0.52f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.32f, h * 0.68f), Offset(w * 0.54f, h * 0.68f), st.width, StrokeCap.Round)
            }
            "apps" -> {
                drawLine(color, Offset(w * 0.14f, h * 0.30f), Offset(w * 0.82f, h * 0.30f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.14f, h * 0.52f), Offset(w * 0.62f, h * 0.52f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.14f, h * 0.74f), Offset(w * 0.48f, h * 0.74f), st.width, StrokeCap.Round)
                val chk = Path().apply {
                    moveTo(w * 0.58f, h * 0.70f)
                    lineTo(w * 0.70f, h * 0.82f)
                    lineTo(w * 0.88f, h * 0.58f)
                }
                drawPath(chk, color, style = st)
            }
            "wifi" -> {
                drawArc(color, 225f, 90f, false, Offset(w * 0.12f, h * 0.22f), Size(w * 0.76f, h * 0.76f), style = st)
                drawArc(color, 225f, 90f, false, Offset(w * 0.26f, h * 0.38f), Size(w * 0.48f, h * 0.48f), style = st)
                drawCircle(color, w * 0.07f, Offset(w * 0.50f, h * 0.74f))
            }
            "hotspot" -> {
                drawArc(color, 140f, 260f, false, Offset(w * 0.14f, h * 0.16f), Size(w * 0.72f, h * 0.72f), style = st)
                drawArc(color, 150f, 240f, false, Offset(w * 0.28f, h * 0.30f), Size(w * 0.44f, h * 0.44f), style = st)
                drawCircle(color, w * 0.08f, Offset(w * 0.50f, h * 0.54f))
            }
            "branch" -> {
                drawCircle(color, w * 0.09f, Offset(w * 0.28f, h * 0.24f), style = st)
                drawCircle(color, w * 0.09f, Offset(w * 0.28f, h * 0.76f), style = st)
                drawCircle(color, w * 0.09f, Offset(w * 0.74f, h * 0.36f), style = st)
                drawLine(color, Offset(w * 0.28f, h * 0.33f), Offset(w * 0.28f, h * 0.67f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.28f, h * 0.56f), Offset(w * 0.74f, h * 0.45f), st.width, StrokeCap.Round)
            }
            "link" -> {
                drawRoundRect(color, Offset(w * 0.14f, h * 0.34f), Size(w * 0.42f, h * 0.32f), CornerRadius(w * 0.16f), style = st)
                drawRoundRect(color, Offset(w * 0.44f, h * 0.34f), Size(w * 0.42f, h * 0.32f), CornerRadius(w * 0.16f), style = st)
            }
            "globe" -> {
                drawCircle(color, w * 0.36f, Offset(w * 0.50f, h * 0.50f), style = st)
                drawLine(color, Offset(w * 0.14f, h * 0.50f), Offset(w * 0.86f, h * 0.50f), st.width)
                drawOval(color, Offset(w * 0.32f, h * 0.14f), Size(w * 0.36f, h * 0.72f), style = st)
            }
            "download" -> {
                drawRoundRect(color, Offset(w * 0.16f, h * 0.32f), Size(w * 0.68f, h * 0.52f), CornerRadius(w * 0.12f), style = st)
                drawLine(color, Offset(w * 0.50f, h * 0.12f), Offset(w * 0.50f, h * 0.62f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.34f, h * 0.48f), Offset(w * 0.50f, h * 0.64f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.66f, h * 0.48f), Offset(w * 0.50f, h * 0.64f), st.width, StrokeCap.Round)
            }
            "power" -> {
                drawArc(color, -60f, 300f, false, Offset(w * 0.18f, h * 0.20f), Size(w * 0.64f, h * 0.64f), style = st)
                drawLine(color, Offset(w * 0.50f, h * 0.12f), Offset(w * 0.50f, h * 0.48f), st.width, StrokeCap.Round)
            }
            "lock" -> {
                drawRoundRect(color, Offset(w * 0.20f, h * 0.44f), Size(w * 0.60f, h * 0.42f), CornerRadius(w * 0.10f), style = st)
                drawArc(color, 180f, 180f, false, Offset(w * 0.30f, h * 0.16f), Size(w * 0.40f, h * 0.48f), style = st)
            }
            "bell" -> {
                drawArc(color, 180f, 180f, false, Offset(w * 0.24f, h * 0.18f), Size(w * 0.52f, h * 0.54f), style = st)
                drawLine(color, Offset(w * 0.16f, h * 0.72f), Offset(w * 0.84f, h * 0.72f), st.width, StrokeCap.Round)
                drawCircle(color, w * 0.08f, Offset(w * 0.50f, h * 0.84f))
            }
            "layers" -> {
                val top = Path().apply {
                    moveTo(w * 0.50f, h * 0.16f)
                    lineTo(w * 0.86f, h * 0.36f)
                    lineTo(w * 0.50f, h * 0.56f)
                    lineTo(w * 0.14f, h * 0.36f)
                    close()
                }
                drawPath(top, color, style = st)
                drawLine(color, Offset(w * 0.14f, h * 0.56f), Offset(w * 0.50f, h * 0.74f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.86f, h * 0.56f), Offset(w * 0.50f, h * 0.74f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.14f, h * 0.72f), Offset(w * 0.50f, h * 0.88f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.86f, h * 0.72f), Offset(w * 0.50f, h * 0.88f), st.width, StrokeCap.Round)
            }
            "brush" -> {
                drawRoundRect(color, Offset(w * 0.22f, h * 0.14f), Size(w * 0.56f, h * 0.32f), CornerRadius(w * 0.08f), style = st)
                drawLine(color, Offset(w * 0.50f, h * 0.46f), Offset(w * 0.50f, h * 0.86f), st.width * 1.4f, StrokeCap.Round)
            }
            "info" -> {
                drawCircle(color, w * 0.38f, Offset(w * 0.50f, h * 0.50f), style = st)
                drawCircle(color, w * 0.06f, Offset(w * 0.50f, h * 0.33f))
                drawLine(color, Offset(w * 0.50f, h * 0.46f), Offset(w * 0.50f, h * 0.70f), st.width, StrokeCap.Round)
            }
            "arrow_up" -> {
                drawLine(color, Offset(w * 0.50f, h * 0.22f), Offset(w * 0.50f, h * 0.78f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.28f, h * 0.44f), Offset(w * 0.50f, h * 0.22f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.72f, h * 0.44f), Offset(w * 0.50f, h * 0.22f), st.width, StrokeCap.Round)
            }
            "arrow_down" -> {
                drawLine(color, Offset(w * 0.50f, h * 0.22f), Offset(w * 0.50f, h * 0.78f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.28f, h * 0.56f), Offset(w * 0.50f, h * 0.78f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.72f, h * 0.56f), Offset(w * 0.50f, h * 0.78f), st.width, StrokeCap.Round)
            }
            "language" -> {
                // "中/A" outline glyph matching orig_settings_top.png
                drawRoundRect(color, Offset(w * 0.14f, h * 0.16f), Size(w * 0.34f, h * 0.30f), CornerRadius(w * 0.06f), style = st)
                drawLine(color, Offset(w * 0.31f, h * 0.10f), Offset(w * 0.31f, h * 0.56f), st.width, StrokeCap.Round)
                // A glyph bottom-right
                val aPath = Path().apply {
                    moveTo(w * 0.54f, h * 0.86f)
                    lineTo(w * 0.70f, h * 0.44f)
                    lineTo(w * 0.86f, h * 0.86f)
                }
                drawPath(aPath, color, style = st)
                drawLine(color, Offset(w * 0.60f, h * 0.72f), Offset(w * 0.80f, h * 0.72f), st.width, StrokeCap.Round)
                drawArc(color, 270f, 90f, false, Offset(w * 0.54f, h * 0.14f), Size(w * 0.30f, h * 0.30f), style = st)
                drawArc(color, 90f, 90f, false, Offset(w * 0.16f, h * 0.56f), Size(w * 0.30f, h * 0.30f), style = st)
            }
            "sparkle" -> {
                // Multi-sparkle icon matching 长按磁贴 in orig_settings_bot.png
                val big = Path().apply {
                    moveTo(w * 0.44f, h * 0.14f)
                    lineTo(w * 0.52f, h * 0.38f)
                    lineTo(w * 0.76f, h * 0.46f)
                    lineTo(w * 0.52f, h * 0.54f)
                    lineTo(w * 0.44f, h * 0.78f)
                    lineTo(w * 0.36f, h * 0.54f)
                    lineTo(w * 0.12f, h * 0.46f)
                    lineTo(w * 0.36f, h * 0.38f)
                    close()
                }
                drawPath(big, color, style = st)
                drawCircle(color, w * 0.06f, Offset(w * 0.76f, h * 0.76f))
                drawCircle(color, w * 0.045f, Offset(w * 0.82f, h * 0.26f))
            }
            "server" -> {
                // Dual stacked server boxes matching 其他代理配置
                drawRoundRect(color, Offset(w * 0.16f, h * 0.14f), Size(w * 0.68f, h * 0.31f), CornerRadius(w * 0.08f), style = st)
                drawRoundRect(color, Offset(w * 0.16f, h * 0.55f), Size(w * 0.68f, h * 0.31f), CornerRadius(w * 0.08f), style = st)
                drawCircle(color, w * 0.05f, Offset(w * 0.30f, h * 0.295f))
                drawCircle(color, w * 0.05f, Offset(w * 0.30f, h * 0.705f))
                drawLine(color, Offset(w * 0.48f, h * 0.295f), Offset(w * 0.72f, h * 0.295f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.48f, h * 0.705f), Offset(w * 0.72f, h * 0.705f), st.width, StrokeCap.Round)
            }
            "grid" -> {
                val cell = w * 0.30f
                val r = CornerRadius(w * 0.08f)
                drawRoundRect(color, Offset(w * 0.14f, h * 0.14f), Size(cell, cell), r, style = st)
                drawRoundRect(color, Offset(w * 0.56f, h * 0.14f), Size(cell, cell), r, style = st)
                drawRoundRect(color, Offset(w * 0.14f, h * 0.56f), Size(cell, cell), r, style = st)
                drawRoundRect(color, Offset(w * 0.56f, h * 0.56f), Size(cell, cell), r, style = st)
            }
            "book" -> {
                val left = Path().apply {
                    moveTo(w * 0.50f, h * 0.28f)
                    cubicTo(w * 0.38f, h * 0.18f, w * 0.22f, h * 0.18f, w * 0.14f, h * 0.24f)
                    lineTo(w * 0.14f, h * 0.78f)
                    cubicTo(w * 0.24f, h * 0.72f, w * 0.38f, h * 0.73f, w * 0.50f, h * 0.82f)
                }
                val right = Path().apply {
                    moveTo(w * 0.50f, h * 0.28f)
                    cubicTo(w * 0.62f, h * 0.18f, w * 0.78f, h * 0.18f, w * 0.86f, h * 0.24f)
                    lineTo(w * 0.86f, h * 0.78f)
                    cubicTo(w * 0.76f, h * 0.72f, w * 0.62f, h * 0.73f, w * 0.50f, h * 0.82f)
                }
                drawPath(left, color, style = st)
                drawPath(right, color, style = st)
                drawLine(color, Offset(w * 0.50f, h * 0.28f), Offset(w * 0.50f, h * 0.82f), st.width, StrokeCap.Round)
            }
            "back" -> {
                val p = Path().apply {
                    moveTo(w * 0.64f, h * 0.16f)
                    lineTo(w * 0.28f, h * 0.50f)
                    lineTo(w * 0.64f, h * 0.84f)
                }
                drawPath(p, color, style = Stroke(w * 0.095f, cap = StrokeCap.Round, join = StrokeJoin.Round))
            }
            "more_vert" -> {
                drawCircle(color, w * 0.075f, Offset(w * 0.50f, h * 0.22f))
                drawCircle(color, w * 0.075f, Offset(w * 0.50f, h * 0.50f))
                drawCircle(color, w * 0.075f, Offset(w * 0.50f, h * 0.78f))
            }
            "plus_circle" -> {
                drawCircle(color, w * 0.38f, Offset(w * 0.50f, h * 0.50f), style = st)
                drawLine(color, Offset(w * 0.50f, h * 0.30f), Offset(w * 0.50f, h * 0.70f), st.width, StrokeCap.Round)
                drawLine(color, Offset(w * 0.30f, h * 0.50f), Offset(w * 0.70f, h * 0.50f), st.width, StrokeCap.Round)
            }
            "check" -> {
                val p = Path().apply {
                    moveTo(w * 0.18f, h * 0.52f)
                    lineTo(w * 0.42f, h * 0.76f)
                    lineTo(w * 0.84f, h * 0.26f)
                }
                drawPath(p, color, style = Stroke(w * 0.11f, cap = StrokeCap.Round, join = StrokeJoin.Round))
            }
            else -> {
                drawRoundRect(color, Offset(w * 0.16f, h * 0.18f), Size(w * 0.68f, h * 0.64f), CornerRadius(w * 0.12f), style = st)
                drawLine(color, Offset(w * 0.16f, h * 0.50f), Offset(w * 0.84f, h * 0.50f), st.width)
            }
        }
    }
}

@Composable
fun BoxCatAboutLogo(color: Color, size: Dp = 112.dp) {
    Canvas(modifier = Modifier.size(size)) {
        val w = this.size.width
        val h = this.size.height
        val st = Stroke(w * 0.042f, cap = StrokeCap.Round, join = StrokeJoin.Round)
        // Open box base
        val boxBody = Path().apply {
            moveTo(w * 0.20f, h * 0.50f)
            lineTo(w * 0.80f, h * 0.50f)
            lineTo(w * 0.74f, h * 0.86f)
            lineTo(w * 0.26f, h * 0.86f)
            close()
        }
        drawPath(boxBody, color.copy(alpha = 0.14f))
        drawPath(boxBody, color, style = st)
        // Left & right open flaps
        val leftFlap = Path().apply {
            moveTo(w * 0.20f, h * 0.50f)
            lineTo(w * 0.08f, h * 0.62f)
            lineTo(w * 0.24f, h * 0.70f)
        }
        val rightFlap = Path().apply {
            moveTo(w * 0.80f, h * 0.50f)
            lineTo(w * 0.92f, h * 0.62f)
            lineTo(w * 0.76f, h * 0.70f)
        }
        drawPath(leftFlap, color, style = st)
        drawPath(rightFlap, color, style = st)
        // Cat head peeking out
        val head = Path().apply {
            moveTo(w * 0.30f, h * 0.50f)
            lineTo(w * 0.30f, h * 0.28f)
            lineTo(w * 0.40f, h * 0.36f)
            lineTo(w * 0.60f, h * 0.36f)
            lineTo(w * 0.70f, h * 0.28f)
            lineTo(w * 0.70f, h * 0.50f)
        }
        drawPath(head, color, style = st)
        // Eyes & smile
        drawCircle(color, w * 0.028f, Offset(w * 0.42f, h * 0.43f))
        drawCircle(color, w * 0.028f, Offset(w * 0.58f, h * 0.43f))
        drawArc(color, 10f, 160f, false, Offset(w * 0.46f, h * 0.44f), Size(w * 0.08f, h * 0.05f), style = Stroke(w * 0.025f, cap = StrokeCap.Round))
        // Tail curl
        val tail = Path().apply {
            moveTo(w * 0.74f, h * 0.50f)
            cubicTo(w * 0.84f, h * 0.36f, w * 0.80f, h * 0.20f, w * 0.72f, h * 0.24f)
        }
        drawPath(tail, color, style = st)
    }
}

/**
 * 1:1 Smooth Cubic Bézier dual-series traffic waveform chart matching `orig_panel.png`.
 */
@Composable
fun TrafficTrendChart(
    uploadSeries: List<Long>,
    downloadSeries: List<Long>,
    upColor: Color,
    downColor: Color,
    modifier: Modifier = Modifier,
) {
    Canvas(modifier = modifier) {
        val w = size.width
        val h = size.height
        val count = maxOf(uploadSeries.size, downloadSeries.size, 2)
        val maxVal = maxOf(
            uploadSeries.maxOrNull() ?: 1L,
            downloadSeries.maxOrNull() ?: 1L,
            1024L,
        ).toFloat()

        fun buildPoints(series: List<Long>): List<Offset> {
            val padded = if (series.size < 2) listOf(0L, 0L) else series
            val stepX = w / (padded.size - 1).coerceAtLeast(1)
            return padded.mapIndexed { idx, v ->
                val ratio = (v.toFloat() / maxVal).coerceIn(0f, 1f)
                Offset(idx * stepX, h - (ratio * h * 0.88f) - h * 0.04f)
            }
        }

        fun drawSmoothCurve(points: List<Offset>, lineColor: Color) {
            if (points.size < 2) return
            val linePath = Path().apply {
                moveTo(points.first().x, points.first().y)
                for (i in 1 until points.size) {
                    val p0 = points[i - 1]
                    val p1 = points[i]
                    val midX = (p0.x + p1.x) * 0.5f
                    cubicTo(midX, p0.y, midX, p1.y, p1.x, p1.y)
                }
            }
            val fillPath = Path().apply {
                addPath(linePath)
                lineTo(points.last().x, h)
                lineTo(points.first().x, h)
                close()
            }
            drawPath(
                path = fillPath,
                brush = Brush.verticalGradient(
                    colors = listOf(lineColor.copy(alpha = 0.20f), lineColor.copy(alpha = 0.01f)),
                    startY = 0f,
                    endY = h,
                ),
            )
            drawPath(
                path = linePath,
                color = lineColor,
                style = Stroke(width = 2.6.dp.toPx(), cap = StrokeCap.Round, join = StrokeJoin.Round),
            )
        }

        drawSmoothCurve(buildPoints(downloadSeries), downColor)
        drawSmoothCurve(buildPoints(uploadSeries), upColor)
        val _unused = count
    }
}
