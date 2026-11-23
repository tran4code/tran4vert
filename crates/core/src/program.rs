//! The program model: lifts, a library of reusable blocks, coach notes, and
//! cycles of weeks of days of blocks. The format is the one in
//! `private/program/program.json`; `tests/fixtures/sample_program.json` has
//! the same shape with invented content.
//!
//! Fields the model does not name (`maxes_from_pdf`, `original_date` and the
//! file's own notes to itself) are not read, so they never reach a screen.
//! Names that point nowhere are an error naming the id.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Days of a week as the file spells them. Sunday is not in the file; it is a
/// rest day.
pub const DOW_KEYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
/// The session kinds a day may have.
pub const KINDS: [&str; 5] = ["jump", "lift", "general", "iso", "rest"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiftDef {
    pub name: String,
    #[serde(default)]
    pub tracks_max: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_from: Option<String>,
}

/// A reusable block: a circuit, a check-in or a hold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LibraryItem {
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub items: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rounds: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dose: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sets: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secs: Option<u32>,
}

/// "The last `last` sets are `pct` percent of the max of lift `of`."
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pct {
    pub last: u32,
    pub pct: u32,
    pub of: String,
}

/// One block of a day. Which fields are set says which kind it is: `ref`,
/// `lift`, `text`, or a bare `note`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Block {
    #[serde(default, rename = "ref", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dose: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lift: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sets: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reps: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secs: Option<u32>,
    #[serde(default)]
    pub each_side: bool,
    #[serde(default)]
    pub hold: bool,
    #[serde(default)]
    pub amap: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pct: Option<Pct>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test: Option<String>,
}

/// What a block is, decided from its fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Ref,
    Lift,
    Text,
    Note,
}

