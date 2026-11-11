// Cloudflare Pages Function for GET /deck/:token on zwipe.net, and for the
// owner and slug form /:owner/deck/:slug/:token, whose file re-exports this one.
//
// Link previews (iMessage, Discord, Slack) read the page's tags without
// running the wasm app, so this serves the site shell with the shared deck's
// own title, description and image in place of the site-wide ones. The app
// still hydrates and draws the deck in the browser.
//
// - A token the API knows: the deck's tags, 200. Its image is the deck's
//   generated preview (`/api/share/deck/:token/preview.png`), versioned by the
//   layout and when the deck last changed, so a new drawing or an edit gets a
//   fresh image; from an API that predates `updated_at`, the commander's art
//   crop.
// - A malformed, revoked or unknown token: the generic tags, 404.
// - The API unreachable or failing: the generic tags, 200, so a visitor still
//   gets the app, which shows its own retry.

/// The API that answers the share token. A Pages environment variable
/// overrides it, for `wrangler pages dev` against a local zerver.
const DEFAULT_API_BASE = "https://api.zwipe.net";

/// The public site, for the canonical link and the fallback image.
const SITE = "https://zwipe.net";

/// The site-wide preview image, used when the deck has no commander art.
const DEFAULT_IMAGE = `${SITE}/assets/og-default.png`;

/// A share token is a UUID; anything else 404s without asking the API.
const TOKEN = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

/// The API's format keys with their display names, as zwipe-core's
/// `Format::display_name` gives them. zite's `og_function` test fails when
/// the two drift apart.
const FORMAT_NAMES = {
  standard: "Standard",
  future: "Future",
  historic: "Historic",
  timeless: "Timeless",
  gladiator: "Gladiator",
  pioneer: "Pioneer",
  modern: "Modern",
  legacy: "Legacy",
  pauper: "Pauper",
  vintage: "Vintage",
  penny: "Penny Dreadful",
  commander: "Commander",
  oathbreaker: "Oathbreaker",
  standardbrawl: "Standard Brawl",
  brawl: "Brawl",
  alchemy: "Alchemy",
  paupercommander: "Pauper Commander",
  duel: "Duel Commander",
  oldschool: "Old School",
  premodern: "Premodern",
  predh: "PreDH",
  explorer: "Explorer",
  historicbrawl: "Historic Brawl",
};

/// The formats whose decks lead with a commander, as `Format::has_commander`.
const COMMANDER_FORMATS = new Set([
  "commander",
  "brawl",
  "standardbrawl",
  "historicbrawl",
  "paupercommander",
  "duel",
  "predh",
  "oathbreaker",
]);

/// The longest a deck name's slug runs, as zwipe-core's `SLUG_MAX`.
const SLUG_MAX = 60;

/// Letters a slug folds by hand: what Unicode decomposition leaves whole.
const FOLDS = { "ß": "ss", "æ": "ae", "œ": "oe", "þ": "th", "ø": "o", "đ": "d", "ł": "l", "ı": "i", "ð": "d" };

/// The deck name's slug, as zwipe-core's `deck_slug` makes it: lower case,
/// accented letters folded to ASCII, every other run one dash, at most
/// `SLUG_MAX` characters cut at a dash, `deck` when nothing is left.
function deckSlug(name) {
  const folded = Array.from(String(name).toLowerCase())
    .map((c) => FOLDS[c] ?? c.normalize("NFKD").replace(/[\u0300-\u036f]/g, ""))
    .join("");
  let slug = folded.replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "");
  if (slug.length > SLUG_MAX) {
    const head = slug.slice(0, SLUG_MAX);
    const dash = head.lastIndexOf("-");
    slug = (dash > 0 ? head.slice(0, dash) : head).replace(/-+$/, "");
  }
  return slug || "deck";
}

/// The deck's canonical link: the owner and slug form, from the deck's
/// current owner and name, when the API names the owner; else the bare token
/// link.
function canonicalUrl(deck, token) {
  const lower = token.toLowerCase();
  if (typeof deck.owner !== "string" || !deck.owner) {
    return `${SITE}/deck/${lower}`;
  }
  return `${SITE}/${encodeURIComponent(deck.owner)}/deck/${deckSlug(deck.name)}/${lower}`;
}

