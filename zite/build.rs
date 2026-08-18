#![allow(clippy::expect_used)]

/// Public base URL — must match the release `WEB_BASE` in zwipe-core's
/// `domain::site` (build scripts can't import the lib, so this is the one
/// mirrored literal).
const WEB_BASE: &str = "https://zwipe.net";

/// Every prerendered, indexable route with its sitemap hints. Kept here as the
/// single source of truth so `public/sitemap.xml` can't drift from the routes
/// the way a hand-edited file does. Dynamic routes (`/verify/:token`,
/// `/reset/:token`) are excluded — they're `Disallow`ed in robots.txt.
/// `(path, changefreq, priority)`.
const ROUTES: &[(&str, &str, &str)] = &[
    ("/", "weekly", "1.0"),
    ("/download/ios", "weekly", "0.9"),
    ("/download/android", "weekly", "0.9"),
    ("/guides", "weekly", "0.8"),
    ("/changelog", "weekly", "0.6"),
    ("/about", "monthly", "0.7"),
    ("/contribute", "monthly", "0.6"),
    ("/discord", "monthly", "0.5"),
    ("/privacy", "yearly", "0.3"),
];

/// Guide article slugs, each rendered at `/guides/<slug>` by `GuidePage`.
/// Checked against `GUIDES` in `src/pages/guides/content.rs` at build time by
/// [`assert_guide_slugs_match`], so a new guide can't quietly miss the sitemap
/// the way three of them did before 2026-08-18.
const GUIDE_SLUGS: &[&str] = &[
    "getting-started",
    "swipe-to-build",
    "remove-cards",
    "swipe-memory",
    "filtering",
    "organize-and-browse",
    "synergy",
    "commander-and-formats",
    "commander-maybeboard",
    "budgeting",
    "land-targets",
    "deck-tags",
    "oracle-tags",
    "oracle-tag-dictionary",
    "card-roles",
    "tags-roles-and-oracle-tags",
    "deck-mvps",
    "share-your-deck",
    "deck-stats",
    "import-export",
];

fn main() {
    assert_guide_slugs_match();
    generate_sitemap();
}

/// Fail the build if [`GUIDE_SLUGS`] drifts from the `GUIDES` array. A guide
/// missing here is invisible to search engines, and the omission is silent:
/// the page still renders, so nothing else catches it.
fn assert_guide_slugs_match() {
    const CONTENT: &str = "src/pages/guides/content.rs";
    println!("cargo:rerun-if-changed={CONTENT}");
    let src = std::fs::read_to_string(CONTENT).expect("failed to read guides content.rs");

    let mut defined: Vec<&str> = src
        .split("slug: \"")
        .skip(1)
        .filter_map(|rest| rest.split('"').next())
        .collect();
    defined.sort_unstable();
    defined.dedup();

    let mut listed: Vec<&str> = GUIDE_SLUGS.to_vec();
    listed.sort_unstable();

    let missing: Vec<&&str> = defined.iter().filter(|s| !listed.contains(s)).collect();
    let extra: Vec<&&str> = listed.iter().filter(|s| !defined.contains(s)).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "GUIDE_SLUGS is out of sync with GUIDES in {CONTENT}.\n  \
         missing from the sitemap: {missing:?}\n  \
         listed but not a real guide: {extra:?}"
    );
}

fn generate_sitemap() {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
    );
    for (path, changefreq, priority) in ROUTES {
        xml.push_str(&format!(
            "  <url>\n    <loc>{WEB_BASE}{path}</loc>\n    \
             <changefreq>{changefreq}</changefreq>\n    \
             <priority>{priority}</priority>\n  </url>\n",
        ));
    }
    for slug in GUIDE_SLUGS {
        xml.push_str(&format!(
            "  <url>\n    <loc>{WEB_BASE}/guides/{slug}</loc>\n    \
             <changefreq>monthly</changefreq>\n    \
             <priority>0.6</priority>\n  </url>\n",
        ));
    }
    xml.push_str("</urlset>\n");

    std::fs::write("public/sitemap.xml", xml).expect("failed to write public/sitemap.xml");
    println!("cargo:rerun-if-changed=build.rs");
}
