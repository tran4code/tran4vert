//! The screens: each function turns the state into panels. The page draws
//! what comes back and decides nothing.
//!
//! Screens: `passcode`, `who`, `today`, `week`, `lift`, `maxes`, `cycles` and
//! `coach`. A session's blocks, working weights and voice notes are built
//! here from the rules in the other modules.

use crate::api::{MoveKey, State};
use crate::data::{AthleteRec, Data, AVATAR_MAX_BYTES, GOALS, MAX_NOTE_SECONDS, MAX_TITLE};
use crate::dates;
use crate::maxes;
use crate::moves::{self, DayView, Empty, Entry};
use crate::notes;
use crate::page::Strings;
use crate::program::{Block, BlockKind, Cycle, LibraryItem, CATEGORIES};
use crate::schedule::{self, CYCLE_WEEKS};
use crate::ui::{Action, Choice, Field, Header, NavItem, Panel, Row, SetBox};

/// Action names that change shared data; they are off while the phone is offline.
pub const CHANGES: [&str; 18] = [
    "set.toggle",
    "block.toggle",
    "session.finish",
    "session.unfinish",
    "avatar.save",
    "goals.set",
    "max.add",
    "max.delete",
    "move.save",
    "move.save_all",
    "move.undo",
    "note.save",
    "note.delete",
    "start.set",
    "cyclenote.set",
    "passcode.submit",
    "demo.start",
    "demo.coach",
];

pub struct Ctx<'a> {
    pub st: &'a State,
    pub data: &'a Data,
    pub athlete: &'a AthleteRec,
    pub today: i64,
    pub s: &'a Strings,
}

impl Ctx<'_> {
    pub fn coach(&self) -> bool {
        self.data.is_coach()
    }

    pub fn short_date(&self, day: i64) -> String {
        short_date(self.s, day)
    }

    pub fn kind_label(&self, kind: &str) -> String {
        self.s.t(&format!("kind_{kind}"))
    }
}

pub fn short_date(s: &Strings, day: i64) -> String {
    let (_, month, d) = dates::civil(day);
    s.f(
        "fmt_short_date",
        &[
            ("dow", &s.t(&format!("dow_{}", dates::weekday(day)))),
            ("day", &d.to_string()),
            ("month", &s.t(&format!("month_{month}"))),
        ],
    )
}

pub fn long_date(s: &Strings, day: i64) -> String {
    let (year, month, d) = dates::civil(day);
    s.f(
        "fmt_long_date",
        &[
            ("dow", &s.t(&format!("dow_long_{}", dates::weekday(day)))),
            ("day", &d.to_string()),
            ("month", &s.t(&format!("month_{month}"))),
            ("year", &year.to_string()),
        ],
    )
}

fn choice(value: &str, label: &str) -> Choice {
    Choice {
        value: value.to_string(),
        label: label.to_string(),
    }
}

// ---- passcode and who ----

/// The jump heights the first screen shows, in inches: before Tran4Vert, and with it.
pub const HERO_BEFORE_IN: u32 = 20;
pub const HERO_AFTER_IN: u32 = 50;

/// The first screen's top: a motivating message and two characters jumping, one
/// to the height before and one to the height after. The page draws the jumpers;
/// the words and the two heights come from here.
pub fn hero(s: &Strings) -> Panel {
    let after = HERO_AFTER_IN.to_string();
    let before = HERO_BEFORE_IN.to_string();
    let mut p = Panel::new("hero", "hero")
        .heading(&s.f("hero_title", &[("n", &after)]))
        .text(&s.t("hero_sub"));
    let mut b = Row::new("hero-before", &s.t("hero_before"));
    b.detail = s.f("hero_inches", &[("n", &before)]);
    let mut a = Row::new("hero-after", &s.t("hero_after"));
    a.detail = s.f("hero_inches", &[("n", &after)]);
    p.rows = vec![b, a];
    for n in 1..=3 {
        p.rows.push(Row::new(
            &format!("hero-line-{n}"),
            &s.t(&format!("hero_line_{n}")),
        ));
    }
    p.data.insert("before".into(), before);
    p.data.insert("after".into(), after);
    p
}

/// A person to tap: the page draws a tile with their picture, or their initial.
fn person_row(id: u64, name: &str, avatar: bool, action: Action) -> Row {
    let mut row = Row::new(&format!("name-{id}"), name);
    row.avatar = avatar.then_some(id);
    row.initial = name.chars().next().map(String::from).unwrap_or_default();
    row.actions = vec![action];
    row
}

