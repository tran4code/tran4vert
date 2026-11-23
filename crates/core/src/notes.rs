//! Voice notes. A note belongs to one lift and shows on every block of that
//! lift, in every week and for every athlete; it shows on no other lift, so a
//! note on one clean does not show on a variant that takes its max from it.

use crate::data::{Data, NoteRec, MAX_NOTE_SECONDS, MAX_TITLE};
use crate::dates;
use crate::program::Program;

/// The notes on a lift, newest first.
pub fn notes_for<'a>(data: &'a Data, lift: &str) -> Vec<&'a NoteRec> {
    let mut found: Vec<&NoteRec> = data.notes.iter().filter(|n| n.lift == lift).collect();
    found.sort_by_key(|n| std::cmp::Reverse((dates::parse(&n.date).unwrap_or(i64::MIN), n.id)));
    found
}

/// A length as minutes and two-digit seconds, `0:42`.
pub fn length(seconds: u32) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// Checks a new note: a known lift, a title, a length in range.
pub fn check_note(program: &Program, lift: &str, title: &str, seconds: u32) -> Result<(), String> {
    if !program.lifts.contains_key(lift) {
        return Err(format!("\"{lift}\" is not a lift."));
    }
    if title.trim().is_empty() {
        return Err("A voice note needs a title.".into());
    }
    if title.chars().count() > MAX_TITLE {
        return Err("The title is too long.".into());
    }
    if seconds == 0 || seconds > MAX_NOTE_SECONDS {
        return Err("The voice note's length is out of range.".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::sample_data;

    #[test]
    fn a_note_belongs_to_its_exact_lift_and_no_other() {
        let data = sample_data();
        let on_squat: Vec<&str> = notes_for(&data, "anvil_squat").iter().map(|n| n.title.as_str()).collect();
        assert_eq!(on_squat, vec!["Depth cue", "Breathing"]);
        // A note on rope_clean does not show on the variant that takes its max from it.
        assert_eq!(notes_for(&data, "rope_clean").len(), 1);
        assert!(notes_for(&data, "rope_hang_clean").is_empty());
        assert!(notes_for(&data, "stone_press").is_empty());
    }

    #[test]
    fn lengths_read_as_minutes_and_seconds() {
        assert_eq!(length(0), "0:00");
        assert_eq!(length(7), "0:07");
        assert_eq!(length(42), "0:42");
        assert_eq!(length(60), "1:00");
        assert_eq!(length(125), "2:05");
    }

    #[test]
    fn a_note_needs_a_known_lift_a_title_and_a_length_in_range() {
        let p = &sample_data().program;
        assert!(check_note(p, "anvil_squat", "Depth", 30).is_ok());
        assert!(check_note(p, "nope", "Depth", 30).unwrap_err().contains("not a lift"));
        assert!(check_note(p, "anvil_squat", "  ", 30).unwrap_err().contains("title"));
        assert!(check_note(p, "anvil_squat", &"x".repeat(61), 30).unwrap_err().contains("too long"));
        assert!(check_note(p, "anvil_squat", "Depth", 0).unwrap_err().contains("length"));
        assert!(check_note(p, "anvil_squat", "Depth", 181).unwrap_err().contains("length"));
        assert!(check_note(p, "anvil_squat", "Depth", 180).is_ok());
    }
}
