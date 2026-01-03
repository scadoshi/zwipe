# Currently Working On 🎯

Active development tasks and immediate focus areas.

**Last Updated**: Fixed iOS safe area overlap issue preventing deck list entries from showing in notch area. Implemented viewport-fit=cover, overflow: clip with clip-margin, and env(safe-area-inset-*) variables for proper iOS handling. Resolved keyboard util-bar repositioning bug and standardized sticky positioning across all screens.

**Current Focus**: CSS overhaul to reduce contrast and create more "normal" looking dark mode experience. Then continue application flow: ensure filter works correctly, finish add card screen, implement remove card screen.

**Recent Achievement**: Solved complex iOS safe area rendering issue where scrolling content appeared behind the notch. Used modern CSS (overflow: clip, overflow-clip-margin) combined with viewport-fit=cover meta tag and CSS environment variables for iOS-safe layouts. Fixed iOS keyboard bug using GPU acceleration with transform: translateZ(0).

**Current Success**: Complete iOS-safe layout system with proper notch handling, keyboard behavior, and sticky header positioning. Content now clips at safe area boundaries preventing overlap in notch/home indicator areas.

**Current Challenge**: Application has high contrast appearance that needs refinement. Dark mode styling needs to be more subtle and "normal" looking. After visual polish, focus on filter functionality verification and completing card management workflows.

---

## Top 5 Priorities

1. **Deck List Screen Redesign** - Better list styling for many decks, improved layout working with utility bar, visual hierarchy

2. **Edit Deck Screen Refinement** - Better labels, rename "card copy rule" to clearer terminology, improve visual coherence

3. **View Deck Screen** - Display deck cards categorized by type (creatures, spells, lands, etc.) with counts and organization

4. **Remove Cards Screen** - Build screen for removing cards from deck with filtering and swipe gestures

5. **Toast Notification System** - Add toast feedback for card add/skip/remove actions teaching user which swipe directions do what
