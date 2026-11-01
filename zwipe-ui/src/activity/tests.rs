use chrono::Weekday;
use zwipe_core::http::contracts::suggestions::HttpActivity;

use super::*;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn day(on: NaiveDate, swipes: u32, searches: u32) -> HttpActivityDay {
    HttpActivityDay {
        day: on,
        swipes,
        searches,
    }
}

fn outcomes(added: u64, skipped: u64, maybed: u64, removed: u64) -> HttpSwipeOutcomes {
    HttpSwipeOutcomes {
        added,
        skipped,
        maybed,
        removed,
    }
}

fn tag(
    slug: &str,
    role: &str,
    mine: HttpSwipeOutcomes,
    everyone: HttpSwipeOutcomes,
) -> HttpTagRates {
    HttpTagRates {
        tag: slug.to_owned(),
        role: role.to_owned(),
        outcomes: mine,
        everyone,
    }
}

fn empty(through: NaiveDate) -> HttpActivity {
    HttpActivity {
        through,
        days: Vec::new(),
        outcomes: HttpSwipeOutcomes::default(),
        roles: Vec::new(),
        tags: Vec::new(),
        passes: Vec::new(),
    }
}

#[test]
fn the_year_is_every_day_in_week_columns_ending_on_through() {
    let through = date(2026, 10, 27);
    let layout = heat_layout(through, &[]);
    assert_eq!(layout.cells.len(), 371, "every day of the reach");
    let last = layout.cells.last().unwrap();
    assert_eq!(last.date, through);
    assert_eq!(last.row, 2, "Oct 27 2026 is a Tuesday");
    assert!(layout.cells.iter().all(|c| c.level == 0 && !c.peak));
    assert_eq!(layout.columns, last.column + 1);
    assert_eq!(layout.columns, 54);
    let first = &layout.cells[0];
    assert_eq!(first.date, date(2025, 10, 22));
    assert_eq!((first.column, first.row), (0, 3), "a Wednesday start");
    assert_eq!(
        layout.index_at(last.column, last.row),
        Some(370),
        "the last cell by place"
    );
    assert_eq!(
        layout.index_at(0, 0),
        None,
        "before the start is off the grid"
    );
}

#[test]
fn a_saturday_through_fills_53_columns() {
    let through = date(2026, 10, 31);
    assert_eq!(through.weekday(), Weekday::Sat);
    let layout = heat_layout(through, &[]);
    assert_eq!(layout.cells[0].row, 0, "the start is a Sunday");
    assert_eq!(layout.columns, 53);
}

#[test]
fn busy_days_shade_by_quarter_and_empty_days_stay_blank() {
    let through = date(2026, 10, 27);
    let days: Vec<HttpActivityDay> = (1..=8)
        .map(|n| {
            day(
                through - Duration::days(n),
                u32::try_from(n).unwrap() * 10,
                0,
            )
        })
        .collect();
    let layout = heat_layout(through, &days);
    let level_on = |d: NaiveDate| layout.cells.iter().find(|c| c.date == d).unwrap().level;
    assert_eq!(level_on(through), 0, "no activity today");
    assert_eq!(level_on(through - Duration::days(1)), 1, "the quietest");
    assert_eq!(level_on(through - Duration::days(4)), 2);
    assert_eq!(level_on(through - Duration::days(6)), 3);
    assert_eq!(level_on(through - Duration::days(8)), 4, "the busiest");
}

#[test]
fn the_level_reads_quartiles_of_the_busy_days() {
    let busy = [1, 2, 3, 4, 5, 6, 7, 8, 9];
    assert_eq!(level(0, &busy), 0);
    assert_eq!(level(3, &busy), 1);
    assert_eq!(level(4, &busy), 2);
    assert_eq!(level(5, &busy), 2);
    assert_eq!(level(7, &busy), 3);
    assert_eq!(level(8, &busy), 4);
    assert_eq!(level(5, &[]), 0, "no busy days");
}

#[test]
fn searches_count_with_swipes() {
    let through = date(2026, 10, 27);
    let layout = heat_layout(through, &[day(through, 0, 3)]);
    let last = layout.cells.last().unwrap();
    assert_eq!((last.swipes, last.searches, last.level), (0, 3, 1));
}