impl Block {
    pub fn kind(&self) -> Option<BlockKind> {
        if self.reference.is_some() {
            Some(BlockKind::Ref)
        } else if self.lift.is_some() {
            Some(BlockKind::Lift)
        } else if self.text.is_some() {
            Some(BlockKind::Text)
        } else if self.note.is_some() {
            Some(BlockKind::Note)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Day {
    pub dow: String,
    pub kind: String,
    #[serde(default)]
    pub blocks: Vec<Block>,
}

impl Day {
    /// 0 for Monday up to 6 for Sunday.
    pub fn dow_index(&self) -> u8 {
        DOW_KEYS.iter().position(|k| *k == self.dow).unwrap_or(0) as u8
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Week {
    pub week: u32,
    #[serde(default)]
    pub missing: bool,
    #[serde(default)]
    pub days: Vec<Day>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cycle {
    pub id: String,
    pub name: String,
    pub emphasis: String,
    #[serde(default)]
    /// One theme per week; the file has `null` for a week whose theme it does not know.
    pub week_themes: Vec<Option<String>>,
    #[serde(default)]
    pub cycle_note: String,
    pub weeks: Vec<Week>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Program {
    #[serde(default)]
    pub lifts: BTreeMap<String, LiftDef>,
    #[serde(default)]
    pub library: BTreeMap<String, LibraryItem>,
    #[serde(default)]
    pub notes: BTreeMap<String, String>,
    #[serde(default)]
    pub cycles: Vec<Cycle>,
    #[serde(default)]
    pub not_captured: Vec<String>,
}

/// The note id a block's text points at, if it is a `@name` reference.
pub fn note_reference(text: &str) -> Option<&str> {
    text.strip_prefix('@')
}

impl Program {
    /// Parses a program file and checks every name in it points somewhere.
    pub fn parse(json: &str) -> Result<Program, String> {
        let program: Program =
            serde_json::from_str(json).map_err(|e| format!("The program could not be read: {e}"))?;
        program.check()?;
        Ok(program)
    }

    /// The text a note field shows: a `@name` becomes the program's note of
    /// that name (or the cycle's own note for `@cycle_note`); other text is
    /// shown as written.
    pub fn note_text(&self, cycle: &Cycle, text: &str) -> String {
        match note_reference(text) {
            Some("cycle_note") => cycle.cycle_note.clone(),
            Some(name) => self.notes.get(name).cloned().unwrap_or_default(),
            None => text.to_string(),
        }
    }

    /// The lift whose max a lift uses: itself, or the lift its `max_from` names.
    pub fn max_lift<'a>(&'a self, lift: &'a str) -> &'a str {
        match self.lifts.get(lift).and_then(|l| l.max_from.as_deref()) {
            Some(other) => other,
            None => lift,
        }
    }

    pub fn lift_name(&self, lift: &str) -> String {
        self.lifts
            .get(lift)
            .map(|l| l.name.clone())
            .unwrap_or_else(|| lift.to_string())
    }

    /// Lifts an athlete records a max for, in id order.
    pub fn max_lifts(&self) -> Vec<&str> {
        self.lifts
            .iter()
            .filter(|(_, l)| l.tracks_max)
            .map(|(id, _)| id.as_str())
            .collect()
    }

    fn check(&self) -> Result<(), String> {
        for (id, lift) in &self.lifts {
            if let Some(from) = &lift.max_from {
                if !self.lifts.contains_key(from) {
                    return Err(format!("Lift \"{id}\" takes its max from \"{from}\", which is not a lift."));
                }
            }
        }
        for (id, item) in &self.library {
            if !["circuit", "checkin", "hold"].contains(&item.kind.as_str()) {
                return Err(format!(
                    "Library item \"{id}\" has the unknown kind \"{}\".",
                    item.kind
                ));
            }
        }
        for (c, cycle) in self.cycles.iter().enumerate() {
            for (w, week) in cycle.weeks.iter().enumerate() {
                if week.week as usize != w + 1 {
                    return Err(format!(
                        "Cycle \"{}\" lists week {} in position {}.",
                        cycle.id,
                        week.week,
                        w + 1
                    ));
                }
                for day in &week.days {
                    self.check_day(&self.cycles[c], week, day)?;
                }
            }
        }
        Ok(())
    }

    fn check_day(&self, cycle: &Cycle, week: &Week, day: &Day) -> Result<(), String> {
        let place = format!("cycle \"{}\" week {} {}", cycle.id, week.week, day.dow);
        if !DOW_KEYS.contains(&day.dow.as_str()) {
            return Err(format!("In {place}: \"{}\" is not a day of the week.", day.dow));
        }
        if !KINDS.contains(&day.kind.as_str()) {
            return Err(format!("In {place}: \"{}\" is not a session kind.", day.kind));
        }
        for block in &day.blocks {
            if block.kind().is_none() {
                return Err(format!("In {place}: a block has no ref, lift, text or note."));
            }
            if let Some(r) = &block.reference {
                if !self.library.contains_key(r) {
                    return Err(format!("In {place}: \"{r}\" is not in the library."));
                }
            }
            if let Some(l) = &block.lift {
                if !self.lifts.contains_key(l) {
                    return Err(format!("In {place}: \"{l}\" is not a lift."));
                }
            }
            if let Some(p) = &block.pct {
                if !self.lifts.contains_key(&p.of) {
                    return Err(format!("In {place}: \"{}\" is not a lift.", p.of));
                }
            }
            if let Some(note) = &block.note {
                self.check_note(&place, cycle, note)?;
            }
        }
        Ok(())
    }

    fn check_note(&self, place: &str, _cycle: &Cycle, note: &str) -> Result<(), String> {
        if let Some(name) = note_reference(note) {
            if name != "cycle_note" && !self.notes.contains_key(name) {
                return Err(format!("In {place}: \"@{name}\" is not one of the program's notes."));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::testing::SAMPLE;

    #[test]
    fn the_sample_program_parses_with_its_cycles_weeks_and_days() {
        let p = Program::parse(SAMPLE).unwrap();
        assert_eq!(p.cycles.len(), 2);
        assert_eq!(p.cycles[0].weeks.len(), 4);
        assert_eq!(p.cycles[0].weeks.iter().filter(|w| w.missing).count(), 2);
        assert_eq!(p.cycles[0].weeks[0].days.len(), 6);
        assert_eq!(p.cycles[0].weeks[1].days.len(), 3);
        assert_eq!(p.cycles[1].weeks[0].days.len(), 2);
        assert_eq!(p.not_captured.len(), 3);
    }

    #[test]
    fn a_week_theme_the_file_leaves_null_is_kept_as_none() {
        let p = Program::parse(SAMPLE).unwrap();
        assert_eq!(p.cycles[0].week_themes[0].as_deref(), Some("Learn the lifts"));
        assert_eq!(p.cycles[1].week_themes[1], None);
        assert_eq!(p.cycles[1].week_themes.len(), 4);
    }

    #[test]
    fn blocks_are_told_apart_by_the_fields_they_carry() {
        let p = Program::parse(SAMPLE).unwrap();
        let mon = &p.cycles[0].weeks[0].days[0];
        let kinds: Vec<_> = mon.blocks.iter().map(|b| b.kind().unwrap()).collect();
        assert_eq!(
            kinds,
            vec![BlockKind::Ref, BlockKind::Note, BlockKind::Lift, BlockKind::Lift]
        );
        let wed = &p.cycles[0].weeks[0].days[2];
        assert_eq!(wed.blocks[1].kind(), Some(BlockKind::Text));
        assert_eq!(wed.blocks[0].dose.as_deref(), Some("2 sets of 20 seconds"));
        let clean = mon.blocks[2].pct.as_ref().unwrap();
        assert_eq!((clean.last, clean.pct, clean.of.as_str()), (3, 75, "rope_clean"));
    }

    #[test]
    fn fields_the_model_does_not_name_never_leave_the_core() {
        let p = Program::parse(SAMPLE).unwrap();
        let out = serde_json::to_string(&p).unwrap();
        assert!(!out.contains("maxes_from_pdf"));
        assert!(!out.contains("999"));
        assert!(!out.contains("original_date"));
        assert!(!out.contains("blocks_legend"));
    }

    #[test]
    fn a_note_reference_resolves_to_the_programs_note_or_the_cycles_own() {
        let p = Program::parse(SAMPLE).unwrap();
        let c = &p.cycles[0];
        assert_eq!(p.note_text(c, "@walk_note"), "Walk for twenty easy minutes.");
        assert_eq!(p.note_text(c, "@cycle_note"), "Work through the weeks in order.");
        assert_eq!(p.note_text(&p.cycles[1], "@cycle_note"), "Keep every rep fast.");
        assert_eq!(p.note_text(c, "Fast feet."), "Fast feet.");
    }

    #[test]
    fn a_lift_that_takes_its_max_from_another_resolves_to_it() {
        let p = Program::parse(SAMPLE).unwrap();
        assert_eq!(p.max_lift("rope_hang_clean"), "rope_clean");
        assert_eq!(p.max_lift("rope_clean"), "rope_clean");
        assert_eq!(p.max_lifts(), vec!["anvil_squat", "kettle_hinge", "rope_clean", "stone_press"]);
    }

    fn with(old: &str, new: &str) -> String {
        assert!(SAMPLE.contains(old), "the sample no longer holds {old}");
        SAMPLE.replacen(old, new, 1)
    }

    #[test]
    fn a_name_that_points_nowhere_is_an_error_naming_it() {
        let e = Program::parse(&with("\"ref\": \"wake_up\"", "\"ref\": \"wake_down\"")).unwrap_err();
        assert!(e.contains("\"wake_down\" is not in the library"), "{e}");
        let e = Program::parse(&with("\"lift\": \"rope_clean\"", "\"lift\": \"rope_dirty\"")).unwrap_err();
        assert!(e.contains("\"rope_dirty\" is not a lift"), "{e}");
        let e = Program::parse(&with("\"of\": \"rope_clean\"", "\"of\": \"rope_dirty\"")).unwrap_err();
        assert!(e.contains("\"rope_dirty\" is not a lift"), "{e}");
        let e = Program::parse(&with("@walk_note", "@walk_nope")).unwrap_err();
        assert!(e.contains("\"@walk_nope\""), "{e}");
        let e = Program::parse(&with("\"max_from\": \"rope_clean\"", "\"max_from\": \"rope_none\"")).unwrap_err();
        assert!(e.contains("\"rope_none\""), "{e}");
    }

    #[test]
    fn a_week_out_of_order_or_a_bad_day_is_refused() {
        let e = Program::parse(&with("\"week\": 2,", "\"week\": 3,")).unwrap_err();
        assert!(e.contains("lists week 3 in position 2"), "{e}");
        let e = Program::parse(&with("\"dow\": \"sat\"", "\"dow\": \"someday\"")).unwrap_err();
        assert!(e.contains("not a day of the week"), "{e}");
        let e = Program::parse(&with("\"kind\": \"iso\"", "\"kind\": \"nap\"")).unwrap_err();
        assert!(e.contains("not a session kind"), "{e}");
    }

    #[test]
    fn text_that_is_not_a_program_is_refused() {
        assert!(Program::parse("not json").unwrap_err().contains("could not be read"));
    }
}
