//! Maxes and working weights, in whole pounds and integer arithmetic.
//!
//! A max is (athlete, lift, weight, date). The max a date uses is the latest
//! entry on or before it for that athlete and lift; on the same date, the one
//! entered last. A working weight is the max times a percentage, rounded to the
//! nearest 5 lb, halves up.

use crate::data::{Data, MaxRec};
use crate::dates;

/// The step working weights are rounded to, in lb.
pub const ROUND_TO: u64 = 5;

/// `max` times `pct` percent, rounded to the nearest 5 lb, halves up.
pub fn working_weight(max: u32, pct: u32) -> u32 {
    // value = max * pct / 100; add half a step (2.5 lb = 250/100), then floor to steps of 5.
    let scaled = u64::from(max) * u64::from(pct) + 250;
    (scaled / (100 * ROUND_TO) * ROUND_TO) as u32
}

/// The max an athlete's date uses for a lift: the latest on or before `on`.
pub fn current_max<'a>(data: &'a Data, athlete: u64, lift: &str, on: i64) -> Option<&'a MaxRec> {
    data.maxes
        .iter()
        .filter(|m| m.athlete == athlete && m.lift == lift)
        .filter(|m| dates::parse(&m.date).is_some_and(|d| d <= on))
        .max_by_key(|m| (dates::parse(&m.date).unwrap_or(i64::MIN), m.id))
}

/// Every max an athlete recorded for a lift, newest first.
pub fn history<'a>(data: &'a Data, athlete: u64, lift: &str) -> Vec<&'a MaxRec> {
    let mut all: Vec<&MaxRec> = data
        .maxes
        .iter()
        .filter(|m| m.athlete == athlete && m.lift == lift)
        .collect();
    all.sort_by_key(|m| std::cmp::Reverse((dates::parse(&m.date).unwrap_or(i64::MIN), m.id)));
    all
}

/// The weight for a prescription of `pct` percent of lift `of`, for an
/// athlete on a date. `None` when the athlete has no max that date uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Weight {
    pub weight: u32,
    pub max: u32,
    /// The lift whose max was used (the lift itself, or the lift it takes its max from).
    pub max_lift: String,
}

pub fn weight_for(data: &Data, athlete: u64, of: &str, pct: u32, on: i64) -> Option<Weight> {
    let max_lift = data.program.max_lift(of).to_string();
    let found = current_max(data, athlete, &max_lift, on)?;
    Some(Weight {
        weight: working_weight(found.weight, pct),
        max: found.weight,
        max_lift,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::sample_data;

    #[test]
    fn working_weights_round_to_the_nearest_five_as_worked_by_hand() {
        // 305 x 75% = 228.75 -> 230
        assert_eq!(working_weight(305, 75), 230);
        // 195 x 110% = 214.5 -> 215
        assert_eq!(working_weight(195, 110), 215);
        // 195 x 60% = 117 -> 115
        assert_eq!(working_weight(195, 60), 115);
        // 215 x 65% = 139.75 -> 140
        assert_eq!(working_weight(215, 65), 140);
    }

    #[test]
    fn exactly_halfway_rounds_up() {
        // 10 x 75% = 7.5, halfway between 5 and 10 -> 10
        assert_eq!(working_weight(10, 75), 10);
        // 10 x 25% = 2.5, halfway between 0 and 5 -> 5
        assert_eq!(working_weight(10, 25), 5);
        // 10 x 24% = 2.4 -> 0
        assert_eq!(working_weight(10, 24), 0);
        // 100 x 52% = 52 -> 50; 100 x 53% = 53 -> 55
        assert_eq!(working_weight(100, 52), 50);
        assert_eq!(working_weight(100, 53), 55);
    }

    fn d(s: &str) -> i64 {
        dates::parse(s).unwrap()
    }

    #[test]
    fn the_max_a_date_uses_is_the_latest_on_or_before_it() {
        let data = sample_data();
        // Athlete 1 has anvil_squat 285 on 2026-10-05 and 305 on 2026-11-02.
        assert_eq!(current_max(&data, 1, "anvil_squat", d("2026-12-01")).unwrap().weight, 305);
        assert_eq!(current_max(&data, 1, "anvil_squat", d("2026-11-02")).unwrap().weight, 305);
        assert_eq!(current_max(&data, 1, "anvil_squat", d("2026-11-01")).unwrap().weight, 285);
        assert!(current_max(&data, 1, "anvil_squat", d("2026-10-04")).is_none());
    }

    #[test]
    fn a_max_dated_after_the_session_is_not_used() {
        let mut data = sample_data();
        data.maxes.push(MaxRec {
            id: 99,
            athlete: 1,
            lift: "anvil_squat".into(),
            weight: 400,
            date: "2026-12-10".into(),
        });
        assert_eq!(current_max(&data, 1, "anvil_squat", d("2026-12-09")).unwrap().weight, 305);
        assert_eq!(current_max(&data, 1, "anvil_squat", d("2026-12-10")).unwrap().weight, 400);
        let before = weight_for(&data, 1, "anvil_squat", 75, d("2026-12-09")).unwrap();
        let after = weight_for(&data, 1, "anvil_squat", 75, d("2026-12-10")).unwrap();
        assert_eq!((before.weight, after.weight), (230, 300));
    }

    #[test]
    fn two_maxes_on_one_date_use_the_one_entered_last() {
        let mut data = sample_data();
        for (id, w) in [(50, 310), (51, 300)] {
            data.maxes.push(MaxRec {
                id,
                athlete: 1,
                lift: "anvil_squat".into(),
                weight: w,
                date: "2026-11-20".into(),
            });
        }
        assert_eq!(current_max(&data, 1, "anvil_squat", d("2026-11-21")).unwrap().weight, 300);
    }

    #[test]
    fn a_lift_that_takes_its_max_from_another_uses_that_lifts_max() {
        let data = sample_data();
        // Hang rope clean takes the rope clean max (195 for athlete 1): 60% = 117 -> 115.
        let w = weight_for(&data, 1, "rope_hang_clean", 60, d("2026-12-01")).unwrap();
        assert_eq!((w.weight, w.max, w.max_lift.as_str()), (115, 195, "rope_clean"));
        // Single kettle hinge takes the kettle hinge max (215): 65% = 139.75 -> 140.
        let w = weight_for(&data, 1, "kettle_single", 65, d("2026-12-01")).unwrap();
        assert_eq!((w.weight, w.max), (140, 215));
    }

    #[test]
    fn no_max_means_no_weight() {
        let data = sample_data();
        // Athlete 3 has recorded nothing.
        assert!(weight_for(&data, 3, "anvil_squat", 75, d("2026-12-01")).is_none());
        // Another athlete's max is not borrowed.
        assert!(current_max(&data, 3, "anvil_squat", d("2026-12-01")).is_none());
    }

    #[test]
    fn history_lists_newest_first() {
        let data = sample_data();
        let h: Vec<u32> = history(&data, 1, "anvil_squat").iter().map(|m| m.weight).collect();
        assert_eq!(h, vec![305, 285]);
    }
}
