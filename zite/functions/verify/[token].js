// Cloudflare Pages Function for /verify/:token on zwipe.net, and for
// /reset/:token and /confirm-email/:token, whose files re-export this one.
//
// These pages carry a per-token link from an email, so the build can't
// prerender them. With a 404.html in the site, Cloudflare answers a path it
// has no file for with that page and a 404, so this serves the site's shell
// with a 200 instead; the app reads the token and draws the page in the
// browser. It answers every method, so a HEAD gets the 200 too.
export async function onRequest({ request, env }) {
  const shell = await env.ASSETS.fetch(new URL("/", request.url));
  return new Response(shell.body, { status: 200, headers: shell.headers });
}