export async function onRequestGet({ params, request, env }) {
  const shell = await env.ASSETS.fetch(new URL("/", request.url));
  const token = String(params.token);
  if (!TOKEN.test(token)) {
    return withStatus(shell, 404);
  }
  const apiBase = env.API_BASE || DEFAULT_API_BASE;
  let reply;
  try {
    reply = await fetch(`${apiBase}/api/share/deck/${token}`, {
      headers: { accept: "application/json" },
    });
  } catch {
    return shell;
  }
  if (reply.status === 404) {
    return withStatus(shell, 404);
  }
  if (!reply.ok) {
    return shell;
  }
  let deck;
  try {
    deck = await reply.json();
  } catch {
    return shell;
  }
  const meta = preview(deck, canonicalUrl(deck, token));
  const generated = previewImage(apiBase, token.toLowerCase(), deck);
  if (generated) {
    meta.image = generated;
    meta.imageSize = PREVIEW_SIZE;
  }
  return rewrite(shell, meta);
}

/// The generated preview image's size, as zerver draws it.
const PREVIEW_SIZE = { width: 1200, height: 630 };

/// The preview image's layout version, zwipe-core's `SHARE_PREVIEW_LAYOUT`.
/// Unfurlers cache the image by URL, so a new drawing needs a new `v`.
const PREVIEW_LAYOUT = 2;

/// The deck's generated preview image on the API, versioned by the layout
/// and the unix seconds of its `updated_at` (`2-1794023263`, the version
/// zerver caches for a year), or null when the payload has no `updated_at`.
function previewImage(apiBase, token, deck) {
  const changed = Date.parse(deck.updated_at || "");
  if (!Number.isFinite(changed)) {
    return null;
  }
  return `${apiBase}/api/share/deck/${token}/preview.png?v=${PREVIEW_LAYOUT}-${Math.floor(changed / 1000)}`;
}

/// The shell as it is, under another status.
function withStatus(shell, status) {
  return new Response(shell.body, { status, headers: shell.headers });
}

/// The title, description, image and link the deck's preview shows.
function preview(deck, url) {
  const name = String(deck.name || "A shared deck");
  const format = FORMAT_NAMES[deck.format] || null;
  const leads = [deck.commander, deck.partner_commander]
    .filter(Boolean)
    .map((card) => card.scryfall_data.name);
  const count = cardCount(deck);
  const cards = `${count} ${count === 1 ? "card" : "cards"}`;
  const kind = format ? `${format} deck` : "Magic: The Gathering deck";
  const led = leads.length > 0 ? ` led by ${leads.join(" and ")}` : "";
  return {
    title: `${name} | Zwipe`,
    description: `${kind}${led}, ${cards}. Shared from Zwipe.`,
    image: artCrop(deck.commander) || DEFAULT_IMAGE,
    url,
  };
}

/// The card count the share page heads with: the main board without the
/// command zone's cards, plus one for each command zone slot it lists.
function cardCount(deck) {
  const zone = [deck.commander, deck.partner_commander, deck.background, deck.signature_spell];
  const zoneOracles = new Set(
    zone.filter(Boolean).map((card) => card.scryfall_data.oracle_id).filter(Boolean),
  );
  const main = (deck.entries || [])
    .filter((e) => e.deck_card.board === "deck" && !zoneOracles.has(e.deck_card.oracle_id))
    .reduce((sum, e) => sum + Number(e.deck_card.quantity || 0), 0);
  const slots = [
    COMMANDER_FORMATS.has(deck.format) && deck.commander,
    deck.partner_commander,
    deck.background,
    deck.signature_spell,
  ].filter(Boolean).length;
  return main + slots;
}

/// The card's art crop, from the card or its front face.
function artCrop(card) {
  const data = card && card.scryfall_data;
  if (!data) {
    return null;
  }
  const front = (data.card_faces || [])[0];
  return (data.image_uris && data.image_uris.art_crop) ||
    (front && front.image_uris && front.image_uris.art_crop) ||
    null;
}

/// The shell with its title and preview tags set to `meta`.
function rewrite(shell, meta) {
  const content = (value) => ({
    element(el) {
      el.setAttribute("content", value);
    },
  });
  return new HTMLRewriter()
    .on("title", {
      element(el) {
        el.setInnerContent(meta.title);
      },
    })
    .on('meta[name="description"]', content(meta.description))
    .on('meta[property="og:title"]', content(meta.title))
    .on('meta[property="og:description"]', content(meta.description))
    .on('meta[property="og:url"]', content(meta.url))
    .on('meta[property="og:image"]', content(meta.image))
    .on('meta[name="twitter:title"]', content(meta.title))
    .on('meta[name="twitter:description"]', content(meta.description))
    .on('meta[name="twitter:image"]', content(meta.image))
    .on('link[rel="canonical"]', {
      element(el) {
        el.setAttribute("href", meta.url);
      },
    })
    .on("head", {
      element(el) {
        if (meta.imageSize) {
          el.append(
            `<meta property="og:image:width" content="${meta.imageSize.width}">` +
              `<meta property="og:image:height" content="${meta.imageSize.height}">`,
            { html: true },
          );
        }
      },
    })
    .transform(shell);
}
