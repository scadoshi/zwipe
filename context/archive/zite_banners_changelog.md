# Announcement banners + changelog page

**Status: SHIPPED (owner-confirmed 2026-07-12; pieces built 2026-07-11).** Prompted by the Android release: a
dismissible "Zwipe is now on Android" banner on the site, plus a browsable
changelog so people can see every version's changes. Banners are modeled on the
portfolio's existing toast banners, promoted into `zwipe-components` so `zite`
and the portfolio share one implementation.

Shipped to the working tree: `zwipe-components/src/banner.rs` (`Banner` +
`BannerStatus`, exported from `lib.rs`), the Banner + `status-tag` CSS section in
`components.css`, the home-page Android banner, a `/changelog` route + page +
nav link, and changelog copy transcribed from the App Store "What's New" history
(em dashes recast for zite style; no dates recorded, none shown). `cargo check`
clean; the only clippy error is a pre-existing redundant clone in the
in-progress `card_row.rs`, left untouched.

## Decisions (locked)

- **Consumers:** `zite` (new) **and** the portfolio (refactor its inline banners
  onto the shared component). Not `zwiper` for now.
- **Dismissal persistence:** none. In-memory only, banners reappear on reload
  (matches the portfolio exactly). No localStorage.
- **Auto-dismiss:** keep the portfolio's 10s countdown bar (pauses on hover).
- **Changelog:** promoted to a shared `zwipe_components::Changelog` component
  (data + chip filter + list + CSS all self-contained in `zwipe-components`), so
  the website and the app render one source and never drift. `zwipe-core` stays
  pure (no release list in the domain crate); no build script. Consumed by the
  zite `/changelog` page and a zwiper `/changelog` **screen** reached from a
  "Changelog" button next to the version in Profile → About.

## Piece 1 — shared `Banner` component

Faithful port of the portfolio banner (`portfolio/src/pages/home.rs` +
`portfolio/assets/main.css`), made reusable. The one thing that can't move: the
CTA. The portfolio uses `Link { to: Route::… }` (app-specific routing), so the
shared component can't own the link — each site passes its message + link in as
slotted children.

### `zwipe-components/src/banner.rs` (new)
Follow the `button.rs` idiom. **Self-contained state** — the 3-state lifecycle
lives inside the component so consumers just drop `Banner { … }`:

