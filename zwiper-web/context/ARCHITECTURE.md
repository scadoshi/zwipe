# zwiper-web architecture

zwiper-web is a client-rendered Dioxus web app. It follows zwiper's hexagonal split, so a screen ported from the app keeps the shape it had there.

```
src/
├── main.rs                 builds the ZwipeClient from BACKEND_URL and launches App
└── lib/
    ├── config.rs           BACKEND_URL and APP_VERSION, fixed at build time
    ├── domain/             the app's rules as pure functions, each with tests
    ├── outbound/           the browser: storage, the session and theme stores, the timer
    └── inbound/
        ├── app.rs          the root: stylesheets, theme, upkeep, toasts, dialogs, router
        ├── router.rs       Route, one variant per screen at the app's path
        ├── session/        ensure_fresh, the upkeep loop, the auth gate, logout
        ├── shell/          the sidebar Shell, PublicFrame, ScreenFrame, Help, nav
        └── screens/        one module per screen, plus placeholders.rs
```

## The root

`App` calls `use_session_upkeep`, which provides these contexts to everything below it:

- `Signal<Option<Session>>`, loaded from `localStorage` at start and owned by the root scope, since the refresh writes it from a task that outlives any screen.
- `Signal<ZwipeClient>`, built in `main` from `BACKEND_URL`.
- `Signal<ThemeConfig>` (the session's preferences, else the stored theme, else the default) and `Signal<PipStyle>`. Every theme change is stored, so Login opens in the last one.
- `UpgradeRequired`, the min-version gate. While it is true the root draws `UpdateRequired` in place of the router.

The upkeep loop runs `ensure_fresh` and then the min-version check, at once and every 60 s after. `ensure_fresh` is the only way to an authed call: it refreshes a token that expires within two minutes, one refresh at a time, stores the new session before setting the signal, and clears the session when the server rejects the refresh. `AuthGate` sends the user to Login the moment the session is gone, with no message, as the app does.

The root also mounts the kit's `ToastStack` (screens raise toasts with `dioxus_primitives::toast::use_toast`), the dialog host (`AlertDialog` anywhere draws there), and `SplashGate` open, since the web has no splash.

## Styling

The root loads, in order: the kit's `THEMES_CSS`, `COMPONENTS_CSS`, `MAGIC_CSS` and `APP_CSS`, zwiper's `main.css` and `accordion.css` (included from `zwiper/assets` as they are, never copied), `TOAST_CSS`, then `assets/web.css`. A screen uses the app's class names, so it looks like the app with no new CSS. `web.css` only holds what a browser window changes: the sidebar, the wide layouts, the 900px breakpoint, and undoing the phone-only rules (`body` capped to the screen height, `.screen` fixed to the viewport). The fonts are copies of zwiper's, because `asset!` only reads files inside the crate.

## Adding a screen

Say the screen is Activity.

1. Make `src/lib/inbound/screens/activity/mod.rs` with a `#[component] pub fn Activity() -> Element`. Copy `screens/login/` as a starting point. A screen with route parameters takes them as props (`deck_id: Uuid`).
2. Wrap it in `ScreenFrame { title: "Activity", .. }` and put the content in `div.screen-content` the way the app does; an `ActionBar` inside the frame becomes the toolbar under the header on a wide window. Lay the panels out as `web-layout.md` says, with classes in `web.css` named `web-<screen>-*`.
3. Port the rules the screen needs into `src/lib/domain/<name>.rs` as pure functions, with the tests the app has for them and any the spec implies. The component calls them; it does not repeat them.
4. Call the API through `zwipe-client`, with the session from `ensure_fresh(session, client)`. Wave A adds the authed-call facade that also reports and toasts; once it lands, screens use it instead.
5. Delete the placeholder from `screens/placeholders.rs`, add `pub mod activity;` to `screens/mod.rs`, and point the import in `router.rs` at the new module. The route, its path and its sidebar entry already exist.
6. If the screen belongs to a sidebar entry other than the one `Section::of` gives it, change that match in `shell/nav.rs` and its test.

Every string comes from the spec, verbatim. A screen that differs from the app in more than layout and input gets a line in the kept deviations of `parity-plan.md`.

## Tests

Pure logic is tested in `domain/` and the stores in `outbound/` against `MemoryStorage`. `BrowserStorage` off wasm reads nothing and refuses writes, so a native test never touches real storage. Components are checked by building for wasm and by the live audit.