pub fn passcode(st: &State, s: &Strings, error: &str) -> Vec<Panel> {
    let mut panels = Vec::new();
    if st.coach_field {
        // The coach's passcode stays out of sight until the coach asks for it.
        let mut p = Panel::new("passcode", "form")
            .heading(&s.t("passcode_coach_heading"))
            .text(&s.t("passcode_coach_text"));
        p.fields = vec![Field::new("coach", &s.t("passcode_coach"), "password", "")];
        p.actions = vec![
            Action::new("passcode.submit", "", &s.t("passcode_enter")),
            Action::new("passcode.group", "", &s.t("passcode_group_button")),
        ];
        if !error.is_empty() {
            p.marker = error.to_string();
        }
        return vec![p];
    }
    if let Some(id) = st.pick_for {
        let name = st
            .names
            .iter()
            .find(|n| n.id == id)
            .map(|n| n.name.clone())
            .unwrap_or_default();
        let mut p = Panel::new("goal", "list")
            .heading(&s.f("goal_heading", &[("name", &name)]))
            .text(&s.t("goal_text"));
        p.actions = GOALS
            .iter()
            .map(|g| {
                Action::new(
                    "goal.pick",
                    &g.to_string(),
                    &s.f("goal_option", &[("n", &g.to_string())]),
                )
            })
            .collect();
        p.actions.push(Action::new("login.back", "", &s.t("goal_back")));
        if !error.is_empty() {
            p.marker = error.to_string();
        }
        return vec![p];
    }
    panels.push(hero(s));
    let mut p = Panel::new("names", "people")
        .heading(&s.t("names_heading"))
        .text(&s.t("names_text"));
    if st.names.is_empty() {
        p.text = s.t("names_loading");
    }
    for n in &st.names {
        p.rows.push(person_row(
            n.id,
            &n.name,
            n.avatar,
            Action::new("login.name", &n.id.to_string(), &n.name),
        ));
    }
    if !error.is_empty() {
        p.marker = error.to_string();
    }
    panels.push(p);
    let mut c = Panel::new("coach-login", "text");
    c.actions = vec![Action::new("passcode.coach", "", &s.t("passcode_coach_button"))];
    panels.push(c);
    panels
}

pub fn who(st: &State, data: &Data, s: &Strings) -> Vec<Panel> {
    // Someone who signed in as an athlete is that athlete: no other names, only this phone.
    if let (Some(me), false) = (data.me, data.is_coach()) {
        let name = data.athlete(me).map(|a| a.name.clone()).unwrap_or_default();
        let mut you = Panel::new("who-you", "text").text(&s.f("who_you", &[("name", &name)]));
        you.actions = vec![Action::new("screen.show", "today", &s.t("back_today"))];
        return vec![you, avatar_panel(data, s, Some(me)), phone_panel(st, s)];
    }
    let mut p = Panel::new("who", "people")
        .heading(&s.t("who_heading"))
        .text(&s.t("who_text"));
    for a in &data.athletes {
        let mut row = person_row(
            a.id,
            &a.name,
            a.avatar,
            Action::new("athlete.pick", &a.id.to_string(), &a.name),
        );
        row.id = format!("athlete-{}", a.id);
        if st.athlete == Some(a.id) {
            row.marker = s.t("who_current");
        }
        p.rows.push(row);
    }
    let mut panels = Vec::new();
    if st.demo {
        panels.push(hero(s));
    }
    panels.push(p);
    if !st.demo {
        panels.push(phone_panel(st, s));
    }
    if st.demo && !data.is_coach() {
        let mut c = Panel::new("demo-coach", "text").text(&s.t("demo_coach_text"));
        c.actions = vec![Action::new("demo.coach", "", &s.t("demo_coach_button"))];
        panels.push(c);
    }
    panels
}

/// A picture for an athlete: the page lets the person choose a file and sends its size here first.
/// `only` is the one athlete an athlete may change; the coach may change anyone.
fn avatar_panel(data: &Data, s: &Strings, only: Option<u64>) -> Panel {
    let mut p = Panel::new("avatar", "avatar")
        .heading(&s.t("avatar_heading"))
        .text(&s.f("avatar_text", &[("kb", &(AVATAR_MAX_BYTES / 1000).to_string())]));
    let choices: Vec<Choice> = data
        .athletes
        .iter()
        .filter(|a| only.is_none_or(|id| id == a.id))
        .map(|a| choice(&a.id.to_string(), &a.name))
        .collect();
    let first = choices.first().map(|c| c.value.clone()).unwrap_or_default();
    p.fields = vec![Field::new("athlete", &s.t("avatar_athlete"), "select", &first).with_choices(choices)];
    p
}

/// What this phone keeps, and a way to forget it, asked about before it is done.
fn phone_panel(st: &State, s: &Strings) -> Panel {
    let mut p = Panel::new("phone", "text").heading(&s.t("phone_heading"));
    if st.forgetting {
        p.text = s.t("phone_forget_ask");
        p.actions = vec![
            Action::new("page.forget", "", &s.t("phone_forget_yes")),
            Action::new("phone.keep", "", &s.t("phone_keep")),
        ];
    } else {
        p.text = s.t("phone_text");
        p.actions = vec![Action::new("phone.ask", "", &s.t("phone_forget"))];
    }
    p
}

// ---- a session ----

fn library_row(ctx: &Ctx, id: &str, item: &LibraryItem, dose: Option<&str>) -> Row {
    let s = ctx.s;
    let mut row = Row::new(id, &item.name);
    let mut parts: Vec<String> = Vec::new();
    match item.kind.as_str() {
        "hold" => {
            if let (Some(sets), Some(secs)) = (item.sets, item.secs) {
                parts.push(s.f(
                    "rx_hold",
                    &[("sets", &sets.to_string()), ("secs", &secs.to_string())],
                ));
            }
        }
        _ => {
            if let Some(r) = item.rounds {
                parts.push(s.f("rx_rounds", &[("rounds", &r.to_string())]));
            }
            if let Some(d) = &item.dose {
                parts.push(d.clone());
            }
        }
    }
    if let Some(d) = dose {
        parts = vec![d.to_string()];
    }
    row.detail = parts.join(" · ");
    row.rows = item
        .items
        .iter()
        .enumerate()
        .map(|(i, text)| Row::new(&format!("{id}-{i}"), text))
        .collect();
    row
}

