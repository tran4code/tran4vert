# Plan

Stage 2, written by the planning session from `BRIEF.md` on 2026-11-23, for a second agent to build from. Each decision is in build order, with its reason and the test that shows it holds.

## Shape

```text
 iPhone Safari                         GitHub Pages (static, public)        Supabase (private, passcode)
 ┌───────────────┐   loads once        ┌──────────────────────────┐         ┌──────────────────────────────┐
 │ web/ page     │ ◄────────────────── │ html, css, js, core.wasm │         │ program (jsonb)              │
 │  draws ui()   │                     │ no program, no names     │         │ athletes, maxes, moves       │
 │  web/net/*.js │ ── after passcode ─────────────────────────────────────► │ voice_notes (audio bytes)    │
 │  web/save/    │    fetch → rpc                                           │ reached only through RPC     │
 └───────┬───────┘                                                          │ functions that check a hash  │
         │ strings in, JSON out                                             └──────────────────────────────┘
 ┌───────▼───────┐
 │ Rust core     │  every rule: dates, weeks, working weights, moves, which note goes with which lift
 └───────────────┘
```

The core never fetches. The page fetches one JSON document (the "state": program, athletes, maxes, moves, note list) and hands it to the core as a string with today's date; the core returns the screen. A change (new max, move, note) is an RPC call, then a refetch.

## Decisions

