//! A section's empty note sits evenly in the web's sections: the rule leaves less room under it
//! than the section keeps at its foot, so a note that opens a section's body takes the
//! difference over it.

const WEB: &str = include_str!("../assets/web.css");

#[test]
fn a_note_that_opens_a_section_body_evens_the_rule_s_room() {
    assert!(
        WEB.contains(
            ".web-section-body > .activity-total:first-child {\n    padding-top: var(--space-1);\n}"
        ),
        "8px under the rule and 4px over the note match the section's 12px foot"
    );
}
