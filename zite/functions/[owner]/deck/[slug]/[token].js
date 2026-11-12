// Cloudflare Pages Function for GET /:owner/deck/:slug/:token on zwipe.net,
// a shared deck's owner and slug link: the same redirect to the web app as
// /deck/:token.
export { onRequestGet } from "../../../deck/[token].js";
