# zwipe-ui

The screen logic Zwipe's three clients share: zwiper (the iOS and Android app), zwiper-web and zwiper-tui. Each client draws its screens its own way; what a screen shows and does is decided here once, as pure functions and plain types with tests.

It names no UI crate, so it builds for iOS, Android, the terminal and `wasm32-unknown-unknown`. Its dependencies are zwipe-core, orangui-core, chrono, uuid and email_address.

## What goes in

Anything that decides what a screen shows or does and that two or more clients need, in the app's behavior. Where a port drifted from the app, the app wins and a test pins it.

- The color map's tiles, shades and rings (`color_map`), with the golden test that checks them against the app.
- Activity's heatmap, totals, rate charts and `year_line`; Playstyle's identities, reasons, profile and staples filters; home's figures, scope and flavor card expiry.
- The deck list's Group by and Show, the create gate (`deck::create_gate`), the deck form's values and rules (Create's request, Edit's sparse update, the knock-on clears, which slots show) and its pickers (`deck::pickers`), the command zone's labels, group headers, row art and slot update (`deck::command_zone`), the deck view's sections, total price, share link, clone name and toasts (`deck::view`), export text, the import toast, warning fixes and buy links.
- The commander maybeboard's pip filter, Quick add gate, highlight and copy, and swipe select's key and re-deal (`commander`).
- The card filter: sections, fields, pickers, chip rows (`filter::chips`), the Apply decision, Add's deck context and the draft the sheet edits (`filter::draft`).
- The stores the card screens share: per-deck filters, parked add stacks, the undo log, adds in flight and the catalog with its 24 hour freshness.
- The swipe stack and its actions, Add's search rules, the deck cards view's pass and quantity bursts, and the land and budget crossings.
- The changelog's filing and filter, a card's faces as text and info lines (`card::info`), the session freshness rule, the version gate and the problem report link.
- The oracle tag dictionary's buckets, letter rail and rows (`dictionary`).
- Config's preference changes, sparse update, toggle settle, exception chips and toasts (`preferences`).
- The account forms: every field check and its words, the Log in gate, the Resend and Delete account labels; Home's greeting.
- Hint copy as typed spans: every `?` topic and every one-time hint (`hint`).
- Toasts as values (`toast`), behavior timings and search limits (`timing`), and the generation ticket for replies that arrive out of order.
- Screen models (`screens`): a screen's state, actions and effects, so a client only draws and wires input. The deck list (`screens::deck_list`) is the first; `context/screen-models.md` holds the pattern.

## What stays out

- Anything that draws, or names Dioxus, ratatui or web-sys. Focus, scroll, gestures, animation and layout belong to each client.
- Domain rules the server shares. Those go in zwipe-core, which keeps its purity rules.
- Networking. Clients fetch through zwipe-client and hand the results in; zwipe-ui does not depend on it.
- Anything only one client needs: the TUI's palettes, privacy policy parser and mana symbol tokens; the app's gesture engine, app icon, haptics, keyring and telemetry buffer; the web's anchors and `localStorage`.
- orangui's pure pieces (number formatting, chart math, toast bands, count-up and decode frames). Those come from orangui-core.

## orangui-core

orangui-core comes from orangui main, the same source components uses for orangui, so Cargo.lock holds one copy of it. `cargo update -p orangui` moves both.

## Who edits the shared crates

One session owns the shared crates: zwipe-ui, zwipe-core, zwipe-client and components, plus orangui and orangui-core. Feature branches never edit them. A shared change goes on a small branch off main, the owner merges it, and feature branches then merge main. zwiper-web and zwiper-tui send requests to the owning session instead of changing shared code themselves.

`context/shared-plan.md` lists the shared branches in order, the drift still to settle and the owner's decisions. `context/inventory.md` records where each piece landed.

## Not yet

Screen models (a state, actions and effects per screen) are under way: the deck list is built and the app draws from it; the other fifteen follow in the shared plan's order. zwiper-tui has switched to everything here; zwiper-web still uses its own copies until it switches over, and the app has switched `filter::fields::same_set`, the account checks and labels, the Log in gate, Home's greeting, the command zone (`deck::command_zone`), the deck form's values, requests, knock-on clears and slot visibility (`deck::form`), the deck form's pickers (`deck::pickers`), the create gate, the deck view (`deck::view`), the commander maybeboard and swipe select rules (`commander`), the oracle tag dictionary (`dictionary`), Config (`preferences`), the filter's chip rows without a mode chip (`filter::chips`) and the card info lines (`card::info`) so far.
