// Oracle-tag description authoring workflow (for the `Workflow` tool).
// Runbook: ./otag_description_authoring.md
//
// Usage: launch with the `Workflow` tool, passing the target slugs as args:
//   Workflow({ scriptPath: "<this file>", args: { slugs: ["scry", "landfall", ...] } })
//
// Pipeline per 7-slug chunk: sonnet DRAFTS reading real oracle text, then opus
// VERIFIES by re-reading oracle text and correcting. Returns
//   { total, items: [{ slug, description, verdict: accurate|minor|wrong, note }] }
// where `description` is the final, verifier-corrected text.
//
// NOTE: `export const meta` must be the FIRST statement in the script (Workflow
// validator requirement), so the ENV constant is declared just below it.

export const meta = {
  name: 'author-verify-otags',
  description: 'Draft then oracle-text-verify plain-English descriptions for a batch of oracle tags',
  phases: [
    { title: 'Draft', detail: 'sonnet drafter reads oracle text + writes descriptions' },
    { title: 'Verify', detail: 'opus verifier checks each vs oracle text, corrects' },
  ],
}

// EDIT THIS: absolute path to <repo>/zerver/.env on the machine you run on.
const ENV = 'zerver/.env' // repo-relative: agents run from the repo root, so neither Mac needs to edit this

const DRAFT_SCHEMA = {
  type: 'object',
  properties: { items: { type: 'array', items: { type: 'object', properties: { slug: { type: 'string' }, description: { type: 'string' } }, required: ['slug', 'description'] } } },
  required: ['items'],
}
const VERIFY_SCHEMA = {
  type: 'object',
  properties: { items: { type: 'array', items: { type: 'object', properties: { slug: { type: 'string' }, description: { type: 'string' }, verdict: { type: 'string', enum: ['accurate', 'minor', 'wrong'] }, note: { type: 'string' } }, required: ['slug', 'description', 'verdict', 'note'] } } },
  required: ['items'],
}

// Workflow args arrive as a JSON STRING, not an object — parse defensively.
const A = typeof args === 'string' ? JSON.parse(args) : (args || {})
const SLUGS = A.slugs || []
// 10 suits hierarchy-grounded batches (less card text per tag than a populated
// batch, where 7 is the safer number).
const CHUNK = 10
const chunks = []
for (let i = 0; i < SLUGS.length; i += CHUNK) chunks.push(SLUGS.slice(i, i + CHUNK))

function groundingCmd(slugs) {
  const arr = slugs.map(s => `'${s}'`).join(',')
  // Grounding is HIERARCHY-AWARE on purpose. Many tags carry zero cards of their
  // own: they are umbrella/parent nodes (`recursion-land`, `typal-creature`) or
  // cycle roots whose members live on child tags (`cycle-fetchland`). A plain
  // JOIN on card_oracle_tags returns nothing for those and the drafter invents.
  // So: `pop` is the tag's OWN card count (0 is normal and fine), `children` and
  // `parents` place it in Scryfall's tree, and `cards` samples the tag AND its
  // direct children, which is what makes `cycle-fetchland` show real fetchlands.
  return `export DATABASE_URL="$(grep '^DATABASE_URL=' ${ENV} | cut -d= -f2-)"
psql "$DATABASE_URL" -t -A -c "
WITH RECURSIVE t(slug) AS (SELECT unnest(ARRAY[${arr}]::text[])),
sub(root, id, slug, depth) AS (
  SELECT t.slug, o.id, o.slug, 0 FROM t JOIN oracle_tags o ON o.slug=t.slug
  UNION ALL
  SELECT s.root, c.id, c.slug, s.depth+1 FROM sub s JOIN oracle_tags c ON s.id = ANY(c.parent_ids) WHERE s.depth < 1
)
SELECT json_agg(row_to_json(r)) FROM (
  SELECT t.slug,
    (SELECT count(DISTINCT oracle_id) FROM card_oracle_tags WHERE oracle_tag=t.slug) AS pop,
    COALESCE(NULLIF(ot.description,''),'') AS scryfall,
    COALESCE((SELECT string_agg(p.slug,', ') FROM oracle_tags p WHERE p.id = ANY(ot.parent_ids)),'') AS parents,
    COALESCE((SELECT string_agg(c.slug,', ') FROM (SELECT slug FROM oracle_tags c2 WHERE ot.id = ANY(c2.parent_ids) ORDER BY slug LIMIT 20) c),'') AS children,
    COALESCE((SELECT json_agg(x) FROM (SELECT DISTINCT ON (sd.oracle_id) sd.name, sd.mana_cost AS cost, sd.type_line AS type, LEFT(sd.oracle_text,240) AS text
       FROM card_oracle_tags co JOIN sub s ON s.slug=co.oracle_tag AND s.root=t.slug
       JOIN scryfall_data sd ON sd.oracle_id=co.oracle_id ORDER BY sd.oracle_id LIMIT 8) x),'[]'::json) AS cards
  FROM t JOIN oracle_tags ot ON ot.slug=t.slug
) r;"
(this query takes ~20s, that is normal, do not kill it)`
}

