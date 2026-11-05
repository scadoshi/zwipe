//! Config: the theme's Dark/Light button and the Themes sheet, the pip style, Universes Beyond and its Exceptions sheet.
//!
//! The theme and the pip style are the client's live values, since every screen draws in them and the Themes sheet previews over them. The client hands them in on the action that changes them, and the model says what to draw through [`Effect::ShowTheme`], [`Effect::ShowPips`] and their saved forms. Universes Beyond and its exceptions are the model's: they wait for the settings to load, and stay off for the visit if the load fails.
//!
//! Dark/Light, pips and Universes Beyond are optimistic toggles. Each tap shows its new value at once and takes a ticket from that toggle's own generation in [`Toggles`]; when the reply lands, [`settings::settle`] keeps the server's value, puts back the value from before that tap when the request failed, or ignores a reply a newer tap has overtaken. The two sheets save without the guard, and every one of their replies lands. Which sheet is up stays in the client: the model only says when to open or close one.

#[cfg(test)]
mod tests;

use std::mem;

use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_core::domain::user::settings::{PipColor, UserSettings};

use crate::filter::fields;
use crate::generation::{Generation, Ticket};
use crate::settings::{self, SettingChange, Toggle, Toggles};
use crate::toast::Toast;

/// The screen's title.
pub const TITLE: &str = "Settings";

/// The header of the list with the theme and pips.
pub const APPEARANCE: &str = "Appearance";

/// The header of the list with Universes Beyond and its exceptions.
pub const CARDS: &str = "Cards";

/// The Theme row's label.
pub const THEME: &str = "Theme";

/// The Pips row's label.
pub const PIPS: &str = "Pips";

/// The Universes Beyond row's label.
pub const UNIVERSES_BEYOND: &str = "Universes Beyond";

/// The Exceptions row's label, and the Exceptions sheet's title.
pub const EXCEPTIONS: &str = "Exceptions";

/// The button that opens the Themes sheet or the Exceptions sheet.
pub const CHANGE: &str = "Change";

/// The Universes Beyond button while they are hidden.
pub const HIDE: &str = "Hide";

/// The Universes Beyond button while they show.
pub const SHOW: &str = "Show";

/// The Exceptions sheet's button that discards the draft.
pub const BACK: &str = "Back";

/// The Exceptions sheet's button that saves the draft.
pub const SAVE: &str = "Save";

/// The Pips button's text for `style`.
#[must_use]
pub fn pips_label(style: PipColor) -> &'static str {
    match style {
        PipColor::Theme => "Theme",
        PipColor::Classic => "Classic",
    }
}

/// The line over the Exceptions sheet's chips, beside its clear button, for a draft of `count` franchises. The line folds away while the draft is empty, still reading "0 selected" as it goes.
#[must_use]
pub fn selected_line(count: usize) -> String {
    format!("{count} selected")
}

/// Where an action sends the user. The client maps each to its own route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Destination {
    /// The screen the user came from, from Back. The app reaches Config from Home's bar and goes back there through its history.
    Previous,
}

/// One of Config's two sheets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sheet {
    /// The Themes sheet, which previews a theme over the whole client.
    Themes,
    /// The Exceptions sheet, a chip picker over the model's draft.
    Exceptions,
}

/// A theme in one mode, as Config shows and sends it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Theme {
    /// The theme's slug.
    pub name: String,
    /// Whether it is the dark variant.
    pub dark_mode: bool,
}

/// Where the settings fetch stands. Only Universes Beyond and its exceptions wait on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// The fetch is out: the Universes Beyond buttons are off.
    Loading,
    /// The fetch failed: the Universes Beyond buttons stay off for the visit.
    Failed,
    /// The settings arrived: the Universes Beyond button is on, and Exceptions while they are hidden.
    Loaded,
}

/// One save in flight, as [`Effect::Save`] hands it out and [`Action::Saved`] or [`Action::SaveFailed`] hands it back. Compare nothing on it; the model does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SaveTicket {
    slot: Slot,
    ticket: Ticket,
}

/// Which control a save came from: a toggle, whose stale replies drop out, or a sheet, whose replies all land.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Slot {
    Toggle(Toggle),
    Themes,
    Exceptions,
}

