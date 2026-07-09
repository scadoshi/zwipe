# iOS build history

Per-release iOS build numbers (`CFBundleVersion`). iOS and Android ship coupled
from the same `main`, so the fuller per-release detail (features, server halves,
migrations) lives in the Android
[`history.md`](../../../android/play-store/submission/history.md); this is the
iOS-side build-number log.

| Version | iOS build | Notes |
|---------|-----------|-------|
| 1.4.0 | 61 | Feature batch: commander popularity ordering, partner autofill, Deck MVPs phase 1, deck share links. Android counterpart vc22. |
| 1.3.1 | 60 | Anonymous pre-auth funnel telemetry. Android vc21. |
| 1.3.0 | 59 | Filter-intent + Reset pass. (58 added the profile About section.) Android vc20. |
| 1.2.3 / 1.3.0 | 56 | Swipe memory (per-deck skip/removal). Android vc17. |
| 1.2.1 | 55 | Card rules dialog + launch-flash fix (uploaded, held behind 1.2.0, superseded by 56 — never went to iOS review). |
| 1.2.0 | 54 | Draw-odds, synergy toggle, power level + other tags. Android vc15. |
| 1.1.4 | 53 | Bottom-sheet flash + clone-nav fixes. Android vc14. |
| 1.1.3 | 51 | Media-day release (card names while swiping, deck-form overhaul, in-app privacy policy). Android vc11. |
| 1.1.2 | 50 | Filter-control consistency on card-swipe screens. Android vc10. |
| 1.1.1 | 49 | In-app help, import/export hints, mailto OS-open fix. Android vc9. |
| 1.1.0 | 48 | Zwipe-select, deck tags, keyword hinter, expanded card detail. Android vc8. |
| 1.0.10 | 44–45 | Update-screen redesign; commander-search indicator. |

Older builds predate this log; see git history if needed.
