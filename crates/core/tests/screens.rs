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

/// Signs in with the sample data after a change to it (logs, finishes, ...).
fn signed_in_with(today: &str, athlete: u64, change: impl FnOnce(&mut Value)) -> Page {
    let mut data: Value = serde_json::from_str(&remote("athlete")).unwrap();
    change(&mut data);
    let mut p = Page::open(today);
    p.act("data.load", &data.to_string());
    p.act("athlete.pick", &athlete.to_string());
    p
}

fn key(block: u32, set: u32) -> String {
    json!({"cycle": "c1", "week": 1, "dow": 1, "block": block, "set": set}).to_string()
}

fn form(value: &str, fields: Value) -> String {
    json!({"value": value, "fields": fields}).to_string()
}

fn s(v: &Value) -> &str {
    v.as_str().unwrap_or("")
}

const TUE_LIFT: &str = "session-c1-1-1";

// ---- the first screens ----

const NAMES: &str = r#"[{"id":1,"name":"Sam"},{"id":2,"name":"Jo"},{"id":3,"name":"Pat"}]"#;

#[test]
fn the_page_starts_with_no_names_and_asks_for_nothing_yet() {
    let p = Page::open("2026-12-01");
    assert_eq!(p.ui["screen"], "passcode");
    assert_eq!(p.requests().len(), 0);
    assert_eq!(p.panel("names")["text"], "Loading the names…");
    // The coach's way in is a button, not a field.
    assert_eq!(p.panel("coach-login")["actions"][0]["name"], "passcode.coach");
    assert!(!p.has_panel("passcode"));
}

#[test]
fn each_lift_block_carries_its_category_as_a_word_and_a_held_block_is_isometric() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    // Tuesday: squat (strength), stone press (accessory), pillar raise (calf).
    for (id, cat, word) in [
        ("session-c1-1-1-b1", "strength", "Strength"),
        ("session-c1-1-1-b2", "accessory", "Accessory"),
        ("session-c1-1-1-b4", "calf", "Calf"),
    ] {
        assert_eq!(p.row(id)["category"], cat, "{id}");
        assert_eq!(p.row(id)["tag"], word, "{id}");
    }
    // The circuit has none.
    assert!(p.row("session-c1-1-1-b0").get("category").is_none());
    // Monday's rope clean is power; Friday's held lift is isometric although its lift says calf.
    p.act("day.show", "2026-11-30");
    assert_eq!(p.row("session-c1-1-0-b2")["category"], "power");
    p.act("day.show", "2026-12-04");
    assert_eq!(p.row("session-c1-1-4-b2")["category"], "isometric");
    assert_eq!(p.row("session-c1-1-4-b2")["tag"], "Isometric");
    // A library hold is isometric too.
    p.act("day.show", "2026-12-05");
    assert_eq!(p.row("session-c1-1-5-b0")["category"], "isometric");
}

