# 344. Meld cards show their melded card

State: fixing | Owner: scotland | PR: #534 | Build seen: release (feature, spec `context/plans/meld.md`)

## Report
Scotty, 2026-11-11, approved feature: a meld half (Gisela, the Broken Blade; Bruna, the Fading Light) gets a toggle that shows the melded card (Brisela, Voice of Nightmares), on its image and in its card details, on both halves, app and web.

## Fix
scotland:
- Data, no wire change: a meld half's `all_parts` already names its result (`meld_result`) and the other half (`meld_part`). `ScryfallData::meld()` (zwipe-core) reads them; the melded card's own printing comes from the existing public `GET /api/card/{scryfall_data_id}`, fetched on the first tap and kept for the session. Shipped clients and the server are untouched.
- Shared UI (`components`, client-free): each app provides a `MeldSource` at its root (the session's fetched results and a fetch callback over `zwipe-client`). `use_meld` drives one toggle per card. Without a source (zite) no toggle shows.
- Image: a Meld button in the flip button's place and style shows the melded card's full image, and shows the half again on a second tap.
- Details: a Meld button in the foot switches the name, type, rules and power/toughness to the melded card, with "Melds with <other half>"; a host that titles the half in its own bar (the drawer) also gets the melded card's name in the body. The set, prices and tags stay the half's.
- A failed fetch is quiet: the half keeps showing, the button reads off, and the next tap tries again.
- Melded cards are never swipeable or addable; swiping a half acts on the half.
- Tests first: `a_meld_half_names_its_result_and_its_other_half`, `the_meld_result_and_other_cards_have_no_meld_toggle` (zwipe-core), `the_melded_card_shows_only_once_wanted_and_fetched` (components).
