# Spec 2: Home, Activity, Playstyle, Commander maybeboard, Oracle tag dictionary and examples

Source of truth: `zwiper/src/lib/inbound/` in `the repo root`, plus `components/src` (zwipe UI kit) and `the orangui repo/src` (generic kit). Every quoted string below is copied verbatim from the code. "TOAST_QUICK" = 3 s, "TOAST_NORMAL" = 5 s, "TOAST_LONG" = 8 s (`orangui/src/toast_timing.rs`).

## 0. Conventions shared by every screen here

### 0.1 Screen frame

- Header (`ScreenHeader` → orangui `PageHeader`): a Support button on the left, the title centered, and a "?" on the right only when the screen passes a one-time hint signal. The "?" reopens that hint dialog.
- Content column: `max-width: 40rem; width: 100%; padding: 0 1rem` on Activity, Playstyle and Commander maybeboard. Sections stack with `--panel-gap`.
- Action bar at the bottom (`ActionBar` of `Button { variant: Action }`).

### 0.2 One-time hints (`use_one_time_hint(key)`, `components/hint_dialog.rs`)

- On mount, if `session.user.has_seen_hint(key)` is false, the dialog opens at once, and the app fires `client.mark_hint_shown(key, &session) -> Result<User, ClientError>` (PATCH `/api/user/hint`). On success the returned `User` replaces `session.user` and the session is saved to disk. Failure is only logged (the hint may auto-open once more later).
- Keys (`zwipe_core::domain::user::models::hints`): `HINT_FIRST_LOGIN = "first_login"`, `HINT_ACTIVITY = "activity"`, `HINT_SUGGESTED_COMMANDERS = "suggested_commanders"`, `HINT_COMMANDER_MAYBEBOARD = "commander_maybeboard"`, `HINT_ORACLE_TAG_DICTIONARY = "oracle_tag_dictionary"`, `HINT_OTAG_EXAMPLES = "otag_examples"`.
- `HintDialog` renders a title, optional `HintLine`s and a `HintBullets` list, and one button "Got it". `HintKey { "X" }` is a highlighted key word (optionally colored by a CSS var). `HintColored { color, "x" }` is a colored, weight-600 word.

### 0.3 "?" info buttons (`InfoButton { topic }`, `components/hint_host.rs`)

A "?" posts a `HintTopic` to the app-root `HintHost`, which opens an `AlertDialog` with the topic title and body and a single "Got it" button. Only one at a time. The topics used on these screens, verbatim:

| Topic | Title | Body |
|---|---|---|
| `ColorMap` | Color map | Bullets: "Each tile stands for a share of the colored cards across your decks" / "Every color grows out from the middle, and the color with the most cards claims the most ground" / "The shape is generated per player" / "Colorless cards are in the key below" |
| `SuggestionRoleChart` | Card roles | Line: "How many of your decks have a tag in each role. Tap a role for its tags" |
| `SuggestionTagChart` | Oracle tags | Line: "Your 10 most impactful tags, highest first. Tap a tag to see what it means" |
| `SuggestionTagDecks` | Decks | Line: "How many of your decks have this tag. The more decks, the higher its impact" |
| `SuggestionTagCards` | Cards | Line: "How many cards carry this tag across those decks. The more cards, the higher its impact" |
| `SuggestionTagImpact` | Impact | Line: "How much more your decks lean on this tag than Magic cards do overall. Your top tag scores 100" |
| `SuggestionList` | Suggested commanders | Bullets: "Commanders you haven't built, ranked against all your decks, best fit first" / "Tap a row to see why it fits. " + key "Swipe" + " goes further down the same ranking" |
| `SuggestionStaples` | Staples | Bullets: "The cards in the most of your decks, with the share of your decks that run each" / "Basic lands are left out. Tap a Type or Color chip to narrow it" |
| `Activity` | Activity | Bullets: "Your swipes and searches each day over the last year, a darker square for a busier day" / "A lit square is one of your biggest days. Tap a square to see its counts" / key "Totals" + " is what the cards dealt to your decks came to: " + key "Added" + " and " + key "Maybeboard" + " you kept, " + key "Skipped" + " you passed on, and " + key "Removed" + " you later took out" |
| `RoleRates` | Card roles | Bullets: "Your rate for each card role. Pick one under " + key "Show" + ": " + key "Added" + ", " + key "Skipped" + " and " + key "Maybeboard" + " are shares of the cards dealt to you, and " + key "Removed" + " is the share of added cards you later took out" / "Tap a role to see the tags you swiped under it" / "From your decks with a commander, for roles dealt at least 10 times" |
| `TagRates` | Oracle tags | Bullets: "The tags where your rate stands out most from other players', with your own rate" / "A tag everyone passes on shows only after a lot of skips. Tap a tag to see what it means" / "From your decks with a commander, for tags dealt at least 10 times" |
| `HardPasses` | Most skipped | Bullets: "Cards you skip and never add or save, when other players usually keep them" / "A card everyone passes on shows only after a lot of skips. The tag counts yours, and an open row names the commanders you skipped it under" / "From your decks with a commander" |

### 0.4 The authed call facade (`components/auth/authed.rs`)

Every authed call goes through `use_authed(screen)`:

- First `session.ensure_fresh(client)` (refreshes the access token if needed; an auth rejection clears the session and the AuthGate sends the user to login).
- `run(op, f)`: on any failure, logs, reports to telemetry and shows an error toast with `ClientError::to_user_message()` for TOAST_NORMAL; yields `None`.
- `try_run(op, f)`: same report and toast, but returns the `Err`.
- `run_quiet(op, f)`: logs and reports only, no toast.

### 0.5 Collapsible section accordion (`screens/deck/components/collapsible_section.rs`)

- A bordered card. Header row: a disclosure arrow "▸" (rotated 90° when open), the title, optional badge, and an optional right-side accessory (here always a "?"). The accessory fades in (opacity 0.25 s) only while the section is open and is not clickable while closed.
- Tapping anywhere on the card except the body and the accessory toggles it. One shared `open_section: Signal<Option<String>>` (keyed by title) means only one section is open at a time; opening another closes the first; tapping the open one closes all.
- Body eases open and shut (grid rows 0fr↔1fr plus opacity). Under the header a full-bleed 1px rule, then the children stacked with a 0.75rem gap. The body stays mounted while closed.

### 0.6 BarChart (`components/src/charts.rs`)

- `Bar { key, label, count, pct (0..=100), revealable }`.
- Optional `ChartLabel` title chip above the bars.
- Vertical: columns 6rem tall, count above each bar (only when count > 0, with `count_suffix`), bar height = pct%, labels in a row below. Labels uppercase by default.
- Horizontal, no `on_bar`: rows of label (5ch, right-aligned, uppercase), a track filled to pct%, then the count (3ch, or 4ch with a suffix).
- Horizontal with `on_bar` ("chip rows"): a grid; the label column is as wide as the widest chip (at most 50%). A revealable bar's label is a tappable `keyword-chip` (uppercased unless `LabelCase::AsIs`), "active" when its key equals `selected`. A non-revealable bar's label is plain text. Under every row a reveal region spans the full width; the selected row's reveal eases open (0.2 s height and opacity) showing `reveal(key)`. Closing keeps the last content while it eases shut. One row open at a time.
- `DeckMetrics::bar_pct(count, max)` (zwipe-core): `0` when count or max is 0, else `max(count*100/max, 4)` (a 4% sliver floor).

### 0.7 Oracle tag reveal pieces (`components/src/role_tags.rs`)

- `OtagDef { slug }`: a paragraph with the tag's description from the oracle tag catalog, or "No description yet" when none, then an "Examples" button (only when the host provides an examples opener) that opens the Oracle tag examples overlay for that slug.
- `RoleTags { tags }`: the tags as `keyword-chip`s; tapping one toggles its `OtagDef` open beneath the chips (one at a time; 0.2 s ease). Renders nothing when `tags` is empty.
- Every screen here that shows tags provides two contexts: `OtagDescribe` (looks up `description` in the cached oracle tag catalog by slug) and `OtagExamplesOpen` (sets a slug and opens `OracleTagExamples` as an overlay). Each such screen calls `CatalogCache::ensure_oracle_tags(client)` on mount.

### 0.8 Card rows (`CardRow`, app wrapper over `components/src/card_row.rs`)

Used by Activity (hard passes), Playstyle (suggestions, staples) and Commander maybeboard. The deck-cards spec covers it in full; what matters here:

