# Edge back-swipe / hardware-back navigation (Android + iOS)

**Status: iOS SHIPPED-READY 2026-07-09 (device-tested, instant). Android next.
Premise corrected below — the original "unify via browser History API" idea does
NOT apply to mobile.**

**One sentence:** the OS back intent (Android hardware/gesture back, iOS
left-edge swipe) should call the Dioxus router's `go_back()` when there's
history, instead of the current broken behavior.

## Current behavior (the bug)

- **Android:** the OS back gesture reaches wry's generated Activity, which
  **finishes (closes) the app** — it never reaches the Dioxus router. A user
  two screens deep loses the whole app.
- **iOS:** left-edge swipe **does nothing** (WKWebView
  `allowsBackForwardNavigationGestures` defaults off, and its back-forward list
  is empty anyway — see below).

## Corrected architecture (what the research changed)

The old plan floated a "unifying option": make the router use the browser
History API so Android back and iOS edge-swipe both reduce to `history.back()`.
**That does not work on mobile.** Confirmed facts:

- zwiper on mobile is **not a wasm SPA in a WebView.** Dioxus mobile
  (`dioxus-desktop`/`wry`) runs the app as **native Rust**; the WKWebView /
  Android WebView is only a *renderer* driven by an IPC bridge (Rust streams DOM
  edits in; the WebView POSTs user events back to `/__events`). Source:
  Dioxus mobile-renderer architecture.
- The router therefore runs Rust-side on the **in-memory history**
  (`MemoryHistory` — no explicit backend is set in `router.rs`, and the native
  webview default is in-memory; there is no URL bar). There is **no browser
  `popstate` / URL history** driving navigation. `history.back()` in JS does
  nothing to the router.
- All "back" today is the 8 on-screen Back buttons calling
  `navigator.go_back()` (register, forgot_password, profile, privacy, deck
  create, card remove/add/view). No hardware/gesture handling anywhere.

