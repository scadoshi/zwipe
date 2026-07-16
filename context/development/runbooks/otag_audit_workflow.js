// Oracle-tag description AUDIT workflow (for the `Workflow` tool).
// Companion to otag_authoring_workflow.js / otag_description_authoring.md.
//
// Independent second-pass QA: each agent re-pulls the real cards carrying a tag
// and adversarially checks the ALREADY-AUTHORED description against oracle text,
// trying to find inaccuracies. Returns
//   { total, items: [{ slug, verdict: accurate|suspect|wrong, issue, example, suggestion }] }
// Only suspect/wrong items are actionable; `example` cites a real card, `suggestion`
// is a proposed corrected description. This workflow does NOT edit the const; the
// human reviews the flagged report and applies fixes.
//
// Usage: pass the authored pairs as args:
//   Workflow({ scriptPath: "<this file>", args: { pairs: [{slug, description}, ...] } })

export const meta = {
  name: 'audit-otag-descriptions',
  description: 'Adversarially re-check authored oracle-tag descriptions against real card oracle text, flag inaccuracies with examples',
  phases: [
    { title: 'Audit', detail: 'opus judge re-reads oracle text, flags inaccurate descriptions' },
  ],
}

// EDIT THIS: absolute path to <repo>/zerver/.env on the machine you run on.
const ENV = '/Users/scottyrayfermo/Developer/zwipe/zerver/.env'

const AUDIT_SCHEMA = {
  type: 'object',
  properties: {
    items: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          slug: { type: 'string' },
          verdict: { type: 'string', enum: ['accurate', 'suspect', 'wrong'] },
          issue: { type: 'string' },
          example: { type: 'string' },
          suggestion: { type: 'string' },
        },
        required: ['slug', 'verdict', 'issue', 'example', 'suggestion'],
      },
    },
  },
  required: ['items'],
}

// Workflow args arrive as a JSON STRING, not an object — parse defensively.
const A = typeof args === 'string' ? JSON.parse(args) : (args || {})
const PAIRS = A.pairs || []
const CHUNK = 8
const chunks = []
for (let i = 0; i < PAIRS.length; i += CHUNK) chunks.push(PAIRS.slice(i, i + CHUNK))

function groundingCmd(slugs) {
  const arr = slugs.map(s => `'${s}'`).join(',')
  return `export DATABASE_URL="$(grep '^DATABASE_URL=' ${ENV} | cut -d= -f2-)"
psql "$DATABASE_URL" -t -A -c "SELECT json_agg(row_to_json(r)) FROM (SELECT t.slug, cnt.n AS pop, x.name, x.type_line AS type, LEFT(x.oracle_text,300) AS text FROM (SELECT unnest(ARRAY[${arr}]::text[]) AS slug) t JOIN (SELECT oracle_tag, COUNT(DISTINCT oracle_id) n FROM card_oracle_tags GROUP BY oracle_tag) cnt ON cnt.oracle_tag=t.slug JOIN LATERAL (SELECT DISTINCT ON (sd.oracle_id) sd.name, sd.type_line, sd.oracle_text FROM card_oracle_tags c JOIN scryfall_data sd ON sd.oracle_id=c.oracle_id WHERE c.oracle_tag=t.slug ORDER BY sd.oracle_id LIMIT 8) x ON true) r;"`
}

const STYLE = `These are user-facing descriptions of Magic: The Gathering "oracle tags" (Scryfall community functional tags). Each should be ONE short plain-English sentence describing what a card with the tag DOES, accurate to the real cards, present tense, no em dashes, no links/URLs, no "&" (write "and"). Sibling precision matters: "gives-X" grants to OTHERS, "gains-X"/"-self" is about ITSELF, "-to-all" hits your whole team, "non-X" is the inverse, "hate-X" punishes/answers X (usually any player's, not just opponents'), "typal-X"/"synergy-X" reward X. Many tags are keyword MECHANICS. If a slug NAME is misleading, the description should trust the CARDS, not the name.`

function auditPrompt(chunk) {
  const listing = chunk.map(p => `- ${p.slug}: "${p.description}"`).join('\n')
  const slugs = chunk.map(p => p.slug)
  return `${STYLE}

You are an ADVERSARIAL AUDITOR (a meticulous MTG rules expert). Below are ${chunk.length} oracle-tag slugs with their CURRENT authored descriptions. Your job is to try to FIND INACCURACIES by comparing each to the real cards that carry the tag.

${listing}

STEP 1 - Pull the real cards. Run this bash and read the JSON (array of {slug, pop, name, type, text}; "text" is real card oracle text, up to 8 cards per tag):
\`\`\`
${groundingCmd(slugs)}
\`\`\`
Retry once if it fails.

STEP 2 - For EACH slug, read the example cards' oracle text and judge the CURRENT description. Actively try to break it. Flag it if it is factually wrong, misleading, overspecified or underspecified (e.g. says "opponent" when it's "any player", "creatures" when it's "permanents", "your creatures" when it's "creatures", "combat damage" when it's all damage), confuses a sibling (gives vs gains, to-all, non-, hate), falls for a slug-name trap, or breaks a STYLE RULE.

Return the structured object: one item per slug with:
- slug
- verdict: "accurate" (description is correct and clear), "suspect" (minor imprecision or unclear), or "wrong" (factually inaccurate or misleading)
- issue: one short phrase on what's wrong, or "" if accurate
- example: a real card name plus the relevant oracle snippet that proves the issue (or supports it), or "" if accurate
- suggestion: a corrected one-sentence description following the STYLE, or "" if accurate

Cover every slug, no extras. Be strict but fair: only flag genuine problems.`
}

const results = await parallel(
  chunks.map((chunk, idx) => () =>
    agent(auditPrompt(chunk), { label: `audit:${idx}`, phase: 'Audit', schema: AUDIT_SCHEMA, model: 'opus', effort: 'high' })
  )
)

const flat = results.filter(Boolean).flatMap(r => (r && r.items) || [])
const wrong = flat.filter(i => i.verdict === 'wrong')
const suspect = flat.filter(i => i.verdict === 'suspect')
log(`audited ${flat.length}: ${flat.length - wrong.length - suspect.length} accurate, ${suspect.length} suspect, ${wrong.length} wrong`)
// `audited` is the exact list of slugs that got a verdict, so a partial run (some
// chunks failing anywhere, not just the tail) can be recorded precisely.
return { total: flat.length, wrong, suspect, audited: flat.map(i => i.slug) }
