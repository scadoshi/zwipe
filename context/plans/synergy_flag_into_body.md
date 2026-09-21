# Move the synergy-warming flag out of the response header

**Status: PLANNED 2026-09-21, deliberately not scheduled. Owner wants the flag
in the body; the migration needs a forced-update cycle, so it waits for one
that is happening anyway.**

**One sentence:** `search_deck_cards` should answer
`{ cards: [...], synergy_applied: bool }` instead of a bare card array plus an
`x-synergy-applied` response header.

## Why

Whether synergy was applied is a fact about the result, not about the
transport. It belongs in the body with the cards it describes.

Two concrete costs of the header today:

- It is invisible to the contract. `search_deck_cards` is the only one of 56
  endpoints that cannot go through the `Endpoint` trait, purely because it
  reads a header.
- A browser cannot see it. The CORS layer sets `allow_origin`,
  `allow_methods` and `allow_headers` but not `expose_headers`, so the header
  is readable by zwiper (native) and silently absent in zite. If zite ever
  grows card search it would always read "synergy applied" and never show the
  warming message, with nothing to explain why.

The header was the right call when it shipped: the body was already a bare
array and an envelope would have broken every installed client at once. This
plan is how to pay that off, not a complaint about it.

## Why it can't just be changed

Shipped clients do `response.json::<Vec<Card>>()`. An envelope makes that
fail to decode on every search, on every version in the field. The bare array
is load-bearing until no old client is left.

## The dance

The force-update gate (`MIN_CLIENT_VERSION`, served at
`/api/client/min-version`, enforced by zwiper's update-required screen) is
what makes this safe: it can guarantee a version floor.

1. **Ship a tolerant reader.** A client release whose search decodes either
   shape: envelope if the payload is an object, bare array if it is an array.
   Server unchanged. Keep reading the header in this release, since the
   server is still sending it.
2. **Raise the floor.** Set `MIN_CLIENT_VERSION` to that release once it is
   live on both stores. Older clients are force-updated, so no installed
   client parses a bare array any more.
3. **Flip the server.** `search_deck_cards` returns the new contract type
   (add `HttpDeckCardSearch { cards, synergy_applied }` to
   `zwipe-core/src/http/contracts/deck.rs`) and stops setting the header.
4. **Clean up.** Drop the array fallback from the client, drop the header
   read, and fold the endpoint back into the `Endpoint` trait: it becomes an
   ordinary `type Response = HttpDeckCardSearch`, and
   `zwiper/.../client/deck/search_deck_cards.rs` loses its hand-written
   transport like the other 55.

Steps 1 and 2 ride a release that is happening for other reasons. Only step 3
is a server deploy, and it is a one-liner once the floor is set.

## Verification

- Step 1: unit-test the tolerant decode against both payload shapes; the app
  still shows the warming message against an unchanged server.
- Step 2: confirm the store versions, then read `/api/client/min-version`.
- Step 3: integration test asserting the envelope, and a manual search on a
  commander whose cache is cold.
- Step 4: the endpoint cross-check (method/status/auth/body) still matches.

## Explicitly out

- Enveloping other list endpoints. This one earns it because it carries
  metadata; the rest do not.
- `expose_headers` for CORS. It would make the header work in a browser, but
  the point of this plan is to delete the header. Add it only if zite grows
  card search before step 3 lands.