- Compact line: optional art-crop thumbnail (shown when the screen's `ShowRowArt` context is true; eases in and out), the leading qty or, when `qty_tags` is non-empty, those tags in its place, the name, the price in USD, the color pips.
- Tapping a row expands it (one row per screen open at a time via `expanded_card`, keyed by scryfall id). The detail shows cost, type, rarity, keywords, oracle text, stats, and the classification (`show_classification: true`): roles each drilling to their oracle tags (with `OtagDef` reveals), plus "Other tags".
- Then the host `footer`, then an action bar: "Image" (opens the fullscreen `ImagePreview` on the face being shown), "Printing" (opens the `PrintingSheet`), then the host's `extra_actions`.
- `PrintingSheet` on these screens: picking a printing calls `on_save(new_card)`; the screen swaps that card in its list in place (matched by `oracle_id`) and keeps the row expanded under the new scryfall id. Nothing goes to the server from these screens.

### 0.9 Animations shared

- `content-enter-delayed`: fade/slide in, 0.3 s ease-out, 0.15 s delay. `content-enter` is the same without the delay.
- `row-enter` (card groups): 0.25 s ease-out entrance.
- `chip-in`: 0.3 s fade up from 0.35rem below.
- Reduced motion disables these and the number rolls.
- Skeletons: no real text, chips or controls; every chip is a bare ghost bar the size of the real one (user rule "Skeletons ghost everything").

---

## 1. Home (`screens/home.rs`, route `/`)

Top to bottom inside `screen-content home-content`:

1. Header: title "Home". No "?".
2. Hero panel (`HomeHeroScoped`).
3. Color map panel (`HomePlayStyle`).
4. Featured flavor panel.
5. Action bar: "Decks", "Account", "Config".
6. Overlays: welcome `HintDialog`, `ImagePreview`, the "Buy card" `AlertDialog`.

### 1.1 Mount effects and toasts

All three fire on every mount of Home (Home remounts on every navigation back to it).

1. User refresh and greeting. `client.fetch_user(&session) -> Result<User, ClientError>` (GET `/api/user`). Not through the facade; no error toast.
   - Ok: replace `session.user` with the fresh user. Then:
     - `email_verified_at.is_none()` → warning toast "Verify your email!" for TOAST_QUICK. Not gated: shows on every Home mount while unverified. No greeting in that case.
     - else, once per app launch (process-wide `static GREETED: AtomicBool`): info toast `"Hello, {username}!"` for TOAST_QUICK.
   - Err: log. If the cached session's user is verified and not yet greeted this launch, info toast `"Hello, {username}!"` (cached username), TOAST_QUICK.
2. Preferences refresh. `client.fetch_preferences(&session) -> Result<UserPreferences, ClientError>` (GET `/api/user/preferences`). Ok: `components::theme_from_preferences(&prefs)` becomes the live theme, the app icon follows it, and `pip_style` is set from `prefs.pip_style`. Err: logged only.
3. Featured flavor (see 1.4).

### 1.2 Hero panel (`components/home_hero.rs`)

A `Panel` (`home-hero`). Inside `hero-head`:

- The logo: `Decode { text: zwipe_core::domain::logo::ZWIPE }` (11-line block-glyph art, 49 columns). `Decode` resolves left to right in two waves: noise glyphs `░ ▒ ▓ ▎ ▍` sweep in first (covering the art over the first 45% of the pass), then a decode wave starting 35% in turns noise into the art; tick 40 ms. Inside a splash it waits for the splash gate.
- `StatsStrip` (compact) keyed by the scope label, so a scope switch remounts it and the figures roll again.

`StatsStrip { figures, compact: true, source }` (orangui `stats_strip.rs`):

- A row of three figures, in this order and with these labels: "Decks created", "Searches run", "Cards swiped". Each is a big number over its label.
- Number display (`CountUp`): `None` (not arrived) rolls random 4-digit numbers (1000..=9999) every 16 ms, indefinitely. Before the first frame it prints four figure spaces (U+2007). When the value arrives, it rolls for 1000 ms (16 ms ticks) with a random offset that shrinks quadratically to zero, never exceeding the value's digit count, then lands exactly. A changed value mid-roll becomes the new target. Each figure has its own random seed. Reduced motion shows the value at once.
- Compact formatting (`short`): under 10,000 → thousands separators (`9,999`); from 10,000 → one decimal, truncated, with `k`, `m` or `b` (`28.0k`, `1.2m`, `3.4b`).
- Under the figures, the `source` chip row (`stats-source`), in order:
  1. "Live" `stat-chip`, only when `live`.
  2. The scope button (`card-action-btn hero-scope`): label "Everybody" or "Only me" (`CountScope::label`). Its width is `calc({chars}ch + 1.2rem + 2px)` so it eases between the two widths; the label span is keyed so the new label fades in (`hero-scope-in`, 0.25 s ease-out).
  3. "Activity" button (`card-action-btn`): pushes `Router::Activity`.

Scope (`outbound/count_scope_store.rs`):

- `CountScope::AllUsers` (label "Everybody", default) or `JustMe` ("Only me"). Restored at launch from the OS keyring (desktop/Apple; service `zwiper-service`, user `zwiper-count-scope`) or a file `count_scope` in Android private storage; stored bytes `all` / `mine`; anything else reads as `AllUsers`. Saved on every toggle; never deleted on logout.
- Tapping the scope button: `next = scope.toggled()`, `next.infallible_save()`, `scope.set(next)`.

Figures and "Live" per scope:

- `AllUsers`: figures from everyone's counters (`HttpPublicMetrics`): `decks_created`, `searches`, `cards_swiped`. `live` = the cached copy was fetched this launch (not a copy restored from last launch).
- `JustMe`: figures from `HttpLifetimeCounters`: `decks_created` (i32), `searches`, and cards swiped = `swipes_right + swipes_left + swipes_up + swipes_down`. `live` = a copy for this user exists. While none exists, all three roll. Negative i64 clamps to 0 (`u64::try_from(n).unwrap_or(0)`).

Caching and fetches:

- Everyone's counters: `client.public_metrics() -> Result<HttpPublicMetrics, ClientError>` (GET `/api/marketing/stats`, unauthenticated). Cache `CachedCounts { metrics, fetched_at: Option<Instant> }` held above the router, seeded at launch from the copy saved by the last launch (`HttpPublicMetrics::infallible_load()`, `fetched_at: None`). On mount, refetch only if missing, restored, or older than `COUNTS_FRESH = 60 s`. On success: save to disk (`infallible_save`) and replace the cache with `fetched_at: Some(now)`. On failure: logged; the old copy stays on screen. Always fetched, whatever the scope (the auth screens also use this cache).
- My counters: `client.my_metrics(&session) -> Result<HttpLifetimeCounters, ClientError>` (GET `/api/user/metrics`) via `run_quiet("my_metrics")`. Cache `CachedMyCounts { user_id, metrics, fetched_at: Instant }` above the router, never saved. Fetched only while the scope is `JustMe`, and only if the cache is missing, belongs to another user, or is 60 s old. A copy belonging to another user is never shown.

`HttpPublicMetrics { cards_swiped: i64, searches: i64, decks_created: i64 }`. `HttpLifetimeCounters { user_id: Uuid, swipes_right: i64, swipes_left: i64, swipes_up: i64, swipes_down: i64, searches: i64, decks_created: i32, decks_completed: i32, updated_at: DateTime<Utc> }`.

### 1.3 Color map panel (`components/home_play_style.rs`)

Container `container-sm text-center flex-col home-flavor content-enter-delayed` (fades in after 0.15 s).

Fetch: `client.play_style_summary(&session) -> Result<HttpPlayStyleSummary, ClientError>` (GET `/api/deck/playstyle`) via `run_quiet("play_style_summary")`, on every Home mount. Cache `CachedPlayStyle { user_id, summary }` above the router; a return to Home shows the cached copy at once while the refetch is out; replaced only on success; another user's copy is never shown. A failure sets a local `failed` flag (only visible when nothing is cached).

`HttpPlayStyleSummary { deck_count: u32, color_map: Option<HttpColorMap> }`. `HttpColorMap { colors: Vec<HttpColorCards { color: String /* "W","U","B","R","G" */, cards: u32 }>, colorless_cards: u32, pairs: Vec<HttpColorPair { colors: [String; 2], decks: u32 }> }`. The app never draws `pairs`.

States:

- Loaded (summary present). Title bar: "Color map" centered with a "?" (`HintTopic::ColorMap`) at the right end. Then:
  - Counts = `ColorMapCounts::from(&color_map)` kept only if `color_map_has_data` (any colorless or colored count > 0) and `deck_count > 0`.
  - With counts: `ColorMap { counts, seed, key_action: PlaystyleButton }` (section 6). The "Playstyle" button sits at the end of the key's chips, wrapping with them.
  - Without: the line "Build a deck to see your playstyle" (muted, padding 1rem), then a row holding the "Playstyle" button.
- Failed with nothing cached: title bar as above, the muted line "Could not load your playstyle", then the "Playstyle" button row.
- Loading with nothing cached (whole panel ghosts): a title bar of a bare round chip where the "?" sits and a bar 30% wide for the title; then `ColorMapSkeleton { key_action: true }` = a `ColorMapGhost` (section 6.6), then the key ghost: five medium chips, one large chip (colorless), one more large chip (the button).

"Playstyle" button (`card-action-btn`): pushes `Router::SuggestedCommanders` (route `/deck/suggested-commanders`).

Seed: `components::seed_from_id(session.user.id.as_u128())`, or 0 with no session. The same seed is used on Playstyle, so both draw the same shape.

### 1.4 Featured flavor panel

Container `container-sm text-center flex-col home-flavor content-enter-delayed`.

Fetch: `client.featured_flavor() -> Result<Card, ClientError>` (GET `/api/card/featured-flavor`, unauthenticated). The server picks one card per UTC hour, the same for everyone. Cache `FlavorCard { card, expires_at }` above the router; `expires_at` = the top of the next UTC hour after the fetch. The effect subscribes to the session (which refreshes about every 60 s), so expiry is re-checked then and on mount. It fetches only when the cache is empty or expired; overwrites only on success; a failure keeps the old card (logged only).

States:

- Card cached and it has `flavor_text`:
  - Header row (centered): "Featured flavor".
  - Quote (`flavor-quote`, padding 1rem): `"{flavor_text} "` (note the trailing space).
  - Buy row (`flavor-buy-row`): the card name; the USD price when it has one; an "Image" button; a "Buy" button only when there is a USD price.
  - Price: `zwipe_core::domain::deck::deck_metrics::card_price(&scryfall_data, PriceCurrency::Usd)` (first parseable of `prices.usd`, `usd_foil`, `usd_etched`), formatted by `PriceCurrency::Usd.format_amount(p)` → `"${amount:.2}"`.
- Card cached without flavor text: the panel renders empty (the container still exists).
- No card yet: skeleton: header with a centered bar 45% wide, three flavor line bars, then a row of one large chip ghost and three medium chip ghosts.

"Image": opens `ImagePreview` with `(scryfall_data, 0)` (front face). Fullscreen image overlay with a flip control for multi-faced cards and a dismiss animation.

"Buy": opens `AlertDialog`:

- Title "Buy card"; body "Buy this card from a retailer."
- Actions, in order: "Close" (dismiss), "TCGplayer ↗" (link), "Card Kingdom ↗" (link). The arrow is U+2197. Links open in the external browser (`target=_blank`) and close the dialog.
- URL rules (`outbound/buy_links.rs`):
  - Name encoding: take the front face (`name.split(" // ").next()`), then replace `' '` → `+`, `,` → `%2C`, `'` → `%27`. Nothing else is escaped.
  - TCGplayer: `https://www.tcgplayer.com/product/{tcgplayer_id}` when `scryfall_data.tcgplayer_id: Option<i32>` is `Some`, else `https://www.tcgplayer.com/massentry?c=1+{encoded}`.
  - Card Kingdom: `https://www.cardkingdom.com/builder?c=1+{encoded}`.

### 1.5 Welcome hint

`use_one_time_hint(HINT_FIRST_LOGIN)`. Title "Welcome to Zwipe". Bullets, verbatim:

- key "Decks" + " holds your decks"
- key "Color map" + " grows from your decks' colors. Tap Playstyle for suggested commanders"
- key "Account" + " has your email, password and log out"
- key "Config" + " sets your theme and which cards show"
- colored (`--color-warning`) "Verify your email" + " in Account for full deck and card limits"

Button "Got it".

### 1.6 Navigation

- In: after login (`Router::Home`), the default route, the Decks screen's Back.
- Out: "Decks" → `Router::DeckList`; "Account" → `Router::Account`; "Config" → `Router::Config`; "Activity" (hero) → `Router::Activity`; "Playstyle" (color map) → `Router::SuggestedCommanders`.

### 1.7 Auth screens' hero (context only)

Login, register and forgot password show `HomeHero`: the same panel with everyone's counters only, chips "Everybody" (plain `stat-chip`, not a button) then "Live". Same 60 s cache.

---

## 2. Activity (`screens/activity/mod.rs`, `sections.rs`; route `/activity`)

Reached only from the Home hero's "Activity" button. Header title "Activity" with "?" (hint `HINT_ACTIVITY`).

### 2.1 Hint dialog

Title "Activity". Line "How you pick, read from your swipes". Bullets:

- key "Activity" + " maps your swipes and searches over the last year, and what the cards dealt to your decks came to"
- key "Card roles" + " and " + key "Oracle tags" + " show how often you add, skip, save or remove each one"
- key "Most skipped" + " lists cards you skip that other players usually keep"

Line "Each section's ? explains it". Button "Got it".

### 2.2 Fetch

`client.activity(&session) -> Result<HttpActivity, ClientError>` (GET `/api/user/activity`) via `try_run("load_activity")` once on mount. Not cached; every visit refetches and shows the skeleton. A failure toasts the error (TOAST_NORMAL) and sets `load_failed`.

```
HttpActivity { through: NaiveDate, days: Vec<HttpActivityDay>, outcomes: HttpSwipeOutcomes,
               roles: Vec<HttpRoleRates>, tags: Vec<HttpTagRates>, passes: Vec<HttpHardPass> }
HttpActivityDay { day: NaiveDate, swipes: u32, searches: u32 }
HttpSwipeOutcomes { added: u64, skipped: u64, maybed: u64, removed: u64 }
HttpRoleRates { role: String /* CardRole snake_case slug */, outcomes: HttpSwipeOutcomes }
HttpTagRates { tag: String, role: String, outcomes: HttpSwipeOutcomes /* user's */, everyone: HttpSwipeOutcomes /* every other player */ }
HttpHardPass { card: Card, skipped: u32, commanders: Vec<String> }
```

Constants (zwipe-core `contracts/suggestions.rs`): `ACTIVITY_DAYS = 371`, `ACTIVITY_ROLES_SENT = 10`, `ACTIVITY_MIN_SEEN = 10`, `ACTIVITY_TAGS_SENT = 100`, `ACTIVITY_PASSES_SENT = 10`.

The oracle tag catalog is warmed on mount (`ensure_oracle_tags`) for tag descriptions.

### 2.3 Layout top to bottom

1. `MyCountsPanel { ready: !is_loading }`: a `Panel` (`home-hero activity-counts`) with no logo, holding a compact `StatsStrip` of the user's own three figures (same labels and math as Home "Only me"). Chips under it: "Live" (only once shown) then "Only me". It uses the same `CachedMyCounts` cache and 60 s rule as Home (fetched via `run_quiet("my_metrics")` regardless of Home's scope). The figures keep rolling until the activity fetch finishes (success or failure), so they land together with the rest; then they show the cached copy if any.
2. Then exactly one of:
   - Loading: `ActivitySkeleton` (2.8).
   - Failed: a muted `chip-note` "Could not load your activity".
   - Loaded and `has_anything` (any day, any outcome > 0, any role, any tag or any pass): `ActivitySections`, then `HardPassesPanel`.
   - Loaded with nothing: `chip-note` "Swipe some cards to see your activity".
