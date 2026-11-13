//! The screenshots zwipe.net serves from `public/guide-images/`, checked
//! against the guides in `zwipe_core::content::guides` and their frames.

use zwipe_core::content::guides::{GUIDES, Pixels, image_size, screenshots};

#[test]
fn every_screenshot_is_served_from_public() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("public/guide-images");
    for guide in GUIDES {
        for file in screenshots(guide) {
            assert!(
                root.join(file).is_file(),
                "{}: no public/guide-images/{file}",
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
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("public/guide-images");
    for guide in GUIDES {
        for file in screenshots(guide) {
            let bytes = std::fs::read(root.join(file)).unwrap();
            let Pixels { width, height } = image_size(file);
            assert_eq!(webp_size(&bytes), (width, height), "{file}");
        }
    }
}
