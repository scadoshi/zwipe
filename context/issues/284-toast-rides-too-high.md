# 284. Toasts sit far above the footer on short pages

State: reported | Owner: - | PR: - | Build seen: 1e03520ef

## Report
Scotty, app.zwipe.net Settings in a small browser window (screenshot): with the footer in view, the toast stack's bottom edge sits about 140px above the footer's top border. Toasts belong at the bottom left as low as they can go, stopping just above the footer (the same gap as the docks), never riding higher. Suspect: web.css's footer-rise animation on `.toast-container` (animation-range entry 0 to 100vh) overshoots on short pages; #383 only turned it off while a sheet is open. Repeated toasts in the shot are deliberate.

## Repro
1. Open app.zwipe.net/settings at 390 wide on a page short enough that the footer shows.
2. Toggle Universes Beyond to make a toast.
3. Compare the toast's bottom edge to the footer's top border.