#[test]
fn a_spike_is_a_peak() {
    let through = date(2026, 10, 27);
    let mut days: Vec<HttpActivityDay> = (1..=10)
        .map(|n| day(through - Duration::days(n), 5, 0))
        .collect();
    days.push(day(through, 200, 0));
    let layout = heat_layout(through, &days);
    let peaks: Vec<NaiveDate> = layout
        .cells
        .iter()
        .filter(|c| c.peak)
        .map(|c| c.date)
        .collect();
    assert_eq!(peaks, [through]);
}

#[test]
fn peaks_are_the_biggest_past_four_times_the_median() {
    assert_eq!(
        chart_math::peak_indices(&[0, 2, 3, 4, 20, 40], 4, 12),
        [5, 4]
    );
    assert_eq!(chart_math::peak_indices(&[0], 4, 12), [0_usize; 0]);
    assert_eq!(chart_math::peak_indices(&[], 4, 12), [0_usize; 0]);
    let mut steady = vec![10; 20];
    steady.push(45);
    assert_eq!(chart_math::peak_indices(&steady, 4, 12), [20]);
    assert_eq!(
        chart_math::peak_indices(&[1, 1, 1, 1, 1, 9, 8, 7], 4, 2),
        [5, 6]
    );
}

#[test]
fn at_most_twelve_peaks_light_up() {
    let through = date(2026, 10, 27);
    let mut days: Vec<HttpActivityDay> = (1..=40)
        .map(|n| day(through - Duration::days(n), 1, 0))
        .collect();
    days.extend((41..=60).map(|n| day(through - Duration::days(n), 100, 0)));
    let layout = heat_layout(through, &days);
    assert_eq!(layout.cells.iter().filter(|c| c.peak).count(), 12);
}

#[test]
fn months_are_labeled_apart_and_in_order() {
    let layout = heat_layout(date(2026, 10, 27), &[]);
    for pair in layout.months.windows(2) {
        assert!(pair[1].0 >= pair[0].0 + LABEL_SPAN, "{:?}", layout.months);
    }
    assert_eq!(layout.months.last().unwrap().1, "Oct");
    // The year starts Oct 22 2025, a Wednesday: its first Sunday is Oct 26, labeled
    // "Oct" at column 1, popped by "Nov" at column 2.
    assert_eq!(layout.months[0], (2, "Nov".to_owned()));
}

#[test]
fn weekday_labels_sit_on_monday_wednesday_and_friday() {
    assert_eq!(WEEKDAY_LABELS, [(1, "Mon"), (3, "Wed"), (5, "Fri")]);
}

#[test]
fn the_year_rests_as_its_totals_and_span() {
    let on = date(2026, 10, 3);
    let days = [day(on, 1200, 2), day(on, 34, 0)];
    assert_eq!(
        year_tags(&days),
        ["1,234 swipes", "2 searches", "Last year"]
    );
    assert_eq!(year_tags(&[]), ["0 swipes", "0 searches", "Last year"]);
    assert_eq!(
        year_tags(&[day(on, 1, 1)]),
        ["1 swipe", "1 search", "Last year"]
    );
}

#[test]
fn a_day_reads_as_its_swipes_searches_and_date() {
    let on = date(2026, 10, 3);
    let cell = |swipes, searches| DayCell {
        column: 0,
        row: 0,
        level: 0,
        peak: false,
        date: on,
        swipes,
        searches,
    };
    assert_eq!(day_tags(&cell(0, 0)), ["0 swipes", "0 searches", "Oct 3"]);
    assert_eq!(
        day_tags(&cell(1200, 1)),
        ["1,200 swipes", "1 search", "Oct 3"]
    );
    assert_eq!(counted(1, "skip", "skips"), "1 skip");
    assert_eq!(counted(2_500, "skip", "skips"), "2,500 skips");
}

#[test]
fn outcomes_scale_against_the_largest() {
    let bars = outcome_bars(&outcomes(5, 20, 0, 1));
    let rows: Vec<(&str, u64, u32)> = bars
        .iter()
        .map(|b| (b.label.as_str(), b.count, b.pct))
        .collect();
    assert_eq!(
        rows,
        [
            ("Added", 5, 25),
            ("Skipped", 20, 100),
            ("Maybeboard", 0, 0),
            ("Removed", 1, 5)
        ]
    );
    assert!(
        outcome_bars(&HttpSwipeOutcomes::default())
            .iter()
            .all(|b| b.pct == 0),
        "nothing over a top of one"
    );
}

