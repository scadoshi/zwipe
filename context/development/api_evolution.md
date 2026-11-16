# API Evolution Rules

How to change HTTP contracts without breaking deployed clients. iOS builds linger for weeks after release; the server must always tolerate every build still in the wild.

## The rule: new request fields are additive and defaulted

Every new field on a request contract gets `#[serde(default)]` (plus a sensible `Default`; for enums, mark the backward-compatible variant `#[default]`). The absent field must mean "the behavior that shipped before the field existed."

```rust
/// Add on top (default) or replace the board.
#[serde(default)]
pub mode: ImportMode,   // enum { #[default] Add, Replace }
```

This buys the two-step deploy with no version gate:

1. Deploy server, old clients omit the field, get the old behavior.
2. Ship client, new clients send the field, get the new behavior.

No "wait for propagation", no flag-day, no cleanup commit later.

Deploy order matters in one direction only: **server first**. An old server silently ignores unknown JSON fields, so a new client against an old server gets old behavior with no error. Never ship the client ahead of the server.

## When the rule can't apply

Changes that alter the meaning of existing fields or responses (wire-format changes, auth-flow changes) can't be expressed as additive fields. Those need the propagation-wait + min-version gate pattern instead. Reach for that only when additive genuinely can't work; it costs weeks, additive costs nothing.

A floor only allows dropping an old value when a store build at or above it *writes* the new value, not just reads it. Clients send values back too, serialized with their own core: requests keep accepting the old value until the floor reaches the first store build that serializes the new one. Raising the floor to a build that only reads the new name, then refusing the old one, broke every live client's settings changes (pip style, 2026-11-09, fixed by #494).

## Corollaries

- New response fields are fine for the same reason (old clients ignore them), but never remove or re-type an existing response field while old clients read it.
- New endpoints are always safe (old clients never call them).
- `0.0.0`-style sentinel defaults beat `Option` when "unset" has a concrete meaning the server can act on.

## Rules the clients enforce: the password policy

The clients check a new password against zwipe-core's policy (`domain::auth::password::validate`) as it is typed, on Register, Change password and zwipe.net's reset form, and hold the submit until it passes. The server checks the same policy on its own. Login and an account's current password are never judged, so an old password always still signs in.

Because a store build carries its copy of the policy for weeks, changing the policy needs a migration plan before any code moves:

- **Stricter** (a new rule, a longer minimum): ship the new policy in the apps first and let it reach the stores, then have the server enforce it. Enforcing first turns down passwords the old apps let through, with only the server's message to explain it.
- **Looser** (dropping a rule): update the apps first, since old builds keep blocking passwords the server would now accept until they update. Raise the min-version floor only if the old builds' blocking matters.
