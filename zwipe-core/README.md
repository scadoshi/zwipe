# zwipe-core

Shared domain types, validation, business rules, and HTTP contracts for the Zwipe ecosystem. Pure Rust: no feature flags, no server-only dependencies.

## Modules

| Module | Purpose |
|--------|---------|
| `domain::auth` | Session, AccessToken, Jwt, RefreshToken, password validation |
| `domain::card` | Card, CardProfile, ScryfallData (Colors, Rarity, Legalities, Prices, ImageUris, CardFaces, AllParts), CardRole, CardFilter, search/group/filter logic |
| `domain::deck` | Deck, DeckProfile, DeckCard, DeckEntry, DeckWarning, Format, DeckMetrics, validate_deck(), all deck request types |
| `domain::user` | User, UserPreferences, Username |
| `domain::moderation` | Content moderation (profanity filtering) |
| `domain::site` | Shared base URLs and contact constants (app, site, server) |
| `domain::logo` | ASCII art logos |
| `http` | Route paths, ApiError, HTTP contract structs (request/response shapes) |
| `legal` | Shared legal copy (privacy policy) rendered by app and site |
