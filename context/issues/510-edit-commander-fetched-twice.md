# 510. Opening Edit deck fetches the commander's card twice

State: fixing | Owner: scotland | Type: issue | PR: - | Build seen: 74040e086

## Report
Surfer (composer, 2026-11-13, taken from laptop): on the web, Edit deck fetches the commander's `/api/card/<id>` twice per load, counted with an in-page fetch wrapper on the deck Isshin (9457535d), every run (three at 1440, one at 390). Fetch once; check the app's model too.

## Before
Headless, fly351's deck "Isshin 510" (d8c7524a, Isshin's printing c6d813e9) with an in-page fetch wrapper: from the deck's page into Edit, `card/c6d813e9` goes out twice, at 1440 and at 390; Edit loaded on its own sends it once (three reloads). A direct count of zwipe-client: two `fetch_card` calls for one id reach the server twice (test below, red).

## Cause
Two screens each fetch the commander and nothing keeps the first: the deck's page fetches it for its name (`zwiper-web/src/lib/inbound/screens/deck/view/mod.rs`), and Edit fetches it again to fill the command zone (`.../deck/edit.rs`). The app does the same (`zwiper/src/lib/inbound/screens/deck/view.rs` `fetch_commander`, then `.../components/deck_form.rs` `fetch_card`). Neither effect runs twice; Edit's own count is one.

## Fix
scotland: `zwipe-client`'s `ZwipeClient` keeps the cards it fetches by printing id, shared by every clone (both clients hand clones of one client around), so `fetch_card` answers a card fetched before from memory: the deck's page and Edit share one fetch, on the web and in the app.

## Verification
- zwipe-client `a_card_is_fetched_once_a_session_across_the_client_s_clones`: two fetches of one card on a client and its clone reach the server once (twice before).
- Headless, the same deck: from its page into Edit, one `card/c6d813e9` at 1440 and at 390, and Edit's commander field reads Isshin, Two Heavens as One; Edit on its own still one.
- Found, not changed: the deck's page asks for `deck/<id>` twice as it loads.
