//! Where the deck screen's panels go for the room it has. A wide terminal shows the Profile column beside a grid of sections, a medium one shows the Profile column beside one tabbed panel, and a narrow one shows a single tabbed panel with Profile as its first tab.
//!
//! The app shows its sections as an accordion because a phone fits one at a time. The terminal shows as many as fit, and every pane stays reachable at every size through the tabs.

use ratatui::layout::{Constraint, Layout, Rect};

use super::Section;

/// One thing the deck screen shows in a panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pane {
    Profile,
    Section(Section),
    /// Ghost rows standing where the stats will load.
    Loading,
}

impl Pane {
    /// Whether the pane holds charts, so it wants a grid cell of its own when there is room.
    fn charts(self) -> bool {
        matches!(
            self,
            Self::Loading
                | Self::Section(Section::Distributions | Section::Mana | Section::DrawOdds)
        )
    }

    /// Whether the pane is Tags or Warnings, which share one panel beside the charts.
    fn shared(self) -> bool {
        matches!(self, Self::Section(Section::Tags | Section::Warnings))
    }
}

/// How much room the body has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mode {
    /// The Profile column beside a grid of panels.
    Wide,
    /// The Profile column beside one tabbed panel.
    Medium,
    /// One tabbed panel, Profile first.
    Narrow,
}

/// One framed panel: where it goes and the panes it holds, tabbed when there is more than one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Panel {
    pub(crate) area: Rect,
    pub(crate) panes: Vec<Pane>,
}

/// The panels for one frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Plan {
    pub(crate) mode: Mode,
    pub(crate) panels: Vec<Panel>,
    /// Room no panel takes because there are no sections, for a line saying why.
    pub(crate) note: Option<Rect>,
}

/// The Profile column's width, frame included: room for a label and a commander's name.
pub(crate) const PROFILE_WIDTH: u16 = 40;

/// The body is wide from this many cells: the Profile column and two grid columns of 59, where the Mana charts still sit side by side.
const WIDE_WIDTH: u16 = 158;

/// The body is wide only from this many rows, so two rows of charts each keep a readable height.
const WIDE_HEIGHT: u16 = 36;

/// The body keeps the Profile column from this many cells, leaving 68 for the tabbed panel.
const MEDIUM_WIDTH: u16 = 108;

/// The Budget panel's height in the Profile column: the frame, the currency chips and two rows.
const BUDGET_HEIGHT: u16 = 5;

/// Tags and Warnings join the Profile column only when it has this many rows left for them.
const SHARED_MIN_HEIGHT: u16 = 8;

/// The space between panels side by side.
const GAP: u16 = 1;

/// The mode for a body of this size.
pub(crate) fn mode(area: Rect) -> Mode {
    if area.width >= WIDE_WIDTH && area.height >= WIDE_HEIGHT {
        Mode::Wide
    } else if area.width >= MEDIUM_WIDTH {
        Mode::Medium
    } else {
        Mode::Narrow
    }
}

/// Lays `panes` out in `area`, Profile first and the rest in the app's order. A `zoomed` pane that shows fills the area alone. `profile_height` is the Profile panel's height, frame included, which the wide layout keeps clear before Budget.
pub(crate) fn plan(area: Rect, panes: &[Pane], zoomed: Option<Pane>, profile_height: u16) -> Plan {
    let mode = mode(area);
    if let Some(pane) = zoomed.filter(|z| panes.contains(z)) {
        return Plan {
            mode,
            panels: vec![Panel {
                area,
                panes: vec![pane],
            }],
            note: None,
        };
    }
    let sections: Vec<Pane> = panes
        .iter()
        .copied()
        .filter(|p| *p != Pane::Profile)
        .collect();
    match mode {
        Mode::Wide => wide(area, &sections, profile_height),
        Mode::Medium => medium(area, sections),
        Mode::Narrow => narrow(area, sections),
    }
}

