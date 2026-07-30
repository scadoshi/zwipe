//! User metrics HTTP request/response contracts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Pre-auth funnel event kinds accepted by the anonymous ingest endpoint.
///
/// A closed set shared by client and server: an unknown kind fails
/// deserialization instead of landing as a stray string. The client only
/// fires these while unauthenticated — once a user exists, the sparse
/// `user_events` log takes over (registration success is its `register` row).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AnonymousEventKind {
    /// App launched with no authenticated session.
    AppOpened,
    /// Register screen viewed.
    RegisterViewed,
    /// Register form submitted (success or not).
    RegisterSubmitted,
}

impl AnonymousEventKind {
    /// String form stored in the `anonymous_events.kind` column.
    pub fn as_str(self) -> &'static str {
        match self {
            AnonymousEventKind::AppOpened => "app_opened",
            AnonymousEventKind::RegisterViewed => "register_viewed",
            AnonymousEventKind::RegisterSubmitted => "register_submitted",
        }
    }
}

/// One pre-auth funnel event posted by an unauthenticated client.
///
/// `session_id` is a random UUID the client generates per install/launch —
/// it carries no identity and exists only so funnel steps from the same
/// session can be counted once (distinct sessions per kind).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpAnonymousEvent {
    /// Random client-generated session id (no PII, not an account id).
    pub session_id: Uuid,
    /// Which funnel step occurred.
    pub kind: AnonymousEventKind,
}

/// Batched usage counters posted by the client.
///
/// The client buffers counts in memory and flushes periodically (every ~30s
/// and on screen-exit / app backgrounding). All fields are additive — the
/// server increments the corresponding lifetime and daily counters.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct HttpUsageBatch {
    /// Right swipes (add card to deck).
    pub swipes_right: u32,
    /// Left swipes (skip card).
    pub swipes_left: u32,
    /// Up swipes.
    pub swipes_up: u32,
    /// Down swipes.
    pub swipes_down: u32,
    /// Card searches issued.
    pub searches: u32,
    /// Per-`(commander, card)` add/skip/maybe/remove tallies from deck building.
    /// Aggregate-only, no user identity. `#[serde(default)]` so older clients
    /// that omit the field still deserialize (backward compatible).
    #[serde(default)]
    pub signals: Vec<CardSignalDelta>,
    /// Per-deck skip/unskip oracle-id deltas (Add-screen left swipes).
    /// `#[serde(default)]` keeps older clients compatible.
    #[serde(default)]
    pub deck_skips: Vec<DeckSkipDelta>,
    /// Per-commander shown/selected/skipped tallies from the Zwipe-select
    /// screen. Aggregate-only, no user identity. `#[serde(default)]` keeps
    /// older clients compatible.
    #[serde(default)]
    pub select_signals: Vec<CommanderSelectDelta>,
    /// Handled-error reports (an error was shown to the user) accumulated over
    /// the flush window. Aggregate-only, no user identity. `#[serde(default)]`
    /// keeps older clients compatible.
    #[serde(default)]
    pub client_errors: Vec<ClientErrorReport>,
}

/// One handled-error report: a `ClientError` was surfaced to the user
/// (toast/dialog) on the client. Deduped client-side within a flush window
/// (`count`), aggregate-only — no user identity beyond the authed usage batch
/// it rides in.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ClientErrorReport {
    /// Host screen, flattened module path (`"deck_edit"`, `"auth_login"`) —
    /// closed vocabulary from client-side consts. The aggregation axis;
    /// component names never leak into it.
    pub screen: String,
    /// `""` for the screen itself; else the component module name
    /// (`"card_filter_sheet"`) — the drill-down breadcrumb. Dialogs/sheets
    /// report their host screen plus their own component name.
    #[serde(default)]
    pub component: String,
    /// What the user was doing (`"save_profile"`, `"filter_apply"`).
    pub action: String,
    /// Error kind: `ClientError` variant plus `ApiError` variant where
    /// applicable (`"api_unauthorized"`, `"decode"`).
    pub kind: String,
    /// Human-readable detail. 4xx messages are user-safe by contract; decode
    /// messages must be reduced to the error's shape client-side (serde
    /// errors quote input fragments — response bodies must never ride a
    /// report). Truncated on both sides.
    pub message: String,
    /// How many times this exact error fired within the flush window.
    pub count: u32,
    /// `CARGO_PKG_VERSION` — self-reported, untrusted debugging metadata.
    pub client_version: String,
    /// `"ios"` / `"android"` / `"web"` — self-reported.
    pub platform: String,
}

