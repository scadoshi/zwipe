# 326. Printings shows blank cards until their images load

State: fixing | Owner: thinker | PR: - | Build seen: 1.18.16

## Report
Composer, 2026-11-09, web and app: on Printings, a card with many printings (Sol Ring, Island) shows blank cards until their images load.

## Diagnosis
thinker, 2026-11-09. Printings is components' cover flow (`components/src/cover_flow.rs`) on both clients. Each card in it was a bare `img` with nothing behind it, so until its bytes arrived the card was empty. The flow already draws only the selected printing and ten on each side, with `loading="lazy"`, so printings further out are never fetched until the selection nears them.

## Fix
Each printing in the flow shows its own card-shaped ghost, pulsing like the other skeletons, until its image's `load` event. The image loads hidden under it, then fades in. The cards don't wait for one another. An image that fails shows the set-name card used for printings without art. An image already seen this session (the set `FlippableCardImage` keeps) shows at once.
