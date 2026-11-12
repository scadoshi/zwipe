# 414. The shortcuts sheet's icon rows show no names

State: fixing | Owner: halo | PR: branch nits/halo-414 (on top of nits/halo-412) | Build seen: f6b89539d

## Report
From Scotty, through the composer, after 411: in the Keyboard shortcuts sheet, rows that show only an icon should show the icon and its name, so people learn the icon. `i` reads as the image icon and Image, `v` as the repeat icon with Flip or the merge icon with meld, since `v` does both, and Shift+S as the star with Star. The same applies on every sheet with those rows: the deck page, card rows, the swipe screens and Examples, on the web and zwipe.net. Hints elsewhere keep icons alone.

## Fix
halo:
- **zwipe-ui `shortcuts`:** `row_parts` splits a row's text into words and `[[name]]` icons (a name no icon has, or an unclosed `[[`, stays words). It replaces `row_icon` and `row_label`, since each icon now has its name beside it. The rows read `[[image]] Image`, `[[flip]] Flip or [[meld]] meld` (the shared `FLIP_ROW`) and `[[star]] Star`.
- **components `ShortcutsSheet`:** draws each run, an icon as its key chip and words as they are.
- **zwiper-web `chrome/shortcuts.rs`:** the swipe screens' and Examples' `v` rows take `FLIP_ROW`.
- **Tests, written first:** the tokenizer (icons, words, an unknown name, an unclosed `[[`) and the three rows.

## Checked
- Headless Chrome, the deck page's sheet at 1400 from the web's sheets: `i` shows the image icon then Image, `v` the repeat icon, Flip or, the merge icon, meld, and Shift+S the star then Star.

## Image icon, changed after 411
Scotty switched Image from the portrait card art to Lucide's square `image` (frame, sun, hill) everywhere 411 and 414 drew it: `components::IMAGE_ICON` (renamed from `CARD_ART`), `KeyIcon::Image` (`image`, renamed from `CardArt`), the sheet's `[[image]] Image` row and the guide's `[[image]]`. Since the path is Lucide's, `components/LICENSE-LUCIDE` carries Lucide's ISC notice and names the Lucide-derived icons: `IMAGE_ICON` (Lucide's `image`) and `MELD` (Lucide's `merge`). orangui's own `FLIP` and `SEND` follow Feather/Lucide paths too and carry no notice in orangui.

## Verification
-
