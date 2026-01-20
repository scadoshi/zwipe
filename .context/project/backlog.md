# Backlog - Future Development 📋

Planned features and improvements for future implementation.

---

## 🚀 Production System Hardening
- **Rate Limiting Implementation**: Request throttling, abuse prevention, and API protection
- **Performance Optimization**: Query optimization, connection pool tuning, database indexing
- **Monitoring & Observability**: Structured logging, metrics collection, health monitoring
- **Caching Layer**: Redis integration for card data and query optimization

## 🎮 Advanced MTG Features
- **Enhanced Card Search**: Format legality, power/toughness filtering, advanced search operators
- **Deck Validation System**: Format legality checking, card limit enforcement
- **Collection Management**: User card ownership tracking, wishlist functionality
- **Deck Analytics**: Mana curve analysis, card type distribution

## 🧪 Testing & Quality Assurance
- **Handler Test Suites**: Comprehensive unit tests for auth, health, deck, and card handlers
- **JWT Middleware Tests**: Security boundary validation, error response testing
- **Integration Test Framework**: Full HTTP request/response testing infrastructure
- **Performance Testing**: Load testing, connection pool optimization
- **End-to-End Test Suite**: Complete user workflow validation

## 🚀 Production Features
- **Rate Limiting**: Request throttling and abuse prevention mechanisms
- **Caching Layer**: Redis integration for card data and query optimization
- **Monitoring & Logging**: Structured logging, metrics collection, health monitoring
- **Database Optimization**: Query performance analysis, indexing strategy
- **Image Handling**: Card image serving, caching, and mobile optimization

## 🎮 MTG-Specific Features
- **Advanced Card Search**: Format legality, power/toughness filtering, advanced search operators
- **Deck Validation**: Format legality checking, card limit enforcement
- **Collection Management**: User card ownership tracking, wishlist functionality
- **Deck Analytics**: Mana curve analysis, card type distribution
- **Import/Export**: Support for various deck formats (MTGA, MTGO, etc.)

## 📱 Mobile Application Features
- **Offline Support**: PWA capabilities for offline deck management
- **Advanced Filtering**: Complex search queries, saved filters
- **Social Features**: Deck sharing, public deck browser, user profiles
- **Real-time Updates**: WebSocket integration for live deck collaboration

## 🚢 iOS/Android Deployment
- **iOS Keychain Entitlements**: Configure Dioxus.toml with bundle identifier and keychain-access-groups for persistent session storage
- **Android KeyStore Configuration**: Verify keyring crate configuration for Android secure storage
- **App Signing**: Set up iOS/Android code signing for device deployment
- **Store Submission**: Prepare assets and metadata for App Store/Play Store submission

## 🌍 Multi-Language Application Support
**Post-MVP Feature**: Internationalization (i18n) for entire application UI
- **Card Language Infrastructure**: ✅ Complete - Dynamic language support already implemented
  - Backend: `/api/card/languages` endpoint with 18+ languages
  - Domain: String-based language filtering with LanguageCodeToFullName trait
  - Database: Supports all Scryfall language codes without code changes
  - Currently using OracleCards (English-only) but infrastructure ready for AllCards
- **Application UI Translation**: Frontend screens, labels, and messages
  - Translation system for all UI text (auth, deck management, filters, etc.)
  - Language selection in user settings
  - Locale-aware date/number formatting
  - Right-to-left (RTL) support for Arabic/Hebrew
- **Backend Messages**: Error messages, validation feedback, API responses
- **Content Strategy**: Determine which languages to prioritize based on MTG player demographics

## 🔮 Future Enhancement: Intelligent Card Recommendations
**Post-MVP Feature**: Advanced search and recommendation system
- **Keywords extraction**: Process oracle text to extract searchable terms like "flying", "graveyard_recursion", "tribal_synergy"
- **Synergy detection**: Algorithm to suggest cards that work well together based on keyword overlap
- **Performance optimization**: PostgreSQL GIN indexes for fast keyword array searches
- **Use cases**: "Show me more graveyard cards for this deck", "Cards that synergize with Lightning Bolt"
