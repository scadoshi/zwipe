# Centralize auth into a router-level gate

**Status: DONE 2026-07-15 (implemented + runtime-tested, archived). Client-only
(zwiper), router + auth components.**
Replace the per-screen `Bouncer` opt-in with a single scoped router layout so every
app screen is authed by default and only the pre-auth screens are public. No
backend, no wire, no visual change.

**One sentence:** promote `Bouncer` to an `AuthGate` router layout that wraps every
route except Login / Register / ForgotPassword, then delete the 11 scattered
per-screen `Bouncer` wrappers — one gate, impossible to forget.

**Related:** unblocks the "authed by default" note in
[`otag_example_cards.md`](otag_example_cards.md) (the examples browse + dictionary
get gated for free). Files: `zwiper/.../inbound/router.rs`,
`zwiper/.../inbound/components/auth/bouncer.rs`.

---

## Why

Auth today is opt-in per screen: each protected screen wraps its body in `Bouncer`
(`components/auth/bouncer.rs`), which redirects to `/login` when there's no valid
session. 11 screens do this (Home, Profile, all deck screens). **Three don't and
are currently reachable with no session: Changelog, PrivacyPolicy,
OracleTagDictionary.** That's the built-in failure mode of per-screen gating — add
a screen, forget the wrapper, leak it. Centralizing makes "authed" the default and
"public" the explicit exception, which matches reality: the only screens that must
work logged-out are the ones that get you logged in.

## Public vs authed (decided)

- **Public** (no gate): `Login`, `Register`, `ForgotPassword`. These are the
  pre-auth flow — a logged-out user must reach all three (register, sign in, reset).
- **Authed** (behind the gate): everything else — `Home`, `Profile`,
  `PrivacyPolicy`, `Changelog`, `OracleTagDictionary`, all deck routes, and the new
  `OracleTagExamples` from the example-cards plan.

Note: `Home` is authed. Its router doc calls it a "landing screen with navigation
to login/register," but it already wraps `Bouncer` today, so a logged-out user at
`/` is already redirected to `/login` — its `session == None` branch is effectively
dead. Moving it under the gate matches current behavior; no change for users.

## Plan

### 1. `AuthGate` layout component
Convert the guard into a layout. Same session check as `Bouncer`
(`session().is_some_and(|s| !s.is_expired())`, redirect to `Router::Login {}` via
`use_effect` when absent), but render `Outlet::<Router> {}` instead of `{children}`
so it wraps child routes rather than injected content:

```rust
#[component]
pub fn AuthGate() -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let has_session = use_memo(move || {
        session().as_ref().is_some_and(|s| !s.is_expired())
    });
    use_effect(move || {
        if !has_session() {
            navigator.push(Router::Login {});
        }
    });
    rsx! { Outlet::<Router> {} }
}
```

Delete the old `Bouncer` component once its usages are gone (below).

### 2. Restructure the router
Scope `AuthGate` to the authed routes. `BackHandlerLayout` stays outermost (wraps
all). Public routes sit between the two layout attributes; authed routes follow
`#[layout(AuthGate)]`. Both layouts close at the enum end, nesting correctly
(BackHandlerLayout outer, AuthGate inner) — no `#[end_layout]` needed:

```rust
#[layout(BackHandlerLayout)]        // all routes: OS back-intent bridge
    // ── public ──
    #[route("/login")]           Login {}
    #[route("/register")]        Register {}
    #[route("/forgot-password")] ForgotPassword {}

    #[layout(AuthGate)]             // everything below requires a session
        #[route("/")]            Home {}
        #[route("/user")]        Profile {}
        #[route("/privacy")]     PrivacyPolicy {}
        #[route("/changelog")]   Changelog {}
        #[route("/oracle-tags")] OracleTagDictionary {}
        #[route("/deck")]        DeckList {}
        // ...all remaining deck routes...
```

Note `/` (Home) now lives under `AuthGate`. Confirm the `Routable` derive is happy
with the default route sitting inside a nested layout (it is — layouts don't change
path matching); `Router::default()` stays `Home {}`.

### 3. Strip the 11 per-screen `Bouncer` wrappers
Remove `Bouncer { ... }` (and its import) from each, un-nesting the body one level:
`home.rs`, `profile/mod.rs`, `deck/{list,view,create,edit,import,export}.rs`,
`deck/card/{add,view,remove}.rs`. Pure de-indentation; no logic moves.

## Verify

- Logged out: visiting any authed route (deep link / back nav) redirects to
  `/login`; Login / Register / ForgotPassword render without redirect.
- Logged in: every screen renders as before; no double-gating, no redirect loop.
- **Session hydration on cold start — CONFIRMED SAFE.** The session context is
  seeded synchronously: `spawn_upkeeper` does `use_signal(Session::infallible_load)`
  (`session_upkeep.rs:124`), which loads the stored session from disk in the signal
  init closure *before* any route renders. So a returning user's session is present
  on the first gated render and `AuthGate` never sees a transient `None` for them —
  only genuinely logged-out users get `None → redirect`. No loading state needed.
- Redirect uses `push` (matches current `Bouncer`); keep it unless we want
  `replace` so login isn't in the back stack — out of scope, note only.
- `cargo +nightly fmt` + clippy clean before push.

## Not doing

- No change to how sessions are created, stored, refreshed, or expired.
- No `replace`-vs-`push` redirect change (call it out, don't bundle it).
- No server-side auth changes — this is purely client route gating.
