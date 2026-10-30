# Spec 3: Decks (list, create, edit, import, export, deck view)

Source of truth: the ZWIPE app in `zwiper/src/lib/inbound/` (worktree `the repo root`). Every quoted string below is copied verbatim from the app. File paths are relative to that worktree unless absolute.

Key app files:

- `zwiper/src/lib/inbound/router.rs` (routes)
- `zwiper/src/lib/inbound/screens/deck/list.rs`, `create.rs`, `edit.rs`, `import.rs`, `export.rs`, `view.rs`
- `zwiper/src/lib/inbound/screens/deck/components/` (`deck_fields.rs`, `format_select.rs`, `tag_select.rs`, `oracle_tag_select.rs`, `swipe_select.rs`, `more_buttons.rs`, `clone_deck_dialog.rs`, `collapsible_section.rs`, `deck_profile.rs`, `deck_budget_section.rs`, `deck_tags_section.rs`, `deck_warnings.rs`, `deck_charts.rs`, `skeletons.rs`)
- `components/src/charts.rs`, `components/src/role_tags.rs` (shared charts)
- `zwiper/src/lib/outbound/buy_links.rs` (retailer URLs)
- `zwiper/src/lib/inbound/components/auth/authed.rs` (error toasting), `hint_dialog.rs`, `hint_host.rs`, `concept_explainers.rs`, `screen_header.rs`
- `the orangui repo/src` (`alert_dialog.rs`, `bottom_sheet.rs`, `chip.rs`, `toast_timing.rs`)

---

## 0. Conventions used across every deck screen

### 0.1 Screen chrome

- **ScreenHeader**: centered title; a "!" support button at the top left (always); a "?" at the top right when the screen has a hint dialog. "?" reopens that screen's hint.
- **ActionBar**: a row of equal action buttons pinned to the bottom.
- **BottomSheet**: slides up from the bottom with a dimmed backdrop, a title, the content, and by default a single "Close" footer button. Backdrop tap or OS back closes it.
- **AlertDialog**: modal with a title, a body, and action buttons. Action kinds (orangui `DialogAction`):
  - `Dismiss(label)`: closes, does nothing.
  - `Confirm { label, danger, on_confirm }`: closes, then runs the action. `danger` draws it in the error color.
  - `Link { label, href }`: opens the URL in the system browser and closes.
  - `Button(ActionButton)`: can be disabled from state and can run async work under a busy label. Busy work returns `true` to close the dialog and `false` to keep it open. While busy, every other action in the dialog is disabled.
- **HintDialog**: the one-time explainer. `use_one_time_hint(KEY)` opens it on first mount if `session.user.has_seen_hint(KEY)` is false, then calls `ZwipeClient::mark_hint_shown(hint: &str, session: &Session) -> Result<User, ClientError>` fire-and-forget and saves the returned user into the session. A failed record only means it may show once more later. All hint dialogs have a single dismiss button.
- **InfoButton** (a small "?" next to a field label): opens a shared AlertDialog titled `HintTopic::title()` with one action "Got it". Texts are listed where each button appears.

### 0.2 Toasts

orangui durations: `TOAST_QUICK` = 3 s, `TOAST_NORMAL` = 5 s, `TOAST_LONG` = 8 s. Kinds: info, success, warning, error. If a duration is not stated it is the library default.

### 0.3 Errors from API calls (the `Authed` facade)

Most calls go through `use_authed(screen)`:

- It first refreshes the session (`ensure_fresh`). If that fails it reports and returns the error.
- On any `ClientError` it logs, reports telemetry, and (unless the call is a `run_quiet*`) shows an **error toast** with `ClientError::to_user_message()` for `TOAST_NORMAL`.
- `run` / `run_at` return `Option<T>` (None on error). `try_run` returns the `Result`.

`ClientError::to_user_message()` (zwipe-client `error.rs`):

- `Network` → "connection error, check your network and try again"
- `BadRequest`, `Decode`, `InternalServerError` → "something went wrong, please try again"
- `PayloadTooLarge` → "request too large, try importing fewer cards"
- everything else → the server's own message verbatim (server 4xx messages are user-safe by contract).

The TUI already has the equivalent: `failure()` in `zwipe-tui/src/lib/inbound/event_loop/mod.rs` maps `Unauthorized` to sign-out and otherwise uses `to_user_message()`.

### 0.4 Animations used on these screens (CSS)

- `.content-enter`: block fades up 0.625rem over 0.3 s ease-out on mount. `.content-enter-delayed`: same, 0.15 s later. Disabled with reduced motion.
- `.row-enter`: row fades in from 2 px above, 0.25 s ease-out.
- `.collapsible` / `.collapsible.open`: height (grid rows 0fr→1fr) and opacity ease 0.25 s. Content stays mounted.
- Disclosure arrow "▸" rotates 90° when open (0.2 s ease).
- `.search-float-chip`: search result chips float up 8 px over 0.25 s, staggered 40 ms per chip (`animation-delay: {i*40}ms, {250+i*40}ms`), then a slow 3.2 s idle drift of 2 px.
- `.swipe-select-screen`: full-screen picker overlays fade in/out over 0.2 s opacity.
- Skeletons: `skeleton-bar` placeholders pulse. Per the project rule, skeletons contain no real text, chips or controls.

A terminal can drop all of these or approximate with an instant swap. Nothing in the logic depends on them.

---

## 1. Deck list (`Router::DeckList`, `/deck`, `list.rs`)

### 1.1 Data and calls on mount

1. `ZwipeClient::fetch_user(&self, session: &Session) -> Result<User, ClientError>`: refreshes the user so `email_verified_at` is current. On success replaces `session.user`. On failure only logs (no toast).
2. `ZwipeClient::fetch_deck_profiles(&self, session: &Session) -> Result<Vec<DeckProfile>, ClientError>` via `authed.try_run("load_decks", ...)` (error toast on failure). Restarted on every mount, so the list is always fresh.
3. Hint `HINT_DECK_LIST` ("deck_list"): opened via `open_and_record_hint` only once decks have loaded and the list is non-empty (it teaches the chip rows, which an empty list hides). Fires once per visit at most.

`DeckProfile` (zwipe-core `domain/deck/models/deck_profile.rs`) fields used: `id, name: DeckName, commander_id, partner_commander_id, background_id, signature_spell_id, format: Option<Format>, tags: Vec<String> (slugs), power_level: Option<PowerLevel>, other_tags: Vec<DeckOtherTag>, oracle_tags: Vec<String>, land_target: Option<i32>, price_target: Option<f64>, price_target_currency: Option<PriceCurrency>, share_token: Option<Uuid>, user_id, card_count: i64, commander_name, partner_commander_name, background_name, signature_spell_name: Option<String>, color_identity: Vec<String> (WUBRG letters), commander_art_url, partner_commander_art_url, background_art_url, signature_spell_art_url: Option<String>`.

### 1.2 Layout top to bottom

1. **ScreenHeader** title "Decks", with "?".
2. **HintDialog** title "Your decks", bullets:
   - "Tap a deck to open it"
   - `[Group by]` " folds the list into sections, " `[Show]` " narrows it by color or tag" (bracketed words are HintKey highlights)
   - `[Create]` " starts a new deck"
3. Content column (max width 40rem, 1rem side padding):
   - **Group by row** (chip row): a label chip "Group by" (stat-chip-p1 style), then four single-select chips "None", "Format", "Color", "Tag". Default "None". Ephemeral per visit.
   - **Show row**: label chip "Show"; chip "All"; five color chips showing mana pips W, U, B, R, G (always all five, even when no deck has that color); then, only if some deck has any tag, a "Tags" fold chip with a "▸" arrow (rotates when open). While Tags is folded, each **selected** tag also sits in this row as a selected chip so it stays visible; tapping it deselects it.
   - **Tag row** (collapsible, under the Show row, only when some deck has a tag): every distinct tag as a toggle chip.
   - The list (groups of rows), or a state message.
4. **ActionBar**: "Back", "Create", "More".
5. **More sheet** (BottomSheet, title "More actions").

The two chip rows are hidden while loading (skeletons instead), on error, and when the loaded list is empty.

### 1.3 Show filter (`matches_filters`)

- "All" is selected when no color and no tag is selected. Tapping "All" clears both sets.
- Color chips toggle membership in `selected_colors`. A deck passes when **every** selected color is in its identity (AND, "build toward a combo").
- Tag chips toggle membership in `selected_tags`. If any tags are selected, a deck passes when it carries **at least one** of them (OR).
- Tag namespace (`tag_labels`): deck tags keyed by raw slug and labeled `deck::deck_tag_label(slug)`; other tags keyed and labeled by `DeckOtherTag::display_name()`. Oracle tags are not part of the list's tags. The tag row lists distinct keys across all decks, sorted by label.
- Identity colors: `profile.color_identity` strings parsed with `Color::try_from`, kept in WUBRG order (`Color::all()`), unknowns dropped.

