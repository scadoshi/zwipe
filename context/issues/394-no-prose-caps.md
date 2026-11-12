# 394. Running text stops at 70ch inside the wide panels

State: fixing | Owner: thinker | PR: - | Build seen: production, web, zwipe.net, portfolio

## Report
Composer for Scotty, 2026-11-12: drop the 70ch prose caps everywhere: the portfolio (every page, project pages included, such as Steller's "How it's built" bullets), zwipe.net and app.zwipe.net. Text and lists fill their panel's width; the 1400px page column stays. Surfer is sweeping for every capped spot.

## Fix
thinker, the shared rules first:
- zwipe.net (`zite/assets/style.css`): the 70ch rule over the guides, privacy, release notes, the hero tagline and panel paragraphs is gone, and so is the 25rem cap on the coming soon text;
- app.zwipe.net (`zwiper-web/assets/web.css`): Home's tagline and the Privacy policy's paragraphs and lists lose their 70ch caps;
- the portfolio (portfolio PR): the 70ch rule on panel paragraphs and list items and the 30rem cap on the orangui kit page's copy are gone.

Caps left in place because they size a component rather than text: forms, dialogs, the dead-end card, the guide filter field, and zite's shared deck page column (74rem). Surfer's sweep may add more.

## Verification
-
