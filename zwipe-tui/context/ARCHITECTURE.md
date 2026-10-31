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
│   ├── home/  decks/  deck/  swipe/  account/  config/
│   └── update_required/
└── widgets/        shared drawing pieces, each with its own tests
```

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
- `hero`: the logo and everybody's three counters on the sign-in screens, over the shared `Counts` copy. `hero::mount(cx)` makes one and fetches the counts when they are stale.
- Animations, each driven by elapsed time with a matching `animating` check for `Screen::animating`: `logo` (the decode, `logo::draw(f, area, art, palette.logo, Some(elapsed))`), `count_up` (`CountUp::new(start, value)`, `set`, `shown_after(elapsed)`), `grid` (the rain, `grid::draw(f, palette, area, Some(elapsed))` and `grid::raining`), `entrance` (`entrance::draw(f, area, elapsed)`; the app runs it on every new screen), `spinner`, `splash` and `wipe` (the app runs both). Logos draw in `palette.logo`.
- `text_input`: a one-line box with cursor movement, masking, a placeholder and an error line. The caller places the terminal cursor with `cursor_position`.
- `chips`: a chip row picking one or many.
- `collapsible`: a section that opens and closes, and an accordion that keeps one open.
- `info`: the "?" popup of bullet points.
- `action_bar`: the bottom row of key hints, and `action_bar::buttons` for a row of buttons the focus moves across.
- `skeleton`: ghost rows for content still loading.
- `stats`: a strip of figures whose digits roll until they load. A screen showing one should animate while `stats::loading` is true.
- `browser`: `browser::open(cx, url)` runs `open` on macOS and `xdg-open` elsewhere, and shows the link in a toast when that fails.
- `clipboard`: `clipboard::copy(text)` writes an OSC 52 sequence.
- `mana`, `charts`, `panel`, `measure`: mana colors and pips, bar charts, a centered framed panel, and text sizes.

A widget no screen uses yet carries `expect(dead_code)` on its `mod` line in `widgets/mod.rs`. The screen that starts using it deletes that line; the build says so if it is forgotten.
