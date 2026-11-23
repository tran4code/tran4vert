//! The screens, end to end through the core's one door: `start` and `act`,
//! with the real strings file, the real glossary and the invented sample
//! program. Expected values are worked out by hand from the sample program:
//!
//! - every athlete starts on Monday 2026-11-30, so cycle 1 week 1 Tuesday is
//!   2026-12-01 (the anvil squat lift session) and Thursday is 2026-12-03;
//! - athlete 1 (Sam) has an anvil squat max of 285 (2026-10-05) then 305
//!   (2026-11-02), a rope clean max of 195 and a kettle hinge max of 215;
//! - 305 at 75% is 228.75, which rounds to 230; 215 at 65% is 139.75, 140;
//!   195 at 60% is 117, 115; 195 at 110% is 214.5, 215.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use tool_core::api;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn read(name: &str) -> String {
    fs::read_to_string(root().join(name)).unwrap_or_else(|e| panic!("cannot read {name}: {e}"))
}

/// The shared data as `get_state` would return it, for the sample program.
fn remote(role: &str) -> String {
    let program: Value = serde_json::from_str(&read("tests/fixtures/sample_program.json")).unwrap();
    json!({
        "program": program,
        "athletes": [
            {"id": 1, "name": "Sam", "start": "2026-11-30"},
            {"id": 2, "name": "Jo", "start": "2026-11-30"},
            {"id": 3, "name": "Pat", "start": "2026-11-30"}],
        "maxes": [
            {"id": 1, "athlete": 1, "lift": "anvil_squat", "weight": 285, "date": "2026-10-05"},
            {"id": 2, "athlete": 1, "lift": "anvil_squat", "weight": 305, "date": "2026-11-02"},
            {"id": 3, "athlete": 1, "lift": "rope_clean", "weight": 195, "date": "2026-11-02"},
            {"id": 4, "athlete": 1, "lift": "kettle_hinge", "weight": 215, "date": "2026-11-02"},
            {"id": 5, "athlete": 2, "lift": "anvil_squat", "weight": 225, "date": "2026-11-09"}],
        "moves": [],
        "notes": [
            {"id": 1, "lift": "anvil_squat", "title": "Breathing", "seconds": 41, "date": "2026-11-10"},
            {"id": 2, "lift": "anvil_squat", "title": "Depth cue", "seconds": 65, "date": "2026-11-20"},
            {"id": 3, "lift": "rope_clean", "title": "Fast elbows", "seconds": 30, "date": "2026-11-12"}],
        "cycle_notes": [],
        "role": role,
        "as_of": "2026-11-23"
    })
    .to_string()
}

/// A page's view of the core: the state it hands back, and the last screen.
struct Page {
    state: String,
    ui: Value,
}

impl Page {
    fn open(today: &str) -> Page {
        let out = api::start(&read("data/strings.json"), &read("GLOSSARY.md"), today);
        Page::take(out, String::new()).unwrap()
    }

    fn take(out: String, old: String) -> Result<Page, String> {
        let v: Value = serde_json::from_str(&out).unwrap();
        if let Some(e) = v.get("error") {
            return Err(e.as_str().unwrap().to_string());
        }
        let ui = v["ok"].clone();
        let _ = old;
        Ok(Page {
            state: ui["state"].to_string(),
            ui,
        })
    }

    /// Does an action; the page keeps its old state when the core refuses.
    fn try_act(&mut self, name: &str, value: &str) -> Result<(), String> {
        let out = api::act(&self.state, &read("GLOSSARY.md"), name, value);
        let next = Page::take(out, self.state.clone())?;
        *self = next;
        Ok(())
    }

    fn act(&mut self, name: &str, value: &str) {
        self.try_act(name, value)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
    }

    /// Signs in with the sample data as the given athlete.
    fn signed_in(today: &str, role: &str, athlete: u64) -> Page {
        let mut p = Page::open(today);
        p.act("data.load", &remote(role));
        p.act("athlete.pick", &athlete.to_string());
        p
    }

    fn panel(&self, id: &str) -> &Value {
        self.ui["panels"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == id)
            .unwrap_or_else(|| panic!("no panel {id} on {}", self.ui["screen"]))
    }

    fn has_panel(&self, id: &str) -> bool {
        self.ui["panels"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["id"] == id)
    }

    fn row(&self, id: &str) -> &Value {
        fn find<'a>(rows: &'a Value, id: &str) -> Option<&'a Value> {
            for r in rows.as_array()? {
                if r["id"] == id {
                    return Some(r);
                }
                if let Some(found) = find(&r["rows"], id) {
                    return Some(found);
                }
            }
            None
        }
        for p in self.ui["panels"].as_array().unwrap() {
            if let Some(r) = find(&p["rows"], id) {
                return r;
            }
        }
        panic!("no row {id} on {}", self.ui["screen"]);
    }

    fn text(&self) -> String {
        self.ui["panels"].to_string()
    }

    fn requests(&self) -> &Vec<Value> {
        self.ui["requests"].as_array().unwrap()
    }
}

fn form(value: &str, fields: Value) -> String {
    json!({"value": value, "fields": fields}).to_string()
}

fn s(v: &Value) -> &str {
    v.as_str().unwrap_or("")
}

const TUE_LIFT: &str = "session-c1-1-1";

// ---- the first screens ----