fn prescription(ctx: &Ctx, block: &Block) -> String {
    let s = ctx.s;
    let mut parts: Vec<String> = Vec::new();
    if block.amap {
        parts.push(s.t("rx_amap"));
    } else if block.hold {
        if let (Some(sets), Some(secs)) = (block.sets, block.secs) {
            parts.push(s.f(
                "rx_hold",
                &[("sets", &sets.to_string()), ("secs", &secs.to_string())],
            ));
        }
    } else if let (Some(sets), Some(reps)) = (block.sets, block.reps) {
        let key = if block.each_side {
            "rx_each_side"
        } else {
            "rx_sets_reps"
        };
        parts.push(s.f(key, &[("sets", &sets.to_string()), ("reps", &reps.to_string())]));
    }
    if let Some(p) = &block.pct {
        parts.push(s.f(
            "rx_pct",
            &[("last", &p.last.to_string()), ("pct", &p.pct.to_string())],
        ));
    }
    parts.join(" · ")
}

fn lift_row(ctx: &Ctx, cycle: &Cycle, block: &Block, id: &str, date: i64) -> Row {
    let s = ctx.s;
    let lift = block.lift.clone().unwrap_or_default();
    let mut row = Row::new(id, &ctx.data.program.lift_name(&lift));
    row.detail = prescription(ctx, block);
    row.open = true;
    if let Some(cat) = ctx.data.program.block_category(block) {
        row.category = cat.to_string();
        row.tag = s.t(&format!("cat_{cat}"));
    }
    row.actions = vec![Action::new("lift.show", &lift, &s.t("lift_open"))];
    if let Some(p) = &block.pct {
        let of_name = ctx.data.program.lift_name(ctx.data.program.max_lift(&p.of));
        let mut w = Row::new(&format!("{id}-weight"), "");
        match maxes::weight_for(ctx.data, ctx.athlete.id, &p.of, p.pct, date) {
            Some(found) => {
                w.label = s.f(
                    "ww_line",
                    &[
                        ("last", &p.last.to_string()),
                        ("weight", &found.weight.to_string()),
                    ],
                );
                w.detail = s.f(
                    "ww_basis",
                    &[
                        ("pct", &p.pct.to_string()),
                        ("lift", &of_name),
                        ("max", &found.max.to_string()),
                    ],
                );
            }
            None => {
                w.label = s.f("ww_none", &[("pct", &p.pct.to_string()), ("lift", &of_name)]);
                w.actions = vec![Action::new(
                    "max.prefill",
                    ctx.data.program.max_lift(&p.of),
                    &s.t("max_add_button"),
                )];
            }
        }
        w.tag = s.t("tag_working_weight");
        row.rows.push(w);
    }
    if let Some(note) = &block.note {
        let text = ctx.data.program.note_text(cycle, note);
        if !text.is_empty() {
            let mut n = Row::new(&format!("{id}-note"), &s.t("tag_coach_note"));
            n.note = text;
            row.rows.push(n);
        }
    }
    if let Some(test) = &block.test {
        let mut t = Row::new(&format!("{id}-test"), &s.f("test_line", &[("test", test)]));
        t.tag = s.t("tag_test");
        t.actions = vec![Action::new("max.prefill", &lift, &s.t("test_enter"))];
        row.rows.push(t);
    }
    for n in notes::notes_for(ctx.data, &lift) {
        row.rows.push(note_row(ctx, n, &format!("{id}-voice-{}", n.id)));
    }
    row
}

fn note_row(ctx: &Ctx, n: &crate::data::NoteRec, id: &str) -> Row {
    let s = ctx.s;
    let mut r = Row::new(id, &n.title);
    r.tag = s.t("tag_voice_note");
    r.detail = s.f(
        "note_detail",
        &[("length", &notes::length(n.seconds)), ("date", &n.date)],
    );
    r.actions = vec![Action::new(
        "note.play",
        &n.id.to_string(),
        &s.f("note_play", &[("title", &n.title)]),
    )];
    r
}

fn block_row(ctx: &Ctx, cycle: &Cycle, block: &Block, id: &str, date: i64) -> Row {
    let s = ctx.s;
    match block.kind() {
        Some(BlockKind::Ref) => {
            let key = block.reference.clone().unwrap_or_default();
            match ctx.data.program.library.get(&key) {
                Some(item) => {
                    let mut row = library_row(ctx, id, item, block.dose.as_deref());
                    if let Some(cat) = ctx.data.program.block_category(block) {
                        row.category = cat.to_string();
                        row.tag = s.t(&format!("cat_{cat}"));
                    }
                    row
                }
                None => Row::new(id, &key),
            }
        }
        Some(BlockKind::Lift) => lift_row(ctx, cycle, block, id, date),
        Some(BlockKind::Text) => {
            let mut r = Row::new(id, block.text.as_deref().unwrap_or_default());
            r.detail = block.detail.clone().unwrap_or_default();
            r
        }
        _ => {
            let mut r = Row::new(id, &s.t("tag_coach_note"));
            r.note = ctx
                .data
                .program
                .note_text(cycle, block.note.as_deref().unwrap_or_default());
            r
        }
    }
}

fn where_text(ctx: &Ctx, slot: &schedule::Slot<'_>) -> String {
    let theme = slot
        .cycle_ref
        .week_themes
        .get(slot.week)
        .and_then(|t| t.clone())
        .unwrap_or_default();
    let place = ctx.s.f(
        "session_where",
        &[
            ("cycle", &slot.cycle_ref.name),
            ("week", &(slot.week + 1).to_string()),
        ],
    );
    if theme.is_empty() {
        place
    } else {
        format!("{place} · {theme}")
    }
}