#[test]
fn the_week_shows_the_categories_each_day_holds_in_a_fixed_order_and_a_colour_key() {
    let mut p = Page::signed_in("2026-12-02", "athlete", 1);
    p.act("screen.show", "week");
    let chips = |p: &Page, date: &str| -> Vec<String> {
        p.row(&format!("day-{date}"))
            .get("chips")
            .and_then(|c| c.as_array())
            .map(|c| c.iter().map(|x| s(&x["label"]).to_string()).collect())
            .unwrap_or_default()
    };
    assert_eq!(chips(&p, "2026-11-30"), vec!["Power"]);
    assert_eq!(chips(&p, "2026-12-01"), vec!["Strength", "Calf", "Accessory"]);
    assert_eq!(chips(&p, "2026-12-02"), vec!["Isometric"]);
    assert_eq!(chips(&p, "2026-12-03"), vec!["Accessory"]);
    assert_eq!(chips(&p, "2026-12-04"), vec!["Strength", "Isometric"]);
    assert_eq!(chips(&p, "2026-12-05"), vec!["Isometric"]);
    assert!(chips(&p, "2026-12-06").is_empty());
    // The first chip carries the key the page colours it by.
    assert_eq!(p.row("day-2026-12-01")["chips"][0]["value"], "strength");
    let key: Vec<&str> = p.panel("legend")["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| s(&r["label"]))
        .collect();
    assert_eq!(key, vec!["Power", "Strength", "Calf", "Accessory", "Isometric"]);
    assert_eq!(p.row("legend-calf")["category"], "calf");
}

#[test]
fn the_first_screen_has_a_motivating_message_two_jump_heights_and_a_tile_for_each_athlete() {
    let mut p = Page::open("2026-12-01");
    let names = r#"[{"id":1,"name":"Sam","avatar":true},{"id":2,"name":"Jo"},{"id":3,"name":"Pat"}]"#;
    p.act("names.load", names);
    let hero = p.panel("hero");
    assert_eq!(hero["kind"], "hero");
    assert_eq!(hero["heading"], "Reach 50 inches.");
    // The two heights the characters jump to: 20 inches before, 50 with Tran4Vert.
    assert_eq!(hero["data"]["before"], "20");
    assert_eq!(hero["data"]["after"], "50");
    assert_eq!(p.row("hero-before")["detail"], "20 inches");
    assert_eq!(p.row("hero-after")["detail"], "50 inches");
    let lines: Vec<&str> = hero["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| s(&r["id"]).starts_with("hero-line"))
        .map(|r| s(&r["label"]))
        .collect();
    assert_eq!(lines.len(), 3);
    // One tile per athlete; a picture only where there is one, an initial otherwise.
    assert_eq!(p.panel("names")["kind"], "people");
    assert_eq!(p.row("name-1")["avatar"], 1);
    assert_eq!(p.row("name-1")["initial"], "S");
    assert!(p.row("name-2").get("avatar").is_none());
    assert_eq!(p.row("name-2")["initial"], "J");
}

#[test]
fn a_picture_is_checked_for_size_and_for_who_may_change_it() {
    let mut a = Page::signed_in("2026-12-01", "athlete", 1);
    // An athlete in the sample data is not marked signed in by tapping, so the screen's own athlete counts.
    a.act(
        "avatar.save",
        &form("", json!({"athlete": "1", "bytes": "152648"})),
    );
    assert_eq!(a.requests()[0]["rpc"], "set_avatar");
    assert_eq!(a.requests()[0]["args"]["athlete"], 1);
    assert_eq!(
        a.try_act(
            "avatar.save",
            &form("", json!({"athlete": "1", "bytes": "300001"}))
        )
        .unwrap_err(),
        "That picture is empty or over 300 KB. Pick a smaller one."
    );
    assert!(a
        .try_act("avatar.save", &form("", json!({"athlete": "1", "bytes": "0"})))
        .is_err());
    assert!(a
        .try_act("avatar.save", &form("", json!({"athlete": "1", "bytes": "big"})))
        .is_err());
    assert_eq!(
        a.try_act("avatar.save", &form("", json!({"athlete": "2", "bytes": "1000"})))
            .unwrap_err(),
        "That name is not yours."
    );
    let mut c = Page::signed_in("2026-12-01", "coach", 1);
    c.act("avatar.save", &form("", json!({"athlete": "2", "bytes": "1000"})));
    assert_eq!(c.requests()[0]["args"]["athlete"], 2);
    // Once applied, the athlete has a picture, and their tile says so.
    let request = c.requests()[0].to_string();
    c.act("data.apply", &request);
    c.act("athlete.switch", "");
    assert_eq!(c.row("athlete-2")["avatar"], 2);
    assert!(c.row("athlete-3").get("avatar").is_none());
}

#[test]
fn the_picture_form_is_on_the_coach_screen_for_anyone_and_on_an_athletes_own_phone_for_them() {
    let mut c = Page::signed_in("2026-12-01", "coach", 1);
    c.act("screen.show", "coach");
    let ids: Vec<&str> = c.panel("avatar")["fields"][0]["choices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| s(&x["value"]))
        .collect();
    assert_eq!(ids, vec!["1", "2", "3"]);
    let mut p = Page::open("2026-12-01");
    p.act("names.load", NAMES);
    p.act("login.name", "2");
    let mut state = serde_json::from_str::<Value>(&remote("athlete")).unwrap();
    state["me"] = json!(2);
    p.act("data.load", &state.to_string());
    p.act("athlete.switch", "");
    let own: Vec<&str> = p.panel("avatar")["fields"][0]["choices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| s(&x["value"]))
        .collect();
    assert_eq!(own, vec!["2"]);
}

#[test]
fn tapping_a_name_then_a_goal_signs_in_and_the_core_never_sees_a_typed_secret() {
    let mut p = Page::open("2026-12-01");
    p.act("names.load", NAMES);
    let names: Vec<&str> = p.panel("names")["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| s(&r["label"]))
        .collect();
    assert_eq!(names, vec!["Sam", "Jo", "Pat"]);
    p.act("login.name", "2");
    assert_eq!(p.panel("goal")["heading"], "Jo, what vertical do you want?");
    let options: Vec<&str> = p.panel("goal")["actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| s(&a["label"]))
        .collect();
    assert_eq!(options.first(), Some(&"30 inches"));
    assert_eq!(options[options.len() - 2], "60 inches");
    assert_eq!(options.len(), 17); // sixteen goals and "Not me"
                                   // 40 inches for Jo: one call for the state, and the sign-in "2:40" for the page to send.
    p.act("goal.pick", "40");
    assert_eq!(p.requests().len(), 1);
    assert_eq!(p.requests()[0]["rpc"], "get_state");
    assert_eq!(p.ui["code"], "2:40");
    // A goal that is not a choice, and a goal with no name tapped, are refused.
    assert_eq!(
        p.try_act("goal.pick", "41").unwrap_err(),
        "That goal is not one of the choices."
    );
    let mut q = Page::open("2026-12-01");
    q.act("names.load", NAMES);
    assert!(q.try_act("goal.pick", "40").is_err());
    assert!(q.try_act("login.name", "9").is_err());
}

#[test]
fn not_me_goes_back_to_the_names_and_a_refusal_is_shown_on_the_goal_screen() {
    let mut p = Page::open("2026-12-01");
    p.act("names.load", NAMES);
    p.act("login.name", "1");
    p.act("net.failed", "bad_passcode");
    assert_eq!(p.panel("goal")["marker"], "That is not it. Try again.");
    p.act("net.failed", "too_many");
    assert_eq!(
        p.panel("goal")["marker"],
        "Too many wrong tries. Wait a few minutes and try again."
    );
    p.act("login.back", "");
    assert!(p.has_panel("names"));
    assert_eq!(p.panel("names")["marker"], Value::Null);
}

#[test]
fn a_signed_in_athlete_is_that_athlete_and_cannot_be_another_but_the_coach_can() {
    let mut p = Page::open("2026-12-01");
    p.act("names.load", NAMES);
    p.act("login.name", "2");
    let mut state = serde_json::from_str::<Value>(&remote("athlete")).unwrap();
    state["me"] = json!(2);
    p.act("data.load", &state.to_string());
    // No name screen: straight to today, as Jo.
    assert_eq!(p.ui["screen"], "today");
    assert_eq!(p.ui["header"]["who"], "Jo");
    assert_eq!(
        p.try_act("athlete.pick", "1").unwrap_err(),
        "That name is not yours."
    );
    p.act("athlete.pick", "2");
    // Tapping the name shows this phone's panel and no other names.
    p.act("athlete.switch", "");
    assert_eq!(p.panel("who-you")["text"], "You are Jo.");
    assert!(p.has_panel("phone") && !p.has_panel("who"));
    // The coach may be anyone.
    let mut c = Page::open("2026-12-01");
    c.act("data.load", &remote("coach"));
    c.act("athlete.pick", "1");
    c.act("athlete.pick", "3");
    c.act("athlete.switch", "");
    assert!(c.has_panel("who"));
}

#[test]
fn the_coach_passcode_field_shows_only_when_asked_for_and_can_be_put_away() {
    let mut p = Page::open("2026-12-01");
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
    assert!(p.has_panel("coach-login"));
}

#[test]
fn an_empty_coach_passcode_is_refused_and_a_typed_one_becomes_one_request() {
    let mut p = Page::open("2026-12-01");
    p.act("passcode.coach", "");
    let e = p
        .try_act("passcode.submit", &form("", json!({"coach": "  "})))
        .unwrap_err();
    assert_eq!(e, "Type a passcode first.");
    p.act("passcode.submit", &form("", json!({"coach": "y"})));
    assert_eq!(p.requests().len(), 1);
    assert_eq!(p.requests()[0]["rpc"], "get_state");
    // The core never holds the passcode: nothing the page keeps names it.
    assert!(!p.state.contains("\"y\""));
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
fn the_coach_sets_goals_and_only_changed_ones_go_out() {
    let mut p = Page::signed_in("2026-12-01", "coach", 1);
    p.act("screen.show", "coach");
    p.act(
        "goals.set",
        &form("", json!({"goal_1": "40", "goal_2": "", "goal_3": "50"})),
    );
    let sent: Vec<(u64, u64)> = p
        .requests()
        .iter()
        .map(|r| {
            (
                r["args"]["athlete"].as_u64().unwrap(),
                r["args"]["goal"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(sent, vec![(1, 40), (3, 50)]);
    assert_eq!(p.requests()[0]["rpc"], "set_goal");
    // Once those are applied, the same form changes nothing.
    for r in p.requests().clone() {
        p.act("data.apply", &r.to_string());
    }
    p.act(
        "goals.set",
        &form("", json!({"goal_1": "40", "goal_2": "", "goal_3": "50"})),
    );
    assert_eq!(p.requests().len(), 0);
    assert_eq!(p.ui["message"], "Nothing changed.");
    assert_eq!(
        p.try_act("goals.set", &form("", json!({"goal_1": "41"})))
            .unwrap_err(),
        "That goal is not one of the choices."
    );
    let mut a = Page::signed_in("2026-12-01", "athlete", 1);
    assert_eq!(
        a.try_act("goals.set", &form("", json!({"goal_1": "40"})))
            .unwrap_err(),
        "That needs the coach passcode."
    );
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
    let actions = p.panel(TUE_LIFT)["actions"].as_array().unwrap().clone();
    let finish = actions.iter().find(|a| a["name"] == "session.finish").unwrap();
    assert_eq!(finish["enabled"], false);
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
        "crates/core/src/rivalry.rs",
    ] {
        code.push_str(&read(path));
    }
    let mut page = String::new();
    for name in ["app.js", "audio.js"] {
        page.push_str(&fs::read_to_string(root().join("web").join(name)).unwrap_or_default());
    }
    let built = |key: &str| {
        [
            "dow_",
            "dow_long_",
            "month_",
            "kind_",
            "rx_",
            "demo_athlete_",
            "hero_line_",
            "cat_",
            "own_done_",
            "rival_first_",
            "rival_heavy_",
        ]
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

// ---- checking things off, finishing, and the messages between athletes ----

#[test]
fn a_lift_has_a_check_box_for_each_set_and_a_weight_that_starts_at_the_working_weight() {
    let p = Page::signed_in("2026-12-01", "athlete", 1);
    // Tuesday: circuit (a check), squat 5 sets, press 3, kettle 3, pillar 3 = 15 boxes.
    let squat = p.row("session-c1-1-1-b1");
    assert_eq!(squat["sets"].as_array().unwrap().len(), 5);
    assert_eq!(squat["sets"][0]["action"]["label"], "Set 1");
    assert_eq!(squat["sets"][0]["action"]["pressed"], false);
    // The program names 75% for the last 2 of 5 sets: 305 at 75% is 230 for sets 4 and 5. Sets 1 to 3 are the
    // athlete's own ramp-up and start empty. 215 at 65% is 140 for the last of 3; a lift with no plan starts empty.
    let value = |row: &str, n: usize| {
        p.row(row)["sets"][n]["field"]["value"]
            .as_str()
            .unwrap()
            .to_string()
    };
    assert_eq!(
        (0..5).map(|n| value("session-c1-1-1-b1", n)).collect::<Vec<_>>(),
        vec!["", "", "", "230", "230"]
    );
    assert_eq!(squat["sets"][3]["working"], true);
    assert_eq!(squat["sets"][3]["action"]["label"], "Set 4 (working)");
    assert_eq!(p.row("session-c1-1-1-b2")["sets"][0]["working"], false);
    assert_eq!(squat["sets"][2]["working"], false);
    assert_eq!(
        (0..3).map(|n| value("session-c1-1-1-b3", n)).collect::<Vec<_>>(),
        vec!["", "", "140"]
    );
    assert_eq!(
        (0..3).map(|n| value("session-c1-1-1-b2", n)).collect::<Vec<_>>(),
        vec!["", "", ""]
    );
    assert_eq!(p.row("session-c1-1-1-b0")["actions"][0]["name"], "block.toggle");
    assert!(p.panel("session-c1-1-1")["text"]
        .as_str()
        .unwrap()
        .ends_with("0 of 15 checked"));
    // A coach's note alone has nothing to check.
    let q = Page::signed_in("2026-11-30", "athlete", 1);
    assert!(q.row("session-c1-1-0-b1").get("actions").is_none());
}

#[test]
fn a_held_lift_and_an_amap_set_are_checked_without_a_weight_where_there_are_no_sets() {
    let mut p = Page::signed_in("2026-12-04", "athlete", 1);
    // Friday: the AMAP squat has no sets, so one check; the held lift has 3 set boxes and no weight.
    let names: Vec<&str> = p.row("session-c1-1-4-b1")["actions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| s(&a["name"]))
        .collect();
    assert!(names.contains(&"block.toggle"), "{names:?}");
    assert!(p.row("session-c1-1-4-b1").get("sets").is_none());
    assert_eq!(p.row("session-c1-1-4-b2")["sets"].as_array().unwrap().len(), 3);
    assert!(p.row("session-c1-1-4-b2")["sets"][0].get("field").is_none());
    p.act("net.offline", "");
    assert_eq!(p.row("session-c1-1-4-b2")["sets"][0]["action"]["enabled"], false);
}

#[test]
fn checking_a_set_sends_the_weight_typed_and_the_box_shows_checked_once_applied() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    p.act("set.toggle", &form(&key(1, 2), json!({"weight": "235"})));
    let r = &p.requests()[0];
    assert_eq!(r["rpc"], "log_set");
    assert_eq!(
        r["args"],
        json!({"athlete": 1, "cycle": "c1", "week": 1, "dow": 1, "block": 1, "set": 2, "weight": 235, "done": true})
    );
    let request = r.to_string();
    p.act("data.apply", &request);
    assert_eq!(p.row("session-c1-1-1-b1")["sets"][1]["action"]["pressed"], true);
    assert_eq!(p.row("session-c1-1-1-b1")["sets"][0]["action"]["pressed"], false);
    assert!(p.panel("session-c1-1-1")["text"]
        .as_str()
        .unwrap()
        .ends_with("1 of 15 checked"));
    // That set shows the weight logged for it, and the others are untouched.
    assert_eq!(p.row("session-c1-1-1-b1")["sets"][1]["field"]["value"], "235");
    assert_eq!(p.row("session-c1-1-1-b1")["sets"][0]["field"]["value"], "");
    assert_eq!(p.row("session-c1-1-1-b1")["sets"][3]["field"]["value"], "230");
    // Tapping it again unchecks it.
    p.act("set.toggle", &form(&key(1, 2), json!({"weight": "235"})));
    assert_eq!(p.requests()[0]["args"]["done"], false);
}

#[test]
fn a_bad_weight_or_a_set_that_is_not_there_is_refused_in_words() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    assert_eq!(
        p.try_act("set.toggle", &form(&key(1, 1), json!({"weight": "heavy"})))
            .unwrap_err(),
        "Type the weight as a whole number of pounds."
    );
    for (k, w) in [
        (key(1, 1), "0"),
        (key(1, 1), "99999"),
        (key(1, 21), "100"),
        (key(40, 1), "100"),
    ] {
        assert_eq!(
            p.try_act("set.toggle", &form(&k, json!({"weight": w})))
                .unwrap_err(),
            "That could not be saved. Check the weight.",
            "{k} {w}"
        );
    }
    p.act("block.toggle", &form(&key(0, 0), json!({})));
    assert_eq!(p.requests()[0]["args"]["set"], 0);
}

#[test]
fn finishing_a_session_shows_finished_and_an_encouraging_message_that_names_who_was_beaten() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    p.act(
        "session.finish",
        &json!({"cycle": "c1", "week": 1, "dow": 1}).to_string(),
    );
    assert_eq!(p.requests()[0]["rpc"], "finish_session");
    assert_eq!(p.requests()[0]["args"]["done"], true);
    let request = p.requests()[0].to_string();
    p.act("data.apply", &request);
    assert_eq!(p.panel("session-c1-1-1")["marker"], "Finished");
    assert_eq!(
        p.panel("session-c1-1-1")["actions"][0]["name"],
        "session.unfinish"
    );
    assert_eq!(
        p.panel("popup")["text"],
        "Sam finished. Nice work, 1 this week. You beat Jo to it."
    );
    let id = p.panel("popup")["actions"][0]["value"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(id, "own:1:c1:1:1");
    p.act("msg.dismiss", &id);
    assert!(!p.has_panel("popup"));
    assert_eq!(p.ui["seen"], json!(["own:1:c1:1:1"]).to_string());
    // Undoing the finish takes the marker away.
    p.act(
        "session.unfinish",
        &json!({"cycle": "c1", "week": 1, "dow": 1}).to_string(),
    );
    assert_eq!(p.requests()[0]["args"]["done"], false);
}

#[test]
fn when_a_rival_finishes_first_the_other_athlete_gets_a_message_once() {
    let finish = |data: &mut Value| {
        data["finishes"] =
            json!([{"athlete": 2, "cycle": "c1", "week": 1, "dow": 1, "at": "2026-12-01T09:00:00Z"}]);
    };
    let mut p = signed_in_with("2026-12-01", 1, finish);
    // For athlete 1 and rival 2 on a Tuesday, the second wording is used.
    assert_eq!(
        p.panel("popup")["text"],
        "Jo is done and probably stretching. You are still on the clock."
    );
    // It comes first on the screen.
    assert_eq!(p.ui["panels"][0]["id"], "popup");
    p.act("msg.dismiss", "first:1:2:c1:1:1");
    assert!(!p.has_panel("popup"));
    // The phone remembers it across visits.
    let mut again = signed_in_with("2026-12-01", 1, finish);
    again.act("seen.load", &json!(["first:1:2:c1:1:1"]).to_string());
    assert!(!again.has_panel("popup"));
    assert!(again.try_act("seen.load", "not json").is_ok());
    assert!(
        again.has_panel("popup"),
        "an unreadable list means the message shows again"
    );
    // Athlete 2 sees no message about their own finish being late, and a finish on another day is not today's news.
    let other_day = |data: &mut Value| {
        data["finishes"] = json!([{"athlete": 2, "cycle": "c1", "week": 1, "dow": 0, "at": "x"}]);
    };
    assert!(!signed_in_with("2026-12-01", 1, other_day).has_panel("popup"));
}

#[test]
fn a_rival_with_more_weight_on_the_same_lift_gets_a_message_with_the_numbers() {
    let heavy = |data: &mut Value| {
        data["logs"] = json!([
            {"athlete": 2, "cycle": "c1", "week": 1, "dow": 1, "block": 1, "set": 1, "weight": 300},
            {"athlete": 1, "cycle": "c1", "week": 1, "dow": 1, "block": 1, "set": 1, "weight": 285}
        ]);
    };
    // Athlete 1 logged 285, rival 2 logged 300 on the Tuesday squat (athlete 1 and 2, Tuesday: wording 2).
    let p = signed_in_with("2026-12-01", 1, heavy);
    assert_eq!(
        p.panel("popup")["text"],
        "Jo is at 300 lb on Anvil Squat. Do not let that stand."
    );
    // The other way round, the lighter one gets no message about the heavier one's lead being theirs.
    let lead = signed_in_with("2026-12-01", 2, heavy);
    assert!(!lead.has_panel("popup"), "the one in front is not nudged");
    // Wording 3 shows the athlete's own number, or that there is none yet.
    let ahead = |data: &mut Value| {
        data["logs"] =
            json!([{"athlete": 1, "cycle": "c1", "week": 1, "dow": 1, "block": 1, "set": 1, "weight": 305}]);
    };
    let pat = signed_in_with("2026-12-01", 3, ahead);
    assert_eq!(
        pat.panel("popup")["text"],
        "Anvil Squat: Sam 305 lb, you no weight yet. Close the gap."
    );
    let with_mine = |data: &mut Value| {
        data["logs"] = json!([
            {"athlete": 1, "cycle": "c1", "week": 1, "dow": 1, "block": 1, "set": 1, "weight": 305},
            {"athlete": 3, "cycle": "c1", "week": 1, "dow": 1, "block": 1, "set": 1, "weight": 290}
        ]);
    };
    assert_eq!(
        signed_in_with("2026-12-01", 3, with_mine).panel("popup")["text"],
        "Anvil Squat: Sam 305 lb, you 290 lb. Close the gap."
    );
}

#[test]
fn a_finish_message_comes_before_a_weight_message() {
    let both = |data: &mut Value| {
        data["finishes"] = json!([{"athlete": 2, "cycle": "c1", "week": 1, "dow": 1, "at": "x"}]);
        data["logs"] =
            json!([{"athlete": 2, "cycle": "c1", "week": 1, "dow": 1, "block": 1, "set": 1, "weight": 300}]);
    };
    let mut p = signed_in_with("2026-12-01", 1, both);
    assert!(p.panel("popup")["text"]
        .as_str()
        .unwrap()
        .starts_with("Jo is done"));
    p.act("msg.dismiss", "first:1:2:c1:1:1");
    assert!(p.panel("popup")["text"]
        .as_str()
        .unwrap()
        .starts_with("Jo is at 300 lb"));
}

#[test]
fn equal_weights_get_no_message_and_two_athletes_on_one_phone_each_read_their_own() {
    let equal = |data: &mut Value| {
        data["logs"] = json!([
            {"athlete": 2, "cycle": "c1", "week": 1, "dow": 1, "block": 1, "set": 1, "weight": 300},
            {"athlete": 1, "cycle": "c1", "week": 1, "dow": 1, "block": 1, "set": 1, "weight": 300}
        ]);
    };
    assert!(
        !signed_in_with("2026-12-01", 1, equal).has_panel("popup"),
        "level is not behind"
    );
    // The coach's phone, two athletes: reading one athlete's message does not hide the other's.
    let mut c = Page::signed_in("2026-12-01", "coach", 1);
    let finish = |c: &mut Page, athlete: u64| {
        c.act("athlete.pick", &athlete.to_string());
        c.act(
            "session.finish",
            &json!({"cycle": "c1", "week": 1, "dow": 1}).to_string(),
        );
        let request = c.requests()[0].to_string();
        c.act("data.apply", &request);
    };
    finish(&mut c, 1);
    let id = c.panel("popup")["actions"][0]["value"]
        .as_str()
        .unwrap()
        .to_string();
    c.act("msg.dismiss", &id);
    finish(&mut c, 2);
    assert!(
        c.has_panel("popup"),
        "the second athlete's own message is not already read"
    );
    assert_ne!(c.panel("popup")["actions"][0]["value"].as_str().unwrap(), id);
}

#[test]
fn each_set_has_its_own_weight_and_the_ramp_up_is_the_athletes_to_choose() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    // A ramp-up: 135, then 185, then 225 for the first three sets, each logged as typed.
    for (set, w) in [(1, "135"), (2, "185"), (3, "225")] {
        p.act("set.toggle", &form(&key(1, set), json!({"weight": w})));
        assert_eq!(p.requests()[0]["args"]["weight"], w.parse::<u64>().unwrap());
        assert_eq!(p.requests()[0]["args"]["set"], set);
        let request = p.requests()[0].to_string();
        p.act("data.apply", &request);
    }
    let shown: Vec<String> = (0..5)
        .map(|n| {
            p.row("session-c1-1-1-b1")["sets"][n]["field"]["value"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(shown, vec!["135", "185", "225", "230", "230"]);
    // A working set can be lifted at a different weight than the plan.
    p.act("set.toggle", &form(&key(1, 4), json!({"weight": "240"})));
    let request = p.requests()[0].to_string();
    p.act("data.apply", &request);
    assert_eq!(p.row("session-c1-1-1-b1")["sets"][3]["field"]["value"], "240");
}

#[test]
fn a_lift_shows_its_warm_up_and_working_sets_as_it_fills_and_is_complete_when_all_are_checked() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    assert_eq!(
        p.row("session-c1-1-1-b1")["progress"],
        "Warm-up 0 of 3, working sets 0 of 2"
    );
    let tick = |p: &mut Page, set: u32, w: &str| {
        p.act("set.toggle", &form(&key(1, set), json!({"weight": w})));
        let request = p.requests()[0].to_string();
        p.act("data.apply", &request);
    };
    tick(&mut p, 1, "135");
    tick(&mut p, 2, "185");
    assert_eq!(
        p.row("session-c1-1-1-b1")["progress"],
        "Warm-up 2 of 3, working sets 0 of 2"
    );
    tick(&mut p, 4, "230");
    assert_eq!(
        p.row("session-c1-1-1-b1")["progress"],
        "Warm-up 2 of 3, working sets 1 of 2"
    );
    assert!(
        p.row("session-c1-1-1-b1").get("marker").is_none(),
        "not complete yet"
    );
    tick(&mut p, 3, "225");
    assert!(
        p.row("session-c1-1-1-b1").get("marker").is_none(),
        "four of five is not complete"
    );
    tick(&mut p, 5, "230");
    assert_eq!(p.row("session-c1-1-1-b1")["marker"], "Complete");
    assert_eq!(
        p.row("session-c1-1-1-b1")["progress"],
        "Warm-up 3 of 3, working sets 2 of 2"
    );
    // A lift with no percentage says how many sets are done.
    assert_eq!(p.row("session-c1-1-1-b2")["progress"], "0 of 3 sets");
    // The session's button says so only when every box in it is checked.
    let finish = |p: &Page| {
        p.panel(TUE_LIFT)["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["name"] == "session.finish")
            .unwrap()["label"]
            .clone()
    };
    assert_eq!(finish(&p), "Finish session");
    // Check everything else on Tuesday: the circuit, the rest of the squat sets, the press, the kettle and the pillar raise.
    p.act("block.toggle", &form(&key(0, 0), json!({})));
    let request = p.requests()[0].to_string();
    p.act("data.apply", &request);
    let mut left = Vec::new();
    for (block, sets) in [(1u32, 5u32), (2, 3), (3, 3), (4, 3)] {
        for set in 1..=sets {
            let row = format!("session-c1-1-1-b{block}");
            if p.row(&row)["sets"][(set - 1) as usize]["action"]["pressed"] != true {
                left.push((block, set));
            }
        }
    }
    let last = left.pop().unwrap();
    for (block, set) in left {
        p.act("set.toggle", &form(&key(block, set), json!({"weight": "100"})));
        let request = p.requests()[0].to_string();
        p.act("data.apply", &request);
    }
    // One box short of everything: the button still says Finish session.
    assert!(p.panel(TUE_LIFT)["text"]
        .as_str()
        .unwrap()
        .ends_with("14 of 15 checked"));
    assert_eq!(finish(&p), "Finish session");
    p.act(
        "set.toggle",
        &form(&key(last.0, last.1), json!({"weight": "100"})),
    );
    let request = p.requests()[0].to_string();
    p.act("data.apply", &request);
    assert!(p.panel(TUE_LIFT)["text"]
        .as_str()
        .unwrap()
        .ends_with("15 of 15 checked"));
    assert_eq!(finish(&p), "Everything is checked. Finish session");
}

#[test]
fn a_lift_with_no_plan_starts_each_set_at_the_weight_of_the_one_before() {
    let mut p = Page::signed_in("2026-12-01", "athlete", 1);
    // The stone press has no percentage: log 60 for set 1 and the next sets start there.
    p.act("set.toggle", &form(&key(2, 1), json!({"weight": "60"})));
    let request = p.requests()[0].to_string();
    p.act("data.apply", &request);
    let shown: Vec<String> = (0..3)
        .map(|n| {
            p.row("session-c1-1-1-b2")["sets"][n]["field"]["value"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(shown, vec!["60", "60", "60"]);
}
