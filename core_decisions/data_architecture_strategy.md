# Data Storage Architecture Strategy

## The Core Question: Local vs Server Storage

For your MTG deck builder, we need to optimize for **performance**, **user experience**, and **cost efficiency** while ensuring users can access their data anywhere.

## Data Categories Analysis

### 1. Card Database (Static MTG Cards)
**Nature**: Large, mostly static, shared across all users
**Size**: ~36,000 cards × ~200KB each = ~7.2GB total (images included)

**Options:**
```
Option A: Always Server-Side
├── Pros: Always up-to-date, small app size
├── Cons: Slow swiping, high server costs, offline unusable
└── Verdict: ❌ Poor user experience

Option B: Always Local Storage
├── Pros: Fast swiping, offline capable, low server costs
├── Cons: Large app download, update complexity
└── Verdict: ⚠️ Mixed trade-offs

Option C: Hybrid Approach (RECOMMENDED)
├── Pros: Fast performance + flexible updates
├── Cons: More complex implementation
└── Verdict: ✅ Best balance
```

### 2. User Data (Decks, Preferences)
**Nature**: Small, user-specific, needs sync across devices
**Size**: ~1-10KB per deck × 20 decks = ~200KB per user

**Strategy**: **Server-primary with local caching**

### 3. User Session/Auth Data
**Nature**: Security-sensitive, device-specific
**Strategy**: **Local storage with server validation**

---

## Recommended Architecture

### 🎯 **Hybrid Storage Strategy**

```
Local Device Storage:
├── Card Database (SQLite)
│   ├── Core card data (name, cost, type, image URLs)
│   ├── Cached card images
│   └── Version metadata
├── User Decks (Cache)
│   ├── Recently viewed decks
│   ├── Offline-created decks
│   └── Sync status flags
└── Session Data
    ├── Auth tokens
    ├── User preferences
    └── App settings

Server Storage:
├── User Accounts
│   ├── Authentication data
│   ├── Profile information
│   └── Account settings
├── User Decks (Master)
│   ├── All user decks
│   ├── Deck metadata
│   └── Sharing permissions
└── Card Database (Master)
    ├── Complete card data
    ├── New releases
    └── Updates/corrections
```

---

## Implementation Strategy

### Phase 1: Card Data Management

#### Initial App Setup
```dart
// Flutter app initialization
1. Check for local card database
2. If missing/outdated:
   - Download base card set (~50MB compressed)
   - Store in local SQLite database
   - Cache frequently used card images
3. If existing:
   - Check version against server
   - Download only updates/new cards
```

#### Card Serving During Swiping
```dart
// Swiping performance optimization
1. Pre-load next 20 cards from local SQLite
2. Load card images from local cache (if available)
3. Background download missing images
4. No server calls during active swiping
5. Buttery smooth 60fps swiping experience
```

#### Card Database Updates
```rust
// Rust API endpoints using Axum
GET /api/cards/version          # Check current card DB version
GET /api/cards/delta/:version   # Get only new/changed cards
GET /api/cards/batch/:ids       # Get specific cards (fallback)
```

### Phase 2: User Data Sync

#### Deck Management Flow
```
User Creates Deck:
├── Save immediately to local SQLite
├── Mark as "needs sync"
├── Background API call to save to server
├── Update sync status on success
└── Handle conflicts (last-write-wins for MVP)

User Opens App:
├── Load decks from local cache (instant)
├── Background sync with server
├── Merge any server changes
└── Update local cache
```

#### Sync Strategy
```rust
// Rust API for deck sync using Axum
POST /api/decks                 # Create new deck
PUT /api/decks/:id              # Update existing deck
GET /api/decks                  # Get all user decks
GET /api/decks/since/:timestamp # Incremental sync
```

---

## Storage Size Breakdown

### Local Storage Requirements
```
Full Card Database (NOT FEASIBLE):
├── Card metadata: ~180MB (36k cards)
├── ALL card images: ~7GB (36k × 200KB each)
├── User decks: ~1MB max
├── App data: ~10MB
└── Total: ~7.2GB (UNACCEPTABLE for mobile)

Practical Approach - Subset Storage:
├── Card metadata (text only): ~180MB (all 36k cards)
├── Cached images: ~200-500MB (1k-2.5k most popular cards)
├── User decks: ~1MB max
├── App data: ~10MB
└── Total: ~400-700MB (reasonable)

User Impact:
├── Initial download: ~200MB (metadata + popular images)
├── Ongoing growth: ~2-5MB per month (new images cached)
└── Similar to other content-heavy apps
```