fn session_key(slot: &schedule::Slot<'_>) -> MoveKey {
    MoveKey {
        cycle: slot.cycle_ref.id.clone(),
        week: slot.week as u32 + 1,
        dow: slot.dow,
    }
}

fn key_value(key: &MoveKey) -> String {
    serde_json::json!({"cycle": key.cycle, "week": key.week, "dow": key.dow}).to_string()
}

pub fn day_panels(ctx: &Ctx, date: i64) -> Vec<Panel> {
    let s = ctx.s;
    let view: DayView = moves::day_view(ctx.data, ctx.athlete, date);
    let mut panels = Vec::new();
    for entry in &view.entries {
        match entry {
            Entry::Session { slot, moved_from } => {
                let key = session_key(slot);
                let id = format!("session-{}-{}-{}", key.cycle, key.week, key.dow);
                let mut p = Panel::new(&id, "session").heading(&ctx.kind_label(&slot.day.kind));
                p.text = where_text(ctx, slot);
                if let Some(from) = moved_from {
                    p.marker = s.f("moved_from", &[("date", &ctx.short_date(*from))]);
                }
                for (i, block) in slot.day.blocks.iter().enumerate() {
                    p.rows
                        .push(block_row(ctx, slot.cycle_ref, block, &format!("{id}-b{i}"), date));
                }
                let (done, total) = checks(ctx, slot, &key, &mut p.rows, date);
                if total > 0 {
                    p.text = format!(
                        "{} · {}",
                        p.text,
                        s.f(
                            "progress_text",
                            &[("done", &done.to_string()), ("total", &total.to_string())]
                        )
                    );
                }
                let finished = ctx.data.finishes.iter().any(|f| {
                    f.athlete == ctx.athlete.id
                        && f.cycle == key.cycle
                        && f.week == key.week
                        && f.dow == key.dow
                });
                if finished {
                    p.marker = if p.marker.is_empty() {
                        s.t("session_finished")
                    } else {
                        format!("{} · {}", p.marker, s.t("session_finished"))
                    };
                    p.actions.push(Action::new(
                        "session.unfinish",
                        &key_value(&key),
                        &s.t("finish_undo"),
                    ));
                } else {
                    // When every box is checked the button says so.
                    let label = if total > 0 && done == total {
                        s.t("finish_ready")
                    } else {
                        s.t("finish_button")
                    };
                    p.actions
                        .push(Action::new("session.finish", &key_value(&key), &label));
                }
                p.actions
                    .push(Action::new("move.start", &key_value(&key), &s.t("move_button")));
                if moved_from.is_some() {
                    p.actions
                        .push(Action::new("move.undo", &key_value(&key), &s.t("move_undo")));
                }
                panels.push(p);
                if ctx.st.moving.as_ref() == Some(&key) {
                    panels.push(move_form(ctx, &key, date));
                }
            }
            Entry::MovedOut { slot, to } => {
                let key = session_key(slot);
                let mut p = Panel::new(
                    &format!("moved-{}-{}-{}", key.cycle, key.week, key.dow),
                    "session",
                )
                .heading(&ctx.kind_label(&slot.day.kind));
                p.text = where_text(ctx, slot);
                p.marker = s.f("moved_to", &[("date", &ctx.short_date(*to))]);
                p.actions
                    .push(Action::new("day.show", &dates::format(*to), &s.t("moved_go")));
                p.actions
                    .push(Action::new("move.undo", &key_value(&key), &s.t("move_undo")));
                panels.push(p);
            }
        }
    }
    if let Some(empty) = &view.empty {
        let text = match empty {
            Empty::BeforeStart(start) => s.f("empty_before", &[("date", &long_date(s, *start))]),
            Empty::NotLoaded { cycle, week } => s.f(
                "empty_not_loaded",
                &[
                    ("cycle", &(cycle + 1).to_string()),
                    ("week", &(week + 1).to_string()),
                ],
            ),
            Empty::Rest => s.t("empty_rest"),
        };
        let heading = match empty {
            Empty::Rest => ctx.kind_label("rest"),
            Empty::NotLoaded { .. } => s.t("tag_not_loaded"),
            Empty::BeforeStart(_) => s.t("tag_not_started"),
        };
        panels.push(Panel::new("empty", "text").heading(&heading).text(&text));
    }
    panels
}

fn move_form(ctx: &Ctx, key: &MoveKey, date: i64) -> Panel {
    let s = ctx.s;
    let mut p = Panel::new("move", "form")
        .heading(&s.t("move_heading"))
        .text(&s.t("move_text"));
    p.fields = vec![Field::new("to", &s.t("move_to"), "date", &dates::format(date))];
    let v = key_value(key);
    p.actions.push(Action::new("move.save", &v, &s.t("move_save")));
    if ctx.coach() {
        p.actions
            .push(Action::new("move.save_all", &v, &s.t("move_save_all")));
    }
    p.actions.push(Action::new("move.cancel", "", &s.t("cancel")));
    p
}

