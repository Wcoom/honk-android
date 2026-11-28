package io.honk.android.bridge

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import org.json.JSONObject
import java.nio.charset.StandardCharsets

/**
 * Ultra-thin JNI bridge to `libboxproxy_app.so` (100% Rust MVI engine).
 * Zero business logic lives on the JVM side.
 */
object RustBridge {
    init {
        System.loadLibrary("boxproxy_app")
    }

    interface PlatformCallback {
        fun onEffect(effectBytes: ByteArray)
        fun setComponentAlias(aliasName: String, enabled: Boolean)
        fun queryInstalledApps(): ByteArray
        fun queryNetworkSnapshot(): ByteArray
        fun hasPermission(permission: String): Boolean
        fun readAssetBytes(assetPath: String): ByteArray?
        fun openExternalUrl(url: String)
    }

    interface StateListener {
        fun onStateSnapshot(version: Long, stateBytes: ByteArray)
    }

    @JvmStatic
    external fun nativeInit(
        filesDir: String,
        cacheDir: String,
        packageName: String,
        appVersion: String,
        sdkInt: Int,
        initialLocaleTag: String,
        callback: PlatformCallback,
    ): Long

    @JvmStatic
    external fun nativeShutdown(handle: Long)

    @JvmStatic
    external fun nativeRegisterAsset(handle: Long, assetPath: String, bytes: ByteArray)

    @JvmStatic
    external fun nativeDispatch(handle: Long, actionBytes: ByteArray): ByteArray

    @JvmStatic
    external fun nativeSubscribeView(handle: Long, viewId: String, listener: StateListener): Long

    @JvmStatic
    external fun nativeUnsubscribeView(handle: Long, subscriptionId: Long)

    @JvmStatic
    external fun nativeQueryTileState(handle: Long): ByteArray

    @JvmStatic
    external fun nativeRenderNotification(handle: Long, forceRefresh: Boolean): ByteArray

    @JvmStatic
    external fun nativeResolveMonetCss(handle: Long, themePaletteJson: ByteArray): String

    @JvmStatic
    external fun nativeImportCoreFromFd(
        handle: Long,
        targetCore: String,
        fd: Int,
        displayName: String,
    ): ByteArray

    @JvmStatic
    external fun nativeExportBackupToFd(
        handle: Long,
        fd: Int,
        scopeMaskJson: ByteArray,
    ): ByteArray

    @JvmStatic
    external fun nativeRestoreBackupFromFd(
        handle: Long,
        fd: Int,
        scopeMaskJson: ByteArray,
    ): ByteArray
}

/**
 * Process-wide holder for the Rust engine handle and latest JSON state snapshot.
 */
object BoxEngineHolder {
    @Volatile
    var handle: Long = 0L
        private set

    private val _state = MutableStateFlow(JSONObject())
    val state: StateFlow<JSONObject> = _state.asStateFlow()

    private val _lastEffect = MutableStateFlow<JSONObject?>(null)
    val lastEffect: StateFlow<JSONObject?> = _lastEffect.asStateFlow()

    private var globalSubId: Long = 0L

    fun initialize(
        filesDir: String,
        cacheDir: String,
        packageName: String,
        appVersion: String,
        sdkInt: Int,
        localeTag: String,
        callback: RustBridge.PlatformCallback,
    ) {
        if (handle != 0L) return
        val h = RustBridge.nativeInit(
            filesDir,
            cacheDir,
            packageName,
            appVersion,
            sdkInt,
            localeTag,
            callback,
        )
        handle = h
        if (h != 0L) {
            globalSubId = RustBridge.nativeSubscribeView(
                h,
                "main",
                object : RustBridge.StateListener {
                    override fun onStateSnapshot(version: Long, stateBytes: ByteArray) {
                        runCatching {
                            val json = JSONObject(String(stateBytes, StandardCharsets.UTF_8))
                            _state.value = json
                        }
                    }
                },
            )
        }
    }

    fun emitEffect(effectBytes: ByteArray) {
        runCatching {
            _lastEffect.value = JSONObject(String(effectBytes, StandardCharsets.UTF_8))
        }
    }

    fun dispatch(action: JSONObject): JSONObject {
        val h = handle
        if (h == 0L) return JSONObject().put("ok", false).put("error", "engine not initialized")
        val raw = RustBridge.nativeDispatch(
            h,
            action.toString().toByteArray(StandardCharsets.UTF_8),
        )
        return runCatching { JSONObject(String(raw, StandardCharsets.UTF_8)) }
            .getOrElse { JSONObject().put("ok", false).put("error", it.message ?: "decode error") }
    }

    fun dispatchAction(name: String, block: JSONObject.() -> Unit = {}): JSONObject {
        val obj = JSONObject().put("action", name).apply(block)
        return dispatch(obj)
    }

    fun queryTileState(): JSONObject {
        val h = handle
        if (h == 0L) return JSONObject()
        val raw = RustBridge.nativeQueryTileState(h)
        return runCatching { JSONObject(String(raw, StandardCharsets.UTF_8)) }.getOrDefault(JSONObject())
    }

    fun renderNotification(forceRefresh: Boolean): JSONObject {
        val h = handle
        if (h == 0L) return JSONObject()
        val raw = RustBridge.nativeRenderNotification(h, forceRefresh)
        return runCatching { JSONObject(String(raw, StandardCharsets.UTF_8)) }.getOrDefault(JSONObject())
    }

    fun resolveMonetCss(palette: JSONObject): String {
        val h = handle
        if (h == 0L) return ""
        return RustBridge.nativeResolveMonetCss(
            h,
            palette.toString().toByteArray(StandardCharsets.UTF_8),
        )
    }
}
