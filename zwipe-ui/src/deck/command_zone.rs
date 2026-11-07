//! The command zone's four slots and what each client shows and sends for them: the role labels, the deck cards list's group headers, the order of a deck row's art crops and the update that sets or clears one slot. The app's words and order, which web and the TUI share.

use uuid::Uuid;
use zwipe_core::domain::deck::{DeckProfile, Format};
use zwipe_core::http::contracts::deck::HttpUpdateDeckProfile;
use zwipe_core::http::helpers::Patch;

use crate::hint::HintTopic;

/// One of the four command-zone slots, on the deck form and in the deck cards view's command zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    /// The commander, or the oathbreaker in Oathbreaker.
    Commander,
    /// The commander's partner.
    Partner,
    /// The commander's background.
    Background,
    /// The oathbreaker's signature spell.
    SignatureSpell,
}

impl Slot {
    /// Every slot, top to bottom on the form.
    pub const ALL: [Self; 4] = [
        Self::Commander,
        Self::Partner,
        Self::Background,
        Self::SignatureSpell,
    ];

    /// Characters typed before a search starts.
    #[must_use]
    pub fn min_chars(self) -> usize {
        match self {
            Self::Commander | Self::SignatureSpell => 3,
            Self::Partner | Self::Background => 1,
        }
    }

    /// Cards a search returns at most.
    #[must_use]
    pub fn limit(self) -> u32 {
        match self {
            Self::Commander | Self::SignatureSpell => 5,
            Self::Partner | Self::Background => 10,
        }
    }

    /// What the swipe picker calls the card it picks for this slot.
    #[must_use]
    pub fn noun(self, format: Option<Format>) -> &'static str {
        match self {
            Self::Commander if format.is_some_and(|f| f.has_signature_spell()) => "oathbreaker",
            Self::Commander => "commander",
            Self::Partner => "partner",
            Self::Background => "background",
            Self::SignatureSpell => "signature spell",
        }
    }

    /// The `?` topic beside the slot's field: the commander's is
    /// [`HintTopic::Oathbreaker`] in a format with a signature spell.
    #[must_use]
    pub fn hint(self, format: Option<Format>) -> HintTopic {
        match self {
            Self::Commander if is_oathbreaker(format) => HintTopic::Oathbreaker,
            Self::Commander => HintTopic::Commander,
            Self::Partner => HintTopic::Partner,
            Self::Background => HintTopic::Background,
            Self::SignatureSpell => HintTopic::SignatureSpell,
        }
    }

    /// The slot's role label, as the featured strip, the profile rows and the form fields show it. The commander is "Oathbreaker" in a format with a signature spell.
    #[must_use]
    pub fn label(self, format: Option<Format>) -> &'static str {
        match self {
            Self::Commander if is_oathbreaker(format) => "Oathbreaker",
            Self::Commander => "Commander",
            Self::Partner => "Partner",
            Self::Background => "Background",
            Self::SignatureSpell => "Signature spell",
        }
    }

    /// The header of the group the slot sits in on the deck cards list. The commander and the partner share one group: "Oathbreaker" in a format with a signature spell, else "Commanders" when the deck has a partner and "Commander" when it has none. The background and the signature spell head their own groups with their role labels.
    #[must_use]
    pub fn group_header(self, format: Option<Format>, has_partner: bool) -> &'static str {
        match self {
            Self::Commander | Self::Partner if is_oathbreaker(format) => "Oathbreaker",
            Self::Commander | Self::Partner if has_partner => "Commanders",
            Self::Commander | Self::Partner => "Commander",
            Self::Background => "Background",
            Self::SignatureSpell => "Signature spell",
        }
    }

    /// The collapse key of the group the slot sits in on the deck cards list:
    /// one key for the commander and the partner, as they share a group, and
    /// apart from every grouping's keys.
    #[must_use]
    pub fn group_key(self) -> &'static str {
        match self {
            Self::Commander | Self::Partner => "zone:commander",
            Self::Background => "zone:background",
            Self::SignatureSpell => "zone:signature-spell",
        }
    }

    /// The sparse deck profile update that puts the printing `id` in this slot, or clears the slot when `id` is `None`. Every other field stays unchanged.
    #[must_use]
    pub fn update_request(self, id: Option<Uuid>) -> HttpUpdateDeckProfile {
        let id = Patch::Set(id);
        let builder = HttpUpdateDeckProfile::builder();
        match self {
            Self::Commander => builder.commander_id(id),
            Self::Partner => builder.partner_commander_id(id),
            Self::Background => builder.background_id(id),
            Self::SignatureSpell => builder.signature_spell_id(id),
        }
        .build()
    }
}

/// Whether the format names its commander the oathbreaker: one with a signature spell.
fn is_oathbreaker(format: Option<Format>) -> bool {
    format.is_some_and(|f| f.has_signature_spell())
}

