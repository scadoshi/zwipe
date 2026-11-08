# 311. zwipe.net guides filter: plain label and a hand-rolled ×

State: merged | Owner: thinker | PR: #455 | Build seen: 69f38270e

## Report
Scotty, 2026-11-08 (screenshot of zwipe.net Guides, "Filter" + search box + × + All / Getting started... chips):
1. The "Filter" label is bare pink text; it should be a label chip like the app's ("Group by", "Show").
2. The × beside the box is its own grey button. Use the shared clear structure (components::ClearWrap, as nit 49) so the × eases into place inside the field and is red, like every other clearable input.