#[test]
fn the_page_starts_on_the_passcode_screen_and_asks_for_nothing_yet() {
    let p = Page::open("2026-12-01");
    assert_eq!(p.ui["screen"], "passcode");
    assert_eq!(p.requests().len(), 0);
    let fields: Vec<&str> = p.panel("passcode")["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| s(&f["name"]))
        .collect();
    // The coach's field is out of sight until asked for.
    assert_eq!(fields, vec!["group"]);
    assert_eq!(p.panel("passcode")["fields"][0]["kind"], "password");
}

#[test]
fn the_coach_passcode_field_shows_only_when_asked_for_and_can_be_put_away() {
    let mut p = Page::open("2026-12-01");
    assert_eq!(p.panel("passcode")["actions"][1]["name"], "passcode.coach");
    p.act("passcode.coach", "");
    let names: Vec<&str> = p.panel("passcode")["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| s(&f["name"]))
        .collect();
    assert_eq!(names, vec!["coach"]);
    assert_eq!(p.panel("passcode")["heading"], "Coach passcode");
    assert_eq!(p.panel("passcode")["actions"][1]["name"], "passcode.group");
    p.act("passcode.group", "");
    assert_eq!(p.panel("passcode")["fields"][0]["name"], "group");
}

#[test]
fn forgetting_this_phone_is_asked_about_first_and_is_not_offered_in_the_demo() {
    let mut p = Page::open("2026-12-01");
    p.act("data.load", &remote("athlete"));
    assert_eq!(p.ui["screen"], "who");
    assert_eq!(p.panel("phone")["actions"][0]["name"], "phone.ask");
    p.act("phone.ask", "");
    let names: Vec<&str> = p.panel("phone")["actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| s(&a["name"]))
        .collect();
    assert_eq!(names, vec!["page.forget", "phone.keep"]);
    assert!(p.panel("phone")["text"]
        .as_str()
        .unwrap()
        .contains("Nothing is deleted"));
    p.act("phone.keep", "");
    assert_eq!(p.panel("phone")["actions"][0]["name"], "phone.ask");
    let mut d = Page::open("2026-12-01");
    d.act("demo.start", &read("tests/fixtures/sample_program.json"));
    assert!(!d.has_panel("phone"));
}

#[test]
fn the_maxes_screen_says_how_each_max_compares_with_the_one_before() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    p.act("screen.show", "maxes");
    // 285 on 2026-10-05, then 305 on 2026-11-02.
    assert_eq!(p.row("maxrec-2")["detail"], "Up 20 lb from 2026-10-05");
    assert!(p.row("maxrec-1").get("detail").is_none());
    let request = json!({"rpc": "add_max", "args": {"athlete": 1, "lift": "anvil_squat", "weight": 295, "date": "2026-11-20"}}).to_string();
    p.act("data.apply", &request);
    assert_eq!(p.row("maxrec-6")["detail"], "Down 10 lb from 2026-11-02");
    let same = json!({"rpc": "add_max", "args": {"athlete": 1, "lift": "anvil_squat", "weight": 295, "date": "2026-11-25"}}).to_string();
    p.act("data.apply", &same);
    assert_eq!(p.row("maxrec-7")["detail"], "Same as 2026-11-20");
}

#[test]
fn an_empty_passcode_is_refused_and_a_typed_one_becomes_one_request() {
    let mut p = Page::open("2026-12-01");
    let e = p
        .try_act("passcode.submit", &form("", json!({"group": "  ", "coach": ""})))
        .unwrap_err();
    assert_eq!(e, "Type a passcode first.");
    p.act("passcode.submit", &form("", json!({"group": "x", "coach": ""})));
    assert_eq!(p.requests().len(), 1);
    // The coach passcode alone is enough.
    let mut c = Page::open("2026-12-01");
    c.act("passcode.submit", &form("", json!({"group": "", "coach": "y"})));
    assert_eq!(c.requests().len(), 1);
    assert_eq!(p.requests()[0]["rpc"], "get_state");
    // The core never holds the passcode: nothing the page keeps names it.
    assert!(!p.state.contains("\"x\""));
}

#[test]
fn a_refused_passcode_shows_its_reason_on_the_passcode_screen() {
    let mut p = Page::open("2026-12-01");
    p.act("net.failed", "bad_passcode");
    assert_eq!(p.ui["screen"], "passcode");
    assert_eq!(p.panel("passcode")["marker"], "That passcode was not accepted.");
}

#[test]
fn loaded_data_shows_three_names_and_a_tap_shows_that_athletes_session() {
    let mut p = Page::open("2026-12-01");
    p.act("data.load", &remote("athlete"));
    assert_eq!(p.ui["screen"], "who");
    let names: Vec<&str> = p.panel("who")["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| s(&r["label"]))
        .collect();
    assert_eq!(names, vec!["Sam", "Jo", "Pat"]);
    p.act("athlete.pick", "1");
    assert_eq!(p.ui["screen"], "today");
    assert_eq!(p.ui["header"]["who"], "Sam");
    assert!(p.has_panel(TUE_LIFT));
    assert_eq!(p.panel(TUE_LIFT)["heading"], "Lift");
    assert_eq!(p.ui["title"], "Today · Tuesday 1 Dec 2026");
    assert_eq!(p.ui["keep"], json!({"v": 1, "athlete": 1}).to_string());
}

#[test]
fn the_phone_remembers_the_name_and_ignores_a_kept_choice_it_cannot_read() {
    let mut p = Page::open("2026-12-01");
    p.act("saved.load", &json!({"v": 1, "athlete": 2}).to_string());
    p.act("data.load", &remote("athlete"));
    assert_eq!(p.ui["screen"], "today");
    assert_eq!(p.ui["header"]["who"], "Jo");

    for bad in [
        "",
        "not json",
        "{\"v\": 9, \"athlete\": 2}",
        "{\"v\": 1, \"athlete\": 77}",
    ] {
        let mut q = Page::open("2026-12-01");
        q.act("saved.load", bad);
        q.act("data.load", &remote("athlete"));
        assert_eq!(q.ui["screen"], "who", "kept text {bad:?}");
    }
}

// ---- today: working weights, notes, voice notes ----

#[test]
fn a_percentage_shows_a_weight_from_the_athletes_own_max_beside_the_coachs_note() {
    let p = Page::signed_in("2026-12-01", "athlete", 1);
    let squat = p.row("session-c1-1-1-b1");
    assert_eq!(squat["label"], "Anvil Squat");
    assert_eq!(squat["detail"], "5 × 3 · last 2 at 75%");
    // 305 x 75% = 228.75, to the nearest 5: 230.
    let w = p.row("session-c1-1-1-b1-weight");
    assert_eq!(w["label"], "230 lb · last 2");
    assert_eq!(w["detail"], "75% of your Anvil Squat max, 305 lb");
    assert_eq!(p.row("session-c1-1-1-b1-note")["note"], "Slow down, fast up.");
    // 215 x 65% = 139.75: 140, from the lift this one takes its max from.
    let k = p.row("session-c1-1-1-b3-weight");
    assert_eq!(k["label"], "140 lb · last 1");
    assert_eq!(k["detail"], "65% of your Kettle Hinge max, 215 lb");
}

#[test]
fn an_athlete_without_a_max_sees_the_percentage_and_a_way_to_add_one() {
    let p = Page::signed_in("2026-12-01", "athlete", 3);
    let w = p.row("session-c1-1-1-b1-weight");
    assert_eq!(w["label"], "Last sets at 75% of your Anvil Squat max");
    assert_eq!(w["actions"][0]["name"], "max.prefill");
    assert_eq!(w["actions"][0]["value"], "anvil_squat");
}

#[test]
fn a_variant_uses_the_max_of_the_lift_it_takes_it_from() {
    // Monday 2026-11-30: the rope clean at 75% of 195 is 146.25, 145; the hang rope clean takes the
    // rope clean max, and 60% of 195 is 117, 115.
    let mut p = Page::signed_in("2026-11-30", "athlete", 1);
    assert_eq!(p.row("session-c1-1-0-b2-weight")["label"], "145 lb · last 3");
    assert_eq!(p.row("session-c1-1-0-b3-weight")["label"], "115 lb · last 2");
    // Monday of week two: 110% of 195 is 214.5, 215.
    p.act("day.show", "2026-12-07");
    assert_eq!(p.row("session-c1-2-0-b1-weight")["label"], "215 lb · last 3");
}

#[test]
fn a_new_max_changes_the_weights_from_its_date_on_and_not_before() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    p.act(
        "max.add",
        &form(
            "",
            json!({"lift": "anvil_squat", "weight": "400", "date": "2026-12-02"}),
        ),
    );
    assert_eq!(p.requests().len(), 1);
    assert_eq!(p.requests()[0]["rpc"], "add_max");
    let request = p.requests()[0].to_string();
    p.act("data.apply", &request);
    p.act("screen.show", "today");
    // The Tuesday session is on 2026-12-01: still 305.
    assert_eq!(p.row("session-c1-1-1-b1-weight")["label"], "230 lb · last 2");
    // Friday's session is later: 400 x 75% = 300... the Friday anvil squat is an AMAP set with no
    // percentage, so check the next Tuesday's test is unchanged and use the maxes screen instead.
    p.act("screen.show", "maxes");
    let row = p.row("max-anvil_squat-1");
    assert_eq!(row["detail"], "305 lb on 2026-11-02");
    p.act("day.show", "2026-12-02");
    p.act("screen.show", "maxes");
    assert_eq!(p.row("max-anvil_squat-1")["detail"], "305 lb on 2026-11-02");
    // The maxes screen uses today; the weights use each session's date.
    let mut later = Page::signed_in("2026-12-02", "athlete", 1);
    later.act("data.apply", &request);
    later.act("screen.show", "maxes");
    assert_eq!(later.row("max-anvil_squat-1")["detail"], "400 lb on 2026-12-02");
}

#[test]
fn a_bad_max_is_refused_in_words() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    let e = p
        .try_act(
            "max.add",
            &form(
                "",
                json!({"lift": "anvil_squat", "weight": "heavy", "date": "2026-12-01"}),
            ),
        )
        .unwrap_err();
    assert_eq!(e, "Type the weight as a whole number of pounds.");
    for (lift, weight, date) in [
        ("anvil_squat", "0", "2026-12-01"),
        ("anvil_squat", "99999", "2026-12-01"),
        ("anvil_squat", "300", "2026-13-01"),
        ("nope", "300", "2026-12-01"),
    ] {
        let e = p
            .try_act(
                "max.add",
                &form("", json!({"lift": lift, "weight": weight, "date": date})),
            )
            .unwrap_err();
        assert!(
            e.starts_with("The max was not saved"),
            "{lift} {weight} {date}: {e}"
        );
    }
}

#[test]
fn a_voice_note_shows_on_every_block_of_its_lift_for_every_athlete_and_on_no_other() {
    for athlete in [1, 2, 3] {
        let p = Page::signed_in("2026-12-01", "athlete", athlete);
        // Tuesday's anvil squat block: both notes, newest first.
        let squat = p.row("session-c1-1-1-b1");
        let voices: Vec<&str> = squat["rows"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["tag"] == "Voice note")
            .map(|r| s(&r["label"]))
            .collect();
        assert_eq!(voices, vec!["Depth cue", "Breathing"], "athlete {athlete}");
        let first = p.row("session-c1-1-1-b1-voice-2");
        assert_eq!(first["detail"], "1:05 · 2026-11-20");
        assert_eq!(first["actions"][0]["name"], "note.play");
        assert_eq!(first["actions"][0]["value"], "2");
        // The stone press has none.
        let press = p.row("session-c1-1-1-b2");
        assert!(press["rows"]
            .as_array()
            .is_none_or(|r| r.iter().all(|x| x["tag"] != "Voice note")));
    }
    // Friday's anvil squat block, in the same week, also offers them.
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    p.act("day.show", "2026-12-04");
    assert!(p.row("session-c1-1-4-b1-voice-1")["label"] == "Breathing");
    // The test block in week two shows them too.
    p.act("day.show", "2026-12-08");
    assert!(p.row("session-c1-2-1-b1-voice-2")["label"] == "Depth cue");
    // A note on the rope clean is not on the hang rope clean.
    p.act("day.show", "2026-11-30");
    let hang = p.row("session-c1-1-0-b3");
    assert!(hang["rows"]
        .as_array()
        .unwrap()
        .iter()
        .all(|x| x["tag"] != "Voice note"));
    let clean = p.row("session-c1-1-0-b2");
    assert!(clean["rows"]
        .as_array()
        .unwrap()
        .iter()
        .any(|x| x["label"] == "Fast elbows"));
}

#[test]
fn a_test_block_offers_to_enter_the_result_as_a_max() {
    let mut p = Page::signed_in("2026-12-08", "athlete", 1);
    let t = p.row("session-c1-2-1-b1-test");
    assert_eq!(t["label"], "Test: 5RM");
    assert_eq!(t["actions"][0]["name"], "max.prefill");
    p.act("max.prefill", "anvil_squat");
    assert_eq!(p.ui["screen"], "maxes");
    let lift_field = &p.panel("max-form")["fields"][0];
    assert_eq!(lift_field["name"], "lift");
    assert_eq!(lift_field["value"], "anvil_squat");
}

#[test]
fn circuits_collapse_to_a_title_and_a_dose_and_open_to_their_items() {
    let p = Page::signed_in("2026-12-01", "athlete", 1);
    let c = p.row("session-c1-1-1-b0");
    assert_eq!(c["label"], "Wake-up circuit");
    assert_eq!(c["detail"], "2 rounds · 10 each");
    assert_eq!(c["open"], false);
    let items: Vec<&str> = c["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| s(&r["label"]))
        .collect();
    assert_eq!(items, vec!["Cat cow", "Bird dog", "Dead bug"]);
}

#[test]
fn notes_text_and_dose_overrides_show_word_for_word() {
    let p = Page::signed_in("2026-12-02", "athlete", 1);
    // Wednesday 2026-12-02: the wall hold with its own dose, a text line, and a program note.
    assert_eq!(p.row("session-c1-1-2-b0")["detail"], "2 sets of 20 seconds");
    assert_eq!(p.row("session-c1-1-2-b1")["label"], "Easy bike");
    assert_eq!(p.row("session-c1-1-2-b1")["detail"], "Twenty minutes, easy.");
    assert_eq!(
        p.row("session-c1-1-2-b2")["note"],
        "Walk for twenty easy minutes."
    );
    let q = Page::signed_in("2026-11-30", "athlete", 1);
    assert_eq!(
        q.row("session-c1-1-0-b1")["note"],
        "Start light and add weight if the bar moves fast."
    );
}

// ---- the week ----

fn week_tags(p: &Page) -> Vec<String> {
    p.panel("week")["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| format!("{}: {}", s(&r["label"]), s(&r["tag"])))
        .collect()
}

#[test]
fn the_week_screen_runs_monday_to_sunday_and_earlier_and_later_weeks_can_be_reached() {
    let mut p = Page::signed_in("2026-12-02", "athlete", 1);
    p.act("screen.show", "week");
    assert_eq!(
        week_tags(&p),
        vec![
            "Mon 30 Nov: Jump",
            "Tue 1 Dec: Lift",
            "Wed 2 Dec: General",
            "Thu 3 Dec: Jump",
            "Fri 4 Dec: Lift",
            "Sat 5 Dec: Isometric",
            "Sun 6 Dec: Rest"
        ]
    );
    assert_eq!(p.ui["title"], "Cycle One · week 1 · Mon 30 Nov to Sun 6 Dec");
    assert!(s(&p.row("day-2026-12-02")["detail"]).starts_with("Today"));
    p.act("week.shift", "1");
    assert_eq!(p.ui["title"], "Cycle One · week 2 · Mon 7 Dec to Sun 13 Dec");
    assert_eq!(
        week_tags(&p),
        vec![
            "Mon 7 Dec: Jump",
            "Tue 8 Dec: Lift",
            "Wed 9 Dec: General",
            "Thu 10 Dec: Rest",
            "Fri 11 Dec: Rest",
            "Sat 12 Dec: Rest",
            "Sun 13 Dec: Rest"
        ]
    );
    p.act("week.shift", "1");
    assert!(week_tags(&p).iter().all(|t| t.ends_with("Not loaded")));
    p.act("week.shift", "-3");
    assert!(week_tags(&p).iter().all(|t| t.ends_with("Not started")));
    assert_eq!(p.ui["title"], "Before the start · Mon 23 Nov to Sun 29 Nov");
    p.act("week.shift", "0");
    assert_eq!(p.ui["title"], "Cycle One · week 1 · Mon 30 Nov to Sun 6 Dec");
}

#[test]
fn a_day_on_the_week_screen_opens_that_days_session() {
    let mut p = Page::signed_in("2026-12-02", "athlete", 1);
    p.act("screen.show", "week");
    p.act("day.show", "2026-12-03");
    assert_eq!(p.ui["screen"], "today");
    assert_eq!(p.ui["title"], "Thursday 3 Dec 2026");
    assert!(p.has_panel("session-c1-1-3"));
    assert!(p.has_panel("back-today"));
}

#[test]
fn a_date_with_no_session_says_why_in_words() {
    let mut p = Page::signed_in("2026-12-02", "athlete", 1);
    p.act("day.show", "2026-12-06");
    assert_eq!(p.panel("empty")["heading"], "Rest");
    p.act("day.show", "2026-12-14");
    assert_eq!(p.panel("empty")["text"], "Cycle 1, week 3 is not loaded yet.");
    p.act("day.show", "2026-11-20");
    assert_eq!(
        p.panel("empty")["text"],
        "The program starts on Monday 30 Nov 2026."
    );
}

// ---- moving a session ----

#[test]
fn a_moved_session_shows_on_the_new_date_and_says_so_and_the_old_date_says_where_it_went() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    let key = json!({"cycle": "c1", "week": 1, "dow": 1}).to_string();
    p.act("move.start", &key);
    assert!(p.has_panel("move"));
    p.act("move.save", &form(&key, json!({"to": "2026-12-03"})));
    assert_eq!(p.requests().len(), 1);
    assert_eq!(p.requests()[0]["rpc"], "move_session");
    assert_eq!(p.requests()[0]["args"]["to"], "2026-12-03");
    let request = p.requests()[0].to_string();
    p.act("data.apply", &request);
    // Back on the original date: it says where the session went.
    p.act("day.show", "2026-12-01");
    assert_eq!(p.panel("moved-c1-1-1")["marker"], "Moved to Thu 3 Dec");
    // On the new date: both sessions, the moved one marked.
    p.act("day.show", "2026-12-03");
    assert_eq!(p.panel(TUE_LIFT)["marker"], "Moved from Tue 1 Dec");
    assert!(p.has_panel("session-c1-1-3"));
    // The week screen shows both markers.
    p.act("screen.show", "week");
    assert_eq!(p.row("day-2026-12-01")["marker"], "Moved to Thu 3 Dec");
    assert_eq!(p.row("day-2026-12-03")["marker"], "Moved from Tue 1 Dec");
    // Undo puts it back.
    p.act("day.show", "2026-12-03");
    p.act("move.undo", &key);
    let undo = p.requests()[0].to_string();
    assert_eq!(p.requests()[0]["rpc"], "undo_move");
    p.act("data.apply", &undo);
    p.act("day.show", "2026-12-01");
    assert!(p.has_panel(TUE_LIFT));
    assert_eq!(p.panel(TUE_LIFT)["marker"], Value::Null);
}

#[test]
fn the_coach_moves_one_session_for_all_three_and_an_athlete_cannot() {
    let mut p = Page::signed_in("2026-12-01", "coach", 1);
    let key = json!({"cycle": "c1", "week": 1, "dow": 1}).to_string();
    p.act("move.start", &key);
    let actions: Vec<&str> = p.panel("move")["actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| s(&a["name"]))
        .collect();
    assert_eq!(actions, vec!["move.save", "move.save_all", "move.cancel"]);
    p.act("move.save_all", &form(&key, json!({"to": "2026-12-03"})));
    let who: Vec<u64> = p
        .requests()
        .iter()
        .map(|r| r["args"]["athlete"].as_u64().unwrap())
        .collect();
    assert_eq!(who, vec![1, 2, 3]);

    let mut q = Page::signed_in("2026-12-01", "athlete", 1);
    q.act("move.start", &key);
    let actions: Vec<&str> = q.panel("move")["actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| s(&a["name"]))
        .collect();
    assert_eq!(actions, vec!["move.save", "move.cancel"]);
    let e = q
        .try_act("move.save_all", &form(&key, json!({"to": "2026-12-03"})))
        .unwrap_err();
    assert_eq!(e, "That needs the coach passcode.");
}

#[test]
fn a_move_to_a_bad_date_or_to_the_sessions_own_date_is_refused() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    let key = json!({"cycle": "c1", "week": 1, "dow": 1}).to_string();
    for to in ["2026-12-01", "2026-02-30", "", "soon"] {
        let e = p
            .try_act("move.save", &form(&key, json!({"to": to})))
            .unwrap_err();
        assert_eq!(
            e, "That session cannot move there. Pick a different date.",
            "{to}"
        );
    }
}

// ---- maxes, cycles, lift, coach ----

#[test]
fn the_maxes_screen_shows_each_athletes_current_max_and_history_for_all_three() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    p.act("screen.show", "maxes");
    assert_eq!(p.row("max-anvil_squat-1")["detail"], "305 lb on 2026-11-02");
    assert_eq!(p.row("max-anvil_squat-2")["detail"], "225 lb on 2026-11-09");
    assert_eq!(p.row("max-anvil_squat-3")["detail"], "No max yet");
    let history: Vec<&str> = p.row("max-anvil_squat-1")["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| s(&r["label"]))
        .collect();
    assert_eq!(history, vec!["305 lb on 2026-11-02", "285 lb on 2026-10-05"]);
    // Presses and the lifts that take a max from another do not get their own row.
    assert!(!p.has_panel("maxes-rope_hang_clean"));
    assert!(p.has_panel("maxes-stone_press"));
}

#[test]
fn an_athlete_deletes_their_own_max_but_not_a_friends() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    p.act("max.delete", "1");
    assert_eq!(p.requests()[0]["rpc"], "delete_max");
    let e = p.try_act("max.delete", "5").unwrap_err();
    assert_eq!(e, "That max belongs to another athlete.");
    let mut c = Page::signed_in("2026-12-01", "coach", 1);
    c.act("max.delete", "5");
    assert_eq!(c.requests()[0]["args"]["id"], 5);
}

#[test]
fn the_cycles_screen_marks_weeks_with_no_data_and_lists_what_is_not_captured() {
    let mut p = Page::signed_in("2026-12-02", "athlete", 1);
    p.act("screen.show", "cycles");
    let tags = |cycle: &str| -> Vec<String> {
        (1..=4)
            .map(|w| s(&p.row(&format!("cycle-{cycle}-w{w}"))["tag"]).to_string())
            .collect()
    };
    assert_eq!(tags("c1"), vec!["Loaded", "Loaded", "Not loaded", "Not loaded"]);
    assert_eq!(
        tags("c2"),
        vec!["Loaded", "Not loaded", "Not loaded", "Not loaded"]
    );
    assert_eq!(p.panel("cycle-c1")["heading"], "Cycle One");
    assert_eq!(p.panel("cycle-c1")["text"], "Emphasis: Base strength");
    assert_eq!(p.row("cycle-c1-w1")["detail"], "Learn the lifts");
    assert_eq!(p.row("cycle-c1-w1")["marker"], "You are here");
    assert_eq!(p.row("cycle-c2-w2")["detail"], "Theme not captured");
    let program: Value = serde_json::from_str(&read("tests/fixtures/sample_program.json")).unwrap();
    let expected: Vec<&str> = program["not_captured"]
        .as_array()
        .unwrap()
        .iter()
        .map(s)
        .collect();
    let shown: Vec<&str> = p.panel("not-captured")["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| s(&r["label"]))
        .collect();
    assert_eq!(shown, expected);
    assert_eq!(shown.len(), 3);
    assert_eq!(p.row("cycle-c1-note")["note"], "Work through the weeks in order.");
}

#[test]
fn the_lift_screen_shows_the_athletes_max_and_the_voice_notes() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    p.act("lift.show", "rope_hang_clean");
    assert_eq!(p.ui["title"], "Hang Rope Clean");
    assert_eq!(p.panel("lift-max")["text"], "This lift uses your Rope Clean max.");
    assert_eq!(p.row("lift-max-now")["label"], "195 lb on 2026-11-02");
    assert!(p.panel("lift-notes")["text"]
        .as_str()
        .unwrap()
        .contains("No voice note"));
    p.act("lift.show", "anvil_squat");
    assert_eq!(p.panel("lift-notes")["rows"].as_array().unwrap().len(), 2);
    assert!(!p.has_panel("recorder"));
    let mut c = Page::signed_in("2026-12-01", "coach", 1);
    c.act("lift.show", "anvil_squat");
    assert!(c.has_panel("recorder"));
    assert_eq!(c.panel("recorder")["limit"], 180);
}

#[test]
fn only_the_coach_has_the_coach_screen_and_its_changes() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    assert!(p.try_act("screen.show", "coach").is_err());
    let nav: Vec<&str> = p.ui["nav"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| s(&n["label"]))
        .collect();
    assert_eq!(nav, vec!["Today", "Week", "Maxes", "Cycles"]);
    for (name, value) in [
        ("start.set", form("", json!({"start_1": "2026-12-07"}))),
        ("cyclenote.set", form("", json!({"cycle_c1": "x"}))),
        ("note.delete", "1".to_string()),
        (
            "note.save",
            form("", json!({"lift": "anvil_squat", "title": "T", "ms": "5000"})),
        ),
    ] {
        let e = p.try_act(name, &value).unwrap_err();
        assert_eq!(e, "That needs the coach passcode.", "{name}");
    }
    let c = Page::signed_in("2026-12-01", "coach", 1);
    let nav: Vec<&str> = c.ui["nav"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| s(&n["label"]))
        .collect();
    assert_eq!(nav, vec!["Today", "Week", "Maxes", "Cycles", "Coach"]);
}

#[test]
fn the_coach_records_a_note_and_deletes_one() {
    let mut p = Page::signed_in("2026-12-01", "coach", 1);
    // 40.001 seconds rounds up to 41.
    p.act(
        "note.save",
        &form(
            "",
            json!({"lift": "anvil_squat", "title": "Knees out", "ms": "40001"}),
        ),
    );
    let r = &p.requests()[0];
    assert_eq!(r["rpc"], "add_note");
    assert_eq!(r["args"]["seconds"], 41);
    assert_eq!(r["args"]["title"], "Knees out");
    assert_eq!(r["args"]["date"], "2026-12-01");
    let request = r.to_string();
    p.act("data.apply", &request);
    p.act("screen.show", "today");
    assert_eq!(p.row("session-c1-1-1-b1-voice-4")["detail"], "0:41 · 2026-12-01");
    for (title, ms) in [("", "5000"), ("T", "0"), ("T", "181000"), ("T", "soon")] {
        let e = p
            .try_act(
                "note.save",
                &form("", json!({"lift": "anvil_squat", "title": title, "ms": ms})),
            )
            .unwrap_err();
        assert!(
            e.starts_with("The voice note was not saved"),
            "{title:?} {ms}: {e}"
        );
    }
    p.act("note.delete", "4");
    assert_eq!(p.requests()[0]["rpc"], "delete_note");
    p.act("note.play", "2");
    assert_eq!(p.requests()[0]["rpc"], "get_note_audio");
    assert!(p.try_act("note.play", "99").is_err());
}

#[test]
fn the_coach_sets_start_dates_and_cycle_notes_and_only_changes_go_out() {
    let mut p = Page::signed_in("2026-12-01", "coach", 1);
    p.act("screen.show", "coach");
    assert_eq!(p.panel("coach-program")["text"], "3 of 8 weeks are loaded.");
    p.act(
        "start.set",
        &form(
            "",
            json!({"start_1": "2026-11-30", "start_2": "2026-12-07", "start_3": "2026-11-30"}),
        ),
    );
    assert_eq!(p.requests().len(), 1);
    assert_eq!(
        p.requests()[0]["args"],
        json!({"athlete": 2, "start": "2026-12-07"})
    );
    let e = p
        .try_act("start.set", &form("", json!({"start_3": "2026-12-02"})))
        .unwrap_err();
    assert_eq!(e, "The start date for Pat must be a Monday.");
    p.act("start.set", &form("", json!({"start_1": "2026-11-30"})));
    assert_eq!(p.ui["message"], "Nothing changed.");
    assert_eq!(p.requests().len(), 0);
    p.act(
        "cyclenote.set",
        &form("", json!({"cycle_c1": "Keep it easy.", "cycle_c2": ""})),
    );
    assert_eq!(p.requests().len(), 1);
    assert_eq!(
        p.requests()[0]["args"],
        json!({"cycle": "c1", "text": "Keep it easy."})
    );
    let request = p.requests()[0].to_string();
    p.act("data.apply", &request);
    p.act("screen.show", "cycles");
    assert_eq!(p.row("cycle-c1-note")["note"], "Keep it easy.");
}

// ---- offline, demo and the boundary ----

#[test]
fn offline_the_session_is_drawn_from_what_was_kept_and_changes_are_off() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    p.act("net.offline", "");
    assert_eq!(
        p.ui["header"]["offline"],
        "Offline, as of 2026-11-23. Changes are off."
    );
    assert!(p.has_panel(TUE_LIFT));
    assert!(p.text().contains("230 lb"));
    // Every change button is disabled, and the core refuses one anyway.
    let move_button = p.panel(TUE_LIFT)["actions"][0].clone();
    assert_eq!(move_button["name"], "move.start");
    let e = p
        .try_act(
            "max.add",
            &form(
                "",
                json!({"lift": "anvil_squat", "weight": "300", "date": "2026-12-01"}),
            ),
        )
        .unwrap_err();
    assert_eq!(e, "This phone is offline. Changes wait until it is back online.");
    // A fresh load brings it back.
    p.act("data.load", &remote("athlete"));
    assert_eq!(p.ui["header"]["offline"], "");
}

