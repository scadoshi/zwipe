# zwiper-web handoff

Where the web build stands as of 2026-11-01, written so a fresh session on another machine can pick it up from this branch alone. `web-layout.md`, `parity-plan.md` and `ARCHITECTURE.md` hold the detail; this is the map.

## Branches

| Branch | What it is |
|---|---|
| `feat/web` | Every app screen built for the browser in waves A to D, integrated and audited (`audit.md`). Not on main. |
| `feat/web-zite` | This branch: the layout review on top of `feat/web`, one screen at a time with the owner. Not merged into `feat/web` yet. |
| `feat/zwipe-ui` | The shared screen-logic crate, owned by another session. `feat/web` depends on it. |
| `feat/in-app-browser`, `plan/in-app-browser` | The app's in-app browser (Safari View Controller, Custom Tabs), waiting on the owner's device test. Separate from the web work. |

Never push to or merge into main. The owner merges.

## Rules the owner set during review

These are recorded in `web-layout.md`; they are repeated here because they came from conversation, not from the specs.

- The page follows zite's grammar: a centered page capped at 74rem with a small top gap, a hero where a screen has one, and the other panels in two balanced columns.
- The web is panels at every width. Every input form is its own panel with a title bar named after its screen and its buttons in an action footer under a full-bleed rule. The mobile app stays panel-less; don't change it.
- Panel footer buttons are the app's compact buttons in a row, not full-width stacks.
- The top nav is zite's nav bar with the Z mark (in `var(--logo)`) linking Home, a hamburger below 900px, and no screen title header under it.
- No navigation-only buttons (Back, Back to login and the like); the nav and the browser cover them. Buttons that do something stay.
- No site footer in the web app. The Scryfall credit and the Fan Content notice live on zite and in the Privacy policy screen.
- Each screen's action bar is pinned to the bottom of the window, always on screen.
- Password rules live behind the `?` next to the Password label, as in the app.
- Web habits win where they differ from the app: Buy is two direct retailer links (TCGplayer, Card Kingdom) opening in a new tab, never a modal; Home's featured flavor shows the flavor with Image and Buy, no card image by default.
- ASCII logos use `var(--logo)`: the text color in dark mode, the accent only in light.
- Screenshots for the owner are viewport-sized at real window sizes (1440x900, 1920x1080, 420x900), never tall full-page captures, and are sent to the owner as each round lands.

## Round 9, in progress

The last commit is a work-in-progress commit. It has the code for compact footer buttons, the password hint back behind `?`, the site footer removed, the action bar pinned to the bottom, and Privacy policy reachable signed out (a pill in the public nav). It has not been gated in full, checked in a browser, or documented. To finish it:

1. Run `zwiper-web/tools/gate.sh` and fix anything red.
2. Live check at 1440x900 and 420x900: every screen's action bar sits at the window's bottom with the `?` lined up at its right end (the `?` placement assumes a 1.9rem button height and is the likeliest thing to need a fix), nothing at the end of a page hides behind the bar, and signed-out `/privacy` shows the Fan content section.
3. Update `web-layout.md` (shared chrome, Login, Register, Account), the README line about toolbars, and `ARCHITECTURE.md`'s "Adding a screen" step that still mentions the toolbar and `--web-toolbar-h`.
4. Render Login, Register (with the `?` hint open), Forgot password, deck view and Add for the owner, then commit and push.

## Review queue

Approved so far: Login, Register and Forgot password (pending round 9's renders). Next, one screen per round with renders for the owner: Home, then Activity, Playstyle, deck view (these four already have a first pass of the hero-plus-columns layout on this branch), then Account, Config, Privacy policy, Changelog, Decks, Create and Edit, Import and Export, Commander maybeboard, Dictionary and Examples, Add, Deck cards, Remove, Update required, Not found. Remove each screen's Back button on its turn.

## Open decisions for the owner

- Browser back with an overlay open leaves the screen instead of closing the overlay. A proper fix needs orangui to track open overlays; recommended to fix there.
- orangui's dialog host logs a harmless debug-only Dioxus warning when a dialog reads live state; a small orangui fix.
- Buy links on card screens: kept off for app parity unless the owner wants them.
- Launch blockers before zwiper-web goes public: the refresh token moves from `localStorage` to an httpOnly cookie, and funnel events get a platform tag (both zerver changes), then the web's funnel events turn on (`SEND_FUNNEL_EVENTS`).
- Hosting at `zwipe.net/app` or `app.zwipe.net`.
- Five helpers marked `// Belongs in zwipe-ui` wait for the zwipe-ui session to lift them.

## Running it and checking live

`zwiper-web/README.md` covers running it. The scripts in `zwiper-web/tools/` do the live checks the waves used:

- `gate.sh` runs the per-commit gate (`gate.sh dx` adds `dx build`).
- `live/zerver.sh` runs zerver from this checkout on the local dev database with the browser origin allowed and a dummy Resend key. It reads `zerver/.env` (or `ZERVER_ENV`), which is not in git, so the new machine needs its own.
- `live/dx.sh` restarts `dx serve` on :8080.
- `live/seed.py` creates a throwaway user with decks and swipes (needs `DATABASE_URL`); delete it afterwards.
- `live/cdp.py` and `live/run.sh` drive headless Chrome through a steps file and save screenshots.

Stop every server you start and delete every test user you create.
