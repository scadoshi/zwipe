---
description: Tracks User's Rust/web development learning progress across confidence levels for AI teaching optimization
alwaysApply: true
---

## Update Instructions for AI Assistants

**Confidence Levels**: CONFIDENT = Could teach others without hesitation. DEVELOPING = Successfully implemented but still learning. LEARNING = Recently introduced, needs guidance. UNEXPLORED = Not yet encountered.

**When to Update**: After major implementations, quiz performance, or when user demonstrates clear confidence level changes. Move concepts between levels based on demonstrated understanding and teaching ability.

**Quiz Strategy**: Continue giving pop quizzes in chat to validate understanding and strengthen knowledge. Focus on recent implementations and concepts showing uncertainty.

---

## Current Learning Status

**Last Updated**: After completing deck card HTTP API implementation with nested resource routes

**Next Learning Focus**: Auth domain security operations consolidation

**Recent Achievement**: Successfully completed deck card HTTP API implementation with solid RESTful design principles. Implemented composite key architecture eliminating surrogate IDs, built complete nested resource routes with proper hierarchical structure, and established tuple path parameter extraction patterns. Demonstrates growing understanding of RESTful API design and proper HTTP semantics.

### 🎯 Currently Working Towards (Top 5)
1. **Auth Domain Security Consolidation** - Centralizing user lifecycle operations for consistent security control
2. **Advanced Generic Type Systems** - Understanding opaque vs concrete types in service architecture  
3. **Production Middleware Patterns** - Advanced route protection and middleware composition
4. **Complex Database Transactions** - Multi-table operations with proper rollback handling
5. **Performance Optimization** - Query optimization and connection pool tuning

### 🤔 Current Uncertainties (Top 5)
1. **Generic Service Architecture** - Why Service<R> pattern is structured this way, trait object trade-offs
2. **Proc Macros** - Deep procedural macro implementation (staying away until necessary)
3. **Advanced Async Patterns** - Complex Future handling and async streaming
4. **Type-Level Programming** - Advanced trait constraints and generic programming
5. **Production Deployment** - Containerization, monitoring, and scaling strategies

---

## CONFIDENT - Could Teach Others 🎓

### 🦀 Core Rust Fundamentals
- **Basic Syntax**: Ownership, borrowing, basic pattern matching, error handling with Result
- **Type System**: Understanding of structs, enums, traits, basic generics
- **Memory Safety**: Conceptual understanding of Rust's ownership model
- **Module System**: Basic use of mod, pub, use statements for code organization
- **Cargo Workflow**: Creating projects, running tests, managing dependencies
- **Debugging**: Using println!, investigating compiler errors, reading documentation

### 🏗️ Hexagonal Architecture & Clean Design
- **Domain-First Design**: Business logic separation from external concerns, ports/adapters pattern
- **Dependency Inversion**: Core domain depends on abstractions, not concrete implementations
- **Clean Boundaries**: HTTP, database, external APIs properly separated from business logic
- **Architectural Decision Making**: When to break rules for convenience vs maintainability
- **Separation of Concerns**: Balancing responsibility with abstraction levels
- **YAGNI vs Future Planning**: Build for future but stay realistic about current needs
- **DRY Balance**: Code reuse without over-abstraction

### 🔒 Security & Authentication
- **JWT Security Flow**: Complete token generation/validation with proper configuration
- **Password Security**: Argon2 hashing, salts, rainbow table prevention, timing attack mitigation
- **Authentication Delays**: Why delays in auth are important for security
- **Information Disclosure Prevention**: Avoiding enumeration attacks through generic error responses
- **Security Spidey Senses**: When to step carefully with sensitive data and implementation details
- **Middleware Security**: Functional type-based authentication through authorization headers

### 🚨 Error Handling Architecture
- **Strategic Error Types**: When to use thiserror (decisive system responses) vs anyhow (console error stacks)
- **Error Flow Design**: Comprehensive error handling patterns throughout application layers
- **HTTP Status Mapping**: Business logic errors to appropriate status codes
- **Two-Tier Error Strategy**: User-facing vs internal error separation

### 🗄️ Database Design & Relationships
- **Relational Modeling**: Foreign keys, composite keys, constraints, and indices
- **Complex Joins**: Multi-table queries and relationship management
- **Database Constraints**: Strategic use for business rule enforcement
- **JSONB Confidence**: Comfortable storing JSON in PostgreSQL tables
- **Composite vs Surrogate Keys**: When to use natural vs artificial primary keys

### 📡 HTTP & RESTful API Design
- **Parameter Extraction**: Body, path, and query parameter handling
- **RESTful Patterns**: Proper HTTP verb usage, nested resource routes
- **Status Code Precision**: Correct HTTP status codes for different operations
- **Parameter Naming Consistency**: Aligned naming across request pipelines

### ⚙️ Basic Implementation Patterns
- **Environment Setup**: .env files, configuration management
- **macro_rules!**: Basic declarative macro creation and usage

### 💾 SQLx Database Operations & Advanced Patterns
- **Connection Pooling**: Production-ready pool configuration with optimized settings
- **Error Handling**: Custom IsConstraintViolation trait with PostgreSQL error code mapping
- **Transaction Management**: Consistent transaction usage across all write operations
- **Query Building**: Dynamic QueryBuilder patterns, bulk operations with parameter optimization
- **Custom Type Integration**: Complex domain types with SQLx traits (Decode, Encode, Type)
- **Migration & Schema Management**: Forward-only migrations, database recreation workflows
- **Bulk Data Processing**: Production-scale processing (35,400+ cards) with resilient error handling
- **Constraint Management**: Advanced PostgreSQL constraint handling and violation detection

