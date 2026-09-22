# Type the request half of `Endpoint`

**Status: STUB 2026-09-22, from the external reassessment. Not scheduled; do
not ride 1.10.2, it touches `call.rs`.**

**One sentence:** give `Endpoint` a `type Request`, so a body is a contract
type rather than a `serde_json::Value` assembled by hand at each call site.

## Why

`Endpoint` types the response and leaves the request stringly-typed. Today:

```rust
pub struct CreateDeck(pub Value);
// caller
let body = serde_json::to_value(&request)?;
self.call(CreateDeck(body), Some(session)).await
```

Measured 2026-09-22: **25** `to_value` calls in `zwipe-client`, **18**
endpoints carrying `pub Value`, **18** `self.0.clone()` calls in `body()`.
Every one of those clones a whole JSON tree to hand it to reqwest, which
serializes it again.

The real cost is not the allocation. It is that a wrong body compiles. The
endpoint says nothing about what it accepts, so the contract type and the
endpoint agree only by convention, and `From<serde_json::Error>` exists on
`ClientError` largely to carry a failure that cannot meaningfully happen.

## The blocker, and the two ways round it

Associated-type defaults are unstable, so `type Request` cannot default to
`()`. Either:

1. Write `type Request = ();` on the roughly 26 bodyless endpoints. Blunt,
   obvious, no new concepts.
2. Split a `GetEndpoint` marker trait for the bodyless ones. Less
   boilerplate, one more concept, and two traits to keep in step.

Prefer 1 unless writing it out reads badly. The boilerplate is honest and a
reader never has to ask which trait an endpoint implements.

## Wire safety: this DOES change the bytes

Not a blocker, but it must be stated, because the repo's habit is to verify
rather than argue.

`serde_json::Value` is backed by a `BTreeMap` unless the `preserve_order`
feature is on, and it is not. So today's bodies go out with **alphabetically
ordered keys**. A typed `.json(&req)` emits **declaration order**. Verified:

```
direct   : {"zeta":1,"alpha":2}
via Value: {"alpha":2,"zeta":1}
```

JSON object order carries no meaning and serde's derived `Deserialize`
ignores it, so zerver parses both identically. Confirm before shipping that
nothing hashes, signs or logs a raw request body expecting a stable
serialization. Nothing is known to.

## Rider: `path()` returns `String`

`fn path(&self) -> String` hands back an allocation for the 30-odd endpoints
whose path is a const. `Cow<'static, str>` gives fixed paths
`Cow::Borrowed` and interpolated ones `Cow::Owned`. A one-line signature
change, worth doing in the same pass since it is the same trait. There is an
HTTP round trip immediately after, so this is not a performance fix; it just
stops undoing the consts that commit `4a6bce43` introduced.

## Sketch

1. `type Request: Serialize` on the trait; `fn body(&self) -> Option<&Self::Request>`.
2. Endpoint tuple structs carry the contract type, not `Value`.
3. `call` does `.json(req)`.
4. Delete the `to_value` calls and, if nothing else needs it,
   `From<serde_json::Error> for ClientError`.
5. `path() -> Cow<'static, str>`.

## Verification

- Full CI gate.
- A test per changed endpoint family asserting the serialized body still
  deserializes into the server's contract type.
- Device smoke on the mutating paths: create deck, add card, update card,
  import. Those are the ones carrying bodies.
