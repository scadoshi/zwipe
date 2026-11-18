// What wire-diff's comparison must do: name every field that differs between the two servers'
// replies, mask what changes run to run, and pass a difference only when approved.json lists it.
//
//   node --test zcripts/wire-diff/lib.test.mjs
import assert from "node:assert/strict";
import { test } from "node:test";

import { alignCards, diff, judge, normalize, shapeOf, shapes } from "./lib.mjs";

test("the same reply has no differences", () => {
  const reply = { status: 200, body: { id: "a", cards: [{ n: 1 }, { n: 2 }] } };
  assert.deepEqual(diff(reply, structuredClone(reply)), []);
});

test("each kind of difference is named by its path", () => {
  const main = { status: 200, body: { kept: 1, gone: true, count: 3, name: "x", list: [1, 2] } };
  const cand = { status: 200, body: { kept: 1, count: "3", name: "y", list: [1], fresh: null } };
  assert.deepEqual(diff(main, cand), [
    { path: "/count", kind: "type", main: 3, cand: "3" },
    { path: "/fresh", kind: "added", main: undefined, cand: null },
    { path: "/gone", kind: "removed", main: true, cand: undefined },
    { path: "/list/1", kind: "removed", main: 2, cand: undefined },
    { path: "/name", kind: "value", main: "x", cand: "y" },
  ]);
});

test("a different status is a difference of its own, and the bodies still compare", () => {
  assert.deepEqual(diff({ status: 200, body: { a: 1 } }, { status: 422, body: "no" }), [
    { path: "", kind: "status", main: 200, cand: 422 },
    { path: "", kind: "type", main: { a: 1 }, cand: "no" },
  ]);
});

test("masked paths read the same whatever they hold, and an absent one stays absent", () => {
  const rules = { mask: ["/access_token/value", "/entries/*/updated_at"] };
  const body = {
    access_token: { value: "abc", expires_at: "x" },
    entries: [{ updated_at: "t1", id: 1 }, { id: 2 }],
  };
  assert.deepEqual(normalize(body, rules), {
    access_token: { value: "<masked:string>", expires_at: "x" },
    entries: [{ updated_at: "<masked:string>", id: 1 }, { id: 2 }],
  });
});

test("a masked value hides what it holds but not its kind", () => {
  const rules = { mask: ["/updated_at"] };
  const main = { status: 200, body: normalize({ updated_at: "2026-11-18T00:00:00Z" }, rules) };
  const later = { status: 200, body: normalize({ updated_at: "2026-11-19T00:00:00Z" }, rules) };
  const gone = { status: 200, body: normalize({ updated_at: null }, rules) };
  assert.deepEqual(diff(main, later), [], "a new time is the same kind");
  assert.deepEqual(diff(main, gone), [
    { path: "/updated_at", kind: "value", main: "<masked:string>", cand: "<masked:null>" },
  ]);
});

test("a key holding / or ~ keeps to one path segment", () => {
  const main = { status: 200, body: { tags: { "a/b": 1, "c~d": 1 } } };
  const cand = { status: 200, body: { tags: { "a/b": 2, "c~d": 1 } } };
  assert.deepEqual(diff(main, cand).map((d) => d.path), ["/tags/a~1b"]);
  assert.deepEqual(normalize(main.body, { mask: ["/tags/c~0d"] }), {
    tags: { "a/b": 1, "c~d": "<masked:number>" },
  });
});

test("arrays listed as unordered sort by their key before comparing", () => {
  const rules = { sort: [{ path: "/decks", by: "name" }] };
  assert.deepEqual(normalize({ decks: [{ name: "b" }, { name: "a" }] }, rules), {
    decks: [{ name: "a" }, { name: "b" }],
  });
});

test("a difference passes only when approved.json lists its route, path and kind", () => {
  const step = { name: "fetch deck", route: "/api/deck/{deck_id}" };
  const diffs = [
    { path: "/entries/0/deck_card/flagged", kind: "added" },
    { path: "/name", kind: "removed" },
  ];
  const approved = [
    { route: "/api/deck/{deck_id}", path: "/entries/*/deck_card/flagged", kind: "added", reason: "Flag", approved_by: "composer" },
    { route: "/api/deck/{deck_id}", path: "/name", kind: "value", reason: "wrong kind", approved_by: "composer" },
    { route: "/api/user", path: "/x", kind: "added", reason: "never seen", approved_by: "composer" },
  ];
  const verdict = judge([{ step, diffs }], approved);
  assert.deepEqual(
    verdict.unapproved.map((d) => [d.step, d.path, d.kind]),
    [["fetch deck", "/name", "removed"]],
  );
  assert.deepEqual(
    verdict.approved.map((d) => [d.step, d.path]),
    [["fetch deck", "/entries/0/deck_card/flagged"]],
  );
  assert.deepEqual(
    verdict.stale.map((a) => [a.route, a.path]),
    [["/api/deck/{deck_id}", "/name"], ["/api/user", "/x"]],
  );
  assert.deepEqual(verdict.incomplete, []);
});

