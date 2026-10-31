//! Login: the header, the hero, the two fields and the action bar; and the frame Register and Forgot password share with it.

use std::time::Instant;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;

use super::{LoginField, LoginForm};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::widgets::action_bar::{self, Button};
use crate::inbound::widgets::header::{self, Corner};
use crate::inbound::widgets::hero::Hero;
use crate::inbound::widgets::measure;
use crate::inbound::widgets::text_input::{BOX_HEIGHT, TextInput};

/// The form is at most this wide, in cells.
pub(crate) const FORM_WIDTH: u16 = 48;

/// The hero is at most this wide, in cells.
const HERO_WIDTH: u16 = 64;

/// Where an auth screen's parts go.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AuthAreas {
    /// The form, `FORM_WIDTH` wide at most and centered.
    pub(crate) body: Rect,
    pub(crate) buttons: Rect,
}

/// Whether an auth screen's content sits in the middle of the screen, as Login and Forgot password do, or from the top, as Register does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Placement {
    Centered,
    Top,
}

/// Draws an auth screen's header titled `title` and its key hints `keys`, wrapped when the screen is narrow, and lays out the rest: the hero over a form `body_height` rows tall, then the action bar. The hero shrinks, then goes, when the screen is short.
pub(crate) fn frame(
    f: &mut Frame<'_>,
    shared: &Shared,
    hero: &Hero,
    title: &str,
    body_height: u16,
    placement: Placement,
    keys: Line<'static>,
) -> AuthAreas {
    let area = f.area();
    let keys_height = action_bar::height(&keys, area.width);
    let [head, _, content, buttons, keys_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(keys_height),
    ])
    .areas(area);
    header::draw(f, &shared.palette, head, title, Corner::Empty);
    f.render_widget(action_bar::wrap(keys, keys_area.width), keys_area);
    let hero_width = content.width.min(HERO_WIDTH);
    let room = content.height.saturating_sub(body_height).saturating_sub(1);
    let hero_height = Hero::height(hero_width, room);
    let total = hero_height
        .saturating_add(u16::from(hero_height > 0))
        .saturating_add(body_height)
        .min(content.height);
    let top = match placement {
        Placement::Centered => content
            .y
            .saturating_add(content.height.saturating_sub(total) / 2),
        Placement::Top => content.y,
    };
    let column = Rect {
        y: top,
        height: total,
        ..content
    };
    let [hero_area, _, body] = Layout::vertical([
        Constraint::Length(hero_height),
        Constraint::Length(u16::from(hero_height > 0)),
        Constraint::Fill(1),
    ])
    .areas(column);
    let hero_area = hero_area.centered_horizontally(Constraint::Length(hero_width));
    hero.draw(f, shared, hero_area, Instant::now());
    AuthAreas {
        body: body.centered_horizontally(Constraint::Length(FORM_WIDTH.min(body.width))),
        buttons,
    }
}

/// Draws a field titled `title` at the top of `area`, and its in-field button (Show or Hide) on the right of its text row, reversed while `button_focused`. Places the cursor in it while `focused`.
pub(crate) fn field(
    f: &mut Frame<'_>,
    palette: &Palette,
    area: Rect,
    input: &TextInput,
    title: &str,
    focused: bool,
) {
    input.draw(f, palette, area, title, focused);
    if focused {
        f.set_cursor_position(input.cursor_position(area));
    }
}

/// Draws a password field's Show or Hide button inside its box at `area`.
pub(crate) fn show_button(
    f: &mut Frame<'_>,
    palette: &Palette,
    area: Rect,
    input: &TextInput,
    focused: bool,
) {
    let label = if input.is_masked() {
        " Show "
    } else {
        " Hide "
    };
    let width = measure::width_of(label);
    if area.width < width.saturating_add(4) || area.height < 2 {
        return;
    }
    let spot = Rect {
        x: area.right().saturating_sub(width).saturating_sub(1),
        y: area.y.saturating_add(1),
        width,
        height: 1,
    };
    let style = if focused {
        palette.accent.bold().reversed()
    } else {
        palette.muted
    };
    f.render_widget(Line::from(label).style(style), spot);
}

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, form: &LoginForm) {
    let palette = &shared.palette;
    let keys = action_bar::line(
        palette,
        &[
            ("Tab", "move"),
            ("Enter", "select"),
            ("F1", "help"),
            ("Esc", "quit"),
        ],
    )
    .centered();
    let areas = frame(
        f,
        shared,
        &form.hero,
        "Login",
        BOX_HEIGHT.saturating_mul(2),
        Placement::Centered,
        keys,
    );
    let [identifier, password] =
        Layout::vertical([Constraint::Length(BOX_HEIGHT); 2]).areas(areas.body);
    field(
        f,
        palette,
        identifier,
        &form.identifier,
        "Username or email",
        form.focus == LoginField::Identifier,
    );
    field(
        f,
        palette,
        password,
        &form.password,
        "Password",
        form.focus == LoginField::Password,
    );
    show_button(
        f,
        palette,
        password,
        &form.password,
        form.focus == LoginField::ShowPassword,
    );
    let log_in = if form.busy { "Logging in..." } else { "Log in" };
    let row = [
        Button::new(log_in).disabled(form.busy),
        Button::new("Create profile").disabled(form.busy),
        Button::new("Forgot password").disabled(form.busy),
    ];
    f.render_widget(
        action_bar::buttons(palette, &row, form.focus.button()).centered(),
        areas.buttons,
    );
}