impl ClientErrorReport {
    /// Maximum characters kept in `message`.
    pub const MAX_MESSAGE_CHARS: usize = 300;

    /// Maximum characters kept in `screen`, `component`, `action`, and `kind`.
    pub const MAX_LABEL_CHARS: usize = 64;

    /// Maximum characters kept in `client_version` and `platform`.
    pub const MAX_META_CHARS: usize = 32;

    /// Returns a copy with every string truncated to its cap and `count`
    /// clamped to [`HttpUsageBatch::MAX_PER_FLUSH`]. Client-side truncation is
    /// untrusted — the server clamps regardless.
    #[must_use]
    pub fn clamped(&self) -> Self {
        Self {
            screen: truncated_chars(&self.screen, Self::MAX_LABEL_CHARS),
            component: truncated_chars(&self.component, Self::MAX_LABEL_CHARS),
            action: truncated_chars(&self.action, Self::MAX_LABEL_CHARS),
            kind: truncated_chars(&self.kind, Self::MAX_LABEL_CHARS),
            message: truncated_chars(&self.message, Self::MAX_MESSAGE_CHARS),
            count: self.count.min(HttpUsageBatch::MAX_PER_FLUSH),
            client_version: truncated_chars(&self.client_version, Self::MAX_META_CHARS),
            platform: truncated_chars(&self.platform, Self::MAX_META_CHARS),
        }
    }
}

/// One crash report, posted unauthenticated on the launch after the crash.
///
/// `crash_id` is stamped at panic time by the client's panic hook; the server
/// upserts on it (`ON CONFLICT DO NOTHING`), so retries across launches store
/// exactly one row per crash.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpCrashReport {
    /// Stamped at panic time — the server's dedupe key.
    pub crash_id: Uuid,
    /// `CARGO_PKG_VERSION` — self-reported, untrusted debugging metadata.
    pub client_version: String,
    /// Platform of the crashed build.
    pub platform: crate::domain::auth::models::platform::ClientPlatform,
    /// Panic payload + location. Truncated on both sides.
    pub message: String,
    /// When the panic hook ran (client clock — untrusted, indicative only).
    pub occurred_at: DateTime<Utc>,
}

impl HttpCrashReport {
    /// Maximum characters kept in `message`.
    pub const MAX_MESSAGE_CHARS: usize = 2_000;

    /// Returns a copy with `message` and `client_version` truncated to their
    /// caps. Client-side truncation is untrusted — the server clamps
    /// regardless.
    #[must_use]
    pub fn clamped(&self) -> Self {
        Self {
            crash_id: self.crash_id,
            client_version: truncated_chars(
                &self.client_version,
                ClientErrorReport::MAX_META_CHARS,
            ),
            platform: self.platform,
            message: truncated_chars(&self.message, Self::MAX_MESSAGE_CHARS),
            occurred_at: self.occurred_at,
        }
    }
}

/// Returns at most the first `max` characters of `s` (char-boundary safe).
fn truncated_chars(s: &str, max: usize) -> String {
    s.chars().take(max).collect()
}