test("an approval without a reason or an approver doesn't count", () => {
  const step = { name: "fetch user", route: "/api/user" };
  const verdict = judge([{ step, diffs: [{ path: "/x", kind: "added" }] }], [
    { route: "/api/user", path: "/x", kind: "added", reason: "", approved_by: "composer" },
  ]);
  assert.equal(verdict.unapproved.length, 1);
  assert.deepEqual(verdict.incomplete.map((a) => a.path), ["/x"], "it's unfinished, not stale");
  assert.deepEqual(verdict.stale, []);
});

test("an approval for one field doesn't pass another on the same route", () => {
  const step = { name: "fetch user", route: "/api/user" };
  const verdict = judge([{ step, diffs: [{ path: "/email", kind: "removed" }] }], [
    { route: "/api/user", path: "/username", kind: "removed", reason: "renamed", approved_by: "composer" },
  ]);
  assert.deepEqual(verdict.unapproved.map((d) => d.path), ["/email"]);
});

test("an unordered array can sort by a nested field", () => {
  const rules = { sort: [{ path: "", by: "/scryfall_data/id" }] };
  const body = [{ scryfall_data: { id: "b" } }, { scryfall_data: { id: "a" } }];
  assert.deepEqual(normalize(body, rules), [{ scryfall_data: { id: "a" } }, { scryfall_data: { id: "b" } }]);
});

test("a shape keeps field names and types, not values or how many items a list holds", () => {
  const release = (version, entries) => ({ version, date: "Nov 18", entries });
  const main = { status: 200, body: shapeOf({ upcoming: [release("1.24.0", ["a"])], releases: [release("1.23.1", ["b", "c"])] }) };
  const more = { status: 200, body: shapeOf({ upcoming: [release("1.24.0", ["a", "x"])], releases: [release("1.23.2", ["d"]), release("1.23.1", ["b", "c"])] }) };
  assert.deepEqual(diff(main, more), [], "new lines and a new release compare equal");
  const renamed = { status: 200, body: shapeOf({ upcoming: [{ version: "1.24.0", date: "Nov 18", changes: ["a"] }], releases: [release("1.23.1", ["b"])] }) };
  assert.deepEqual(diff(main, renamed).map((d) => [d.path, d.kind]), [
    ["/upcoming/0/changes", "added"],
    ["/upcoming/0/entries", "removed"],
  ]);
  const retyped = { status: 200, body: shapeOf({ upcoming: [release(124, ["a"])], releases: [release("1.23.1", ["b"])] }) };
  assert.deepEqual(diff(main, retyped).map((d) => [d.path, d.kind]), [["/upcoming/0/version", "value"]]);
});

test("an empty list matches any list's shape", () => {
  const [main, cand] = shapes(
    { upcoming: [{ version: "1.24.0", entries: ["a"] }] },
    { upcoming: [{ version: "1.25.0", entries: [] }] },
  );
  assert.deepEqual(diff({ status: 200, body: main }, { status: 200, body: cand }), [], "a cut's empty list isn't a change");
  const [none, some] = shapes({ list: [] }, { list: [{ a: 1 }] });
  assert.deepEqual(diff({ status: 200, body: none }, { status: 200, body: some }), []);
  const [filled, renamed] = shapes(
    { upcoming: [{ version: "1.24.0", entries: ["a"] }] },
    { upcoming: [{ version: "1.24.0", changes: ["a"] }] },
  );
  assert.deepEqual(
    diff({ status: 200, body: filled }, { status: 200, body: renamed }).map((d) => [d.path, d.kind]),
    [["/upcoming/0/changes", "added"], ["/upcoming/0/entries", "removed"]],
    "filled lists still compare their items",
  );
});

test("items of mixed kinds merge by kind name", () => {
  assert.deepEqual(shapeOf([{ a: 1 }, null]), ["null|object"]);
  assert.deepEqual(shapeOf([1, "x", 2]), ["number|string"]);
});

// A data change: the candidate's database picks another printing for some cards (578's
// latest_cards tiebreak). One entry covers every difference inside those cards, on any route.
const printings = (oracleIds, extra = {}) => ({
  data: "printings",
  oracle_ids: oracleIds,
  reason: "tied printings pick by collector number",
  approved_by: "composer",
  ...extra,
});
const deck = (entries) => ({ entries });
const entry = (oracle, printing, tags) => ({
  card: { scryfall_data: { oracle_id: oracle, id: printing, name: `card ${oracle}` }, card_profile: { oracle_tags: tags } },
  deck_card: { scryfall_data_id: printing },
});
const judged = (main, cand, approved, route = "/api/deck/{deck_id}") => {
  const step = { name: "fetch deck", route };
  const diffs = diff({ status: 200, body: main }, { status: 200, body: cand });
  return judge([{ step, diffs, main, cand }], approved);
};

