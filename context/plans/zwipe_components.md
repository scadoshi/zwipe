# zwipe-components — a shared UI component crate

**Status: v1 SHIPPED 2026-07-07 (`244dd83a` crate + button migration,
`06ac48da` workspace import regroup). The crate is a live workspace member;
`Button` (Primary/Small/Util + danger/disabled/class/style), `Chip`, and
`ActionBar` are authored; `components.css` is copied into both apps by
`build.rs` and linked; ~50 zwiper buttons and ~14 action bars migrated; zite's
shared-deck filter chips use the shared `Chip`. Next: continue moving in the
components that make sense, incrementally — the roadmap below.**

**What this builds, in one sentence:** a new workspace crate `zwipe-components`
that both `zwiper` (the app) and `zite` (the site) depend on for shared Dioxus
components — `Button`, `Chip`, `ActionBar` — plus a `components.css` each app
copies into its own bundle, so buttons and bars look and behave identically
across surfaces instead of drifting between hand-written `button { class: … }`.

**Why now:** the raw counts show the drift risk — `class="util-btn"` appears 58
times, `class="btn"` 16, `btn-xs` 5, plus `-danger`/`-eye`/`-clear`/page-header
combos, and 21 hand-rolled `div class="util-bar"` action rows across ~30 files.
Every one is a copy-pasted class string a typo away from wrong. The pattern is
already proven twice: `Chip` (zwiper's `components/chip.rs`) wraps `.chip` as a
component, and `shared/themes.css` is copied into both apps at build time. This
just applies both patterns to the button system and lifts them into a crate.

**Why a new crate, not `zwipe-core` (owner call 2026-07-07):** core is pure
domain — no Dioxus, no UI deps (see [`feedback_zwipe_core_philosophy`]). UI
components are not core enough. A dedicated `zwipe-components` keeps core pure
and gives a home a future `portfolio` site can also pull from.

---

## Architecture

New workspace member `zwipe-components`. Dependency graph:

```
zwiper ──→ zwipe-components ──→ dioxus (base, no platform features)
zite   ──→ zwipe-components ──→ zwipe-core
zwiper ──→ zwipe-core        ←── zite
```

- **Dioxus dep is base only** — `dioxus = { version = "0.7.9", default-features
  = false }`. NO `web`/`mobile`/`fullstack`/`server` feature: each consuming app
  enables its own platform. Components need only rsx/signals/`EventHandler`,
  which live in the base crate. This is the single biggest technical care point
  — a stray platform feature here would force it on every consumer.
- Both apps already pin dioxus 0.7.9, so versions align today; the crate must
  track that pin.
- Depends on `zwipe-core` for any domain types a future component needs (Chip
  needs none; a `CardRow`/`ManaCost` port later would).

## CSS strategy (mirrors `shared/themes.css`)

Components render class names; the class *rules* must ship too or the crate is
half a component. Follow the theme precedent exactly:

- Crate ships `zwipe-components/assets/components.css` with the shared rules
  (`.btn`, `.btn-xs`, `.util-btn` + variants, `.chip`, `.action-bar`),
  extracted from `zwiper/assets/main.css` and `zite/assets/style.css`.
- Each app's `build.rs` copies it into `assets/` alongside the themes copy, and
  the app links it with `document::Stylesheet`.
- Rules reference theme CSS variables (`--accent-primary`, `--bg-primary`, …)
  which both apps already define, so one component resolves to whichever theme
  the host app has active. No color literals in `components.css`.
- The extracted rules leave their app-specific homes; app CSS keeps only what
  isn't shared. Watch for class-name collisions during extraction (zite and
  zwiper both define `.util-btn` today — reconcile to one rule).

## Components (v1)

