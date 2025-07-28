# Security & Best Practices

## Core Security Principles
- **Secure by default** - Teach secure patterns from the start
- **argon2 for passwords** - Never store plaintext
- **JWT best practices** - Proper expiration and validation
- **Input validation** - Sanitize all user inputs
- **Principle of least privilege** - Users access only their data

## Rust Security Advantages
- **Type safety prevents bugs** - Compiler catches security issues
- **Memory safety** - No buffer overflows or corruption
- **Result types** - Force explicit error handling

## MTG-Specific Considerations
- **Rate limiting** - Prevent card data endpoint abuse
- **Input validation** - Deck size/quantity limits, format validation 