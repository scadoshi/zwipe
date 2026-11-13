# 477. Holding J with a card's details drawer open drops the frame rate

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: production

## Report
Scotty's video (composer, 2026-11-13), the wide deck page in Stacks with a card's details drawer open: holding J to run down the cards, the frame rate drops a lot, and the drawer shows each old card fading over the new.

## Before
Headless at 1440, a deck's Stacks, the drawer open, J held (a repeat every 33ms for 3s), three runs: 15.0, 11.0 and 14.6 frames a second (60 idle), a frame's p50 50ms and p95 183 to 467ms; only 34 to 49 of about 90 repeats got through, each key holding the main thread about 51ms.

## Cause
`components/src/drawer_swap.rs`: the drawer's crossfade captured on every keydown, its repeats too. Each capture cloned the drawer's title bar and whole content; each swap then laid the clones over the drawer (setting their `scrollTop`, which forces a layout) and ran a 200ms fade on them and on the live content, so at the repeat rate several fades of full drawer copies overlapped. A CPU profile of the hold: of about 1.7s busy on the main thread, its `play` took 1140ms; the cursor's key handler 38ms, the app's re-render about 95ms. The cursor's smooth `scrollIntoView` restarted on each step.

## Fix
scotland:
- A key repeat captures nothing and drops what still fades (ghosts removed, the live fade cancelled), so a held key swaps the card at once; a single press still crossfades.
- The cursor scrolls its card into view at once during a repeat (`__zwipeDeckHeld`), smoothly on a single press.

## Verification
- Headless, the same hold, three runs: 73, 74 and 74 repeats through; 21.8, 20.9 and 19.6 frames a second, p95 83 to 100ms; the crossfade's `play` 53ms; keydown dispatch about 7ms a key (51 before); the main thread idle 2.3s of the 3. The rest is headless software raster (its GPU thread busy 3.4s of 3s), which a real GPU doesn't share; the frame rate on a Mac is Scotty's check.
- Follow-up, List with a row open (the composer took it on): J held opened each next row inline, 1.4 to 7 frames a second headless over four runs, keydown dispatch about 46ms a key (872ms over 19 keys), so a 30-a-second repeat outruns the main thread. Turning every transition off left it at 4 to 5, so the row's ease is not the cost; the row's opening is (its details mount, then a commit of the whole list's paint). Fix: a held key's repeats move only the cursor, and the cursor's row opens once the key rests 150ms (`RowOpen`, `deck_cursor.rs`); a single press opens at once. After: dispatch about 14ms a key (466ms over 34), the cursor's card open when the key rests (3 of 3). Frames stay at 3 to 8 headless, even on llvmpipe, where Stacks reaches 26 to 37: with no row open at all List is 9 to 12, its commits waiting on raster (lazy art loading as it scrolls). Whether List's frame rate suffers on a real GPU is not checked here.
