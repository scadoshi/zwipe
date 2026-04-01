# zwipe-core

Shared domain logic for the Zwipe ecosystem.

Currently provides password validation (policy enforcement + common password dictionary). As Zwipe grows — potentially including a full-featured web app — additional shared logic will land here rather than being duplicated across crates.

## Consumers

- **zerver** — backend API server (validates passwords during registration, password change, and reset)
- **zweb** — web frontend (client-side validation on the password reset form)
- **zwiper** — mobile app (may consume in the future)

## What's inside

| Module | Purpose |
|--------|---------|
| `password` | Password policy validation, error types, common password dictionary |

## Usage

```rust
use zwipe_core::password::{validate, InvalidPassword};

match validate("candidate_password") {
    Ok(()) => println!("password meets all requirements"),
    Err(e) => println!("validation failed: {e}"),
}
```
