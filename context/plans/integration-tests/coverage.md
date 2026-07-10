# Integration tests — coverage targets

Priority-ordered slices; each is one buildable session. HTTP-level unless
marked **[repo]**. Every slice ends green with `cargo test -p zerver`.

---

## Status (2026-07-09)

**Harness + CI shipped + the card fixture builder is built** (overview slices 1–2,
plus `card()`/`seed_cards()` in `tests/common/mod.rs`). **21 integration tests green.
Covered so far:**
- `tests/auth_flows.rs` — register → authed `GET /api/user` → login → refresh, plus
  no-token/wrong-password 401s.
- `tests/deck_flows.rs` — profile CRUD, unverified deck cap → verify unlock,
  duplicate-name reject, cross-user isolation (404), clone.
- `tests/deck_cards.rs` — add/bump(delta)/remove, maybeboard placement, text import
  (resolved + unresolved).
- `tests/card_serving.rs` — get-by-id round-trip, missing→404, search by
  name_contains / cmc_range / color_identity_within, search-requires-auth.
- `tests/repo_card.rs` — [repo] synergy ordering (scored-desc vs UNSCORED_ANCHOR),
  `exclude_oracle_ids` drop, NULL-`oracle_id` deck-aware-shuffle regression,
  `card_signal_rollup` math.

**Next:** Slice 4 below (metrics/user/health) and the auth edges. Deferred within
card serving (optional): deck-aware serve suppression exclusion + land auto-stop
(HTTP), band-boundary/different-deck shuffle + clone card-copy ([repo]). Everything
else below is open.

## Recommended build order — fastest path to full-system coverage

Optimized for coverage-per-hour, not for the slice numbers below (which are
grouped by area). Grab them in this order:

1. **Deck lifecycle + its repo tests** — biggest untested surface, the core
   product, **zero coverage today**. Highest coverage-per-hour. (Slice 2 below.)
2. **Card serving + its repo tests** — highest *regression* risk (the band-shuffle
   NULL bug lived here). **Do the `card(...)` / `seed_cards(...)` fixture builder
   first** (`harness.md` §5) — a one-time investment that unlocks every
   search/serve/signal test. (Slice 3.)
3. **Metrics + user + health** — smaller surface, mostly straightforward. (Slice 4.)
4. **Remaining auth edges** — verify/reset via captured email, lockout 429, IDOR
   spot-check. Lower priority: auth already has strong *unit* coverage, so these
   are the gaps units can't reach, not virgin territory. (rest of Slice 1.)
5. **Future features land WITH tests** — ongoing (Slice 5).

Rationale: deck + card are the product and have **no** server-side coverage;
auth is already the best-tested area. Front-load the `seed_cards` fixture (step 2)
because it's the only real scaffolding left after the harness.

## Endpoint coverage map — the "entire system" target

Track full coverage against this. ✅ = has an integration test; ⬜ = open.

**Auth** — ✅ `POST /api/auth/{register,login,refresh}` · ⬜ `verify-email`,
`request-password-reset`, `reset-password`, `resend-verification`, `logout`
**User** — ✅ `GET /api/user` · ⬜ `GET /api/user/preferences`, change
`username`/`email`/`password`, `DELETE` account, `/api/user/hint`
**Deck** — ✅ `GET/POST /api/deck`, `GET/PUT/DELETE /api/deck/{id}`,
`profile/{id}`, `clone` (`tests/deck_flows.rs`), `POST/PUT/DELETE
/api/deck/{id}/card`, `card/import` (`tests/deck_cards.rs`) · ⬜ deck-aware
`card/search` (serve), `import/archidekt`, `share`, `tokens`, public
`GET /api/deck/{token}`
**Card** — ✅ `POST /api/card/search` (name/cmc/color-identity),
`GET /api/card/{scryfall_data_id}` (`tests/card_serving.rs`) · ⬜
`{oracle_id}/printings`, `artists`, `types`, `keywords`, `oracle-words`,
`languages`, `sets`; **[repo]** synergy/band-shuffle/rollup
**Metrics** — ⬜ all: usage batch, `anonymous`, `stats`
**Health / client** — ⬜ `health`, `/server`, `/database`, `min-version`

