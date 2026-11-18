//! A card holds 1 to 99 copies, in the domain and on the wire.

use proptest::prelude::*;

use crate::domain::deck::Quantity;

/// Any `i32`, weighted toward the edges of 1..=99.
fn any_count() -> impl Strategy<Value = i32> {
    prop_oneof![any::<i32>(), -2..=102i32]
}

proptest! {
    /// Guards the copy range: a `Quantity` exists exactly for 1 through 99 and holds the
    /// count it was given.
    #[test]
    fn a_quantity_exists_exactly_for_1_through_99(n in any_count()) {
        let quantity = Quantity::new(n);
        prop_assert_eq!(quantity.is_ok(), (1..=Quantity::MAX).contains(&n));
        if let Ok(quantity) = quantity {
            prop_assert_eq!(*quantity, n);
        }
    }

    /// Guards the wire: every valid count goes out as a bare number and reads back the same.
    #[test]
    fn every_quantity_round_trips_through_json(n in 1..=Quantity::MAX) {
        let quantity = Quantity::new(n).unwrap();
        let json = serde_json::to_string(&quantity).unwrap();
        prop_assert_eq!(&json, &n.to_string());
        let back: Quantity = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back, quantity);
    }

    /// Guards the wire's edge: no count outside 1 through 99 parses into a `Quantity`.
    #[test]
    fn json_outside_1_through_99_never_parses(
        n in any_count().prop_filter("outside the range", |n| !(1..=Quantity::MAX).contains(n))
    ) {
        prop_assert!(serde_json::from_str::<Quantity>(&n.to_string()).is_err());
    }
}
