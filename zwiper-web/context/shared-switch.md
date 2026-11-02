# Shared code to switch to

The zwiper-helpers session (scotland-server) lands shared branches on main and adds each one here. Whoever works on zwiper-web next: merge origin/main into feat/web, then work down this list. Tick an item when the web uses the shared piece and its own copy is deleted. Read `zwipe-ui/context/shared-plan.md`, `zwipe-ui/context/inventory.md` and `zwipe-ui/README.md` on main first.

Before you run `fmt`, update nightly. CI's rustfmt merges imports that older nightlies leave split. Don't edit zwipe-ui, zwipe-core, zwipe-client, components or orangui on this branch. If a shared piece doesn't fit the web, write it down under "Doesn't fit" below and zwiper-helpers folds it into the shared crate.

## On main, to switch

- [ ] **5, deck form (#116).** `zwipe_ui::deck::form`: `FormValues` (`saved`, `slot`, `slot_mut`, `shows`, `pick_format`, `clear_format`, `set_commander`, `create_request`, `update_request(&saved) -> Option<..>`) and `Cascade { cleared, reset, filters_on }`. It replaces the web's form logic in `domain/deck.rs` (`:41`, `:102`, `:136`), `deck/form/mod.rs:167` and `form/fields.rs:63`.
- [ ] **7, create gate (#118).** `zwipe_ui::deck::create_gate(verified, deck_count: usize) -> Result<(), Toast>` and `deck::VERIFY_TO_CREATE` (Warning, LONG). The four deck limits live in `zwipe_core::domain::deck::limits`, re-exported from `zwipe_core::domain::deck`. Switch the copies in `deck/list.rs`, `playstyle/mod.rs` and `commander_maybeboard.rs`.
- [ ] **8, deck view (#118).** `zwipe_ui::deck::view`: `has_budget`, `tag_count`, `metrics_possible`, `share_link`, `total_price_line`, `clone_name`, plus `Done` toasts with `.toast()`: `Cloned(name)`, `LinkCopied`, `LinkDisabled`, `SkipsCleared`, `QuantitySet(n)`, `CommanderCleared(format)` and `CardRemoved`, all Info, QUICK. Swap the `domain/deck.rs` versions, use `clone_name` in `view/mod.rs` and `view/more.rs`, and raise the `Done` toasts. Error toasts stay the web's.
- [ ] **9, commander (#118).** `zwipe_ui::commander::maybeboard`: `passes_pips`, `is_addable`, `step_highlight` with `Step::{Down, Up}` (the web's `forward: true` is `Step::Down`), and `LIST_TITLE`, `NO_COMMANDERS`, `NO_MATCH`, `LOAD_FAILED`. `commander::swipe_select`: `card_key`, `redeal`. Delete `domain/commander_maybeboard.rs` and `domain/swipe_select.rs`.
- [ ] **See guide links (#117).** `zwipe_ui::hint::guide`: `HintTopic::guide_slug`, `one_time_slug(OneTimeHint)`, `url(slug)`. Screen hints link out to zwipe.net's guides, and a panel's `?` doesn't. orangui is repinned to a59165d, where `HintDialog` puts Got it before the host's actions.

- [ ] **10, dictionary (#120).** `zwipe_ui::dictionary`: `bucket_of`, `letters`, `rows_for`, `OTHER`. Delete `bucket_of` and `rows_for` in `inbound/screens/oracle_tag_dictionary.rs`, along with their tests.
- [ ] **11, preferences (#120).** `zwipe_ui::preferences`: `PreferenceChange`, `From<PreferenceChange> for HttpUpdatePreferences`, `settle` on `Generation` (the web's `RequestSeq` becomes `Generation`, and `begin` becomes `next_ticket`), `exception_names`, `saved_toast`, `theme_unchanged`, `exceptions_unchanged`. Delete `domain/preferences.rs`. Replace the local `update` helper and the hand-built requests in `config/themes.rs` and `exceptions.rs`.
- [ ] **13, filter chips (#120).** `zwipe_ui::filter::chips` is the web's `domain/filter_chips.rs` lifted with the same API (`PRODUCED`, `ChipOption`, `options`, `is_selected`, `tap`). Delete the web's file and import the shared one.
- [ ] **14, card info (#120).** `zwipe_ui::card::info`: `price_line`, `set_line`, `released_line`, `artist_line`, where `None` means a blank line. Switch `inbound/cards/card_info.rs`, which keeps its `\u{a0}` for a blank line.

## Coming

- 6, deck pickers: one draft model for the format, deck tag and oracle tag pickers.
- 12, telemetry: the usage buffer that replaces the web's disabled funnel, with server-side platform tagging.
- Then the screen models, one per screen, starting with Deck list, Home, Activity and Deck view.

## Doesn't fit

(none yet)
