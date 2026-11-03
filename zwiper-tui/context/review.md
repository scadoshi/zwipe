# zwiper-tui visual review

zwiper-tui covers every app screen and has passed the parity audit, live checks, the narrow-terminal pass and a style review. What remains is a visual review with the owner, one screen at a time, from fresh renders. This file is the state of that review and everything needed to pick it up on another machine.

## Where things stand

On hold since 2026-11-03: the main app's visual grammar is changing, and those changes will guide the TUI's. Resume by reading what changed in the app, then carry it into the decided rules below before reviewing more screens.

- Branch `feat/tui`, 647 tests, every gate green, pushed.
- Every screen drives its zwipe-ui screen model (switched 2026-11-03; the recipe is in `ARCHITECTURE.md`).
- Reviewed with the owner: Login, Register, Reset password, Home. Every owner-approved item is built.
- Next screen when it resumes: Activity, then Playstyle, Decks, the deck view and its sheets, Add, Remove, Deck cards, Account, Config, Privacy, Changelog.
- Open outside the TUI: Scryfall's `*italic*` markers in flavor text show as literal asterisks in the TUI and likely the app; a shared zwipe-ui helper would fix every client.

## Decided (owner-approved; keep these)

Chrome:
- No title row on any screen. The `?` hint sits in the key bar as `? hint` (or `H hint` where `?` opens a section's About), just before `F1 help`. F1 opens Help everywhere, and every bar names it.
- The key bar is centered on every screen, each row of a wrapped bar too (the deck view's two bars included). `action_bar` does it in `with_help` and `wrap`, so no screen aligns its own bar.
- The grid backdrop draws behind every screen. Tiles are 4 cells by 2 rows with no gaps, shaded with orangui-core's `SHADES`, and faded down the screen by orangui's grid mask (`.grid-cells-canvas` in orangui's `assets/components.css`: full at the top, 0.45 at 45% of the height, gone at the bottom). Every box, on the grid or over it (panels, fields, text areas, counter tiles, card rows, dialogs, sheets, toasts, popups, menus, Help), has square corners and is solid to its outer edge, border cells included. A terminal cell holds one background, so a rounded corner can never be filled cleanly; square corners make every box one solid rectangle. Loose text, buttons and the logo have no background, so the tiles show up to them.
- The splash keeps its rain-in over the grid.
- Every title on a box's top border (panels such as Color map and Featured flavor, sheet titles, charts, Decklist, the deck view's panels, a loading ghost's title) takes the text field label's style: `palette.tertiary`, not bold. They all draw through `panel::title`, so no title can drift from the labels. The deck view's shown tab is a bold chip only while its panel has the focus; the tabs not shown stay muted.

Logo:
- The ZWIPE mark draws its `▉` blocks as full `█` so letters are solid. The thin `▎▎` shadow strokes are part of the logo's design: never remove them.
- Logos use `palette.logo` (text color on dark themes, accent on light).

Forms and keys:
- No placeholder text in any box. Import's example decklist lives in its `?` hint.
- Labels sit centered on the box border in `palette.tertiary`.
- A field's error sits centered under its box, in the error color.
- Esc ends typing in a field and never quits. Only `q` quits, and only when no field takes letters. Ctrl+C always quits.
- With no field active, letters are commands. Login: `c Create profile`, `f Forgot password`. Register and Reset password: Esc with no field active goes back to Login.
- Navigation to another screen lives in the key bar, not in buttons on the screen. The form's own submit stays on the screen, right under its fields.
- The owner dislikes F-key shortcuts like `F2 Save`. Revisit the deck form's `F2 Save` when it comes up in review, likely as Esc then a letter.
- Every button looks the same; focus is the only highlight. A button is its label in brackets on one line, `[ Log in ]`, at every size, in rows, inside lines of text and in dialogs. At rest the brackets take the muted text color, brighter than the border, and the label the text color (error for danger, muted when disabled). Focused, brackets and label are bold in the accent (a danger button's own color), with no fill. There is no boxed button, no filled pill and no accent `primary` style; Account's Save changes is plain like the rest.
- A filled block is always a tag (Everybody, Live, deck tags, chips) and never focusable.
- Every About `?` marker on screen is a bracketed `[?]` button (no inner spaces): brackets and `?` in the muted text color at rest at every site, bold accent brackets and `?` while it is the marker the About key opens. It sits one space after its label. Key bar hints like `? hint` stay bare keys.
- Playstyle's picked Oracle tags column shows its label in bold accent beside its lit `[?]`, never a reversed (filled) label.
- The deck form's fields each carry a `[?]` About marker after their label. On a field that takes no typing, `i` opens its explainer and its marker lights; the text boxes (Deck name, Price, the command-zone fields) keep theirs at rest, since letters type there. The form has no F6 key; the pickers keep F6 for their own hint.
- The deck view's sections carry their explainer's `[?]` on the shown tab of each panel, the chip's padding the space before it; it lights while that panel has the focus, since `i` opens it then. Profile has none.
- Account, Config, Privacy and Changelog have no Back button; `Esc back` in the key bar does it. Account keeps Log out and More.
- Account's Change sheets (username, email, password) have no Back button either: Save changes is their one button, and `Esc back` in the key bar goes back.
- Register sits a row down from the top when there is room, so its logo never touches the screen's top edge.

Home:
- Home's logo always has a free row over it, so it never touches the screen's top edge. The flavor text always shows whole; within that, the first that fits wins: the full logo over a full-scale map, the compact Z over a full-scale map, the full logo over a half-scale map, the Z over a half-scale map, else no logo, and the figures start on the first row.
- Home stacks its panels at every size: the Color map on top, the Featured flavor right under it. Each is only as tall as what it holds, and the flavor panel grows to hold the featured card's image where the terminal draws one. A screen too short for both at full height keeps the small-terminal fallbacks: the map draws at half scale so the image keeps its room, then the image gives way to the text alone, and the map crops around its middle only when even half scale does not fit.
- The Featured flavor quote centers on its words at every size, like the card line under it: each line is trimmed, so no stray space shifts it.

Earlier rules that still hold: swipe labels are only add, remove, skip, undo and maybeboard; toasts and hint prose use the app's text verbatim; the deck view uses responsive panels; telemetry stays; the color map matches the app.

## Waiting on the owner

Paused on 2026-11-01 to save agent usage.

Approved by the owner, not built yet: none.

Still open:

- The deck form's text boxes (Deck name, Price, Commander, Partner, Background, Signature spell) show a `[?]` marker but no key opens it, since letters type there and F6 is gone. `i` opens the explainer on fields that take no typing. Pick a key for the typing fields, or accept it.
- At Add's end of results the TUI shows "u steps back into the pile" under the empty card; the app shows nothing there. Keep or drop?
- The deck form's own swipe picker (`deck_form/swipe_pick.rs`, the command-zone slots) still runs its own pile; it could move onto `swipe_select`.

- Two kinds of hint text stayed after placeholders went: the filter sheet's `[Name contains]` row labels and the deck form's `Not set` readouts. Keep or drop?
- With the title row gone, some screens no longer say where you are: the deck form and its pickers (they look alike), Changelog, Add and Remove cards, Deck cards, Oracle tags and Examples, Activity, Playstyle, Commander maybeboard, the commander swipe, Create profile and Reset password. Decide per screen during the review.
- Move the grid fade (the two mask values) into orangui-core's `grid_rain` so the TUI reads it from the same place as the shades, with an orangui test that fails if the CSS mask and the constants disagree. That belongs on orangui's `feat/orangui-core` branch.
- At 200x50, stretch content to fill or center it at a readable width.
- The deck view's warning fix buttons are still filled pills; convert them to brackets when the deck view comes up.
- The privacy text says "a deck builder for mobile" (shared legal copy in zwipe-core).

## Merged prerequisites

All four branches the TUI waited on are on main as of 2026-11-03: the `terminal` client platform, the app binary renamed to `zwiper` (the TUI installs as `zwipe`), zwipe-ui, and orangui-core. `feat/tui` has main merged in and is switching each screen to its zwipe-ui screen model (the recipe is in `ARCHITECTURE.md`). Never merge or push to main from a work session; the owner merges.

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