### 🌐 Advanced HTTP & Middleware Patterns
- **Custom Middleware**: AuthenticatedUser extractor with FromRequestParts trait
- **Type-Safe Authentication**: Compile-time route protection through handler parameters
- **Generic Handler Patterns**: Complex generic state types across all handlers
- **Error Architecture**: Sophisticated ApiError with domain error mapping
- **Request/Response Conversion**: Clean TryFrom patterns for HTTP-to-domain conversion
- **Bearer Token Extraction**: JWT validation and claims parsing in middleware

### 🔄 Async Programming & Trait Constraints
- **Async Function Design**: Building async functions with proper trait bounds
- **Send + Sync Constraints**: Understanding trait requirements for async service patterns
- **Generic Async Patterns**: Handler functions with generic state types and async operations
- **Future Handling**: Async trait implementations across hexagonal boundaries
- *Note: Manual thread synchronization (.lock(), message passing) not yet implemented*

### 📊 Production API Design
- **Nested Resource Routes**: Hierarchical /api/deck/{deck_id}/card/{card_profile_id} patterns
- **Tuple Path Extraction**: Path<(String, String)> for multi-parameter routes
- **Composite Key Architecture**: Natural primary keys eliminating surrogate IDs
- **Search & Pagination**: Complex query building with multiple filter parameters
- **RESTful Patterns**: Proper HTTP verb usage, status code precision, parameter naming consistency

---

## DEVELOPING - Active Implementation (Working But Learning) 🔧

### 🏗️ Service Architecture & Dependency Injection
- **Generic Service Patterns**: Service<R> and Service<DR, CR> implementations across domains
- **Repository Abstraction**: Understanding why services depend on repository traits vs concrete types
- **Cross-Domain Orchestration**: DeckService coordinating between multiple repositories
- **Dependency Injection Theory**: Understand the purpose but not fully confident in the architectural reasoning
- **Service Layer Separation**: Clear on what services do, less clear on why they're structured this way
- *Note: Understand the "what" and "why" conceptually, but "how" implementation details still developing*

### 🏗️ Advanced Architecture Patterns
- **Configuration Management**: Production-ready config loading at startup vs runtime env reads
- **Performance Optimization**: Resolved repeated file system access inefficiencies

### 🧪 Testing & Validation
- **Test Organization**: Clean categorization of test functions by concern
- **Edge Case Testing**: Validation of error conditions and boundary cases
- **Environment Testing**: Understanding environment coupling vs testable design
- **Newtype Testing**: Testing validation at correct levels

### 🌐 External API Integration & Data Processing
- **HTTP Client Setup**: reqwest crate integration with proper headers and error handling
- **JSON Processing**: serde_json parsing and complex JSON deserialization
- **Scryfall API Understanding**: Complete MTG card data structure and API patterns
- **Custom Serde Deserializers**: Flexible type handling for inconsistent API data

### 🔮 Advanced Type Systems
- **Opaque vs Concrete Types**: Understanding `impl UserService` vs `US: UserService` trade-offs
- **Type Inference Patterns**: When type inference works vs explicit generic parameters needed
- **Generic Constraints**: Complex trait bounds and generic programming patterns

---

## LEARNING - Recently Introduced, Needs Guidance 📚

### 🔮 Advanced Rust Patterns
- **Advanced Async Patterns**: Complex Future handling, async streaming, async iterators
- **Type-Level Programming**: Advanced trait constraints, generic programming patterns
- **Complex Lifetime Management**: Advanced lifetime parameters and borrowing patterns

### 🚀 Production Deployment & Scaling
- **Containerization**: Docker, Kubernetes deployment strategies
- **Monitoring & Observability**: Metrics collection, logging, distributed tracing
- **Performance Tuning**: Query optimization, connection pool sizing, caching strategies
- **Rate Limiting**: Request throttling, abuse prevention mechanisms

### 🎮 MTG-Specific Business Logic
- **Format Validation**: Standard/Modern legality checking, card legality rules
- **Deck Rules**: 60-card minimums, 4-card limits, sideboard validation
- **Card Interactions**: Rules engine for card interactions and abilities

---

## UNEXPLORED - Future Learning Areas 🔍

### 🧬 Advanced Rust Language Features
- **Procedural Macros**: Deep proc macro implementation (staying away until necessary)
- **Unsafe Rust**: Memory manipulation, FFI, performance optimizations
- **Embedded Rust**: Hardware programming, real-time systems
- **WebAssembly**: Rust to WASM compilation and browser integration

### 🏢 Enterprise Infrastructure
- **Microservices Architecture**: Service discovery, distributed systems patterns
- **Message Queues**: RabbitMQ, Kafka integration for async processing
- **Distributed Databases**: Sharding, replication, consistency patterns
- **Cloud Platforms**: AWS/GCP deployment, serverless architectures

### 🎨 Frontend Integration
- **Dioxus Mobile**: Cross-platform mobile app development
- **WebRTC**: Real-time communication for multiplayer features
- **Progressive Web Apps**: Offline functionality, service workers
- **State Management**: Complex client-side state synchronization

### 🔬 Advanced Performance Engineering
- **Profiling & Benchmarking**: CPU profiling, memory analysis, performance testing
- **Custom Allocators**: Memory management optimization
- **SIMD Programming**: Vectorized operations for data processing
- **Lock-Free Programming**: Concurrent data structures, atomic operations

---

## LEGACY KNOWLEDGE - Previously Used Technologies 📚

### 🗂️ Diesel ORM (Migrated to SQLx)
- **Connection Pooling**: r2d2 integration and connection management
- **Query Building**: .filter(), .select() patterns and query construction
- **Foreign Key Queries**: Table-qualified queries and relationship handling
- **Schema Usage**: Diesel schema files and compile-time query verification
- **Migration System**: Diesel CLI and database migration management

*Note: Migrated to SQLx for direct SQL control and better performance*
