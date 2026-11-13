//! The command zone each format had as an open deck form last left it, so picking that format
//! again in the same form brings its zone back. Nothing outlives the form: a new form starts empty.

use zwipe_core::domain::deck::Format;

/// Each format's zone `Z` (the command-zone cards, and whatever the client draws them with) as
/// the form last left that format.
#[derive(Debug, Clone, PartialEq)]
pub struct LeftZones<Z> {
    zones: Vec<(Format, Z)>,
}

impl<Z> Default for LeftZones<Z> {
    fn default() -> Self {
        Self { zones: Vec::new() }
    }
}

impl<Z: Clone> LeftZones<Z> {
    /// The form moves from format `from` to `to`, another format or none: keeps `zone` (the zone
    /// as it stands before the move) for `from`. A move to the same format keeps nothing.
    pub fn leave(&mut self, from: Option<Format>, to: Option<Format>, zone: impl FnOnce() -> Z) {
        let Some(from) = from.filter(|from| Some(*from) != to) else {
            return;
        };
        self.zones.retain(|(kept, _)| *kept != from);
        self.zones.push((from, zone()));
    }

    /// The zone format `to` had when the form last left it, to put back as the form picks it.
    #[must_use]
    pub fn back(&self, to: Option<Format>) -> Option<Z> {
        let to = to?;
        self.zones
            .iter()
            .find(|(kept, _)| *kept == to)
            .map(|(_, zone)| zone.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_format_left_and_picked_again_gets_its_zone_back() {
        let mut zones = LeftZones::default();
        zones.leave(
            Some(Format::Commander),
            Some(Format::Oathbreaker),
            || "Krenko",
        );
        assert_eq!(zones.back(Some(Format::Commander)), Some("Krenko"));
        assert_eq!(zones.back(Some(Format::Oathbreaker)), None, "never left");
        zones.leave(Some(Format::Commander), None, || "Edgar");
        assert_eq!(
            zones.back(Some(Format::Commander)),
            Some("Edgar"),
            "the last leave wins"
        );
        zones.leave(Some(Format::Commander), Some(Format::Commander), || "same");
        assert_eq!(
            zones.back(Some(Format::Commander)),
            Some("Edgar"),
            "no move, nothing kept"
        );
        assert_eq!(zones.back(None), None);
        zones.leave(None, Some(Format::Commander), || "none");
        assert_eq!(zones.back(None), None, "no format keeps nothing");
    }
}
