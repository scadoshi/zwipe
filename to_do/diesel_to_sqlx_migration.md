# Diesel to SQLx Migration Plan 🚀

**Goal**: Transition from Diesel ORM to SQLx for direct SQL control, better performance, and compile-time query verification.

## Phase 1: Dependencies & Core Infrastructure

### 1.1 Update Cargo.toml Dependencies
- [x] **Remove Diesel dependencies**:
  ```toml
  # REMOVE these lines:
  diesel = { version = "2.1", features = ["postgres", "r2d2", "chrono", "128-column-tables", "uuid"] }
  diesel_migrations = "2.1"
  ```

- [x] **Add SQLx dependencies**:
  ```toml
  # ADD these lines:
  sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "chrono", "uuid", "json"] }
  sqlx-macros = "0.8"  # For compile-time query checking
  ```

- [x] **Keep supporting dependencies**:
  - uuid, chrono, serde - no changes needed
  - Remove `strum` if only used for Diesel SQL conversion

### 1.2 Database Connection Pool Migration
- [x] **Replace `utils.rs` connection management**:
  - Remove r2d2 Pool types
  - Add SQLx PgPool implementation
  - Update `connect_to()` function → `get_pool()` function
  - Change return type from `PooledConn` → `&PgPool`

- [x] **Update AppState structure**:
  ```rust
  // OLD: DbPool = Pool<ConnectionManager<PgConnection>>
  // NEW: DbPool = sqlx::PgPool
  ```

## Phase 2: Schema & Model Migration

### 2.1 Eliminate Diesel Schema
- [x] **Remove `schema.rs` file entirely**
  - Diesel's table! macros no longer needed
  - Replace with SQL query constants or query builder functions

- [ ] **Create SQL constants file** (`src/sql/mod.rs`):
  ```rust
  // Define all your SQL queries as constants
  pub const CREATE_USER: &str = "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *";
  pub const FIND_USER_BY_IDENTIFIER: &str = "SELECT * FROM users WHERE username = $1 OR email = $1";
  // ... etc for all operations
  ```
  I didn't do this. The macro will help me align my queries! :)

### 2.2 Model Struct Migration
- [x] **Update all model structs**:
  - Remove `#[derive(Queryable, Selectable, Insertable, AsChangeset)]`
  - Add `#[derive(sqlx::FromRow)]` for result mapping
  - Remove `#[diesel(...)]` attributes
  - Keep `#[derive(Serialize, Deserialize)]` for JSON responses

- [x] **Models to update**:
  - [x] `models/user.rs` - User (simplified to single struct)
  - [x] `models/deck.rs` - Deck (simplified to single struct)  
  - [x] `models/deck_card.rs` - DeckCard (with FromRow derive)
  - [x] `models/card/mod.rs` - Card (simplified, sub-modules commented out for now)
  - [x] `models/types.rs` - MtgFormat enum (kept with strum)

### 2.3 Custom Types Migration
- [x] **Color enum migration**:
  - [x] Remove `FromSql<Text, Pg>` and `ToSql<Text, Pg>` implementations
  - [x] Add SQLx type implementations:
    ```rust
    impl sqlx::Type<sqlx::Postgres> for Color { ... }
    impl sqlx::Encode<'_, sqlx::Postgres> for Color { ... }
    impl sqlx::Decode<'_, sqlx::Postgres> for Color { ... }
    ```

- [x] **Array handling migration**:
  - [x] Remove Colors wrapper newtype (no longer needed for orphan rule)
  - [x] Use `Vec<Color>` directly with SQLx array support
  - [x] Update PostgreSQL array serialization logic

## Phase 3: Handler Migration

### 3.1 Auth Handler Migration (`handlers/auth.rs`)
- [x] **Replace Diesel imports**:
  ```rust
  // COMPLETED: All handlers now use SQLx imports
  use sqlx::query_as;
  ```

- [x] **Migrate `authenticate_user()` function**:
  ```rust
  // COMPLETED: Clean SQLx implementation with query_as! macro
  let user = query_as!(User, "SELECT * FROM users WHERE email = $1 OR username = $1", identifier)
      .fetch_one(&app_state.db_pool)
      .await?;
  ```

- [x] **Migrate `register_user()` function**:
  ```rust
  // COMPLETED: SQLx insert with proper error handling
  let user = query_as!(User, "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *",
      username, email, password_hash)
      .fetch_one(&app_state.db_pool)
      .await?;
  ```

- [x] **Update error handling**:
  - [x] Replace `DatabaseError(DatabaseErrorKind::UniqueViolation, _)` pattern matching
  - [x] Use SQLx error types: `sqlx::Error::Database` with PostgreSQL error codes (`23505`)

### 3.2 Deck Handler Migration (`handlers/decks.rs`)
- [x] **Replace Diesel imports with SQLx**
- [x] **Migrate `get_decks()` function**:
  ```rust
  // COMPLETED: Clean SQLx implementation with query_as! macro
  let user_decks = query_as!(Deck, "SELECT * FROM decks WHERE user_id = $1", authenticated_user.user_id)
      .fetch_all(&app_state.db_pool)
      .await?;
  ```

### 3.3 Update All Handler Functions
- [ ] **Change function signatures**:
  - Replace `State<AppState>` extraction pattern (keep this)
  - Update database operations to async/await
  - Change from `&mut conn` to `&pool`

- [ ] **Error handling updates**:
  - Replace Diesel-specific error patterns
  - Update StatusCode mapping for SQLx errors
  - Maintain existing logging patterns

## Phase 4: Main Application Updates

### 4.1 Update `main.rs`
- [x] **Replace database initialization**:
  ```rust
  // COMPLETED: SQLx native connection pooling
  let db_pool = PgPoolOptions::new()
      .min_connections(2)
      .max_connections(10)
      .idle_timeout(Some(Duration::from_secs(300)))
      .acquire_timeout(Duration::from_secs(5))
      .connect(&database_url)
      .await?;
  ```