/// The deck's command-zone art crops for its list row, in zone order: commander (or oathbreaker), partner, background, signature spell, keeping only those the deck has. The server resolves them on the profile, so a row draws its command zone without fetching cards.
#[must_use]
pub fn art(profile: &DeckProfile) -> Vec<&str> {
    [
        profile.commander_art_url.as_deref(),
        profile.partner_commander_art_url.as_deref(),
        profile.background_art_url.as_deref(),
        profile.signature_spell_art_url.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::test_support::deck_profile;

    #[test]
    fn labels_follow_the_app_in_every_format() {
        for &format in Format::all() {
            let commander = if format == Format::Oathbreaker {
                "Oathbreaker"
            } else {
                "Commander"
            };
            let got = Slot::ALL.map(|slot| slot.label(Some(format)));
            assert_eq!(
                got,
                [commander, "Partner", "Background", "Signature spell"],
                "format {format:?}"
            );
        }
        assert_eq!(
            Slot::ALL.map(|slot| slot.label(None)),
            ["Commander", "Partner", "Background", "Signature spell"],
            "no format"
        );
    }

    #[test]
    fn the_leaders_share_one_group_header() {
        let cases = [
            (Some(Format::Commander), false, "Commander"),
            (Some(Format::Commander), true, "Commanders"),
            (None, false, "Commander"),
            (None, true, "Commanders"),
            (Some(Format::Oathbreaker), false, "Oathbreaker"),
            (Some(Format::Oathbreaker), true, "Oathbreaker"),
        ];
        for (format, has_partner, want) in cases {
            for slot in [Slot::Commander, Slot::Partner] {
                assert_eq!(
                    slot.group_header(format, has_partner),
                    want,
                    "{slot:?} in {format:?}, partner {has_partner}"
                );
            }
        }
    }

    #[test]
    fn each_zone_group_folds_by_its_own_key() {
        assert_eq!(Slot::Commander.group_key(), Slot::Partner.group_key());
        let keys = [Slot::Commander, Slot::Background, Slot::SignatureSpell].map(Slot::group_key);
        assert!(keys.iter().all(|k| k.starts_with("zone:")));
        assert_ne!(keys[0], keys[1]);
        assert_ne!(keys[1], keys[2]);
        assert_ne!(keys[0], keys[2]);
    }

    #[test]
    fn the_background_and_spell_head_their_own_groups() {
        for format in [None, Some(Format::Commander), Some(Format::Oathbreaker)] {
            for has_partner in [false, true] {
                assert_eq!(
                    Slot::Background.group_header(format, has_partner),
                    "Background"
                );
                assert_eq!(
                    Slot::SignatureSpell.group_header(format, has_partner),
                    "Signature spell"
                );
            }
        }
    }

    #[test]
    fn art_runs_in_zone_order_and_skips_empty_slots() {
        let mut profile = deck_profile("Art");
        assert_eq!(art(&profile), Vec::<&str>::new());

        profile.commander_art_url = Some("commander".to_owned());
        assert_eq!(art(&profile), ["commander"]);

        profile.background_art_url = Some("background".to_owned());
        assert_eq!(art(&profile), ["commander", "background"]);

        profile.partner_commander_art_url = Some("partner".to_owned());
        assert_eq!(art(&profile), ["commander", "partner", "background"]);

        profile.background_art_url = None;
        profile.signature_spell_art_url = Some("spell".to_owned());
        assert_eq!(art(&profile), ["commander", "partner", "spell"]);

        profile.commander_art_url = None;
        profile.partner_commander_art_url = None;
        assert_eq!(art(&profile), ["spell"]);
    }

    /// The request as the app built it before the move, field by field.
    fn app_request(slot: Slot, id: Option<Uuid>) -> HttpUpdateDeckProfile {
        let id = Patch::Set(id);
        match slot {
            Slot::Commander => HttpUpdateDeckProfile::builder().commander_id(id).build(),
            Slot::Partner => HttpUpdateDeckProfile::builder()
                .partner_commander_id(id)
                .build(),
            Slot::Background => HttpUpdateDeckProfile::builder().background_id(id).build(),
            Slot::SignatureSpell => HttpUpdateDeckProfile::builder()
                .signature_spell_id(id)
                .build(),
        }
    }

    #[test]
    fn update_requests_set_or_clear_one_slot() {
        let id = Uuid::new_v4();
        let fields = [
            (Slot::Commander, "commander_id"),
            (Slot::Partner, "partner_commander_id"),
            (Slot::Background, "background_id"),
            (Slot::SignatureSpell, "signature_spell_id"),
        ];
        for (slot, field) in fields {
            let set = serde_json::to_value(slot.update_request(Some(id))).unwrap();
            assert_eq!(set, json!({ field: id }), "set {slot:?}");
            assert_eq!(
                set,
                serde_json::to_value(app_request(slot, Some(id))).unwrap(),
                "set {slot:?} as the app sends it"
            );

            let clear = serde_json::to_value(slot.update_request(None)).unwrap();
            assert_eq!(clear, json!({ field: Value::Null }), "clear {slot:?}");
            assert_eq!(
                clear,
                serde_json::to_value(app_request(slot, None)).unwrap(),
                "clear {slot:?} as the app sends it"
            );
        }
    }
}
