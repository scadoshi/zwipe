# 412. Stars take three different colors

State: fixing | Owner: halo | PR: branch nits/halo-412 (on top of nits/halo-411) | Build seen: a6d1f424c

## Report
From 411: the Star toggle filled in the warning gold, which needed an exception to the status-color gate, while a starred row's ★ marker and the MVP chips drew in `--palette-6` (a coral red in rustbox). Scotty approved a star color token, so every star draws from one place.

## Fix
halo:
- **orangui 8626ccf** (main): `--color-star` and `--ink-color-star` in every theme, defaulting to the theme's `--color-warning` and its ink. The default sits in a `:where(...)` rule at the end of the sheet, so a theme block may name its own and keep it. It's documented in `styling.md` beside `--color-stat` and pinned by `tests/themes_star.rs`. Nothing else in the sheet changed.
- **zwipe:** orangui repinned to 8626ccf, and the web's copied sheets and zerver's share-preview copy of `themes.css` refreshed. The star toggle's fill and border, the starred row's marker (`.card-row-mvp`), the deck head's MVP chip and the featured strip's MVP chip draw in `--color-star`. zwipe-ui's `Role::Star` (`--color-star`) colors the Star hint chip. The status-color gate is back to its one exception.
- **Tests, written first:** `Role::Star`'s variable, the toggle's star color, and the marker and MVP chip rules naming `--color-star` and not `--palette-6`.

## Verification
-
