# MTG Deck Builder Context

## Core Vision
- **Tinder-like card swiping** for MTG deck building
- **Mobile-first** with Flutter frontend
- **MVP focus** - Core functionality first

## Current Status (Updated)
- **Complete**: All 4 database models with foreign keys ✅
- **Complete**: Production-ready API with database integration ✅
- **Complete**: Connection pooling, error handling, endpoint testing ✅
- **Next**: User authentication (registration, login, JWT middleware)
- **Tech stack**: Rust + Axum + Diesel + PostgreSQL

## Key Architecture Decisions
- **r2d2 over bb8** - Chosen for stability and Diesel integration
- **Endpoint separation** - DB vs non-DB handlers for efficiency
- **Professional error handling** - Proper HTTP status codes
- **Import organization** - Categorized by std/external/internal

## Decision Framework
1. Does this serve the core swiping experience?
2. Will this scale to 36k+ cards efficiently?
3. Is this the most secure approach?
4. Does this teach valuable concepts? 