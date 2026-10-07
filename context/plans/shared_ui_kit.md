# Shared UI kit: wiring the consumers (phase 2)

**Status: phase 1 done on `shared/crate`.** zwipe-components and zwipe-core now carry shared versions of code that zwiper, zite, the portfolio and cairn each had a copy of. Nothing switched yet: every consumer still uses its own copy. Phase 2 switches each consumer on its own review branch and deletes the local copy. This doc is the checklist for that.

## What landed in phase 1

zwipe-core:

- `COLORBLIND_THEMES` beside `ALLOWED_THEMES` in `domain/user/models/preferences.rs`, plus a test pinning `display_theme_name` for all 31 themes.

zwipe-components, for everyone:

- Re-exports `COLORBLIND_THEMES` and `display_theme_name` next to `ALLOWED_THEMES`.
- `curve`, `area`, `tip_anchor`, `peak_indices(counts, ratio, max)` (`chart_math.rs`). `curve` is the portfolio's monotone version, output identical to it.
- `Panel`'s root now carries `data-reveal`, and `NavBar`'s `ul.nav-links` and `ThemePicker`'s `.theme-switcher` carry `data-nav-glide`. Inert until a host loads the shared scripts.
- `components.css` gained a `StatsStrip` base (`.stats-strip`, `.stats-strip .stat`, `.stat-num`, `.stat-label`): only the declarations zite, the portfolio and zwiper all had, so no host's look moved.

App kit (zwiper, cairn), styled by the opt-in `APP_CSS`:

- `OverlayBackStack`, `use_overlay_back_stack()`, `use_overlay_back(open)`, `use_overlay_back_action(is_open, close)` (`overlay_stack.rs`).
- `BottomSheet { open, title, footer: Option<Element>, on_dismiss: Option<EventHandler<()>>, hidden: bool, hint: Option<Signal<bool>>, children }`. cairn's markup: `span.modal-title` and a `page-header-corner` hint button.
- `PageHeader { title, leading: Element, on_hint: Option<Callback<()>> }`, rendering `header.page-header`.
- `ThemeSheet { open, theme: Signal<ThemeConfig>, on_save: EventHandler<ThemeConfig>, on_unchanged: EventHandler<()>, hint: Option<Signal<bool>> }`. Reads `ThemeFollow` from context. Closes itself before `on_save`; calls `on_unchanged` once a discarded pick has wiped back out.
- `InfoButton { onclick }`, `HintLine`, `HintBullets`, `HintBullet`, `HintKey { color }` (`hint.rs`).
- `ToastStack { max_toasts = 3, default_duration = TOAST_NORMAL, timed_fade = false, children }` and `TOAST_CSS`, behind the `toast` cargo feature.

Site kit (zite, portfolio):

- `use_persisted_theme(key) -> Signal<ThemeConfig>` (`theme_store.rs`).
- `REVEAL_JS` and `NAV_GLIDE_JS`, targeting `[data-reveal]` and the children of `[data-nav-glide]`. They wait for `DOMContentLoaded` themselves, so they can be inlined in the head.
- `StatsStrip { figures: Vec<(Option<u64>, &'static str)>, compact: bool, source: Option<Element> }`: `div.hero-figures > section.stats-strip > div.stat > span.stat-num (CountUp) + span.stat-label`, then `div.tag-row.stats-source` holding `source` when given.
- `GalleryFrame { index: Signal<usize>, total, noun, class: Option<String>, children }` (the `.gallery-body` with prev/next) and `GalleryFooter { index: usize, total, caption: Option<String> }` (the `.gallery-footer`).
- `SITE_CSS`: the gallery base (`.gallery-body`, `.gallery-nav`, `.gallery-prev`, `.gallery-next`, `.gallery-footer`, `.gallery-caption`, `.gallery-counter`), only what both sites share.

## CSS load order

Themes, components, then the opt-in sheet, then the host's stylesheet: `THEMES_CSS`, `COMPONENTS_CSS`, `APP_CSS` or `SITE_CSS`, host CSS. `TOAST_CSS` goes where the host's toast sheet goes today, after its main stylesheet.

When a host deletes a rule that now lives in `APP_CSS` or `SITE_CSS`, that rule moves earlier in the cascade. A scan of both apps and both sites found no earlier same-specificity rule that would now beat a moved one, but eyeball each screen anyway.

Where the apps differed, the shared sheet has the common part and each host keeps its difference in its own stylesheet. After switching, trim the host's copy down to that difference:

- zwiper keeps `.modal-header { gap: 0.75rem; }` and `.pref-row { margin-bottom: 0.4rem; }`.
- cairn keeps `.modal-title { letter-spacing: 0.1em; }`, `.modal-content`'s grid background, `.info-button { font-family; cursor }`, `.hint-line` (0.6rem margin, line height, font size, last-child margin), `.hint-bullets` (font size, line height), `.hint-key { font-weight: 700; }`.
- Left entirely in the apps because they differ: `body`, `*`, `.screen`, `.label`, `.container-sm`, `.logo`, `.home-hero`, `.chip-row`, `.profile-sections`, `.profile-list`, `.card-title`, `.profile-row-value`, `.pref-note`, `.content-enter-delayed`.
- zite keeps its `.gallery-nav` colors, opacity, transition and z-index, `.gallery-nav:hover`, `.gallery-caption` animation and `.gallery-counter { color: var(--text-subtle); }`. The portfolio keeps its `.gallery-nav` background, opacity and transition, its hover, `.gallery-footer { width: 100%; }` and `.gallery-counter { color: var(--text-muted); }`.

