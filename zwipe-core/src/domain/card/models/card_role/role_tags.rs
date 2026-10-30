//! The rules that map oracle tags to card roles, shared by zerver and the
//! synergy worker.
//!
//! A role owns the whole subtree under each of its [`CATEGORY_ROOTS`] in the
//! `oracle_tags` hierarchy, plus the exact slugs in [`ROLE_TAG_OVERRIDES`], minus
//! the exact slugs in [`ROLE_TAG_EXCLUSIONS`]. zerver runs these rules as SQL,
//! binding the pairs below into its recursive CTE, when it derives
//! `card_profiles.card_roles`, groups a card's tags by role and scores commander
//! suggestions. [`RoleMembership::from_hierarchy`] runs the same rules over the
//! hierarchy as plain data, so the synergy worker builds commander tag profiles
//! with the same role filter (the `has_role` of
//! [`profile_weights`](crate::domain::deck::tag_profile::profile_weights)).
//!
//! Role keys are `CardRole` serde names (`card_advantage`), checked by unit test.
//! Slugs are not checked here; zervice warns about override and exclusion slugs
//! the live catalog lacks.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use uuid::Uuid;

/// `CardRole` (serde `snake_case`) to the oracle-tag root slugs whose subtree
/// defines it. Each root is expanded through `oracle_tags.parent_ids`.
///
/// `pump`, `stax`, `protection`, `graveyard_hate` and `tokens` are supplements:
/// those roles are also tagged by the heuristic in `oracle_tag_gaps` (or, for
/// `tokens`, from Scryfall's `all_parts`), and their roots here group their tags
/// under the role on the card display. Only `finisher` is absent.
pub const CATEGORY_ROOTS: &[(&str, &[&str])] = &[
    (
        "anthem",
        &[
            "anthem",
            "keyword-anthem",
            "power-boost-to-all",
            "toughness-boost-to-all",
        ],
    ),
    (
        "aggression",
        &[
            "attacking-matters",
            "attacking-matters-self",
            "attacking-matters-any",
            "attack-trigger",
            "gives-haste",
        ],
    ),
    ("blink", &["flicker"]),
    ("burn", &["burn"]),
    // Broad umbrella: everything that nets cards. Draw (below) is the literal-draw
    // subset of this, so the two intentionally overlap.
    ("card_advantage", &["card-advantage"]),
    ("copy", &["copy", "clone"]),
    ("counterspell", &["counterspell"]),
    (
        "counters",
        &[
            "gains-pp-counters",
            "gives-pp-counters",
            "repeatable-pp-counters",
            "counters-matter",
            "gives-mm-counters",
            "counter-increaser",
            "gains-mm-counters",
        ],
    ),
    ("drain", &["drain-life"]),
    // Literal card-drawing only: the narrow subset of `card_advantage`.
    (
        "draw",
        &[
            "pure-draw",
            "repeatable-pure-draw",
            "burst-draw",
            "draw-engine",
            "impulsive-draw",
            "repeatable-impulsive-draw",
            "long-term-impulsive-draw",
            "force-draw",
        ],
    ),
    (
        "energy",
        &[
            "energy-generator",
            "counter-fuel-energy",
            "energy-increaser",
            "synergy-energy",
        ],
    ),
    ("evasion", &["evasion", "gives-evasion"]),
    ("graveyard_hate", &["hate-graveyard"]),
    ("lifegain", &["lifegain", "lifegain-matters"]),
    ("mill", &["mill"]),
    ("protection", &["protection"]),
    ("pump", &["combat-trick"]),
    ("ramp", &["ramp", "mana-producer"]),
    ("recursion", &["recursion", "reanimate"]),
    ("removal", &["removal", "lockdown"]),
    ("sacrifice", &["sacrifice-outlet"]),
    ("stax", &["tax", "lockdown", "pillowfort"]),
    // Supplements the all_parts-based Tokens membership: nests the token-generation
    // otag family under the Tokens role instead of dumping it in "Other".
    (
        "tokens",
        &[
            "repeatable-token-generator",
            "synergy-token",
            "token-versions-of-cards",
            "token-increaser",
        ],
    ),
    ("tutor", &["tutor"]),
    ("untap", &["untapper"]),
    ("wipe", &["sweeper", "mass-land-denial"]),
];

