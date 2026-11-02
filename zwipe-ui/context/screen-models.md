# Screen models

A screen model holds what one screen knows and decides, so the app, zwiper-web and zwiper-tui only draw it and wire input to it. The deck list (`src/screens/deck_list/`) is the worked example; the next fifteen follow it.

## Layout and naming

- One module per screen under `zwipe_ui::screens`, named after the screen in snake case: `screens::deck_list`, `screens::home`, `screens::deck_view`. A big screen splits into `mod.rs` plus leaf files; tests sit in `tests.rs` beside `mod.rs`.
- `screens` holds models only. The rules a model leans on stay where they are (`deck::list`, `deck::create_gate`, `hint`, `toast`) and the model calls them, so a client that only needs a rule never pulls in a model.
- Every model names its three types the same: `State`, `Action`, `Effect`. Clients import them through the module (`use zwipe_ui::screens::deck_list::{self, Action, Effect, State}`) so `deck_list::State` reads plainly where two models meet.
- A model's copy is `pub const`s at the top of its module (`deck_list::TITLE`, `deck_list::NO_MATCH`), or a `label()` on the enum it labels (`MoreItem::label`). The words are the app's, verbatim; a `the_copy_matches_the_app` test pins them.

## The shape

```rust
#[derive(Debug, Default, Clone, PartialEq)]
pub struct State { /* private fields */ }

pub enum Action { Load, Loaded { ticket: Ticket, decks: Vec<DeckProfile> }, Failed { ticket: Ticket }, PickGroupBy(GroupBy), /* ... */ }

pub enum Effect { RefreshUser, FetchDecks(Ticket), OfferHint(OneTimeHint), Navigate(Destination), Toast(Toast) }

impl State {
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect>;
    // read-only views: status(), groups(), message(), show_chips() ...
}
```

