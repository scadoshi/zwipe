# 387. app.zwipe.net Home runs the window's width

State: fixing | Owner: thinker | PR: #590 | Build seen: production, web

## Report
Composer for Scotty, 2026-11-12: app.zwipe.net Home is still full width (from #561). It should sit on the same 1400px column as zwipe.net and the portfolio: the hero across that column, the tagline at 70ch, and Decks, Colors and Featured flavor inside it. Only Home and the other few-element pages; the deck page and the card grid screens stay full width.

## Cause
#561 gave Home its own `--web-wide-max` (seven card columns, wider than a 2560 window), and every other page sat on `--web-page-max`, zwipe.net's old 74rem column, which #579 moved to 1400px on zwipe.net only.

## Fix
thinker, `zwiper-web/assets/web.css`:
- `--web-page-max` is zwipe.net's 87.5rem (1400px), and Home drops its own width, so Home, Settings, Account, Activity, Playstyle, About, Privacy, Decks, the deck form, Commander maybeboard and the Oracle tag dictionary share it;
- Home's tagline keeps its 70ch measure, and the Privacy policy's paragraphs and lists take a 70ch measure, as zwipe.net's running text does;
- a dialog keeps its old 74rem cap (`--web-dialog-max`);
- the deck page and the card screens keep their own rules and run the window's width.

From 1920, Decks, Colors and Featured flavor still share one row under the hero, inside the column.

## Verification
-
