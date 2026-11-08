# 265. Partner Swipe stack still offers illegal partners

State: fixing | Owner: laptop | PR: #419 | Build seen: 1e03520ef

## Report
Approved nit 265 (#399): the partner list offers only legal partners. Verification failed in part.

## Repro
Surfer, live 1e03520ef: with Tymna the Weaver as commander, the Partner typeahead is right (Thrasios and The Prismatic Piper offered; "Pir" and "Ian Ches" give nothing). The Partner **Swipe** stack still offers Toothy, Slurrk, Ukkima, Brallin, Shabraz and Sam (Partner with) and Wernog and Cecily (Friends forever) in its first 22 cards.

## Fix
Branch `nits/laptop-265`. Cause: on web, the deck form's Partner Swipe select got no `commander` prop, so its pile had nothing to pair against: no narrowing by partner ability and no `legal_partners` filter, just every card with any partner keyword. The typeahead passes the commander, which is why it was right. The app's pile already carries the commander (`swipe_select::Mode::Partner(commander)` from the form's memo, tested by `tymnas_partner_pile_deals_only_plain_partners`).

Fix: `Slot::pile_needs_commander()` in zwipe-ui says which piles depend on the commander (Partner and Signature spell). Web's `DeckFormOverlays` hands the commander to each of those piles and keys them by it, so picking a commander deals a fresh partner pile. Failing test first: `the_partner_and_signature_spell_piles_need_the_commander`.

## Verify 37d09dbf9: FAIL
Surfer: Tymna's (plain Partner) pile drops Partner with and Doctor's companions but still offers other Partner variants: Father & son (Kratos), Character select (TMNT), Friends forever (Othelm, Wernog, Cecily...), Survivors (Joel). Wernog (Friends forever) gets the same plain-Partner pile. Each variant pairs only with its own kind: plain Partner with plain Partner, Friends forever with Friends forever, and so on. Back to laptop.

## Fix (after verify 37d09dbf9)
Branch `nits/laptop-265c`. Scryfall now keys every partner variant as plain "Partner" and names the variant only in the rules text ("Partner—Friends forever", "Partner—Character select", "Partner—Father & son", "Partner—Survivors"), so `commander_eligibility::partner_kind` read them all as plain Partner. It now reads the variant from that line into `PartnerKind::Variant(name)` (which replaces `FriendsForever`; the old keyword still maps to it), and `are_valid_partners` pairs a variant only with the same variant. `legal_partners` decides both the Swipe select pile and the command-zone typeahead, in the app and on web, so both follow the one rule. The search narrowing matches: a variant searches its own "Partner—<variant>" text, and plain Partner leaves out any "Partner—" card. `validate_deck` uses the same rule, so the server now refuses a plain-plus-variant pair too. Failing tests first: `tymnas_pile_excludes_every_partner_variant`, `wernogs_pile_holds_only_friends_forever`, `a_variant_search_narrows_by_its_rules_text`, plus core's `each_variant_pairs_only_with_its_own_kind`.