| # | Decision | Reason | Test that shows it holds |
| --- | --- | --- | --- |
| 1 | **Toolchain first.** Install rustup with the pinned toolchain, the `wasm32-unknown-unknown` target, `wasm-bindgen-cli =0.2.127`, and run `scripts/setup.sh` with `python3.13` (the default `python3` on this machine is 3.7). Ask the person once before installing. Record the baseline in `MEASUREMENTS.md`. | Nothing is installed (measured 2026-11-23: no cargo, no wasm-bindgen, no Playwright). | `./scripts/check.sh` passes on the untouched template. |
| 2 | **Journal before code.** Write the journal's opening entry (*reading*), a *brief* entry and a *plan* entry from `BRIEF.md` and this file. | CLAUDE.md rule 14. The plan was written before the opening entry (R1.1), so the entry is honest about the order. | `scripts/journal_check.py`. |
| 3 | **Glossary.** Add the terms in the brief's Words section. `Not:` lines: max (not PR, personal best), session (not workout, practice), working weight (not load, target), voice note (not memo, audio). | One word per thing on a small screen; D15. | `copy.rs`. |
| 4 | **Kits.** Copy `kits/saves` (one store file) and apply `kits/mobile` (touch baseline, manifest, `webkit-iphone` in the check; skip the tablet pass). Take only the fence-test pattern from `kits/online` for `web/net/`; do not copy its rollback session. Paste each kit's row into the change table in `BUILDER-SPEC.md`. | The phone must remember passcode and name (D17 change); phone is the main device; Supabase is a declared off-origin feature (R7). | The kits' own tests, each with a `tests/breaks.json` row. |
| 5 | **Program model in the core** (`program.rs`). Parse the format of `private/program/program.json`: `lifts`, `library`, `notes`, `cycles[].weeks[].days[].blocks[]` with block kinds `ref`, `note`, `lift`, `text`, and `missing` weeks. `@name` notes resolve to `notes` or the cycle note. Unknown fields are kept out, unknown ids are an `{"error"}`. | The file is the single source; the person will add weeks by giving PDFs to an agent. | Unit tests on `tests/fixtures/sample_program.json`, a made-up two-week program with invented lift names (no real program text in the repo). A local-only test parses the private file when it exists and is skipped otherwise. |
| 6 | **Dates in the core** (`dates.rs`), integers only: ISO date string to day number and back, weekday, add days. The page passes today's date as a string. | R2: no clock in the core. | Hand-worked cases: 2024-07-01 is a Monday; 2024-02-28 + 2 = 2024-03-01; 2026-12-31 + 1 = 2027-01-01. |
| 7 | **Schedule rule** (`schedule.rs`). Each athlete has a start date (a Monday) for Cycle 1 week 1. Cycles follow each other, 4 weeks each. Program day (cycle, week, dow) maps to start + 28·c + 7·w + dow. Before the start date: "starts on …". After the last loaded week: "not loaded yet". Sunday: rest. | The PDFs are from July 2024 and the friends start later; athletes may be at different points. | Start 2026-11-30: c1 w1 Tue is 2026-12-01; c2 w1 Mon is 2026-12-28; a date in c2 w2 answers not-loaded. |
| 8 | **Moved sessions** (`moves.rs`). A move is (athlete, cycle, week, dow) → new date. The session shows on the new date marked "moved from Tue"; its old date shows "moved to Thu". Two sessions may share a date (both are listed). A move can be undone. An athlete moves their own; the coach moves anyone's, or one session for all three. | "move workouts around if people request it". | Move then week view shows both markers; undo restores; two-on-one-day lists both in program order. |
| 9 | **Maxes and working weights** (`maxes.rs`). A max is (athlete, lift, weight lb, date). Current max for a date is the latest entry on or before it. Working weight = max × pct ÷ 100, rounded to the nearest 5 lb, halves up, integer maths. A lift with `max_from` uses that lift's max. No max: show the percentage and "add a max". `test` blocks show an "enter result" action that pre-fills the max form. | Done-means 4 and 5; D2 bans floats. | 305 at 75% is 228.75 → 230; 195 at 110% → 215; 195 at 60% → 115; 215 at 65% → 140; a max dated after the session date is not used. |
| 10 | **Voice notes belong to a lift** (`notes.rs`). A note is (id, lift id, title, seconds, date). A lift block lists every note whose lift id equals the block's lift, newest first. Notes on `power_clean` are not shown on `clean_pull`: association is by exact lift, as the person described. | "for everything that squats are assigned, that voice recording is associated to that particular lift". | A note on `back_squat` appears on every back-squat block in every week and for every athlete, and on no front-squat block. |
| 11 | **View model** (`page.rs`, `api.rs`). `ui(state, today, session)` returns one of: `passcode`, `who`, `today`, `week`, `lift`, `maxes`, `cycles`, `coach`. `act(name, value)` changes screen, athlete, week offset. The core also returns the pending remote request for an action (`{"rpc": "add_max", "args": {...}}`) so the page builds no request of its own. | R4, R6: the page draws and sends; it decides nothing. | `api.rs` tests per screen; `page_keeps_no_rule…` boundary test. |
| 12 | **Cycles screen.** One card per cycle: name, emphasis, four week themes, which weeks are loaded, and the athlete's place. Below, the `not_captured` list word for word, and a coach note per cycle that the coach can edit. | "see the cycles at a high level but do notes of what I haven't captured". | Fixture with a missing week shows "not loaded"; the list length equals the fixture's. |
| 13 | **Supabase schema** (`supabase/schema.sql`, committed; holds no secret). Tables `program` (one jsonb row), `athletes` (first name, start date), `maxes`, `moves`, `voice_notes` (metadata + `bytea` audio), `cycle_notes`, `secrets` (two password hashes via `pgcrypto`). Row-level security on with no policies, so the public key can read and write nothing directly. | The public key ships in a public page, so tables must be closed to it. | `tools/remote_check.py`: a direct `select` on each table with the public key returns nothing. |
| 14 | **RPC functions** (`security definer`), each taking the passcode: `get_state(code)`, `add_max`, `delete_max`, `move_session`, `undo_move`, `get_note_audio(code, id)`; and with the coach passcode: `add_note`, `delete_note`, `set_start_date`, `set_program`, `set_cycle_note`. Wrong passcode: one fixed error, after a short `pg_sleep`. Audio is capped at 2 MB. | One small surface to test; no SDK to vendor, plain `fetch`. | `tools/remote_check.py` against the live project: wrong code refused on every function; group code refused on every coach function. |
| 15 | **Off-origin fence.** `web/net/config.js` holds the project URL and the public key and is the only file naming another origin; `web/net/client.js` is the only file calling `fetch` off-origin. CSP `connect-src 'self' <that origin>`, `media-src blob:`. No request leaves before the passcode form is submitted. | R7's declared off-origin feature; done-means 1. | Fence test: config origin equals CSP origin, no other file names one. Browser check: 10 s idle on first load, zero off-origin requests. |
| 16 | **What the phone keeps** (`web/save/store.js` only): the group passcode, the coach passcode if entered, the chosen athlete, and the last state fetched. With no signal the page draws the kept state, marked "offline, as of …", and disables changes. | Gyms have poor signal; friends should type the passcode once. | `storage_fence.rs`; browser interaction: load, enter, reload offline, session still drawn. |
| 17 | **Recording** (`web/audio.js`). Coach screen and lift screen: record with `MediaRecorder` (iOS Safari gives `audio/mp4`), a visible timer, stop, listen, title, save or discard. Limit and bitrate in `data/audio.json`. Playback fetches the bytes on tap and plays a blob URL. Microphone is asked for only on tapping Record. | iPhones; D3 keeps numbers out of page code. | Browser interaction with a fake media stream in Chromium: record, save, note appears on the lift. On a real iPhone: the person (stage 7). |
| 18 | **Demo mode.** With no `web/net/config.js` values, the page loads `tests/fixtures/sample_program.json` and keeps changes in memory. | The browser check and the first screenshots must not need the live database or the real program. | The whole browser check runs in demo mode. |
| 19 | **Seed tool** (`tools/seed.py`). Reads `private/program/program.json` and calls `set_program`; reads the coach passcode from a prompt (`getpass`), never from an argument or a file. Prints a per-week summary before sending. | The program reaches the database without touching the repository. | Run against the live project by the person; `get_state` then returns five loaded weeks. |
| 20 | **Leak check** (`scripts/leak_check.py`, a step of `check.sh`). Fails if `private/` is tracked, or if any tracked file or `dist/` file contains a line from the private program's notes, a number from `maxes_from_pdf` next to a lift id, or a line of `.env`. Skips cleanly when `private/` is absent (CI). | Done-means 9. | Planted defect: copy one coach note into `data/strings.json`, the check fails. |
| 21 | **Screens on a phone.** Bottom bar: Today, Week, Maxes, Cycles. Today: date, cycle and week, session kind label, blocks in order; circuits collapsed to a title and dose, opening to their items; lift blocks as sets × reps with working weight, the coach's note word for word, and a play button per voice note. Tapping the name at the top switches athlete. | The person's description, in order of use. | Screenshots at 390 × 844 in the journal; `webkit-iphone` pass. |
| 22 | **Stop before publishing.** After `check.sh` passes in demo mode and against the live project: write `HANDOFF.md` and a *ready for you* journal entry, and stop. Creating the public repository `tran4vert` and running `publish.sh` wait for the person's word. | Brief: Deploys `on my word`. | n/a |

