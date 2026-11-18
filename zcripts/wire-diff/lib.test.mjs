// What wire-diff's comparison must do: name every field that differs between the two servers'
// replies, mask what changes run to run, and pass a difference only when approved.json lists it.
//
//   node --test zcripts/wire-diff/lib.test.mjs
import assert from "node:assert/strict";
import { test } from "node:test";

import { diff, judge, normalize } from "./lib.mjs";

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
    access_token: { value: "<masked>", expires_at: "x" },
    entries: [{ updated_at: "<masked>", id: 1 }, { id: 2 }],
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
});

test("an approval without a reason or an approver doesn't count", () => {
  const step = { name: "fetch user", route: "/api/user" };
  const verdict = judge([{ step, diffs: [{ path: "/x", kind: "added" }] }], [
    { route: "/api/user", path: "/x", kind: "added", reason: "", approved_by: "composer" },
  ]);
  assert.equal(verdict.unapproved.length, 1);
});

test("an approval for one field doesn't pass another on the same route", () => {
  const step = { name: "fetch user", route: "/api/user" };
  const verdict = judge([{ step, diffs: [{ path: "/email", kind: "removed" }] }], [
    { route: "/api/user", path: "/username", kind: "removed", reason: "renamed", approved_by: "composer" },
  ]);
  assert.deepEqual(verdict.unapproved.map((d) => d.path), ["/email"]);
});