#[test]
fn anything_at_all_counts_as_activity() {
    let through = date(2026, 10, 27);
    assert!(!has_anything(&empty(through)));
    let mut with_day = empty(through);
    with_day.days.push(day(through, 1, 0));
    assert!(has_anything(&with_day));
    let mut with_removed = empty(through);
    with_removed.outcomes.removed = 1;
    assert!(has_anything(&with_removed));
    assert!(has_outcomes(&with_removed.outcomes));
    let mut with_tag = empty(through);
    with_tag
        .tags
        .push(tag("t", "ramp", outcomes(0, 0, 0, 0), outcomes(0, 0, 0, 0)));
    assert!(has_anything(&with_tag));
}

#[test]
fn removed_is_a_share_of_the_added_cards_and_the_rest_of_the_dealt() {
    let o = outcomes(4, 5, 1, 1);
    assert_eq!(RateKind::Added.parts(&o), (4, 10));
    assert_eq!(RateKind::Removed.parts(&outcomes(2, 0, 0, 5)), (2, 2));
    assert_eq!(RateKind::Added.percent(&o), Some(40));
    assert_eq!(RateKind::Skipped.percent(&o), Some(50));
    assert_eq!(RateKind::Maybed.percent(&o), Some(10));
    assert_eq!(RateKind::Removed.percent(&o), Some(25));
    assert_eq!(
        RateKind::Removed.percent(&outcomes(0, 9, 0, 0)),
        None,
        "nothing added"
    );
    assert_eq!(
        RateKind::ALL.map(RateKind::label),
        ["Added", "Skipped", "Maybeboard", "Removed"]
    );
    assert_eq!(RateKind::default(), RateKind::Added);
}

#[test]
fn role_rates_sort_highest_first_and_reveal_only_swiped_tags() {
    let role = |slug: &str, o| HttpRoleRates {
        role: slug.to_owned(),
        outcomes: o,
    };
    let swiped = [tag(
        "mana-rock",
        "ramp",
        outcomes(5, 5, 0, 0),
        outcomes(0, 0, 0, 0),
    )];
    let bars = role_rate_bars(
        RateKind::Added,
        &[
            role("ramp", outcomes(1, 2, 0, 0)),
            role("removal", outcomes(9, 1, 0, 0)),
            role("card_draw", outcomes(0, 0, 0, 0)),
        ],
        &swiped,
    );
    let rows: Vec<(String, String, u64, bool)> = bars
        .into_iter()
        .map(|b| (b.key, b.label, b.count, b.revealable))
        .collect();
    assert_eq!(
        rows,
        [
            ("removal".to_owned(), "remov".to_owned(), 90, false),
            ("ramp".to_owned(), "ramp".to_owned(), 33, true)
        ],
        "a role with nothing dealt is left out"
    );
    assert_eq!(tags_under("ramp", &swiped), ["mana-rock"]);
    assert_eq!(tags_under("removal", &swiped), Vec::<String>::new());
}

#[test]
fn ties_keep_the_servers_order() {
    let role = |slug: &str| HttpRoleRates {
        role: slug.to_owned(),
        outcomes: outcomes(1, 1, 0, 0),
    };
    let keys: Vec<String> = role_rate_bars(RateKind::Added, &[role("b"), role("a")], &[])
        .into_iter()
        .map(|b| b.key)
        .collect();
    assert_eq!(keys, ["b", "a"]);
}

#[test]
fn the_baseline_is_everyones_share_or_even_odds() {
    let tags = [
        tag("a", "ramp", outcomes(0, 0, 0, 0), outcomes(1, 3, 0, 0)),
        tag("b", "ramp", outcomes(0, 0, 0, 0), outcomes(1, 5, 0, 0)),
    ];
    assert!((baseline(RateKind::Added, &tags) - 0.2).abs() < 1e-9);
    assert!((baseline(RateKind::Added, &[]) - 0.5).abs() < 1e-9);
}

#[test]
fn a_lift_needs_a_share_above_everyone_elses() {
    let standout = tag("x", "ramp", outcomes(1, 9, 0, 0), outcomes(80, 20, 0, 0));
    let score = lift_score(RateKind::Skipped, &standout, 0.5).unwrap();
    // mine 0.9, others (20 + 5) / 110.
    let expected = 9.0 * (0.9_f64 / (25.0 / 110.0)).ln();
    assert!((score - expected).abs() < 1e-9, "{score}");
    assert_eq!(
        lift_score(RateKind::Added, &standout, 0.5),
        None,
        "lower than theirs"
    );
    let nothing = tag("y", "ramp", outcomes(0, 0, 0, 0), outcomes(1, 1, 0, 0));
    assert_eq!(lift_score(RateKind::Added, &nothing, 0.5), None);
}

