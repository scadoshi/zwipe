//! zwipe.net's Changelog lives at /about/changelog/, linked from the About page beside Privacy
//! policy and from the footer, not the top nav, and its first address /changelog moves there
//! for good.

/// zwipe.net's root, with the nav and the footer.
const MAIN: &str = include_str!("../src/main.rs");
/// The About page.
const ABOUT: &str = include_str!("../src/pages/about.rs");
/// The site's routes.
const ROUTES: &str = include_str!("../src/route.rs");
/// The changelog page.
const PAGE: &str = include_str!("../src/pages/changelog.rs");
/// Cloudflare Pages' redirects, served from the site's root.
const REDIRECTS: &str = include_str!("../public/_redirects");
/// The sitemap `build.rs` writes.
const SITEMAP: &str = include_str!("../public/sitemap.xml");

/// The text of `src` from the first `start` to the first `end` after it.
fn between<'a>(src: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let (_, rest) = src.split_once(start)?;
    let (inside, _) = rest.split_once(end)?;
    Some(inside)
}

#[test]
fn the_top_nav_has_no_changelog() {
    let links = between(MAIN, "links: rsx! {", "trailing: rsx! {").expect("the nav's links");
    assert!(links.contains("Route::About {}"), "{links}");
    assert!(!links.contains("Route::Changelog {}"), "{links}");
}

#[test]
fn the_footer_lists_changelog_beside_privacy_community_and_contribute() {
    let footer = between(MAIN, "::components::SiteFooter {", "}\n    }\n}").expect("the footer");
    for page in ["Privacy", "Changelog", "Community", "Contribute"] {
        assert!(
            footer.contains(&format!(
                r#"Link {{ to: Route::{page} {{}}, class: "nav-cta""#
            )),
            "{page} in {footer}"
        );
    }
}

#[test]
fn about_links_changelog_beside_privacy_policy() {
    let actions = between(ABOUT, "actions: rsx! {", "\n                    },")
        .expect("the About header's actions");
    let privacy = actions
        .find(r#"Link { class: "panel-action", to: Route::Privacy {}"#)
        .expect("Privacy policy");
    let changelog = actions
        .find(r#"Link { class: "panel-action", to: Route::Changelog {}, "Changelog" }"#)
        .expect("Changelog");
    assert!(
        changelog > privacy,
        "Changelog follows Privacy policy: {actions}"
    );
}

#[test]
fn the_changelog_lives_under_about_and_its_old_address_moves_there_for_good() {
    assert!(
        ROUTES.contains("#[route(\"/about/changelog/\")]\n    Changelog {},"),
        "{ROUTES}"
    );
    assert!(
        ROUTES.contains("#[redirect(\"/changelog\", || Route::Changelog {})]"),
        "a link followed inside the site moves too"
    );
    assert!(
        PAGE.contains("path: \"/about/changelog/\","),
        "the canonical"
    );
    let rules: Vec<Vec<&str>> = REDIRECTS
        .lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| line.split_whitespace().collect())
        .collect();
    for from in ["/changelog", "/changelog/"] {
        assert!(
            rules.contains(&vec![from, "/about/changelog/", "301"]),
            "{from} answers 301: {rules:?}"
        );
    }
    assert!(SITEMAP.contains("<loc>https://zwipe.net/about/changelog/</loc>"));
    assert!(!SITEMAP.contains("<loc>https://zwipe.net/changelog/</loc>"));
}