After filtering, decks sort by `name.to_lowercase()`.

### 1.4 Grouping (`group_decks`)

Every mode renders as collapsible groups; "None" is one group headed "All".

- **None**: one group, header "All".
- **Format**: header `format.display_name()`, or "No format". Groups sorted alphabetically with "No format" last.
- **Color**: one group per exact identity. Header shows the identity's mana pips (no text) and "({count})". The colorless group's header text is "Colorless". Sort: colorless last, then fewer colors first, then WUBRG order of the pips.
- **Tag**: one group per tag label (deck tags plus other tags); a deck with several tags appears under each. Groups sorted by label. Decks with no tags go into "Untagged", appended last.

Group header: "▸" arrow, then either pips + "({count})" (color mode) or "{header} ({count})". Tapping the header toggles that group's collapsed state. Collapse key: the header text, or for pip groups the concatenated short color names (e.g. "WU"). Collapsed state is ephemeral per visit. Collapsed groups keep rows mounted and hide them via the collapsible animation.

### 1.5 Deck row (`DeckRow`)

Tapping a row → `Router::ViewDeck { deck_id }`. Row contents, inline and wrapping:

1. Command-zone art crops, in order commander, partner, background, signature spell (`*_art_url`, only those present; realistically one or two). Small images, lazy loaded.
2. Deck name (light, wide tracking).
3. Identity pips (WUBRG order), when non-empty.
4. Count chip "{count} cards". Count = `card_count` + 1 for a commander (only when the format `has_commander()` and `commander_id` is set) + 1 each for partner, background, signature spell. The chip uses the warning style when the format's `min_cards()` is set and count is below it, or `max_cards()` is set and count is above it. Note the app always writes "cards", even for 1.
5. Format chip `format.display_name()` (accent p3 style).
6. Power level chip `power_level.display_name()` (p5 style), e.g. "Upgraded (3)".
7. Command-zone name chips (p3 style): commander_name, partner_commander_name, background_name, signature_spell_name, each when present.
8. Deck tag chips `deck_tag_label(slug)` (muted style).
9. Other tag chips `display_name()` (p5 style).

Art crops in a terminal: there is no room for real art in a row. Options, in order of preference: skip them; or show a one-cell color swatch per command-zone card (from the identity); or, when the terminal supports images (the TUI already has `ratatui-image` and `App.images: Option<Picker>`), show the crop only in a detail pane for the highlighted deck, never per row.

### 1.6 States

- **Loading** (resource not resolved): two `ChipRowSkeleton`s (Group by: label md, chips sm md sm sm; Show: label sm, chips sm×6 + md), then `DeckListSkeleton`.
- **Error**: chip rows hidden; message "Could not load decks". (The error toast from §0.3 also fires.)
- **Empty** (loaded, zero decks): message "No decks".
- **Filter matches nothing**: message "No decks match" (the chip rows stay).

### 1.7 Action bar

- "Back" → `Router::Home {}`.
- "Create": proactive limit guard. If `session.user.email_verified_at` is `None` **and** the loaded list is non-empty → warning toast "Verify your email to create more than 1 deck" (`TOAST_LONG`), stay. Otherwise → `Router::CreateDeck`. (The server enforces it too: "deck limit reached, verify your email to unlock more"; verified users have a cap reported as "deck limit reached". The list file notes a 20-deck cap.)
- "More" → opens the More sheet.

### 1.8 More sheet (title "More actions")

- "Commander maybeboard": closes the sheet → `Router::CommanderMaybeboard {}`.
- "Playstyle": closes the sheet → `Router::SuggestedCommanders {}`.
- Default "Close" footer.

(Those two screens are outside this spec.)

---

## 2. Create deck (`Router::CreateDeck`, `/deck/create`, `create.rs`) and Edit deck (`Router::EditDeck { deck_id }`, `/deck/update/:deck_id`, `edit.rs`)

Both render the shared form `DeckFields` (`deck_fields.rs`) plus overlay pickers. Differences are listed at the end.

### 2.1 Calls and context

- Deck tag catalog: `ZwipeClient::fetch_deck_tags(&self, session: &Session) -> Result<Vec<DeckTagView>, ClientError>` (app-wide cache, `CatalogCache::ensure_deck_tags`). `DeckTagView { slug, display_name, description, seed_otags: Vec<String> }`. Empty until loaded.
- Oracle tag catalog (for the oracle tag picker): `ZwipeClient::fetch_oracle_tags(&self) -> Result<Vec<OracleTag>, ClientError>` (no session; ~4,500 tags; `OracleTag { slug, description: Option<String>, curated: bool, ... }`).
- Command-zone typeahead: `ZwipeClient::search_cards(&self, card_filter: &CardQuery, session: &Session) -> Result<Vec<Card>, ClientError>`.
- Swipe picker: `ZwipeClient::search_commanders(&self, card_filter: &CardQuery, session: &Session) -> Result<Vec<Card>, ClientError>`.
- Up-swipe save in the swipe picker: `add_commander_maybeboard_card(oracle_id: Uuid, session) -> Result<(), ClientError>`, undo via `remove_commander_maybeboard_card(oracle_id, session)`.
- Create submit: `create_deck_profile(&self, request: &HttpCreateDeckProfile, session: &Session) -> Result<DeckProfile, ClientError>`.
- Edit load: `fetch_deck(&self, deck_id: Uuid, session: &Session) -> Result<Deck, ClientError>`, then `fetch_card(&self, scryfall_data_id: Uuid) -> Result<Card, ClientError>` (no session) once each for commander, partner, background, signature spell ids that are set.
- Edit submit: `update_deck_profile(&self, deck_id: Uuid, body: &HttpUpdateDeckProfile, session: &Session) -> Result<DeckProfile, ClientError>`.

### 2.2 Screen chrome

- Create: ScreenHeader "Create deck" with "?"; hint key `HINT_CREATE_DECK` ("create_deck").
- Edit: ScreenHeader "Edit deck" with "?"; hint key `HINT_EDIT_DECK` ("edit_deck").
- Both use `DeckFieldsHint`: HintDialog title "Building a deck", bullets:
  - "Profile" " is what the deck is, " "Budget" " what it may cost, " "Tags" " what it does" (the three section words are colored accent-primary, accent-secondary, accent-tertiary)
  - "Tags shape which cards we suggest"
  - "Tap a field's " "?" " for what it does"

### 2.3 Form, top to bottom (`DeckFields`)

All text inputs have autocapitalize none, autocorrect off, spellcheck off. "×" below is the clear button (`\u{00d7}`).

**Section heading "Profile"**

1. **Deck name** (TextInput). Label "Deck name", InfoButton topic DeckName, placeholder "Not set". Error text shown under the field.
   - InfoButton "Deck name": "Shown in your deck list. Change it any time"
   - Inline validation: once the field has had any non-empty value ("touched"), every change runs `DeckName::new(value)` and sets or clears the error. Messages (`InvalidDeckname`):
     - empty or whitespace: "deck name minimum length is 1 character"
     - more than 64 chars after trim: "deck name maximum length is 64 characters"
     - profanity (`ContainsBadWord`): "no naughty bad words please"
2. **Format**. Label row: "Format", InfoButton Format, and "×" when a format is set. "×" clears format, commander, signature spell (and their display text). Below: a tappable input-styled box showing `format.display_name()`, or placeholder "Not set". Tap opens the Format picker (§2.5).
   - InfoButton "Format" bullets: "Sets deck size, card pool and the command zone"; "Changing it clears the commander and signature spell"
3. **Commander / Oathbreaker** (collapsible; shown when `format.has_commander()`). Label is "Oathbreaker" when `format.has_signature_spell()` (Oathbreaker), else "Commander". Details in §2.4.
   - InfoButton "Commander" bullets: "Search only shows cards that can lead this format. " `[Filter]` " searches any card"; `[Swipe]` " picks one by swiping"
4. **Partner** (collapsible; shown when commander is shown and `commander_eligibility::partner_kind(&commander).is_some()`). Label "Partner". InfoButton: "Shows when the commander allows a partner"
5. **Background** (collapsible; shown when commander is shown and `commander_eligibility::has_choose_a_background(&commander)`, i.e. oracle text contains "choose a background"). Label "Background". InfoButton: "Shows when the commander can take a background"
6. **Signature spell** (collapsible; shown when `format.has_signature_spell()`). Label "Signature spell". InfoButton: "Oathbreaker only: the instant or sorcery that starts in the command zone"
7. **Power level**. Label row "Power level", InfoButton, "×" when set (clears). A wrapped, centered chip row of `PowerLevel::all()` with `display_name()`: "Exhibition (1)", "Core (2)", "Upgraded (3)", "Optimized (4)", "cEDH (5)". Single select; tapping the selected chip deselects it.
   - InfoButton "Power level": "The Commander bracket the deck aims for, 1 to 5"