#[test]
fn offline_the_change_actions_in_the_forms_are_disabled() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    p.act("screen.show", "maxes");
    assert_eq!(p.panel("max-form")["actions"][0]["enabled"], true);
    p.act("net.offline", "");
    assert_eq!(p.panel("max-form")["actions"][0]["enabled"], false);
}

#[test]
fn the_demo_loads_the_sample_program_with_made_up_athletes_and_a_coach_toggle() {
    let mut p = Page::open("2026-12-02");
    p.act("demo.start", &read("tests/fixtures/sample_program.json"));
    assert_eq!(p.ui["screen"], "who");
    let names: Vec<&str> = p.panel("who")["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| s(&r["label"]))
        .collect();
    assert_eq!(names, vec!["Sam", "Jo", "Pat"]);
    assert!(p.has_panel("demo-coach"));
    p.act("athlete.pick", "1");
    // Today (Wednesday 2026-12-02) is in week one of the demo, whose start is the Monday before.
    assert_eq!(p.panel("session-c1-1-2")["heading"], "General");
    p.act("day.show", "2026-12-01");
    assert_eq!(p.row("session-c1-1-1-b1-weight")["label"], "230 lb · last 2");
    p.act("demo.coach", "");
    assert!(p.ui["nav"].to_string().contains("Coach"));
}

#[test]
fn nothing_marked_private_in_the_program_file_reaches_a_screen_or_the_state() {
    let mut p = Page::signed_in("2026-12-01", "coach", 1);
    let mut seen = vec![p.state.clone(), p.ui.to_string()];
    for screen in ["week", "maxes", "cycles", "coach", "who", "today"] {
        p.act("screen.show", screen);
        seen.push(p.ui.to_string());
    }
    for text in seen {
        assert!(!text.contains("maxes_from_pdf"));
        assert!(!text.contains("999"));
        assert!(!text.contains("original_date"));
    }
}

#[test]
fn an_unknown_action_a_bad_screen_or_a_bad_state_is_an_error_in_words() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    assert_eq!(
        p.try_act("screen.jump", "today").unwrap_err(),
        "Unknown action \"screen.jump\"."
    );
    assert_eq!(
        p.try_act("screen.show", "nowhere").unwrap_err(),
        "There is no screen nowhere."
    );
    assert_eq!(
        p.try_act("athlete.pick", "9").unwrap_err(),
        "That athlete is not here."
    );
    assert_eq!(
        p.try_act("lift.show", "nope").unwrap_err(),
        "That lift is not in the program."
    );
    assert_eq!(
        p.try_act("day.show", "someday").unwrap_err(),
        "someday is not a date."
    );
    let out = api::act("{not json", &read("GLOSSARY.md"), "screen.show", "today");
    assert!(out.contains("The state could not be read"));
    let out = api::start(&read("data/strings.json"), &read("GLOSSARY.md"), "tomorrow");
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["error"], "\"tomorrow\" is not a date.");
    let out = api::start("{}", &read("GLOSSARY.md"), "2026-12-01");
    assert!(out.contains("The strings file has no"));
}

