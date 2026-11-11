# 326. Printings shows blank cards until their images load

State: reproduced | Owner: scotland | PR: #503, #516, #531 | Build seen: 1.18.16

## Report
Composer, 2026-11-09, web and app: on Printings, a card with many printings (Sol Ring, Island) shows blank cards until their images load.

## Diagnosis
thinker, 2026-11-09. Printings is components' cover flow (`components/src/cover_flow.rs`) on both clients. Each card in it was a bare `img` with nothing behind it, so until its bytes arrived the card was empty. The flow already draws only the selected printing and ten on each side, with `loading="lazy"`, so printings further out are never fetched until the selection nears them.

## Fix
Each printing in the flow shows its own card-shaped ghost, pulsing like the other skeletons, until its image's `load` event. The image loads hidden under it, then fades in. The cards don't wait for one another. An image that fails shows the set-name card used for printings without art. An image already seen this session (the set `FlippableCardImage` keeps) shows at once.

## Verification
Droid, live aea5d98c4, 2026-11-10 (clips on the laptop, `~/Developer/droid-clips/2026-11-10-verify-aea5d/clips/326-*`): back to reproduced.
- Web, phone width in the emulator's Chrome: the center card's ghost goes about 0.58s before its image appears, leaving an empty slot.
- App: the center is an empty box for 2 to 3 frames, because the ghost goes on load and the image then fades in from transparent.

## Second fix
scotland, #516: each printing's image shows (its `shown` class) only once `img.decode()` resolves, and its ghost stays under it, the image painting over it, until the image's fade-in ends (`transitionend`), so no frame has neither. A ghost under a shown image stops pulsing, so one whose fade end never comes costs nothing. Failing-first test `the_ghost_stays_under_the_image_until_it_has_faded_in` (`components/src/cover_flow.rs`): a decoded image draws over its ghost, and no state leaves neither. Measured on web, 1260 wide, headless with each printing's image held 0.3 to 2s: release showed an empty card for one frame as each image arrived (5 in one opening); this branch 0. The app is not run on scotland.

## Phone width, third fix
Droid, live 21ca16a04 (with #516), emulator Chrome 145 at 411px, 2026-11-11: the center printing sat empty about 0.5s while the sides painted, then dropped out for a frame. Desktop passed. Cause, from droid's clips on tailnet candidates 2 to 7:
- An image that had loaded once this session skipped its ghost, so the center showed nothing while its own copy loaded, lazily, as the sheet slid in.
- A fading image, and the flow's stage (masked, its cards turned in 3D), each get a layer of their own, which mobile Chrome draws as the page's background until it has painted it, hiding whatever ghost was under them.

scotland, #531 (candidate 6):
- Every printing ghosts until its own image has loaded and decoded, and the ghost stays under its image for good. No image fades, and the middle loads at once.
- Until the middle image is in and has had three frames to paint (3s limit), the stage stays at 1% opacity, a steady layer (`will-change: opacity`), over a flat ghost outside the stage in the middle card's place.
- On the web, the stage's frame is filled with the sheet body's color.
- Failing-first test: `a_shown_image_always_has_its_ghost_under_it`.
- Droid at full speed, cache cleared, Sol Ring cold: the app passes (flat ghost from the first frame, then Sol Ring already drawn, no gap). On the web the center passes, but for about 2 frames (~50ms) the page behind the sheet shows through a side slot and over the header as the side printings arrive, down from about 0.35s on candidate 5.
- Candidate 7 (an opaque cover over the web stage instead of 1% opacity, and an isolated sheet) cleared that bleed but left the center empty for 260 to 340ms, so it is not in.
- The 2-frame web bleed is left as a follow-up.
