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
- **New response fields are `#[serde(default)]` too.** If otags get added to the served
  `Card` payload, note `CardProfile` (`zwipe-core/src/domain/card/models/card_profile.rs`)
  already carries `mechanical_categories`; a new `otags` field there must be
  `#[serde(default)]` so an **old client** deserializing a **new server's** `Card` does not
  choke on the unknown-but-present field, and a **new client** reading an **old server**
  gets an empty default.
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
- **New `card_otags` table + `zervice` sync** — server-internal, invisible to clients. No
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
