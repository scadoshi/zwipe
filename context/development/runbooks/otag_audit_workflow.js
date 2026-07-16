// Oracle-tag description AUDIT workflow (for the `Workflow` tool).
// Companion to otag_authoring_workflow.js / otag_description_authoring.md.
//
// Two-stage QA that does NOT edit the const (findings only, human-in-the-loop):
//   1. AUDIT   - an opus agent re-pulls the real cards carrying each tag and
//      adversarially checks the ALREADY-AUTHORED description, flagging inaccuracies.
//   2. VERIFY  - every flag is independently re-checked by a second opus agent that
//      is biased to OVERTURN it: a flag only survives if the card data clearly proves
//      it. This catches false positives (a suggested "fix" is itself a change, so it
//      must be verified too), especially cost/color/rarity/hybrid claims.
//
// Returns { total, wrong, suspect, overturned, audited } where wrong/suspect are the
// VERIFIED-surviving flags (suggestion = the verifier's final corrected text),
// overturned lists flags the verify stage rejected, and audited is every slug that
// got an audit verdict (for resume tracking, robust to partial runs).
//
// Usage: Workflow({ scriptPath: "<this file>", args: { pairs: [{slug, description}, ...] } })

export const meta = {
  name: 'audit-otag-descriptions',
  description: 'Audit authored oracle-tag descriptions against real cards, then independently verify each flag before reporting it',
  phases: [
    { title: 'Audit', detail: 'opus judge re-reads oracle text + card data, flags inaccurate descriptions' },
    { title: 'Verify', detail: 'second opus judge tries to refute each flag; only proven ones survive' },
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

const VERIFY_SCHEMA = {
  type: 'object',
  properties: {
    items: {
      type: 'array',
      items: {
        type: 'object',
        properties: {
          slug: { type: 'string' },
          verdict: { type: 'string', enum: ['upheld', 'overturned'] },
          note: { type: 'string' },
          suggestion: { type: 'string' },
        },
        required: ['slug', 'verdict', 'note', 'suggestion'],
      },
    },
  },
  required: ['items'],
}

// Workflow args arrive as a JSON STRING, not an object — parse defensively.
const A = typeof args === 'string' ? JSON.parse(args) : (args || {})
const PAIRS = A.pairs || []
const CHUNK = 8
const VERIFY_CHUNK = 6
const chunks = []
for (let i = 0; i < PAIRS.length; i += CHUNK) chunks.push(PAIRS.slice(i, i + CHUNK))

// Grounding pull. Includes the card DATA an accuracy judge needs so it never has to
// GUESS: cost (mana cost), colors, rarity, mv (mana value), plus name/type/oracle
// text. `pop` is the true number of cards carrying the tag; up to 12 are sampled so
// "whole cycle is uniform" claims can be checked against more of the set.
function groundingCmd(slugs) {
  const arr = slugs.map(s => `'${s}'`).join(',')
  return `export DATABASE_URL="$(grep '^DATABASE_URL=' ${ENV} | cut -d= -f2-)"
psql "$DATABASE_URL" -t -A -c "SELECT json_agg(row_to_json(r)) FROM (SELECT t.slug, cnt.n AS pop, x.name, x.mana_cost AS cost, x.colors, x.rarity, x.cmc AS mv, x.type_line AS type, LEFT(x.oracle_text,300) AS text FROM (SELECT unnest(ARRAY[${arr}]::text[]) AS slug) t JOIN (SELECT oracle_tag, COUNT(DISTINCT oracle_id) n FROM card_oracle_tags GROUP BY oracle_tag) cnt ON cnt.oracle_tag=t.slug JOIN LATERAL (SELECT DISTINCT ON (sd.oracle_id) sd.name, sd.mana_cost, sd.colors, sd.rarity, sd.cmc, sd.type_line, sd.oracle_text FROM card_oracle_tags c JOIN scryfall_data sd ON sd.oracle_id=c.oracle_id WHERE c.oracle_tag=t.slug ORDER BY sd.oracle_id LIMIT 12) x ON true) r;"`
}

const STYLE = `These are user-facing descriptions of Magic: The Gathering "oracle tags" (Scryfall community functional tags). Each should be ONE short plain-English sentence describing what a card with the tag DOES, accurate to the real cards, present tense, no em dashes, no links/URLs, no "&" (write "and"). Sibling precision matters: "gives-X" grants to OTHERS, "gains-X"/"-self" is about ITSELF, "-to-all" hits your whole team, "non-X" is the inverse, "hate-X" punishes/answers X (usually any player's, not just opponents'), "typal-X"/"synergy-X" reward X. Many tags are keyword MECHANICS. If a slug NAME is misleading, the description should trust the CARDS, not the name.`

// Evidence discipline shared by both stages: the pulled JSON carries the real fields,
// so never assert cost/color/rarity/mv from memory, and don't over-generalize a
// cycle from a partial sample.
const EVIDENCE_RULES = `EVIDENCE RULES (critical):
- Each pulled card has: name, cost (mana cost, e.g. {2}{G/U}), colors (array), rarity, mv (mana value), type, text (oracle text). USE these fields.
- NEVER assert a card's mana cost, colors, hybrid-ness, rarity, mana value, or whether it has {X} unless it is shown in the data you pulled. A {X/Y} symbol in cost means HYBRID (either color). Colors [] means colorless (e.g. devoid).
- "pop" is the TOTAL number of cards with the tag; you see up to 12. Do NOT claim a whole cycle/set is uniform ("all gold", "all creatures", "one member is X") unless every pulled card agrees AND pop is not much larger than what you see. When unsure, describe the common case rather than over-specifying.
- Any card you cite as evidence must quote its real cost or oracle text VERBATIM from the pulled data.`

function auditPrompt(chunk) {
  const listing = chunk.map(p => `- ${p.slug}: "${p.description}"`).join('\n')
  const slugs = chunk.map(p => p.slug)
  return `${STYLE}

You are an ADVERSARIAL AUDITOR (a meticulous MTG rules expert). Below are ${chunk.length} oracle-tag slugs with their CURRENT authored descriptions. Your job is to try to FIND INACCURACIES by comparing each to the real cards that carry the tag.

${listing}

STEP 1 - Pull the real cards. Run this bash and read the JSON (array of {slug, pop, name, cost, colors, rarity, mv, type, text}; up to 12 cards per tag):
\`\`\`
${groundingCmd(slugs)}
\`\`\`
Retry once if it fails.

${EVIDENCE_RULES}

STEP 2 - For EACH slug, read the card data and judge the CURRENT description. Actively try to break it. Flag it if it is factually wrong, misleading, overspecified or underspecified (e.g. says "opponent" when it's "any player", "creatures" when it's "permanents", "your creatures" when it's "creatures", "combat damage" when it's all damage), confuses a sibling (gives vs gains, to-all, non-, hate), falls for a slug-name trap, or breaks a STYLE RULE.

Return the structured object: one item per slug with:
- slug
- verdict: "accurate" (correct and clear), "suspect" (minor imprecision or unclear), or "wrong" (factually inaccurate or misleading)
- issue: one short phrase on what's wrong, or "" if accurate
- example: a real card name plus the VERBATIM cost/oracle snippet that proves the issue, or "" if accurate
- suggestion: a corrected one-sentence description following the STYLE, or "" if accurate

Cover every slug, no extras. Be strict but fair: only flag genuine problems.`
}

function verifyPrompt(findings) {
  const listing = findings.map(f =>
    `- ${f.slug}\n    current: "${f.description}"\n    flagged as: ${f.verdict} — ${f.issue}\n    auditor example: ${f.example}\n    auditor suggested: "${f.suggestion}"`
  ).join('\n')
  const slugs = findings.map(f => f.slug)
  return `${STYLE}

You are a SKEPTICAL VERIFIER. Another auditor flagged the oracle-tag descriptions below as inaccurate. Flags are often WRONG (the auditor guessed at a card's cost/color/rarity, misread a card, or over-generalized a cycle from a partial view). Your job is to REFUTE each flag. Default to OVERTURNED: only uphold a flag if the pulled card data CLEARLY proves the current description is inaccurate.

${listing}

STEP 1 - Pull the real cards. Run this bash and read the JSON ({slug, pop, name, cost, colors, rarity, mv, type, text}; up to 12 cards per tag):
\`\`\`
${groundingCmd(slugs)}
\`\`\`
Retry once if it fails.

${EVIDENCE_RULES}

STEP 2 - For EACH slug, check whether the flag holds against the real card data. Watch for the auditor's mistakes: asserting a cost/color/rarity that the data contradicts (a {X/Y} cost is hybrid, not "both colors"; a card with {X} in cost does have X), citing a card whose real cost/text differs from what they quoted, or claiming a cycle is non-uniform on a card that actually matches.

Return one item per slug with:
- slug
- verdict: "upheld" (the flag is correct, the current description really is inaccurate) or "overturned" (the flag is unfounded, keep the current description)
- note: one short phrase citing the deciding card evidence
- suggestion: if upheld, YOUR final corrected one-sentence description (do not blindly copy the auditor's, verify it too and fix if needed); if overturned, ""

Cover every slug, no extras.`
}

// ---- Stage 1: audit ----
const auditResults = await parallel(
  chunks.map((chunk, idx) => () =>
    agent(auditPrompt(chunk), { label: `audit:${idx}`, phase: 'Audit', schema: AUDIT_SCHEMA, model: 'opus', effort: 'high' })
  )
)
const flat = auditResults.filter(Boolean).flatMap(r => (r && r.items) || [])
const descOf = Object.fromEntries(PAIRS.map(p => [p.slug, p.description]))
const flagged = flat
  .filter(i => i.verdict === 'wrong' || i.verdict === 'suspect')
  .map(i => ({ ...i, description: descOf[i.slug] || '' }))
log(`audited ${flat.length}: ${flat.length - flagged.length} accurate, ${flagged.length} flagged; verifying flags...`)

// ---- Stage 2: verify each flag (skeptic, biased to overturn) ----
const vChunks = []
for (let i = 0; i < flagged.length; i += VERIFY_CHUNK) vChunks.push(flagged.slice(i, i + VERIFY_CHUNK))
const verifyResults = await parallel(
  vChunks.map((vc, idx) => () =>
    agent(verifyPrompt(vc), { label: `verify:${idx}`, phase: 'Verify', schema: VERIFY_SCHEMA, model: 'opus', effort: 'high' })
  )
)
const verdictOf = {}
for (const r of verifyResults.filter(Boolean)) for (const v of r.items || []) verdictOf[v.slug] = v

// Partition: verified-surviving flags vs overturned. A flag whose verifier never ran
// (agent died) is surfaced anyway, marked verified:false, so nothing is silently lost.
const wrong = []
const suspect = []
const overturned = []
for (const f of flagged) {
  const v = verdictOf[f.slug]
  if (v && v.verdict === 'overturned') {
    overturned.push({ slug: f.slug, auditIssue: f.issue, verifyNote: v.note })
    continue
  }
  const out = {
    slug: f.slug,
    issue: f.issue,
    example: f.example,
    suggestion: v && v.verdict === 'upheld' && v.suggestion ? v.suggestion : f.suggestion,
    verifyNote: v ? v.note : '',
    verified: !!v,
  }
  ;(f.verdict === 'wrong' ? wrong : suspect).push(out)
}

log(`verified: ${wrong.length} wrong, ${suspect.length} suspect survived; ${overturned.length} flags overturned`)
// `audited` is every slug that got an audit verdict, so a partial run records precisely.
return { total: flat.length, wrong, suspect, overturned, audited: flat.map(i => i.slug) }
