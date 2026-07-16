# Dictionary + Examples as overlays — follow-through & adopt

**Status: PLANNED (2026-07-15). Supersedes the earlier route-based design in this
same file** (the A/B "how does the tag survive navigation" question is **deleted** —
see below). Client-only.

**One sentence:** make the oracle-tag **dictionary** and **examples** browse
**in-place overlays** (like `SwipeSelect`/`OracleTagSelect`) instead of routes, so
the host form/filter never unmounts — then **Use** writes the host's *live* signal
directly via an `on_use(slug)` callback and just toasts + stays, and **Examples**
peeks without losing anything.

**Why the rewrite:** the whole "persist the adopted tag across navigation"
problem (old Options A/inbox and B/deck-store) existed **only because the
dictionary was a route** that unmounted the origin. Owner's call: don't make it a
route. `SwipeSelect` proves a full swipe screen works as an overlay while
preserving its host's state. Remove the route → remove the problem. **A and B are
dropped.**

**Related:**
- [`otag_examples_followup.md`](otag_examples_followup.md) — parent hand-off (P2/P3
  done; this is its Dictionary follow-through, now overlay-based).
- [`no_image_card_placeholder.md`](no_image_card_placeholder.md) — orthogonal.

---

## Owner decisions (locked)

- **Dictionary and Examples are overlays, not routes.** Drop both routes
  (`/oracle-tags`, `/oracle-tags/examples/:slug`). They were under `AuthGate`; the
  hosts that embed them are already authed, so no auth regression.
- **Use applies to the live signal and STAYS** (no navigation), toasting
  "Tag added to filter" / "Tag added to deck". Row body is display-only; only the
  **Examples** / **Use** buttons are hit targets.
- Filter Dictionary link works from **both** the include and the exclude sections;
  Use writes to whichever one opened it.
- Gestures on Examples stay **left = next, down = back**; right/up inert. Eyeball
  dialog stays.

## The crux: overlay back-stack (NEW infrastructure — the part to get right)

**Finding:** the OS back intent is *not* overlay-aware today. `BackHandlerLayout`
(`back_handler.rs:43,63`) routes the iOS edge-swipe / Android back **straight to
`nav.go_back()`**. Nothing coordinates overlays with it — the existing overlays
(`SwipeSelect`, `OracleTagSelect`) close **only** via their own in-UI Back/Cancel
buttons. So an edge-swipe while any overlay is open **exits the whole screen**
today (a latent rough edge). Stacking overlays 3–4 deep makes this unacceptable, so
we build a small app-level **overlay back-stack** and have the back intent consult
it *before* falling through to `go_back()`.

### Design

App-level, provided in `spawn_upkeeper` (`session_upkeep.rs`), above the router:

```rust
/// LIFO stack of open overlays, each identified by its component ScopeId and able
/// to close itself. The OS-back handler pops the top before touching the router.
#[derive(Clone, Copy)]
pub struct OverlayBackStack {
    entries: Signal<Vec<(ScopeId, Signal<bool>)>>,
}
// impl: push(id, open), remove(id), top() -> Option<Signal<bool>>
```

A hook every overlay calls:

```rust
/// Register `open` with the back-stack while it's true, so the OS back gesture
/// closes THIS overlay (top-of-stack) instead of navigating the router.
pub fn use_overlay_back(open: Signal<bool>) {
    let mut stack: OverlayBackStack = use_context();
    let id = current_scope_id().expect("scope");   // stable per instance
    use_effect(move || {
        if open() { stack.push(id, open); } else { stack.remove(id); }
    });
    use_drop(move || stack.remove(id));            // safety on unmount
}
```

Back handler change (`back_handler.rs`, both iOS + Android arms):

```rust
if let Some(mut open) = stack.top() {
    open.set(false);          // close the top overlay; its use_effect pops it
} else if nav.can_go_back() {
    nav.go_back();
}
```

**Properties:** LIFO matches visual order; closing via an in-UI button also flips
`open` → the effect pops it, so the stack stays truthful regardless of *how* an
overlay closes. Reading `stack.top()` at intent-time (peek) is race-free enough for
a user gesture.

**Retrofit (recommended, small):** have `SwipeSelect`, `OracleTagSelect`, and the
`CardFilterSheet` bottom sheet also call `use_overlay_back`, so edge-swipe closes
*them* too — fixing the existing rough edge and making the whole stack consistent.

### Z-index

`.swipe-select-screen` is `z-index: 210`; modal-backdrop 200 / bottom-sheet 201
(`main.css`). Assign a rising band so the stack layers correctly:
selector/swipe-select 210 → **dictionary overlay 220** → **examples overlay 230**.
Each overlay is `position: fixed` full-screen (like `.swipe-select-screen`), shown
via an `.show` class gated on `open()`.

## The two overlay components

### `OracleTagDictionary` → overlay
Refactor the current route screen into an overlay component:
- Props: `open: Signal<bool>`, `on_use: Option<EventHandler<String>>`,
  `on_examples: EventHandler<String>` (host decides how to open the examples
  overlay), `on_close: EventHandler<()>`.
