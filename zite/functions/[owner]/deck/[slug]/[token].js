// Cloudflare Pages Function for /:owner/deck/:slug/:token on zwipe.net, a
// shared deck's owner and slug link. It answers every method, so a HEAD
// redirects too.
//
// A shared deck lives on the web app, so the link moves to the same path on
// app.zwipe.net for good, and a link pasted before the move keeps opening. A
// link-preview crawler follows the redirect and reads the web app's tags for
// the deck.

/// The web app, where a shared deck lives.
const APP = "https://app.zwipe.net";

export async function onRequest({ request }) {
  const url = new URL(request.url);
  return Response.redirect(`${APP}${url.pathname}${url.search}`, 301);
}
