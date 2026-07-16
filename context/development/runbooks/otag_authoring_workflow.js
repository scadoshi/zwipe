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
const ENV = '/Users/scottyrayfermo/Developer/zwipe/zerver/.env'

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
const CHUNK = 7
const chunks = []
for (let i = 0; i < SLUGS.length; i += CHUNK) chunks.push(SLUGS.slice(i, i + CHUNK))

function groundingCmd(slugs) {
  const arr = slugs.map(s => `'${s}'`).join(',')
  return `export DATABASE_URL="$(grep '^DATABASE_URL=' ${ENV} | cut -d= -f2-)"
psql "$DATABASE_URL" -t -A -c "SELECT json_agg(row_to_json(r)) FROM (SELECT t.slug, cnt.n AS pop, COALESCE(NULLIF(ot.description,''),'') AS scryfall, x.name, x.mana_cost AS cost, x.colors, x.rarity, x.cmc AS mv, x.type_line AS type, LEFT(x.oracle_text,300) AS text FROM (SELECT unnest(ARRAY[${arr}]::text[]) AS slug) t JOIN oracle_tags ot ON ot.slug=t.slug JOIN (SELECT oracle_tag, COUNT(DISTINCT oracle_id) n FROM card_oracle_tags GROUP BY oracle_tag) cnt ON cnt.oracle_tag=t.slug JOIN LATERAL (SELECT DISTINCT ON (sd.oracle_id) sd.name, sd.mana_cost, sd.colors, sd.rarity, sd.cmc, sd.type_line, sd.oracle_text FROM card_oracle_tags c JOIN scryfall_data sd ON sd.oracle_id=c.oracle_id WHERE c.oracle_tag=t.slug ORDER BY sd.oracle_id LIMIT 12) x ON true) r;"`
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

STEP 1 - Run this bash command; read the JSON (array of {slug, pop, scryfall, name, cost, colors, rarity, mv, type, text}; up to 12 cards per tag; "cost" is mana cost e.g. {2}{G/U}, "colors" is an array ([] = colorless), "mv" is mana value; "scryfall" is Scryfall's own note, do NOT copy it):
\`\`\`
${groundingCmd(chunk)}
\`\`\`
Retry once if it fails.

STEP 2 - For each slug, read the card data and write one accurate description in our voice per the STYLE RULES. Never state a card's cost, color, hybrid-ness, rarity, or mana value unless the pulled data shows it (a {X/Y} cost is HYBRID). Don't claim a whole cycle is uniform unless every pulled card agrees and "pop" isn't much larger than what you see.

Return the structured object: one item per slug (slug + description), no extras.`
}

function verifyPrompt(chunk, draftItems) {
  return `${STYLE}

You are the ADVERSARIAL VERIFIER (a meticulous MTG rules expert). A drafter wrote these:
${JSON.stringify(draftItems, null, 2)}

STEP 1 - Pull real cards with rules text. Run this and read the JSON ({slug, pop, name, cost, colors, rarity, mv, type, text}; "cost" is mana cost, "colors" is an array, "mv" is mana value):
\`\`\`
${groundingCmd(chunk)}
\`\`\`

STEP 2 - For EACH tag, read the card data and judge whether the drafted description is accurate and clear. Fix it if it is wrong, misleading, overspecified/underspecified (e.g. says "opponent" when it's "any player", "creatures" when it's "permanents", "your creatures" when it's "creatures"), copies Scryfall, or breaks a STYLE RULE. Watch for slug-name traps. Never assert a card's cost, color, hybrid-ness, rarity, or mana value unless the data shows it (a {X/Y} cost is HYBRID); don't over-generalize a cycle from a partial sample.

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