- `State` fields are private. Clients read through accessors and change it only through `update`, so a model can hold invariants (the deck list's `Loaded` is never empty; an empty reply is `Empty`).
- `State::new()` (and `Default`) is the screen as it first draws: loading, nothing picked. Opening the screen is the client sending `Action::Load` (or the screen's first action) and performing what comes back.
- `Action` is one variant per thing a user does plus one per reply. Values the model can't know come in on the action: `Action::Create { verified }` carries the session's verification rather than the model holding the user.
- `Effect` is everything the client must do: a fetch (named for the zwipe-core call, carrying its `Ticket`), `Navigate(Destination)`, `Toast(Toast)`, `OfferHint(OneTimeHint)`, and later timers. `update` returns them in the order to perform them.
- `Destination` is per screen and lists only the places that screen leads. Each client maps it to its route in one `match`.
- Views are computed from the state on demand (`groups()`, `show_chips()`, `message()`), not stored, so they can't go stale. Return owned values; the lists are small.
- A status enum (`Loading`, `Failed`, `Empty`, `Loaded`) tells the client which branch to draw. `message()` returns the line to show, if any, so no client picks between "No decks" and "No decks match" itself.
- Several sections can be open at once. Collapsed groups are a set, folds are independent flags; no model keeps "the open section".
- Toasts are `zwipe_ui::toast::Toast` values with kind, text and band. A request's failure toast is the exception: the client's request runner reports the error and raises it, because mapping a `ClientError` to words is the client's (inventory decision 3). The model only hears `Failed { ticket }`.

## Loading and generations

Every fetch takes a `Ticket` from the model's own `Generation` inside `update`, and the effect carries it. The client hands it back on the reply. `update` drops any reply whose ticket a newer fetch has replaced, so a reload, a fast retype or a slow first request can't land over newer data. A request that continues the current one (a next page) uses `Generation::current` instead of `next_ticket`. Tests cover it with an `a_late_reply_is_ignored` case per fetch.

## What stays in the client

- Drawing, layout, focus, scroll, hover, gestures and animation. The TUI's focused chip and selected row, the web's wide layout that keeps the tag row open.
- Overlays that are pure presentation: whether the More sheet or a `?` dialog is up. The items in them and where they lead are the model's.
- I/O: zwipe-client calls, session storage, the hint's "seen" check and record, opening URLs.
- Telemetry and error reporting, through each client's request runner.

## Variations seen so far

- Back that returns to wherever the user came from emits `Navigate(Destination::Previous)`; the client maps it to its own history back (the app's `go_back`, the TUI's `cx.back()`). Activity does this; the deck list goes to Home.
- When clients lay out a screen's sections differently (an accordion with one open on the phone and in the terminal, all open on a wide web page), openness stays in the client. The model lists which sections draw (`sections()`) and which one opens on entry (`Section::opens_on_entry`).
- A hint offered on arrival goes out with the first fetch in `Load`'s effects. The app dispatches `Load` from `use_hook` so the hint opens on the first render, as before.
- A view with more than two shapes gets its own enum instead of two flags. Activity's `Rates` is `Unrated(line)`, `Uncharted(line)` or `Charted(bars)`.
- A screen with no toast besides its request failure has no `Effect::Toast`.
- When a model shares its name with a Phase 1 module (`zwipe_ui::activity` the math, `screens::activity` the model), clients import the model `as model`.
- A screen whose data outlives a visit takes what the client holds in `Load` (Home's `Cached`). The model asks only for what is missing or old, and the client writes each reply to its own cache as well as handing it back.
- Fetches that run side by side and restart independently each get their own `Generation`, so a reply to one never drops another. Home has five: user, counts, my counts, playstyle and flavor.
- The model never reads the clock. Anything that depends on the time takes it in the action (`CheckFlavor { now }`), so tests pin it.
- When the first render must already show the cached state, the app runs `Load` inside the `use_signal` initializer instead of `use_effect`, and performs the effects it returns right after.
- A screen with many requests in flight at once (the deck view's load, commander art, edits) takes every ticket from one `Generation` and keeps a waiting list keyed by ticket, so each reply finds its own request and a reply after a newer `Load` changes nothing.
- A reload keeps the last result on screen while it runs, so a refresh after an edit doesn't flash the skeleton.
- A screen with several dialogs emits `OpenDialog(Dialog)`; which dialog is up stays in the client.
- Copying to the clipboard is `Effect::Copy { text, done }`: the client copies `text`, then raises `done`; a client that can tell the copy failed raises its own toast instead.
- A screen opened for one item takes its id in `Load` (`Load(deck_id)`).
- Client-wide values that a sheet previews over (Config's theme and pips) stay the client's. The action carries the value shown, and the model answers with `ShowTheme`/`ShowPips` for an optimistic value or a rollback, and `ThemeSaved`/`PipsSaved` once the server holds it.
- Saves that race on different controls carry a `SaveTicket` (slot plus ticket) and wait in a list. Toggles drop an overtaken reply through `Toggles` generations; a sheet's save always lands.
- A model can open and close sheets with `OpenSheet`/`CloseSheet` while openness stays in client signals.
- A draft stays readable after its sheet closes, so the sheet sliding away still draws it.
- A debounce is an effect, `Debounce { ticket, wait }`, that the client answers with `Debounced { ticket }` after the wait, so models stay free of timers. A newer edit takes a new ticket, which drops the older wait.
- A filter the client owns in a shared sheet goes into views as a parameter (`shown(&filter)`, `message(&filter)`).
- State handed to another screen goes as an effect ahead of `Navigate` (`SeedCommander(card)`), which keeps `Destination` `Copy`.
- A quiet fetch that only feeds a gate (the maybeboard's deck count) needs no ticket and sends nothing back on failure.
- Timers are a self-renewing `Effect::Wait(Ticket)` answered by `Action::Tick { ticket }`. The model counts whole seconds and asks for another wait while time is left; a wait whose count was replaced is answered and ignored, so clients never cancel one.
- Text forms can live in the model (Account keeps one per sheet), with the client sending each keystroke as `Edit`. Do this when another client duplicates the touched and attempted rules; otherwise the text stays in the client and comes in on submit.
- Passwords travel as the model's `Password`, whose `Debug` is redacted.
- A shared component with an inline confirm takes a callback (`LogoutDialog { on_confirm }`) so the confirm goes through the model.
- A screen with no fetch opens with `Open(id)` instead of `Load` (Import).
- Input that lives in the client (a text area) comes in on the submit action, and `can_submit(input)` takes it for the disabled check.
- A model that holds a zwipe-core type without `PartialEq` drops `PartialEq` from `State` and `Action`.
- A screen that reads an app-wide cache it doesn't own takes the cache's state into its views by reference (`Catalog<'_>` into `body`) and holds no copy; the client reports a failure as an action (`CatalogFailed`).
- A hand-off to the host can be an effect with no `Navigate` when the overlay stays up (the dictionary's `AdoptTag`).
- A swipe browse's enter animation is an effect (`Enter(Direction)`): the model owns the cards and the cursor, the client the animation.
- One model can serve two screens through a mode passed in `Load` (Create and Edit share `screens::deck_form` with `Open::Create { seed }` or `Open::Edit(id)`).
- A picker's draft and which picker is open belong to the model, since the draft lives exactly as long as the picker; swipe overlays stay in the client.
- A text input that owns its own signal stays in step with the model both ways.
- Effects that carry zwipe-core request contracts can't derive `PartialEq`; tests use `matches!`.
- A sheet over a host's value takes it in `Open` and hands it back with `Effect::Apply`; its Back is `Effect::Close`, not a navigation.
- A client whose controls edit the host's value in place syncs it with `Action::Edit` before each action and performs `Restore` on Back; a client with a separate draft ignores `Restore`.
- A model whose lists come from a client-wide store emits `EnsureLists` and takes no tickets.
- A warning raised once per opening resets on close, so the order of effects doesn't matter.
- Typed passwords are `zwipe_ui::account::Password` in actions, effects and state, with its `Debug` redacted. Login, Register and Forgot password give each field its own edit action; Account's shared `Edit` carries `Typed`, whose `Debug` prints only the length.
- When the model changes a field the client's input owns, it says so in an effect (`SetIdentifier`, `ClearPassword`).
- Requests that may overlap on purpose (Forgot password's Send) use the waiting list, and a flag any reply clears mirrors the app's spinner.
- Funnel events stay in the client and go out as it performs the effect.
- A model that needs a client-owned store's answer emits a request effect (`TakeUndo`) the client answers at once with an action (`Taken`), and writes back with `RecordUndo`.
- The client's request runner may hold animation state, such as clearing the entrance on a failed session refresh.
- A client-owned counter other widgets watch can keep driving the model with `Filter(builder)`.
- Shared chip rows take the value shown plus `on_pick(Option<T>)` (`None` is All), and the model decides the toggle.
- Client filter state that resets when its panel unmounts becomes a model rule keyed on a view (Playstyle's `shows_staples`).
- A reveal whose content comes from model data goes to a child component as a `Callback` reading the model (`role_tags`).
- A sheet that only closes has no `Destination` and emits `Effect::Close`.
- When the shown index belongs to a gesture or carousel, it comes in as `at` on the actions and views that need it.
- A list laid out new is `Show(at)`; a reopen on the cached list is `Rewind(at)`.
- Host props (the printing sheet's mode and Save toast) go into `State::new(..)`.
- A client-owned filter the model also writes goes out as `SetFilter` and comes back through the client's counter as `Filter`; the model's `Rerun` is that bump.
- Two piles means two entrance signals: `Enter { pile, from }` and `StopEnter(pile)`.
- An undo that needs a fresh session answers `NotSent` apart from `Failed`, because the two compensate differently.
- A stack that pages marks its search ticket as the epoch, and every page carries it.
- An overlay a host keeps mounted takes everything it needs in `Open { .. }` each time it opens; the model decides whether to keep the pile or deal afresh.
- A right swipe that hands a card to the host is `Effect::Pick`; Back on an overlay is `Navigate(Previous)`, which the host maps to its close.
- Paged searches and fire-and-forget requests share one generation and waiting list; a fresh deal drops only the pile's searches.

## Wiring it in Dioxus (the app and zwiper-web)

The app's deck list keeps the model in one signal and puts it with the handles that perform effects in a `Copy` struct:

```rust
#[derive(Clone, Copy)]
struct Driver { model: Signal<State>, navigator: Navigator, toast: Toasts, authed: Authed, /* session, client, hint_open */ }

impl Driver {
    fn dispatch(self, action: Action) {
        let effects = self.model.write().update(action);
        for effect in effects { self.perform(effect); }
    }
    fn perform(self, effect: Effect) {
        match effect {
            Effect::FetchDecks(ticket) => { spawn(async move {
                let reply = self.authed.try_run("load_decks", |c, s| async move { c.fetch_deck_profiles(&s).await }).await;
                self.dispatch(match reply { Ok(decks) => Action::Loaded { ticket, decks }, Err(_) => Action::Failed { ticket } });
            }); }
            Effect::Navigate(to) => { self.navigator.push(route(to)); }
            Effect::Toast(toast) => ui_toast::raise(self.toast, toast),
            // ...
        }
    }
}
```

- Mount sends the first action from `use_effect(move || driver.dispatch(Action::Load))`.
- Render reads `driver.model.read()` once, copies out what the markup needs, and drops the guard before `rsx!`.
- Every handler is `move |_| driver.dispatch(Action::...)`. Child components take an `EventHandler` (`ontoggle`, `onopen`) rather than the signal.
- `use_resource` goes away for the model's fetches: the generation does what the resource's restart did, and the reply goes through `update`.

## Wiring it in ratatui (zwiper-tui)

The view struct keeps the model beside its focus fields: `struct DeckList { model: State, focus: Focus, show_focus: usize, selected: usize, more: Option<Menu> }`. A key handler maps the focused chip or row to an `Action` (`model.show_chips()[show_focus].action()`, `Action::ToggleGroup(group.key())`, `Action::OpenDeck(id)`). It performs effects through its `Context`: `FetchDecks(ticket)` becomes a `Request` whose `Update` carries the ticket back into `apply`, which sends `Action::Loaded`; `Navigate` pushes or pops a screen; `Toast` calls `cx.toast_for` with the toast's kind, text and duration. After each `update` the view clamps its focus to the new rows.
