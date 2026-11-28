package io.honk.android

import android.content.Intent
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import io.honk.android.bridge.BoxEngineHolder
import io.honk.android.bridge.RustBridge
import java.nio.charset.StandardCharsets
import java.util.concurrent.Executors

/**
 * M38 External file import activity passing zero-copy `fd` directly to Rust ELF64 AArch64 verifier.
 */
class CoreImportActivity : ComponentActivity() {
    private val ioExecutor = Executors.newSingleThreadExecutor()
    private val cores = listOf("honk", "mihomo", "meow", "sing-box", "xray", "v2fly", "hysteria")

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setFinishOnTouchOutside(false)
        val uri: Uri? = when (intent?.action) {
            Intent.ACTION_VIEW -> intent.data
            Intent.ACTION_SEND -> if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                intent.getParcelableExtra(Intent.EXTRA_STREAM, Uri::class.java)
            } else {
                @Suppress("DEPRECATION")
                intent.getParcelableExtra(Intent.EXTRA_STREAM)
            }
            else -> null
        }

        setContent {
            var selectedCore by remember { mutableStateOf("honk") }
            MaterialTheme(colorScheme = darkColorScheme()) {
                Surface(
                    shape = RoundedCornerShape(20.dp),
                    modifier = Modifier.fillMaxWidth().padding(20.dp),
                ) {
                    Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                        Text("导入核心二进制（Rust ELF64 校验）", style = MaterialTheme.typography.titleMedium)
                        Text(uri?.lastPathSegment ?: "未选择文件", style = MaterialTheme.typography.bodySmall)
                        Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                            cores.chunked(4).forEach { row ->
                                Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                                    row.forEach { core ->
                                        FilterChip(
                                            selected = selectedCore == core,
                                            onClick = { selectedCore = core },
                                            label = { Text(core) },
                                        )
                                    }
                                }
                            }
                        }
                        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                            TextButton(onClick = { finish() }) { Text("取消") }
                            Button(enabled = uri != null, onClick = {
                                val targetUri = uri ?: return@Button
                                ioExecutor.execute {
                                    val pfd = contentResolver.openFileDescriptor(targetUri, "r") ?: return@execute
                                    val res = RustBridge.nativeImportCoreFromFd(
                                        BoxEngineHolder.handle,
                                        selectedCore,
                                        pfd.detachFd(),
                                        targetUri.lastPathSegment ?: selectedCore,
                                    )
                                    val msg = String(res, StandardCharsets.UTF_8)
                                    runOnUiThread {
                                        Toast.makeText(this@CoreImportActivity, msg, Toast.LENGTH_LONG).show()
                                        finish()
                                    }
                                }
                            }) { Text("校验并安装") }
                        }
                    }
                }
            }
        }
    }
}
