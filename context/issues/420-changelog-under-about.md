# 420. zwipe.net's Changelog sits in the top nav

State: fixing | Owner: thinker | PR: - | Build seen: production

## Report
Composer for Scotty, 2026-11-12: on zwipe.net, take Changelog out of the top nav, link it from the About page beside Privacy policy (the same button as Source, Built by Scotty and Privacy policy) and from the footer beside Privacy policy, Community and Contribute. Scotty's addition: the page moves to /about/changelog/, and the old /changelog, live for a long time, answers with a permanent 301 there, old release anchors included. Sitemap, canonical and internal links follow.

## Fix
thinker:
- `zite/src/main.rs`: the nav drops Changelog; the footer's pills are Privacy policy, Changelog, Community and Contribute.
- `zite/src/pages/about.rs`: About's header actions end with Changelog after Privacy policy.
- `zite/src/route.rs`: Changelog is `/about/changelog/`, with a router redirect from `/changelog` for links followed inside the site; the page's canonical is `/about/changelog/`.
- `zite/public/_redirects`: Cloudflare Pages answers `/changelog` and `/changelog/` with a 301 to `/about/changelog/`. A browser keeps the `#v1-…` fragment across it, so old anchors land on their release.
- `build.rs` lists `/about/changelog/` for the sitemap in place of `/changelog/`. No other code links to zwipe.net/changelog (the app, zwipe-ui and the guides don't); `release_notes_guidelines.md` names the new address.
- `zite/tests/changelog_link.rs` checks the nav, footer, About actions, route, canonical, 301 rules and sitemap.

## Verification
- After deploy: `curl -I https://zwipe.net/changelog` gives 301 with `location: /about/changelog/`.
