//! Create deck and Edit deck: the shared `zwipe_ui::screens::deck_form` model, drawn with the terminal's own text boxes, focus, picker search boxes and swipe picker.
//!
//! The model holds every value, the name check, the knock-on clears, the command-zone searches, the pickers' drafts and the save; this screen keeps the boxes in step with it, moves the focus, and performs its effects through [`dispatch`].

pub(crate) mod draw;
pub(crate) mod keys;
pub(crate) mod pickers;
pub(crate) mod swipe_pick;
#[cfg(test)]
mod tests;

use std::time::Duration;

use serde::Serialize;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::deck::{Deck, DeckProfile, DeckTagView};
use zwipe_core::http::contracts::deck::{HttpCreateDeckProfile, HttpUpdateDeckProfile};
use zwipe_ui::deck::form::Slot;
use zwipe_ui::deck::pickers::TagPicker;
use zwipe_ui::generation::Ticket;
use zwipe_ui::hint::HintTopic;
use zwipe_ui::screens::deck_form::{Action, Destination, Effect, Mode, Open, Results, State};

use self::pickers::{FormatPicker, TagGrid};
use self::swipe_pick::SwipePicker;
use super::{Screen, card_sheets};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::hint;
use crate::inbound::widgets::text_input::TextInput;
use crate::inbound::widgets::toast::{self, ToastKind};
use crate::outbound::account::Account;

/// One command-zone field's box, the result its arrows highlight, and the filter its swipe picker applied last.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CardInput {
    /// The chosen card's name, or the text typed, as the model shows it.
    pub(crate) input: TextInput,
    /// The highlighted result, back at the top whenever the results change.
    pub(crate) highlighted: usize,
    /// The filter the swipe picker applied last, kept while the picker is closed.
    pub(crate) swipe_filter: CardQueryBuilder,
}

impl Default for CardInput {
    fn default() -> Self {
        Self {
            input: TextInput::default(),
            highlighted: 0,
            swipe_filter: CardQueryBuilder::new(),
        }
    }
}

/// The four command-zone boxes, one per [`Slot`].
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct CardInputs {
    commander: CardInput,
    partner: CardInput,
    background: CardInput,
    signature_spell: CardInput,
}

impl CardInputs {
    pub(crate) fn get(&self, slot: Slot) -> &CardInput {
        match slot {
            Slot::Commander => &self.commander,
            Slot::Partner => &self.partner,
            Slot::Background => &self.background,
            Slot::SignatureSpell => &self.signature_spell,
        }
    }

    pub(crate) fn get_mut(&mut self, slot: Slot) -> &mut CardInput {
        match slot {
            Slot::Commander => &mut self.commander,
            Slot::Partner => &mut self.partner,
            Slot::Background => &mut self.background,
            Slot::SignatureSpell => &mut self.signature_spell,
        }
    }
}

/// A focusable field, top to bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Field {
    Name,
    Format,
    Card(Slot),
    Power,
    Land,
    Currency,
    Price,
    DeckTags,
    OracleTags,
    OtherTags,
}

impl Field {
    /// Whether the field is a text box, where letters type instead of acting as keys.
    pub(crate) fn takes_typing(self) -> bool {
        matches!(self, Self::Name | Self::Price | Self::Card(_))
    }

    /// The field's explainer, which its `[?]` marker stands for.
    pub(crate) fn topic(self) -> HintTopic {
        match self {
            Self::Name => HintTopic::DeckName,
            Self::Format => HintTopic::Format,
            Self::Card(Slot::Commander) => HintTopic::Commander,
            Self::Card(Slot::Partner) => HintTopic::Partner,
            Self::Card(Slot::Background) => HintTopic::Background,
            Self::Card(Slot::SignatureSpell) => HintTopic::SignatureSpell,
            Self::Power => HintTopic::PowerLevel,
            Self::Land => HintTopic::LandTarget,
            Self::Currency | Self::Price => HintTopic::PriceTarget,
            Self::DeckTags => HintTopic::DeckTags,
            Self::OracleTags => HintTopic::OracleTags,
            Self::OtherTags => HintTopic::OtherTags,
        }
    }
}

/// A full-screen picker over the form. Which picker the model has open says what it edits; the overlay keeps only its search box, focus and definition bar.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Overlay {
    Format(FormatPicker),
    Tags(TagPicker, TagGrid),
    Swipe(Box<SwipePicker>),
}

