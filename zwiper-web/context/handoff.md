# zwiper-web handoff

Where the web build stands as of 2026-11-01, written so a fresh session on another machine can pick it up from this branch alone. `web-layout.md`, `parity-plan.md` and `ARCHITECTURE.md` hold the detail; this is the map.

## Since then: the unified UI (2026-11-04)

The app's layout rework (`context/plans/unified_ui.md`, shipping as 1.18.0) came to `feat/web` in five slices, and the rules below that name action bars, panel footers, More sheets or a wide nav with Account and Config pills are history. Below 900px the web is the app: its header with the back arrow and hamburger, its menu, floating docks, Danger zone panels, bottom sheets closed from an × or a drag down. On a wide window the page grammar stays, but with no bars and no footers either: the sign-in submits are full width with link chips beneath, forms save from the dock, every More is gone. `web-layout.md` describes the branch as it is now; `shared-switch.md` lists what moved into `components` and what the web still keeps as a copy.

## Branches

| Branch | What it is |
|---|---|
| `feat/web` | The one web branch: every app screen built for the browser in waves A to D, integrated and audited (`audit.md`), plus the layout review done with the owner one screen at a time. Not on main. The old `feat/web-a` to `-d`, `feat/web-int` and `feat/web-zite` were folded in and deleted. |
| `main` | Holds zwipe-ui (the shared screen-logic crate, owned by the zwiper-tui session) and the app's in-app browser. `feat/web` merges main in as each shared branch lands. |

Never push to or merge into main. The owner merges.

The shared crates (zwipe-ui, zwipe-core, zwipe-client, components, and orangui with orangui-core) belong to one session, zwiper-tui on scotland-server. Don't edit them on feat/web. When zwiper-web needs something shared, send that session a request naming what it needs and where it lives here (file and function). It builds the change on a branch off main, the owner merges it, and then main gets merged into feat/web. The ten `// Belongs in zwipe-ui` helpers are already requested; it reads them from origin/feat/web.

## Rules the owner set during review

These are recorded in `web-layout.md`; they are repeated here because they came from conversation, not from the specs.

