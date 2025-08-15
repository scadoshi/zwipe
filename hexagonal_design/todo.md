# Hexagonal Architecture & Newtype Refactoring TODO

## Primitive Obsession Fixes (High Priority - Security Issues)

### 1. Replace raw String types for emails with EmailAddress newtype wrapper
- **Found in**: User model, LoginRequest, RegistrationRequest, UserClaims, and auth handlers
- **Risk**: Could accidentally log passwords if parameters swapped
- **Status**: Pending

### 2. Replace raw String types for passwords with Password newtype wrapper  
- **Found in**: LoginRequest, RegistrationRequest, and auth functions accepting &str password parameters
- **Risk**: Security liability - passwords should never be raw strings in function signatures
- **Status**: Pending

### 3. Replace raw String types for JWT tokens with JwtToken newtype wrapper
- **Found in**: LoginResponse and JWT generation/validation functions
- **Risk**: No type safety between generation and validation
- **Status**: Pending

### 4. Replace raw i32 user_id with UserId newtype wrapper
- **Found in**: Throughout models, JWT claims, and database operations
- **Risk**: No validation that IDs are positive, reasonable ranges
- **Status**: Pending

## Business Logic Separation (Medium Priority - Architecture)

### 5. Move password hashing logic out of auth handlers into domain services
- **Found in**: auth handlers directly call hash_password instead of domain service
- **Issue**: Business logic mixed with HTTP concerns
- **Status**: Pending

### 6. Move email validation and normalization out of JWT generation into EmailAddress constructor
- **Found in**: JWT function directly normalizes email
- **Issue**: Business rules scattered across infrastructure
- **Status**: Pending

### 7. Move user ID validation out of JWT generation into UserId constructor
- **Found in**: JWT function validates range 1-100M directly
- **Issue**: Domain validation in infrastructure layer
- **Status**: Pending

## Domain/Infrastructure Separation (Medium Priority - Clean Architecture)

### 8. Separate database operations from business logic in auth functions
- **Found in**: authenticate_user and register_user mix SQLx queries with business rules
- **Issue**: No clean separation between domain and data access
- **Status**: Pending

### 9. Extract user repository pattern to separate database access from auth handlers
- **Found in**: handlers directly use query_as! macros
- **Issue**: Handlers coupled to specific database implementation
- **Status**: Pending

## Validation Consolidation (Medium Priority - Maintainability)

### 10. Consolidate all email validation rules into EmailAddress::new() constructor
- **Found in**: Scattered validation instead of centralized newtype validation
- **Issue**: No single source of truth for what constitutes valid email
- **Status**: Pending

### 11. Consolidate all password validation rules into Password::new() constructor
- **Found in**: argon2 functions accepting any &str instead of validated Password type
- **Issue**: No enforced password policy at type level
- **Status**: Pending

## Error Handling Improvements (Low Priority - Maintainability)

### 12. Create domain-specific error types instead of mixing SQLx::Error with StatusCode
- **Found in**: Error boundaries mixed between infrastructure and HTTP concerns
- **Issue**: Error handling not following hexagonal boundaries
- **Status**: Pending

### 13. Fix function signatures that accept multiple String parameters
- **Found in**: Functions with (identifier, password, email) parameters that could be swapped
- **Issue**: Parameter order safety - the exact problem from newtype article
- **Status**: Pending

## Hexagonal Architecture Implementation (Low Priority - Long-term Architecture)

### 14. Define port interfaces (traits) for UserRepository, EmailService, and PasswordHasher
- **Found in**: Direct dependencies on concrete implementations
- **Issue**: No inversion of control through interfaces
- **Status**: Pending

### 15. Create adapter implementations (PostgresUserRepository, Argon2PasswordHasher)
- **Found in**: Need adapters that implement the port traits
- **Issue**: Infrastructure should be pluggable through adapters
- **Status**: Pending

### 16. Move pure business logic into domain core
- **Found in**: Business logic depends on concrete database/crypto implementations
- **Issue**: Domain should depend only on port interfaces
- **Status**: Pending

## Testing & Safety (Low Priority - Quality Assurance)

### 17. Add comprehensive test coverage for all newtype constructors
- **Found in**: Need tests for edge cases, validation failures, and round-trip scenarios
- **Issue**: Newtypes need thorough validation testing
- **Status**: Pending

### 18. Add _unchecked constructors for newtypes when loading from trusted sources
- **Found in**: Need bypass for expensive validation when loading from database
- **Issue**: Performance optimization for trusted data sources
- **Status**: Pending

---

## References
- [Ultimate Guide to Rust Newtypes](https://www.howtocodeit.com/articles/ultimate-guide-rust-newtypes)
- [Master Hexagonal Architecture in Rust](https://www.howtocodeit.com/articles/master-hexagonal-architecture-rust)

## Implementation Strategy
Start with primitive obsession fixes (items 1-4) as these address immediate security concerns, then move through business logic separation, and finally implement full hexagonal architecture.
