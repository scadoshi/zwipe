# Deck share page — server changes

## 1. Migration — `zerver/migrations/<ts>_add_deck_share_token.sql`

```sql
-- Public share link: NULL = private (default). The token is the capability:
-- unguessable UUID, revoked by nulling, rotated by regenerating.
ALTER TABLE decks ADD COLUMN share_token UUID;
CREATE UNIQUE INDEX idx_decks_share_token ON decks (share_token)
    WHERE share_token IS NOT NULL;
```

## 2. Contracts — `zwipe-core/src/http/contracts/deck.rs`

- Deck profile responses gain `share_token: Option<Uuid>`
  (`#[serde(default)]`) so the app can render share state + build the URL.
- New response `HttpSharedDeck` for the public endpoint: deck name, format,
  commander/partner/background/signature-spell cards, the card list (same
  card payload the deck GET uses), land/price targets optional. **No
  user identity** — no username, no user_id, no email. The page shows a deck,
  not an account.
- Paths: `share_deck_route(deck_id)` (private) and
  `get_shared_deck_route(token)` (public) in `http/paths.rs`.

## 3. Private endpoints — share management

`POST /api/deck/{deck_id}/share` → generates (or regenerates) `share_token`,
returns it. `DELETE /api/deck/{deck_id}/share` → nulls it. Owner-checked like
every deck mutation. Handler:
`zerver/src/lib/inbound/http/handlers/deck/share_deck.rs` (two fns, mirrors
`skip_deck_card.rs`'s post/delete pairing). Deck service + repository methods
(`set_share_token`, `clear_share_token`) + erased twins, following the
existing port pattern.

## 4. Public endpoint — the share read

`GET /api/share/deck/{token}` in **public_routes** (no auth):

- Resolves `decks WHERE share_token = $1`; 404 when absent (revoked links
  die).
- Response: `HttpSharedDeck` (deck + full card data — reuse the existing
  get_deck assembly path in the deck service, minus ownership check, plus
  identity stripping).
- Rate limit: its own IP-keyed governor next to the other public configs in
  `routes.rs` (30 req / 2s per IP like the marketing config — pages fetch
  once).
- Cacheability: responses may be CF-cached briefly (~5 min) — a shared deck
  updating a few minutes late is fine; a revoked token dying a few minutes
  late is acceptable (note in handler comment).

## 5. `.sqlx`

New queries → `cargo sqlx prepare --workspace` from the **workspace root**,
commit `.sqlx/`. The deploy verify step covers drift.

## Clone rule (owner call 2026-07-06)

**Cloned decks start private.** `clone_deck`
(`outbound/sqlx/deck/mod.rs` ~633) copies profile fields from the source
deck row — `share_token` must be explicitly excluded from that copy (leave
NULL) or clones would silently inherit a live public link. Add a test: clone
a shared deck, assert the clone's `share_token IS NULL`.

## Compatibility

All additive: old clients never send/read `share_token` and are unaffected.
Server + these endpoints can deploy before any client knows about them.
