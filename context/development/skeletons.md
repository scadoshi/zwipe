# Loading skeletons

Decided 2026-10-27, replacing the closed-vs-open-vocabulary rule from the 1.8.1 review round (static chrome renders real from the first frame). That rule is superseded; the release entries that mention it are history.

A loading skeleton never renders real text, real chips or working controls. While a page or region isn't usable, everything in it ghosts:

- Text (titles, group headers, row labels like "Show") becomes a `skeleton-bar` the size of the text. Inside a live `.card-group-header` the bar pads itself to the text line's height.
- A chip with text becomes a bare `skeleton-chip` (`skeleton-chip-sm`, `-md`, `-lg`, `-xl` for width).
- A colored chip or block keeps its color. Add `skeleton-tinted` next to the class that sets `--tone` (`stat-chip-p1`, `podium-first`) and drop the text; on a bare `.skeleton-chip` it paints the stat-chip's tinted fill and edge. Without a tone class it takes the bare stat-chip's `--accent-primary`.
- Controls ghost too: chip rows (`ChipRowSkeleton`, its label a p1-tinted chip), Quick add (`QuickAddSkeleton`), export and filter rows. Nothing tappable shows until the data lands.
- Every ghost keeps its element's shape, size and place, so the swap to real content doesn't jump.

The screen header and the action bar stay real: they are how the user leaves a screen that is still loading.

To look at them on a device, build with `SKELETON_DELAY_MS=2000` (see `zwipe-client/src/call.rs`), which holds every GET for that long.