/// The Create and Edit form: the model, the boxes that show its text, which field has the focus, and the overlay open over it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DeckForm {
    pub(crate) model: State,
    pub(crate) name: TextInput,
    pub(crate) price: TextInput,
    pub(crate) inputs: CardInputs,
    pub(crate) focus: Field,
    pub(crate) power_focus: usize,
    pub(crate) tag_focus: usize,
    pub(crate) oracle_focus: usize,
    pub(crate) other_focus: usize,
    /// The deck tag catalog the deck tag picker lists; the model holds its own copy for the oracle tag seed.
    pub(crate) deck_tag_catalog: Vec<DeckTagView>,
    /// The oracle tag catalog, `None` until it loads.
    pub(crate) oracle_catalog: Option<Vec<OracleTag>>,
    pub(crate) overlay: Option<Overlay>,
}

impl Default for DeckForm {
    fn default() -> Self {
        Self {
            model: State::new(),
            name: TextInput::default(),
            price: TextInput::default(),
            inputs: CardInputs::default(),
            focus: Field::Name,
            power_focus: 0,
            tag_focus: 0,
            oracle_focus: 0,
            other_focus: 0,
            deck_tag_catalog: Vec::new(),
            oracle_catalog: None,
            overlay: None,
        }
    }
}

impl DeckForm {
    /// Comes back into view: takes the tag the oracle tag dictionary's Use left, into the swipe picker's filter sheet or the oracle tag picker, whichever opened it.
    pub(crate) fn on_return(&mut self, cx: &mut Context<'_>) {
        if let Some(Overlay::Swipe(picker)) = &mut self.overlay
            && card_sheets::filter::returned_from_dictionary(picker.sheet.as_mut(), cx)
        {
            return;
        }
        let Some(slug) = cx.shared.caches.handoff.oracle_tag.take() else {
            return;
        };
        if matches!(self.overlay, Some(Overlay::Tags(TagPicker::OracleTags, _))) {
            dispatch(self, Action::AdoptOracleTag(slug), cx);
        }
    }

    /// The fields that show, top to bottom.
    pub(crate) fn fields(&self) -> Vec<Field> {
        let mut fields = vec![Field::Name, Field::Format];
        fields.extend(
            Slot::ALL
                .into_iter()
                .filter(|s| self.model.shows(*s))
                .map(Field::Card),
        );
        fields.extend([
            Field::Power,
            Field::Land,
            Field::Currency,
            Field::Price,
            Field::DeckTags,
            Field::OracleTags,
            Field::OtherTags,
        ]);
        fields
    }

    /// Moves the focus to the next or the previous field, stopping at the ends.
    pub(crate) fn step_focus(&mut self, toward: Toward) {
        let fields = self.fields();
        let here = fields.iter().position(|f| *f == self.focus).unwrap_or(0);
        let next = toward.clamped(here, fields.len());
        if let Some(field) = fields.get(next) {
            self.focus = *field;
        }
    }

    /// Writes the model's text into the boxes that show it, and keeps the focus on a field that shows. `before` is each slot's results before the action, so a change puts the highlight back at the top.
    fn sync(&mut self, before: &[Option<Results>]) {
        let values = self.model.values();
        if self.name.value() != values.name {
            self.name.set_value(&values.name);
        }
        self.name
            .set_error(self.model.name_error().map(str::to_owned));
        if self.price.value() != values.price_text {
            self.price.set_value(&values.price_text);
        }
        for (slot, was) in Slot::ALL.into_iter().zip(before) {
            let shown = self.model.display(slot);
            let field = self.inputs.get_mut(slot);
            if field.input.value() != shown {
                field.input.set_value(shown);
            }
            if self.model.results(slot) != *was {
                field.highlighted = 0;
            }
        }
        if !self.fields().contains(&self.focus) {
            self.focus = Field::Format;
        }
    }
}

/// Applies `action` to the model and performs what it returns, then brings the boxes and the focus in step.
pub(crate) fn dispatch(form: &mut DeckForm, action: Action, cx: &mut Context<'_>) {
    run(form, action, cx, /* on_top */ true);
}

