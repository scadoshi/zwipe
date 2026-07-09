#!/usr/bin/env bash
# Patch the dx-generated MainActivity.kt so the Android OS back intent
# (edge-swipe gesture AND hardware button) drives the Dioxus router instead of
# finishing the app.
#
# Why a patch: zwiper's router is native-Rust in-memory history, and the OS back
# never reaches it — wry's Activity finishes the app (wry issue #1564). The
# unified OnBackPressedDispatcher catches both the gesture and the button; we
# forward it to the app as a `zwipe:back` DOM event, and the Rust side
# (components/navigation/back_handler.rs) decides: go_back, or finish the
# Activity at a root screen.
#
# dx REGENERATES MainActivity.kt on every `dx bundle`, so run this AFTER
# `dx bundle` and BEFORE the Gradle repackage — the same window as
# launcher-icons.sh. See
# context/operations/android/play-store-submission/build-and-submit.md and
# context/plans/back_swipe_gesture.md.
#
# Usage: zcripts/android/back-handler.sh [MAIN_ACTIVITY_KT]
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
KT="${1:-$REPO_ROOT/target/dx/zwipe/release/android/app/app/src/main/kotlin/dev/dioxus/main/MainActivity.kt}"

[ -f "$KT" ] || { echo "MainActivity.kt not found: $KT" >&2; exit 1; }

cat > "$KT" <<'EOF'
package dev.dioxus.main

import android.os.Bundle
import android.webkit.WebView
import androidx.activity.OnBackPressedCallback

typealias BuildConfig = com.scadoshi.zwipe.BuildConfig

class MainActivity : WryActivity() {
    private var appWebView: WebView? = null

    // WryActivity hands us the WebView on creation; keep a reference so the back
    // callback can dispatch into it.
    override fun onWebViewCreate(webView: WebView) {
        super.onWebViewCreate(webView)
        appWebView = webView
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Intercept the OS back intent (edge-swipe gesture AND hardware button)
        // via the unified dispatcher and hand it to the Dioxus router by
        // dispatching a DOM event the app listens for. Always enabled: the Rust
        // side owns the decision (navigate back, or finish this Activity to exit
        // from a root screen).
        onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() {
                appWebView?.evaluateJavascript(
                    "window.dispatchEvent(new Event('zwipe:back'))",
                    null
                )
            }
        })
    }
}
EOF

echo "Patched back-navigation into $KT"
