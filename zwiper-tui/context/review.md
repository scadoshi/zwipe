# zwiper-tui visual review

zwiper-tui covers every app screen and has passed the parity audit, live checks, the narrow-terminal pass and a style review. What remains is a visual review with the owner, one screen at a time, from fresh renders. This file is the state of that review and everything needed to pick it up on another machine.

## Where things stand

- Branch `feat/tui`, 604 tests, every gate green.
- Login is reviewed. Its last open question is the button grammar below.
- Next screen: Register, then the rest in screen order (Reset password, Home, Activity, Playstyle, Decks, the deck view and its sheets, Add, Remove, Deck cards, Account, Config, Privacy, Changelog).

## Decided (owner-approved; keep these)

Chrome:
- No title row on any screen. The `?` hint sits in the key bar as `? hint` (or `H hint` where `?` opens a section's About), just before `F1 help`. F1 opens Help everywhere, and every bar names it.
- The grid backdrop draws behind every screen. Tiles are 4 cells by 2 rows with no gaps, shaded with orangui-core's `SHADES`, and faded down the screen by orangui's grid mask (`.grid-cells-canvas` in orangui's `assets/components.css`: full at the top, 0.45 at 45% of the height, gone at the bottom). Boxes (panels, fields, buttons, sheets, dialogs, toasts) are solid on it; loose text and the logo have no background, so the tiles show up to them.
- The splash keeps its rain-in over the grid.

Logo:
- The ZWIPE mark draws its `▉` blocks as full `█` so letters are solid. The thin `▎▎` shadow strokes are part of the logo's design: never remove them.
- Logos use `palette.logo` (text color on dark themes, accent on light).

Forms and keys:
- No placeholder text in any box. Import's example decklist lives in its `?` hint.
- Labels sit centered on the box border in `palette.tertiary`.
- Esc ends typing in a field and never quits. Only `q` quits, and only when no field takes letters. Ctrl+C always quits.
- With no field active, letters are commands. Login: `c Create profile`, `f Forgot password`. Register and Reset password: Esc with no field active goes back to Login.
- Navigation to another screen lives in the key bar, not in buttons on the screen. The form's own submit stays on the screen, right under its fields.
- The owner dislikes F-key shortcuts like `F2 Save`. Revisit the deck form's `F2 Save` when it comes up in review, likely as Esc then a letter.
- Every button looks the same; focus is the only highlight. A focused button shows the accent on its edges and its bold label, and its inside stays unfilled.
- Tags that aren't clickable (Everybody, Live, deck tags) are filled blocks.

Earlier rules that still hold: swipe labels are only add, remove, skip, undo and maybeboard; toasts and hint prose use the app's text verbatim; the deck view uses responsive panels; telemetry stays; the color map matches the app.

## Waiting on the owner

The button grammar. The proposal on the table, which the owner leans toward:
- A button is bracketed on one line, `[ Log in ]`, at every size, in rows, button rows and dialogs alike. No more boxed buttons and no more filled pills. Focus is bold accent brackets and label.
- A filled block is always a tag and never focusable.
- Drop the Back buttons on Account, Config, Privacy and Changelog, since `Esc back` in the bar does the same.
- Convert the sign-in screens first for review, then the rest.

Why: today a button is boxed where it fits and a filled pill where it doesn't, and a filled pill reads as a tag. A survey of every button screen at 200x50, 120x40, 100x30, 80x24 and 60x20 found:
- Always boxed: Account's and Config's bottom rows.
- Always a pill: Privacy's and Changelog's Back.
- Inline pills in rows: Account's and Config's row actions.
- Switch with size: Register (pill at 60x20), Reset password (pill at 80x24 and below) and dialogs (boxed, pills, or stacked pills).

The keys-only alternative (every action a key, tags as colored text) was considered and set aside: some screens have too many actions, and it scatters letters everywhere.

Smaller open questions:
- Account's Save changes is the last button with the accent `primary` style. Make it plain like the rest?
- Two kinds of hint text stayed after placeholders went: the filter sheet's `[Name contains]` row labels and the deck form's `Not set` readouts. Keep or drop?
- With the title row gone, some screens no longer say where you are: the deck form and its pickers (they look alike), Changelog, Add and Remove cards, Deck cards, Oracle tags and Examples, Activity, Playstyle, Commander maybeboard, the commander swipe, Create profile and Reset password. Decide per screen during the review.
- Move the grid fade (the two mask values) into orangui-core's `grid_rain` so the TUI reads it from the same place as the shades, with an orangui test that fails if the CSS mask and the constants disagree. That belongs on orangui's `feat/orangui-core` branch.
- At 200x50, stretch content to fill or center it at a readable width.
- The privacy text says "a deck builder for mobile" (shared legal copy in zwipe-core).

Known bug to fix with Register: at 120x40 Register's form is anchored to the top and the full logo does not fit, so its top rows are clipped. Fall back to the compact Z, or drop the logo, rather than clip.

## Branches waiting on owner merges

Never merge or push to main from a work session; the owner merges.

1. `chore/terminal-platform`: the server accepts the `terminal` client platform. Until it deploys, production rejects the TUI's sign-in. A Desktop fallback was offered as the alternative.
2. `chore/zwiper-bin-name`: renames the app's binary to `zwiper` (the iOS and Android display names stay "Zwipe"). After it merges, merge main into `feat/tui` and set the TUI's `[[bin]] name = "zwipe"`.
3. `feat/zwipe-ui`: the shared logic crate zwiper, zwiper-web and zwiper-tui draw from.
4. orangui `feat/orangui-core`: the Dioxus-free half of orangui that zwiper-tui pins by rev. Merge it, then bump the pins.

## How the work runs

Gate for every change, all on `-p zwiper-tui` unless noted:
- `cargo +nightly fmt`
- `cargo clippy -p zwiper-tui --all-targets -- -D warnings`
- `cargo test -p zwiper-tui`
- `RUSTDOCFLAGS="-D warnings" cargo doc -p zwiper-tui --no-deps --document-private-items`
- before a push that touches more than the TUI: `cargo clippy --workspace --all-targets -- -D warnings`

Environment: `BACKEND_URL=https://example.invalid RUST_LOG=info RUST_BACKTRACE=0` for builds and tests. Each agent gets its own `CARGO_TARGET_DIR`; a shared one gave flaky test counts.

Renders for review: run the TUI in a tmux session on its own socket (`tmux -L <name>`) at a fixed size, with `COLORTERM=truecolor` and `ZWIPER_TUI_IMAGES=off`, then `tmux capture-pane -e -p -J` and render the ANSI to PNG. The ANSI-to-PNG renderer and capture scripts were scratch tools on the old machine and are not in the repo; a small Rust program that paints each cell's background and glyph from the 24-bit escapes is enough to rebuild it. Recordings are frames captured about every 40 ms and joined with ffmpeg; MP4 reaches the owner where GIF did not. Point `XDG_CONFIG_HOME` at a scratch folder so captures never touch a real session, and wait about 10 seconds after launch, since the splash swallows the first key.

Live checks use the owner's test account (credentials are never committed), may only read or write scratch decks, and never change account settings.

Commits are one line, with no AI attribution, American English and no em dashes. Markdown is not hard-wrapped.
