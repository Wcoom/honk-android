package io.honk.android.webview

import android.annotation.SuppressLint
import android.view.View
import android.webkit.JavascriptInterface
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.webkit.WebViewCompat
import androidx.webkit.WebViewFeature
import io.honk.android.bridge.BoxEngineHolder
import io.honk.android.ui.IosNavEasing
import io.honk.android.ui.LocalBoxPalette
import io.honk.android.ui.dispatchAsync
import kotlinx.coroutines.delay
import org.json.JSONObject
import java.io.ByteArrayInputStream
import java.nio.charset.StandardCharsets

/**
 * Safe Zashboard KernelSU compatibility shim (Section 4.2.4).
 * Intentionally exposes NO command execution methods.
 */
class ZashboardKsuBridge {
    @JavascriptInterface
    fun fullScreen(@Suppress("UNUSED_PARAMETER") enabled: Boolean) {}

    @JavascriptInterface
    fun enableInsets(@Suppress("UNUSED_PARAMETER") enabled: Boolean) {}

    @JavascriptInterface
    fun moduleInfo(): String = "{}"
}

@SuppressLint("SetJavaScriptEnabled")
@Composable
fun BoxWebViewHost(
    url: String,
    oneTimeToken: String,
    modifier: Modifier = Modifier.fillMaxSize(),
    reloadNonce: Int = 0,
    onScrollCollapse: (() -> Unit)? = null,
) {
    val cs = MaterialTheme.colorScheme
    val boxPalette = LocalBoxPalette.current
    val isDark = boxPalette.isDark
    // Match Doona's exact Rosé Pine base (#191724 dark / #FAF4ED light) so WebView never flashes white!
    val surfaceMatch = if (isDark) Color(0xFF191724) else Color(0xFFFAF4ED)
    val surfaceArgb = surfaceMatch.toArgb()

    var commitSeen by remember(url) { mutableStateOf(false) }
    var pageRevealed by remember(url) { mutableStateOf(false) }

    LaunchedEffect(commitSeen, url) {
        if (commitSeen) {
            // Wait 1–2 frames after Chromium's first visual commit so GPU tiles are painted
            delay(45)
            pageRevealed = true
        }
    }
    // Safety fallback in case a SPA hash navigation doesn't re-fire commit
    LaunchedEffect(url) {
        delay(900)
        pageRevealed = true
    }

    val webAlpha by animateFloatAsState(
        targetValue = if (pageRevealed) 1f else 0f,
        animationSpec = tween(240, easing = IosNavEasing),
        label = "webRevealAlpha",
    )
    val webScale by animateFloatAsState(
        targetValue = if (pageRevealed) 1f else 0.986f,
        animationSpec = tween(280, easing = IosNavEasing),
        label = "webRevealScale",
    )

    val palette = JSONObject()
        .put("is_dark", isDark)
        .put("primary", colorToHex(cs.primary.toArgb()))
        .put("on_primary", colorToHex(cs.onPrimary.toArgb()))
        .put("primary_container", colorToHex(cs.primaryContainer.toArgb()))
        .put("surface", colorToHex(cs.surface.toArgb()))
        .put("surface_container", colorToHex(cs.surfaceContainer.toArgb()))
        .put("on_surface", colorToHex(cs.onSurface.toArgb()))
        .put("outline", colorToHex(cs.outline.toArgb()))

    val latestPalette by rememberUpdatedState(palette)
    val latestIsDark by rememberUpdatedState(isDark)
    val latestToken by rememberUpdatedState(oneTimeToken)
    val latestOnScrollCollapse by rememberUpdatedState(onScrollCollapse)
    var webViewRef by remember { mutableStateOf<WebView?>(null) }
    var canGoBack by remember { mutableStateOf(false) }
    val lifecycleOwner = LocalLifecycleOwner.current

    // Navigate back inside WebView history before closing the entire WebUI route.
    BackHandler(enabled = canGoBack) {
        webViewRef?.goBack()
    }

    // Pause WebView JS timers, SSE/WebSocket activity & rendering whenever app goes to background or screen turns off!
    DisposableEffect(lifecycleOwner, webViewRef) {
        val wv = webViewRef
        val observer = LifecycleEventObserver { _, event ->
            when (event) {
                Lifecycle.Event.ON_PAUSE -> {
                    wv?.onPause()
                    wv?.pauseTimers()
                }
                Lifecycle.Event.ON_RESUME -> {
                    wv?.resumeTimers()
                    wv?.onResume()
                }
                else -> {}
            }
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose {
            lifecycleOwner.lifecycle.removeObserver(observer)
        }
    }

    Box(modifier = modifier.background(surfaceMatch)) {
        AndroidView(
            modifier = Modifier
                .fillMaxSize()
                .graphicsLayer {
                    alpha = webAlpha
                    scaleX = webScale
                    scaleY = webScale
                },
            factory = { ctx ->
                WebView(ctx).apply {
                    webViewRef = this
                    tag = reloadNonce
                    setBackgroundColor(surfaceArgb)
                    setLayerType(View.LAYER_TYPE_HARDWARE, null)
                    isVerticalScrollBarEnabled = false
                    settings.javaScriptEnabled = true
                    settings.domStorageEnabled = true
                    settings.allowFileAccess = true
                    addJavascriptInterface(ZashboardKsuBridge(), "ksu")
                    setOnScrollChangeListener { _, _, scrollY, _, oldScrollY ->
                        if (scrollY > oldScrollY + 18 && scrollY > 44) {
                            latestOnScrollCollapse?.invoke()
                        }
                    }
                    if (WebViewFeature.isFeatureSupported(WebViewFeature.WEB_MESSAGE_LISTENER)) {
                        WebViewCompat.addWebMessageListener(
                            this,
                            "BoxAndroid",
                            setOf("*"),
                        ) { _, message, _, _, _ ->
                            val raw = message.data ?: return@addWebMessageListener
                            dispatchAsync("handle_web_message") {
                                put("raw_json", raw)
                            }
                        }
                    }
                    webViewClient = object : WebViewClient() {
                        override fun shouldInterceptRequest(
                            view: WebView?,
                            request: WebResourceRequest?,
                        ): WebResourceResponse? {
                            val reqUrl = request?.url?.toString().orEmpty()
                            if (reqUrl.equals("https://mui.kernelsu.org/internal/colors.css", ignoreCase = true)) {
                                val css = BoxEngineHolder.resolveMonetCss(latestPalette)
                                return WebResourceResponse(
                                    "text/css",
                                    "utf-8",
                                    200,
                                    "OK",
                                    mapOf("Cache-Control" to "no-store"),
                                    ByteArrayInputStream(css.toByteArray(StandardCharsets.UTF_8)),
                                )
                            }
                            return super.shouldInterceptRequest(view, request)
                        }

                        override fun doUpdateVisitedHistory(view: WebView?, visitedUrl: String?, isReload: Boolean) {
                            super.doUpdateVisitedHistory(view, visitedUrl, isReload)
                            canGoBack = view?.canGoBack() == true
                        }

                        override fun onPageCommitVisible(view: WebView?, loadedUrl: String?) {
                            super.onPageCommitVisible(view, loadedUrl)
                            commitSeen = true
                        }

                        override fun onPageFinished(view: WebView?, loadedUrl: String?) {
                            super.onPageFinished(view, loadedUrl)
                            commitSeen = true
                            canGoBack = view?.canGoBack() == true
                            val schemeStr = if (latestIsDark) "dark" else "light"
                            view?.evaluateJavascript(
                                "(function(){try{if(!localStorage.getItem('doona.scheme')){document.documentElement.dataset.scheme='$schemeStr';}}catch(e){}})();",
                                null,
                            )
                            val tok = latestToken
                            if (tok.isNotEmpty()) {
                                val quotedTok = JSONObject.quote(tok)
                                val js = "(function(){try{var a=!!document.querySelector('#app');" +
                                    "var t=(document.title||'').trim().toLowerCase();" +
                                    "if(a&&t==='zashboard'&&window.BoxAndroid&&window.BoxAndroid.postMessage){" +
                                    "window.BoxAndroid.postMessage(JSON.stringify({type:'zashboard-detected',pageUrl:location.href,token:$quotedTok}));" +
                                    "}}catch(e){}})();"
                                view?.evaluateJavascript(js, null)
                            }
                        }
                    }
                    loadUrl(url)
                }
            },
            update = { webView ->
                webViewRef = webView
                webView.setBackgroundColor(surfaceArgb)
                val prevNonce = webView.tag as? Int ?: 0
                if (url.isNotEmpty() && webView.url != url) {
                    commitSeen = false
                    pageRevealed = false
                    webView.tag = reloadNonce
                    webView.loadUrl(url)
                } else if (reloadNonce != prevNonce) {
                    webView.tag = reloadNonce
                    webView.reload()
                }
            },
            onRelease = { webView ->
                webViewRef = null
                runCatching {
                    webView.stopLoading()
                    webView.onPause()
                    webView.loadUrl("about:blank")
                    webView.destroy()
                }
            },
        )

        AnimatedVisibility(
            visible = !pageRevealed,
            enter = fadeIn(tween(120)),
            exit = fadeOut(tween(220, easing = IosNavEasing)),
            modifier = Modifier.fillMaxSize(),
        ) {
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .background(surfaceMatch),
                contentAlignment = Alignment.Center,
            ) {
                Box(
                    modifier = Modifier
                        .size(44.dp)
                        .clip(CircleShape)
                        .background(boxPalette.card.copy(alpha = 0.75f)),
                    contentAlignment = Alignment.Center,
                ) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(22.dp),
                        strokeWidth = 2.2.dp,
                        color = boxPalette.accent,
                    )
                }
            }
        }
    }
}

private fun colorToHex(argb: Int): String = String.format("#%06X", 0xFFFFFF and argb)
