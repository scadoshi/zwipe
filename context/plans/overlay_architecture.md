# Overlay architecture — back-aware overlays + route audit

**Status: Parts 1 + 2 + the dictionary/examples conversion SHIPPED 2026-07-16
(commit `dc44ce3f`), device-tested working. The remaining route conversions from the
Part 3 audit (Export, Import, ViewDeckCard, EditDeck, Privacy, Changelog) are optional
future work — none is required for the primitive to be complete.** Umbrella /
infrastructure plan. Client-only (zwiper). Establishes a reusable **overlay** primitive
that participates in the OS back gesture, the **route-vs-overlay principle**, and an
**audit** of every current route against it. The oracle-tag dictionary/examples
conversion ([`dictionary_adopt_flow.md`](dictionary_adopt_flow.md)) was the first
consumer and proving ground, and is done.

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

## As-built (2026-07-16, commit `dc44ce3f`) — source of truth

The shipped API diverges from the Part 1 sketch below in one deliberate way: the
stack stores a **close `Callback`**, not the overlay's `open` signal. That lets both
signal-toggled overlays *and* callback-closed ones (the shared `AlertDialogRoot`
wrapper, which closes through `on_open_change`) live on one stack. The Part 1 code
blocks are the original design sketch; this section is authoritative.

- **`overlay_stack.rs`** (`zwiper/.../components/navigation/`) exports:
  - `OverlayBackStack { entries: Signal<Vec<(u64, Callback<()>)>> }` — a `Copy`
    handle. Entries pair a stable per-instance id with a close callback.
  - `use_overlay_back_stack()` — creates it; provided in `spawn_upkeeper`
    (`session_upkeep.rs`) above the router via `use_context_provider`.
  - `use_overlay_back(open: Signal<bool>)` — for signal-toggled overlays; builds a
    close callback that sets `open = false`.
  - `use_overlay_back_action(is_open: ReadSignal<bool>, on_close: Callback<()>)` —
    for overlays that close through a callback (the `AlertDialogRoot` wrapper).
  - `close_top() -> bool` — pops the top entry and invokes its close callback;
    returns whether one was closed.
- **Instance id** comes from a process-wide `AtomicU64` counter grabbed once via
  `use_hook` (Dioxus `current_scope_id` isn't exposed in this version). Registration
  is a `use_effect` (push while open, remove while closed) plus a `use_drop` guard
  for unmount-while-open.
- **Back handler** (`back_handler.rs`, iOS + Android arms):
  `if !overlays.close_top() && nav.can_go_back() { nav.go_back(); }` (Android falls
  through to `finish_activity()` at a root screen).
- **Retrofitted** `SwipeSelect`, `OracleTagSelect`, `CardFilterSheet`, **and the
  shared `AlertDialogRoot` wrapper** — so every dialog built on it (hints, card
  details, confirms) closes on back with no per-call wiring.
- **Dictionary + Examples converted to overlays** (Part 3, done): the dictionary
  embeds the examples browse as a nested overlay; the dictionary itself is embedded
  in the deck oracle-tag picker (opened from the hint's action bar) and in the card
  filter (a "Dictionary" button beside each include/exclude label). The filter's copy
  is rendered as a **sibling of the bottom sheet**, outside its `transform`, because a
  `position: fixed` overlay nested inside a transformed ancestor is clipped to that
  ancestor's box. Adopt flow (Examples / Use, "Added to filter" / "Tag added to
  deck"): [`dictionary_adopt_flow.md`](dictionary_adopt_flow.md).
- **Z-index registry** documented in `main.css` (sheets 200–202, swipe-select 210,
  dictionary 220, examples 230).
- fmt + clippy clean. **Device-tested working**: edge-swipe closes the top overlay
  (not the whole screen), nested overlays close top-first, a screen with no overlay
  still navigates back, and the hint / dictionary / examples stack pops one layer per
  back step.

## Part 1 — Infrastructure (original design sketch; see As-built for shipped API)

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
| `OracleTagDictionary` | deck oracle-tag selector + card filter | **DONE (→OVERLAY)** | Converted `dc44ce3f`. Reference layer over a picker; Use writes the host's live signal ("Tag added to deck" / "Added to filter"). Route dropped. See [`dictionary_adopt_flow.md`](dictionary_adopt_flow.md). |
| `OracleTagExamples` | dictionary | **DONE (→OVERLAY)** | Converted `dc44ce3f`. Nested swipe-screen overlay over the dictionary; route dropped. Proves a swipe screen works as an overlay. |
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

1. ~~**Infra (Part 1):** `OverlayBackStack` + `use_overlay_back` + back-handler
   change + z-index registry.~~ **DONE `dc44ce3f`.**
2. ~~**Retrofit (Part 2):** wire the existing overlays.~~ **DONE `dc44ce3f`** — plus
   the shared `AlertDialogRoot` wrapper, so all dialogs close on back too.
3. ~~**Dictionary + Examples (→OVERLAY):** the first real conversion.~~ **DONE
   `dc44ce3f`** — spec in [`dictionary_adopt_flow.md`](dictionary_adopt_flow.md);
   swipe-screen-as-overlay proven end to end.
4. **Export, then Import (→OVERLAY):** clear wins, self-contained modals over deck
   view. *Not started — optional.*
5. **?OWNER batch:** decide ViewDeckCard / EditDeck / CreateDeck / Add / Remove /
   Privacy / Changelog per the audit; convert the approved ones. *Not started —
   optional.*

Steps 1–3 are shipped. Steps 4–5 are independently shippable if/when wanted.

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

## Verify (infra-level) — all confirmed 2026-07-16

- [x] `OverlayBackStack` provided app-wide; registration pushes on open, pops on
      close and on unmount.
- [x] iOS edge-swipe: with an overlay open, closes the top overlay; with none open,
      navigates the route back; at a root screen, no-ops.
- [x] Retrofitted `SwipeSelect`/`OracleTagSelect`/filter sheet (and all
      `AlertDialog` dialogs) close on edge-swipe.
- [x] Left-edge back vs mid-screen card swipe don't interfere.
- [x] Z-order correct at every depth; no click/scroll-through (incl. the filter
      dictionary rendered outside the sheet transform).
- [x] fmt + clippy clean.

## Open decisions (owner)

1. **?OWNER audit rows (still open):** which of ViewDeckCard / EditDeck / CreateDeck
   / Add / Remove / Privacy / Changelog to convert (and priority)? None are urgent —
   nothing "breaks" today.
2. ~~Retrofit existing overlays now or with their next touch?~~ **Resolved — did it
   now (`dc44ce3f`).**
3. ~~Where do converted overlay components live?~~ **Resolved — kept under
   `screens/`** (dictionary/examples stayed put; no `components/overlays/` home).
