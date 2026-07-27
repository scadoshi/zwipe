---
name: human-copy
description: |
  Write Zwipe user-facing copy that doesn't read as AI-generated: video VO
  scripts, social posts, store listings, on-screen captions, site copy. Load
  BEFORE drafting any marketing/social/VO text, or when asked to review copy
  for AI tells. Distilled from blader/humanizer (MIT, Wikipedia's "Signs of AI
  writing") and the StoryScope paper (arXiv:2604.03136) on AI narrative tells.
---

# Human copy for Zwipe

Two failure layers, both obvious to readers: sentence-level tells (word choice,
rhythm, punctuation) and discourse-level tells (structure, how much gets
explained, whether everything resolves). Fix both or it still smells generated.

## House rules (non-negotiable, override everything)

- No em or en dashes anywhere in user-facing copy. Recast with comma, colon,
  period, or parentheses.
- Sentence case. "Zwipe" capitalized. Features are not branded "Zwipe X".
- No glow words: seamless, effortless, elevate, unleash, supercharge, stunning,
  beautiful, magical, powerful, game-changing.
- Store-listing copy stays generic where the copycat rules demand it (the
  Android listing says "tags," not "Scryfall" or MTG-specific terms).
- Post copy in files: one paragraph per line, no hard wraps.

## Sentence-level tells (the humanizer core, marketing cut)

Kill on sight:

- **Rule of three.** "Fast, simple, and powerful." Two items, or four, or one.
  If a triple survives, it's because the facts really are three.
- **Negative parallelism.** "It's not just a deck builder, it's..." and
  "No ads. No accounts. No limits." staccato stacks. One plain sentence instead.
- **"Whether you're X or Y" audience scaffolding.** "Whether you're a cEDH
  grinder or a kitchen-table brewer..." Name one real situation instead.
- **False ranges.** "From budget brews to competitive powerhouses." Same fix.
- **Copula avoidance.** "boasts," "features," "offers," "serves as." Say "is"
  and "has."
- **Manufactured punchlines.** A run of clipped fragments engineered to sound
  quotable. One short sentence lands; three in a row is a robot flexing.
- **Signposting.** "Let's take a look." "Here's the thing." Just show it.
- **Aphorism formulas.** "X is the Y of Z." Cut.
- **AI vocabulary.** delve, elevate, streamline, robust, intuitive, vibrant,
  comprehensive, empower, transform, journey. Also exclamation inflation.
- **Generic upbeat closers.** "...and so much more!" "Your perfect deck awaits."
  End on the last concrete thing, not a send-off.

## Discourse-level tells (the StoryScope lessons, script cut)

AI-written scripts cluster: tidy single-track arcs, flat escalation, and the
theme spelled out at the end. Human scripts are messier and more specific.

- **Don't caption the moral.** If the toast fires on screen, the viewer got it.
  "It keeps you in control of your budget" is the AI move; "Watch. Seventy five
  percent." is the human one. Show the thing, say the fact, skip the meaning.
- **Not every beat resolves.** End a script on a fact, a number, or a use case,
  not a summary. It's fine for the last line to just stop.
- **Vary the rhythm.** Beats of different lengths, one aside, one dry line.
  Uniform beat-caption-beat-caption cadence reads generated even when each
  line is clean.
- **Specificity is the strongest human signal.** A real commander (Krenko, not
  "your commander"), a real dollar amount, a real tag slug, an opinion about a
  theme. LLMs round off to the generic; humans hoard particulars.
- **Allow one imperfection.** A hedge, a joke at the product's expense, a
  preference stated as preference. Clean takes on everything = generated.

## Voice for VO and posts

The channel is one person building in public. First person, present tense,
talking not narrating. The test: read it aloud; if it sounds like a landing
page, rewrite it as what you'd actually say showing a friend your phone.
Opinions welcome ("this theme hurts"), invented facts never: numbers, card
names, and behaviors must be real and verifiable in the app.

Captions are a different genre from VO: fragments are fine there (they're
labels, not prose), but the same word bans apply, and a caption must never
explain what the footage already shows.

## Delivery checklist

Before handing copy over, verify:

1. Zero em/en dashes, zero banned words, sentence case throughout.
2. No triple lists, no "not just X," no "whether you're," no false ranges.
3. At least two concrete specifics (name, number, card, price) per script.
4. The ending is a fact or a use case, not a moral.
5. Read aloud: any sentence you wouldn't say to a friend gets rewritten.
6. Nothing claimed that the app doesn't visibly do in the current build.
