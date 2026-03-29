# User Preferences — Overview

## What

Let users pick a theme (9 options) and light/dark mode. Persisted server-side, delivered
in JWT claims, applied immediately after login.

## Why

- The portfolio already has 9 complete theme definitions with CSS variables
- Users expect basic personalization
- Foundation for future preferences (sort defaults, card density, etc.)

## Architecture Decisions

1. **Separate `user_preferences` table** — not a JSONB column on `users`. Clean schema,
   easy to extend, standard FK with cascade delete.

2. **No new AppState generic** — preferences are a user concern. Add methods to existing
   `UserService` and `UserRepository` traits. Keeps `AppState<AS, US, HS, CS, DS>`
   unchanged — every handler, route, and test stays the same.

3. **Preferences in JWT claims** — `theme` and `dark_mode` added to `UserClaims`. Frontend
   reads from session on login. No extra API call on app start.

4. **Defaults on missing row** — existing users have no row. Return
   `theme: "zwipe", dark_mode: true`. Row created lazily on first update.

5. **Theme values are strings** — `VARCHAR` in DB, validated against an allowed list at
   the domain level. Adding a new theme = adding a string, no migration.

6. **No password required** — unlike change_email/change_password, preferences are
   low-risk. Just authenticated user + new values.

7. **Dark-only themes** — Zwipe and Vantablack force `dark_mode: true` at the service
   layer. Frontend disables the toggle for these.

## Execution Order

Work in four sessions, each independently committable:

| Session | What | Depends on |
|---------|------|------------|
| 1 | Database + domain layer | Nothing |
| 2 | Backend HTTP handlers + routes | Session 1 |
| 3 | JWT claims + session integration | Session 2 |
| 4 | Frontend CSS + preferences screen | Session 3 |

## File Index

- `overview.md` — this file
- `01-database-and-domain.md` — migration, models, ports, services, repository
- `02-http-handlers.md` — handlers, routes, paths, error mappings
- `03-jwt-and-session.md` — claims, session struct, login/refresh flow
- `04-frontend.md` — CSS themes, client traits, preferences screen, theme signal
