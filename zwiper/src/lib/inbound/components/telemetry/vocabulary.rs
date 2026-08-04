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
}