- **`Button { variant, danger, disabled, class, style, onclick, children }`**
  - `variant: ButtonVariant` → `Primary` (`.btn`), `Small` (`.btn-xs`),
    `Util` (`.util-btn`). `danger` picks the right variant's danger class
    (`.btn-danger` vs `.util-btn-danger`). `class` appends extras
    (`util-btn-eye`, `util-btn-clear`, `page-header-*`); `style` covers the
    handful of inline-styled sites. Explicit props, matching the codebase idiom
    (no attribute-spread pattern is used anywhere today).
  - Out: the few `<a class="btn">` links (they're anchors, not buttons) — leave
    raw, or add an `href` variant later if it's worth it.
- **`Chip { selected, onclick, children }`** — moved verbatim from
  `zwiper/components/chip.rs`; zite gets it for free. Selected state stays
  accent-2 (`.chip.selected`).
- **`ActionBar { children }`** — wraps the `div class="util-bar"` footer row.
  Keep the class name `util-bar` internally (a standalone rename is 21 divs of
  churn for no gain); the *component* carries the good name. This lands the
  "FooterActions/ActionBar" idea the owner raised without a class-rename pass.

**Later (explicitly out of v1):** the alert-dialog action buttons
(`.alert-dialog-action`/`-cancel`/`-danger`) are already wrapped via
`dioxus-primitives` in `components/alert_dialog/` — a separate surface; fold
them into `Button` only if it's clean, not as v1 scope.

## Migration surface

~30 files. By class: `.util-btn` ×58, `.btn` ×16, `.btn-xs` ×5,
`.btn btn-danger` ×3, `.util-btn util-btn-*` ×6, `.util-bar` divs ×21. Heaviest
files: `profile/mod.rs` (8 util-btn), `deck/card/view.rs` (5), `deck/view.rs`
(4). Every screen area is touched (auth, profile, deck, legal, home), so most of
the risk is *volume*, not difficulty — it's mechanical:
`button { class: "util-btn", onclick: X, "L" }` → `Button { variant: Util,
onclick: X, "L" }`.

## Phases (v1 — DONE)

1. ✅ **Scaffold + prove the pipeline.** Crate is a workspace member; `Button` +
   `Chip` + `ActionBar` + `components.css` authored; both apps wired via
   `build.rs` + stylesheet link; `Chip` proven in both apps.
2. ✅ **Migrate zwiper.** ~50 `.btn`/`.util-btn` sites → `Button`, ~14
   `.util-bar` rows → `ActionBar`; only intentional raw markup left (`<a
   class="btn">` links, the aria `!` help button, out-of-scope classes).
3. ✅ **Migrate zite.** Shared-deck filter chips use the shared `Chip`; the
   home hero labels renamed off `.chip` to free it for the shared component.
   (Import grouping was landed workspace-wide in the same effort.)

## Continuing incrementally — component roadmap

**Guiding principle: move what makes sense, one component per pass.** A
component earns a move when it is *pure presentation* (depends only on Dioxus +
`zwipe-core` types) and either a second surface already needs it (zite, the
portfolio, a future web Zwipe) or it's obviously reusable UI. App-logic
components stay in `zwiper`. Mixed ones **split**: the dumb presentational shell
moves into the crate; the smart wiring (session, API calls, hooks) stays in the
app and is passed in as props/callbacks — "dumb components in the crate, smart
wiring in the app." Every move is its own small, verifiable change — lift the
component **and its CSS** into `components.css`, migrate call sites, confirm both
apps still render. Never a big-bang, and prefer moving a component when a real
second consumer appears over doing it speculatively.

**Move in (pure presentation — lift when a second consumer wants them):**
- `fields/` (`TextInput`, password field) — forms look the same everywhere.
- `accordion/` — generic disclosure.
- `tri_toggle.rs` — generic three-state toggle.
- `screen_header.rs` — title + optional hint trigger (the hint *hook* stays in
  the app; pass an `on_hint` callback + the open flag in).
- `bottom_sheet.rs` — generic slide-up overlay.
- `interactions/` (`swipe`, `carousel`) — gesture/UI mechanics, no app deps.
- `alert_dialog/`, `toast/` — shareable, but moving them pulls
  `dioxus-primitives` into the crate. Fine; just note the crate grows that git
  dep the day they move.

**Keep in `zwiper` (app logic, not UI — a web build wires these itself):**
- `auth/` (`bouncer`, `ensure_session`) — session handling.
- `telemetry/` (`usage_buffer`, flush loop) — talks to the API.
- `logout_dialog.rs`, `update_required.rs` — logout call; min-version gate +
  store links.

**Split (mixed — the useful pattern):**
- `hint_dialog.rs` — the dialog shell moves; `use_one_time_hint` +
  the `mark-hint-shown` API call stay in the app, passed in.
- `support.rs` — the sheet UI moves; the support email/links + open-url
  plumbing stay.

**Domain-shaped components (later, high value):**
- ✅ **`OracleText` + `KeywordChips` — moved 2026-07-08.** Both were verbatim
  duplicates between the app and zite's shared-deck page; now single copies in
  the crate, with `.oracle-sym` + `.keyword-*` rules lifted into
  `components.css`. Zite keeps its smaller text as `.shared-deck`-scoped
  font-size overrides. The `.ms-cost` sizing/shadow combo rules stayed in the
  apps (they belong to the `card-detail-*` family below).
- `CardRow` / `ManaCost` — zite's shared-deck page still re-implements
  `CardRow` (see `zite/src/pages/shared_deck.rs`). The app's version is already
  callback-shaped (`on_qty_change`/`on_printing`/`on_toggle_mvp`/`on_move_to`
  are `Option` props), so the split is close: unify on optional actions +
  zite's hover-preview/Image extras, and lift the duplicated
  `card-row-*`/`card-detail-*` CSS family with it. The 2026-07-08 polish pass
  had to make every edit twice — that's the payoff case.

**End goal:** a web Zwipe (zwiper already has a `web` feature), the portfolio,
and zite all render the same components — one product across many surfaces. Grow
the crate toward that as real second-consumers appear.

## Risks / notes

- **Concurrency.** This is a tree-wide, ~30-file sweep — high collision risk
  with any other session on `main` (see [`feedback_concurrent_ai_dont_disrupt`]).
  Start it only when the tree is otherwise clean; do it as its own focused
  branch/commit, never folded into unrelated work.
- **CSS extraction is the sharp edge**, not the Rust. Moving rules out of two
  large stylesheets risks specificity/ordering regressions and the `.util-btn`
  double-definition. Extract conservatively; diff render before/after.
- **Verification is manual and broad** — there's no snapshot test; both a full
  simulator pass and a full zite pass are the gate. Budget for it.

## Effort

v1 (crate + button/chip/bar migration) is **done**. Going forward the work is
**incremental and small per component** — each move is one component + its CSS +
its call sites + a quick visual check, sized S/M, landed as its own commit. No
more big cross-cutting sweeps; pick a candidate from the roadmap when a second
consumer makes it worth it.

Related: [`../development/`] coding standards; `Chip` (`zwipe-components/src/chip.rs`)
and `shared/themes.css` are the two precedents this generalizes.