/// The set boxes, weight fields and block checks of a session, from the athlete's own log.
/// Returns how many boxes are checked and how many there are.
fn checks(
    ctx: &Ctx,
    slot: &schedule::Slot<'_>,
    key: &MoveKey,
    rows: &mut [Row],
    date: i64,
) -> (usize, usize) {
    let s = ctx.s;
    let me = ctx.athlete.id;
    let logged = |block: usize, set: u32| {
        ctx.data.logs.iter().find(|l| {
            l.athlete == me
                && l.cycle == key.cycle
                && l.week == key.week
                && l.dow == key.dow
                && l.block as usize == block
                && l.set == set
        })
    };
    let (mut done, mut total) = (0, 0);
    for (i, block) in slot.day.blocks.iter().enumerate() {
        let Some(row) = rows.get_mut(i) else { continue };
        let value = |set: u32| {
            serde_json::json!({"cycle": key.cycle, "week": key.week, "dow": key.dow, "block": i, "set": set})
                .to_string()
        };
        let sets = block
            .sets
            .filter(|n| *n > 0 && !block.amap)
            .unwrap_or(0)
            .min(crate::data::MAX_SETS);
        if block.lift.is_some() && sets > 0 {
            // The program names a percentage for the last sets; the ones before are the athlete's own ramp-up.
            let working_from = match &block.pct {
                Some(p) => sets.saturating_sub(p.last) + 1,
                None => 1,
            };
            let planned = block
                .pct
                .as_ref()
                .and_then(|p| maxes::weight_for(ctx.data, me, &p.of, p.pct, date))
                .map(|w| w.weight);
            let (mut ramp_done, mut ramp_total, mut work_done, mut work_total) = (0, 0, 0, 0);
            let mut carried: Option<u32> = None;
            for n in 1..=sets {
                let log = logged(i, n);
                let on = log.is_some();
                // Only a lift with a prescribed percentage has working sets, marked as such.
                let working = block.pct.is_some() && n >= working_from;
                total += 1;
                done += usize::from(on);
                if working {
                    work_total += 1;
                    work_done += usize::from(on);
                } else {
                    ramp_total += 1;
                    ramp_done += usize::from(on);
                }
                let label = s.f(
                    if working { "set_box_working" } else { "set_box" },
                    &[("n", &n.to_string())],
                );
                let action = Action::new("set.toggle", &value(n), &label).pressed(on);
                // A set shows the weight logged for it; a working set starts at the plan, and a lift with no
                // plan starts at the weight of the set before. A ramp-up set starts empty.
                let field = if block.hold {
                    None
                } else {
                    let shown = log.and_then(|l| l.weight).or(if working {
                        planned
                    } else if block.pct.is_none() {
                        carried
                    } else {
                        None
                    });
                    carried = log.and_then(|l| l.weight).or(carried);
                    Some(Field::new(
                        "weight",
                        &s.f("set_weight", &[("n", &n.to_string())]),
                        "number",
                        &shown.map(|w| w.to_string()).unwrap_or_default(),
                    ))
                };
                row.sets.push(SetBox {
                    action,
                    field,
                    working,
                });
            }
            row.progress = if block.pct.is_some() && ramp_total > 0 {
                s.f(
                    "set_progress_ramp",
                    &[
                        ("a", &ramp_done.to_string()),
                        ("b", &ramp_total.to_string()),
                        ("c", &work_done.to_string()),
                        ("d", &work_total.to_string()),
                    ],
                )
            } else {
                s.f(
                    "set_progress",
                    &[
                        ("a", &(ramp_done + work_done).to_string()),
                        ("b", &sets.to_string()),
                    ],
                )
            };
            if ramp_done + work_done == sets as usize {
                row.marker = s.t("lift_complete");
            }
        } else if block.kind() != Some(BlockKind::Note) {
            let on = logged(i, 0).is_some();
            total += 1;
            done += usize::from(on);
            row.actions
                .push(Action::new("block.toggle", &value(0), &s.t("block_box")).pressed(on));
        }
    }
    (done, total)
}

/// The message to show on Today, if there is one not yet read.
fn popup(ctx: &Ctx) -> Option<Panel> {
    let s = ctx.s;
    let first = crate::rivalry::messages(ctx.data, s, ctx.athlete, ctx.today, &ctx.st.seen)
        .into_iter()
        .next()?;
    let mut p = Panel::new("popup", "popup")
        .heading(&s.t("popup_heading"))
        .text(&first.text);
    p.actions = vec![Action::new("msg.dismiss", &first.id, &s.t("popup_ok"))];
    Some(p)
}

pub fn today(ctx: &Ctx) -> (String, Vec<Panel>) {
    let date = ctx.st.day.as_deref().and_then(dates::parse).unwrap_or(ctx.today);
    let mut title = long_date(ctx.s, date);
    if date == ctx.today {
        title = ctx.s.f("today_title", &[("date", &title)]);
    }
    let mut panels = Vec::new();
    // A message about today's session, shown over everything else until it is read.
    if date == ctx.today {
        panels.extend(popup(ctx));
    }
    panels.extend(day_panels(ctx, date));
    if date != ctx.today {
        let mut back = Panel::new("back-today", "text");
        back.actions = vec![Action::new("screen.show", "today", &ctx.s.t("back_today"))];
        panels.push(back);
    }
    (title, panels)
}

// ---- the week ----

