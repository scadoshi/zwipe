# zwiper-web architecture

zwiper-web is a client-rendered Dioxus web app. It follows zwiper's hexagonal split, so a screen ported from the app keeps the shape it had there.

```
src/
├── main.rs                 builds the ZwipeClient from BACKEND_URL and launches App
└── lib/
    ├── config.rs           BACKEND_URL and APP_VERSION, fixed at build time
    ├── domain/             rules zwipe-ui does not hold yet, as pure functions with tests
    ├── outbound/           the browser: storage, the session, theme, counter and count scope stores, new tabs, the clipboard, the timer
    └── inbound/
        ├── app.rs          the root: stylesheets, theme, splash, upkeep, toasts, hints, dialogs, keys, router
        ├── caches.rs       what the root fetches once and shares: counters, my counters, the playstyle summary, the flavor card, changelog, franchises, the Create deck seed
        ├── card_filter/    the card filter panel: a side panel, the app's sheet, or docked open beside Add and Remove
        ├── card_stores.rs  zwipe-ui's card stores in root signals: filters per deck, parked add stacks, the undo log, adds in flight
        ├── cards/          card rows, image preview, printing sheet, card info, oracle tag reveals, the swipe stack and Swipe select, shared charts, buy links
        ├── catalog.rs      the catalog lists over zwipe-ui's store, fetched on demand
        ├── chrome/         the authed-call facade, the ? hints, one-time hints, collapsible sections, page keys and Ctrl+Z, the window width, sheet forms, the More menu
        ├── root_signal.rs  signals owned by the root scope, for state root tasks write
        ├── router.rs       Route, one variant per screen at the app's path
        ├── session/        ensure_fresh, the upkeep loop, the auth gate, logout, the funnel events
        ├── shell/          the top nav Shell, PublicFrame, ScreenFrame, Help, nav
        └── screens/        one module per screen (the deck screens share `deck/`, the card screens are `deck/cards/`), plus not_found.rs
```

## The root

`App` calls `use_session_upkeep`, which provides these contexts to everything below it:

- `Signal<Option<Session>>`, loaded from `localStorage` at start and owned by the root scope, since the refresh writes it from a task that outlives any screen.
- `Signal<ZwipeClient>`, built in `main` from `BACKEND_URL`.

