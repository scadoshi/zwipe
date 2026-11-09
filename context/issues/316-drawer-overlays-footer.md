# 316. The web side drawer covers the footer

State: approved | Owner: scotland | PR: - | Build seen: 1fd04e479

## Report
Scotty, 2026-11-09 (screenshot, web Account, Change email drawer): the side drawer runs to the bottom of the window and sits over the footer. Like toasts (284), it should stop at the footer's top when the footer is in view; full height otherwise.

Scotty, 2026-11-09: happens on every drawer (Add Filter drawer too); fix in the shared drawer so all drawers and their docks stop above the footer.
