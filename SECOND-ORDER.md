# Second-order notebook

Stage 6, kept by the agent as the work happens, not afterwards. `B3 Build in self-questioning`.

The code and tests are the work; this file is the record about the work. It is where a decision the agent made alone, an assumption that turned out wrong, or something only the person can do becomes visible to the person. With the plan approved in advance, it is the main place those decisions surface before the person uses the tool. [`docs/procedure.md`](https://github.com/sgilson7/builder-setting/blob/main/docs/procedure.md#the-second-order-notebook) says why.

One row per assumption, surprise or open item. **Kind** is *divergence* (what was assumed, what turned out to be true, what changed), *finding*, or *worklist* (the person's to do). Number rows by round (R1.1, R1.2, then R2.1 after the first triage) so a later round does not renumber an earlier one. **Rule** is the rule the row produced for later sessions, and where it now lives (`CLAUDE.md`, a test, `tests/breaks.json`), or blank. **Status** is *done*, *open*, or *person*; every *person* row is named in `HANDOFF.md`. `scripts/notebook_check.py` checks the form of each row and that the notebook is not empty once `PLAN.md` holds a decision, not what the rows say.

| # | Kind | Assumed or found | What is true | What changed | Rule | Status |
| --- | --- | --- | --- | --- | --- | --- |
| R1.1 | divergence | The opening journal entry is written before `PLAN.md` (CLAUDE.md rule 14). | The person asked one model to plan and another to build; the planning session wrote `BRIEF.md` and `PLAN.md` and started the journal, but left the opening entry to the build session. | The build session writes the opening, brief and plan entries first (PLAN decision 2). | | open |
| R1.2 | finding | The description names the three athletes. | The repository must be public (free GitHub plan). | Names are replaced with [me], [friend 1], [friend 2] in `BRIEF.md`; first names live only in the database. | No athlete's name in a tracked file; `scripts/leak_check.py` (PLAN decision 20) | open |
| R1.3 | finding | `python3` runs the tools. | `python3` here is 3.7.6; `python3.13` is installed. No cargo, wasm-bindgen or Playwright. | Journal started with `python3.13`; PLAN decision 1 installs the rest after asking. | | open |
| R1.4 | worklist | The five PDFs are the whole program. | Cycle 2 weeks 2 to 4 and all later cycles are missing; see `not_captured` in the private program file. | The app marks them not loaded. The person supplies more PDFs. | | person |
| R1.5 | worklist | Percentages name their max. | Three prescriptions do not (front squat 75%; single-leg 70% and 65% in Cycle 1 week 3). | Assumptions recorded in the program file; PLAN questions 2 and 3. | | person |
| R1.6 | worklist | The program text may be shown to the two friends. | It is paid coaching material from a third party. | Kept out of the repository and behind a passcode; sharing it is the person's decision (BRIEF). | | person |
| R1.7 | finding | Transcription was done by reading the PDFs, not by a parser. | 29 sessions were typed by hand by the planning agent; ids were validated, numbers were not checked by a second reader. | The person should compare one lift day against TeamBuildr at stage 7. | | person |
| R2.1 | finding | Cargo links with the system compiler. | The Xcode license is not accepted on this machine, so `cc` exits with status 69 and no Rust build links (found installing `wasm-bindgen-cli`). | Every cargo and `scripts/*.sh` command in this build runs with `DEVELOPER_DIR=/Library/Developer/CommandLineTools`. The person can accept the license (`sudo xcodebuild -license accept`) or keep exporting the variable. | | person |
| R2.2 | divergence | `scripts/setup.sh` makes its virtual environment with `python3`. | `python3` here is 3.7.6, which Playwright 1.63.0 does not support. | One line of `scripts/setup.sh` now calls `python3.13`. `check.sh` runs the environment's own Python, so nothing else changed. | | done |
| R2.3 | finding | The copy passes `./scripts/check.sh` before any edit (PLAN decision 1). | It passes steps 1 to 6 and the browser check; steps 7 and 8 fail on the planned records (the `person` rows are not yet named in `HANDOFF.md` or the journal; the opening entry was blank). | Baseline recorded in `MEASUREMENTS.md`; the opening entry and later the handoff close those two steps. | | done |
| R2.4 | divergence | Take `kits/saves` whole (PLAN decision 4 says "one store file"). | The kit's `save.rs`, `playtest.rs` and `old_saves.rs` version a game's progress. What the phone keeps here is two passcodes, a chosen athlete and a copy of the last state, all of which the database can give again. | Only `store.js` (own version, plain key and value) and `storage_fence.rs` are taken. A kept choice the core cannot read is ignored and the person is asked again; that has a test. | | done |
| R2.5 | divergence | The template's placeholder rule and its two `breaks.json` rows stay. | `page.rs` held only a title and an intro check. The screens replaced it, and `strings.json` became a map of keys. | The two template rows were replaced by 42 rows for the new rules; `tests/breaks.json` is the list. | | done |
| R2.6 | finding | A home-screen icon can sit in `web/`. | `boundary.rs` reads every file under `web/` as text, so a PNG there fails it. | The icon is in `assets/` and `scripts/build.sh` copies it to `dist/`. No test was changed. | | done |
| R2.7 | finding | `data/audio.json` can list MIME types. | `copy.rs` rules out the word "audio" for "voice note", and it matched `audio/mp4`. | One `ALLOWED` entry in `copy.rs` with its reason, as that file provides. The glossary is unchanged. | | done |
| R2.8 | finding | The page can turn a clock reading into `YYYY-MM-DD`. | Page JavaScript holds no number but 0 and 1, so no slicing or arithmetic. | `toLocaleDateString("en-CA")` gives the date; every sum and every length (the recorder's timer too, through the shim's `length`) is in the core. | page scan in `boundary.rs` | done |
| R2.9 | divergence | The core says which screen follows which action with the template's `screen.show`. | The core now also returns `requests`, the calls the page must make to the database, and takes `data.load`, `data.apply` and `net.*` actions. | The page makes no request of its own, and in demo mode `data.apply` changes the core's copy. | `crates/core/tests/screens.rs` | done |
| R2.10 | worklist | Voice notes can be played in the demo. | A voice note recorded in the demo is kept in memory by the page, and the demo plays the last one for any note. | Only the live build stores real audio. The person checks playback on a phone with the live project. | | person |
| R2.11 | finding | The week list is easy to tap. | Each day row has a separate Open button, so the row itself is not a target (seen in the screenshots). | Left as built; a row-sized tap is a later change for the person to ask for. | | open |
| R2.12 | finding | `supabase/schema.sql` can be tested here. | No Postgres was installed here, so the SQL was untested until the person ran it on 2026-11-23: it ran, `tools/seed.py` loaded 5 of 8 weeks, and `tools/remote_check.py --write` passed every check (tables closed, bad passcode refused, coach-only enforced, 2 MB cap). | `tools/remote_check.py` is the test of the schema and its functions; the live-path browser test (`tests/browser/live_mock.py`) runs the page against a pretend database that follows the same function names and arguments. | `tools/remote_check.py` | done |
| R2.13 | worklist | The project address and key, and the passcodes, exist. | The project and its publishable key were given on 2026-11-23 and are set in the page. The passcodes do not exist yet: the person creates the Supabase project, runs the schema, types both passcodes into the SQL editor and adds the three athletes there. | Stopped here and asked for the address and the publishable key only. `tools/set_project.py` writes them into the page; `tools/seed.py` and `tools/remote_check.py` prompt for passcodes with getpass. | | person |
| R2.14 | worklist | A list the agent made is not shown to a reader. | The made-up demo names (Sam, Jo, Pat), the weekday and month names, the session kind labels and every other string are agent-written and are listed at the foot of this file. | None changed. The person reads them. | `TONE.md`, `copy.rs` | person |
| R2.15 | divergence | The week screen picks the current week from the athlete's own start date (PLAN decision 7). | Before the start it shows the first week with "Not started" days rather than a blank, and earlier weeks can be paged to. | Tested in `screens.rs`. | `the_week_screen_runs_monday_to_sunday...` | done |
| R2.16 | finding | The Cycle 2 front-squat percentage names a max (R1.5). | The core uses the lift named by the block's `of` field and has no rule of its own; the sample program's equivalent line names a different lift to show that. | Question 3 (round 1) stays with the person; changing the answer is a program-file edit, not code. | | person |
| R2.17 | divergence | Entry is a typed group passcode (BRIEF done-means 1 and 2; DECISIONS: "Group passcode"). | The person asked for tapping instead of typing: each athlete taps their name, then a vertical (40 inches for one, 50 for another). The agent said that a few choices can be guessed and that names must be fetched before anyone signs in; the person chose "Replace the passcode". | Sixteen goals; the coach sets each athlete's; five wrong tries lock that athlete out for fifteen minutes; each wrong try waits half a second; an athlete changes only their own maxes and moves; goals are sent only to the coach; names and ids are public. The page now makes one call on load, for the names, so done-means 1 ("no request until a passcode is submitted") no longer holds as written. The coach keeps a typed passcode behind a button. | `schema.sql`, `screens.rs`, `live_mock.py` | person |
| R2.18 | worklist | The live database matches the code. | It has the old functions and no goals. | The person re-runs `supabase/schema.sql` (it keeps data), sets each athlete's goal, and runs `tools/remote_check.py`. Publishing waits for that: the new page needs `get_names`. | | person |
| R2.19 | finding | A photo the person supplied can be an athlete's profile picture. | It is a press photograph of a named real person, and the repository and the site are public. The agent cannot check its rights and did not copy it anywhere. | Built a picture upload instead: the coach (or an athlete, for themselves) chooses a file, up to 300 KB, and it is stored in the person's own database, not in git. The person uploads it, and the rights are theirs to judge. | `screens.rs` (size and who may change); `schema.sql` | person |
| R2.20 | divergence | Names are public (R2.17). | Pictures are public too: the first screen shows them to anyone with the link, so `get_avatar` takes no sign-in. | Said to the person in the same message; `schema.sql` comments say it. | `live_mock.py` (a picture is fetched without a sign-in) | done |
| R2.21 | worklist | The database has the picture columns. | Not until the person runs `supabase/schema.sql` again (it keeps the data). Until then the page works and shows initials, and an upload fails. | | | person |
| R2.22 | worklist | Each lift has a category (power, strength, calf, accessory); a held block is isometric. | The program file did not say. The agent assigned each lift in `private/program/program.json` by its name: cleans, depth landing and the jump session as power; squats, Romanian deadlifts and the leg press as strength; the calf raises as calf; the Nordic and the cable abduction as accessory. The calf isometric lift is calf in the file and shows as isometric because its blocks are holds. | The page colours each category and always says it in words. The person corrects any lift in the file and runs `tools/seed.py` again. | `program.rs`, `screens.rs` | person |
| R2.23 | worklist | The new categories reach the live page. | They are in the local program file only until the person runs `python3.13 tools/seed.py` again. | | | person |

## Strings written by the agent

Each reader-visible string the agent wrote, with its key, for the person to read.

| Key | String |
| --- | --- |
| `title_passcode` | Welcome |
| `title_who` | Who are you? |
| `title_maxes` | Maxes |
| `title_cycles` | Cycles |
| `title_coach` | Coach |
| `passcode_coach` | Coach passcode |
| `passcode_enter` | Enter |
| `who_heading` | Tap your name |
| `who_text` | The coach set up one name for each athlete. |
| `who_current` | You |
| `demo_coach_text` | This is a demo with made-up data. Open it as the coach to see the coach tools. |
| `demo_coach_button` | Open as coach |
| `demo_athlete_1` | Sam |
| `demo_athlete_2` | Jo |
| `demo_athlete_3` | Pat |
| `nav_today` | Today |
| `nav_week` | Week |
| `nav_maxes` | Maxes |
| `nav_cycles` | Cycles |
| `nav_coach` | Coach |
| `today_title` | Today · {date} |
| `fmt_short_date` | {dow} {day} {month} |
| `fmt_long_date` | {dow} {day} {month} {year} |
| `kind_jump` | Jump |
| `kind_lift` | Lift |
| `kind_general` | General |
| `kind_iso` | Isometric |
| `kind_rest` | Rest |
| `session_where` | {cycle} · week {week} |
| `moved_from` | Moved from {date} |
| `moved_to` | Moved to {date} |
| `moved_go` | Open the new date |
| `move_button` | Move this session |
| `move_undo` | Put it back |
| `move_heading` | Move this session |
| `move_text` | Pick the new date. The session shows there, marked as moved. |
| `move_to` | New date |
| `move_save` | Move |
| `move_save_all` | Move for all athletes |
| `cancel` | Cancel |
| `back_today` | Back to today |
| `empty_before` | The program starts on {date}. |
| `empty_not_loaded` | Cycle {cycle}, week {week} is not loaded yet. |
| `empty_rest` | No session on this day. |
| `tag_coach_note` | Coach's note |
| `tag_working_weight` | Working weight |
| `tag_test` | Test |
| `tag_voice_note` | Voice note |
| `tag_not_loaded` | Not loaded |
| `tag_loaded` | Loaded |
| `tag_moved` | Moved |
| `tag_today` | Today |
| `tag_not_started` | Not started |
| `rx_sets_reps` | {sets} × {reps} |
| `rx_each_side` | {sets} × {reps} each side |
| `rx_hold` | {sets} × {secs} seconds |
| `rx_rounds` | {rounds} rounds |
| `rx_amap` | As many as you can |
| `rx_pct` | last {last} at {pct}% |
| `lift_open` | Open lift |
| `ww_line` | {weight} lb · last {last} |
| `ww_basis` | {pct}% of your {lift} max, {max} lb |
| `ww_none` | Last sets at {pct}% of your {lift} max |
| `max_add_button` | Add a max |
| `test_line` | Test: {test} |
| `test_enter` | Enter the result |
| `note_detail` | {length} · {date} |
| `note_play` | Play {title} |
| `note_delete` | Delete note |
| `lift_max_heading` | Your max |
| `lift_max_from` | This lift uses your {lift} max. |
| `max_none` | You have no max for this lift yet. |
| `max_none_short` | No max yet |
| `max_line` | {weight} lb on {date} |
| `lift_notes_heading` | Voice notes |
| `lift_notes_none` | No voice note on this lift yet. |
| `week_title` | {place} · {from} to {to} |
| `week_place` | {cycle} · week {week} |
| `week_before` | Before the start |
| `week_earlier` | Earlier week |
| `week_this` | This week |
| `week_later` | Later week |
| `day_open` | Open |
| `cycle_n` | Cycle {n} |
| `week_n` | Week {week} |
| `max_form_heading` | Add a max |
| `max_form_text` | A new max changes the working weights from its date on. |
| `max_lift` | Lift |
| `max_athlete` | Athlete |
| `max_weight` | Weight (lb) |
| `max_date` | Date |
| `max_save` | Save max |
| `max_delete` | Delete |
| `cycle_emphasis` | Emphasis: {emphasis} |
| `theme_none` | Theme not captured |
| `cycle_here` | You are here |
| `not_captured_heading` | Not captured yet |
| `not_captured_none` | The program file has no gaps listed. |
| `coach_program_heading` | Program |
| `coach_program_text` | {loaded} of {all} weeks are loaded. |
| `coach_starts_heading` | Start dates |
| `coach_starts_text` | Each athlete's first week starts on a Monday. |
| `coach_starts_save` | Save start dates |
| `coach_notes_heading` | Voice notes |
| `coach_notes_none` | No voice notes yet. |
| `coach_cycle_heading` | Cycle notes |
| `coach_cycle_text` | A note here replaces the program's note for that cycle. Leave it empty to use the program's. |
| `coach_cycle_save` | Save cycle notes |
| `rec_heading` | Record a voice note |
| `rec_text` | Pick the lift, give the note a title, then record. It plays on each session that has that lift. |
| `rec_lift` | Lift |
| `rec_title` | Title |
| `rec_limit` | Up to {seconds} seconds. Titles up to {chars} characters. |
| `rec_start` | Record |
| `rec_stop` | Stop |
| `rec_listen` | Listen |
| `rec_save` | Save note |
| `rec_discard` | Discard |
| `offline_note` | Offline, as of {date}. Changes are off. |
| `err_date` | {date} is not a date. |
| `err_max` | The max was not saved. Check the lift, the weight and the date. |
| `err_move` | That session cannot move there. Pick a different date. |
| `err_no_athlete` | That athlete is not here. |
| `err_no_data` | The program has not loaded yet. |
| `err_no_lift` | That lift is not in the program. |
| `err_no_max` | That max is not here. |
| `err_no_move` | That session has not moved. |
| `err_no_note` | That voice note is not here. |
| `err_no_screen` | There is no screen {screen}. |
| `err_not_coach` | That needs the coach passcode. |
| `err_not_yours` | That max belongs to another athlete. |
| `err_note` | The voice note was not saved. Check the title and the length. |
| `err_offline` | This phone is offline. Changes wait until it is back online. |
| `err_passcode_empty` | Type a passcode first. |
| `err_start` | The start date for {name} must be a Monday. |
| `err_unreachable` | The database did not answer. Check the signal and try again. |
| `err_bad_passcode` | That is not it. Try again. |
| `err_weight` | Type the weight as a whole number of pounds. |
| `nothing_changed` | Nothing changed. |
| `page_loading` | Loading… |
| `page_checking` | Checking the passcode… |
| `page_start_failed` | The page could not start. |
| `page_demo` | Demo with made-up data |
| `page_rec_denied` | The microphone is blocked. Allow it in the browser's settings for this site, then try again. |
| `page_rec_unsupported` | This browser cannot record. |
| `page_rec_recording` | Recording |
| `page_rec_ready` | Ready |
| `page_rec_saved` | Saved |
| `page_playing` | Playing |
| `page_play_failed` | The voice note could not be played. |
| `page_switch` | Your name and this phone |
| `page_home_screen` | To fill the screen, choose Share, then Add to Home Screen. |
| `page_note_missing` | Fill in the lift and the title first. |
| `page_save_failed` | The voice note could not be saved. |
| `page_nav` | Screens |
| `dow_0` | Mon |
| `dow_long_0` | Monday |
| `dow_1` | Tue |
| `dow_long_1` | Tuesday |
| `dow_2` | Wed |
| `dow_long_2` | Wednesday |
| `dow_3` | Thu |
| `dow_long_3` | Thursday |
| `dow_4` | Fri |
| `dow_long_4` | Friday |
| `dow_5` | Sat |
| `dow_long_5` | Saturday |
| `dow_6` | Sun |
| `dow_long_6` | Sunday |
| `month_1` | Jan |
| `month_2` | Feb |
| `month_3` | Mar |
| `month_4` | Apr |
| `month_5` | May |
| `month_6` | Jun |
| `month_7` | Jul |
| `month_8` | Aug |
| `month_9` | Sep |
| `month_10` | Oct |
| `month_11` | Nov |
| `month_12` | Dec |
| `err_coach_only` | That needs the coach passcode. |
| `err_server` | The database refused the request. Try again. |
| `passcode_coach_button` | I'm the coach |
| `passcode_group_button` | Back to the names |
| `passcode_coach_heading` | Coach passcode |
| `passcode_coach_text` | Type the coach passcode. It also opens everything an athlete sees. |
| `phone_heading` | This phone |
| `phone_text` | This phone remembers the passcode and your name, so you type them once. |
| `phone_forget` | Forget this phone |
| `phone_forget_ask` | The passcode and your name are forgotten on this phone, and you start again from the passcode screen. Nothing is deleted from the database. |
| `phone_forget_yes` | Yes, forget |
| `phone_keep` | Keep |
| `max_up` | Up {n} lb from {date} |
| `max_down` | Down {n} lb from {date} |
| `max_same` | Same as {date} |
| `names_heading` | Tap your name |
| `names_text` | Then tap the vertical you want. |
| `names_loading` | Loading the names… |
| `goal_heading` | {name}, what vertical do you want? |
| `goal_text` | Tap your goal, in inches. The coach set it for you. |
| `goal_option` | {n} inches |
| `goal_back` | Not me |
| `coach_goals_heading` | Vertical goals |
| `coach_goals_text` | Each athlete taps their goal to sign in. Set one for each athlete, then tell them which to tap. |
| `goal_not_set` | Not set |
| `goals_save` | Save goals |
| `err_goal` | That goal is not one of the choices. |
| `err_not_you` | That name is not yours. |
| `err_too_many` | Too many wrong tries. Wait a few minutes and try again. |
| `page_names_failed` | The names could not be loaded. Check the signal and reload. |
| `who_you` | You are {name}. |
| `hero_title` | Reach {n} inches. |
| `hero_sub` | Each session is a chance to add an inch. Tap your name to start. |
| `hero_before` | Before Tran4Vert |
| `hero_after` | With Tran4Vert |
| `hero_inches` | {n} inches |
| `hero_line_1` | Small jumps, stacked week after week. |
| `hero_line_2` | Beat last week's number. |
| `hero_line_3` | Show up. Jump. Repeat. |
| `avatar_heading` | A picture |
| `avatar_text` | Choose a picture from this phone, up to {kb} KB. It shows on the first screen to anyone with the link. |
| `avatar_athlete` | Whose picture |
| `err_avatar` | That picture is empty or over {kb} KB. Pick a smaller one. |
| `page_avatar_choose` | Choose a picture |
| `cat_power` | Power |
| `cat_strength` | Strength |
| `cat_calf` | Calf |
| `cat_accessory` | Accessory |
| `cat_isometric` | Isometric |
| `legend_heading` | Colour key |
