//! zwipe.net answers a path it has no page for with its not-found page and a 404: the build moves
//! the prerendered not-found page to the top-level `404.html`, which Cloudflare Pages serves with
//! that status. Pages stops falling back to the site's shell once `404.html` exists, so the
//! per-token pages get a Function that serves the shell with a 200, and the router's client-side
//! redirects answer as 301s from `_redirects`.

const WORKFLOW: &str = include_str!("../../.github/workflows/deploy-zite.yml");
const REDIRECTS: &str = include_str!("../public/_redirects");
const SHELL: &str = include_str!("../functions/verify/[token].js");
const RESET: &str = include_str!("../functions/reset/[token].js");
const CONFIRM: &str = include_str!("../functions/confirm-email/[token].js");

#[test]
fn the_build_makes_the_not_found_page_the_site_s_404_html() {
    let ssg = WORKFLOW
        .find("dx build --release --platform web --ssg")
        .expect("the SSG build");
    let moved = WORKFLOW
        .find(r#"mv "$public/404/index.html" "$public/404.html""#)
        .expect("the not-found page moves to 404.html");
    assert!(ssg < moved, "after the SSG writes it");
    assert!(
        !WORKFLOW.contains("rm -f site/404.html"),
        "nothing drops it"
    );
}

#[test]
fn the_per_token_pages_get_the_shell_with_a_200() {
    assert!(
        SHELL.contains("export async function onRequest("),
        "every method"
    );
    assert!(SHELL.contains(r#"env.ASSETS.fetch(new URL("/", request.url))"#));
    assert!(SHELL.contains("status: 200"));
    for other in [RESET, CONFIRM] {
        assert!(
            other.contains(r#"export { onRequest } from "../verify/[token].js";"#),
            "{other}"
        );
    }
}

#[test]
fn the_router_s_redirects_answer_as_301s() {
    for line in [
        "/discord /community/ 301",
        "/guides/suggested-commanders /guides/playstyle/ 301",
        "/changelog /about/changelog/ 301",
    ] {
        assert!(REDIRECTS.contains(line), "{line}");
    }
}