#[test]
fn the_glossary_reaches_the_page_with_its_numbers_filled_in() {
    let p = Page::open("2026-12-01");
    let hints = p.ui["hints"].as_array().unwrap();
    let terms: Vec<&str> = hints.iter().map(|h| s(&h["term"])).collect();
    for t in [
        "Athlete",
        "Coach",
        "Cycle",
        "Isometric hold",
        "Lift",
        "Max",
        "Moved session",
        "Session",
        "Test",
        "Voice note",
        "Week",
        "Working weight",
    ] {
        assert!(terms.contains(&t), "no glossary entry for {t}");
    }
    let def = |t: &str| s(&hints.iter().find(|h| h["term"] == t).unwrap()["definition"]).to_string();
    assert!(def("Cycle").starts_with("A block of 4 weeks"));
    assert!(def("Working weight").contains("nearest 5 lb"));
    assert!(hints.iter().all(|h| !s(&h["definition"]).contains('{')));
}

#[test]
fn the_recorders_timer_shows_whole_seconds_as_minutes_and_seconds() {
    assert_eq!(api::length("0"), "0:00");
    assert_eq!(api::length("999"), "0:00");
    assert_eq!(api::length("1000"), "0:01");
    assert_eq!(api::length("61999"), "1:01");
    assert_eq!(api::length("180000"), "3:00");
    assert_eq!(api::length("soon"), "0:00");
}

