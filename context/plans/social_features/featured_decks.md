# Featured decks — owner-curated showcase with MVPs

**Depends on:** [`deck_share_page`](../deck_share_page/overview.md) (share
tokens + the zite deck page these link to) and, for the full effect,
[`deck_mvps`](../deck_mvps/overview.md) (the stars shown on each tile).
Shippable with placeholder-less tiles before MVPs exist; better after.

## 1. Migration — `zerver/migrations/<ts>_add_deck_featured_at.sql`

```sql
-- Owner-curated showcase flag. Only decks with a live share_token are
-- served as featured (enforced in the query, not a constraint, so
-- unsharing simply drops a deck from the showcase without data loss).
ALTER TABLE decks ADD COLUMN featured_at TIMESTAMPTZ;
CREATE INDEX idx_decks_featured ON decks (featured_at DESC)
    WHERE featured_at IS NOT NULL;
```

## 2. Curation — zcript, not endpoint

No admin API v1. `zcripts/featured/feature-deck.sql` (+ a companion
unfeature): parameterized `UPDATE decks SET featured_at = now() WHERE id =
:'deck_id' AND share_token IS NOT NULL RETURNING name;` — the
`share_token IS NOT NULL` guard means you cannot feature an unshared deck
by accident. **Process rule: ask the deck's builder on Discord before
featuring** — sharing consents to link-reachability, featuring puts them on
the homepage; that's a bigger ask and it doubles as a delight touchpoint
("we want to feature your deck").

## 3. Public endpoint

`GET /api/share/decks/featured` in public_routes (same governor class as
the other public marketing configs):

- Query: `WHERE featured_at IS NOT NULL AND share_token IS NOT NULL
  ORDER BY featured_at DESC LIMIT 12`.
- Response `Vec<HttpFeaturedDeck>` (`zwipe-core` contracts): deck name,
  format, commander card(s) (image URIs for the tile art), color identity,
  card count, `share_token` (to build the `/deck/{token}` link), and
  `mvps: Vec<HttpCard>` (0–3, from `deck_cards.mvp_at IS NOT NULL`, no
  vesting filter — the showcase is the owner's statement, not signal).
- **No user identity**, consistent with `HttpSharedDeck`. Opt-in attribution
  ("built by …") is a later decision, not a v1 default.
- CF-cacheable ~1h; the showcase changes when you run the zcript, not per
  request.

`.sqlx` prepare from workspace root as always.

## 4. zite — the showcase

- **Home strip:** a "Featured decks" section on `pages/home.rs` (below the
  demo videos): 3–4 tiles, link to the full page. Empty response → section
  doesn't render (safe to deploy before anything is featured).
- **Full page:** `#[route("/decks")]` → grid of tiles. Each tile: commander
  art, deck name, format chip, color identity glyphs, card count, and the
  MVP row — up to three small card thumbnails under a ★ header (this is
  the hook: the three cards the builder swears by). Tile click → the
  existing `/deck/{token}` share page.
- Terminal aesthetic, crisp, single column on phones / grid on wide
  screens, `overflow-x` contained. `noindex` NOT set here (unlike share
  pages, the showcase is deliberately public and crawlable — add it to
  `zite/build.rs` ROUTES for the sitemap).
- Copy: sentence case, no em dashes. Footer CTA same as share page
  ("Built with Zwipe").

## 5. Testing

- Server: featured query excludes unshared/unfeatured decks; identity
  absent from the payload; MVP subselect caps at 3 (integration harness
  cases once it exists).
- zite: 0-featured (section hidden), 1, and 12-deck states; tile → share
  page navigation; mobile layout.

## Later

- Rotation cadence ("featured this week" framing) once there are enough
  candidates to rotate.
- Algorithm-nominated candidates (vested-MVP density, activity) with owner
  approval — the hybrid model, deferred until MVP volume exists.
- In-app featured surface (the app is for building; the showcase can stay
  web-first until there's a discover surface worth building).