/// Manual one-off patches: exact oracle-tag slugs whose wording dodges Scryfall's
/// hierarchy but clearly belong to a role. Unlike [`CATEGORY_ROOTS`], these are
/// matched **exactly** (never subtree-expanded), so an entry adds precisely that
/// tag and nothing beneath it, whether or not the catalog carries it. Key = a
/// real `CardRole` (`snake_case`), checked by unit test; value = real oracle-tag
/// slugs, checked by a non-fatal `zervice` warn. Grow as you audit; applies on
/// the next `zervice` run.
pub const ROLE_TAG_OVERRIDES: &[(&str, &[&str])] = &[
    (
        "protection",
        &[
            "fog-selective",
            "damage-prevention",
            "phasing",
            "gains-indestructible",
            "regenerates-self",
            "damage-prevention-you",
            "damage-prevention-self",
            "regenerates-other",
            "pseudo-fog",
        ],
    ),
    (
        "ramp",
        &[
            "gives-mana-ability",
            "repeatable-treasures",
            "adds-multiple-mana",
        ],
    ),
    ("aggression", &["gains-haste", "extra-combat-phase"]),
    ("burn", &["pinger"]),
    ("copy", &["conjure-duplicate"]),
    ("counters", &["move-counters", "pseudo-proliferate"]),
    ("drain", &["drain-creature"]),
    ("graveyard_hate", &["hate-graveyard-cast"]),
    ("lifegain", &["lifegain-increaser"]),
    ("mill", &["synergy-mill"]),
    ("pump", &["shade-pump", "firebreathing"]),
    (
        "recursion",
        &[
            "castable-from-graveyard",
            "temporary-reanimation",
            "mass-reanimation",
        ],
    ),
    ("removal", &["swap-removal", "mass-land-denial"]),
    (
        "sacrifice",
        &[
            "your-sacrifice-matters",
            "sacrifice-self",
            "synergy-sacrifice-self",
        ],
    ),
    (
        "stax",
        &[
            "mass-land-denial",
            "prevent-activation",
            "prevent-cast",
            "hatebear",
        ],
    ),
    (
        "tokens",
        &[
            "out-of-color-token",
            "unique-token",
            "temporary-token",
            "donate-token",
            "unprinted-token",
            "named-token",
        ],
    ),
    ("tutor", &["booster-tutor", "synergy-tutor"]),
    (
        "untap",
        &[
            "extra-untap",
            "untapper-planeswalker",
            "untapper-equipment",
            "ritual-untap",
        ],
    ),
];

/// Exact `(role, oracle-tag slug)` pairs to **subtract** from a role after subtree
/// expansion and overrides: tags whose Scryfall parent chain drags them under a
/// role's root but which don't belong there. The subtree mechanism only adds,
/// and a tag can sit under several roots at once (multi-parent), so narrowing a
/// root can't remove these cleanly; an explicit exclusion is the only
/// sync-proof lever. Matched **exactly** (no subtree expansion: the excluded
/// tag's children stay), keyed by a real `CardRole` (unit-test checked); the
/// slug side is warn-checked against the live catalog. Grow as you audit;
/// applies on the next `zervice` run.
pub const ROLE_TAG_EXCLUSIONS: &[(&str, &[&str])] = &[
    // burn-self hangs off burn-creature deep under the removal subtree; it is a
    // self-burn creature, not removal.
    ("removal", &["burn-self"]),
    // twiddle is a direct child of the `untapper` root but also means *tap*
    // (opponent's permanents), so it over-selects for the untap role.
    ("untap", &["twiddle"]),
    // counter-fuel-aesthetic is flavor/counter-fuel, not a +1/+1-counters payoff.
    ("counters", &["counter-fuel-aesthetic"]),
    // repeatable-maps (Maps) is a multi-parent artifact-token/surveil tag; it
    // lands under mill via `surveil` but isn't a mill payoff.
    ("mill", &["repeatable-maps"]),
];

