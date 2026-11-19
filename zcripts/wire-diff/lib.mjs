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

/**
 * The value at `by` in `item`: a JSON pointer ("/scryfall_data/id"), or a plain key for a field
 * of the item itself.
 */
export const valueAt = (item, by) =>
  by.startsWith("/") ? segments(by).reduce((v, k) => (v == null ? undefined : v[k]), item) : item?.[by];

const kindOf = (v) => (v === null ? "null" : Array.isArray(v) ? "array" : typeof v);

/** What a masked value reads as: its kind kept, so a type change still shows. */
const masked = (v) => `<masked:${kindOf(v)}>`;

/**
 * `body` with every value at a `rules.mask` path replaced by a marker that keeps its kind, and
 * every array at a `rules.sort` path sorted by its items' `by` field (a key or a JSON pointer),
 * so two runs of one request compare equal.
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
      const key = (item) => JSON.stringify(valueAt(item, order.by) ?? null);
      return [...items].sort((a, b) => (key(a) < key(b) ? -1 : key(a) > key(b) ? 1 : 0));
    }
    if (value !== null && typeof value === "object") {
      return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, walk(v, `${path}/${escape(k)}`)]));
    }
    return value;
  };
  return walk(body, "");
};

/** Two shapes as one: objects take every key, lists merge their items, unlike kinds join. */
const merge = (a, b) => {
  if (a === undefined) return b;
  if (b === undefined) return a;
  if (Array.isArray(a) && Array.isArray(b)) return [merge(a[0], b[0])].filter((s) => s !== undefined);
  if (kindOf(a) === "object" && kindOf(b) === "object") {
    return Object.fromEntries([...new Set([...Object.keys(a), ...Object.keys(b)])].sort().map((k) => [k, merge(a[k], b[k])]));
  }
  if (JSON.stringify(a) === JSON.stringify(b)) return a;
  const names = (s) => (typeof s === "string" ? s.split("|") : [kindOf(s)]);
  return [...new Set([...names(a), ...names(b)])].sort().join("|");
};

/**
 * `value`'s shape: field names and JSON kinds, without the values or how many items a list holds.
 * A list's items merge into one shape. For a reply whose values change on nearly every commit
 * (the changelog) but whose fields must not.
 */
export const shapeOf = (value) => {
  if (Array.isArray(value)) return [value.map(shapeOf).reduce(merge, undefined)].filter((s) => s !== undefined);
  if (kindOf(value) === "object") {
    return Object.fromEntries(Object.entries(value).map(([k, v]) => [k, shapeOf(v)]));
  }
  return kindOf(value);
};

/**
 * The shapes of `main` and `cand` ([`shapeOf`]), with an empty list on either side taking the
 * other side's list: an empty list says nothing about its items, so it matches any list.
 */
