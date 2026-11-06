# Shared hover preview and card filter

**Status: SCOPED (2026-11-06), nothing built.** Web nits 22 and 21 in [`../progress/web_nits.md`](../progress/web_nits.md), with 23 and 11 hanging off 21. Both moves put code in `components` so zwipe.net (zite) and the web app (zwiper-web) draw the same thing.

## 22. The card row hover preview

### zite today

`zite/src/pages/shared_deck.rs` wraps the shared `components::CardRow` (imported as `SharedCardRow`) in its own `CardRow`, which wires the row's `on_hover_enter`, `on_hover_leave` and `on_face_change` to a page-level stack.

- State lives in `SharedDeckView`: `preview_stack: Signal<Vec<PreviewCard>>` (`PreviewCard { id: u64, url, leaving: bool }`), `preview_next_id: Signal<u64>`, `preview_on_right: Signal<bool>` and `viewport_width: Signal<f64>` (read once from `window.innerWidth` on wasm).
- Enter pushes the row's showing face (`face_image_url(current_face, Normal)`) to the front of the stack with a fresh id and truncates to 5. The row remembers that id. No timer runs while the cursor stays on the row.
- Leave starts that entry's life: after `PREVIEW_LIFE_MS` (2000) it is marked `leaving`, and after `PREVIEW_FADE_MS` (400, the `sd-preview-out` animation) it is removed. Each entry has its own timer, so previews linger and fall away one by one after the cursor moves on.
- Flipping a row's face pushes the new face as its own entry with the 2 s timer started at once (there is no leave to retire it).
- The same card hovered twice gets two entries (ids are per hover, not per card).
- Placement: the stack is `position: fixed`, `top: 5.5rem; left: 1.5rem; width: 18rem; z-index: 50`, entries absolutely placed `calc(i * 2.6rem)` apart with `z-index: 200 - i`, newest on top. An `onmousemove` on the page's outer `div.shared-deck` sets `preview_on_right` when the cursor is in the left half of the window (written only on change); `.pin-right` glides the stack to the right gutter with `transform: translateX(calc(100vw - 100% - 3rem))` over 0.35 s.
- Motion: `sd-preview-in` (0.35 s, drops 40px from above and fades in), `sd-preview-out` (0.4 s, fades out 28px down).
- Hidden below 1100px (`@media (max-width: 1100px)`), and never filled on touch (no `mouseenter`).
- Row highlight: zite adds `@media (hover: hover) { .card-row:not(.expanded):hover { scale(1.02), 0.4rem radius, 8% accent tint, shadow-sm } }` in `zite/assets/style.css`, the same look `magic.css` gives `.card-row.expanded`. The web has no hover highlight on rows today.
- CSS: `.sd-hover-stack`, `.sd-hover-stack.pin-right`, `.sd-preview-card`, `.sd-preview-card img`, `.sd-preview-card.leaving`, the two keyframes and the 1100px rule, about 65 lines in `zite/assets/style.css` (~1767 to 1830), plus the row hover rule (~1997).

### The web today

`zwiper-web/src/lib/inbound/cards/card_row.rs` wraps the same shared row in `div.web-card-row` and has two modes.