State a root task writes (a `spawn_forever` that outlives the screen that started it) is made with `root_signal::use_root_signal`, so it belongs to the root scope those tasks run in; `use_signal_for_root_tasks` does the same for a screen's own signal and drops it with the screen.
- `Signal<ThemeConfig>` (the session's preferences, else the stored theme, else the default) and `Signal<PipStyle>`. Every theme change is stored, so Login opens in the last one.
- `UpgradeRequired`, the min-version gate. While it is true the root draws `UpdateRequired` in place of the router.

The upkeep loop runs `ensure_fresh` and then the min-version check, at once and every 60 s after. `ensure_fresh` is the only way to an authed call: it refreshes a token that expires within two minutes, one refresh at a time, stores the new session before setting the signal, and clears the session when the server rejects the refresh. `AuthGate` sends the user to Login the moment the session is gone, with no message, as the app does.

`use_root_caches` provides what screens share without fetching again: everybody's counters for the public heroes (one copy, refetched by a hero that mounts when it is a minute old, stored for the next visit), the changelog (fetched once per visit, `Loading`, `Loaded` or `Failed`) and the Universes Beyond franchises. `LoginPrefill` carries the username from Register back to Login.

The root also mounts the kit's `ToastStack` (screens raise toasts with `dioxus_primitives::toast::use_toast`), the `HintHost` that draws whatever `?` topic is posted to the root `Signal<Option<HintTopic>>`, the dialog host (`AlertDialog` anywhere draws there), and the app's splash with its `SplashGate`, which holds the first screen's decode and counters until the splash fades. One page-wide key listener handles Escape (an open dialog first, as a click on its overlay would, then the top of the overlay back stack), Enter in a dialog (its non-danger primary action, never a danger one) and the Themes panel's arrows and Enter.

## Chrome

- `chrome::authed`: `use_authed()` gives a `Copy` handle whose `run`, `try_run` and `run_quiet` refresh the session, make the call, and on failure log it and (except `run_quiet`) toast `to_user_message()` for 5 s. The app also reports each failure to telemetry; the web has none.
- `chrome::hint_topic`: the root `HintHost` for zwipe-ui's `HintTopic`, `InfoButton { topic }` for a row label and `open_topic(topic)` for a kit field's `on_info`. `chrome::hint_body` draws zwipe-ui's typed hint copy with the kit's hint pieces, so no copy lives here.
- `chrome::one_time_hint`: `use_one_time_hint(OneTimeHint::X)` (core's enum) opens a hint once per account and records it with `mark_hint_shown`; `open_and_record_hint` is the same for a gate that passes later. `OneTimeHintDialog` draws every hint from zwipe-ui's copy; `SwipeSelectHint` takes the overlay's mode.
- `chrome::section`: the app's `CollapsibleSection`. Inside a `web-sections-open` box a wide window shows every section open.
- `chrome::page_keys`: `use_page_keys` for a screen's single-key shortcuts such as `/`, only for the newest screen and never while typing or with a dialog open.
- `chrome::sheet`: `SheetFrame { form: SheetForm::Modal | SheetForm::SidePanel, BottomSheet { .. } }` keeps the kit's sheet and moves it on a wide window.
- `chrome::more_menu`: `use_more_menu()`, `MoreButton` in the action bar and `MoreMenu` beside it: a popover under the button on a wide window, the app's sheet (titled `More` unless the screen passes the app's title) below the breakpoint.

## Cards

- `cards::card_row`: `CardRow` over the kit's row, with `use_card_overlays()` and `CardOverlaysHost` for the `Image` preview and the printing sheet, `ShowRowArt` for the `Art` chip, and the image beside the cursor on a wide window.
- `cards::otag`: `use_otag_reveals()` provides the tag description lookup and the `Examples` opener; `OtagExamplesHost` draws the examples over the screen.
- `cards::swipe_stack`: the stack with mouse and touch drag, `SwipeButtons`, and `use_swipe_keys` for the arrows; keys and buttons go through `SwipeControl` so they animate like a drag. `cards::swipe_select` is Swipe select in suggested and commander modes.
- `cards::charts`: zwipe-ui bars as kit bars, the user's color map seed and counts, the map's ghost and empty line.
- `cards::buy_links`: `Buy` as one link per retailer that opens a new tab, as zite does it, for a card (`CardBuyLinks`) and as a `More` item for a deck (`MenuBuyLink`); the URLs are zwipe-ui's `buy`.

## Decks

- `screens::deck::form`: the deck form Create and Edit share. `use_deck_form()` gives a `Copy` `DeckForm` of signals, one `SlotState` per command-zone slot, and runs the form's rules (the name check, a format change turning each `Filter` back on, a commander change clearing its partner and background). `DeckFields` draws the two columns, `DeckFormOverlays` the three pickers and a Swipe select per slot.
- `domain::deck` holds what the form sends: `FormValues`, `create_request`, and `update_request`, which leaves unchanged fields off the wire and is `None` when nothing changed, the rule behind `Save changes`.
- `screens::deck::view`: the deck view's panels, its read-only cards column over zwipe-ui's `deck_cards::pass`, and `DeckMore`, the menu and its dialogs.

## Cards and filters

- `card_stores`: `CardStores` holds zwipe-ui's `FilterStore`, `AddStackCache`, `UndoStore` and `InFlightAdds`, plus Add's live search stack and the filter that produced it, in signals the root owns. Work that outlives a screen (an add's answer, a skip, a promote) runs with `spawn_forever` and records into them, so the deck cards view's `Undo` sees swipes made on Add and Remove, and their gesture undo skips a call `Undo` already made.
- `card_filter`: `CardFilterPanel` edits a zwipe-ui `FilterDraft` over the host's applied filter and hands the committed filter to `on_apply`; the host stores it and runs its search or pass again. Its rows are drawn from zwipe-ui's `sections::rows`, the chips from `domain::filter_chips`. `use_filter_docked(true)` docks it as a column at 1280px and up (Add, Remove), where `Back` drops the edits and the panel stays; elsewhere it is a right side panel on a wide window and the app's sheet below. The Oracle tags rows open the dictionary over it.
- `screens::deck::cards`: `add` (Search and Maybeboard sources, Synergy, the parked stack, the gestures and their undo), `remove`, and `view`, the deck cards editor: zwipe-ui's `deck_cards::pass` and `Bursts` for the debounced quantities, rows selected into a pane on a wide window, drag onto a board's header or chip, and the undo log walked back through `deck_cards::is_stale` and `undo_message`.

## Styling

The root loads, in order: the kit's `THEMES_CSS`, `COMPONENTS_CSS`, `MAGIC_CSS` and `APP_CSS`, zwiper's `main.css` and `accordion.css` (included from `zwiper/assets` as they are, never copied), `TOAST_CSS`, then `assets/web.css`. A screen uses the app's class names, so it looks like the app with no new CSS. `web.css` only holds what a browser window changes: the top nav, the wide layouts, the 900px breakpoint, and undoing the phone-only rules (`body` capped to the screen height, `.screen` fixed to the viewport). The fonts are copies of zwiper's, because `asset!` only reads files inside the crate.

## Adding a screen

Say the screen is Activity.

1. Make `src/lib/inbound/screens/activity/mod.rs` with a `#[component] pub fn Activity() -> Element`. Copy `screens/account/` as a starting point. A screen with route parameters takes them as props (`deck_id: Uuid`).
2. Wrap it in `ScreenFrame { title: "Activity", .. }` and put the content in `div.screen-content` the way the app does; an `ActionBar` inside the frame becomes the toolbar under the header on a wide window. A screen is one window tall and its `screen-content` is what scrolls, so a `position: sticky` column sticks to it. Sheets and dialogs go beside the frame, not inside the action bar, which is a containing block for fixed elements. Lay the panels out as `web-layout.md` says, with classes in `web.css` named `web-<screen>-*`.
3. A one-time hint is `use_one_time_hint(OneTimeHint::Activity)` with a `OneTimeHintDialog` beside the frame and the frame's `on_hint` reopening it.
4. Take the screen's rules, copy, timings and caps from zwipe-ui. What zwipe-ui lacks goes in `src/lib/domain/<name>.rs` as pure, tested functions marked `// Belongs in zwipe-ui`. The component calls them; it does not repeat them.
5. Call the API through `use_authed()`, which refreshes the session and toasts a failure. Unauthenticated calls go straight to the `Signal<ZwipeClient>`.
6. Add `pub mod activity;` to `screens/mod.rs` and a route for it in `router.rs`, then its nav entry in `shell/nav.rs` if it has one.
7. If the screen belongs to a nav entry other than the one `Section::of` gives it, change that match in `shell/nav.rs` and its test.

Every string comes from the spec, verbatim. A screen that differs from the app in more than layout and input gets a line in the kept deviations of `parity-plan.md`.

## Tests

Pure logic is tested in `domain/` and the stores in `outbound/` against `MemoryStorage`. `BrowserStorage` off wasm reads nothing and refuses writes, so a native test never touches real storage. Components are checked by building for wasm and by the live audit.