3. Action bar: "Back" (`navigator.go_back()`).
4. Overlays: `PrintingSheet` (hard pass rows), `OracleTagExamples` (from tag reveals), `ImagePreview`.

### 2.4 Accordion sections

One accordion, `open_section` starts at "Activity". Each section is left out when it has nothing.

#### Section "Activity" (present when `days` non-empty or any outcome > 0)

Header accessory "?" → `HintTopic::Activity`. Body:

1. Heatmap (only when `days` non-empty). Remounted each time the section opens (keyed on an open counter), so its sweep replays in view.
2. A muted full-bleed rule, only when both the heatmap and the outcomes show.
3. Totals chart (only when any outcome > 0): `BarChart { label: "Totals", bars: outcome_bars(outcomes) }`, vertical.

Heatmap (`heat_layout(through, days)`, app-local, `sections.rs`):

- Days: every date from `start = through - 370 days` to `through` inclusive (371 cells). `first_sunday = start - start.weekday().num_days_from_sunday()`. For each date, `offset = date - first_sunday` in days; `column = offset / 7`, `row = offset % 7` (row 0 = Sunday ... row 6 = Saturday). Columns = last cell's column + 1 (53 or 54).
- A day's count = `swipes + searches` (saturating). Days not in `days` are 0.
- Level (`level(count, busy)`), where `busy` = the sorted nonzero counts from `days`: 0 when count is 0; else with `quartile(q) = busy[(len-1)*q/4]`: `count <= quartile(1)` → 1, `<= quartile(2)` → 2, `<= quartile(3)` → 3, else 4.
- Peaks: `orangui::peak_indices(counts, ratio = 4, max = 12)` over all 371 counts: median of nonzero counts = `busy_sorted[len/2]`; a day is a peak when `count > 0 && count >= median*4`; the largest 12, biggest first. Peak cells get a lit edge plus a glow halo rect 2.5 units larger on each side.
- Month labels: at each row-0 (Sunday) cell whose month differs from the last labeled month, push `(column, date.format("%b"))` ("Jan".."Dec"); if the previous label is fewer than 3 columns before, the previous label is popped first (so a partial first month goes unlabeled). Labels sit above the grid.
- Weekday labels on the left: "Mon" (row 1), "Wed" (row 3), "Fri" (row 5).
- Geometry: cell 11 units, gap 2 (step 13), corner radius 2; left margin 24 for weekday labels, top margin 14 for month labels. Width = `24 + columns*13`, height = `14 + 7*13`.
- Colors: `.heat-0`..`.heat-4` from the theme's accent at rising strength; level 0 is an empty cell.
- The grid sits in a horizontally scrolling box that opens scrolled to the right end (the newest week).
- Sweep: each cell animates in with `heat-sweep` 0.4 s ease-out, delayed `column * 12 ms` (left to right).
- Screen reader label: `"{swipes} and {searches} in the last year"`, e.g. "1,234 swipes and 2 searches in the last year" (`counted` uses thousands separators and singular "swipe"/"search" for 1).
- Readout under the grid (`HeatReadout`): resting tags, all in one muted tone: `"{N} swipes"`, `"{N} searches"`, `"Last year"` (totals over `days`, thousands separators, singular for 1). Tapping a cell replaces them with that day's tags `"{swipes} swipe(s)"`, `"{searches} search(es)"`, `"%b %-d"` date (e.g. "Oct 3") for `HEAT_PICK_MS = 4000` ms, then the resting tags come back; a newer tap restarts the 4 s. Each row fades up (`chip-in`) when it appears.
- Hover devices with room instead show a floating chip near the hovered cell with the day's three tags joined by ", ". Not relevant to a terminal; use the readout behavior.

