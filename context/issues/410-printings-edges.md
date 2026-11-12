# 410. Printings fades its cards out short of the sheet's edges

State: fixing | Owner: laptop | PR: - (pushed branch, hash in the composer's log) | Build seen: ea76d3002

## Report
Scotty, on the web and in the app: the printings fan should run to the sheet's left and right edges with no fade-out, and the arrow buttons should overlay the cards, readable over art with their button background. Today the side cards fade out before reaching the arrows.

## Cause
The cover flow laid its step buttons beside the stage, inside the sheet's side padding, and masked the stage with a gradient that faded its outer ninth on each side.

## Fix
laptop: the stage has no mask and fills the flow's width; the step buttons sit over it (absolute, above the cards), `--flow-step-inset` in from each edge, in their own button background. Each host runs the flow out through its sheet's side padding: the web's narrow sheet and wide modal by negative margins, the app's sheet body already full width. On a phone the stage is wider than tall (1 / 0.62, was 1 / 0.95 between the buttons), so the middle card keeps its size and the stack fills the sides. The loading fan from 409 follows the same layout. Stacked on 409's branch.

## Checked (headless Chrome, app.zwipe.net local, deck "Krenko goblins")
- 1400, Beetleback Chief 4 of 11: the seven printings to the right run to the modal's edge under the arrow, no fade; the arrows read over the art. The loading fan reaches both edges.
- 390, a Mountain: the stack runs out to both screen edges, the arrows over the cards; the middle card is the size it was (about 160px wide). The loading fan reaches both edges.
- The app (zwiper) builds and its tests pass; it does not build for the browser, so it has no headless shot.

## Verification
-
