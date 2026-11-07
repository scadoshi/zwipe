# Wire changes in flight

Every change a shipped client can see, where it stands, and the floor it waits on. Each follows [`../development/api_evolution.md`](../development/api_evolution.md): the server reads both names, the client ships reading both, the floor rises, then the old name goes. Update a row when its phase moves; move it to the bottom list when its last phase lands.

| Change | Plan | Where it is | Waits on |
| --- | --- | --- | --- |
| Pip value `natural` to `classic` | [`pip_color_wire.md`](pip_color_wire.md) | Phase 1 on main, riding 1.17.1. Phase 4 built on `server/pip-color-classic`, not merged | Floor at 1.17.1 or later, then merge Phase 4 |
| Pip value `theme` to `themed` | [`pip_themed_wire.md`](pip_themed_wire.md) | Phase 1 on `nits/thinker-143`, riding 1.18.1. Phase 4 built on `server/pip-themed`, not merged | Floor at 1.18.1 or later, then merge Phase 4 |
| Route `/api/user/preferences` and key `preferences` to `settings` | [`settings_rename_migration.md`](settings_rename_migration.md) | Phase 1 in production 2026-11-05. Phase 2 rides 1.18.0 | Floor at 1.18.0, then drop the legacy route and key |

## Additive, no floor

- Cookie sessions for the browser: `/api/auth/cookie/login`, `/cookie/refresh` and `/cookie/logout` keep the refresh token in an `HttpOnly` cookie for zwiper-web. New routes beside the app's, so no shipped client changes. Built 2026-11-06, not committed, under review.

## Done

None yet.
