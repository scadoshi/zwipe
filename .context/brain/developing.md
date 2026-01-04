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

## 🍞 Toast API Patterns (Recently Applied)
- **Toast Types**: toast.info(), toast.success(), toast.warning(), toast.error() for different contexts
- **ToastOptions Builder**: ToastOptions::default().duration(Duration::from_millis(1500)) for custom timing
- **Swipe Feedback**: Using toast.info("skipped") and toast.success("added to deck") for immediate user feedback
- **End State Feedback**: toast.warning() for "end of results" when pagination exhausted
- *Note: Working well, may need to explore description field and permanent options*

## 🎛️ Filter UI Patterns (Recently Applied)
- **FilterMode Generic Enum**: Reusable enum (Equals/Range) for switching between filter modes
- **Stepper Controls**: Increment/decrement buttons for numeric filters (power/toughness)
- **Direct Write Pattern**: Eliminating local signal + use_effect by writing directly to context filter
- **Filter Reset Signal**: Incrementing counter to collapse accordions and clear search queries on apply
- *Note: Filter architecture solid, applying same patterns to new filter types*

---

Previously DEVELOPING items graduated to **[confident.md](confident.md)**:
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
