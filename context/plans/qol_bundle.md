# QOL Bundle — Launch-Feedback Quick Wins

## Goal

A single **client-only** (`zwiper`) release bundling the cheapest, most-visible
quality-of-life fixes requested in the Reddit launch thread. One build, no server
deploy, no migration — so it ships fast and reads to users as "the dev is
listening." Source list: `context/progress/feature_requests.md`.

**In scope (all client-side):** A–E below.
**Deferred (need core/server/data work):** see the bottom section.

---

## A. Live swipe drag indicators  (req #1, Tenellum)

**Problem:** "I kept forgetting which way is which and had to undo." Today the only
feedback is the post-swipe toast — nothing *while* dragging.

**Approach:** In the `SwipeStack` (`components/interactions/swipe/stack.rs`, drag
state in `state.rs`), as the top card is dragged past a small reveal threshold,
show a directional cue that intensifies with drag distance:
- a colored edge/overlay on the card, and
- a short text label ("Add" / "Skip" / "Maybe" / "Undo").

Reuse the color vocabulary already established in the swipe hint dialog so it's
consistent: `--color-success` (right/add), `--color-error` (left/skip),
`--color-warning` (up/maybe), `--accent-tertiary` (down/undo). CSS in
`assets/main.css`.

**Files:** `swipe/stack.rs`, `swipe/state.rs`, `assets/main.css`.
**Effort:** M (the meatiest item, but self-contained). **Impact:** High.

## B. Always show the card name  (req #8, MrMarijuanuh + AdditionalLeopard688)

**Problem:** Foreign / alt-art printings show an image with no name anywhere, so
users can't tell what the card is. Confirmed: `CardInfoDisplay`
(`screens/deck/card/components/card_info.rs`) renders price/set/released/artist
but **not the name**.

**Approach:**
- Add the card name as a prominent line in `CardInfoDisplay` (and/or a small
  overlay label on the card image in the stack).
- Confirm the existing detail/printing sheet (`printing_sheet.rs`) is reachable
  from the swipe card so a user can always identify/inspect the card. If the tap
  affordance isn't obvious, add a hint.

**Files:** `card_info.rs` (+ maybe `flippable_card_image.rs` for an image overlay).
**Effort:** S. **Impact:** High (kills two complaints — unidentifiable cards and
"bad images").

## C. Password rule errors under the field  (req #17, PatataMaxtex)

**Problem:** On the register screen the validation errors render in a block above
all the inputs, so it's unclear which field a "must include a number" message
belongs to.

**Approach:** In `screens/auth/register.rs`, move each field's error `div` to sit
directly under its input (username/email/password), so the message is adjacent to
the field it belongs to rather than stacked at the top. **Additionally, when a
field has an active error, give that input a red outline** so the offending field
is obvious at a glance — a `TextInput` error/invalid state (e.g. an `is_error:
bool` prop that adds an `.input-error` class driven by the per-field error
signal). Wire the existing `*_error` signals into both the message placement and
the outline.

**Files:** `register.rs`, `components/fields/text_input.rs` (add the error-state
prop + `.input-error` styling), `assets/main.css` (red-outline rule). Consider the
same pattern for `login.rs` / `change_email.rs` if cheap.
**Effort:** S. **Impact:** Low-Med (signup-funnel friction).

## D. Deck stats reachable while building  (req #12, AdditionalLeopard688)

**Problem:** While swiping in the Add (and Remove) screen there's no quick way to
see card count or mana curve — you have to leave the stack.

**Approach:** Add a **util-bar button on both the Add and Remove screens** that
opens a **bottom sheet showing deck information** — **reuse the metrics/stats
panel from the deck profile screen** (count, curve, etc.) rather than rebuilding
it. Extract that panel into a shared component if it isn't already, then mount it
inside a `BottomSheet` opened from each screen's util bar. Opening/closing the
sheet must not disturb the swipe stack position.

**Files:** `screens/deck/card/add.rs`, `screens/deck/card/remove.rs`, and the deck
profile metrics/stats component (`screens/deck/components/deck_profile.rs` or
wherever the count/curve panel lives — extract/share it).
**Effort:** M (mostly extracting + reusing the existing stats panel). **Impact:** Med.

## E. Browse all tags up front  (req #15, eragon690)

**Problem:** Users want to see the available strategy tags at deck create/edit
rather than discovering them piecemeal. (You already told the requester you'd add
"see all tags off the jump, maybe a pop-out at create/edit.")

**Approach:** A **hint dialog that lists every deck tag with a one-line
definition** of what it means — same pattern as the existing keyword hint /
swipe hint dialogs. Triggered by the `?`-style hint affordance on the tag picker
at create/edit. Requires adding short **per-tag description text** to the
`DeckTag` enum in `zwipe-core` (a `description()` method alongside the existing
`display_name()`); this is pure display text, no wire-format or validation change,
so it ships in the client build (no server deploy needed). Iterate over
`DeckTag::all()` to render name + definition.

**Files:** `zwipe-core/.../deck/models/deck_tag.rs` (add `description()`),
`deck_fields.rs` (+ `create.rs` / `edit.rs`) for the hint dialog trigger.
**Effort:** S. **Impact:** Low-Med.

---

## Open questions / decisions

1. **A — label style:** big centered word vs. corner chip; how aggressively the
   color floods the card as you drag.
2. **B — name placement:** info-row line only, or also an overlay on the image?
3. **C — scope:** just register, or apply the under-field pattern + red outline to
   login / change-email too while we're in there?
4. **D — RESOLVED:** util-bar button on Add + Remove → bottom sheet reusing the
   deck profile metrics panel.
5. **E — RESOLVED:** hint dialog listing all tags with definitions; needs
   `DeckTag::description()` text written.

## Deferred (not in this client-only bundle, and why)

- **"Typal" tag** (req #13): touches the `DeckTag` enum in `zwipe-core` + server
  validation → needs a server deploy first. Fold into the next server batch
  (alongside e.g. the suggestion-signal or limit-message deploy), not this bang.
- **Prefer original/English printing** (req #9): printing-selection ranking,
  server/data work. Pairs with B but is bigger.
- **Price threshold filter** (#10): needs query support.
- **Persist skipped cards** (#11): needs storage.
- **Auto land base / land cap / MV weighting** (#4, #5, #6): logic/recommender.
- **CubeCobra import** (#16): new import source.

## Verification & ship

- `cargo check -p zwiper` + `cargo clippy -p zwiper --all-targets -- -D warnings`.
- Manual pass in `dx serve`: drag a card each direction and watch the indicator;
  confirm the card name shows on a foreign/alt-art printing; trigger a password
  error and confirm it's under the field; open deck stats mid-swipe; see all tags
  at create/edit.
- **Deploy:** client-only — rides the next iOS/Android build. No server deploy, no
  migration, no min-version gate.
