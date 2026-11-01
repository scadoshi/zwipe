//! Login: the hero, the two fields and the action bar; and the frame Register and Forgot password share with it.

use std::time::Instant;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;

use super::{LoginField, LoginForm};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::widgets::button::{self, Button};
use crate::inbound::widgets::hero::Hero;
use crate::inbound::widgets::text_input::{BOX_HEIGHT, TextInput};
use crate::inbound::widgets::{action_bar, measure};

/// The form is at most this wide, in cells.
pub(crate) const FORM_WIDTH: u16 = 48;

/// The hero is at most this wide, in cells.
const HERO_WIDTH: u16 = 64;

/// Where an auth screen's form goes: `FORM_WIDTH` wide at most and centered.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AuthAreas {
    pub(crate) body: Rect,
}

/// Whether an auth screen's content sits in the middle of the screen, as Login and Forgot password do, or from the top, as Register does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Placement {
    Centered,
    Top,
}

/// An auth screen's own parts, which [`frame`] lays out around the hero.
#[derive(Debug)]
pub(crate) struct AuthScreen<'parts> {
    /// The form's rows.
    pub(crate) body_height: u16,
    pub(crate) placement: Placement,
    /// The key hints, wrapped when the screen is narrow.
    pub(crate) keys: Line<'static>,
    /// The action bar's buttons, and the one with the focus.
    pub(crate) buttons: &'parts [Button<'parts>],
    pub(crate) focused: Option<usize>,
}

/// Draws an auth screen's buttons and its key hints, and lays out the rest: the hero over the form. The buttons are boxes while the form fits beside them and the hero loses nothing to them, else pills; the hero shrinks, then goes, when the screen is short.
pub(crate) fn frame(
    f: &mut Frame<'_>,
    shared: &Shared,
    hero: &Hero,
    screen: AuthScreen<'_>,
) -> AuthAreas {
    let AuthScreen {
        body_height,
        placement,
        keys,
        buttons: row,
        focused,
    } = screen;
    let area = f.area();
    let keys_height = action_bar::height(&keys, area.width);
    let hero_width = area.width.min(HERO_WIDTH);
    // The hero's room once the buttons and the keys take theirs.
    let hero_room = |buttons: u16| {
        area.height
            .saturating_sub(keys_height)
            .saturating_sub(buttons)
            .saturating_sub(body_height)
            .saturating_sub(1)
    };
    // Boxes while the form fits beside them and the hero keeps what it shows over pills.
    let boxed = button::height(row, area.width, hero_room(1).saturating_add(1)) == button::HEIGHT
        && Hero::height(hero_width, hero_room(button::HEIGHT))
            == Hero::height(hero_width, hero_room(1));
    let buttons_height = if boxed { button::HEIGHT } else { 1 };
    let [content, buttons, keys_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(buttons_height),
        Constraint::Length(keys_height),
    ])
    .areas(area);
    button::draw_row(f, &shared.palette, buttons, row, focused);
    f.render_widget(action_bar::wrap(keys, keys_area.width), keys_area);
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
    let log_in = if form.busy { "Logging in..." } else { "Log in" };
    let row = [
        Button::new(log_in).primary().disabled(form.busy),
        Button::new("Create profile").disabled(form.busy),
        Button::new("Forgot password").disabled(form.busy),
    ];
    let areas = frame(
        f,
        shared,
        &form.hero,
        AuthScreen {
            body_height: BOX_HEIGHT.saturating_mul(2),
            placement: Placement::Centered,
            keys,
            buttons: &row,
            focused: form.focus.button(),
        },
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
}