- Guard the whole render on `if open()` and add `use_overlay_back(open)` (mirror
  `OracleTagSelect`).
- Keep letter rail + search + rows. **Rows: drop `onclick`/`dict-row-tappable`;**
  add per-row **Examples** (calls `on_examples(slug)`) and **Use** (calls
  `on_use(slug)`; hidden when `on_use` is `None`, i.e. pure-reference use).
- The dictionary stays *dumb*: it never touches decks/filters, it only calls back.
- ActionBar: a **Back/Done** that calls `on_close`.
- Hint copy: letter rail + search (keep), Examples shows cards, Use applies to the
  deck/filter you came from. Drop "tap the row."

### `OracleTagExamples` → overlay
Refactor the just-built route screen (`oracle_tag_examples.rs`) into an overlay,
modeled on `SwipeSelect`:
- Props: `open: Signal<bool>`, `slug: String` (or `Signal<Option<String>>` set by
  the dictionary's Examples button), `on_close: EventHandler<()>`.
- Guard on `if open()`; `use_overlay_back(open)`. Keep everything else built:
  SwipeStack, left/down only, EDHREC sort, stack cap, ensure_fresh toast, eyeball.
- Opened from the dictionary overlay → stacks on top (z 230).
- **Reuses all the load/gesture logic already written** — only the mount model
  (route → `open`-gated overlay) and the `slug` source change.

## Host wiring (who passes `on_use`)

The dictionary is embedded once per host that can open it; each supplies its own
`on_use`, which writes the **live** signal and toasts:

| Host | Opens dictionary from | `on_use(slug)` does |
|------|----------------------|---------------------|
| **Deck strategy selector** (`OracleTagSelect`) | its existing "Dictionary" button (`:125`) → now toggles an `open` signal, not `navigator.push` | push slug into the live `selected: Signal<Vec<String>>` (respect `MAX_DECK_ORACLE_TAGS`); toast "Tag added to deck" |
| **Card filter — include** (`filter/oracle_tags.rs`) | new "Dictionary" control in the include section | `write_selected` add slug to include (current any/all mode); toast "Tag added to filter" |
| **Card filter — exclude** | new "Dictionary" control in the exclude section | `write_excluded` add slug; toast "Tag added to filter" |
| Pure reference (none) | n/a | pass `on_use: None` → **Use** button hidden |

Because the host stays mounted, these are plain live-signal writes — **no
FilterStore juggling, no inbox, no return-intent enum.** The filter case doesn't
even need `(scope, deck_id)` anymore.

Examples is embedded similarly (host holds `examples_open: Signal<bool>` +
`examples_slug`); the dictionary's `on_examples(slug)` sets them.

## Route removal

- Delete `OracleTagExamples` + `OracleTagDictionary` from `router.rs` and their
  imports; drop `navigator.push(Router::OracleTagDictionary{})` call sites
  (`oracle_tag_select.rs:125,233`) in favor of toggling the embedded overlay's
  `open`.
- `screens/mod.rs`: the two modules stay (now overlay components), just no longer
  route targets. Move them if a components/ location fits better (optional).

## Migration note

The route-based `oracle_tag_examples.rs` + the dictionary-row `onclick` +
`dict-row-tappable` CSS were built earlier this session; this converts them to
overlays. Net-new vs. rework: the browse/dictionary *logic* is reused; the mount
model and the row affordance change.

## Verify

- [ ] Deck: form → selector → **Dictionary** overlay → **Use** → live deck tag
      added, toast, still on dictionary; **Back**/edge-swipe closes dictionary →
      selector (form state intact throughout).
- [ ] Filter include/exclude → Dictionary → Use writes the correct list; edge-swipe
      steps back one overlay at a time, never blows past to another screen.
- [ ] Examples overlay opens over the dictionary; left/down gestures; edge-swipe /
      Back returns to the dictionary, not out of the host.
- [ ] At the bottom of the overlay stack, edge-swipe finally does `nav.go_back()`.
- [ ] Pure-reference dictionary open (no `on_use`) → Use hidden.
- [ ] Retrofitted `SwipeSelect`/`OracleTagSelect`: edge-swipe now closes them too.
- [ ] Z-order correct at every depth; no click-through to lower layers.
- [ ] fmt + clippy clean.

## Open decisions (owner)

1. **Retrofit existing overlays** (`SwipeSelect`, `OracleTagSelect`, filter sheet)
   onto `use_overlay_back` now, or only wire the new ones and retrofit later?
   (Recommend now — small, and consistent edge-swipe behavior.)
2. **Button label** "Swipe" vs "Swipe select" on the deck-field launcher (separate,
   already discussed — leaning "Swipe").
3. Keep `oracle_tag_examples.rs` / `oracle_tag_dictionary.rs` under `screens/`, or
   move to a `components/` home now that they're overlays? (Cosmetic.)
