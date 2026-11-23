//! Shared by the unit tests: the invented sample program and sample data.

use crate::data::{AthleteRec, Data, MaxRec, NoteRec};
use crate::program::Program;

pub const SAMPLE: &str = include_str!("../../../tests/fixtures/sample_program.json");

/// Three invented athletes who all start on Monday 2026-11-30. Athlete 1 has
/// maxes (two for the anvil squat, so the latest-on-or-before rule shows),
/// athlete 2 has one, athlete 3 none. Two voice notes sit on the anvil squat
/// and one on the rope clean.
pub fn sample_data() -> Data {
    let athlete = |id: u64, name: &str| AthleteRec {
        id,
        name: name.to_string(),
        start: "2026-11-30".to_string(),
        goal: None,
    };
    let max = |id: u64, athlete: u64, lift: &str, weight: u32, date: &str| MaxRec {
        id,
        athlete,
        lift: lift.to_string(),
        weight,
        date: date.to_string(),
    };
    let note = |id: u64, lift: &str, title: &str, seconds: u32, date: &str| NoteRec {
        id,
        lift: lift.to_string(),
        title: title.to_string(),
        seconds,
        date: date.to_string(),
    };
    Data {
        program: Program::parse(SAMPLE).unwrap(),
        athletes: vec![athlete(1, "Sam"), athlete(2, "Jo"), athlete(3, "Pat")],
        maxes: vec![
            max(1, 1, "anvil_squat", 285, "2026-10-05"),
            max(2, 1, "anvil_squat", 305, "2026-11-02"),
            max(3, 1, "rope_clean", 195, "2026-11-02"),
            max(4, 1, "kettle_hinge", 215, "2026-11-02"),
            max(5, 2, "anvil_squat", 225, "2026-11-09"),
        ],
        moves: Vec::new(),
        notes: vec![
            note(1, "anvil_squat", "Breathing", 41, "2026-11-10"),
            note(2, "anvil_squat", "Depth cue", 65, "2026-11-20"),
            note(3, "rope_clean", "Fast elbows", 30, "2026-11-12"),
        ],
        cycle_notes: Vec::new(),
        role: "athlete".to_string(),
        me: None,
        as_of: "2026-11-23".to_string(),
    }
}
