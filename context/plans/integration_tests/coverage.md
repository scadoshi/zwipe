# Integration tests — coverage targets

Priority-ordered slices; each is one buildable session. HTTP-level unless
marked **[repo]**. Every slice ends green with `cargo test -p zerver`.

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
instead of extending the gap: share tokens (`deck_share_page`: public
endpoint 404-on-revoked, identity stripping), MVPs (`deck_mvps`: 3-slot cap
422, vesting math **[repo]**), wildcard slot (page splice position,
deep-pool floor **[repo]**). Their plan docs already specify the cases;
this harness is where they run.

## Explicitly out of scope

- zwiper/zite UI tests (different toolchain problem, no harness today)
- load/perf testing
- coverage percentage targets — the goal is the enumerated behaviors above,
  not a number