pub fn week(ctx: &Ctx) -> (String, Vec<Panel>) {
    let s = ctx.s;
    let start = dates::parse(&ctx.athlete.start).unwrap_or(ctx.today);
    let current = schedule::week_number(start, ctx.today).max(0);
    let number = current + ctx.st.week_offset;
    let monday = schedule::week_start(start, number);
    let place = if number < 0 {
        s.t("week_before")
    } else {
        let cycle = (number / CYCLE_WEEKS) as usize;
        let name = ctx
            .data
            .program
            .cycles
            .get(cycle)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| s.f("cycle_n", &[("n", &(cycle + 1).to_string())]));
        s.f(
            "week_place",
            &[
                ("cycle", &name),
                ("week", &(number % CYCLE_WEEKS + 1).to_string()),
            ],
        )
    };
    let title = s.f(
        "week_title",
        &[
            ("place", &place),
            ("from", &short_date(s, monday)),
            ("to", &short_date(s, monday + 6)),
        ],
    );
    let mut list = Panel::new("week", "list");
    for offset in 0..7 {
        let date = monday + offset;
        let view = moves::day_view(ctx.data, ctx.athlete, date);
        let mut row = Row::new(&format!("day-{}", dates::format(date)), &short_date(s, date));
        let mut names: Vec<String> = Vec::new();
        for entry in &view.entries {
            match entry {
                Entry::Session { slot, moved_from } => {
                    if row.tag.is_empty() {
                        row.tag = ctx.kind_label(&slot.day.kind);
                    } else {
                        row.tag = format!("{} + {}", row.tag, ctx.kind_label(&slot.day.kind));
                    }
                    for b in &slot.day.blocks {
                        if let Some(l) = &b.lift {
                            let n = ctx.data.program.lift_name(l);
                            if !names.contains(&n) {
                                names.push(n);
                            }
                        }
                    }
                    if let Some(from) = moved_from {
                        row.marker = s.f("moved_from", &[("date", &short_date(s, *from))]);
                    }
                }
                Entry::MovedOut { to, .. } => {
                    row.marker = s.f("moved_to", &[("date", &short_date(s, *to))]);
                    if row.tag.is_empty() {
                        row.tag = s.t("tag_moved");
                    }
                }
            }
        }
        if row.tag.is_empty() {
            row.tag = match &view.empty {
                Some(Empty::NotLoaded { .. }) => s.t("tag_not_loaded"),
                Some(Empty::BeforeStart(_)) => s.t("tag_not_started"),
                _ => ctx.kind_label("rest"),
            };
        }
        // The categories the day holds, in the fixed order, each as a coloured word.
        let held: Vec<&str> = view
            .entries
            .iter()
            .filter_map(|e| match e {
                Entry::Session { slot, .. } => Some(slot),
                Entry::MovedOut { .. } => None,
            })
            .flat_map(|slot| slot.day.blocks.iter())
            .filter_map(|b| ctx.data.program.block_category(b))
            .collect();
        row.chips = CATEGORIES
            .iter()
            .filter(|c| held.contains(c))
            .map(|c| choice(c, &s.t(&format!("cat_{c}"))))
            .collect();
        row.detail = names.join(", ");
        if date == ctx.today {
            row.note = String::new();
            row.detail = if row.detail.is_empty() {
                s.t("tag_today")
            } else {
                format!("{} · {}", s.t("tag_today"), row.detail)
            };
        }
        row.actions = vec![Action::new("day.show", &dates::format(date), &s.t("day_open"))];
        list.rows.push(row);
    }
    let mut nav = Panel::new("week-nav", "text");
    nav.actions = vec![
        Action::new("week.shift", "-1", &s.t("week_earlier")),
        Action::new("week.shift", "0", &s.t("week_this")),
        Action::new("week.shift", "1", &s.t("week_later")),
    ];
    let mut legend = Panel::new("legend", "legend").heading(&s.t("legend_heading"));
    for c in CATEGORIES {
        let mut r = Row::new(&format!("legend-{c}"), &s.t(&format!("cat_{c}")));
        r.category = c.to_string();
        legend.rows.push(r);
    }
    (title, vec![nav, list, legend])
}

// ---- a lift ----

fn max_choices(ctx: &Ctx) -> Vec<Choice> {
    ctx.data
        .program
        .max_lifts()
        .into_iter()
        .map(|id| choice(id, &ctx.data.program.lift_name(id)))
        .collect()
}

fn lift_choices(ctx: &Ctx) -> Vec<Choice> {
    ctx.data
        .program
        .lifts
        .iter()
        .map(|(id, l)| choice(id, &l.name))
        .collect()
}

pub fn recorder(ctx: &Ctx, lift: &str) -> Panel {
    let s = ctx.s;
    let mut p = Panel::new("recorder", "recorder")
        .heading(&s.t("rec_heading"))
        .text(&s.t("rec_text"));
    p.limit = Some(MAX_NOTE_SECONDS);
    p.lifts = lift_choices(ctx);
    p.fields = vec![
        Field::new("lift", &s.t("rec_lift"), "select", lift).with_choices(lift_choices(ctx)),
        Field::new("title", &s.t("rec_title"), "text", "").with_max(MAX_TITLE as u32),
    ];
    p.actions = vec![
        Action::new("rec.start", "", &s.t("rec_start")),
        Action::new("rec.stop", "", &s.t("rec_stop")),
        Action::new("rec.listen", "", &s.t("rec_listen")),
        Action::new("rec.save", "", &s.t("rec_save")),
        Action::new("rec.discard", "", &s.t("rec_discard")),
    ];
    p.marker = s.f(
        "rec_limit",
        &[
            ("seconds", &MAX_NOTE_SECONDS.to_string()),
            ("chars", &MAX_TITLE.to_string()),
        ],
    );
    p
}