/// One tabbed panel with Profile first, or Profile alone over a line for the note.
fn narrow(area: Rect, sections: Vec<Pane>) -> Plan {
    if sections.is_empty() {
        let [profile, note] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        return Plan {
            mode: Mode::Narrow,
            panels: vec![Panel {
                area: profile,
                panes: vec![Pane::Profile],
            }],
            note: Some(note),
        };
    }
    let mut panes = vec![Pane::Profile];
    panes.extend(sections);
    Plan {
        mode: Mode::Narrow,
        panels: vec![Panel { area, panes }],
        note: None,
    }
}

/// The Profile column beside one tabbed panel of every section.
fn medium(area: Rect, sections: Vec<Pane>) -> Plan {
    let [left, right] = columns(area);
    let mut panels = vec![Panel {
        area: left,
        panes: vec![Pane::Profile],
    }];
    let note = if sections.is_empty() {
        Some(right)
    } else {
        panels.push(Panel {
            area: right,
            panes: sections,
        });
        None
    };
    Plan {
        mode: Mode::Medium,
        panels,
        note,
    }
}

/// The Profile column with Budget, and Tags and Warnings when they fit under it, beside a grid of the charts.
fn wide(area: Rect, sections: &[Pane], profile_height: u16) -> Plan {
    let [left, right] = columns(area);
    let charts: Vec<Pane> = sections.iter().copied().filter(|p| p.charts()).collect();
    let mut cells: Vec<Vec<Pane>> = charts.iter().map(|p| vec![*p]).collect();
    let mut column = vec![(vec![Pane::Profile], profile_height)];

    if charts.is_empty() {
        // Nothing to chart: the grid has the room, so each section takes a cell of its own.
        cells = sections.iter().map(|p| vec![*p]).collect();
    } else {
        let budget = Pane::Section(Section::Budget);
        if sections.contains(&budget) {
            column.push((vec![budget], BUDGET_HEIGHT));
        }
        let shared: Vec<Pane> = sections.iter().copied().filter(|p| p.shared()).collect();
        if !shared.is_empty() {
            let taken = column
                .iter()
                .fold(0_u16, |sum, (_, h)| sum.saturating_add(*h));
            if left.height.saturating_sub(taken) >= SHARED_MIN_HEIGHT {
                column.push((shared, 0));
            } else {
                cells.push(shared);
            }
        }
    }

    // The column's last panel takes what is left of it.
    let last = column.len().saturating_sub(1);
    let constraints = column.iter().enumerate().map(|(i, (_, height))| {
        if i == last {
            Constraint::Fill(1)
        } else {
            Constraint::Length(*height)
        }
    });
    let areas = Layout::vertical(constraints).split(left);
    let mut panels: Vec<Panel> = column
        .into_iter()
        .zip(areas.iter())
        .map(|((panes, _), area)| Panel { area: *area, panes })
        .collect();

    let note = if cells.is_empty() {
        Some(right)
    } else {
        panels.extend(grid(right, cells));
        None
    };
    Plan {
        mode: Mode::Wide,
        panels,
        note,
    }
}

/// The Profile column and the room beside it.
fn columns(area: Rect) -> [Rect; 2] {
    Layout::horizontal([Constraint::Length(PROFILE_WIDTH), Constraint::Fill(1)])
        .spacing(GAP)
        .areas(area)
}

