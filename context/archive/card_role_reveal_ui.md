# Card-role otag reveal — block-quote framing + optional hint drill-down

**Status: DONE 2026-07-12.** Part 1 (block-quote the revealed otags) SHIPPED (`d8f5fac2`).
Part 2 (per-chip clickable hint text) was **superseded** — the otag-education need was met
instead by in-app education pages + a help affordance (`38014ca9`, and the `help` prop on
`CardRoleChips`), with otags shown as raw slugs (`ffd52c5e`). No live remaining work.

Two UI polish ideas for the `CardRoleChips` drill-down
(`zwipe-components/src/card_role_chips.rs`): (1) frame the revealed otags in a block-quote
container so it's obvious they're the exposed otags of the tapped role, and (2) optionally
make an otag chip clickable to reveal its hint text when it has one.

## Current rendering (grounded)

Tapping an expandable role opens the shared `keyword-reveal` animated container; inside,
`keyword-reveal-inner → div.card-detail-meta.card-detail-otags → span.detail-chip` per otag
(labels via `prettify_oracle_tag_slug`). So the revealed otags are plain chips, visually the
same as the meta chips above — nothing signals "these belong to the role you just tapped."

Block-quote styles already exist to borrow from: `.card-detail-oracle` (the dark rounded
oracle-text block) and `.keyword-reveal-text` (keyword reminder text).

## Part 1 — block-quote the revealed otags (small, self-contained, do first)

Wrap the revealed `card-detail-otags` chip row in a block-quote-style container (reuse the
`.card-detail-oracle` look, or a new `.otag-reveal-block` sharing its border/inset/tint) so
the reveal reads as a distinct "these are `<Role>`'s oracle tags" panel, mirroring how the
keyword reminder sits in its own block.

- **Files:** `card_role_chips.rs` (wrap the reveal `div`) + `components.css` (one block rule).
- **No component signature change**, so every consumer (zwiper card row, swipe `card_info`,
  zite shared deck, portfolio) gets it for free. Pure visual.
- **Optional nicety:** a tiny caption line ("`<Role>` tags") atop the block for extra clarity.

## Part 2 — clickable otag chips → hint text (larger; data dependency)

Make an otag chip in the reveal tappable to expose its **description**, the way keyword chips
expose reminder text.

**The blocker — descriptions aren't on the card.** `card_profiles.oracle_tags_by_role` carries
only **slugs** (`BTreeMap<String, Vec<String>>`). Otag descriptions live on the catalog:
`OracleTag.description: Option<String>` (`oracle_tag.rs`), served by `GET /api/card/oracle-tags`
— and only **~29%** of otags carry one (measured at ingest). So to show hints, the component
needs a `slug -> description` map supplied by the host.

**Approach:** host fetches the oracle-tags catalog once (already have `ClientGetOracleTags`),
builds a `slug -> Option<description>` map, passes it into `CardRoleChips` as a new prop. A
chip is a `button` (tappable) **only when a description exists**; otherwise it stays a plain
`span` (same pattern the role chips already use: expandable iff it has content). Tapping opens
a second inner reveal with the description text.

**Costs / "is it too much?":**
- **Component signature change** → touches every `CardRoleChips` consumer (add the map prop;
  `None`/empty map = today's behavior, so it degrades gracefully).
- **A third nesting level** (role → otags → hint), nested inside the already-animated reveal.
  Manageable but it's real interaction depth; keep the hint reveal visually distinct from the
  otag block so it doesn't read as more chips.
- **Only ~29% clickable**, so the affordance must be subtle (e.g. only described chips get the
  button styling) to avoid implying every chip does something.
- **Couples to the catalog work.** Supplying descriptions client-side is exactly what the
  server-driven catalog effort sets up (`plans/server_driven_catalogs.md`, currently gated
  behind the in-flight otags impl). Cleanest to build Part 2 **after** that lands, reusing its
  catalog fetch, rather than bolting a one-off oracle-tags fetch onto the card row now.

## Recommendation

- **Ship Part 1 now** (once the otags agent's shared-component churn settles) — cheap, no
  signature change, immediate clarity win.
- **Defer Part 2** to ride on `server_driven_catalogs.md`: it needs client-side otag
  descriptions, which that plan already delivers. Not "too much," just better sequenced there
  than as a standalone fetch.
