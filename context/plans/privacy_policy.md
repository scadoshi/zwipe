# In-App Privacy Policy (shared content)

## Goal

A native Privacy Policy page in `zwiper`, reachable from the **Profile** screen,
sourced from a **single shared `const`** so the app and the website
(`zite/src/pages/privacy.rs`) can never drift apart. Compliance need: the QA audit
flagged that no Privacy Policy is reachable inside the app.

## Architecture (decided)

Single source of truth, no new crate, no markdown engine, core stays pure.

- **`zwipe-core`** holds the policy body as a plain **HTML string `const`**
  (headings, paragraphs, lists, inline `https://` links). A `&str` is pure data —
  no Dioxus, no deps, no purity violation. Suggested home: a new `legal` module
  (`zwipe-core/src/legal/mod.rs`) exposing `pub const PRIVACY_POLICY_HTML: &str`
  and `pub const PRIVACY_LAST_UPDATED: &str`.
- **Both apps render it** with `div { dangerous_inner_html: PRIVACY_POLICY_HTML }`,
  each inside its own chrome and CSS. `dangerous_inner_html` is safe here — it's
  our own static legal copy, never user input.

### Why links work without extra wiring
Dioxus already intercepts external `<a href>` clicks app-wide and routes them
through `webbrowser::open`, which handles `http(s)` fine on mobile. So the policy's
inline `https://` links **just work** in the blob on device — no `open_url` needed
for those.

### The one exception — the support `mailto:`
`webbrowser` rejects non-http URLs, so a raw `mailto:` anchor silently fails on
mobile (the exact bug `zwiper/src/lib/outbound/open_url.rs` was built to fix).
So **keep the support-email contact line OUT of the shared HTML blob** and render
it per-app:
- `zite`: a normal `mailto:` `<a>` (web handles it).
- `zwiper`: a Dioxus element with `onclick -> open_url::open("mailto:…")` (the same
  call the support sheet already uses successfully).

## Work by crate

1. **`zwipe-core`** — add the `legal` module with `PRIVACY_POLICY_HTML` (+
   `PRIVACY_LAST_UPDATED`). Convert the existing policy text from
   `zite/src/pages/privacy.rs` into the HTML string; keep its `https` links inline;
   do **not** include the support `mailto:` (that's rendered per-app). Re-export
   so both crates can reach it.
2. **`zite`** — refactor `src/pages/privacy.rs` to render
   `PRIVACY_POLICY_HTML` via `dangerous_inner_html` instead of inline rsx; keep
   `Nav` / `Footer` / `PageMeta` and the existing `mailto:` contact line. The
   rendered page should look the same as today.
3. **`zwiper`** —
   - New screen (suggest `screens/legal/privacy_policy.rs`) rendering the const via
     `dangerous_inner_html`, wrapped in the standard `ScreenHeader` chrome, with a
     support-email line using `open_url::open`.
   - Add a route in `inbound/router.rs` (e.g. `#[route("/privacy")] PrivacyPolicy {}`).
   - Add a **"Privacy Policy"** link/row on the **Profile** screen
     (`screens/profile/mod.rs`) that navigates to it.
   - CSS in `zwiper/assets/main.css` to style the rendered HTML (`.privacy-content`
     descendant rules — mirror what `zite` does for readability).

## Acceptance criteria

- Privacy Policy reachable from Profile; renders the full policy; inline `https`
  links open externally on device; the support `mailto:` opens the mail app via
  `open_url`.
- `zite`'s privacy page still renders correctly, now from the shared `const`
  (visually unchanged).
- Editing `PRIVACY_POLICY_HTML` updates **both** app and web — single source.
- `cargo check --workspace` + `cargo clippy --workspace --all-targets -- -D warnings`
  clean. `zwipe-core` purity respected (the const is pure `&str`).

## Deploy note

Touches **`zwipe-core` and `zite`** (both deploy paths) → merging this
**redeploys the website's privacy page**. Not a blind handoff / requires push
confirmation. The `zwiper` page ships with the next mobile build. (No API contract
change, so no server-first ordering concern — but it IS a prod web deploy.)

## Out of scope

Terms & Conditions (can reuse this exact pattern later with a second const).