/// One card signal delta accumulated over a flush window.
///
/// Keyed by the swiped card plus its deck (`deck_id`); the server derives the
/// deck's commander (EDH) or generalized `(format, color-identity)` context
/// from it. `shown` is the impression denominator — currently the client sends
/// `added + skipped + maybed`, leaving room for true impressions later.
///
/// The legacy `commander_oracle_id` wire field was dropped 2026-07-24 (Phase 5S
/// step 3), after `MIN_CLIENT_VERSION` was floored to 1.7.0 — every serving
/// client sends `deck_id`. A straggler payload still carrying the old field
/// deserializes fine (serde ignores unknown fields).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CardSignalDelta {
    /// Card oracle id.
    pub card_oracle_id: Uuid,
    /// Deck the swipe belongs to — the sole context key: the server derives the
    /// deck's commander, format, color identity, and tags from it
    /// (context/plans/otags/ Phase 5). Kept `Option` for wire leniency; the
    /// server drops signals it can't resolve to an owned deck.
    #[serde(default)]
    pub deck_id: Option<Uuid>,
    /// Times the card was shown for a decision (add-stack impressions).
    pub shown: u32,
    /// Right swipes on the add stack (added to deck).
    pub added: u32,
    /// Left swipes (skipped).
    pub skipped: u32,
    /// Up swipes (maybeboarded).
    pub maybed: u32,
    /// Deliberate removals from a deck (a delayed negative).
    pub removed: u32,
}

/// One commander-select signal delta accumulated over a flush window.
///
/// Keyed by the **shown candidate's** oracle id (the card swiped on the
/// Zwipe-select screen), unlike [`CardSignalDelta`] whose lead key is the
/// deck's commander. `shown` is the impression denominator — the client
/// sends `selected + skipped` (down-swipe undo is not a decision and is
/// excluded).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CommanderSelectDelta {
    /// Oracle id of the command-zone candidate that was swiped.
    pub commander_oracle_id: Uuid,
    /// Times the candidate was shown for a decision (select-stack impressions).
    pub shown: u32,
    /// Right swipes (picked into the command zone).
    pub selected: u32,
    /// Left swipes (passed).
    pub skipped: u32,
}

/// Per-deck skip deltas accumulated over a flush window.
///
/// Skips are keyed by **oracle id** so a skip covers every printing. `skipped`
/// carries new left-swipes; `unskipped` carries undos of skips that had
/// already flushed (a pre-flush undo simply drops the pending entry
/// client-side and never reaches the wire). Ingest writes these into the
/// deck's suppression set (`source = 'skip'`) after verifying ownership;
/// removal suppressions never ride this contract — the server records them
/// directly on the delete-card endpoint.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DeckSkipDelta {
    /// Deck the skips belong to (ownership is verified at ingest).
    pub deck_id: Uuid,
    /// Oracle ids left-swiped since the last flush.
    pub skipped: Vec<Uuid>,
    /// Oracle ids whose skip was undone after it had already flushed.
    pub unskipped: Vec<Uuid>,
}

impl HttpUsageBatch {
    /// Maximum accepted value per counter per flush.
    ///
    /// The client buffers only ~30s of activity before flushing, so legitimate
    /// values are tiny (tens). This caps untrusted input so a client can't
    /// inflate the lifetime / public-marketing totals, and keeps each week's
    /// accumulation comfortably within the weekly-signal `INTEGER` columns
    /// even at the endpoint's request rate limit.
    pub const MAX_PER_FLUSH: u32 = 10_000;

    /// Maximum accepted number of distinct `(commander, card)` signal deltas per
    /// flush. A legitimate ~30s window touches a few dozen cards at most; this
    /// caps an untrusted client from sending a runaway upsert set.
    pub const MAX_SIGNALS_PER_FLUSH: usize = 1_000;

    /// Maximum accepted number of per-deck skip deltas per flush. A flush
    /// window realistically touches one deck; this caps an untrusted client
    /// from fanning writes across arbitrary decks.
    pub const MAX_SKIP_DECKS_PER_FLUSH: usize = 50;

    /// Maximum accepted number of distinct handled-error reports per flush.
    /// Matches the client-side buffer cap — beyond it, an error loop stops
    /// adding entries (its repeats still tally into existing `count`s).
    pub const MAX_CLIENT_ERRORS_PER_FLUSH: usize = 20;

