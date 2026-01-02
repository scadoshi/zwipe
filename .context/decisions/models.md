# Models: Shared Code Architecture Decision

## The Problem

When building both a Rust backend (zerver) and Dioxus frontend (zwiper), I faced the classic problem of code duplication. Both sides needed the same data structures (User, Deck, Card) and HTTP request types (HttpLoginUser, HttpRegisterUser). The backend already had a beautiful hexagonal architecture working perfectly, but I needed to share models without duplicating code or breaking the existing workflow.

## My Three Options

**Option 1: Separate Shared Library (ztructs)**
Create a dedicated shared library containing all models. Both zerver and zwiper import from it.
- ✅ Clean separation, minimal frontend binary
- ❌ Breaks my amazing workflow, requires extracting everything from zerver, three-way coordination

**Option 2: Shared Library with Feature Flags**
Same as Option 1 but use #[cfg] tags to keep related concepts together.
- ✅ Less file fragmentation than Option 1
- ❌ Still breaks workflow, still requires extraction, #[cfg] tags everywhere

**Option 3: Import Directly from zerver**
Keep all models in zerver, add feature flags to hide server-only stuff. Frontend imports from zerver.
- ✅ Preserves workflow, no extraction needed, single source of truth
- ❌ Feels architecturally "weird" to import from backend

## My Initial Confusion

I got hung up on the idea that "frontend shouldn't import from backend" - it felt wrong architecturally. I started implementing Option 1, creating ztructs and extracting models, but it felt like I was breaking something that worked perfectly.

## The Breakthrough Insight

Then I realized: **zerver IS my domain layer**. I wasn't building a "backend" - I was building a domain-centric application with multiple adapters:
- SQLx adapters (for database)  
- HTTP adapters (for REST API)
- Dioxus adapter (for UI) ← This is zwiper!

In hexagonal architecture, adapters are *supposed* to depend on the domain. The frontend importing from zerver isn't wrong - it's exactly right.

## My Decision: Option 3

**Why I chose this:**

1. **Workflow preservation**: My current zerver workflow is amazing. Add domain model → build SQLx repo → build HTTP handler. Why break something that works?

2. **Architectural clarity**: Once I stopped thinking "backend vs frontend" and started thinking "domain vs adapters", it made perfect sense. zwiper is just another adapter.

3. **Practical benefits**: Single source of truth, no extraction work, no three-way coordination, feature flags are standard Rust.

4. **Future flexibility**: I can always extract to a separate domain library later if the project grows huge, but YAGNI applies here.

**Implementation:**
- Keep all models in zerver/domain/ 
- Add feature flags to hide server-only adapters (SQLx, HTTP server)
- zwiper imports zerver without server features
- Clean, simple, works

**The lesson**: Sometimes the "obvious" architectural choice isn't right for your specific situation. Developer experience and workflow efficiency matter. The best architecture is the one that helps you ship great software, not the one that looks perfect on paper.
