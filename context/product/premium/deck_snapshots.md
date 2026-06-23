# Deck snapshots / version history

**Tier: DECIDED 2026-06-10 — premium.** Cheap storage, high attachment value —
the kind of quiet feature that makes cancelling feel like losing something.

## Concept

Point-in-time snapshots of a deck with diffs between them:

- "What did I change before last week's pod night?"
- Restore a previous version after an experiment flops.
- Auto-snapshot on meaningful events (before a Replace-mode import is the
  obvious first trigger — it's the destructive operation) plus manual
  "save version" with an optional note.

## Implementation sketch

- A `deck_snapshots` table: (deck_id, created_at, note, cards jsonb) — the
  full card list per snapshot is a few KB; even hundreds of snapshots per deck
  are negligible. No diff storage needed — diffs are computed on read by
  comparing two snapshots' card lists.
- Retention: cap per deck (e.g. last N + any manually-pinned) to keep it
  bounded.
- Diff view reuses the import-result idiom (added/removed/quantity-changed
  lists).

## Tier interaction

Free users could get the auto-snapshot safety net silently recorded but only
premium can browse/restore history — "upgrade to recover the version you had
before that import" is a sympathetic upsell moment, though it should never
feel like ransom: maybe free gets restore-latest-only, premium gets the full
timeline. Decide the exact split when building.
