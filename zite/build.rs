#![allow(clippy::expect_used)]

/// Public base URL — must match `WEB_BASE` in `src/main.rs`.
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
    ("/about", "monthly", "0.7"),
    ("/contribute", "monthly", "0.6"),
    ("/discord", "monthly", "0.5"),
    ("/privacy", "yearly", "0.3"),
];

/// Guide article slugs, each rendered at `/guides/<slug>` by `GuidePage`.
/// Keep in sync with `GUIDES` in `src/pages/guides/content.rs`.
const GUIDE_SLUGS: &[&str] = &[
    "getting-started",
    "swipe-to-build",
    "remove-cards",
    "swipe-memory",
    "filtering",
    "synergy",
    "commander-and-formats",
    "budgeting",
    "land-targets",
    "deck-tags",
    "deck-mvps",
    "deck-stats",
    "import-export",
];

fn main() {
    // Copy shared themes into assets so asset!() can find them
    std::fs::copy("../shared/themes.css", "assets/themes.css")
        .expect("failed to copy shared/themes.css into zite/assets/");
    println!("cargo:rerun-if-changed=../shared/themes.css");

    // Copy the shared component styles (zwipe-components) the same way.
    std::fs::copy(
        "../zwipe-components/assets/components.css",
        "assets/components.css",
    )
    .expect("failed to copy zwipe-components/assets/components.css into zite/assets/");
    println!("cargo:rerun-if-changed=../zwipe-components/assets/components.css");

    generate_sitemap();
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
