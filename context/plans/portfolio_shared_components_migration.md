# Portfolio → shared zwipe-components migration

**Status: PLANNED (2026-07-11).** Targets the **portfolio repo**
(`~/Developer/portfolio`), not this one. Follow-on to
[[zite_banners_changelog]] (its "Piece 3" is superseded/expanded here). Adopts
the shared `Banner`, `Panel`, filled `status-tag`, and `panel-action` button
that now live in `zwipe-components`, deleting the portfolio's hand-rolled
equivalents so all three surfaces (zwiper, zite, portfolio) share one system.

## Prerequisite (blocks everything)

Portfolio pins `zwipe-components` via the **GitHub git dep**, currently rev
`04dd9276…` / version 1.4.0 (`Cargo.lock`; `Cargo.toml` floats the default
branch). None of this can build until the new shared components are on the zwipe
repo's default branch. Then, in the portfolio:

```
cargo update -p zwipe-components   # advance the pinned rev
```

Watch the `getrandom` 0.4 `wasm_js` pin (zwipe-core's uuid pulls it) if the
update bumps transitive deps.

The CSS plumbing already exists — `src/main.rs` inlines `THEMES_CSS` then
`COMPONENTS_CSS` via `document::Style`, then links the site's `main.css`
(cascade: themes → components → site). So the new `.announcement-banner` /
`.panel-*` / filled `.status-tag` rules arrive automatically with the update;
the work is swapping markup to the components and deleting the now-duplicated
local CSS.

## Decisions

- **Arrows, normalized:** external (off-site) links get **↗ `\u{2197}`**;
  internal SPA `Link`s get **→ `\u{2192}`**. Today the portfolio is inconsistent
  (hero uses ↗, cards/banner use → for external). Fix during the swap.
- **External links become `panel-action` pill buttons** in card/CTA contexts.
  Inline-in-prose external links (`linked_text.rs` article refs) **stay text
  links** — a button mid-sentence breaks prose. Footer links: keep as text (or a
  later pass), owner to confirm.
- **Body content stays local.** `Panel` styles only the card chrome (eyebrow,
  title, rules, actions). The summary paragraph, bullets, and impact metric ride
  inside `Panel`'s `children`, so `.card-summary` / `.card-bullets` /
  `.card-impact` CSS is **kept** (it's body styling, not chrome).
- **Detail headers are a later, optional phase** — they use a parallel
  `project-header` class set, not `.project-card`.

## Phase 1 — Banner (`src/pages/home.rs`)

Two inline banners (`Banner` enum + `class()` + two `use_signal` + two
`div.announcement-banner` blocks, lines ~10-100) → shared
`zwipe_components::Banner`. The CTA is passed as `children` (the shared Banner
can't own routing), so the external and internal CTAs both work:

```rust
use zwipe_components::{Banner, BannerStatus};

Banner {
    category: "Announcement",
    status: BannerStatus::Done,
    "Zwipe, the deck builder MTG deserved. "
    a { href: "https://zwipe.net", target: "_blank", rel: "noopener noreferrer",
        "Try it now \u{2197}" }          // was → , now ↗ (external)
}
Banner {
    category: "Featured",
    status: BannerStatus::Doing,
    "Diprotodon, a hand-written Redis-compatible KV server. "
    Link { to: Route::SideQuestDetail { slug: "diprotodon".into() },
           "Check it out \u{2192}" }       // internal, stays →
}
```

- Wrap both in the site's own `div { class: "banner-stack", … }` (positioning
  stays site-owned).
- Delete the local `Banner` enum, `class()`, both `use_signal`s, and the
  outer `if …Dismissed` guard (the shared Banner self-manages its lifecycle).
- **CSS:** delete the local banner rules in `main.css` (~lines 101-233:
  `.announcement-banner`, `.banner-header/-category/-text/-dismiss/-progress`,
  `.banner-leaving`, and the `banner-leave` / `banner-countdown` /
  `banner-slide-in` keyframes). **KEEP** `.banner-stack` positioning (~89-99)
  and its two responsive overrides (`body:has(.nav-panel-open) .banner-stack`
  at ~1043-1049 and ~1064-1068) — these are site-specific placement the shared
  component deliberately doesn't own.

## Phase 2 — Panel (the cards)

Three card call sites collapse onto `zwipe_components::Panel`
(`eyebrow` + `title` + `status` → rule → `children` body → rule → `actions`):

| Call site | Variant | Notes |
|---|---|---|
| `src/components/project_card.rs` | eyebrow+status / summary / bullets / **impact** / 2 actions | fullest |
| `src/pages/side_quests.rs` (inline) | same, **no impact** | dedupe: reuse `ProjectCard` or Panel directly |
| `src/pages/contribute.rs` (×3) | eyebrow only (no status), summary, **1 external action** | simplest |

Example (`project_card.rs`):

```rust
use zwipe_components::{Panel, BannerStatus};

Panel {
    eyebrow: "{category}",
    title: "{name}",
    status: /* map data::Status -> BannerStatus */,
    actions: rsx! {
        Link { to: Route::ProjectDetail { slug }, class: "panel-action",
               "View Project \u{2192}" }
        a { class: "panel-action", href: "{repo_url}",
            target: "_blank", rel: "noopener noreferrer", "GitHub \u{2197}" }
    },
    p { class: "card-summary", "{summary}" }
    ul { class: "card-bullets", for b in bullets { li { "{b}" } } }
    div { class: "card-impact", "{impact_metric}" }
}
```

- **Status mapping:** portfolio's `data::Status` currently yields
  `status_class` / `status_label` strings. Map it to `BannerStatus`
  (Done→`Done`, Doing→`Doing`) and pass `status_label` if the label differs
  from the pill default. (Portfolio only uses done/doing today.)
- **Contribute cards:** omit `status`; single external `panel-action` with ↗.
- **CSS deletions in `main.css`:** `.project-card` (+`:hover`) → `.panel-card`;
  `.card-category` → `.panel-eyebrow`; `.card-title` → `.panel-title`
  (note: shared is 1.1rem vs local 1.2rem — accept or override); `.card-actions`
  + `.card-link` + `.card-link-secondary` → `.panel-actions` / `.panel-action`;
  `.status-tag` / `.status-done` / `.status-doing` (now shared, filled). **KEEP**
  `.card-summary`, `.card-bullets` (+`::before`), `.card-impact` — body content.

## Phase 3 — External-link buttons elsewhere

Apply the "external → `panel-action` ↗" rule outside cards where it reads well:

- **Hero links** (`home.rs` GitHub/LinkedIn/Email): already pill-ish via
  `.hero-links a`. Either re-point to `.panel-action` (and drop `.hero-links a`
  CSS) or leave as-is — cosmetic. Owner to pick.
- **Footer** (`footer.rs` GitHub/LinkedIn/Email/This site): plain text today.
  Leave as text links, or a light pass to `panel-action` — **owner decision**.
- **`linked_text.rs`** (article/bare URLs inside prose): **keep as text links**
  (buttons break inline flow). Just ensure they read as external.

## Phase 4 — Detail headers (optional)

`src/pages/project_detail.rs` and `src/pages/side_quest_detail.rs` are
**byte-for-byte identical** headers (only the data lookup differs) using
`project-header` / `project-category-tag` / `project-name` / `project-headline`
/ `repo-link` / `tag-row`. Two options:
1. **Minimal:** convert the `repo-link` external anchor to `panel-action` ↗ and
   let the shared filled `status-tag` take over; keep the header classes. Also
   note the shared `.status-tag` has no `margin-left` (uses eyebrow-row gap), so
   the inline pill in `.project-category-tag` may need a small local margin.
2. **Full:** fold both into one shared component (they're duplicates) rendered
   with `Panel` in a header layout. Bigger refactor; defer unless desired.

## Ship / verify

1. Land shared components on the zwipe default branch (other session /
   [[zite_banners_changelog]]), then `cargo update -p zwipe-components`.
2. Phase 1 → 2 → 3 in the portfolio; build with `dx serve`, eyeball each page
   (home banners, projects grid, side quests, contribute, a detail page) in a
   couple themes + light/dark.
3. Grep for now-dead CSS after each phase and delete; confirm no class is
   referenced by both a deleted local rule and live markup.
4. No server, no data changes — pure client/CSS. Portfolio deploys on its own
   pipeline (separate repo).
