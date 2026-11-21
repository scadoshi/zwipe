//! A card list's Show more leaves on an eased exit once it deals its page: it fades on the kit's
//! entrance curve and length, and with motion allowed it also folds away, taking back the chip
//! gap over it so what sits under it glides up to the panel gap it has without the chip. Under
//! reduced motion it only fades.

const SCREENS_B: &str = include_str!("../assets/screens/screens-b.css");

#[test]
fn show_more_sits_a_chip_row_s_gap_under_its_panel_and_a_panel_gap_over_the_next() {
    assert!(
        SCREENS_B.contains(
            ".card-group:has(+ .card-list-more) {\n    margin-bottom: var(--space-2);\n}"
        ),
        "8px between the panel and its chip, as a chip row keeps over its panel"
    );
    assert!(
        SCREENS_B.contains(
            ".card-list-more {\n    display: grid;\n    grid-template-rows: 1fr;\n    width: fit-content;\n    margin: 0 auto var(--panel-gap);\n}"
        ),
        "one row it can fold to nothing, and the panel gap to what follows"
    );
    assert!(
        SCREENS_B.contains(".card-list-more > * {\n    display: flex;\n    min-height: 0;\n}"),
        "its content lets its row fold to nothing, and holds the chip without a line box's extra pixel"
    );
}

#[test]
fn leaving_it_fades_on_the_kit_s_entrance_curve_and_length() {
    assert!(
        SCREENS_B.contains(
            ".card-list-more.card-list-more-leaving {\n    pointer-events: none;\n    animation: card-list-more-fade var(--enter-duration) var(--ease-enter) forwards;\n}"
        ),
        "{SCREENS_B}"
    );
    assert!(
        SCREENS_B
            .contains("@keyframes card-list-more-fade {\n    to {\n        opacity: 0;\n    }\n}"),
        "the fade alone moves nothing"
    );
}

#[test]
fn with_motion_allowed_it_folds_away_and_hands_back_the_chip_gap() {
    assert!(
        SCREENS_B.contains(
            "@media (prefers-reduced-motion: no-preference) {\n    .card-list-more.card-list-more-leaving {\n        overflow: hidden;\n        transform-origin: top center;\n        animation-name: card-list-more-leave;\n    }\n}"
        ),
        "{SCREENS_B}"
    );
    assert!(
        SCREENS_B.contains(
            "@keyframes card-list-more-leave {\n    to {\n        grid-template-rows: 0fr;\n        margin-top: calc(-1 * var(--space-2));\n        opacity: 0;\n        transform: scale(0.96);\n    }\n}"
        ),
        "its row folds to nothing and its top takes back the panel's 8px, leaving the panel gap"
    );
}