**Versions:** `dioxus 0.7.9` (router feature), `wry 0.53.5`, `tao 0.34.8`,
default build feature `mobile`. wry issue
[#1564](https://github.com/tauri-apps/wry/issues/1564) ("Android: add Back
button listener", opened 2025-06) is **still open** — wry does not forward the
Android back press. So upstream won't save us short-term; watch that issue.

## The one shared piece (both platforms funnel here)

A single app-scoped Rust listener that turns a "back intent" into router action,
reusing the **exact `document::eval` bridge idiom already in the codebase**
(`components/telemetry/flush_loop.rs:47-58` registers JS listeners and pumps
events to Rust via `dioxus.send(...)` + `eval.recv().await`):

```rust
// mounted once near the Router (e.g. in App / a BackHandler component)
let nav = use_navigator();
use_future(move || async move {
    let mut eval = document::eval(
        "window.addEventListener('zwipe:back', () => dioxus.send(true));");
    while eval.recv::<bool>().await.is_ok() {
        if nav.can_go_back() { nav.go_back(); }
        else { /* root: Android -> finish via JNI; iOS -> no-op */ }
    }
});
```

`Navigator::can_go_back()` gates root behavior. The remaining work is
**platform-specific: how does the OS back intent dispatch `zwipe:back` (or
otherwise reach this listener)?** The two platforms differ.

## Android

**Problem:** intercept the OS back before wry finishes the Activity, and route
it to the listener above.

- **No native source is checked in.** The Activity is dx's stock
  `dev.dioxus.main.MainActivity`, and `dx bundle` **regenerates the whole
  Gradle project every time** (it already needs post-bundle patches for
  `targetSdk`/`versionCode` and `launcher-icons.sh` for the icons). So any
  Kotlin change must ride a **post-bundle patch script**, mirroring
  `zcripts/android/launcher-icons.sh`.
- **Likely approach (NativePHP pattern):** patch `MainActivity` so its back
  callback runs
  `webView.evaluateJavascript("window.dispatchEvent(new Event('zwipe:back'))")`
  instead of `finish()`. The Rust listener consumes it; at root (`!can_go_back`)
  Rust exits the app via the **existing JNI bridge** (`jni 0.21` +
  `ndk-context 0.1` are already deps, used for session storage / intents) by
  calling `Activity.finish()`.
- **Spike question #1 (decides everything):** does wry 0.53's Android Activity
  already forward back to `webView.goBack()` when the WebView `canGoBack()`? If
  it does, a **pure-JS solution needs no native patch**: mirror each router
  nav into WebView history with `history.pushState`, and the system back pops
  it and fires `popstate` → `go_back()`. The bug report ("closes the app")
  suggests it does *not* forward, but confirm by reading wry 0.53 Android source
  / a 10-line pushState test before committing to the patch route.
- **Watch out:** R8 minification is on in release and can strip WebView/JNI
  classes; the patch must survive it. And the patch must re-apply on every
  bundle (script it, don't hand-edit).

## iOS — SHIPPED (custom recognizer)

No regeneration problem for runtime code — iOS native calls are done in-Rust via
`objc2 0.6` (already a dep, used for `UIApplication.openURL`).

**What we tried first and rejected:** `WKWebView.allowsBackForwardNavigation
Gestures = true` + a same-document `history.pushState` "trap" + a `popstate`
listener. It *worked* (device-confirmed) but was **~3s slow vs. an instant
on-screen Back**. Root cause is architectural, not fixable: in a single-page
Dioxus app the "previous screen" was never a separate page load, so WKWebView has
no snapshot to animate its native back-transition toward. The parallax animator
gets nothing (`interruptibleAnimatorForTransition ... returned nil`), falls back
to a degraded transition, and — the killer — `popstate` (our `go_back` trigger)
only fires *after* that transition commits and settles. So a free, WebView-native
interactive slide is impossible here.

**What shipped:** a custom `UIScreenEdgePanGestureRecognizer` (left edge, via
`objc2` with a Rust-declared target class — no Swift, no regeneration issue)
attached to the WKWebView. On `STATE_BEGAN` it sends on a `tokio` channel; a
Dioxus task drains it into `nav.go_back()` (guarded by `can_go_back()`), skipping
WebView history entirely. Result: instant, same speed as the Back button. No
`pushState` trap, no `allowsBackForwardNavigationGestures`, no `popstate`. Lives
in `components/navigation/back_handler.rs::ios`. At root: no-ops (never exits).

**Known tradeoff (parked by owner 2026-07-09):** no finger-tracking slide
animation — it's a trigger → instant swap, not an interactive transition. A
custom *app-drawn* slide (recognizer reports `translationInView` → CSS-transform
the route content → commit/spring-back on release) could be both interactive and
instant, but it's a real feature, not a tweak. Revisit only if the missing slide
starts to bother.

## Root-screen behavior & gotchas

- **Root (no history):** Android exits (JNI `finish()`), iOS no-ops. Gate on
  `nav.can_go_back()`.
- **Auth bounce:** `components/auth/bouncer.rs` pushes `Login` via `use_effect`
  when the session is missing. Backing into a protected route can bounce forward
  to Login — verify the back stack doesn't create a Login/loop trap when
  combined with programmatic `go_back()`.
- **Screens with their own state:** filter screens now park/restore via
  `FilterStore` (see [`filter_persistence.md`](filter_persistence.md)) and the
  add-stack cache — `go_back()` unmounts them and fires their `use_drop` park,
  same as the on-screen Back button, so persistence is unaffected. Good: the
  gesture should feel identical to tapping Back.

## Remaining work (Android)

1. ~~iOS~~ — **DONE** (custom recognizer, see above).
2. **Android wry-forwarding probe** — determine whether wry forwards the OS back
   to the WebView (read source or a pushState test). Result picks the Android
   path (pure-JS `popstate` vs. a MainActivity patch that dispatches `zwipe:back`
   / bridges to Rust).
3. **Android implementation** — whichever path #2 selects; if the patch route,
   write the post-bundle script alongside `launcher-icons.sh` and document it in
   `operations/android/play-store-submission/build-and-submit.md`. Add root-screen
   exit (JNI `finish()` via the existing `jni`/`ndk-context` bridge) since Android
   should close from a root screen, unlike iOS.

## Scope / notes

- App-only (zwiper). zite (real browser) already has working browser back.
- Test on a **physical Android device** (gesture is OS-level; emulator differs)
  and a physical iPhone.
- Lands in a future store build (1.4.1+). Not blocking the current build.
- If wry #1564 lands upstream before we build Android, prefer the native-forward
  API over the patch script.
