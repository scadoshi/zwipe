# Wire changes in flight

Every change a shipped client can see, where it stands, and the floor it waits on. Each follows [`../development/api_evolution.md`](../development/api_evolution.md): the server reads both names, the client ships reading both, the floor rises, then the old name goes. Update a row when its phase moves; move it to the bottom list when its last phase lands.

| Change | Plan | Where it is | Waits on |
| --- | --- | --- | --- |
| Pip value `natural` to `classic` | [`pip_color_wire.md`](pip_color_wire.md) | Phase 1 on main, riding 1.17.1. Floor at 1.18.1 in production. Phase 4 rebuilt on `server/pip-classic-only`, PR to release | Merge Phase 4, then deploy zerver and zite |
| Pip value `theme` to `themed` | [`pip_themed_wire.md`](pip_themed_wire.md) | Phase 1 on `nits/thinker-143`, riding 1.18.1. Phase 4 built on `server/pip-themed`, not merged | Floor at 1.18.1 or later, then merge Phase 4 |
| Route `/api/user/preferences` and key `preferences` to `settings` | [`settings_rename_migration.md`](settings_rename_migration.md) | Phase 1 in production 2026-11-05. Phase 2 rides 1.18.0 | Floor at 1.18.0, then drop the legacy route and key |

## Additive, no floor

- `swipe_confirmations` on `UserSettings` and `HttpUpdateSettings` (nit 153): `chip` (the default), `toasts` or `off`. `#[serde(default)]` on both, and no shipped `UserSettings` sets `deny_unknown_fields`, so 1.15.0 to 1.18.0 skip it in a session or settings reply and never send it; a PATCH without it leaves the column alone. Migration `20261107130000_user_settings_swipe_confirmations.sql` adds the column with `DEFAULT 'chip'`. Built 2026-11-07 on `nits/thinker-153`.

- Cookie sessions for the browser: `/api/auth/cookie/login`, `/cookie/refresh` and `/cookie/logout` keep the refresh token in an `HttpOnly` cookie for zwiper-web. New routes beside the app's, so no shipped client changes. Built 2026-11-06, not committed, under review.

## Done

None yet.
