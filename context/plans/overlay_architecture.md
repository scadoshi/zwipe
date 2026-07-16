# Overlay architecture — back-aware overlays + route audit

**Status: PLANNED (2026-07-15). Umbrella / infrastructure plan.** Client-only
(zwiper). Establishes a reusable **overlay** primitive that participates in the OS
back gesture, the **route-vs-overlay principle**, and an **audit** of every current
route against it. The oracle-tag dictionary/examples conversion
([`dictionary_adopt_flow.md`](dictionary_adopt_flow.md)) is the first consumer and
proving ground.

**One sentence:** the OS back handler is *our* code, so we make it overlay-aware —
`back` closes the top overlay before touching the router — which frees us to build
"layers over a screen that preserve its state" as overlays instead of routes, and
this plan defines the primitive, the principle, and which existing pages should
switch.

---

## The principle (route vs overlay)

Decide per surface:

| Use a **route** when… | Use an **overlay** when… |
|-----------------------|--------------------------|
| It's a distinct **destination** that replaces the current context | It's a **layer on top of** a specific host screen |
| You'd want it in history as its own place | Losing the host's live state on return would be bad |
| It doesn't need the previous screen's state kept | It's a modal action, picker, or reference *about* the host |
| Hubs & flows: Home, DeckList, ViewDeck, auth | Pickers, modals, reference docs, sub-flows |

The old reflex ("everything is a page") came from the back gesture only knowing how
to pop routes. Once back can also close an overlay, that constraint is gone. Routes
still exist for real destinations; overlays cover everything that should sit *on
top* without destroying what's underneath.

## Part 1 — Infrastructure (build first, reusable forever)

### 1a. `OverlayBackStack` (app-level)
Provided in `spawn_upkeeper` (`session_upkeep.rs`), above the router:

```rust
/// LIFO of open overlays. Each entry knows its component ScopeId (stable per
/// instance) and holds the overlay's `open` signal so the back gesture can close
/// the top one. Copy handle over a signal.
#[derive(Clone, Copy)]
pub struct OverlayBackStack {
    entries: Signal<Vec<(ScopeId, Signal<bool>)>>,
}

impl OverlayBackStack {
    pub fn push(&mut self, id: ScopeId, open: Signal<bool>) {
        let mut v = self.entries.write();
        v.retain(|(i, _)| *i != id);      // idempotent
        v.push((id, open));
    }
    pub fn remove(&mut self, id: ScopeId) {
        self.entries.write().retain(|(i, _)| *i != id);
    }
    /// The open-signal of the top overlay, if any (peeked — no subscription).
    pub fn top(&self) -> Option<Signal<bool>> {
        self.entries.peek().last().map(|(_, open)| *open)
    }
    pub fn is_empty(&self) -> bool { self.entries.peek().is_empty() }
}
```

### 1b. `use_overlay_back(open)` hook
Every overlay calls this once. It keeps the stack in sync with the overlay's
`open`, regardless of *how* it closes (own button, programmatic, unmount):

```rust
pub fn use_overlay_back(open: Signal<bool>) {
    let mut stack: OverlayBackStack = use_context();
    let id = current_scope_id().expect("overlay needs a scope");
    use_effect(move || {
        if open() { stack.push(id, open); } else { stack.remove(id); }
    });
    use_drop(move || stack.remove(id));   // safety if unmounted while open
}
```

### 1c. Back handler consults the stack
`back_handler.rs`, both the iOS and Android arms, change identically:

```rust
// was: if nav.can_go_back() { nav.go_back(); }
if let Some(mut open) = stack.top() {
    open.set(false);          // close top overlay; its use_effect pops it
} else if nav.can_go_back() {
    nav.go_back();
} else {
    // Android only: finish_activity()
}
```

`BackHandlerLayout` reads `OverlayBackStack` from context (it's above the router,
so available). No change to the native iOS/Android bridge itself — only what the
drained intent *does*.