    /// Returns a copy with every counter clamped to [`Self::MAX_PER_FLUSH`], the
    /// signal list truncated to [`Self::MAX_SIGNALS_PER_FLUSH`], each signal's
    /// tallies clamped per field, and the deck-skip deltas truncated to
    /// [`Self::MAX_SKIP_DECKS_PER_FLUSH`] decks of at most
    /// [`Self::MAX_SIGNALS_PER_FLUSH`] ids per list.
    #[must_use]
    pub fn clamped(&self) -> Self {
        Self {
            swipes_right: self.swipes_right.min(Self::MAX_PER_FLUSH),
            swipes_left: self.swipes_left.min(Self::MAX_PER_FLUSH),
            swipes_up: self.swipes_up.min(Self::MAX_PER_FLUSH),
            swipes_down: self.swipes_down.min(Self::MAX_PER_FLUSH),
            searches: self.searches.min(Self::MAX_PER_FLUSH),
            signals: self
                .signals
                .iter()
                .take(Self::MAX_SIGNALS_PER_FLUSH)
                .map(CardSignalDelta::clamped)
                .collect(),
            deck_skips: self
                .deck_skips
                .iter()
                .take(Self::MAX_SKIP_DECKS_PER_FLUSH)
                .map(DeckSkipDelta::clamped)
                .collect(),
            select_signals: self
                .select_signals
                .iter()
                .take(Self::MAX_SIGNALS_PER_FLUSH)
                .map(CommanderSelectDelta::clamped)
                .collect(),
            client_errors: self
                .client_errors
                .iter()
                .take(Self::MAX_CLIENT_ERRORS_PER_FLUSH)
                .map(ClientErrorReport::clamped)
                .collect(),
        }
    }
}

impl CommanderSelectDelta {
    /// Returns a copy with each tally clamped to [`HttpUsageBatch::MAX_PER_FLUSH`].
    #[must_use]
    pub fn clamped(&self) -> Self {
        Self {
            commander_oracle_id: self.commander_oracle_id,
            shown: self.shown.min(HttpUsageBatch::MAX_PER_FLUSH),
            selected: self.selected.min(HttpUsageBatch::MAX_PER_FLUSH),
            skipped: self.skipped.min(HttpUsageBatch::MAX_PER_FLUSH),
        }
    }
}

impl CardSignalDelta {
    /// Returns a copy with each tally clamped to [`HttpUsageBatch::MAX_PER_FLUSH`].
    #[must_use]
    pub fn clamped(&self) -> Self {
        Self {
            card_oracle_id: self.card_oracle_id,
            deck_id: self.deck_id,
            shown: self.shown.min(HttpUsageBatch::MAX_PER_FLUSH),
            added: self.added.min(HttpUsageBatch::MAX_PER_FLUSH),
            skipped: self.skipped.min(HttpUsageBatch::MAX_PER_FLUSH),
            maybed: self.maybed.min(HttpUsageBatch::MAX_PER_FLUSH),
            removed: self.removed.min(HttpUsageBatch::MAX_PER_FLUSH),
        }
    }
}

impl DeckSkipDelta {
    /// Returns a copy with each id list truncated to
    /// [`HttpUsageBatch::MAX_SIGNALS_PER_FLUSH`].
    #[must_use]
    pub fn clamped(&self) -> Self {
        Self {
            deck_id: self.deck_id,
            skipped: self
                .skipped
                .iter()
                .copied()
                .take(HttpUsageBatch::MAX_SIGNALS_PER_FLUSH)
                .collect(),
            unskipped: self
                .unskipped
                .iter()
                .copied()
                .take(HttpUsageBatch::MAX_SIGNALS_PER_FLUSH)
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AnonymousEventKind, CardSignalDelta, ClientErrorReport, CommanderSelectDelta,
        DeckSkipDelta, HttpAnonymousEvent, HttpCrashReport, HttpUsageBatch,
    };
    use uuid::Uuid;

    #[test]
    fn anonymous_kind_wire_form_matches_db_string() {
        // The serde wire form and as_str (the anonymous_events.kind column
        // value) must never diverge, or funnel GROUP BYs would split a kind
        // into two spellings.
        for kind in [
            AnonymousEventKind::AppOpened,
            AnonymousEventKind::RegisterViewed,
            AnonymousEventKind::RegisterSubmitted,
        ] {
            let wire = serde_json::to_string(&kind).unwrap();
            assert_eq!(wire, format!("\"{}\"", kind.as_str()));
            let back: AnonymousEventKind = serde_json::from_str(&wire).unwrap();
            assert_eq!(back, kind);
        }
    }

    #[test]
    fn anonymous_event_parses_and_rejects_unknown_kind() {
        let json =
            r#"{"session_id":"00000000-0000-0000-0000-000000000000","kind":"register_viewed"}"#;
        let event: HttpAnonymousEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.kind, AnonymousEventKind::RegisterViewed);
        assert_eq!(event.session_id, Uuid::nil());

        let bad = r#"{"session_id":"00000000-0000-0000-0000-000000000000","kind":"totally_fake"}"#;
        assert!(serde_json::from_str::<HttpAnonymousEvent>(bad).is_err());
    }

