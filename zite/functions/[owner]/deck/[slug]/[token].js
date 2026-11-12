// Cloudflare Pages Function for /:owner/deck/:slug/:token on zwipe.net,
// a shared deck's owner and slug link: the same redirect to the web app as
// /deck/:token.
export { onRequest } from "../../../deck/[token].js";
