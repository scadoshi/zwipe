//! zwipe.net's side of the guides: the screenshots' compiled assets and the frames' sizes. The
//! guides themselves are `zwipe_core::content::guides`.

use dioxus::prelude::*;
use zwipe_core::content::guides::{Pixels, image_size};

use crate::components::MediaSize;

/// Compile-time asset registry for guide screenshots. `asset!()` needs a
/// literal path, so each shipped image gets one arm here, added when the file
/// lands (see `context/plans/guide_images.md`). `None` renders nothing, so a
/// screenshot without its registry arm is a wiring bug caught by eye.
#[expect(
    clippy::too_many_lines,
    reason = "an asset table, one arm per screenshot"
)]
pub(crate) fn guide_image(file: &str) -> Option<Asset> {
    match file {
        "getting-started/01-create-deck.webp" => {
            Some(asset!("/assets/guides/getting-started/01-create-deck.webp"))
        }
        "getting-started/02-add-cards.webp" => {
            Some(asset!("/assets/guides/getting-started/02-add-cards.webp"))
        }
        "getting-started/03-deck-cards.webp" => {
            Some(asset!("/assets/guides/getting-started/03-deck-cards.webp"))
        }
        "swipe-to-build/01-swipe-hint.webp" => {
            Some(asset!("/assets/guides/swipe-to-build/01-swipe-hint.webp"))
        }
        "swipe-to-build/02-filter-sheet.webp" => {
            Some(asset!("/assets/guides/swipe-to-build/02-filter-sheet.webp"))
        }
        "swipe-to-build/03-maybeboard-stack.webp" => Some(asset!(
            "/assets/guides/swipe-to-build/03-maybeboard-stack.webp"
        )),
        "commander-and-formats/01-format-picker.webp" => Some(asset!(
            "/assets/guides/commander-and-formats/01-format-picker.webp"
        )),
        "commander-and-formats/02-swipe-select.webp" => Some(asset!(
            "/assets/guides/commander-and-formats/02-swipe-select.webp"
        )),
        "filtering/02-filter-hint.webp" => {
            Some(asset!("/assets/guides/filtering/02-filter-hint.webp"))
        }
        "commander-maybeboard/01-swipe-select-hint.webp" => Some(asset!(
            "/assets/guides/commander-maybeboard/01-swipe-select-hint.webp"
        )),
        "commander-maybeboard/02-saved-commanders.webp" => Some(asset!(
            "/assets/guides/commander-maybeboard/02-saved-commanders.webp"
        )),
        "commander-maybeboard/03-expanded-row.webp" => Some(asset!(
            "/assets/guides/commander-maybeboard/03-expanded-row.webp"
        )),
        "commander-maybeboard/04-maybeboard-hint.webp" => Some(asset!(
            "/assets/guides/commander-maybeboard/04-maybeboard-hint.webp"
        )),
        "organize-and-browse/01-deck-list-by-tag.webp" => Some(asset!(
            "/assets/guides/organize-and-browse/01-deck-list-by-tag.webp"
        )),
        "organize-and-browse/02-cards-by-type-collapsed.webp" => Some(asset!(
            "/assets/guides/organize-and-browse/02-cards-by-type-collapsed.webp"
        )),
        "organize-and-browse/03-quick-add-results.webp" => Some(asset!(
            "/assets/guides/organize-and-browse/03-quick-add-results.webp"
        )),
        "share-your-deck/01-share-dialog.webp" => Some(asset!(
            "/assets/guides/share-your-deck/01-share-dialog.webp"
        )),
        "share-your-deck/02-shared-page.webp" => {
            Some(asset!("/assets/guides/share-your-deck/02-shared-page.webp"))
        }
        "share-your-deck/03-stop-share.webp" => {
            Some(asset!("/assets/guides/share-your-deck/03-stop-share.webp"))
        }
        "remove-cards/01-remove-swipe.webp" => {
            Some(asset!("/assets/guides/remove-cards/01-remove-swipe.webp"))
        }
        "swipe-memory/01-clear-skips.webp" => {
            Some(asset!("/assets/guides/swipe-memory/01-clear-skips.webp"))
        }
        "synergy/01-synergy-on.webp" => Some(asset!("/assets/guides/synergy/01-synergy-on.webp")),
        "deck-stats/01-distributions.webp" => {
            Some(asset!("/assets/guides/deck-stats/01-distributions.webp"))
        }
        "deck-stats/02-draw-odds.webp" => {
            Some(asset!("/assets/guides/deck-stats/02-draw-odds.webp"))
        }
        "budgeting/01-budget-section.webp" => {
            Some(asset!("/assets/guides/budgeting/01-budget-section.webp"))
        }
        "land-targets/01-mana-section.webp" => {
            Some(asset!("/assets/guides/land-targets/01-mana-section.webp"))
        }
        "deck-mvps/01-featured-strip.webp" => {
            Some(asset!("/assets/guides/deck-mvps/01-featured-strip.webp"))
        }
        "deck-tags/01-tag-picker.webp" => {
            Some(asset!("/assets/guides/deck-tags/01-tag-picker.webp"))
        }
        "oracle-tags/01-tag-search.webp" => {
            Some(asset!("/assets/guides/oracle-tags/01-tag-search.webp"))
        }
        "oracle-tags/02-tags-section.webp" => {
            Some(asset!("/assets/guides/oracle-tags/02-tags-section.webp"))
        }
        "card-roles/01-role-chips.webp" => {
            Some(asset!("/assets/guides/card-roles/01-role-chips.webp"))
        }
        "oracle-tag-dictionary/01-by-letter.webp" => Some(asset!(
            "/assets/guides/oracle-tag-dictionary/01-by-letter.webp"
        )),
        "oracle-tag-dictionary/02-search.webp" => Some(asset!(
            "/assets/guides/oracle-tag-dictionary/02-search.webp"
        )),
        "import-export/01-import-paste.webp" => {
            Some(asset!("/assets/guides/import-export/01-import-paste.webp"))
        }
        "import-export/02-imported.webp" => {
            Some(asset!("/assets/guides/import-export/02-imported.webp"))
        }
        "import-export/03-export.webp" => {
            Some(asset!("/assets/guides/import-export/03-export.webp"))
        }
        "activity/01-heatmap-and-totals.webp" => {
            Some(asset!("/assets/guides/activity/01-heatmap-and-totals.webp"))
        }
        "activity/02-card-roles.webp" => Some(asset!("/assets/guides/activity/02-card-roles.webp")),
        "activity/03-most-skipped.webp" => {
            Some(asset!("/assets/guides/activity/03-most-skipped.webp"))
        }
        "playstyle/01-colors.webp" => Some(asset!("/assets/guides/playstyle/01-colors.webp")),
        "playstyle/02-oracle-tag-impact.webp" => {
            Some(asset!("/assets/guides/playstyle/02-oracle-tag-impact.webp"))
        }
        "playstyle/03-why-this-commander.webp" => Some(asset!(
            "/assets/guides/playstyle/03-why-this-commander.webp"
        )),
        "tags-roles-and-oracle-tags/01-deck-and-oracle-tags.webp" => Some(asset!(
            "/assets/guides/tags-roles-and-oracle-tags/01-deck-and-oracle-tags.webp"
        )),
        "tags-roles-and-oracle-tags/02-role-to-oracle-tags.webp" => Some(asset!(
            "/assets/guides/tags-roles-and-oracle-tags/02-role-to-oracle-tags.webp"
        )),
        _ => None,
    }
}

