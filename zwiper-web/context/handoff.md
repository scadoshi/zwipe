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

The shared crates (zwipe-ui, zwipe-core, zwipe-client, components, and orangui with orangui-core) belong to one session, zwiper-tui on scotland-server. Don't edit them on these branches. When zwiper-web needs something shared, send that session a request naming what it needs and where it lives here (file and function). It builds the change on a branch off main, the owner merges it, and then main gets merged into feat/web and feat/web-zite. The ten `// Belongs in zwipe-ui` helpers are already requested; it reads them from origin/feat/web.

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

## Round 9, done

Compact footer buttons, the password hint behind `?`, no site footer, the action bar pinned to the bottom, and Privacy policy open signed out. Gated in full and checked live at 1440x900 and 420x900 on every screen: the bar sits flush with the window's bottom edge, the `?` sits inside it at its right end without covering a button, nothing at the end of a page hides behind the bar, and signed-out `/privacy` has its nav pill and the Fan content section. Login's three compact buttons wrap to two rows at the 25rem panel width.

## When zwipe-ui lands on main

The shared plan is `zwipe-ui/context/shared-plan.md` on feat/zwipe-ui. The zwiper-tui session messages when zwipe-ui merges. Then:

- Merge main into feat/web, then into feat/web-zite, and run the gate.
- Switch the web's copies to zwipe-ui where it already has them: `domain/session.rs`, `version_gate.rs`, `support.rs`, `validation.rs`, the timing constants, the changelog cache, and `toggled`.
- Theme: resolve retired slugs through orangui-core `theme::current_theme` instead of `ThemeConfig::resolve`.
- Deck rows read "1 card" for one card (`ui::deck::cards_label`), by the owner's decision.
- Toasts take the app's kind, band and words as each helper branch lands them.
- The usage buffer replaces the web's disabled funnel once the telemetry branch (branch 12) lands. That branch also tags usage with the platform server side.

## Review queue

Approved so far: Login, Register and Forgot password (round 9's renders sent for sign-off). Next, one screen per round with renders for the owner: Home, then Activity, Playstyle, deck view (these four already have a first pass of the hero-plus-columns layout on this branch), then Account, Config, Privacy policy, Changelog, Decks, Create and Edit, Import and Export, Commander maybeboard, Dictionary and Examples, Add, Deck cards, Remove, Update required, Not found. Remove each screen's Back button on its turn.

## Open decisions for the owner

- Browser back with an overlay open leaves the screen instead of closing the overlay. A proper fix needs orangui to track open overlays; recommended to fix there.
- orangui's dialog host logs a harmless debug-only Dioxus warning when a dialog reads live state; a small orangui fix.
- Buy links on card screens: kept off for app parity unless the owner wants them.
- Launch blockers before zwiper-web goes public: the refresh token moves from `localStorage` to an httpOnly cookie, and funnel events get a platform tag (both zerver changes), then the web's funnel events turn on (`SEND_FUNNEL_EVENTS`).
- Hosting at `zwipe.net/app` or `app.zwipe.net`.
- The ten helpers marked `// Belongs in zwipe-ui` (8 in `src/lib/domain/deck.rs`, 2 in `src/lib/inbound/screens/oracle_tag_dictionary.rs`) wait for the shared-crates session to lift them once zwipe-ui is on main.

## Running it and checking live

`zwiper-web/README.md` covers running it. The scripts in `zwiper-web/tools/` do the live checks the waves used:

- `gate.sh` runs the per-commit gate (`gate.sh dx` adds `dx build`).
- `live/zerver.sh` runs zerver from this checkout on the local dev database with the browser origin allowed and a dummy Resend key. It reads `zerver/.env` (or `ZERVER_ENV`), which is not in git, so the new machine needs its own.
- `live/dx.sh` restarts `dx serve` on :8080.
- `live/seed.py` creates a throwaway user with decks and swipes (needs `DATABASE_URL`); delete it afterwards.
- `live/cdp.py` and `live/run.sh` drive headless Chrome through a steps file and save screenshots.

Stop every server you start and delete every test user you create.
