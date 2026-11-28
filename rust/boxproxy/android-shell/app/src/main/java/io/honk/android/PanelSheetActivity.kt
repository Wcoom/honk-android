package io.honk.android

import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.view.WindowManager
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import io.honk.android.bridge.BoxEngineHolder
import io.honk.android.ui.LocalBoxPalette
import io.honk.android.ui.PanelQuickContent
import io.honk.android.ui.boxPaletteFor
import org.json.JSONObject

/**
 * M56 Bottom-anchored translucent quick panel sheet (`singleInstance`, `dimAmount=0.18f`, API 31+ blurBehind).
 */
class PanelSheetActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setFinishOnTouchOutside(true)
        window.addFlags(WindowManager.LayoutParams.FLAG_DIM_BEHIND)
        window.attributes = window.attributes.apply { dimAmount = 0.18f }
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
            window.addFlags(WindowManager.LayoutParams.FLAG_BLUR_BEHIND)
            window.attributes = window.attributes.apply { blurBehindRadius = 48 }
        }

        val openMainAndFinish: () -> Unit = {
            startActivity(
                Intent(this@PanelSheetActivity, MainActivity::class.java)
                    .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_REORDER_TO_FRONT),
            )
            finish()
        }

        setContent {
            val state by BoxEngineHolder.state.collectAsState()
            val appSettings = state.optJSONObject("prefs")?.optJSONObject("boxapp_settings") ?: JSONObject()
            val themeMode = appSettings.optInt("theme_mode", 0)
            val pureBlack = appSettings.optBoolean("pure_black_dark_theme", true)
            val isDark = when (themeMode) {
                1 -> false
                2 -> true
                else -> isSystemInDarkTheme()
            }
            val accentOverride = if (appSettings.optBoolean("accent_use_default", true)) {
                null
            } else {
                Color(appSettings.optInt("accent_color", 0xFF455A8A.toInt()))
            }
            val p = remember(isDark, pureBlack, accentOverride) {
                boxPaletteFor(isDark, pureBlack, accentOverride)
            }

            CompositionLocalProvider(LocalBoxPalette provides p) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .clickable(
                            interactionSource = remember { MutableInteractionSource() },
                            indication = null,
                        ) { finish() }
                        .padding(horizontal = 14.dp, vertical = 14.dp),
                    contentAlignment = Alignment.BottomCenter,
                ) {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .navigationBarsPadding()
                            .clip(RoundedCornerShape(28.dp))
                            .background(p.card)
                            .clickable(
                                interactionSource = remember { MutableInteractionSource() },
                                indication = null,
                            ) {}
                            .padding(20.dp),
                        verticalArrangement = Arrangement.spacedBy(14.dp),
                    ) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Text(
                                "BoxProxy Honk",
                                fontSize = 19.sp,
                                fontWeight = FontWeight.Medium,
                                color = p.textPrimary,
                            )
                            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                                Box(
                                    modifier = Modifier
                                        .clip(RoundedCornerShape(12.dp))
                                        .background(p.accentSoft)
                                        .clickable(onClick = openMainAndFinish)
                                        .padding(horizontal = 12.dp, vertical = 6.dp),
                                ) {
                                    Text("主应用", fontSize = 13.sp, color = p.accent, fontWeight = FontWeight.Medium)
                                }
                                Box(
                                    modifier = Modifier
                                        .clip(RoundedCornerShape(12.dp))
                                        .background(p.background)
                                        .clickable { finish() }
                                        .padding(horizontal = 12.dp, vertical = 6.dp),
                                ) {
                                    Text("关闭", fontSize = 13.sp, color = p.textSecondary)
                                }
                            }
                        }
                        PanelQuickContent(
                            state = state,
                            onRouteSelected = openMainAndFinish,
                        )
                    }
                }
            }
        }
    }
}
