//! robots.txt, read the way RFC 9309 crawlers read it: a crawler obeys the
//! one group naming its product token, else the `*` group, and within it the
//! longest matching rule decides, `Allow` winning a tie. Search engines keep
//! out of shared decks; the link-preview bots fetch them to draw the card a
//! shared link unfurls into.

#![expect(
    clippy::unwrap_used,
    reason = "a test: a malformed robots.txt fails it"
)]

const ROBOTS: &str = include_str!("../public/robots.txt");

/// The bots that draw a shared link's preview card and honor robots.txt.
const PREVIEW_BOTS: [&str; 9] = [
    "Twitterbot",
    "facebookexternalhit",
    "LinkedInBot",
    "Slackbot-LinkExpanding",
    "Discordbot",
    "TelegramBot",
    "WhatsApp",
    "SkypeUriPreview",
    "Teams",
];

/// One group: its user agents and its `(allow, path)` rules.
struct Group {
    agents: Vec<String>,
    rules: Vec<(bool, String)>,
}

fn groups() -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    let mut in_agents = false;
    for line in ROBOTS.lines() {
        let line = line.split('#').next().unwrap().trim();
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let (key, value) = (key.trim().to_lowercase(), value.trim().to_owned());
        match key.as_str() {
            "user-agent" => {
                if !in_agents {
                    groups.push(Group {
                        agents: Vec::new(),
                        rules: Vec::new(),
                    });
                }
                groups.last_mut().unwrap().agents.push(value.to_lowercase());
                in_agents = true;
            }
            "allow" | "disallow" => {
                in_agents = false;
                if let Some(group) = groups.last_mut() {
                    group.rules.push((key == "allow", value));
                }
            }
            _ => in_agents = false,
        }
    }
    groups
}

/// Whether `bot` may fetch `path`.
fn allowed(bot: &str, path: &str) -> bool {
    let groups = groups();
    let bot = bot.to_lowercase();
    let group = groups
        .iter()
        .find(|g| g.agents.contains(&bot))
        .or_else(|| groups.iter().find(|g| g.agents.iter().any(|a| a == "*")))
        .unwrap();
    group
        .rules
        .iter()
        .filter(|(_, rule)| !rule.is_empty() && path.starts_with(rule.as_str()))
        .max_by_key(|(allow, rule)| (rule.len(), *allow))
        .is_none_or(|(allow, _)| *allow)
}

#[test]
fn search_engines_stay_out_of_shared_decks() {
    for bot in ["Googlebot", "Bingbot", "DuckDuckBot"] {
        assert!(!allowed(bot, "/deck/abc"), "{bot} on /deck/");
        assert!(allowed(bot, "/"), "{bot} on /");
        assert!(allowed(bot, "/guides/"), "{bot} on /guides/");
    }
}

#[test]
fn preview_bots_fetch_shared_decks_and_nothing_private() {
    for bot in PREVIEW_BOTS {
        assert!(allowed(bot, "/deck/abc"), "{bot} on /deck/");
        assert!(allowed(bot, "/"), "{bot} on /");
        for private in ["/verify/x", "/confirm-email/x", "/reset/x"] {
            assert!(!allowed(bot, private), "{bot} on {private}");
        }
    }
}