/// Flattens [`CATEGORY_ROOTS`] into parallel `(role, root slug)` arrays, the
/// shape SQL `unnest` takes.
#[must_use]
pub fn root_pairs() -> (Vec<String>, Vec<String>) {
    pairs(CATEGORY_ROOTS)
}

/// Flattens [`ROLE_TAG_OVERRIDES`] into parallel `(role, slug)` arrays.
#[must_use]
pub fn override_pairs() -> (Vec<String>, Vec<String>) {
    pairs(ROLE_TAG_OVERRIDES)
}

/// Flattens [`ROLE_TAG_EXCLUSIONS`] into parallel `(role, slug)` arrays.
#[must_use]
pub fn exclusion_pairs() -> (Vec<String>, Vec<String>) {
    pairs(ROLE_TAG_EXCLUSIONS)
}

/// Flattens a `(role, &[slug])` table into parallel `(role, slug)` arrays.
fn pairs(table: &[(&str, &[&str])]) -> (Vec<String>, Vec<String>) {
    table
        .iter()
        .flat_map(|(role, slugs)| {
            slugs
                .iter()
                .map(|slug| ((*role).to_string(), (*slug).to_string()))
        })
        .unzip()
}

/// One row of the `oracle_tags` hierarchy: the columns role membership reads.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OracleTagNode {
    /// Scryfall's tag id, which other rows name in their `parent_ids`.
    pub id: Uuid,
    /// The slug cards are tagged with.
    pub slug: String,
    /// Ids of this tag's parents. A tag may have several, and an id may name a
    /// tag missing from the hierarchy.
    pub parent_ids: Vec<Uuid>,
}

/// Which card roles each oracle tag slug falls under.
///
/// Built by [`RoleMembership::from_hierarchy`] with the same result as zerver's
/// `membership(role, slug)` CTE: a role holds every slug in the subtree of each
/// of its roots present in the hierarchy (following children through
/// `parent_ids`, so a tag with several parents joins every role above it), plus
/// its override slugs whether or not the hierarchy carries them, minus its
/// exclusion slugs, which beat both. A role left with no slugs is absent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoleMembership {
    by_role: BTreeMap<String, BTreeSet<String>>,
    by_slug: BTreeMap<String, BTreeSet<String>>,
}

impl RoleMembership {
    /// Applies [`CATEGORY_ROOTS`], [`ROLE_TAG_OVERRIDES`] and
    /// [`ROLE_TAG_EXCLUSIONS`] to the `oracle_tags` hierarchy, every row of
    /// `SELECT id, slug, parent_ids FROM oracle_tags`. A cycle in the hierarchy
    /// is walked once.
    #[must_use]
    pub fn from_hierarchy(tags: &[OracleTagNode]) -> Self {
        Self::from_rules(
            tags,
            CATEGORY_ROOTS,
            ROLE_TAG_OVERRIDES,
            ROLE_TAG_EXCLUSIONS,
        )
    }

