//! Removing the printings Scryfall deleted:
//! which stored printings the bulk file no longer lists, the printing each one's deck references
//! move to, and which stay because a deck holds a card with no other printing. The plan is pure;
//! zervice gathers its inputs from the database and the bulk file it just synced, and applies it.

use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;

use uuid::Uuid;

/// One stored printing, as the plan and its log need it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Printing {
    /// Its Scryfall id.
    pub id: Uuid,
    /// The card it prints; `None` for a printing Scryfall gives no oracle id, which has no
    /// sibling to move to.
    pub oracle_id: Option<Uuid>,
    /// The card's name.
    pub name: String,
    /// Its set code.
    pub set: String,
    /// Its collector number in that set.
    pub collector_number: String,
}

/// How many deck references point at a printing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct References {
    /// Deck card rows that hold it.
    pub deck_cards: u32,
    /// Command zone slots (commander, partner, background, signature spell) that name it.
    pub slots: u32,
}

impl References {
    /// Whether anything points at the printing.
    #[must_use]
    pub fn any(self) -> bool {
        self.deck_cards > 0 || self.slots > 0
    }
}

/// The surviving printings a removed one can move to, by oracle id, none of them removed.
#[derive(Debug, Clone, Default)]
pub struct Survivors {
    /// The cheapest regular printing of each card, as import and Quick add pick it.
    pub cheapest: HashMap<Uuid, Printing>,
    /// The printing `latest_cards` would show for each card, for a card with no regular priced
    /// survivor.
    pub preferred: HashMap<Uuid, Printing>,
}

/// A printing the plan removes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Removal {
    /// The removed printing.
    pub printing: Printing,
    /// Where its references move; `None` only when nothing references it.
    pub replacement: Option<Printing>,
    /// What points at it now.
    pub references: References,
}

/// A removed printing the plan keeps: a deck holds it and its card has no other printing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kept {
    /// The printing.
    pub printing: Printing,
    /// What points at it.
    pub references: References,
}

/// What a run removes and keeps. Removals and kept cards are ordered by name, then id, so a log
/// reads the same on every run.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RemovalPlan {
    /// The printings to remove, each with its replacement.
    pub removals: Vec<Removal>,
    /// The printings to keep and report.
    pub kept: Vec<Kept>,
}

impl RemovalPlan {
    /// Whether the run has nothing to remove or report.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.removals.is_empty() && self.kept.is_empty()
    }

    /// The ids of the printings to remove.
    #[must_use]
    pub fn removed_ids(&self) -> Vec<Uuid> {
        self.removals.iter().map(|r| r.printing.id).collect()
    }

    /// The plan as log lines: a summary, then one line per removal with where its references
    /// move, then one per kept card, so a preview can be read before the removal is switched on.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        let summary = format!(
            "{} printing(s) to remove, {} kept because a deck holds a card with no other printing",
            self.removals.len(),
            self.kept.len()
        );
        let removals = self.removals.iter().map(|r| {
            let to = r.replacement.as_ref().map_or_else(
                || "nothing holds it".to_owned(),
                |p| format!("moves to {}", p.label()),
            );
            format!(
                "remove {}: {to}; {}",
                r.printing.label(),
                r.references.label()
            )
        });
        let kept = self.kept.iter().map(|k| {
            format!(
                "keep {}: no other printing; {}",
                k.printing.label(),
                k.references.label()
            )
        });
        std::iter::once(summary)
            .chain(removals)
            .chain(kept)
            .collect()
    }
}

impl Printing {
    /// `Ashling, Rekindled (pecl #124p, <id>)`.
    fn label(&self) -> String {
        format!(
            "{} ({} #{}, {})",
            self.name, self.set, self.collector_number, self.id
        )
    }
}

impl References {
    /// `2 deck card(s), 1 slot(s)`.
    fn label(self) -> String {
        format!("{} deck card(s), {} slot(s)", self.deck_cards, self.slots)
    }
}

/// Whether a run applies its plan or only reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalMode {
    /// Remap, remove and record.
    Apply,
    /// Compute and log the plan; change nothing.
    Preview,
}

impl RemovalMode {
    /// The mode zervice's settings give: it applies only when `PRINTING_REMOVAL` is on and the
    /// whole run is not a dry run.
    #[must_use]
    pub fn of(removal_on: bool, dry_run: bool) -> Self {
        if removal_on && !dry_run {
            Self::Apply
        } else {
            Self::Preview
        }
    }
}

/// What an applied plan changed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RemovalCounts {
    /// Printings deleted.
    pub removed: u64,
    /// Deck card rows moved to a replacement.
    pub remapped_deck_cards: u64,
    /// Command zone slots moved to a replacement.
    pub remapped_slots: u64,
}

