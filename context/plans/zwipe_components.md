# zwipe-components — a shared UI component crate

**Status: PLANNED (2026-07-07). Scaffold started (crate `Cargo.toml` +
`src/lib.rs` stubs exist, NOT yet a workspace member, so inert). No components
authored, no call sites migrated.**

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

## Phases

1. **Scaffold + prove the pipeline (low risk, additive).** Finish the crate,
   add it to workspace members, author `Button` + `Chip` + `ActionBar` +
   `components.css`, wire both apps' `build.rs` + stylesheet link. Prove it
   end-to-end by migrating **one** `Chip` site in each app and confirming both
   render. Checkpoint here.
2. **Migrate zwiper.** All `.btn`/`.util-btn`/`.util-bar` sites → components,
   compiling per screen area. Full **simulator pass** — this is a
   look-sensitive refactor; screens without obvious diffs (auth/profile/legal)
   still need eyes.
3. **Migrate zite.** Same, plus reconcile the duplicate `.util-btn` rule.
   Full **site pass** in the browser.
4. **Later:** dialog action buttons (if clean); a shared `portfolio` consumer;
   possibly `CardRow`/`ManaCost` (zite's shared-deck page already re-implements
   these from the app — prime candidates once the crate exists).

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

**M/L, cross-cutting.** The crate + CSS + two-app wiring is small; the ~90-site
migration + two full visual passes is the bulk. Its own task, its own commit.

Related: [`../development/`] coding standards; `Chip` (`zwiper/components/chip.rs`)
and `shared/themes.css` are the two precedents this generalizes.
