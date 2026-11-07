//! Settings changes sent to the server with no reply yet, kept on the device.
//!
//! A client marks a change pending as it sends it and takes it off when the
//! save is answered, either way, so a failed save is the screen's to show and
//! only a change the app died before hearing back about stays. On the next
//! launch, or the next login by the same user, the client sends what is
//! pending before it takes the server's settings, and shows the pending
//! values over the server's while it cannot.

use uuid::Uuid;
use zwipe_core::domain::user::models::settings::UserSettings;
use zwipe_core::http::contracts::user::HttpUpdateSettings;

use crate::settings::SettingChange;

/// One user's unanswered changes, at most one per setting, newest last.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingSettings {
    user_id: Uuid,
    changes: Vec<SettingChange>,
}

/// Which setting a change writes. A theme pick and a mode toggle both write
/// the theme and its mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Setting {
    Theme,
    Pips,
    SwipeConfirmations,
    HideUniversesBeyond,
    Exceptions,
}

/// `change` as the record holds it: a mode toggle as the theme pick it
/// amounts to, the form an update reads back as.
fn normalized(change: SettingChange) -> SettingChange {
    match change {
        SettingChange::DarkMode { theme, dark_mode } => SettingChange::Theme { theme, dark_mode },
        other => other,
    }
}

fn setting(change: &SettingChange) -> Setting {
    match change {
        SettingChange::Theme { .. } | SettingChange::DarkMode { .. } => Setting::Theme,
        SettingChange::Pips(_) => Setting::Pips,
        SettingChange::SwipeConfirmations(_) => Setting::SwipeConfirmations,
        SettingChange::HideUniversesBeyond(_) => Setting::HideUniversesBeyond,
        SettingChange::Exceptions(_) => Setting::Exceptions,
    }
}

impl PendingSettings {
    /// What is pending once `change` goes out for `user_id`: it takes the
    /// place of an older change to the same setting. A record left by another
    /// user is dropped.
    #[must_use]
    pub fn record(previous: Option<Self>, user_id: Uuid, change: SettingChange) -> Self {
        let mut changes = previous
            .filter(|p| p.user_id == user_id)
            .map(|p| p.changes)
            .unwrap_or_default();
        let change = normalized(change);
        changes.retain(|c| setting(c) != setting(&change));
        changes.push(change);
        Self { user_id, changes }
    }

    /// What is still pending once the save of `change` is answered: it comes
    /// off unless a newer change to the same setting went out since. `None`
    /// once nothing is left.
    #[must_use]
    pub fn answered(mut self, change: &SettingChange) -> Option<Self> {
        let change = normalized(change.clone());
        self.changes.retain(|c| *c != change);
        (!self.changes.is_empty()).then_some(self)
    }

    /// The record when it is `user_id`'s.
    #[must_use]
    pub fn of(self, user_id: Uuid) -> Option<Self> {
        (self.user_id == user_id).then_some(self)
    }

    /// The user the changes are for.
    #[must_use]
    pub fn user_id(&self) -> Uuid {
        self.user_id
    }

    /// Every pending change as one sparse update.
    #[must_use]
    pub fn update(&self) -> HttpUpdateSettings {
        let mut update = empty_update();
        for change in &self.changes {
            let one = HttpUpdateSettings::from(change.clone());
            update.theme = one.theme.or(update.theme);
            update.dark_mode = one.dark_mode.or(update.dark_mode);
            update.pip_style = one.pip_style.or(update.pip_style);
            update.swipe_confirmations = one.swipe_confirmations.or(update.swipe_confirmations);
            update.exclude_universes_beyond = one
                .exclude_universes_beyond
                .or(update.exclude_universes_beyond);
            update.universes_beyond_exceptions = one
                .universes_beyond_exceptions
                .or(update.universes_beyond_exceptions);
        }
        update
    }

    /// The record that `update` stores for `user_id`, the inverse of
    /// [`update`](Self::update). `None` when the update sets nothing.
    #[must_use]
    pub fn from_update(user_id: Uuid, update: HttpUpdateSettings) -> Option<Self> {
        let mut changes = Vec::new();
        if let (Some(theme), Some(dark_mode)) = (update.theme, update.dark_mode) {
            changes.push(SettingChange::Theme { theme, dark_mode });
        }
        if let Some(style) = update.pip_style {
            changes.push(SettingChange::Pips(style));
        }
        if let Some(value) = update.swipe_confirmations {
            changes.push(SettingChange::SwipeConfirmations(value));
        }
        if let Some(hide) = update.exclude_universes_beyond {
            changes.push(SettingChange::HideUniversesBeyond(hide));
        }
        if let Some(slugs) = update.universes_beyond_exceptions {
            changes.push(SettingChange::Exceptions(slugs));
        }
        (!changes.is_empty()).then_some(Self { user_id, changes })
    }

