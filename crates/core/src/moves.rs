//! Moved sessions. A move names one session of the program for one athlete
//! (cycle, week, weekday) and gives a new date. On the new date the session
//! shows, marked as moved from its own date; its own date shows that it moved.
//! Two sessions on one date are both listed, in program order. A move can be
//! undone, and moving a session again replaces the earlier move.

use crate::data::{AthleteRec, Data, MoveRec};
use crate::dates;
use crate::schedule::{self, Place, Slot};

/// One thing a date shows for an athlete.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry<'a> {
    /// A session on this date. `moved_from` is the date it belongs to in the
    /// program when it was moved here.
    Session { slot: Slot<'a>, moved_from: Option<i64> },
    /// The session that belongs to this date and was moved to `to`.
    MovedOut { slot: Slot<'a>, to: i64 },
}

/// What a date shows when no session is on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Empty {
    BeforeStart(i64),
    NotLoaded { cycle: usize, week: usize },
    Rest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayView<'a> {
    pub date: i64,
    pub entries: Vec<Entry<'a>>,
    /// Why there is nothing to do on this date, when `entries` has no session.
    pub empty: Option<Empty>,
}

impl DayView<'_> {
    /// True when at least one session is on the date.
    pub fn has_session(&self) -> bool {
        self.entries.iter().any(|e| matches!(e, Entry::Session { .. }))
    }
}

fn slot_of<'a>(data: &'a Data, m: &MoveRec) -> Option<Slot<'a>> {
    let cycle = data.program.cycles.iter().position(|c| c.id == m.cycle)?;
    let cycle_ref = &data.program.cycles[cycle];
    let week = (m.week as usize).checked_sub(1)?;
    let week_ref = cycle_ref.weeks.get(week)?;
    if week_ref.missing {
        return None;
    }
    let day = week_ref.days.iter().find(|d| d.dow_index() == m.dow)?;
    Some(Slot {
        cycle,
        week,
        dow: m.dow,
        cycle_ref,
        week_ref,
        day,
    })
}

fn moves_of<'a>(data: &'a Data, athlete: &AthleteRec) -> impl Iterator<Item = &'a MoveRec> {
    let id = athlete.id;
    data.moves.iter().filter(move |m| m.athlete == id)
}

fn order_key(slot: &Slot<'_>) -> (usize, usize, u8) {
    (slot.cycle, slot.week, slot.dow)
}

/// What a date holds for an athlete, with moves applied.
pub fn day_view<'a>(data: &'a Data, athlete: &AthleteRec, date: i64) -> DayView<'a> {
    let Some(start) = dates::parse(&athlete.start) else {
        return DayView {
            date,
            entries: Vec::new(),
            empty: Some(Empty::Rest),
        };
    };
    let mut entries: Vec<Entry<'a>> = Vec::new();
    let natural = schedule::locate(&data.program, start, date);
    let mut empty = None;
    match natural {
        Place::Session(slot) => {
            let moved = moves_of(data, athlete).find(|m| {
                data.program.cycles[slot.cycle].id == m.cycle
                    && slot.week + 1 == m.week as usize
                    && slot.dow == m.dow
            });
            match moved.and_then(|m| dates::parse(&m.to)) {
                Some(to) if to != date => entries.push(Entry::MovedOut { slot, to }),
                _ => entries.push(Entry::Session {
                    slot,
                    moved_from: None,
                }),
            }
        }
        Place::BeforeStart(s) => empty = Some(Empty::BeforeStart(s)),
        Place::NotLoaded { cycle, week } => empty = Some(Empty::NotLoaded { cycle, week }),
        Place::Rest { .. } => empty = Some(Empty::Rest),
    }
    for m in moves_of(data, athlete) {
        if dates::parse(&m.to) != Some(date) {
            continue;
        }
        let Some(slot) = slot_of(data, m) else {
            continue;
        };
        let from = schedule::natural_day(start, slot.cycle, slot.week, slot.dow);
        if from == date {
            continue;
        }
        entries.push(Entry::Session {
            slot,
            moved_from: Some(from),
        });
    }
    entries.sort_by_key(|e| match e {
        Entry::Session { slot, .. } | Entry::MovedOut { slot, .. } => order_key(slot),
    });
    if entries.iter().any(|e| matches!(e, Entry::Session { .. })) {
        empty = None;
    }
    DayView { date, entries, empty }
}

/// Checks that a move names a session that exists and a real, different date.
pub fn check_move(data: &Data, rec: &MoveRec) -> Result<(), String> {
    let athlete = data
        .athlete(rec.athlete)
        .ok_or_else(|| format!("There is no athlete {}.", rec.athlete))?;
    let to = dates::parse(&rec.to).ok_or_else(|| format!("\"{}\" is not a date.", rec.to))?;
    let slot = slot_of(data, rec).ok_or_else(|| "That session is not in the program.".to_string())?;
    let start = dates::parse(&athlete.start).ok_or_else(|| "The athlete has no start date.".to_string())?;
    if to == schedule::natural_day(start, slot.cycle, slot.week, slot.dow) {
        return Err("That is the session's own date; undo the move instead.".into());
    }
    Ok(())
}