- Pinned, when the screen provides the `HoverPreview(Signal<Option<HoverCard>>)` context: enter sets `hovering`, waits `HOVER_DELAY` (450 ms), and if still hovering sets the signal to the card's primary image (`HoverCard { url, name, x }`, x being the cursor's). Leave and drag start clear it at once. `HoverPreviewHost` draws the one card as `.web-hover-preview.web-hover-pinned`, 15rem wide, under the nav (`top: calc(var(--sheet-drawer-top, 5.5rem) + 0.5rem)`), its `left` a clamp that throws it to the gutter away from the cursor.
- Floating, with no context: the same 450 ms settle, then the row draws its own `.web-hover-preview` beside the cursor (`--hover-x`, `--hover-y` from the row's `onmousemove`), gone on leave.
- It always shows the front face (`primary_image_url`), not the flipped one.
- Both are visible only at `(min-width: 900px) and (hover: hover)`. CSS: `web.css` ~973 to 996 (floating) and ~1954 (pinned).

Screens that host the web's preview:

- Pinned: the wide deck page only, `screens/deck/view/page.rs` (`use_context_provider(|| HoverPreview(hover))` ~103, `HoverPreviewHost` ~541). `/deck/get/:id` and, on a wide window, `/deck/card/view/:id` (which returns `ViewDeck`).
- Floating: every other wide screen with card rows. Playstyle (`playstyle/rows.rs`, the suggestions and Staples), Commander maybeboard (`commander_maybeboard.rs`), Activity (`activity/sections.rs`, `HardPassRow`). The phone deck cards list (`deck/cards/editor.rs` `row_view` via `cards/view.rs`) only draws below the breakpoint, where the preview is hidden.
- None: Add, Remove and Swipe select draw no `CardRow`.

### Proposed `components` API

One module, `components/src/hover_stack.rs`, re-exported from `lib.rs`.

- `HoverStack`: a `Copy` handle (`Signal<Vec<HoverEntry>>` plus `Signal<u64>` for ids and `Signal<bool>` for the side), from `use_hover_stack()`. Methods: `enter(url: String) -> u64` (push to front, cap 5, no timer), `leave(id)` (start the 2 s life, then the 0.4 s fade, then remove), `flash(url)` (push with the timer already running, for a face flip), and `track(x: f64)` (set the side from the cursor, written only on change).
- `HoverStackHost { stack: HoverStack }`: draws the stack. Mount it outside any transformed ancestor (`position: fixed` must mean the window), as `HoverPreviewHost` is today.
- `use_hover_stack()` provides the handle as context. `components::CardRow` reads it with `try_use_context::<HoverStack>()` and does the enter, leave and flip bookkeeping itself (holding its own entry id, using the face it shows), so neither host writes hover code in its row wrapper. The row's `on_hover_enter` and `on_hover_leave` props stay for anything else.
- Cursor side: the host calls `stack.track(e.client_coordinates().x)` from an `onmousemove` on its page wrapper, or `HoverStackHost` takes the window width once by eval as zite does now. Dropping the `viewport_width` read for a CSS answer (`left` clamped as the web's pinned rule does) would also do; pick one in the build.
- Timers: the life and fade sleeps need a timer that works in both clients. components has no timer today; zite has `sleep_ms` and the web `outbound::timer`. Either pass a sleep in (a `Callback`), or add a small wasm-and-native sleep to components (gloo-timers on wasm, nothing elsewhere since the stack is hover-only). The second is simpler; it is a new dependency for components, so it needs Scotty's yes.
- Constants move with it: `PREVIEW_LIFE_MS`, `PREVIEW_FADE_MS`, the cap of 5, the 2.6rem step.

### CSS that moves

To `components/assets/magic.css`, renamed off the `sd-` prefix (`hover-stack`, `hover-stack-card`): the stack, `.pin-right`, the card, `.leaving`, both keyframes, the 1100px hide, and the row hover highlight, which both clients then share (the app never has `hover: hover` on a phone, so it is unaffected).

### What each host deletes

- zite: `PreviewCard`, the two constants, `preview_stack`, `preview_next_id`, the three hover handlers and `my_preview_id` in its `CardRow` (it keeps `current_face` only for the overlay), the stack markup in `SharedDeckView`, and the CSS above. It adds `use_hover_stack()`, `HoverStackHost` and the `onmousemove` call.
- zwiper-web: `HoverPreview`, `HoverCard`, `HoverPreviewHost`, `HOVER_DELAY`, `hovering`, `hover_shown`, `cursor` and the floating preview in `cards/card_row.rs`; the provider and host in `deck/view/page.rs`; both `.web-hover-preview` blocks in `web.css`. Each hosting screen (deck page, Playstyle, Commander maybeboard, Activity) adds `use_hover_stack()` and `HoverStackHost`. A screen that adds neither gets no preview, which is the choice to make for each screen.

### What the web may not accept as is

- Width: zite hides the stack below 1100px; the web shows its preview from 900px. Matching exactly drops the web's preview between 900 and 1100px. The alternative is a narrower stack there, which is no longer "exactly zite's".
- Lingering previews over overlays: an entry lives 2.4 s after leave. The stack is z-index 50; Swipe select (z-index 210), the filter drawer and the image preview sit above it, so a lingering card hides under them rather than over them, but it shows again for its last moments if the overlay closes fast. Clearing the stack when an overlay opens is one line per host if it reads badly.
- Full-height swipe screens (Add, Remove) draw no rows and need no host. The swipe card itself is not a hover surface; nothing to change.
- Drag: the web's deck page drags rows onto board headers. The web clears its preview on `dragstart`; the stack should do the same (`leave` on drag start), or the dragged card's preview sits there for 2 s.
- The web's 450 ms settle goes. Fast scanning down a list will fill the stack to 5 at once; that is zite's behavior and what item 22 asks for.

### Size

About 150 lines of Rust and 75 of CSS into components; about 130 lines out of zite and 120 out of zwiper-web plus four small host additions. One factory batch, one worker, since it touches components, zite and zwiper-web.

## 21. The full card filter on zite's shared deck

### What the drawer is

`zwiper-web/src/lib/inbound/card_filter/` (`mod.rs` 324 lines, `rows.rs` 466): `CardFilterPanel { open, filter: ReadSignal<CardQueryBuilder>, config: SheetConfig, on_apply, quiet_reset }`. Hosts today: the deck page, the phone deck cards list, Add, Remove, Commander maybeboard and Swipe select.

### What it depends on

- zwipe-ui, already a components dependency: `FilterDraft`, `SheetConfig`, `ApplyOutcome` (`filter::draft`), `Section` and the row model (`filter::sections`), `PickerField` and `Sources` (`filter::picker`), `filter::chips`, `filter::fields`, `screens::card_filter` (titles), `stores::catalog::{Catalog, Kind}`. All pure; nothing to move.
- components (orangui re-exports), already shared: `SheetFrame`, `SheetWide::Drawer`, `ActionDock`, `DockAction`, `DockPlace`, `Button`, `Chip`, `ClearWrap`, `use_overlay_back`, the icons.
- zwiper-web only, which must move or be passed in:
  - `CatalogCache` (`inbound/catalog.rs`): the root's `Signal<Catalog>` plus `ensure`, which fetches through `zwipe-client`. components has no zwipe-client and should not gain it (the zwipe-ui rule: clients fetch, then hand results in). Pass `catalog: Signal<Catalog>` and `on_need: Callback<&'static [Kind]>` instead; each client keeps its own fetching.
  - `ZwipeClient` and `Session` contexts: only used for `cache.ensure` and the one-time hint record. Gone once those are passed in.
  - Toasts: `dioxus_primitives::toast::use_toast` and `chrome::toast::raise` for `Filter unchanged`, Apply's message and refusals. zite has no toast provider. Pass `on_toast: Option<Callback<Toast>>` (zwipe-ui's toast value); zite can drop them or add a toast host later.
  - `HintTopic`/`InfoButton` (the Oracle tags `?`) and `OneTimeHint::Filter` with `open_and_record_hint` (records on the server, needs a session): web-only chrome. Make both optional slots (`oracle_tags_info: Option<Element>`, `on_first_open: Option<Callback<()>>`); zite passes none.
  - `OracleTagDictionary` (`screens/oracle_tag_dictionary.rs`, 237 lines), opened from the Oracle tags picker: depends on the catalog and an `on_examples` that opens the web's Examples browse. Either move it too with `on_examples` optional (zite has no Examples, as its card rows already say), or make the dictionary button an optional callback and leave the dictionary in the web for now.
  - CSS: the sheet and drawer come from orangui's `components.css`, which zite loads. The accordion (`zwiper/assets/accordion.css`), the picker float, `.chip-remove`, `.input-compact` and `.clear-btn` (`zwiper/assets/main.css`) and the drawer tuning (`web.css`, `.web-filter-accordion`, `.accordion-trigger`) zite does not load. Those rules, about 200 lines, move to `magic.css` or a new `components` sheet both load. Item 12's stylesheet split (laptop, `plans/web-stylesheet-split`) is reorganizing the same files; sequence after it or agree the target sheet with it.

### What zite can and cannot load

The shared deck page is a deck host, so `Sources::deck(cards)` draws keywords, oracle words, type words, artists and sets from the deck's own cards with no fetch. `Sources::Deck(..).needs()` is only `CardRoles` and `OracleTags`, and both are public: `fetch_card_roles()` and `fetch_oracle_tags()` take no session, and zite already calls `fetch_oracle_tags()` for its tag descriptions. No section needs data zite cannot get.

- Every section draws: Name, Oracle text, Types, Mana, Combat, Flavor text, Artist, Rarity, Card roles, Oracle tags, Set, Price, Sort, Config. Format shows only with `show_format`, which the web's deck page sets; zite can too.
- Deck tags need a session (`fetch_deck_tags(&session)`), but the filter does not read them (`Sources::list` returns `None` for `Kind::DeckTags`).
- Load: zite builds a `Signal<Catalog>` in `SharedDeckView`, and its `on_need` runs `api::client().fetch_card_roles()` and `fetch_oracle_tags()` the way the web's `CatalogCache::ensure` does. Its existing `otags` resource folds into the same catalog.
- Applying: zite already filters through `deck_cards::pass(&mainboard, &zone, &builder, group_by)` with a `CardQueryBuilder` it builds from the inline row. It holds a `Signal<CardQueryBuilder>` instead and hands it to the panel, the same as the web's deck page.
- zite needs a way to open it: a funnel. The web uses the action dock's `DockFilter`; zite has no dock. A Filter chip or button in the place of the inline row (which item 21 removes) is the natural spot.

### Size of the move

Roughly 1,000 lines of Rust move to components (the panel and rows, 790; the dictionary, 237, if it moves), with about 15 call-site edits across the six web hosts for the new props, and about 200 lines of CSS. zite gains about 60 lines (catalog signal, fetches, the funnel, the builder signal) and loses the inline row's three signals and the builder code. Larger than a nit: two batches, below.

## How 23 and 11 depend on 21

The inline Filter row (Name, six type chips, five pips) is part of `components::DeckControls`, which both zite's shared deck and the web's wide deck page draw. The app's Cards screen does not use `DeckControls`.

- 23 (the web drops the inline row) does not need 21. It needs the row to be optional in `DeckControls`: make `name`, `types` and `colors` `Option<Signal<..>>`, or add `show_filter: bool` defaulting on. The web passes none; zite keeps its row until 21 lands. The web's `inline_name`, `inline_types` and `inline_colors` in `deck/cards/editor.rs` and their merge into the display memo then go.
- 11 (fold-all at the end of the Filter row on zwipe.net) only holds while zite has a Filter row. 21 removes that row, so 11's target disappears. Wherever fold-all goes must survive the row going: the end of the Show row (today's spot), or the end of whatever row replaces Filter on zite (the funnel). The web's deck page has no fold-all yet (item 2) and no Filter row after 23, so it needs the same answer.
- So decide fold-all's home once, against the page without the inline row, and do 2, 11 and 23 in one pass, as the nits plan already groups them. The cheapest consistent answer is to keep fold-all where `DeckControls` draws it now, after the Show chips, and give the web's page `on_fold_all` (item 2). That makes 11 a "won't do" once 21 is planned.

## Phases

1. **Hover stack into components (22).** One worker: components, zite and zwiper-web. Needs the timer decision first.
2. **DeckControls without the inline row (23, with 2 and 11 settled).** One worker: components `DeckControls` gains the optional row; the web stops passing it and gains fold-all; zite unchanged. Touches `components/src/deck_page.rs` and the web's deck page.
3. **Filter CSS into a shared sheet (21a).** One worker, CSS only, after or coordinated with item 12's split. No behavior change: the web loads the same rules from the new place.
4. **Filter panel into components (21b).** One worker: move `card_filter/` (and the dictionary, or stub its button) with the catalog, toast, hint and info slots as props; update the six web hosts. The web looks and behaves the same.
5. **zite uses the shared filter (21c).** One worker, zite only: the catalog signal and fetches, a funnel where the inline row was, the `CardQueryBuilder` signal; pass no inline row to `DeckControls`.

Parallel: 1 and 2 can run together if 1 keeps out of `deck_page.rs` (the row hover rule moves into `magic.css`, but 2 edits `deck_page.rs` Rust, not that CSS). 3 can run beside 1 and 2 but conflicts with item 12 in `web.css`. 4 waits for 3 (or carries its CSS itself if 3 is folded in), and both 1 and 4 edit the deck page's `page.rs`, so 4 follows 1. 5 waits for 2 and 4.

## Recommendation

Do 22 now as phase 1: it is self-contained, about a day, and the only open call is the timer dependency. Do 2, 11 and 23 together as phase 2, keeping fold-all after the Show chips everywhere and recording 11 as superseded by 21. Hold 21 (phases 3 to 5) until item 12's stylesheet split has merged, then run it as three batches. Moving the filter is a real refactor with six web hosts and the dictionary, not a nit.
