# Handoff

Stage 9, written by the build agent on 2026-11-23. A fresh session starts from this file, not from memory.

## What this is

A phone web page (opened from a link in Safari, not an app store app) for three friends on the same four-week-cycle jump-training program. An athlete taps their name and then the vertical they want (the coach sets each one), the phone remembers it, and they see today's session and the week, with working weights worked out from their own maxes. The coach has a typed passcode, kept out of sight until "I'm the coach" is tapped, records voice notes against a lift and can move sessions. A cycles screen shows what is not loaded yet. Static files (GitHub Pages) plus a free Supabase project reached only through functions that check the sign-in. The lock is light on purpose (R2.17).

## Rules that hold it up

| Rule | Guarded by |
| --- | --- |
| Every rule is in the core; the page and shim decide nothing | `crates/core/tests/boundary.rs`, `screens.rs` |
| Working weight is the max times the percentage, nearest 5 lb, halves up, integer maths: 305 at 75% is 230 | `maxes.rs` tests; `screens.rs`; browser check |
| A date uses the latest max on or before it | `maxes.rs`, `screens.rs` |
| A voice note belongs to its exact lift and shows on every block of it for every athlete | `notes.rs`, `screens.rs` |
| A moved session shows on its new date marked, and its old date says where it went | `moves.rs`, `screens.rs`, browser check |
| The program, maxes, recordings and passcodes are not in the repository or in `dist/` | `scripts/leak_check.py` (needs `private/`; its self-test needs nothing) |
| The one other host is the one in `web/net/config.js`, matched to every page's policy; nothing is sent before a passcode | `crates/core/tests/net_fence.rs`; `tests/browser/live_mock.py` |
| Only `web/save/store.js` names browser storage | `crates/core/tests/storage_fence.rs` |
| Every string the screens use is in `data/strings.json`, and every string in it is used | `crates/core/tests/screens.rs`, `copy.rs` |
| The database closes every table to the publishable key and checks the sign-in in every function; five wrong goals lock an athlete out; an athlete changes only their own data | `supabase/schema.sql`; `tools/remote_check.py`; `tests/browser/live_mock.py` |

## Shape of the code

| Where | What |
| --- | --- |
| `crates/core/src/` | `dates`, `program`, `schedule`, `moves`, `maxes`, `notes`, `data` (the shared document and requests), `view` (screens), `api` (`start`, `act`, `length`), `ui`, `page` (strings), `glossary` |
| `crates/shim/` | three forwarding functions |
| `web/` | `app.js` draws the core's panels; `audio.js` the recorder; `net/client.js` the only caller of the database; `net/config.js` its address and key; `save/store.js` the only storage |
| `data/` | strings, glossary hints' source is `GLOSSARY.md`, `ui.json`, `audio.json`, `app.json` (manifest) |
| `supabase/schema.sql` | tables, functions, grants |
| `tools/` | `seed.py`, `remote_check.py`, `set_project.py`, `screenshots.py`, the journal tools |
| `tests/` | `fixtures/sample_program.json` (invented), `browser/` (`interactions.py` demo, `live_mock.py` live path), `breaks.json` |
| `private/` | the transcribed program; gitignored; never copy from it |

## Commands

Set `DEVELOPER_DIR=/Library/Developer/CommandLineTools` first if the Xcode license is not accepted (R2.1).

| Command | Prints when it passes |
| --- | --- |
| `./scripts/check.sh` | `BUILDER CHECK PASSED` |
| `cargo test --workspace` | every `test result: ok` |
| `python3 scripts/break_tests.py` | `N of N breaks caught` |
| `python3.13 tools/set_project.py URL KEY` | the address is in the page and its policy |
| `python3.13 tools/seed.py` | the loaded-week count (asks for the coach passcode) |
| `python3.13 tools/remote_check.py [--write]` | `REMOTE CHECK PASSED` (asks for passcodes) |
| `.venv/bin/python tools/screenshots.py DIR` | the screenshot paths |

## What will bite within the hour

- Page JavaScript holds no number but 0 and 1, no `Math`, no parsing. A date, a length and a weight all come from the core; the recorder's timer asks the shim's `length`.
- `web/` cannot hold a non-text file (the page scan reads it as text); the icon is in `assets/`.
- Playwright routes registered later win; `live_mock.py` relies on that.
- Set the clock with `page.clock.install(...)` and reload in the browser checks; Firefox and WebKit apply it to a page loaded after.
- The demo records a voice note in memory only.

## The journal

`../tran4vert-journal/index.html`, its own repository, not published. Newest entry: the ready-for-you entry of 2026-11-23.

## Open for the person

| Row | What the person decides or supplies |
| --- | --- |
| R1.4 | More PDFs: cycle 2 weeks 2 to 4 and later cycles are not loaded |
| R1.5 | Which max three prescriptions use (round 1 questions 2 and 3) |
| R1.6 | Whether the paid program text may be shown to the two friends |
| R1.7 | Compare one lift day with TeamBuildr; the transcription was typed by hand |
| R2.1 | Accept the Xcode license (`sudo xcodebuild -license accept`) or keep the environment variable |
| R2.10 | Check voice-note playback on a phone with the live project |
| R2.13 | Done 2026-11-23: project, schema, passcodes, program loaded. Still yours: confirm the three athletes' start dates (coach screen), friends' maxes are entered by themselves |
| R2.17 | The sign-in is a name and a tapped goal. Anyone with the link can read the names, and can reach an athlete's data by guessing among sixteen goals, five tries per fifteen minutes. The paid program text is behind the same lock (R1.6). Chosen by the person on 2026-11-23 |
| R2.18 | Run `supabase/schema.sql` again in the SQL editor, give each athlete their goal with `update athletes set goal = 40 where name = '...'`, and run `tools/remote_check.py`; the new page is not published until then |
| R2.14 | Read the agent-written strings at the foot of `SECOND-ORDER.md` |
| R2.16 | Which max the cycle 2 squat percentage refers to |

Then: try it on the phone, say whether to create the public repository and publish.

## Mistakes so far

By system, from `SECOND-ORDER.md`: the build environment (R2.1, R2.2), a binary file in `web/` (R2.6), the glossary's ruled-out word in a MIME list (R2.7), the page doing arithmetic (R2.8). Each produced a rule that lives in a test or a note.

## What passed

2026-11-23, on the final build: `./scripts/check.sh` ended `BUILDER CHECK PASSED` (ten steps; browser check and live-path check in chromium, firefox, webkit, webkit-iphone). `./scripts/count-tests.sh`: `120 passed, 0 failed, 0 ignored`. `python3 scripts/break_tests.py`: `48 of 48 breaks caught`. Commit: the one that adds this file's last edit (see `git log`). On 2026-11-23 the person ran `tools/seed.py` (5 of 8 weeks) and `tools/remote_check.py --write` against the live project: `REMOTE CHECK PASSED`.

## The next action

Try the live page on a phone (it needs a web address, so publishing comes first; say the word). Then check 305 at 75% shows 230, record and play a voice note, and move a session.
