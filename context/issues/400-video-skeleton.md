# 400. zwipe.net's videos paint half-drawn while they load

State: fixing | Owner: laptop | PR: #612 | Build seen: zwipe.net live, 2026-11-12

## Report
From the composer: the guide pages' In the app panel (for example `/guides/swipe-memory/`) shows a half-painted image with a white block while it loads. Every video on zwipe.net should show a skeleton instead: its exact box reserved from its known size, so nothing shifts, filled with the existing loading skeleton, the media hidden until it can paint its first frame, then faded in. Match the feel of thinker's portfolio#50.

## Fix
laptop, #612: zite's `MediaFrame` wraps every framed media: the home page's Watch it work clips, the guide gallery's screenshots and the inline guide figure.
- **The box.** The frame takes the media's width and height (`DEMO_SIZE`, 540x1110; `guide_image_size`, 860x1862 except the 1236x1213 shared-page shot), so it holds the media's exact shape at its height cap from the first paint. Tests check the sizes against each MP4's track header and each WebP's header.
- **The skeleton.** Components' `.skeleton-bar` fills the frame: `--border-muted` with the 2s `skeleton-pulse`, as on portfolio#50.
- **The hold.** The media stays invisible, controls and all, until it is marked `data-ready`. A script in `index.html` sets the mark on an image's `load`, a video's `loadeddata`, or an `error` (so a broken file never pulses forever), and also marks media that finished before the script ran. The hold applies from the first paint while the page streams in. A `noscript` style lifts it when script is off.
- **The fade.** Once ready, the pulse stops and the media fades in over 0.25s (`gallery-fade-in`).

## Checked (headless Chrome, network throttled to 60 KB/s with 400ms latency)
- `/guides/swipe-memory/` at 1422: the frame is 249x540 from its first frame to the end. It pulses for about 6.5s, then the image is marked ready and fades from 0 to 1 over about 0.25s.
- `/` at 1422: the demo video's frame is 306x630 (70vh) throughout. It pulses until `loadeddata`, then fades in the same way.
- Gallery swap (getting-started, Next): the new screenshot's frame shows the skeleton at the same 249x540 until it loads.
- 390 wide: the home frame is 226x464 and the guide's is 234x506, with no horizontal scroll.

## Verification
-
