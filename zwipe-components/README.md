# zwipe-components

Shared Dioxus UI components and CSS for Zwipe, consumed by the app (`zwiper`), the website (`zite`), and, as GitHub git dependencies, the owner's portfolio site and cairn (a counter app). Rendering and styling live here once so the surfaces never drift.

## Components

For every consumer: `ActionBar`, `Banner`, `Button`, `CardDetails`, `CardRoleChips`, `CardRow`, `Changelog`, `Chip`, `FlippableCardImage`, `KeywordChips`, `NavBar`, `NavDropdown`, `OracleText`, `PageMeta`, `Panel`, `ThemePicker`; the deck chart family (`DeckCharts`, `ManaCurve`, `ManaFulfillment`, `DrawOdds`, `ChartLabel`); the diagram pieces (`DiagramNode`, `DiagramArrow`, `DiagramDefs`); and the entrance figures (`CountUp`, `Decode`, `Figure`).

The app kit, for zwiper and cairn: `BottomSheet`, `PageHeader`, `ThemeSheet`, the hint pieces (`InfoButton`, `HintLine`, `HintBullets`, `HintBullet`, `HintKey`), the OS back stack (`OverlayBackStack`, `use_overlay_back_stack`, `use_overlay_back`, `use_overlay_back_action`), and, behind the `toast` feature, `ToastStack`.

The site kit, for zite and the portfolio: `use_persisted_theme`, `StatsStrip`, `GalleryFrame`, `GalleryFooter`, and the `REVEAL_JS` and `NAV_GLIDE_JS` scripts.

Chart helpers: `curve`, `area`, `tip_anchor`, `peak_indices`.

## CSS exports

- `THEMES_CSS`: the 31 theme palettes, each with light and dark variants (`assets/themes.css`)
- `COMPONENTS_CSS`: component styles (`assets/components.css`)
- `APP_CSS`: the app shell, for the apps only (`assets/app.css`)
- `SITE_CSS`: the gallery base, for the websites only (`assets/site.css`)
- `TOAST_CSS`: the toast stack, behind the `toast` feature (`assets/toast.css`)

Load them in that order (themes, components, then the opt-in sheet), then the host's own stylesheet, so host rules override shared ones at equal specificity. `TOAST_CSS` goes after the host's stylesheet. External consumers include these strings directly; the app and site inline the first two through their own asset pipelines.

## Features

- `toast`: `ToastStack` and `TOAST_CSS`. Off by default, so the sites never build `dioxus-primitives`.

## Notes

Domain types (themes, card data) come from `zwipe-core`; this crate adds only presentation. The `Changelog` component renders the shared release history from `changelog.rs`, identical on web and in-app.
