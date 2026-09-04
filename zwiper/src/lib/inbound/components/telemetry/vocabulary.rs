//! Closed vocabulary for error-report breadcrumbs.
//!
//! The single home for every `screen`/`component` value that may ride a
//! [`ClientErrorReport`] — names are the flattened module path of the screen
//! or component that surfaces the error (`screens/deck/edit.rs` →
//! `deck_edit`), so the const is derivable from the file you're standing in.
//! Free-form strings never enter reports: the server aggregates on `screen`,
//! and a typo'd axis fragments every rollup query.
//!
//! `component` is the drill-down: `""` (via [`component::NONE`]) when the
//! screen itself surfaced the error, else the component's module name.
//! Dialogs and sheets report their HOST screen plus their own component name.
//!
//! [`ClientErrorReport`]: zwipe_core::http::contracts::metrics::ClientErrorReport

/// Host screens (`screens/` module paths, flattened).
#[allow(missing_docs)]
pub mod screen {
    pub const HOME: &str = "home";
    pub const AUTH_LOGIN: &str = "auth_login";
    pub const AUTH_REGISTER: &str = "auth_register";
    pub const AUTH_FORGOT_PASSWORD: &str = "auth_forgot_password";
    pub const CHANGELOG: &str = "changelog";
    pub const COMMANDER_MAYBEBOARD: &str = "commander_maybeboard";
    pub const DECK_LIST: &str = "deck_list";
    pub const DECK_CREATE: &str = "deck_create";
    pub const DECK_EDIT: &str = "deck_edit";
    pub const DECK_VIEW: &str = "deck_view";
    pub const DECK_IMPORT: &str = "deck_import";
    pub const DECK_EXPORT: &str = "deck_export";
    pub const DECK_CARD_ADD: &str = "deck_card_add";
    pub const DECK_CARD_REMOVE: &str = "deck_card_remove";
    pub const DECK_CARD_VIEW: &str = "deck_card_view";
    pub const ORACLE_TAG_DICTIONARY: &str = "oracle_tag_dictionary";
    pub const ORACLE_TAG_EXAMPLES: &str = "oracle_tag_examples";
    pub const PROFILE: &str = "profile";
    pub const PROFILE_CHANGE_EMAIL: &str = "profile_change_email";
    pub const PROFILE_CHANGE_PASSWORD: &str = "profile_change_password";
    pub const PROFILE_CHANGE_USERNAME: &str = "profile_change_username";
    pub const PROFILE_PREFERENCES: &str = "profile_preferences";
}

/// Typed screen identity for the authed facade: the same closed vocabulary as
/// [`screen`], carried as structured data instead of a bare string. Grouped
/// the way the screens modules are (auth/, deck/, deck/card/, profile/), so a
/// failure site reads as where it lives. `as_str` maps every variant onto its
/// [`screen`] constant byte-for-byte: the wire format and the server's
/// `client_errors` dedupe keys never change, the enum only adds compile-time
/// structure on top.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// The home screen.
    Home,
    /// The changelog screen.
    Changelog,
    /// The account-wide commander maybeboard screen.
    CommanderMaybeboard,
    /// Screens under `screens/auth/`.
    Auth(AuthScreen),
    /// Screens under `screens/deck/` (card subscreens included).
    Deck(DeckScreen),
    /// The oracle-tag dictionary screens.
    OracleTag(OracleTagScreen),
    /// Screens under `screens/profile/`.
    Profile(ProfileScreen),
}

/// `screens/auth/` members.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthScreen {
    Login,
    Register,
    ForgotPassword,
}

/// `screens/deck/` members, `card/` subscreens included.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeckScreen {
    List,
    Create,
    Edit,
    View,
    Import,
    Export,
    CardAdd,
    CardRemove,
    CardView,
}

/// Oracle-tag dictionary screens.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OracleTagScreen {
    Dictionary,
    Examples,
}

/// `screens/profile/` members.
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileScreen {
    Main,
    ChangeEmail,
    ChangePassword,
    ChangeUsername,
    Preferences,
}

