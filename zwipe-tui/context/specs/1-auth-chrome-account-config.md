# Spec 1: auth, app chrome, Account, Config, Privacy policy, Changelog

Source of truth, read for this spec (worktree `the repo root`):

- App: `zwiper/src/bin/zwipe.rs`, `zwiper/src/lib/inbound/router.rs`, `zwiper/src/lib/inbound/components/{auth/*, hint_dialog.rs, hint_host.rs, info_button.rs, screen_header.rs, support.rs, logout_dialog.rs, update_required.rs, home_hero.rs, concept_explainers.rs}`, `zwiper/src/lib/inbound/screens/{auth/*, account/**, config/{mod,preferences,universes_beyond}.rs, changelog.rs, legal/privacy_policy.rs}`, `zwiper/assets/main.css`.
- Kit: `components/src/{lib.rs, changelog.rs, theme.rs}`, `components/assets/magic.css`; orangui at `the orangui repo` (same commit `1bd3943` the worktree's Cargo.lock pins): `src/{entrance, splash, stats_strip, toast_stack, toast_timing, alert_dialog, bottom_sheet, action_bar, page_header, hint, text_input, password_input, button, panel, theme, theme_sheet, theme_picker, theme_wipe, grid_cells}.rs`, `assets/{app,components,toast}.css`, `assets/grid-cells.js`.
- Domain/API: `zwipe-core/src/{domain/user/models/*, domain/auth/models/{password,session}.rs, domain/site.rs, http/contracts/*, http/paths.rs, version.rs, legal/mod.rs, content/changelog}`, `zwipe-client/src/**`.
- TUI today: `zwipe-tui/src/lib/**`.

Conventions in this file: strings in `code` are verbatim. `↗` is U+2197, `·` is U+00B7, `•` is U+2022. "Toast QUICK/NORMAL/LONG" means 3 s / 5 s / 8 s (see Chrome, Toasts). "Facade" means the app's `Authed` helper (Section 0.4): refresh session, run the call, on failure log + telemetry + error toast of `ClientError::to_user_message()` for NORMAL (5 s).

---

## 0. Session plumbing (applies to every authed screen)

### 0.1 Session model and storage

- `zwipe_core::domain::auth::models::session::Session { user: User, access_token: AccessToken, refresh_token: RefreshToken, preferences: UserPreferences }`. `Session::is_expired()` is true when `refresh_token.expires_at < now` (refresh token 14 d; access token 24 h).
- `User { id: Uuid, username: Username, email: Email, email_verified_at: Option<DateTime<Utc>>, hints_shown: HashMap<String, bool>, pending_email: Option<Email>, pending_email_expires_at: Option<DateTime<Utc>> }`. `User::has_seen_hint(key)` = `hints_shown.get(key) == Some(true)`.
- `UserPreferences { theme: String, dark_mode: bool, exclude_universes_beyond: bool, universes_beyond_exceptions: Vec<String>, pip_style: PipStyle }`. Default: `gruvbox`, dark, false, empty, `PipStyle::Theme`. `PipStyle` is `Theme` | `Natural`, wire `"theme"` / `"natural"`, `toggled()` flips.
- The app loads the stored session at start (`Session::infallible_load()`), saves after login and after each refresh (save BEFORE updating the in-memory session, so a crash never strands a rotated token), deletes on logout, on refresh rejection and on refresh-token expiry.

### 0.2 Refresh (`ensure_fresh`, `components/auth/ensure_session.rs`)

- `REFRESH_AHEAD = 2 minutes`: an access token that expires within 2 min is stale.
- Order: no session → `Err(Unauthorized("not logged in"))`. Refresh token expired → delete stored session, clear in-memory session, `Err(Unauthorized("session expired"))`. Access token fresh → return it. Else single-flight refresh: one process-wide mutex; losers wait, then re-check and reuse the winner's session.
- Refresh call: `ZwipeClient::refresh(&self, request: &HttpRefreshSession) -> Result<Session, ClientError>`, `POST /api/auth/refresh`, no auth header. `HttpRefreshSession::new(user_id: &str, refresh_token: &str)` (fields `user_id`, `refresh_token`, optional `client_version`).
- On `Unauthorized | Forbidden | NotFound`: delete stored session and clear in-memory session (the auth gate then sends the user to Login). Any other error: keep the session, return the error (next attempt retries).
- The refresh runs in a detached task so a cancelled caller cannot lose a token rotation.

### 0.3 Auth gate (`components/auth/auth_gate.rs`)

- Every route except `/login`, `/register`, `/forgot-password` sits under `AuthGate`.
- `has_session = session.is_some() && !session.is_expired()`. Whenever it turns false, `navigator.push(Login)`. No toast, no message. This is the only "you were signed out" behavior: logout, account deletion, refresh rejection and refresh-token expiry all land on Login silently.
- Default route is `/` (Home).

### 0.4 The `Authed` facade (`components/auth/authed.rs`)

- `run(op, f) -> Option<T>`: `ensure_fresh`, then `f(client, session)`. On any error (refresh or call): `tracing::warn!("{op} failed: {e}")`, telemetry `report_error(screen, component, op, &e)`, and `toast.error(e.to_user_message(), duration TOAST_NORMAL)`. Returns `None`.
- `run_at(component, op, f)`: same with a component breadcrumb.
- `try_run(op, f) -> Result<T, ClientError>`: same reporting and toast, but hands the error back (used for optimistic toggles that revert).
- `run_quiet` / `run_quiet_at`: telemetry and log only, no toast (prefetches, things the user never waited on).
- `ClientError::to_user_message()` (`zwipe-client/src/error.rs`): `Network` → `connection error, check your network and try again`; `BadRequest | Decode | InternalServerError` → `something went wrong, please try again`; `PayloadTooLarge` → `request too large, try importing fewer cards`; every other variant (401, 403, 404, 409, 422, 429) → the server's message verbatim. Note these three fallback strings are lowercase and have no period; keep them verbatim (the codebase is moving away from all-lowercase UI copy, but don't re-case server or client strings).

### 0.5 Session upkeep loop (`components/auth/session_upkeep.rs::spawn_upkeeper`)

Runs once at the app root (above the router), so it runs on every screen including Login.

- `UPKEEP_INTERVAL = 60 s`, `tokio::time::interval`, first tick fires immediately (that is the cold-start refresh).
- Each tick: `let _ = session.ensure_fresh(client).await;` (errors ignored here; an auth rejection clears the session and the gate redirects). Then `client.fetch_min_client_version()` (`GET /api/client/min-version`, no auth) → `HttpMinClientVersion { min_version: String }`. `required = !version_at_least(APP_VERSION, &min.min_version)` with `APP_VERSION = env!("CARGO_PKG_VERSION")`. If `required` differs from the current gate value, set it (it can flip back to false). Fails open: a failed fetch changes nothing.
- `zwipe_core::version::version_at_least(current, minimum)`: parses `major.minor.patch` (missing segments read as 0), compares tuples; if either side fails to parse it returns `true` (never locks anyone out).
- Other one-time startup work done in the same place (all background, none blocking):
  - Changelog fetch: `client.fetch_changelog()` (`GET /api/changelog`, no auth) → `HttpChangelog`, stored in a root `ChangelogCache` = `Loading` | `Loaded(HttpChangelog)` | `Failed`. Fetched once per launch, even logged out. No retry.
  - Public catalog prefetch (artists, sets, keywords, keyword reminders, oracle words, card types, card roles, oracle tags, UB franchises), 1-day TTL; deck tags warm once a session exists.
  - Logged-out launch only: anonymous funnel event `AppOpened` (see 1.2).
  - Pending crash report post (native only).
  - Usage telemetry flushers (30 s timer + on app backgrounding).
- Theme at start: the stored session's `preferences` (via `theme_from_preferences`) if a session exists, else the last theme cached locally, else default `gruvbox` dark. Every theme change is persisted locally so pre-auth screens render in the last theme. Pip style at start: stored session's `preferences.pip_style`, else `Theme`.

### 0.6 Update required screen (`components/update_required.rs`)

- Shown by the root INSTEAD of the router (and toasts, hints, dialogs) whenever the gate is true. No dismiss, no back, no way past it.
- Layout top to bottom: an empty header bar (`div.page-header`, no title, no buttons); centered content (`content-enter`): a card containing the headline `Update required` (red `#ff3030`, bold, uppercase, pulsing opacity 1 → 0.35 → 1 over 1.6 s ease-in-out, infinite), a full-bleed rule, then the paragraph: `This version of ` + **`Zwipe`** (in `--palette-3`) + ` is no longer supported. ` + **`Update to keep building.`** (in `--accent-primary`). Below the card a full-width button-styled link: `Open App Store ↗` on iOS / `Open Play Store ↗` on Android, linking to `{WEB_BASE}/download/ios` or `{WEB_BASE}/download/android`. Then an empty action bar.
- `WEB_BASE` = `https://zwipe.net` in release, `http://localhost:8080` in debug.

### 0.7 Logout (`components/auth/signal_logout.rs`)

- `LOGOUT_TIMEOUT = 5 s`. `session.logout(client)`: spawn: `ensure_fresh`, then `client.logout(&session)` (`POST /api/auth/logout`, authed), bounded by `tokio::time::timeout(5 s)`. Whether it succeeded, failed or timed out, then delete the stored session and set it to `None` (auth gate redirects to Login). No toast in any case.

---

## 1. Auth screens

