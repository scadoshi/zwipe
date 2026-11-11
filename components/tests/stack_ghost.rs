//! The swipe stack's ghost card: drawn as a stack card in the top card's
//! own box, it must survive every rule that hides the stack's other cards,
//! or a pile's ghost never shows.

const SCREENS_A: &str = include_str!("../assets/screens/screens-a.css");

/// The selector lines of every rule in screens-a.css that hides stack cards.
fn hiding_rules() -> Vec<&'static str> {
    let mut rules = Vec::new();
    let mut lines = SCREENS_A.lines().peekable();
    while let Some(line) = lines.next() {
        if line.contains(".swipe-stack-card") && line.trim_end().ends_with('{') {
            let body: Vec<&str> = lines.by_ref().take_while(|l| l.trim() != "}").collect();
            if body.iter().any(|l| l.contains("visibility: hidden")) {
                rules.push(line);
            }
        }
    }
    rules
}

#[test]
fn the_ghost_card_is_never_hidden_with_the_stack_s_cards() {
    let rules = hiding_rules();
    assert!(!rules.is_empty(), "the stack has rules hiding its cards");
    for rule in rules {
        assert!(
            rule.contains("swipe-stack-ghost-card"),
            "this rule would hide the ghost card: {rule}"
        );
    }
}