### Server Storage (Per User)
```
User account data: ~1KB
User decks: ~10-50KB (20 decks max)
Session data: ~1KB
Total per user: ~50KB

For 10,000 users: ~500MB total user data
Very manageable and cost-effective
```

---

## Performance Implications

### Card Swiping Performance
```
Local SQLite Query: ~1-5ms
Local Image Load: ~10-50ms
Network Card Fetch: ~200-1000ms
Network Image Load: ~500-2000ms

Result: 100x faster with local storage
Critical for smooth swiping experience
```

### Offline Capability
```
With Hybrid Approach:
├── ✅ Browse cards offline
├── ✅ Create/edit decks offline
├── ✅ View existing decks offline
├── ❌ Login/register (needs network)
└── Auto-sync when network returns
```

---

## Cost Analysis

### Server Costs (Monthly)
```
With Always-Server Approach:
├── Card image serving: ~$200-500/month
├── API calls: ~$50-100/month
├── Database queries: ~$30-50/month
└── Total: ~$280-650/month

With Hybrid Approach:
├── User data sync: ~$20-30/month
├── Card updates: ~$10-20/month
├── Auth/API: ~$20-30/month
└── Total: ~$50-80/month

Savings: ~$200-500/month (85% reduction)
```

### Development Complexity
```
Always-Server: Simple but slow
Always-Local: Simple but inflexible
Hybrid: More complex but optimal

For MVP: Start simple, evolve to hybrid
```

---

## Migration Strategy

### MVP Phase (Simplified)
```
Start with server-heavy approach:
├── Store all cards on server
├── Cache recently viewed cards locally
├── Simple deck sync
└── Focus on core functionality
```

### Phase 2 (Optimized)
```
Migrate to hybrid:
├── Full local card database
├── Intelligent image caching
├── Robust offline support
└── Optimized sync logic
```

---

## Specific Implementation

### Local Database Schema (SQLite)
```sql
-- Card storage
CREATE TABLE cards (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  mana_cost TEXT,
  card_type TEXT,
  image_url TEXT,
  oracle_text TEXT,
  set_name TEXT,
  rarity TEXT,
  created_at TIMESTAMP,
  updated_at TIMESTAMP
);

-- Local deck cache
CREATE TABLE local_decks (
  id INTEGER PRIMARY KEY,
  server_id INTEGER,
  name TEXT NOT NULL,
  format TEXT,
  cards_json TEXT, -- Serialized deck cards
  needs_sync BOOLEAN DEFAULT false,
  last_synced TIMESTAMP
);

-- App metadata
CREATE TABLE app_data (
  key TEXT PRIMARY KEY,
  value TEXT,
  updated_at TIMESTAMP
);
```

### Server API Design
```rust
// Rust API structure using Axum framework
use axum::{Json, Path, Query, Extension, http::StatusCode};

// Cards controller handlers
async fn list_cards(Query(params): Query<CardListParams>) -> Json<CardListResponse> { }
async fn get_card(Path(id): Path<i32>) -> Json<Card> { }
async fn get_cards_version() -> Json<VersionResponse> { }
async fn batch_cards(Json(ids): Json<Vec<i32>>) -> Json<Vec<Card>> { }

// Decks controller handlers  
async fn list_decks(Extension(user): Extension<User>) -> Json<Vec<Deck>> { }
async fn get_deck(Path(id): Path<i32>, Extension(user): Extension<User>) -> Json<Deck> { }
async fn create_deck(Json(deck): Json<CreateDeckRequest>, Extension(user): Extension<User>) -> Json<Deck> { }
async fn update_deck(Path(id): Path<i32>, Json(updates): Json<UpdateDeckRequest>) -> Json<Deck> { }
async fn delete_deck(Path(id): Path<i32>, Extension(user): Extension<User>) -> StatusCode { }
```

---

## Recommendation Summary (Revised for 36k Cards)

**For MVP**: **Smart Hybrid Approach**
- Store all card metadata locally (~180MB)
- Cache popular card images locally (~200MB)
- Load other images on-demand with aggressive caching
- Server stores user decks and handles authentication

**Revised Strategy**:
```
Local Storage:
├── All card metadata (36k cards, ~180MB)
├── Popular card images (~1-2k cards, ~200MB)
├── Recently viewed images (LRU cache, ~100MB)
└── User deck cache

Server Storage:
├── All card images (backup/source)
├── User accounts and decks
├── Card metadata updates
└── Usage analytics (for popular cards)
```

**The Reality Check**: With 36,000 cards, we can't store everything locally. But we can store:
- **ALL metadata locally** (fast text searches, instant card info)
- **Popular images locally** (covers 80% of cards users actually see)
- **Smart caching** for the rest (download once, keep forever)

This gives 80% of the performance benefit with 10% of the storage cost. 