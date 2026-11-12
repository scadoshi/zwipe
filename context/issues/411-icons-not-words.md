# 411. Card buttons say Image, Flip, Meld, Star and Select in words

State: fixing | Owner: halo | PR: branch nits/halo-411 | Build seen: 53cdecc25

## Report
From Scotty, through the composer, in line with icons-not-words: the card buttons that say Image, Flip, Meld, Star or Unstar, and the web Printings sheet's Select, become icon buttons everywhere they show, on the web, in the app and on zwipe.net. Each keeps its name for a screen reader and as a tooltip, matches the height of the text buttons beside it, and any hint, guide or shortcut row that names one shows its icon as a key chip. Create deck on Home stays a word. Scotty picked from rendered option sheets: card art for Image (drawn in our grammar, no third-party license), repeat (today's FLIP) for Flip, merge for Meld, the sharp star for Star, filled in the warning gold when starred, and a check for Select.

## Fix
halo:
- **components `CardIconButton`** (`card_icon_button.rs`): a `card-action-btn` box around a `DockIcon`, named by `aria-label` and `title`, with `aria-pressed` when it toggles. The icon draws at 1.2em and the padding gives that back (`calc(0.2rem - 0.1em)`), so it stands as high as Printings. `ImageButton` (`CARD_ART`) and `StarButton` (`STAR`, outline unstarred, filled in `--color-warning` when starred, labeled Star or Unstar) are built on it. `MELD` is the merge path; Flip uses orangui's `FLIP`.
- **Swapped in:** the card details foot's Image, Flip and Meld (card rows and every drawer and side sheet), the card image's own Flip and Meld (swipe stacks, image previews, Examples, the no-image card), `CardEditButtons`' Star, the Featured flavor rows on the app, the web and zwipe.net, the web Home ghost, and the Printings sheet's Select.
- **Hints:** zwipe-ui `KeyIcon` gains `CardArt`, `Flip`, `Meld` and `Star`. The shortcuts sheet draws a row whose text is one `[[icon]]` as that icon's key chip with its name for a screen reader (`shortcuts::row_icon` and `row_label`), and the `i`, `v` and Shift+S rows use them. The deck's first hint shows the Star chip as the star, and zite's guides say `[[card-art]]` and `[[star]]`.
- **Tests, written first:** the icon names, the sheet rows and labels, the buttons' markup and sizing, the star's fill in warning gold, and Select's check. Three HTML goldens changed for the Star toggle and the Flip overlay.

## Checked (headless Chrome, the real sheets)
- Web at 1400 and app at 393, dark and light: the stepper and Star toggle (outline, then gold-filled), a two-faced card's Flip, card art and Printings, a meld half's Meld off and on, the Featured flavor row, and the Printings Select check. Every icon button stands level with the text buttons beside it.

## Not changed
- `zite/src/pages/guides/content.rs`, the commander maybeboard guide's screenshot alt text: it describes the old screenshot, which still shows the words, so it waits for a new screenshot.
- A starred row's ★ marker draws in `--palette-6` (a coral red in rustbox), while the star toggle and the Star hint use `--color-warning` (gold).

## Later
- 414 switched Image from the card art to Lucide's square `image`, with Lucide's ISC notice in `components/LICENSE-LUCIDE`.

## Verification
-