#[test]
fn a_tag_others_keep_ranks_above_one_everyone_passes_on() {
    let common = tag(
        "generic",
        "ramp",
        outcomes(1, 9, 0, 0),
        outcomes(10, 90, 0, 0),
    );
    let standout = tag(
        "mana-rock",
        "ramp",
        outcomes(1, 9, 0, 0),
        outcomes(80, 20, 0, 0),
    );
    let keys: Vec<String> = tag_rate_bars(RateKind::Skipped, &[common, standout])
        .into_iter()
        .map(|b| b.key)
        .collect();
    assert_eq!(keys, ["mana-rock", "generic"]);
}

#[test]
fn a_tag_the_user_passes_on_less_than_others_is_left_out() {
    let kept = tag("kept", "ramp", outcomes(8, 2, 0, 0), outcomes(10, 90, 0, 0));
    assert_eq!(tag_rate_bars(RateKind::Skipped, &[kept]), Vec::<Bar>::new());
}

#[test]
fn the_tag_chart_draws_its_rows_highest_share_first() {
    let steady = tag(
        "steady",
        "ramp",
        outcomes(40, 60, 0, 0),
        outcomes(90, 10, 0, 0),
    );
    let sharp = tag(
        "sharp",
        "ramp",
        outcomes(1, 9, 0, 0),
        outcomes(90, 10, 0, 0),
    );
    let rows: Vec<(String, u32, bool)> = tag_rate_bars(RateKind::Skipped, &[steady, sharp])
        .into_iter()
        .map(|b| (b.label, b.pct, b.revealable))
        .collect();
    assert_eq!(
        rows,
        [
            ("sharp".to_owned(), 90, true),
            ("steady".to_owned(), 60, true)
        ]
    );
}

#[test]
fn a_single_maybeboard_over_other_players_rate_still_shows() {
    let tags: Vec<HttpTagRates> = (0..3)
        .map(|i| {
            tag(
                &format!("t{i}"),
                "ramp",
                outcomes(0, 19, 1, 0),
                outcomes(60, 37, 3, 0),
            )
        })
        .collect();
    assert_eq!(tag_rate_bars(RateKind::Maybed, &tags).len(), 3);
}

#[test]
fn the_tag_chart_shows_at_most_its_rows() {
    let tags: Vec<HttpTagRates> = (0..20)
        .map(|i| {
            tag(
                &format!("t{i:02}"),
                "ramp",
                outcomes(0, 20, 0, 0),
                outcomes(90, 10, 0, 0),
            )
        })
        .collect();
    let bars = tag_rate_bars(RateKind::Skipped, &tags);
    assert_eq!(bars.len(), TAG_ROWS);
    assert_eq!(bars[0].key, "t00", "ties go by slug");
}

#[test]
fn an_empty_tag_chart_says_nothing_was_removed_only_on_removed_with_none() {
    let none_removed = [tag(
        "ramp",
        "ramp",
        outcomes(2, 8, 0, 0),
        outcomes(5, 5, 0, 0),
    )];
    let one_removed = [tag(
        "ramp",
        "ramp",
        outcomes(2, 8, 0, 1),
        outcomes(5, 5, 0, 5),
    )];
    assert_eq!(
        tag_empty_line(RateKind::Removed, &none_removed),
        NONE_REMOVED,
        "Removed with nothing removed under any tag"
    );
    assert_eq!(
        tag_empty_line(RateKind::Removed, &one_removed),
        NO_TAG_STANDS_OUT,
        "Removed with a removal that does not stand out"
    );
    assert_eq!(
        tag_empty_line(RateKind::Added, &none_removed),
        NO_TAG_STANDS_OUT,
        "any other share"
    );
}

#[test]
fn a_year_before_the_first_date_has_no_grid() {
    let layout = heat_layout(NaiveDate::MIN, &[]);
    assert_eq!(layout.cells, Vec::new());
    assert_eq!(layout.columns, 0);
}

#[test]
fn the_year_line_reads_the_year_for_screen_readers() {
    assert_eq!(year_line(&[]), "0 swipes and 0 searches in the last year");
    let on = NaiveDate::from_ymd_opt(2026, 10, 1).unwrap();
    assert_eq!(
        year_line(&[day(on, 1, 1), day(on, 1_233, 0)]),
        "1,234 swipes and 1 search in the last year"
    );
}
