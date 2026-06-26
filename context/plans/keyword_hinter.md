# Keyword hinter + `feat/zwipe-select` branch summary

Record before a context compaction. Part 1 captures what shipped on the
`feat/zwipe-select` branch; Part 2 is the plan for the next feature, a keyword
hinter.

## Part 1 — `feat/zwipe-select` branch (committed, not yet merged to main)

Branch is ahead of `main` by these commits (oldest first):

- `wip(zwiper): Zwipe select — commander swipe-to-pick + EdhrecRank sort` — Phase 1.
  - `OrderByOption::EdhrecRank` in zwipe-core (enum + both `filter_cards.rs` sort
    sites) and zerver SQL (`ORDER BY edhrec_rank ASC NULLS LAST, name ASC`).
    **Additive; server must deploy before any client build that sends it.**
  - Initial `CommanderSwipe` screen + hint-key renames to match screens
    (`add_deck_cards`, `remove_deck_cards`, `swipe_select`, `create_deck`,
    `edit_deck`).
- `fix(zwiper): gate add-cards auto-serve and prompt for a filter` — add-cards only
  auto-serves a meaningful query (commander format **+** commander → synergy from
  cache; else a real user filter). Otherwise empty stack + a yellow warn toast
  "Try a filter to start swiping". Gated on `deck_loaded`, Search mode only.
- `feat(zwiper): explainer hint on the card filter sheet` — `?` in the sheet header
  opens a dialog; auto-opens once per account; new `HINT_FILTER` key.
- `feat(zwiper): card-row detail polish` — expanded deck-card rows: full-width
  dividers, type/rarity/set as separate **info chips** (no pipe), oracle text with
  `{mana}/{tap}/{energy}` etc. rendered as **Mana-font glyphs** (`OracleText`
  component) + blank lines between abilities, and a smooth grid-rows
  expand/collapse.
- `feat(zwiper): Zwipe select for partner / background / signature spell` —
  `CommanderSwipe` → **`SwipeSelect`** with a `SwipeMode` enum
  (`Commander(Format)` / `Partner` / `Background` / `SignatureSpell(Colors)`); a
  "Zwipe" chip on each command-zone field; a picker per field in create/edit.
  Picker: cross-fades, keeps filter + position, Back/Refresh, skip/undo/select
  toasts, format-aware noun (oathbreaker vs commander). Also **eased the deck-form
  fields** open/closed via a `Collapsible` (grid-rows) instead of popping.
- `style(zwiper): styles for Zwipe select, collapsible fields, and card-row detail`
  — all the supporting `main.css` (couldn't hunk-split it).

Key reusable pieces introduced:
- **Collapse animation pattern**: always-mount + CSS `grid-template-rows: 0fr<->1fr`
  + opacity, toggled by an `.open`/`.show` class. Used by the deck-form fields
  (`Collapsible` in `deck_fields.rs`), the card-row expand (`.card-row-collapse`),
  and the Zwipe-select overlay (`.swipe-select-screen.show`). This is how to
  animate *both* directions in Dioxus (it can't animate an unmount).
- **`OracleText`** (`screens/deck/card/components/oracle_text.rs`): parses `{...}`
  tokens → Mana-font `<i class="ms ms-…">` glyphs. Reuse it anywhere oracle text
  shows.

## Part 2 — Keyword hinter (next feature)

**Goal:** surface a card's keyword abilities (Flying, Trample, Ward, …) with a
short reminder of what each does, so players (especially new ones) understand the
card. Lives in the expanded card-row detail (and optionally the swipe card-info /
image preview).

**Data already available:** `scryfall_data.keywords: Option<Vec<String>>` — the
keyword names on that printing (e.g. `["Flying", "Trample"]`). No backend work
needed for the list itself.

**The one new thing needed:** a keyword → reminder-text lookup. Build it static and
pure.

### Steps

1. **zwipe-core — keyword reminders (pure, no deps).**
   New `domain/card/models/keyword.rs` (or `keyword_reminder.rs`) exporting
   `pub fn keyword_reminder(name: &str) -> Option<&'static str>`. Match on the
   lowercased keyword → a short reminder string. Start with the ~20 evergreen
   keywords (Flying, First strike, Double strike, Deathtouch, Lifelink, Vigilance,
   Trample, Haste, Reach, Menace, Defender, Flash, Hexproof, Indestructible, Ward,
   Protection, Equip, Enchant, Flying, Prowess) plus ~30–40 common ones. Unknown
   keyword → `None`. Keep it lowercased-keyed and tolerant of trailing reminder
   detail (e.g. "Ward {2}" → match on the leading word).

2. **zwiper — `KeywordChips` component** (in `screens/deck/card/components/`).
   Props: `keywords: Vec<String>`. Renders a chip per keyword (reuse the new
   `detail-chip` look, but tappable). Tapping a chip reveals its reminder —
   simplest is an **inline expand** below the chip row (toggle a
   `Signal<Option<usize>>` for which is open) rather than a dialog. Keywords with
   no reminder still render as a plain chip (name only, not tappable) or are
   skipped — decide during build.

3. **Wire into the card detail.** In `card_row.rs`, inside the expanded
   `card-row-detail`, add a Keywords section (shown only when `keywords` is
   non-empty) between the type-line chip and the oracle text, using `KeywordChips`.
   Optionally also add to the swipe `CardInfoDisplay` for parity.

4. **CSS** (`main.css`): a tappable keyword-chip style (accent-bordered like
   `chip-primary`, or a distinct color) + the inline reminder reveal (muted text,
   small). If using the grid-rows collapse for the reveal, reuse the existing
   pattern.

5. **Edge cases:** empty/absent `keywords` → hide the section; unknown keyword →
   name-only chip; very long keyword lists → the chip row wraps (flex-wrap).

### Open decisions (resolve at build time)

- Inline expand vs. a small popover/dialog for the reminder (lean inline).
- How exhaustive the reminder map needs to be (start evergreen + popular, grow
  later). Consider whether to source reminder text from Scryfall's own reminder
  text in oracle text where present, vs. the static map (static is simpler and
  consistent).
- Surface only in `card_row`, or also the swipe screen + image preview.
