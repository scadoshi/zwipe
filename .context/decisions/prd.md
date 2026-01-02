# Deck Builder - Product Requirements Document

## Vision
A mobile app that makes Magic: The Gathering deck building fun and intuitive through a Tinder-like swiping interface. Users swipe through cards to build decks quickly and enjoyably.

## Core Problem
Building MTG decks is overwhelming for new players and time-consuming for experienced players. Current tools are complex, desktop-focused, and lack an engaging mobile experience.

## Solution
A mobile-first app where users swipe right to add cards to their deck, left to skip, creating an engaging and simple deck-building experience.

---

## Technical Architecture

### Tech Stack
- **Backend**: Rust (Axum framework) + PostgreSQL
- **Frontend**: Dioxus (Rust cross-platform) for iOS/Android/Web
- **API**: RESTful JSON API with shared Rust types
- **Architecture**: Full-stack Rust with shared models and business logic

### Core Data Models
```
User
├── id, email, username
├── created_at, updated_at
└── has_many :decks

Deck
├── id, name, user_id
├── format (standard, commander, etc.)
├── created_at, updated_at
└── has_many :deck_cards

Card
├── id, name, mana_cost
├── card_type, rarity, set_name
├── image_url, oracle_text
└── has_many :deck_cards

DeckCard
├── deck_id, card_id
├── quantity (1-4 copies)
└── belongs_to :deck, :card
```

---

## MVP Features (Core Functionality Only)

### 1. Card Swiping Interface
**User Story**: As a user, I want to swipe through cards to decide which ones to add to my deck.

**Functionality**:
- Display one card at a time with image and basic info
- **Right swipe**: Add card to current deck
- **Left swipe**: Skip card
- **Tap card**: View card details (larger image, rules text)
- Simple card queue system (pre-load next 10 cards)

**Success Criteria**:
- Smooth swiping animation
- Card images load quickly
- Clear visual feedback for swipe actions

### 2. Deck Management
**User Story**: As a user, I want to create, view, and manage my decks.

**Functionality**:
- Create new deck (name + format)
- View deck list (card names, quantities, total count)
- Remove cards from deck
- Delete entire deck
- Basic deck validation (60 card minimum for Standard)

**Success Criteria**:
- Deck saves automatically
- Deck list updates in real-time
- Clear deck statistics (total cards, colors)

### 3. User Authentication
**User Story**: As a user, I want to save my decks across sessions.

**Functionality**:
- Simple email/password registration
- Login/logout
- Persistent sessions
- Password reset (basic email-based)

**Success Criteria**:
- User stays logged in between app sessions
- Decks are tied to user account
- Basic security (password hashing)

### 4. Card Database
**User Story**: As a user, I want access to current Magic cards for deck building.

**Functionality**:
- Core set of Standard-legal cards (1000-2000 cards)
- Basic card information (name, cost, type, image)
- Simple filtering by format legality
- Card data seeded from public MTG API

**Success Criteria**:
- Cards display correctly with images
- Fast card loading
- Accurate card information

---

## User Flow (MVP)

```
1. User opens app
2. Login/Register (if not authenticated)
3. Tap "Build New Deck"
4. Enter deck name and format
5. Start swiping through cards
6. Right swipe adds to deck
7. View current deck at any time
8. Continue swiping until satisfied
9. Save and name deck
10. View deck in "My Decks" list
```

---

## UI/UX Requirements

### Mobile-First Design
- **Primary Screen**: Card swiping interface
- **Secondary Screen**: Deck list view
- **Tertiary Screen**: Individual deck details

### Core Screens
1. **Swipe Screen**: Full-screen card display with swipe gestures
2. **Deck List**: Simple list of user's decks
3. **Deck Detail**: List of cards in deck with quantities
4. **Login/Register**: Basic authentication forms

### Design Principles
- **Simple**: One primary action per screen
- **Fast**: Minimal loading states
- **Clear**: Obvious next steps for users
- **Mobile-Optimized**: Thumb-friendly touch targets

---

## Success Metrics (MVP)

### User Engagement
- Cards swiped per session (target: 50+)
- Decks created per user (target: 2+)
- Session length (target: 5+ minutes)
- Return users within 7 days (target: 30%+)

### Technical Performance
- App launch time < 3 seconds
- Card swipe response < 100ms
- Image load time < 2 seconds
- API response time < 500ms

---

## MVP Scope Boundaries

### What's INCLUDED:
- Basic card swiping
- Simple deck creation and viewing
- User accounts and data persistence
- Core MTG card database
- Mobile app for iOS and Android

### What's EXCLUDED (Future Versions):
- ~~AI recommendations~~ → **Future: Keyword-based synergy suggestions**
- ~~Advanced deck analysis~~ → **Future: Mana curve, power level analysis**
- ~~Mana curve visualization~~
- ~~Card price integration~~
- ~~Social features/sharing~~
- ~~Multiple TCG support~~
- ~~Advanced filtering~~ → **Future: Keyword search with GIN indexes**
- ~~Deck importing/exporting~~
- ~~Tournament tools~~
- ~~Real-time features~~

### Future Enhancement: Intelligent Card Recommendations
**Post-MVP Feature**: Advanced search and recommendation system
- **Keywords extraction**: Process oracle text to extract searchable terms like "flying", "graveyard_recursion", "tribal_synergy"
- **Synergy detection**: Algorithm to suggest cards that work well together based on keyword overlap
- **Performance optimization**: PostgreSQL GIN indexes for fast keyword array searches
- **Use cases**: "Show me more graveyard cards for this deck", "Cards that synergize with Lightning Bolt"

---

## Development Phases

### Phase 1: Backend Foundation (3-4 weeks)
- Rust API setup with Axum framework
- User authentication with JWT
- Basic card and deck models with Diesel ORM
- PostgreSQL database setup
- Card data seeding

### Phase 2: Dioxus Mobile App (3-4 weeks)
- Dioxus project setup for mobile targets
- Shared type definitions with backend
- Component-based UI architecture
- API integration with shared HTTP client
- Card swiping interface with touch gestures

### Phase 3: MVP Polish (1-2 weeks)
- Bug fixes and performance optimization
- Basic error handling
- App store preparation
- Simple onboarding flow

**Total MVP Timeline: 6-9 weeks**

---

## Target Audience
- **Primary**: MTG players who want a fun, mobile deck-building experience
- **Secondary**: New MTG players looking for simple deck creation tools
- **Demographic**: 18-35 years old, mobile-first users

## Platform Priority
1. **Android** (can test natively on Linux)
2. **iOS** (deploy via cloud builds)

This PRD focuses exclusively on proving the core concept: "Can we make deck building fun through swiping?" Everything else is deferred until we validate this fundamental assumption. 