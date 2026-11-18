// wire-diff's comparison: what differs between the two servers' replies to one request, with
// the values that change run to run masked, and which differences approved.json lets through.
//
// Paths are JSON pointers ("/entries/0/deck_card/quantity"). In normalize.json and
// approved.json a "*" segment stands for any one segment ("/entries/*/deck_card/flagged").

/** A key as one JSON pointer segment (RFC 6901): "~" as "~0", "/" as "~1". */
const escape = (key) => String(key).replaceAll("~", "~0").replaceAll("/", "~1");

/** The JSON pointer segments of `path`, unescaped. */
const segments = (path) =>
  path === "" ? [] : path.split("/").slice(1).map((s) => s.replaceAll("~1", "/").replaceAll("~0", "~"));

/** Whether `path` matches `pattern`, segment by segment, a "*" matching any one. */
export const matches = (pattern, path) => {
  const want = segments(pattern);
  const got = segments(path);
  return want.length === got.length && want.every((w, i) => w === "*" || w === got[i]);
};

const kindOf = (v) => (v === null ? "null" : Array.isArray(v) ? "array" : typeof v);

/** What a masked value reads as: its kind kept, so a type change still shows. */
const masked = (v) => `<masked:${kindOf(v)}>`;

/**
 * `body` with every value at a `rules.mask` path replaced by a marker that keeps its kind, and every array at a
 * `rules.sort` path sorted by its items' `by` field, so two runs of one request compare equal.
 */
export const normalize = (body, rules = {}) => {
  const mask = rules.mask ?? [];
  const sort = rules.sort ?? [];
  const walk = (value, path) => {
    if (mask.some((m) => matches(m, path))) return masked(value);
    if (Array.isArray(value)) {
      const items = value.map((item, i) => walk(item, `${path}/${i}`));
      const order = sort.find((s) => matches(s.path, path));
      if (!order) return items;
      const key = (item) => JSON.stringify(item?.[order.by] ?? null);
      return [...items].sort((a, b) => (key(a) < key(b) ? -1 : key(a) > key(b) ? 1 : 0));
    }
    if (value !== null && typeof value === "object") {
      return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, walk(v, `${path}/${escape(k)}`)]));
    }
    return value;
  };
  return walk(body, "");
};

/**
 * Every difference between the main server's reply and the candidate's, `{status, body}` each,
 * sorted by path: "status", then per field "added", "removed", "type" (a different JSON type)
 * or "value".
 */
export const diff = (main, cand) => {
  const found = [];
  const walk = (a, b, path) => {
    if (kindOf(a) !== kindOf(b)) {
      found.push({ path, kind: "type", main: a, cand: b });
    } else if (kindOf(a) === "array" || kindOf(a) === "object") {
      const keys = kindOf(a) === "array"
        ? [...Array(Math.max(a.length, b.length)).keys()].map(String)
        : [...new Set([...Object.keys(a), ...Object.keys(b)])];
      for (const k of keys) {
        const inA = kindOf(a) === "array" ? Number(k) < a.length : Object.hasOwn(a, k);
        const inB = kindOf(b) === "array" ? Number(k) < b.length : Object.hasOwn(b, k);
        const at = `${path}/${escape(k)}`;
        if (inA && !inB) found.push({ path: at, kind: "removed", main: a[k], cand: undefined });
        else if (!inA && inB) found.push({ path: at, kind: "added", main: undefined, cand: b[k] });
        else walk(a[k], b[k], at);
      }
    } else if (a !== b) {
      found.push({ path, kind: "value", main: a, cand: b });
    }
  };
  walk(main.body, cand.body, "");
  found.sort((x, y) => (x.path < y.path ? -1 : x.path > y.path ? 1 : 0));
  return main.status === cand.status
    ? found
    : [{ path: "", kind: "status", main: main.status, cand: cand.status }, ...found];
};

/** Whether `entry` is a usable approval: every field filled in. */
const complete = (entry) =>
  ["route", "path", "kind", "reason", "approved_by"].every((k) => typeof entry[k] === "string" && entry[k].trim() !== "");

/**
 * Sorts every difference in `results` (`[{step: {name, route}, diffs}]`) into approved (an entry
 * in `approved` names its route, path and kind) and unapproved. Lists the entries missing a
 * field as incomplete, and the complete ones that matched nothing as stale.
 */
export const judge = (results, approved) => {
  const usable = approved.filter(complete);
  const used = new Set();
  const verdict = { approved: [], unapproved: [], stale: [], incomplete: approved.filter((a) => !complete(a)) };
  for (const { step, diffs } of results) {
    for (const d of diffs) {
      const entry = usable.find((a) => a.route === step.route && a.kind === d.kind && matches(a.path, d.path));
      const row = { step: step.name, route: step.route, ...d };
      if (entry) {
        used.add(entry);
        verdict.approved.push({ ...row, reason: entry.reason });
      } else {
        verdict.unapproved.push(row);
      }
    }
  }
  verdict.stale = usable.filter((a) => !used.has(a));
  return verdict;
};
