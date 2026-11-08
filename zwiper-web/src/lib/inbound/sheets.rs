//! The app's stylesheets, linked in `index.html`'s head at build.
//!
//! dx writes a render-blocking `<link rel="stylesheet">` into the built
//! `index.html` for every CSS asset declared `with_static_head`, under its
//! hashed name, so the browser holds every rule before it paints the first
//! frame. A sheet the wasm inserts at runtime (`document::Style`) lands after
//! the markup has painted, and the page flashes unstyled.
//!
//! dx bundles only files inside this crate, so the shared sheets (orangui's
//! and components') are copied into `assets/sheets`, and a test keeps each
//! copy equal to the string it copies. After a change to one of them, or a
//! `cargo update -p orangui`, rewrite the copies with
//! `ZWIPE_UPDATE_SHEETS=1 cargo test -p zwiper-web sheets`.
//!
//! dx writes the links in the order of the files' paths, so the file names
//! carry the cascade order: the fonts, orangui's themes and components, the
//! nav's fold under this app's width (`chrome::media::nav_fold_css`), the
//! Magic UI, orangui's app sheet, the screens, the toasts, the filter, and
//! last `web.css`.

use dioxus::prelude::*;

/// Every stylesheet the app uses, in cascade order, each linked in the head
/// of the built `index.html`.
pub const SHEETS: [Asset; 10] = [
    asset!(
        "/assets/sheets/01-fonts.css",
        AssetOptions::css().with_static_head(true)
    ),
    asset!(
        "/assets/sheets/02-themes.css",
        AssetOptions::css().with_static_head(true)
    ),
    asset!(
        "/assets/sheets/03-components.css",
        AssetOptions::css().with_static_head(true)
    ),
    asset!(
        "/assets/sheets/04-nav-fold.css",
        AssetOptions::css().with_static_head(true)
    ),
    asset!(
        "/assets/sheets/05-magic.css",
        AssetOptions::css().with_static_head(true)
    ),
    asset!(
        "/assets/sheets/06-app.css",
        AssetOptions::css().with_static_head(true)
    ),
    asset!(
        "/assets/sheets/07-screens.css",
        AssetOptions::css().with_static_head(true)
    ),
    asset!(
        "/assets/sheets/08-toast.css",
        AssetOptions::css().with_static_head(true)
    ),
    asset!(
        "/assets/sheets/09-filter.css",
        AssetOptions::css().with_static_head(true)
    ),
    asset!(
        "/assets/web.css",
        AssetOptions::css().with_static_head(true)
    ),
];

/// The fonts `01-fonts.css` names, bundled without a hash suffix so its
/// relative urls and `index.html`'s preloads can name them. Their contents
/// never change, so they need no cache-busting name.
pub const FONTS: [Asset; 3] = [
    asset!(
        "/assets/fonts/jetbrains-mono-400.woff2",
        AssetOptions::builder().with_hash_suffix(false)
    ),
    asset!(
        "/assets/fonts/jetbrains-mono-500.woff2",
        AssetOptions::builder().with_hash_suffix(false)
    ),
    asset!(
        "/assets/fonts/jetbrains-mono-700.woff2",
        AssetOptions::builder().with_hash_suffix(false)
    ),
];

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::*;
    use crate::inbound::chrome::media;

    /// Each copied sheet's file in `assets/sheets`, with the css it copies.
    fn copies() -> [(&'static str, String); 8] {
        [
            ("02-themes.css", components::THEMES_CSS.to_owned()),
            ("03-components.css", components::COMPONENTS_CSS.to_owned()),
            ("04-nav-fold.css", media::nav_fold_css()),
            ("05-magic.css", components::MAGIC_CSS.to_owned()),
            ("06-app.css", components::APP_CSS.to_owned()),
            ("07-screens.css", components::SCREENS_CSS.to_owned()),
            ("08-toast.css", components::TOAST_CSS.to_owned()),
            ("09-filter.css", components::FILTER_CSS.to_owned()),
        ]
    }

    fn crate_dir() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn the_copied_sheets_match_the_shared_css() {
        let dir = crate_dir().join("assets/sheets");
        let update = std::env::var_os("ZWIPE_UPDATE_SHEETS").is_some();
        let mut stale = Vec::new();
        for (file, css) in copies() {
            let path = dir.join(file);
            if update {
                std::fs::write(&path, &css).unwrap();
            } else if std::fs::read_to_string(&path).ok() != Some(css) {
                stale.push(file);
            }
        }
        assert!(
            stale.is_empty(),
            "{stale:?} differ from the shared css; run ZWIPE_UPDATE_SHEETS=1 cargo test -p zwiper-web sheets"
        );
    }

    #[test]
    fn every_sheet_is_linked_in_the_head_in_cascade_order() {
        let sources: Vec<PathBuf> = SHEETS
            .iter()
            .map(|sheet| PathBuf::from(sheet.bundled().absolute_source_path()))
            .collect();
        let mut by_path = sources.clone();
        by_path.sort();
        assert_eq!(
            sources, by_path,
            "dx links sheets by path: rename to keep the cascade"
        );
        for sheet in SHEETS {
            let options = *sheet.bundled().options();
            assert!(
                matches!(options.variant(), AssetVariant::Css(css) if css.static_head()),
                "{} is not linked in the head",
                sheet.bundled().absolute_source_path()
            );
        }
        let linked: Vec<&str> = std::iter::once("01-fonts.css")
            .chain(copies().into_iter().map(|(file, _)| file))
            .collect();
        let mut on_disk: Vec<String> = std::fs::read_dir(crate_dir().join("assets/sheets"))
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        on_disk.sort();
        assert_eq!(on_disk, linked, "every file in assets/sheets is linked");
    }

    #[test]
    fn nothing_inserts_a_stylesheet_at_runtime() {
        let mut offenders = Vec::new();
        let mut dirs = vec![crate_dir().join("src")];
        while let Some(dir) = dirs.pop() {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    dirs.push(path);
                } else if path.extension().is_some_and(|e| e == "rs")
                    && !path.ends_with("sheets.rs")
                {
                    let source = std::fs::read_to_string(&path).unwrap();
                    for needle in [
                        "document::Style",
                        "document::Stylesheet",
                        "rel: \"stylesheet\"",
                    ] {
                        if source.contains(needle) {
                            offenders.push(format!("{} uses {needle}", path.display()));
                        }
                    }
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "a sheet inserted at runtime paints after the markup; declare it in SHEETS instead:\n{}",
            offenders.join("\n")
        );
    }
}
