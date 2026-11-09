# 314. The menu sheet flickers when switching light/dark mode

State: verified | Owner: and-droid | PR: #474, #475 | Build seen: 1.18.14

## Report
Scotty, 2026-11-08, mobile app, signed in and signed out: open the hamburger menu sheet, tap the light/dark mode toggle. For a split second (a frame or two) the sheet disappears and the background shows through as the theme sweep starts, then it comes back. Happens both ways (light to dark and dark to light).

## Repro
Droid to capture: screenrecord at the highest frame rate the emulator allows (or adb shell screenrecord --bit-rate high, then step frames), toggle a few times each way, find the frames where the sheet vanishes, and note the timing relative to the sweep.

## Cause
and-droid, 2026-11-08. It's a one-frame WebKit glitch on the iPhone, at the moment the theme sweep's view transition captures the old page. On that frame the header layer, menu included, is drawn one top safe-area inset too low. Everything else stays put.

Evidence, from Scotty's 60fps recording (`droid-clips/2026-11-08-314/iphone.mp4`):

- Light to dark from the menu (frames 700 to 709): the mode icon flips at 700. Frame 703 draws the header and the open menu 58 video px lower than frames 702 and 704, about 61 pt, which is the iPhone's top inset. The content below moves 0 px. Frames 704 to 708 hold still, which is WebKit holding rendering during the update callback, and the sweep starts at 709. Frame 891 repeats it on the next toggle (41 px with the header mid-transition).
- The status bar strip above the displaced header shows bare page on that frame. That's the "sheet vanishes and the background shows through" Scotty sees. It happens on any theme change because every change runs the same `use_theme_follow` view transition (orangui `src/theme_wipe.rs:201`, `WIPE_JS` at :32). With the Themes sheet up, the backdrop dims the area, so it's less visible there.
- Only the header moves. It is the one layer placed by an inset offset: `.screen > .app-nav { position: absolute; top: env(safe-area-inset-top); z-index: 101 }` (`zwiper/assets/main.css:103`). The content takes its inset as padding (`components/assets/screens/base.css:156`), and it doesn't move.

Ruled out:

- Android WebView: Droid's 60fps take (`raw.mp4`) has no displaced frame in any of its toggles.
- Chrome and Playwright WebKit on zwiper-web: frozen at 0 to 120 ms, both snapshots hold the panel.
- The macOS desktop build (WebKit): the html class swap at `zwiper/src/bin/zwiper.rs:127` lands inside the update callback, before the new snapshot. The page never removes, rekeys or closes the menu.
- The iOS 26.5 simulator (iPhone 16e) runs the same app without the displaced frame, so it needs a device (or the Dynamic Island inset) to show. The fix has to be checked on Scotty's phone.

## Proposed fix
Stop placing the header with an inset offset, so there is nothing for the capture frame to apply twice. In `zwiper/assets/main.css`, `.screen > .app-nav` takes `top: 0` and `padding-top: env(safe-area-inset-top, 0px)` with the primary background, so the bar's box starts at the screen's top edge and its contents sit below the status bar as now. header_hide.js keeps working: it moves the bar by its own height, now including the inset, so `pushed` caps at the bar's height less the inset, keeping the strip covered. This is one CSS rule and a clamp in header_hide.js, checked on Scotty's phone in a release build: menu toggle both ways, a Themes sheet pick, and scroll-hide on a long screen.

If the device still shows the frame, the fallback is in orangui's `WIPE_JS`: give `.app-nav` a `view-transition-name` so WebKit captures it as its own group from its live box. Last resort: skip the view transition where `CSS.supports("-webkit-touch-callout", "none")` (iOS WebKit), so iPhones get the instant swap and no sweep.

## Verified
Scotty, 2026-11-08: plan B (#475, Themes sheet mounted at the app root like Cairn) fixes it on his iPhone. Rides 1.18.15.