## Slice 1 — auth flows (proves the harness)

Auth has the best unit coverage already; this slice is thin on purpose —
it exists to prove the full stack works under test, plus the cases units
can't reach:

- register → verification email captured → verify → login → `GET` a
  private route with the token (the happy path, whole middleware stack)
- refresh rotation: old refresh token single-use (second use 401s)
- password reset via captured email token; old sessions' refresh behavior
- lockout: N bad passwords → 429, correct creds still locked
- **no-IDOR spot check:** authed user A hitting user B's deck routes → 404/403
- middleware: missing/garbage/expired bearer → 401; min-version gate header

## Slice 2 — deck lifecycle (the core product, currently zero coverage)

- create deck (format rules: commander required fields, partners,
  backgrounds) → get → update profile → delete
- deck cards: add / quantity update / remove / maybeboard moves; the
  response shapes the client depends on
- unverified-email deck cap enforced; verified lifts it
- text import: happy path, replace vs merge mode, malformed lines,
  over-cap import
- **[repo]** `clone_deck` (`outbound/sqlx/deck/mod.rs` ~633): clones cards
  + profile fields; asserts the exclusion list — clone must NOT copy
  suppressions-independent identity fields and (once share tokens exist)
  `share_token IS NULL` (the settled privacy rule from the share-page plan)
- **[repo]** suppressions: skip insert, `source='removal'` on single delete,
  re-add deletes the suppression, unskip only removes `source='skip'`,
  5,000-cap eviction (oldest first), bulk import deletes do NOT suppress
- Archidekt import: only if the fetch is behind a port that `tests/common`
  can fake; if it's a concrete client, test the parse/apply layer and leave
  the fetch out (note it as a known gap rather than hitting archidekt.com)

## Slice 3 — card serving (highest regression risk, hand-tested only today)

The band-shuffle/signal code has had one NULL-handling bug already; this
slice replaces the throwaway dev harness with permanent tests:

- name/filter search over the fixture set: color identity gating,
  type/cmc/price filters, pagination
- deck-aware serve: suppressed cards excluded (`NOT EXISTS`), deck cards
  excluded, land-target auto-stop
- **[repo]** synergy ordering: scored fixture map → base ordering correct;
  unscored cards anchor below scored (UNSCORED_ANCHOR); signal term shifts
  a card with seeded `commander_card_signal` rows after rollup refresh
- **[repo]** band shuffle: same (deck, day) seed → identical order across
  two calls; different deck → different in-band order; NULL `oracle_id`
  cards do NOT float to the top (regression test for the 2026-07-06 bug);
  band boundaries respect BAND_SIZE
- **[repo]** `refresh_card_signal_rollup` idempotent + rollup math
  (net = added + 0.5·maybed − removed over shown)

## Slice 4 — metrics + user + health

- usage-batch ingest: daily activity bumps, week signal rows appear
  (`user_week_signal`, `user_week_facet_signal`), deck_skips ride-along
- anonymous events: the three kinds accepted, garbage kind rejected,
  no auth required
- user: change username/email/password (password re-auth required),
  delete account cascades (decks, suppressions, signal rows gone)
- health endpoint; last-active debounce writes `users.last_active_at`

## Slice 5 — future features land WITH tests

Once this harness exists, the plans in flight each ship with their tests
instead of extending the gap: share tokens (`deck-share-page`: public
endpoint 404-on-revoked, identity stripping), MVPs (`deck-mvps`: 3-slot cap
422, vesting math **[repo]**), wildcard slot (page splice position,
deep-pool floor **[repo]**). Their plan docs already specify the cases;
this harness is where they run.

## Explicitly out of scope

- zwiper/zite UI tests (different toolchain problem, no harness today)
- load/perf testing
- coverage percentage targets — the goal is the enumerated behaviors above,
  not a number
