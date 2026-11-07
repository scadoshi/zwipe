//! The home screen's figures, scope, flavor card expiry and greeting. The
//! buy links live in [`crate::buy`] and the Colors panel's bars in
//! [`crate::colors`].

use chrono::{DateTime, Local, TimeDelta, Timelike, Utc};
use zwipe_core::domain::user::User;
use zwipe_core::http::contracts::metrics::{HttpLifetimeCounters, HttpPublicMetrics};

use crate::toast::{Toast, ToastKind};

/// The three counters under the logo, each `None` while it loads or when
/// it is no number to show. Negative counts read as zero.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Figures {
    /// Decks created.
    pub decks_created: Option<u64>,
    /// Searches run.
    pub searches: Option<u64>,
    /// Cards swiped, every direction together.
    pub cards_swiped: Option<u64>,
}

/// What the tag under the counters says about how new they are.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CountsTag {
    /// Made under [`COUNTS_LIVE`](crate::timing::COUNTS_LIVE) ago.
    Live,
    /// Made longer ago, at this time.
    AsOf(DateTime<Utc>),
}

impl CountsTag {
    /// The tag for a copy made at `made_at`, seen at `now`. A copy that does
    /// not say when it was made came straight from the server and is live.
    #[must_use]
    pub fn at(made_at: Option<DateTime<Utc>>, now: DateTime<Utc>) -> Self {
        let window = TimeDelta::from_std(crate::timing::COUNTS_LIVE).unwrap_or(TimeDelta::MAX);
        match made_at {
            Some(made) if now.signed_duration_since(made) >= window => Self::AsOf(made),
            _ => Self::Live,
        }
    }

    /// `Live`, or "as of" the copy's time in the device's time zone, as
    /// `as of 14:05`.
    #[must_use]
    pub fn label(self) -> String {
        match self {
            Self::Live => crate::screens::home::LIVE.to_owned(),
            Self::AsOf(made) => format!("as of {}", made.with_timezone(&Local).format("%H:%M")),
        }
    }
}

/// A count the server keeps signed, never below zero.
fn count(n: i64) -> u64 {
    u64::try_from(n).unwrap_or(0)
}

impl From<&HttpPublicMetrics> for Figures {
    fn from(metrics: &HttpPublicMetrics) -> Self {
        Self {
            decks_created: Some(count(metrics.decks_created)),
            searches: Some(count(metrics.searches)),
            cards_swiped: Some(count(metrics.cards_swiped)),
        }
    }
}

impl From<&HttpLifetimeCounters> for Figures {
    /// The user's own counters. Every swipe direction counts as a card
    /// swiped, and a sum past `i64` is no number to show, so it waits like a
    /// loading one, as in the app.
    fn from(counters: &HttpLifetimeCounters) -> Self {
        let swiped = counters
            .swipes_right
            .checked_add(counters.swipes_left)
            .and_then(|n| n.checked_add(counters.swipes_up))
            .and_then(|n| n.checked_add(counters.swipes_down));
        Self {
            decks_created: Some(count(i64::from(counters.decks_created))),
            searches: Some(count(counters.searches)),
            cards_swiped: swiped.map(count),
        }
    }
}

impl Figures {
    /// The labels under the three figures, in order.
    pub const LABELS: [&'static str; 3] = ["Decks created", "Searches run", "Cards swiped"];

    /// The three values in the order of [`Figures::LABELS`].
    #[must_use]
    pub fn values(self) -> [Option<u64>; 3] {
        [self.decks_created, self.searches, self.cards_swiped]
    }
}

/// Whose counters the home screen shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Scope {
    /// Everyone's counters, the default.
    #[default]
    Everybody,
    /// The signed-in user's own counters.
    OnlyMe,
}

impl Scope {
    /// The scope chip's label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Everybody => "Everybody",
            Self::OnlyMe => "Only me",
        }
    }

    /// The other scope, what a tap on the chip switches to.
    #[must_use]
    pub fn toggled(self) -> Self {
        match self {
            Self::Everybody => Self::OnlyMe,
            Self::OnlyMe => Self::Everybody,
        }
    }

    /// The bytes the app stores the scope as: `all` or `mine`.
    #[must_use]
    pub fn as_bytes(self) -> &'static [u8] {
        match self {
            Self::Everybody => b"all",
            Self::OnlyMe => b"mine",
        }
    }

    /// The scope stored as `bytes`, reading anything but `mine` as everybody.
    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes == b"mine" {
            Self::OnlyMe
        } else {
            Self::Everybody
        }
    }
}

