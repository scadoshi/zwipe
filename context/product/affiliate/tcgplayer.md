# TCGplayer affiliate (via Impact)

**Status: application submitted 2026-06-23 — In Review** on Impact (impact.com).
TCGplayer runs its affiliate program on Impact Radius; they notify on a response.

## Why

Zwipe already builds TCGplayer + Card Kingdom buy URLs (`zwiper`'s
`outbound/buy_links.rs` — deck mass-entry, surfaced in the "Buy deck" sheet). The
affiliate program lets us **earn commission on purchases that already flow through
those links, with no UI change.** Fits the free / no-ads / no-IAP positioning:
revenue is purchase-driven and invisible to the user. On-strategy with
[`../premium/price_intelligence.md`](../premium/price_intelligence.md) (the
"drop alert → tap → buy through our link" idea) and
[`../monetization.md`](../monetization.md).

## Signup copy (saved for reference)

**Business category:** use **Content/Reviews** (deck-building = informed purchasing
decisions), *not* "Search/Comparison" (we don't compare prices across vendors).

**Description** (≤1000 chars) — plain text, copy the three paragraphs:

Zwipe is a mobile-first Magic: The Gathering deck builder with a swipe-based interface — swipe right to add a card, left to skip. It turns the slow, cluttered desktop deck-building experience into something that fits in one thumb. Built for the 100-card Commander format, players browse 110,000+ cards, build and manage multiple decks, see mana-curve and price stats, and import/export decklists — all synced across devices.

Our audience is high-intent Magic players actively assembling decks who then need to buy the cards they've added. Zwipe surfaces one-tap buy links straight from a finished deck, so the path from "I want this card" to checkout is immediate. The app is free with no ads — our revenue is purchase-driven, which aligns our incentives directly with sending brands ready-to-buy customers.

Live on the iOS App Store, in closed testing on Google Play, and on the web at zwipe.net.

**Content & interests** (comma-separated, no spaces):

```
magic,magicthegathering,mtg,commander,edh,deckbuilding,deckbuilder,decklists,tradingcardgames,tcg,cardgames,cardcollecting,tabletopgaming,hobbygames
```

> Note: name "Magic: The Gathering" freely here — this is a private business
> profile to a brand, not public store metadata, so the App Store copycat scrub
> (which renamed the listing to "Zwipe TCG") does **not** apply.

## Integration (once approved)

1. Impact issues a tracking-link format — a `tcgplayer.pxf.io/…` deep link, or an
   appended tracking param carrying your publisher/campaign ID.
2. Wrap/append it inside `tcgplayer_url()` in
   `zwiper/src/lib/outbound/buy_links.rs` → every existing "Buy deck → TCGplayer"
   tap earns commission. **Zero UI change.**
3. **Per-card buy links** (new feature): a "Buy ↗" on individual cards in search /
   the deck list, not just the whole-deck mass entry — impulse-buy the card you
   just swiped right on. Highest-leverage add once tracking is live.

## Gotchas

- **Card Kingdom has no public self-serve affiliate sign-up** (unlike TCGplayer).
  It's a **partnership/invite program** — you email CK directly to get set up; they
  issue an affiliate code and/or a `partner` URL param (e.g. their links already
  support `?partner=archidekt` — Archidekt, a deck-builder in our exact space, has
  a CK partnership, which is a strong precedent to cite in the pitch). Until then
  `cardkingdom_url()` stays untracked (still a useful convenience link).
- **FTC disclosure** required — a "we may earn a commission from purchases" line
  (privacy/about page, or near the buy links).
- **Store policies** — App Store and Play both allow affiliate links; keep them
  functional and non-deceptive.

## Keys / IDs

Once issued, store the Impact **publisher/campaign ID** and any secret in the
password manager — **never commit real keys** (use env vars + placeholders if a
tracking ID ever needs to live in code). Campaign seen during signup:
`TCGplayer.brand`, campaignId `24933` (from the signup URL — not a secret).
