# zwipe-tui architecture

Every screen lives in its own folder under `src/lib/inbound/screens/`, and the rest of the crate reaches it through one match arm per screen in `screens/mod.rs`. Two people adding two screens touch the same file only on those arm lines.

```
src/lib/inbound/
├── app/            App (the back stack plus Shared), Context, Effects, Navigation
├── nav/            Nav, the back stack
├── event_loop/     the loop: keys, results, settling navigation, spawning requests
│   └── harness.rs  test-only Harness that drives screens with no network
├── screens/
│   ├── mod.rs      Screen, Request, Update, and the arms that route to each screen
│   ├── login/      mod.rs, keys.rs, draw.rs, tests.rs
│   ├── register/  forgot_password/  update_required/
│   ├── home/  activity/  playstyle/  commander_maybeboard/  commander_swipe/
│   ├── oracle_tags/  otag_examples/  printings/  image_preview/
│   ├── decks/  deck/  deck_form/  deck_import/  deck_export/
│   ├── swipe/  deck_cards/  remove_cards/  card_sheets/
│   └── account/  config/  privacy/  changelog/
└── widgets/        shared drawing pieces, each with its own tests
```

## Screens

- Sign-in: `login`, `register`, `forgot_password`, and `update_required`, which the app draws over every screen while the server refuses this version.
- Home and what it leads to: `home` (counters, color map, featured card), `activity` (`v`), `playstyle` (`p`, the color map's Playstyle), `commander_maybeboard`, and `commander_swipe`, the `SwipeSelect` overlay in its Suggested and Commander modes.
- Oracle tags: `oracle_tags`, the dictionary, whose Use leaves a slug in `shared.caches.handoff.oracle_tag` for the screen under it; `otag_examples`, a tag's example cards; `printings`, a card's printings to pick or browse; and `image_preview`.
- Decks: `decks` (its More sheet opens the Commander maybeboard and Playstyle), `deck`, `deck_form` (Create and Edit; `open_create_with_commander` seeds Create from Playstyle, the commander swipe and the maybeboard), `deck_import`, `deck_export`.
- Cards: `swipe` (Add), `deck_cards`, `remove_cards`, and `card_sheets`, which holds the filter, details and printing sheets those screens and the commander screens open over themselves.
- `account`, `config`, `privacy`, `changelog`.

A sheet that opens a screen over its host, as the filter sheet's Dictionary does, gets its answer back through the host's arm of `Screen::on_return`: `card_sheets::filter::returned_from_dictionary(sheet, cx)` adopts the slug and says whether the return was the dictionary's, so the host skips its own refresh.

## How a key or a result flows

A key goes to `Screen::handle_key` on the screen at the top of the stack, which calls that screen's `keys::handle(view, code, cx)`. The handler changes its own state directly. Anything else it asks for through the `Context`: `cx.request(..)` queues background work, `cx.push`, `cx.back`, `cx.replace` and `cx.reset` queue navigation, `cx.quit()` ends the app, `cx.toast(..)` shows a toast, and `cx.shared` holds the theme, the user, the status line and the card-image picker. Handlers never touch the network or the stack themselves.

When the handler returns, `EventLoop::settle` applies the navigation in order. Going back pops the top screen and calls `Screen::on_return` on the one uncovered, which is how the deck view reloads after swiping and the deck list reloads after a deck closes. Back from the bottom screen quits.

Each queued `Request` runs on a tokio task through `screens::perform`, which hands a screen's own requests to that screen's `perform(account, request)`. The answer comes back as an `Update`. `screens::apply` routes a screen's update to its `apply(nav, update, cx)`, which finds its own screen anywhere in the stack with `nav.find_mut(pick)`, so a result still lands when another screen is open over it. A screen decides for itself what an update means when its screen is closed; config's preferences, for one, still set the theme.

The app's own work sits in the same enums as plain variants: `CheckVersion`, `SignOut`, `RememberTheme`, `MarkHintShown`, and the `TooOld`, `SessionEnded`, `SignedOut`, `Status`, `Toast` and `HintMarked` updates.

## Adding a screen

Say the screen is `search`.

1. Make `screens/search/` with `mod.rs`, `keys.rs`, `draw.rs` and `tests.rs`. Copy `decks/` as a starting point: it is the smallest screen with background work.
2. In `search/mod.rs`, define the state (`SearchView`), and if it does background work, a `Request` and an `Update` enum with a `From` impl for each into `screens::Request` and `screens::Update`. Add `open(cx, ..)` that pushes the screen and queues its first requests, `perform(account, request)`, and `apply(nav, update, cx)` with a private `pick` that matches `Screen::Search`.
3. In `search/keys.rs`, write `handle(view, code, cx)`. Esc is usually `cx.back()`.
4. In `search/draw.rs`, write `draw(f, shared, view)`. Use `shared.palette` for every style.
5. Register it in `screens/mod.rs`, one line each:
   - `pub(crate) mod search;`
   - a `Screen::Search(Box<SearchView>)` variant
   - an arm in `Screen::handle_key` and one in `Screen::draw`
   - a `Request::Search(search::Request)` variant and its arm in `perform`
   - an `Update::Search(search::Update)` variant and its arm in `apply`
   - an arm in `Screen::animating` only if something on it moves, and in `Screen::on_return` only if it should refresh when a screen over it closes
6. Call `search::open(cx)` from the screen that leads to it.

A screen with no background work, like `update_required`, skips the request and update enums and their arms.

Tests go through `Harness`: `Harness::on(Screen::Search(..))` or `harness.open(search::open)`, then `press`, `type_text`, `apply`, and assertions on `pending()` and `screen()`. `test_support::screen_text(&harness.app)` renders the app at 100 by 40 for drawing tests.

## Widgets

All live in `widgets/` and draw in the account theme's `Palette`.

- `toast`: success, error, info and warning toasts in the top-left corner, at most three, the newest on top and the older ones tucked behind it; Ctrl+T expands them. `cx.toast(kind, message)` shows one for `toast::NORMAL` (5 s), `cx.toast_for(kind, message, toast::QUICK | NORMAL | LONG)` for 3, 5 or 8 s, and the `Toast` and `ToastFor` updates do the same from a result.
- `alert`: `AlertDialog` with a title, rules, a body and buttons, one focused; a button can be danger or disabled, and `draw_with` leaves room under the body for a field.
- `sheet`: a bottom sheet with a scrolling body, a focused line kept in view, an optional `?` corner and footer keys. `sheet::frame` draws only the chrome, for a sheet with a form inside.
- `header`: the screen header, `!` on the left, the title, and `?` on the right when the screen has a hint. `!` and F1 open `help`, the Help & feedback dialog (`help::open(cx)`).
- `hint`: `?` topics and one-time hints. `hint::topic(cx, HintTopic::Email)` opens a topic; `hint::once(cx, key)` opens the app's one-time hint for a key from `zwipe_core::domain::user::hints` the first time only, marks it and sends `mark_hint_shown`; `hint::reopen(cx, key)` opens it again for a header `?`; `hint::keys::swipe_select(..)` with `hint::once_with` covers the one hint whose copy changes. While a hint is up it takes every key.
- `hero`: the logo and everybody's three counters on the sign-in screens, over the shared `Counts` copy. `hero::mount(cx)` makes one and fetches the counts when they are stale. Home reads the same `shared.counts` for its Everybody figures and asks for `Request::FetchCounts` under the same 60 second rule, so Login and Home never fetch twice; the user's own counters live in `shared.caches`.
- `figures`: home's and Activity's compact strip of three rolling figures.
- `color_map`: the color map's tiles two cells wide in their four shades, the ring sweep, the loading ghost, and the key with its chips and Playstyle button.
- `stagger`: entrances inside a screen, delayed and rising, and the hero's scope-in.
- `card_rows`: expandable card rows with classification, printings (`printings::open` in `Mode::Pick`) and image keys, for Activity, Playstyle and the maybeboard.
- `otag`: an oracle tag's definition with its `x Examples` key, and `RoleTags`, a row of tag chips with one open. The details sheet and the deck cards view's open row reveal a card's tags with it.
- `item_list`: a list of items of different heights, one focused, paged to keep it in view.
- `card_text` and `card_info`: a card as text, and the card screens' info block, details body and text stand-in.
- `menu` and `pill`: a sheet of buttons, and one chip drawn on its own.
- `text_area`: a text box of many lines, for pasting a decklist.
- Animations, each driven by elapsed time with a matching `animating` check for `Screen::animating`: `logo` (the decode, `logo::draw(f, area, art, palette.logo, Some(elapsed))`), `count_up` (`CountUp::new(start, value)`, `set`, `shown_after(elapsed)`), `grid` (the rain, `grid::draw(f, palette, area, Some(elapsed))` and `grid::raining`), `entrance` (`entrance::draw(f, area, elapsed)`; the app runs it on every new screen), `spinner` (a request in flight), `splash` (the opening over the first screen) and `wipe` (the sweep between themes); the app runs the last two itself. Logos draw in `palette.logo`.
- `card_motion`: a swiped card flying off and an undone one coming back, after the app's `card-stack-exit-*` (0.18 s) and `card-stack-enter-*` (0.2 s), both on CSS `ease-out`. The card slides off in the swipe direction, clipped at the bounds, and narrows toward an edge-on sliver (its width for left and right, its height for up) to stand in for the tilt, dim for the second half. Down is undo and has no exit. A swipe screen adopts it in four steps:
  1. Keep `motion: Option<CardMotion<T>>` on the view. `T` is whatever draws the card that left; `card_motion::Snapshot { card, face }` suits a card screen.
  2. On a committed swipe, before the stack moves on, set `CardMotion::exit(direction, snapshot, Instant::now())`. On an undo that brings a card back, set `CardMotion::enter(side_it_left, Instant::now())`. A new swipe replaces the motion; keys act on the stack at once.
  3. In `draw`, keep only a motion that is `running(now)`. Draw the resting card first: the new top card during an exit, the card under the top one during an entrance. Then take `motion.placement(now, rest, bounds)` for the moving card's resting rect and call `card_motion::draw(f.buffer_mut(), &placement, palette.base, |buf| widget.render(rest, buf))`. An exit draws its snapshot; an entrance draws the stack's own top card.
  4. Report `running(Instant::now())` from the screen's arm in `Screen::animating`.

  `card_motion::side_style` gives a direction's color. Add holds Sixel and iTerm2 images back while a card moves, since those protocols draw as one escape that later cells skip; half blocks and kitty placeholders are ordinary cells and stay.
- `text_input`: a one-line box with cursor movement, masking, a placeholder and an error line. The caller places the terminal cursor with `cursor_position`.
- `chips`: a chip row picking one or many.
- `collapsible`: a section that opens and closes, and an accordion that keeps one open.
- `info`: the "?" popup of bullet points.
- `action_bar`: the bottom row of key hints, and `action_bar::buttons` for a row of buttons the focus moves across. A swipe screen builds its swipe keys with `action_bar::swipes(palette, right, left, up, down)`, which colors each verb like the app's swipe hints (right success, left error, up warning, down tertiary), and draws them with `styled_line`. The labels use only add, remove, skip, maybeboard and undo (the commander swipe and the oracle tag Examples too: Examples reads skip and undo); toasts and hint prose stay the app's own words, so Remove's skip still toasts "Kept".
- `skeleton`: ghost rows for content still loading.
- `stats`: a strip of figures whose digits roll until they load. A screen showing one should animate while `stats::loading` is true.
- `browser`: `browser::open(cx, url)` runs `open` on macOS and `xdg-open` elsewhere, and shows the link in a toast when that fails.
- `clipboard`: `clipboard::copy(text)` writes an OSC 52 sequence.
- `mana`, `charts`, `panel`, `measure`: mana colors and pips, bar charts, a centered framed panel, and text sizes.

A widget no screen uses yet carries `expect(dead_code)` on its `mod` line in `widgets/mod.rs`, and an item no screen uses yet in a widget that is otherwise in use carries one of its own. The screen that starts using it deletes that line; the build says so if it is forgotten. A helper only tests call is `#[cfg(test)]`.
