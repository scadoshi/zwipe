# 369. The Share dialog's link sits in an odd dark box

State: fixing | Owner: laptop | PR: #564 | Build seen: production, web

## Report
Composer for Scotty, 2026-11-11: the Share deck dialog shows its link in a dark backgrounded box that matches nothing else in the app. It should be an existing colored chip in a color that reads as a link, holding the full URL, hugging its width and centered, on one line until the screen is too narrow, then breaking anywhere in the URL. Web and app.

## Cause
`ShareUrl` (components, used by the web's and the app's Share dialogs) drew the link as `.share-url`: a sunk monospace field cut with an ellipsis, a shape no other control has.

## Fix
laptop, #564: `ShareUrl` is a `.stat-chip` (the kit's tinted chip, the default accent tone) centered under the dialog's text. It holds the whole URL, hugs it while it fits, and breaks at any character (`word-break: break-all`) once it does not. A tap still selects all of it. One component, so the app's dialog changes with the web's.

## Verification
-
