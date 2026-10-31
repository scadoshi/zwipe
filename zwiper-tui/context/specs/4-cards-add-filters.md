# Spec 4: Add cards, card filter sheet, deck cards view, remove cards

Source of truth (worktree `the repo root`):

- `zwiper/src/lib/inbound/screens/deck/card/add.rs` (Add), `view.rs` (deck cards), `remove.rs` (Remove)
- `zwiper/src/lib/inbound/screens/deck/card/components/*` (stack, caches, sheets, quick add, undo log)
- `zwiper/src/lib/inbound/screens/deck/card/filter/**` (filter sheet and every section)
- `zwiper/src/lib/inbound/components/interactions/swipe/*` (SwipeStack gesture engine)
- `components/src/{card_row,card_details,flippable_card_image,keyword_chips,card_role_chips}.rs` (shared UI)
- Routes (`zwiper/src/lib/inbound/router.rs`): `/deck/card/add/:deck_id` → `AddDeckCard`, `/deck/card/view/:deck_id` → `ViewDeckCard`, `/deck/card/remove/:deck_id` → `RemoveDeckCard`.

Conventions used below:

- Toast kinds: `info`, `success`, `warning`, `error`. Durations: QUICK = 3 s, NORMAL = 5 s, LONG = 8 s (`orangui/src/toast_timing.rs`). "default" = `ToastOptions::default()` (library default duration, no explicit band).
- "authed.run" = the app's call facade (`zwiper/.../components/auth/authed.rs`): refresh the session (`ensure_fresh`), run the call, on failure report telemetry and toast `error` with `ClientError::to_user_message()` for NORMAL; returns `None`. `try_run` is the same but returns the `Err` to the caller (still toasts). `run_quiet` reports and logs but never toasts.
- Every string in double quotes is copied verbatim from the code.

---

## Shared state, stores and helpers used by all four screens

These are app-scoped (created once in `spawn_upkeeper`, provided as context), in-memory only. An app restart forgets all of them. Nothing here is persisted to disk.

### FilterStore (`components/filter_store.rs`)

- `HashMap<(FilterScope, Uuid deck_id), CardQueryBuilder>`, unbounded.
- `FilterScope` = `Cards` (deck cards view), `Remove`, `AddSearch`, `AddMaybeboard`.
- `restore(scope, deck_id) -> Option<CardQueryBuilder>` (clone), `park(scope, deck_id, builder)` (replace).
- Each screen seeds its local filter signal from `restore` on mount and `park`s it on unmount. So every screen on every deck remembers its own filter for the session; filters never leak across screens.

### AddStackCache (`components/add_stack_cache.rs`)

- MRU list of `(deck_id, ParkedStack)`, max `MAX_PARKED_DECKS = 20` decks; oldest evicted.
- `ParkedStack { cards: Vec<Card>, index: usize, history: Vec<AddAction>, filter: CardQueryBuilder, exhausted: bool }`.
- `park(deck_id, parked)`: trims swiped-past cards to `PARKED_BEHIND_CARDS = 50` behind the cursor (`cut = index - 50` if positive; drain `cards[..cut]`, `index -= cut`), then drops history entries deeper than `index` (`excess = history.len() - index`, drain oldest), removes any prior entry for the deck, inserts at front, truncates to 20.
- `take(deck_id) -> Option<ParkedStack>` removes and returns.

### The add screen's search stack is itself app-scoped

`CardStack<AddAction>` is provided as context (outlives the screen) so re-entering the same deck resumes mid-stack even without the cache. The cache exists so that switching between decks resumes each deck.

### CardStack<A> (`components/card_stack.rs`)

Fields: `cards: Vec<Card>`, `index: usize` (cursor), `history: Vec<A>` (undo), `entering: Option<Direction>` (enter animation for the top card).

- `window()` = `cards[index .. index + STACK_DEPTH]` where `STACK_DEPTH = 10` (render window).
- `current()` = `cards.get(index)` (linear). `current_wrapping()` = `cards[index % len]` (cycling; None when empty).
- `replace(cards)` sets cards and index = 0; history kept. `append(more)`. `rewind()` index = 0, clear history, entering = None (cards kept). `reset()` clears everything.
- `park_state()`, `restore(cards, index, history)` (index clamped to len, entering = None).
- `remove_current()` removes `cards[index]` if in range (cursor unchanged so next slides in). `insert_current(card)` inserts at `min(index, len)`. `replace_current(card)` overwrites `cards[index]`.
- `advance()` (linear): `index + 1 <= len` → set and true; cursor may land one past the end. `advance_wrapping()`: `(index+1) % len`. `retreat_wrapping()`: index 0 → len-1, else index-1; false when empty.
- `record(action)` push history; `pop_action()`; `unwind_undo(action)` (push back, index + 1, entering = None); `prime_entering(dir)`; `cancel_entering()`; `step_back(&action)` (linear: false at index 0, else index - 1 and entering = action.exited()).

### Stack action models (`components/action_history.rs`)

- `AddAction` (linear, data-free): `Skip` (exit Left), `Add` (exit Right), `Maybe` (exit Up).
- `MaybeboardAction` (cycling): `Skip` (Left), `Promote { card: Box<Card> }` (Right).
- `RemoveAction` (cycling): `Keep` (Left), `Remove { card, board: Board, quantity: i32 }` (Right), `MoveBoard { card, from: Board, to: Board }` (Up).
- `BrowseAction::Next` (Left) is used by the oracle tag examples browse (not this spec).
- `MAX_CARDS_IN_STACK = 500`, `CARDS_WARNING_THRESHOLD = 400`.

### UndoStore and UndoLog (`components/undo_log.rs`): the global per-deck undo stack

