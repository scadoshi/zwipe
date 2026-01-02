# Currently Working On 🎯

Active development tasks and immediate focus areas.

**Last Updated**: AlertDialog component implementation complete with global CSS loading fix. Discovered document::Link inside components doesn't reliably load CSS—solution is global loading in main.rs. Replaced all inline yes/no confirmation prompts with styled AlertDialog modals. Standardized utility bar button placement across all 20+ screens for consistent navigation UX.

**Current Focus**: Deck list and edit screen UX improvements, view deck card categorization, remove cards workflow, toast notification system for swipe feedback.

**Recent Achievement**: Solved CSS loading mystery for dioxus_primitives components—document::Link elements nested inside components don't render to <head> reliably. Moved accordion, alert-dialog, and toast CSS to global app-level loading ensuring styles available on mount.

**Current Success**: Complete AlertDialog modal system with centered positioning, dark backdrop, styled buttons, and proper animations. All confirmation workflows (logout, delete deck) now use consistent modal pattern. Utility bars standardized across auth, deck, and profile screens with util-btn styling.

**Current Challenge**: Improve deck list visual design for many decks, refine edit deck labels and field names, build view deck categorization by card type, implement remove cards screen with filtering.

---

## Top 5 Priorities

1. **Deck List Screen Redesign** - Better list styling for many decks, improved layout working with utility bar, visual hierarchy

2. **Edit Deck Screen Refinement** - Better labels, rename "card copy rule" to clearer terminology, improve visual coherence

3. **View Deck Screen** - Display deck cards categorized by type (creatures, spells, lands, etc.) with counts and organization

4. **Remove Cards Screen** - Build screen for removing cards from deck with filtering and swipe gestures

5. **Toast Notification System** - Add toast feedback for card add/skip/remove actions teaching user which swipe directions do what
