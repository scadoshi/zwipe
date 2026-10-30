//! Swiping cards into one deck: the stack, the images around the card on screen, and the swipes sent to the server.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;

use image::DynamicImage;
use ratatui_image::protocol::StatefulProtocol;
use uuid::Uuid;
use zwipe_client::ClientError;
use zwipe_core::domain::card::scryfall_data::ImageSize;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::deck::{Deck, DeckProfile};
use zwipe_core::http::contracts::deck::HttpDeckCardSearch;
use zwipe_core::http::contracts::deck_card::HttpCreateDeckCard;

use super::{Screen, failure};
use crate::domain::swipe::{self, Effect, Stack};
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::outbound::account::Account;
use crate::outbound::images;

/// Images kept for cards already swiped, so an undo shows its image at once.
const IMAGES_BEHIND: usize = 1;

/// Images fetched ahead of the card on screen.
const IMAGES_AHEAD: usize = 3;

/// Swiping cards into one deck.
#[derive(Debug)]
pub(crate) struct SwipeView {
    pub(crate) deck: DeckProfile,
    /// `None` until the deck has loaded and the query is known.
    pub(crate) stack: Option<Stack>,
    pub(crate) query: Option<CardQuery>,
    /// Set while a page of cards is being fetched.
    pub(crate) fetching: bool,
    /// Copies added to the main board here, less the ones undone.
    pub(crate) added: i64,
    pub(crate) error: Option<String>,
    /// Images for the cards around the one on screen, by printing ID.
    pub(crate) images: HashMap<Uuid, CardImage>,
    /// On a narrow terminal, whether the card shows as its image instead of its text.
    pub(crate) show_image: bool,
}

/// A card's image as it loads.
pub(crate) enum CardImage {
    Loading,
    /// Encoded for this terminal on first draw, and again when its area changes.
    Ready(Box<RefCell<StatefulProtocol>>),
    Failed,
}

impl fmt::Debug for CardImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Loading => "Loading",
            Self::Ready(_) => "Ready",
            Self::Failed => "Failed",
        })
    }
}

impl SwipeView {
    pub(crate) fn opening(deck: DeckProfile) -> Self {
        Self {
            deck,
            stack: None,
            query: None,
            fetching: false,
            added: 0,
            error: None,
            images: HashMap::new(),
            show_image: false,
        }
    }

    /// Fetches the next page when the stack is running low and no fetch is out.
    pub(crate) fn refill(&mut self, cx: &mut Context<'_>) {
        let (Some(stack), Some(query)) = (&self.stack, &self.query) else {
            return;
        };
        if !self.fetching && stack.needs_more() {
            self.fetching = true;
            cx.request(
                Request::FetchCards {
                    deck_id: self.deck.id,
                    query: Box::new(query.clone()),
                }
                .into(),
            );
        }
    }

    /// Asks for the images of the cards around the one on screen, and drops the images of cards further away. Does nothing when card images are off.
    pub(crate) fn sync_images(&mut self, cx: &mut Context<'_>) {
        if cx.shared.images.is_none() {
            return;
        }
        let Some(stack) = &self.stack else {
            return;
        };
        let window = stack.around(IMAGES_BEHIND, IMAGES_AHEAD);
        let wanted: HashSet<Uuid> = window.iter().map(|c| c.scryfall_data.id).collect();
        self.images.retain(|id, _| wanted.contains(id));
        for card in window {
            let card_id = card.scryfall_data.id;
            if self.images.contains_key(&card_id) {
                continue;
            }
            match card.scryfall_data.primary_image_url(ImageSize::Normal) {
                Some(url) => {
                    self.images.insert(card_id, CardImage::Loading);
                    cx.request(
                        Request::Image {
                            card_id,
                            url: url.to_owned(),
                        }
                        .into(),
                    );
                }
                None => {
                    self.images.insert(card_id, CardImage::Failed);
                }
            }
        }
    }
}

