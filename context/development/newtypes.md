---
description: The newtypes zwipe actually has, what each one guarantees, and when to add another
alwaysApply: true
---

# Newtypes

Zwipe wraps a handful of primitives in types that guarantee something about their contents. This file is the inventory and the rules for adding to it. The architecture those types sit inside (hexagonal layering, ports and adapters, the database adapter pattern) is described in [`../architecture/structure.md`](../architecture/structure.md), and the reasoning behind it in [`../architecture/decisions.md`](../architecture/decisions.md).

## What exists

| Type | Crate | Wraps | Guarantee |
|------|-------|-------|-----------|
| `Username` | zwipe-core | `String` | 3-20 chars, no whitespace, no profanity, trimmed |
| `DeckName` | zwipe-core | `String` | 1-64 chars, no profanity, trimmed |
| `Quantity` | zwipe-core | `i32` | At least 1 |
| `Limit` | zwipe-core | `u32` | Clamped to `Limit::MAX`, including on deserialize |
| `Secret` | zwipe-core | `String` | Never printed by `Debug` or `Display` |
| `Jwt` | zwipe-core | `String` | Access-token material, distinct from other strings |
| `Password` | zerver | `String` | Meets the password policy; never printed |
| `HashedPassword` | zerver | `String` | Argon2 output, not plaintext |
| `JwtSecret` | zerver | `String` | Signing key, never leaves the server |

## IDs are bare `Uuid`, deliberately

There is no `UserId`, no `DeckId`, no `CardId`. `DeckProfile { id: Uuid, user_id: Uuid }` is the intended shape. Adding ID wrappers is a recurring suggestion and the answer has been no: the compile-time win never paid for the conversion noise at every boundary, and Postgres hands back `Uuid` either way.

Do not propose them. If the tradeoff ever changes, that is a `decisions.md` entry, not a refactor someone starts.

## The validating pattern

`Username` is the worked example. Every validating newtype follows it:

```rust
pub struct Username(String);

impl Username {
    pub fn new(raw: impl AsRef<str>) -> Result<Self, InvalidUsername> {
        let trimmed = raw.as_ref().trim();
        // one check per error variant, cheapest first
        Ok(Self(trimmed.to_string()))
    }
}
```

Four things travel with it. A `new` returning `Result<Self, InvalidX>`, where `InvalidX` is a `thiserror` enum with one variant per rule, so the caller can tell the user which rule they broke. A private field, so the only way in is through `new`. A `Deref` to the inner value for read access (`str` for the string types, `i32` for `Quantity`). And a `Deserialize` that routes through `new`, written by hand:

```rust
impl<'de> Deserialize<'de> for Username {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(d)?;
        Username::new(raw).map_err(serde::de::Error::custom)
    }
}
```

That last part is the one people skip. A derived `Deserialize` on a single-field struct is transparent: it takes the inner value straight off the wire and never calls `new`, so the type still compiles and still lies.

`Limit` shows the other valid answer. Rather than reject an over-large page size it clamps to `Limit::MAX` on deserialize, because a client asking for too many rows is not an error worth failing a search over. Reject or clamp are both fine. Passing the value through untouched is not.

`DeckName` currently derives it. That is harmless today, because no HTTP contract carries a `DeckName` (they carry `String`, and `CreateDeckProfile::build` validates), but the two types do not offer the same guarantee, and only one of them is safe to put in a contract.

## Secret-bearing types

`Password` and `Secret` hand-write `Debug` and `Display` to print a placeholder instead of their contents:

```rust
/// Never derive this: the derive prints the plaintext, and every struct
/// holding a `Password` inherits that through its own `Debug`.
impl std::fmt::Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Password(REDACTED)")
    }
}
```

`read()` is the only way to the value, which makes every access a visible call site. Both carry a test that fails if someone re-derives either trait.

`Secret` adds `#[serde(transparent)]`: it must serialize as a bare string, because it crosses the wire in request bodies that shipped clients already send. Redaction is for logs, not for the protocol.

**`HashedPassword` is the exception.** Its `Display` is the database write path (`request.password_hash.to_string()` in `outbound/sqlx/auth/mod.rs`). Redacting it would write the literal string `REDACTED` into the password column for every registration. Leave it alone.

## Adding one

Worth a newtype when a primitive has a rule that must hold everywhere, and the cost of it being wrong is real: a name that breaks the UI, a quantity that corrupts a deck, a secret in a log.

Not worth one when the rule is local to a single function, when the type would exist only to rename `Uuid`, or when it would have to be unwrapped at every use.

If you add one, it needs a validating `Deserialize` before it can appear in anything under `zwipe-core/src/http/contracts/`.
