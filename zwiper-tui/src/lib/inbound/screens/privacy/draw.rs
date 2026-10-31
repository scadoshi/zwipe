//! Privacy policy: the header, then the first card (heading, `Last updated` chip, intro), a card per policy section, the Contact card, and the action bar.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use zwipe_core::domain::site::{DISCORD_URL, SUPPORT_EMAIL};
use zwipe_core::legal::{PRIVACY_INTRO_HTML, PRIVACY_LAST_UPDATED, PRIVACY_POLICY_HTML};

use super::PrivacyView;
use crate::domain::legal::{self, Block, Run};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::widgets::action_bar::{self, Button};
use crate::inbound::widgets::header::{self, Corner};
use crate::inbound::widgets::measure;

/// The text is at most this wide, in cells.
const WIDTH: u16 = 76;

/// Runs as spans: bold in bold, links underlined in the accent with their address after.
fn spans(palette: &Palette, runs: &[Run]) -> Vec<Span<'static>> {
    runs.iter()
        .flat_map(|run| match run {
            Run::Plain(text) => vec![Span::raw(text.clone())],
            Run::Bold(text) => vec![Span::raw(text.clone()).bold()],
            Run::Link { text, url } => {
                let trimmed = text.trim();
                let mut out = Vec::new();
                if text.starts_with(' ') {
                    out.push(Span::raw(" "));
                }
                out.push(Span::styled(trimmed.to_owned(), palette.accent).underlined());
                out.push(Span::styled(format!(" ({url})"), palette.muted));
                if text.ends_with(' ') {
                    out.push(Span::raw(" "));
                }
                out
            }
        })
        .collect()
}

/// A card's heading and its rule.
fn heading(
    palette: &Palette,
    title: &str,
    rule: usize,
    chip: Option<String>,
) -> Vec<Line<'static>> {
    let mut head = vec![Span::styled(title.to_owned(), palette.accent).bold()];
    if let Some(chip) = chip {
        head.push(Span::raw("  "));
        head.push(Span::styled(format!(" {chip} "), palette.muted.reversed()));
    }
    vec![
        Line::from(head),
        Line::from("─".repeat(rule)).style(palette.border),
    ]
}

/// Every line of the policy, card after card.
pub(crate) fn lines(palette: &Palette, width: usize) -> Vec<Line<'static>> {
    let mut lines = heading(
        palette,
        "Privacy policy",
        width,
        Some(format!("Last updated {PRIVACY_LAST_UPDATED}")),
    );
    let blocks = |lines: &mut Vec<Line<'static>>, blocks: &[Block]| {
        for block in blocks {
            match block {
                Block::Paragraph(runs) => lines.push(Line::from(spans(palette, runs))),
                Block::Bullet(runs) => {
                    let mut line = vec![Span::styled("• ", palette.muted)];
                    line.extend(spans(palette, runs));
                    lines.push(Line::from(line));
                }
            }
        }
    };
    for section in legal::parse(PRIVACY_INTRO_HTML) {
        blocks(&mut lines, &section.blocks);
    }
    for section in legal::parse(PRIVACY_POLICY_HTML) {
        lines.push(Line::default());
        lines.extend(heading(palette, &section.heading, width, None));
        blocks(&mut lines, &section.blocks);
    }
    lines.push(Line::default());
    lines.extend(heading(palette, "Contact", width, None));
    lines.push(Line::from(spans(
        palette,
        &[
            Run::Plain("Questions or requests? Email ".to_owned()),
            Run::Link {
                text: SUPPORT_EMAIL.to_owned(),
                url: format!("mailto:{SUPPORT_EMAIL}"),
            },
            Run::Plain(" or join the ".to_owned()),
            Run::Link {
                text: "Discord".to_owned(),
                url: DISCORD_URL.to_owned(),
            },
            Run::Plain(" for support.".to_owned()),
        ],
    )));
    lines
}

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &PrivacyView) {
    let palette = &shared.palette;
    let [head, _, body, buttons, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(f.area());
    header::draw(f, palette, head, "Privacy policy", Corner::Empty);
    let body = body.centered_horizontally(Constraint::Length(WIDTH.min(body.width)));
    let text = Paragraph::new(lines(palette, usize::from(body.width))).wrap(Wrap { trim: false });
    let total = measure::height_of(text.line_count(body.width));
    let scroll = view.scroll.min(total.saturating_sub(body.height));
    f.render_widget(text.scroll((scroll, 0)), body);
    f.render_widget(
        action_bar::buttons(palette, &[Button::new("Back")], None).centered(),
        buttons,
    );
    f.render_widget(
        action_bar::line(
            palette,
            &[("j/k", "scroll"), ("!", "help"), ("Esc", "back")],
        )
        .centered(),
        keys,
    );
}
