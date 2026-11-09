# 314. The menu sheet flickers when switching light/dark mode

State: reported | Owner: - | PR: - | Build seen: 1.18.14

## Report
Scotty, 2026-11-08, mobile app, signed in and signed out: open the hamburger menu sheet, tap the light/dark mode toggle. For a split second (a frame or two) the sheet disappears and the background shows through as the theme sweep starts, then it comes back. Happens both ways (light to dark and dark to light).

## Repro
Droid to capture: screenrecord at the highest frame rate the emulator allows (or adb shell screenrecord --bit-rate high, then step frames), toggle a few times each way, find the frames where the sheet vanishes, and note the timing relative to the sweep.