/// What a run did: its plan, and what it changed when it applied it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RemovalReport {
    /// The removals and kept cards.
    pub plan: RemovalPlan,
    /// What applying it changed; `None` for a preview.
    pub counts: Option<RemovalCounts>,
}

/// The stored printings `bulk` (the ids of a complete bulk file) no longer lists, by name then id.
#[must_use]
pub fn gone<S: BuildHasher>(stored: &[Printing], bulk: &HashSet<Uuid, S>) -> Vec<Printing> {
    let mut gone: Vec<Printing> = stored
        .iter()
        .filter(|p| !bulk.contains(&p.id))
        .cloned()
        .collect();
    gone.sort_by(|a, b| a.name.cmp(&b.name).then(a.id.cmp(&b.id)));
    gone
}

/// The plan for `gone`: each printing moves to its card's cheapest regular survivor, else the
/// card's preferred survivor. A printing with no survivor is removed when nothing references it
/// and kept when something does, so a deck never loses a card.
#[must_use]
pub fn plan<S: BuildHasher>(
    gone: Vec<Printing>,
    survivors: &Survivors,
    references: &HashMap<Uuid, References, S>,
) -> RemovalPlan {
    let removed: HashSet<Uuid> = gone.iter().map(|p| p.id).collect();
    let survivor = |oracle_id: Uuid| {
        [&survivors.cheapest, &survivors.preferred]
            .into_iter()
            .filter_map(|picks| picks.get(&oracle_id))
            .find(|p| !removed.contains(&p.id))
            .cloned()
    };
    let mut plan = RemovalPlan::default();
    for printing in gone {
        let held = references.get(&printing.id).copied().unwrap_or_default();
        match printing.oracle_id.and_then(survivor) {
            Some(replacement) => plan.removals.push(Removal {
                printing,
                replacement: Some(replacement),
                references: held,
            }),
            None if held.any() => plan.kept.push(Kept {
                printing,
                references: held,
            }),
            None => plan.removals.push(Removal {
                printing,
                replacement: None,
                references: held,
            }),
        }
    }
    plan
}

#[cfg(test)]
mod tests {
    use super::*;

    fn printing(n: u128, oracle: Option<u128>, name: &str, set: &str) -> Printing {
        Printing {
            id: Uuid::from_u128(n),
            oracle_id: oracle.map(Uuid::from_u128),
            name: name.to_owned(),
            set: set.to_owned(),
            collector_number: n.to_string(),
        }
    }

    fn ids(printings: &[Printing]) -> Vec<Uuid> {
        printings.iter().map(|p| p.id).collect()
    }

    const ASHLING: u128 = 0xa5;

    #[test]
    fn gone_lists_the_stored_printings_the_bulk_file_no_longer_has_by_name() {
        let stored = [
            printing(1, Some(ASHLING), "Ashling, Rekindled", "pecl"),
            printing(2, Some(ASHLING), "Ashling, Rekindled", "ecl"),
            printing(3, Some(0xb0), "Bolt", "lea"),
            printing(4, Some(0xa0), "Ancestral Recall", "lea"),
        ];
        let bulk: HashSet<Uuid> = [2, 3].into_iter().map(Uuid::from_u128).collect();
        assert_eq!(
            ids(&gone(&stored, &bulk)),
            vec![Uuid::from_u128(4), Uuid::from_u128(1)],
            "Ancestral Recall, then Ashling's deleted promo"
        );
    }

    #[test]
    fn a_removed_printing_moves_to_its_cards_cheapest_regular_survivor() {
        let promo = printing(1, Some(ASHLING), "Ashling, Rekindled", "pecl");
        let survivors = Survivors {
            cheapest: HashMap::from([(
                Uuid::from_u128(ASHLING),
                printing(2, Some(ASHLING), "Ashling, Rekindled", "ecl"),
            )]),
            preferred: HashMap::from([(
                Uuid::from_u128(ASHLING),
                printing(3, Some(ASHLING), "Ashling, Rekindled", "ecl"),
            )]),
        };
        let held = HashMap::from([(
            promo.id,
            References {
                deck_cards: 2,
                slots: 1,
            },
        )]);
        let plan = plan(vec![promo.clone()], &survivors, &held);
        assert_eq!(
            plan.removals,
            vec![Removal {
                printing: promo,
                replacement: Some(printing(2, Some(ASHLING), "Ashling, Rekindled", "ecl")),
                references: References {
                    deck_cards: 2,
                    slots: 1
                },
            }],
            "the cheapest wins over the preferred printing"
        );
        assert_eq!(plan.kept, Vec::new());
    }