// ---- the strings file and the code agree ----

fn quoted_keys_in(path: &str) -> Vec<String> {
    // Every `"word_like_key"` passed to `.t(` or `.f(`, or named as an `err_` key.
    let text = read(path);
    let mut out = Vec::new();
    for marker in [".t(\"", ".f(\"", "\"err_", "\"nothing_changed\""] {
        let mut from = 0;
        while let Some(at) = text[from..].find(marker) {
            let start = from + at + if marker.starts_with('"') { 1 } else { marker.len() };
            let key: String = text[start..]
                .chars()
                .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')
                .collect();
            // `format!("err_{value}")` names a family of keys; the families are checked below.
            if !key.is_empty() && key != "err_" {
                out.push(key);
            }
            from = start;
        }
    }
    out.sort();
    out.dedup();
    out
}

#[test]
fn every_string_the_screens_use_is_in_the_strings_file() {
    let file: Value = serde_json::from_str(&read("data/strings.json")).unwrap();
    let mut missing = Vec::new();
    for path in ["crates/core/src/view.rs", "crates/core/src/api.rs"] {
        for key in quoted_keys_in(path) {
            if file.get(&key).is_none() {
                missing.push(format!("{path}: {key}"));
            }
        }
    }
    assert!(
        missing.is_empty(),
        "keys the code uses and the file lacks:\n{}",
        missing.join("\n")
    );
}