## Build order

1. Decisions 1 to 4 (setup, journal, glossary, kits). Commit.
2. Decisions 5 to 12 with the fixture (the whole core). Commit per rule with its test.
3. Decisions 18 and 21 (page in demo mode), browser check. **First thing the person can look at.**
4. Decisions 13 to 16 and 19, once the person has made the Supabase project (questions 1 and 2).
5. Decision 17 (recording), decision 20 (leak check).
6. Decision 22.

## What the person does, and when

| When | What |
| --- | --- |
| Before step 1 | Say yes to installing Rust and Playwright. |
| Before step 4 | Create a free project at supabase.com. Paste `supabase/schema.sql` into its SQL editor and run it. Run the two `set passcode` lines there with passcodes of your own. Give the agent the project URL and the *publishable* (anon) key only. |
| After step 4 | Run `python3.13 tools/seed.py` and type the coach passcode. Set each athlete's start date on the coach screen. |
| After step 6 | Try it on your phone: check the worked case (305 at 75% shows 230), record a note, move a session. Then say whether to publish. |
| Each new week of PDFs | Give the PDF to an agent in this folder: it adds the week to `private/program/program.json`, you run the seed tool. |

## Not in this plan

- Logging what was actually lifted per set, pain scores, and a done tick per session. The PDFs have Result columns and Joint Health scores; they can be a later round.
- Jump-height tracking (the point of the program). Likely the next thing worth adding to Maxes.
- Reading PDFs in the page. Transcription stays with an agent and the person's check.
- Individual accounts. One group passcode and one coach passcode is a light lock for three friends, not security against a determined person.
- Notifications and reminders.

## Open questions for the person

Answerable by number; each has a default the build uses until answered.

1. **Start dates.** When does each athlete start Cycle 1 week 1? Default: all three on Monday 2026-11-30, changeable on the coach screen.
2. **Cycle 1 week 3.** It has walks, single-leg work and "choice" push/pull and no isometrics. Standard program, or a one-off for you that the friends should not copy? Default: show it as transcribed.
3. **Front squat 75%** in Cycle 2 week 1: of a front squat max (default), or a share of the back squat max?
4. **Clean variants.** Should a voice note on Power Clean also show on Hang Power Clean and Clean Pull? Default: no, exact lift only.
5. **Who may move sessions.** Default: an athlete moves their own, the coach moves anyone's.
6. **Supabase pauses a free project after about a week with no requests.** Three people training weekly keeps it awake. If it pauses over a break, you un-pause it from the dashboard. Acceptable? Default: yes.
7. **Friends' maxes.** Only yours were on the PDFs (and they are from July 2024). Default: everyone enters their own on first use; yours are not pre-filled.

## Status

- [x] Read by the person on: 2026-11-23