pub fn lift(ctx: &Ctx, lift: &str) -> (String, Vec<Panel>) {
    let s = ctx.s;
    let name = ctx.data.program.lift_name(lift);
    let mut panels = Vec::new();
    let max_lift = ctx.data.program.max_lift(lift);
    let mut mine = Panel::new("lift-max", "list").heading(&s.t("lift_max_heading"));
    if max_lift != lift {
        mine.text = s.f(
            "lift_max_from",
            &[("lift", &ctx.data.program.lift_name(max_lift))],
        );
    }
    match maxes::current_max(ctx.data, ctx.athlete.id, max_lift, ctx.today) {
        Some(m) => {
            mine.rows.push(Row::new(
                "lift-max-now",
                &s.f(
                    "max_line",
                    &[("weight", &m.weight.to_string()), ("date", &m.date)],
                ),
            ));
        }
        None => mine.text = format!("{} {}", mine.text, s.t("max_none")).trim().to_string(),
    }
    mine.actions = vec![Action::new("max.prefill", max_lift, &s.t("max_add_button"))];
    panels.push(mine);
    let mut voice = Panel::new("lift-notes", "list").heading(&s.t("lift_notes_heading"));
    for n in notes::notes_for(ctx.data, lift) {
        let mut r = note_row(ctx, n, &format!("note-{}", n.id));
        if ctx.coach() {
            r.actions
                .push(Action::new("note.delete", &n.id.to_string(), &s.t("note_delete")));
        }
        voice.rows.push(r);
    }
    if voice.rows.is_empty() {
        voice.text = s.t("lift_notes_none");
    }
    panels.push(voice);
    if ctx.coach() {
        panels.push(recorder(ctx, lift));
    }
    (name, panels)
}

// ---- maxes ----

pub fn maxes_screen(ctx: &Ctx) -> (String, Vec<Panel>) {
    let s = ctx.s;
    let mut panels = Vec::new();
    for lift in ctx.data.program.max_lifts() {
        let mut p = Panel::new(&format!("maxes-{lift}"), "list").heading(&ctx.data.program.lift_name(lift));
        for a in &ctx.data.athletes {
            let hist = maxes::history(ctx.data, a.id, lift);
            let mut row = Row::new(&format!("max-{lift}-{}", a.id), &a.name);
            match maxes::current_max(ctx.data, a.id, lift, ctx.today) {
                Some(m) => {
                    row.detail = s.f(
                        "max_line",
                        &[("weight", &m.weight.to_string()), ("date", &m.date)],
                    );
                }
                None => row.detail = s.t("max_none_short"),
            }
            if a.id == ctx.athlete.id {
                row.marker = s.t("who_current");
            }
            for (i, m) in hist.iter().enumerate() {
                let mut h = Row::new(
                    &format!("maxrec-{}", m.id),
                    &s.f(
                        "max_line",
                        &[("weight", &m.weight.to_string()), ("date", &m.date)],
                    ),
                );
                // How this max compares with the one before it.
                if let Some(before) = hist.get(i + 1) {
                    let n = m.weight.abs_diff(before.weight).to_string();
                    let key = match m.weight.cmp(&before.weight) {
                        std::cmp::Ordering::Greater => "max_up",
                        std::cmp::Ordering::Less => "max_down",
                        std::cmp::Ordering::Equal => "max_same",
                    };
                    h.detail = s.f(key, &[("n", &n), ("date", &before.date)]);
                }
                if a.id == ctx.athlete.id || ctx.coach() {
                    h.actions = vec![Action::new("max.delete", &m.id.to_string(), &s.t("max_delete"))];
                }
                row.rows.push(h);
            }
            p.rows.push(row);
        }
        panels.push(p);
    }
    let mut form = Panel::new("max-form", "form")
        .heading(&s.t("max_form_heading"))
        .text(&s.t("max_form_text"));
    let first = ctx
        .data
        .program
        .max_lifts()
        .first()
        .map(|x| x.to_string())
        .unwrap_or_default();
    let chosen = ctx.st.prefill.clone().unwrap_or(first);
    form.fields =
        vec![Field::new("lift", &s.t("max_lift"), "select", &chosen).with_choices(max_choices(ctx))];
    if ctx.coach() {
        form.fields.push(
            Field::new(
                "athlete",
                &s.t("max_athlete"),
                "select",
                &ctx.athlete.id.to_string(),
            )
            .with_choices(
                ctx.data
                    .athletes
                    .iter()
                    .map(|a| choice(&a.id.to_string(), &a.name))
                    .collect(),
            ),
        );
    }
    form.fields
        .push(Field::new("weight", &s.t("max_weight"), "number", ""));
    form.fields.push(Field::new(
        "date",
        &s.t("max_date"),
        "date",
        &dates::format(ctx.today),
    ));
    form.actions = vec![Action::new("max.add", "", &s.t("max_save"))];
    panels.insert(0, form);
    (s.t("title_maxes"), panels)
}

// ---- cycles ----

pub fn cycles(ctx: &Ctx) -> (String, Vec<Panel>) {
    let s = ctx.s;
    let start = dates::parse(&ctx.athlete.start).unwrap_or(ctx.today);
    let here = schedule::week_number(start, ctx.today);
    let mut panels = Vec::new();
    for (c, cycle) in ctx.data.program.cycles.iter().enumerate() {
        let mut p = Panel::new(&format!("cycle-{}", cycle.id), "list").heading(&cycle.name);
        p.text = s.f("cycle_emphasis", &[("emphasis", &cycle.emphasis)]);
        for w in 0..CYCLE_WEEKS as usize {
            let week = cycle.weeks.get(w);
            let loaded = week.is_some_and(|x| !x.missing);
            let theme = cycle.week_themes.get(w).and_then(|t| t.clone());
            let mut row = Row::new(
                &format!("cycle-{}-w{}", cycle.id, w + 1),
                &s.f("week_n", &[("week", &(w + 1).to_string())]),
            );
            row.detail = theme.unwrap_or_else(|| s.t("theme_none"));
            row.tag = if loaded {
                s.t("tag_loaded")
            } else {
                s.t("tag_not_loaded")
            };
            if here >= 0 && here == (c as i64) * CYCLE_WEEKS + w as i64 {
                row.marker = s.t("cycle_here");
            }
            p.rows.push(row);
        }
        let note = ctx
            .data
            .cycle_notes
            .iter()
            .find(|n| n.cycle == cycle.id)
            .map(|n| n.text.clone())
            .unwrap_or_else(|| cycle.cycle_note.clone());
        if !note.is_empty() {
            let mut n = Row::new(&format!("cycle-{}-note", cycle.id), &s.t("tag_coach_note"));
            n.note = note;
            p.rows.push(n);
        }
        panels.push(p);
    }
    let mut gaps = Panel::new("not-captured", "list").heading(&s.t("not_captured_heading"));
    for (i, line) in ctx.data.program.not_captured.iter().enumerate() {
        gaps.rows.push(Row::new(&format!("gap-{i}"), line));
    }
    if gaps.rows.is_empty() {
        gaps.text = s.t("not_captured_none");
    }
    panels.push(gaps);
    (s.t("title_cycles"), panels)
}

