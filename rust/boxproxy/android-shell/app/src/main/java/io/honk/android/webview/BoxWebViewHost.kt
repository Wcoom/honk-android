package io.honk.android.webview

import android.annotation.SuppressLint
import android.webkit.JavascriptInterface
import android.webkit.WebResourceRequest
import android.webkit.WebResourceResponse
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.viewinterop.AndroidView
import androidx.webkit.WebViewCompat
import androidx.webkit.WebViewFeature
import io.honk.android.bridge.BoxEngineHolder
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
) {
    val cs = MaterialTheme.colorScheme
    val palette = JSONObject()
        .put("is_dark", true)
        .put("primary", colorToHex(cs.primary.toArgb()))
        .put("on_primary", colorToHex(cs.onPrimary.toArgb()))
        .put("primary_container", colorToHex(cs.primaryContainer.toArgb()))
        .put("surface", colorToHex(cs.surface.toArgb()))
        .put("surface_container", colorToHex(cs.surfaceContainer.toArgb()))
        .put("on_surface", colorToHex(cs.onSurface.toArgb()))
        .put("outline", colorToHex(cs.outline.toArgb()))

    AndroidView(
        modifier = modifier,
        factory = { ctx ->
            WebView(ctx).apply {
                settings.javaScriptEnabled = true
                settings.domStorageEnabled = true
                settings.allowFileAccess = true
                addJavascriptInterface(ZashboardKsuBridge(), "ksu")
                if (WebViewFeature.isFeatureSupported(WebViewFeature.WEB_MESSAGE_LISTENER)) {
                    WebViewCompat.addWebMessageListener(
                        this,
                        "BoxAndroid",
                        setOf("*"),
                    ) { _, message, _, _, _ ->
                        val raw = message.data ?: return@addWebMessageListener
                        BoxEngineHolder.dispatchAction("handle_web_message") {
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
                            val css = BoxEngineHolder.resolveMonetCss(palette)
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

                    override fun onPageFinished(view: WebView?, loadedUrl: String?) {
                        super.onPageFinished(view, loadedUrl)
                        if (oneTimeToken.isNotEmpty()) {
                            val js = "(function(){try{var a=!!document.querySelector('#app');" +
                                "var t=(document.title||'').trim().toLowerCase();" +
                                "if(a&&t==='zashboard'&&window.BoxAndroid&&window.BoxAndroid.postMessage){" +
                                "window.BoxAndroid.postMessage(JSON.stringify({type:'zashboard-detected',pageUrl:location.href,token:'$oneTimeToken'}));" +
                                "}}catch(e){}})();"
                            view?.evaluateJavascript(js, null)
                        }
                    }
                }
                loadUrl(url)
            }
        },
        update = { webView ->
            if (url.isNotEmpty() && webView.url != url) {
                webView.loadUrl(url)
            }
        },
    )
}

private fun colorToHex(argb: Int): String = String.format("#%06X", 0xFFFFFF and argb)
