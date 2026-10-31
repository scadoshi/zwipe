//! The fullscreen card image the app opens from a row's or the featured card's "Image": one face at a time, flipped with `f` on a card with more than one. With card images off, the face's text stands in.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::fmt;

use image::DynamicImage;
use ratatui_image::protocol::StatefulProtocol;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

use super::Screen;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;
use crate::inbound::screens;
use crate::outbound::account::Account;
use crate::outbound::images;

/// The face image as it loads.
pub(crate) enum FaceImage {
    /// Card images are off, or the face has no image.
    None,
    Loading,
    Ready(Box<RefCell<StatefulProtocol>>),
    Failed,
}

impl fmt::Debug for FaceImage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::None => "None",
            Self::Loading => "Loading",
            Self::Ready(_) => "Ready",
            Self::Failed => "Failed",
        })
    }
}

/// The preview's state: the card, the face on screen and its image.
#[derive(Debug)]
pub(crate) struct ImagePreview {
    pub(crate) card: Card,
    pub(crate) face: usize,
    pub(crate) image: FaceImage,
}

impl ImagePreview {
    /// How many faces have their own image: two or more for a card that flips.
    pub(crate) fn faces(&self) -> usize {
        let data = &self.card.scryfall_data;
        let faces = data.card_faces.as_ref().map_or(0, |f| f.len());
        (1..faces)
            .take_while(|&i| data.face_image_url(i, ImageSize::Normal).is_some())
            .count()
            .saturating_add(1)
    }

    /// Shows the next face, back to the front after the last, and asks for its image.
    pub(crate) fn flip(&mut self, cx: &mut Context<'_>) {
        let faces = self.faces();
        if faces < 2 {
            return;
        }
        self.face = self.face.saturating_add(1) % faces;
        self.load(cx);
    }

    /// Asks for the face's image when card images are on.
    fn load(&mut self, cx: &mut Context<'_>) {
        let url = self
            .card
            .scryfall_data
            .face_image_url(self.face, ImageSize::Normal);
        self.image = match url {
            Some(url) if cx.shared.images.is_some() => {
                cx.request(
                    Request::Image {
                        face: self.face,
                        url: url.to_owned(),
                    }
                    .into(),
                );
                FaceImage::Loading
            }
            _ => FaceImage::None,
        };
    }
}

/// The preview's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    Image { face: usize, url: String },
}

/// The preview's results.
pub(crate) enum Update {
    Image {
        face: usize,
        image: Option<Box<DynamicImage>>,
    },
}

impl fmt::Debug for Update {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self::Image { face, image } = self;
        write!(f, "Image {{ face: {face}, loaded: {} }}", image.is_some())
    }
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::ImagePreview(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::ImagePreview(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut ImagePreview> {
    match screen {
        Screen::ImagePreview(view) => Some(view),
        _ => None,
    }
}

/// Opens `card` full screen on `face`.
pub(crate) fn open(cx: &mut Context<'_>, card: Card, face: usize) {
    let mut view = ImagePreview {
        card,
        face,
        image: FaceImage::None,
    };
    view.load(cx);
    cx.push(Screen::ImagePreview(Box::new(view)));
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let Request::Image { face, url } = request;
    let image = images::fetch(&account.client.client, &url)
        .await
        .ok()
        .map(Box::new);
    Some(Update::Image { face, image }.into())
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    let Update::Image { face, image } = update;
    let Some(view) = nav.find_mut(pick) else {
        return;
    };
    if view.face != face || !matches!(view.image, FaceImage::Loading) {
        return;
    }
    view.image = match (image, &cx.shared.images) {
        (Some(image), Some(picker)) => {
            FaceImage::Ready(Box::new(RefCell::new(picker.new_resize_protocol(*image))))
        }
        _ => FaceImage::Failed,
    };
}