## zwiper

- `Cargo.toml`: `zwipe-components = { path = "../zwipe-components", features = ["toast"] }`. Keep the direct `dioxus-primitives` dep: the alert dialogs and `use_toast` still use it.
- `src/bin/zwipe.rs`: add `document::Style { {zwipe_components::APP_CSS} }` after `COMPONENTS_CSS`. Replace the `ToastProvider` block and the `toasts_expanded`/`toasts_animating` signals with `ToastStack { default_duration: zwipe_components::TOAST_NORMAL, ... }` (no `timed_fade`). Swap the `TOAST_CSS` asset for `document::Style { {zwipe_components::TOAST_CSS} }` in the same place, and delete `assets/toast.css`.
- Delete `components/navigation/overlay_stack.rs`; its 13 importers (`grep -rln overlay_stack zwiper/src`) take `zwipe_components::{use_overlay_back, use_overlay_back_action, use_overlay_back_stack, OverlayBackStack}`. The back handler calls `close_top()` the same way.
- Delete `components/bottom_sheet.rs`; its 9 importers take `zwipe_components::BottomSheet`. Its header markup changes to cairn's (`span.modal-title`, hint button with the `page-header-corner` class, which adds `display: inline-flex` and `height: 1.7rem` over today's inline style). Only `screens/profile/universes_beyond.rs` passes a `hint`.
- `components/screen_header.rs`: keep `ScreenHeader` as a thin wrapper so its 23 call sites stay: `PageHeader { title, leading: rsx! { SupportButton {} }, on_hint: hint.map(|mut h| Callback::new(move |()| h.set(true))) }`. The element becomes `header` instead of `div`; no CSS selects on the tag.
- `screens/profile/preferences.rs`: delete `COLORBLIND_THEMES`, `ThemeRow` and the body of `PreferencesSheet`; render `ThemeSheet { open, theme: theme_config, on_save, on_unchanged }`. `on_save` gets the picked `ThemeConfig` and runs today's `update_preferences` call, setting `theme_config` from the response and toasting "Theme saved" on `TOAST_QUICK`. `on_unchanged` toasts "Theme unchanged" on `TOAST_QUICK`.
- `components/hint_dialog.rs`: delete `HintLine`, `HintBullets`, `HintBullet`, `HintKey` (keep `HintColored` and the dialog); the 21 files using them import from `zwipe_components`. The look comes from `APP_CSS`, identical to today's inline styles.
- `components/info_button.rs`: keep the topic-posting `InfoButton` as a wrapper over the shared one: `zwipe_components::InfoButton { onclick: move |_| hint.set(Some(topic)) }`. Gains `aria-label="What is this?"`.
- `components/home_hero.rs`: replace the `section.stats-strip` and `div.home-source` with `StatsStrip { figures, compact: true, source: rsx! { span { class: "stat-chip", ... } ... } }`. Add `.home-hero .hero-figures { display: contents; }` so the strip and the source row stay flex items of `.hero-head`, and retarget `.home-hero .home-source` to `.home-hero .stats-source`.
- `assets/main.css`: delete the rules now in `APP_CSS` (every selector in `zwipe-components/assets/app.css`), keeping the deltas above.

## cairn

