#!/usr/bin/env bash
# Rewrite the dx-generated WryActivity.kt for the vendored tao 0.37.
#
# dx 0.7.10 copies wry 0.53.5's Kotlin activity, whose native entry points are
# `create`, `start`, `resume`, `pause`, `stop`, `save`, `destroy`, `memory`
# and `focus`: the names tao 0.34 exports. The vendored tao 0.37
# (vendor/tao/VENDOR.md, needed for iOS 27) exports `onFirstActivityCreate`,
# `onCreate`, `onStart`, `onResume`, `onPause`, `onStop`, `onDestroy`,
# `onWindowFocusChanged`, `onLowMemory` and `onNewIntent` instead, and its
# `onCreate` reads the activity's `id`. `onLowMemory` and `onNewIntent` are
# not declared here: Kotlin will not overload an Activity method with an
# external of the same name, and the app uses neither. With the stock file the first native
# call throws UnsatisfiedLinkError ("No implementation found for
# WryActivity.create") and the app dies in onCreate on every device. Play's
# pre-launch lab reported it as a 16 KB page-size failure on 1.10.3 vc46.
#
# dx REGENERATES this file on every `dx bundle`, so run it AFTER `dx bundle`
# and BEFORE the Gradle repackage, like the other patches. Delete this script
# with the vendored tao.
#
# Usage: zcripts/android/wry_activity.sh [WRY_ACTIVITY_KT]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
KT="${1:-$REPO_ROOT/target/dx/zwipe/release/android/app/app/src/main/kotlin/dev/dioxus/main/WryActivity.kt}"

[ -f "$KT" ] || { echo "WryActivity.kt not found: $KT" >&2; exit 1; }

cat > "$KT" <<'KOTLIN'
// Patched by zcripts/android/wry_activity.sh: tao 0.37's native entry points.

package dev.dioxus.main

import dev.dioxus.main.RustWebView
import android.annotation.SuppressLint
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.webkit.WebView
import android.view.KeyEvent
import androidx.annotation.Keep
import androidx.appcompat.app.AppCompatActivity

abstract class WryActivity : AppCompatActivity() {
    private lateinit var mWebView: RustWebView
    // Read by tao through JNI as getId(), which R8 cannot see, so it is kept
    // by annotation.
    @get:Keep
    var id: Int = 0
    open val handleBackNavigation: Boolean = true

    open fun onWebViewCreate(webView: WebView) { }

    fun setWebView(webView: RustWebView) {
        mWebView = webView
        onWebViewCreate(webView)
    }

    val version: String
        @SuppressLint("WebViewApiAvailability", "ObsoleteSdkInt")
        get() {
            // Check getCurrentWebViewPackage() directly if above Android 8
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                return WebView.getCurrentWebViewPackage()?.versionName ?: ""
            }

            // Otherwise manually check WebView versions
            var webViewPackage = "com.google.android.webview"
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
              webViewPackage = "com.android.chrome"
            }
            try {
                @Suppress("DEPRECATION")
                val info = packageManager.getPackageInfo(webViewPackage, 0)
                return info.versionName.toString()
            } catch (ex: Exception) {
                Logger.warn("Unable to get package info for '$webViewPackage'$ex")
            }

            try {
                @Suppress("DEPRECATION")
                val info = packageManager.getPackageInfo("com.android.webview", 0)
                return info.versionName.toString()
            } catch (ex: Exception) {
                Logger.warn("Unable to get package info for 'com.android.webview'$ex")
            }

            // Could not detect any webview, return empty string
            return ""
        }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        id = savedInstanceState?.getInt(ACTIVITY_ID_KEY) ?: intent.extras?.getInt(ACTIVITY_ID_KEY) ?: hashCode()
        onCreate(this)
        // Spawns the Rust main thread once per process, after the first
        // activity is registered.
        if (!firstActivityCreated) {
            firstActivityCreated = true
            onFirstActivityCreate()
        }
    }

    override fun onStart() {
        super.onStart()
        onStart(this)
    }

    override fun onResume() {
        super.onResume()
        onResume(this)
        if (::mWebView.isInitialized) {
            mWebView.onResume()
        }
    }

    override fun onPause() {
        super.onPause()
        onPause(this)
        if (::mWebView.isInitialized) {
            mWebView.onPause()
        }
    }

    override fun onStop() {
        super.onStop()
        onStop(this)
    }

    override fun onWindowFocusChanged(hasFocus: Boolean) {
        super.onWindowFocusChanged(hasFocus)
        onWindowFocusChanged(this, hasFocus)
    }

    override fun onSaveInstanceState(outState: Bundle) {
        super.onSaveInstanceState(outState)
        outState.putInt(ACTIVITY_ID_KEY, id)
    }

    override fun onDestroy() {
        super.onDestroy()
        onDestroy(this)
        onActivityDestroy(this)
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent?): Boolean {
        if (handleBackNavigation && keyCode == KeyEvent.KEYCODE_BACK && mWebView.canGoBack()) {
            mWebView.goBack()
            return true
        }
        return super.onKeyDown(keyCode, event)
    }

    fun getAppClass(name: String): Class<*> {
        return Class.forName(name)
    }

    // Called by tao through JNI when Rust opens another activity.
    @Keep
    fun startActivity(cls: Class<*>): Int {
        val intent = Intent(this, cls)
        val id = kotlin.random.Random.nextInt()
        intent.putExtra(ACTIVITY_ID_KEY, id)
        startActivity(intent)
        return id
    }

    companion object {
        const val ACTIVITY_ID_KEY = "dev.dioxus.main.ACTIVITY_ID"
        @Volatile private var firstActivityCreated = false

        init {
            System.loadLibrary("main")
        }
    }

    private external fun onFirstActivityCreate()
    private external fun onCreate(activity: WryActivity)
    private external fun onStart(activity: WryActivity)
    private external fun onResume(activity: WryActivity)
    private external fun onPause(activity: WryActivity)
    private external fun onStop(activity: WryActivity)
    private external fun onDestroy(activity: WryActivity)
    private external fun onActivityDestroy(activity: WryActivity)
    private external fun onWindowFocusChanged(activity: WryActivity, focus: Boolean)
}
KOTLIN

# R8 strips members only JNI calls. Keep the whole activity so tao's lookups
# by name (getId, startActivity, getLocalClassName) survive minification.
RULES="$(dirname "$KT")/../../../../../../proguard-rules.pro"
if [ -f "$RULES" ] && ! grep -q 'dev.dioxus.main.WryActivity' "$RULES"; then
  printf '\n# tao 0.37 reaches these through JNI; R8 cannot see the calls.\n-keep class dev.dioxus.main.WryActivity { *; }\n-keep class dev.dioxus.main.MainActivity { *; }\n' >> "$RULES"
fi

echo "Patched WryActivity.kt for tao 0.37 at $KT"
