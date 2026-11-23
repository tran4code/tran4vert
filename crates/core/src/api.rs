//! The door into the core. Every function takes strings and returns a JSON
//! string: `{"ok": ...}` on success or `{"error": "..."}` with a message the
//! page can show as it is. The shim forwards these calls and nothing else.
//!
//! The core describes the screen as a view model (`Ui`): the screen's name,
//! its panels and their actions, and the glossary's definitions. The page
//! builds the DOM from that description without knowing what a panel means.
//! Settings, screens and flags all come back through one function, `act`,
//! with a dotted name such as `screen.show`, so a new option needs a new
//! `match` arm here and no new shim function. (After Peter's Mandible.)
//!
//! A change to shared data comes back as `requests` for the page to send to
//! the database; the page then fetches the data again and hands it to
//! `data.load`. In demo mode the page hands each request to `data.apply`
//! instead, and the core changes its own copy.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::data::{apply, Data, Request};
use crate::dates;
use crate::glossary;
use crate::maxes;
use crate::moves;
use crate::page::{self, Strings};
use crate::program::Program;
use crate::schedule;
use crate::ui::{args, Hint, Ui};
use crate::view::{self, Ctx, CHANGES};

fn ok<T: Serialize>(value: &T) -> String {
    serde_json::json!({ "ok": value }).to_string()
}

fn error(message: String) -> String {
    serde_json::json!({ "error": message }).to_string()
}

/// A session of the program, named the way the database names it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveKey {
    pub cycle: String,
    pub week: u32,
    pub dow: u8,
}

/// What the page hands back on the next call. The page stores it and does
/// not read it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub strings: Strings,
    pub today: String,
    pub screen: String,
    #[serde(default)]
    pub data: Option<Data>,
    #[serde(default)]
    pub athlete: Option<u64>,
    #[serde(default)]
    pub day: Option<String>,
    #[serde(default)]
    pub week_offset: i64,
    #[serde(default)]
    pub lift: Option<String>,
    #[serde(default)]
    pub prefill: Option<String>,
    #[serde(default)]
    pub moving: Option<MoveKey>,
    #[serde(default)]
    pub demo: bool,
    #[serde(default)]
    pub offline: bool,
    /// The strings key of the last refusal, shown on the passcode screen.
    #[serde(default)]
    pub failure: String,
}

type Out = Result<(State, Vec<Request>, String), String>;

fn fill_numbers(text: &str) -> String {
    page::fill(
        text,
        &[
            ("cycle.weeks", &schedule::CYCLE_WEEKS.to_string()),
            ("working_weight.step", &maxes::ROUND_TO.to_string()),
        ],
    )
}

fn hints_from(glossary_md: &str) -> Result<Vec<Hint>, String> {
    let entries = glossary::parse(glossary_md).map_err(|e| e.message())?;
    Ok(entries
        .into_iter()
        .map(|e| Hint {
            term: e.term,
            definition: fill_numbers(&e.definition),
        })
        .collect())
}

/// The text the phone keeps so it remembers who is signed in.
fn keep_text(st: &State) -> String {
    match st.athlete {
        Some(id) => json!({"v": 1, "athlete": id}).to_string(),
        None => String::new(),
    }
}