- `Cargo.toml`: `zwipe-components = { git = "https://github.com/scadoshi/zwipe", features = ["toast"] }`, then `cargo update -p zwipe-components`. Keep `dioxus-primitives` for `use_toast`. `tokio` stays only if something besides the bottom sheet and the toast tap still needs it.
- `ui/mod.rs`: add `document::Style { {zwipe_components::APP_CSS} }` after `COMPONENTS_CSS`. Replace the signals at lines 131-137 and the `ToastProvider` block at 214-245 with `ToastStack { timed_fade: true, ... }` around the router, dialog host and celebration host. Swap `assets/toast.css` for `zwipe_components::TOAST_CSS` and delete the file. `timed_fade` keeps today's CSS fade: info toasts live 3s, the rest 5s.
- `ui/mod.rs` Shell, lines 282-292: `PageHeader { title, leading: rsx! { components::about::AboutButton {} }, on_hint: hint() }`.
- Delete `components/navigation/overlay_stack.rs` and `components/bottom_sheet.rs`; importers (11 lines) take the crate's.
- `components/hint.rs`: delete `InfoButton` (line 39) and `HintLine`, `HintBullets`, `HintBullet`, `HintKey` (lines 84-117); keep `ScreenHint`, `use_screen_hint`, `HintDialog`, `HintChip`.
- `screens/config.rs`: delete `COLORBLIND_THEMES` (32), `display_theme_name` (35-55, output identical to core's for every theme), `ThemeRow` (741) and `PreferencesSheet` (777); render `ThemeSheet { open: preferences_open, theme, hint: Some(hint_open), on_save: move |_| toast.success("Theme saved", TOAST_NORMAL), on_unchanged: move |()| toast.info("Theme unchanged", TOAST_QUICK) }`. Import `ALLOWED_THEMES` stays if anything else uses it. Behavior change: the shared sheet re-reads dark mode each time it opens, as zwiper's does. cairn's copy read it once at mount, so after the Config dark toggle a theme pick reverted the mode.
- `domain/counter/heat.rs`: `peaks` (154) becomes `peak_indices(&counts, PEAK_RATIO, MAX_PEAKS)` over the in-range logged days, mapped back to their dates. Port nothing else: `quartiles` and `level` are cairn's alone.
- `components/heatmap.rs:119`: `class: tip_anchor(t.left)`.
- `components/line_chart.rs:240`: `smooth_path` can become `curve`. This changes the line's shape (Catmull-Rom to monotone, which never overshoots) and the path text format (`M0.0,10.0 C...` to `M 0.0 10.0 C ...`), so it is an owner call, not a mechanical swap; the two tests at 269-278 go with it.
- `assets/main.css`: delete the rules now in `APP_CSS`, keeping the deltas above.

## zite

- `src/main.rs`: replace `let mut theme = use_signal(ThemeConfig::default)` and the `loaded` signal with `let theme = zwipe_components::use_persisted_theme("zwipe.theme");`. Keep providing it as context. The load effect at 145-153 keeps only the `hydrated` class eval; the body-class effect at 174-190 drops the `loaded`/`save` lines. Delete `mod theme_store` and `src/theme_store.rs`; `web-sys`'s `Storage` feature may then be unused.
- Scripts: replace the `REVEAL_JS`/`NAV_GLIDE_JS` assets (27-28, 214-216) with `document::Script { {zwipe_components::REVEAL_JS} }` and `document::Script { {zwipe_components::NAV_GLIDE_JS} }`; delete `assets/reveal.js` and `assets/nav-glide.js`. Add `"data-nav-glide": "true"` to `div.nav-stores-persistent` (main.rs:251). Panels already carry `data-reveal`.
- Add `document::Style { {zwipe_components::SITE_CSS} }` after `COMPONENTS_CSS` (or copy `site.css` in through `build.rs` like the other two sheets).
- `components/stats_strip.rs`: keep the data fetch, render `StatsStrip { figures: vec![(decks, "Decks created"), (searches, "Searches run"), (swiped, "Cards swiped")], source: rsx! { Link { ... } ... } }`.
- `pages/home.rs:294-326` and `pages/guides/mod.rs:92-127`: `GalleryFrame { index, total, noun: "demo" / "screenshot", class: "guide-gallery-body" (guides only), video/img with key }`, then the host's `hr.gallery-rule`, then `GalleryFooter { index: index(), total, caption }` (the guide passes its clamped `i`).
- `assets/style.css`: trim the gallery rules to the deltas above.

## portfolio

- `src/main.rs`: same theme swap as zite (`use_persisted_theme("zwipe.theme")`, lines 101-126); the load effect keeps only the `hydrated` eval. Delete `theme_store.rs` and the `gloo-storage` dep.
- Scripts: same swap as zite (19-20, 176-178), delete the two asset files. Add `"data-reveal": "true"` to `div { class: "heatmap" }` (heatmap.rs:288); the reveal used to select `.heatmap` too. Do not mark `nav-stores-persistent`: the portfolio's glide never included it.
- Add `SITE_CSS` after `COMPONENTS_CSS`.
- `pages/home.rs:66-97`: `StatsStrip { figures: vec![(Some(totals.commits), "Commits"), (Some(u64::from(totals.repos)), "Repos"), (Some(totals.stars), "Stars")], source: rsx! { ...the three tag links/chips... } }`.
- `components/gallery.rs`: body becomes `GalleryFrame { index, total, noun: "image", ...media }`; the `actions` slot becomes `GalleryFooter { index: index(), total, caption }`. The caption element changes from `figcaption` to a keyed `span`; keep the `figure` wrapper, or keep the local footer if the figcaption matters.
- Delete `components/curve.rs` (identical output) and `chart::anchor` (use `tip_anchor`); `heatmap.rs:175` `peaks` becomes `peak_indices(&counts, PEAK_RATIO, MAX_PEAKS)` mapped to dates, and its test moves with it.

## Left out on purpose

- `quartile_cuts`: only cairn computes heatmap levels; the portfolio reads them from GitHub.
- The heat palette (`.heat-1` to `.heat-4`, `.heat-peak`): identical in the portfolio and cairn, but those classes sit on cells whose base fill rule (`.heatmap-cell`, `.heat-cell`) has the same specificity, so loading them earlier in a shared sheet would let the base fill win.
- The scroll reveal's own CSS (`body .reveal-pending` in zite, `.theme-wrapper .reveal-pending` in the portfolio) stays in each site.
