# API Evolution Rules

How to change HTTP contracts without breaking deployed clients. iOS builds
linger for weeks after release; the server must always tolerate every build
still in the wild.

## The rule: new request fields are additive and defaulted

Every new field on a request contract gets `#[serde(default)]` (plus a
sensible `Default` — for enums, mark the backward-compatible variant
`#[default]`). The absent field must mean "the behavior that shipped before
the field existed."

```rust
/// Add on top (default) or replace the board.
#[serde(default)]
pub mode: ImportMode,   // enum { #[default] Add, Replace }
```

This buys the two-step deploy with no version gate:

1. Deploy server — old clients omit the field, get the old behavior.
2. Ship client — new clients send the field, get the new behavior.

No "wait for propagation", no flag-day, no cleanup commit later.

Deploy order matters in one direction only: **server first**. An old server
silently ignores unknown JSON fields, so a new client against an old server
gets old behavior with no error — never ship the client ahead of the server.

## When the rule can't apply

Changes that alter the meaning of existing fields or responses (wire-format
changes, auth-flow changes) can't be expressed as additive fields. Those need
the propagation-wait + min-version gate pattern instead — see Pending Gated Merges in `context/progress/todo.md`. Reach for that only when additive genuinely can't
work; it costs weeks, additive costs nothing.

## Corollaries

- New response fields are fine for the same reason (old clients ignore them) —
  but never remove or re-type an existing response field while old clients read it.
- New endpoints are always safe (old clients never call them).
- `0.0.0`-style sentinel defaults beat `Option` when "unset" has a concrete
  meaning the server can act on.