const STYLE = `You are writing user-facing descriptions of Magic: The Gathering "oracle tags" (Scryfall's community functional tags). Each appears in a mobile app next to a card and in a tag dictionary.

STYLE RULES (follow exactly):
- ONE short sentence, ideally under ~90 characters.
- Plain English. Describe what a card with this tag DOES, functionally.
- NO em dashes. Use commas, colons, or periods.
- NO Scryfall cross-link syntax like [label](slug) and NO URLs. NO ampersand entities; write "and".
- Present tense. Address the player as "you" where natural.
- Start with a verb ("Deals...", "Grants...", "Removal that...") or "A <noun> that...".
- Be ACCURATE to the real card oracle text you are shown. If the slug NAME is misleading, trust the cards.
- Many of these tags are keyword MECHANICS (e.g. convoke, threshold, phasing, heroic, morbid, imprint, strive). Describe the mechanic plainly in one sentence.
- Sibling precision: "gives-X" grants to OTHERS, "gains-X"/"-self" is about ITSELF, "-to-all" hits your whole team, "repeatable-X" can be done again and again, "typal-X" cares about creatures of type X, "synergy-X"/"hate-X" reward/punish X.

EXAMPLE voice:
spot-removal => Removal aimed at a single target.
sweeper => Removal that wipes many or all permanents at once.
cantrip => Draws you a card when it resolves or enters.
group-slug => Makes each opponent lose life or take damage.
french-vanilla => A creature whose only abilities are keywords.
mana-dork => A creature that produces or helps pay for extra mana.
gives-flying => Grants flying to a creature.
landfall => Triggers an effect whenever a land enters the battlefield under your control.
tutor-to-hand => Searches your library for a card and puts it into your hand.`

function draftPrompt(chunk) {
  return `${STYLE}

Write a description for each of these ${chunk.length} oracle-tag slugs:
${chunk.join(', ')}

STEP 1 - Run this bash command; read the JSON. One row per slug: {slug, pop, scryfall, parents, children, cards[]}, where each card is {name, cost, type, text}. "pop" is how many cards carry THIS tag; **pop 0 is normal** for umbrella and cycle-root tags, and it does NOT mean the tag is meaningless. "cards" samples the tag plus its direct children, so a cycle root shows its real members. "children" and "parents" are the tag's place in Scryfall's tree and are often the STRONGEST evidence of meaning. "scryfall" is Scryfall's own note: use it as a hint, do NOT copy it, and NEVER carry over its cross-link or URL syntax:
\`\`\`
${groundingCmd(chunk)}
\`\`\`
Retry once if it fails.

STEP 2 - For each slug, read the evidence and write one accurate description in our voice per the STYLE RULES. Weigh it in this order: real cards first, then the children list, then the slug name, then Scryfall's note. For an umbrella tag (pop 0 with many children) describe the FAMILY, e.g. "A cycle of lands that ..." or "Cards that return X from a graveyard". For a cycle tag, say what the cycle's cards have in common, and only name a set or block if the evidence shows it. Never state a card's cost, color, hybrid-ness, rarity, or mana value unless the pulled data shows it (a {X/Y} cost is HYBRID). Don't claim a whole cycle is uniform unless every pulled card agrees and "pop" isn't much larger than what you see.

Return the structured object: one item per slug (slug + description), no extras.`
}

function verifyPrompt(chunk, draftItems) {
  return `${STYLE}

You are the ADVERSARIAL VERIFIER (a meticulous MTG rules expert). A drafter wrote these:
${JSON.stringify(draftItems, null, 2)}

STEP 1 - Pull the evidence. Run this and read the JSON ({slug, pop, scryfall, parents, children, cards[]}; pop 0 is normal for umbrella and cycle-root tags; "cards" samples the tag plus its direct children):
\`\`\`
${groundingCmd(chunk)}
\`\`\`

STEP 2 - For EACH tag, read the card data and judge whether the drafted description is accurate and clear. Fix it if it is wrong, misleading, overspecified/underspecified (e.g. says "opponent" when it's "any player", "creatures" when it's "permanents", "your creatures" when it's "creatures"), copies Scryfall, carries any cross-link or URL syntax, or breaks a STYLE RULE. A description that is vague to the point of saying nothing ("Cards that do things with counters") counts as wrong, tighten it using the children list. Watch for slug-name traps. Never assert a card's cost, color, hybrid-ness, rarity, or mana value unless the data shows it (a {X/Y} cost is HYBRID); don't over-generalize a cycle from a partial sample.

Return the structured object: one item per tag with:
- slug
- description: the FINAL description (your corrected version if you changed it, else the draft)
- verdict: "accurate" (draft was fine), "minor" (tightened), or "wrong" (draft was inaccurate)
- note: one short phrase on what you changed, or "" if accurate.

Cover every tag, no extras.`
}

const results = await pipeline(
  chunks,
  (chunk, _o, idx) => agent(draftPrompt(chunk), { label: `draft:${idx}`, phase: 'Draft', schema: DRAFT_SCHEMA, model: 'sonnet', effort: 'medium' }),
  (draft, chunk, idx) => agent(verifyPrompt(chunk, (draft && draft.items) || []), { label: `verify:${idx}`, phase: 'Verify', schema: VERIFY_SCHEMA, model: 'opus', effort: 'high' })
)

const flat = results.filter(Boolean).flatMap(r => (r && r.items) || [])
const w = flat.filter(i => i.verdict === 'wrong').length
const m = flat.filter(i => i.verdict === 'minor').length
log(`done ${flat.length}: ${flat.length - w - m} accurate, ${m} minor, ${w} wrong`)
return { total: flat.length, items: flat }
