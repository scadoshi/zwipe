# Local theme persistence (zwiper + zite)

**Status: PLANNED / scoping (2026-07-11). Client-only, additive. No server,
contract, or DB change — the theme is already account-level (JWT prefs); this
adds a *local* last-known cache so pre-auth screens are themed.**

## Goal

Pre-auth / logged-out screens (login, register, verify, reset, zite's public
pages) should render in the **last theme this device saw**, not the hardcoded
`gruvbox-dark` default. Account theme can drift (changed on another device), so
the local cache is a best-effort "last known", overridden by the account's real
prefs the moment a session is present.

## Current behavior (the gap)

- **zwiper** — `spawn_upkeeper` (`session_upkeep.rs:133`) seeds the theme signal
  from `session.preferences` **only if a persisted session loads**; otherwise
  `ThemeConfig::default()`. So logged-out / expired / fresh-install-after-logout
  all show gruvbox-dark on the login flow. Theme is re-set on login
  (`auth/login.rs:67`), prefs update (`profile/preferences.rs:125`,
  `profile/mod.rs:129`), and home (`home.rs:124`) — but **never persisted
  locally**. (Session *is* persisted via the `Persist` trait in
  `outbound/session.rs`: keyring on Apple/desktop, private-storage JSON on
  Android.)
- **zite** — `App` (`main.rs:99`) is hardcoded `use_signal(ThemeConfig::default)`;
  the `ThemePicker` choice is lost on reload. A `use_effect` (`main.rs:104`)
  already applies `theme.css_class()` to `<body>` reactively — the natural single
  write point.

## Shared prerequisite (zwipe-core)

`ThemeConfig` (`zwipe-core/.../user/models/theme.rs`) currently derives only
`Clone, PartialEq`. Add `Serialize, Deserialize` (serde is an allowed core dep)
so it can round-trip to JSON / localStorage. One-line, additive, no purity
concern. (Alternatively persist the `{name, dark}` pair, but serializing
`ThemeConfig` directly is cleanest.)

## zwiper — mirror the session persistence pattern (DECIDED: reuse keyring)

A new `outbound/theme_store.rs` mirrors `session.rs`'s `#[cfg]` structure exactly,
storing a small serialized `ThemeConfig` blob under a `"theme"` key:
- **Apple/desktop** → keyring entry (service `zwiper-service`, username
  `zwiper-theme`), same `default_credential_builder` calls as `session.rs`.
- **Android** → `theme.json` in private internal storage, reusing the existing
  `files_dir()` JNI helper. Factor `files_dir()` out of `session.rs`'s Android
  module into a shared spot (both `session.json` and `theme.json` live in the same
  dir) rather than duplicating the ~15 JNI lines.
- **Web** (`#[cfg(not(any(feature="desktop",feature="mobile")))]`) → `localStorage`.
- Infallible `save`/`load` (log-and-continue), like `Session::infallible_*`.

The theme isn't a secret, but reusing the proven keyring/JNI matrix is the
lowest-risk path (zero new deps, no new Apple/desktop path helper) — chosen over a
plain-file store on that basis. A theme string in the keychain is harmless.

**Wiring (two edits):**
1. **Seed precedence** in `session_upkeep.rs:133` — session prefs win when logged
   in, else the local cache, else default:
   ```rust
   let theme = use_signal(|| {
       session.peek().as_ref()
           .map(|s| ThemeConfig::from(&s.preferences))
           .or_else(ThemeStore::infallible_load)
           .unwrap_or_default()
   });
   ```
2. **Auto-save on any change** — a single `use_effect` right after
   `use_context_provider(|| theme)`:
   ```rust
   use_effect(move || ThemeStore::infallible_save(&theme.read()));
   ```
   This persists every theme change (login, prefs update, picker) at one point —
   no need to touch the individual `theme.set(...)` call sites.

**Logout:** keep the theme cache (do *not* delete it alongside the session) — the
whole point is that the login screen after logout keeps the last theme.

**Platform note:** zwiper ships native (desktop/iOS/Android); web is a `dx serve`
dev target. Keyring on wasm won't work, hence the web localStorage branch.

## zite — localStorage (wasm SPA)

1. **Init from storage** (`main.rs:99`):
   `use_signal(|| ThemeStore::load().unwrap_or_default())`.
2. **Write on change** — fold the save into the existing body-class `use_effect`
   (`main.rs:104`), which already fires on every theme change:
   `ThemeStore::save(&theme.read());`
3. **Storage impl:** `gloo-storage::LocalStorage` (ergonomic serde get/set) under
   key `"zwipe.theme"`; add the `gloo-storage` dep (zite already uses `gloo_timers`).