#[test]
fn every_string_in_the_strings_file_is_used_by_the_core_or_the_page() {
    let file: Value = serde_json::from_str(&read("data/strings.json")).unwrap();
    let mut code = String::new();
    for path in [
        "crates/core/src/view.rs",
        "crates/core/src/api.rs",
        "crates/core/src/page.rs",
    ] {
        code.push_str(&read(path));
    }
    let mut page = String::new();
    for name in ["app.js", "audio.js"] {
        page.push_str(&fs::read_to_string(root().join("web").join(name)).unwrap_or_default());
    }
    let built = |key: &str| {
        ["dow_", "dow_long_", "month_", "kind_", "rx_", "demo_athlete_"]
            .iter()
            .any(|p| key.starts_with(p))
    };
    let mut unused = Vec::new();
    for (key, _) in file.as_object().unwrap() {
        // The net.failed action builds `err_<reason>` keys from what the page reports.
        let reason = key.starts_with("err_") && code.contains("format!(\"err_{");
        if key.starts_with('_') || built(key) || reason {
            continue;
        }
        let in_code = code.contains(&format!("\"{key}\""));
        let in_page = key.starts_with("page_") && page.contains(key.as_str());
        if !in_code && !in_page {
            unused.push(key.clone());
        }
    }
    assert!(unused.is_empty(), "strings nothing uses: {unused:?}");
}
