# Mechanical Category — Remaining Work

Phases 1-3 shipped (`91640771`, `c311462e`). Schema, heuristics, filtering, grouping, metrics, and frontend are live. ~73% classification rate from heuristics (79k / 108k cards).

---

## Heuristic Refinement

Current accuracy needs improvement. Audit each category against real card data:

- [ ] Audit sample of 20+ cards per category for false positives/negatives
- [ ] Lands should NOT be ramp (fix landed, verify no regressions)
- [ ] Add ramp patterns: treasure token creators, rituals (Dark Ritual), cost reducers
- [ ] Add removal patterns: fight mechanics, "exile target" with qualifiers
- [ ] Burn: should ETB damage creatures count? (Flametongue Kavu)
- [ ] Stax: false positives from "can't" in reminder text
- [ ] Blink: widen proximity regex if still missing cards
- [ ] Consider adding sub-categories or secondary tags for edge cases

## Layer 2: Zort (AI Classification Client)

Standalone binary connecting to Postgres, classifying in batches via LLM.

- [ ] Create `zort/` crate in workspace
- [ ] Subcommands: `zort classify` (untagged), `zort reclassify` (all), `zort delta` (changed), `zort audit` (compare vs heuristics)
- [ ] LLM prompt with 24-category taxonomy + definitions
- [ ] Batch read → classify → UPDATE pipeline
- [ ] Delta sync: only classify cards where heuristic tags differ or card text changed
- [ ] Cost: ~$5-15 for full 108k card run via Haiku

## Layer 3: Fine-Tuned Model (Future)

- [ ] Export training data from Layer 2 corrected tags
- [ ] Train small model: oracle_text + type_line → category tags
- [ ] Embed in zervice sync pipeline for real-time classification