**SSR / FOUC caveat (zite-specific):** zite is SSR + hydration (`#[server]` fns,
`Route::static_routes`). `localStorage` is client-only, so reading it during the
initial signal would (a) not exist on the server render and (b) risk a hydration
mismatch / flash of default theme. Two mitigations:
- **Simple:** keep the SSR/initial value at `default`, then read localStorage in a
  **mount-time `use_effect`** and `theme.set(...)` — accepts a one-frame
  default→stored flash (same class of behavior as today, just corrected fast).
- **No-flash (recommended for a public site):** inject a tiny inline script in the
  document `<head>` that reads `localStorage["zwipe.theme"]` and sets
  `document.body.className` **before** hydration, so first paint is already themed.
  Dioxus renders the signal to `default` but the body class is already correct;
  reconcile the signal on mount.
zwiper (webview SPA, no SSR) has no FOUC problem — the seed is synchronous.

## Shared-code question

Storage stays **per-app** (like session, which lives only in zwiper): zwiper's
native backends can't live in the shared `zwipe-components` UI crate. If we later
want a common web helper, a `theme_store` (localStorage) could move into
`zwipe-components` for both to share on web — but that's not worth the coupling
now.

## File-by-file changes (implementation checklist)

**zwipe-core**
- `domain/user/models/theme.rs` — add `Serialize, Deserialize` to `ThemeConfig`'s
  derive (currently `Clone, PartialEq`). serde is already a core dep.

**zwiper**
- `outbound/session.rs` — hoist the Android `files_dir()` JNI helper out of the
  `#[cfg(target_os="android")] mod platform` so `theme_store` can reuse it
  (extract to e.g. `outbound/android_fs.rs`, or a shared `pub(crate) fn`).
  `session.json` and `theme.json` land in the same dir.
- `outbound/theme_store.rs` (NEW) — `ThemeStore::{infallible_save(&ThemeConfig),
  infallible_load() -> Option<ThemeConfig>}`, same `#[cfg]` split as `session.rs`:
  keyring (Apple/desktop, service `zwiper-service`, username `zwiper-theme`),
  Android `theme.json`, web `localStorage`.
- `outbound/mod.rs` — `pub mod theme_store;` (+ `android_fs` if extracted).
- `inbound/components/auth/session_upkeep.rs` — two edits at the theme block
  (~L133): (1) seed `session prefs → ThemeStore::infallible_load() → default`;
  (2) add `use_effect(move || ThemeStore::infallible_save(&theme.read()))` right
  after `use_context_provider(|| theme)` so every change persists at one point.

**zite**
- `Cargo.toml` — add `gloo-storage`.
- `src/main.rs` — init the theme signal from storage (or default + mount-effect
  for SSR), fold `LocalStorage::set("zwipe.theme", …)` into the existing
  body-class `use_effect` (L104), add the inline no-FOUC `<head>` script.

**Deliberately NOT touched:** `zwipe-components` (theme storage stays per-app) —
so zero collision with the otags agent's `lib.rs` / `card_row.rs` /
`oracle_tag_chips.rs` / `card_role_chips.rs` / `components.css` work.

## Coordination with the otags frontend work (checked 2026-07-11)

All six target files (`session.rs`, `outbound/mod.rs`, `session_upkeep.rs`,
`bin/zwipe.rs`, `zite/src/main.rs`, `theme.rs`) are **currently clean** — the
otags agent's footprint (`screens/deck/**`, `zwipe-components/{lib,card_row,
*_chips}.rs`, `components.css`, card/deck domain + contracts) is disjoint from
every file here. The one co-owned file across all theme+persistence work is
`zwipe-components/src/changelog.rs` (both append to the 1.6.0 `entries` array) —
trivial additive conflicts only; keep edits on distinct lines.

Operational: the zwiper crate does not compile while the otags agent has
`card_info.rs` mid-edit, so **end-to-end verify of the zwiper persistence piece
is gated on their crate compiling** (the zite piece verifies independently). Do
not run crate-wide fmt or tree-wide git ops on zwiper meanwhile; scope fmt to the
touched files.

## Rollout / verify

Additive, no bump. Per `context/development/commit_guidelines.md` (nightly fmt,
clippy, tests). Verify:
- **zwiper:** pick a non-default theme → force logout (or expire session) →
  confirm the login screen renders in that theme, not gruvbox. Re-launch the app
  logged-out → still themed. Log in with an account whose server prefs differ →
  confirm server prefs override the cache. Check all three platforms (keyring /
  Android file / web localStorage).
- **zite:** pick a theme → hard reload → confirm it persists with no (or one
  frame of) flash; confirm no hydration-mismatch warning in the console.