- The page follows zite's grammar: a centered page capped at 74rem with a small top gap, a hero where a screen has one, and the other panels in two balanced columns.
- The web is panels at every width. Every input form is its own panel with a title bar named after its screen and its buttons in an action footer under a full-bleed rule. The mobile app stays panel-less; don't change it.
- Panel footer buttons are the app's compact buttons in a row, not full-width stacks.
- The top nav is zite's nav bar with the Z mark (in `var(--logo)`) linking Home and a hamburger below 900px. Every screen but Home and the three auth forms opens with a page head on a wide window (title, `?`, one-line description); below 900px the title and `?` sit centered in the folded nav. This replaced round 9's "no screen title header".
- No navigation-only buttons (Back, Back to login and the like); the nav and the browser cover them. Buttons that do something stay.
- No site footer in the web app. The Scryfall credit and the Fan Content notice live on zite and in the Privacy policy screen.
- Each screen's action bar is pinned to the bottom of the window, always on screen.
- Password rules live behind the `?` next to the Password label, as in the app.
- Lists of cards (Most skipped, Staples, Suggested commanders) keep the app's card-group header: the theme's accent-tertiary (Gruvbox's pink) in bold. Other panels use the teal section title.
- Dialogs as few as possible on the web: where a link or a page does the job, it replaces the app's dialog (Help is the nav's `Community` link to zite's Community page). Form panels are page-wide with their fields held to a reading width; dialogs size to their content up to the page width.
- Web habits win where they differ from the app: Buy is two direct retailer links (TCGplayer, Card Kingdom) opening in a new tab, never a modal; Home's featured flavor shows the flavor with Image and Buy, no card image by default.
- ASCII logos use `var(--logo)`: the text color in dark mode, the accent only in light.
- Screenshots for the owner are viewport-sized at real window sizes (1440x900, 1920x1080, 420x900), never tall full-page captures, and are sent to the owner as each round lands.

## Round 9, done

Compact footer buttons, the password hint behind `?`, no site footer, the action bar pinned to the bottom, and Privacy policy open signed out. Gated in full and checked live at 1440x900 and 420x900 on every screen: the bar sits flush with the window's bottom edge, the `?` sits inside it at its right end without covering a button, nothing at the end of a page hides behind the bar, and signed-out `/privacy` has its nav pill and the Fan content section. Login's three compact buttons wrap to two rows at the 25rem panel width.

## Shared code

zwipe-ui landed on main (c55c3977), and main is merged into feat/web. The shared plan is `zwipe-ui/context/shared-plan.md`. The web now takes the session rules, version gate, support mailto, account checks, timing constants, changelog cache, filter toggle, examples cap and "1 card" wording from zwipe-ui. The theme already resolves retired slugs through orangui-core.

**Start with `context/shared-switch.md`**: the shared branches already on main that the web still has to switch to, with exact names and files. zwiper-helpers (formerly zwiper-tui) keeps it current.

Still to come, as zwiper-helpers lands each shared branch: toasts take the app's kind, band and words; the `// Belongs in zwipe-ui` helpers move; and the usage buffer replaces the web's disabled funnel once the telemetry branch (branch 12) lands with server-side platform tagging.

## Review queue

Approved: Login, Register and Forgot password (round 10: page-wide form panels with fields at 26rem, content-sized dialogs, Community in place of Help, Esc closing dialogs, Enter submitting the form with or without focus); Home (zite's tagline under the hero); Activity (no Back, the Activity panel spanning the page with its heatmap at size, Most skipped keeping the app's card-list header); Playstyle (paired rows: Summary with Color map, Card roles with Oracle tags, Suggested commanders with Staples; card lists holding their own filters; Clear skips on the bar when wide and in More on phones); Not found (zite's dead end under either nav). Page heads then went onto every screen (title, `?`, one-line description on wide windows; title and `?` centered in the folded nav), and the owner accepted the auth screens showing their title twice on phones.

Next session starts with the deck view. Renders of it as it stood were sent on 2026-11-01, with four proposals the owner hasn't answered yet:
1. Profile becomes the first panel in the left column instead of a lone 40rem strip.
2. The page head shows the deck's name instead of "Deck".
3. On wide windows More breaks up: the TCGplayer and Card Kingdom links go in Budget's footer, Import and Export cards go at the top of the cards column, and Clone, Share, Clear skips and Delete go on the bar beside Edit and Cards (the destructive two in red). Phones keep the app's More sheet.
4. The Warnings panel's `Add cards` button stays on one line.

Then one screen per round with renders: Account, Config, Privacy policy, Changelog, Decks, Create and Edit, Import and Export, Commander maybeboard, Dictionary and Examples, Add, Deck cards, Remove, Update required. Remove each screen's Back button on its turn. When a change touches approved screens, re-render from Login onward for the owner to re-confirm.

## Open decisions for the owner

- Browser back with an overlay open leaves the screen instead of closing the overlay. A proper fix needs orangui to track open overlays; recommended to fix there.
- orangui's dialog host logs a harmless debug-only Dioxus warning when a dialog reads live state; a small orangui fix.
- Buy links on card screens: kept off for app parity unless the owner wants them.
- Launch blockers before zwiper-web goes public: the refresh token moves from `localStorage` to an httpOnly cookie, and funnel events get a platform tag (both zerver changes), then the web's funnel events turn on (`SEND_FUNNEL_EVENTS`).
- Hosting at `zwipe.net/app` or `app.zwipe.net`.
- Parked by the owner (2026-11-01), to come back to after the screens: the glue between zite and the web app, making them feel like one site. The idea on file is to host at `zwipe.net/app`, have the Z mark go to zwipe.net with a `Home` pill for the app's home, give the signed-out web nav zite's links plus `Log in`, point Changelog and Privacy policy at zite's pages, and add a `Log in` pill on zite at launch.
- The ten helpers marked `// Belongs in zwipe-ui` (8 in `src/lib/domain/deck.rs`, 2 in `src/lib/inbound/screens/oracle_tag_dictionary.rs`) wait for the shared-crates session to lift them once zwipe-ui is on main.

## Running it and checking live

`zwiper-web/README.md` covers running it. The scripts in `zwiper-web/tools/` do the live checks the waves used:

- `gate.sh` runs the per-commit gate (`gate.sh dx` adds `dx build`).
- `live/zerver.sh` runs zerver from this checkout on the local dev database with the browser origin allowed and a dummy Resend key. It reads `zerver/.env` (or `ZERVER_ENV`), which is not in git, so the new machine needs its own.
- `live/dx.sh` restarts `dx serve` on :8080.
- `live/seed.py` creates a throwaway user with decks and swipes (needs `DATABASE_URL`); delete it afterwards.
- `live/cdp.py` and `live/run.sh` drive headless Chrome through a steps file and save screenshots.

Stop every server you start and delete every test user you create.
