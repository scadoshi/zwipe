# zwipe-components — portfolio adoption (response to the consumer request)

**Status: ACCEPTED 2026-07-08, landing in passes. Pass 1 (CSS ownership +
export) shipped the same day.**

The portfolio site (scottyfermo.com, separate public repo) filed a request
(`~/Developer/portfolio/context/zwipe_components_request.md`) to consume
`zwipe-components` as its second external surface: it duplicates zite's theme
palettes (~650 lines, byte-identical), theme picker, `ThemeConfig`, nav
hamburger shell, and `PageMeta` by hand, and the copies have started to drift.
This doc is the maintainer ruling and the landing plan.

## Rulings

| Request | Ruling |
|---|---|
| Public/git-dep consumption | **Granted trivially** — the repo is already public; `zwipe-components = { git = "https://github.com/scadoshi/zwipe" }` works today (the internal `path` dep on zwipe-core resolves inside the fetched repo). crates.io is a later option and would require publishing zwipe-core too. |
| `domain` feature gate | **Declined.** `ThemeConfig` stays in zwipe-core (below), so theme consumers pull core transitively regardless — and core is small and pure (serde/uuid/chrono; no sqlx/tokio). A gate adds maintenance for ~no win. Revisit only if core compile time ever bothers a consumer. |
| Move `ThemeConfig` out of zwipe-core | **Declined as asked, granted in effect.** `ThemeConfig` is a persisted user-preference domain type — zerver stores it, so it must stay in core (a UI crate dep on the server, or core→components re-export, would break the dependency graph / core purity). Instead **zwipe-components re-exports it** (`pub use`), giving consumers one import path with the same ergonomics. |
| `themes.css` into the crate | **Granted** — done in Pass 1. |
| `ThemePicker` into the crate | **Granted.** Reconciliation ruling: zite's version is canonical; the portfolio's `has_light_mode` guard is dropped (owner ruling 2026-07-08: **every theme has a light mode**, no exceptions) and its `vantablack` branch is dead code — gone. |
| Parameterized `PageMeta` | **Granted** — site-config props (`base_url`, `site_name`, title-suffix rule, optional og image, twitter card type). |
| Slotted nav shell | **Granted, last** — brand/links/trailing slots; hamburger + panel + 60rem breakpoint CSS into `components.css`. |

**Correction to the portfolio's commitment list:** "mirror zite's build.rs copy
pipeline" does not work cross-repo — a git-dep consumer's crate sources live in
`~/.cargo/git/checkouts/…`, unreachable by relative path. The crate instead
exports the CSS as string constants (`COMPONENTS_CSS`, `THEMES_CSS`); external
consumers inline them via `document::Style`. Workspace apps keep the copy
pipeline.

## Landing order

1. ✅ **CSS ownership + export (shipped 2026-07-08).** `shared/themes.css`
   moved to `zwipe-components/assets/themes.css` (the `shared/` dir is gone);
   both apps' `build.rs` copy from the crate; `COMPONENTS_CSS` / `THEMES_CSS`
   `include_str!` constants exported for external consumers.
2. ✅ **Theme re-exports + `ThemePicker` (shipped 2026-07-08).** The crate
   re-exports `ThemeConfig` + `ALLOWED_THEMES` (one import path for UI
   consumers) and owns `ThemePicker` — zite's version verbatim, except the
   host now passes its `Signal<ThemeConfig>` as a prop instead of the
   component reaching for context, so any provider/apply strategy works. The
   picker CSS moved to `components.css` with a self-contained pill look for
   the trigger/mode-toggle (zite had styled those via its nav group selector);
   zite keeps only its `.nav-panel .theme-*` context overrides (Pass 4 scope).
3. ✅ **`PageMeta` (shipped 2026-07-08).** Crate owns the head-meta component,
   parameterized by a `SiteMeta` config (`base_url`, `site_name`, optional
   `og_image_path`; the Twitter card type follows from image presence, and a
   title equal to the bare site name renders unsuffixed — the portfolio's home
   rule, a no-op for zite whose pages always pass descriptive titles). zite
   keeps a thin local `PageMeta` wrapper baking in its `SiteMeta`, so page
   call sites are untouched.
4. ✅ **Nav shell (shipped 2026-07-08).** `NavBar { open, brand, persistent,
   links, trailing }`: the crate owns the structure (sticky wrapper, hamburger
   toggle, collapsing panel, `ul.nav-links` + link-pill CSS, the 60rem
   breakpoint block including the picker-in-panel overrides); the host owns
   the content and the `open` signal (so its link `onclick`s close the panel).
   `BRAND_RESET_JS` (scroll-to-top + logo animation restart) is exported for
   brand links. zite keeps only its specifics: the ASCII `.nav-logo`, the
   `.nav-stores-persistent` CTAs, and `.store-link` styling.

The portfolio migrates to each piece as it lands (its stated commitment), which
doubles as second-consumer verification.

## Post-Pass-4 tweak round (portfolio review, all granted 2026-07-08)

- **Bare `nav` selector scoped** to `.nav-wrapper nav` (base + breakpoint
  rules) — a consumer's other semantic `<nav>` (breadcrumbs, TOC) no longer
  inherits site-header layout. The one real defect of Pass 4.
- **Pill styling covers button nav items**: `.nav-links button.nav-link`
  joins the anchor selectors (base, hover, `.active`, panel-width) so e.g. a
  dropdown trigger shares the pill without local duplication.
- **Nav width is a variable**: `max-width: var(--nav-max-width, 60rem)` —
  zite untouched, the portfolio matches its 760px column with one variable.
- **CSS cascade order documented** (lib.rs crate docs): `THEMES_CSS` →
  `COMPONENTS_CSS` → site CSS. Both workspace apps already load in this order.
- **Page weight note:** `components.css` ships whole — a consumer carries
  every component's rules even for components it doesn't render (as zwiper
  does with the nav shell). It's a few KB of inert CSS; not worth a split
  today, revisit only if the file grows an order of magnitude.
- **Version tags:** git-dep consumers should pin `rev = "…"` for
  reproducibility until the maintainers start tagging (`zwipe-components-vX`)
  or publish to crates.io; tags are cheap and can start whenever the first
  external pin lands.

Consuming before the repo is pushed: a local `git = "file:///…/zwipe"` dep
against committed history works for same-machine development; flip to the
GitHub URL once pushed (required for any other machine or CI).

Related: [`zwipe_components.md`](zwipe_components.md) (crate plan + roadmap),
memory note `project-components-crates-io` (crates.io constraints if that day
comes).