    #[test]
    fn clamped_caps_each_field_and_leaves_small_ones() {
        let clamped = HttpUsageBatch {
            swipes_right: u32::MAX,
            swipes_left: 5,
            swipes_up: HttpUsageBatch::MAX_PER_FLUSH + 1,
            swipes_down: 0,
            searches: u32::MAX,
            signals: Vec::new(),
            deck_skips: Vec::new(),
            select_signals: Vec::new(),
            client_errors: Vec::new(),
        }
        .clamped();
        assert_eq!(clamped.swipes_right, HttpUsageBatch::MAX_PER_FLUSH);
        assert_eq!(clamped.swipes_left, 5);
        assert_eq!(clamped.swipes_up, HttpUsageBatch::MAX_PER_FLUSH);
        assert_eq!(clamped.swipes_down, 0);
        assert_eq!(clamped.searches, HttpUsageBatch::MAX_PER_FLUSH);
    }

    #[test]
    fn clamped_truncates_signal_list_and_caps_tallies() {
        let delta = CardSignalDelta {
            card_oracle_id: Uuid::nil(),
            deck_id: None,
            shown: u32::MAX,
            added: 3,
            skipped: HttpUsageBatch::MAX_PER_FLUSH + 1,
            maybed: 0,
            removed: 0,
        };
        let batch = HttpUsageBatch {
            signals: vec![delta; HttpUsageBatch::MAX_SIGNALS_PER_FLUSH + 5],
            ..Default::default()
        }
        .clamped();
        assert_eq!(batch.signals.len(), HttpUsageBatch::MAX_SIGNALS_PER_FLUSH);
        let first = batch.signals.first().unwrap();
        assert_eq!(first.shown, HttpUsageBatch::MAX_PER_FLUSH);
        assert_eq!(first.added, 3);
        assert_eq!(first.skipped, HttpUsageBatch::MAX_PER_FLUSH);
    }

    #[test]
    fn batch_deserializes_without_signals_field() {
        // An older client omits `signals`, `deck_skips`, `select_signals`,
        // and `client_errors`; must still parse (→ empty).
        let json =
            r#"{"swipes_right":2,"swipes_left":1,"swipes_up":0,"swipes_down":0,"searches":3}"#;
        let batch: HttpUsageBatch = serde_json::from_str(json).unwrap();
        assert!(batch.signals.is_empty());
        assert!(batch.deck_skips.is_empty());
        assert!(batch.select_signals.is_empty());
        assert!(batch.client_errors.is_empty());
        assert_eq!(batch.swipes_right, 2);
    }

