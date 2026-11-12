# 355. The sites leave a wide window's sides empty

State: fixing | Owner: scotland | PR: zite #PR, zwiper-web home -, portfolio - | Build seen: live, web

## Report
Scotty via composer, 2026-11-11: use the full screen width on every site, the way the web deck page does (350). Repos: the portfolio, zite (zwipe.net and its marketing pages) and zwiper-web's home. Containers lose their fixed max width and use the deck page's gutters. Grids (cards, decks, projects, feature tiles) add columns as the width grows, with tiles keeping their size, stopping at about 6 or 7 columns and leaving the rest as margin. Text and hero sections stay capped and centered, with prose at about 70ch. Reuse orangui tokens, and change the shared container there if it lives there. One PR per repo, checked at 1280, 1920 and 2560 wide and at phone width.

## Cause
Each site caps its own page container at 74rem (zite `.page` and `.shared-deck`, the web's `--web-page-max`, the portfolio's own), so a wide window gets one 71rem column with empty grid on both sides. orangui has no shared container, so there is nothing to repin.

## Fix
zite, scotland, #PR (style.css):
- `.page` has no cap and keeps its 1.5rem gutter.
- Each section stays at `--zite-text-max` (71rem, the hero's width), centered. A section that holds a card grid runs wider, with its head still at the text width.
- From 1280px the card grids add columns: each card 22 to 26rem wide, centered, up to seven across (four at 1920, five at 2560).
- Running text (guide paragraphs and lists, privacy sections, release notes) keeps a 70ch measure.
- The shared deck page drops its 74rem cap and pairs its groups in two columns from 1400px, as the web deck page does.
- At 1280 and below the pages look as they did.

Test `zite/tests/full_width.rs` failed on the old stylesheet and passes now. Shots in `target/shots-355/` (before at 1920; after at 1280, 1920, 2560 and 390).