/// [`dispatch`] for a reply, which may land while another screen covers the form; navigation then waits for nobody, as the app's task dies with its screen.
fn run(form: &mut DeckForm, action: Action, cx: &mut Context<'_>, on_top: bool) {
    let before: Vec<Option<Results>> = Slot::ALL.map(|s| form.model.results(s)).to_vec();
    for effect in form.model.update(action) {
        match effect {
            Effect::OfferHint(offer) => {
                hint::once(cx, offer.key());
            }
            Effect::FetchDeck { ticket, deck_id } => {
                cx.request(Request::LoadDeck { ticket, deck_id }.into());
            }
            Effect::FetchCard {
                ticket,
                slot,
                card_id,
            } => cx.request(
                Request::FetchCard {
                    ticket,
                    slot,
                    card_id,
                }
                .into(),
            ),
            Effect::Debounce { ticket, wait } => {
                cx.request(Request::Debounce { ticket, wait }.into());
            }
            Effect::SearchCards { ticket, query } => {
                cx.request(Request::Search { ticket, query }.into());
            }
            Effect::FindPartner {
                picked,
                mate,
                query,
            } => cx.request(
                Request::FindPartner {
                    picked,
                    mate,
                    query,
                }
                .into(),
            ),
            Effect::CreateDeck { ticket, request } => {
                cx.request(
                    Request::Create {
                        ticket,
                        body: Body(request),
                    }
                    .into(),
                );
            }
            Effect::UpdateDeck {
                ticket,
                deck_id,
                request,
            } => cx.request(
                Request::Update {
                    ticket,
                    deck_id,
                    body: Body(request),
                }
                .into(),
            ),
            Effect::Navigate(_) if !on_top => {}
            Effect::Navigate(Destination::Previous) => cx.back(),
            // Edit opens over the deck view, so going to the deck is going back; Create's new deck takes the form's place.
            Effect::Navigate(Destination::Deck(deck_id)) => match form.model.mode() {
                Mode::Create => screens::deck::replace_with(cx, deck_id),
                Mode::Edit(_) => cx.back(),
            },
            Effect::Toast(toast) => cx.show(toast),
        }
    }
    form.sync(&before);
}

/// Shows the form's hint again.
pub(crate) fn reopen_hint(cx: &mut Context<'_>, form: &DeckForm) {
    hint::reopen(cx, form.model.mode().hint().key());
}

/// A request body the server takes, compared by the JSON it sends, since the contracts have no `PartialEq`.
#[derive(Debug, Clone)]
pub(crate) struct Body<T>(pub(crate) Box<T>);

impl<T: Serialize> PartialEq for Body<T> {
    fn eq(&self, other: &Self) -> bool {
        serde_json::to_value(&*self.0).ok() == serde_json::to_value(&*other.0).ok()
    }
}

/// The form's work, each of the model's requests carrying its ticket.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Request {
    DeckTagCatalog,
    OracleCatalog,
    /// Waits out the typing pause before a command-zone search.
    Debounce {
        ticket: Ticket,
        wait: Duration,
    },
    /// A command-zone search, quiet on failure.
    Search {
        ticket: Ticket,
        query: Box<CardQuery>,
    },
    /// Looks for the partner a "Partner with" commander names; a failure sends nothing back.
    FindPartner {
        picked: Uuid,
        mate: String,
        query: Box<CardQuery>,
    },
    LoadDeck {
        ticket: Ticket,
        deck_id: Uuid,
    },
    FetchCard {
        ticket: Ticket,
        slot: Slot,
        card_id: Uuid,
    },
    Create {
        ticket: Ticket,
        body: Body<HttpCreateDeckProfile>,
    },
    Update {
        ticket: Ticket,
        deck_id: Uuid,
        body: Body<HttpUpdateDeckProfile>,
    },
    /// A page of the swipe picker's pile.
    SwipePage {
        generation: Ticket,
        query: Box<CardQuery>,
    },
    MaybeboardAdd(Uuid),
    MaybeboardRemove(Uuid),
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::DeckTagCatalog => "deck_tag_catalog",
            Self::OracleCatalog => "oracle_catalog",
            Self::Debounce { .. } => "debounce",
            Self::Search { .. } => "search",
            Self::FindPartner { .. } => "find_partner",
            Self::LoadDeck { .. } => "load_deck",
            Self::FetchCard { .. } => "fetch_card",
            Self::Create { .. } => "create",
            Self::Update { .. } => "update",
            Self::SwipePage { .. } => "swipe_page",
            Self::MaybeboardAdd(..) => "maybeboard_add",
            Self::MaybeboardRemove(..) => "maybeboard_remove",
        }
    }
}

