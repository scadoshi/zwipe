# 297. Web Decks shows every deck tag filter up front

State: fixing | Owner: laptop | PR: #435 | Build seen: 1d7053e88

## Report
Scotty, 2026-11-08 (screenshot, web Decks, wide): under Group by, Sort and Show, all 19 deck tag filter chips (+1/+1 Counters through Vehicles) are always listed. In the mobile app that row is collapsible: the tags start folded and people expand it to pick one. Match the app on web: same toggle, same default (folded), same behavior when a tag is active (match the app; check what it does).

## Fix
Branch `nits/laptop-297`. Web Decks already had the app's Tags fold (folded by default, the picked tags staying in the Show row while folded), but from 900px web.css hid the fold chip and forced the tag row open. That override is gone, so every width folds like the app. The chip's label is `deck_list::TAGS`, the app's constant. Test: `the_tags_fold_chip_shows_at_every_width` fails if a web.css rule hides the fold chip.
