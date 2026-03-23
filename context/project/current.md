# Currently Working On

Active development tasks and immediate focus areas.

**Last Updated**: Completed generics sweep — all domain validators accept `impl AsRef<str>`.

**Current Focus**: Bug fixes and UI polish.

**Recent Achievements**:
- **Generics Sweep**: All domain validator constructors now accept `impl AsRef<str>` (or `impl Into<String>` for `DeckName`).
  - `Username::new`, `Password::new` — core domain types
  - `RawRegisterUser::new`, `RegisterUser::new`, `ChangeUsername::new`, `ChangePassword::new`, `AuthenticateUser::new`, `CreateDeckProfile::new` — request constructors cascaded
  - SQLx outbound models now consume owned `String`s directly (no `&value.field` borrow)
  - Frontend screens pass signal values directly (no `&signal()` borrow)
- **Deref Refactor (zerver)**: All domain newtypes implement `Deref` instead of bespoke getters
- **Unit Tests**: Full coverage on all pure domain logic (filter_cards 24, group_cards 15, copy_max 9, quantity, SwipeState 32)

---

## Top 5 Priorities

1. **Bug Fixes** - Layout shift after deck creation, iOS keyboard push issues

2. **CardFilter Enhancements** - Refine default filter to exclude non-playable cards

3. **Polish & UX** - Tri-toggle labels ("any" instead of "neither"), language filter refinement

4. **Deck List Redesign** - Better list styling, improved layout with utility bar

5. **Integration Tests** - Repository tests with real PostgreSQL (longer-term)