/// The swipe screen's work.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Request {
    FetchCards {
        deck_id: Uuid,
        query: Box<CardQuery>,
    },
    /// Sends a swipe or an undo.
    Send { deck_id: Uuid, effect: Effect },
    /// Downloads one card's image.
    Image { card_id: Uuid, url: String },
}

/// The swipe screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    Cards {
        deck_id: Uuid,
        page: HttpDeckCardSearch,
    },
    CardsFailed(String),
    /// A card's image, or `None` when it could not be fetched.
    Image {
        card_id: Uuid,
        image: Option<Box<DynamicImage>>,
    },
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Swipe(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Swipe(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut SwipeView> {
    match screen {
        Screen::Swipe(view) => Some(view),
        _ => None,
    }
}

/// Starts swiping into the loaded deck: builds its card query, opens the swipe screen over the current one, and fetches the first page.
pub(crate) fn open(cx: &mut Context<'_>, deck: &Deck) {
    let mut view = SwipeView::opening(deck.deck_profile.clone());
    match swipe::query_for(deck) {
        Ok(query) => {
            view.stack = Some(Stack::new(swipe::cards_in(deck)));
            view.query = Some(query);
            view.refill(cx);
        }
        Err(e) => view.error = Some(format!("Could not build the card search: {e}")),
    }
    cx.shared.status.clear();
    cx.push(Screen::Swipe(Box::new(view)));
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    match request {
        Request::FetchCards { deck_id, query } => Some(
            match async {
                let session = account.fresh().await?;
                client.search_deck_cards(deck_id, &query, &session).await
            }
            .await
            {
                Ok(page) => Update::Cards { deck_id, page }.into(),
                Err(e) => failure(&e, |m| Update::CardsFailed(m).into()),
            },
        ),
        Request::Image { card_id, url } => Some(
            Update::Image {
                card_id,
                image: images::fetch(&client.client, &url).await.ok().map(Box::new),
            }
            .into(),
        ),
        Request::Send { deck_id, effect } => send(account, deck_id, effect).await.err().map(|e| {
            failure(&e, |m| {
                screens::Update::Status(format!("That swipe did not save: {m}"))
            })
        }),
    }
}

/// Sends one swipe or undo.
async fn send(account: &Account, deck_id: Uuid, effect: Effect) -> Result<(), ClientError> {
    let client = &account.client;
    let session = account.fresh().await?;
    match effect {
        Effect::Add {
            scryfall_data_id,
            oracle_id,
            maybeboard,
        } => {
            let request = HttpCreateDeckCard {
                scryfall_data_id: scryfall_data_id.to_string(),
                oracle_id: oracle_id.map(|id| id.to_string()).unwrap_or_default(),
                quantity: 1,
                board: maybeboard.then(|| "maybeboard".to_owned()),
            };
            client
                .create_deck_card(deck_id, &request, &session)
                .await
                .map(drop)
        }
        Effect::Skip { oracle_id } => client.skip_deck_card(deck_id, oracle_id, &session).await,
        Effect::Unskip { oracle_id } => client.unskip_deck_card(deck_id, oracle_id, &session).await,
        Effect::Remove { scryfall_data_id } => {
            client
                .delete_deck_card(deck_id, scryfall_data_id, &session)
                .await
        }
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let Some(view) = nav.find_mut(pick) else {
        return;
    };
    match update {
        Update::CardsFailed(error) => {
            view.fetching = false;
            view.error = Some(error);
        }
        Update::Cards { deck_id, page } => {
            if view.deck.id != deck_id {
                return;
            }
            let Some(stack) = &mut view.stack else {
                return;
            };
            stack.push_page(page);
            view.fetching = false;
            view.refill(cx);
            view.sync_images(cx);
        }
        Update::Image { card_id, image } => {
            if let Some(picker) = &cx.shared.images
                && let Some(slot @ CardImage::Loading) = view.images.get_mut(&card_id)
            {
                *slot = match image {
                    Some(image) => {
                        CardImage::Ready(Box::new(RefCell::new(picker.new_resize_protocol(*image))))
                    }
                    None => CardImage::Failed,
                };
            }
        }
    }
}