/// Cells in rows of two, the rows sharing the height, an odd last cell spanning its row.
fn grid(area: Rect, cells: Vec<Vec<Pane>>) -> Vec<Panel> {
    let mut rows: Vec<Vec<Vec<Pane>>> = Vec::new();
    for cell in cells {
        match rows.last_mut() {
            Some(row) if row.len() < 2 => row.push(cell),
            _ => rows.push(vec![cell]),
        }
    }
    let row_areas = Layout::vertical(rows.iter().map(|_| Constraint::Fill(1))).split(area);
    let mut panels = Vec::new();
    for (row, row_area) in rows.into_iter().zip(row_areas.iter()) {
        let cell_areas = Layout::horizontal(row.iter().map(|_| Constraint::Fill(1)))
            .spacing(GAP)
            .split(*row_area);
        panels.extend(
            row.into_iter()
                .zip(cell_areas.iter())
                .map(|(panes, area)| Panel { area: *area, panes }),
        );
    }
    panels
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Pane; 7] = [
        Pane::Profile,
        Pane::Section(Section::Budget),
        Pane::Section(Section::Tags),
        Pane::Section(Section::Distributions),
        Pane::Section(Section::Mana),
        Pane::Section(Section::DrawOdds),
        Pane::Section(Section::Warnings),
    ];

    /// The body of a terminal this size: a margin of one and four rows of chrome.
    fn body(width: u16, height: u16) -> Rect {
        Rect::new(1, 1, width.saturating_sub(2), height.saturating_sub(5))
    }

    fn panes_of(plan: &Plan) -> Vec<Vec<Pane>> {
        plan.panels.iter().map(|p| p.panes.clone()).collect()
    }

    /// Every pane sits in exactly one panel, and the panels stay inside the body without overlapping.
    fn assert_sound(plan: &Plan, area: Rect, panes: &[Pane]) {
        for pane in panes {
            let homes = plan
                .panels
                .iter()
                .filter(|p| p.panes.contains(pane))
                .count();
            assert_eq!(homes, 1, "{pane:?} in {plan:?}");
        }
        for (i, a) in plan.panels.iter().enumerate() {
            assert!(
                area.union(a.area) == area || a.area.is_empty(),
                "{a:?} leaves {area:?}"
            );
            for b in plan.panels.iter().skip(i.saturating_add(1)) {
                assert!(!a.area.intersects(b.area), "{a:?} overlaps {b:?}");
            }
        }
    }

    #[test]
    fn the_mode_follows_the_width_and_the_height() {
        assert_eq!(mode(body(200, 50)), Mode::Wide);
        assert_eq!(mode(body(160, 41)), Mode::Wide);
        assert_eq!(mode(body(159, 50)), Mode::Medium);
        assert_eq!(mode(body(200, 40)), Mode::Medium, "too short for two rows");
        assert_eq!(mode(body(140, 40)), Mode::Medium);
        assert_eq!(mode(body(110, 24)), Mode::Medium);
        assert_eq!(mode(body(109, 50)), Mode::Narrow);
        assert_eq!(mode(body(80, 24)), Mode::Narrow);
        assert_eq!(mode(body(12, 5)), Mode::Narrow);
    }

    #[test]
    fn wide_puts_budget_tags_and_warnings_under_the_profile_and_grids_the_charts() {
        let area = body(200, 50);
        let plan = plan(area, &ALL, None, 8);
        assert_eq!(plan.mode, Mode::Wide);
        assert_eq!(
            panes_of(&plan),
            [
                vec![Pane::Profile],
                vec![Pane::Section(Section::Budget)],
                vec![
                    Pane::Section(Section::Tags),
                    Pane::Section(Section::Warnings)
                ],
                vec![Pane::Section(Section::Distributions)],
                vec![Pane::Section(Section::Mana)],
                vec![Pane::Section(Section::DrawOdds)],
            ]
        );
        let [profile, budget, ..] = plan.panels.as_slice() else {
            panic!("no column: {plan:?}");
        };
        assert_eq!(profile.area.height, 8);
        assert_eq!(budget.area.height, BUDGET_HEIGHT);
        let draw_odds = &plan.panels[5];
        assert_eq!(
            draw_odds.area.width,
            plan.panels[3]
                .area
                .width
                .saturating_add(plan.panels[4].area.width)
                .saturating_add(GAP),
            "the odd cell spans its row"
        );
        assert_sound(&plan, area, &ALL);
    }

    #[test]
    fn wide_without_charts_gives_each_section_a_cell() {
        let panes = [
            Pane::Profile,
            Pane::Section(Section::Budget),
            Pane::Section(Section::Tags),
        ];
        let area = body(200, 50);
        let plan = plan(area, &panes, None, 8);
        assert_eq!(
            panes_of(&plan),
            [
                vec![Pane::Profile],
                vec![Pane::Section(Section::Budget)],
                vec![Pane::Section(Section::Tags)],
            ]
        );
        assert_eq!(plan.panels[0].area.height, area.height);
        assert_sound(&plan, area, &panes);
    }

    #[test]
    fn wide_with_a_tall_profile_moves_tags_and_warnings_into_the_grid() {
        let area = body(200, 50);
        let plan = plan(area, &ALL, None, 36);
        assert_eq!(
            plan.panels.last().map(|p| p.panes.clone()),
            Some(vec![
                Pane::Section(Section::Tags),
                Pane::Section(Section::Warnings)
            ])
        );
        assert_eq!(plan.panels.len(), 6);
        assert_sound(&plan, area, &ALL);
    }

    #[test]
    fn medium_tabs_every_section_beside_the_profile() {
        let area = body(140, 40);
        let plan = plan(area, &ALL, None, 8);
        assert_eq!(plan.mode, Mode::Medium);
        assert_eq!(panes_of(&plan), [vec![Pane::Profile], ALL[1..].to_vec()]);
        assert_eq!(plan.panels[0].area.width, PROFILE_WIDTH);
        assert_sound(&plan, area, &ALL);
    }

    #[test]
    fn narrow_tabs_everything_with_the_profile_first() {
        let area = body(80, 24);
        let plan = plan(area, &ALL, None, 8);
        assert_eq!(plan.mode, Mode::Narrow);
        assert_eq!(panes_of(&plan), [ALL.to_vec()]);
        assert_eq!(plan.panels[0].area, area);
        assert_eq!(plan.note, None);
    }

    #[test]
    fn a_profile_alone_leaves_room_for_the_note_at_every_size() {
        for (width, height) in [(200, 50), (140, 40), (80, 24)] {
            let area = body(width, height);
            let plan = plan(area, &[Pane::Profile], None, 8);
            assert_eq!(panes_of(&plan), [vec![Pane::Profile]], "{width}x{height}");
            let note = plan.note.expect("a note");
            assert!(!note.intersects(plan.panels[0].area));
        }
    }

    #[test]
    fn a_zoomed_pane_fills_the_body_and_a_missing_one_is_ignored() {
        for (width, height) in [(200, 50), (140, 40), (80, 24)] {
            let area = body(width, height);
            let zoomed = plan(area, &ALL, Some(Pane::Section(Section::Mana)), 8);
            assert_eq!(
                zoomed.panels,
                [Panel {
                    area,
                    panes: vec![Pane::Section(Section::Mana)]
                }]
            );
            let missing = plan(area, &ALL[..2], Some(Pane::Section(Section::Mana)), 8);
            assert!(missing.panels.len() > 1 || mode(area) == Mode::Narrow);
            assert_sound(&missing, area, &ALL[..2]);
        }
    }

    #[test]
    fn loading_takes_the_charts_cell_and_every_size_stays_sound() {
        let loading = [
            Pane::Profile,
            Pane::Section(Section::Budget),
            Pane::Section(Section::Tags),
            Pane::Loading,
        ];
        for (width, height) in [
            (200, 50),
            (160, 41),
            (140, 40),
            (110, 30),
            (80, 24),
            (12, 5),
            (1, 1),
            (0, 0),
        ] {
            let area = body(width, height);
            for panes in [&ALL[..], &loading[..], &ALL[..1]] {
                let plan = plan(area, panes, None, 9);
                assert_sound(&plan, area, panes);
            }
        }
        let plan = plan(body(200, 50), &loading, None, 9);
        assert_eq!(
            plan.panels.last().map(|p| p.panes.clone()),
            Some(vec![Pane::Loading])
        );
    }
}