export const shapes = (main, cand) => {
  const align = (a, b) => {
    if (Array.isArray(a) && Array.isArray(b)) {
      if (a.length === 0 || b.length === 0) return a.length === 0 ? [b, b] : [a, a];
      const [x, y] = align(a[0], b[0]);
      return [[x], [y]];
    }
    if (kindOf(a) === "object" && kindOf(b) === "object") {
      const x = { ...a };
      const y = { ...b };
      for (const k of Object.keys(a)) if (Object.hasOwn(b, k)) [x[k], y[k]] = align(a[k], b[k]);
      return [x, y];
    }
    return [a, b];
  };
  return align(shapeOf(main), shapeOf(cand));
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

/** The kinds of data change an approval can name: "printings", another printing of the same card. */
const DATA_KINDS = ["printings"];

/** Whether `entry` is a usable data change: a kind wire-diff knows, its oracle ids and why. */
const completeData = (entry) =>
  DATA_KINDS.includes(entry.data) &&
  (entry.oracle_ids === "*" ||
    (Array.isArray(entry.oracle_ids) && entry.oracle_ids.length > 0 && entry.oracle_ids.every((id) => typeof id === "string"))) &&
  ["reason", "approved_by"].every((k) => typeof entry[k] === "string" && entry[k].trim() !== "") &&
  (entry.route === undefined || typeof entry.route === "string");

/**
 * Whether `entry` is a usable approval: every field of a path entry or a data change filled in. A
 * path entry's `path` may be "", the reply's root, which a new route's status and body differ at.
 */
const complete = (entry) =>
  entry.data !== undefined
    ? completeData(entry)
    : typeof entry.path === "string" &&
      ["route", "kind", "reason", "approved_by"].every((k) => typeof entry[k] === "string" && entry[k].trim() !== "");

/** The oracle id of a card object: its own, its `scryfall_data`'s, or its `card`'s. */
const oracleOf = (value, depth = 0) => {
  if (kindOf(value) !== "object" || depth > 2) return undefined;
  if (typeof value.oracle_id === "string") return value.oracle_id;
  return oracleOf(value.scryfall_data, depth + 1) ?? oracleOf(value.card, depth + 1);
};

/**
 * The card a difference at `path` sits in: the oracle id of its nearest enclosing object that is
 * a card on both sides, when both sides name the same card there; `undefined` outside any card or
 * where the two sides hold different cards (a reorder, not another printing).
 */
const cardOf = (main, cand, path) => {
  const parts = segments(path);
  const under = (body, depth) => parts.slice(0, depth).reduce((v, k) => (v == null ? undefined : v[k]), body);
  for (let depth = parts.length - 1; depth >= 0; depth--) {
    const a = oracleOf(under(main, depth));
    const b = oracleOf(under(cand, depth));
    if (a !== undefined || b !== undefined) return a === b ? a : undefined;
  }
  return undefined;
};

/**
 * A printing id (a UUID), the first one in `value` when it's a string: an id, or a URL holding
 * one. A string holding two takes the first, which a URL keyed by something else first would get
 * wrong; no reply has one today.
 */
const printingIn = (value) => (typeof value === "string" ? value.match(/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/)?.[0] : undefined);

/**
 * Every printing id the replies in `results` name beside its card's oracle id, as a map to that
 * oracle id: a card's own `id` next to its `oracle_id`, and a row's `scryfall_data_id` next to its.
 */
const printingsOf = (results) => {
  const found = new Map();
  const walk = (v) => {
    if (kindOf(v) === "array") v.forEach(walk);
    if (kindOf(v) !== "object") return;
    if (typeof v.oracle_id === "string") {
      for (const key of ["id", "scryfall_data_id"]) if (typeof v[key] === "string") found.set(v[key], v.oracle_id);
    }
    Object.values(v).forEach(walk);
  };
  for (const { main, cand } of results) [main, cand].forEach(walk);
  return found;
};

/**
 * The card fields zwipe passes through from Scryfall untyped (`serde_json::Value` in zwipe-core):
 * which links they hold depends on the printing, so their keys are not zwipe's contract.
 */
const UNTYPED_MAPS = ["related_uris", "purchase_uris"];

/**
 * Whether difference `d` is one another printing of the same card can make: a value, a field null
 * on one side only (no flavor text, no image on a double-faced printing), an item more or fewer
 * in a list (another face, another tag), or a link more or fewer in an [`UNTYPED_MAPS`] field. A
 * key added, dropped or renamed anywhere else, or a type that changes otherwise, is the card's
 * shape changing, which no data change covers.
 */
const printingMayDiffer = (d) => {
  if (d.kind === "value") return true;
  if (d.kind === "type") return d.main === null || d.cand === null;
  if (d.kind === "added" || d.kind === "removed") {
    const parts = segments(d.path);
    return /^\d+$/.test(parts.at(-1) ?? "") || UNTYPED_MAPS.includes(parts.at(-2) ?? "");
  }
  return false;
};

/**
 * Whether data change `entry` covers difference `d` on `route`, the replies `main` and `cand`:
 * inside a listed card, the same card on both sides, a difference another printing may make
 * ([`printingMayDiffer`]), or a value outside any card that names a printing of the same listed
 * card on each side (`printings` maps printing ids to oracle ids).
 */
const coversData = (entry, route, d, main, cand, printings) => {
  if (d.kind === "status" || (entry.route !== undefined && entry.route !== route)) return false;
  const listed = (card) => card !== undefined && (entry.oracle_ids === "*" || entry.oracle_ids.includes(card));
  const card = cardOf(main, cand, d.path);
  if (card !== undefined) return listed(card) && printingMayDiffer(d);
  if (d.kind !== "value") return false;
  const was = printings.get(printingIn(d.main));
  return was !== undefined && was === printings.get(printingIn(d.cand)) && listed(was);
};

/** Whether `approved` holds a usable printings data change. */
const printingsApproved = (approved) => approved.some((a) => a.data === "printings" && completeData(a));

/**
 * `main` and `cand` with every card list that holds the same cards on both sides put in oracle id
 * order, so each card compares with itself, while `approved` holds a printings data change:
 * another printing can reorder a list sorted by printing. A list that gained or lost a card keeps
 * its order, so that still shows. Without a printings data change both come back as they were.
 */
export const alignCards = (main, cand, approved) => {
  if (!printingsApproved(approved)) return [main, cand];
  const align = (a, b) => {
    if (Array.isArray(a) && Array.isArray(b)) {
      const ids = (list) => list.map((item) => oracleOf(item));
      const [x, y] = [ids(a), ids(b)];
      const cards = x.length > 0 && [...x, ...y].every((id) => id !== undefined);
      const sameCards = cards && x.length === y.length && [...x].sort().join() === [...y].sort().join();
      const order = (list) => [...list].sort((p, q) => (oracleOf(p) < oracleOf(q) ? -1 : oracleOf(p) > oracleOf(q) ? 1 : 0));
      const [p, q] = sameCards ? [order(a), order(b)] : [a, b];
      const pairs = p.map((item, i) => (i < q.length ? align(item, q[i]) : [item, undefined]));
      return [pairs.map(([m]) => m), [...pairs.slice(0, q.length).map(([, c]) => c), ...q.slice(p.length)]];
    }
    if (kindOf(a) === "object" && kindOf(b) === "object") {
      const x = { ...a };
      const y = { ...b };
      for (const k of Object.keys(a)) if (Object.hasOwn(b, k)) [x[k], y[k]] = align(a[k], b[k]);
      return [x, y];
    }
    return [a, b];
  };
  return align(main, cand);
};

/**
 * Sorts every difference in `results` (`[{step: {name, route}, diffs, main, cand}]`, `main` and
 * `cand` the compared replies) into approved and unapproved. A path entry approves a difference
 * by its route, path and kind; a data change (`"data": "printings"`) approves one inside a card
 * whose oracle id it lists (or `"*"` for any), the same card on both sides. Lists the entries
 * missing a field as incomplete, and the complete ones that matched nothing as stale.
 */
export const judge = (results, approved) => {
  const usable = approved.filter(complete);
  const printings = printingsApproved(usable) ? printingsOf(results) : new Map();
  const used = new Set();
  const verdict = { approved: [], unapproved: [], stale: [], incomplete: approved.filter((a) => !complete(a)) };
  for (const { step, diffs, main, cand } of results) {
    for (const d of diffs) {
      const entry = usable.find((a) =>
        a.data === undefined
          ? a.route === step.route && a.kind === d.kind && matches(a.path, d.path)
          : coversData(a, step.route, d, main, cand, printings),
      );
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
