# Keep all JNI entry points and Rust -> JVM callbacks intact under R8
-keep class io.honk.android.bridge.RustBridge { *; }
-keep class io.honk.android.bridge.RustBridge$** { *; }
-keep interface io.honk.android.bridge.RustBridge$** { *; }
-keep class * implements io.honk.android.bridge.RustBridge$PlatformCallback { *; }
-keep class * implements io.honk.android.bridge.RustBridge$StateListener { *; }

# Keep WebView @JavascriptInterface methods
-keepclassmembers class io.honk.android.webview.ZashboardKsuBridge {
    @android.webkit.JavascriptInterface <methods>;
}
