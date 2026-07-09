# Deck share page — a public URL for any deck

**Status: SHIPPED 2026-07-07 (`985dde3e` server, `d8f7dd4e` zite, `f3b1d17a`
client). All three legs landed the same day: the `share_token` migration +
owner-checked POST/DELETE `/api/deck/{id}/share` + public IP-limited GET
`/api/share/deck/{token}` (identity-stripped, clone stays private); the zite
`/deck/:token` page reusing core GroupCards + in-memory filtering, with
app-parity card rows (tap-to-expand, keyword reveals, mana glyphs), the
deck-list stat chips, and MVP stars; and the app's Share action in a single
More-sheet dialog (create/copy/stop). As-built deltas: the page shows all
deck-list tags (power level, command zone, archetype/other tags) plus a green
price chip, not just format/count; the "Built with Zwipe" CTA was dropped;
Show-lands and Show-command-zone toggles + Min/Max MV steppers were added to the
control panel; debug builds point at localhost and skip the SSG incremental
cache so dynamic routes serve under `dx serve`; a `darkreader-lock` meta was
added (Dark Reader was flattening the themed palette to black).**

**Remaining (tracked in 1.4.0 release prep, NOT blockers):** a server test
asserting a cloned shared deck's `share_token IS NULL` (behavior verified by
hand, no formal test yet); the "Sharing your deck" zite guide section; the
What's New line. OG link-preview cards stay out (SPA limitation, see "Later").

**What this builds, in one sentence:** an owner taps "Share deck" and gets a
`https://zwipe.net/deck/{token}` link that renders the deck as a clean
grouped card list on zite — the same groupings and filters as the app's deck
cards view, readable by anyone, no account needed.

**Why:** highest-leverage growth surface on the board — every shared deck is
a Zwipe ad delivered by a trusted person into an MTG space, and it needs
zero network density (a link works with one user). Social *artifact*, not
social infrastructure.

## Design principles

- **Opt-in, revocable.** Decks stay private until the owner shares. The
  token is an unguessable UUID; "Stop sharing" invalidates the link
  (regenerating produces a new URL). No browsing/discovery surface — a link
  is reachable only by having the link.
- **Reuse, don't rebuild.** Grouping (`GroupCards`/`GroupByOption`) and
  in-memory filtering (`CardQuery`'s client-side path) live in zwipe-core,
  which zite already imports — the share page runs the *same code* as the
  app's deck cards view. Groupings: card type / mana value / color /
  category, commander (and partner/background/signature spell) pinned above
  the groups, exactly like the app.
- **Read-only v1.** No copy-to-account, no comments, no view counts. (Clone-
  from-link is the obvious v2 growth hook: "Open in Zwipe" → app store or
  import. Note it, don't build it.)
- **The look:** Archidekt-style neat grouped columns/list, in zite's
  terminal aesthetic. Crisp, no glow, sentence case, no em dashes.

## The pieces

| Piece | Doc | Ships |
|---|---|---|
| Share token + public endpoint | [`server.md`](server.md) | anytime (server-only) |
| `zwipe.net/deck/{token}` page | [`zite.md`](zite.md) | anytime after server |
| Share/stop-sharing in the app | [`client.md`](client.md) | 1.4.0 batch |

Sequencing note: server + zite can deploy fully before 1.4.0 — the page 404s
gracefully until real tokens exist, and the owner can hand-share early via a
manually-set token for testing/marketing.

## Later (explicitly out of v1)

- MVP stars on the page (once `deck-mvps` ships — the share page is where
  MVPs become a personality statement).
- "Open in Zwipe" / import-this-deck CTA (growth v2).
- OG meta / link-preview card (title = deck name + commander; GitHub Pages
  SPA limitation: previews need prerendering or an edge function — investigate
  separately, do not block v1).
- Optional discovery/browse (needs density; see social-features discussion).
