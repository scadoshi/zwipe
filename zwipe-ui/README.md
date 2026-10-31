# zwipe-ui

The screen logic Zwipe's three clients share: zwiper (the iOS and Android app), zwiper-web and zwiper-tui. Each client draws its screens its own way; what a screen shows and does is decided here once, as pure functions and plain types with tests.

It names no UI crate, so it builds for iOS, Android, the terminal and `wasm32-unknown-unknown`. Its dependencies are zwipe-core, orangui-core, chrono, uuid and email_address.

## What goes in

Anything that decides what a screen shows or does and that two or more clients need, in the app's behavior. Where a port drifted from the app, the app wins and a test pins it.

- The color map's tiles, shades and rings (`color_map`), with the golden test that checks them against the app.
- Activity's heatmap, totals, rate charts and `year_line`; Playstyle's identities, reasons, profile and staples filters; home's figures, scope and flavor card expiry.
- The deck list's Group by and Show, the deck form's rules, export text, the import toast, warning fixes and buy links.
- The card filter: sections, fields, pickers, the Apply decision, Add's deck context and the draft the sheet edits (`filter::draft`).
- The stores the card screens share: per-deck filters, parked add stacks, the undo log, adds in flight and the catalog with its 24 hour freshness.
- The swipe stack and its actions, Add's search rules, the deck cards view's pass and quantity bursts, and the land and budget crossings.
- The changelog's filing and filter, a card's faces as text, the account forms' wording, the session freshness rule, the version gate and the problem report link.
- Hint copy as typed spans: every `?` topic and every one-time hint (`hint`).
- Toasts as values (`toast`), behavior timings and search limits (`timing`), and the generation ticket for replies that arrive out of order.

## What stays out

- Anything that draws, or names Dioxus, ratatui or web-sys. Focus, scroll, gestures, animation and layout belong to each client.
- Domain rules the server shares. Those go in zwipe-core, which keeps its purity rules.
- Networking. Clients fetch through zwipe-client and hand the results in; zwipe-ui does not depend on it.
- Anything only one client needs: the TUI's palettes, privacy policy parser and mana symbol tokens; the app's gesture engine, app icon, haptics, keyring and telemetry buffer; the web's anchors and `localStorage`.
- orangui's pure pieces (number formatting, chart math, toast bands, count-up and decode frames). Those come from orangui-core.

## orangui-core

orangui-core is pinned to rev `932abb5` on orangui's `feat/orangui-core` branch. When the owner merges that branch, the rev moves to orangui main's merge commit.

## Not yet

Screen models (a state, actions and effects per screen) come next, starting with zwiper-web's waves B to D. The clients still use their own copies of everything here until each one switches over.