**Gesture-conflict check (already safe):** the iOS recognizer is
`UIScreenEdgePanGestureRecognizer` with `setEdges: EDGE_LEFT` — it fires **only from
the left screen edge**, so mid-screen `SwipeStack` card pans don't trigger it. A
left-edge swipe inside an overlay closes that overlay (via the stack); a card swipe
mid-screen still swipes the card. They compose.

### 1d. Overlay component convention (document once)
A back-aware overlay is:
- Props include `open: Signal<bool>` (+ `on_close` if the host needs to react).
- Body wrapped in `if open() { … }` so it mounts nothing when closed (zero cost).
- Calls `use_overlay_back(open)`.
- Full-screen `position: fixed` opaque layer (covers the host — no click-through /
  scroll-through), shown via a `.show` class gated on `open()` (mirror
  `.swipe-select-screen`).
- Assigned a z-index from the registry below.
- In-UI Back/Cancel/Done buttons set `open = false` (the stack stays truthful).

### 1e. Z-index registry (define + centralize)
Today: modal-backdrop 200, bottom-sheet 201, filter-sheet ~202,
`.swipe-select-screen` 210. Establish an overlay band and document it in one CSS
comment block so depths never collide:

| Layer | z-index |
|-------|--------|
| base screen | (default) |
| modal backdrop / bottom sheet / filter sheet | 200–202 |
| swipe-select / oracle-tag-select | 210 |
| **dictionary overlay** | 220 |
| **examples overlay** | 230 |
| toasts | (top, unchanged) |

Leave gaps (10s) for future insertion.

## Part 2 — Retrofit existing overlays

These already exist as `open`-gated overlays but only close via their own buttons.
Add `use_overlay_back(open)` to each so the back gesture closes them (fixes the
current rough edge where edge-swipe blows past them and exits the whole screen):

- `OracleTagSelect` (`deck/components/oracle_tag_select.rs`)
- `SwipeSelect` (`deck/components/swipe_select.rs`)
- `CardFilterSheet` bottom sheet (`filter/card_filter_sheet.rs`) and other
  `bottom_sheet.rs` consumers (each takes an `open` signal already).

Small, mechanical, high consistency payoff. Do alongside the infra.

## Part 3 — Route audit (the "pages that break")

Verdicts from the nav graph. **KEEP** = legitimate route; **→OVERLAY** =
convert; **?OWNER** = judgment call, needs a decision.

| Route | Entered from | Verdict | Reasoning |
|-------|-------------|---------|-----------|
| `Login` / `Register` / `ForgotPassword` | auth flow | **KEEP** | Pre-auth destinations; distinct history. |
| `Home` | root / post-login | **KEEP** | Hub. |
| `DeckList` | Home | **KEEP** | Hub. |
| `ViewDeck` | list, and returned-to from edit/import/export/create | **KEEP** | The per-deck hub; everything returns here. |
| `Profile` | Home | **KEEP** | Settled destination. |
| `OracleTagDictionary` | deck oracle-tag selector | **→OVERLAY** | Reference layer over a picker; Use must write the host's live signal. First conversion — see [`dictionary_adopt_flow.md`](dictionary_adopt_flow.md). |
| `OracleTagExamples` | dictionary | **→OVERLAY** | Peek layer over the dictionary; `SwipeSelect` proves a swipe screen works as an overlay. |
| `ExportDeck` | deck view (more/warnings) | **→OVERLAY** (strong) | It's "show me this deck's decklist text" — a modal over the deck. No reason to leave ViewDeck. |
| `ImportDeck` | deck view (more/warnings) | **→OVERLAY** (likely) | Paste-a-decklist modal action on this deck; returning should land back on the same deck view untouched. |
| `ViewDeckCard` | deck view | **?OWNER** | "Browse this deck's cards" — a layer over ViewDeck. Overlay would preserve ViewDeck scroll/state; but it's a substantial screen. Lean overlay. |
| `EditDeck` | deck view | **?OWNER** | A big form editing *this* deck. Overlay would preserve ViewDeck underneath and stop the current "lose edits if you open the dictionary" problem — but it's the heaviest conversion. Ties into whether we want deck-form state preserved (see dictionary deck-Use discussion). |
| `CreateDeck` | deck list | **?OWNER** | Creation form → ViewDeck on done. Overlay over DeckList is plausible; creation is also defensibly its own destination. Lower priority. |
| `AddDeckCard` / `RemoveDeckCard` | deck view / warnings | **?OWNER / defer** | Big swipe screens; their filter state is already persisted via `FilterStore`, so they don't "break" today. Convert only if we want them layered over ViewDeck. Low urgency. |
| `PrivacyPolicy` / `Changelog` | Profile | **→OVERLAY** (minor) | Read-only reference docs over Profile; overlay preserves profile state and matches the dictionary treatment. Low priority, easy. |