    #[test]
    fn clamped_truncates_client_errors_and_their_strings() {
        let report = ClientErrorReport {
            screen: "s".repeat(ClientErrorReport::MAX_LABEL_CHARS + 40),
            component: "card_filter_sheet".to_string(),
            action: "save".to_string(),
            kind: "api_unprocessable".to_string(),
            message: "m".repeat(ClientErrorReport::MAX_MESSAGE_CHARS + 500),
            count: u32::MAX,
            client_version: "v".repeat(ClientErrorReport::MAX_META_CHARS + 8),
            platform: "ios".to_string(),
        };
        let batch = HttpUsageBatch {
            client_errors: vec![report; HttpUsageBatch::MAX_CLIENT_ERRORS_PER_FLUSH + 5],
            ..Default::default()
        }
        .clamped();
        assert_eq!(
            batch.client_errors.len(),
            HttpUsageBatch::MAX_CLIENT_ERRORS_PER_FLUSH
        );
        let first = batch.client_errors.first().unwrap();
        assert_eq!(
            first.screen.chars().count(),
            ClientErrorReport::MAX_LABEL_CHARS
        );
        assert_eq!(
            first.message.chars().count(),
            ClientErrorReport::MAX_MESSAGE_CHARS
        );
        assert_eq!(
            first.client_version.chars().count(),
            ClientErrorReport::MAX_META_CHARS
        );
        assert_eq!(first.count, HttpUsageBatch::MAX_PER_FLUSH);
        assert_eq!(first.component, "card_filter_sheet");
        assert_eq!(first.action, "save");
    }

