# 304. Web Home's stat chips sit far below the counters

State: verified | Owner: laptop | PR: #443 | Build seen: 1d7053e88

## Report
Scotty, 2026-11-08 (two screenshots): on zwipe.net's hero, the "as of Nov 8" and "Everybody" chips sit just under the three stat counters (Decks created, Searches run, Cards swiped). On the web app's Home, the same chips sit much farther down. Align the web app's spacing to zwipe.net's (same gap above the chips, same padding), at every width.

## Fix
Branch `nits/laptop-304`. From 900px the hero is a grid with the mark spanning two rows, so the counters' row and the chips' row split the mark's height between them and the chips sat 46px under the counters at 1200 (36 at 1000). The grid now has spare rows above and below (`1fr auto auto 1fr`), the mark spans all four, and the counters and chips sit together in the middle with zite's 1rem between them, centered beside the mark. Measured against zwipe.net: 16px from the counters to the chips at 1400, 1200, 1000, 900, 899 and 390 on both.