/// The form's results, the model's carrying their tickets back.
#[derive(Debug)]
pub(crate) enum Update {
    DeckTagCatalog(Vec<DeckTagView>),
    OracleCatalog(Vec<OracleTag>),
    Debounced {
        ticket: Ticket,
    },
    Searched {
        ticket: Ticket,
        cards: Vec<Card>,
    },
    PartnerSearched {
        picked: Uuid,
        mate: String,
        cards: Vec<Card>,
    },
    Loaded {
        ticket: Ticket,
        deck: Box<Deck>,
    },
    CardLoaded {
        ticket: Ticket,
        card: Box<Card>,
    },
    Created {
        ticket: Ticket,
        profile: Box<DeckProfile>,
    },
    Updated {
        ticket: Ticket,
    },
    /// One of the model's requests failed: `Some` is the error toast to raise first, `None` a quiet search.
    Failed {
        ticket: Ticket,
        message: Option<String>,
    },
    SwipePage {
        generation: Ticket,
        cards: Vec<Card>,
    },
    MaybeboardAdded,
    MaybeboardRemoved,
    /// A toast for a failure with nothing else to show.
    Error(String),
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::DeckForm(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::DeckForm(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut DeckForm> {
    match screen {
        Screen::DeckForm(form) => Some(form),
        _ => None,
    }
}

/// Opens Create with an empty form.
pub(crate) fn open_create(cx: &mut Context<'_>) {
    open_with(cx, Open::Create { seed: None });
}

/// Opens Create seeded with a commander, as Playstyle, the commander swipe and the Commander maybeboard's "Create deck" do: the format is Commander and the card is the commander.
pub(crate) fn open_create_with_commander(cx: &mut Context<'_>, commander: Card) {
    open_with(
        cx,
        Open::Create {
            seed: Some(Box::new(commander)),
        },
    );
}

/// Opens Edit for `deck_id` and loads the deck.
pub(crate) fn open_edit(cx: &mut Context<'_>, deck_id: Uuid) {
    open_with(cx, Open::Edit(deck_id));
}

fn open_with(cx: &mut Context<'_>, open: Open) {
    let mut form = DeckForm::default();
    cx.request(Request::DeckTagCatalog.into());
    cx.request(Request::OracleCatalog.into());
    dispatch(&mut form, Action::Load(open), cx);
    cx.push(Screen::DeckForm(Box::new(form)));
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    let error = |e: &zwipe_client::ClientError| screens::failure(e, |m| Update::Error(m).into());
    match request {
        Request::DeckTagCatalog => {
            match async { client.fetch_deck_tags(&account.fresh().await?).await }.await {
                Ok(tags) => Some(Update::DeckTagCatalog(tags).into()),
                Err(e) => Some(error(&e)),
            }
        }
        Request::OracleCatalog => match client.fetch_oracle_tags().await {
            Ok(tags) => Some(Update::OracleCatalog(tags).into()),
            Err(e) => Some(error(&e)),
        },
        Request::Debounce { ticket, wait } => {
            tokio::time::sleep(wait).await;
            Some(Update::Debounced { ticket }.into())
        }
        Request::Search { ticket, query } => Some(
            match async { client.search_cards(&query, &account.fresh().await?).await }.await {
                Ok(cards) => Update::Searched { ticket, cards }.into(),
                // A failed search only closes the results, as in the app.
                Err(_) => Update::Failed {
                    ticket,
                    message: None,
                }
                .into(),
            },
        ),
        Request::FindPartner {
            picked,
            mate,
            query,
        } => {
            let cards = async { client.search_cards(&query, &account.fresh().await?).await }
                .await
                .ok()?;
            Some(
                Update::PartnerSearched {
                    picked,
                    mate,
                    cards,
                }
                .into(),
            )
        }
        Request::LoadDeck { ticket, deck_id } => Some(
            match async { client.fetch_deck(deck_id, &account.fresh().await?).await }.await {
                Ok(deck) => Update::Loaded {
                    ticket,
                    deck: Box::new(deck),
                }
                .into(),
                Err(e) => failed(ticket, &e),
            },
        ),
        Request::FetchCard {
            ticket, card_id, ..
        } => Some(match client.fetch_card(card_id).await {
            Ok(card) => Update::CardLoaded {
                ticket,
                card: Box::new(card),
            }
            .into(),
            Err(e) => failed(ticket, &e),
        }),
        Request::Create { ticket, body } => Some(create(account, ticket, &body).await),
        Request::Update {
            ticket,
            deck_id,
            body,
        } => Some(save(account, ticket, deck_id, &body).await),
        Request::SwipePage { generation, query } => Some(
            match async {
                client
                    .search_commanders(&query, &account.fresh().await?)
                    .await
            }
            .await
            {
                Ok(cards) => Update::SwipePage { generation, cards }.into(),
                Err(e) => error(&e),
            },
        ),
        Request::MaybeboardAdd(oracle_id) => Some(maybeboard_add(account, oracle_id).await),
        Request::MaybeboardRemove(oracle_id) => Some(maybeboard_remove(account, oracle_id).await),
    }
}

/// Creates the new deck's profile.
async fn create(
    account: &Account,
    ticket: Ticket,
    body: &Body<HttpCreateDeckProfile>,
) -> screens::Update {
    match async {
        account
            .client
            .create_deck_profile(&body.0, &account.fresh().await?)
            .await
    }
    .await
    {
        Ok(profile) => Update::Created {
            ticket,
            profile: Box::new(profile),
        }
        .into(),
        Err(e) => failed(ticket, &e),
    }
}

/// Saves the edited deck's changed fields.
async fn save(
    account: &Account,
    ticket: Ticket,
    deck_id: Uuid,
    body: &Body<HttpUpdateDeckProfile>,
) -> screens::Update {
    match async {
        account
            .client
            .update_deck_profile(deck_id, &body.0, &account.fresh().await?)
            .await
    }
    .await
    {
        Ok(_) => Update::Updated { ticket }.into(),
        Err(e) => failed(ticket, &e),
    }
}

/// A model request's failure, raising its error toast, or the session's end.
fn failed(ticket: Ticket, error: &zwipe_client::ClientError) -> screens::Update {
    screens::failure(error, |m| {
        Update::Failed {
            ticket,
            message: Some(m),
        }
        .into()
    })
}

/// Adds a commander to the user's commander maybeboard.
async fn maybeboard_add(account: &Account, oracle_id: Uuid) -> screens::Update {
    match async {
        account
            .client
            .add_commander_maybeboard_card(oracle_id, &account.fresh().await?)
            .await
    }
    .await
    {
        Ok(()) => Update::MaybeboardAdded.into(),
        Err(e) => screens::failure(&e, |m| Update::Error(m).into()),
    }
}

/// Takes a commander off the user's commander maybeboard.
async fn maybeboard_remove(account: &Account, oracle_id: Uuid) -> screens::Update {
    match async {
        account
            .client
            .remove_commander_maybeboard_card(oracle_id, &account.fresh().await?)
            .await
    }
    .await
    {
        Ok(()) => Update::MaybeboardRemoved.into(),
        Err(e) => screens::failure(&e, |m| Update::Error(m).into()),
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let on_top = matches!(nav.top(), Screen::DeckForm(_));
    // An error toast goes up before the lookup, so it shows even when the form has closed.
    if let Update::Error(message)
    | Update::Failed {
        message: Some(message),
        ..
    } = &update
    {
        cx.toast(ToastKind::Error, message.clone());
    }
    let Some(form) = nav.find_mut(pick) else {
        return;
    };
    let action = match update {
        Update::DeckTagCatalog(tags) => {
            form.deck_tag_catalog.clone_from(&tags);
            Action::Catalog(tags)
        }
        Update::OracleCatalog(tags) => {
            form.oracle_catalog = Some(tags);
            return;
        }
        Update::Debounced { ticket } => Action::Debounced { ticket },
        Update::Searched { ticket, cards } => Action::Searched { ticket, cards },
        Update::PartnerSearched {
            picked,
            mate,
            cards,
        } => Action::PartnerSearched {
            picked,
            mate,
            cards,
        },
        Update::Loaded { ticket, deck } => Action::Loaded {
            ticket,
            profile: Box::new(deck.deck_profile),
        },
        Update::CardLoaded { ticket, card } => Action::CardLoaded { ticket, card },
        Update::Created { ticket, profile } => Action::Created {
            ticket,
            deck_id: profile.id,
        },
        Update::Updated { ticket } => Action::Updated { ticket },
        Update::Failed { ticket, .. } => Action::Failed { ticket },
        Update::SwipePage { generation, cards } => {
            if let Some(Overlay::Swipe(picker)) = &mut form.overlay {
                picker.take_page(generation, cards, cx);
            }
            return;
        }
        Update::MaybeboardAdded => {
            cx.toast(
                ToastKind::Success,
                "Added to your commander maybeboard".to_owned(),
            );
            return;
        }
        Update::MaybeboardRemoved => {
            cx.toast_for(
                ToastKind::Info,
                "Removed from your commander maybeboard".to_owned(),
                toast::QUICK,
            );
            return;
        }
        Update::Error(_) => return,
    };
    run(form, action, cx, on_top);
}
