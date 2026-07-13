# zwipe-components

Shared Dioxus UI components and CSS for Zwipe, consumed by the app (`zwiper`),
the website (`zite`), and the owner's portfolio site (via a GitHub git
dependency). Rendering and styling live here once so the surfaces never drift.

## Components

`ActionBar`, `Banner`, `Button`, `CardDetails`, `CardRoleChips`, `CardRow`,
`Changelog`, `Chip`, `FlippableCardImage`, `KeywordChips`, `NavBar`,
`NavDropdown`, `OracleText`, `PageMeta`, `Panel`, `ThemePicker`.

## CSS exports

- `COMPONENTS_CSS`: component styles (`assets/components.css`)
- `THEMES_CSS`: the 31 theme palettes, each with light and dark variants (`assets/themes.css`)

External consumers include these strings directly; the app and site inline them
through their own asset pipelines.

## Notes

Domain types (themes, card data) come from `zwipe-core`; this crate adds only
presentation. The `Changelog` component renders the shared release history from
`changelog.rs`, identical on web and in-app.
