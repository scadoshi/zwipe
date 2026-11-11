# 326. Printings shows blank cards until their images load

State: reproduced | Owner: scotland | PR: #503, #516 | Build seen: 1.18.16

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
