//! The preview: the face's image filling the screen, or its text when images are off or it failed to load.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin};
use ratatui::text::Line;
use ratatui::widgets::{Block, Padding, Paragraph, Wrap};
use ratatui_image::StatefulImage;

use super::{FaceImage, ImagePreview};
use crate::domain::card;
use crate::inbound::app::Shared;
use crate::inbound::widgets::{action_bar, card_text};

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &ImagePreview) {
    let palette = &shared.palette;
    let area = f.area().inner(Margin::new(1, 0));
    let [body, keys] = Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
    match &view.image {
        FaceImage::Ready(protocol) => {
            // A protocol already borrowed by a draw in progress is skipped for this frame.
            if let Ok(mut protocol) = protocol.try_borrow_mut() {
                f.render_stateful_widget(StatefulImage::default(), body, &mut *protocol);
            }
        }
        FaceImage::Loading => f.render_widget(
            Paragraph::new(Line::styled("Loading image...", palette.muted)).centered(),
            body.centered_vertically(Constraint::Length(1)),
        ),
        FaceImage::None | FaceImage::Failed => {
            let faces = card::faces(&view.card.scryfall_data);
            let lines = faces
                .get(view.face)
                .or_else(|| faces.first())
                .map(|face| card_text::face_lines(palette, face))
                .unwrap_or_default();
            f.render_widget(
                Paragraph::new(lines).wrap(Wrap { trim: true }).block(
                    Block::bordered()
                        .border_style(palette.accent)
                        .padding(Padding::horizontal(1)),
                ),
                body.centered_horizontally(Constraint::Max(48)),
            );
        }
    }
    let mut hints = vec![("Esc", "close")];
    if view.faces() > 1 {
        hints.push(("f", "flip"));
    }
    f.render_widget(action_bar::line(palette, &hints), keys);
}
