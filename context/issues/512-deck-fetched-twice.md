# 512. A deck's page asks the server for the deck twice as it loads

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: 3949574fc

## Report
Found by scotland in 510's run; surfer confirmed 3 of 3 at 1440 and 390 (composer, 2026-11-13): every deck page load asks `/api/deck/<id>` twice, and Edit asks once more. Find why, fetch once; check the app.

## Before
Headless with an in-page fetch wrapper, fly351's "Isshin 510": each load of the deck's page sends `deck/<id>` twice (at 1440 and at 390), beside one `deck/profile/<id>`; Edit, its own screen, sends one more.

## Cause
`zwiper-web/src/lib/inbound/screens/deck/view/mod.rs` kept a `deck` resource of its own, while the page under it (`view/page.rs`) loads the deck through its editor (`cards/editor.rs`), which is what the page draws. The view's copy fed only the not-found redirect (the profile's fetch catches a missing deck too), Retry, and the quantity fix's undo baseline, and each warning fix fetched both copies again. The app does not: its Cards screen answers from the deck its route holds (`zwiper/.../deck/card/view.rs` `fetch_deck`).

## Fix
scotland: the view drops its resource; the page keeps the view's `deck_copy` signal current from the editor's deck, which the fixes read. Edit keeps its one fetch, a screen of its own.

## Verification
- `view/mod.rs` test: the view names no deck fetch, the page none, the editor one (the old view plus the test counts two).
- Headless: three loads of the deck's page, one `deck/<id>` each at 1440 and at 390, the cards drawn at 1440; the page then Edit, two in all.
- Not checked: a warning fix's undo baseline, read from the shared copy (the deck here has no warning to fix).
