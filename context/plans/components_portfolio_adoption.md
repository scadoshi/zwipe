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
2. **Theme re-exports + `ThemePicker`.** Re-export `ThemeConfig` + the theme
   list from the crate; lift zite's `ThemePicker` (canonical, per the
   reconciliation ruling) + its CSS into the crate; migrate zite.
3. **`PageMeta`** with site-config props; migrate zite.
4. **Nav shell** (most API design); migrate zite.

The portfolio migrates to each piece as it lands (its stated commitment), which
doubles as second-consumer verification.

Related: [`zwipe_components.md`](zwipe_components.md) (crate plan + roadmap),
memory note `project-components-crates-io` (crates.io constraints if that day
comes).