/// What a user does on Config, and each reply the client hands back.
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    /// Offers the screen's hint and fetches the settings, as the screen does when it opens. What has loaded stays shown.
    Load,
    /// The settings arrived for the fetch holding `ticket`.
    Loaded {
        /// The ticket from the [`Effect::FetchSettings`] this answers.
        ticket: Ticket,
        /// The account's settings.
        settings: Box<UserSettings>,
    },
    /// The fetch holding `ticket` failed. The client has already reported it; the app raises no toast for it.
    Failed {
        /// The ticket from the [`Effect::FetchSettings`] this answers.
        ticket: Ticket,
    },
    /// Presses the Theme row's Dark/Light button.
    ToggleDarkMode {
        /// The theme the client draws in now.
        shown: Theme,
    },
    /// Presses the Theme row's Change.
    OpenThemes,
    /// Saves the theme the Themes sheet kept. The client has already closed the sheet and keeps drawing in the pick.
    KeepTheme(Theme),
    /// Closes the Themes sheet without saving. The client has already put back the theme it opened on.
    DiscardTheme {
        /// Whether a different theme was picked before the sheet closed.
        picked: bool,
    },
    /// Presses the Pips button.
    TogglePips {
        /// The pip style the client draws in now.
        shown: PipColor,
    },
    /// Presses the Universes Beyond button. Does nothing until the settings load.
    ToggleUniversesBeyond,
    /// Presses the Exceptions row's Change, seeding the draft from the saved list. Does nothing unless the settings loaded and Universes Beyond is hidden.
    OpenExceptions,
    /// Picks or unpicks the franchise with this slug in the draft.
    ToggleException(String),
    /// Unpicks every franchise in the draft.
    ClearExceptions,
    /// Presses the Exceptions sheet's Save: closes it and sends the draft.
    SaveExceptions,
    /// Presses the Exceptions sheet's Back or its backdrop: closes it and drops the draft.
    DiscardExceptions,
    /// The save holding `ticket` went through, and the server answered with `settings`.
    Saved {
        /// The ticket from the [`Effect::Save`] this answers.
        ticket: SaveTicket,
        /// The account's settings as the server holds them now.
        settings: Box<UserSettings>,
    },
    /// The save holding `ticket` failed. The client has already reported it and raised its error toast.
    SaveFailed {
        /// The ticket from the [`Effect::Save`] this answers.
        ticket: SaveTicket,
    },
    /// Goes back to the screen before.
    Back,
}

/// What an action asks the client to do, in the order it should do it.
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    /// Opens this one-time hint if the user hasn't seen it, and records it as seen.
    OfferHint(OneTimeHint),
    /// Fetches the settings and answers with [`Action::Loaded`] or [`Action::Failed`] carrying this ticket.
    FetchSettings(Ticket),
    /// Sends the change as its sparse update and answers with [`Action::Saved`] or [`Action::SaveFailed`] carrying this ticket.
    Save {
        /// The ticket to hand back.
        ticket: SaveTicket,
        /// What to send.
        change: SettingChange,
    },
    /// Draws the client in this theme: a Dark/Light tap's new mode, or the mode from before a tap whose save failed.
    ShowTheme(Theme),
    /// Draws pips in this style: a tap's new style, the loaded setting, or the style from before a tap whose save failed.
    ShowPips(PipColor),
    /// The server holds this theme: draw in it, resolved the way the client reads a theme from the account.
    ThemeSaved(Theme),
    /// The server holds this pip style: draw in it and keep it with the stored session, so the next launch opens with it.
    PipsSaved(PipColor),
    /// Opens this sheet.
    OpenSheet(Sheet),
    /// Closes this sheet.
    CloseSheet(Sheet),
    /// Goes to this screen.
    Navigate(Destination),
    /// Raises this toast.
    Toast(Toast),
}

/// The settings fetch as the last reply left it.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum Load {
    #[default]
    Loading,
    Failed,
    Loaded,
}

/// Whether the visit has offered a one-time hint yet.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
enum HintOffer {
    #[default]
    Waiting,
    Offered,
}

/// A save the server has not answered yet.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Waiting {
    ticket: SaveTicket,
    change: SettingChange,
    /// The same setting as it stood before the change, put back if a toggle's save fails. A sheet's save never puts anything back and keeps its own change here.
    undo: SettingChange,
}

