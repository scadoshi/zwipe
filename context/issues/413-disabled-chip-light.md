# 413. A disabled chip looks enabled in light themes

State: fixing | Owner: thinker | PR: - | Build seen: production, app

## Report
Composer, 2026-11-12, from droider on the app: in a light theme, the grayed-out Art chip in Stacks looks the same as an enabled chip.

## Cause
orangui's `components.css` fades `.chip.disabled` to 0.45 opacity, and a light-theme rule shared with the disabled buttons set it back to 1 and kept the full border, with only the ink moved to `--text-muted`. A button keeps its fill on a light panel, but a chip has the panel's own fill, so with full opacity and full border a disabled chip read as an enabled one (on tokyo-night light the muted ink is blue, so it looked tinted).

## Fix
thinker:
- orangui 00a8954: the light-theme exception covers the disabled buttons only, so a disabled chip fades as it does on dark themes, on the app, the web and zwipe.net. `tests/disabled_chip.rs` checks the fade and that no light-theme rule undoes it.
- zwipe: orangui pinned to 00a8954 (it also brings 6d97b04, test-only), zwiper-web's component sheet refreshed.
- Headless Chrome, web deck page in Stacks, gruvbox light: the Art chip was opacity 1 with a full border; it is now 0.45, as on gruvbox dark.

## Verification
-
