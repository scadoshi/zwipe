# Stacks probe

`stacks_probe.py` measures the deck page's Stacks on zwipe web at a phone's width (390), with the CPU throttled 4x as a stand-in for a phone, on a mocked 100-card deck in 7 stacks. Run it before and after any change to the stacks, the settle animator or the card faces.

```bash
cd zwiper-web && dx serve --platform web --port 8097     # in one terminal
python3 zcripts/perf/stacks_probe.py http://localhost:8097 regroup base
```

The page is signed in with a fake session. Every API answer is mocked from `fixtures/` except public reads; the deck's card search is answered locally; every other write is failed and printed. Nothing reaches a server.

## Actions

- `regroup`: Stacks, then Group by Color. A regroup moves every card to a new stack.
- `switch`: List, then Stacks, the stacks dealing in.
- `raise`: a tap raises a card in the largest stack, a second tap raises another, then the card stays up for 3 seconds.
- `flip`: the two-faced card's details open, then its Flip.
- `frames` and `rframes`: unthrottled screenshots of a regroup or a raise, saved to `$FRAMES`, for a by-eye check.

## Variants

`base` is the build as served. The others change it in the page to isolate a cost:
- `nosettle` turns the settle animator off.
- `replace` runs its moves without `composite: 'add'`.
- `noghost` skips the copies a removed group leaves.
- `timed` times the animator's snapshot, moves and copies.
- `floatoff` stops a raised card's drift.
- `marginoff` drops the tap stacks' `margin-top` ease.

## What it prints

- frame times from `requestAnimationFrame`: the worst three, how many ran over 50ms, and the time over 16.7ms summed
- long tasks
- Chrome's layout and style counts and durations, around a raise
- render counts, but only from a build that logs them

The budget: a regroup's longest task under 400ms at 4x. Release `a9ccce71f` took 1,096ms; with the settle fixes it takes about 335ms.

`cdplib.py` drives headless Chrome over its debugging pipe. It looks for Playwright's headless shell first, then `chromium`.