    /// [`Self::from_hierarchy`] over any rule tables.
    fn from_rules(
        tags: &[OracleTagNode],
        roots: &[(&str, &[&str])],
        overrides: &[(&str, &[&str])],
        exclusions: &[(&str, &[&str])],
    ) -> Self {
        let mut children: HashMap<Uuid, Vec<&OracleTagNode>> = HashMap::new();
        for tag in tags {
            for parent in &tag.parent_ids {
                children.entry(*parent).or_default().push(tag);
            }
        }
        let mut by_role: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (role, root_slugs) in roots {
            let mut seen: HashSet<Uuid> = HashSet::new();
            let mut stack: Vec<&OracleTagNode> = tags
                .iter()
                .filter(|tag| root_slugs.contains(&tag.slug.as_str()))
                .collect();
            while let Some(tag) = stack.pop() {
                if !seen.insert(tag.id) {
                    continue;
                }
                by_role
                    .entry((*role).to_string())
                    .or_default()
                    .insert(tag.slug.clone());
                if let Some(kids) = children.get(&tag.id) {
                    stack.extend(kids.iter().copied());
                }
            }
        }
        for (role, slugs) in overrides {
            by_role
                .entry((*role).to_string())
                .or_default()
                .extend(slugs.iter().map(|slug| (*slug).to_string()));
        }
        for (role, slugs) in exclusions {
            if let Some(members) = by_role.get_mut(*role) {
                for slug in *slugs {
                    members.remove(*slug);
                }
            }
        }
        by_role.retain(|_, members| !members.is_empty());
        let mut by_slug: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for (role, members) in &by_role {
            for slug in members {
                by_slug
                    .entry(slug.clone())
                    .or_default()
                    .insert(role.clone());
            }
        }
        Self { by_role, by_slug }
    }

    /// Whether `slug` falls under at least one card role. Pass it as the
    /// `has_role` of
    /// [`profile_weights`](crate::domain::deck::tag_profile::profile_weights).
    #[must_use]
    pub fn has_role(&self, slug: &str) -> bool {
        self.by_slug.contains_key(slug)
    }

    /// The roles `slug` falls under, or `None` when it falls under none.
    #[must_use]
    pub fn roles_of(&self, slug: &str) -> Option<&BTreeSet<String>> {
        self.by_slug.get(slug)
    }

    /// The slugs under `role`, or `None` when it has none.
    #[must_use]
    pub fn slugs_of(&self, role: &str) -> Option<&BTreeSet<String>> {
        self.by_role.get(role)
    }

