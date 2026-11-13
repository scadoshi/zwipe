# 514. The shortcuts sheet lists ? on a page with no hint

State: fixing | Owner: laptop | Type: issue | PR: - (pushed branch, hash in the composer's log) | Build seen: release 7b335fb52

## Report
Scotty, via the composer: on zwipe.net the keyboard shortcuts sheet (Shift+H) lists `?` for the page's hint, but no zwipe.net page has a hint, so `?` does nothing. Remove the row wherever the page has no hint; the web app keeps it where a hint exists.

## Before
Production zwipe.net `/`, headless at 1440 and 390: Shift+H lists `t Themes`, `? This page's hint`, `Shift+H These shortcuts`, `Esc Close the open sheet` under Page, with no hint on the page.

## Cause
`zwipe_ui::shortcuts::sheet_groups` puts the hint row on every screen's Page group, and the root list every page falls back to (`everywhere_groups`) is built from it. Whether `?` opens anything is only known at runtime, from the newest `use_global_keys` registration's `on_hint`; zite registers only the root's `use_global_keys(None)`.

## Fix
- `components::keys`: `use_global_keys` keeps each registration's "has a hint" in a context the root provides; `ShortcutsSheet` drops the hint row while the newest registration has none (`zwipe_ui::shortcuts::without_hint`).
- The web app's pages with a hint (their `ScreenFrame` passes `on_hint`) keep the row; its pages without one lose it. The app has no shortcuts sheet.

## Checked
- `components/tests/shortcuts_sheet.rs`, red before: a site with no hint lists no hint key; a page with no hint over one with a hint lists none (the newest answers `?`). Control: a page with a hint lists it. `zwipe-ui`: `without_hint` drops only that row and an emptied group.
- zite after, local `dx serve`, headless: `/` at 1440 and 390 and `/guides` at 1440 list `t`, `Shift+H`, `Esc` under Page, no `?`.

## Not checked
- The web app live: its pages go through the same `ScreenFrame` `on_hint`, covered by the components tests.

## Verification
-
