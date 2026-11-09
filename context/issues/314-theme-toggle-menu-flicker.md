# 314. The menu sheet flickers when switching light/dark mode

State: reported | Owner: - | PR: - | Build seen: 1.18.14

## Report
Scotty, 2026-11-08, mobile app, signed in and signed out: open the hamburger menu sheet, tap the light/dark mode toggle. For a split second (a frame or two) the sheet disappears and the background shows through as the theme sweep starts, then it comes back. Happens both ways (light to dark and dark to light).

## Repro
Droid to capture: screenrecord at the highest frame rate the emulator allows (or adb shell screenrecord --bit-rate high, then step frames), toggle a few times each way, find the frames where the sheet vanishes, and note the timing relative to the sweep.

## Cause
Not pinned yet. and-droid, 2026-11-08. What the code and a web repro rule out:

- No remount or key change. The toggle (`zwiper/src/lib/inbound/components/navigation/menu_theme.rs:118`) only sets the app-wide `ThemeConfig`. The menu's open state is `MenuOpen` at the root (`zwiper/src/bin/zwiper.rs:172`), and the panel class only follows it (orangui `src/nav_bar.rs:131`). Nothing in the switch path writes `open` or keys a node on the theme.
- The sweep is orangui's `use_theme_follow` (`src/theme_wipe.rs:201`, `WIPE_JS` at :32): a root view transition that swaps the theme class on `.app-shell` inside the update callback. There's no separate overlay, so z-order can't hide the panel. Both snapshots are flat images of the whole page.
- zwiper-web runs the same hook on the same `.app-shell` with the same NavBar. In headless Chrome and Playwright WebKit at 390px, with the transition frozen at 0, 16, 33, 66 and 120 ms both ways, the open panel is in the old and new snapshots every time. The DOM log shows the panel staying `nav-panel-open`, opacity 1, visible, full height through the whole swap.

So the flicker comes from something only the app has. The suspects, in order:

1. The app's header CSS. The panel drops out of an absolutely positioned header (`zwiper/assets/main.css:104`, `:159`), and the header's contents fade through `opacity: var(--nav-fade)` (`:144`). The web shell lays out differently.
2. `document.documentElement.className = <theme>` (`zwiper/src/bin/zwiper.rs:127`). It runs from an effect after `shown` commits, so it may land outside the transition's update callback and repaint html mid-sweep. zwiper-web sets custom properties on html instead (`zwiper-web/src/lib/inbound/app.rs:173`).
3. The native WebView's capture of the old snapshot (Android WebView or WKWebView) with the panel overflowing its header.

## Proposed fix
Pending the cause. If suspect 2 is it, move the html class swap into the transition: add `document.documentElement` to `WIPE_JS`'s swap, next to `.app-shell`, or do it before `commit()`. Then drop the effect.
