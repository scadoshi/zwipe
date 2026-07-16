# No-image cards — render a text "identity frame" instead of hiding them

**Status: IMPLEMENTED 2026-07-16 (local, uncommitted, not yet runtime-tested).
Client-only; shared component + serve screens.**

**As-built:** `FlippableCardImage` no-image branch renders a `.no-image-card`
identity frame (name + mana, type line, oracle text box, P/T or loyalty footer);
CSS in `zwipe-components/assets/components.css` (themed, no glow). Dropped the image
clause from every serve filter: examples (`oracle_tag_examples.rs`, now no client
filter at all), add (`add.rs` ×3 serve paths, in-deck/dedup kept), swipe_select
(`swipe_select.rs` ×2, `!seen` kept). Removed the now-unused `ImageSize` imports.
fmt + clippy clean. The swipe-stack *exit* animation overlay
(`interactions/swipe/stack.rs`) was also switched from a raw `img` to
`FlippableCardImage { flippable: false }`, so the fly-off matches the resting card
and image-less cards keep their frame throughout (no blank flash). Every swipe
surface now renders cards through the one component.

---

**Original plan below (PLANNED 2026-07-15). Client-only; shared component + examples
screen.**
Stop discarding cards that lack art. Render them as a card-shaped **identity
frame** (name, mana, type, oracle text) so the user can still read the card and
swipe past it. Lives in the shared `FlippableCardImage`, so every swipe/preview
surface gets it; the oracle-tag **examples** browse is the first screen to stop
filtering image-less cards.

**One sentence:** a card's worth on an *examples* browse is its rules text, not
its art, so where there's no image we draw a text proxy in the card's footprint
instead of dropping the card (which also deletes the barren-page paging problem
for examples).

**Related:**
- [`otag_example_cards.md`](otag_example_cards.md) — the browse this unblocks.
- [`otag_examples_followup.md`](otag_examples_followup.md) — re-scoped: examples no
  longer needs `fetch_usable_page` (no client filter → no barren pages). The helper
  stays planned for Add / swipe_select, where the filter drops *common* in-deck /
  duplicate cards.

**Files:**
- `zwipe-components/src/flippable_card_image.rs` — add the no-image branch.
- `zwipe-components` CSS (wherever `.flippable-card-*` lives) — `.no-image-card`.
- `zwiper/.../screens/oracle_tag_examples.rs` — drop the image filter.

---

## Why

- **Image-less ≠ unworthy.** In Scryfall data the image-less population is small
  and deep-tail (brand-new/unspoiled cards awaiting scans, a few obscure printings;
  tokens are already excluded via `is_token(false)`). But each one still *matches
  the tag*, and for "what does this tag catch" the oracle text carries the entire
  lesson. Hiding them teaches less.
- **The missing thing is art — so use that space for text.** Rather than a bare
  "No Image" label, fill the card footprint with the card's identity (name, mana,
  type, text). It reads as a real card, not a broken tile, and gives *more* room
  for the detail than an art card would.
- **It deletes a bug class.** The only client-side filter on the examples screen is
  the image check; that filter is the sole reason a server page can "filter to
  empty" (the P1 barren-page issue). No filter → no barren pages → no
  `fetch_usable_page` needed *for examples*.

## Decisions (locked with owner)

- **Option B (identity frame), not a bare "No Image" label.** Text proxy in the
  card's footprint.
- **No buttons on the card.** The eyeball keeps opening the **dialog** (tried-and-
  true, clean separation of content vs chrome). The earlier idea of flipping the
  card into a details overlay with on-card Printings/close buttons is **rejected** —
  a card is a manipulable content object; buttons on it create swipe/tap ambiguity.
  The spoof-card styling survives only as the placeholder's look, not as a dialog
  replacement.
- **Show image-less cards everywhere** (owner call). Drop the *image* part of the
  filter on Examples, Add, Remove, and swipe_select. Add / Remove / swipe_select
  **keep** their in-deck / duplicate filtering — only the image check goes.

## Plan

### 1. `FlippableCardImage` — no-image branch (shared)
Today the render is `if let Some(url) = image_url { img { … } }` with **no else**,
so an image-less card collapses to an empty `.flip-face`. Add an `else` that draws
the identity frame from the `sd: ReadSignal<ScryfallData>` the component already
holds:

- **Name** (`sd.read().name`) — title row.
- **Mana cost** (`sd.read().mana_cost`) — right of the name; render as plain text
  (e.g. `{2}{U}{U}`) to keep the shared component free of zwiper-specific symbol
  assets. A symbol renderer can come later if it's worth it.
- **Type line** (`sd.read().type_line`).
- **Oracle text** (`sd.read().oracle_text`) — the "text box", scroll/clamp if long.

Keep everything else intact: the flip button still renders when `flippable &&
total > 1` (a DFC with a missing face image is treated single-faced by
`face_count()` already, so the realistic case is a whole-card text frame). The
frame is **content only** — no buttons beyond the existing flip control.

The placeholder must honor the same aspect/size the image would occupy so the
swipe stack, peeking cards, and exit animation don't shift layout.

### 2. CSS — `.no-image-card`
A crisp card-aspect frame (MTG ~5:7), matching the app's terminal/grid aesthetic —
**no glow / haze / soft gradients**. Sections: title row (name + mana), type line,
a bordered text box for oracle text. Reuse existing card-frame tokens
(`--border-*`, `--bg-*`) so it themes with light/dark automatically.

### 3. Drop the image check from "usable" everywhere
Image presence is no longer part of whether a card is shown, on any swipe screen:
- **Examples** (`oracle_tag_examples.rs` `load_more`): remove the
  `primary_image_url(ImageSize::Large).is_some()` filter; append the server page
  as-is. Success path simplifies to: server `[]` → `pagination_exhausted = true`;
  else advance offset + `stack.append(new_cards)`. With **no** client filter,
  examples can't hit a barren page → **no `fetch_usable_page` needed there.**
- **Add / Remove / swipe_select:** drop only the image clause from their filters;
  **keep** the in-deck / duplicate clauses. Because those clauses remain (and drop
  *common* cards), these screens can still barren-page → `fetch_usable_page` still
  earns its keep there (its `is_usable` no longer mentions images).
- **Optional nit (examples):** the placeholder already shows name/type/text, so the
  `CardInfoDisplay` strip below is partly redundant for an image-less card —
  optionally hide/trim it. Not required.

### 4. Keep (unchanged from the follow-up plan, independent of this)
- P2 stack cap (`MAX_CARDS_IN_STACK`) on examples `load_more`.
- P3 toast on `ensure_fresh` failure.

## Verify

- Image-less card renders the identity frame (name/mana/type/text), not an empty
  box or broken icon; sits in the same footprint as an art card.
- It's swipeable like any card (examples: left = next, down = back; right/up inert).
- Eyeball still opens the details dialog over it; flip control still works for any
  true multi-face case.
- Light + dark both read cleanly; no glow.
- Examples: a tag whose tail has image-less cards keeps serving them instead of
  ending early; a truly zero-hit tag still shows the empty copy.
- `cargo +nightly fmt` + clippy clean (both `zwipe-components` and `zwiper`).

## Not doing

- No on-card buttons / card-as-dialog (rejected).
- No mana-symbol glyphs in the frame this pass (plain text).
- `fetch_usable_page` is not built here — it moves to the Add/swipe_select pass
  (still needed there for in-deck/dup barren pages; image is no longer a factor).
