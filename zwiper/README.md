# zwiper

Dioxus cross-platform mobile app for Zwipe. Swipe-based MTG deck builder.

## Targets

- iOS (primary) — `dx build --platform ios --device`
- Web preview — `dx serve`
- Android — `dx build --platform android` (near submission ready)
- Desktop — `dx serve --platform desktop`

## Architecture

- **Screens** — Dioxus components in `inbound/ui/screens/`
- **API client** — HTTP client modules in `outbound/client/`
- **Session** — JWT + refresh token stored in platform keychain
- **Domain types** — imported from `zwipe-core`
- **9 themes** — user-selectable with dark mode support