/// The move on a session for an athlete, if any.
pub fn move_of<'a>(data: &'a Data, athlete: u64, cycle: &str, week: u32, dow: u8) -> Option<&'a MoveRec> {
    data.moves
        .iter()
        .find(|m| m.athlete == athlete && m.cycle == cycle && m.week == week && m.dow == dow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{apply, Request};
    use crate::testing::sample_data;
    use serde_json::json;

    fn d(s: &str) -> i64 {
        dates::parse(s).unwrap()
    }

    fn mv(data: &mut Data, athlete: u64, week: u32, dow: u8, to: &str) -> Result<(), String> {
        apply(
            data,
            &Request::new(
                "move_session",
                json!({"athlete": athlete, "cycle": "c1", "week": week, "dow": dow, "to": to}),
            ),
        )
    }

    // Athlete 1 starts Monday 2026-11-30, so cycle 1 week 1 Tuesday is 2026-12-01.

    #[test]
    fn a_moved_session_shows_on_its_new_date_marked_and_the_old_date_says_where_it_went() {
        let mut data = sample_data();
        mv(&mut data, 1, 1, 1, "2026-12-03").unwrap();
        let a = data.athlete(1).unwrap().clone();
        let old = day_view(&data, &a, d("2026-12-01"));
        assert!(matches!(old.entries[..], [Entry::MovedOut { to, .. }] if to == d("2026-12-03")));
        assert!(!old.has_session());
        let new = day_view(&data, &a, d("2026-12-03"));
        // The moved Tuesday lift session comes before Thursday's own jump session: program order.
        assert_eq!(new.entries.len(), 2);
        match (&new.entries[0], &new.entries[1]) {
            (
                Entry::Session { slot: first, moved_from: Some(from) },
                Entry::Session { slot: second, moved_from: None },
            ) => {
                assert_eq!(first.day.dow, "tue");
                assert_eq!(second.day.dow, "thu");
                assert_eq!(*from, d("2026-12-01"));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn undoing_a_move_puts_the_session_back_on_its_own_date() {
        let mut data = sample_data();
        mv(&mut data, 1, 1, 1, "2026-12-03").unwrap();
        apply(
            &mut data,
            &Request::new("undo_move", json!({"athlete": 1, "cycle": "c1", "week": 1, "dow": 1})),
        )
        .unwrap();
        let a = data.athlete(1).unwrap().clone();
        let back = day_view(&data, &a, d("2026-12-01"));
        assert!(matches!(back.entries[..], [Entry::Session { moved_from: None, .. }]));
        assert_eq!(day_view(&data, &a, d("2026-12-03")).entries.len(), 1);
    }

    #[test]
    fn two_sessions_on_one_date_are_both_listed_in_program_order() {
        let mut data = sample_data();
        // Move Monday's session (week 1) onto Tuesday: Monday's comes first.
        mv(&mut data, 1, 1, 0, "2026-12-01").unwrap();
        let a = data.athlete(1).unwrap().clone();
        let day = day_view(&data, &a, d("2026-12-01"));
        let dows: Vec<_> = day
            .entries
            .iter()
            .map(|e| match e {
                Entry::Session { slot, .. } => slot.day.dow.clone(),
                Entry::MovedOut { .. } => "out".to_string(),
            })
            .collect();
        assert_eq!(dows, vec!["mon", "tue"]);
    }

    #[test]
    fn moving_a_session_again_replaces_the_first_move() {
        let mut data = sample_data();
        mv(&mut data, 1, 1, 1, "2026-12-03").unwrap();
        mv(&mut data, 1, 1, 1, "2026-12-04").unwrap();
        assert_eq!(data.moves.len(), 1);
        let a = data.athlete(1).unwrap().clone();
        assert_eq!(day_view(&data, &a, d("2026-12-03")).entries.len(), 1);
        assert_eq!(day_view(&data, &a, d("2026-12-04")).entries.len(), 2);
    }

    #[test]
    fn a_move_belongs_to_one_athlete() {
        let mut data = sample_data();
        mv(&mut data, 1, 1, 1, "2026-12-03").unwrap();
        let other = data.athlete(2).unwrap().clone();
        // Athlete 2 starts the same Monday and keeps the session on its own date.
        let day = day_view(&data, &other, d("2026-12-01"));
        assert!(matches!(day.entries[..], [Entry::Session { moved_from: None, .. }]));
    }

    #[test]
    fn a_move_to_the_sessions_own_date_or_to_a_missing_session_is_refused() {
        let mut data = sample_data();
        assert!(mv(&mut data, 1, 1, 1, "2026-12-01").unwrap_err().contains("undo"));
        // Week three of cycle one is not loaded.
        assert!(mv(&mut data, 1, 3, 0, "2026-12-20").unwrap_err().contains("not in the program"));
        assert!(mv(&mut data, 1, 1, 1, "2026-12-32").unwrap_err().contains("not a date"));
        assert!(mv(&mut data, 9, 1, 1, "2026-12-03").unwrap_err().contains("no athlete"));
        assert!(data.moves.is_empty());
    }

    #[test]
    fn a_date_with_no_session_says_why() {
        let data = sample_data();
        let a = data.athlete(1).unwrap().clone();
        assert_eq!(day_view(&data, &a, d("2026-11-29")).empty, Some(Empty::BeforeStart(d("2026-11-30"))));
        assert_eq!(day_view(&data, &a, d("2026-12-06")).empty, Some(Empty::Rest));
        assert_eq!(
            day_view(&data, &a, d("2026-12-14")).empty,
            Some(Empty::NotLoaded { cycle: 0, week: 2 })
        );
    }
}
