# Plan: the web stylesheet split

Step 3 of the web nits order in [`../progress/web_nits.md`](../progress/web_nits.md), a launch blocker: zwiper-web stops inlining `zwiper/assets/main.css` and owns the rules it draws with. It also settles item 12 (scrollbars on a wide window). This is a scoping write-up; no code moves until a phase below is planned for a worker.

## How the web gets the app's CSS today

`zwiper-web/src/lib/inbound/app.rs` pulls both of zwiper's sheets in at compile time with `include_str!` across the crate boundary (`ZWIPER_MAIN_CSS` from `../../../../zwiper/assets/main.css`, `ZWIPER_ACCORDION_CSS` from `../../../../zwiper/assets/accordion.css`) and writes each into an inline `document::Style`. Only `web.css` is a real asset (`asset!("/assets/web.css")`, a `document::Link`).

The order in the web's root, every sheet inline except the last:

| # | Sheet | Source | Lines |
|---|---|---|---|
| 1 | `@font-face` block | inline in `app.rs` | 3 |
| 2 | `THEMES_CSS` | orangui `assets/themes.css` | 4,825 |
| 3 | `COMPONENTS_CSS` | orangui `assets/components.css` | 2,811 |
| 4 | `MAGIC_CSS` | `components/assets/magic.css` | 2,040 |
| 5 | `APP_CSS` | orangui `assets/app.css` | 655 |
| 6 | `ZWIPER_MAIN_CSS` | `zwiper/assets/main.css` | 4,202 |
| 7 | `ZWIPER_ACCORDION_CSS` | `zwiper/assets/accordion.css` | 130 |
| 8 | `TOAST_CSS` | orangui `assets/toast.css` | 276 |
| 9 | `web.css` | `zwiper-web/assets/web.css` (linked) | 1,956 |

The app (`zwiper/src/bin/zwiper.rs`) loads the same sequence, with `main.css` and `accordion.css` as linked assets in slots 6 and 7 and no `web.css`. zite loads `THEMES_CSS`, `COMPONENTS_CSS`, `MAGIC_CSS`, `SITE_CSS` and its own stylesheet, and never sees `main.css`.

Because the include is a compile-time read of another crate's file, `.github/workflows/deploy-web.yml` carries `zwiper/assets/**` in its path filter ("The web build still inlines the app's stylesheets"), so an app-only CSS edit rebuilds and redeploys the web, and can break it, without anyone looking at the web.

## Which main.css rules the web uses