Totals (`outcome_bars`): four vertical columns in this order with these labels: "Added" (`added`), "Skipped" (`skipped`), "Maybeboard" (`maybed`), "Removed" (`removed`). `top = max(the four, 1)`; `pct = n*100/top` (integer). Count printed above each column when > 0. Labels render uppercase (BarChart default).

#### Section "Card roles" (present when `roles` non-empty)

Header accessory "?" → `HintTopic::RoleRates`. Body:

1. `RateChips`: a chip row: label chip "Show" (p1 tint), then single-select chips "Added", "Skipped", "Maybeboard", "Removed". One is always selected (default "Added"). The same signal drives the Oracle tags section's row, so picking in either switches both.
2. Keyed on the picked rate (a switch mounts a fresh chart):
   - If no bars: "Nothing to show for this rate yet" (`activity-total`).
   - Else horizontal chip-row `BarChart` with `count_suffix: "%"`; tapping a revealable role chip toggles it open (one at a time); the reveal is `RoleTags` of every `tags[].tag` whose `tags[].role == role` (the user's swiped tags under that role, server order).

Rate math (`RateKind`, app-local):

- `dealt = added + skipped + maybed` (saturating).
- Added: `(added, dealt)`; Skipped: `(skipped, dealt)`; Maybeboard: `(maybed, dealt)`; Removed: `(min(removed, added), added)`.
- `percent = count*100/of` rounded down; `None` when `of == 0`.

`role_rate_bars(kind, roles, tags)`: keep roles with `Some(percent)`; sort by percent descending (stable, so ties keep server order). Each bar: `key = role slug`, `label = card_role::role_short_label(slug)` (zwipe-core: the `CardRole::to_short_name` abbreviation such as "ramp", "draw", "remov", "wipe", "cspel", "protc", "evasn", "fnshr", "tokns", "lifgn", "blink", "recur", "mill", "burn", "drain", "pump", "anthm", "cntrs", "copy", "sacrf", "stax", "untap", "tutor", "grvht", "cadv", "enrgy", "aggro"; an unknown slug prints as-is), shown uppercase; `count = pct = percent`; `revealable` = any tag has this role.

#### Section "Oracle tags" (present when `tags` non-empty)

Header accessory "?" → `HintTopic::TagRates`. Body: the same `RateChips` row, then, keyed on the rate, either "Nothing to show for this rate yet" or a horizontal chip-row `BarChart` with `label_case: AsIs` (slugs as written), `count_suffix: "%"`; tapping a tag chip toggles an `OtagDef` reveal (description + "Examples").

`tag_rate_bars(kind, tags)`:

- `baseline` = sum over all tags of `kind.parts(everyone)` count / sum of `of`; 0.5 when the total `of` is 0.
- `lift_score(tag)`: `(count, of) = kind.parts(tag.outcomes)`; `None` if either is 0. `mine = count/of`. `others = (theirs + 10*baseline) / (their_of + 10)` where `(theirs, their_of) = kind.parts(tag.everyone)` (`TAG_BASELINE_WEIGHT = 10`). `Some(count * ln(mine/others))` only when `others > 0 && mine > others`.
- Sort scored tags by score descending, ties by slug ascending; take the first 10 (`TAG_ROWS`); map to bars `key = label = slug`, `count = pct = percent`, `revealable: true`; then stable-sort by percent descending.

### 2.5 Most skipped (`HardPassesPanel`)

Rendered only when `passes` is non-empty, below the accordion. A card group (`maybeboard-list card-group row-enter`). Header: "Most skipped" with a "?" (`HintTopic::HardPasses`). Rows in server order (most skipped against other players first), each a `CardRow` with:

- `qty = skipped`, `qty_tags = ["{n} skips"]` (singular "skip" for 1, thousands separators) in place of the qty.
- Footer when expanded and `commanders` non-empty: a `chips-label` "Skipped under", then each commander name as a `detail-chip`.
- Actions: only the row's own "Image" and "Printing".
- Art thumbnails on (`ShowRowArt(true)`), no toggle on this screen.

### 2.6 Navigation, caching

- In: Home hero "Activity". Out: "Back" (history back), plus overlays.
- No cache for the activity payload; counts share Home's `CachedMyCounts`.
- Printing swaps update the in-memory `passes` entry by `oracle_id`.

### 2.7 Animations

Heat sweep (12 ms per column, 0.4 s each), readout `chip-in`, accordion ease, `row-enter` on the group, the counters' roll that resolves when `ready`.

### 2.8 Skeleton

- Activity section open on its loaded shape: header bar plus a "?" ghost chip; empty space at the heatmap's exact height (`HeatGhost`, height `14 + 7*13 = 105`); three ghost tags; a muted rule; Totals as a ghost title chip over four ghost columns at heights 70%, 100%, 20%, 35% and four ghost labels.
- Card roles and Oracle tags as two collapsed ghost headers.
- The Most skipped group: header bar plus "?" ghost, then 10 ghost rows (thumb + bar).

---

## 3. Playstyle / Suggested commanders (`screens/deck/suggested_commanders.rs`, route `/deck/suggested-commanders`)

Header title "Playstyle" with "?" (hint `HINT_SUGGESTED_COMMANDERS`).

### 3.1 Hint dialog

Title "Playstyle". Line "Your colors, card roles and top oracle tags, read from every deck you own in any format". Bullets:

- key "Create deck" + " begins a deck with that commander"
- key "Maybeboard" + " saves it to your commander maybeboard"
- key (`--color-error`) "Remove" + " hides it for a week and pulls the next one up"
- key "Swipe" + " deals the same ranking: " + colored success "right" + " starts a deck, " + colored warning "up" + " saves, " + colored error "left" + " skips it"
- key "Show" + " narrows the list to one color identity"

Line "Below them, your top 10 commanders ranked against all your decks, best fit first. Swipe keeps going down the same ranking". Button "Got it".

### 3.2 Fetches

- Main: `client.commander_suggestions(100, &session) -> Result<HttpCommanderSuggestions, ClientError>` (GET `/api/commander/suggestions?limit=100`; `HELD_SIZE = MAX_SUGGESTION_LIMIT = 100`) via `try_run("load_suggestions")`.
  - `HttpCommanderSuggestions { suggestions: Vec<HttpCommanderSuggestion>, deck_count: u32, profile: Option<HttpBuilderProfile> }`.
  - On Ok: set `deck_count`, `profile`, `entries = suggestions`, clear `load_failed`. On Err: error toast (facade) and `load_failed = entries.is_empty()` (a refetch failure with rows showing keeps them). `is_loading` becomes false after the first reply.
  - Each load takes a generation ticket; only the newest load's reply lands. `refetch()` bumps the ticket (invalidating a load in flight) and bumps `reload`, which reruns the effect. A refetch with entries present is silent (no skeleton).
  - Refetch triggers: the Swipe overlay closes; a row drop leaves the filtered list under 10 or a load is still out; Clear skips succeeds.
- Create gate: `client.fetch_deck_profiles(&session) -> Result<Vec<DeckProfile>, ClientError>` via `run_quiet("load_deck_profiles")` once on mount. `at_deck_limit` = user unverified and at least one deck exists.
- Oracle tag catalog warmed on mount.

Types:

```
HttpCommanderSuggestion { card: Card, reasons: HttpSuggestionReasons }
HttpSuggestionReasons { tags: Vec<HttpSharedTag { slug, role: Option<String> }>, colors: Vec<HttpColorAffinity { color: String, decks: u32 }> }
HttpBuilderProfile { deck_count: u32, colors: Vec<HttpColorAffinity>, top_identity: Option<HttpColorIdentityCount { colors: Vec<String>, decks: u32 }>,
                     tags: Vec<HttpProfileTag { slug, role: Option<String>, decks: u32, cards: u32, score: f64 }>,
                     roles: Vec<HttpProfileRole { role: String, decks: u32 }>, staples: Vec<HttpStaple { card: Card, decks: u32 }>,
                     color_map: Option<HttpColorMap> }
```

Server constants: `PROFILE_TAG_SENT = 100`, `PROFILE_ROLE_SHOWN = 10`, `PROFILE_STAPLES_SENT = 100`, `STAPLE_MIN_DECKS = 2`. The server spreads its 100 suggestions across identities (at most four per identity keep their rank; the rest move behind; none dropped).

### 3.3 Layout top to bottom

1. One of:
   - First load (`is_loading`): skeleton (3.10).
   - `load_failed`: `chip-note` "Could not load your suggestions".
   - `deck_count == 0` (cold start): `chip-note` "Build a deck to see your playstyle".
   - Else the column below.
2. Column (gap `--panel-gap`):
   1. `BuilderProfileSections` when `profile` is `Some` (3.4).
   2. Show row (3.6) with the "Art" toggle at its end.
   3. The list: if `entries` empty: "No suggestions right now"; else if the shown rows are empty: `empty_identity_line(active identity)`; else the "Suggested commanders" card group (3.7).
   4. Staples panel when `profile.staples` non-empty (3.8).
3. Action bar: "Back" (`go_back`, returns to Home or the Decks screen), "Swipe" (hidden on cold start), "More".
4. Overlays: "More actions" bottom sheet, "Clear skips" dialog, `SwipeSelect` in suggested mode, `PrintingSheet`, `OracleTagExamples`, `ImagePreview`.

### 3.4 Profile sections (`BuilderProfileSections`)

One accordion with "Color map" open on entry. Order: Color map, Summary, Card roles, Oracle tags.

1. "Color map" (always present). "?" → `HintTopic::ColorMap`. Body: `ColorMap { counts, seed }` (no key action) when `profile.color_map` converts to counts with data; else "Build a deck to see your playstyle". Seed = `seed_from_id(user id)`, the same as Home.
2. "Summary" (always present, no "?"). Settings rows (label left, value right):
   - "Decks": `profile.deck_count` as plain text.
   - "Top colors" (only when `top_identity` is `Some`): the identity's pips via `IdentityPips` with the name and count: name = `identity_name(pips)` unless the identity has exactly one color, pips in WUBRG, then `"({decks})"`. Codes outside WUBRG are dropped.
   - "Top color" (only when any `profile.colors[]` parses as a color): the first parseable entry (the server sends most decks first). Shows the long name ("White", "Blue", "Black", "Red", "Green") before its single pip, then `"({decks})"`.
   - `profile_summary()` computes these.
3. "Card roles" (present when `role_bars` non-empty). "?" → `HintTopic::SuggestionRoleChart`. Horizontal chip-row `BarChart` (labels uppercase). `role_bars(roles, tags)`: one bar per role in server order; `max = largest decks`; `pct = DeckMetrics::bar_pct(decks, max)`; `label = role_short_label(role)`; `count = decks`; `revealable` = some profile tag has `role == Some(this role)`. Tap a role → reveal `RoleTags { role_tags(profile.tags, role) }` (the profile's tags under that role, in score order).
4. "Oracle tags" (present when `tag_rows` non-empty). "?" → `HintTopic::SuggestionTagChart`. `TagTable` (3.5).

### 3.5 Oracle tags table (`TagTable`)

A 4-column grid. Header row: "Tag" | "Decks" + "?" (`SuggestionTagDecks`) | "Cards" + "?" (`SuggestionTagCards`) | "Impact" + "?" (`SuggestionTagImpact`). Then one row per tag:

- Tag: a `keyword-chip` button with the slug, "active" when open.
- Decks: `stat-chip` p3 tint with `decks`.
- Cards: `stat-chip` p5 tint with `cards`.
- Impact: `stat-chip` p2 tint with the impact, or nothing when `None`.
- A full-width reveal under each row; tapping a chip toggles it open (one at a time, 0.2 s ease), showing `OtagDef` (description + "Examples").

`tag_rows(tags)`: `max = max(score)` folded from 0.0; take the first 10 (`TAG_TABLE_ROWS`) in server order (highest score first). `impact(score, max)`: `None` when `max` is NaN or `<= 0`; else `round(max(score,0)/max*100)` clamped to `0..=100`. The top tag reads 100.

### 3.6 Show row (`ShowIdentityChips`)

A chip row: label chip "Show" (p1), "All" chip (selected when no identity is active; tapping sets All), then one chip per identity, then the screen's "Art" chip.

- Identities (`identities_present`): distinct `identity_of(card.color_identity)` across all 100 held entries plus the currently selected one (so a chip stays after its last commander goes). Sorted by (colorless last, fewer colors first, then WUBRG index order).
- `identity_of(colors)`: WUBRG order, each once; empty = colorless.
- Chip content (`IdentityPips`): the name (`identity_name`, e.g. "Azorius", "Jeskai", "Five-color", "Colorless") unless exactly one color, then the pips (mana font glyphs). A single color shows its pip alone; colorless shows "Colorless" alone.
- Single select over `active_identity: Option<Vec<Color>>`; tapping the selected chip returns to All.
- "Art" chip: toggles row thumbnails (`ShowRowArt`), default on.
- The same signal drives the Swipe overlay's Filter sheet chips.

Filtering: `filtered = of_identity(entries, card.color_identity, active)` = items whose exact identity equals the selected one (All keeps all), in held order. `rows = first 10 of filtered` (`LIST_SIZE = 10`).

Empty under a chip (`empty_identity_line`): `"No {name} commanders in your top 100"`, where `name = identity_name`, lowercased for 0, 1 or 5 colors ("No colorless commanders...", "No mono-red commanders...", "No five-color commanders..."), as-is otherwise ("No Jeskai commanders in your top 100").

### 3.7 Suggestions list

Card group, header "Suggested commanders" with "?" (`SuggestionList`). Each `SuggestionRow` is a `CardRow` (`qty: 1`) with:

- Footer: a muted rule, then `SuggestionReasons { foldable: true }`:
  - Renders nothing if both `reasons.tags` and the color lines are empty.
  - A chip "Why this commander?" followed by a disclosure arrow "▸" (rotates when open), closed on mount; tapping toggles a collapsible (eased) with a bullet list:
    - When `reasons.tags` non-empty: "Shares oracle tags with your decks" followed by `CardRoleChips` (no label): the tags grouped by role, roles in order of first appearance (tags arrive strongest first), each role drilling down to its tag chips (each tag with an `OtagDef` reveal), role-less tags in an "Other" bucket (`group_shared_tags`).
    - One bullet per `reasons.colors` entry with a WUBRG code (`color_lines`): `"You build in {white|blue|black|red|green} ({decks} of your {deck_count} deck|decks)"` ("deck" when `deck_count == 1`).
- Extra actions (after "Image" and "Printing"): "Create deck"; and when the card has an `oracle_id`: "Maybeboard", "Remove".

Actions:

- "Create deck" (`create_with`): if `at_deck_limit`, warning toast "Verify your email to create more than 1 deck" for TOAST_LONG. Else put the card in the `CreateDeckCommanderSeed` slot and push `Router::CreateDeck` (the create screen takes the seed: format Commander, commander set to that card).
- "Maybeboard": `client.add_commander_maybeboard_card(oracle_id, &session) -> Result<(), ClientError>` (POST `/api/user/commander-maybeboard/{oracle_id}`) via `run`. On success: drop the row, success toast "Added to your commander maybeboard" for TOAST_QUICK. On failure: facade error toast; row stays.
- "Remove": `client.dismiss_commander(oracle_id, &session) -> Result<(), ClientError>` (POST `/api/commander/suggestions/dismissals`, body `HttpDismissCommander { oracle_id }`; hides it for a week) via `run`. On success: drop the row, info toast "Skipped for a week" for TOAST_QUICK.
- Drop row (`drop_row`): remove the entry with that `oracle_id` from `entries`; the next held suggestion of that identity moves up; refetch if the filtered count is now under 10 or a load is still out.

### 3.8 Staples panel

Below the list, independent of the Show row; its filters reset when the screen remounts.

- Two chip rows stacked:
  - "Type" (p1 label), "All", then each of Creature, Artifact, Enchantment, Instant, Sorcery, Planeswalker, Land that at least one staple has (fixed order; text is `CardType`'s Display). Single select; tapping the selected one returns to All.
  - "Color" (p1 label), "All", then each of W, U, B, R, G (single pip) and Colorless ("Colorless" text) that at least one staple falls under. A multicolor card falls under each of its colors; colorless = empty identity.
- Type test (`has_type`): the front face of the type line (split on `" // "`), split on whitespace, contains the type word exactly. Type line = `type_line`, else the first face's `type_line`.
- Rows: the first 10 staples (server order) that match both selections (`matching_staples`).
- Empty: `chip-note` "No staples match".
- Else card group "Staples" with "?" (`SuggestionStaples`), rows as `CardRow` with `qty = decks`, `qty_tags = [staple_percent]`, footer chip `staple_decks`, no extra actions.
  - `staple_percent(decks, deck_count)` = `(decks*200 + deck_count) / (deck_count*2)` + "%" (round half up), `None` when `deck_count == 0`.
  - `staple_decks(decks, deck_count)` = `"{decks} of {deck_count} deck|decks"`.

### 3.9 More, Clear skips, Swipe

- "More" opens a bottom sheet titled "More actions" with one danger button "Clear skips".
- "Clear skips" opens an `AlertDialog`: title "Clear skips", body "Commanders you skipped will be suggested again. This can't be undone.", actions "Cancel" (dismiss) and "Clear" (danger confirm).
- Confirm: close the sheet, `client.clear_commander_dismissals(&session) -> Result<u32, ClientError>` (DELETE `/api/commander/suggestions/dismissals`, returns `HttpClearedDismissals { cleared }`) via `run`. On success: info toast "Skips cleared" (TOAST_QUICK), refetch.
- "Swipe" opens the shared `SwipeSelect` overlay in `SwipeMode::Suggested` (title "Suggested commanders"). It deals `SuggestedPile { suggestions: filtered, deck_count, identity }` (the same filtered held list, not just 10). Its Filter sheet holds the same Show chips over the same signal; a change re-deals while keeping already-swiped cards behind the top card.
  - Right: success toast "Commander selected" (TOAST_QUICK), close the overlay, `create_with(card)`.
  - Up: `add_commander_maybeboard_card` (not through the facade). Success toast "Added to your commander maybeboard" for 1500 ms. Failure: error toast with the user message. A card without `oracle_id`: error toast "This card can't be saved", counted as a plain skip.
  - Left: `dismiss_commander` quietly (failure only logged), info toast "Skipped" (TOAST_QUICK).
  - Down: undo. At index 0: info toast "Nothing to undo". Undo of an up: `remove_commander_maybeboard_card`, info toast "Removed from your commander maybeboard" (TOAST_QUICK). Undo of a left in suggested mode: `client.undismiss_commander(oracle_id)` (DELETE `/api/commander/suggestions/dismissals/{oracle_id}`) quietly, info toast "Undid skip".
  - Hint (title "Suggested commanders"): bullets "Swipe right starts a deck with that commander", "Swipe left skips it for a week", "Swipe up saves it to your commander maybeboard", "Swipe down undoes the last swipe"; line "Best fit first, 100 deep. The eye shows why each one was suggested"; line key "Filter" + " narrows the stack to one color identity".
  - Closing the overlay triggers a refetch of the main list.
  - The full overlay belongs to the swipe spec.

### 3.10 Skeleton (first load only)

- `BuilderProfileSkeleton`: the Color map section open: header bar plus a "?" ghost, `ColorMapSkeleton` (ghost map plus five medium and one large key chip ghosts, no button); then three collapsed ghost section headers (Summary, Card roles, Oracle tags). (The code comment says Summary opens on three rows; the code draws it collapsed.)
- `ShowChipRowSkeleton`: a p1-tinted small label ghost, then chip ghosts sized sm, sm, md, md, sm.
- The list group: a wide header bar plus a "?" ghost, then 6 ghost rows.

### 3.11 Navigation

- In: Home color map panel's "Playstyle" button; Decks screen "More" sheet → "Playstyle".
- Out: "Back" (history back); "Create deck" or a right swipe → `Router::CreateDeck` with the seed.

### 3.12 Caching

None across visits: each mount refetches from scratch with the skeleton. In-session state is local; the Home `CachedPlayStyle` is separate and not updated from here.

---

## 4. Commander maybeboard (`screens/deck/commander_maybeboard.rs`, route `/deck/commander-maybeboard`)

Header title "Commander maybeboard" with "?" (hint `HINT_COMMANDER_MAYBEBOARD`).

### 4.1 Hint dialog

Title "Commander maybeboard". Line "Commanders you save land here". Bullets:

- "Swipe " + colored warning "up" + " while picking a commander saves it here"
- key "Swipe" + " deals commanders: right starts a deck, up saves one"
- key "Quick add" + " searches by name"
- key "Create deck" + " starts a deck with that commander"
- key (`--color-error`) "Remove" + " takes it off the list"

### 4.2 Fetches

- List: `client.fetch_commander_maybeboard(&session) -> Result<Vec<Card>, ClientError>` (GET `/api/user/commander-maybeboard`) via `try_run("load_maybeboard")`. Server order is newest save first, preferred printing. Generation tickets as on Playstyle: only the newest reply lands; on failure `load_failed = entries.is_empty()`; refetch is silent with entries present. A local edit while a load is out restarts that load (its reply predates the edit).
- Refetch triggers: the Swipe overlay closes; after a quick add succeeds.
- Quick-add search: when `query.trim().len() >= 2` (`MIN_QUERY_LEN`), debounce 300 ms (`DEBOUNCE_MS`), drop if the query changed (ticket), then build `CardQueryBuilder::with_name_contains(q.trim())` + `set_is_commander_in_format(Format::Commander)` + `set_is_token(false)` + `set_sort(CardSortKey::Name)` + `set_limit(8)`, record a search in the usage buffer, and call `client.search_commanders(&filter, &session) -> Result<Vec<Card>, ClientError>` (POST `/api/card/search/commanders`) via `run_quiet("maybeboard_search")`. Results land only if still current; a failure leaves the old results. Under 2 characters clears results.
- Create gate: `fetch_deck_profiles` via `run_quiet` on mount (as Playstyle).
- Oracle tag catalog warmed on mount.

### 4.3 Layout top to bottom

First load: skeleton: `QuickAddSkeleton` (large tinted label ghost plus an empty input box), `ChipRowSkeleton` (sm label, seven sm chips), a list group with a header bar and 6 ghost rows.

Loaded:

1. Quick add row: label chip "Quick add" (p1), a compact text input (placeholder "Search a commander to add", no autocapitalize, autocorrect or spellcheck), and a "×" (U+00D7) clear button while the query is non-empty.
2. Floating results under the input (only while the trimmed query is at least 2 characters), hung downward over the content:
   - Searching: one chip "Searching...".
   - No results (after removing already-saved oracle ids): one chip "No results".
   - Else one chip per result with its name; chips animate in staggered (`animation-delay: {i*40}ms, {250 + i*40}ms`).
3. Show row: "Show" label (p1), "All" (selected when no color is picked; tapping clears all), five color chips W U B R G (pip glyphs only; multi-select toggles), then "Art" (toggles thumbnails, default on).
4. Then one of:
   - `load_failed`: "Could not load your commander maybeboard".
   - No entries: "No commanders".
   - Entries but none pass the filters: "No commanders match".
   - Else a card group with header "Commanders" (no "?"), rows as `CardRow` (`qty: 1`), extra actions "Create deck" and, with an `oracle_id`, "Remove".
5. Action bar: "Back" (pushes `Router::DeckList`, not history back), "Filter" (with a dot when the filter has any criterion or a sort), "Swipe", "More".

### 4.4 Filtering and sorting (in memory)

- Color pips: contains-all on `color_identity` (every selected color must be in the card's identity; none selected = All).
- Then, if the filter sheet's builder is non-empty: `Cards::from(filtered).matching(&builder.build_criteria())` (zwipe-core).
- Then, if the sheet set a sort: `Cards::sorted(sort, direction)`. Otherwise saved order (newest first) stands.
- The list re-filters only when the sheet's Apply bumps `filter_reset_counter` (the builder itself is peeked, so mid-edit changes do not reshuffle).
- The quick-add input never narrows the list.

### 4.5 Actions and toasts

- Tap a result chip (`add_from_search`): remove that chip at once; `add_commander_maybeboard_card(oracle_id)` via `run`. On success: insert the card at the top if not already there, silent refetch, info toast `"Added {name}"` (TOAST_QUICK). Card without `oracle_id`: ignored.
- "Create deck": same gate and seed as Playstyle (toast "Verify your email to create more than 1 deck", TOAST_LONG; else seed + `Router::CreateDeck`). A printing picked earlier carries into the new deck.
- "Remove": `client.remove_commander_maybeboard_card(oracle_id, &session)` (DELETE `/api/user/commander-maybeboard/{oracle_id}`) via `run`. Success: drop the row, info toast "Removed" (TOAST_QUICK).
- "More" → bottom sheet "More actions" with danger button "Clear maybeboard" → `AlertDialog` title "Clear maybeboard", body "Every commander on your maybeboard will be removed. This can't be undone.", actions "Cancel" and "Clear" (danger). Confirm: close the sheet, `client.clear_commander_maybeboard(&session)` (DELETE `/api/user/commander-maybeboard`) via `run`; success empties the list and toasts "Maybeboard cleared" (TOAST_QUICK).
- "Filter" → the shared `CardFilterSheet` (no format filter, active indicators shown; "Clear" empties the builder). Covered by the filter spec.
- "Swipe" → `SwipeSelect` in `SwipeMode::Commander(Format::Commander)`, excluding already-saved oracle ids. Up saves to the maybeboard; right selects → `create_with`; left skips; down undoes (see 3.9 for toasts; the non-suggested hint reads "Swipe right picks that commander" / "Swipe left skips it" / up / down, line "Most-played first. " + key "Filter" + " narrows by color or anything else"). Close → refetch.
- Printing: display-only swap in `entries` (the maybeboard is oracle-keyed; a refetch returns the preferred printing).

### 4.6 Navigation

- In: Decks screen "More" sheet → "Commander maybeboard".
- Out: "Back" → `Router::DeckList`; create flows → `Router::CreateDeck`.

---

## 5. Oracle tag dictionary and examples

Neither is a route. Both are in-place overlays (`use_overlay_back(open)`: the OS back gesture closes them), each painting its own backdrop (`GridCells { still: true }`).

### 5.1 Oracle tag dictionary (`screens/oracle_tag_dictionary.rs`)

Hosts: the deck's oracle tag picker (`oracle_tag_select.rs`) and the card filter sheet (`card_filter_sheet.rs`). Props: `open: Signal<bool>`, `on_use: EventHandler<String>` (adopt the slug where the user came from).

Header title "Oracle tags" with "?" (hint `HINT_ORACLE_TAG_DICTIONARY`). Hint: title "Oracle tags"; bullets "Tap a " + key (`--palette-1`) "letter" + " to browse, or " + colored (`--accent-secondary`) "search" + " by name or description" / key "Examples" + " shows real cards with that tag" / key "Use" + " adds the tag where you came from" / "Some tags have no description yet".

Data: `CatalogCache.oracle_tags`, prefetched at app start, 24 h TTL, stale-while-revalidate, single-flight, in memory only. `ensure_oracle_tags(client)` on mount. Source: `client.fetch_oracle_tags() -> Result<Vec<OracleTag>, ClientError>` (GET `/api/card/oracle-tags`, unauthenticated, ~4,500 rows). `OracleTag { slug, label, description: Option<String>, parent_slugs: Vec<String>, curated: bool }`.

States (`CatalogCell`):

- Loading: skeleton: a search bar ghost, a rail of 12 letter ghosts, 7 rows of slug + description ghosts.
- Failed (no cached data): `"Couldn't load the dictionary. Pull back and try again."` and, once, an error toast "Couldn't load the oracle-tag dictionary" for TOAST_NORMAL.
- Loaded:
  1. Search input, placeholder "Search tags or descriptions" (no autocapitalize, autocorrect or spellcheck).
  2. A horizontally scrolling letter rail: chips "A" to "Z", then "#" only when some slug starts with a non-letter. The selected letter (default 'a') is highlighted only while not searching. Tapping a letter clears the search and selects it.
  3. List:
     - Not searching: tags whose bucket matches the letter (bucket = first character lowercased if ASCII alphabetic, else '#'), sorted by slug.
     - Searching (`q = query.trim().to_lowercase()` non-empty): `zwipe_core::domain::card::oracle_tag::search_oracle_tags(tags, &q)` truncated to 100 (`SEARCH_RESULT_CAP`). Ranking: tier 0 exact slug or label match, tier 1 slug or label contains, tier 2 description contains (case-insensitive); alphabetical by slug within a tier.
     - Empty: searching `"No tags match “{q}”."` (curly quotes U+201C/U+201D around the lowercased query); browsing `"No tags start with {LETTER}."` (uppercase letter).
     - Each row: the slug; the description or a muted "No description yet"; the parent slugs as small chips when any; a divider; buttons "Examples" (opens 5.2 stacked on top) and "Use" (`on_use(slug)`; the host adopts it).
     - Rows are keyed by slug, so switching letter or query remounts them and their entrance replays.
- Action bar: "Back" (closes).

### 5.2 Oracle tag examples (`screens/oracle_tag_examples.rs`)

Hosts: the dictionary's "Examples", and every `OtagDef` "Examples" button (Activity, Playstyle, Commander maybeboard, deck screens). Props: `open`, `slug`. A new tag always remounts.

Header title "Examples" with "?" (hint `HINT_OTAG_EXAMPLES`). Hint title "Example cards"; bullets "Swipe " + colored success "left" + " for the next card" / "Swipe " + colored (`--accent-secondary`) "down" + " to go back one".

Fetch: pages of 25 (`PAGE_LIMIT`) via `client.search_cards(&filter, &session) -> Result<Vec<Card>, ClientError>` (POST `/api/card/search`) through `run("load_examples")` (error toast on failure). Filter: `CardQueryBuilder::new()` + `set_oracle_tags_contains_any(vec![slug])` + `set_is_token(false)` + `set_sort(CardSortKey::EdhrecRank)` + `set_limit(25)` + `set_offset(offset)`. An empty page marks the end. Offset grows by 25 per non-empty page. The stack is capped at `MAX_CARDS_IN_STACK = 500` cards. The next page loads when `index + 1 >= total - 5` after an advance.

Body:

- With a current card: the swipe stack (`SwipeStack`) with only Left and Down committing (distance 90 px or speed 2.5 px/ms); right and up spring back. Below it, `CardInfoDisplay` (the card's text info).
  - Left: record and advance.
  - Down: step back one. With no history: info toast "Already at the first card" (TOAST_QUICK).
- Past the last card: while a page loads, a card skeleton; else "No more cards. Swipe down to go back." (a downward drag of 60 px that is more vertical than horizontal goes back).
- Nothing loaded yet: card skeleton. Nothing at all: "No example cards for this tag yet."
- Action bar: "Back" (closes) and, with a current card, the eyeball `RulesButton` that opens `CardDetailsSheet`; its "Printings" opens a read-only `PrintingSheet`.

---

## 6. ColorMap: what it draws (`components/src/color_map.rs`)

Note: the map is not nodes and pair edges. It is a field of square tiles grown from a seed. `HttpColorMap.pairs` is not used anywhere in the app.

### 6.1 Inputs

- `ColorMapCounts { cards: [u32; 5] /* WUBRG, cards whose identity includes the color */, colorless: u32 }`, built from `HttpColorMap` by matching each color's short name ("W","U","B","R","G"); a missing color reads 0.
- `seed: u64` = `seed_from_id(user_id.as_u128())`.

### 6.2 Seed and hashing

```
mix(z): z += 0x9e3779b97f4a7c15; z = (z ^ z>>30) * 0xbf58476d1ce4e5b9; z = (z ^ z>>27) * 0x94d049bb133111eb; z ^ z>>31   (wrapping)
seed_from_id(id: u128) = mix(high64 ^ mix(low64))
hash(seed, salt, (x,y)) = mix(seed ^ mix(salt ^ mix((x as u32 as u64) << 32 | (y as u32 as u64))))
unit(h) = ((h >> 32) as f64 + 1) / 2^32          // in (0, 1]
```

### 6.3 Tile count and quotas

- `colored = sum(cards)`. `tile_total(colored)`: 0 when colored is 0; else `min(23 + isqrt(34*34*colored)/10, 295)` (`MIN_TILES = 23`, `TILES_PER_ROOT_TENTHS = 34`, `MAX_TILES = 295`). Examples: 1 → 26, 100 → 57, 720 → 114, 1000 → 130, 3000 → 209, 6400+ → 295.
- `quotas(cards, total)`: largest remainder. Each `floor(total*n/sum)`, then leftover tiles go to the largest remainders `(total*n) % sum`, ties by WUBRG index. Sums to `total`.
- Colorless never takes a tile; it only appears in the key.

### 6.4 Growth (`grow`)

- Grid from -40 to 40 on both axes (81 × 81). Neighbors are 4-connected.
- Start cells, WUBRG: W (0,-2), U (2,-1), B (1,2), R (-1,2), G (-2,-1). Home angles, degrees clockwise from +x in screen space (y down): W -90, U -18, B 54, R 126, G 198.
- `step_cost(seed, salt, cell) = 0.2 + (-ln(unit(hash(seed, salt, cell))))^2`. `COST_SALT = 0x636f7374`.
- `color_cost(seed, c, cell) = (0.6*step_cost(seed, COST_SALT, cell) + 0.4*step_cost(seed, COST_SALT ^ mix(c+1), cell)) * (1 + 0.3*(1 - cos(atan2(y,x) - home_c)))`.
- A single min-heap of claims `(cost, cell, color)`, ordered cheapest first, ties by smaller cell `(x, y)` then smaller color index. Push each color with quota > 0 at its start with cost 0.
- Pop: skip if that color's quota is used up or it already visited this cell. Mark visited (per color). If the cell is free, the color takes it (quota -1) and `toll = 1`; else (another color holds it) `toll = 6` (crossing). Push each unvisited (for this color) on-grid neighbor with `cost + toll*color_cost(neighbor)`.
- After the heap empties, any color still short takes free cells adjacent to any claimed cell, choosing by (Manhattan distance to its own nearest tile, then `color_cost`, then cell), until its quota is met.

### 6.5 Pocket filling, layout, shade, ring

- `close_pockets`: up to 4 passes. Find empty cells inside the bounding box that a flood fill from one cell outside the box cannot reach ("walled"). For each, in row order: the color holding most of its 4 neighbors (ties WUBRG) takes it, and that color gives up one of its tiles that has an open (outside-reachable) side, choosing the one with the most open sides, then the farthest from (0,0) (x²+y²), then the smallest cell. Quotas stay exact.
- Sort tiles by (y, x). `column = x - min_x`, `row = y - min_y`. `columns`, `rows` = bounding box size.
- Middle = `((columns-1)/2, (rows-1)/2)`. For each tile, `ratio = euclidean(column,row → middle) / max over tiles` (0 if max is 0).
- `band(ratio, steps)` = number of k in `1..steps` with `ratio*steps >= k`.
- `shade = 4 - band(ratio, 4)` (4 = full at the middle, 1 at the farthest). `ring = band(ratio, 7)` (0..=6).
- Fill: shade 4 → the color's mana var; shade 1, 2, 3 → mix of the mana color over the panel background at 30%, 55%, 80%.
- Accessible label: `"Color map. {Name} {p} percent, {name} {p} percent..."` (largest first, percent = round(n*100/sum), first name capitalized, the rest lowercase), then `". Colorless {n} card(s)"` when colorless > 0; empty: "Color map. No colored cards yet".

### 6.6 Rendering

- Each tile is a square at a 12 px pitch with a 12% gap and an 18% corner radius. The SVG is `columns*12 × rows*12` px.
- The map runs the full container width and is up to 14rem tall; when larger it scrolls both ways (with fade edges) and opens centered on the middle.
- Entrance: each tile runs `heat-sweep` (0.4 s ease-out) delayed `ring * 50 ms`, so the map sweeps out from the middle; the last ring lands about 700 ms after mount. Held while a splash is up.
- Key under a full-bleed rule: six chips: for each of W U B R G, the mana pip glyph and the count (zeros included), then the colorless pip with `"Colorless {n}"`, then the host's `key_action` (Home: "Playstyle").
- `ColorMapGhost` (loading): the model for counts `[8, 6, 6, 5, 5]`, seed `0x5eedc0de` (about 40 tiles), drawn as ghost tiles in a box as tall as a ~2,200-card map. Ghost tiles bloom by ring in a 2.6 s infinite ease-in-out loop (grow out ring by ring, fold back tips first), dimmer toward the edge.

### 6.7 Terminal approximation

- Compute `ColorMapModel::new(&counts, seed)` exactly (it is pure).
- Draw each tile as two terminal cells ("██") so tiles look square. Fill color: the palette's mana color for that tile's color, blended toward the background at 30/55/80/100% by shade. An empty cell is two spaces.
- Width needed = `columns*2`. At most about 40 columns × 40 rows; when it does not fit, crop to a window centered on the middle (the app scrolls; a terminal can pan with arrow keys or just crop).
- Optional: a half-block mode ("▀" with fg = upper tile, bg = lower tile) packs two rows per line.
- Entrance: reveal ring by ring, 50 ms apart.
- Key line under it: `W {n}  U {n}  B {n}  R {n}  G {n}  Colorless {n}`, with mana-colored letters (or pip glyphs if the font has them).

---

## TUI today (`zwiper-tui/src`)

What exists:

- `Screen::Home(HomeView)` (`inbound/app/mod.rs`, `inbound/ui/home.rs`, `domain/home/mod.rs`). Opening home queues `Request::PublicMetrics`, `Request::PlayStyle`, `Request::Flavor` every time (`open_home`).
- Title line "Home" plus the username; the ZWIPE logo (or the compact Z, or none, by terminal size) with its own decode entrance (`domain/decode`, 850 ms).
- Figures line: "{n} Decks created   {n} Searches run   {n} Cards swiped" with full thousands separators (`home::grouped`), or "Counting..." while missing. Same figure math as the app (`Figures::from`).
- Scope line: "Everybody" / "Only me" plus "  s to switch". `s` toggles and fetches my metrics when switching to Only me. `r` reopens home (refetching everything), keeping the scope.
- Color map panel: per-color horizontal bars (W U B R G then C) sized against the max, with counts, plus a "Most shared: WU 3  ..." line from `HttpColorMap.pairs`. Empty: "Build a deck to see your playstyle"; failure: "Could not load your playstyle"; loading: "Loading...".
- Featured flavor panel (only when the card has non-blank flavor text): italic quote, then the name and USD price (`card_price` + `format_amount`). Side by side from 90 columns wide.
- Keys: d decks (also Enter), a account, c config, s everybody/only me, r refresh, O sign out, q/Esc quit.
- Status line "Hello, {username}!" set at sign-in.
- Swipe screen with ratatui-image card images (`inbound/ui/swipe.rs`), horizontal bar rows (`inbound/ui/charts.rs::row`) and vertical `columns` charts.

Gaps and differences, Home:

- No Activity entry or screen.
- No "Live" chip; no distinction between restored and fresh counts.
- No count roll animation (`CountUp`) and no compact `28.0k` format; the TUI prints full digits.
- Scope is not persisted across launches (app restores it from the keyring); default always Everybody.
- No 60 s counter cache, no saved public counts from the last launch, no per-user cache for my counts; `r` and every return to home refetch.
- Public metrics failure leaves "Counting..." forever; the app keeps the last copy (even from the last launch).
- Color map is bars plus pairs, not the seeded tile map; the app never shows pairs. No ColorMap key chips, no "?" topic, no "Playstyle" button (no Playstyle screen at all).
- The app shows an empty-state when `deck_count == 0` even if the map has data; the TUI checks the same (`deck_count > 0`), which matches.
- Featured flavor: no hourly cache (refetched on every home open; app caches to the top of the next UTC hour and keeps the last card on failure); no "Image" preview; no "Buy" dialog or retailer links; no skeleton (the panel is just absent while loading). Quote omits the app's trailing space (cosmetic).
- No user refresh on home (`fetch_user`), so no "Verify your email!" warning on each visit and no once-per-launch "Hello" gated on verification; the TUI's status greeting ignores verification.
- No preferences refresh on home (theme and pip style mid-session).
- No first-login welcome hint and no `mark_hint_shown`.
- Navigation keys exist for Decks, Account, Config; Activity and Playstyle need keys.

Gaps, other screens:

- Activity: missing entirely (fetch, heatmap, readout, totals, rate chips, role and tag rate charts, Most skipped rows, hint).
- Playstyle: missing entirely (profile sections, tag table, Show row, list with reasons and actions, staples, Clear skips, suggested swipe mode).
- Commander maybeboard: missing entirely (list, quick add search, pip filter, filter sheet, swipe in commander mode, clear).
- Oracle tag dictionary and examples: missing; no oracle tag catalog cache; no tag descriptions anywhere.
- No one-time hints, no "?" topic dialogs, no toasts (the TUI has a single status line), no expandable card rows, no printing sheet, no create-deck seed flow.
- Decks screen has no "More" menu with "Commander maybeboard" and "Playstyle".

## Shared pieces the TUI needs

1. Pure logic to lift out of Dioxus crates so the TUI can call it (today it lives in `components` or `zwiper`, both depending on dioxus):
   - `ColorMapModel`, `ColorMapCounts`, `seed_from_id` (`components/src/color_map.rs`), ideally into zwipe-core (or a no-dioxus module). The TUI's `Cargo.toml` depends only on zwipe-core and zwipe-client.
   - Activity math from `screens/activity/sections.rs`: `heat_layout`, `level`, `day_tags`, `year_tags`, `outcome_bars`, `RateKind` (`parts`, `percent`), `role_rate_bars`, `baseline`, `lift_score`, `tag_rate_bars`, `has_anything`.
   - Playstyle helpers from `suggested_commanders.rs`: `identity_of`, `identities_present`, `of_identity`, `shown`, `identity_label`, `empty_identity_line`, `staple_percent`, `staple_decks`, `color_lines`, `group_shared_tags`, `profile_summary`, `role_bars`, `impact`, `tag_rows`, `role_tags`, `has_type`, `staple_types_present`, `staple_colors_present`, `matching_staples`, `color_map_has_data`.
   - orangui helpers: `peak_indices`, `with_separators`, `short` (compact numbers), `CountUp`'s `rolling`/`waiting` if the roll is copied.
   - Buy links: `tcgplayer_card_url`, `cardkingdom_card_url` (`zwiper/src/lib/outbound/buy_links.rs`).
   - `CountScope` persistence (the TUI could keep it in its config dir beside the theme file).
2. Already in zwipe-core and usable now: `card_role::role_short_label`, `colors::identity_name`, `Color::to_long_name`/`to_short_name`, `oracle_tag::search_oracle_tags`, `deck_metrics::card_price`, `PriceCurrency::format_amount`, `DeckMetrics::bar_pct`, `Cards::matching`/`sorted`, `CardQueryBuilder`, all `Http*` contract types and `ACTIVITY_*`/`PROFILE_*`/`MAX_SUGGESTION_LIMIT` constants, hint keys.
3. zwipe-client calls needed: `fetch_user`, `fetch_preferences`, `public_metrics`, `my_metrics`, `play_style_summary`, `featured_flavor`, `activity`, `commander_suggestions(limit)`, `dismiss_commander`, `undismiss_commander`, `clear_commander_dismissals`, `fetch_commander_maybeboard`, `add_commander_maybeboard_card`, `remove_commander_maybeboard_card`, `clear_commander_maybeboard`, `search_commanders`, `search_cards`, `fetch_deck_profiles`, `fetch_oracle_tags`, `mark_hint_shown`.
4. TUI widgets to build once and reuse:
   - Toasts with levels (info, success, warning, error) and durations 3/5/8 s and 1.5 s, replacing the single status line.
   - A one-time hint dialog plus a "?" topic dialog (all the topic texts in 0.3), "Got it" button.
   - Confirm dialog (Cancel / danger action) and a "More actions" menu.
   - Accordion of collapsible sections with one open at a time and a per-section "?".
   - Chip rows: single-select ("Show"/"Type"/"Color"/rate kinds) and multi-select (WUBRG pips), with a label chip.
   - Horizontal bar chart with selectable labels and a reveal row under the selected bar (`%` suffix option, uppercase or as-is labels); vertical column chart with a title (existing `charts::columns` is close).
   - Heatmap grid: 7 rows × ~54 columns, one glyph per day (e.g. "■" or two-cell blocks) in 5 levels of the accent, peaks highlighted, month and weekday labels, the year totals readout and a cursor to pick a day (shows that day's tags for 4 s).
   - ColorMap tile renderer (6.7) with the key line.
   - Expandable card row list with art toggle, qty tags, footer and extra actions; image preview via ratatui-image; printing picker.
   - Text input with debounce for quick add, floating result list.
   - Oracle tag catalog cache (24 h TTL, single fetch) with description lookup and an Examples browser (left = next, down = back, pages of 25 by EDHREC rank).
   - Stats strip with optional roll animation and compact numbers; "Live" and scope chips.
   - External URL opener for the Buy links (or print the URL when no opener exists).