fn render(st: State, hints: Vec<Hint>, requests: Vec<Request>, message: String) -> Ui<State> {
    let s = st.strings.clone();
    let today = dates::parse(&st.today).unwrap_or(0);
    let data = st.data.clone();
    let athlete = match (&data, st.athlete) {
        (Some(d), Some(id)) => d.athlete(id).cloned(),
        _ => None,
    };
    let keep = keep_text(&st);
    let mut screen = st.screen.clone();
    let (title, panels, nav, header) = match (&data, &athlete) {
        (None, _) => {
            screen = "passcode".into();
            let err = if st.failure.is_empty() { String::new() } else { s.t(&st.failure) };
            (
                s.t("title_passcode"),
                view::passcode(&s, &err),
                Vec::new(),
                view::header(None, &st, &s),
            )
        }
        (Some(d), None) => {
            screen = "who".into();
            (s.t("title_who"), view::who(&st, d, &s), Vec::new(), view::header(None, &st, &s))
        }
        (Some(d), Some(a)) => {
            let ctx = Ctx {
                st: &st,
                data: d,
                athlete: a,
                today,
                s: &s,
            };
            if screen == "passcode" {
                screen = "today".into();
            }
            let header = view::header(Some(&ctx), &st, &s);
            if screen == "who" {
                (s.t("title_who"), view::who(&st, d, &s), Vec::new(), header)
            } else {
                let (title, panels) = match screen.as_str() {
                    "week" => view::week(&ctx),
                    "maxes" => view::maxes_screen(&ctx),
                    "cycles" => view::cycles(&ctx),
                    "coach" if ctx.coach() => view::coach(&ctx),
                    "lift" => view::lift(&ctx, st.lift.as_deref().unwrap_or_default()),
                    _ => {
                        screen = "today".into();
                        view::today(&ctx)
                    }
                };
                let nav = view::nav(&ctx, &screen);
                (title, panels, nav, header)
            }
        }
    };
    let mut ui = Ui {
        screen,
        title,
        header,
        nav,
        panels,
        hints,
        requests,
        keep,
        message,
        labels: page_labels(&s),
        state: st,
    };
    if ui.state.offline {
        disable_changes(&mut ui);
    }
    ui
}