    #[test]
    fn client_error_report_component_defaults_to_empty() {
        // `component` is `#[serde(default)]` — a report from the screen
        // itself omits it entirely.
        let json = r#"{"screen":"deck_edit","action":"save","kind":"api_unprocessable",
            "message":"deck limit reached","count":2,"client_version":"1.8.0","platform":"ios"}"#;
        let report: ClientErrorReport = serde_json::from_str(json).unwrap();
        assert_eq!(report.component, "");
        assert_eq!(report.screen, "deck_edit");
    }

    #[test]
    fn crash_report_round_trips_and_clamps_message() {
        use crate::domain::auth::models::platform::ClientPlatform;

        let report = HttpCrashReport {
            crash_id: Uuid::nil(),
            client_version: "1.8.0".to_string(),
            platform: ClientPlatform::Android,
            message: "p".repeat(HttpCrashReport::MAX_MESSAGE_CHARS + 100),
            occurred_at: chrono::Utc::now(),
        };
        let wire = serde_json::to_string(&report).unwrap();
        let back: HttpCrashReport = serde_json::from_str(&wire).unwrap();
        assert_eq!(back.crash_id, report.crash_id);
        assert_eq!(back.platform, ClientPlatform::Android);

        let clamped = back.clamped();
        assert_eq!(
            clamped.message.chars().count(),
            HttpCrashReport::MAX_MESSAGE_CHARS
        );
        assert_eq!(clamped.client_version, "1.8.0");
    }

    #[test]
    fn truncation_is_char_boundary_safe() {
        // Multi-byte chars at the cut point must not panic or split bytes.
        let report = ClientErrorReport {
            message: "é".repeat(ClientErrorReport::MAX_MESSAGE_CHARS + 10),
            ..Default::default()
        };
        let clamped = report.clamped();
        assert_eq!(
            clamped.message.chars().count(),
            ClientErrorReport::MAX_MESSAGE_CHARS
        );
    }

    #[test]
    fn signal_deck_id_defaults_to_none_when_omitted() {
        // Wire leniency: a signal with no deck_id still parses (→ None); the
        // server drops it at ingest rather than failing the whole batch.
        let json = r#"{"swipes_right":0,"swipes_left":0,"swipes_up":0,"swipes_down":0,"searches":0,
            "signals":[{"card_oracle_id":"00000000-0000-0000-0000-000000000000",
                        "shown":1,"added":1,"skipped":0,"maybed":0,"removed":0}]}"#;
        let batch: HttpUsageBatch = serde_json::from_str(json).unwrap();
        assert_eq!(batch.signals.len(), 1);
        let signal = batch.signals.first().unwrap();
        assert_eq!(signal.deck_id, None);
        assert!(signal.clamped().deck_id.is_none());
    }

    #[test]
    fn signal_with_legacy_commander_field_still_parses() {
        // The legacy `commander_oracle_id` wire field was dropped (Phase 5S
        // step 3). A straggler payload still carrying it must deserialize —
        // serde ignores unknown fields — with deck_id carrying the context.
        let json = r#"{"swipes_right":0,"swipes_left":0,"swipes_up":0,"swipes_down":0,"searches":0,
            "signals":[{"commander_oracle_id":"11111111-1111-1111-1111-111111111111",
                        "card_oracle_id":"00000000-0000-0000-0000-000000000000",
                        "deck_id":"22222222-2222-2222-2222-222222222222",
                        "shown":1,"added":1,"skipped":0,"maybed":0,"removed":0}]}"#;
        let batch: HttpUsageBatch = serde_json::from_str(json).unwrap();
        let signal = batch.signals.first().unwrap();
        assert!(
            signal.deck_id.is_some(),
            "deck_id carries the context; the legacy field is ignored"
        );
    }

    #[test]
    fn clamped_truncates_select_signal_list_and_caps_tallies() {
        let delta = CommanderSelectDelta {
            commander_oracle_id: Uuid::nil(),
            shown: u32::MAX,
            selected: 2,
            skipped: HttpUsageBatch::MAX_PER_FLUSH + 1,
        };
        let batch = HttpUsageBatch {
            select_signals: vec![delta; HttpUsageBatch::MAX_SIGNALS_PER_FLUSH + 5],
            ..Default::default()
        }
        .clamped();
        assert_eq!(
            batch.select_signals.len(),
            HttpUsageBatch::MAX_SIGNALS_PER_FLUSH
        );
        let first = batch.select_signals.first().unwrap();
        assert_eq!(first.shown, HttpUsageBatch::MAX_PER_FLUSH);
        assert_eq!(first.selected, 2);
        assert_eq!(first.skipped, HttpUsageBatch::MAX_PER_FLUSH);
    }

    #[test]
    fn clamped_truncates_deck_skip_deltas_and_id_lists() {
        let delta = DeckSkipDelta {
            deck_id: Uuid::nil(),
            skipped: vec![Uuid::nil(); HttpUsageBatch::MAX_SIGNALS_PER_FLUSH + 7],
            unskipped: vec![Uuid::nil(); 3],
        };
        let batch = HttpUsageBatch {
            deck_skips: vec![delta; HttpUsageBatch::MAX_SKIP_DECKS_PER_FLUSH + 2],
            ..Default::default()
        }
        .clamped();
        assert_eq!(
            batch.deck_skips.len(),
            HttpUsageBatch::MAX_SKIP_DECKS_PER_FLUSH
        );
        let first = batch.deck_skips.first().unwrap();
        assert_eq!(first.skipped.len(), HttpUsageBatch::MAX_SIGNALS_PER_FLUSH);
        assert_eq!(first.unskipped.len(), 3);
    }
}

/// Public app-wide aggregates surfaced on zwipe.net.
///
/// Counts span all users. Numbers are sums over `user_lifetime_counters` at
/// query time; the endpoint is cached at the CF edge for ~1h to keep origin
/// load near zero.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpPublicMetrics {
    /// Sum of every swipe across every user (right + left + up + down).
    pub cards_swiped: i64,
    /// Sum of every card search across every user.
    pub searches: i64,
    /// Decks created across every user.
    pub decks_created: i64,
}

/// Per-user lifetime metric totals returned by `GET /api/user/metrics`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpLifetimeCounters {
    /// Owning user id.
    pub user_id: Uuid,
    /// Total right swipes.
    pub swipes_right: i64,
    /// Total left swipes.
    pub swipes_left: i64,
    /// Total up swipes.
    pub swipes_up: i64,
    /// Total down swipes.
    pub swipes_down: i64,
    /// Total searches.
    pub searches: i64,
    /// Decks created.
    pub decks_created: i32,
    /// Decks that have reached a valid state at least once.
    pub decks_completed: i32,
    /// Last write to this counter row. Not a last-active signal —
    /// `users.last_active_at` is the canonical one.
    pub updated_at: DateTime<Utc>,
}