/// The latest point into an hour [`next_hour`] counts, so the gap it adds
/// is always at least a second.
const LAST_SECOND_OF_HOUR: chrono::Duration = chrono::Duration::seconds(3599);

/// The top of the UTC hour after `now`, when the server picks a new featured
/// card. It keeps `now`'s fraction of a second, so the gap is whole seconds
/// and at least one, and past the end of time it is the end of time.
#[must_use]
pub fn next_hour(now: DateTime<Utc>) -> DateTime<Utc> {
    let hour_start = now
        .with_minute(0)
        .and_then(|t| t.with_second(0))
        .unwrap_or(now);
    let into_hour = now
        .signed_duration_since(hour_start)
        .min(LAST_SECOND_OF_HOUR);
    chrono::Duration::hours(1)
        .checked_sub(&into_hour)
        .and_then(|gap| now.checked_add_signed(gap))
        .unwrap_or(DateTime::<Utc>::MAX_UTC)
}

/// The nudge an unverified user gets on Home.
pub const VERIFY_EMAIL: &str = "Verify your email!";

/// Where the user Home greets came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserSource {
    /// Home's own refresh of the user, made each time it opens.
    Fetched,
    /// The session's copy, used when that refresh failed.
    Cached,
}

/// The toast Home raises once it knows who the user is.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Greeting {
    /// "Hello, {username}!" for a verified user, once per launch.
    Hello(String),
    /// [`VERIFY_EMAIL`] for an unverified user, every time Home opens.
    VerifyEmail,
}

impl Greeting {
    /// The toast to raise: an info for [`Greeting::Hello`] and a warning for [`Greeting::VerifyEmail`], both up for [`QUICK`](crate::toast::QUICK).
    #[must_use]
    pub fn toast(&self) -> Toast {
        match self {
            Self::Hello(username) => Toast::quick(ToastKind::Info, format!("Hello, {username}!")),
            Self::VerifyEmail => Toast::quick(ToastKind::Warning, VERIFY_EMAIL),
        }
    }

    /// Whether raising this uses up the launch's greeting, so the client sets its greeted flag.
    #[must_use]
    pub fn uses_greeting(&self) -> bool {
        matches!(self, Self::Hello(_))
    }
}

