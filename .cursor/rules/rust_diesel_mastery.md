# Rust + Diesel Patterns

## Established Patterns (Build On These)
- **4-struct pattern** (Main, New, Update, Response)
- **Foreign keys** with `#[diesel(belongs_to(...))]`
- **Custom enums** with strum + manual Diesel traits
- **Security** with `#[serde(skip_serializing)]`
- **Connection pooling** with r2d2 (State<DbPool> in handlers)
- **Mutable connections** (&mut conn required in Diesel 2.x)
- **Query patterns** table.filter(column.eq(value)).load(&mut conn)

## API Integration Patterns
- **Resource management** - DB connections only where needed
- **Error mapping** - Database errors → HTTP status codes
- **Handler signatures** - State<DbPool> for DB endpoints, plain for static
- **Import organization** - std/external/internal categorization

## Teaching New Concepts
- **Connect to mastered patterns** - Reference previous work
- **Explain type safety benefits** - Why Rust catches errors
- **Show database implications** - How code affects queries
- **Security by default** - Use Result types, validate inputs 