    #[test]
    fn with_no_regular_priced_survivor_it_moves_to_the_preferred_printing() {
        let promo = printing(1, Some(ASHLING), "Ashling, Rekindled", "pecl");
        let foreign = printing(5, Some(ASHLING), "Ashling, Rekindled", "ecl");
        let survivors = Survivors {
            cheapest: HashMap::new(),
            preferred: HashMap::from([(Uuid::from_u128(ASHLING), foreign.clone())]),
        };
        let plan = plan(vec![promo], &survivors, &HashMap::new());
        assert_eq!(
            plan.removals.first().and_then(|r| r.replacement.clone()),
            Some(foreign)
        );
    }

    #[test]
    fn a_card_with_no_survivor_is_removed_when_nothing_holds_it_and_kept_when_a_deck_does() {
        let loose = printing(1, Some(0xc1), "Unheld", "pxyz");
        let held = printing(2, Some(0xc2), "Held", "pxyz");
        let orphan = printing(3, None, "No oracle", "pxyz");
        let references = HashMap::from([
            (
                held.id,
                References {
                    deck_cards: 1,
                    slots: 0,
                },
            ),
            (
                orphan.id,
                References {
                    deck_cards: 0,
                    slots: 1,
                },
            ),
        ]);
        let plan = plan(
            vec![loose.clone(), held.clone(), orphan.clone()],
            &Survivors::default(),
            &references,
        );
        assert_eq!(
            plan.removals,
            vec![Removal {
                printing: loose,
                replacement: None,
                references: References::default(),
            }],
            "an unheld card with no other printing goes"
        );
        assert_eq!(
            ids(&plan
                .kept
                .iter()
                .map(|k| k.printing.clone())
                .collect::<Vec<_>>()),
            vec![held.id, orphan.id],
            "a held one stays, a printing with no oracle id too, since it can have no sibling"
        );
    }

    #[test]
    fn a_survivor_that_is_itself_removed_is_never_a_replacement() {
        let promo = printing(1, Some(ASHLING), "Ashling, Rekindled", "pecl");
        let also_gone = printing(2, Some(ASHLING), "Ashling, Rekindled", "pecl");
        let survivors = Survivors {
            cheapest: HashMap::from([(Uuid::from_u128(ASHLING), also_gone.clone())]),
            preferred: HashMap::from([(Uuid::from_u128(ASHLING), also_gone.clone())]),
        };
        let held = HashMap::from([(
            promo.id,
            References {
                deck_cards: 1,
                slots: 0,
            },
        )]);
        let plan = plan(vec![promo.clone(), also_gone], &survivors, &held);
        assert_eq!(
            plan.kept.iter().map(|k| k.printing.id).collect::<Vec<_>>(),
            vec![promo.id],
            "the held promo stays rather than move onto a printing this same run deletes"
        );
        assert_eq!(plan.removed_ids(), vec![Uuid::from_u128(2)]);
    }

    #[test]
    fn the_plan_reads_as_a_summary_then_each_removal_and_each_kept_card() {
        let promo = Printing {
            collector_number: "124p".to_owned(),
            ..printing(1, Some(ASHLING), "Ashling, Rekindled", "pecl")
        };
        let regular = printing(2, Some(ASHLING), "Ashling, Rekindled", "ecl");
        let held = printing(3, Some(0xc2), "Held", "pxyz");
        let plan = RemovalPlan {
            removals: vec![Removal {
                printing: promo,
                replacement: Some(regular),
                references: References {
                    deck_cards: 2,
                    slots: 1,
                },
            }],
            kept: vec![Kept {
                printing: held,
                references: References {
                    deck_cards: 1,
                    slots: 0,
                },
            }],
        };
        assert_eq!(
            plan.lines(),
            vec![
                "1 printing(s) to remove, 1 kept because a deck holds a card with no other printing".to_owned(),
                format!(
                    "remove Ashling, Rekindled (pecl #124p, {}): moves to Ashling, Rekindled (ecl #2, {}); 2 deck card(s), 1 slot(s)",
                    Uuid::from_u128(1),
                    Uuid::from_u128(2)
                ),
                format!(
                    "keep Held (pxyz #3, {}): no other printing; 1 deck card(s), 0 slot(s)",
                    Uuid::from_u128(3)
                ),
            ]
        );
    }

    #[test]
    fn it_applies_only_when_switched_on_and_not_a_dry_run() {
        assert_eq!(RemovalMode::of(true, false), RemovalMode::Apply);
        assert_eq!(RemovalMode::of(false, false), RemovalMode::Preview);
        assert_eq!(
            RemovalMode::of(true, true),
            RemovalMode::Preview,
            "a dry run applies nothing"
        );
    }
}