/// The strings the page uses for its own controls.
fn page_labels(s: &Strings) -> std::collections::BTreeMap<String, String> {
    s.0.iter()
        .filter(|(k, _)| k.starts_with("page_"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

fn disable_changes(ui: &mut Ui<State>) {
    fn walk(list: &mut [crate::ui::Row]) {
        for r in list {
            for a in &mut r.actions {
                if CHANGES.contains(&a.name.as_str()) {
                    a.enabled = false;
                }
            }
            walk(&mut r.rows);
        }
    }
    for p in &mut ui.panels {
        for a in &mut p.actions {
            if CHANGES.contains(&a.name.as_str()) {
                a.enabled = false;
            }
        }
        walk(&mut p.rows);
    }
}

/// Checks the strings file and the glossary and returns the first screen: the
/// passcode screen, with today's date kept for every rule that needs it.
pub fn start(strings_json: &str, glossary_md: &str, today: &str) -> String {
    let run = || -> Result<Ui<State>, String> {
        let strings = page::parse_strings(strings_json, &page::built_keys()).map_err(|e| e.message())?;
        let hints = hints_from(glossary_md)?;
        dates::parse(today).ok_or_else(|| format!("\"{today}\" is not a date."))?;
        let state = State {
            strings,
            today: today.to_string(),
            screen: "passcode".into(),
            data: None,
            athlete: None,
            day: None,
            week_offset: 0,
            lift: None,
            prefill: None,
            moving: None,
            demo: false,
            offline: false,
            failure: String::new(),
        };
        Ok(render(state, hints, Vec::new(), String::new()))
    };
    match run() {
        Ok(u) => ok(&u),
        Err(e) => error(e),
    }
}

// ---- values the page sends ----

/// A form's value as the page sends it: the action's own value and the
/// fields' values by name.
struct Form {
    value: String,
    fields: serde_json::Map<String, Value>,
}

impl Form {
    fn parse(text: &str) -> Result<Form, String> {
        let v: Value = serde_json::from_str(text).map_err(|_| "The form could not be read.".to_string())?;
        let fields = v
            .get("fields")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let value = v.get("value").and_then(Value::as_str).unwrap_or_default().to_string();
        Ok(Form { value, fields })
    }

    fn field(&self, name: &str) -> String {
        match self.fields.get(name) {
            Some(Value::String(s)) => s.trim().to_string(),
            Some(other) => other.to_string(),
            None => String::new(),
        }
    }
}

fn key_of(text: &str) -> Result<MoveKey, String> {
    serde_json::from_str(text).map_err(|_| "That session could not be read.".to_string())
}

fn number(text: &str, key: &str, s: &Strings) -> Result<u64, String> {
    text.trim().parse::<u64>().map_err(|_| s.t(key))
}

/// Applies a request to a copy of the data to see whether it makes sense.
fn checked(data: &Data, request: Request) -> Result<Request, String> {
    let mut probe = data.clone();
    apply(&mut probe, &request)?;
    Ok(request)
}

fn need_data(st: &State) -> Result<&Data, String> {
    st.data.as_ref().ok_or_else(|| st.strings.t("err_no_data"))
}

fn need_athlete(st: &State) -> Result<u64, String> {
    st.athlete.ok_or_else(|| st.strings.t("err_no_athlete"))
}

fn demo_data(program_json: &str, s: &Strings, today: i64) -> Result<Data, String> {
    let program = Program::parse(program_json)?;
    let start = dates::format(dates::monday_on_or_before(today));
    let before = dates::format(dates::monday_on_or_before(today) - 7);
    let athletes = (1..=3u64)
        .map(|n| crate::data::AthleteRec {
            id: n,
            name: s.t(&format!("demo_athlete_{n}")),
            start: start.clone(),
        })
        .collect();
    let max = |id: u64, lift: &str, weight: u32| crate::data::MaxRec {
        id,
        athlete: 1,
        lift: lift.to_string(),
        weight,
        date: before.clone(),
    };
    Ok(Data {
        program,
        athletes,
        maxes: vec![
            max(1, "anvil_squat", 305),
            max(2, "rope_clean", 195),
            max(3, "kettle_hinge", 215),
        ],
        moves: Vec::new(),
        notes: Vec::new(),
        cycle_notes: Vec::new(),
        role: "athlete".to_string(),
        as_of: dates::format(today),
    })
}

fn after_load(st: &mut State) {
    let valid = match (&st.data, st.athlete) {
        (Some(d), Some(id)) => d.athlete(id).is_some(),
        _ => false,
    };
    if !valid {
        st.athlete = None;
    }
    st.failure.clear();
    st.moving = None;
    st.screen = if valid && st.screen != "who" && st.screen != "passcode" {
        st.screen.clone()
    } else if valid {
        "today".into()
    } else {
        "who".into()
    };
}

fn step(mut st: State, name: &str, value: &str) -> Out {
    let s = st.strings.clone();
    let mut requests: Vec<Request> = Vec::new();
    let mut message = String::new();
    if st.offline && CHANGES.contains(&name) {
        return Err(s.t("err_offline"));
    }
    match name {
        "saved.load" => {
            // An unreadable kept choice is ignored: the person is asked again.
            if let Ok(v) = serde_json::from_str::<Value>(value) {
                if v.get("v").and_then(Value::as_u64) == Some(1) {
                    st.athlete = v.get("athlete").and_then(Value::as_u64);
                }
            }
        }
        "data.load" => {
            let data = Data::parse(value)?;
            st.data = Some(data);
            st.offline = false;
            after_load(&mut st);
        }
        "data.apply" => {
            let request: Request =
                serde_json::from_str(value).map_err(|_| "The request could not be read.".to_string())?;
            if request.rpc != "get_note_audio" {
                let data = st.data.as_mut().ok_or_else(|| s.t("err_no_data"))?;
                apply(data, &request)?;
            }
        }
        "net.offline" => st.offline = true,
        "net.failed" => {
            st.failure = format!("err_{value}");
            if !s.has(&st.failure) {
                st.failure = "err_unreachable".into();
            }
            if st.data.is_some() {
                message = s.t(&st.failure);
            }
        }
        "passcode.submit" => {
            let form = Form::parse(value)?;
            if form.field("group").is_empty() {
                st.failure = "err_passcode_empty".into();
                return Err(s.t("err_passcode_empty"));
            }
            st.failure.clear();
            requests.push(Request::new("get_state", args(Vec::new())));
        }
        "demo.start" => {
            let today = dates::parse(&st.today).unwrap_or(0);
            st.data = Some(demo_data(value, &s, today)?);
            st.demo = true;
            st.athlete = None;
            after_load(&mut st);
        }
        "demo.coach" => {
            let data = st.data.as_mut().ok_or_else(|| s.t("err_no_data"))?;
            data.role = "coach".into();
        }
        "athlete.pick" => {
            let id = number(value, "err_no_athlete", &s)?;
            if need_data(&st)?.athlete(id).is_none() {
                return Err(s.t("err_no_athlete"));
            }
            st.athlete = Some(id);
            st.screen = "today".into();
            st.day = None;
            st.week_offset = 0;
            st.moving = None;
        }
        "athlete.switch" => st.screen = "who".into(),
        "screen.show" => {
            let coach = need_data(&st)?.is_coach();
            let known = ["today", "week", "maxes", "cycles", "who"].contains(&value) || (value == "coach" && coach);
            if !known {
                return Err(s.f("err_no_screen", &[("screen", value)]));
            }
            st.screen = value.to_string();
            st.moving = None;
            if value == "today" {
                st.day = None;
            }
            if value != "maxes" {
                st.prefill = None;
            }
        }
        "week.shift" => {
            let by: i64 = value.parse().map_err(|_| s.t("err_no_screen"))?;
            st.week_offset = if by == 0 { 0 } else { st.week_offset + by };
        }
        "day.show" => {
            dates::parse(value).ok_or_else(|| s.f("err_date", &[("date", value)]))?;
            st.day = Some(value.to_string());
            st.screen = "today".into();
            st.moving = None;
        }
        "lift.show" => {
            if !need_data(&st)?.program.lifts.contains_key(value) {
                return Err(s.t("err_no_lift"));
            }
            st.lift = Some(value.to_string());
            st.screen = "lift".into();
        }
        "max.prefill" => {
            st.prefill = Some(value.to_string());
            st.screen = "maxes".into();
        }
        "max.add" => {
            let data = need_data(&st)?;
            let form = Form::parse(value)?;
            let athlete = if data.is_coach() && !form.field("athlete").is_empty() {
                number(&form.field("athlete"), "err_no_athlete", &s)?
            } else {
                need_athlete(&st)?
            };
            let weight = number(&form.field("weight"), "err_weight", &s)?;
            let request = checked(
                data,
                Request::new(
                    "add_max",
                    json!({"athlete": athlete, "lift": form.field("lift"), "weight": weight, "date": form.field("date")}),
                ),
            )
            .map_err(|_| s.t("err_max"))?;
            requests.push(request);
            st.prefill = None;
        }
        "max.delete" => {
            let data = need_data(&st)?;
            let id = number(value, "err_no_max", &s)?;
            let rec = data.maxes.iter().find(|m| m.id == id).ok_or_else(|| s.t("err_no_max"))?;
            if !data.is_coach() && Some(rec.athlete) != st.athlete {
                return Err(s.t("err_not_yours"));
            }
            requests.push(Request::new("delete_max", json!({"id": id})));
        }
        "move.start" => {
            need_data(&st)?;
            st.moving = Some(key_of(value)?);
        }
        "move.cancel" => st.moving = None,
        "move.save" | "move.save_all" => {
            let data = need_data(&st)?;
            let form = Form::parse(value)?;
            let key = key_of(&form.value)?;
            let to = form.field("to");
            let targets: Vec<u64> = if name == "move.save_all" {
                if !data.is_coach() {
                    return Err(s.t("err_not_coach"));
                }
                data.athletes.iter().map(|a| a.id).collect()
            } else {
                vec![need_athlete(&st)?]
            };
            for id in targets {
                let rec = crate::data::MoveRec {
                    athlete: id,
                    cycle: key.cycle.clone(),
                    week: key.week,
                    dow: key.dow,
                    to: to.clone(),
                };
                moves::check_move(data, &rec).map_err(|_| s.t("err_move"))?;
                requests.push(crate::data::move_request(&rec));
            }
            st.moving = None;
            if dates::parse(&to).is_some() {
                st.day = Some(to);
            }
        }
        "move.undo" => {
            let data = need_data(&st)?;
            let key = key_of(value)?;
            let athlete = need_athlete(&st)?;
            if moves::move_of(data, athlete, &key.cycle, key.week, key.dow).is_none() {
                return Err(s.t("err_no_move"));
            }
            requests.push(Request::new(
                "undo_move",
                json!({"athlete": athlete, "cycle": key.cycle, "week": key.week, "dow": key.dow}),
            ));
            st.moving = None;
        }
        "note.play" => {
            let data = need_data(&st)?;
            let id = number(value, "err_no_note", &s)?;
            if !data.notes.iter().any(|n| n.id == id) {
                return Err(s.t("err_no_note"));
            }
            requests.push(Request::new("get_note_audio", json!({"id": id})));
        }
        "note.save" => {
            let data = need_data(&st)?;
            if !data.is_coach() {
                return Err(s.t("err_not_coach"));
            }
            let form = Form::parse(value)?;
            let ms = number(&form.field("ms"), "err_note", &s)?;
            // Whole seconds, rounded up, so a very short recording is not zero.
            let seconds = ms.div_ceil(1000);
            let request = checked(
                data,
                Request::new(
                    "add_note",
                    json!({"lift": form.field("lift"), "title": form.field("title"), "seconds": seconds, "date": st.today}),
                ),
            )
            .map_err(|_| s.t("err_note"))?;
            requests.push(request);
        }
        "note.delete" => {
            let data = need_data(&st)?;
            if !data.is_coach() {
                return Err(s.t("err_not_coach"));
            }
            let id = number(value, "err_no_note", &s)?;
            requests.push(Request::new("delete_note", json!({"id": id})));
        }
        "start.set" => {
            let data = need_data(&st)?;
            if !data.is_coach() {
                return Err(s.t("err_not_coach"));
            }
            let form = Form::parse(value)?;
            for a in &data.athletes {
                let wanted = form.field(&format!("start_{}", a.id));
                if wanted.is_empty() || wanted == a.start {
                    continue;
                }
                let request = checked(
                    data,
                    Request::new("set_start_date", json!({"athlete": a.id, "start": wanted})),
                )
                .map_err(|_| s.f("err_start", &[("name", &a.name)]))?;
                requests.push(request);
            }
            if requests.is_empty() {
                message = s.t("nothing_changed");
            }
        }
        "cyclenote.set" => {
            let data = need_data(&st)?;
            if !data.is_coach() {
                return Err(s.t("err_not_coach"));
            }
            let form = Form::parse(value)?;
            for c in &data.program.cycles {
                let wanted = form.field(&format!("cycle_{}", c.id));
                let current = data
                    .cycle_notes
                    .iter()
                    .find(|n| n.cycle == c.id)
                    .map(|n| n.text.clone())
                    .unwrap_or_default();
                if wanted != current {
                    requests.push(Request::new("set_cycle_note", json!({"cycle": c.id, "text": wanted})));
                }
            }
            if requests.is_empty() {
                message = s.t("nothing_changed");
            }
        }
        other => return Err(format!("Unknown action \"{other}\".")),
    }
    Ok((st, requests, message))
}

/// A length in milliseconds, as text, written `m:ss` and cut down to whole
/// seconds. The recorder's timer shows it. Text that is not a number is `0:00`.
pub fn length(milliseconds: &str) -> String {
    let ms: u64 = milliseconds.trim().parse().unwrap_or(0);
    crate::notes::length(u32::try_from(ms / 1000).unwrap_or(u32::MAX))
}

/// Applies one named action to the state the page holds.
pub fn act(state_json: &str, glossary_md: &str, name: &str, value: &str) -> String {
    let run = || -> Result<Ui<State>, String> {
        let state: State =
            serde_json::from_str(state_json).map_err(|e| format!("The state could not be read: {e}"))?;
        let hints = hints_from(glossary_md)?;
        let (state, requests, message) = step(state, name, value)?;
        Ok(render(state, hints, requests, message))
    };
    match run() {
        Ok(u) => ok(&u),
        Err(e) => error(e),
    }
}