impl Screen {
    /// The wire string for this screen: exactly the matching [`screen`]
    /// constant, never a new value.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Home => screen::HOME,
            Self::Changelog => screen::CHANGELOG,
            Self::CommanderMaybeboard => screen::COMMANDER_MAYBEBOARD,
            Self::Auth(AuthScreen::Login) => screen::AUTH_LOGIN,
            Self::Auth(AuthScreen::Register) => screen::AUTH_REGISTER,
            Self::Auth(AuthScreen::ForgotPassword) => screen::AUTH_FORGOT_PASSWORD,
            Self::Deck(DeckScreen::List) => screen::DECK_LIST,
            Self::Deck(DeckScreen::Create) => screen::DECK_CREATE,
            Self::Deck(DeckScreen::Edit) => screen::DECK_EDIT,
            Self::Deck(DeckScreen::View) => screen::DECK_VIEW,
            Self::Deck(DeckScreen::Import) => screen::DECK_IMPORT,
            Self::Deck(DeckScreen::Export) => screen::DECK_EXPORT,
            Self::Deck(DeckScreen::CardAdd) => screen::DECK_CARD_ADD,
            Self::Deck(DeckScreen::CardRemove) => screen::DECK_CARD_REMOVE,
            Self::Deck(DeckScreen::CardView) => screen::DECK_CARD_VIEW,
            Self::OracleTag(OracleTagScreen::Dictionary) => screen::ORACLE_TAG_DICTIONARY,
            Self::OracleTag(OracleTagScreen::Examples) => screen::ORACLE_TAG_EXAMPLES,
            Self::Profile(ProfileScreen::Main) => screen::PROFILE,
            Self::Profile(ProfileScreen::ChangeEmail) => screen::PROFILE_CHANGE_EMAIL,
            Self::Profile(ProfileScreen::ChangePassword) => screen::PROFILE_CHANGE_PASSWORD,
            Self::Profile(ProfileScreen::ChangeUsername) => screen::PROFILE_CHANGE_USERNAME,
            Self::Profile(ProfileScreen::Preferences) => screen::PROFILE_PREFERENCES,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every variant maps onto the closed vocabulary, one to one: the enum
    /// can neither invent a wire string nor leave a constant unreachable.
    #[test]
    fn screen_enum_covers_the_vocabulary_exactly() {
        let variants = [
            Screen::Home,
            Screen::Changelog,
            Screen::CommanderMaybeboard,
            Screen::Auth(AuthScreen::Login),
            Screen::Auth(AuthScreen::Register),
            Screen::Auth(AuthScreen::ForgotPassword),
            Screen::Deck(DeckScreen::List),
            Screen::Deck(DeckScreen::Create),
            Screen::Deck(DeckScreen::Edit),
            Screen::Deck(DeckScreen::View),
            Screen::Deck(DeckScreen::Import),
            Screen::Deck(DeckScreen::Export),
            Screen::Deck(DeckScreen::CardAdd),
            Screen::Deck(DeckScreen::CardRemove),
            Screen::Deck(DeckScreen::CardView),
            Screen::OracleTag(OracleTagScreen::Dictionary),
            Screen::OracleTag(OracleTagScreen::Examples),
            Screen::Profile(ProfileScreen::Main),
            Screen::Profile(ProfileScreen::ChangeEmail),
            Screen::Profile(ProfileScreen::ChangePassword),
            Screen::Profile(ProfileScreen::ChangeUsername),
            Screen::Profile(ProfileScreen::Preferences),
        ];
        let mut mapped: Vec<&str> = variants.iter().map(|v| v.as_str()).collect();
        mapped.sort_unstable();
        let mut constants = vec![
            screen::HOME,
            screen::AUTH_LOGIN,
            screen::AUTH_REGISTER,
            screen::AUTH_FORGOT_PASSWORD,
            screen::CHANGELOG,
            screen::COMMANDER_MAYBEBOARD,
            screen::DECK_LIST,
            screen::DECK_CREATE,
            screen::DECK_EDIT,
            screen::DECK_VIEW,
            screen::DECK_IMPORT,
            screen::DECK_EXPORT,
            screen::DECK_CARD_ADD,
            screen::DECK_CARD_REMOVE,
            screen::DECK_CARD_VIEW,
            screen::ORACLE_TAG_DICTIONARY,
            screen::ORACLE_TAG_EXAMPLES,
            screen::PROFILE,
            screen::PROFILE_CHANGE_EMAIL,
            screen::PROFILE_CHANGE_PASSWORD,
            screen::PROFILE_CHANGE_USERNAME,
            screen::PROFILE_PREFERENCES,
        ];
        constants.sort_unstable();
        assert_eq!(mapped, constants);
    }
}

/// Components (module names) — the optional drill-down under a host screen.
#[allow(missing_docs)]
pub mod component {
    /// The screen itself surfaced the error (no component breadcrumb).
    pub const NONE: &str = "";
    pub const CARD_FILTER_SHEET: &str = "card_filter_sheet";
    pub const CLONE_DECK_DIALOG: &str = "clone_deck_dialog";
    pub const DECK_WARNINGS: &str = "deck_warnings";
    pub const DELETE_ACCOUNT_DIALOG: &str = "delete_account_dialog";
    pub const EMAIL_VERIFICATION: &str = "email_verification";
    pub const MORE_BUTTONS: &str = "more_buttons";
    pub const PRINTING_SHEET: &str = "printing_sheet";
    pub const QUICK_ADD: &str = "quick_add";
    pub const SWIPE_SELECT: &str = "swipe_select";
}
