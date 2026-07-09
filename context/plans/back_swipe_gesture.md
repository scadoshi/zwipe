# Edge back-swipe / hardware-back navigation (Android + iOS)

**Status: iOS + Android SHIPPED-READY 2026-07-09 (both device/emulator-tested,
instant). Premise corrected below — the original "unify via browser History API"
idea does NOT apply to mobile. Each platform ended up with its own native bridge
into one shared decision (`go_back` vs. exit); see the platform sections.**

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

## The shared decision (both platforms funnel here)

Both platforms end at the same Rust logic, keyed on `Navigator::can_go_back()`:
**if there's router history, `go_back()`; else exit (Android) / no-op (iOS).**
They differ only in how the OS back intent *reaches* that logic — iOS via an
objc2 channel, Android via a `zwipe:back` DOM event through the `document::eval`
bridge (`flush_loop.rs`'s `dioxus.send` + `eval.recv().await` idiom). All in
`components/navigation/back_handler.rs` under a `BackHandlerLayout` that wraps
every route (so it lives inside the router context and `use_navigator()`
resolves).

## Android — SHIPPED (MainActivity patch → JS event)

**Why a native patch is unavoidable:** wry's `WryActivity` (an
`AppCompatActivity`) handles only the hardware **button** in `onKeyDown`, gated
on `mWebView.canGoBack()` — always false for us (SPA, no WebView history) — and
the **edge-swipe gesture** never reaches `onKeyDown` at all; it goes through the
Activity's back dispatcher straight to the default `finish()`. tao *does* surface
`KEYCODE_BACK` as `Key::BrowserBack` to the event loop, but that's the button
path only, not the gesture. So catching the gesture requires Activity code.

**What shipped:** `zcripts/android/back_handler.sh` overwrites the dx-generated
`MainActivity.kt` (trivially `class MainActivity : WryActivity()`) with an
override that registers an `OnBackPressedCallback` on the **unified**
`onBackPressedDispatcher` — which catches **both** the gesture and the button —
and dispatches `window.dispatchEvent(new Event('zwipe:back'))` into the WebView
(reference captured via the overridable `onWebViewCreate`). The Rust listener
(`back_handler.rs`, `#[cfg(target_os = "android")]`) receives it and either
`go_back()`s or, at a root screen, finishes the Activity via JNI (`jni` +
`ndk-context`, same idiom as `outbound/open_url.rs`) to exit.

**Build integration (critical):** `dx` **regenerates `MainActivity.kt` on every
`dx bundle`/`dx serve`/`dx build`**, so the patch must run *after* the last dx
invocation and *before* the Gradle build — the same post-`dx bundle` window as
`launcher_icons.sh`. Documented in
[`../operations/android/play-store-submission/build-and-submit.md`](../operations/android/play-store-submission/build-and-submit.md).
Dev-loop gotcha: you can't verify via plain `dx serve` (it rebuilds and wipes the
patch); build through Gradle yourself after patching (`dx build` → `back_handler.sh`
→ `./gradlew :app:assembleDebug` → `adb install`). If `:app:compileDebugKotlin`
reports `UP-TO-DATE`, the patch didn't take (dx ran after it). R8 keeps the
callback/`evaluateJavascript` (both are used), but smoke-test the release build.

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

## Follow-ups / notes

- App-only (zwiper). zite (real browser) already has working browser back.
- Both verified 2026-07-09: iOS on a physical iPhone, Android on the Pixel_9a
  emulator (back arrow + edge swipe navigate back mid-stack, exit at root).
- Ships in the next store build (1.4.1+). The Android release AAB **must** run
  `back_handler.sh` in the build pipeline or it ships without the fix.
- Open refinements (not blocking): iOS interactive slide animation (parked, see
  above); Android "double-back-to-exit" instead of immediate finish at root.
- If wry [#1564](https://github.com/tauri-apps/wry/issues/1564) lands a
  native-forward API upstream, reconsider replacing the Android Kotlin patch.
