// app.zwipe.net answers a path no route of the app matches with a 404,
// still serving the app's page, which draws its own Page not found.
//
// Cloudflare Pages serves index.html with a 200 for any path it has no file
// for, so the app's routes load as a single-page app. This runs before every
// request (the shared deck link's Function included, which it passes on to):
// a path that matches a route, and every file, keeps its answer; the app's
// page for any other path goes out with a 404.
//
// ROUTES is the app's Route enum, each route's path with `:name` for a
// segment the route reads, then its redirects; a test in the router
// (zwiper-web/src/lib/inbound/router.rs) holds it equal to the enum.

const ROUTES = [
  "/",
  "/:owner/deck/:slug/:token",
  "/:owner/deck/:slug/:token/export",
  "/about",
  "/account",
  "/activity",
  "/changelog",
  "/config",
  "/decks",
  "/decks/:deck_id",
  "/decks/:deck_id/add",
  "/decks/:deck_id/cards",
  "/decks/:deck_id/edit",
  "/decks/:deck_id/export",
  "/decks/:deck_id/import",
  "/decks/:deck_id/remove",
  "/decks/new",
  "/forgot-password",
  "/guides",
  "/guides/:slug",
  "/login",
  "/maybeboard",
  "/playstyle",
  "/privacy",
  "/register",
  "/settings",
];

/** A path's segments, with no empty ones from a leading or trailing slash. */
function segments(path) {
  return path.split("/").filter((part) => part !== "");
}

const PATTERNS = ROUTES.map(segments);

/** Whether `path` matches one of the app's routes. */
function routed(path) {
  const parts = segments(path);
  return PATTERNS.some(
    (pattern) =>
      pattern.length === parts.length &&
      pattern.every((part, i) => part.startsWith(":") || part === parts[i]),
  );
}

export async function onRequest(context) {
  const response = await context.next();
  if (routed(new URL(context.request.url).pathname)) {
    return response;
  }
  const type = response.headers.get("content-type") || "";
  if (response.status === 200 && type.includes("text/html")) {
    return new Response(response.body, {
      status: 404,
      statusText: "Not Found",
      headers: response.headers,
    });
  }
  return response;
}
