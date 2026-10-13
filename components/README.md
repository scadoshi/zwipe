# components

The Magic pieces of the Zwipe UI, over [orangui](https://github.com/scadoshi/orangui). orangui is the generic Dioxus kit (themes, panels, nav, charts, the entrance effects, the app and site kits); this crate re-exports all of it and adds what needs `zwipe-core`. The app (`zwiper`) and the website (`zite`) depend on it and import everything as `components::`. The owner's portfolio and cairn take orangui directly.

## Components

Here: `CardDetails`, `CardRow`, `CardRoleChips`, `KeywordChips`, `OracleText`, `FlippableCardImage`, `Changelog`, the deck chart family (`DeckCharts`, `ManaCurve`, `ManaFulfillment`, `DrawOdds`, `ChartLabel`) and `ZwipeArchitecture`.

From orangui, re-exported whole (`pub use orangui::*`): `ActionBar`, `Banner`, `Button`, `Chip`, `NavBar`, `NavDropdown`, `PageMeta`, `Panel`, `PasswordInput`, `ThemePicker`, the diagram pieces, the entrance figures, the app kit (`BottomSheet`, `PageHeader`, `ThemeSheet`, the hint pieces, the overlay back stack and, behind the `toast` feature, `ToastStack`), the site kit (`use_persisted_theme`, `StatsStrip`, `GalleryFrame`, `GalleryFooter`, the scripts), `HeatGrid`, the chart helpers and the theme model (`ThemeConfig`, `ALLOWED_THEMES`, `COLORBLIND_THEMES`, `display_theme_name`).

## Theme

The hosts hold orangui's `ThemeConfig`. `theme_from_preferences` builds one from a user's saved `UserPreferences`. zwipe-core keeps its own theme list because zerver validates a saved theme against it and core cannot depend on a UI crate; a test in `src/theme.rs` fails if the two lists or their display names differ.

## CSS exports

- `THEMES_CSS`, `COMPONENTS_CSS`, `APP_CSS`, `SITE_CSS`, `TOAST_CSS` (behind `toast`): orangui's sheets, re-exported
- `MAGIC_CSS`: this crate's rules (`assets/magic.css`): oracle text, keyword chips, card rows, the flippable card image, the changelog and the mana glyph inside a chip

Load them in this order: `THEMES_CSS`, `COMPONENTS_CSS`, `MAGIC_CSS`, then `APP_CSS` or `SITE_CSS`, then the host's own stylesheet, so host rules override shared ones at equal specificity. `TOAST_CSS` goes after the host's stylesheet.

## Features

- `toast`: enables orangui's `ToastStack` and `TOAST_CSS`. Off by default, so the sites never build `dioxus-primitives`.

## Notes

Domain types (card data, the changelog entries, the user's preferences) come from `zwipe-core`; this crate adds only presentation. The `Changelog` component renders the shared release history, identical on web and in-app. `scripts/swatch/` renders a themes.css as a review page; point it at orangui's.