**Section heading "Budget"**

8. **Land target**. Label row "Land target", InfoButton, "×" when set (back to None). A stepper: "-" button, value, "+" button. Value shows the number or "Not set".
   - From "Not set", either button sets it to the format's `default_land_target()` (37 for 100-card formats, 17 for 60-card formats) or 0 with no format.
   - "-" saturates at 0 (`saturating_sub(1)`); "+" caps at 100 (`MAX_LAND_TARGET`).
   - InfoButton "Land target" bullets: "How many lands the deck should run. Fewer warns on the deck"; "Not set uses the format's usual count"
9. **Price target**. Label row "Price target", InfoButton, "×" when the text is non-empty (clears to ""). A centered currency chip row "USD", "EUR", "TIX" (`PriceCurrency::all()`, `label()`), single select, default USD. Then a text input (decimal keypad), placeholder "Not set". Input filter: only ASCII digits and "." are kept (multiple dots are not prevented by the filter; they fail to parse and count as no budget).
   - InfoButton "Price target": "A budget for the mainboard. Going over warns on the deck"

**Section heading "Tags"**

10. **Deck tags**. Label row "Deck tags", InfoButton DeckTags, count "{n}/5" (`MAX_DECK_TAGS` = 5), "×" when non-empty (clears all). Body: placeholder box "Not set" when empty, else a chip box of selected chips each labeled `deck_tag_label(slug)` with its own "×" that removes only that tag. Tapping the box (not a chip's "×") opens the Deck tags picker (§2.6).
    - InfoButton "Deck tags" bullets: "The " "archetypes" " the deck is built around, like Aristocrats or Voltron"; "Picking one pre-selects the " "oracle tags" " that define it"; "Up to " "5"
11. **Oracle tags**. Label row "Oracle tags", InfoButton, count "{n}/30" (`MAX_DECK_ORACLE_TAGS` = 30), "×" when non-empty. Body: "Not set" box or chip box of slugs, each with "×". Tap opens the Oracle tags picker (§2.7).
    - InfoButton "Oracle tags" bullets: "The " "specific" " things the deck does, like spot removal, ramp or reanimation"; "They " "shape which cards we suggest"; "Your " "deck tags" " pre-pick a starter set from about 4,500. Fine to leave as is"
12. **Other tags**. Label row "Other tags", InfoButton, "×" when non-empty. A wrapped, centered chip row of `DeckOtherTag::all()`: "Budget", "Jank", "Meme", "Precon", "Upgraded Precon". Multi-select toggle. (Server cap `MAX_DECK_OTHER_TAGS` = 5, which is every option.)
    - InfoButton "Other tags": "Labels that do not change suggestions, like Budget or Jank"

### 2.4 Command-zone search fields (commander, partner, background, signature spell)

Each field has the same shape:

- Label row: label, InfoButton, a small toggle chip "Filter" (on by default, highlighted when on), "×" when a card is chosen, and a small primary chip "Swipe" that opens the swipe picker (§2.8) for that slot.
- A floating results layer above the input (when the dropdown is shown):
  - while searching: one chip "Searching..."
  - no hits: one chip "No results"
  - hits: one chip per card name; tapping it chooses the card, sets the input to the card name, hides the dropdown.
- Input row: text input placeholder "Not set" showing the chosen card's name; a "×" button when the search query is non-empty (clears query and input text, hides dropdown; does not clear an already chosen card).
- Tapping (clicking) the input clears the query, the input text, **and the chosen card**.
- Typing sets the query and the input text.

Search rules (all debounced 800 ms; a generation ticket drops replies for stale queries; each search records a "search" telemetry event):

| Field | Min chars | Query (zwipe-core `CardQueryBuilder`) | Limit |
|---|---|---|---|
| Commander | 3 | `with_name_contains(q)`; if Filter on and a format is set, `set_is_commander_in_format(format)` | 5 |
| Partner | 1 | `new()` + `set_name_contains(q)`; if Filter on, `set_is_partner(true)` | 10 |
| Background | 1 | `new()` + `set_name_contains(q)`; if Filter on, `set_is_background(true)` | 10 |
| Signature spell | 3 | `with_name_contains(q)`; if Filter on, `set_is_signature_spell(true)` and, if a commander is set, `set_color_identity_within(commander.color_identity)` | 5 |

Below the minimum the dropdown hides. At or above it the dropdown shows "Searching..." immediately during the debounce. On error the dropdown hides (logged only, no toast).

Cascades:

- **Format change** (any): resets all four Filter toggles to on.
- **Format picked in the picker** (`on_select`): sets the format, clears commander (and its text), and clears signature spell unless the new format has one.
- **Format cleared** (picker re-tap or "×"): clears format, commander, signature spell.
- **Commander changed** from one card to another, or from a card to none: clears partner and background (card, text, query, dropdown). The None→Some transition (first load on edit) does **not** clear, so the edit screen's racing loads keep the partner.
- **Named partner autofill** (`autofill_named_partner`), run only on an explicit commander pick (typeahead tap or swipe right), never on load: if `commander_eligibility::partner_kind(card)` is `PartnerKind::Named(mate)`, search `with_name_contains(mate)` limit 5, pick the exact name match (case-insensitive), else the card whose front face (`name.split(" // ")[0]`) matches. If the commander is still the same card and the partner slot is still empty, set the partner and show info toast "Partner found and selected: {name}" (`TOAST_NORMAL`). Failures are logged only. Generic Partner, Friends forever and Doctor's companion are left alone.

### 2.5 Format picker (`FormatSelect`, full-screen overlay)

- ScreenHeader "Format" with "?".
- Controls: label "Search"; input placeholder "Search formats" (filters `Format::all()` by case-insensitive substring of `display_name()`).
- Definition bar: before any tap, name "Hint" and text "Tap a format to see its details here.". After tapping a format, name `display_name()` and a list:
  - "Pool: {card_pool()}"
  - "Cards: {size_text}" where size_text is "{n} cards" (min = max), "{a} to {b} cards", "{a}+ cards", or "Any size", then ", singleton" (copy_max 1) or ", up to 4 copies"
  - "Life: {starting_life()}"
  - "Command zone: {text}" only for commander or signature-spell formats. Text: "Planeswalker commander + signature spell" (Oathbreaker), "1 uncommon creature commander" (Pauper Commander), "1 legendary commander (partners & backgrounds OK)" (Commander, Duel, PreDH), "1 legendary commander" (other commander formats)
  - "Commander damage: {n}" when `commander_damage()` is Some (21 for Commander, Duel, PreDH; 16 for Pauper Commander)
- Grid of format chips in `Format::all()` order (commander formats first: Brawl, Commander, Duel Commander, Historic Brawl, Oathbreaker, Pauper Commander, PreDH, Standard Brawl; then Alchemy, Explorer, Future, Gladiator, Historic, Legacy, Modern, Old School, Pauper, Penny Dreadful, Pioneer, Premodern, Standard, Timeless, Vintage). Selected chip highlighted. Tapping focuses it (fills the bar) and selects it; tapping the selected one clears the format. Changes apply live with the cascade above. Empty search result: "No results".
- ActionBar: "Back" (reverts to the snapshot taken on open: format, commander, commander text, signature spell, its text; shows info toast "Format unchanged" (`TOAST_QUICK`) if something had changed; closes) and "Save" (disabled while the pick equals the one it opened on; closes keeping changes). OS back = Back.
- HintDialog "Format": "Tap the picked format again to clear it"; "Changing the format clears the commander and signature spell".

### 2.6 Deck tags picker (`TagSelect`, full-screen overlay)

- ScreenHeader "Deck tags" with "?".
- Controls head: "Search", count "{n}/5", "×" when non-empty (clears selection and focus).
- Input placeholder "Search tags" (filters the server catalog by `display_name` substring, case-insensitive). Catalog order is as served.
- Definition bar: "Hint" / "Tap a tag to see its definition here."; after a tap, `display_name` / `description`.
- Grid of chips (`display_name`), selected highlighted. Tapping focuses it, **clears the search**, and toggles it. At 5 already selected, adding shows warning toast "You may only choose up to 5 tags" (`TOAST_QUICK`). Empty: "No results" (also what shows while the catalog has not loaded).
- ActionBar "Back" (revert to the opening snapshot, info toast "Deck tags unchanged" (`TOAST_QUICK`) when it changed, close) and "Save" (disabled when the set equals the snapshot, order ignored). OS back = Back.
- HintDialog "Deck tags": the DeckTagsExplainer bullets (§2.3 item 10) plus "Tap a tag to add or remove it. The bar up top shows what it means".
- **On close (both Back and Save)** the host reconciles oracle tags: `new_seed = deck::seed_oracle_tags_from_catalog(&selected_tags, &catalog)`; drop every oracle tag that was in the previous seed (`applied_seed`), keep manual picks, then append each new seed slug not already present while the total is under 30. Store `applied_seed = new_seed`. If anything was appended, info toast "Oracle tags derived from your deck tags" (`TOAST_NORMAL`).
  - Create starts `applied_seed` empty. Edit initializes it to `deck::seed_oracle_tags(&saved_tags)` (the compiled enum map, not the catalog).

### 2.7 Oracle tags picker (`OracleTagSelect`, full-screen overlay)

- ScreenHeader "Oracle tags" with "?".
- Controls head: "Search", a small "Dictionary" button (opens the Oracle Tag Dictionary overlay; out of scope), count "{n}/30", "×" when non-empty.
- Input placeholder "Search all oracle tags".
- Definition bar: "Hint" / "Tap a tag to see what it means. Picking a deck tag pre-selects some of these for you."; after a tap, the slug / description or "No description for this tag.".
- Grid:
  - catalog not loaded: 12 skeleton chips
  - empty search: every tag with `curated == true` plus every selected slug, sorted by slug
  - non-empty search: `oracle_tag::search_oracle_tags(&tags, &query)` truncated to 40 (exact slug/label matches, then substring, then description-only, each alphabetical)
  - no hits: "No results"
- Tapping a chip focuses it, clears the search, toggles it. Over 30: warning toast "You may only choose up to 30 oracle tags" (`TOAST_QUICK`).
- Dictionary "Use" adopts a slug: already present → info "Tag already added"; room → success "Tag added to deck"; full → warning "You may only choose up to 30 oracle tags" (all `TOAST_QUICK`). Stays in the picker.
- ActionBar "Back" (revert, info "Oracle tags unchanged" if changed) and "Save" (disabled when unchanged). OS back = Back.
- HintDialog "Oracle tags": OracleTagsExplainer bullets, plus a "Dictionary" confirm action that opens the dictionary.

### 2.8 Swipe picker (`SwipeSelect`, full-screen overlay)

Opened by each field's "Swipe" chip. Mode per slot: `Commander(format)` (needs a format; mode is None otherwise so nothing loads), `Partner`, `Background`, `SignatureSpell(commander.color_identity)` (needs a commander). Stays mounted while closed, so its pile and filter persist; reopening with an unchanged filter and mode keeps the pile.

- ScreenHeader "Swipe select" with "?". HintDialog (auto-opens once, `HINT_SWIPE_SELECT` "swipe_select", on first open) title "Swipe select", bullets: "Swipe " "right" " picks that {noun}"; "Swipe " "left" " skips it"; "Swipe " "up" " saves it to your commander maybeboard"; "Swipe " "down" " undoes the last swipe"; then line "Most-played first. " `[Filter]` " narrows by color or anything else". `noun` is "oathbreaker" (commander mode in Oathbreaker), "commander", "partner", "background", or "signature spell".
- Query: the user's filter sheet builder + the mode constraint (commander: `set_is_commander_in_format`; partner: `set_is_partner(true)`; background: `set_is_background(true)`; signature spell: `set_is_signature_spell(true)` + `set_color_identity_within`) + `set_is_token(false)` + limit 25 + offset. No sort is set, so the server returns popularity order. Calls `search_commanders`. Prefetches the next page when 5 or fewer cards remain (up to 4 page pulls if dedup empties a page). An empty page marks the pile exhausted.
- Card stack with the card's info under it. Gestures: left = skip (info toast "Skipped", `TOAST_QUICK`); right = choose (success toast "{Noun} selected", e.g. "Commander selected", `TOAST_QUICK`; the host sets the card, runs the partner autofill for commander, closes the overlay); up (commander and partner modes only) = save to commander maybeboard and advance (success "Added to your commander maybeboard" for 1.5 s; no oracle id → error "This card can't be saved"; API error → error toast with user message); down = undo (at index 0 → info "Nothing to undo"; undoing an up-save calls `remove_commander_maybeboard_card` and toasts "Removed from your commander maybeboard"; undoing a skip toasts "Undid skip").
- Loading: card skeleton. Empty pile: idle card skeleton.
- ActionBar: "Back" (closes, no selection), "Filter" (opens the shared CardFilterSheet without the format filter; a dot shows when any user filter or explicit sort is set), "Refresh" (re-runs the search from the top, info "Stack refreshed" `TOAST_QUICK`), and the eyeball "Rules" button when a card is up (card details sheet; printing sheet can swap the printing, which right-swipe then commits).
- Fades in/out over 0.2 s.

The card stack, filter sheet, details and printing sheets belong to the swipe/card specs; a terminal client needs a keyboard picker that deals this same query and maps keys to left/right/up/down.

### 2.9 Create: submit

ActionBar: "Back" (`navigator.go_back()`, disabled while saving) and "Save" (label "Saving..." while saving, disabled while saving).

On Save:

1. `DeckName::new(name)`; on error set the inline error and stop (no request).
2. Build `HttpCreateDeckProfile::builder(&name)` with:
   - `commander_id`, `partner_commander_id`, `background_id`, `signature_spell_id`: chosen cards' `scryfall_data.id` or None
   - `format`: `format.to_legality_key()` (e.g. "commander") or None
   - `tags`: Some(slugs) or None when empty
   - `power_level`: `PowerLevel::to_string()` (snake_case, e.g. "upgraded") or None
   - `other_tags`: snake_case strings or None when empty
   - `oracle_tags`: Some or None when empty
   - `land_target`: Option<i32>
   - `price_target`: text parsed as f64, kept only when `> 0`; else None
   - `price_target_currency`: the chip, only when a price target is sent
3. `authed.try_run("create_deck", create_deck_profile)`. Success → `Router::ViewDeck { deck_id: created.id }`. Error → toast (§0.3) and Save re-enables.

Server-side messages a user can see (422/409): "invalid deck name: {e}", "invalid format: {e}", "a deck may have at most 5 tags", "a deck may have at most 5 other-tags", "a deck may have at most 30 oracle tags", "land target must be between 0 and 100", "price target can't be negative", "deck with name and user combination already exists", "deck limit reached", "deck limit reached, verify your email to unlock more".

Commander seed: when the Commander maybeboard's "Create deck" parked a card in `CreateDeckCommanderSeed`, Create takes it once on mount: format = Commander, commander = that card (bypassing the format cascade).

### 2.10 Edit: load, change tracking, submit

Load: `fetch_deck` (`authed.try_run("load_deck")`). On success fill every field from `deck.deck_profile` and store each as the "original". Price target text = `format!("{v}")` (e.g. 50.0 → "50"); currency = stored or USD. Then `fetch_card` for each command-zone id; each success fills the card and its text and its original. Each failure: error toast with the user message (`TOAST_NORMAL`).

States: loading → `EditDeckSkeleton`; error → "Could not load deck".

Change tracking, field by field (unchanged fields are left off the wire via `Patch::Unchanged`):

- name: `Some(name)` if it differs from the original, else None
- commander/partner/background/signature spell: `Patch::Set(Some(id) | None)` when the card differs
- format: `Patch::Set(legality key or None)`
- tags: `Patch::Set(Some(slugs))` when the vector differs (order matters here)
- land target: `Patch::Set(value)`; None clears back to the heuristic
- price target: compares the parsed `> 0` values; empty or zero sends `Patch::Set(None)` (clears the budget)
- currency: `Patch::Set(Some(currency))` when it differs
- power level, other tags, oracle tags: `Patch::Set(...)` when different

`has_made_changes` is the OR of all of these.

ActionBar: "Back" (→ `Router::ViewDeck { deck_id }`, disabled while saving) and, **only when there are changes**, "Save changes" ("Saving..." while saving).

On Save: `DeckName::new` check as in create; if somehow no changes, error toast "must update at least one field" (`InvalidUpdateDeckProfile::NoUpdates`). Build `HttpUpdateDeckProfile::builder()...build()` and call `authed.try_run("save_profile", update_deck_profile)`. Success → `Router::ViewDeck { deck_id }`. Error → toast, Save re-enables. Server messages as in create plus "deck profile not found", "deck not found", "invalid deck tag: {e}", "invalid power level: {e}", "invalid other-tag: {e}", "must update at least one field".

---

## 3. Import deck (`Router::ImportDeck { deck_id }`, `/deck/card/import/:deck_id`, `import.rs`)

Reached from the deck view's More sheet ("Import cards") and from the below-minimum warning's "Import" button.

### 3.1 Layout

1. ScreenHeader "Import" with "?". Hint key `HINT_IMPORT` ("import"). HintDialog "Importing cards":
   - "Paste a decklist, or switch " `[From]` " to an Archidekt deck link"
   - `[Add]` " keeps your current cards. " `[Replace]` (warning color) " swaps the board's contents"
   - `[Board]` " picks where the cards go"
   - "Importing clears the deck's undo history"
2. Fixed controls (do not scroll):
   - Row "From": chips "Text" (default), "Archidekt".
   - Row "Mode": chips "Add" (default, `ImportMode::Add`), "Replace" (`ImportMode::Replace`).
   - Row "Board": chips "Main" (default, board `None`), "Maybe" (`Some("maybeboard")`), "Side" (`Some("sideboard")`).
   - Explainer line, with the board word highlighted; board word is "mainboard", "maybeboard" or "sideboard":
     - Replace: "Removes all existing cards in the " {board} ", replacing with imported cards."
     - Add: "Adds imported cards into the " {board} ". Existing cards stay but take new quantities if applicable."
3. Scrolling area:
   - Text source: label "Paste decklist", multi-line textarea (min ~12 rows), placeholder three lines "1 Krenko, Mob Boss" / "1 Urza, Lord High Artificer" / "1 Sol Ring".
   - Archidekt source: label "Paste an Archidekt deck URL", single-line URL input, placeholder "https://archidekt.com/decks/...".
   - Results (after an import): label "Imported" then one bubble per card: name and "x{quantity}" (dimmed). Label "Unresolved" then one error-styled bubble per card: name and reason (dimmed).
4. ActionBar: "Back" (→ `Router::ViewDeck { deck_id }`) and the submit button. Submit label: "Importing..." while loading, else "Replace" in Replace mode, else "Import". Disabled while loading or when the active input (text or URL) is blank after trim.

Text and URL keep their own values when switching From.

### 3.2 Calls

- Text: `ZwipeClient::import_deck_cards(&self, deck_id: Uuid, text: &str, board: Option<&str>, mode: ImportMode, session: &Session) -> Result<ImportDeckCardsResult, ClientError>` (body `HttpImportDeckCards { text, board, mode }`).
- Archidekt: `ZwipeClient::import_archidekt_deck(&self, deck_id: Uuid, url: &str, board: Option<&str>, mode: ImportMode, session: &Session) -> Result<ImportDeckCardsResult, ClientError>` (body `HttpImportArchidektDeck { url, board, mode }`).
- Both via `authed.try_run("import_cards", ...)`.
- `ImportDeckCardsResult { imported: Vec<ImportedCard { name, quantity: i32 }>, unresolved: Vec<UnresolvedCard { name, reason }> }`, imported sorted by name.

The previous result is cleared when a new import starts.

### 3.3 Parsing (server side, zwipe-core `ImportDeckCards::parse`)

The client sends raw text; parsing happens on the server with zwipe-core's parser, so a TUI that wants a local preview can call the same function (`zwipe_core::domain::deck::requests::import_deck_cards::ImportDeckCards::parse(user_id, deck_id, text, email_verified, board_override: Option<Board>, mode)`). Rules:

- One `[qty] [card name]` per line. Blank lines skipped.
- Quantity: the first word, with an optional trailing "x" ("4" or "4x"). If it does not parse, or starts with "+", quantity is 1 and the whole line is the name ("+2 Mace" is a name). Quantity < 1 or empty name → line skipped.
- Quantity over 9,999 (`MAX_IMPORT_LINE_QUANTITY`) → rejected, reason "line {n}: quantity is over the 9999 limit".
- Trailing metadata stripped at the first "(" or "[" (Archidekt/Moxfield exports: "Aether Hub (drc) 145 [Land]" → "Aether Hub").
- Section headers, only when no board override (i.e. the "Main" chip, which sends `None`): "// Sideboard", "// Maybeboard", "// Deck" switch the board (case-insensitive, optional space after "//"); any other "// ..." line (e.g. "// Commander") is skipped; a bare "Sideboard" line (MTGA) switches to sideboard. With "Maybe" or "Side" chosen, everything goes to that board and comment lines are skipped.
- Double-faced names resolve from "A // B", "A / B", or just "A" (`entry_front_face`, `dfc_front_face`).
- Replace mode replaces each board present in the import.
- Server unresolved reasons: "not found", "missing oracle id", "quantity too large".
- Archidekt: the server extracts the deck id from the URL, fetches the public deck, resolves printings by Scryfall id. "Main" sends no board and the server defaults to the deck board.

Server errors seen as toasts: "deck not found"; "invalid board value"; "card limit reached (mainboard, maybeboard, and sideboard all count toward it)"; "card limit reached across all boards, verify your email to unlock more"; "could not parse an archidekt deck id from the url"; "deck not found on archidekt, make sure it's public and the url is correct"; "deck from archidekt is too large to import"; Archidekt upstream failure is a 500, so the user sees "something went wrong, please try again"; an oversized body is "request too large, try importing fewer cards".

### 3.4 Result toasts

After a successful call, with `imported = r.imported.len()` and `unresolved = r.unresolved.len()`:

- (0, 0): info "No cards found" (`TOAST_QUICK`)
- (0, n): error "{n} card unresolved" / "{n} cards unresolved" (`TOAST_NORMAL`)
- (n, 0): success "Imported {n} card" / "Imported {n} cards" (`TOAST_QUICK`)
- (n, m): info "Imported {n}, {m} unresolved" (`TOAST_QUICK`)

If `imported > 0`, the deck's local undo history is cleared (`UndoStore::clear(deck_id)`). The screen stays put; the user leaves with Back.

---

## 4. Export deck (`Router::ExportDeck { deck_id }`, `/deck/card/export/:deck_id`, `export.rs`)

Reached from the More sheet's "Export cards" (disabled when the deck has no metrics, see §5.9).

### 4.1 Layout

1. ScreenHeader "Export" with "?". Hint key `HINT_EXPORT` ("export"). HintDialog "Exporting your deck": `[Export]` " picks which boards to include"; `[Copy]` " puts the decklist on your clipboard".
2. Fixed controls: while loading, a ChipRowSkeleton (label md; chips sm md sm). Then row "Export" with toggle chips "Deck" (default on), "Maybe" (off), "Side" (off).
   - At least one stays on: turning off "Deck" when Maybe and Side are both off does nothing; turning off Maybe or Side when it would leave none on turns "Deck" back on.
3. Scrolling area: label "Decklist" over a read-only textarea (min ~16 rows) with the text. Loading: export skeleton (label bar + textarea block). Error: "Could not load deck".
4. ActionBar: "Back" (→ `Router::ViewDeck { deck_id }`) and "Copy" (writes the text to the clipboard; info toast "Copied to clipboard" `TOAST_QUICK`; no-op before load).

### 4.2 Call

`ZwipeClient::fetch_deck(&self, deck_id: Uuid, session: &Session) -> Result<Deck, ClientError>` via `authed.try_run("load_deck")`. `Deck { deck_profile, entries: Vec<DeckEntry { card, deck_card }>, warnings, command_zone_cards: Vec<Card> }`. `deck_card.board: Board` (`is_active()` = Deck, `is_maybeboard()`, `is_sideboard()`), `deck_card.quantity: Quantity` (deref to i32).

### 4.3 Text format (built client-side, entries in served order)

```
// Commander            <- only when any command-zone name exists
1 {commander_name}
1 {partner_commander_name}
1 {background_name}
1 {signature_spell_name}
                        <- blank line after the command zone
// Deck                 <- when Deck is on and the deck board has cards
{qty} {card name}
...

// Maybeboard           <- when Maybe is on and non-empty (preceded by a blank line)
{qty} {name}

// Sideboard            <- when Side is on and non-empty (preceded by a blank line)
{qty} {name}
```

Lines are joined with "\n". Order is Commander, Deck, Maybeboard, Sideboard. This round-trips through the importer (the importer skips "// Commander").

---

## 5. Deck view (`Router::ViewDeck { deck_id }`, `/deck/get/:deck_id`, `view.rs`)

### 5.1 Calls

All keyed on `deck_id`, so navigating ViewDeck→ViewDeck (after Clone) refetches.

1. `fetch_deck_profile(&self, deck_id: Uuid, session: &Session) -> Result<DeckProfile, ClientError>` (`try_run("load_deck")`).
2. `fetch_card(commander_id)` when the profile has a commander. Its name is preferred for the Profile's Commander row. Error → error toast (user message, `TOAST_NORMAL`).
3. `fetch_deck(deck_id, session)` (`try_run("load_deck")`) → `(entries, warnings, command_zone_cards)`.
4. Oracle tag catalog (`fetch_oracle_tags`, cached) for the Roles tag descriptions.
5. One-time hint `HINT_FIRST_DECK` ("first_deck") on mount.

### 5.2 zwipe-core work

- `metrics = DeckMetrics::from_entries_and_command_zone(&entries, &command_zone_cards)` only when entries or command zone is non-empty. Counts only `Board::Deck` entries (qty ≥ 1 each); command-zone cards add 1 to `total_cards` and to prices but never to distributions; a command-zone card that is also an active entry is not double-counted.
- `chart_metrics` = metrics only when `land_count + nonland_count > 0` (some mainboard card).
- From chart_metrics: `mana_curve_bars() -> [(usize, u32); 7]`, `type_bars()`, `color_bars()` (labels abbreviated: types "lands", "creat", "plnsw", "artif", "enchn", "instn", "sorcr", "other"; colors "white", "blue", "black", "red", "green", "multi", "clrls"), `card_role_bars() -> Option<Vec<(&str, usize, u32)>>` (sorted by count desc), `card_role_tags: BTreeMap<&str, Vec<String>>`, `draw_odds_buckets() -> (deck_size, [("Land", lands), (role, count)...])`, `mana_balance_rows() -> Vec<ManaBalanceRow { label, consumed, produced, fill_pct, is_surplus }>` (only colors with consumed > 0). Percentages via `DeckMetrics::bar_pct` (tallest = 100, nonzero floor 4).
- Draw odds: `zwipe_core::domain::deck::draw_odds::p_at_least_one(deck_size, k, draws)`, draws = 7 + turn.
- Price formatting: `PriceCurrency::format_amount` → "$12.34", "€12.34", "12.34 TIX".
- Retailer URLs: `zwiper/src/lib/outbound/buy_links.rs` (app code, not core; the TUI must copy it).

### 5.3 Layout top to bottom

1. ScreenHeader "Deck" with "?".
2. Content (fades in):
   - **Loading** (profile not loaded): `DeckProfileSkeleton` + `DeckStatsSkeleton`.
   - **Error**: "Could not load deck".
   - **Loaded**: Profile card, then the collapsible sections in this order: Budget, Tags, [Distributions, Mana, Draw odds], Warnings. Each appears only when it applies (rules below). While `fetch_deck` is in flight and metrics are possible (card_count > 0 or any command-zone id set), a `DeckStatsSkeleton` stands where Distributions/Mana/Draw odds go.
3. ActionBar (fades in 0.15 s after content): "Back" (→ `Router::DeckList {}`), "Edit" (→ `Router::EditDeck { deck_id }`), "Cards" (→ `Router::ViewDeckCard { deck_id }`), "More" (opens the More sheet).
4. Dialogs and sheets (§5.9 to §5.13), the Oracle tag examples overlay, and the first-deck hint.

### 5.4 Profile card (always first; not collapsible)

Title "Profile", then label/value rows:

- "Name": `deck_profile.name`
- "Cards": `metrics.total_cards` if metrics exist, else `max(card_count, 0)`
- "Format": `display_name()` or "None"
- Commander row only when the format `has_commander()`: label "Oathbreaker" in Oathbreaker, else "Commander"; value = fetched commander card name, else `commander_name`, else "None"
- "Partner": `partner_commander_name`, when set
- "Background": `background_name`, when set
- "Signature spell": `signature_spell_name`, when set
- "Power level": `display_name()`, when set

### 5.5 Collapsible sections (accordion)

`CollapsibleSection`: a bordered card (warning-colored border for Warnings) with a header "▸ {title}" and an optional faint badge, plus an optional header accessory on the right. The whole card toggles; the body and accessory do not. **Only one section is open at a time** (`open_section: Option<String>` holds the open title); tapping the open one closes it, leaving none open. Default open: "Budget". The accessory fades in only while open. Body has a full-width rule under the header. Body eases open/closed 0.25 s.

**Budget** (shown when `price_target.is_some()` or metrics exist)

- Accessory (only when metrics exist): currency chips "USD", "EUR", "TIX". Default `usd`, seeded once from `price_target_currency` when the profile loads; later chip taps stick.
- Rows:
  - "Total price" (when metrics or a target exist): running total in the selected currency, `format_amount(total.unwrap_or(0.0))` (an unpriced deck reads "$0.00"); with a target: "{total} / {target}" where the target uses **its own stored currency** (`price_target_currency`, default USD). No conversion; switching chips reads a different pre-summed total.
  - "Average card price" (only when metrics exist): `format_amount(avg)` or "N/A".

**Tags** (shown when any of tags, oracle_tags, other_tags is non-empty). Badge: total count across all three.

- "Deck tags" (InfoButton DeckTags): chips `deck_tag_label(slug)`
- "Oracle tags" (InfoButton OracleTags): chips of slugs
- "Other tags": chips `display_name()` Each row only when non-empty; label above, chips below (stacked row).

**Distributions** (shown when chart_metrics exist)

- "Average P/T" row, only when `avg_power` or `avg_toughness` is Some: "{p} / {t}" each with one decimal, "-" when None.
- **Types** chart: vertical bars, label chip "Types" with InfoButton TypeChart ("Mainboard cards by type"). Each column: count above when > 0, bar height = pct, label below (uppercased).
- **Roles** chart (when `card_role_bars()` is non-empty): horizontal rows, label chip "Roles" with InfoButton RoleChart ("Mainboard cards by role, like ramp or removal. Cards with no role are left out. Tap a role to see its tags").
  - **Tap to expand**: each role whose `card_role_tags` entry is non-empty draws its label as a tappable chip (uppercased); a role with no tags is plain text and not tappable. Tapping a role opens a reveal directly under its row (eases open), one role at a time; tapping it again closes it. The reveal (`RoleTags`) shows that role's oracle tag chips, ranked by copies, most first, ties by slug. Tapping a tag chip opens its definition under the chips (one at a time; tap again closes): the description from the oracle tag catalog, or "No description yet", and an "Examples" button. "Examples" opens the Oracle tag examples overlay (`OracleTagExamples { slug }`, ScreenHeader "Examples": a read-only swipe browse of cards carrying the tag via `search_cards`; out of scope here).

**Mana** (shown when chart_metrics exist **or** `land_target` is set)

- "Lands": `land_count` (0 without metrics), or "{land_count} / {land_target}" when a target is set.
- "Average mana value": `avg_cmc` one decimal, only when chart_metrics exist.
- **Mana curve**: label chip "Mana curve" + InfoButton ("Nonland mainboard cards by mana value"); 7 vertical bars labeled "0" "1" "2" "3" "4" "5" "6+", count above when > 0.
- **Colors**: label chip "Colors" + InfoButton ("Mainboard cards by color"); vertical bars with the color labels.
- **Mana cost fulfillment** (when any row): label chip "Mana cost fulfillment" + InfoButton (bullets "Per color, the pips your lands and sources make against the pips your spells need"; "A check means you make at least as much as you need"). One row per consumed color: letter (W/U/B/R/G), bar filled to `fill_pct`, "✔" when `is_surplus` else blank, then "{produced}/{consumed}".

**Draw odds** (shown when chart_metrics exist)

- Header accessory: InfoButton DrawOdds (bullets "Chance of at least one card from each group in the opening hand, or by a turn. " `[-]` " and " `[+]` " step the turn"; "Random draw only. Mulligans, tutors and card draw are not counted").
- Body: stepper "-" [heading chip] "+". Heading: "Odds of ≥1 in opening hand" at turn 0, else "Odds of ≥1 by turn {n}". Turn range 0..=20; buttons no-op at the bounds (no gray-out).
- One row per bucket: label uppercased ("LAND", then each role), bar to pct, "{pct}%", pct = `round(p_at_least_one(deck_size, k, 7 + turn) * 100)`.

**Warnings** (shown when `fetch_deck` returned warnings). Warning border, badge = count. One row per warning: the message (muted) and, on the right, its fix (small warning-styled buttons):

- **Below-minimum card count** (no card id and the message contains "requires at least", e.g. "deck has 60 cards, commander requires at least 100"): two buttons "Add cards" (→ `Router::AddDeckCard { deck_id }`) and "Import" (→ `Router::ImportDeck { deck_id }`). Matched on the text on purpose; the wire enum stays unchanged.
- Card warning with `WarningAction::FixQuantity(n)` (copy limit, e.g. "Sol Ring exceeds copy limit (2/1)"): button "Fix to {n}". Calls `update_deck_card(&self, deck_id: Uuid, scryfall_data_id: Uuid, request: &HttpPatchDeckCard, session: &Session) -> Result<DeckCard, ClientError>` with `HttpPatchDeckCard::new(Some(n), None)` (absolute quantity). Success: info toast "Quantity set to {n}" (`TOAST_QUICK`), reload the deck. Error: toast (§0.3).
- Card warning with `WarningAction::ClearCommander` ("{name} is not a valid commander for {format}"): button "Clear". Calls `update_deck_profile(deck_id, HttpUpdateDeckProfile::builder().commander_id(Patch::Set(None)).build())`. Success: info toast "Oathbreaker cleared" in Oathbreaker, else "Commander cleared" (`TOAST_QUICK`); reload profile and deck.
- Any other warning that carries a card id (action None or `Remove`, e.g. "{card} is not legal in {format}", "{card} is banned in {format}", "{card} is outside commander's color identity"): button "Remove". Calls `delete_deck_card(&self, deck_id: Uuid, scryfall_data_id: Uuid, session: &Session) -> Result<(), ClientError>` directly (not through Authed; a missing session yields "Session expired"). Success: info "Card removed" (`TOAST_QUICK`), reload the deck. Error: error toast with the user message (`TOAST_NORMAL`).
- Deck-level warnings with no card id and no "requires at least" text get no button.

Full warning texts from `validate_deck.rs` (for reference): "deck has {n} {card|cards}, {format} requires at least {min}"; "deck has {n} {card|cards}, {format} allows at most {max}"; "deck has {n} {land|lands}, below the land target of {target}"; "deck totals {amount}, over your {budget} budget"; "the format, {format}, requires a commander"; "{card} is not legal in {format}"; "{card} is banned in {format}"; "{card} exceeds copy limit ({n}/{max})"; "{card} is outside commander's color identity"; "{card} is not a valid commander for {format}"; "{format} does not support partner commanders"; "{card} does not have a partner ability"; "{a} and {b} cannot be partners"; "{format} does not support backgrounds"; "{card} does not have 'choose a background'"; "{card} is not a valid background enchantment"; "a commander cannot have both a partner and a background"; "{format} does not use sideboards"; "sideboard has {n} cards, {format} allows at most {max}"; "oathbreaker format requires a signature spell"; "{format} does not use signature spells"; "{card} must be an instant or sorcery to be a signature spell"; "{card} is outside the oathbreaker's color identity".

### 5.6 First-deck hint

`use_one_time_hint(HINT_FIRST_DECK)`: auto-opens once per account the first time any deck view mounts; "?" reopens. HintDialog title "Welcome to your deck", bullets:

- `[Cards]` " browses, adds and removes cards"
- `[Edit]` " changes the name, format, command zone, tags and targets"
- `[More]` " imports, exports, clones, shares or deletes the deck"
- "Skipped and removed cards stay out of the add stack. " `[More]` " then Clear skips brings them back"
- "Stats fill in as the deck grows. Tap a section to expand it"
- "Warnings flag rule problems, each with a one-tap fix"

### 5.7 State seeding

- `share_token` is seeded from the loaded profile and updated in place by Share / Stop share.
- `has_cards` = metrics exist (any active entry or command-zone card).
- `tcg_url` / `ck_url` exist once `fetch_deck` has loaded (built from active **and sideboard** entries; maybeboard excluded).

### 5.8 Buy URL rules (`buy_links.rs`)

- Card name encoding (`encode_name`): take the front face (`name.split(" // ")[0]`), then replace " " with "+", "," with "%2C", "'" with "%27". Nothing else is escaped.
- Command-zone names (commander, partner, background, signature spell, from the profile, in that order) come first, each as "1+{name}".
- Then each entry (deck + sideboard boards, served order) as "{qty}+{name}".
- TCGplayer: `https://www.tcgplayer.com/massentry?c=` + items joined by "||".
- Card Kingdom: `https://www.cardkingdom.com/builder?c=` + items joined by "%0A".

### 5.9 More sheet (`MoreButtons`, BottomSheet title "More actions")

Buttons in order:

1. "Import cards": closes sheet → `Router::ImportDeck { deck_id }`.
2. "Export cards": disabled when `!has_cards`; closes sheet → `Router::ExportDeck { deck_id }`.
3. "Buy deck": disabled when `!has_cards`; closes sheet, opens the Buy dialog.
4. "Clone deck": closes sheet, opens the Clone dialog.
5. "Share deck": opens the Share dialog **over** the sheet (the sheet stays open underneath until an action in the dialog closes it).
6. "Clear skips" (danger style): opens the Clear skips dialog over the sheet.
7. "Delete deck" (danger style): opens the Delete dialog over the sheet. Default "Close" footer.

### 5.10 Buy dialog

Title "Buy deck". Body "Buy this deck's cards from a retailer." when at least one URL exists, else "No buy links are available for this deck." (only possible if the deck had not loaded). Actions: "Close" (dismiss), "TCGplayer ↗" (link), "Card Kingdom ↗" (link) (`\u{2197}`). A link opens the system browser and closes the dialog.

### 5.11 Clone dialog (`CloneDeckDialog`)

- Title "Clone deck". Body "Make an exact copy of your deck under a new name." and a text input, placeholder "New deck name", prefilled with "{deck name} (clone)" each time the dialog opens.
- Actions: "Cancel" (dismiss) and "Save" (ActionButton, disabled while the trimmed name is empty; busy label "Cloning...").
- Save: `clone_deck(&self, source_deck_id: Uuid, body: &HttpCloneDeck { new_name: trimmed }, session: &Session) -> Result<HttpClonedDeck { deck_id }, ClientError>` via `authed.run_at(CLONE_DECK_DIALOG, "clone_deck")`. Success: info toast `Cloned as "{name}"` (`TOAST_QUICK`), navigate to `Router::ViewDeck { deck_id: cloned.deck_id }`, dialog closes. Failure: error toast (§0.3), dialog stays open. No client-side name validation; server messages: "invalid deck name: {e}", "a deck with that name already exists", "deck limit reached", "deck limit reached, verify your email to unlock more", "source deck not found", "deck not found".

### 5.12 Share dialog

Title "Share deck". One dialog covers the whole lifecycle, and stopping acts directly with no second confirm.

- Not shared (`share_token` None): body "Anyone with the link can view this deck. You can stop sharing at any time." Actions: "Close", "Create link".
  - "Create link": closes the More sheet; `share_deck(&self, deck_id: Uuid, session: &Session) -> Result<HttpDeckShareToken { share_token: Uuid }, ClientError>` (`run_at(MORE_BUTTONS, "share_deck")`). Success: store the token, copy the link, info toast "Share link copied" (`TOAST_QUICK`).
- Shared: body "Anyone with the link can view this deck. Stop sharing disables the link; sharing again creates a new one." Actions: "Close", "Copy link", "Stop share" (danger).
  - "Copy link": closes the More sheet, copies the current link, toast "Share link copied".
  - "Stop share": closes the More sheet; `unshare_deck(&self, deck_id: Uuid, session: &Session) -> Result<(), ClientError>` (`run_at(MORE_BUTTONS, "unshare_deck")`). Success: token = None, info toast "Link disabled" (`TOAST_QUICK`).
- Link: `format!("{WEB_BASE}/deck/{token}")`. `zwipe_core::domain::site::WEB_BASE` is "http://localhost:8080" in debug builds and "https://zwipe.net" in release builds (no trailing slash). Copied with `navigator.clipboard.writeText` in the app.
- Re-sharing after Stop creates a new token; old links die.

### 5.13 Clear skips dialog

Title "Clear skips". Body "Skipped cards come back on their own after a week, and removed cards after a month. Clearing brings them all back now. This can't be undone." Actions: "Cancel", "Clear" (danger).

"Clear": closes the More sheet; first flushes the pending telemetry buffer (`flush_loop::flush_once`) so skips not yet sent are cleared too; then `clear_deck_suppressions(&self, deck_id: Uuid, session: &Session) -> Result<HttpClearedSuppressions { cleared: u64 }, ClientError>` (`run_at(MORE_BUTTONS, "clear_suppressions")`). Success: info toast "Skips cleared" (`TOAST_QUICK`). The count is not shown.

### 5.14 Delete dialog

Title "Delete deck". Body "Are you sure you want to delete this deck?" Actions: "Cancel", "Delete" (danger).

"Delete": `delete_deck(&self, deck_id: Uuid, session: &Session) -> Result<(), ClientError>` (`authed.run("delete_deck")`). Success: navigate to `Router::DeckList {}` (no toast). Failure: error toast ("deck not found" etc.). If cancelled, the More sheet is still open underneath.

### 5.15 Navigation summary

In: deck list row; Create save; Edit save and Back; Import/Export Back; Clone success (ViewDeck→ViewDeck). Out: Back → DeckList; Edit → EditDeck; Cards → ViewDeckCard; More → ImportDeck / ExportDeck; warnings → AddDeckCard / ImportDeck; Delete → DeckList; Clone → ViewDeck(new); Buy → external browser.

---

## 6. TUI today (`zwipe-tui/src/lib`)

What exists:

- Screens: `Screen::Decks(DeckList)` and `Screen::Deck(Box<DeckView>)` (`inbound/app/mod.rs`), drawn by `inbound/ui/decks.rs` and `inbound/ui/deck.rs`, keys in `inbound/event_loop/mod.rs` (`handle_decks_key`, `handle_deck_key`), requests `Request::LoadDecks` (`fetch_deck_profiles`) and `Request::LoadDeck(id)` (`fetch_deck`).
- `domain/deck/mod.rs`: `card_count` (same command-zone rule as the app), `cards_label` ("1 card" / "{n} cards", which differs from the app's always-plural "cards"), `count_fits`, `pips`.
- `inbound/ui/charts.rs`: `columns` (vertical ratatui BarChart) and `row` (horizontal block bar). Reusable for every chart.
- Deck list: bordered "Decks" list, j/k or arrows, Enter opens, r reloads, Esc home, O sign out, q quit. Header "ZWIPE  {username}". Rows: name, pips (WUBRG letters in mana colors), count (warning color when it breaks the format size), format, power level, command-zone names (accent), deck and other tags joined by ", ". Sorted by lowercase name. States: "Loading decks...", the error message in red, "No decks yet. Create one in the ZWIPE app.".
- Deck view: header is the deck name; left Profile column (Name, Cards, Format, Commander/Oathbreaker from `commander_name`, Partner, Background, Signature spell, Power level); right tabs for the sections that apply (same visibility rules as the app via `DeckView::sections()`); Tab/Shift-Tab/h/l switch tabs; `c` cycles currency; `+`/`=` and `-` step the draw-odds turn (0..20); `r` reloads; `s` starts swiping; Esc back to decks; q quits. Currency defaults to `price_target_currency`. Warnings tab title "Warnings ({n})" in warning color. States: "Loading deck...", error text, "Stats fill in as the deck grows.".

Gaps and differences:

Deck list
1. No Group by row (None/Format/Color/Tag), no collapsible groups or "({count})" headers, no "All" group header.
2. No Show row: no All, no color filter (AND), no tag filter (OR), no Tags fold chip and tag row, no "No decks match".
3. No hint dialog ("Your decks") or `mark_hint_shown`.
4. No Create action, so no unverified gate toast "Verify your email to create more than 1 deck"; the app also refreshes the user (`fetch_user`) on mount for that gate.
5. No More sheet (Commander maybeboard, Playstyle).
6. Empty text differs ("No decks yet. Create one in the ZWIPE app." vs "No decks"); error shows the raw message instead of "Could not load decks" plus an error toast.
7. Count label pluralization differs ("1 card" vs the app's "1 cards").
8. Tags are one comma-joined span instead of separate chips; command-zone art crops are absent (fine; see §1.5 for options).
9. No skeletons (text placeholders are a fair terminal substitute).

Create / Edit
10. Neither screen exists. Needed: the whole form (§2.3), inline `DeckName` validation, format picker with the details bar, deck tag picker (catalog fetch, cap 5, oracle-tag seeding and its toast), oracle tag picker (catalog, curated grid, ranked search capped at 40, cap 30, dictionary), other tags, power level, land stepper with format heuristic, price target with currency and digit filter, the four command-zone typeaheads with Filter toggles, debounce and min-length rules, the commander cascades, named-partner autofill, the swipe picker (or a keyboard equivalent), Back/Save with revert snapshots and the "unchanged" toasts, edit-mode change tracking with `Patch` and the "Save changes" visibility rule.

Import / Export
11. Neither exists. Import needs From/Mode/Board selectors, explainer line, multi-line paste input (terminal: a multi-line editor or reading from a file/stdin/clipboard), Archidekt URL input, both client calls, result lists, the four result toasts. Export needs board toggles with the at-least-one rule, the exact text format, and a copy action.

Deck view
12. Layout differs: the app stacks all sections as an accordion (one open, default Budget, can be all closed); the TUI shows one tab at a time. Tabs are an acceptable terminal translation, but the TUI always has one section showing.
13. Budget: the TUI draws currency chips and "Average card price" even without metrics; the app shows the chips and that row only when metrics exist. Values match otherwise.
14. Tags: the TUI omits the per-row "?" explainers and the badge count (tab title has no count).
15. Distributions: the TUI always shows "Average P/T"; the app shows it only when either average exists. Types chart labels are not uppercased in the TUI. **Roles has no tap-to-expand**: no role selection, no tag chips, no tag definitions ("No description yet"), no Examples, and no oracle tag catalog fetch.
16. Mana: matches, except the TUI has no "?" explainers.
17. Draw odds: matches (heading, 0..20, rounding), minus the "?".
18. Warnings: messages only. Missing every fix action: "Add cards" / "Import" for the below-minimum warning, "Fix to {n}" (`update_deck_card`), "Clear" (commander, `update_deck_profile`), "Remove" (`delete_deck_card`), their toasts and the reload.
19. Profile: uses `commander_name`, not a fetched card (same visible result). "Cards" matches.
20. No action bar equivalents for Edit or Cards (Cards → the card list/add/remove screens belong to other specs; the TUI's `s` jumps straight to swiping).
21. No More sheet: Import, Export, Buy (URL builder and opening a browser), Clone (dialog with prefilled name, busy state), Share/Copy/Stop share (token state, WEB_BASE link, clipboard), Clear skips (flush then `clear_deck_suppressions`), Delete (confirm, then back to the list).
22. No first-deck hint ("Welcome to your deck") and no one-time hint recording anywhere on these screens.
23. No toast system: the TUI has a single status line (`app.status`). Every toast in this spec needs a home, with kind (info/success/warning/error) and a timeout.
24. The TUI fetches only `fetch_deck` (profile comes with it); the app also calls `fetch_deck_profile`. Not a visible difference.

---

## 7. Shared pieces the TUI needs

1. **Toast queue**: kinds info/success/warning/error, durations 3 s / 5 s / 8 s (`TOAST_QUICK/NORMAL/LONG`), drawn over the status line or a corner. Requires timed redraws (extend `App::animating`).
2. **Authed call helper**: refresh session, call, on error toast `to_user_message()` (or sign out on `Unauthorized`), with quiet and non-quiet variants. The TUI's `failure()` covers half of this.
3. **Confirm dialog widget** matching `AlertDialog`: title, body, actions of kinds Dismiss / Confirm(danger) / Link / busy Button that can keep the dialog open. The account screen's `Dialog` (`inbound/app/account.rs`) and `panel()` in `inbound/ui/mod.rs` are a starting point. Used by Buy, Clone, Share, Clear skips, Delete, and InfoButton explainers ("Got it").
4. **Bottom-sheet / menu widget** for "More actions" (deck list and deck view), with disabled items and danger styling.
5. **Hint dialogs**: one-time display keyed by `HINT_*` (check `user.has_seen_hint`, call `mark_hint_shown`, store the returned user), plus a "?" key to reopen. Texts are in this spec.
6. **Info explainer popup** keyed by `HintTopic` (titles and bodies in `zwiper/src/lib/inbound/components/hint_host.rs`), so each field and chart can show its "?" text.
7. **Chip rows**: single-select (Group by, Format, Power level, From/Mode/Board, currency) and multi-select (Show colors and tags, Other tags, Export boards) with a label chip and a selected style; keyboard focus and toggle.
8. **Collapsible group list** (deck list groups) and an accordion or tab set (deck view sections).
9. **Text inputs**: single-line with placeholder "Not set", inline error, digit filter; multi-line for the import paste; read-only scrollable text for export.
10. **Debounced typeahead** with generation tickets (800 ms, per-field min length and limit), result list, "Searching..." and "No results".
11. **Searchable picker overlay** (format, deck tags, oracle tags): search box, definition bar, chip grid, count "n/max", clear-all, Back (revert + "unchanged" toast) and Save (disabled when unchanged).
12. **Catalog cache** for `fetch_deck_tags` (session) and `fetch_oracle_tags` (no session), loaded once and reused by the pickers and the Roles reveal.
13. **Clipboard**: OSC 52 is the practical terminal route (works over SSH); fall back to printing the text or link in the status line. Used by Export Copy and Share links.
14. **Open URL**: for Buy links (and any Link action). Spawn the platform opener (`xdg-open` / `open`), and show the URL so the user can copy it if that fails.
15. **Port of `buy_links.rs`** (`tcgplayer_url`, `cardkingdom_url`, `encode_name`) into the TUI or, better, into zwipe-core so both clients share it.
16. **Deck-list grouping and filtering logic** (`group_decks`, `matches_filters`, `tag_labels`, `identity_colors`) is app code today; lifting it into zwipe-core would let the TUI reuse it byte for byte. Same for the export text builder and the warning-fix classification (`is_below_min_cards`).
17. **Chart primitives**: the existing `columns` and `row` in `inbound/ui/charts.rs`, plus a selectable horizontal chart with an inline reveal row for Roles.
18. **Keyboard swipe picker** for command-zone slots, reusing the TUI's swipe stack (`domain/swipe`) with `search_commanders` and the per-mode constraint, plus the up-swipe maybeboard save and undo.
19. **Commander seed handoff** from the commander maybeboard into Create (format Commander + commander preset), if that screen is ported.
