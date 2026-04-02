# zwipe-core

Shared domain logic for the Zwipe ecosystem.

Provides domain types, validation, and business rules shared across crates. As Zwipe grows, additional shared logic lands here rather than being duplicated.

## Consumers

- **zerver** — backend API server
- **zweb** — web frontend (client-side validation)
- **zwiper** — mobile app

## What's inside

| Module | Purpose |
|--------|---------|
| `domain::auth::password` | Password policy validation, error types |
| `domain::deck::deck_name` | Deck name validation (1-64 chars, no profanity) |
| `domain::deck::quantity` | Card quantity validation |
| `domain::moderation` | Content moderation (profanity filtering) |
| `domain::user::username` | Username validation (3-20 chars, no profanity) |
| `domain::EmailAddress` | Re-exported from `email_address` crate |

## Usage

```rust
use zwipe_core::domain::auth::password::{validate, InvalidPassword};

match validate("candidate_password") {
    Ok(()) => println!("password meets all requirements"),
    Err(e) => println!("validation failed: {e}"),
}
```
