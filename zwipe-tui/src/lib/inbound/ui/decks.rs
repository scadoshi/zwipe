//! The deck list.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, HighlightSpacing, List, ListItem, ListState, Paragraph};

use super::hints;
use crate::domain::theme::Palette;
use crate::inbound::app::{App, DeckList};

pub(super) fn draw(f: &mut Frame<'_>, palette: &Palette, app: &App, list: &DeckList) {
    let [header, body, status, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .margin(1)
    .areas(f.area());

    let user = app.username.as_deref().unwrap_or_default();
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
            .map(|deck| {
                let commander = deck
                    .commander_name
                    .as_deref()
                    .map_or_else(String::new, |c| format!("  {c}"));
                ListItem::new(Line::from(vec![
                    Span::raw(deck.name.to_string()),
                    Span::styled(commander, palette.muted),
                    Span::styled(format!("  {} cards", deck.card_count), palette.muted),
                ]))
            })
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

    f.render_widget(Line::from(app.status.clone()).style(palette.muted), status);
    f.render_widget(
        hints(
            palette,
            &[
                ("j/k", "move"),
                ("Enter", "swipe"),
                ("r", "reload"),
                ("O", "sign out"),
                ("q", "quit"),
            ],
        ),
        keys,
    );
}
