# 315. Add's pile flashes a stale card after applying a filter

State: fixing | Owner: thinker | PR: - | Build seen: 1.18.14

## Report
Droid, 2026-11-08, Android app (prod): on Add, applying a filter shows one card for about 0.3s (Deadly Rollick), then a different top card (Drakuseth) replaces it. Same in an earlier take (Abrade, then Rakdos Charm), so it isn't new. Likely the old pile's next card renders before the filtered pile arrives. Footage: ~/Developer/video-factory/videos/oracle-tags/footage/raw/o4.mp4 at ~9.6s.

## Diagnosis
thinker, 2026-11-09, by reading the code on release. Not reproduced on a device.

**Root cause: the top card is see-through until its own image loads, and a new pile loads all its images at once.** The pile state is right. The flash is a card from the *new* pile showing through the top card, not the old pile's next card.

- A card image is `opacity: 0` until its bytes arrive (`components/assets/magic.css:1188-1202`). `onload` then adds `img-loaded` (`components/src/flippable_card_image.rs:170-171`), unless the URL is already in the session's seen set (`:143`).
- The swipe stack draws `STACK_DEPTH = 10` cards (`zwiper/src/lib/inbound/components/interactions/swipe/stack.rs:40`) with `PEEK_OFFSET_PX = 0.0` (`:43`). Every layer is `.swipe-stack-card`, `position: absolute; inset: 0` with no background (`components/assets/screens/screens-a.css:22`), so all ten sit exactly on top of each other. The top card's image is passed without `on_load` (`stack.rs:289`).
- Every action that deals a new pile also emits `Effect::EaseImages`, which empties the seen set (`zwipe-ui/src/screens/add_cards/mod.rs:751` in `fresh_search`, `:1488` in `refresh`; `zwiper/.../card/add.rs:184`). So all ten images start at opacity 0 and race. Whichever lands first shows, through the transparent top, until the top card's image arrives about 0.3s later.

The model is not at fault:
- Applying a filter runs `fresh_search`, which clears everything from the cursor on (`mod.rs:748`).
- Redeal runs `refresh`, which resets the stack (`:1481`).
- The bolt toggle emits `Rerun`, which the client turns back into `Action::Filter` (`add.rs:177`, `:610`), so it is `fresh_search` too.
- In all three, `pile()` is `Loading` and the client draws the skeleton until the reply lands (`mod.rs:1637`), and `replace_ahead` puts the new top first (`:821`).
- The client hands the filter over from an effect (`add.rs:610`), which leaves only one render frame of lag, not 0.3s.

This also explains Droid's second report (bolt toggle and redeal: a one-card flash, then Zurgo). The card that flashes is whichever new card's image loaded first. It happens on every newly dealt pile, not only after a filter.

**Web:** affected the same way, but not through the shared screen model. zwiper-web's Add still runs its own driver over `zwipe_ui`'s `CardStack`. It draws the same stacked `.swipe-stack-card` layers with the same `FlippableCardImage` and no `on_load` (`zwiper-web/src/lib/inbound/cards/swipe_stack.rs:267`, `:310`), and it resets the seen set on a new search and on refresh (`zwiper-web/src/lib/inbound/screens/deck/cards/add.rs:267`, `:868`). Faster desktop loads likely make it rarer, not absent.

## Proposed fix
Keep the existing loading skeleton up, ghosting the whole pile, from the moment a new pile lands until its top card's image has loaded. Then show the stack.
- Model (`zwipe-ui` add_cards): a fresh or refreshed search that serves cards sets a `revealing` flag, and `pile()` stays `Pile::Loading` while it's set.
  - A new `Action::TopShown` clears it.
  - So do swipes, undo and source switches, so a gesture never waits on it.
  - A reply that serves nothing goes straight to `End` or `Empty` as today.
- App: while the model is revealing, render the stack invisibly (`visibility: hidden`), so its images can load, with the same `CardSkeleton` over it. Pass the top card's `FlippableCardImage` an `on_load` that dispatches `Action::TopShown`. `on_load` already fires on a failed load or a card with no art, and the featured-cards strip uses it the same way (`zwiper/src/lib/inbound/screens/deck/components/featured_cards.rs:29-36`). Add a short ceiling (about 1.5s) that dispatches `TopShown` anyway, so a stalled image can't hide the pile.
- Web: the same `revealing` gate in its own driver and `SwipeStack`, until web Add moves onto the shared model.

## First failing test
In `zwipe-ui/src/screens/add_cards/tests.rs`: `a_new_pile_stays_ghosted_until_its_top_card_shows`.
- Load, deliver the deck, deliver a search page with cards. Assert `pile() == Pile::Loading` and `current()` is the page's first card.
- Dispatch `Action::TopShown`, then assert `pile() == Pile::Cards`.
- Repeat for `Action::Filter`, `Action::Refresh` and `Action::ToggleSynergy` followed by its rerun `Action::Filter`.
- Add a second test: a swipe while revealing clears the flag.

Today the first assertion fails, because `pile()` is `Cards` as soon as the page lands.

## Fix
thinker, 2026-11-09, as proposed above, on `nits/thinker-315`.
- Model: `State::revealing` holds the ticket of a pile that landed but whose top card is not on screen yet. `pile()` reads `Loading` and `current()` is `None` meanwhile. `Action::TopShown { ticket }` ends it, and so do undo, a source switch and every new search.
- App and web: the stack renders hidden under `CardSkeleton` while the pile reveals. The top card's `on_load` sends `TopShown`, with `timing::PILE_REVEAL_FALLBACK` (1.5s) as a ceiling.
- Inside the reveal, the stack fades in as one piece and its images skip their own fade. Otherwise the top image, still easing in from transparent, would show the card under it for a frame or two.
- Reproduced on web with mocked replies and the top image held 0.9s. On release, an under-card showed for 46 to 53 frames on load and redeal. On the branch: 0 frames, the ghost held about 0.95s, then the real top faded in.

