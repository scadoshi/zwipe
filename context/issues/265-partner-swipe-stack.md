# 265. Partner Swipe stack still offers illegal partners

State: merged | Owner: laptop | PR: #419 | Build seen: 1e03520ef

## Report
Approved nit 265 (#399): the partner list offers only legal partners. Verification failed in part.

## Repro
Surfer, live 1e03520ef: with Tymna the Weaver as commander, the Partner typeahead is right (Thrasios and The Prismatic Piper offered; "Pir" and "Ian Ches" give nothing). The Partner **Swipe** stack still offers Toothy, Slurrk, Ukkima, Brallin, Shabraz and Sam (Partner with) and Wernog and Cecily (Friends forever) in its first 22 cards.

## Fix
Branch `nits/laptop-265`. Cause: on web, the deck form's Partner Swipe select got no `commander` prop, so its pile had nothing to pair against: no narrowing by partner ability and no `legal_partners` filter, just every card with any partner keyword. The typeahead passes the commander, which is why it was right. The app's pile already carries the commander (`swipe_select::Mode::Partner(commander)` from the form's memo, tested by `tymnas_partner_pile_deals_only_plain_partners`).

Fix: `Slot::pile_needs_commander()` in zwipe-ui says which piles depend on the commander (Partner and Signature spell). Web's `DeckFormOverlays` hands the commander to each of those piles and keys them by it, so picking a commander deals a fresh partner pile. Failing test first: `the_partner_and_signature_spell_piles_need_the_commander`.

## Verify 37d09dbf9: FAIL
Surfer: Tymna's (plain Partner) pile drops Partner with and Doctor's companions but still offers other Partner variants: Father & son (Kratos), Character select (TMNT), Friends forever (Othelm, Wernog, Cecily...), Survivors (Joel). Wernog (Friends forever) gets the same plain-Partner pile. Each variant pairs only with its own kind: plain Partner with plain Partner, Friends forever with Friends forever, and so on. Back to laptop.

## Decision (partner pairs)
Scotty, 2026-11-08: a mismatched partner pair (e.g. Tymna + Wernog) only warns, like illegal cards. The server never refuses a save over it; the pile and typeahead still offer only matching kinds.