- One undo stack per deck, `MAX_UNDO_ACTIONS = 100`, oldest dropped on overflow (`push_capped`).
- `UndoStore` (app-scoped `HashMap<deck_id, Vec<UndoAction>>`): `restore(deck_id)` (clone, empty if none), `park(deck_id, log)`, `push(deck_id, action)` (used by Add/Remove while the view's live log is unmounted), `take_newest(deck_id, predicate) -> Option<UndoAction>` (removes newest match, `rposition`), `clear(deck_id)` (import uses it).
- `UndoLog(Signal<Vec<UndoAction>>)` = the deck cards view's live copy, seeded from `UndoStore::restore` on mount and parked back on unmount. `push(action)`.
- `UndoAction` variants and what their inverse needs:
  - `Added { card_id, card_name }`: undo deletes the card.
  - `Removed { entry: DeckEntry, baseline: i32 }`: undo re-creates on the old board at `baseline` quantity.
  - `QuantityChanged { card_id, card_name, net, baseline }`: undo applies `-net`.
  - `MovedBoard { card_id, card_name, from: Board }`: undo moves back to `from`.
  - `PrintingChanged { old_card: Card, new_id: Uuid }`: undo restores the old printing.
  - `CommandZonePrintingChanged { slot: CommandZoneSlot, old_card, new_id }`: undo rewrites the deck profile slot. `CommandZoneSlot` = `Commander | Partner | Background | SignatureSpell`.
- `create_board_field(board) -> Option<String>`: `Board::Deck` → `None` (server default), else `Some(board.display_name())` (`"maybeboard"` / `"sideboard"`).
- Reconciliation rule used by every gesture undo on Add and Remove: before reversing, `take_newest` the matching entry. `Some` → the mutation is outstanding, send the reversing call (and push the entry back if that call fails). `None` and the world already reversed (card gone / card already back) → the deck cards Undo button already did it: skip the server call. `None` but the card is still in the deck → it aged out of the cap: reverse anyway.

### InFlightAdds (`components/in_flight_adds.rs`)

Per card id: `posts` (POSTs sent, unanswered) and `undone` (undos parked).

- `begin(card_id)`: posts += 1.
- `undo(card_id) -> bool`: if an entry exists and `undone < posts`, undone += 1 and return true (the add's own task will delete the card after its POST answers; the caller sends nothing). Else false.
- `finish(card_id) -> AddOutcome`: posts -= 1; if undone > 0 → undone -= 1, `Undo`; else `Keep`. Entry removed when posts reaches 0.

### Other helpers

- `Board` (`zwipe-core/src/domain/deck/models/board.rs`): `Deck` (default, active), `Maybeboard`, `Sideboard`. `display_name()` → `"deck"`, `"maybeboard"`, `"sideboard"`. `is_active()`, `is_maybeboard()`, `is_sideboard()`.
- `deck_metrics::card_price(&ScryfallData, PriceCurrency) -> Option<f64>` (nonfoil, then foil fallback), `deck_metrics::deck_price(&entries, &command_zone_cards, currency) -> f64` (mainboard total), `deck_metrics::budget_tier(budget, total)` (50/75/100% bands). `PriceCurrency::format_amount(f64)`.
- `ScryfallData::is_land()` (combined type line, MDFC land faces count), `is_basic_land()`, `face_count()`, `face_image_url(face, size)`, `primary_image_url(size)`, `art_crop_url()`.
- `Format::default_land_target()`, `Format::to_legality_key()`, `Format::checks_color_identity()`, `Format::has_commander()`, `Format::has_signature_spell()`, `Format::display_name()`.
- `CardQueryBuilder` (`zwipe-core/.../search_card/card_filter/builder.rs`), `Cards::from(Vec<Card>).matching(&CardCriteria)` and `.sorted(Option<CardSortKey>, direction)` (`search_card/cards.rs`), `cards::sort_deck_entries(&mut entries, key, direction)`, `GroupCards::group_by(GroupByOption)` (`search_card/group_cards.rs`).

### Common UI pieces

- **ScreenHeader** `{ title, hint }`: page header with a Support button leading and a grayed "?" that opens the screen's hint dialog.
- **One-time hints** (`use_one_time_hint(key)`): auto-opens the hint dialog on the user's first visit if `session.user.has_seen_hint(key)` is false, then calls `client.mark_hint_shown(key, &session)` and saves the returned user into the session. Failure is logged only.
- **ActionBar**: the footer row of `ButtonVariant::Action` buttons.
- **CardSkeleton { is_loading }**: a card-shaped ghost (image block + 5 bars: name, price, set, date, artist). When `is_loading == false` the image block shows the text "No cards"; while loading it is a plain ghost (no spinner, no text).
- **CardInfoDisplay { card }** (under the swipe card and in the printing sheet), five lines:
  1. card name (`card-info-name`)
  2. prices: `"Prices:"` followed by ` $<usd>`, then ` |` + ` €<eur>`, then ` |` + ` <tix> TIX` for each present, e.g. `Prices: $1.23 | €1.10 | 0.05 TIX`. If none present: a non-breaking space (blank line kept for layout).
  3. `"Set: {set_name}"`
  4. `"Released: {released_at}"`
  5. `"Artist: {artist}"`, or a non-breaking space when no or empty artist.
- **RulesButton { open }**: action-bar eye-icon button that toggles `open`.
- **CardDetailsSheet { open, card, on_printings?, footer? }**: a BottomSheet titled with the card name.
  - Body: the shared `CardDetails` with `show_name: false, show_cost: true, show_classification: true, show_flip: false`, face driven by the sheet's own face signal, `describe_tag` from the oracle-tag catalog cache.
  - Sheet footer buttons: "Back" (close), "Flip" (only when `card_face_count(card) > 1`, cycles the face, sheet stays open), "Printings" (only when `on_printings` is given).
- **CardDetails** (shared, `components/src/card_details.rs`): head (name if `show_name`, mana cost of the current face if `show_cost`), meta chips (current face type line, rarity long name, set name), oracle text of the face (mana symbols rendered), stats chip (`"{p}/{t}"`, else `"Loyalty {l}"`, else `"Defense {d}"`), `KeywordChips` (label "Keywords"; tapping a chip reveals its reminder text from the served keyword catalog), and if `show_classification` the `CardRoleChips` (label "Card roles"; each role chip drills down to its oracle tags, plus an "Other tags" bucket; a tag reveals its description or "No description yet", plus an "Examples" button when the host provides one). Faces: when the card has no top-level oracle text and has `card_faces`, one entry per face with type line or oracle; otherwise one entry. Optional bottom action bar: "Flip" (when faces > 1 and `show_flip`), "Image" (when the card has art and the host passes `on_image`), then the host's actions.
- **FlippableCardImage** (shared, `components/src/flippable_card_image.rs`): the card image for the current face (`face_image_url(face, size)`), with a "Flip" button overlay (aria-label "Flip card") when `flippable && face_count > 1`. Tapping Flip cycles `(face + 1) % total` and stops propagation (so it never starts a swipe). No-art cards render a text proxy instead: name, Flip button (if DFC), mana cost, rule, an art box reading "No image", chips for type line, rarity long name and set name, oracle text, flavor text, and a P/T (or loyalty) chip. First-time image loads ease in; a session-wide set of seen URLs skips the ease for cached images. `reset_image_ease()` clears that set (called on every fresh search so new results fade in).

---

## 1. Add cards (the swipe screen)

Route `/deck/card/add/:deck_id` (`Router::AddDeckCard { deck_id }`). Reached from the deck cards view's "Add" button (push) and from the deck warnings panel on the deck view (push). Leaves with "Back" (`navigator.go_back()`) or OS back.

### 1.1 State on mount

- Local `filter_builder: Signal<CardQueryBuilder>` seeded from `FilterStore::restore(AddSearch, deck_id)` (mount always starts on the Search source), else default. `had_restored_filter` remembers whether a restore happened. Provided as context for the filter sheet.
- Context: app-scoped `stack: CardStack<AddAction>`, `stack_cache: AddStackCache`, `last_search_filter: Signal<Option<CardQueryBuilder>>` (app-scoped; the filter that produced the current stack), `undo_store`.
- Hint: `use_one_time_hint(HINT_ADD_DECK_CARDS)`.
- Local: `deck_cards_ids: HashSet<Uuid>` (oracle ids in the deck incl. command zone, used to drop deck cards from results), `deck_format`, `deck_color_identity: Option<Colors>`, `deck_has_commander`, `deck_loaded`, `mainboard_land_count: i32`, `land_target: Option<i32>`, `deck_total_price: f64`, `price_budget: Option<f64>`, `price_budget_currency` (default USD), `add_source: AddSource` (`Search` default | `Maybeboard`), `synergy_warming`, `related_ids: HashSet<Uuid>`, `related_announced`, `mb_entries: Vec<DeckEntry>`, `mb_stack: CardStack<MaybeboardAction>` (screen-local), `filters_overlay_open`, `filter_reset_counter: u32` (context; bumping it re-runs the search), `is_loading_more`, `pagination_exhausted`, `is_loading_cards`, `in_flight_adds`, `show_details`, `printing_open`.
- Constants: `SWIPE_DISTANCE_PX = 90`, `SWIPE_SPEED_PX_PER_MS = 2.5`, `BACK_PULL_PX = 60`, `STACK_WARNING_EVERY = 100`, page size `pagination_limit = 25`, `load_more_threshold = 5`.

`lands_at_target()` = `(land_target or deck_format.default_land_target())` is Some(t) and `mainboard_land_count >= t`.

`ensure_lands_excluded(filter)`: remove `CardType::Land` from `card_type_contains_any` and `card_type_contains_all` (setting an empty vec clears that filter), and add `Land` to `card_type_excludes_any` if missing. Prevents an include+exclude clash on Land.

### 1.2 Unmount (two drops)

1. Park the filter under the live source's scope (`AddSearch` or `AddMaybeboard`).
2. Park the search stack in `AddStackCache` if `last_search_filter` is Some and the stack is not empty: `(cards, index, history) = stack.park_state()`, with `filter = last_search_filter` and `exhausted = pagination_exhausted`.

### 1.3 Deck load (mount effect)

`authed.run_quiet("load_deck_context", fetch_deck(deck_id, &s))` (quiet: the screen works without deck context). Client: `ZwipeClient::fetch_deck(&self, deck_id: Uuid, session: &Session) -> Result<Deck, ClientError>`. On success:

1. `deck_cards_ids` = oracle ids of every `deck.entries` card (all boards).
2. For each of `deck_profile.commander_id`, `partner_commander_id`, `background_id`: `client.fetch_card(id)` (`fetch_card(&self, scryfall_data_id: Uuid) -> Result<Card, ClientError>`, unauthenticated), add its oracle id to `deck_cards_ids` and collect its `color_identity`. `signature_spell_id`: oracle id only (no color identity). Errors ignored.
3. `mb_entries` = entries whose board is maybeboard.
4. `deck_has_commander = commander_id.is_some()`. If no filter was restored: `filter.set_synergy(commander_id.is_some())` (synergy default ON with a commander, OFF otherwise; a restored filter keeps the user's toggle).
5. `deck_loaded = true`.
6. `mainboard_land_count` = sum of quantities of active-board land entries. `land_target = deck_profile.land_target` (explicit only, used for the toast). If the effective target (`land_target` or the format default) is met, `ensure_lands_excluded(filter)`.
7. Currency = `price_target_currency` or USD; `deck_total_price = deck_metrics::deck_price(entries, command_zone_cards, currency)`; `price_budget = price_target`.
8. If format is set: `deck_format = fmt`; if the filter has no `legalities_contains_any`, set it to `[fmt.to_legality_key()]`. If `fmt.checks_color_identity()` and there is a commander: sort and dedupe the collected colors into `Colors`, store in `deck_color_identity`, and set `color_identity_within(colors)` if the filter has none.
9. Auto-serve: if the stack is empty, bump `filter_reset_counter` (runs the search effect).

### 1.4 The search effect (runs on mount and whenever `filter_reset_counter` changes)

Builder = filter clone with `set_is_token(false)`, `set_limit(25)`, `set_offset(0)`.

**First run only (mount):** `stack_cache.take(deck_id)`; normalize the parked filter the same way; if it equals the current builder and the parked cards are non-empty: `pagination_exhausted = parked.exhausted`, `last_search_filter = parked.filter`, `stack.restore(cards, index, history)`, and stop (no fetch). Otherwise fall through. (Note: if no cached entry and the app-scoped stack is non-empty, the fall-through still resets it. The resume-without-refetch path is the cache.)

**Fresh fetch:**

1. Start a new generation ticket (any in-flight search or load-more for the old stack is ignored when it lands). If `is_loading_cards`, clear it.
2. `stack.reset()`, `reset_image_ease()`, `last_search_filter = None`, `pagination_exhausted = false`.
3. Gate (only when the deck has loaded and the source is Search): serve if commander format → `has_commander || filter.has_search_intent()`; else `has_search_intent()`. If not served: toast warning "Try a filter to start swiping" (NORMAL) and stop.
4. `builder.build()`; on Err (truly empty builder, e.g. deck context not loaded yet) stop silently.
5. `is_loading_cards = true`; snapshot the filter; record a search in telemetry.
6. Flush buffered usage telemetry first (`flush_loop::flush_once`) because swipe signals feed synergy ordering.
7. `authed.try_run("search_cards", search_deck_cards(deck_id, &filter, &s))`. Client: `search_deck_cards(&self, deck_id: Uuid, card_filter: &CardQuery, session: &Session) -> Result<HttpDeckCardSearch, ClientError>`. `HttpDeckCardSearch { cards: Vec<Card>, synergy_applied: bool, related_from: Option<usize> }` (`zwipe-core/src/http/contracts/deck.rs`). The deck-aware endpoint excludes deck cards and the user's active skips server-side, orders by synergy when synergy is on and no sort is set, and tells the client where "related" cards begin.
8. Drop if the ticket is stale. On Ok:
   - `synergy_warming = !synergy_applied`.
   - `related_ids` = printing ids (`card_profile.scryfall_data_id`) of `cards[related_from..]`; `related_announced = false`.
   - Drop cards whose oracle id is in `deck_cards_ids`.
   - If nothing is left: telemetry `record_pile_end(deck_id, PileEndKind::Empty, synergy)` and toast warning (NORMAL): "No results for this filter", plus, when synergy is on, either "; Synergy is warming up, so all cards were searched" (warming) or "; tap Synergy to turn it off". Full examples: `No results for this filter; tap Synergy to turn it off`.
   - `stack.replace(served)`, `last_search_filter = snapshot`, `is_loading_cards = false`.
   - On Err: `is_loading_cards = false` (the facade already toasted the error).

### 1.5 Load more (pagination)

Triggered from `advance_after_commit` when the cursor is within 5 of the end.

1. If `stack.len() >= 500`: toast warning "Card stack limit reached, please refresh your search to continue" (NORMAL), stop.
2. If already loading more, stop. Set `is_loading_more`.
3. Same normalized builder, **offset always 0** (swiped cards leave the server's pool via skips/adds, so the top of what is left is the next page; dedup handles cards still in the stack).
4. Joins the current generation (not a new one). Records a search.
5. `authed.try_run("load_more", search_deck_cards(...))`. If the generation moved on, just clear `is_loading_more`.
6. On Ok: update `synergy_warming`, extend `related_ids`; keep new cards not already in the stack (by `card_profile.scryfall_data_id`) and not in the deck (oracle id). If none are new: `pagination_exhausted = true` and `record_pile_end(Empty, synergy)`. Else `stack.append`. Clear `is_loading_more`. On Err just clear it (error was toasted by the facade).

### 1.6 Layout, top to bottom

1. **ScreenHeader** title "Add deck cards", "?" opens the swipe hint.
2. **Source row** (`chip-row`, max width 40rem):
   - Label chip "From".
   - Chip "Search" and chip "Maybeboard" (single select; tapping the selected one is a no-op).
   - Right-aligned chip "Synergy", shown only when source is Search and the deck has a commander. Selected = `filter.synergy()`. Tap: toggle `set_synergy(!on)` and bump the counter (immediate refetch, no Apply).
3. **Card area** (`form-container`):
   - **Search source**:
     - If the stack is non-empty and `index < len`: the **SwipeStack** (window of up to 10 cards; only the top one interacts), then **CardInfoDisplay** of the current card.
     - Else if non-empty (cursor one past the end): if `is_loading_more` → `CardSkeleton { is_loading: true }`; else a `CardSkeleton` ("No cards") wrapped in a gesture area that accepts a **down-pull to undo**: on touch/mouse release, if `dy >= 60px` and `|dy| > |dx|`, run `undo_last_action()`. This is how the user steps back into the pile from end-of-results.
     - Else if `is_loading_cards` → loading skeleton (no text).
     - Else → `CardSkeleton` with "No cards".
   - **Maybeboard source**: if `mb_stack` non-empty → SwipeStack over `mb_stack.window()` + CardInfoDisplay of `current_wrapping()`; else CardSkeleton ("No cards").
4. **ActionBar**:
   - "Back" → `navigator.go_back()`.
   - "Filter" → opens the filter sheet. Shows a dot (`filter-dot`) when `!filter.is_empty_ignoring_deck_context_and_auto_lands(lands_at_target()) || filter.sort().is_some()`. (Deck legality, commander identity, synergy and the automatic land exclusion do not count as user filters.)
   - "Refresh" (see 1.10).
   - Eye **RulesButton**, only when there is a current card (`mb_stack.current_wrapping()` on Maybeboard, `stack.current()` on Search).
5. Root-mounted overlays (siblings of the ActionBar so they cover it):
   - **CardDetailsSheet**: Maybeboard source → without Printings. Search source → with "Printings" (`on_printings` opens the printing sheet).
   - **PrintingSheet** (Search source current card only) with `host_screen = DECK_CARD_ADD`, `saved_message: "Printing swapped"`, `on_save`: `stack.replace_current(new_card)` and close the details sheet. Swiping right then adds that exact printing. No server call (the card is not in the deck yet).
   - **HintDialog** title "Swipe to build":
     - "Swipe **right** adds the card" (right in success color)
     - "Swipe **left** skips it for a week" (error color)
     - "Swipe **up** sends it to the maybeboard" (warning color)
     - "Swipe **down** undoes the last swipe" (tertiary accent)
     - "**Synergy** on starts with cards that work with your commander, then cards like your deck. Off shows every legal card" (Synergy as a key chip)
     - Line: "Sort only reorders. It never hides cards"
   - **CardFilterSheet** with `show_format_filter: true`, `show_active_indicators: true`, `validate_before_apply: add_source == Search`, `on_clear: clear_filters` (see 1.11).

### 1.7 The swipe engine (SwipeStack, `interactions/swipe/stack.rs` + `state.rs`)

- Renders up to `STACK_DEPTH = 10` cards stacked (later cards under; peek offset and scale are both 0 so they sit exactly under the top card). Each card is a `FlippableCardImage` at `ImageSize::Large`, `draggable: false`, flip button only on the top card.
- Only the top card takes touch/mouse input, and not while an enter animation is playing (`entering.is_some()`).
- Drag: the axis locks on first movement (X or Y). X drag translates the card by dx and rotates it `dx * 0.06` degrees. Y drag only follows upward motion (`dy.min(0)`), and only when Up is an allowed direction; a downward drag does not move the card. While dragging, no transition; on release without commit the card springs back (`0.3s cubic-bezier(0.34, 1.56, 0.64, 1)`).
- Commit on release when `distance > 90px` OR (`distance > 32px` AND `speed > 2.5 px/ms`). Direction = sign of the locked axis delta: X<0 Left, X>0 Right, Y<0 Up, Y>0 Down.
- Left/Right/Up: the card is pushed to an "exiting" overlay that animates off-screen from its release position (`card-stack-exit-left|right|up`), and the matching handler fires immediately. Down: no exit animation, `on_swipe_down` fires (used as undo).
- Undo animation: when `entering = Some(dir)` the new top card plays `card-stack-enter-{left|right|up|down}` (comes back in from the side it left), then `entering` is cleared on `animationend`.
- No tap action on the card itself (tapping only hits the Flip button on DFCs). The eye button opens details.
- Allowed directions on all three swipe surfaces here: Left, Right, Up, Down.

### 1.8 Gestures, Search source

**Right (add to deck, quantity 1, mainboard):**

1. Telemetry: `record_swipe(Right)`, `record_signal(deck_id, oracle_id, Right)`.
2. `stack.record(AddAction::Add)`.
3. `add_card_to_deck(card)`: request `HttpCreateDeckCard::new(&card.scryfall_data, 1, None)` (`{ scryfall_data_id, oracle_id, quantity: 1, board: None }`). `in_flight_adds.begin(card_id)`. Spawn `authed.run("add_card", create_deck_card(deck_id, &request, &s))` (`create_deck_card(&self, deck_id: Uuid, request: &HttpCreateDeckCard, session: &Session) -> Result<DeckCard, ClientError>`). When it answers: `outcome = in_flight_adds.finish(card_id)`; if it succeeded and `outcome == Undo` → `delete_undone_add` (spawn `authed.run("undo_add", delete_deck_card(deck_id, card_id, &s))`); else if it succeeded → insert the oracle id into `deck_cards_ids` and `undo_store.push(deck_id, UndoAction::Added { card_id, card_name })`.
4. Toast success "Added to deck" (QUICK) immediately (optimistic).
5. If the card is a land: `mainboard_land_count += 1`; if `land_target` is Some(t) and `prev < t <= now`: toast info `"Land target reached ({target})"` (NORMAL). The filter is not changed mid-session (lands get excluded on the next entry).
6. Budget: if `price_budget > 0`: `deck_total_price += card_price(card, currency)`; if `budget_tier(after) > budget_tier(before)`: message `"Deck at {pct:.2}% of your {amount} budget"` (pct = after / budget * 100, amount = `currency.format_amount(budget)`), toast **warning** if `after >= budget` else **info** (NORMAL).
7. `advance_after_commit()`.

**Left (skip for a week):**

1. Telemetry swipe + signal (Left).
2. If the card has an oracle id: spawn `authed.run_quiet("skip_card", skip_deck_card(deck_id, oracle_id, &s))` (`skip_deck_card(&self, deck_id: Uuid, oracle_id: Uuid, session: &Session) -> Result<(), ClientError>`). Posted immediately (durable). Quiet: no per-swipe error toast.
3. `stack.record(AddAction::Skip)`; toast info "Skipped" (QUICK); `advance_after_commit()`.

**Up (to maybeboard):**

1. Telemetry swipe + signal (Up).
2. `stack.record(AddAction::Maybe)`.
3. `add_card_to_maybeboard(card)`: request `HttpCreateDeckCard::new(&sd, 1, Some("maybeboard"))`, `in_flight_adds.begin`, spawn `authed.run("add_maybeboard", create_deck_card(...))`. On answer, same in-flight handling as Add; on a kept success: oracle id into `deck_cards_ids`, `undo_store.push(Added { card_id, card_name })`, and push `DeckEntry { card, deck_card }` into `mb_entries` (so the Maybeboard source shows it without a refetch).
4. Toast info "Added to maybeboard" (QUICK); `advance_after_commit()`.

**Down:** telemetry `record_swipe(Down)`, then `undo_last_action()`.

**Tap:** none (Flip button on DFC images only; it does not trigger a swipe).

**`advance_after_commit()`**: `total = stack.len()`; `stack.advance()` (stop if false). If `400 <= total < 500` and `total % 100 == 0`: toast info "Approaching the card stack limit, refresh your search to continue" (QUICK). Then if `index >= total && pagination_exhausted`: toast warning "End of results" (default duration). Else if `total > 0 && index + 1 >= total - 5`: `load_more_cards()`.

### 1.9 Undo, Search source (`undo_last_action`)

1. `stack.pop_action()`; none → toast info "Nothing to undo" (default).
2. `stack.step_back(&action)`; false (at the first card) → toast warning "No previous card" (default), `stack.record(action)` back, stop. This primes the enter animation from the action's exit side.
3. The card is now `stack.current()` (linear stacks keep swiped cards). If None → `unwind_undo(action)`.
4. By action:
   - **Skip**: spawn `authed.run("undo_skip", unskip_deck_card(deck_id, oracle_id, &s))` (`unskip_deck_card(&self, deck_id: Uuid, oracle_id: Uuid, session: &Session) -> Result<(), ClientError>`; deletes the suppression row). Toast info "Undid skip" (QUICK).
   - **Add**: compute `was_land`, `was_price`.
     - If `in_flight_adds.undo(card_id)` → the POST hasn't answered; its task will delete. Rewind counts now (land count - 1, floor 0; total price - was_price, floor 0). Toast success "Undid add" (QUICK). Done.
     - `taken = undo_store.take_newest(deck_id, Added { card_id == this })`; `still_in_deck = deck_cards_ids contains oracle id`. If `taken.is_none() && !still_in_deck` → the deck cards Undo already deleted it: toast success "Undid add" (QUICK), no server call.
     - Else spawn: `ensure_fresh` session; on failure: report, toast error `e.to_user_message()` (default), `stack.unwind_undo(action)` (restores history and cursor), push `taken` back. Then `client.delete_deck_card(deck_id, card_id, &session)` (`delete_deck_card(&self, deck_id: Uuid, scryfall_data_id: Uuid, session: &Session) -> Result<(), ClientError>`). Ok: remove oracle id from `deck_cards_ids`, rewind land count and price, toast success "Undid add" (QUICK). Err: toast error `"Failed to undo: {e}"` (default), push `taken` back; cursor and history are NOT restored (the user can add again).
   - **Maybe**: same shape as Add without land/price changes. In-flight → toast success "Undid maybeboard" (QUICK). Already reversed → same toast, no call. Else delete; Ok → remove from `deck_cards_ids`, remove from `mb_entries`, toast success "Undid maybeboard"; Err → `"Failed to undo: {e}"` and push `taken` back. Refresh failure → `unwind_undo` + push back.

### 1.10 Maybeboard source (cycling)

- Filter effect (runs when `add_source` or the counter changes, only on Maybeboard): cards = `mb_entries` cards; if the builder is non-empty, `build_criteria()` and `Cards::matching` (on Err keep all); then `Cards::sorted(builder.sort(), builder.direction())`; `mb_stack.replace(filtered)`. All client-side, no API call.
- Current card = `mb_stack.current_wrapping()`.
- **Right (promote to deck)**: record `MaybeboardAction::Promote { card }`; optimistic: remove from `mb_entries`, `mb_stack.remove_current()`, add oracle id to `deck_cards_ids`; spawn `authed.run("promote_maybeboard", update_deck_card(deck_id, id, &HttpPatchDeckCard::new(None, Some("deck")), &s))` (`update_deck_card(&self, deck_id: Uuid, scryfall_data_id: Uuid, request: &HttpPatchDeckCard, session: &Session) -> Result<DeckCard, ClientError>`); on success `undo_store.push(MovedBoard { card_id, card_name, from: Maybeboard })`. Toast success "Moved to deck" (QUICK). No rollback on failure (the facade toasts). The cursor does not move (next card slides in).
- **Left**: record `Skip`; toast info "Skipped" (QUICK); `mb_stack.advance_wrapping()`. No server call (no durable skip here).
- **Up**: toast info "Already on maybeboard" (QUICK). Nothing recorded; the card exits visually but the stack index is unchanged, so it re-renders as the top card.
- **Down**: `mb_undo_last_action()`:
  - pop; none → toast info "Nothing to undo" (default). `prime_entering(action.exited())`.
  - **Skip**: `retreat_wrapping()` (false → cancel entering, stop); toast info "Undid skip" (QUICK).
  - **Promote { card }**: `taken = take_newest(MovedBoard { card_id, from: Maybeboard })`; `already_back = mb_entries contains card`. If none and already back → cancel entering, toast info "Already changed" (QUICK). Else `mb_stack.insert_current(card)`, spawn: refresh failure → toast error, cancel entering, push taken back; `update_deck_card(..., HttpPatchDeckCard::new(None, Some("maybeboard")))`; Ok → remove oracle id from `deck_cards_ids`, push a `DeckEntry` (quantity 1, board Maybeboard, `mvp_at: None`) to `mb_entries`, toast success "Undid move to deck" (QUICK); Err → toast error `"Failed to undo: {e}"`, push taken back.

### 1.11 Source switch, Synergy, Refresh, Reset

**Switching source** (chip tap, not already selected):

- To Maybeboard: park the current filter as `AddSearch`; set the filter to `restore(AddMaybeboard)` or default (blank).
- To Search: park as `AddMaybeboard`; restore `AddSearch` if any, else build the fresh Search default: legality `[fmt.to_legality_key()]` if a format, `color_identity_within(deck colors)` if known, `synergy = deck_has_commander`, and `ensure_lands_excluded` if `lands_at_target()`.
- Then `add_source = variant`, `mb_stack.rewind()`, bump the counter (Search refetches; Maybeboard refilters).

**Refresh button:**

- Maybeboard: spawn `authed.run("search_cards", fetch_deck(deck_id))`; on success, reset `mb_entries` to the maybeboard entries, `mb_stack.rewind()`, bump the counter. Toast info "Maybeboard refreshed" (QUICK) immediately.
- Search: normalized builder; `build()` Err → toast warning "Filter is empty" (QUICK), stop. Else new ticket, `stack.reset()`, `reset_image_ease()`, `last_search_filter = None`, `pagination_exhausted = false`, `is_loading_cards = true`, record search, then spawn (flush telemetry, `try_run("search_cards", search_deck_cards)`, same result handling as 1.4 but without the empty-results toast). Toast info "Stack refreshed" (QUICK) immediately. No gate check here.

**Reset (filter sheet "Reset" on this screen → `clear_filters`)**, stages a default; Apply commits it:

- Keep `synergy` (it is a mode, not a filter). `has_sort = filter.sort().is_some()`.
- Maybeboard: if `filter.is_empty() && !has_sort` → toast warning "Filter already at default" (QUICK). Else `clear()`, restore synergy, toast info "Filter reset" (QUICK).
- Search: `at_default = filter.is_empty_ignoring_deck_context_and_auto_lands(lands_at_target())`. If `at_default && !has_sort` → "Filter already at default". Else `clear()`, re-apply legality (format), `color_identity_within` (deck colors), synergy, and `ensure_lands_excluded` if at target; toast info "Filter reset" (QUICK).

### 1.12 Synergy warming and related_from toasts

- Synergy warming: an effect watches `synergy_warming`; on a false→true transition (deduped through a previous-value cell so re-searches while still warming do not repeat it) toast info "Synergy warming up; showing all cards for now" (NORMAL).
- related_from: an effect on the current card. When the top card's printing id is in `related_ids` and this has not been announced since the last fresh stack: `related_announced = true`, toast info "That's every synergy pick. Showing cards like your deck" (NORMAL), and telemetry `record_pile_end(deck_id, PileEndKind::SynergyExhausted, true)`.

### 1.13 End of results

- Reaching one past the last card with `pagination_exhausted` toasts warning "End of results" (default).
- The end state shows the "No cards" skeleton that accepts a down-pull (≥ 60 px, vertical dominant) to undo back into the pile.
- If a prefetch is still running, the loading skeleton shows instead.

### 1.14 Quantities and the sideboard on Add

- Every swipe adds exactly 1 copy. There is no quantity control and no sideboard action on this screen. Right = mainboard (`board: None`), Up = maybeboard. Sideboard only via the deck cards view's move buttons.

### 1.15 Image preview on Add

- None. The card image is the swipe card; DFCs flip via the Flip overlay; details via the eye sheet; printings via the details sheet.

### 1.16 Animations

- Screen content uses `content-enter`.
- Card drag tilt and spring-back, exit animations (left/right/up), enter animation on undo from the exit side, image ease-in on first load (reset on each fresh search), skeleton shimmer.
- Bottom sheets slide up with a backdrop. The printing sheet is a carousel.

### 1.17 Toast inventory, Add screen

| Trigger | Kind | Text | Duration |
|---|---|---|---|
| synergy turns cold | info | Synergy warming up; showing all cards for now | NORMAL |
| stack ≥ 500 on load more | warning | Card stack limit reached, please refresh your search to continue | NORMAL |
| every 100 cards from 400 | info | Approaching the card stack limit, refresh your search to continue | QUICK |
| past last card, exhausted | warning | End of results | default |
| undo with no history | info | Nothing to undo | default |
| undo at first card | warning | No previous card | default |
| undo skip | info | Undid skip | QUICK |
| undo add | success | Undid add | QUICK |
| undo maybe | success | Undid maybeboard | QUICK |
| undo failed | error | Failed to undo: {e} | default |
| session refresh failed in undo | error | {ClientError::to_user_message()} | default |
| reset at default | warning | Filter already at default | QUICK |
| reset | info | Filter reset | QUICK |
| gate refuses blank query | warning | Try a filter to start swiping | NORMAL |
| empty result | warning | No results for this filter[; Synergy is warming up, so all cards were searched \| ; tap Synergy to turn it off] | NORMAL |
| related cards begin | info | That's every synergy pick. Showing cards like your deck | NORMAL |
| maybeboard: promote undo, already reversed | info | Already changed | QUICK |
| maybeboard: undo promote | success | Undid move to deck | QUICK |
| swipe left | info | Skipped | QUICK |
| swipe right (search) | success | Added to deck | QUICK |
| land target crossed | info | Land target reached ({target}) | NORMAL |
| budget band crossed | info / warning (≥100%) | Deck at {pct:.2}% of your {amount} budget | NORMAL |
| swipe up (search) | info | Added to maybeboard | QUICK |
| swipe right (maybeboard) | success | Moved to deck | QUICK |
| swipe up (maybeboard) | info | Already on maybeboard | QUICK |
| refresh (maybeboard) | info | Maybeboard refreshed | QUICK |
| refresh with empty filter | warning | Filter is empty | QUICK |
| refresh (search) | info | Stack refreshed | QUICK |
| printing saved | info | Printing swapped | QUICK |
| any facade failure | error | {to_user_message()} | NORMAL |

### 1.18 The printing sheet (`components/printing_sheet.rs`)

Props: `card`, `open`, `on_save(Card)`, `host_screen`, `read_only` (default false), `saved_message` (default "Printing saved").

- Hand-built bottom sheet layered over the details sheet (`sheet-over`), with a backdrop and the GridCells background.
- Header: "Printings".
- On open (requires an oracle id; without one nothing loads): if the cached oracle id equals this card's, reuse the list and jump the carousel to the current printing (no animation). Else `is_loading = true` and `client.fetch_printings(oracle_id)` (`fetch_printings(&self, oracle_id: Uuid) -> Result<Vec<Card>, ClientError>`, unauthenticated). On Ok: initial index = position of the current printing id (else 0), cache the oracle id, build the carousel. On Err: report and toast error `to_user_message()` (NORMAL).
- Body:
  - Loading: `PrintingSheetSkeleton` (image ghost, 5 dot ghosts, 4 info bars).
  - More than one printing: a swipeable **Carousel** of `FlippableCardImage` (Large), **CarouselDots** (current/total), and `CardInfoDisplay` of the visible printing.
  - Exactly one: the image (if it has one) centered, then `CardInfoDisplay`.
- `has_changed` = list non-empty and carousel index ≠ initial index.
- Footer ActionBar: "Back" (if not read-only and changed: toast info "Printing unchanged" (QUICK); then close). "Save" (hidden when read-only; disabled until changed): `on_save(visible)`, toast info `saved_message` (QUICK), close.
- Backdrop tap and OS back: close; if not read-only and changed, toast **warning** "Printing unchanged" (QUICK).

---

## 2. The card filter sheet (`filter/card_filter_sheet.rs` + sections)

One component, `CardFilterSheet`, shared by Add, deck cards view and Remove. Props: `open: Signal<bool>`, `show_format_filter` (default false; all three hosts pass true), `show_active_indicators` (default false; all three pass true), `validate_before_apply` (default false; Add passes `add_source == Search`), `on_clear: Option<EventHandler>` (Add only).

It edits the host's `Signal<CardQueryBuilder>` from context (the "draft"), and commits by bumping the host's `filter_reset_counter` (context). Each host decides what a commit does: Add Search refetches from the server; Add Maybeboard, the view and Remove refilter in memory.

### 2.1 Draft vs applied

- When the sheet opens, snapshot the builder (`applied_snapshot`).
- Sections write straight into the builder (live draft).
- Closing without Apply (Back button, backdrop tap, OS back): if the builder differs from the snapshot, restore the snapshot and toast info "Filter unchanged" (QUICK). No toast if nothing was edited.
- Apply drops the snapshot first, so the close keeps the commit.
- `unchanged` = snapshot == current builder. Apply is **disabled** (grayed) while unchanged.

### 2.2 Sheet frame, top to bottom

- Backdrop (tap closes). Bottom sheet with the GridCells background.
- Header: title "Filter" and a "?" button (dimmed, right) that opens the filter hint dialog.
- The first open per account auto-opens the hint (`HINT_FILTER`, via `open_and_record_hint`, gated on the first `open`).
- When the sheet opens and any public catalog prefetch failed (`CatalogCache::any_public_failed()`), toast warning "Filter lists didn't load, trying again" (NORMAL) once per opening. Each picker's `ensure_*` refetches failed catalogs on mount, so opening is the retry.
- Content: an **accordion** (single open, collapsible), mounted only while the sheet is open so every section starts collapsed on each reopen. Opening a section smooth-scrolls it to the top of the sheet.
- Each accordion trigger shows its title and, when that section has anything set (the "active indicator"), a "×" clear button that clears the section's fields (stopping propagation so it does not toggle the accordion). Clearing writes to the draft only; Apply still commits.
- Footer ActionBar: "Back", "Apply", "Reset".

### 2.3 Accordion sections in order (title, content, what the × unsets)

| # | Title | Section | Active when / × unsets |
|---|---|---|---|
| 1 | "Name" | Name | `name_contains`, `name_not_contains` |
| 2 | "Oracle text" | OracleText (TextContains, OracleWords, Keywords) | `oracle_text_contains`, `oracle_text_contains_any`, `oracle_text_contains_all`, `keywords_contains_any`, `keywords_contains_all`, `oracle_text_not_contains`, `oracle_text_excludes_any`, `keywords_excludes` |
| 3 | "Types" | Types (BasicTypes, OtherTypes) | `type_line_contains`, `type_line_contains_any`, `type_line_contains_all`, `card_type_contains_any`, `card_type_contains_all`, `type_line_not_contains`, `type_line_excludes_any`, `card_type_excludes_any` |
| 4 | "Mana" | Mana (Cmc, ColorIdentity, ProducedMana) | `cmc_equals`, `cmc_range`, `color_identity_equals`, `color_identity_within`, `produced_mana_contains_any`, `produced_mana_contains_all`, `produced_mana_excludes` |
| 5 | "Combat" | Combat (Power, Toughness) | `power_equals`, `power_range`, `toughness_equals`, `toughness_range` |
| 6 | "Flavor text" | FlavorText | `flavor_text_contains`, `has_flavor_text`, `flavor_text_not_contains` |
| 7 | "Artist" | Artist | `artist_equals_any`, `artist_excludes_any` |
| 8 | "Rarity" | Rarity | `rarity_equals_any`, `rarity_excludes_any` |
| 9 | "Card roles" | CardRole | `card_roles_contains_any`, `card_roles_contains_all`, `card_roles_excludes` |
| 10 | "Oracle tags" plus an info "?" button (`HintTopic::OracleTags`, opens the oracle-tags explainer) | OracleTags | `oracle_tags_contains_any`, `oracle_tags_contains_all`, `oracle_tags_excludes` |
| 11 | "Set" | Set | `set_equals_any`, `set_excludes_any` |
| 12 | "Format" (only if `show_format_filter`) | FormatFilter | `legalities_contains_any`, `is_commander_in_format`, `is_partner`, `is_background`, `is_signature_spell` |
| 13 | "Price" | PriceFilter | `price_min`, `price_max` (currency is not an indicator) |
| 14 | "Sort" | Sort | `sort` is Some. × calls `unset_sort()` (direction untouched) |
| 15 | "Config" | Config | any of `is_playable`, `digital`, `oversized`, `promo`, `content_warning` differs from the default. × sets `is_playable = true`, `digital = false`, `oversized = false`, `promo = false`, `content_warning = false` (note: × sets promo to `false`, while the builder default for promo is `None`) |

There is no separate "Deck cards", "Match mode" or "Filter mode" section: those are the in-section mechanics described in 2.7 (`deck_cards.rs` = picker data source; `match_mode.rs` = Any/All chip; `filter_mode.rs` = Exact/Range chip).

### 2.4 Footer buttons

**Back**: `open = false` (draft discarded by the close rule, with "Filter unchanged" if edited).

**Apply** (disabled while unchanged):

1. If `filter.build()` returns `InvalidCardCriteria::Contradiction { field, .. }`: toast warning `"Filter can't both include and exclude {field}"` (NORMAL), keep the sheet open. `field` is one of "card types", "type line", "oracle text", "keywords", "produced mana", "mechanical categories" (card roles), "oracle tags", "sets", "artists", "rarities", or, for contains vs doesn't-contain pairs, "name", "oracle text", "flavor text", "type line".
2. `to = has_search_intent(draft)`, `from = has_search_intent(snapshot)`; `apply_action(validate_before_apply, from, to, unchanged)`:
   - `validate && from Empty && to Empty` → **RefuseEmpty**: toast warning "Filter is already empty" (QUICK), keep the sheet open (no close, no revert).
   - `unchanged` → **NoChange**: close silently (no refetch; keeps the user's place in the stack).
   - else **Commit { cleared: from Set && to Empty }**.
3. Commit: if the host provides `CollapseExpanded` (view only), set it true (collapse the expanded row). Bump `filter_reset_counter`. Drop the snapshot. Toast success "Filter cleared" if cleared else "Filter applied" (QUICK). Close.

`has_search_intent()` = `!is_empty_ignoring_deck_context() || sort.is_some() || synergy`. `is_empty()` compares against the builder with only config retained (`limit`, `offset`, `is_token`, `is_playable`, `digital`, `oversized`, `promo`, `content_warning`, `language`, `sort`, `direction`, `synergy`, `include_skipped`, `price_currency`). So config toggles, sort, direction, synergy and currency never make a filter "non-empty" by themselves. `is_empty_ignoring_deck_context()` additionally ignores `legalities_contains_any` and `color_identity_within`.

**Reset**: if the host gave `on_clear` (Add), call it (see 1.11). Otherwise `filter.clear()` (back to `CardQueryBuilder::default()`) and toast info "Filter reset" (QUICK). Either way this only stages the default; Apply commits, Back reverts.

`CardQueryBuilder::default()`: everything None except `is_playable: Some(true)`, `digital: Some(false)`, `oversized: Some(false)`, `promo: None`, `content_warning: Some(false)`, `language: Some("en")`, `limit: Limit::DEFAULT`, `offset: 0`, `sort: None`, `direction: Ascending`, `synergy: false`, `include_skipped: false`. `clear()` = reset to default (note: unlike `retain_config` it drops synergy, which is why Add re-applies it).

### 2.5 Hint dialog "Filters"

- "Picks which cards show: the ones served to **swipe**, or which of the deck's cards list" (swipe in primary accent)
- "Open a section to set name, mana, type, color, **Oracle tags** and more. Stack as many as you want" (tertiary accent)
- "**Apply** uses it. **Reset** then **Apply** goes back to the default view" (Apply as key chip; Reset as warning key chip)
- "Each screen keeps its own filter"

### 2.6 Oracle tag dictionary overlay

The include and exclude "Dictionary" chips in the Oracle tags section open `OracleTagDictionary` (full-screen, rendered outside the sheet) with `dict_exclude` false/true. Its "Use" on a tag (`adopt_otag`): include bucket = All if `oracle_tags_contains_all` is set, else Any. If the slug is already in the target list → toast info "Already in filter" (QUICK). Else append it → toast success "Added to filter" (QUICK). The dictionary itself (title "Oracle tags", search placeholder "Search tags or descriptions", letter browse, "Examples", "Use", "Back", empty states "No tags match “{q}”." and "No tags start with {L}.", load error toast "Couldn't load the oracle-tag dictionary" and inline "Couldn't load the dictionary. Pull back and try again.") belongs to another spec; only the hook-up is here.

### 2.7 Shared control vocabulary in sections

- **Section label row**: small label (`label-xs`), then an optional small mode chip (`chip-xs`), then an optional "×" clear button.
- **Mode chips**: `MatchMode` label "Any" / "All" (default Any; the section opens as All when the `*_contains_all` list is set). `FilterMode` label "Exact" / "Range" (default Exact). `IncludeExclude` label "Include" / "Exclude" (default Include; opens as Exclude when the excludes list is set). Toggling a mode moves the current values to the other builder field.
- **Toggle chips**: `chip` / `chip selected`.
- **Searchable picker**: selected values as `chip selected` pills each with a "×" remove button; a text input (placeholder "Type to search" unless noted) with its own "×" clear when non-empty; while the input is non-empty, a floating result list above it of `chip-unselected` items (case-insensitive substring over the source list, excluding values already picked; capped at 8 for keywords, oracle words, type line and oracle tags, and 5 for artist and set); tapping a result appends it. The search boxes reset to empty whenever the host's `filter_reset_counter` changes. Picking does not clear the search text.
- **Text inputs** write on every keystroke; an empty string unsets (setters map "" to None).
- **Numeric text inputs** (mana value, price) keep a local string and parse on blur.
- **Steppers** (power, toughness): "-" value "+"; the value shows "-" when unset.
- **Data source for pickers** (`deck_cards.rs`): when the host provides `DeckCards` context (view and Remove do; Add does not), keywords, oracle words, type-line words, artists and sets come from the deck's own cards (`deck_cards::extract_*`), so the lists only offer what the deck has. Otherwise they come from the app-wide `CatalogCache`, fetched once: `fetch_keywords()`, `fetch_oracle_words()`, `fetch_card_types()`, `fetch_artists()`, `fetch_sets()` (all `-> Result<Vec<String>, ClientError>`, unauthenticated). Card roles and oracle tags always come from the catalog: `fetch_card_roles() -> Vec<CardRoleView>` (`slug`, `display_name`), `fetch_oracle_tags() -> Vec<OracleTag>` (`slug`, `label`, `description`). While a catalog is not loaded the picker just shows no results or chips.
- `deck_cards::extract_type_words` (split type lines on whitespace, trim `: - ? ,`, lowercase, drop `TYPE_STOP_WORDS`, sort, dedup), `extract_oracle_words` (keep ASCII letters only, lowercase, drop `ORACLE_STOP_WORDS`), `extract_keywords` (trim, lowercase), `extract_artists`, `extract_sets` (set names). Stop words: `zwipe-core/.../search_card/stop_words.rs`.

### 2.8 Each section exactly

**Name** (`filter/name.rs`)

- Label "Name contains" (+ × when set), input placeholder "Name contains" → `set_name_contains(v)` / `unset_name_contains()`.
- Label "Name doesn't contain" (+ ×), input placeholder "Name doesn't contain" → `set_name_not_contains` / `unset_name_not_contains`.

**Oracle text** (`filter/oracle_text/*`), three blocks in order:

1. TextContains: "Oracle text contains" / placeholder "Oracle text contains" → `set_oracle_text_contains`; "Oracle text doesn't contain" / placeholder "Oracle text doesn't contain" → `set_oracle_text_not_contains`. Each with ×.
2. OracleWords: label "Oracle words include", mode chip Any/All (always shown), × when any selected (clears the list and the search). Picker → Any: `set_oracle_text_contains_any(words)`, All: `set_oracle_text_contains_all(words)` (writing always unsets both first; empty → both None). Label "Oracle words exclude" (+ ×) and its picker → `set_oracle_text_excludes_any` / `unset_oracle_text_excludes_any`. Source: deck words or `fetch_oracle_words`. Cap 8, results exclude words already included or excluded.
3. Keywords: label "Keywords include", mode chip Any/All (always shown), ×; picker → `set_keywords_contains_any` / `set_keywords_contains_all`. Label "Keywords exclude" (+ ×), picker → `set_keywords_excludes` / `unset_keywords_excludes`. Source: deck keywords (lowercased) or `fetch_keywords`. Cap 8.

**Types** (`filter/types/*`):

1. BasicTypes: label "Basic types include", mode chip Any/All **only when something is selected**, × when selected. Chips `Artifact`, `Creature`, `Planeswalker`, `Instant`, `Sorcery`, `Enchantment`, `Land` (`Vec::<CardType>::with_all_card_types()`), toggling membership → Any: `set_card_type_contains_any(Vec<CardType>)`, All: `set_card_type_contains_all`. Label "Basic types exclude" (+ ×), the same seven chips → `set_card_type_excludes_any` / `unset_card_type_excludes_any`.
2. OtherTypes: label "Type line include", mode chip Any/All (always shown), ×; searchable picker → `set_type_line_contains_any` / `set_type_line_contains_all`. Label "Type line exclude" (+ ×), picker → `set_type_line_excludes_any`. Source: deck type words or `fetch_card_types`. Cap 8.
   - `type_line_contains` and `type_line_not_contains` (single strings) have no UI; only the section × clears them.

**Mana** (`filter/mana/*`):

1. CmcFilter: label "Mana value", mode chip Exact/Range (always shown; toggling unsets the other mode's field and clears its strings), × when either is set.
   - Exact: one narrow input, placeholder "Mana value", decimal keypad. On blur: empty → `unset_cmc_equals`; parses as f64 → `set_cmc_equals(n)` and normalize the text to `n.to_string()`; else toast error "Invalid mana value" (QUICK), builder unchanged.
   - Range: inputs placeholder "Min", text "to", placeholder "Max". On blur of either: both empty → `unset_cmc_range`; one empty → nothing; both parse → `set_cmc_range((min, max))`; else toast error "Invalid mana value range" (QUICK).
2. ColorIdentityFilter: label "Color identity", mode chip Exact/Range (always shown). Exact maps to `color_identity_equals(Colors)`, Range ("within") maps to `color_identity_within(Colors)`. Initial mode: Exact if `equals` is set, Range if `within` is set, else Exact. Toggling moves the selected colors to the other field. Five mana-symbol chips W U B R G (`Color::all()`, tooltip = long name); toggling reads whichever field is set, then writes into the current mode's field (empty → unset). × unsets both. On Add the deck context pre-sets `within` = commander colors, so the section opens in Range with those colors selected.
3. ProducedManaFilter: label "Produces", mode chip Any/All (always shown), ×. Six symbol chips `W` White, `U` Blue, `B` Black, `R` Red, `G` Green, `C` Colorless → strings into `set_produced_mana_contains_any` / `_all`. Label "Doesn't produce" (+ ×), same six chips → `set_produced_mana_excludes` / `unset_produced_mana_excludes`.

**Combat** (`filter/combat/*`), Power then Toughness, identical:

- Label "Power" (or "Toughness"), mode chip Exact/Range (always shown; toggling unsets the other mode's field), × when set.
- Exact: stepper "-" [value, or "-" when unset] "+". From unset, "-" sets -1 and "+" sets 1 (i32, saturating) → `set_power_equals(n)`.
- Range: two steppers separated by "to" (min, max), both starting from `(0, 0)` when unset → `set_power_range((min, max))`.
- Toughness uses `toughness_equals` / `toughness_range`.

**Flavor text** (`filter/flavor_text.rs`):

- "Flavor text contains" / placeholder "Flavor text contains" → `set_flavor_text_contains`.
- "Flavor text doesn't contain" / placeholder "Flavor text doesn't contain" → `set_flavor_text_not_contains`.
- `has_flavor_text` has no control; only the section × clears it.

**Artist** (`filter/artist.rs`):

- Label "Artist"; when something is selected, a mode chip "Include"/"Exclude" and ×. Selected pills, picker (placeholder "Type to search", cap 5, excludes already selected). Include → `set_artist_equals_any(Vec<String>)`, Exclude → `set_artist_excludes_any` (writing unsets both first). Source: deck artists or `fetch_artists`.

**Rarity** (`filter/rarity.rs`):

- Label "Rarity"; mode chip "Include"/"Exclude" and × only when something is selected. Chips "Common", "Uncommon", "Rare", "Mythic", "Bonus", "Special" (`CardRarity::all()`, `to_long_name()`), toggle → `set_rarity_equals_any(Rarities)` / `set_rarity_excludes_any(Rarities)` (empty → unset).

**Card roles** (`filter/card_role.rs`):

- Label "Card roles include"; mode chip Any/All and × only when selected. One chip per `CardRoleView` (shows `display_name`, stores `slug`) → `set_card_roles_contains_any(slugs)` / `set_card_roles_contains_all(slugs)`. The mode is derived from the builder (All iff `contains_all` is set).
- Label "Card roles exclude" (+ ×), the same chips → `set_card_roles_excludes` / `unset_card_roles_excludes`.
- Source: `fetch_card_roles()` via the catalog cache (always, even with DeckCards).

**Oracle tags** (`filter/oracle_tags.rs`):

- Label "Oracle tags include", a "Dictionary" chip (opens the dictionary for includes), and when selected: mode chip Any/All and ×. Selected slugs as pills with ×. Input placeholder "Search all oracle tags"; results from `oracle_tag::search_oracle_tags(tags, query)` (rank: exact slug or label match, then substring in slug or label, then substring in description; ties by slug), minus already selected, cap 8, each shown by slug → `set_oracle_tags_contains_any` / `set_oracle_tags_contains_all`.
- Label "Oracle tags exclude", "Dictionary" chip (excludes), × when set; pills; input "Search all oracle tags"; results minus selected and excluded → `set_oracle_tags_excludes` / `unset_oracle_tags_excludes`.
- Source: `fetch_oracle_tags()` via the catalog cache.

**Set** (`filter/set.rs`): identical to Artist with label "Set". Include → `set_set_equals_any(Vec<String>)` (set names), Exclude → `set_set_excludes_any`. Source: deck set names or `fetch_sets`. Cap 5.

**Format** (`filter/format.rs`, only when `show_format_filter`):

- Label "Is commander in" (+ × when set). Single-select chips over `Format::commander_formats()`: Brawl, Commander, Duel Commander, Historic Brawl, Oathbreaker, Pauper Commander, PreDH, Standard Brawl (display names, in that order). Tapping the selected one → `unset_is_commander_in_format()`; else `set_is_commander_in_format(fmt)`.
- Label "Is legal in" (+ ×). Multi-select chips over `Format::all()` (commander formats first, then the rest alphabetically), display names. Toggling edits `legalities_contains_any(Vec<String>)` of `fmt.to_legality_key()` strings (empty → unset). On Add this list is pre-seeded with the deck's format.
- `is_partner`, `is_background`, `is_signature_spell` have no UI here; only the section × clears them.

**Price** (`filter/price.rs`):

- Label "Price" (+ × when min or max is set; × also clears the text boxes).
- Currency chips "USD", "EUR", "TIX" (`PriceCurrency::all()`, `label()`), single select, selected = `price_currency()` or the USD default → `set_price_currency(c)`. Currency alone never makes the filter active.
- Inputs placeholder "Min", "to", placeholder "Max" (decimal keypad). On blur: empty → unset; parses f64 → `set_price_min(n)` / `set_price_max(n)`; else toast error "Invalid price" (QUICK).

**Sort** (`filter/sort.rs`):

- Label "Sort by" (+ × when set). Single-select chips (tapping the selected one unsets): "Name", "Mana value", "Power", "Toughness", "Rarity", "Release Date", "Price (USD)", "Price (EUR)", "Price (TIX)", "Popularity (EDHREC)", "Random" (`CardSortKey::all()`) → `set_sort(key)` / `unset_sort()`.
- Label "Sort order", a single chip showing "Ascending" or "Descending"; tap → `set_direction(direction.flipped())`. Default Ascending. Direction alone is config (no indicator).
- Per host: Add Search sends the sort to the server (an explicit sort replaces synergy ordering; synergy pool membership still applies while synergy is on). Add Maybeboard and Remove sort in memory with `Cards::sorted(sort, direction)` (None keeps order). The view uses Name when no sort is set.

**Config** (`filter/config.rs`): five tri-toggles, each a label then three chips "Yes", "No", "Any" (selected chip reflects `Some(true)`, `Some(false)`, `None`; tapping the selected Yes or No unsets; "Any" unsets):

- "Is playable" → `is_playable` (default Yes)
- "Is digital" → `digital` (default No)
- "Is oversized" → `oversized` (default No)
- "Is promo" → `promo` (default Any)
- "Has content warning" → `content_warning` (default No)

### 2.9 How each host consumes the builder

- `build()` → `CardQuery` for the server (`build_criteria()?` + limit, offset, sort, direction, synergy, include_skipped). `build_criteria()` returns `InvalidCardCriteria::Empty` only when `is_empty() && sort.is_none() && !synergy`, else validates contradictions.
- In memory: `build_criteria()` then `Cards::from(cards).matching(&criteria)`; pinned single cards use `criteria.matches(&card)`.
- Add sets `is_token(false)`, `limit(25)`, `offset(0)` on a clone before `build()`. Quick add uses `CardQueryBuilder::with_name_contains(q)`, `set_synergy(false)`, `set_include_skipped(true)`, `set_sort(CardSortKey::Name)`, `set_limit(5)`.

### 2.10 Toasts in the filter sheet

| Trigger | Kind | Text | Duration |
|---|---|---|---|
| open with a failed catalog | warning | Filter lists didn't load, trying again | NORMAL |
| close with edits, no Apply | info | Filter unchanged | QUICK |
| contradiction on Apply | warning | Filter can't both include and exclude {field} | NORMAL |
| Apply refused (validate, empty to empty) | warning | Filter is already empty | QUICK |
| Apply commit | success | Filter applied / Filter cleared | QUICK |
| Reset (no host override) | info | Filter reset | QUICK |
| dictionary Use, already present | info | Already in filter | QUICK |
| dictionary Use | success | Added to filter | QUICK |
| bad mana value | error | Invalid mana value | QUICK |
| bad mana range | error | Invalid mana value range | QUICK |
| bad price | error | Invalid price | QUICK |

### 2.11 Animations

Sheet slide-up with backdrop fade; accordion expand and collapse with a smooth scroll of the opened item; floating search results stagger in (`animation-delay: i*40ms, 250+i*40ms`).

---

## 3. Deck cards view (`view.rs`)

Route `/deck/card/view/:deck_id` (`Router::ViewDeckCard { deck_id }`). Reached by push from the deck view (`screens/deck/view.rs`). Leaves with "Back" (`go_back`), "Add" (push `AddDeckCard`), "Remove" (push `RemoveDeckCard`).

### 3.1 State

- `filter_builder` from `FilterStore::restore(Cards, deck_id)` or default; parked on unmount. Provided as context.
- `filter_reset_counter` (context): bumping it re-runs the filter + group pass.
- `UndoLog` = `UndoStore::restore(deck_id)`, provided as context (quick add records into it), parked back on unmount.
- `deck_entries: Vec<DeckEntry>` (source of truth: all non-command-zone entries, every board), `command_zone_cards: Vec<Card>` (for price totals), deck identity (`deck_name`, `deck_format`, `deck_power`, `deck_tags: Vec<String>`, `deck_other_tags: Vec<DeckOtherTag>`), `commander_card`, `partner_card`, `background_card`, `signature_spell_card` (+ their filtered `displayed_*`), `is_oathbreaker`.
- `DeckCards(deck_cards_for_filter)` context = all entry cards (every board) for the filter pickers.
- `land_target` (explicit only), `price_budget`, `price_budget_currency` (context for rows; default USD).
- `group_by_option` default `GroupByOption::CardType`; `expanded_card: Option<Uuid>` (one row open at a time); `show_tokens` false; `show_command_zone` true; `show_deck` true; `show_maybe` false; `show_side` false; `show_row_art` true (context `ShowRowArt`); `collapsed_groups: HashSet<String>` (ephemeral per visit).
- `should_collapse_expanded` (context `CollapseExpanded`): when true, the next filter pass collapses the expanded row.
- `preview_card: Option<(ScryfallData, face)>`, `preview_dismissing`, printing sheet state (`printing_sheet_open`, `printing_sheet_card`, `command_zone_slot: Option<CommandZoneSlot>`), oracle-tag examples state (`otag_examples_open`, `otag_examples_slug`), `OtagDescribe` and `OtagExamplesOpen` contexts (rows can reveal tag descriptions and open examples).
- `pending_qty: HashMap<Uuid, PendingQty { baseline, delta, generation }>`, `removed_rows: HashMap<Uuid, DeckEntry>`. `QTY_FLUSH_MS = 300`.
- `row_meta` memo: printing id → `{ qty, board, mvp }` (first entry wins).

### 3.2 Loading

Mount: one `authed.run("load_deck", ...)` doing:

1. `c.fetch_deck(deck_id, &s)` (`Deck { deck_profile, entries, command_zone_cards, .. }`). Failure aborts (facade toasts).
2. `command_zone_cards = deck.command_zone_cards`.
3. `c.fetch_deck_profile(deck_id, &s)` (`fetch_deck_profile(&self, deck_id: Uuid, session: &Session) -> Result<DeckProfile, ClientError>`); errors tolerated. Sets name, format, power level, tags, other tags, `is_oathbreaker = format.has_signature_spell()`, `land_target`, `price_budget`, currency.
4. For commander, signature spell, partner, background ids: `c.fetch_card(id)`; remove any entry with the same **oracle id** from `entries` (command-zone cards never appear in the groups); store the card in its slot.
5. `deck_cards_for_filter = entries' cards`; `deck_entries = entries`; `deck_loaded = true`.
6. If the restored filter is non-empty: toast warning "Filter is active" (QUICK).
7. Bump the counter.

Tokens: a resource `authed.run_quiet("load_tokens", fetch_deck_tokens(deck_id, &s))` (`fetch_deck_tokens(&self, deck_id: Uuid, session: &Session) -> Result<Vec<Card>, ClientError>`), failure → empty.

Hint: once `deck_entries` becomes non-empty for the first time, `open_and_record_hint(HINT_DECK_CARDS, ...)` (an empty deck does not burn the one showing).

### 3.3 Filter + group pass (on counter or group-by change, after load)

1. Active cards = entries with an active board (`Board::Deck`).
2. If the builder is non-empty: `build_criteria()`, `Cards::matching` on the active cards, and filter each pinned command-zone card with `criteria.matches` (a filtered-out pinned card disappears). On Err keep everything.
3. Sort with `builder.sort().unwrap_or(CardSortKey::Name)` and `builder.direction()`.
4. Partition into lands (`is_land()`) and nonlands; `nonlands.group_by(group_by_option)` → `Vec<CardGroup { label, pips: Option<Vec<Color>>, cards }>` (`zwipe-core/.../group_cards.rs`).
5. Store `displayed_groups`, `displayed_lands`, `displayed_*` pinned cards. If `should_collapse_expanded`, collapse the expanded row and reset the flag.

### 3.4 Layout, top to bottom

1. **ScreenHeader** "Deck cards" ("?" opens the hint).
2. **Header block** (max width 40rem):
   - Loaded: deck name (`deck-cards-header-name`), then chips in one wrapping row: format `display_name()` (p3 accent), power level `display_name()` (p5), each tag as `deck::deck_tag_label(&tag)` (muted), each other tag `display_name()` (p5).
   - Not loaded: a title bar ghost plus three chip ghosts (sm, md, sm).
3. **FeaturedCards** (`screens/deck/components/featured_cards.rs`): `cards` = command zone then MVPs: `(commander, "Oathbreaker" if oathbreaker else "Commander")`, `(partner, "Partner")`, `(background, "Background")`, `(signature spell, "Signature spell")`, then mainboard entries with `mvp_at` set, sorted by `mvp_at` ascending, each labeled "MVP". Renders nothing until the deck has loaded or when empty. Each card: image (`ImageSize::Normal`, flippable), name, role chip (MVP styled differently from zone roles). The row stays at zero height until every image has loaded or 4 s pass, then deals the cards in; once open it stays open. Tap a card → open the fullscreen image preview on face 0. No skeleton.
4. Controls (only once loaded; otherwise skeletons: `QuickAddSkeleton`, three `ChipRowSkeleton`s and `DeckCardListSkeleton`):
   - **QuickAdd** (see 3.6).
   - **Group by** row: label chip "Group by", chips "Type", "Mana value", "Color", "Card role" (`GroupByOption::all()`, single select). Tap → set collapse flag, set the option (the pass reruns).
   - **Boards** row: label "Boards", chips "Main", "Maybe", "Side" (multi-select). Each tap toggles; if all three would be off, "Main" snaps back on. Set the collapse flag, bump the counter. "Maybe" is disabled when the deck has no maybeboard cards and it is not already on; same for "Side" with the sideboard.
   - **Show** row: label "Show", chips "Tokens" (toggle `show_tokens`), "Command zone" (toggle `show_command_zone`), "Art" (toggle row art thumbnails).
5. **Sections**, in this order:
   1. **Tokens** (if `show_tokens` and the token list is loaded and non-empty): collapsible group "Tokens ({n})", rows sorted by the builder's sort or Name. Token rows are plain (qty 1, no actions besides the defaults Flip/Image). The filter does not apply to tokens.
   2. **Maybeboard** (if `show_maybe` and non-empty after the filter): entries on the maybeboard, filtered with the builder's criteria when non-empty, sorted with `cards::sort_deck_entries(key or Name, direction)`; group "Maybeboard ({count})" (count = rows). Rows: qty, qty stepper, move buttons for `current_board = Maybeboard`, Printing. No star.
   3. **Sideboard** (if `show_side` and non-empty): same, "Sideboard ({count})", `current_board = Sideboard`.
   4. **Command zone** (if `show_command_zone`), non-collapsible `card-group`s:
      - If displayed commander or partner: header "Oathbreaker" if oathbreaker, else "Commanders" if a partner is displayed, else "Commander"; a row for the commander and one for the partner (qty 1, Printing only).
      - Displayed background: header "Background", row with Printing.
      - Displayed signature spell: header "Signature spell", row with Printing.
   5. **Card groups** (if `show_deck`): one collapsible group per `CardGroup`. Title `"{label} ({qty_count})"` where `qty_count` = sum of row quantities; color groups have an empty label and show identity pips before `"({qty_count})"`. `section_key` = label, or the pips' short names joined. Rows: full actions incl. MVP star.
   6. **Lands** (if `show_deck` and any): collapsible "Lands ({qty_count})", key "Lands", full actions incl. star. Lands are always pulled out of the group-by pipeline and pinned last.
   7. Empty: if `show_deck` and no groups and no lands and loaded: muted text "No cards".
6. **ActionBar**: "Back"; "Add" → push Add; "Remove" → push Remove; "Filter" (dot when `!filter.is_empty() || sort.is_some()`); "Undo" only while the undo log is non-empty; "Reset filter" (`btn-action-clear`) only while `!filter.is_empty()`: `filter.clear()`, collapse flag, bump the counter, toast info "Filter reset" (QUICK). This one applies immediately (no sheet).
7. Overlays: **CardFilterSheet** (`show_format_filter`, `show_active_indicators`; no `on_clear`, no validation), **ImagePreview**, **HintDialog**, **PrintingSheet** (when a printing card is set), **OracleTagExamples** (when open).

**Collapsible group** (`CollapsibleGroup`): header with a "▸" arrow (rotated down when open), optional pips, title; tap toggles the key in `collapsed_groups`. Body collapses with a grid-rows ease; rows stay mounted so the expanded row survives. Groups enter with `row-enter`.

**Hint dialog** "Browsing your deck":

- Line: "Tap a row for details, quantity and printings"
- "Tap a **keyword** chip for what it does, or a **role** chip for its oracle tags" (palette-1 color)
- "**Group by** sorts by type, mana value, color or role"
- "**Boards** picks main, maybe or side"
- "**Show** toggles tokens, the command zone and row art"
- "**Star** marks up to three MVPs. Suggestions lean toward cards that share their roles" (Star as a warning-colored key)

### 3.5 Card rows (shared `CardRow`, `components/src/card_row.rs`, wrapped by `components/card_row.rs`)

**Compact line** (tap toggles `expanded_card` to this printing id, or closes it):

- Art-crop thumbnail at the far left when the host provides `ShowRowArt` (view only) and the card has an art crop; hidden (eased out) when Art is off.
- Arrow "▸" (rotates when expanded).
- Quantity number (or `qty_tags` in its place; unused here).
- Name, prefixed by a "★" when the row is an MVP.
- Stat tag: `"{p}/{t}"` if both present, else the loyalty number.
- Price in the deck currency (`deck_metrics::card_price` + `format_amount`), omitted when no price.
- Color identity pips in WUBRG order.

**Expanded** (eases open; a rule above and below):

- `CardDetails` with `show_classification: true`, `describe_tag` (tag descriptions from the catalog), `on_examples` (opens `OracleTagExamples` for a tag slug).
- Default actions row: "Flip" (multi-faced), "Image" (has art): opens the fullscreen **ImagePreview** on the face the row was showing.
- Host actions row: "-" [qty] "+" (when `on_qty_change`), "Printing" (when `on_printing`), "Star"/"Unstar" (when `mvp` is Some and a toggle is given).
- Move row (when `on_move_to`), by `current_board` (None → Deck): Deck → "To maybeboard", "To sideboard"; Maybeboard → "To mainboard", "To sideboard"; Sideboard → "To mainboard", "To maybeboard".
- All action buttons stop propagation (do not collapse the row).

Which rows get which actions:

| Row | qty stepper | move | Printing | Star |
|---|---|---|---|---|
| Group and Lands rows | yes | yes (board from `row_meta`) | yes (deck card) | yes |
| Maybeboard / Sideboard rows | yes | yes | yes (deck card) | no |
| Commander, partner, background, signature spell | no | no | yes (profile slot) | no |
| Tokens | no | no | no | no |

### 3.6 QuickAdd (`components/quick_add.rs`)

- Row: label chip "Quick add", text input (id `quick-add`, placeholder "Search a card to add", no autocorrect), "×" clear button when non-empty (clears query, results, closes dropdown).
- Focus scrolls the screen content to the top (so the results have room above).
- Each edit takes a generation ticket. Query trimmed shorter than 2 chars → close the dropdown, stop. Else show the dropdown at once with "Searching...", wait 300 ms, bail if a newer edit happened, then search: `CardQueryBuilder::with_name_contains(q.trim())`, `set_synergy(false)`, `set_include_skipped(true)` (typed names also surface skipped and removed cards), `set_sort(CardSortKey::Name)`, `set_limit(5)`; `authed.run_quiet_at(QUICK_ADD, "quick_add_search", search_deck_cards(deck_id, &filter, &s))` (deck cards are excluded server-side). Stale replies are dropped. Reply → results; failure → close the dropdown (quiet).
- Dropdown floats above the bar: "Searching..." while searching; "No results" when empty; else up to 5 chips with card names (staggered entrance). Tap a chip:
  - Remove it from the results at once (no double add).
  - `authed.run_at(QUICK_ADD, "quick_add_card", create_deck_card(deck_id, &HttpCreateDeckCard::new(&sd, 1, None), &s))`.
  - On success: push `DeckEntry { card, deck_card }` into `deck_entries`, bump the counter, `undo_log.push(Added { card_id, card_name })`, toast info `"Added {card_name}"` (QUICK). On failure the facade toasts.

### 3.7 Quantity change (`change_quantity(card_id, delta, _is_basic_land, record)`)

Optimistic per tap, debounced server write per card.

1. `current = qty_for(card_id)` (default 1). `should_delete = current + delta < 1`. Capture land count and total price before.
2. If deleting:
   - Telemetry `record_removal(deck_id, oracle_id)` (a deliberate removal).
   - Save the entry in `removed_rows`. If `record`: push `UndoAction::Removed { entry, baseline }` where baseline = the pending burst's baseline, or `current` if none.
   - Remove the entry from `deck_entries`, bump the counter.
   - Land and budget crossing checks (below). Toast info "Card removed" (QUICK).
3. Else: set the entry's quantity to `current + delta`, bump the counter, crossing checks.
4. Fold into `pending_qty[card_id]` (`baseline` = quantity before the burst, `delta += delta`, `generation += 1`) and spawn a 300 ms timer. When it fires and the generation is still the latest, remove the burst and its `removed_rows` entry:
   - Net delta 0 → nothing.
   - `baseline + delta < 1` → `authed.try_run("change_quantity", delete_deck_card(deck_id, card_id))`. On failure: `roll_back_failed_delete` (drop the burst's `Removed` undo entry; re-add the row at `baseline` unless it is already back), bump the counter.
   - Else → `authed.try_run("change_quantity", update_deck_card(deck_id, card_id, &HttpPatchDeckCard::new(Some(baseline + delta), None)))` (absolute quantity, replay-safe). On success and `record`: push `QuantityChanged { card_id, card_name, net: delta, baseline }` (one undo entry per burst, only after the server took it). On failure: revert the quantity to `baseline`, bump the counter.
5. On screen unmount, every still-pending burst with a non-zero delta is posted with a detached task (delete or PATCH to the absolute target); failures only report and toast.

**Land crossing** (`warn_land_crossing(before, after)`, only with an explicit `land_target`): `before >= t && after < t` → toast warning `"Below land target ({target})"` (NORMAL); `before < t && after >= t` → toast info `"Land target reached ({target})"` (NORMAL). Land count = sum of quantities of active-board land entries.

**Budget crossing** (`warn_budget_crossing`): same as on Add (`"Deck at {pct:.2}% of your {amount} budget"`, warning at ≥ 100%, info otherwise, NORMAL), fires only when the tier goes up.

The quantity is not capped client-side (format copy limits are the server's and the deck warnings' business). The "-" at 1 deletes.

### 3.8 Move board (`move_to_board(card_id, target, record)`)

- Read the old board and name. If `record`: push `MovedBoard { card_id, card_name, from: old }`.
- Optimistic: set the entry's board, bump the counter.
- `authed.try_run("move_board", update_deck_card(deck_id, card_id, &HttpPatchDeckCard::new(None, Some(target.display_name()))))`; failure → set the old board back, bump.
- If `record`: toast info `"Moved to {target.display_name()}"` (QUICK), i.e. "Moved to deck", "Moved to maybeboard", "Moved to sideboard".
- No land/budget toast on moves here.

### 3.9 Printing change

Row "Printing" → `printing_sheet_card = card`, `command_zone_slot` = None for deck cards or the slot for command-zone rows, open the sheet (default `saved_message` "Printing saved"). On Save with the new printing:

- Command-zone slot: request `HttpUpdateDeckProfile::builder().<slot>_id(Patch::Set(Some(new_id))).build()` (`commander_id`, `partner_commander_id`, `background_id` or `signature_spell_id`); `authed.run("change_printing", update_deck_profile(deck_id, &request, &s))` (`update_deck_profile(&self, deck_id: Uuid, body: &HttpUpdateDeckProfile, session: &Session) -> Result<DeckProfile, ClientError>`). On success: set the slot card, push `CommandZonePrintingChanged { slot, old_card, new_id }`, `printing_sheet_card = new`, bump.
- Deck card: `authed.run("change_printing", update_deck_card(deck_id, old_id, &HttpPatchDeckCard::with_printing(&new_id.to_string())))`. On success: replace the entry's card and `deck_card.scryfall_data_id`, push `PrintingChanged { old_card, new_id }`, `printing_sheet_card = new`, bump.
- The sheet's own "Printing saved" toast fires at Save (before the server answers). Not optimistic: the row updates only after success.

### 3.10 MVP star (`toggle_mvp(card_id, currently_mvp)`)

- Optimistic: set `mvp_at = now` (star) or None (unstar); bump.
- `authed.try_run("toggle_mvp", update_deck_card(deck_id, card_id, &HttpPatchDeckCard::with_mvp(target)))`. Success → adopt the server's `mvp_at` (the vesting clock). Failure → restore the old `mvp_at`, bump. The server enforces at most 3 MVPs per deck; its 422 message reaches the user through the facade's error toast.
- Star only exists on mainboard rows (groups and lands). The featured strip updates live (MVPs ordered by `mvp_at`).
- No undo entry for star changes.

### 3.11 Undo button (`apply_undo`)

Pops the newest entry from the live log (consumed even if stale). Stale check first; a stale entry toasts info "Already changed" (QUICK) and does nothing. Undo-driven calls pass `record: false` (no new entries, no user-action toasts).

| Entry | Stale when | Effect | Toast (QUICK, info) |
|---|---|---|---|
| `Added { card_id, card_name }` | card not in `deck_entries` | remove the row optimistically, bump; `authed.run("undo", delete_deck_card)`. No removal telemetry. | `"Removed {card_name}"` |
| `Removed { entry, baseline }` | card already present | push the entry back at `baseline` qty, bump; `authed.try_run("undo", create_deck_card(HttpCreateDeckCard::new(&sd, baseline, create_board_field(board))))`; success adopts the returned `DeckCard`; failure removes the row again. | `"Re-added {card_name}"` |
| `QuantityChanged { card_id, card_name, net, baseline }` | card not present | `change_quantity(card_id, -net, false, record: false)` | `"{card_name} back to ×{baseline}"` |
| `MovedBoard { card_id, card_name, from }` | card not present | `move_to_board(card_id, from, record: false)` | `"{card_name} back to {from.display_name()}"` |
| `PrintingChanged { old_card, new_id }` | no entry with `new_id` | swap the entry's card back, bump; `authed.run("undo", update_deck_card(deck_id, new_id, with_printing(old_id)))` | `"{card_name} printing restored"` |
| `CommandZonePrintingChanged { slot, old_card, new_id }` | the slot no longer holds `new_id` | set the slot card back, bump; `authed.run("undo", update_deck_profile(<slot>_id = old_id))` | `"{card_name} printing restored"` |

The log is per deck and shared with Add, Remove and quick add (see the UndoStore section): swipes on Add/Remove made while this screen was away show up here as entries, and reversing them here makes the swipe screens' own gesture undo skip its server call.

### 3.12 Image preview (`components/image_preview.rs`)

- Opened by a row's "Image" (on the face the row shows) or a featured card tap (face 0).
- Full-screen backdrop and a large flippable image (`ImageSize::Large`, Flip button for DFCs, starting on the given face).
- Tap anywhere: play the dismiss animation for 200 ms, then close.

### 3.13 Toast inventory, deck cards view

| Trigger | Kind | Text | Duration |
|---|---|---|---|
| load with a remembered filter | warning | Filter is active | QUICK |
| qty to 0 | info | Card removed | QUICK |
| land crossing down / up | warning / info | Below land target ({target}) / Land target reached ({target}) | NORMAL |
| budget band up | info / warning | Deck at {pct:.2}% of your {amount} budget | NORMAL |
| move board | info | Moved to {deck\|maybeboard\|sideboard} | QUICK |
| quick add | info | Added {card_name} | QUICK |
| printing save | info | Printing saved | QUICK |
| leave printing sheet after moving the carousel | info (Back) / warning (backdrop, OS back) | Printing unchanged | QUICK |
| Reset filter button | info | Filter reset | QUICK |
| undo, stale | info | Already changed | QUICK |
| undo variants | info | Removed {name} / Re-added {name} / {name} back to ×{baseline} / {name} back to {board} / {name} printing restored | QUICK |
| any failed call | error | {to_user_message()} | NORMAL |

### 3.14 Animations

`row-enter` on groups and the header, collapsible group ease, row expand ease, art thumbnail ease in/out, featured cards deal-in, quick-add results stagger, image preview dismiss (200 ms).

---

## 4. Remove cards (`remove.rs`)

Route `/deck/card/remove/:deck_id` (`Router::RemoveDeckCard { deck_id }`). Reached by push from the deck cards view's "Remove" button. Leaves with "Back" (`go_back`) or OS back.

### 4.1 State

- `filter_builder` from `FilterStore::restore(Remove, deck_id)` or default; parked on unmount. Context.
- `filter_reset_counter` (context). `DeckCards(deck_cards_for_filter)` context = all entry cards (pickers draw from the deck).
- Hint `use_one_time_hint(HINT_REMOVE_DECK_CARDS)`.
- `deck_entries` (all entries, every board), `command_zone_cards`, `stack: CardStack<RemoveAction>` (screen-local, cycling), `deck_loaded`, `board_filter: BoardFilter` = `Deck` (default) | `Maybeboard` | `Sideboard` | `All`, `land_target` (explicit), `price_budget`, `price_budget_currency`, `show_details`, `printing_open`.
- Swipe config: all four directions, 90 px / 2.5 px/ms (same engine as Add).

### 4.2 Load

`authed.run("load_deck", fetch_deck(deck_id, &s))`. On success: `deck_cards_for_filter`, `land_target`, budget and currency, `command_zone_cards`, `deck_entries = deck.entries` (command-zone cards are not in entries), `deck_loaded = true`, bump the counter. On failure: the facade toasts; the screen shows the empty skeleton.

### 4.3 Filter pass (on counter or board filter change, after load)

1. Board: Deck → active entries; Maybeboard → maybeboard; Sideboard → sideboard; All → everything.
2. If the builder is non-empty: `build_criteria()` + `Cards::matching` (Err keeps all).
3. `Cards::sorted(builder.sort(), builder.direction())` (no sort keeps server order).
4. `stack.replace(filtered)` (cursor to 0; history kept).

### 4.4 Layout

1. **ScreenHeader** "Remove deck cards" ("?" opens the hint).
2. **Boards** row: label chip "Boards", single-select chips "Main", "Maybe", "Side", "All". Tap → set the filter, bump the counter.
3. Card area: if the stack is non-empty → SwipeStack (window of 10 from the cursor) + CardInfoDisplay of `current_wrapping()`; else `CardSkeleton` ("No cards"). No loading skeleton (the skeleton reads "No cards" while loading too).
4. **ActionBar**: "Back"; "Filter" (dot when `!filter.is_empty() || sort.is_some()`); "Refresh": `stack.rewind()` (cursor 0, history cleared); if the sort is Random, bump the counter to reshuffle; toast info "Stack refreshed" (QUICK); eye RulesButton when there is a current card.
5. Overlays (when there is a current card): **CardDetailsSheet** with "Printings", and a **read-only PrintingSheet** (`read_only: true`: no Save, no "Printing unchanged" toasts; browsing never changes the deck since removal keys on the exact printing id).
6. **HintDialog** "Swipe to trim":
   - "Swipe **right** removes the card" (success color)
   - "Swipe **left** keeps it" (error color)
   - "Swipe **up** moves it between main and maybeboard" (warning color)
   - "Swipe **down** undoes the last swipe" (tertiary)
   - Line: "The board chips up top pick which board you are trimming"
7. **CardFilterSheet** (`show_format_filter`, `show_active_indicators`; plain Reset; no validation).

### 4.5 Gestures

**Left (keep)**: telemetry swipe; `stack.record(Keep)`; toast info "Kept" (QUICK); `advance_wrapping()`. No server call.

**Right (remove all copies)**:

1. Telemetry swipe and `record_removal(deck_id, oracle_id)`.
2. Snapshot `(board, quantity)` from the entry (fallback `(Deck, 1)`); `stack.record(Remove { card, board, quantity })`.
3. `delete_card_from_deck()`: snapshot the entry, spawn `authed.run("remove_card", delete_deck_card(deck_id, scryfall_data_id))`; on success `undo_store.push(deck_id, Removed { entry, baseline: entry.quantity })`. No rollback on failure (the card stays gone locally; the facade toasts).
4. Toast success "Removed from deck" (QUICK).
5. Capture land count and price; `remove_current_card()` (drop the entry from `deck_entries` and `stack.remove_current()`; the next card slides in); then `warn_if_below_target` (toast warning `"Below land target ({target})"` (NORMAL) only on a downward crossing of an explicit target) and the budget crossing check (upward only, so removals never toast in practice).

**Up (toggle board)**: `from` = the entry's board; `to = Deck` if `from` is maybeboard, else `Maybeboard` (sideboard cards go to the maybeboard). `stack.record(MoveBoard { card, from, to })`; `move_card_to_board(to)`: `authed.run("move_card", update_deck_card(deck_id, id, &HttpPatchDeckCard::new(None, Some(to.display_name()))))`, on success `undo_store.push(MovedBoard { card_id, card_name, from })`. Toast info "Moved to maybeboard" or "Moved to main" (QUICK). Then `move_current_card_locally(to)` (update the entry's board, drop it from the displayed stack even in the All view), land and budget checks.

**Down**: telemetry; `undo_last_action()`:

- pop; none → toast info "Nothing to undo" (default). `prime_entering(action.exited())`.
- **Keep**: `retreat_wrapping()` (false → cancel entering); toast info "Undid keep" (QUICK).
- **Remove { card, board, quantity }**: `taken = take_newest(Removed for this card)`; `already_back = deck_entries contains it`. None and already back → cancel entering, toast info "Already changed" (QUICK). Else `stack.insert_current(card)`; spawn: refresh failure → toast error (default), cancel entering, push taken back; `client.create_deck_card(deck_id, &HttpCreateDeckCard::new(&sd, quantity, create_board_field(board)), &session)`; Ok → push `DeckEntry` back into `deck_entries`, toast success "Undid remove" (QUICK); Err → toast error `"Failed to undo: {e}"` (default), push taken back (the card stays reinserted in the stack).
- **MoveBoard { card, from, .. }**: `taken = take_newest(MovedBoard { card_id, from })`; `already_back` = an entry with this id on `from`. None and already back → cancel, "Already changed". Else reinsert, spawn: refresh failure as above; `update_deck_card(... HttpPatchDeckCard::new(None, Some(from.display_name())))`; Ok → set the entry's board to `from`, toast success "Undid move" (QUICK); Err → "Failed to undo: {e}", push taken back.

**Tap**: none (Flip on DFC images only).

### 4.6 Quantities and the sideboard on Remove

- Right removes the whole row (all copies) in one DELETE. Undo re-creates it at the snapshotted quantity on its old board.
- Sideboard: visible through "Side" or "All". Up on a sideboard card moves it to the maybeboard (not to main). There is no way to move to the sideboard here.

### 4.7 Toast inventory, Remove

| Trigger | Kind | Text | Duration |
|---|---|---|---|
| swipe left | info | Kept | QUICK |
| swipe right | success | Removed from deck | QUICK |
| land below target | warning | Below land target ({target}) | NORMAL |
| budget band up | info / warning | Deck at {pct:.2}% of your {amount} budget | NORMAL |
| swipe up | info | Moved to maybeboard / Moved to main | QUICK |
| nothing to undo | info | Nothing to undo | default |
| undo keep | info | Undid keep | QUICK |
| undo, already reversed | info | Already changed | QUICK |
| undo remove | success | Undid remove | QUICK |
| undo move | success | Undid move | QUICK |
| undo failed | error | Failed to undo: {e} | default |
| refresh | info | Stack refreshed | QUICK |

### 4.8 Animations

Same swipe engine and animations as Add (drag tilt, exits, enter-on-undo from the exit side), `content-enter` on the content.

---

## TUI today

Read on 2026-10-30 from `zwiper-tui/src`. The TUI was being restructured while I read it: `inbound/ui/*` is gone, `inbound/screens/{deck,swipe,...}` were written minutes earlier, `screens/swipe/` held only `draw.rs` (no `mod.rs`/`keys.rs` yet), and `inbound/event_loop/mod.rs` still carried the old swipe key handling and requests. Re-check file locations before building; the behavior below is what the code does.

### What exists

- **Swipe screen** (`domain/swipe/mod.rs`, `inbound/screens/swipe/draw.rs`, swipe handling in `inbound/event_loop/mod.rs`). Opened from the deck screen with `s` (`screens/deck/keys.rs`; the deck view hands its loaded `Deck` over and reloads when the swipe screen closes).
- **Default query** `swipe::query_for(&Deck)`: `synergy = commander_id.is_some()`; legality `[format.to_legality_key()]` when a format; `color_identity_within` = combined identity of `command_zone_cards` minus the signature spell, when the format checks identity and there is a commander; `card_type_excludes_any = [Land]` once the land count (active board, quantity-aware) reaches `land_target` or the format default; `sort = EdhrecRank` when the deck has neither commander nor format; `is_token(false)`, `limit(25)`, `offset(0)`. Same defaults as the app's fresh Search filter, except the last rule (see gaps).
- **Stack** (`swipe::Stack`): `cards`, `index`, `history: Vec<Swipe>`, `in_deck` (oracle ids from entries + command zone, plus every card added here), `held` (printing ids), `exhausted`, `related_start`. `push_page` dedupes by printing id and in-deck oracle id and marks `exhausted` when a page adds nothing. `needs_more()` = not exhausted and `remaining() <= 5`. Every page uses offset 0. This matches the app's pagination and dedup.
- **Keys** (`handle_swipe_key`): `l` / Right = add (Swipe::Add, mainboard, qty 1), `h` / Left = pass (skip), `m` / Up = maybe (maybeboard, qty 1), `u` / Down / Backspace = undo, `i` = toggle image vs text when the terminal is too narrow for both, `Esc` / `q` = back to the deck screen. Action bar: `h pass`, `l add`, `m maybe`, `u undo`, `i image/text`, `Esc deck`.
- **Server effects** (`send`): Add → `create_deck_card` with `HttpCreateDeckCard { scryfall_data_id, oracle_id, quantity: 1, board: None | Some("maybeboard") }`; Skip → `skip_deck_card`; Unskip → `unskip_deck_card`; Remove (undo of add or maybe) → `delete_deck_card`. Failures show `"That swipe did not save: {message}"` in the status line; `Unauthorized` ends the session ("Your session ended. Sign in again.").
- **Undo** pops history, steps the cursor back, removes the oracle id from `in_deck` for add/maybe, and sends the reverse effect.
- **related_from**: remembers the printing id at `related_from` from the first page that has one; while that card is on screen the status line reads "That's every synergy pick. Showing cards like your deck." (with a trailing period), cleared on the next key.
- **Drawing**: header line = deck name (accent, bold), commander name (muted), card count incl. adds this session; body = card text (each face: name + colored mana symbols, italic type line, rules paragraphs, right-aligned stats; faces separated by "//"; then set name) in a bordered box max 64 cells wide, with the image to the left when there is room (`ratatui_image`, `ImageSize::Normal`), else `i` swaps between them; image placeholders "Loading image..." and "No image"; footer "{n} left" or "{n} left, finding more..."; a status line; the key bar.
- **Placeholders**: error text if any; "Loading the deck..." (no stack yet); "Finding cards..." (fetching, no card); "No more cards match this deck." (exhausted).
- **Images**: one behind and three ahead are fetched and kept (`IMAGES_BEHIND = 1`, `IMAGES_AHEAD = 3`).

### Gaps and differences, Add screen

1. **Screen title**: the app's header is "Add deck cards" with a "?" hint; the TUI shows deck name, commander and count. No "Swipe to build" hint.
2. **Blank-query gate**: the app refuses to serve when there is no search intent (non-commander format, or commander format without a commander, with no user filter, sort or synergy) and toasts "Try a filter to start swiping". The TUI serves anyway (format legality alone, or EDHREC order when neither commander nor format). Decide whether to copy the app here.
3. **No source selector** ("From" Search / Maybeboard) and **no maybeboard swiping** (promote right, skip left, "Already on maybeboard" up, its own undo).
4. **No Synergy chip**: the TUI cannot toggle synergy. `synergy_applied` is ignored, so no "Synergy warming up; showing all cards for now" toast and no warming suffix on empty results.
5. **No filter sheet** at all (section 2), no filter dot, no Reset with deck-context defaults, no Refresh ("Stack refreshed" / "Filter is empty").
6. **No per-deck stack parking or filter memory**: leaving the swipe screen drops the stack; the app's `AddStackCache` (20 decks, 50 behind) and `FilterStore` are missing.
7. **Undo while an add is in flight**: the TUI sends the delete immediately; if it beats the create, the delete fails and the add stands. The app parks the undo (`InFlightAdds`) and deletes once the POST lands.
8. **No global undo log**: TUI adds do not record `UndoAction::Added`; there is no reconciliation with a deck cards Undo (that screen does not exist yet either).
9. **Undo feedback**: the app toasts "Undid add", "Undid skip", "Undid maybeboard", "Nothing to undo", "No previous card", "Failed to undo: {e}", and on a refresh failure restores the cursor; the TUI shows nothing on success and only a generic failure line. On a failed undo-delete the app keeps the cursor where it is; the TUI also leaves it rewound.
10. **Swipe feedback**: no "Added to deck", "Skipped", "Added to maybeboard".
11. **Land and budget signals**: no running land count or "Land target reached ({target})", no running price total or "Deck at {pct:.2}% of your {amount} budget". The TUI header count does add swipes.
12. **Stack cap**: no 500-card cap ("Card stack limit reached, please refresh your search to continue") and no "Approaching the card stack limit..." every 100 from 400.
13. **End of results**: app = toast warning "End of results" + "No cards" skeleton that accepts a down-pull (60 px) undo; TUI = "No more cards match this deck." and undo via `u`/Down (equivalent behavior, different text, no toast).
14. **Empty first page**: app toasts "No results for this filter" (+ synergy suffix) and records a pile-end; TUI shows "No more cards match this deck." (exhausted on an empty first page).
15. **related_from**: the app tracks every related printing id across all pages and announces once per fresh stack when the first one reaches the top (toast, NORMAL, no trailing period), plus telemetry. The TUI only checks the first page that carried `related_from`, and only that exact card; text has a trailing period.
16. **Card info**: the app shows image + "Prices: $x | €y | z TIX", "Set: ...", "Released: ...", "Artist: ..." under it, and keeps rules text in the eye details sheet (with keywords, card roles, oracle tags, Flip, Printings). The TUI shows rules text inline and only the set name; no prices, release date, artist, keywords, roles or tags.
17. **DFC**: the app shows one face at a time with a Flip control; the TUI prints all faces stacked with "//" and shows only the primary image.
18. **Printing sheet**: missing (browse printings via `fetch_printings(oracle_id)`, re-skin the top card, "Printing swapped").
19. **Skip error handling**: the app posts skips quietly (no error toast); the TUI reports a failed skip in the status line.
20. **Errors loading cards**: the app toasts via the facade and keeps the screen; the TUI replaces the card area with the error text.
21. **Telemetry**: none in the TUI (`record_swipe`, `record_signal`, `record_search`, `record_pile_end`, `record_removal`, flush before each fresh search). Synergy ordering on the server feeds on these signals, so TUI swipes do not train it.
22. **Animations**: none (expected for a terminal; the enter-from-exit-side cue on undo could be a brief highlight).
23. **Image window**: TUI fetches 1 behind and 3 ahead; the app renders a 10-card window and relies on its image cache.

### Gaps, deck cards view, filter sheet and Remove

- **Deck cards view**: not present. The TUI deck screen shows the profile and stat tabs only. Missing: header chips, FeaturedCards, QuickAdd, Group by, Boards, Show, all sections and rows, row expansion with details, qty stepper with debounced PATCH, move board, printing change (deck card and command-zone slot), MVP star, Undo button over the global log, Reset filter, image preview, oracle tag examples.
- **Card filter sheet**: not present (see section 2 for the full contract).
- **Remove screen**: not present.

---

## Shared pieces the TUI needs

Domain (pure, unit-testable, ideally lifted from zwiper rather than rewritten; most only depend on zwipe-core):

1. **`CardStack<A>`** with both disciplines: linear (Add search) and cycling (Add maybeboard, Remove), incl. `step_back`, `retreat_wrapping`, `insert_current`, `remove_current`, `replace_current`, `unwind_undo`, `rewind`, `restore`. The current `swipe::Stack` covers only the linear case and keeps oracle bookkeeping inside; generalize or wrap it.
2. **Action models**: `AddAction`, `MaybeboardAction`, `RemoveAction` with `exited()`.
3. **`InFlightAdds`** (copy as is; it has tests).
4. **`UndoAction` + `UndoStore`/`UndoLog`** (per-deck, cap 100, `take_newest`, `create_board_field`) and the reconciliation rule. Needed by Add, Remove, quick add and the view's Undo; `roll_back_failed_delete` from `view.rs`.
5. **`FilterStore`** keyed by `(FilterScope, deck_id)` with scopes `Cards`, `Remove`, `AddSearch`, `AddMaybeboard`.
6. **`AddStackCache`** (20 decks MRU, 50 cards behind, history trimmed, invalidated by filter change).
7. **Add-screen filter defaults**: `ensure_lands_excluded`, `lands_at_target`, the fresh Search default, `clear_filters` (keeps synergy, re-applies deck context), the serve gate (`has_search_intent`, `has_commander`), `related_ids_of(page)`.
8. **Apply decision**: `apply_action(validate, from, to, unchanged)` and `FilterState` (copy as is; it has tests), the open-snapshot / restore-on-close rule, contradiction message.
9. **Filter section models**: read/write helpers per section (Any/All moves, Include/Exclude moves, Exact/Range moves, tri-toggle), numeric parse-on-commit for mana value and price, power/toughness steppers, picker search (substring, caps 8/5, exclusion of picked values), `deck_cards::extract_*` for view/Remove, `oracle_tag::search_oracle_tags` for tags.
10. **Catalog cache** for `fetch_keywords`, `fetch_oracle_words`, `fetch_card_types`, `fetch_artists`, `fetch_sets`, `fetch_card_roles`, `fetch_oracle_tags` (unauthenticated, fetched once, retried on next open after a failure, "Filter lists didn't load, trying again"), and `fetch_keyword_reminders` for keyword chips.
11. **Deck metrics hooks**: running land count, `land_target` crossings (up on Add; both ways on the view; down on Remove), budget tier crossings via `deck_metrics::{card_price, deck_price, budget_tier}` and `PriceCurrency::format_amount`.
12. **Quantity burst debounce** (`PendingQty`, 300 ms, absolute PATCH, delete under 1, flush on leave, rollback rules).
13. **View pipeline**: active-only filter + sort (Name default) + land split + `group_by(GroupByOption)`, pinned command-zone filtering by `criteria.matches`, maybe/side sections via `cards::sort_deck_entries`, `row_meta`, featured strip ordering (zone roles then MVPs by `mvp_at`).
14. **Quick add search**: `with_name_contains`, synergy off, `include_skipped(true)`, sort Name, limit 5, min 2 chars, 300 ms debounce, generation tickets.

Client calls (all already in `zwipe-client`): `fetch_deck`, `fetch_deck_profile`, `fetch_card`, `fetch_deck_tokens`, `search_deck_cards`, `create_deck_card`, `update_deck_card` (`HttpPatchDeckCard::new(qty, board)`, `with_printing`, `with_mvp`), `delete_deck_card`, `skip_deck_card`, `unskip_deck_card`, `update_deck_profile` (`HttpUpdateDeckProfile::builder()` + `Patch::Set`), `fetch_printings`, the catalog fetches above, `mark_hint_shown` (one-time hints `HINT_ADD_DECK_CARDS`, `HINT_REMOVE_DECK_CARDS`, `HINT_DECK_CARDS`, `HINT_FILTER`). Telemetry needs the usage buffer endpoints if the TUI should feed synergy signals.

UI pieces (terminal equivalents):

- A **toast queue** with the four kinds and QUICK/NORMAL/LONG durations, replacing the single status line, since all four screens lean on toasts for every outcome.
- A **bottom-sheet / modal** primitive with backdrop-dismiss semantics (filter sheet, details sheet, printing sheet, oracle tag dictionary, hint dialogs, image preview).
- An **accordion** (single open) with per-item clear "×" and active indicators.
- **Chip rows** (single and multi select, disabled state), **mode chips**, **tri-toggles**, **steppers**, **text inputs** with clear, **searchable picker** with a floating result list.
- A **card info block** (name, prices line, set, released, artist) and a **card details body** (cost, type/rarity/set chips, oracle text, stats, keywords with reminders, card roles with tags and descriptions), with face flipping.
- A **carousel** for printings (prev/next keys, dots as "3/12").
- **Expandable card rows** with compact line (qty, ★, name, P/T or loyalty, price in deck currency, color pips, optional art), and the action rows (− qty +, Printing, Star/Unstar, To mainboard/maybeboard/sideboard, Flip, Image).
- **Collapsible groups** with counts and color pips.
- A **one-time hint** mechanism tied to `user.has_seen_hint` and `mark_hint_shown`.
- Key mapping for gestures: Right/`l` = right swipe, Left/`h` = left, Up/`m` = up, Down/`u` = undo (already in place), plus keys for Filter, Refresh, details (eye), Printings, source switch and Synergy on Add, board chips on Remove.
