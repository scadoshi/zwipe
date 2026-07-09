# Edge back-swipe navigation (Android + iOS)

**Status: PLANNING (needs research before building) — raised by owner 2026-07-09.**

**One sentence:** dragging from the left screen edge should navigate back one
screen (when there's router history), instead of the current broken behavior.

## Current behavior (the bug)

- **Android:** left-edge swipe fires the OS **back gesture**, which the app's
  Activity handles by **finishing (closing) the app** — it never reaches the
  Dioxus router. Deeply wrong: a user two screens in loses the whole app.
- **iOS:** left-edge swipe **does nothing**. WKWebView's
  `allowsBackForwardNavigationGestures` defaults off, and even on it would
  drive *WebView* page history, not the app.

## The core challenge

zwiper is a native WebView wrapping a **Dioxus SPA** — one web page, all
navigation is the **client-side Dioxus router** (`navigator.go_back()`, what
the on-screen Back buttons already call). The native/WebView "back" (Android
hardware/gesture, iOS WebView history) is a **separate stack** that the
Dioxus router doesn't observe. So the fix isn't "enable the gesture" — it's
"route the platform's back intent into `navigator.go_back()`, and no-op /
exit only when there's no router history."

## Candidate approaches (to validate)

**Android**
- Intercept the back gesture via the Activity's `OnBackPressedDispatcher`
  (modern) or `onBackPressed` in the generated Kotlin `MainActivity`, and
  instead of finishing, signal the WebView to run the router's back.
- Bridge to the SPA: native → JS (`evaluateJavascript`) calling a handler the
  Dioxus app registers, which calls the router back; if router history is
  empty, fall through to the default (exit).
- Watch out: dx **regenerates the Android Gradle/manifest/Activity on every
  `dx bundle`** (see `operations/android/play-store-submission/build-and-submit.md`),
  so anything in the generated `MainActivity` must be re-applied or injected
  via a mechanism dx won't clobber. Prefer a Dioxus-side hook if one exists.

**iOS**
- Enabling `allowsBackForwardNavigationGestures` alone won't help (SPA = empty
  WebView history). Either (a) mirror router navigations into the browser
  History API (`pushState`) so WebView back/forward maps to router back and
  the gesture "just works," or (b) add a custom left-edge pan recognizer that
  calls the router back.

**Unifying option worth checking first:** if the Dioxus 0.7 mobile router can
be made to use the **browser History API** (observe `popstate`), then *both*
the Android hardware back and the iOS edge gesture reduce to `history.back()`
— one mechanism, no per-platform native code. Verify whether the mobile
router already integrates popstate or can be configured to.

## Open questions (resolve during research)

1. Does Dioxus 0.7 expose a back-press / hardware-back hook for mobile, or must
   we touch the native wrappers?
2. Does the Dioxus mobile router observe `popstate` / use the History API? If
   yes, the unifying option is the whole fix.
3. How to inject Android Activity changes so `dx bundle` doesn't wipe them
   (analogous to `launcher-icons.sh` / the gradle patch step)?
4. "No history" behavior: Android should still be able to exit the app from a
   root screen (double-back-to-exit?); iOS gesture should just do nothing.

## Scope / notes

- Native-shell + router integration work — **not** a CSS/markup change.
- App-only (zwiper); zite/browser already has real history + browser back.
- Lands in a future store build (1.4.1+). Test on both a physical Android
  device and iOS, since the gesture is OS-level and emulator/simulator
  behavior can differ.