/// What Home says to `user`, or `None`. `greeted` is the client's flag for this launch, set once a [`Greeting::Hello`] was raised; Home opens again on every return to it, and the hello is not repeated.
///
/// A verified user is greeted when not yet greeted. An unverified user is nudged to verify each time, since the nudge is a prompt to act on, not a welcome, and stops once the email is verified. When the refresh failed and only the session's copy is at hand ([`UserSource::Cached`]), a verified user is still greeted but an unverified one is not nudged, as the app does.
#[must_use]
pub fn greeting(user: &User, source: UserSource, greeted: bool) -> Option<Greeting> {
    let verified = user.email_verified_at.is_some();
    match (verified, source) {
        (true, _) => (!greeted).then(|| Greeting::Hello(user.username.to_string())),
        (false, UserSource::Fetched) => Some(Greeting::VerifyEmail),
        (false, UserSource::Cached) => None,
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use uuid::Uuid;

    use super::*;

    #[test]
    fn a_copy_reads_live_for_five_minutes_then_as_of_its_time() {
        let made = Utc.with_ymd_and_hms(2026, 11, 7, 8, 0, 0).unwrap();
        let after = |secs: i64| made + TimeDelta::seconds(secs);
        assert_eq!(CountsTag::at(Some(made), after(0)), CountsTag::Live);
        assert_eq!(CountsTag::at(Some(made), after(299)), CountsTag::Live);
        assert_eq!(CountsTag::at(Some(made), after(300)), CountsTag::AsOf(made));
        assert_eq!(
            CountsTag::at(Some(made), after(7200)),
            CountsTag::AsOf(made)
        );
        assert_eq!(CountsTag::at(None, after(7200)), CountsTag::Live);
    }

    #[test]
    fn the_tag_reads_live_or_as_of_the_local_time() {
        assert_eq!(CountsTag::Live.label(), "Live");
        let made = Utc.with_ymd_and_hms(2026, 11, 7, 8, 5, 0).unwrap();
        let local = made.with_timezone(&Local).format("%H:%M").to_string();
        assert_eq!(CountsTag::AsOf(made).label(), format!("as of {local}"));
    }

    fn mine(swipes_right: i64) -> HttpLifetimeCounters {
        HttpLifetimeCounters {
            user_id: Uuid::nil(),
            swipes_right,
            swipes_left: 1,
            swipes_up: 0,
            swipes_down: 0,
            searches: -9,
            decks_created: 6,
            decks_completed: 0,
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn my_figures_count_every_swipe_direction_and_clamp_at_zero() {
        let mut counters = mine(5);
        counters.swipes_left = 7;
        counters.swipes_up = 1;
        counters.swipes_down = 2;
        assert_eq!(
            Figures::from(&counters).values(),
            [Some(6), Some(0), Some(15)]
        );
    }

    #[test]
    fn my_cards_swiped_past_i64_waits_instead_of_saturating() {
        assert_eq!(
            Figures::from(&mine(i64::MAX)).values(),
            [Some(6), Some(0), None]
        );
        assert_eq!(Figures::default().values(), [None; 3], "loading");
    }

    #[test]
    fn scope_toggles_names_itself_and_round_trips_the_app_bytes() {
        assert_eq!(Scope::Everybody.toggled(), Scope::OnlyMe);
        assert_eq!(Scope::OnlyMe.toggled().label(), "Everybody");
        assert_eq!(Scope::from_bytes(Scope::OnlyMe.as_bytes()), Scope::OnlyMe);
        assert_eq!(Scope::from_bytes(b"all"), Scope::Everybody);
        assert_eq!(Scope::from_bytes(b"junk"), Scope::Everybody);
    }

    #[test]
    fn a_flavor_card_expires_at_the_top_of_the_next_hour() {
        let at = |h, m, s| {
            Utc.with_ymd_and_hms(2026, 10, 30, h, m, s)
                .unwrap()
                .with_nanosecond(789_000_000)
                .unwrap()
        };
        assert_eq!(next_hour(at(12, 34, 56)), at(13, 0, 0));
        assert_eq!(next_hour(at(12, 0, 0)), at(13, 0, 0));
        assert_eq!(next_hour(at(12, 59, 59)), at(13, 0, 0));
    }

    #[test]
    fn a_flavor_card_fetched_at_the_end_of_time_never_expires() {
        assert_eq!(
            next_hour(DateTime::<Utc>::MAX_UTC),
            DateTime::<Utc>::MAX_UTC
        );
    }

    fn user(verified: bool) -> User {
        let mut user = User::new(
            Uuid::nil(),
            zwipe_core::domain::user::Username::new("scotty").expect("a valid username"),
            zwipe_core::domain::user::Email::new("me@zwipe.net").expect("a valid email"),
        );
        user.email_verified_at = verified.then(Utc::now);
        user
    }

    #[test]
    fn a_verified_user_is_greeted_once_per_launch() {
        for source in [UserSource::Fetched, UserSource::Cached] {
            let hello = greeting(&user(true), source, false);
            assert_eq!(
                hello,
                Some(Greeting::Hello("scotty".to_owned())),
                "{source:?}"
            );
            assert_eq!(
                greeting(&user(true), source, true),
                None,
                "{source:?}, already greeted"
            );
        }
    }

    #[test]
    fn an_unverified_user_is_nudged_on_every_fresh_refresh() {
        assert_eq!(
            greeting(&user(false), UserSource::Fetched, false),
            Some(Greeting::VerifyEmail)
        );
        assert_eq!(
            greeting(&user(false), UserSource::Fetched, true),
            Some(Greeting::VerifyEmail),
            "the nudge ignores the greeted flag"
        );
    }

    #[test]
    fn a_failed_refresh_never_nudges_from_the_cached_user() {
        assert_eq!(greeting(&user(false), UserSource::Cached, false), None);
        assert_eq!(greeting(&user(false), UserSource::Cached, true), None);
    }

    #[test]
    fn only_the_hello_uses_up_the_greeting() {
        assert!(Greeting::Hello("scotty".to_owned()).uses_greeting());
        assert!(!Greeting::VerifyEmail.uses_greeting());
    }

    #[test]
    fn the_greeting_toasts_are_the_apps_kind_band_and_words() {
        assert_eq!(
            Greeting::Hello("scotty".to_owned()).toast(),
            Toast::new(ToastKind::Info, "Hello, scotty!", crate::toast::QUICK)
        );
        assert_eq!(
            Greeting::VerifyEmail.toast(),
            Toast::new(
                ToastKind::Warning,
                "Verify your email!",
                crate::toast::QUICK
            )
        );
    }
}
