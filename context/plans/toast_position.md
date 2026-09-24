# Move toasts to the top left

**Status: SCOPED 2026-09-24, not started. Ships on a store build, so it rides whatever release is next rather than going out on its own.**

Toasts currently sit bottom right, 7rem up from the bottom, which puts them directly over the `util-bar` on any screen that has one. That bar is where the buttons live, so a toast lands on the controls someone is most likely reaching for at the moment it fires. Cairn hit the same thing and moved to the top left; `~/Developer/cairn/assets/toast.css` carries the reasoning in a comment.

## What this actually is

Almost entirely a CSS change to one file, `zwiper/assets/toast.css`. None of the 152 `toast.info/success/error` call sites move, the provider mount does not change, and no Rust changes are required for the position itself.

The work is in deciding what the toast covers instead, and in three details that are easy to get wrong.

## What is there now, measured

The provider is mounted in `zwiper/src/bin/zwipe.rs:125` with `max_toasts: 3` and `class: "toast-container"`. That class reaches the DOM through `#[props(extends = GlobalAttributes)]` on `ToastProviderProps`, which is what every selector in `toast.css` hangs off. The library owns the inner DOM: `div.toast-container > ol > li > div[role="alertdialog"][data-type]`.

Position today is `right: 1rem; bottom: 7rem`, with `flex-direction: column-reverse` and `margin-top: -2.5rem` on the items, so the stack overlaps into a deck with a z-index ladder keeping the newest on top. Animation is a single 0.2s `toast-slide-in`, and the library removes the toast when its duration expires.

Durations are set per call site rather than from named constants: 1500ms appears 64 times, 3000ms 21 times, 2000ms 16 times, 2500ms 13 times, and a handful of others. Nothing in the CSS depends on those numbers, which matters below.

## What transfers from cairn, and what does not

Cairn's positioning block transfers directly:

```css
left: 1rem;
top: calc(env(safe-area-inset-top, 0px) + 4rem);
```

The `4rem` clears its header. Ours would need to clear `.page-header`, which is `position: relative` inside a flex column with `flex-shrink: 0`, so it stays put and never scrolls away. A fixed toast at the top left sits over it permanently unless the offset clears it.

Cairn also swaps the overlapping deck for a plain `column` with `gap: 0.5rem`, because overlapping left the older toast peeking out a sliver and a new arrival read as landing on top of the previous one rather than above it. That reasoning applies here too, and it deletes the z-index ladder.

**What does not transfer is cairn's animation.** Cairn drives the whole lifecycle from CSS with a `toast-life` keyframe keyed on `data-type`, which forces the stylesheet to know how long each toast lives. Its own comment says change one and change the other or a toast pops out while still opaque. Ours has no such coupling: a 0.2s slide-in and the library handles removal. Copying cairn's animation wholesale would import that coupling across 10 distinct durations at 152 call sites. Do not do it. Flip the slide direction and stop there.

## The decision worth making before writing any CSS

Top left is not empty. `.page-header-corner` holds the left button on 23 screens: the hint "?" on most, the support "!" on login. So the question is not whether a toast covers something, it is which thing is worse to cover.

Covering the bottom bar interrupts an action in progress. Covering the header corner hides a button that is nearly always optional and rarely pressed mid-flow. That argues for top left, and it is the same argument cairn made, but it is worth the owner looking at a real screen before it ships rather than reasoning about it here.

Clearing the header entirely is the other option: an offset that puts the toast under `.page-header` covers screen content instead of chrome. On the add screen that content is the card, which is the thing being swiped.

## Steps

1. Change the four positioning lines in `zwiper/assets/toast.css`: `left` for `right`, `top` with a safe-area offset for `bottom: 7rem`.
2. Replace `column-reverse` plus negative margins with `column` and a `gap`, and delete the five `li:nth-child` z-index rules that only existed to order the overlap.
3. Flip `toast-slide-in` so it enters from above rather than below, matching the direction the stack now grows.
4. Look at it on a device on the three screens that matter most: the add screen mid-swipe, a deck with the bottom sheet open, and login.

## Traps

`components/toast/` held a wrapper around the library provider that nothing used, and it dropped `attributes`. Switching the mount to it would have lost the class and stopped every rule in `toast.css` from matching, silently. Deleted 2026-09-24, before this plan starts, so the only `ToastProvider` in reach is the library's. Its siblings `accordion` and `alert_dialog` are real and stay.

Toasts are `z-index: 9999` and the highest thing in `main.css` is 230, so they float above modals and the bottom sheet. At the top left they will overlay the top of a bottom sheet rather than its buttons. Probably fine, worth seeing.

The safe area matters more at the top than the bottom. `env(safe-area-inset-top)` on a notched device is substantial, and a fixed offset without it puts the first toast under the status bar.

## Verification

There is nothing here a test can assert: it is position and animation. The check is a device pass on the three screens above, in both themes, with three toasts stacked so the gap and ordering are visible. Trigger three quickly from the add screen, which is the only place that naturally produces a burst.

## Out of scope

Named duration constants. Ten distinct durations across 152 call sites is its own cleanup, and it only becomes necessary if the CSS ever drives the lifecycle the way cairn's does.

zite has no toast stylesheet and does not use toasts, so this is zwiper only.
