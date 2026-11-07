# Plan: long-running app health

Nit 151 in [`../progress/web_nits.md`](../progress/web_nits.md). Zwipe (zwiper) and Cairn both degrade when left open for hours, or backgrounded and resumed: a screen goes see-through (nit 150), taps stop landing (Cairn's counters), data goes stale (Cairn shows yesterday's count until a relaunch). Both are Dioxus 0.7.10 mobile apps in WKWebView over orangui. This is the diagnosis; no fix is built yet.

Method: code reading of zwiper, components, zwipe-ui, Cairn, orangui (zwipe's pin `4eb7a78`, Cairn's `23938a8`), and the shells both lock files resolve to: dioxus and dioxus-desktop 0.7.10, wry 0.53.5, objc2-web-kit 0.3.2, and the tao zwipe vendors (`vendor/tao`, 0.37, which Cairn takes from zwipe's git). Nothing below was measured on a device or in the simulator yet; the last section lists what to measure and how.

## What the evidence says, in order of likely impact

### 1. Nothing recovers a WebContent process that iOS killed (both apps: dead taps, blank or half-drawn screens)

iOS kills a backgrounded WKWebView's web content process under memory pressure. wry always installs a navigation delegate that implements `webViewWebContentProcessDidTerminate:` (`wry-0.53.5/src/wkwebview/class/wry_navigation_delegate.rs`, `navigation.rs:106-115`), but it only calls an optional handler, and dioxus-desktop 0.7.10 never sets one (no `with_on_web_content_process_terminate_handler` anywhere in its source). Because the delegate implements the selector, WebKit's own reload-on-crash does not happen either. The page stays dead.

A reload would not save it as things stand:

- dioxus allows exactly one `dioxus://` navigation (`dioxus-desktop-0.7.10/src/webview.rs:370-379`, `page_loaded.swap(true)`), so `reload()` is cancelled.
- If a fresh page did load, its `initialize` makes dioxus `rebuild()` the already-mounted VirtualDom without unmounting it, and the Rust side's record of templates it already sent (`dioxus-interpreter-js-0.7.10/src/write_native_mutations.rs`) means the new page gets references to templates it never received.
- The edits websocket's old connection is still marked connected until a write to it fails, so a new page is refused at first, and one batch of edits is reported applied though it never arrived (`dioxus-desktop-0.7.10/src/edits.rs:308-375`).

What the user sees: a page that no longer receives renders or sends events. Taps "stop landing"; the last frame, possibly mid-animation or mid-transition, stays up. This fits Cairn's dead counters best, and is one way a see-through or half-drawn screen gets stuck. Confidence: high on the code paths, medium on WebKit's exact behavior when the delegate implements the selector.

### 2. No foreground or day-rollover refresh (Cairn stale count; zwipe stale data)

Neither app, nor orangui, listens for a return to the foreground. tao emits `WindowEvent::Resumed` on `sceneWillEnterForeground` and `Focused(true)` on `sceneDidBecomeActive` (`vendor/tao/src/platform_impl/ios/scene.rs:100-123`), but dioxus-desktop's loop drops them (`launch.rs:30-37`, `_ => {}`). They reach app code only through `use_wry_event_handler` or `Config::with_custom_event_handler`, and neither app uses either. Neither app handles `visibilitychange` to visible, `pageshow` or window `focus`; zwipe's only visibility handler flushes telemetry on hide (`zwiper/src/lib/inbound/components/telemetry/flush_loop.rs:43-55`).

Cairn: "today" is `ui::today()` (`src/lib/inbound/ui/mod.rs:52-57`), read only during a render or a tap. Home loads counters once on mount (`home.rs:61-65`) and re-renders only when its own signals change; there is no midnight timer. After midnight, or a long background, Home keeps yesterday's date and figures until a tap, a navigation or a relaunch. A tap then works from stale numbers: `today_count` comes from the last render while the write uses a fresh day (`counter_list.rs:89-121`), so the first minus after the day turns is clamped against yesterday's count and can store a negative event (`src/lib/outbound/sqlite/mod.rs:350-363`).

Zwipe: Rust loops keep running in the background (session upkeep every 60 s, telemetry every 30 s), with tokio's default `MissedTickBehavior::Burst`, so missed ticks fire back to back on resume. Caches have time-to-live checks (counts 60 s on `Instant`, which stops while the device sleeps; flavor at the UTC hour; catalogs 24 h), but they are re-checked only when a screen mounts. A theme changed on another device shows only when Home loads (nit 144; the web now also checks on `visibilitychange`, PR #250).

### 3. The header fade makes the theme sheet see-through (nit 150, zwipe)

Not an accumulation, a missing reset, and it explains 150 on its own. `header_hide.js` writes `--nav-fade` (0 to 1) on the header as the screen scrolls, and `main.css` puts `opacity: var(--nav-fade)` on the header's `nav`. The Themes sheet renders inside that `nav` (the kit's NavBar holds the menu panel). The menu-open rule pins `--nav-fade: 1`; the sheet-open rule (`.screen > .app-nav:has(.theme-picker-sheet .bottom-sheet.show)`) pins only `transform`. Open the menu with the header part-faded (scroll down, then a little back up, as on a long Home), choose Themes, the menu folds, and the sheet takes the leftover fade. Fix ready on `nits/laptop-150`.

### 4. Things that pile up over a session (zwipe)

None of these is a smoking gun on its own; together they raise memory, which raises the odds of finding 1.

