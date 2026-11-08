# 293. An empty Swipe pile stays on a skeleton card

State: verified | Owner: laptop | PR: #432 | Build seen: 37d09dbf9

## Report
Droid, Android app (debug APK of release at 37d09dbf9 on prod, Pixel 9a emulator), 2026-11-08. Create deck with Pir, Imaginative Rascal as commander: the form fills Partner with Toothy, Imaginary Friend on its own. Opening the Partner Swipe pile then shows a grey skeleton card for over 30 seconds and never deals a card or says the pile is empty. Toothy is the only legal partner and is already picked, so the pile is probably empty; it should say so instead of looking like it's loading.

## Repro
1. Decks, Create deck, Format Commander.
2. Commander: pick Pir, Imaginative Rascal. Partner fills with Toothy.
3. Tap Swipe beside Partner.
4. Swipe select shows only `.skeleton-card` and no images, toast or empty message after 30 s.

## Fix
Branch `nits/laptop-293`. With Pir's partner Toothy already picked, the pile's search finds only Toothy, the host excludes it, and the next page is empty, so the pile is Empty. The app's model said so once with a quick toast ("No partners match your filters") and then drew the skeleton card, since only the suggested pile had a line in place of the card; web said nothing and kept the skeleton. Now any search pile that dealt no card shows a line in place of the card, from `zwipe_ui::screens::swipe_select::empty_pile_line`: "No partners to show", or "No partners match your filters" while a filter is on, with the pile's own noun. The app takes it from `State::message` (which replaces the toast); web shows it once the first page has landed and nothing is loading. A pile swiped to its end keeps its "No cards left" toast. Failing tests first: `a_partner_pile_with_its_one_card_picked_says_it_is_empty`, `an_empty_filtered_pile_blames_the_filter`.
