//! The shared data the page fetches in one document: the program, the three
//! athletes, their maxes, moved sessions and voice-note list. The core never
//! fetches; the page hands the document over as text, and a change comes back
//! from the core as a `Request` for the page to send.
//!
//! `apply` carries out a request on the data in memory. Demo mode uses it so
//! the page keeps changes without a database, and the tests use it to check
//! what a request means.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::dates;
use crate::program::Program;

/// The heaviest weight the app accepts, in lb.
pub const MAX_WEIGHT: u32 = 1500;
/// The longest voice note, in seconds. The page's recorder stops here.
pub const MAX_NOTE_SECONDS: u32 = 180;
/// The longest voice note title, in characters.
pub const MAX_TITLE: usize = 60;
/// The verticals an athlete can tap to sign in, in inches. The coach sets one
/// for each athlete. It is a light lock among friends, not a strong one.
pub const GOALS: [u32; 16] = [30, 32, 34, 36, 38, 40, 42, 44, 46, 48, 50, 52, 54, 56, 58, 60];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AthleteRec {
    pub id: u64,
    pub name: String,
    /// The Monday of cycle 1 week 1, as `YYYY-MM-DD`.
    pub start: String,
    /// The vertical this athlete taps to sign in. Only the coach is sent it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<u32>,
}

/// A name to tap before anyone is signed in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NameRec {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaxRec {
    pub id: u64,
    pub athlete: u64,
    pub lift: String,
    pub weight: u32,
    pub date: String,
}

/// A session moved to another date: the session is named by cycle id, week
/// number (from 1) and weekday (0 for Monday).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveRec {
    pub athlete: u64,
    pub cycle: String,
    pub week: u32,
    pub dow: u8,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteRec {
    pub id: u64,
    pub lift: String,
    pub title: String,
    pub seconds: u32,
    pub date: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CycleNoteRec {
    pub cycle: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Data {
    pub program: Program,
    #[serde(default)]
    pub athletes: Vec<AthleteRec>,
    #[serde(default)]
    pub maxes: Vec<MaxRec>,
    #[serde(default)]
    pub moves: Vec<MoveRec>,
    #[serde(default)]
    pub notes: Vec<NoteRec>,
    #[serde(default)]
    pub cycle_notes: Vec<CycleNoteRec>,
    /// `coach` when the passcode given was the coach's, else `athlete`.
    #[serde(default)]
    pub role: String,
    /// The athlete who signed in by tapping a goal; none for the coach and the demo.
    #[serde(default)]
    pub me: Option<u64>,
    /// The date the database answered on; shown when the phone is offline.
    #[serde(default)]
    pub as_of: String,
}

impl Data {
    /// Reads the document `get_state` returns, and checks it.
    pub fn parse(json: &str) -> Result<Data, String> {
        let data: Data =
            serde_json::from_str(json).map_err(|e| format!("The shared data could not be read: {e}"))?;
        // The program is read again from its own text so it is checked.
        let program_text = serde_json::to_string(&data.program).map_err(|e| e.to_string())?;
        Program::parse(&program_text)?;
        for a in &data.athletes {
            if dates::parse(&a.start).is_none() {
                return Err(format!(
                    "An athlete has the start date \"{}\", which is not a date.",
                    a.start
                ));
            }
        }
        Ok(data)
    }

    pub fn athlete(&self, id: u64) -> Option<&AthleteRec> {
        self.athletes.iter().find(|a| a.id == id)
    }

    pub fn is_coach(&self) -> bool {
        self.role == "coach"
    }
}

/// A call the page makes to the database for the core.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Request {
    pub rpc: String,
    pub args: Value,
}

impl Request {
    pub fn new(rpc: &str, args: Value) -> Request {
        Request {
            rpc: rpc.to_string(),
            args,
        }
    }
}

fn text<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("The request lacks \"{key}\"."))
}

fn number(args: &Value, key: &str) -> Result<u64, String> {
    args.get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("The request lacks the number \"{key}\"."))
}

fn next_id(ids: impl Iterator<Item = u64>) -> u64 {
    ids.max().unwrap_or(0) + 1
}