Method: parse `main.css` into rules (comments stripped, `@media` nesting followed, rules inside the 21 `@keyframes` set aside), take the class names in each selector, and match them against every class-like token in string literals of the code that renders web markup (`zwiper-web/src`, `components/src`, orangui's `src`, `zwipe-ui/src`, with `format!` prefixes such as `card-stack-exit-{..}` matched as prefixes). A selector counts as used when all its classes appear. The same pass against `zwiper/src` gives the app's side. It overcounts a little (any literal containing the word counts) and misses classes built only in JavaScript, so treat the numbers as an estimate.

Of 526 rules outside keyframes:

| Bucket | Rules | Meaning |
|---|---|---|
| Shared | 409 | Matched by both the web's and the app's markup |
| Web only | 10 | Matched by the web's markup alone (mostly skeleton ghosts the web added) |
| App only | 31 | Matched by the app's markup alone |
| Global | 6 | No class in the selector: the reset layer, `html`, `body`, form fonts, the scrollbar hide |
| Unused | 70 | Matched by neither; candidates for deletion |

So about 80% of `main.css` (419 rules) reaches the web, and `web.css` already has rules on the classes of 118 of those. The web is not borrowing a few rules; it is drawing with the app's sheet.

The biggest used groups, by `main.css` section (used of total):

| Section | Used |
|---|---|
| skeleton | 59 of 70 |
| label row with clear button | 33 of 34 |
| oracle-tag dictionary | 28 of 29 |
| Zwipe select | 26 of 26 |
| cards/containers | 20 of 23 |
| swipe stack | 17 of 21 |
| home screen | 16 of 20 |
| browse screen | 16 of 17 |
| info list | 15 of 15 |
| header and main menu | 14 of 26 |
| action dock | 12 of 17 |
| screen layout | 10 of 12 |

The app-only rules cluster in the phone chrome: header and main menu (5), action dock (5), skeleton (10, app-only ghosts), swipe stack (4), deck view actions (3). The unused 70 are spread through the utility sections (spacing 10, page header 8, typography 7, borders 6, layout 5) and old screens (logo 3, home 4).

## Options

**(a) Copy the used rules into the web's own sheets and stop inlining `main.css`.** Cost: about 420 rules copied into `zwiper-web/assets/`, then pruned against `web.css`'s 118 overrides. Risk: two copies of 80% of a stylesheet that drift apart from the first app change; every shared fix lands twice or the clients diverge unseen, the same blind spot as today, flipped. Cheap to start, expensive forever.

**(b) Move the shared rules into `components/assets/magic.css`.** Cost: the same move as (c) or (d), into a file that already exists. Risk: zite loads `MAGIC_CSS`, so zwipe.net would ship ~3,300 lines of app screen rules it never draws, including `.screen { position: fixed; inset: 0 }`, `.chip`, `.card-image` and the swipe stack, which could collide with zite's own markup. `magic.css` today is the Magic UI pieces all three clients draw; app screen rules don't belong there.

**(c) Split `main.css` into a shared file and an app-only file inside `zwiper/assets/`, the web including only the shared one.** Cost: the same move, no new crate surface. Risk: the web still reads a file in the app crate by path, `deploy-web.yml` still has to watch `zwiper/assets/`, and an app developer editing "zwiper's" shared file still doesn't know the web draws with it. It fixes the scrollbar leak but not the ownership problem the item is about.

**(d) A new shared sheet in `components`, loaded by the app and the web but not zite.** `components/assets/screens.css`, exported as `components::SCREENS_CSS`, holds the 419 rules both clients draw with. `main.css` keeps only the app's own rules (phone chrome, the `html`/`body` lock, hidden scrollbars, safe areas), `web.css` keeps only the web's, and zite never loads the new sheet. Cost: the same move as (c), plus one constant in `components/src/lib.rs` and one line in each client's root. Risk: cascade order shifts while rules move (see the phases), which is manageable and temporary.

**Recommendation: (d).** It is the only option where each sheet has one owner and the shared rules live in the crate both clients already share for exactly this reason. Code that both clients use already lives in `components` or `zwipe-ui`, never in one client's crate. `deploy-web.yml` can then drop `zwiper/assets/**` (it already watches `components/**`), and an app-only CSS change stops redeploying the web. `accordion.css` follows the same path: the web's filter accordion draws with it, so it moves to `components` too, or into `screens.css`.

## Item 12 under the recommendation

The rule responsible today is `main.css` lines 82 to 94, in the base section:

```css
*::-webkit-scrollbar { width: 0; height: 0; display: none; }
* { scrollbar-width: none; -ms-overflow-style: none; }
```

It hides every scrollbar on every element, and the web gets it through the inline `ZWIPER_MAIN_CSS`. Nothing in `web.css` undoes it. (The kit's own `scrollbar-width: none` rules are scoped to the nav panel and the nav dropdown and are not the cause.)

Under (d) the rule is app-only: it stays in `main.css`, which the web no longer loads, so a wide window shows the browser's scrollbars with no web rule at all. For the phone layout `web.css` adds the same two rules inside `@media (max-width: 899px)`. The same phase deletes the reverts in `web.css` that exist only to undo `main.css`'s base (`html { overflow: auto; touch-action: auto }`, `body { max-width: none; overflow: auto; user-select: text }`), since the web never sees the rules they undo.

If item 12 has to ship before the split, the stopgap is a `web.css` override at `min-width: 900px` that restores `scrollbar-width: auto` and the WebKit bar sizes; the split then deletes it.

## Phases

Each phase leaves both clients drawing exactly as before, except the last, which is the visible change (scrollbars). Phases 1 to 4 can run on four machines at once.

**Phase 0, the empty frame (one worker, small).** Add `components/assets/screens/` with one empty file per batch below (`base.css`, `components.css`, `screens-a.css`, `screens-b.css`), a `SCREENS_CSS` constant in `components/src/lib.rs` that `concat!`s them in `main.css` order, and load it in both roots right after `APP_CSS` and before `main.css`. Nothing changes on screen. This phase owns `components/src/lib.rs` and both root files, so no later batch needs to touch them.

**Phases 1 to 4, the moves (one worker each, in parallel).** Each worker moves one contiguous span of `main.css` into its own file, leaving the app-only rules of that span in `main.css` and deleting confirmed-unused rules only after grepping the class across all clients. Each worker edits a different file under `screens/` and a different region of `main.css`, so git merges the four branches cleanly.

| Phase | `main.css` span | Target | Holds |
|---|---|---|---|
| 1 | lines 8 to 963 | `base.css` | reset layer, form fonts, screen layout, action dock, deck view and list header actions, header and main menu, utilities. `html`, `body`, the scrollbar hide and the app-only chrome rules stay in `main.css` |
| 2 | 964 to 1768 | `components.css` | buttons, inputs, labels, cards and containers, skeleton |
| 3 | 1769 to 2860 | `screens-a.css` | swipe stack, carousel, home, spinner, toggle, images, dropdown, chips, card image preview, compact accordion and modal, collapsible |
| 4 | 2861 to 4202 | `screens-b.css` | label row, stepper, browse, info list, settings, Zwipe select, dictionary, privacy, featured strip, heatmap |

Cascade note: while some spans have moved and others haven't, a moved rule loads before every rule still in `main.css`, where it used to load after the earlier ones. That only matters when two rules of equal specificity in different spans set the same property on the same element, so each worker greps its span's selectors against the spans before it and keeps any such pair in its own span's order (or raises it). Once all four are in, `SCREENS_CSS` runs in `main.css`'s original order and the question goes away. `web.css` loads last throughout, so its overrides keep winning.

Check per phase: the app and the web at a phone width and at a wide width on the screens the span covers, before and after. Phase 3 and 4 workers check the swipe screens and the deck pages; the skeletons need `SKELETON_DELAY_MS` to stay up long enough to compare.

**Phase 5, the cut (one worker, after 1 to 4 merge).** Drop `ZWIPER_MAIN_CSS` and `ZWIPER_ACCORDION_CSS` from the web's root (moving `accordion.css` to `components` first if Phase 3 didn't absorb it), drop `zwiper/assets/**` from `deploy-web.yml`'s paths, add the narrow-only scrollbar hide to `web.css`, and delete the `html`/`body` reverts. Then sweep `web.css` for overrides that only undid an app-only rule the web no longer loads. This is the one visible change: scrollbars on a wide window (item 12).

**Phase 6, optional.** The kit's `APP_CSS` is the same kind of question one level down (an app sheet the web loads whole); look at it once the split has settled, not as part of this plan.
