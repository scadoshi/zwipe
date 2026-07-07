# Deck share page — client changes (zwiper, 1.4.0 batch)

## 1. API client — `zwiper/src/lib/outbound/client/deck/share_deck.rs`

New module: `share_deck(deck_id)` (POST, returns the token) and
`unshare_deck(deck_id)` (DELETE), mirroring `skip_deck_card.rs`'s paired
trait pattern. Deck profile responses now carry `share_token` for rendering
state.

## 2. Share action — deck view More sheet

`zwiper/src/lib/inbound/screens/deck/components/more_buttons.rs` (where
Clone / Export / Clear skips already live — sharing is the same rarity
class):

- Not shared: **"Share deck"** → POST → copy
  `https://zwipe.net/deck/{token}` to the clipboard (however export copies
  text today; else the OS share sheet via the `open_url`/share plumbing) →
  toast "Share link copied".
- Shared: row shows **"Copy share link"** and **"Stop sharing"**. Stop →
  DELETE → toast "Link disabled". Re-sharing later generates a fresh URL
  (communicate: "Sharing again creates a new link").
- `WEB_DOMAIN` const already exists in `profile/mod.rs` — hoist/share it
  rather than hardcoding the URL twice.

## 3. Visual state

A small "shared" indicator on the deck view header (subtle chip or icon)
so owners can see at a glance which decks have live links. Crisp, no glow.

## 4. Hint + copy

One-time hint not needed (the More sheet is discoverable); store What's New
line at release: "Share any deck as a link anyone can open." Sentence case,
no em dashes. zite guide section ships in the same release (see
[`zite.md`](zite.md) §4).

## 5. Testing checklist (device)

- Share → link opens the zite page with the deck grouped correctly
- Edit deck → page reflects changes (within the CF cache window)
- Stop sharing → link 404s with the friendly message
- Re-share → new URL works, old URL stays dead
- Clone/export unaffected; share state does not clone (settled 2026-07-06:
  cloned decks start private — server excludes `share_token` from the clone
  copy, covered by a server test)
