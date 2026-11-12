// Cloudflare Pages Function for GET /deck/:token on zwipe.net, an older shared
// deck link, and for the owner and slug form /:owner/deck/:slug/:token, whose
// file re-exports this one.
//
// A shared deck lives on the web app, so every shared deck link here moves
// there for good: the owner and slug form to the same path on app.zwipe.net,
// and the bare token form to the owner form with a placeholder owner and slug,
// which the web app swaps for the deck's own once it loads. Phones built
// before the move still make zwipe.net links, and links already pasted keep
// opening. A link-preview crawler follows the redirect and reads the web
// app's tags for the deck.

/// The web app, where a shared deck lives.
const APP = "https://app.zwipe.net";

/// The owner and slug the bare token form takes on the way, swapped for the
/// deck's own by the web app.
const PLACEHOLDER = "/_/deck/deck";

export async function onRequestGet({ params, request }) {
  const url = new URL(request.url);
  const path = params.owner === undefined
    ? `${PLACEHOLDER}/${encodeURIComponent(String(params.token))}`
    : url.pathname;
  return Response.redirect(`${APP}${path}${url.search}`, 301);
}
