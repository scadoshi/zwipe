# IAP infrastructure — the prerequisite for charging anyone

**Tier: n/a — this is the gate.** None of the premium features can ship as
*premium* until this exists. It's also the real scope of "launch premium" —
the features are mostly smaller than the billing plumbing.

## Architecture (per ../monetization.md technical path)

1. **StoreKit 2** on iOS: subscription product ($3–5/mo, $20–25/yr), purchase
   + restore flows in zwiper.
2. **Server-side entitlement** — the source of truth. App Store
   Server API / server notifications (renewals, cancellations, refunds,
   grace periods) update an entitlement record per user; an entitlement flag
   rides the JWT (or is checked per-request) so zerver can gate routes.
3. **Route gating**: premium routes return **402** for free users — the
   client treats 402 as "show the upsell screen," which also keeps old
   clients graceful when new premium routes appear.
4. **Android later**: Google Play Billing is a parallel adapter writing to the
   same entitlement record. Design the entitlement table store-agnostic from
   day one (store, original_transaction_id, expires_at, status).

## Gotchas to scope in

- **Receipt validation is server-side** — never trust the client's claim of
  entitlement.
- **Restore purchases** is an App Store review requirement.
- **Clock skew / grace periods**: honor Apple's grace period states or
  renewals will flap entitlements at month boundaries.
- **Sandbox testing** needs its own App Store Connect setup; budget real time
  for it.
- Apple takes 15% (Small Business Program, under $1M/yr) — price targets in
  `../monetization.md` are gross.

## Sequencing

Android ships before premium (decided 2026-06-10). When premium starts, build
this first against ONE already-finished feature (price intelligence or AI
analysis) rather than holding the launch for the full catalog — the catalog
grows behind the same 402 gate afterward.
