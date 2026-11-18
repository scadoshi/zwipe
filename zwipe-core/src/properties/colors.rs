//! A color identity reads the same however its colors arrive.

use proptest::prelude::*;

use super::strategies::colors;
use crate::domain::card::scryfall_data::colors::identity_of;

proptest! {
    /// Guards the identity every check and chip compares: it lists exactly the colors given,
    /// in WUBRG order, each once.
    #[test]
    fn an_identity_is_the_colors_given_in_wubrg_order_once_each(given in colors()) {
        let identity = identity_of(&given);
        prop_assert!(identity.windows(2).all(|pair| pair.first() < pair.get(1)), "{identity:?}");
        prop_assert!(identity.iter().all(|c| given.contains(c)));
        prop_assert!(given.iter().all(|c| identity.contains(c)));
    }

    /// Guards comparing identities: order and repeats never change one, and normalizing twice
    /// is normalizing once.
    #[test]
    fn order_and_repeats_never_change_an_identity(
        (given, shuffled) in colors().prop_flat_map(|v| (Just(v.clone()), Just(v).prop_shuffle()))
    ) {
        let identity = identity_of(&given);
        prop_assert_eq!(&identity_of(&shuffled), &identity);
        prop_assert_eq!(&identity_of(&identity), &identity);
        let doubled: Vec<_> = given.iter().chain(given.iter()).copied().collect();
        prop_assert_eq!(identity_of(&doubled), identity);
    }
}
