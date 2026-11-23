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

## Strings written by the agent

Each reader-visible string the agent wrote, with its key, for the person to read.

| Key | String |
| --- | --- |
