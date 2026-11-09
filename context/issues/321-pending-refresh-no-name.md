# 321. Account's Pending refresh button has no accessible name

State: fixing | Owner: scotland | PR: - | Build seen: aeda394a4

## Report
Surfer, live web 1.18.15 (aeda394a4), wide, 2026-11-09, while verifying 318. On Account's Pending row, the refresh button next to Resend is icon-only and has no name at all: no `aria-label`, no `title`, no text, no `<title>` or sr-only text in its icon, and the svg isn't hidden. A screen reader announces it as just "button". Something like `aria-label="Check whether it's confirmed"` (or "Refresh") would fix it; the icon could then take `aria-hidden`.

## Repro
1. With an email change pending, open Account on the web.
2. Inspect the round button right of Resend (`button.btn-action.btn-action-eye`): it has no accessible name.

## Fix
Branch nits/scotland-321. Every Account refresh button now has a name, in the app and on the web: the Pending row's is "Check the new email" and the Verification row's (also icon-only) is "Check verification". Both come from shared zwipe-ui constants and sit in an sr-only span inside the button, as the deck page's fold-all does, since the kit's Button takes no aria-label. The icon svg takes `aria-hidden`. The web's hover titles use the same constants. These components have no tests, so there is no failing-first test.