test("a printings data change approves what differs inside a listed card, at any depth", () => {
  const main = deck([entry("o1", "p1", ["a", "b"]), entry("o2", "p2", [])]);
  const cand = deck([entry("o1", "p9", ["a"]), entry("o2", "p2", [])]);
  const verdict = judged(main, cand, [printings(["o1"])]);
  assert.deepEqual(verdict.unapproved, []);
  assert.deepEqual(
    verdict.approved.map((d) => [d.path, d.kind]),
    [
      ["/entries/0/card/card_profile/oracle_tags/1", "removed"],
      ["/entries/0/card/scryfall_data/id", "value"],
      ["/entries/0/deck_card/scryfall_data_id", "value"],
    ],
    "the row's own fields, beside the card, count as the card's",
  );
  assert.deepEqual(verdict.stale, []);
});

test("a printings data change doesn't pass another card in the same place, or an unlisted one", () => {
  const swapped = judged(deck([entry("o1", "p1", [])]), deck([entry("o2", "p2", [])]), [printings("*")]);
  assert.ok(swapped.unapproved.length > 0, "a different card at that index is a reorder, not a printing");
  const unlisted = judged(deck([entry("o3", "p3", [])]), deck([entry("o3", "p4", [])]), [printings(["o1"])]);
  assert.deepEqual(unlisted.unapproved.map((d) => d.path), ["/entries/0/card/scryfall_data/id", "/entries/0/deck_card/scryfall_data_id"]);
  assert.deepEqual(unlisted.stale.map((a) => a.data), ["printings"], "a data change nothing matched is stale");
});

test("a printings data change for every card still leaves what no card holds", () => {
  const verdict = judged(["Ann", "Bo"], ["Ann", "Cy"], [printings("*")], "/api/card/artists");
  assert.deepEqual(verdict.unapproved.map((d) => d.path), ["/1"], "the artist list needs its own entry");
});

test("a data change can keep to one route", () => {
  const main = deck([entry("o1", "p1", [])]);
  const cand = deck([entry("o1", "p9", [])]);
  const verdict = judged(main, cand, [printings("*", { route: "/api/card/search" })]);
  assert.equal(verdict.approved.length, 0);
  assert.equal(verdict.unapproved.length, 2);
});

test("a data change without ids, or of a kind wire-diff doesn't know, doesn't count", () => {
  const main = deck([entry("o1", "p1", [])]);
  const cand = deck([entry("o1", "p9", [])]);
  const verdict = judged(main, cand, [printings([]), { ...printings("*"), data: "prices" }]);
  assert.equal(verdict.unapproved.length, 2);
  assert.deepEqual(verdict.incomplete.map((a) => a.data), ["printings", "prices"]);
});

test("with a printings data change, a card list in a new order still pairs each card with itself", () => {
  const main = deck([entry("o1", "p1", []), entry("o2", "p2", [])]);
  const cand = deck([entry("o2", "p2", []), entry("o1", "p9", [])]);
  const [a, b] = alignCards(main, cand, [printings("*")]);
  const diffs = diff({ status: 200, body: a }, { status: 200, body: b });
  const verdict = judge([{ step: { name: "fetch deck", route: "/api/deck/{deck_id}" }, diffs, main: a, cand: b }], [printings("*")]);
  assert.deepEqual(verdict.unapproved, [], "the reorder is the data change's, and o1's new printing is approved");
  assert.equal(verdict.approved.length, 2);
  const [same] = alignCards(main, cand, []);
  assert.deepEqual(same, main, "without a printings data change nothing moves");
  const grown = deck([entry("o1", "p1", []), entry("o2", "p2", []), entry("o3", "p3", [])]);
  const [x, y] = alignCards(main, grown, [printings("*")]);
  assert.deepEqual([x, y], [main, grown], "a list that gained or lost a card keeps its order, so that shows");
});

test("a printings data change covers a bare printing id of a listed card, named anywhere in the run", () => {
  const profile = (id) => ({ deck_profile: { commander_id: id, commander_art_url: `https://cards.scryfall.io/art_crop/front/x/y/${id}.jpg?1` } });
  const step = { name: "deck", route: "/api/deck/{deck_id}" };
  const main = { ...profile("11111111-1111-1111-1111-111111111111"), ...deck([entry("o1", "11111111-1111-1111-1111-111111111111", [])]) };
  const cand = { ...profile("99999999-9999-9999-9999-999999999999"), ...deck([entry("o1", "99999999-9999-9999-9999-999999999999", [])]) };
  const diffs = diff({ status: 200, body: main }, { status: 200, body: cand });
  const verdict = judge([{ step, diffs, main, cand }], [printings(["o1"])]);
  assert.deepEqual(verdict.unapproved, [], "commander_id and its art URL name o1's two printings");
  const other = judge([{ step, diffs, main, cand }], [printings(["o2"])]);
  assert.ok(other.unapproved.some((d) => d.path === "/deck_profile/commander_id"), "not for an unlisted card");
});
