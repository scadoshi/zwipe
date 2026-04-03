# zwipe-core

Shared domain types, validation, and business rules for the Zwipe ecosystem. Pure Rust — no feature flags, no server-only dependencies.

## Consumers

- **zerver** — backend API server (re-exports all types, adds service-layer errors and database adapters)
- **zwiper** — mobile app
- **zweb** — web frontend

## What's inside

| Module | Purpose |
|--------|---------|
| `domain::auth::password` | Password policy validation |
| `domain::card` | Card, CardProfile, ScryfallData and all nested types (Colors, Rarity, Legalities, Prices, ImageUris, CardFaces, AllParts) |
| `domain::deck` | DeckProfile, DeckCard, Deck, DeckEntry, DeckWarning, Format, DeckName, Quantity, validate_deck(), DeckMetrics |
| `domain::deck::requests` | CreateDeckProfile, UpdateDeckProfile, DeleteDeck, CreateDeckCard, UpdateDeckCard, DeleteDeckCard, ImportDeckCards, etc. |
| `domain::moderation` | Content moderation (profanity filtering) |
| `domain::user` | User, UserPreferences, Username |
| `domain::user::requests` | GetUser |
| `domain::EmailAddress` | Re-exported from `email_address` crate |
