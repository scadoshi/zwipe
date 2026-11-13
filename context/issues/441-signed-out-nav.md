# 441. The web app's signed-out nav lists Forgot password

State: fixing | Owner: thinker | Type: issue | PR: - | Build seen: production

## Report
Composer for Scotty, 2026-11-12: the web app's signed-out nav shows Log in, Create account and Forgot password (seen on a shared deck). Forgot password belongs on the Log in screen only; the signed-out nav is Log in and Create account. Check a shared deck and the signed-out pages at 1450 and 390.

## Fix
thinker:
- `PublicNav` reads its links from `public_links()`, Log in and Create account; Forgot password stays the Log in screen's link and its route.

## Verification
- `cargo test -p zwiper-web the_signed_out_menu`.
- Signed out at 1450 and 390 (menu open): /login, /register, /forgot-password and a shared deck show Log in and Create account only.
