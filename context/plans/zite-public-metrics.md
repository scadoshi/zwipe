# Zite public stats strip + CF cache rule

## Context

The backend half of public marketing metrics shipped on the
`feat/user-metrics` branch (this session):

- `GET /api/marketing/stats` returns `{ cards_swiped, searches, decks_created }`
- Public, no auth, IP rate-limited (30 req/2s)
- Numbers come from a single `SUM(...)` over `user_lifetime_counters`
- Aggregates are sub-millisecond at the origin

What's deliberately **not** in that branch:

1. The Cloudflare Cache Rule that fronts the endpoint at the edge.
2. The zite component that consumes the endpoint and renders the
   numbers on zwipe.net.

The deferral lets us ship the API surface cleanly, watch it behave in
prod for a bit, then add the public-facing pieces in a small follow-up
PR with its own review.

Intended outcome: zwipe.net's hero shows a three-stat strip
("X cards swiped · Y searches run · Z decks created") backed by the
existing endpoint, fronted by CF so origin gets one hit per POP per
~2 hours.

## On the CF piece — what it actually is

There's nothing to build in Cloudflare. It's one dashboard rule:

- **Caching → Cache Rules → Create rule**
- **Name**: `Cache marketing aggregates`
- **Condition**: `starts_with(http.request.uri.path, "/api/marketing/")`
- **Action**: Eligible for cache · Ignore origin Cache-Control · Edge
  TTL **2 hours** (free-plan minimum — see note below)

CF reads the response body from origin on the first miss per POP, stores
it at the edge for 2 hours, and serves cached bytes on every subsequent
hit. No CF Worker, no KV, no aggregation in CF — just a response-body
cache. The SUM query at origin runs roughly (POP count × 12) times a
day total, regardless of how many people load the page.

**Why 2h and not 1h:** Cloudflare's free plan enforces a 2h minimum for
custom Edge TTLs on Cache Rules. Functionally indistinguishable for our
use case — vanity totals don't move meaningfully inside a 2h window
anyway. If a launch milestone ever needs immediate refresh, CF API
purge-by-URL clears the cache in seconds.

The same path-prefix rule covers any future `/api/marketing/*`
endpoints, so this single rule is the only ops work needed even if more
public metrics get added later.

Already done by user on 2026-06-08 (per session note). Verify with:

```bash
curl -sI https://api.zwipe.net/api/marketing/stats | grep -i cf-cache-status
# first hit on a cold POP: MISS
# subsequent hits within the POP: HIT
```

## Branch

Cut a fresh `feat/zite-public-stats` branch from `main` **after**
`feat/user-metrics` has merged. Don't bundle this with the backend
branch — keeps reviews scoped and the zerver deploy independent of any
zite static-site rebuild.

## Design decisions (defaults with override notes)

| Decision | Default | Notes |
|---|---|---|
| **Where the strip lives** | Home page hero, below the tagline | Inside the existing `.hero` block. Single horizontal strip, centered, three stats. Alternative: dedicated `/stats` route — overkill for three numbers. |
| **Fetch timing** | Server-render fetch via `use_resource` | Zite uses Dioxus fullstack with incremental SSR. `use_resource` runs once on the server during prerender; the value lands in the HTML. Client doesn't re-fetch. Pairs perfectly with CF: the rendered HTML is also cached by GitHub Pages, so the visitor's browser never even sees the API call. |
| **Failure mode** | Hide the strip silently | If the fetch returns non-200 or fails to parse, render nothing. Marketing page must never break on a metrics outage. |
| **Number formatting** | Commas every 3 digits | "12,345 cards swiped". No abbreviation (`12K`), no animation. Use a tiny pure-Rust helper, no extra crate. |
| **Labels** | `cards swiped` · `searches run` · `decks created` | Lowercase, lighter weight than the numbers. Matches the rest of the page voice. |
| **Styling** | Inline with `style.css` site theme | Uses existing CSS variables (`--accent-primary`, `--text-primary`). Numbers are bold + accented; labels are small + muted. |
| **Refresh behavior** | None on the client | The page is static once SSR'd. To refresh, the page rebuilds (GH Pages deploy) or the visitor reloads. Acceptable for vanity numbers updated hourly. |

## File-by-file plan

All paths are relative to the repo root.

### 1. New component

**New file:** `zite/src/components/stats_strip.rs`

