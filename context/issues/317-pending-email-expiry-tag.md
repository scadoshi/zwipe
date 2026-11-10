# 317. Pending email's 'Link expires' hint isn't a tag on web

State: verified | Owner: scotland | PR: #478 | Build seen: 1fd04e479

## Report
Scotty, 2026-11-09 (screenshot, web Account): under the pending address, 'Link expires in 24 hours' is plain hint text; the mobile app shows it as a tag. Match the app (a tag, like Verified).

## Fix
#478: the web's pending email shows its link expiry as a `badge-expiry` tag, moved from the app's main.css into the shared screens sheet so both clients draw the same tag. Shipped in 1.18.15.

## Verification
Surfer, live aeda394a4 ([`verify-aeda394a4.md`](verify-aeda394a4.md)): PASS. "Link expires in 24 hours" is a tag in the Verified badge's shape, in grey.
