package io.honk.android

import android.Manifest
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import android.widget.Toast
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.ContextCompat
import io.honk.android.bridge.BoxEngineHolder
import io.honk.android.bridge.RustBridge
import io.honk.android.ui.BoxAppRoot
import org.json.JSONObject
import java.nio.charset.StandardCharsets
import java.util.concurrent.Executors
import java.util.concurrent.ScheduledFuture
import java.util.concurrent.TimeUnit

class MainActivity : ComponentActivity() {
    companion object {
        const val EXTRA_TARGET = "io.honk.android.EXTRA_TARGET"
    }

    private val ioExecutor = Executors.newSingleThreadScheduledExecutor()
    @Volatile
    private var pendingBackupScopeJson: String = "{}"
    private var pollFuture: ScheduledFuture<*>? = null

    private val requestNotifPermissionLauncher =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
            if (!granted) {
                Toast.makeText(this, "未授予通知权限", Toast.LENGTH_SHORT).show()
            }
        }

    private val exportBackupLauncher =
        registerForActivityResult(ActivityResultContracts.CreateDocument("application/zip")) { uri ->
            if (uri == null) return@registerForActivityResult
            val scopeBytes = pendingBackupScopeJson.toByteArray(StandardCharsets.UTF_8)
            ioExecutor.execute {
                val pfd = contentResolver.openFileDescriptor(uri, "w") ?: return@execute
                val res = RustBridge.nativeExportBackupToFd(
                    BoxEngineHolder.handle,
                    pfd.detachFd(),
                    scopeBytes,
                )
                val msg = String(res, StandardCharsets.UTF_8)
                runOnUiThread { Toast.makeText(this, msg, Toast.LENGTH_SHORT).show() }
            }
        }

    private val restoreBackupLauncher =
        registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
            if (uri == null) return@registerForActivityResult
            val scopeBytes = pendingBackupScopeJson.toByteArray(StandardCharsets.UTF_8)
            ioExecutor.execute {
                val pfd = contentResolver.openFileDescriptor(uri, "r") ?: return@execute
                val res = RustBridge.nativeRestoreBackupFromFd(
                    BoxEngineHolder.handle,
                    pfd.detachFd(),
                    scopeBytes,
                )
                val msg = String(res, StandardCharsets.UTF_8)
                runOnUiThread { Toast.makeText(this, msg, Toast.LENGTH_SHORT).show() }
            }
        }

    private val importCoreLauncher =
        registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
            if (uri == null) return@registerForActivityResult
            ioExecutor.execute {
                val pfd = contentResolver.openFileDescriptor(uri, "r") ?: return@execute
                val core = BoxEngineHolder.state.value
                    .optJSONObject("runtime_profile")
                    ?.optString("core_name", "honk")
                    ?.ifEmpty { "honk" } ?: "honk"
                val displayName = uri.lastPathSegment ?: core
                val res = RustBridge.nativeImportCoreFromFd(
                    BoxEngineHolder.handle,
                    core,
                    pfd.detachFd(),
                    displayName,
                )
                val msg = String(res, StandardCharsets.UTF_8)
                runOnUiThread { Toast.makeText(this, msg, Toast.LENGTH_SHORT).show() }
            }
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        handleIntent(intent)

        // Initial one-shot bootstrap probes (status + latency + IP geo)
        ioExecutor.execute {
            BoxEngineHolder.dispatchAction("refresh_status")
            BoxEngineHolder.dispatchAction("probe_latency_targets")
            BoxEngineHolder.dispatchAction("refresh_ip_geo")
        }

        setContent {
            BoxAppRoot(
                onLaunchBackupExport = { scope ->
                    pendingBackupScopeJson = scope.toString()
                    exportBackupLauncher.launch("boxapp.backup.zip")
                },
                onLaunchBackupRestore = { scope ->
                    pendingBackupScopeJson = scope.toString()
                    restoreBackupLauncher.launch(arrayOf("application/zip", "*/*"))
                },
                onLaunchImportCore = {
                    importCoreLauncher.launch(arrayOf("*/*"))
                },
                onRequestNotificationPermission = {
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                        val granted = ContextCompat.checkSelfPermission(
                            this,
                            Manifest.permission.POST_NOTIFICATIONS,
                        ) == PackageManager.PERMISSION_GRANTED
                        if (!granted) {
                            requestNotifPermissionLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
                        }
                    }
                },
            )
        }
    }

    override fun onStart() {
        super.onStart()
        pollFuture?.cancel(false)
        pollFuture = ioExecutor.scheduleWithFixedDelay(
            { runCatching { BoxEngineHolder.dispatchAction("refresh_status") } },
            1500L,
            2000L,
            TimeUnit.MILLISECONDS,
        )
    }

    override fun onStop() {
        super.onStop()
        pollFuture?.cancel(false)
        pollFuture = null
    }

    override fun onDestroy() {
        super.onDestroy()
        ioExecutor.shutdownNow()
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        handleIntent(intent)
    }

    private fun handleIntent(intent: Intent?) {
        val componentClass = intent?.component?.className.orEmpty()
        val explicitTarget = intent?.getStringExtra(EXTRA_TARGET)
        val target = when {
            componentClass.endsWith("BoxTilePreferencesWebUiActivity") -> "web_ui"
            componentClass.endsWith("BoxTilePreferencesAppActivity") -> "app"
            !explicitTarget.isNullOrEmpty() -> explicitTarget
            else -> return
        }
        if (target == "panel") {
            startActivity(
                Intent(this, PanelSheetActivity::class.java).apply {
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK)
                },
            )
            finish()
            return
        }
        BoxEngineHolder.dispatchAction("handle_launch_target") {
            put("target", target)
        }
    }
}