/// Carries out a request on the data. Returns what is wrong when the request
/// does not make sense; the database does the same checks.
pub fn apply(data: &mut Data, request: &Request) -> Result<(), String> {
    let a = &request.args;
    match request.rpc.as_str() {
        "add_max" => {
            let athlete = number(a, "athlete")?;
            let lift = text(a, "lift")?.to_string();
            let weight =
                u32::try_from(number(a, "weight")?).map_err(|_| "The weight is too large.".to_string())?;
            let date = text(a, "date")?.to_string();
            if data.athlete(athlete).is_none() {
                return Err(format!("There is no athlete {athlete}."));
            }
            if !data.program.lifts.contains_key(&lift) {
                return Err(format!("\"{lift}\" is not a lift."));
            }
            if weight == 0 || weight > MAX_WEIGHT {
                return Err("The weight is out of range.".into());
            }
            if dates::parse(&date).is_none() {
                return Err(format!("\"{date}\" is not a date."));
            }
            let id = next_id(data.maxes.iter().map(|m| m.id));
            data.maxes.push(MaxRec {
                id,
                athlete,
                lift,
                weight,
                date,
            });
        }
        "delete_max" => {
            let id = number(a, "id")?;
            data.maxes.retain(|m| m.id != id);
        }
        "move_session" => {
            let rec = MoveRec {
                athlete: number(a, "athlete")?,
                cycle: text(a, "cycle")?.to_string(),
                week: u32::try_from(number(a, "week")?).map_err(|_| "The week is too large.".to_string())?,
                dow: u8::try_from(number(a, "dow")?).map_err(|_| "The day is too large.".to_string())?,
                to: text(a, "to")?.to_string(),
            };
            crate::moves::check_move(data, &rec)?;
            data.moves.retain(|m| !same_session(m, &rec));
            data.moves.push(rec);
        }
        "undo_move" => {
            let rec = MoveRec {
                athlete: number(a, "athlete")?,
                cycle: text(a, "cycle")?.to_string(),
                week: u32::try_from(number(a, "week")?).map_err(|_| "The week is too large.".to_string())?,
                dow: u8::try_from(number(a, "dow")?).map_err(|_| "The day is too large.".to_string())?,
                to: String::new(),
            };
            data.moves.retain(|m| !same_session(m, &rec));
        }
        "add_note" => {
            let lift = text(a, "lift")?.to_string();
            let title = text(a, "title")?.trim().to_string();
            let seconds =
                u32::try_from(number(a, "seconds")?).map_err(|_| "The length is too large.".to_string())?;
            let date = text(a, "date")?.to_string();
            crate::notes::check_note(&data.program, &lift, &title, seconds)?;
            let id = next_id(data.notes.iter().map(|n| n.id));
            data.notes.push(NoteRec {
                id,
                lift,
                title,
                seconds,
                date,
            });
        }
        "delete_note" => {
            let id = number(a, "id")?;
            data.notes.retain(|n| n.id != id);
        }
        "set_start_date" => {
            let athlete = number(a, "athlete")?;
            let start = text(a, "start")?.to_string();
            let day = dates::parse(&start).ok_or_else(|| format!("\"{start}\" is not a date."))?;
            if !crate::schedule::is_start_ok(day) {
                return Err(format!("\"{start}\" is not a Monday."));
            }
            let rec = data
                .athletes
                .iter_mut()
                .find(|x| x.id == athlete)
                .ok_or_else(|| format!("There is no athlete {athlete}."))?;
            rec.start = start;
        }
        "set_goal" => {
            let athlete = number(a, "athlete")?;
            let goal = u32::try_from(number(a, "goal")?).map_err(|_| "The goal is too large.".to_string())?;
            if !GOALS.contains(&goal) {
                return Err(format!("{goal} is not one of the choices."));
            }
            let rec = data
                .athletes
                .iter_mut()
                .find(|x| x.id == athlete)
                .ok_or_else(|| format!("There is no athlete {athlete}."))?;
            rec.goal = Some(goal);
        }
        "set_cycle_note" => {
            let cycle = text(a, "cycle")?.to_string();
            let body = text(a, "text")?.trim().to_string();
            if !data.program.cycles.iter().any(|c| c.id == cycle) {
                return Err(format!("There is no cycle \"{cycle}\"."));
            }
            data.cycle_notes.retain(|n| n.cycle != cycle);
            if !body.is_empty() {
                data.cycle_notes.push(CycleNoteRec { cycle, text: body });
            }
        }
        other => return Err(format!("Unknown request \"{other}\".")),
    }
    Ok(())
}

fn same_session(a: &MoveRec, b: &MoveRec) -> bool {
    a.athlete == b.athlete && a.cycle == b.cycle && a.week == b.week && a.dow == b.dow
}

/// A request built from plain values, for the actions that make one.
pub fn move_request(rec: &MoveRec) -> Request {
    Request::new(
        "move_session",
        json!({"athlete": rec.athlete, "cycle": rec.cycle, "week": rec.week, "dow": rec.dow, "to": rec.to}),
    )
}