All three are public routes. Every one uses `ScreenHeader` (so the left `!` help button is present; none has a `?` hint) and shows `HomeHero` (logo + everybody's counters) above the form.

### 1.0 HomeHero (shared by Login, Register, Forgot password; `components/home_hero.rs`)

- A `Panel` containing, stacked: the `Decode` logo of `zwipe_core::domain::logo::ZWIPE` (see Chrome, Decode), then a compact `StatsStrip` with three figures in this order: `Decks created`, `Searches run`, `Cards swiped`, each rolling (see Chrome, CountUp). Under the strip a chip row: `Everybody` chip, then a `Live` chip only when the numbers were fetched during this launch.
- Data: `client.public_metrics()` (`GET /api/marketing/stats`, no auth) → `HttpPublicMetrics { cards_swiped: i64, searches: i64, decks_created: i64 }`.
- Cache: one root copy shared by every screen with the hero. `COUNTS_FRESH = 60 s`. On mount, if the copy is missing, restored-from-disk, or older than 60 s, refetch; the stale copy stays on screen meanwhile and is only replaced on success. Every successful fetch is saved to disk; the next launch starts from that restored copy (shown without `Live`).
- Loading: a figure whose value is `None` rolls random 4-digit numbers until it arrives (never blank, never "0"). Error: logged only; whatever copy exists stays. No toast, no error text.
- Each mount of the hero re-runs the logo decode and re-rolls the counters, so moving between Login, Register and Forgot password replays both.

### 1.1 Login (`/login`, `screens/auth/login.rs`)

Top to bottom:

1. `ScreenHeader { title: "Login" }`: `!` help button left, title `Login` centered, no `?`.
2. Content (`screen-content centered content-enter`): `HomeHero`; then a centered narrow form:
   - `TextInput` label `Username or email`, placeholder `Username or email`, id `identity`, type text. Autocapitalize/autocorrect/spellcheck off.
   - `TextInput` label `Password`, placeholder `Password`, id `password`, type password (a `PasswordInput`: masked, with a `Show` / `Hide` toggle button inside the field; aria labels `Show password` / `Hide password`).
   - No inline error lines on this screen.
3. `ActionBar` with three Action buttons, all disabled while a request is out:
   - `Log in` (label `Logging in...` while loading).
   - `Create profile` → push `/register`.
   - `Forgot password` → push `/forgot-password`.

Behavior:

- Prefill: a root `LoginPrefill(Signal<Option<String>>)`. Whenever it holds a value (set by Register on success), the identifier field takes it, the password field clears, and the prefill resets to `None`. Works even while Login is mounted under Register.
- Submit gate: if a request is out, ignore the tap. Else if `identifier.trim()` is empty or the password is empty, `toast.error("Enter your username or email and password", TOAST_NORMAL)` and stop. Login deliberately applies NO username/email/password policy (rules may tighten; the server is the only judge).
- Request: `HttpAuthenticateUser::new(&identifier, &password)` (note: identifier sent untrimmed as typed); `client.authenticate_user(request) -> Result<Session, ClientError>` (`POST /api/auth/login`, no auth). The client fills `platform = ClientPlatform::CURRENT` and `client_version = CARGO_PKG_VERSION`.
- Success: save session; compute theme from `session.preferences` (`theme_from_preferences`: unknown slug → default, retired slug → replacement), apply it (app-wide theme wipe), set pip style, set session, clear the password field, clear loading, `navigator.push(Home)`. No toast.
- Failure: `toast.error(e.to_user_message(), TOAST_NORMAL)`, clear loading, fields kept. (Wrong credentials surface the server's 401 message verbatim.)

### 1.2 Register (`/register`, `screens/auth/register.rs`)

Top to bottom:

1. `ScreenHeader { title: "Create profile" }`.
2. Content (`screen-content content-enter`, not vertically centered): `HomeHero`; then the form:
   - `TextInput` label `Username`, placeholder `Username`, id `username`, error line under it.
   - `TextInput` label `Email`, placeholder `Email`, id `email`, type `email`, error line.
   - `TextInput` label `Password`, placeholder `Password`, id `password`, type password, error line, and a `?` beside the label that opens hint topic `Password` (dialog title `Password`, bullets = `password::requirements()`, see 1.4).
   - `TextInput` label `Confirm password`, placeholder `Confirm password`, id `confirm_password`, type password, no error line of its own (mismatch shows under Password).
3. `ActionBar`: `Back to login` (go back), `Create profile` (`Creating...` while loading). Both disabled while loading.

On mount: anonymous funnel event `RegisterViewed` (once per mount).

Validation (live; each field starts validating once it has been non-empty at least once, or after a submit attempt; re-validates on every change):

- Username: `zwipe_core::domain::user::username::Username::new(raw)` (trims first). Errors (displayed as `e.to_string()`, verbatim, lowercase): contains a bad word → `no naughty bad words please`; any whitespace or zero-width char (U+200B, U+200C, U+200D, U+2060, U+FEFF) → `cannot contain whitespace`; fewer than 3 chars → `must be at least 3 characters long`; more than 20 → `must not exceed 20 characters`. Checked in that order.
- Email: `Email::new(email.trim())` (email_address crate, TLD required, no domain literal, no display text). Error text via `UserFacing::to_user_facing_string` (`zwiper/src/lib/domain/error.rs`): `invalid character in email`, `missing @ symbol`, `missing text before @`, `text before @ is too long`, `missing domain after @`, `domain is too long`, `empty part in domain`, `a part of the domain is too long`, `domain must have at least one dot`, `invalid dot placement in domain`, `unbalanced quotes in email`, `invalid comment in email`, `invalid ip address in domain`, `domain literal not supported`, `display name not supported`, `missing display name`, `missing closing bracket`.
- Password: `zwipe_core::domain::auth::password::validate(&pw)`; first failure wins, in this order, shown as `e.to_string()`: length < 8 bytes → `must be at least 8 characters long`; > 128 → `must not exceed 128 characters`; `must have at least one uppercase letter`; `must have at least one lowercase letter`; `must have at least one number`; no char from `~!@#$%^&*()_+=[]{}\/?|:;<>,.` → `must have at least one symbol from ~!@#$%^&*()_+=[]{}\/?|:;<>,.`; `must not contain whitespace characters`; fewer than 6 distinct chars → `must contain at least 6 unique characters`; a char repeated more than 3 times in a row → `must not contain more than 3 repeated characters`. Then, if the policy passes and confirm differs from password: `Passwords do not match`, but only once a submit was attempted OR the confirm field is at least as long (in chars) as the password.
- Submit: mark attempted, validate all three; stop if any error. Then anonymous event `RegisterSubmitted`, then `HttpRegisterUser::new(&username, &email.trim(), &password)` (username sent as typed, untrimmed), `client.register(request) -> Result<HttpRegistrationAccepted, ClientError>` (`POST /api/auth/register`, no auth). `HttpRegistrationAccepted { status: RegistrationStatus::VerificationSent }` (wire `{"status":"verification_sent"}`); the answer is identical whether or not the email already has an account.
- Success: clear loading; set `LoginPrefill` to the USERNAME (not the email); `toast.info("Check your inbox to verify your email, then log in", TOAST_NORMAL)`; `navigator.go_back()` (back to Login, which takes the prefill).
- Failure: `toast.error(e.to_user_message(), TOAST_NORMAL)`, clear loading.
- Anonymous events: `client.record_anonymous_event(&HttpAnonymousEvent { session_id, kind })` (`POST /api/metrics/anonymous`, no auth); `session_id` is one random UUID per process; kinds `AppOpened` / `RegisterViewed` / `RegisterSubmitted` (wire `app_opened` etc.). Fire-and-forget, failures logged at debug.

### 1.3 Forgot password (`/forgot-password`, `screens/auth/forgot_password.rs`)

Top to bottom:

1. `ScreenHeader { title: "Reset password" }`.
2. Content (`screen-content centered content-enter`): `HomeHero`; then either:
   - Form state: `TextInput` label `Email`, placeholder `Email`, id `email`, type text, error line. While the request is out, a spinner under the field (`.spinner`: a bordered rectangle spinning 360° every 0.5 s, linear, infinite).
   - Success state (replaces the form): a centered `Panel` titled `Check your email` with body `If that email is registered, a reset link is on its way.`
3. `ActionBar`: `Back to login` (go back) always; `Send reset link` only while not yet successful. Neither button is disabled while loading (a second tap sends again).

Validation: `Email::new(email)` (NOT trimmed here). On error the message is the fixed `Please enter a valid email address` (not the per-error text). Same touched/attempted rule as Register.

Request: `HttpRequestPasswordReset::new(&email)`; `client.request_password_reset(request) -> Result<(), ClientError>` (`POST /api/auth/forgot-password`, no auth). Success → success state. Failure → `toast.error(e.to_user_message(), TOAST_NORMAL)`. Clear loading either way.

The rest of the flow is NOT in the app. There is no code-entry step. The email contains a link to the website, `{WEB_BASE}/reset/{token}` (zite `src/pages/reset.rs`), which:

- Shows a panel `Reset password` with `New password` (with a `?` opening a `Password rules` panel listing `password::requirements()` and a `Got it` button) and `Confirm password` (placeholders `New password` / `Confirm password`), and the action `Set new password` (`Updating...` while out).
- Checks, in order: ignore while loading (token is single use); policy error shown as `Password ` + the error (for example `Password must have at least one number`); `Passwords do not match`; empty token → `This link is missing its code. Open it from the email again.` Live mismatch line `Passwords do not match` once confirm is non-empty and at least as long as the password. Editing either field clears a submit error.
- Calls `client.reset_password(token: String, new_password: Secret) -> Result<(), ClientError>` (`POST /api/auth/reset-password`, body `HttpResetPassword { token, new_password }`). Network/decode errors show `to_user_message()`; any other error shows `This link is invalid or has expired. Request a new one from the app.`
- Success panel `Password reset`: `Your password is updated and every session is signed out.`

If the TUI wants an in-terminal reset step (optional, the app has none), the client method above is the one to call with the token from the link.

Other email-link endpoints the TUI may see referenced: `verify_email(token)` (`/api/auth/verify-email`) and `confirm_email_change(token)` (`/api/auth/confirm-email-change`) are also website-only; the app never calls them.

### 1.4 Hint topic `Password` (used by Register and Change password)

Dialog title `Password`, bullets (from `password::requirements()`):

- `8 to 128 characters`
- `At least one uppercase letter`
- `At least one lowercase letter`
- `At least one number`
- `At least one symbol from ~!@#$%^&*()_+=[]{}\/?|:;<>,.`
- `No spaces`
- `At least 6 different characters`
- `No character more than 3 times in a row`

Button `Got it`.

### 1.5 Navigation summary

- Cold start: no valid session → gate pushes `/login`. Valid session → `/` (Home).
- Login → Register (push), → Forgot password (push), → Home on success (push).
- Register → back to Login (go_back) on `Back to login` or on success.
- Forgot password → back to Login (go_back).
- Any authed screen → Login whenever the session clears.

### TUI today (auth and session)

What exists:

- `Screen::Login(LoginForm)` (`inbound/app/mod.rs`, `inbound/ui/login.rs`): static ZWIPE logo (no decode) over a bordered form panel 48 cells wide with `Username or email` and `Password` boxes (password masked with `•`, no show/hide), focus border in `palette.add`. Keys: Tab/BackTab/Up/Down switch fields, Enter on identifier moves to password, Enter on password submits, Esc quits, Backspace deletes. Busy line `Signing in...`; idle hints `Tab switch  Enter sign in  Esc quit`.
- Empty-field check message `Enter your username or email and your password.` shown inline in red (app: toast `Enter your username or email and password`, no "your", no period). Server errors shown inline (app: toast). TUI trims the identifier before sending (app sends it untrimmed).
- Session: `outbound/account/mod.rs::Account` holds the session behind a mutex (single-flight refresh), `REFRESH_AHEAD = 2 min`, saves to `session.json` (owner-only) on sign-in/refresh, clears on reject (401/403/404) or expiry. Order of checks differs slightly (TUI checks staleness before refresh-token expiry; same outcome in practice).
- Startup: one `CheckVersion` request at launch; `Update::TooOld` swaps to `Screen::UpdateRequired` (TUI copy: `Update required` / `This version of zwipe-tui is no longer supported. Update to keep building:` / `cargo install --git https://github.com/scadoshi/zwipe zwipe-tui`, key `q` quit). Esc or q quits.
- Unauthorized on any request → `SessionEnded` → Login with inline error `Your session ended. Sign in again.` (app: silent redirect, no message).
- Sign out: `O` on Home and Decks signs out immediately with status `Signing out...`, no confirm; `Account::sign_out` awaits `logout` with no 5 s cap.
- On sign-in: applies preferences palette, remembers theme to a local theme file, status `Hello, {username}!`, opens Home.

Gaps and differences to close:

- No 60 s upkeep loop: no proactive refresh and the min-version gate is checked only once at launch and can never flip back. Add a 60 s interval (first tick immediate) that calls `fresh()` then `fetch_min_client_version()`, fail-open, two-way.
- No Register screen, no Forgot password screen, no `Create profile` / `Forgot password` actions on Login.
- No `HomeHero` on auth screens: no counters, no `Everybody`/`Live` chips, no decode animation on the login logo, no 60 s shared counter cache, no disk-restored counters.
- No toasts at all (inline errors and a status line instead); see Shared pieces.
- No `!` help button / Help & feedback dialog.
- No anonymous funnel events (`AppOpened`, `RegisterViewed`, `RegisterSubmitted`); optional, but the app sends them.
- Labels: app button says `Log in` / `Logging in...`; TUI says `sign in` / `Signing in...`. Title `Login`.
- Logout differs: app logs out only from Account through a confirm dialog and caps the server call at 5 s; TUI has `O` with no confirm.
- Update-required copy is intentionally TUI-specific (install command instead of store link); keep, but it should replace every screen and block all keys except quit (it does).

---

## 2. App chrome

### 2.1 ScreenHeader and its buttons (`components/screen_header.rs`, orangui `page_header.rs`)

- Row: left corner `!` button, centered `h2` title, right corner `?` button only when the screen passes a hint signal (no `?` otherwise; corner left empty).
- `!` (support, `components/support.rs`): Action button, class `page-header-support`, aria `Help and feedback`, glyph `!`, faded. Opens AlertDialog:
  - Title `Help & feedback`
  - Body `Reach the team or join the community.`
  - Actions in order: `Close` (dismiss), `Report a problem ↗` (non-danger confirm; opens `mailto:support@zwipe.net?subject=Zwipe%20problem%20report&body=...` where body is percent-encoded `Describe the problem:\n\n\n---\nZwipe {APP_VERSION} · {PLATFORM}`, PLATFORM `iOS` or `Android`; RFC 3986 unreserved chars pass through, everything else `%XX` uppercase), `Join the Discord ↗` (link to `https://discord.gg/s2UReqUUeg`).
- `?` (hint): Action button class `page-header-corner`, glyph `?`, faded; sets the screen's hint `open` signal to true (re-opens the screen's one-time hint on demand).
- Present on every screen in this spec except Update required (empty header bar there).

### 2.2 Inline `?` info buttons and the HintHost (`components/info_button.rs`, `hint_host.rs`)

- Small `?` (`info-button`, aria `What is this?`) beside a field or row label; stops click propagation. Posts a `HintTopic` into a single root channel `Signal<Option<HintTopic>>`; one `HintHost` at the root renders it as an AlertDialog with title `topic.title()`, the body, and one button `Got it`. Only one hint at a time; closing clears the channel. These are NOT one-time; no server call.
- Topics and copy. Formatting: `[X]` is a `HintKey` (an inert button-looking chip, default color `--palette-2` unless noted); `{X}` is a `HintColored` word (bold, colored). "Line:" is a single paragraph; otherwise bullets.

| Topic | Title | Body |
|---|---|---|
| Password | `Password` | see 1.4 |
| Email | `Email` | Line: `Where password resets and verification go` |
| Verification | `Verification` | Line: `Confirms the email is yours. ` `[Resend]` ` if it never came` |
| Theme | `Theme` | • `The app's palette. Picks preview as you tap` • `The last four are color blind modes` • `[☀ light]` ` ` `[☾ dark]` ` switches between the theme's two palettes` (icons are the sun/moon ModeIcon SVGs) |
| Pips | `Pips` | • `[Theme]` ` draws mana symbols in your theme's colors` • `[Natural]` ` keeps Magic's classic colors on every theme` |
| AppIcon | `App icon` | • `The icon on your home screen, in any theme's colors` • `[Match theme]` ` keeps it in your theme as you change it` (skip for TUI) |
| UniversesBeyond | `Universes Beyond` | • `[Hide]` ` keeps crossover cards out of searches and commander picks` • `Exceptions keeps chosen franchises showing` |
| Website | `Website` | Line: `Guides, the changelog and news, in your browser` |
| Version | `Version` | Line: `The build you are on. ` `[Changelog]` ` lists what changed in each release` |
| DeckTags | `Deck tags` | • `The ` `{archetypes}`(--accent-secondary) ` the deck is built around, like Aristocrats or Voltron` • `Picking one pre-selects the ` `{oracle tags}`(--accent-tertiary) ` that define it` • `Up to ` `{MAX_DECK_TAGS}`(--accent-primary) |
| OracleTags | `Oracle tags` | • `The ` `{specific}`(--accent-tertiary) ` things the deck does, like spot removal, ramp or reanimation` • `They ` `{shape which cards we suggest}`(--accent-secondary) • `Your ` `{deck tags}`(--accent-primary) ` pre-pick a starter set from about 4,500. Fine to leave as is` |
| DeckName | `Deck name` | Line: `Shown in your deck list. Change it any time` |
| Format | `Format` | • `Sets deck size, card pool and the command zone` • `Changing it clears the commander and signature spell` |
| Commander | `Commander` | • `Search only shows cards that can lead this format. ` `[Filter]`(--color-warning) ` searches any card` • `[Swipe]` ` picks one by swiping` |
| Partner | `Partner` | Line: `Shows when the commander allows a partner` |
| Background | `Background` | Line: `Shows when the commander can take a background` |
| SignatureSpell | `Signature spell` | Line: `Oathbreaker only: the instant or sorcery that starts in the command zone` |
| PowerLevel | `Power level` | Line: `The Commander bracket the deck aims for, 1 to 5` |
| LandTarget | `Land target` | • `How many lands the deck should run. Fewer warns on the deck` • `Not set uses the format's usual count` |
| PriceTarget | `Price target` | Line: `A budget for the mainboard. Going over warns on the deck` |
| OtherTags | `Other tags` | Line: `Labels that do not change suggestions, like Budget or Jank` |
| TypeChart | `Types` | Line: `Mainboard cards by type` |
| RoleChart | `Roles` | Line: `Mainboard cards by role, like ramp or removal. Cards with no role are left out. Tap a role to see its tags` |
| ColorChart | `Colors` | Line: `Mainboard cards by color` |
| ManaCurve | `Mana curve` | Line: `Nonland mainboard cards by mana value` |
| ManaFulfillment | `Mana cost fulfillment` | • `Per color, the pips your lands and sources make against the pips your spells need` • `A check means you make at least as much as you need` |
| DrawOdds | `Draw odds` | • `Chance of at least one card from each group in the opening hand, or by a turn. ` `[-]` ` and ` `[+]` ` step the turn` • `Random draw only. Mulligans, tutors and card draw are not counted` |
| ColorMap | `Color map` | • `Each tile stands for a share of the colored cards across your decks` • `Every color grows out from the middle, and the color with the most cards claims the most ground` • `The shape is generated per player` • `Colorless cards are in the key below` |
| SuggestionRoleChart | `Card roles` | Line: `How many of your decks have a tag in each role. Tap a role for its tags` |
| SuggestionTagChart | `Oracle tags` | Line: `Your 10 most impactful tags, highest first. Tap a tag to see what it means` |
| SuggestionTagDecks | `Decks` | Line: `How many of your decks have this tag. The more decks, the higher its impact` |
| SuggestionTagCards | `Cards` | Line: `How many cards carry this tag across those decks. The more cards, the higher its impact` |
| SuggestionTagImpact | `Impact` | Line: `How much more your decks lean on this tag than Magic cards do overall. Your top tag scores 100` |
| SuggestionList | `Suggested commanders` | • `Commanders you haven't built, ranked against all your decks, best fit first` • `Tap a row to see why it fits. ` `[Swipe]` ` goes further down the same ranking` |
| SuggestionStaples | `Staples` | • `The cards in the most of your decks, with the share of your decks that run each` • `Basic lands are left out. Tap a Type or Color chip to narrow it` |
| Activity | `Activity` | • `Your swipes and searches each day over the last year, a darker square for a busier day` • `A lit square is one of your biggest days. Tap a square to see its counts` • `[Totals]` ` is what the cards dealt to your decks came to: ` `[Added]` ` and ` `[Maybeboard]` ` you kept, ` `[Skipped]` ` you passed on, and ` `[Removed]` ` you later took out` |
| RoleRates | `Card roles` | • `Your rate for each card role. Pick one under ` `[Show]` `: ` `[Added]` `, ` `[Skipped]` ` and ` `[Maybeboard]` ` are shares of the cards dealt to you, and ` `[Removed]` ` is the share of added cards you later took out` • `Tap a role to see the tags you swiped under it` • `From your decks with a commander, for roles dealt at least 10 times` |
| TagRates | `Oracle tags` | • `The tags where your rate stands out most from other players', with your own rate` • `A tag everyone passes on shows only after a lot of skips. Tap a tag to see what it means` • `From your decks with a commander, for tags dealt at least 10 times` |
| HardPasses | `Most skipped` | • `Cards you skip and never add or save, when other players usually keep them` • `A card everyone passes on shows only after a lot of skips. The tag counts yours, and an open row names the commanders you skipped it under` • `From your decks with a commander` |

`MAX_DECK_TAGS` is `zwipe_core::domain::deck::MAX_DECK_TAGS`. Note the Theme hint says "The last four are color blind modes" but the picker's last group is `High contrast & color blind` with six themes (four color-blind); copy as is.

### 2.3 One-time hint dialogs (`components/hint_dialog.rs`)

Mechanism:

- `use_one_time_hint(key)` at screen mount, or `open_and_record_hint(key, session, client, open)` from an effect once an async gate first passes (guarded by a local `fired` flag so it runs once per mount).
- If there is no session, or `session.user.has_seen_hint(key)`, nothing happens. Else open the dialog immediately (optimistic), then in the background: `ensure_fresh`, `client.mark_hint_shown(key: &str, session: &Session) -> Result<User, ClientError>` (`PATCH /api/user/hint`, authed, body `HttpMarkHintShown { hint: String }`), and on success replace `session.user` with the returned user, persist the session. Failure: logged only (the hint may auto-open once more later). No toast.
- Server-side key rule (`MarkHintShown::new`): trimmed, non-empty, ≤ 64 bytes, only `[a-z0-9_]`. Keys are not checked against a list; new keys need no server deploy.
- The screen's header `?` re-opens the same dialog any time (not recorded again).
- Rendering: `HintDialog` = AlertDialog with title, body in a `hint-body` block, optional extra actions, then a final dismiss button `Got it`.

Keys (`zwipe_core::domain::user::models::hints`), where they fire, and copy. `[X]` HintKey, `{X}` HintColored with its color var.

| Key | Fires | Title | Body |
|---|---|---|---|
| `first_login` | Home mount | `Welcome to Zwipe` | • `[Decks]` ` holds your decks` • `[Color map]` ` grows from your decks' colors. Tap Playstyle for suggested commanders` • `[Account]` ` has your email, password and log out` • `[Config]` ` sets your theme and which cards show` • `{Verify your email}`(--color-warning) ` in Account for full deck and card limits` |
| `profile` | Account mount | `Your account` | • `Tap a row's ` `[?]`(--palette-6) ` for what it does` • `[More]` ` deletes your account` |
| `config` | Config mount | `Config` | • `Set how Zwipe looks and which cards it shows` • `Tap a row's ` `[?]`(--palette-6) ` for what it does` |
| `universes_beyond_exceptions` | first time the Exceptions sheet opens in this mount | `Franchise exceptions` | Line: `Picked franchises still show while Universes Beyond is hidden` |
| `deck_list` | Deck list, once the list has at least one deck | `Your decks` | • `Tap a deck to open it` • `[Group by]` ` folds the list into sections, ` `[Show]` ` narrows it by color or tag` • `[Create]` ` starts a new deck` |
| `create_deck` | Create deck mount | `Building a deck` | • `{Profile}`(--accent-primary) ` is what the deck is, ` `{Budget}`(--accent-secondary) ` what it may cost, ` `{Tags}`(--accent-tertiary) ` what it does` • `Tags shape which cards we suggest` • `Tap a field's ` `[?]`(--palette-6) ` for what it does` |
| `edit_deck` | Edit deck mount | `Building a deck` | same as `create_deck` |
| `first_deck` | Deck view mount | `Welcome to your deck` | • `[Cards]` ` browses, adds and removes cards` • `[Edit]` ` changes the name, format, command zone, tags and targets` • `[More]` ` imports, exports, clones, shares or deletes the deck` • `Skipped and removed cards stay out of the add stack. ` `[More]` ` then Clear skips brings them back` • `Stats fill in as the deck grows. Tap a section to expand it` • `Warnings flag rule problems, each with a one-tap fix` |
| `deck_cards` | Deck cards view, once the deck has entries | `Browsing your deck` | Line: `Tap a row for details, quantity and printings`; • `Tap a ` `{keyword}`(--palette-1) ` chip for what it does, or a ` `{role}`(--palette-1) ` chip for its oracle tags` • `[Group by]` ` sorts by type, mana value, color or role` • `[Boards]` ` picks main, maybe or side` • `[Show]` ` toggles tokens, the command zone and row art` • `[Star]`(--color-warning) ` marks up to three MVPs. Suggestions lean toward cards that share their roles` |
| `add_deck_cards` | Add cards mount | `Swipe to build` | • `Swipe ` `{right}`(--color-success) ` adds the card` • `Swipe ` `{left}`(--color-error) ` skips it for a week` • `Swipe ` `{up}`(--color-warning) ` sends it to the maybeboard` • `Swipe ` `{down}`(--accent-tertiary) ` undoes the last swipe` • `[Synergy]` ` on starts with cards that work with your commander, then cards like your deck. Off shows every legal card`; Line: `Sort only reorders. It never hides cards` |
| `remove_deck_cards` | Remove cards mount | `Swipe to trim` | • `Swipe ` `{right}`(--color-success) ` removes the card` • `Swipe ` `{left}`(--color-error) ` keeps it` • `Swipe ` `{up}`(--color-warning) ` moves it between main and maybeboard` • `Swipe ` `{down}`(--accent-tertiary) ` undoes the last swipe`; Line: `The board chips up top pick which board you are trimming` |
| `swipe_select` | first time the Swipe select overlay opens in this mount | `Swipe select`, or `Suggested commanders` in suggested mode | Suggested: • `Swipe ` `{right}` ` starts a deck with that commander` • `Swipe ` `{left}` ` skips it for a week`; otherwise: • `Swipe ` `{right}` ` picks that {noun}` • `Swipe ` `{left}` ` skips it`; then both: • `Swipe ` `{up}` ` saves it to your commander maybeboard` • `Swipe ` `{down}` ` undoes the last swipe`. Suggested lines: `Best fit first, 100 deep. The eye shows why each one was suggested`, plus when a filter exists `[Filter]` ` narrows the stack to one color identity`; else line `Most-played first. ` `[Filter]` ` narrows by color or anything else`. noun: `oathbreaker` / `commander` / `partner` / `background` / `signature spell`. Colors as in add_deck_cards. |
| `filter` | first time the card filter sheet opens in this mount | `Filters` | • `Picks which cards show: the ones served to ` `{swipe}`(--accent-primary) `, or which of the deck's cards list` • `Open a section to set name, mana, type, color, ` `{Oracle tags}`(--accent-tertiary) ` and more. Stack as many as you want` • `[Apply]` ` uses it. ` `[Reset]`(--color-warning) ` then ` `[Apply]` ` goes back to the default view` • `Each screen keeps its own filter` |
| `import` | Import mount | `Importing cards` | • `Paste a decklist, or switch ` `[From]` ` to an Archidekt deck link` • `[Add]` ` keeps your current cards. ` `[Replace]`(--color-warning) ` swaps the board's contents` • `[Board]` ` picks where the cards go` • `Importing clears the deck's undo history` |
| `export` | Export mount | `Exporting your deck` | • `[Export]` ` picks which boards to include` • `[Copy]` ` puts the decklist on your clipboard` |
| `oracle_tag_dictionary` | Dictionary mount | `Oracle tags` | • `Tap a ` `[letter]`(--palette-1) ` to browse, or ` `{search}`(--accent-secondary) ` by name or description` • `[Examples]` ` shows real cards with that tag` • `[Use]` ` adds the tag where you came from` • `Some tags have no description yet` |
| `otag_examples` | Example cards mount | `Example cards` | • `Swipe ` `{left}`(--color-success) ` for the next card` • `Swipe ` `{down}`(--accent-secondary) ` to go back one` |
| `commander_maybeboard` | Commander maybeboard mount | `Commander maybeboard` | Line: `Commanders you save land here`; • `Swipe ` `{up}`(--color-warning) ` while picking a commander saves it here` • `[Swipe]` ` deals commanders: right starts a deck, up saves one` • `[Quick add]` ` searches by name` • `[Create deck]` ` starts a deck with that commander` • `[Remove]`(--color-error) ` takes it off the list` |
| `suggested_commanders` | Playstyle mount | `Playstyle` | Line: `Your colors, card roles and top oracle tags, read from every deck you own in any format`; • `[Create deck]` ` begins a deck with that commander` • `[Maybeboard]` ` saves it to your commander maybeboard` • `[Remove]`(--color-error) ` hides it for a week and pulls the next one up` • `[Swipe]` ` deals the same ranking: ` `{right}`(--color-success) ` starts a deck, ` `{up}`(--color-warning) ` saves, ` `{left}`(--color-error) ` skips it` • `[Show]` ` narrows the list to one color identity`; Line: `Below them, your top 10 commanders ranked against all your decks, best fit first. Swipe keeps going down the same ranking` |
| `activity` | Activity mount | `Activity` | Line: `How you pick, read from your swipes`; • `[Activity]` ` maps your swipes and searches over the last year, and what the cards dealt to your decks came to` • `[Card roles]` ` and ` `[Oracle tags]` ` show how often you add, skip, save or remove each one` • `[Most skipped]` ` lists cards you skip that other players usually keep`; Line: `Each section's ? explains it` |

### 2.4 Toasts (orangui `toast_stack.rs`, `toast_timing.rs`, `assets/toast.css`; dioxus_primitives toast)

- Kinds: `success`, `error`, `info`, `warning` (app calls `.success`, `.error`, `.info`; no `.warning` in this spec's screens). The message is the toast title; there is a small close `×` button.
- Durations (bands): `TOAST_QUICK = 3 s` (routine confirmations: saved, copied, added), `TOAST_NORMAL = 5 s` (worth reading; also the app-wide default), `TOAST_LONG = 8 s` (failures and anything naming a next step). The app mounts `ToastStack { default_duration: TOAST_NORMAL }`, `max_toasts = 3`, `timed_fade` off (the library removes a toast when its time is up).
- Look: fixed top-left over the header (left 1rem, top safe-area + 0.5rem), max width `min(16rem, 100vw - 2rem)`. Border color by kind: success `--border-success`, error `--border-error`, warning `--border-warning`, info `--border-primary`. Text centered, 0.9rem, letter-spacing 0.1em.
- Stack: newest first at the top. Collapsed by default: older toasts tuck behind the newest (margin-top -2.75rem, showing a sliver), message clamped to 2 lines with ellipsis. Tapping any toast toggles expanded (no overlap, 0.5rem gap, full message wrapped). Expand/collapse animates 0.2s only during a tap (a 250 ms `animating` window); new arrivals never animate the stack.
- Arrival animation: `toast-drop-in` 0.22 s ease-out, from opacity 0, translateY(-1.5rem), scale 0.96 to rest.
- Wording rules, as practiced in code: sentence case, no trailing period, short; success says what happened with the committed value when there is one (`Username changed to {name}`); errors from the server go through `to_user_message()` verbatim; duration band matches intent (QUICK confirmation, NORMAL readable result, LONG failure or next step).
- Every toast in this spec's screens is listed in its section; summary at the end of this file.

### 2.5 Action bars (orangui `action_bar.rs`, `button.rs`)

- `div.action-bar` pinned at the bottom of each screen and of each bottom sheet: a horizontal row of compact Action buttons (`.btn-action`; danger variant `.btn-action-danger`). Sheets add bottom safe-area padding.
- Buttons: `Primary` (`.btn`, full-width), `Small` (`.btn-xs`), `Action` (`.btn-action`); each with optional `danger` and `disabled`.

### 2.6 Bottom sheets (orangui `bottom_sheet.rs`)

- Backdrop (fades to `--bg-overlay` over 0.3 s ease; tap = dismiss) and a sheet sliding up from the bottom (transform 0.3 s ease; max height 85vh; rounded top corners). First render has transitions off (50 ms premount) so it does not slide on insert.
- Structure: header (title centered, `--accent-tertiary`; optional corner `?` if a hint signal is given), scrollable content (column, gap 0.5rem), action bar footer. Default footer is a single `Close` button when the sheet gives none.
- Dismiss (backdrop tap or OS back): calls `on_dismiss` if provided (sheet then closes itself), else closes. Registers on the overlay back stack (OS back closes the topmost overlay first).
- The sheet carries its own still GridCells backdrop at 0.8 opacity under the content.

### 2.7 AlertDialog (orangui `alert_dialog.rs`)

- Drawn once at the app root (`DialogHostView`); one dialog at a time. Overlay dim `--bg-overlay`; dialog: title (`h2`), rule, body, rule, actions row. Fades in 150 ms ease-out and out 150 ms ease-in (opacity only), then unmounts.
- Actions: `Dismiss(label)` (cancel style; closes), `Confirm { label, danger, on_confirm }` (closes, then runs the callback), `Link { label, href }` (opens link, closes), `Button(ActionButton)` with optional live label, live disabled flag, and run mode: `on_click` (close then run), `stay_open` (run, stay), `busy(busy_label, async work -> bool)` (shows busy label, disables all actions and the overlay tap while running; closes only when work returns true).
- Overlay tap = dismiss unless a busy action is running. OS back closes it.

### 2.8 Loading skeletons, spinners, empty/error notes

- Skeleton bars: `.skeleton-bar` height 0.65rem, radius 0.375rem, fill `--border-muted`, pulsing `skeleton-pulse` 2 s cubic-bezier(0.4,0,0.6,1) infinite: opacity 1 → 0.4 at 61.8% → 1. Rule from the user's notes: skeletons ghost everything, no real text, chips or controls inside them.
- Skeleton container entrance `skeleton-enter` 250 ms opacity 0 → 1 (card skeletons).
- Spinner: `.spinner` 1.25rem × 1.75rem bordered box rotating 360° in 0.5 s linear infinite (used on Forgot password).
- Load-failure note: `span.chip-note` with a `Could not load ...` sentence (Changelog: `Could not load the changelog`).

### 2.9 Animations and transitions, with timing

App-wide (in scope here):

- Splash (orangui `splash.rs`, shown once at app start over the first screen; children = the ZWIPE logo in `Decode`): `MARK_AT_MS = 400` (mark mounts and starts decoding, 850 ms), `LEAVE_AT_MS = 1350` (mark evaporates over `EXIT_MS = 450`), `OUT_AT_MS = 1800` (overlay fades, opacity transition 0.3 s; the root `SplashGate` opens here, which starts the first screen's Decode and CountUp), `FADE_MS = 300` then unmount at 2100 ms. `pace` = 100 (percent). Under the mark, the overlay's GridCells rain in.
- GridCells backdrop (`grid-cells.js`): every grid cell (size `--grid-size`, default 28px) a rounded square with a seeded shade from `[0.015, 0.03, 0.05, 0.075, 0.1]` alpha of the text color, brightest at top fading down. Rain: each cell lands at `row * 18 ms + (rand >>> 4) % 160 ms`, falls 3 cells over 350 ms with ease-out-cubic. The shell's copy is `still` (already in place). Theme change redraws without raining.
- Decode (orangui `entrance.rs::Decode`; logo on splash and HomeHero): starts from all blanks (spaces keep the shape). `DECODE_MS = 850`, `DECODE_TICK_MS = 40` → `frames = ceil(850/40) = 22`; frame k = 1..=22 after each 40 ms sleep. Noise front `n = min((cols+1) * k*100 / (frames*45), cols+1)` (noise covers the art in the first 45%); decode front `d = (cols+4) * max(0, k*100 - frames*35) / (frames*65)` (starts at 35%, overruns by 4 columns). Per non-space glyph: `reach = column + wobble`, wobble = next xorshift % 4; `reach < d` → glyph; `reach >= n` → space; else noise glyph from `['░','▒','▓','▎','▍']` picked by next xorshift % 5. RNG: `state = (frame as u64).wrapping_mul(0x9e3779b97f4a7c15) | 1`, then per draw `state ^= state<<13; state ^= state>>7; state ^= state<<17`. After the last frame, the exact art. Leaving (splash only): the reverse (`encoded`), art → noise → blank over 450 ms. Reduced motion: settled at once. Waits on the SplashGate.
- CountUp rolling counters (orangui `CountUp`, inside `StatsStrip`): `COUNT_MS = 1000`, `COUNT_TICK_MS = 16` → 63 frames. While the value is `None`: every 16 ms show a random 4-digit number `1000 + noise(seed + tick) % 9000`. Once a value exists: for frame f = 1..=63, `t = f/63`, `spread = (1-t)^2`, `swing = (noise(f) % 2001)/1000 - 1` (in -1..1; note the landing phase seeds on the frame number only, not the per-counter seed), shown = `round(max(0, v + v*spread*swing))` capped at the largest number with the same digit count as v (`digit_ceiling`), then exactly v. Before any frame: four U+2007 figure spaces. Each CountUp has its own seed (global counter × 2654435761, wrapping) used while waiting, so neighbors don't flicker in step. `noise(x)`: `s = x.wrapping_mul(0x9e3779b97f4a7c15) | 1; s ^= s<<13; s ^= s>>7; s ^= s<<17`. Compact print: under 10,000 → with thousands separators (`2,065`); else one decimal, truncated, with `k`/`m`/`b` (`28.0k`, `1.2m`, `2.5b`). Rolls on every mount, waits on the SplashGate.
- Screen entrance `.content-enter`: 0.3 s ease-out, opacity 0 → 1 and translateY(0.625rem) → none. `.content-enter-delayed` adds a 0.15 s delay. Every screen's content column uses it. There is no route transition beyond this.
- Chip entrance `.chip-in`: 0.3 s ease-out, translateY(0.35rem). The Config exception chips stagger `animation-delay: i * 45 ms`.
- Hero scope label / Live chip: `hero-scope-in` 0.25 s ease-out, translateY(2px) → none.
- Theme wipe (orangui `use_theme_follow`): any theme change sweeps the new theme over the old via a view transition, 0.4 s cubic-bezier(0.65,0,0.35,1). At phone widths (< 60rem) it runs top to bottom; light → dark runs bottom to top. Wide screens: left to right, light → dark right to left. Reduced motion: instant.
- Bottom sheet slide 0.3 s, backdrop 0.3 s; AlertDialog fade 150 ms; toast drop-in 0.22 s, stack expand 0.2 s.
- Collapsible (`.collapsible`, used for the Exceptions sheet's "N selected" row): grid rows 0fr → 1fr and opacity 0 → 1 over 0.25 s ease.
- Update-required headline pulse 1.6 s; skeleton pulse 2 s; spinner 0.5 s.
- Changelog cards `changelog-card-in` 0.35 s ease-out, translateY(8px) → 0, replayed on every filter change (cards are re-keyed).
- Account countdowns: resend cooldown ticks every 1 s from 60; delete countdown every 1 s from 5.

Elsewhere in the app (other specs own the screens; listed so nothing is missed): swipe card enter 0.2 s ease-out per direction and exit 0.18 s; `row-enter` 0.2–0.3 s (maybeboard rows staggered 45 ms, first at 320 ms); `dict-row-in` 0.18 s; `flavor-enter` 0.2 s (+0.1 s delay) and 0.3 s (+0.15 s); `deck-featured-deal` 0.78 s starting at 0.8 s, staggered 180 ms; `dropdown-enter` 0.3 s; image preview drop-in 0.3 s / out 0.2 s; accordion open 300 ms, chevron rotate 150 ms; `search-float-in` 0.25 s then 3.2 s idle drift; color-map ghost bloom 2.6 s; chart line draw 1.1 s; chart tip 0.18 s; carousel snap 300 ms.

---

## 3. Account (`/user`, `screens/account/mod.rs`)

Reached from Home (bar button `Account`). Telemetry screen `Profile(Main)`.

On mount: one-time hint `profile` (2.3). Also on every open: `client.fetch_user(&session) -> Result<User, ClientError>` (`GET /api/user`, authed; called with the CURRENT session directly, not via ensure_fresh) → on success replace `session.user` (in memory only, not persisted); on failure log only (no toast).

Top to bottom:

1. `ScreenHeader { title: "Account", hint }` (`!` left, `?` right opens `Your account`).
2. Content (`content-enter`), only when a session exists, two settings lists:
   - List header `Account`:
     - Row `Username` | value `{username}` + Action `Change` → opens Change username sheet.
     - Row `Email` + `?` (topic Email) | value `{email}` + badge `Verified` (green, success color) or `Unverified` (red, error color) + Action `Change` → Change email sheet.
     - Row `Pending` (only when `user.pending_email` is set) | value column, right-aligned: `{pending_email}` and under it, muted small, the link note when `pending_email_expires_at` is set (see below).
     - Row `Verification` + `?` (topic Verification) (only while `email_verified_at` is None) | Action `Resend` (live label, see below) + a refresh-icon Action button (circular arrows SVG, class `btn-action-eye`) that checks verification.
     - Row `Password` | value `•••••••` (7 bullets) + Action `Change` → Change password sheet.
   - List header `About`:
     - Row `Website` + `?` (topic Website) | Action `zwipe.net ↗` → opens `WEB_BASE` externally.
     - Row `Privacy policy` | Action `View` → push `/privacy`.
     - Row `Version` + `?` (topic Version) | value `v{APP_VERSION}` + Action `Changelog` → push `/changelog`.
3. `ActionBar`: `Back` (go back), `Log out` (danger) → Log out dialog, `More` → More sheet.

Pending note (`link_expiry_text(expires_at, now)`): `left = expires_at - now`. If `left <= 0`: `Link expired; tap Change to send a new one`. Else `minutes = ceil(left_seconds / 60)`. If `minutes < 60`: `Link expires in 1 minute` or `Link expires in {n} minutes`. Else `hours = (minutes + 30) / 60` (integer): `Link expires in 1 hour` / `Link expires in {n} hours`. Computed at render (no ticking timer).

Verification row behavior (`components/email_verification.rs`):

- `RESEND_COOLDOWN_SECS = 60` (matches the server's resend rate limit, burst 1 then 1/60 s per user).
- Resend button label: `Sending...` while the request is out; else `Resend in {n}s` while cooldown > 0; else `Resend`. Disabled while sending or cooling down.
- Tap: set cooldown to 60 immediately (optimistic). `ensure_fresh`; on error: telemetry, `toast.error(to_user_message, TOAST_LONG)`, cooldown 0, stop. Then `client.resend_verification(&session) -> Result<(), ClientError>` (`POST /api/auth/resend-verification`, authed):
  - Ok → `toast.success("Verification email sent", TOAST_NORMAL)`.
  - `TooManyRequests` → `toast.info("Please wait a moment", TOAST_NORMAL)`, keep the cooldown.
  - Other error → telemetry, `toast.error(to_user_message, TOAST_LONG)`, cooldown 0, stop.
  - After Ok/429: count the cooldown down by 1 every 1 s to 0.
- Check button: disabled while checking. Facade `run_at(EMAIL_VERIFICATION, "check_verified", fetch_user)`. On success replace `session.user`; if verified `toast.success("Email verified", TOAST_NORMAL)` (the badge flips and the Verification row disappears), else `toast.info("Not verified yet. Check your inbox", TOAST_NORMAL)`. Failure: facade error toast.

### 3.1 More sheet

`BottomSheet` title `More`; content: one full-width danger Primary button `Delete account` → opens the Delete account dialog (the sheet stays open underneath). Footer: default `Close`.

### 3.2 Log out dialog (`components/logout_dialog.rs`)

AlertDialog title `Log out`, body `Are you sure you want to log out?`, actions `Cancel` (dismiss), `Log out` (danger confirm) → `session.logout(client)` (0.7). Afterwards the auth gate shows Login. No toast.

### 3.3 Delete account dialog (`components/delete_account_dialog.rs`)

- AlertDialog title `Delete account`; body text `This will permanently delete your account, all decks, and all card data. This cannot be undone.` followed by a password `TextInput` (no label, placeholder `Password`, masked with Show/Hide). The password clears each time the dialog opens.
- Actions: `Cancel` (dismiss), then a danger busy ActionButton. `DELETE_COUNTDOWN_SECS = 5`: each time the dialog opens, a countdown starts at 5 and decrements every 1 s; closing or reopening cancels and restarts it. Label `Delete ({n})` while n > 0, disabled; at 0 label `Delete`, enabled. It is NOT disabled for an empty password (the server rejects).
- Tap Delete: busy label `Deleting...`, all actions and overlay tap disabled. Facade `run_at(DELETE_ACCOUNT_DIALOG, "delete_account", |c, s| c.delete_user(HttpDeleteUser { password: Secret::new(password) }, &s))` → `ZwipeClient::delete_user(&self, request: HttpDeleteUser, session: &Session) -> Result<(), ClientError>` (`DELETE /api/user/delete-user`, authed, body `{"password": "..."}`).
  - Success: `session.logout(client)` (attempts server logout with 5 s cap, then clears) and the dialog closes; gate shows Login. No toast.
  - Failure: facade error toast (5 s), dialog stays open with the button re-enabled.

### 3.4 Change username sheet (`change_username.rs`)

- `BottomSheet` title `Change username`. Fields: `TextInput` label `New username`, placeholder `New username`, id `new_username`, error line; `TextInput` label `Password`, placeholder `Password`, id `password`, type password, error line.
- Footer: `Back` (closes), `Save changes` (`Saving...` while out). Both disabled while out.
- Every open resets both fields, errors, touched flags and the attempt flag.
- Validation (live after touched or attempted): username via `Username::new` (rules in 1.2); password non-empty else `Password is required`.
- Submit: `HttpChangeUsername::new(&new_username, &password)` (raw, untrimmed); facade `run("change_username", c.change_username(request, &s))` → `ZwipeClient::change_username(&self, request: HttpChangeUsername, session: &Session) -> Result<User, ClientError>` (`PATCH /api/user/change-username`, body `{ new_username, password }`).
- Success: `session.user.username = updated.username`; `toast.success("Username changed to {new_name}", TOAST_QUICK)`; clear inputs; close. Failure: facade toast; sheet stays open with input kept.

### 3.5 Change email sheet (`change_email.rs`)

- Title `Change email`. Fields: `New email` (placeholder `New email`, id `new_email`, type text, error line), `Password` (password, error line). Footer `Back`, `Save changes` / `Saving...`. Reset on every open.
- Validation: `Email::new(new_email)` (untrimmed) with the user-facing email messages (1.2); password `Password is required`.
- Submit: `HttpChangeEmail::new(&new_email, &password)`; facade `run("change_email", ...)` → `ZwipeClient::change_email(&self, request: HttpChangeEmail, session: &Session) -> Result<User, ClientError>` (`PATCH /api/user/change-email`, body `{ email, password }`). The answer is the same whether or not the address is free.
- Success: replace the whole `session.user` with the response (so `Pending` appears); `toast.success("Check {typed address} for a link to confirm the change", TOAST_NORMAL)` (uses the typed address, not the response); clear; close. Failure: facade toast.

### 3.6 Change password sheet (`change_password.rs`)

- Title `Change password`. Fields: `Current password` (placeholder `Current password`, id `current_password`, password, NO validation and no error line), `New password` (placeholder `New password`, id `new_password`, password, error line, `?` → topic Password), `Confirm password` (placeholder `Confirm new`, id `confirm_password`, password, no error line). Footer `Back`, `Save changes` / `Saving...`, both disabled while out. Reset on every open.
- Validation of the new password (live once new or confirm has been non-empty, or after an attempt), first failure wins: policy (`password::validate`, messages in 1.2) → `Passwords do not match` (only once attempted or confirm length ≥ new length) → `New password must be different from your current password`.
- `Save changes`: mark attempted, validate; if clean open the confirm AlertDialog: title `Change password`, body `Changing your password will log you out on all other devices.`, actions `Cancel` (dismiss), `Confirm` (non-danger) → re-validate, then `HttpChangePassword::new(&current, &new)`; facade `run("change_password", ...)` → `ZwipeClient::change_password(&self, request: HttpChangePassword, session: &Session) -> Result<(), ClientError>` (`PATCH /api/user/change-password`, body `{ current_password, new_password }`).
- Success: `toast.success("Password change successful", TOAST_QUICK)`; clear; close. This device stays signed in. Failure (for example wrong current password): facade toast, sheet stays.

### 3.7 Account navigation

In: Home's `Account` button. Out: `Back` (go back), `View` → Privacy policy, `Changelog` → Changelog, `zwipe.net ↗` → browser, logout/delete → Login via the gate.

### TUI today (Account)

What exists (`inbound/app/account.rs`, `inbound/ui/account.rs`, `inbound/event_loop/settings.rs`, `domain/account/mod.rs`):

- Opened with `a` from Home; shows the last-known user and fetches `fetch_user` via `fresh()`; on failure sets the status line to the error (app: silent).
- Rows (selectable with j/k, Enter opens): Username (`Change`), Email with `Verified` (green) / `Unverified` (warning color; app uses red/error) and `Change`, Pending + note, `Verification` row with `Resend` / `Resend in {n}s`, a separate `Check` row, Password `•••••••` `Change`, About with `Website` (plain text of `WEB_BASE`, not openable) and `Version v{x}`, then `Log out` and `Delete account` rows.
- Forms open as centered panels (title, fields, error line, hints `Tab next field  Enter Save changes|Delete (n)|Delete  Esc back`; busy `Saving...` / `Deleting...`). Form titles match the app. Field labels match except Change password's third field label `Confirm password` (same) and placeholder concept (none in TUI).
- Validation only on submit (app validates live as you type once touched). Order and strings match `zwipe-core`. Username and email are trimmed before validation and sending (app sends raw).
- Change password asks with a confirm panel `Change password` / `Changing your password will log you out on all other devices.` keys `y Confirm  n cancel`. Log out panel `Log out` / `Are you sure you want to log out?` keys `y Log out  n cancel`.
- Delete form: intro text matches; 5 s delay computed from `opened_at` (not reset on reopen beyond creating a new form, which is equivalent); success forgets the session locally WITHOUT calling logout and shows status `Account deleted` on Login (app: no message, and it calls logout).
- Results go to a one-line status, not toasts. Messages match the app's toast strings (`Username changed to {name}`, `Check {email} for a link to confirm the change`, `Password change successful`, `Verification email sent`, `Please wait a moment`, `Email verified`, `Not verified yet. Check your inbox`). Status shows `Sending...`, `Checking...`, `Logging out...` (app has none of these).
- Errors on account changes appear inline in the form (app: toast).

Gaps and differences:

- No `!` help button, no header `?`, no one-time `profile` hint, no row `?` info buttons (Email, Verification, Website, Version).
- No Privacy policy row/screen, no Changelog button/screen, Website is not actionable.
- No More sheet: Delete is a row (app hides it under `More` → `Delete account`). Log out is a row (app: action bar button, danger).
- Pending note math differs from the app: TUI uses seconds (`< 3600 s` → minutes; hours = `(s + 1800) / 3600`); app uses `minutes = ceil(s/60)`, `< 60 minutes` → minutes, hours = `(minutes + 30) / 60`. Example: 3599 s → app `Link expires in 1 hour`, TUI `Link expires in 60 minutes`; 89 min 30 s → app `2 hours`, TUI `1 hour`. Port `link_expiry_text` exactly.
- Resend: app reports refresh failures and send failures with an 8 s error toast and resets the cooldown; TUI resets the cooldown and shows the error in status (fine), but its `Sending...` status is extra.
- Delete: the app keeps the dialog open with an error toast on failure; TUI shows the error inline (acceptable). App calls logout after delete; TUI doesn't.
- Unverified badge color: app red (`--color-error`), TUI warning.
- No live label `Delete ({n})` on a button per se; TUI puts it in the hints line (acceptable mapping).

---

## 4. Config (`/config`, `screens/config/mod.rs`)

Reached from Home (bar button `Config`). Telemetry screen `Profile(Preferences)`.

On mount: one-time hint `config`. Also once: facade `run_quiet("get_preferences", c.fetch_preferences(&s))` → `ZwipeClient::fetch_preferences(&self, session: &Session) -> Result<UserPreferences, ClientError>` (`GET /api/user/preferences`). On success: set `ub_hide = exclude_universes_beyond`, `ub_exceptions = universes_beyond_exceptions`, `ub_loaded = true`, `pip_style = prefs.pip_style`. On failure: quiet (no toast); the two Universes Beyond controls stay disabled for the life of the screen.

Top to bottom:

1. `ScreenHeader { title: "Config", hint }` (`?` opens `Config` hint).
2. Content (`content-enter`), two settings lists:
   - Header `Appearance`:
     - Row `Theme` + `?` (topic Theme) | value `display_theme_name(theme.name)` (from the live app theme, not the server) + `DarkModeButton` (Action button showing the moon icon + `Dark` when dark, sun icon + `Light` when light) + Action `Change` → Themes sheet.
     - Row `Pips` + `?` (topic Pips) | Action button labeled with the current style: `Theme` or `Natural` (tap toggles).
     - Row `App icon` (only where the platform supports it; skip for the TUI).
   - Header `Cards`:
     - Row `Universes Beyond` + `?` (topic UniversesBeyond) | Action button labeled with the current state: `Hide` when hidden, `Show` when shown (tap toggles). Disabled until preferences load.
     - Row `Exceptions` + `?` (opens the screen-owned `Franchise exceptions` HintDialog, NOT via HintHost) | Action `Change` → Exceptions sheet, disabled unless loaded AND hidden. The whole row is dimmed (opacity 0.45, no pointer) while Universes Beyond is shown. When the saved list is non-empty, a wrapped row of muted chips under it: franchise display names (`zwipe_core::domain::card::scryfall_data::universe::franchise_by_slug(slug).name`, unknown slugs dropped), sorted alphabetically, each `chip-in` with delay `i * 45 ms`.
3. `ActionBar`: `Back` only.

`display_theme_name(slug)`: special cases `rose-pine` → `Rosé Pine`, `vscode` → `VS Code`, `github` → `GitHub`, `github-high-contrast` → `GitHub High Contrast`, `synthwave-84` → `Synthwave '84`, `powershell` → `PowerShell`, `docs-rs` → `docs.rs`; else split on `-` and capitalize each word's first letter (`kanagawa-dragon` → `Kanagawa Dragon`, `one-dark` → `One Dark`).

Toggles (all optimistic, all share this shape): flip the shown value at once; number the request (`RequestSeq`, per toggle); send `update_preferences`; when the reply arrives, act only if no newer tap went out since (`settle`): success → show the server's value and toast; failure → put back the value from before THIS tap (facade already toasted the error). Toggle toasts are success, TOAST_NORMAL.

- `ZwipeClient::update_preferences(&self, request: HttpUpdatePreferences, session: &Session) -> Result<UserPreferences, ClientError>` (`PATCH /api/user/preferences`). `HttpUpdatePreferences { theme: Option<String>, dark_mode: Option<bool>, exclude_universes_beyond: Option<bool>, universes_beyond_exceptions: Option<Vec<String>>, pip_style: Option<PipStyle> }`; absent = unchanged; exceptions replace the whole list.
- Dark mode (`DarkModeButton`): next = same theme name, `is_dark` flipped; theme applied at once (wipe). Request `{ theme: Some(name), dark_mode: Some(next) }` via `try_run`. Success: theme = `theme_from_preferences(reply)`; toast `Dark mode on` or `Dark mode off` (by the reply's `dark_mode`). Failure: revert theme.
- Pips: request `{ pip_style: Some(next) }`. Success: show reply's style; also write it into `session.preferences.pip_style` and persist the session; toast `Pips in theme colors` (Theme) or `Pips in natural colors` (Natural). Failure: revert. Pip style is app-wide (mana symbols drawn in theme colors vs classic fills).
- Universes Beyond: request `{ exclude_universes_beyond: Some(next) }`. Success: toast `Universes Beyond hidden` or `Universes Beyond shown`. Failure: revert.

### 4.1 Themes sheet (`preferences.rs` + orangui `ThemeSheet`)

- `BottomSheet` title `Themes`, no hint. Content: for each group in `THEME_GROUPS` order, a section label then one row per theme. Row: button with the display name on the left and six swatch dots on the right (`--palette-1` … `--palette-6` of that theme in the CURRENT mode); selected row highlighted. Groups (labels and slugs, verbatim order):
  - `Warm & earthy`: darcula, flexoki, gruvbox, kanagawa-dragon, melange, miasma, rustbox, zenburn
  - `Cool & blue`: ayu, bluloco, kanagawa, night-owl, nightfox, nord, one-dark, poimandres, powershell, solarized, tokyo-night
  - `Green & teal`: everforest, hackerman, osaka-jade, terafox
  - `Purple & pink`: catppuccin, dracula, rose-pine, synthwave-84
  - `Black & gray`: docs-rs, eclipse, github, matte-black, neovim, oxocarbon, vantablack, vscode
  - `Bright & colorful`: ethereal, material, monokai, monokai-pro, monokai-pro-machine, monokai-pro-octagon, monokai-pro-ristretto, monokai-pro-spectrum, xcode
  - `High contrast & color blind`: achromatopsia, deuteranopia, github-high-contrast, modus, protanopia, tritanopia
- Retired slugs map: `ristretto` → `monokai-pro-ristretto`, `visual-studio` → `vscode`.
- No dark/light control inside the sheet; the mode is the one in use at open.
- On open: snapshot the live theme (original), select its name.
- Tapping a row: select it and set the app's live theme to `{ name, is_dark: current mode }` at once (live preview via the wipe).
- Footer: `Back`, `Save` (disabled while the selection equals the original).
- Save: close the sheet, then `update_preferences({ theme: Some(name), dark_mode: Some(is_dark) })` via facade `run`. Success: theme = `theme_from_preferences(reply)`; `toast.success("Theme saved", TOAST_QUICK)`. Failure: facade error toast; the previewed theme stays applied (the app does NOT revert on a failed save).
- Back / backdrop / OS back: if nothing was picked, just close. If a pick is on screen: restore the original theme (it wipes back in), the sheet holds still and disappears inside the wipe, and once the sweep finishes `toast.info("Theme unchanged", TOAST_QUICK)`.

### 4.2 Exceptions sheet (`universes_beyond.rs`)

- `BottomSheet` title `Exceptions`, header `?` opens the `Franchise exceptions` hint. First open in this mount triggers the one-time hint `universes_beyond_exceptions`.
- Data: the franchise catalog from the startup cache, `client.fetch_ub_franchises() -> Result<Vec<UbFranchiseView>, ClientError>` (`GET /api/card/ub-franchises`, no auth; `UbFranchiseView { slug, name }`), already sorted by the server; ensured on mount. No compiled fallback. Loading or failed: no chips at all (no message, no toast).
- On each open: draft = copy of the saved list.
- Content top to bottom: a collapsible row (open only while the draft is non-empty, 0.25 s): label `{n} selected` and a clear button `×` (U+00D7) that empties the draft. Then a centered wrap of chips, one per franchise, label = `name`, `selected` style when in the draft; tap toggles (append on select, so order is tap order).
- Footer: `Back` (discard), `Save` (disabled when the draft equals the saved list as a set, ignoring order).
- Discard (Back, backdrop, OS back): if the draft differs from saved as a set, `toast.info("Exceptions unchanged", TOAST_QUICK)`; close.
- Save: close at once, then facade `run("update_preferences", { universes_beyond_exceptions: Some(draft) })`. Success: saved list = reply's `universes_beyond_exceptions`; `toast.success("Exceptions saved", TOAST_QUICK)`. Failure: facade error toast; saved list unchanged (not optimistic).

### 4.3 Config navigation

In: Home `Config`. Out: `Back`.

### TUI today (Config)

What exists (`inbound/app/config.rs`, `inbound/ui/config.rs`, `inbound/event_loop/settings.rs`):

- Opened with `c` from Home. Whole body shows `Loading...` until `fetch_preferences` answers (app shows Theme/Pips at once from local state and only disables the UB controls).
- Rows: Appearance: `Theme` (display name), `Mode` (`Dark`/`Light`, its own row), `Pips` (`Theme`/`Natural`); Cards: `Universes Beyond` (`Hide`/`Show`), `Exceptions` (comma-joined sorted names or `None`, dimmed when shown). Enter/Space activates. Esc home, q quit.
- Optimistic changes with a single sequence counter for all toggles (app keeps one per toggle) and revert-to-last-confirmed on failure (app reverts to the value before that tap). Status line messages match the app's toast strings (`Theme saved`, `Dark mode on/off`, `Pips in theme colors/natural colors`, `Universes Beyond hidden/shown`, `Exceptions saved`, `Theme unchanged`, `Exceptions unchanged`).
- Theme picker: centered list titled `Themes` with group headers (muted bold) and theme names; j/k previews live; Enter saves when changed (closes silently when unchanged); Esc/q restores and says `Theme unchanged` if moved. Hints `j/k preview  Enter save  Esc back`.
- Exceptions editor: list `[x]`/`[ ]` per franchise, title `Exceptions` or `Exceptions: {n} selected`, Space pick, x clear, Enter save (only if changed), Esc/q back with `Exceptions unchanged` if changed. Fetches franchises on open and shows `Loading...` until then.

Gaps and differences:

- No `!` / `?` header buttons, no one-time `config` hint, no `universes_beyond_exceptions` hint, no row `?` info dialogs (Theme, Pips, Universes Beyond, Franchise exceptions).
- The app has no separate Mode row; the mode toggle is a button inside the Theme row showing icon + `Dark`/`Light`. A TUI can keep a key for it but the row text should match (`Theme  {Name}  [☾ Dark]  [Change]`).
- No swatches in the theme picker (app shows six palette dots per row).
- App theme save failure keeps the previewed theme; TUI reverts. Exceptions save in the app is not optimistic; TUI shows the new list before the server answers.
- Exceptions row with nothing picked: app shows nothing (no `None`).
- On preferences fetch failure the app leaves UB controls disabled silently; TUI's Config stays on `Loading...` forever (it ignores `PreferencesUnavailable`), which blocks theme and pips too.
- No toasts (status line).

---

## 5. Privacy policy (`/privacy`, `screens/legal/privacy_policy.rs`)

- `ScreenHeader { title: "Privacy policy" }` (no `?`).
- Content (`content-enter`, `privacy-content`), a column of section cards:
  1. First card: heading `Privacy policy` with a chip `Last updated {PRIVACY_LAST_UPDATED}` (currently `Last updated October 2026`), a rule, then `PRIVACY_INTRO_HTML` (one paragraph: `Zwipe is a Magic: The Gathering deck builder for mobile. This policy describes what data we collect, how we use it, and your rights.`, with "Magic: The Gathering" linking to `https://magic.wizards.com/en`).
  2. `PRIVACY_POLICY_HTML` split on `<h2>`: one card per section, each `<h2>` heading followed by an inserted `<hr>`, then the section's HTML. Current sections, in order: `Data we collect`, `How we use your data`, `Third-party services`, `Data retention`, `Where your data lives`, `Security`, `Children`, `Fan content`. The HTML uses `<p>`, `<ul>/<li>`, `<strong>` and `<a href>` (about 20 tags); a TUI must render it to styled text (headings, bullets, bold, links shown as text with URL).
  3. Last card: heading `Contact`, rule, `Questions or requests? Email ` + link `support@zwipe.net` (opens `mailto:support@zwipe.net`) + ` or join the ` + link `Discord` (opens `https://discord.gg/s2UReqUUeg`) + ` for support.`
- Source constants: `zwipe_core::legal::{PRIVACY_INTRO_HTML, PRIVACY_LAST_UPDATED, PRIVACY_POLICY_HTML}`, `zwipe_core::domain::site::{SUPPORT_EMAIL, DISCORD_URL}`. Compiled in; no network, no loading/error states.
- `ActionBar`: `Back`.
- In: Account → `Privacy policy` → `View`. Out: `Back`.

### TUI today (Privacy policy)

Not implemented. Needs a scrollable screen rendering the three parts above from the `zwipe-core` constants (HTML to text), with links printed (no browser required) and Back.

---

## 6. Changelog (`/changelog`, `screens/changelog.rs` + `components/src/changelog.rs`)

- `ScreenHeader { title: "Changelog" }` (no `?`).
- Content (`content-enter`, `changelog-screen`) by the root `ChangelogCache` (fetched once at startup, 0.5):
  - `Loading`: skeleton: a wrap of 8 chip-shaped bars (2.75rem × 1.5rem), then 3 card skeletons, each with a version bar (3rem × 1rem) and a date bar (5rem), then 5, 3 and 4 bullet bars (widths cycling 100%, 82%, 90%). All pulse.
  - `Failed`: `span.chip-note` `Could not load the changelog`. No retry, no toast.
  - `Loaded(HttpChangelog)`: the shared component below.
- `ActionBar`: `Back`.
- Data: `HttpChangelog { upcoming: Vec<HttpRelease>, releases: Vec<HttpRelease> }`, `HttpRelease { version: String, date: String, released_on: Option<NaiveDate>, entries: Vec<String> }`. A release's day = `released_on`, else `date` parsed with `DISPLAY_DATE_FORMAT = "%b %-d, %Y"` (for example `Oct 9, 2026`), else unknown.

Component behavior:

- Listed order: all upcoming releases first, then shipped releases (newest first, as served).
- Place of each release: year (`%Y`), month key (`%Y-%m`), month label (`%b`, for example `Oct`). An upcoming release with no day takes the newest shipped release's place. A release with no readable day goes under `Earlier` (year and month both `earlier`, label `Earlier`).
- Chip rows (filters), each a row of `chip` buttons, lit = `chip selected`:
  - Row 1: `Latest`, then each distinct year newest first, then `Earlier` last if any release is there.
  - Row 2 (when a year is selected): that year's month labels, newest first.
  - Row 3 (when a month is selected): distinct minor versions in that month (`1.12` for 1.12.x; a version without a second dot is its own minor).
  - Clicking a chip selects it (clearing deeper levels). Clicking the deepest lit chip goes up one level (minor → month → year → Latest).
- Visible cards: Latest = every upcoming release plus the first shipped one; Year = releases in that year; Month = in that month; Minor = in that month with that minor.
- Card: `h2` version (`1.15.0`), date text as served (`date`; may be empty for upcoming), then a tag: `Upcoming` (status-doing) for upcoming entries, `Latest` (status-done) for the first shipped entry, none otherwise. Then a bullet list of `entries` verbatim. Cards ease in 0.35 s on mount and again whenever the filter changes.
- Starts on `Latest` on every mount.
- In: Account → `Version` row → `Changelog`. Out: `Back`.

### TUI today (Changelog)

Not implemented. No startup changelog fetch either. Needs: background `fetch_changelog()` at launch (logged in or not), a cache with Loading/Loaded/Failed, a screen with the three chip rows (keyboard: move between chips, Enter selects, selecting the lit deepest chip goes up), the release cards, a skeleton while loading, and the failure note.

---

## 7. Every toast in these screens

| Where | Kind | Duration | Text |
|---|---|---|---|
| Login, empty field | error | NORMAL | `Enter your username or email and password` |
| Login failure | error | NORMAL | `to_user_message()` |
| Register success | info | NORMAL | `Check your inbox to verify your email, then log in` |
| Register failure | error | NORMAL | `to_user_message()` |
| Forgot password failure | error | NORMAL | `to_user_message()` |
| Any facade call failure | error | NORMAL | `to_user_message()` |
| Resend sent | success | NORMAL | `Verification email sent` |
| Resend 429 | info | NORMAL | `Please wait a moment` |
| Resend other failure / refresh failure | error | LONG | `to_user_message()` |
| Check verified, yes | success | NORMAL | `Email verified` |
| Check verified, no | info | NORMAL | `Not verified yet. Check your inbox` |
| Username changed | success | QUICK | `Username changed to {username}` |
| Email change requested | success | NORMAL | `Check {typed email} for a link to confirm the change` |
| Password changed | success | QUICK | `Password change successful` |
| Dark mode toggle | success | NORMAL | `Dark mode on` / `Dark mode off` |
| Pips toggle | success | NORMAL | `Pips in theme colors` / `Pips in natural colors` |
| UB toggle | success | NORMAL | `Universes Beyond hidden` / `Universes Beyond shown` |
| Theme saved | success | QUICK | `Theme saved` |
| Theme discarded (after a pick) | info | QUICK | `Theme unchanged` |
| Exceptions saved | success | QUICK | `Exceptions saved` |
| Exceptions discarded (after a change) | info | QUICK | `Exceptions unchanged` |

No toast on: logout, account deletion, session expiry, hint recording, Account user refresh, Config preferences fetch, counters, changelog.

---

## Shared pieces the TUI needs

1. Toast stack: success/error/info/warning, per-toast duration with the three bands (3/5/8 s, default 5 s), max 3, newest on top, collapsed into one line with a count or sliver and expandable with a key; drawn top-left over the header. Replaces today's single status line for every message listed in section 7, and errors that the TUI shows inline (login, forms) where the app toasts.
2. Screen header: centered title, left `!` key opening Help & feedback (`Close` / `Report a problem ↗` printing or opening the mailto with version and a TUI platform label / `Join the Discord ↗` showing the URL), right `?` key re-opening the screen's hint when it has one.
3. Dialog widget (AlertDialog): title, rule, body, rule, button row; Dismiss / Confirm (danger) / Link / live-label button with busy state and countdown; overlay key (Esc) dismisses unless busy; one at a time.
4. Hint system: `HintDialog` (title, lines/bullets, inline key chips and colored words, final `Got it`), the `HintTopic` table (2.2) reachable from row-level `?` keys, and the one-time hint mechanism (`has_seen_hint`, optimistic open, `mark_hint_shown`, replace and persist `session.user`) with the key table (2.3), including gated hints that fire on first data or first sheet open.
5. Bottom sheet equivalent: titled modal panel with a footer action row, optional header `?`, `on_dismiss` override (used by Themes and Exceptions to discard and toast).
6. Text field widget: label, optional `?`, placeholder, password masking with a show/hide key, an error line under the field, and live validation once the field has been touched or a submit attempted.
7. Decode animation exactly as orangui (22 frames of 40 ms, same fronts and RNG); the TUI port uses 21 frames and starts at frame 0, so align `frame_count` to `ceil(850/40)` and frame numbering to 1..=22. Plus the splash timeline (mark at 400 ms, leave at 1350 ms with a 450 ms evaporate, fade at 1800 ms, gone at 2100 ms) and the gate that holds the first screen's entrances until the fade.
8. Rolling counter (CountUp): 16 ms ticks over 1000 ms, random 4-digit roll while waiting, compact `k/m/b` formatting under a 10,000 threshold, replay on mount. Today the TUI prints `Counting...` then static numbers.
9. HomeHero block (logo + three counters + `Everybody`/`Live` chips) shared by Login, Register, Forgot password (and Home with the scope switch), with the 60 s shared counter cache and a disk-restored copy for the first paint.
10. Session layer additions: 60 s upkeep loop (first tick immediate) that refreshes and re-checks the min version both ways; gate-style redirect to Login on any session clear without a message; logout capped at 5 s; `fetch_user` refresh on Account open that never surfaces an error.
11. Startup caches: changelog (Loading/Loaded/Failed), UB franchises (and the other public catalogs other screens use).
12. Skeleton and loading primitives: pulsing bar blocks (2 s cycle, opacity 1 ↔ 0.4) that ghost layout without real text, a spinner, and `Could not load ...` notes.
13. Optimistic toggle helper with a per-toggle request sequence: show at once, ignore stale replies, revert to the value before that tap on failure, toast on success.
14. Theme list data (`THEME_GROUPS`, retired-slug map, `display_theme_name`) and palette swatches; live preview on move, restore on discard, `Theme unchanged` only after a real pick.
15. A simple HTML-to-text renderer for the privacy policy constants (h2, p, ul/li, strong, a).
