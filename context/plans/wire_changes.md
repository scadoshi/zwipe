# Wire changes in flight

Every change a shipped client can see, where it stands, and the floor it waits on. Each follows [`../development/api_evolution.md`](../development/api_evolution.md): the server reads both names, the client ships reading both, the floor rises, then the old name goes. Update a row when its phase moves; move it to the bottom list when its last phase lands.

| Change | Plan | Where it is | Waits on |
| --- | --- | --- | --- |
| Pip value `natural` to `classic` | [`pip_color_wire.md`](pip_color_wire.md) | Phase 4 merged (#491): the server writes `classic`. The settings request still takes `natural`, because builds through 1.18.16 send it | Floor at the first store build after 1.18.16 (the first that writes `classic`), then drop the request alias (Phase 5) |
| Pip value `theme` to `themed` | [`pip_themed_wire.md`](pip_themed_wire.md) | Phase 4 merged (#492): the server writes `themed`. The settings request still takes `theme`, because builds through 1.18.16 send it | Floor at the first store build after 1.18.16 (the first that writes `themed`), then drop the request alias (Phase 5) |

## Additive, no floor

- `swipe_confirmations` on `UserSettings` and `HttpUpdateSettings` (nit 153): `chip` (the default), `toasts` or `off`. `#[serde(default)]` on both, and no shipped `UserSettings` sets `deny_unknown_fields`, so 1.15.0 to 1.18.0 skip it in a session or settings reply and never send it; a PATCH without it leaves the column alone. Migration `20261107130000_user_settings_swipe_confirmations.sql` adds the column with `DEFAULT 'chip'`. Built 2026-11-07 on `nits/thinker-153`.

- Cookie sessions for the browser: `/api/auth/cookie/login`, `/cookie/refresh` and `/cookie/logout` keep the refresh token in an `HttpOnly` cookie for zwiper-web. New routes beside the app's, so no shipped client changes. Built 2026-11-06, not committed, under review.

## Done

- Route `/api/user/preferences` and key `preferences` to `settings`, [`archive/settings_rename_migration.md`](archive/settings_rename_migration.md): Phase 3's server half landed 2026-11-10 with the floor at 1.18.1; the legacy route and the login reply's `preferences` key are gone. `Session` keeps reading `preferences` (`SessionWire`) because app builds before 1.18.0 stored their session with it; that read fallback goes 30 days after the floor reached 1.18.0.