// ---- the coach ----

pub fn coach(ctx: &Ctx) -> (String, Vec<Panel>) {
    let s = ctx.s;
    let mut panels = Vec::new();
    let all: usize = ctx.data.program.cycles.iter().map(|c| c.weeks.len()).sum();
    let loaded: usize = ctx
        .data
        .program
        .cycles
        .iter()
        .map(|c| c.weeks.iter().filter(|w| !w.missing).count())
        .sum();
    panels.push(
        Panel::new("coach-program", "text")
            .heading(&s.t("coach_program_heading"))
            .text(&s.f(
                "coach_program_text",
                &[("loaded", &loaded.to_string()), ("all", &all.to_string())],
            )),
    );
    let mut starts = Panel::new("coach-starts", "form")
        .heading(&s.t("coach_starts_heading"))
        .text(&s.t("coach_starts_text"));
    for a in &ctx.data.athletes {
        starts
            .fields
            .push(Field::new(&format!("start_{}", a.id), &a.name, "date", &a.start));
    }
    starts.actions = vec![Action::new("start.set", "", &s.t("coach_starts_save"))];
    panels.push(starts);
    let mut goals = Panel::new("coach-goals", "form")
        .heading(&s.t("coach_goals_heading"))
        .text(&s.t("coach_goals_text"));
    for a in &ctx.data.athletes {
        let mut choices = vec![choice("", &s.t("goal_not_set"))];
        choices.extend(
            GOALS
                .iter()
                .map(|g| choice(&g.to_string(), &s.f("goal_option", &[("n", &g.to_string())]))),
        );
        let current = a.goal.map(|g| g.to_string()).unwrap_or_default();
        goals
            .fields
            .push(Field::new(&format!("goal_{}", a.id), &a.name, "select", &current).with_choices(choices));
    }
    goals.actions = vec![Action::new("goals.set", "", &s.t("goals_save"))];
    panels.push(goals);
    panels.push(avatar_panel(ctx.data, ctx.s, None));
    panels.push(recorder(ctx, ""));
    let mut notes_panel = Panel::new("coach-notes", "list").heading(&s.t("coach_notes_heading"));
    for n in &ctx.data.notes {
        let mut r = note_row(ctx, n, &format!("cn-{}", n.id));
        r.label = format!("{} · {}", ctx.data.program.lift_name(&n.lift), n.title);
        r.actions
            .push(Action::new("note.delete", &n.id.to_string(), &s.t("note_delete")));
        notes_panel.rows.push(r);
    }
    if notes_panel.rows.is_empty() {
        notes_panel.text = s.t("coach_notes_none");
    }
    panels.push(notes_panel);
    let mut cn = Panel::new("coach-cycle-notes", "form")
        .heading(&s.t("coach_cycle_heading"))
        .text(&s.t("coach_cycle_text"));
    for cycle in &ctx.data.program.cycles {
        let current = ctx
            .data
            .cycle_notes
            .iter()
            .find(|n| n.cycle == cycle.id)
            .map(|n| n.text.clone())
            .unwrap_or_default();
        cn.fields.push(Field::new(
            &format!("cycle_{}", cycle.id),
            &cycle.name,
            "text",
            &current,
        ));
    }
    cn.actions = vec![Action::new("cyclenote.set", "", &s.t("coach_cycle_save"))];
    panels.push(cn);
    (s.t("title_coach"), panels)
}

pub fn nav(ctx: &Ctx, screen: &str) -> Vec<NavItem> {
    let s = ctx.s;
    let mut items = vec![
        ("today", s.t("nav_today")),
        ("week", s.t("nav_week")),
        ("maxes", s.t("nav_maxes")),
        ("cycles", s.t("nav_cycles")),
    ];
    if ctx.coach() {
        items.push(("coach", s.t("nav_coach")));
    }
    items
        .into_iter()
        .map(|(name, label)| NavItem {
            current: name == screen || (name == "today" && screen == "lift"),
            label,
            name: "screen.show".to_string(),
            value: name.to_string(),
        })
        .collect()
}

pub fn header(ctx: Option<&Ctx>, st: &State, s: &Strings) -> Header {
    let mut h = Header::default();
    if let Some(c) = ctx {
        h.who = c.athlete.name.clone();
        h.coach = c.coach();
        h.avatar = c.athlete.avatar.then_some(c.athlete.id);
    }
    if st.offline {
        let as_of = st.data.as_ref().map(|d| d.as_of.clone()).unwrap_or_default();
        h.offline = s.f("offline_note", &[("date", &as_of)]);
    }
    h
}