/// A guide screenshot's size as zwipe.net's frame takes it.
pub(crate) fn guide_image_size(file: &str) -> MediaSize {
    let Pixels { width, height } = image_size(file);
    MediaSize { width, height }
}

#[cfg(test)]
mod tests {
    use zwipe_core::content::guides::{GUIDES, screenshots};

    use super::*;

    #[test]
    fn every_screenshot_is_registered() {
        for guide in GUIDES {
            for file in screenshots(guide) {
                assert!(
                    guide_image(file).is_some(),
                    "{}: {file} has no guide_image arm",
                    guide.slug
                );
            }
        }
    }

    /// A WebP's width and height from its header, for the three encodings
    /// (lossy `VP8 `, lossless `VP8L`, extended `VP8X`).
    fn webp_size(bytes: &[u8]) -> (u32, u32) {
        let le = |range: std::ops::Range<usize>| {
            bytes[range]
                .iter()
                .rev()
                .fold(0u32, |n, b| (n << 8) | u32::from(*b))
        };
        let plus_one = |n: u32| n.checked_add(1).unwrap();
        match &bytes[12..16] {
            b"VP8 " => (le(26..28) & 0x3fff, le(28..30) & 0x3fff),
            b"VP8L" => {
                let bits = le(21..25);
                (plus_one(bits & 0x3fff), plus_one((bits >> 14) & 0x3fff))
            }
            b"VP8X" => (plus_one(le(24..27)), plus_one(le(27..30))),
            other => panic!("not a WebP chunk: {other:?}"),
        }
    }

    #[test]
    fn every_screenshot_frame_has_the_file_s_size() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/guides");
        for guide in GUIDES {
            for file in screenshots(guide) {
                let bytes = std::fs::read(root.join(file)).unwrap();
                let Pixels { width, height } = image_size(file);
                assert_eq!(webp_size(&bytes), (width, height), "{file}");
            }
        }
    }
}
