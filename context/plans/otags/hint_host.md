# Hint host — app-root sender/receiver for on-demand help

**Status: PLAN (2026-07-12).** A platform pattern for on-demand "?" help, born
from the otag `InfoButton` work (`user_education.md` §7 Ticket 2). It supersedes
that ticket's inline-dialog approach, which had a fatal flaw: an `InfoButton`
that renders its own `HintDialog` inline gets **trapped by any ancestor that
creates a containing block** (a `transform`/`filter`/`will-change`), so the
`position: fixed` dialog is clipped to the content column instead of the screen.

## The insight (owner, 2026-07-12)

Every screen needs **one** hint slot at the root; a **signal** flows down via
context, and any child can **post** to it. One receiver renders the alert; "Got
it" clears it. A sender/receiver pattern. The existing one-time hints already
work precisely because they render at `div.screen` root (outside
`.screen-content`) — this generalizes that, once, at the **app root**.

## Why app-root

`zwipe.rs::App` → `spawn_upkeeper()` (context hub) → `ThemeWrapper` >
`ToastProvider` > `Router::<Router> {}`. A host rendered **beside `Router`** sits
above every screen, outside `.screen-content` (the scroll + `content-enter`
animated container) and every other per-screen wrapper — so its `position: fixed`
dialog can never be trapped. One host covers the whole app; **zero per-screen
wiring**.

## Design

- **Message:** `#[derive(Clone, Copy, PartialEq)] enum HintTopic { DeckTags,
  OracleTags, CardRoles }` — a cheap `Copy` value, NOT an `Element` (Dioxus
  can't safely hold `Element`s in a signal). Extensible: new hint = one variant.
- **Channel:** `Signal<Option<HintTopic>>` in context. `Some(t)` = open with topic
  `t`; `None` = closed. A single `Option` = **one hint at a time by construction**
  (serves the anti-fatigue goal).
- **Receiver:** one `HintHost` at the app root; when `Some(topic)`, renders the
  full-screen dialog (title + explainer + "Got it"); on close, clears to `None`.
- **Sender:** `InfoButton { topic }` — just the inline `"?"`; on click it sets the
  context signal. **It renders no dialog**, so nothing can trap it.

Keep the `content-enter` `transform: none` fix (a genuine latent bug), but the
host is the actual cure.

---

## Execution spec (build tickets)

Verify after: `cargo +nightly fmt` then
`cargo clippy -p zwiper -p zwipe-components --all-targets -- -D warnings`.
Copy rules unchanged (sentence case, no em dashes, lead with benefit).

### Ticket 1 — `HintTopic` + `HintHost`
**New file:** `zwiper/src/lib/inbound/components/hint_host.rs`.
- `pub enum HintTopic { DeckTags, OracleTags, CardRoles }` (+ `Clone, Copy,
  PartialEq`). Give it `fn title(&self) -> &'static str` returning "Deck tags" /
  "Oracle tags" / "Card roles".
- `#[component] pub fn HintHost()`:
  ```rust
  let mut hint: Signal<Option<HintTopic>> = use_context();
  rsx! {
      if let Some(topic) = hint() {
          // Render the AlertDialog shell DIRECTLY (do not reuse HintDialog's
          // Signal<bool> API — bridging two signals invites a feedback loop).
          // Model the markup on hint_dialog.rs::HintDialog: AlertDialogRoot {
          //   open: true, on_open_change: move |o| if !o { hint.set(None) },
          //   AlertDialogContent { AlertDialogTitle{topic.title()} hr.dialog-rule
          //     AlertDialogDescription { match topic {
          //        HintTopic::DeckTags   => rsx!{ DeckTagsExplainer {} },
          //        HintTopic::OracleTags => rsx!{ OracleTagsExplainer {} },
          //        HintTopic::CardRoles  => rsx!{ CardRolesExplainer {} },
          //     } }
          //     hr.dialog-rule
          //     AlertDialogActions { AlertDialogAction { on_click: move |_| hint.set(None), "Got it" } } } }
      }
  }
  ```
  Explainers come from `concept_explainers.rs` (already built). AlertDialog\* +
  `HintDialog` live in `components/alert_dialog` / `components/hint_dialog`.
- Register `hint_host` in `components/mod.rs`.

### Ticket 2 — provide the channel + mount the host (app root)
- **`components/auth/session_upkeep.rs` `spawn_upkeeper()`** — beside the other
  `use_context_provider` calls, add:
  ```rust
  let hint_topic: Signal<Option<HintTopic>> = use_signal(|| None);
  use_context_provider(|| hint_topic);
  ```
- **`zwiper/src/bin/zwipe.rs` `App`** — render `HintHost {}` as a **sibling of
  `Router::<Router> {}`** inside `ToastProvider`:
  ```rust
  ToastProvider { max_toasts: 3usize, class: "toast-container",
      Router::<Router> {}
      HintHost {}
  }
  ```
  (Import `HintHost` from `zwiper::inbound::components::hint_host`.)

### Ticket 3 — `InfoButton` becomes a pure sender
**Edit `components/info_button.rs`:**
```rust
#[component]
pub fn InfoButton(topic: HintTopic) -> Element {
    let mut hint: Signal<Option<HintTopic>> = use_context();
    rsx! {
        button {
            class: "info-button",
            r#type: "button",
            onclick: move |evt| { evt.stop_propagation(); hint.set(Some(topic)); },
            "?"
        }
    }
}
```
Drop the `title`/`children` props and the inline `HintDialog` (+ its `use_signal`).
Keep the `.info-button` CSS and `stop_propagation` (still needed for the filter
accordion).

### Ticket 4 — repoint the 4 call sites
Change each `InfoButton { title: "...", XExplainer {} }` to `InfoButton { topic:
HintTopic::X }`:
- `zwipe-components/src/card_role_chips.rs` consumer — the zwiper side that fills
  the `help` slot (`card_info.rs`): `InfoButton { topic: HintTopic::CardRoles }`.
- `screens/deck/components/deck_fields.rs` — Deck tags label →
  `HintTopic::DeckTags`; Oracle tags label → `HintTopic::OracleTags`.
- `screens/deck/components/deck_tags_section.rs` — same two topics.
- `screens/deck/card/filter/card_filter_sheet.rs` — Oracle tags section →
  `HintTopic::OracleTags`.

The `CardRoleChips.help: Option<Element>` slot in `zwipe-components` **stays**
(still additive, portal-safe) — it now receives the sender-only `InfoButton`.

### Acceptance
- The "?" opens a **full-screen** dialog on the edit, create, deck-view, and
  card screens (no clipping to the content column).
- Only one hint dialog can be open at a time.
- `clippy -p zwiper -p zwipe-components -D warnings` + nightly fmt clean.
- Portfolio (`zwipe-components` consumer) still builds — `card_role_chips.rs`
  change is unchanged from Agent A (additive `help` slot).

### Notes for the executor
- This is a shared tree with other AI sessions. Stage only your own files; never
  tree-wide git ops; do not push.
- Does not touch the one-time hint system (`use_one_time_hint`) — those already
  render at screen root and keep working. This host is the on-demand channel; a
  future cleanup could route one-time hints through it too, but that's out of scope.
