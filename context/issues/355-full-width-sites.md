# 355. The sites leave a wide window's sides empty

State: fixing | Owner: scotland | PR: zite #557, zwiper-web home -, portfolio - | Build seen: live, web

## Report
Scotty via composer, 2026-11-11: use the full screen width on every site, the way the web deck page does (350). Repos: the portfolio, zite (zwipe.net and its marketing pages) and zwiper-web's home. Containers lose their fixed max width and use the deck page's gutters. Grids (cards, decks, projects, feature tiles) add columns as the width grows, with tiles keeping their size, stopping at about 6 or 7 columns and leaving the rest as margin. Text and hero sections stay capped and centered, with prose at about 70ch. Reuse orangui tokens, and change the shared container there if it lives there. One PR per repo, checked at 1280, 1920 and 2560 wide and at phone width.

## Cause
Each site caps its own page container at 74rem (zite `.page` and `.shared-deck`, the web's `--web-page-max`, the portfolio's own), so a wide window gets one 71rem column with empty grid on both sides. orangui has no shared container, so there is nothing to repin.

## Fix
zite, scotland, #557 (style.css), reworked after composer's review:
- `.page` has no fixed cap and keeps its 1.5rem gutter. Every section, and the hero card, takes one width, `--zite-page-max` (seven 26rem columns, so the full window up to 2560). Headers, filters, panels and the card grids under them share their edges.
- From 1280px the card grids fill that width edge to edge, each card at least 22rem: five across at 1920 and six at 2560. Below 1280 they keep their 3/2/1 columns. A pair stays a pair across the page.
- Home: from 1920 the demo and the three feature panels share one row (the demo, then the panels side by side). Featured flavor and the reviews span the width.
- Running text keeps a 70ch measure: the hero's tagline (centered), guide paragraphs and lists, privacy sections and release notes. The About diagram stays at 57rem inside its panel.
- The shared deck page drops its 74rem cap and pairs its groups in two columns from 1400px, as the web deck page does.

Test `zite/tests/full_width.rs` failed on the old stylesheet and passes now. Shots in `target/shots-355/`: `before-*-1920.png`, and `r4-{home,guides,about,changelog,community,contribute,privacy}-{1280,1920,2560,390}.png` after.
