# A fallback for a killed web page in the apps

**Status: parked, 2026-11-06.** Build it only if page kills still happen after the long-running health fixes ([`long_running_app_health.md`](long_running_app_health.md), on `nits/laptop-151` until it merges). Scotty is researching the Dioxus side himself and will file anything upstream himself. No agent files issues or PRs with Dioxus or wry.

## The problem

iOS kills a backgrounded WKWebView's web content process under memory pressure. wry implements `webViewWebContentProcessDidTerminate:` but only calls an optional handler, and dioxus-desktop 0.7.10 never sets one. Because wry implements the selector, WebKit's own reload-on-crash is also skipped. The page stays dead: the last frame stays on screen (sometimes half-drawn and see-through), and taps go nowhere.

A plain reload would not save it either. dioxus allows one `dioxus://` navigation per webview, so `reload()` gets cancelled. Even if a fresh page loaded, dioxus can't redraw the running VirtualDom into it, because it assumes the page already has state that only the first page received.

This hits zwipe (zwiper) and Cairn alike. Both apps run on the same shell.

## The fallback

When the page dies, show the user a clean way back instead of a frozen screen.

1. **Detect the kill.** On iOS, register a terminate handler on the raw `WKWebView`, which both apps already reach for the back-swipe (`zcripts/android/back_handler.sh`'s iOS twin, `back_handler.rs`). Wrap wry's navigation delegate rather than replacing it, so wry's own handling keeps working. On Android, look for the matching signal: `onRenderProcessGone`, which wry may expose differently.
2. **Cover the dead page natively.** Put a native view over the webview (UIKit on iOS), not HTML, since the page can't draw. It holds the app's backdrop color, the logo, one line ("Zwipe needs to reload") and a Reload button. Use the theme's colors if the native side can read them cheaply. Otherwise use a neutral dark.
3. **Reload.** In order of preference:
   - Rebuild the webview and the VirtualDom in place. That needs a spike, and may not be possible on dioxus 0.7.10 without patching it.
   - Failing that, exit so the next tap relaunches cleanly (`exit(0)` after the user taps Reload). Apple discourages an app quitting on its own, but quitting on the user's tap from an error screen is the accepted pattern.
4. **Keep state safe.** Anything the user had in flight (an unsaved deck form, a Cairn counter tap) is either already on the server or in local storage. Check that a relaunch loses nothing the user would expect kept.

## Where it lives

In orangui, if it can stay framework-level: a `use_page_kill_fallback()` or a config hook that both apps call once at launch, plus the native overlay. Otherwise in each app's shell. Cairn's copy follows Cairn's rules: it goes straight to `main`, with changelog entries in Cairn and in the portfolio's `src/changelog.rs`.

## How to test

In the simulator, background the app, find the WebContent process (`xcrun simctl spawn booted launchctl list | grep WebContent`), kill it, and return. Without the fallback, taps do nothing. With it, the overlay shows and Reload brings the app back on the same screen or on Home. On Android, `adb shell am kill` doesn't target the renderer. Find the renderer's pid and kill it.

## Open questions

- Can dioxus 0.7.10 rebuild a webview without a fork, or does that wait on upstream? (This is Scotty's research.)
- Should the overlay show at once, or only on the first tap after a resume? At once is simpler and honest.
