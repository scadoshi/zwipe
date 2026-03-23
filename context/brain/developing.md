# DEVELOPING - Active Implementation (Working But Learning) 🔧

Working implementations where you're still building deep understanding.

---

## 🎨 iOS Safe Area & Mobile Layout (Working Implementation)
- **viewport-fit=cover Understanding**: Conceptual grasp of safe vs unsafe zones, header extending into notch area
- **Safe Area Purpose**: Prevents app elements from being affected by system UI (notch, home indicator)
- **Practical Application**: Successfully implemented header extending into notch to cover scrolling content
- **Negative Margin + Padding Pattern**: Negative margins push element boundaries upward without moving visual content
- *Note: Good conceptual understanding, needs more practice for full confidence teaching others*

## 🔄 Refresh & State Reset Patterns (Recently Applied)
- **Refresh Trigger Pattern**: Bool signal toggle (`refresh_trigger.set(!refresh_trigger())`) forcing use_effect re-run
- **Pagination Exhaustion Tracking**: Signal tracking when API returns no unique cards, used to show feedback
- **State Reset on Filter Change**: Resetting offset, index, and exhaustion flag when filter changes
- **Conditional Load vs Feedback**: Check exhaustion before calling load_more, show toast if exhausted
- *Note: Applied successfully in add card screen, pattern reusable for similar scenarios*

---

Previously DEVELOPING items graduated to **[confident.md](confident.md)**:
- Toast API Patterns
- Filter UI Patterns (Filter Reset Signal, Direct Write Pattern)
- Frontend Deck CRUD & Complete Flow
- Swipe-Based Navigation & Gestures
- Service Architecture & Dependency Injection
- Feature Flag Architecture & Shared Models
- Advanced Architecture Patterns
- Testing & Validation
- External API Integration & Data Processing
- Advanced Type Systems
- Backend Session & Token Architecture
- Frontend Session Management
- Card Filtering System & Modular Architecture
- Deck Card Management & Swipe Navigation