- `enum BannerState { Shown, Leaving, Dismissed }` (module-private; rename off
  the portfolio's `Banner` to avoid colliding with the component name).
- `enum BannerStatus { Done, Doing }` (public) → the colored pill class
  (`status-done` / `status-doing`) with a default label ("Live" / "Doing").
- `#[component] pub fn Banner(...)`:
  - `category: String` — the eyebrow ("Announcement", "Featured").
  - `status: BannerStatus`
  - `#[props(default)] status_label: Option<String>` — override the pill text.
  - `#[props(default = 10)] auto_dismiss_secs: u32` — countdown duration (prop
    now so a future persistent banner is a one-line change; default keeps 10s).
  - `children: Element` — the message + the site's own CTA link.
  - Internal `use_signal(|| BannerState::Shown)`, the `onanimationend` handlers
    (dismiss on `banner-leave`, `Leaving` on countdown end), the `✕` button, and
    the `.banner-progress` bar — all lifted verbatim from the portfolio.
- Renders **only** the `.announcement-banner` element. The `.banner-stack`
  positioning wrapper stays a **site concern** — each site writes
  `div { class: "banner-stack", Banner {…} Banner {…} }`.

### `zwipe-components/src/lib.rs`
Add `mod banner;` and `pub use banner::{Banner, BannerStatus};`.

### `zwipe-components/assets/components.css`
New `/* ---- Banner (zwipe_components::Banner) ---- */` section. Move these
rules out of `portfolio/assets/main.css` verbatim (they already use theme vars):
`.banner-stack`, `.announcement-banner` (+ `.banner-leaving`), `.banner-header`,
`.banner-category`, `.banner-text`, `.banner-dismiss`, `.banner-progress`, and
the `@keyframes` (`banner-slide-in`, `banner-leave`, `banner-countdown`). Also
add the `.status-tag` / `.status-done` / `.status-doing` pills (currently
portfolio-local; the banner header depends on them). Theme vars they need
(`--color-success/-warning`, `--border-success/-warning`) already exist in
`themes.css` — verified.

- **Leave in the portfolio's `main.css`:** the `body:has(.nav-panel-open)
  .banner-stack` mobile-nav shift — it's tied to the portfolio's nav markup, not
  generic. (If zite wants the same behavior, add an equivalent rule against
  zite's own nav-open class in `zite/assets/style.css`.)

## Piece 2 — `zite` consumption

### Banner on the home page — `zite/src/pages/home.rs`
Add a `div.banner-stack` with the Android announcement:
```rust
Banner {
    category: "Announcement",
    status: BannerStatus::Done,
    "Zwipe is now on Android. "
    Link { to: Route::Android {}, "Get it \u{2192}" }   // internal route exists
}
```
`use zwipe_components::{Banner, BannerStatus};` at file scope (no inline `use`).
Right arrow via `\u{2192}`; sentence case, "Zwipe" capitalized, no em dashes.

### Changelog page
- **Route** — `zite/src/main.rs`: add `#[route("/changelog")] Changelog {}` to
  the `Route` enum (it's `#[derive(Routable)]`; static, so it lands in
  `static_routes()` → sitemap/SSG automatically).
- **Page** — `zite/src/pages/changelog.rs` (register in `pages/mod.rs`):
  `PageMeta` + newest-first list of releases, each: version, date, and entries
  grouped by kind (Added / Fixed / Changed).
- **Data** — a hand-authored `Vec<Release>` (e.g. `zite/src/data.rs` or a
  `changelog` data module):
  ```rust
  pub struct Release { pub version: &'static str, pub date: &'static str,
                       pub sections: &'static [(ChangeKind, &'static [&'static str])] }
  pub enum ChangeKind { Added, Fixed, Changed }
  ```
  Seed with the current shipped versions (server v1.0.5 / build 31, Android
  release). Owner supplies the copy.
- **CSS** — changelog page styles go in `zite/assets/style.css` (site-specific,
  not shared). Theme-var driven, crisp (no glow), sentence case.
- **Discoverability (optional):** add a `Changelog` nav link in
  `zite/src/main.rs` alongside Guides/About/Contribute, and/or a footer link.
  Owner to confirm whether it goes in the top nav or just the footer + banner.

## Piece 3 — portfolio refactor (separate repo, sequenced last)

The portfolio pins `zwipe-components` via the **GitHub git dep**
(`{ git = "https://github.com/scadoshi/zwipe" }`, exact commit in `Cargo.lock`).
So it can only adopt the shared `Banner` **after** Piece 1 is merged to the
zwipe repo's `main`.

1. `cargo update -p zwipe-components` in the portfolio to pull the new commit.
2. Replace the two inline banner blocks in `home.rs` with `Banner { … }`, moving
   each message + `Link`/`<a>` into the children. Delete the local `Banner` enum
   and the per-banner `use_signal`s.
3. Strip the migrated banner + status-tag rules from `portfolio/assets/main.css`
   (keep the `nav-panel-open` shift). Confirm nothing else used `.status-tag`
   (it's used elsewhere on the portfolio — keep those rules if so, or leave the
   pills in place and don't duplicate in `components.css`; reconcile at edit
   time).

## Sequencing & ship

1. **zwipe repo:** Piece 1 (component + CSS + export) and Piece 2 (zite banner +
   changelog) on one branch. `zite`'s `build.rs` copies the updated
   `components.css` into `zite/assets/` on build — do **not** hand-edit
   `zite/assets/components.css`.
2. Verify: `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`, then `dx serve` zite — check the banner
   slide-in/dismiss/countdown and the `/changelog` route render across a couple
   themes. No server half, no migration, no min-version gate.
3. **After merge to `main`:** Piece 3 in the portfolio repo (own PR).
