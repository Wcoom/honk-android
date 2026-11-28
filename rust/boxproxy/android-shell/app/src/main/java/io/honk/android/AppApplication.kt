package io.honk.android

import android.app.Application
import android.content.ComponentName
import android.content.Intent
import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import android.net.ConnectivityManager
import android.net.NetworkCapabilities
import android.net.Uri
import android.net.wifi.WifiManager
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.service.quicksettings.TileService
import android.telephony.TelephonyManager
import android.widget.Toast
import androidx.core.content.ContextCompat
import io.honk.android.bridge.BoxEngineHolder
import io.honk.android.bridge.RustBridge
import io.honk.android.notification.BoxPersistentNotificationService
import io.honk.android.tile.BoxServiceTileService
import org.json.JSONArray
import org.json.JSONObject
import java.nio.charset.StandardCharsets
import java.util.Locale

class AppApplication : Application() {
    private val mainHandler = Handler(Looper.getMainLooper())

    override fun onCreate() {
        super.onCreate()
        BoxEngineHolder.initialize(
            filesDir = filesDir.absolutePath,
            cacheDir = cacheDir.absolutePath,
            packageName = packageName,
            appVersion = BuildConfig.VERSION_NAME,
            sdkInt = Build.VERSION.SDK_INT,
            localeTag = Locale.getDefault().toLanguageTag(),
            callback = createPlatformCallback(),
        )
        registerBundledAssets()
    }

    private fun registerBundledAssets() {
        val h = BoxEngineHolder.handle
        if (h == 0L) return
        for (path in listOf("boxctl/arm64-v8a/boxctl", "boxctl/arm64-v8a/boxbpf")) {
            runCatching {
                assets.open(path).use { input ->
                    RustBridge.nativeRegisterAsset(h, path, input.readBytes())
                }
            }
        }
    }

    private fun createPlatformCallback(): RustBridge.PlatformCallback =
        object : RustBridge.PlatformCallback {
            override fun onEffect(effectBytes: ByteArray) {
                BoxEngineHolder.emitEffect(effectBytes)
                val obj = runCatching {
                    JSONObject(String(effectBytes, StandardCharsets.UTF_8))
                }.getOrNull() ?: return
                when (obj.optString("effect")) {
                    "toast" -> {
                        val msg = obj.optString("message")
                        if (msg.isNotEmpty()) {
                            mainHandler.post {
                                Toast.makeText(this@AppApplication, msg, Toast.LENGTH_SHORT).show()
                            }
                        }
                    }
                    "refresh_tile" -> {
                        runCatching {
                            TileService.requestListeningState(
                                this@AppApplication,
                                ComponentName(this@AppApplication, BoxServiceTileService::class.java),
                            )
                        }
                    }
                    "sync_notification_service" -> {
                        val enabled = obj.optBoolean("enabled", false)
                        val intent = Intent(this@AppApplication, BoxPersistentNotificationService::class.java)
                        if (enabled) {
                            runCatching {
                                ContextCompat.startForegroundService(this@AppApplication, intent)
                            }
                        } else {
                            runCatching { stopService(intent) }
                        }
                    }
                    "open_panel_sheet" -> {
                        val intent = Intent(this@AppApplication, PanelSheetActivity::class.java).apply {
                            addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK)
                        }
                        mainHandler.post { runCatching { startActivity(intent) } }
                    }
                    "open_external_url" -> {
                        val url = obj.optString("url")
                        if (url.isNotEmpty()) openExternalUrl(url)
                    }
                }
            }

            override fun setComponentAlias(aliasName: String, enabled: Boolean) {
                runCatching {
                    val component = ComponentName(packageName, "io.honk.android.$aliasName")
                    val state = if (enabled) {
                        PackageManager.COMPONENT_ENABLED_STATE_ENABLED
                    } else {
                        PackageManager.COMPONENT_ENABLED_STATE_DISABLED
                    }
                    packageManager.setComponentEnabledSetting(
                        component,
                        state,
                        PackageManager.DONT_KILL_APP,
                    )
                }
            }

            override fun queryInstalledApps(): ByteArray {
                val arr = JSONArray()
                val pm = packageManager
                val pkgs = runCatching {
                    pm.getInstalledPackages(PackageManager.GET_PERMISSIONS)
                }.getOrDefault(emptyList())
                for (pkg in pkgs) {
                    val ai = pkg.applicationInfo ?: continue
                    val hasNet = pkg.requestedPermissions?.contains(android.Manifest.permission.INTERNET) == true
                    arr.put(
                        JSONObject()
                            .put("uid", ai.uid)
                            .put("package_name", pkg.packageName)
                            .put("label", runCatching { pm.getApplicationLabel(ai).toString() }.getOrDefault(pkg.packageName))
                            .put("is_system", (ai.flags and ApplicationInfo.FLAG_SYSTEM) != 0)
                            .put("has_internet_permission", hasNet),
                    )
                }
                return arr.toString().toByteArray(StandardCharsets.UTF_8)
            }

            override fun queryNetworkSnapshot(): ByteArray {
                val cm = getSystemService(ConnectivityManager::class.java)
                val caps = cm?.getNetworkCapabilities(cm.activeNetwork)
                val wifiConnected = caps?.hasTransport(NetworkCapabilities.TRANSPORT_WIFI) == true
                val cellConnected = caps?.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR) == true
                var ssid = ""
                var bssid = ""
                if (wifiConnected) {
                    runCatching {
                        val wm = applicationContext.getSystemService(WifiManager::class.java)
                        @Suppress("DEPRECATION")
                        val info = wm?.connectionInfo
                        ssid = info?.ssid?.trim('"').orEmpty()
                        bssid = info?.bssid.orEmpty()
                    }
                }
                var mccMnc = ""
                runCatching {
                    val tm = getSystemService(TelephonyManager::class.java)
                    mccMnc = tm?.networkOperator.orEmpty()
                }
                return JSONObject()
                    .put("wifi_connected", wifiConnected)
                    .put("wifi_ssid", ssid)
                    .put("wifi_bssid", bssid)
                    .put("cellular_connected", cellConnected)
                    .put("mcc_mnc", mccMnc)
                    .put("sim1_mcc_mnc", mccMnc)
                    .put("sim2_mcc_mnc", "")
                    .toString()
                    .toByteArray(StandardCharsets.UTF_8)
            }

            override fun hasPermission(permission: String): Boolean {
                return ContextCompat.checkSelfPermission(this@AppApplication, permission) ==
                    PackageManager.PERMISSION_GRANTED
            }

            override fun readAssetBytes(assetPath: String): ByteArray? {
                return runCatching {
                    assets.open(assetPath).use { it.readBytes() }
                }.getOrNull()
            }

            override fun openExternalUrl(url: String) {
                mainHandler.post {
                    runCatching {
                        val i = Intent(Intent.ACTION_VIEW, Uri.parse(url))
                            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                        startActivity(i)
                    }
                }
            }
        }
}
