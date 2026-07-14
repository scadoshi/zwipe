# zwiper

Dioxus cross-platform app for Zwipe. Swipe-based MTG deck builder, live on the
iOS App Store and Google Play.

## Targets

- iOS: `dx build --platform ios --device`
- Android: `dx build --platform android`
- Web preview: `dx serve`
- Desktop (dev): `dx serve --platform desktop`

Feature flags select the Dioxus renderer: `mobile` (default), `web`, `desktop`.

## Architecture

- **Screens**: Dioxus components in `src/lib/inbound/screens/` (auth, deck, card browse + filters, profile, legal)
- **Components**: reusable UI in `src/lib/inbound/components/` (fields, dialogs, swipe/carousel interactions, navigation, toast, telemetry)
- **API client**: HTTP client modules in `src/lib/outbound/client/`
- **Session**: JWT + refresh token, stored in the iOS keychain (`keyring`) and in an app-private file on Android (which has no keyring backend)
- **Domain types**: imported from `zwipe-core`
- **Shared UI**: `zwipe-components` supplies cross-surface components and CSS (buttons, chips, card row/details, theme picker, changelog)
- **31 themes**: user-selectable, each with a light and dark variant
