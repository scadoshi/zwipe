//! Login: the hero, the two fields, `Log in` under them and the key bar; and the frame Register and Forgot password share with it.

use std::time::Instant;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::Line;
use zwipe_ui::screens::login::{CREATE_PROFILE, FORGOT_PASSWORD, Field};

use super::{LoginField, LoginForm};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::widgets::button::{self, Button};
use crate::inbound::widgets::hero::Hero;
use crate::inbound::widgets::text_input::{BOX_HEIGHT, Label, TextInput};
use crate::inbound::widgets::{action_bar, measure};

/// The form is at most this wide, in cells.
pub(crate) const FORM_WIDTH: u16 = 48;

/// The hero is at most this wide, in cells.
const HERO_WIDTH: u16 = 64;

/// Where an auth screen's form goes: `FORM_WIDTH` wide at most and centered, with the row under it, between the form and its buttons, for a screen that has a line to put there.
#[derive(Debug, Clone, Copy)]
pub(crate) struct AuthAreas {
    pub(crate) body: Rect,
    /// The row between the form and its buttons, as wide as the form; empty when there is none.
    pub(crate) gap: Rect,
}

/// Whether an auth screen's content sits in the middle of the screen, as Login and Forgot password do, or from the top, a row down when there is room, as Register does.
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
    /// The buttons under the form, if any, and the one with the focus.
    pub(crate) buttons: &'parts [Button<'parts>],
    pub(crate) focused: Option<usize>,
    /// The rows the screen needs between the form and its buttons, 0 or 1; the frame adds one when the hero loses nothing to it.
    pub(crate) min_gap: u16,
}

/// Draws an auth screen's buttons and its key hints, and lays out the rest: the hero over the form, the buttons on one line under the form, the whole group placed as one. The hero shrinks, then goes, when the screen is short.
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
        min_gap,
    } = screen;
    let area = f.area();
    let keys_height = action_bar::height(&keys, area.width);
    let [content, keys_area] =
        Layout::vertical([Constraint::Fill(1), Constraint::Length(keys_height)]).areas(area);
    f.render_widget(action_bar::wrap(keys, keys_area.width), keys_area);
    let hero_width = area.width.min(HERO_WIDTH);
    let buttons_height = u16::from(!row.is_empty());
    // The hero's room once the form, `gap` rows and the buttons take theirs, a row kept between the hero and the form.
    let hero_room = |gap: u16, buttons: u16| {
        content
            .height
            .saturating_sub(body_height)
            .saturating_sub(gap)
            .saturating_sub(buttons)
            .saturating_sub(1)
    };
    let min_gap = if row.is_empty() { 0 } else { min_gap };
    let hero_height = Hero::height(hero_width, hero_room(min_gap, buttons_height));
    // A row between the form and its buttons while the hero keeps what it shows without it.
    let gap = if !row.is_empty()
        && Hero::height(hero_width, hero_room(1, buttons_height)) == hero_height
        && hero_room(1, buttons_height) > 0
    {
        1
    } else {
        min_gap
    };
    let hero_gap = u16::from(hero_height > 0);
    let total = hero_height
        .saturating_add(hero_gap)
        .saturating_add(body_height)
        .saturating_add(gap)
        .saturating_add(buttons_height)
        .min(content.height);
    let top = match placement {
        Placement::Centered => content
            .y
            .saturating_add(content.height.saturating_sub(total) / 2),
        // A spare row over the hero, so the logo never sits on the screen's top edge.
        Placement::Top => content.y.saturating_add(u16::from(content.height > total)),
    };
    let column = Rect {
        y: top,
        height: total,
        ..content
    };
    let [hero_area, _, body, gap_area, buttons] = Layout::vertical([
        Constraint::Length(hero_height),
        Constraint::Length(hero_gap),
        Constraint::Length(body_height),
        Constraint::Length(gap),
        Constraint::Length(buttons_height),
    ])
    .areas(column);
    if !row.is_empty() {
        button::draw_row(f, &shared.palette, buttons, row, focused);
    }
    let hero_area = hero_area.centered_horizontally(Constraint::Length(hero_width));
    hero.draw(f, shared, hero_area, Instant::now());
    let form_width = Constraint::Length(FORM_WIDTH.min(body.width));
    AuthAreas {
        body: body.centered_horizontally(form_width),
        gap: gap_area.centered_horizontally(form_width),
    }
}

/// Draws a field titled `title` at the top of `area`, and its in-field button (Show or Hide) on the right of its text row, reversed while `button_focused`. Places the cursor in it while `focused`.
pub(crate) fn field<'text>(
    f: &mut Frame<'_>,
    palette: &Palette,
    area: Rect,
    input: &TextInput,
    title: impl Into<Label<'text>>,
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

/// Login's key hints: the form's keys while the focus is in it, with Enter named for what it does from the focused stop and the screen's letters once no field takes them; the way back into the form and the letters once Esc has ended typing.
pub(crate) fn keys(form: &LoginForm) -> Vec<(&'static str, &'static str)> {
    let letters = [("c", CREATE_PROFILE), ("f", FORGOT_PASSWORD)];
    let Some(focus) = form.focus else {
        let mut keys = vec![("Tab", "fields")];
        keys.extend(letters);
        keys.extend([("F1", "help"), ("q", "quit")]);
        return keys;
    };
    let enter = match focus {
        LoginField::Identifier => "next",
        LoginField::Password | LoginField::LogIn => "log in",
        LoginField::ShowPassword if form.password.is_masked() => "show",
        LoginField::ShowPassword => "hide",
    };
    let mut keys = vec![("Tab", "move"), ("Enter", enter)];
    if matches!(focus, LoginField::ShowPassword | LoginField::LogIn) {
        keys.extend(letters);
    }
    keys.extend([("F1", "help"), ("Esc", "done")]);
    keys
}

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, form: &LoginForm) {
    let palette = &shared.palette;
    let keys = action_bar::line(palette, &keys(form));
    let row = [Button::new(form.model.submit_label()).disabled(form.model.is_loading())];
    let areas = frame(
        f,
        shared,
        &form.hero,
        AuthScreen {
            body_height: BOX_HEIGHT.saturating_mul(2),
            placement: Placement::Centered,
            keys,
            buttons: &row,
            focused: form.focus.and_then(LoginField::button),
            min_gap: 0,
        },
    );
    let [identifier, password] =
        Layout::vertical([Constraint::Length(BOX_HEIGHT); 2]).areas(areas.body);
    field(
        f,
        palette,
        identifier,
        &form.identifier,
        Field::Identifier.label(),
        form.focus == Some(LoginField::Identifier),
    );
    field(
        f,
        palette,
        password,
        &form.password,
        Field::Password.label(),
        form.focus == Some(LoginField::Password),
    );
    show_button(
        f,
        palette,
        password,
        &form.password,
        form.focus == Some(LoginField::ShowPassword),
    );
}
