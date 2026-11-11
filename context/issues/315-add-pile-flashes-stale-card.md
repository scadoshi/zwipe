# 315. Add's pile flashes a stale card after applying a filter

State: reproduced | Owner: scotland | PR: #501, #516, #530 | Build seen: 1.18.14

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

## Verification
Droid, live aea5d98c4, 2026-11-10 (clips on the laptop, `~/Developer/droid-clips/2026-11-10-verify-aea5d/clips/315-*`): back to reproduced.
- Web, phone width in the emulator's Chrome: on every redeal the ghost goes about 0.3 to 0.4s before any card shows; other cards flash first, and the top card draws half way while it streams in.
- App: passes, but right after the filter's Apply the old top card shows for one frame as the sheet slides away.

## Second fix
scotland, #516.
- Cause, web: the ghost goes when the pile reveals, and the reveal also comes from the 1.5s fallback. On a slow phone network that fires before the top card's image is in, so the slot shows nothing, then whichever card under it loaded first, then the top card still arriving. A load event also comes before the image is decoded, so even an on-time reveal could leave a frame before it paints.
- `FlippableCardImage` counts an image as loaded (its `img-loaded` class and `on_load`) only once `img.decode()` resolves, where the browser has it.
- On Add, app and web, the ghost stays mounted under the stack after the reveal and hides only once the top card's image is loaded (or the card has no art), and the cards under the top stay hidden until then (`screens-a.css`). A pile the fallback shows early shows the ghost, never an empty slot or another card.
- App: Apply bumps the filter counter, and the screen hands the filter to the model from an effect a frame later. Until it has, the screen draws the ghost, so the frame after Apply never shows the old top card.
- Measured on web, 390 wide, headless with each pile's image requests held (the top card's 3s, the others 0.4s, so the fallback fires first), three fresh loads each: release showed a card from under the top for 95 to 96 frames per load; this branch 0, with the ghost held until the top card shows. No unit test: this is render and CSS; the probe is the check. The app is not run on scotland.

## Phone width, third fix
Droid, live 21ca16a04 (with #516), emulator Chrome 145 at 411px, 2026-11-11: still failing on the web at phone width. First load showed another card, then the top half drawn; Apply and Refresh showed a card from under the top half drawn while it downloaded. Desktop passed. Cause, from droid's clips and three tailnet candidates:
- An image that had loaded once this session counted as loaded in any new element (`FlippableCardImage`'s seen set), so a stack card could show at full opacity while its own copy was still downloading, drawn top-down.
- Once the top card's image counted as in, the cards under it could show while a phone had yet to paint the top.
- A fading element gets a layer of its own, which mobile Chrome draws as the page's background until it has painted it: the pile's fade-in and the ghost's fade-in each left an empty frame.

scotland, #530:
- Each stack (web and app) tracks the cards whose own image has loaded and decoded (`on_load`). Until the top card's has, and for `TOP_PAINT_GRACE` (0.4s) after, the stack draws its ghost under its cards and the cards under the top do not paint (`zwipe_ui::stack::ghost_held`, class `ghost-held`).
- Stack images no longer take the seen set (`FlippableCardImage` `trust_seen: false`), and every image counts as loaded from its own load, `loaded_here`.
- The pile no longer fades in, and the ghost on the swipe screens shows at full strength the frame it mounts.
- Failing-first test: `the_ghost_holds_until_the_top_card_shows_and_has_had_time_to_paint`.
- Droid, candidates 2 to 4 at full speed with the cache cleared: web and app first load, Apply and Refresh PASS (ghost straight to the full card, no other card, nothing half drawn, no empty frame), while live failed. Throttled runs were inconclusive: the emulator's search failed at UMTS over the tailnet, and at HSDPA both builds stayed on the ghost.
- Printings (326) is split out.