    /// Every role with its slugs, both in sorted order.
    #[must_use]
    pub fn by_role(&self) -> &BTreeMap<String, BTreeSet<String>> {
        &self.by_role
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::card::card_role::CardRole;

    fn node(id: u128, slug: &str, parents: &[u128]) -> OracleTagNode {
        OracleTagNode {
            id: Uuid::from_u128(id),
            slug: slug.to_string(),
            parent_ids: parents.iter().copied().map(Uuid::from_u128).collect(),
        }
    }

    fn set(slugs: &[&str]) -> BTreeSet<String> {
        slugs.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn rule_roles_are_card_roles() {
        for (name, table) in [
            ("CATEGORY_ROOTS", CATEGORY_ROOTS),
            ("ROLE_TAG_OVERRIDES", ROLE_TAG_OVERRIDES),
            ("ROLE_TAG_EXCLUSIONS", ROLE_TAG_EXCLUSIONS),
        ] {
            for (role, _) in table {
                assert!(
                    CardRole::try_from(*role).is_ok(),
                    "{name}: '{role}' is not a CardRole variant"
                );
            }
        }
    }

    /// The roots cover the otag-derived roles plus the supplements; only
    /// finisher stays out.
    #[test]
    fn roots_cover_every_role_but_finisher() {
        let mapped: Vec<&str> = CATEGORY_ROOTS.iter().map(|(c, _)| *c).collect();
        for supplement in ["pump", "stax", "protection", "graveyard_hate", "tokens"] {
            assert!(mapped.contains(&supplement), "{supplement} is not mapped");
        }
        assert!(!mapped.contains(&"finisher"));
        assert_eq!(mapped.len(), 26);
    }

    #[test]
    fn pairs_flatten_in_table_order() {
        let (roles, slugs) = pairs(&[("a", &["x", "y"]), ("b", &[]), ("c", &["z"])]);
        assert_eq!(roles, ["a", "a", "c"]);
        assert_eq!(slugs, ["x", "y", "z"]);
        let (roles, slugs) = root_pairs();
        let total: usize = CATEGORY_ROOTS.iter().map(|(_, s)| s.len()).sum();
        assert_eq!((roles.len(), slugs.len()), (total, total));
    }

    #[test]
    fn roots_expand_through_every_descendant() {
        let tags = [
            node(1, "root", &[]),
            node(2, "child", &[1]),
            node(3, "grandchild", &[2]),
            node(4, "stray", &[]),
            node(5, "orphan", &[99]),
        ];
        let m = RoleMembership::from_rules(&tags, &[("r", &["root", "missing-root"])], &[], &[]);
        assert_eq!(
            m.slugs_of("r"),
            Some(&set(&["root", "child", "grandchild"]))
        );
        assert!(!m.has_role("stray") && !m.has_role("orphan"));
    }

    #[test]
    fn a_tag_with_several_parents_joins_each_role() {
        let tags = [
            node(1, "a-root", &[]),
            node(2, "b-root", &[]),
            node(3, "shared", &[1, 2]),
            node(4, "under-shared", &[3]),
        ];
        let m =
            RoleMembership::from_rules(&tags, &[("a", &["a-root"]), ("b", &["b-root"])], &[], &[]);
        assert_eq!(m.roles_of("shared"), Some(&set(&["a", "b"])));
        assert_eq!(m.roles_of("under-shared"), Some(&set(&["a", "b"])));
    }

    #[test]
    fn overrides_add_exact_slugs_even_off_the_hierarchy() {
        let tags = [node(1, "patched", &[]), node(2, "below-patched", &[1])];
        let m = RoleMembership::from_rules(&tags, &[], &[("r", &["patched", "uncataloged"])], &[]);
        assert_eq!(m.slugs_of("r"), Some(&set(&["patched", "uncataloged"])));
        assert!(!m.has_role("below-patched"), "overrides never expand");
    }

    #[test]
    fn exclusions_remove_exact_slugs_from_one_role() {
        let tags = [
            node(1, "a-root", &[]),
            node(2, "b-root", &[]),
            node(3, "dragged", &[1, 2]),
            node(4, "below-dragged", &[3]),
        ];
        let m = RoleMembership::from_rules(
            &tags,
            &[("a", &["a-root"]), ("b", &["b-root"])],
            &[("a", &["extra"])],
            &[("a", &["dragged", "extra"]), ("c", &["anything"])],
        );
        assert_eq!(m.roles_of("dragged"), Some(&set(&["b"])));
        assert_eq!(m.roles_of("below-dragged"), Some(&set(&["a", "b"])));
        assert!(!m.has_role("extra"), "exclusions beat overrides");
    }

    #[test]
    fn an_emptied_role_is_absent() {
        let tags = [node(1, "only", &[])];
        let m = RoleMembership::from_rules(&tags, &[("r", &["only"])], &[], &[("r", &["only"])]);
        assert!(m.by_role().is_empty());
        assert_eq!(m, RoleMembership::default());
    }

    #[test]
    fn cycles_terminate() {
        let tags = [node(1, "root", &[2]), node(2, "loop", &[1])];
        let m = RoleMembership::from_rules(&tags, &[("r", &["root"])], &[], &[]);
        assert_eq!(m.slugs_of("r"), Some(&set(&["root", "loop"])));
    }

    #[test]
    fn real_rules_apply_overrides_and_exclusions() {
        let tags = [
            node(1, "removal", &[]),
            node(2, "burn-creature", &[1]),
            node(3, "burn-self", &[2]),
            node(4, "untapper", &[]),
            node(5, "twiddle", &[4]),
        ];
        let m = RoleMembership::from_hierarchy(&tags);
        assert_eq!(m.roles_of("burn-creature"), Some(&set(&["removal"])));
        assert!(!m.has_role("burn-self") && !m.has_role("twiddle"));
        assert_eq!(m.roles_of("pinger"), Some(&set(&["burn"])));
        assert_eq!(
            m.roles_of("mass-land-denial"),
            Some(&set(&["removal", "stax"]))
        );
    }
}