- An inline `<script>` per `GridCells` mount, never removed. dioxus de-duplicates `document::Script` only when it has a `src` (`dioxus-document-0.7.10/src/elements/script.rs:95-104`); `GridCells` inlines its script (orangui `src/grid_cells.rs:35`). The script guards itself, so it runs once, but every mount adds about 5 KB to `<head>`. Mount sites that repeat: the deck form's selects, Swipe select, the oracle tag dictionary and examples.
- Full-screen canvases kept mounted while hidden. The deck form always mounts four `SwipeSelect`, `TagSelect`, `OracleTagSelect` and `FormatSelect` (`deck_form.rs:400-470`), each with its own `GridCells` canvas at viewport × device pixel ratio (about 12 MB each at 3x), hidden only by `opacity: 0`. With the app's own layer that is eight at once on Create or Edit deck. The maybeboard and suggested commanders keep a `SwipeSelect` mounted too.
- iOS edge-swipe recognizers added on every run of `use_os_back`'s effect, unguarded, their targets leaked with `mem::forget` (orangui `src/os_back.rs:201-227`). It runs again whenever the router remounts (the upgrade gate flipping). Android's `zwipe:back` window listener has the same shape.
- Document-wide MutationObservers that never disconnect and run on every change: `header_hide.js` (writes the header's style each mutation), `grid-cells.js` (a rAF and `getComputedStyle` per layer on every class change anywhere), `scroll-fade.js` (where scroll timelines are missing, unguarded). CPU per change, not memory per se.
- `CountUp` re-renders every 16 ms for as long as its value is `None` while mounted (orangui `src/entrance.rs:111-118`): the hero's figures spin forever if their fetch never lands.
- Eval channels held by long-lived listeners are never freed, by design of dioxus's eval bridge; small.

Cairn shares the GridCells, edge-swipe and observer items, plus `entrance.js` (unguarded, but mounted once) and a 1 Hz `QuoteCard` re-render while Home is mounted.

### 5. What can sit over the screen and take taps

Mostly fine: sheet backdrops take taps only with `.show`, closed sheets go `visibility: hidden`, closed dialogs are `pointer-events: none`. Two to note: the orangui Splash's `.splash-out` fades with `opacity` only and keeps taking taps until its Rust timer unmounts it (`app.css:637-651`, `splash.rs:93-98`); and a theme wipe holds a `::view-transition` snapshot over the page until a Rust-JS handshake finishes (`theme_wipe.rs:37-70`). If the page is suspended or killed mid-wipe, that snapshot is the last frame left.

### 6. Screens have no fill of their own

`.screen` and `.screen-content` are transparent; the only backdrop is the fixed `GridCells` layer (and `body`, whose `--bg-primary` resolves to the `:root` default, Gruvbox dark, whatever the theme). Anything that stops that layer painting, or a dead page, shows the wrong color through every screen. Worth a solid fill on the theme root as a floor.

## Proposed fixes, to confirm before building

Shared (orangui, so both apps get them):

1. A lifecycle hook, `use_app_lifecycle` (or similar), over `use_wry_event_handler`: it reports `Resumed`/`Focused(true)` and `Suspended` as a signal both apps can watch, and also listens for `visibilitychange` in the page as a fallback.
2. Recovery from a killed web content process: on iOS, register the terminate handler through the raw `WKWebView`, which both apps already reach (`back_handler.rs`), wrapping wry's delegate rather than replacing it. Since dioxus 0.7.10 cannot re-initialize a fresh page, the reliable recovery is to restart: rebuild the window and VirtualDom, or at worst exit so the next tap relaunches cleanly. This needs a spike; it may belong upstream in dioxus.
3. Guard the inline GridCells script so it is inserted once (render it from the app root only, or check before inserting), and guard `install_edge_back` like `fill_window`.
4. A floor fill on the theme root, so a page that stops drawing its layer is never see-through.

Zwipe:

5. On resume: run session upkeep once at once, re-check the counts, flavor and catalog TTLs, and run the settings sync (nit 144's web version, in shared form). Set `MissedTickBehavior::Delay` on the upkeep and telemetry intervals.
6. Mount the deck form's selects only while open, as DrawerSheet mounts its children.
7. Nit 150's fix (ready).

Cairn (its own rules: straight to main, changelog entries also in the portfolio's `src/changelog.rs`):

8. A day signal that changes at the rollover hour and on resume, which Home and the counter screen read instead of calling `ui::today()` in render; reload counters on resume.
9. Read the day once per tap, and take `today_count` from the store at that day, not from the last render.
10. `ui::today()` calls `try_use_context` (a hook) from handlers and from inside `sort_by_key`; pass the preferences in instead.

## What to measure, and how

Release builds are not inspectable: dioxus enables devtools only when the context menu is enabled (`Config::with_disable_context_menu(false)`), which neither app sets. A debug build in the simulator is inspectable from Safari's Develop menu.

- Over 30 minutes on Home, then 30 on Create deck: Web Inspector's Timelines (JavaScript and events, Layout and rendering, Memory) and the Layers tab, plus `document.head.childElementCount` and `document.querySelectorAll('canvas').length` in the console before and after.
- Kill the content process: background the app in the simulator and run `xcrun simctl spawn booted launchctl list | grep WebContent` to find it, then kill that pid (or use Develop > Simulator > the app > Terminate Web Content Process if offered). Return to the app and tap: if taps do nothing, finding 1 is confirmed.
- Cairn rollover: set the simulator's clock across the rollover hour with the app open, or background it overnight.
- On Scotty's phone: whether a dead app comes back after the next tap, and whether the see-through state matches 150's steps.
