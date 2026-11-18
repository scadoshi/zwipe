// Replays requests.json against two zervers, main and the candidate, step by step, and fails
// unless every difference in their replies is listed in approved.json.
//
//   node replay.mjs <main-url> <cand-url> [requests.json] [approved.json] [normalize.json]
//
// A step sends one request to main, then the same request to the candidate. Each server keeps
// its own captured values, tokens and cookies, because rows a step creates get different ids on
// each side. A step:
//   {"name": "fetch deck", "as": "tester", "method": "GET", "route": "/api/deck/{deck_id}",
//    "path": "/api/deck/{{deck}}", "body": {...}, "query": "?undo=true",
//    "capture": {"deck": "/0/id"}, "mask": ["/id"], "sort": [{"path": "", "by": "/scryfall_data/id"}]}
// "as" names an account signed in by an earlier {"login": ...} step, or "none". "route" is the
// route's template, which approved.json matches on. "{{name}}" in "path" or "body" is a value an
// earlier step captured on that same server. "mask" adds to normalize.json's masks for this
// step: values that differ between the servers by design, such as a row the step creates.
// "sort" sorts an array whose order the server doesn't fix (ties in a search) before comparing;
// only for those, since an order that is the contract has to compare as it comes. "shape": true
// compares the reply's field names and types only, for a reply whose values change on nearly
// every commit (the changelog).
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { alignCards, diff, judge, normalize, shapes } from "./lib.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const [, , mainUrl, candUrl, requestsFile, approvedFile, normalizeFile] = process.argv;
if (!mainUrl || !candUrl) {
  console.error("usage: node replay.mjs <main-url> <cand-url> [requests.json] [approved.json] [normalize.json]");
  process.exit(2);
}
const load = (file, fallback) => JSON.parse(readFileSync(file ?? join(here, fallback), "utf8"));
const script = load(requestsFile, "requests.json");
const approved = load(approvedFile, "approved.json");
const rules = load(normalizeFile, "normalize.json");

/** One server: its base URL, the values captured from its replies, and its signed-in accounts. */
const side = (base) => ({ base, vars: {}, tokens: {}, cookies: {} });
const servers = { main: side(mainUrl), cand: side(candUrl) };

/** `template` with every {{name}} filled from `vars`. */
const fill = (template, vars) =>
  template.replace(/\{\{([a-z_0-9]+)\}\}/g, (_, name) => {
    if (!(name in vars)) throw new Error(`no captured value named ${name}`);
    return vars[name];
  });

/** The value at a JSON pointer in `body`. */
const at = (body, pointer) =>
  pointer === "" ? body : pointer.split("/").slice(1).reduce((v, k) => (v == null ? undefined : v[k]), body);

/** The Origin every request carries; run.sh lists it in both servers' ALLOWED_ORIGINS. */
const ORIGIN = process.env.WIRE_ORIGIN ?? "http://wire-diff.localhost";

const send = async (server, step, index) => {
  // The rate limits key on CF-Connecting-IP: each step comes from an address of its own, the
  // same on both servers, so a limit spent by an earlier step never answers a later one.
  const headers = {
    "content-type": "application/json",
    origin: ORIGIN,
    "cf-connecting-ip": `10.${(index >> 16) & 255}.${(index >> 8) & 255}.${index & 255}`,
  };
  const who = step.as ?? "none";
  if (who !== "none") {
    if (server.tokens[who]) headers.authorization = `Bearer ${server.tokens[who]}`;
    if (server.cookies[who]) headers.cookie = server.cookies[who];
  }
  const body = step.body === undefined ? undefined : fill(JSON.stringify(step.body), server.vars);
  const res = await fetch(server.base + fill(step.path, server.vars) + (step.query ?? ""), {
    method: step.method,
    headers,
    body,
  });
  const text = await res.text();
  let parsed;
  try {
    parsed = text === "" ? null : JSON.parse(text);
  } catch {
    parsed = text;
  }
  const setCookie = res.headers.getSetCookie?.() ?? [];
  if (step.login && setCookie.length) server.cookies[step.login] = setCookie.map((c) => c.split(";")[0]).join("; ");
  return { status: res.status, body: parsed };
};

const results = [];
for (const [index, raw] of script.steps.entries()) {
  const step = { as: "none", ...raw };
  const replies = {};
  for (const [name, server] of Object.entries(servers)) {
    replies[name] = await send(server, step, index + 1);
    // Captured from the reply with its unordered arrays sorted, so both servers take the same row.
    const sorted = normalize(replies[name].body, { sort: [...(rules.sort ?? []), ...(step.sort ?? [])] });
    for (const [key, pointer] of Object.entries(step.capture ?? {})) {
      const value = at(sorted, pointer);
      if (value !== undefined) server.vars[key] = typeof value === "string" ? value : JSON.stringify(value);
    }
    if (step.login && step.token) server.tokens[step.login] = at(replies[name].body, step.token);
  }
  const stepRules = {
    mask: [...(rules.mask ?? []), ...(step.mask ?? [])],
    sort: [...(rules.sort ?? []), ...(step.sort ?? [])],
  };
  const [mainBody, candBody] = alignCards(
    ...(step.shape
      ? shapes(normalize(replies.main.body, stepRules), normalize(replies.cand.body, stepRules))
      : [normalize(replies.main.body, stepRules), normalize(replies.cand.body, stepRules)]),
    approved,
  );
  const found = diff({ status: replies.main.status, body: mainBody }, { status: replies.cand.status, body: candBody });
  results.push({ step: { name: step.name, route: step.route }, diffs: found, main: mainBody, cand: candBody });
  const mark = found.length === 0 ? "same" : `${found.length} difference(s)`;
  console.log(`${String(replies.main.status).padEnd(3)} ${step.method.padEnd(6)} ${step.route}  ${step.name}: ${mark}`);
}

const verdict = judge(results, approved);
const show = (v) => (v === undefined ? "(absent)" : JSON.stringify(v)?.slice(0, 120));
for (const d of verdict.approved) console.log(`approved  ${d.route} ${d.path} ${d.kind}: ${d.reason}`);
for (const a of verdict.incomplete) {
  console.log(
    a.data === undefined
      ? `incomplete ${a.route ?? "?"} ${a.path ?? "?"}: an approval needs route, path, kind, reason and approved_by`
      : `incomplete data ${a.data}: a data change needs data "printings", oracle_ids (a list, or "*"), reason and approved_by`,
  );
}
for (const a of verdict.stale) {
  console.log(
    a.data === undefined
      ? `stale     ${a.route} ${a.path} ${a.kind}: nothing differs there now; remove it from approved.json`
      : `stale     data ${a.data} ${a.route ?? "(any route)"}: no card differs that way now; remove it from approved.json`,
  );
}
for (const d of verdict.unapproved) {
  console.log(`UNAPPROVED ${d.kind} at ${d.route} ${d.path || "(status)"} (${d.step})\n  main: ${show(d.main)}\n  cand: ${show(d.cand)}`);
}
console.log(
  `${results.length} steps, ${verdict.approved.length} approved, ${verdict.unapproved.length} unapproved, ${verdict.stale.length} stale`,
);
process.exit(verdict.unapproved.length === 0 ? 0 : 1);
