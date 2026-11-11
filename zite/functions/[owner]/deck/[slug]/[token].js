// Cloudflare Pages Function for GET /:owner/deck/:slug/:token on zwipe.net,
// a shared deck's owner and slug link: the same handler as /deck/:token,
// which reads only the token.
export { onRequestGet } from "../../../deck/[token].js";
