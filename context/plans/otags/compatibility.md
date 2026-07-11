# Compatibility — do not break installed clients

Mobile clients are **deployed binaries in the wild** — we cannot assume everyone is on the
latest build. Compile-time safety (the `zwiper` build depends on the `zerver`/`zwipe-core`
crates, so a breaking symbol change fails the frontend build) protects the *next* release,
**not** the versions already installed. Two runtime guardrails protect those, and this
feature must use both.

## Guardrail 1 — additive-only wire changes

Every one of these is already an established, unit-tested pattern in the repo. otags must
follow them:

- **New request fields are `#[serde(default)]`.** `HttpCreateDeckProfile` /
  `HttpUpdateDeckProfile` (`zwipe-core/src/http/contracts/deck.rs`) already do this, with
  tests pinning that "a client predating these fields sends none of them and must still
  parse." Any new otag field on a request contract gets `#[serde(default)]`.
- **Partial updates use `Opdate<T>`** (`zwipe-core/src/http/helpers.rs`) so *absent* means
  "unchanged," never "clear." Deck otag updates must use `Opdate`, like `other_tags` today.
- **New response fields are `#[serde(default)]` too.** When `oracle_tags` gets added to the
  served `Card` payload, note `CardProfile` (`zwipe-core/src/domain/card/models/card_profile.rs`)
  already carries `mechanical_categories`; the new `oracle_tags` field must be
  `#[serde(default)]` so an **old client** deserializing a **new server's** `Card` does not
  choke on the unknown-but-present field, and a **new client** reading an **old server**
  gets an empty default. (The separate `mechanical_categories → card_roles` rename is
  dual-emitted through its own migration — see §Naming below.)
- **Enums are append-only.** `DeckOtherTag` (`zwipe-core/src/domain/deck/models/deck_other_tag.rs`)
  documents that variants are *only added, never removed or renamed*, so stored values keep
  parsing. If otags become an enum, same rule. (otags are more likely a `String`/newtype
  given hundreds of values — which sidesteps this entirely.)
- **Never remove or rename** a route path in `zwipe-core/src/http/paths.rs` or a field an
  old client depends on.

## Guardrail 2 — the min-version gate

For any change that genuinely **cannot** be made additively, hold it behind the version
floor rather than shipping a break:

- Handshake: `GET /api/client/min-version` → `HttpMinClientVersion { min_version }`
  (`zerver/src/lib/inbound/http/handlers/client.rs`, config `MIN_CLIENT_VERSION` in
  `zerver/src/lib/config.rs`).
- Client polls it in `zwiper/src/lib/inbound/components/auth/session_upkeep.rs`, compares
  via `version_at_least` (`zwipe-core/src/version.rs`, which **fails open** — a malformed
  version never locks anyone out), and flips the blocking "Update required" screen.
- **The intended workflow** is documented at
  `zwiper/src/lib/inbound/screens/deck/components/deck_warnings.rs:35`: *"the wire stays
  untouched until a min-version floor allows the variant"* — i.e. ship the new wire variant
  **dark**, then bump `MIN_CLIENT_VERSION` once you want to rely on clients understanding it.

## What this means for otags specifically

- **Deck otag selection** — the wire is *already backward-safe*: `other_tags` exists on the
  create/update/shared contracts with `#[serde(default)]`. Expanding deck tagging is
  UI-plus-additive-field work; **no version bump needed** if done additively.
- **otags on served cards** — additive `#[serde(default)]` field on `CardProfile`. No bump.
- **New filter predicates** — additive request fields. No bump. An old client simply never
  sends them.
- **New `card_oracle_tags` table + `zervice` sync** — server-internal, invisible to clients. No
  wire impact at all.
- **`(format, CI, otag)` signal keying** — additive new tables + a new/extended metrics
  write. Ship dark and collect; no client break.
- **When a version bump IS unavoidable** (e.g. a serve response whose *shape* changes such
  that old clients misrender): ship it dark, then bump `MIN_CLIENT_VERSION`. Coordinate with
  the release cadence — see `context/operations/` and the min-version gate history.

**Rule of thumb for this feature:** every phase in `scope.md` can be done additively.
Treat any design that *seems* to require removing/renaming a wire field or route as a
mistake to redesign around first, and only reach for the min-version gate as the last
resort.

## Naming: `oracle_tag` canon + the `mechanical_categories → card_roles` wire migration

**Settled 2026-07-11 (owner).** Two **distinct** functional concepts live on the wire and
**both survive** — they are different granularities, not replacements for each other:

- **`oracle_tags`** — the granular community tags (hundreds). Canon everywhere: DB
  (`oracle_tags`, `card_oracle_tags`, `card_profiles.oracle_tags`), Rust (`OracleTag`), wire
  (`oracle_tags` field + `oracle_tags_*` criteria). This is an **additive new field** — no
  migration, just `#[serde(default)]` (see Guardrail 1).
- **The coarse ~24 categories** — human-friendly filter buttons (removal, ramp, …), derived
  from oracle_tag subtrees + `all_parts` (Tokens) + ~4 kept heuristics (Q1). This concept is
  **renamed** `mechanical_category → CardRole`. The old word `mechanical_categories` is our
  pre-otag term and clashes with canon, so it is being **migrated off the wire too.**

**The `mechanical_categories → card_roles` wire migration is a committed, version-gated track
(owner, 2026-07-11)** — deliberately, not the cheap "keep the ugly key forever" shortcut. It
follows the standard dual-emit dance because **serde `rename`/`alias` only fixes
deserialization** (new server reading an old client's *request*), NOT serialization (an old
client reading the new server's *response* looks for the exact key it shipped with):

- **Internal rename is free + first** (Phase 2): `MechanicalCategory → CardRole`, module
  `card/models/mechanical_category/ → card_role/`, `classify.rs → card_role/oracle_tag_gaps.rs`.
  The `CardProfile` field name stays `mechanical_categories` and the DB column stays
  `card_profiles.mechanical_categories` — renaming the *type* doesn't touch the JSON key.
- **Wire migration** (its own later phase — see `sequencing.md`): responses **dual-emit**
  `card_roles` (new) **and** `mechanical_categories` (legacy, same values); requests accept
  **both** `card_roles_*` and `mechanical_categories_*` criteria; clients migrate to
  `card_roles`; then **sunset** `mechanical_categories` (field + criteria + rename the DB
  column) behind a `MIN_CLIENT_VERSION` floor once installs age out.

Net wire end-state: **`oracle_tags`** (granular) + **`card_roles`** (coarse). `mechanical_categories`
is a legacy alias of `card_roles` that exists only through the migration window.
