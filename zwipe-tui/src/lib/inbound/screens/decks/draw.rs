//! The deck list.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState, Paragraph};
use zwipe_core::domain::deck::DeckProfile;
use zwipe_core::domain::deck::deck_tag::deck_tag_label;

use super::DeckList;
use crate::domain;
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::widgets::action_bar;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, list: &DeckList) {
    let palette = &shared.palette;
    let [header, body, status, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .margin(1)
    .areas(f.area());

    let user = shared.username.as_deref().unwrap_or_default();
    f.render_widget(
        Line::from(vec![
            Span::styled("ZWIPE", palette.accent).bold(),
            Span::styled(format!("  {user}"), palette.muted),
        ]),
        header,
    );

    let block = Block::bordered()
        .title(" Decks ")
        .border_style(palette.border);
    let placeholder = if list.loading {
        Some(Line::from("Loading decks...").style(palette.muted))
    } else if let Some(error) = &list.error {
        Some(Line::from(error.clone()).style(palette.error))
    } else if list.decks.is_empty() {
        Some(Line::from("No decks yet. Create one in the ZWIPE app.").style(palette.muted))
    } else {
        None
    };

    if let Some(placeholder) = placeholder {
        f.render_widget(Paragraph::new(placeholder).block(block), body);
    } else {
        let items: Vec<ListItem<'_>> = list
            .decks
            .iter()
            .map(|deck| ListItem::new(deck_row(palette, deck)))
            .collect();
        let deck_list = List::new(items)
            .block(block)
            .highlight_style(palette.accent.bold())
            .highlight_symbol("> ")
            .highlight_spacing(HighlightSpacing::Always);
        f.render_stateful_widget(
            deck_list,
            body,
            &mut ListState::default().with_selected(Some(list.selected)),
        );
    }

    f.render_widget(
        Line::from(shared.status.clone()).style(palette.muted),
        status,
    );
    f.render_widget(
        action_bar::line(
            palette,
            &[
                ("j/k", "move"),
                ("Enter", "open"),
                ("r", "reload"),
                ("Esc", "home"),
                ("O", "sign out"),
                ("q", "quit"),
            ],
        ),
        keys,
    );
}

/// A deck's row, as the app lays it out: name, color pips, card count, format, power level, the command zone, then tags.
fn deck_row(palette: &Palette, deck: &DeckProfile) -> Line<'static> {
    let gap = || Span::raw("  ");
    let mut spans = vec![Span::raw(deck.name.to_string()).bold(), gap()];
    for pip in domain::deck::pips(deck) {
        let style = palette.mana.get(pip).copied().unwrap_or(palette.muted);
        spans.push(Span::styled(
            ["W", "U", "B", "R", "G"].get(pip).copied().unwrap_or("?"),
            style.bold(),
        ));
    }
    let count = domain::deck::card_count(deck);
    let count_style = if domain::deck::count_fits(deck, count) {
        palette.muted
    } else {
        palette.warning
    };
    spans.extend([
        gap(),
        Span::styled(domain::deck::cards_label(count), count_style),
    ]);
    if let Some(format) = deck.format {
        spans.extend([
            gap(),
            Span::styled(format.display_name().to_owned(), palette.muted),
        ]);
    }
    if let Some(power) = deck.power_level {
        spans.extend([gap(), Span::styled(power.display_name(), palette.muted)]);
    }
    for name in [
        &deck.commander_name,
        &deck.partner_commander_name,
        &deck.background_name,
        &deck.signature_spell_name,
    ]
    .into_iter()
    .flatten()
    {
        spans.extend([gap(), Span::styled(name.clone(), palette.accent)]);
    }
    let tags: Vec<String> = deck
        .tags
        .iter()
        .map(|t| deck_tag_label(t))
        .chain(deck.other_tags.iter().map(|t| t.display_name().to_owned()))
        .collect();
    if !tags.is_empty() {
        spans.extend([gap(), Span::styled(tags.join(", "), palette.muted)]);
    }
    Line::from(spans)
}
