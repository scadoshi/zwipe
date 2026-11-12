# 395. app.zwipe.net Import squeezes into the left of its page

State: fixing | Owner: thinker | PR: - | Build seen: production, web

## Report
Composer for Scotty, 2026-11-12, from surfer's text-width sweep: a deck's Import screen sits in a block about 530px wide at the left of its 1152px column (the chips, the hint and the Paste decklist panel), leaving the right side empty, while Export fills the same column. Import should fill it as Export does.

## Cause
From 900px Import was a two-column grid: the controls and the input on the left, the results on the right. Until an import runs the results are empty, so the right column stood empty.

## Fix
thinker, `web.css`: Import is one column, as Export is: the controls, the input panel, then the results under it, a panel gap apart.

## Verification
-
