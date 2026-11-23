//! The schedule rule: which program day falls on which date for an athlete.
//!
//! An athlete has a start date (a Monday) for cycle 1 week 1 day Monday.
//! Cycles follow one another, {CYCLE_WEEKS} weeks each, so program day
//! (cycle `c`, week `w`, weekday `d`, all counted from 0) falls on
//! `start + 28 * c + 7 * w + d`.

use crate::dates;
use crate::program::{Cycle, Day, Program, Week};

pub const CYCLE_WEEKS: i64 = 4;
pub const CYCLE_DAYS: i64 = CYCLE_WEEKS * 7;

/// What a date holds for an athlete, before any move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place<'a> {
    /// The date is before the athlete's start; the start day is given.
    BeforeStart(i64),
    /// The cycle or week has no program in the file.
    NotLoaded { cycle: usize, week: usize },
    /// A loaded week with no session on this weekday.
    Rest { cycle: usize, week: usize, dow: u8 },
    /// A session of the program.
    Session(Slot<'a>),
}

/// A session of the program and where it sits in it.
#[derive(Clone, PartialEq, Eq)]
pub struct Slot<'a> {
    pub cycle: usize,
    pub week: usize,
    pub dow: u8,
    pub cycle_ref: &'a Cycle,
    pub week_ref: &'a Week,
    pub day: &'a Day,
}

impl std::fmt::Debug for Slot<'_> {
    // Short on purpose: the whole cycle hangs off a slot, and a failing test
    // should print where it is, not the program.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slot(cycle {}, week {}, {})", self.cycle, self.week, self.day.dow)
    }
}

/// The day number a program day falls on for an athlete who starts on `start`.
pub fn natural_day(start: i64, cycle: usize, week: usize, dow: u8) -> i64 {
    start + CYCLE_DAYS * cycle as i64 + 7 * week as i64 + i64::from(dow)
}

/// Finds the program day on `date`.
pub fn locate(program: &Program, start: i64, date: i64) -> Place<'_> {
    if date < start {
        return Place::BeforeStart(start);
    }
    let offset = date - start;
    let cycle = (offset / CYCLE_DAYS) as usize;
    let week = ((offset % CYCLE_DAYS) / 7) as usize;
    let dow = (offset % 7) as u8;
    let Some(cycle_ref) = program.cycles.get(cycle) else {
        return Place::NotLoaded { cycle, week };
    };
    let Some(week_ref) = cycle_ref.weeks.get(week) else {
        return Place::NotLoaded { cycle, week };
    };
    if week_ref.missing {
        return Place::NotLoaded { cycle, week };
    }
    match week_ref.days.iter().find(|d| d.dow_index() == dow) {
        Some(day) => Place::Session(Slot {
            cycle,
            week,
            dow,
            cycle_ref,
            week_ref,
            day,
        }),
        None => Place::Rest { cycle, week, dow },
    }
}

/// The week of the program a date falls in, counted from the start: the week
/// number since the start (0 for the first), which is negative before it.
pub fn week_number(start: i64, date: i64) -> i64 {
    (date - start).div_euclid(7)
}

/// The Monday that opens a week counted from the start.
pub fn week_start(start: i64, week_number: i64) -> i64 {
    start + 7 * week_number
}

/// A start date must be a Monday.
pub fn is_start_ok(day: i64) -> bool {
    dates::weekday(day) == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::SAMPLE;

    fn d(s: &str) -> i64 {
        dates::parse(s).unwrap()
    }

    fn sample() -> Program {
        Program::parse(SAMPLE).unwrap()
    }

    #[test]
    fn cycle_one_week_one_tuesday_is_the_day_after_a_monday_start() {
        // Start Monday 2026-11-30: its Tuesday is 2026-12-01.
        let p = sample();
        match locate(&p, d("2026-11-30"), d("2026-12-01")) {
            Place::Session(slot) => {
                assert_eq!((slot.cycle, slot.week, slot.dow), (0, 0, 1));
                assert_eq!(slot.day.kind, "lift");
            }
            other => panic!("{other:?}"),
        }
        assert_eq!(natural_day(d("2026-11-30"), 0, 0, 1), d("2026-12-01"));
    }

    #[test]
    fn cycle_two_week_one_monday_is_twenty_eight_days_after_the_start() {
        let p = sample();
        let start = d("2026-11-30");
        assert_eq!(natural_day(start, 1, 0, 0), d("2026-12-28"));
        match locate(&p, start, d("2026-12-28")) {
            Place::Session(slot) => assert_eq!((slot.cycle, slot.week, slot.dow), (1, 0, 0)),
            other => panic!("{other:?}"),
        }
        // The Sunday before belongs to cycle one week four.
        assert_eq!(
            locate(&p, start, d("2026-12-27")),
            Place::NotLoaded { cycle: 0, week: 3 }
        );
    }

    #[test]
    fn a_week_that_is_not_loaded_says_so_and_so_does_a_cycle_that_is_missing() {
        let p = sample();
        let start = d("2026-11-30");
        // Cycle two week two starts 2027-01-04.
        assert_eq!(
            locate(&p, start, d("2027-01-05")),
            Place::NotLoaded { cycle: 1, week: 1 }
        );
        // Cycle three is not in the file at all.
        assert_eq!(
            locate(&p, start, d("2027-01-25")),
            Place::NotLoaded { cycle: 2, week: 0 }
        );
    }

    #[test]
    fn before_the_start_the_answer_is_the_start_date() {
        let p = sample();
        let start = d("2026-11-30");
        assert_eq!(locate(&p, start, d("2026-11-29")), Place::BeforeStart(start));
    }

    #[test]
    fn a_weekday_the_week_has_no_session_for_is_a_rest_day() {
        let p = sample();
        let start = d("2026-11-30");
        // Sunday of week one, and Thursday of week two (week two lists Mon to Wed).
        assert_eq!(
            locate(&p, start, d("2026-12-06")),
            Place::Rest {
                cycle: 0,
                week: 0,
                dow: 6
            }
        );
        assert_eq!(
            locate(&p, start, d("2026-12-10")),
            Place::Rest {
                cycle: 0,
                week: 1,
                dow: 3
            }
        );
    }

    #[test]
    fn the_week_number_counts_from_the_start_and_goes_negative_before_it() {
        let start = d("2026-11-30");
        assert_eq!(week_number(start, d("2026-11-30")), 0);
        assert_eq!(week_number(start, d("2026-12-06")), 0);
        assert_eq!(week_number(start, d("2026-12-07")), 1);
        assert_eq!(week_number(start, d("2026-11-29")), -1);
        assert_eq!(week_start(start, 2), d("2026-12-14"));
    }

    #[test]
    fn a_start_date_must_be_a_monday() {
        assert!(is_start_ok(d("2026-11-30")));
        assert!(!is_start_ok(d("2026-12-01")));
    }
}
