//! Messages between athletes who train the same program: who finished a
//! session first, who put up more weight on a lift, and a word of
//! encouragement after finishing. They are for the athlete's own session of
//! today, so an old finish does not pop up again. Which line is used is picked
//! from the athletes and the day, never at random, so the same state gives the
//! same message. A message's id names the athlete it is for, so two athletes
//! on one phone each read their own.

use crate::data::{AthleteRec, Data};
use crate::moves::{day_view, Entry};
use crate::page::Strings;

/// A message and the id the phone remembers once it has been read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: String,
    pub text: String,
}

/// How many wordings each kind of message has.
const VARIANTS: u64 = 3;

fn variant(me: u64, other: u64, dow: u8) -> u64 {
    (me + other + u64::from(dow)) % VARIANTS + 1
}

/// The heaviest weight an athlete logged on a lift in one session.
fn heaviest(
    data: &Data,
    athlete: u64,
    cycle: &str,
    week: u32,
    dow: u8,
    lift: &str,
    blocks: &[Option<&str>],
) -> Option<u32> {
    data.logs
        .iter()
        .filter(|l| l.athlete == athlete && l.cycle == cycle && l.week == week && l.dow == dow)
        .filter(|l| blocks.get(l.block as usize).copied().flatten() == Some(lift))
        .filter_map(|l| l.weight)
        .max()
}

/// The messages for an athlete on a date, most important first: their own
/// finish, then each rival who finished first, then each rival who lifted more.
/// Messages the phone has already shown (`seen`) are left out.
pub fn messages(data: &Data, s: &Strings, me: &AthleteRec, today: i64, seen: &[String]) -> Vec<Message> {
    let mut out = Vec::new();
    for entry in day_view(data, me, today).entries {
        let Entry::Session { slot, .. } = entry else {
            continue;
        };
        let cycle = slot.cycle_ref.id.clone();
        let week = slot.week as u32 + 1;
        let dow = slot.dow;
        let key = format!("{cycle}:{week}:{dow}");
        let finished = |who: u64| {
            data.finishes
                .iter()
                .find(|f| f.athlete == who && f.cycle == cycle && f.week == week && f.dow == dow)
        };
        let rivals: Vec<&AthleteRec> = data.athletes.iter().filter(|a| a.id != me.id).collect();
        if finished(me.id).is_some() {
            let done_this_week = data
                .finishes
                .iter()
                .filter(|f| f.athlete == me.id && f.cycle == cycle && f.week == week)
                .count();
            let v = variant(me.id, 0, dow);
            let mut text = s.f(
                &format!("own_done_{v}"),
                &[("name", &me.name), ("n", &done_this_week.to_string())],
            );
            if let Some(behind) = rivals.iter().find(|r| finished(r.id).is_none()) {
                text = format!("{text} {}", s.f("own_ahead", &[("rival", &behind.name)]));
            }
            out.push(Message {
                id: format!("own:{}:{key}", me.id),
                text,
            });
        } else {
            for r in &rivals {
                if finished(r.id).is_some() {
                    let v = variant(me.id, r.id, dow);
                    out.push(Message {
                        id: format!("first:{}:{}:{key}", me.id, r.id),
                        text: s.f(&format!("rival_first_{v}"), &[("rival", &r.name)]),
                    });
                }
            }
        }
        let lifts: Vec<Option<&str>> = slot.day.blocks.iter().map(|b| b.lift.as_deref()).collect();
        let mut seen_lifts: Vec<&str> = Vec::new();
        for lift in lifts.iter().flatten() {
            if seen_lifts.contains(lift) {
                continue;
            }
            seen_lifts.push(lift);
            let mine = heaviest(data, me.id, &cycle, week, dow, lift, &lifts);
            for r in &rivals {
                let theirs = heaviest(data, r.id, &cycle, week, dow, lift, &lifts);
                if let Some(w) = theirs.filter(|w| mine.is_none_or(|m| *w > m)) {
                    let v = variant(me.id, r.id, dow);
                    let yours = match mine {
                        Some(m) => s.f("rival_mine", &[("n", &m.to_string())]),
                        None => s.t("rival_mine_none"),
                    };
                    out.push(Message {
                        id: format!("heavy:{}:{}:{key}:{lift}", me.id, r.id),
                        text: s.f(
                            &format!("rival_heavy_{v}"),
                            &[
                                ("rival", &r.name),
                                ("w", &w.to_string()),
                                ("lift", &data.program.lift_name(lift)),
                                ("mine", &yours),
                            ],
                        ),
                    });
                }
            }
        }
    }
    out.retain(|m| !seen.contains(&m.id));
    out
}