```rust
//! Live aggregate stats strip surfaced on the marketing site.
//!
//! Fetches `GET {API_BASE}/api/marketing/stats` during SSR. CF caches the
//! API response at the edge (~2h TTL), GH Pages caches the rendered HTML,
//! so cost-per-pageview is near zero. On error the strip hides itself —
//! don't break the marketing page on a metrics outage.

use crate::API_BASE;
use dioxus::prelude::*;
use zwipe_core::http::contracts::metrics::HttpPublicMetrics;
use zwipe_core::http::paths::public_metrics_route;

#[component]
pub fn StatsStrip() -> Element {
    let stats: Resource<Option<HttpPublicMetrics>> = use_resource(|| async {
        let url = format!("{}{}", API_BASE, public_metrics_route());
        let res = reqwest::Client::new().get(&url).send().await.ok()?;
        if !res.status().is_success() {
            return None;
        }
        res.json::<HttpPublicMetrics>().await.ok()
    });

    let value = stats.read();
    let Some(Some(s)) = &*value else {
        return rsx! {};
    };

    rsx! {
        section { class: "stats-strip",
            div { class: "stat",
                span { class: "stat-num", "{format_count(s.cards_swiped)}" }
                span { class: "stat-label", "cards swiped" }
            }
            div { class: "stat",
                span { class: "stat-num", "{format_count(s.searches)}" }
                span { class: "stat-label", "searches run" }
            }
            div { class: "stat",
                span { class: "stat-num", "{format_count(s.decks_created)}" }
                span { class: "stat-label", "decks created" }
            }
        }
    }
}

/// 12345 → "12,345"
fn format_count(n: i64) -> String {
    let s = n.abs().to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    if n < 0 {
        out.push('-');
    }
    out.chars().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::format_count;

    #[test]
    fn formats_small_numbers() {
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(42), "42");
        assert_eq!(format_count(999), "999");
    }

    #[test]
    fn formats_thousands() {
        assert_eq!(format_count(1_000), "1,000");
        assert_eq!(format_count(12_345), "12,345");
        assert_eq!(format_count(123_456), "123,456");
        assert_eq!(format_count(1_234_567), "1,234,567");
    }
}
```

### 2. Component export

**Edit:** `zite/src/components/mod.rs`

```rust
mod page_meta;
mod stats_strip;
pub use page_meta::PageMeta;
pub use stats_strip::StatsStrip;
```

### 3. Wire into home page

**Edit:** `zite/src/pages/home.rs`

- Import: `use crate::components::{PageMeta, StatsStrip};`
- Drop `StatsStrip {}` into the `.hero` div, after the tagline `p`:

```rust
div { class: "hero",
    div { class: "logo", "{LOGO_ASCII}" }
    p { class: "tagline", /* ... */ }
    StatsStrip {}
}
```

### 4. Styling

**Edit:** `zite/assets/style.css` — append after the `.hero .tagline`
block (around line 549):

```css
.stats-strip {
    display: flex;
    justify-content: center;
    gap: 2.5rem;
    flex-wrap: wrap;
    margin-top: 1.5rem;
    padding: 0 1rem;
}

.stats-strip .stat {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.15rem;
}

.stats-strip .stat-num {
    font-size: 1.6rem;
    font-weight: 600;
    color: var(--accent-primary);
    line-height: 1.1;
    font-variant-numeric: tabular-nums;
}

.stats-strip .stat-label {
    font-size: 0.75rem;
    color: var(--text-primary);
    letter-spacing: 0.06em;
    opacity: 0.7;
    text-transform: lowercase;
}
```

### 5. Verification

```bash
# 1. Workspace builds + tests pass (includes format_count unit tests)
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings

# 2. Confirm CF rule is live (should already be done before this branch)
curl -sI https://api.zwipe.net/api/marketing/stats | grep -i cf-cache-status
# expect: HIT (or MISS on first hit per POP, then HIT)

# 3. Local zite dev server
cd zite && dx serve
# open http://localhost:8080, confirm the strip renders below the tagline
# with real numbers from prod

# 4. Mobile width sanity
# The strip uses flex-wrap, so on a phone-narrow viewport the three stats
# stack vertically instead of cramming horizontally.

# 5. After GH Pages deploy
# Visit zwipe.net. View source — the numbers should be inline in the HTML
# (SSR'd), not blank with a client-side fetch holding it up.
```

## Files modified / created

### New

- `zite/src/components/stats_strip.rs`

### Touched

- `zite/src/components/mod.rs` — submodule + re-export
- `zite/src/pages/home.rs` — import + render `StatsStrip {}`
- `zite/assets/style.css` — `.stats-strip` + `.stat` styles

## Out of scope (deferred)

- **More metrics on the strip** — user count, DAU/MAU, completion rate.
  Already decided these stay internal.
- **Per-deck completion percentage** — derivable from
  `decks_completed / decks_created` but reads ambiguous ("87% finish
  their decks" sounds great until someone reads it as "13% of users
  fail"). Skip.
- **Historical chart on the marketing site** — line graph of growth
  over time using `user_daily_activity`. Different surface, different
  visual treatment, different review.
- **Cache-busting on demand** — if we want to ship a milestone post and
  refresh the page immediately, the CF API supports purge-by-URL.
  Trivial to add when needed.
- **Animated count-up on page load** — would be cute, not load-bearing.
  Skip unless we end up with a marketing redesign.
