# 321. Account's Pending refresh button has no accessible name

State: reported | Owner: - | PR: - | Build seen: aeda394a4

## Report
Surfer, live web 1.18.15 (aeda394a4), wide, 2026-11-09, while verifying 318. On Account's Pending row, the refresh button next to Resend is icon-only and has no name at all: no `aria-label`, no `title`, no text, no `<title>` or sr-only text in its icon, and the svg isn't hidden. A screen reader announces it as just "button". Something like `aria-label="Check whether it's confirmed"` (or "Refresh") would fix it; the icon could then take `aria-hidden`.

## Repro
1. With an email change pending, open Account on the web.
2. Inspect the round button right of Resend (`button.btn-action.btn-action-eye`): it has no accessible name.