/// Config for one visit. Starts loading with Universes Beyond shown and no exceptions; send [`Action::Load`] to start the fetch.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct State {
    load: Load,
    hide_universes_beyond: bool,
    exceptions: Vec<String>,
    /// The Exceptions sheet's picks: the open sheet's, or the ones it closed on.
    draft: Vec<String>,
    editing_exceptions: bool,
    hint: HintOffer,
    exceptions_hint: HintOffer,
    generation: Generation,
    toggles: Toggles,
    themes: Generation,
    exception_saves: Generation,
    waiting: Vec<Waiting>,
}

impl State {
    /// The screen as it first draws: loading, Universes Beyond shown, no exceptions.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies `action` and returns what the client must do, in order.
    ///
    /// A fetch reply whose ticket a newer [`Action::Load`] has replaced changes nothing, and so does a toggle's save reply that a newer tap on the same toggle has overtaken. Config's hint is offered once per visit as the screen opens; the Exceptions hint once per visit, the first time that sheet opens.
    #[must_use = "the client performs the effects"]
    pub fn update(&mut self, action: Action) -> Vec<Effect> {
        match action {
            Action::Load => self.load(),
            Action::Loaded { ticket, settings } => self.loaded(ticket, *settings),
            Action::Failed { ticket } => {
                if self.generation.is_current(ticket) && self.load == Load::Loading {
                    self.load = Load::Failed;
                }
                Vec::new()
            }
            Action::ToggleDarkMode { shown } => self.toggle_dark_mode(shown),
            Action::OpenThemes => vec![Effect::OpenSheet(Sheet::Themes)],
            Action::KeepTheme(theme) => {
                let change = SettingChange::Theme {
                    theme: theme.name,
                    dark_mode: theme.dark_mode,
                };
                vec![
                    Effect::CloseSheet(Sheet::Themes),
                    self.send(Slot::Themes, change.clone(), change),
                ]
            }
            Action::DiscardTheme { picked } => {
                let mut effects = vec![Effect::CloseSheet(Sheet::Themes)];
                if picked {
                    effects.push(Effect::Toast(settings::theme_unchanged()));
                }
                effects
            }
            Action::TogglePips { shown } => {
                let next = shown.toggled();
                vec![
                    Effect::ShowPips(next),
                    self.send(
                        Slot::Toggle(Toggle::Pips),
                        SettingChange::Pips(next),
                        SettingChange::Pips(shown),
                    ),
                ]
            }
            Action::ToggleUniversesBeyond => self.toggle_universes_beyond(),
            Action::OpenExceptions => self.open_exceptions(),
            Action::ToggleException(slug) => {
                if self.editing_exceptions {
                    self.draft = fields::toggled(mem::take(&mut self.draft), &slug);
                }
                Vec::new()
            }
            Action::ClearExceptions => {
                if self.editing_exceptions {
                    self.draft.clear();
                }
                Vec::new()
            }
            Action::SaveExceptions => self.save_exceptions(),
            Action::DiscardExceptions => self.discard_exceptions(),
            Action::Saved { ticket, settings } => self.settle(ticket, Some(&settings)),
            Action::SaveFailed { ticket } => self.settle(ticket, None),
            Action::Back => vec![Effect::Navigate(Destination::Previous)],
        }
    }

    /// Where the settings fetch stands.
    #[must_use]
    pub fn status(&self) -> Status {
        match self.load {
            Load::Loading => Status::Loading,
            Load::Failed => Status::Failed,
            Load::Loaded => Status::Loaded,
        }
    }

    /// Whether the Universes Beyond button can be pressed: once the settings loaded.
    #[must_use]
    pub fn universes_beyond_enabled(&self) -> bool {
        self.load == Load::Loaded
    }

    /// Whether Universes Beyond cards are hidden, as the button shows it.
    #[must_use]
    pub fn hides_universes_beyond(&self) -> bool {
        self.hide_universes_beyond
    }