**Headline "breakers"** (lose host state or are really modals): **Export, Import,
Dictionary, Examples**, and secondarily **ViewDeckCard / EditDeck**. Those are where
the route model actively hurts.

## Part 4 — Sequencing

1. **Infra (Part 1):** `OverlayBackStack` + `use_overlay_back` + back-handler change
   + z-index registry. Nothing else can land without this.
2. **Retrofit (Part 2):** wire the three existing overlays. Immediately fixes
   edge-swipe behavior; validates the primitive on known-good components.
3. **Dictionary + Examples (→OVERLAY):** the first real conversion — full spec in
   [`dictionary_adopt_flow.md`](dictionary_adopt_flow.md). Proves swipe-screen-as-
   overlay end to end.
4. **Export, then Import (→OVERLAY):** clear wins, self-contained modals over deck
   view.
5. **?OWNER batch:** decide ViewDeckCard / EditDeck / CreateDeck / Add / Remove /
   Privacy / Changelog per the audit; convert the approved ones.

Each step is independently shippable and testable.

## Part 5 — Risks / correctness

- **Back-stack integrity.** Closing via an in-UI button must flip `open` so the
  effect pops the entry — never mutate the stack by hand elsewhere. `use_drop`
  guards the unmount-while-open case. Test: open A→B→C, close B via its button
  (shouldn't happen in strict LIFO, but) → stack stays consistent by ScopeId.
- **Opaque layers / no click-through.** Every overlay must fully cover and block
  the host (fixed, opaque, own scroll). Verify no taps/scrolls reach lower layers.
- **Deep stacks.** Test 3–4 deep (form → selector → dictionary → examples): each
  back step closes exactly one layer; the last back finally `go_back()`s the route.
- **Android.** Back-stack applies to the `zwipe:back` funnel too; inert until the
  Android native patch ships, then works for free.
- **Zero-cost when closed.** `if open()` guard means closed overlays mount nothing —
  embedding several per host is fine.
- **Scroll/focus restoration.** Overlays preserve host state by construction (host
  never unmounts) — a win over routes, but verify input focus/scroll aren't stolen.

## Verify (infra-level)

- [ ] `OverlayBackStack` provided app-wide; `use_overlay_back` pushes on open, pops
      on close and on unmount.
- [ ] iOS edge-swipe: with an overlay open, closes the top overlay; with none open,
      navigates the route back; at a root screen, no-ops.
- [ ] Retrofitted `SwipeSelect`/`OracleTagSelect`/filter sheet close on edge-swipe.
- [ ] Left-edge back vs mid-screen card swipe don't interfere.
- [ ] Z-order correct at every depth; no click/scroll-through.
- [ ] fmt + clippy clean.

## Open decisions (owner)

1. **?OWNER audit rows:** which of ViewDeckCard / EditDeck / CreateDeck / Add /
   Remove / Privacy / Changelog to convert (and priority)?
2. Retrofit existing overlays now (recommended) or with their next touch?
3. Where do converted overlay components live — keep under `screens/`, or a
   `components/overlays/` home? (Cosmetic.)