    /// Writes every pending change over `settings`, the server's copy, the
    /// way the device shows them.
    pub fn apply(&self, settings: &mut UserSettings) {
        for change in &self.changes {
            change.apply(settings);
        }
    }
}

fn empty_update() -> HttpUpdateSettings {
    HttpUpdateSettings {
        theme: None,
        dark_mode: None,
        exclude_universes_beyond: None,
        universes_beyond_exceptions: None,
        pip_style: None,
        swipe_confirmations: None,
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::user::models::settings::{PipStyle, SwipeConfirmations};

    use super::*;

    fn dark(on: bool) -> SettingChange {
        SettingChange::DarkMode {
            theme: "gruvbox".to_owned(),
            dark_mode: on,
        }
    }

    #[test]
    fn a_change_stays_pending_until_its_save_is_answered() {
        let user = Uuid::new_v4();
        let pending = PendingSettings::record(None, user, dark(true));
        assert_eq!(pending.clone().answered(&dark(true)), None);
        assert_eq!(pending.user_id(), user);
    }

    #[test]
    fn a_newer_change_to_the_same_setting_outlives_the_older_reply() {
        let user = Uuid::new_v4();
        let first = PendingSettings::record(None, user, dark(true));
        let both = PendingSettings::record(Some(first), user, dark(false));
        let left = both.answered(&dark(true)).unwrap();
        assert_eq!(left.update().dark_mode, Some(false));
    }

    #[test]
    fn changes_to_different_settings_go_up_together() {
        let user = Uuid::new_v4();
        let one = PendingSettings::record(None, user, dark(true));
        let two = PendingSettings::record(Some(one), user, SettingChange::Pips(PipStyle::Classic));
        let update = two.update();
        assert_eq!(update.theme.as_deref(), Some("gruvbox"));
        assert_eq!(update.dark_mode, Some(true));
        assert_eq!(update.pip_style, Some(PipStyle::Classic));
        assert_eq!(update.swipe_confirmations, None);
        let left = two.answered(&dark(true)).unwrap();
        assert_eq!(left.update().dark_mode, None, "the theme's reply came");
    }

    #[test]
    fn another_users_record_is_dropped_and_never_handed_out() {
        let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
        let theirs = PendingSettings::record(None, a, dark(true));
        assert_eq!(theirs.clone().of(b), None);
        let mine = PendingSettings::record(
            Some(theirs),
            b,
            SettingChange::SwipeConfirmations(SwipeConfirmations::Off),
        );
        assert_eq!(mine.update().dark_mode, None);
        assert_eq!(mine.clone().of(b), Some(mine));
    }

    #[test]
    fn the_stored_update_reads_back_as_the_same_record() {
        let user = Uuid::new_v4();
        let one = PendingSettings::record(None, user, dark(false));
        let two =
            PendingSettings::record(Some(one), user, SettingChange::HideUniversesBeyond(true));
        let back = PendingSettings::from_update(user, two.update()).unwrap();
        assert_eq!(back.update().dark_mode, Some(false));
        assert_eq!(back.update().exclude_universes_beyond, Some(true));
        assert_eq!(PendingSettings::from_update(user, empty_update()), None);
    }

    #[test]
    fn a_reply_clears_a_change_read_back_from_the_store() {
        let user = Uuid::new_v4();
        let sent = PendingSettings::record(None, user, dark(true));
        let stored = PendingSettings::from_update(user, sent.update()).unwrap();
        assert_eq!(stored, sent);
        assert_eq!(stored.answered(&dark(true)), None);
    }

    #[test]
    fn pending_values_show_over_the_servers() {
        let user = Uuid::new_v4();
        let pending = PendingSettings::record(None, user, dark(true));
        let mut settings = UserSettings {
            dark_mode: false,
            ..UserSettings::default()
        };
        pending.apply(&mut settings);
        assert!(settings.dark_mode);
    }
}
