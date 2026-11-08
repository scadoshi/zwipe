//! A preview's SVG rasterized to PNG with resvg and the embedded fonts.

use std::sync::{Arc, LazyLock};

use resvg::usvg::fontdb::Database;
use resvg::usvg::{ImageKind, Options, Tree};
use resvg::{tiny_skia, usvg};

use super::layout::{HEIGHT, WIDTH};

/// `JetBrains Mono`, the app's text face.
const TEXT_REGULAR: &[u8] =
    include_bytes!("../../../../../assets/share_preview/JetBrainsMono-Regular.ttf");

/// `JetBrains Mono` Bold, for the deck name and the numbers.
const TEXT_BOLD: &[u8] =
    include_bytes!("../../../../../assets/share_preview/JetBrainsMono-Bold.ttf");

/// The Mana font, for the pips' symbols.
const MANA: &[u8] = include_bytes!("../../../../../assets/share_preview/mana.ttf");

/// The embedded fonts, loaded once. Nothing reads the system's fonts, so the
/// image looks the same on every machine.
static FONTS: LazyLock<Arc<Database>> = LazyLock::new(|| {
    let mut db = Database::new();
    for font in [TEXT_REGULAR, TEXT_BOLD, MANA] {
        db.load_font_data(font.to_vec());
    }
    Arc::new(db)
});

/// Why a preview didn't render.
#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    /// The layout's SVG didn't parse.
    #[error("preview svg does not parse: {0}")]
    Svg(#[from] usvg::Error),
    /// The canvas couldn't be allocated.
    #[error("preview canvas could not be made")]
    Canvas,
    /// The PNG encoder failed.
    #[error("preview png does not encode: {0}")]
    Png(String),
}

/// `svg` drawn to a [`WIDTH`]x[`HEIGHT`] PNG. `<image href="art{i}">` takes
/// `images[i]` (JPEG, PNG or WebP); any other href draws nothing, so the SVG
/// can't reach a file or the network.
///
/// # Errors
///
/// Returns [`RenderError`] when the SVG doesn't parse, the canvas can't be
/// allocated or the PNG doesn't encode.
pub fn png(svg: &str, images: Vec<Vec<u8>>) -> Result<Vec<u8>, RenderError> {
    let images: Vec<Arc<Vec<u8>>> = images.into_iter().map(Arc::new).collect();
    let mut options = Options {
        fontdb: Arc::clone(&FONTS),
        font_family: "JetBrains Mono".to_owned(),
        ..Options::default()
    };
    options.image_href_resolver.resolve_data = Box::new(|_, _, _| None);
    options.image_href_resolver.resolve_string = Box::new(move |href, _| {
        let index: usize = href.strip_prefix("art")?.parse().ok()?;
        images.get(index).map(|data| kind(Arc::clone(data)))
    });
    let tree = Tree::from_str(svg, &options)?;
    let mut canvas = tiny_skia::Pixmap::new(WIDTH, HEIGHT).ok_or(RenderError::Canvas)?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut canvas.as_mut());
    canvas
        .encode_png()
        .map_err(|e| RenderError::Png(e.to_string()))
}

/// The image format `data` is in, by its first bytes. Scryfall's art crops
/// are JPEG, so anything unrecognized is tried as JPEG.
fn kind(data: Arc<Vec<u8>>) -> ImageKind {
    if data.starts_with(b"\x89PNG") {
        ImageKind::PNG(data)
    } else if data.starts_with(b"RIFF") && data.get(8..12) == Some(b"WEBP") {
        ImageKind::WEBP(data)
    } else {
        ImageKind::JPEG(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_plain_svg_renders_at_the_preview_size() {
        let svg = format!(
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}"><rect width="10" height="10" fill="#ff0000"/><text x="20" y="40" font-size="30">Hi</text></svg>"##
        );
        let png = png(&svg, Vec::new()).unwrap();
        let decoded = tiny_skia::Pixmap::decode_png(&png).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (WIDTH, HEIGHT));
        let first = decoded.pixel(0, 0).unwrap();
        assert_eq!((first.red(), first.green(), first.blue()), (255, 0, 0));
    }

    #[test]
    fn an_href_outside_the_given_art_draws_nothing() {
        let svg = format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}"><image href="/etc/hostname" width="100" height="100"/><image href="art3" width="100" height="100"/></svg>"#
        );
        let png = png(&svg, Vec::new()).unwrap();
        let decoded = tiny_skia::Pixmap::decode_png(&png).unwrap();
        assert_eq!(
            decoded.pixel(50, 50).unwrap().alpha(),
            0,
            "nothing was drawn"
        );
    }

    #[test]
    fn the_fonts_load() {
        let families: Vec<String> = FONTS
            .faces()
            .flat_map(|face| face.families.iter().map(|(name, _)| name.clone()))
            .collect();
        assert!(
            families.iter().any(|f| f == "JetBrains Mono"),
            "{families:?}"
        );
        assert!(families.iter().any(|f| f == "Mana"), "{families:?}");
    }
}
