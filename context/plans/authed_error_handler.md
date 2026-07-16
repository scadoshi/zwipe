# Centralized authed-call error handling (facade)

**Status: BACKLOG (2026-07-15).** Not urgent — captured so it isn't lost. A
cross-cutting cleanup, not part of the otag/examples batch.

**One sentence:** replace the per-call-site `ensure_fresh` + hand-rolled error
handling with one thin authed facade so every authed request refreshes, retries,
and surfaces failures the same way — killing the "some screens toast, some swallow"
inconsistency at its root.

**Related:** the examples-screen silent-failure bug
([`otag_examples_followup.md`](otag_examples_followup.md) P3, since patched with a
local toast) was a *symptom* of this. This plan is the structural fix.

---

## Why

Every authed call today is:

```rust
let session = session.ensure_fresh(client).await?;   // refresh if stale, or bail
client().some_call(&args, &session).await             // then the actual call
```

`ensure_fresh` lives on the session signal (app state), not the client, because it
also **persists the refreshed token, updates the shared session signal, and clears
the session on auth failure** (which the `AuthGate` turns into a login redirect).
The `ZwipeClient` stays a dumb outbound HTTP adapter. That separation is correct.

The cost is boilerplate at every call site, and — worse — **each site hand-rolls
its own error handling**. Result: inconsistent behavior we can see today:

- `remove.rs` mutation paths → toast `e.to_user_message()` on `ensure_fresh` Err.
- `add.rs` `load_more_cards` → **silent** on `ensure_fresh` Err.
- `oracle_tag_examples.rs` → **was silent** (now patched to toast).

Hand-rolled ceremony → some sites do it wrong. Centralizing fixes the whole class.

## Options (pros/cons already worked out)

### A. Authed facade *(recommended)*
A wrapper that, per authed call, does `ensure_fresh` → call → uniform error
surfacing (toast transient / let auth-failure fall through to the AuthGate
redirect), all in one place. Call sites become `authed.search_cards(&filter)`.

- **Pros:** reuses `ensure_fresh` as-is; refresh stays *out* of the HTTP adapter
  (clean layering); deterministic; central error handling; low risk.
- **Cons:** refreshes pre-emptively off a client-side expiry check (occasional
  needless refresh); tiny race between ensure and call; won't catch a server-side
  revoked token the client thinks is valid; each authed method needs a facade
  counterpart (or a closure-passing generic).

### B. 401-retry interceptor
Call with the current token; on 401, refresh once and retry; if refresh fails,
clear session → redirect.

- **Pros:** lazy — only refreshes when the server actually rejects; correct under
  clock skew / server-side revocation; zero per-call ceremony; closes the expiry
  race.
- **Cons:** a wasted 401 round-trip at the expiry boundary; needs single-flight so
  a burst of concurrent 401s don't all refresh; retry-safety (fine for reads, think
  twice for mutations); to live as middleware it needs the session signal injected
  **into the client** — re-coupling app state into the outbound adapter, the exact
  smell the current design avoids.

## Recommendation

**Facade (A)**, unless excessive refreshes are ever measured as a real problem. It
keeps `ZwipeClient` a dumb adapter, reuses the existing single-flight
`ensure_fresh`, and centralizes error handling with the least risk. The interceptor
is more correct on refresh *timing* but buys concurrency/retry complexity and a
layering regression we don't need.

Possible hybrid later: a facade that uses 401-retry internally — gets B's timing
correctness while keeping refresh out of the raw client. Only if timing bites.

## Scope / cost

Cross-cutting: touches every authed call site (deck CRUD, deck-card CRUD, search,
user, preferences, skip/suggestion posts, ...). Do it as its own deliberate PR, not
folded into a feature. Migrate call sites incrementally — the facade can coexist
with direct `ensure_fresh` during the transition.

## Not doing now

- No refactor as part of the otag/examples work; screens keep the per-call pattern.
- Local per-screen toasts (like the examples fix) are fine stopgaps until this lands.