- [x] **Update AppState**:
  ```rust
  // COMPLETED: SQLx PgPool integration
  pub struct AppState {
      pub db_pool: sqlx::PgPool,  // ✅ Changed type
      pub jwt_config: JwtConfig,
  }
  ```

### 4.2 Update Health Check Handler
- [x] **Migrate `handlers/health.rs`**:
  ```rust
  // COMPLETED: Clean SQLx health check
  query("SELECT 1").fetch_one(&app_state.db_pool).await?;
  ```

## Phase 5: Migration Scripts & Database Management

### 5.1 Database Migrations Strategy
- [ ] **Choose migration approach**:
  - **Option A**: Use SQLx migrations (`sqlx migrate add`)
  - **Option B**: Keep existing PostgreSQL schema, remove Diesel migration runner
  - **Option C**: Manual SQL scripts in `migrations/` folder

- [ ] **Recommended: SQLx migrations**:
  ```bash
  # Install SQLx CLI
  cargo install sqlx-cli --features postgres
  
  # Initialize migrations (if starting fresh)
  sqlx migrate add initial_schema
  
  # Or migrate existing Diesel migrations to SQLx format
  ```

### 5.2 Environment & Configuration
- [ ] **Update `.env` requirements**:
  - Same `DATABASE_URL` format works for both
  - Add `SQLX_OFFLINE=true` for compile-time query checking without DB connection

- [ ] **Add `sqlx-data.json`** (for offline compilation):
  ```bash
  # Generate after migration complete
  cargo sqlx prepare
  ```

## Phase 6: Testing & Validation

### 6.1 Compilation & Type Checking
- [ ] **Fix compilation errors incrementally**:
  1. Start with `cargo check` - fix import errors
  2. Update function signatures 
  3. Fix query syntax and parameter binding
  4. Update error handling patterns

### 6.2 Runtime Testing
- [ ] **Test all endpoints**:
  - [ ] `POST /api/v1/auth/register` - User registration
  - [ ] `POST /api/v1/auth/login` - User authentication  
  - [ ] `GET /api/v1/decks` - Deck listing with JWT auth
  - [ ] `GET /health/deep` - Database connectivity

- [ ] **Verify data integrity**:
  - Same query results as Diesel version
  - Proper parameter binding (no SQL injection vulnerabilities)
  - Error handling maintains same behavior

### 6.3 Performance Validation  
- [ ] **Compare query performance**:
  - SQLx should be faster (no ORM overhead)
  - Connection pooling efficiency
  - Memory usage comparison

## Phase 7: Advanced Features & Cleanup

### 7.1 SQLx-Specific Optimizations
- [ ] **Implement compile-time query checking**:
  ```rust
  // Use sqlx::query! macro for compile-time verification
  let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
      .fetch_one(pool)
      .await?;
  ```

- [ ] **Add prepared statement caching**:
  - SQLx automatically caches prepared statements
  - Consider explicit `PgPool::connect_with()` options for tuning

### 7.2 Code Organization Improvements
- [ ] **Create query modules**:
  ```
  src/
  ├── queries/
  │   ├── mod.rs
  │   ├── users.rs    # All user-related queries
  │   ├── decks.rs    # All deck-related queries
  │   └── cards.rs    # All card-related queries
  ```

- [ ] **Extract common patterns**:
  - Database error → StatusCode mapping utility
  - Transaction helpers for complex operations
  - Query result → JSON response utilities

### 7.3 Final Cleanup
- [ ] **Remove unused dependencies**:
  - Diesel, diesel_migrations from Cargo.toml
  - r2d2 if not used elsewhere
  - Clean up unused imports

- [ ] **Update documentation**:
  - README.md database setup instructions
  - API documentation updates
  - Development setup changes

## Success Criteria ✅

**Migration is complete when**:
1. ✅ All Diesel dependencies removed from Cargo.toml
2. ✅ All handlers use SQLx queries instead of Diesel query builder
3. ✅ Application compiles without Diesel-related errors
4. ✅ All existing API endpoints work identically
5. ✅ Database operations are properly parameterized (SQL injection safe)
6. ✅ Error handling maintains same user experience
7. ✅ Performance is equal or better than Diesel version

## Migration Order Recommendation

**Suggested implementation order** (minimizes broken state):
1. **Dependencies** → Update Cargo.toml, fix compilation
2. **Utils & AppState** → Database connection infrastructure  
3. **Models** → Struct definitions and derives
4. **One Handler at a time** → auth.rs first (most complex error handling)
5. **Main & Health** → Application setup and health checks
6. **Testing** → Validate each piece works before moving to next
7. **Cleanup** → Remove old code, optimize, document

---

**This migration leverages your strong SQL knowledge while eliminating ORM complexity. SQLx gives you the performance and control you want while maintaining compile-time safety through its macro system.**

## Learning Opportunities 🧠

**This migration will strengthen these neural pathways**:
- **Raw SQL Mastery**: Direct query writing, parameter binding, result mapping
- **Async Database Operations**: SQLx's native async design vs Diesel's sync model
- **Compile-time Query Verification**: SQLx macros for catching SQL errors at compile time
- **Performance Optimization**: Understanding ORM overhead vs direct SQL execution
- **PostgreSQL-specific Features**: Arrays, UUIDs, custom types without ORM abstraction

**New concepts to learn**:
- SQLx type system (`Type`, `Encode`, `Decode` traits)
- Prepared statement caching and connection pooling differences
- `sqlx::query!` vs `sqlx::query_as!` vs `sqlx::query_as` patterns
- Transaction handling with `sqlx::Transaction`