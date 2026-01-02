# Project Development Tracker

Tracks project development status and provides development context for AI assistants.

**Last Updated**: Restructured into modular files for better AI context management.

---

## Project Status Index

Navigate to specific status files:

- **[current.md](project/current.md)** - Active work and top 5 priorities
- **[next.md](project/next.md)** - Immediate next steps after current work
- **[complete-backend.md](project/complete-backend.md)** - Production-ready backend implementations
- **[complete-frontend.md](project/complete-frontend.md)** - Production-ready frontend implementations
- **[backlog.md](project/backlog.md)** - Future planned features and improvements

---

## Update Instructions for AI Assistants

**Status Categories**:
- CURRENT = Active work right now (top 5-10 items)
- NEXT = Planned immediate priorities
- COMPLETE = Production-ready implementations (split by frontend/backend)
- BACKLOG = Future planned work

**When to Update**: After major feature completions, architectural decisions, or priority changes. Move items between categories as development progresses.

**Development Strategy**: Focus on completing current priorities before starting new work. Maintain clean architecture and comprehensive error handling throughout.

**File Structure**: Keep files focused and under 400 lines for efficient AI context loading. Frontend and backend complete items are separated to prevent mixing concerns.

---

## Quick Context

**Tech Stack**: Rust backend (Axum, SQLx, PostgreSQL, JWT auth), Dioxus frontend (web/mobile), hexagonal architecture

**Current Phase**: UX improvements - deck list redesign, view deck categorization, remove cards workflow, toast notifications

**Recent Achievement**: Complete AlertDialog modal system with global CSS loading fix. All confirmation workflows now use consistent modal pattern.
