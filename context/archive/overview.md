# Project Development Tracker

Tracks project development status and provides development context for AI assistants.

**Last Updated**: 2026-03-26.

---

## Project Status Index

Navigate to specific status files:

- **[current.md](project/current.md)** - Active work and top 5 priorities
- **[next.md](project/next.md)** - Immediate next steps after current work
- **[complete_backend.md](project/complete_backend.md)** - Production-ready backend implementations
- **[complete_frontend.md](project/complete_frontend.md)** - Production-ready frontend implementations
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

**Current Phase**: Live in production — app running on iPhone, backend on Raspberry Pi 5 at `api.zwipe.net`. Active work is UX polish from first real-device testing.

**Recent Achievement**: Full production deployment (2026-03-26). iOS app signed and deployed to physical device via `ios-deploy`. zerver cross-compiled for aarch64 running as systemd service on Pi 5 behind Cloudflare Tunnel. iOS Keychain session persistence confirmed working.