    /// The Universes Beyond button's text: [`HIDE`] while they are hidden, else [`SHOW`].
    #[must_use]
    pub fn universes_beyond_label(&self) -> &'static str {
        if self.hide_universes_beyond {
            HIDE
        } else {
            SHOW
        }
    }

    /// Whether the Exceptions row's Change can be pressed: once the settings loaded, while Universes Beyond is hidden.
    #[must_use]
    pub fn exceptions_enabled(&self) -> bool {
        self.universes_beyond_enabled() && self.hide_universes_beyond
    }

    /// Whether the Exceptions row and its chips draw muted: while Universes Beyond shows, since the exceptions do nothing then.
    #[must_use]
    pub fn exceptions_muted(&self) -> bool {
        !self.hide_universes_beyond
    }

    /// The saved exceptions' slugs, in the order the server keeps them.
    #[must_use]
    pub fn exceptions(&self) -> &[String] {
        &self.exceptions
    }

    /// Whether the Exceptions row draws its chip row: whenever any exception is saved, even one this build cannot name.
    #[must_use]
    pub fn shows_exception_chips(&self) -> bool {
        !self.exceptions.is_empty()
    }

    /// The Exceptions row's chips: the saved franchises by name, alphabetical, unknown slugs dropped.
    #[must_use]
    pub fn exception_names(&self) -> Vec<&'static str> {
        settings::exception_names(&self.exceptions)
    }

    /// Whether the Exceptions sheet is open, so its draft takes edits.
    #[must_use]
    pub fn editing_exceptions(&self) -> bool {
        self.editing_exceptions
    }

    /// The Exceptions sheet's picks in the order they were picked. After the sheet closes they stay as it closed on them, so a sheet sliding away still draws them; the next open starts over from the saved list.
    #[must_use]
    pub fn draft(&self) -> &[String] {
        &self.draft
    }

    /// Whether the draft picks the franchise with this slug.
    #[must_use]
    pub fn is_drafted(&self, slug: &str) -> bool {
        self.draft.iter().any(|picked| picked == slug)
    }

    /// Whether the sheet's Save is off: the draft picks the saved set, in any order.
    #[must_use]
    pub fn draft_unchanged(&self) -> bool {
        fields::same_set(&self.draft, &self.exceptions)
    }

    /// Starts a fetch, offering the hint the first time.
    fn load(&mut self) -> Vec<Effect> {
        if self.load == Load::Failed {
            self.load = Load::Loading;
        }
        let mut effects = Vec::new();
        if self.hint == HintOffer::Waiting {
            self.hint = HintOffer::Offered;
            effects.push(Effect::OfferHint(OneTimeHint::Config));
        }
        effects.push(Effect::FetchSettings(self.generation.next_ticket()));
        effects
    }

    /// Lands the settings fetched under `ticket`, unless a newer fetch has replaced it.
    fn loaded(&mut self, ticket: Ticket, settings: UserSettings) -> Vec<Effect> {
        if !self.generation.is_current(ticket) {
            return Vec::new();
        }
        self.load = Load::Loaded;
        self.hide_universes_beyond = settings.exclude_universes_beyond;
        self.exceptions = settings.universes_beyond_exceptions;
        vec![Effect::ShowPips(settings.pip_style)]
    }

    /// Shows `shown` in the other mode at once and saves it.
    fn toggle_dark_mode(&mut self, shown: Theme) -> Vec<Effect> {
        let next = Theme {
            name: shown.name.clone(),
            dark_mode: !shown.dark_mode,
        };
        let change = SettingChange::DarkMode {
            theme: next.name.clone(),
            dark_mode: next.dark_mode,
        };
        let undo = SettingChange::DarkMode {
            theme: shown.name,
            dark_mode: shown.dark_mode,
        };
        vec![
            Effect::ShowTheme(next),
            self.send(Slot::Toggle(Toggle::DarkMode), change, undo),
        ]
    }

    /// Flips Universes Beyond at once and saves it, once the settings loaded.
    fn toggle_universes_beyond(&mut self) -> Vec<Effect> {
        if !self.universes_beyond_enabled() {
            return Vec::new();
        }
        let shown = self.hide_universes_beyond;
        self.hide_universes_beyond = !shown;
        vec![self.send(
            Slot::Toggle(Toggle::UniversesBeyond),
            SettingChange::HideUniversesBeyond(!shown),
            SettingChange::HideUniversesBeyond(shown),
        )]
    }

    /// Closes the open Exceptions sheet and sends its draft.
    fn save_exceptions(&mut self) -> Vec<Effect> {
        if !mem::replace(&mut self.editing_exceptions, false) {
            return Vec::new();
        }
        let change = SettingChange::Exceptions(self.draft.clone());
        vec![
            Effect::CloseSheet(Sheet::Exceptions),
            self.send(Slot::Exceptions, change.clone(), change),
        ]
    }

    /// Closes the open Exceptions sheet, saying so first when its draft differed from the saved list.
    fn discard_exceptions(&mut self) -> Vec<Effect> {
        if !mem::replace(&mut self.editing_exceptions, false) {
            return Vec::new();
        }
        let mut effects = Vec::new();
        if !self.draft_unchanged() {
            effects.push(Effect::Toast(settings::exceptions_unchanged()));
        }
        effects.push(Effect::CloseSheet(Sheet::Exceptions));
        effects
    }

    /// Opens the Exceptions sheet on the saved list, offering its hint the first time.
    fn open_exceptions(&mut self) -> Vec<Effect> {
        if !self.exceptions_enabled() {
            return Vec::new();
        }
        self.draft.clone_from(&self.exceptions);
        self.editing_exceptions = true;
        let mut effects = vec![Effect::OpenSheet(Sheet::Exceptions)];
        if self.exceptions_hint == HintOffer::Waiting {
            self.exceptions_hint = HintOffer::Offered;
            effects.push(Effect::OfferHint(OneTimeHint::UniversesBeyondExceptions));
        }
        effects
    }

    /// Takes a ticket for a save from `slot`, keeps the change and its undo for the reply, and returns the effect that sends it.
    fn send(&mut self, slot: Slot, change: SettingChange, undo: SettingChange) -> Effect {
        let ticket = SaveTicket {
            slot,
            ticket: match slot {
                Slot::Toggle(toggle) => self.toggles.next_ticket(toggle),
                Slot::Themes => self.themes.next_ticket(),
                Slot::Exceptions => self.exception_saves.next_ticket(),
            },
        };
        self.waiting.push(Waiting {
            ticket,
            change: change.clone(),
            undo,
        });
        Effect::Save { ticket, change }
    }

    /// Lands the reply to the save holding `ticket`: `saved` is the server's settings, or `None` when the save failed.
    fn settle(&mut self, ticket: SaveTicket, saved: Option<&UserSettings>) -> Vec<Effect> {
        let Some(at) = self.waiting.iter().position(|w| w.ticket == ticket) else {
            return Vec::new();
        };
        let waiting = self.waiting.remove(at);
        // `undo` read from the reply is the change as the server holds it.
        let held = saved.map(|settings| waiting.change.undo(settings));
        let shown = match ticket.slot {
            Slot::Toggle(toggle) => {
                let generation = self.toggles.generation(toggle);
                settings::settle(generation, ticket.ticket, held, waiting.undo)
            }
            // A sheet's save changes nothing on failure, and every reply lands.
            Slot::Themes | Slot::Exceptions => held,
        };
        let Some(shown) = shown else {
            return Vec::new();
        };
        let mut effects: Vec<Effect> = self
            .show(shown, /* saved */ saved.is_some())
            .into_iter()
            .collect();
        if let Some(settings) = saved {
            effects.push(Effect::Toast(waiting.change.saved_toast(settings)));
        }
        effects
    }

    /// Puts `change` on screen: the model's own settings change here, the client's come back as an effect. `saved` says the server holds it.
    fn show(&mut self, change: SettingChange, saved: bool) -> Option<Effect> {
        match change {
            SettingChange::Theme { theme, dark_mode }
            | SettingChange::DarkMode { theme, dark_mode } => {
                let theme = Theme {
                    name: theme,
                    dark_mode,
                };
                Some(if saved {
                    Effect::ThemeSaved(theme)
                } else {
                    Effect::ShowTheme(theme)
                })
            }
            SettingChange::Pips(style) => Some(if saved {
                Effect::PipsSaved(style)
            } else {
                Effect::ShowPips(style)
            }),
            SettingChange::HideUniversesBeyond(hide) => {
                self.hide_universes_beyond = hide;
                None
            }
            SettingChange::Exceptions(slugs) => {
                self.exceptions = slugs;
                None
            }
        }
    }
}
