# Triage

Raw intake that still needs a decision. Items land here first, get a verdict
(build / drop / defer), then promote to `../feature_requests.md`,
`../backlog.md`, or `../todo.md` — or get deleted. Keep one focused file per
item; delete the file once it's promoted or rejected.

Distinct from `feature_requests.md` (already-weighted candidates) and `todo.md`
(committed, actionable work).

| Item | Source | Verdict |
|------|--------|---------|
| [price-target-field-size](price-target-field-size.md) | Self-noted | **Decided — build** (small) |
| [empty-filter-warning](empty-filter-warning.md) | Self-noted | Undecided — consider |

Logged 2026-06-29; to be built out / triaged the following day.

**Resolved & removed:**
- **landing-screen-fouc** (Self-noted) — 2026-07-01. Shipped: native WebView
  background color + a hidden-until-styled `#main` gate kills the load flash on
  iOS/Android.
- **card-oracle-text-fallback** (User, 2026-06-30) — 2026-07-01. Shipped as the
  card-rules dialog (util-bar eye button → oracle text + stats), completing FR #8.
- **viberank-growth-feedback** (Viberank outreach) — 2026-06-30. Acted on the
  SEO/marketing observations (SEO batch shipped in `zite`: OG share image, keyword
  title + `<h1>`, JSON-LD, testimonials, "Free/no ads" line, generated sitemap;
  blog play planned in [`../../plans/seo_guides.md`](../../plans/seo_guides.md)).
  Free viberank.dev submission done. Nothing left to decide.
