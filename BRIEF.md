# Brief

Stage 1. The person's own words come first; the rest was written by the planning agent from them, for the person to read.

## The description, word for word

Names of the three athletes are replaced with [me], [friend 1] and [friend 2] because this repository is public (R1.2). Nothing else is changed.

> hey, i want to use fable to create a plan and then create the code in a lower model of your suggestion.
> I want to git clone this repository. I want to create a webapplication hosted on my github where i have two friends who want to increase there vertical. The training I am doing is on the teambuildr app and I can get the pdf for the workwith the notes you can copy over. The way the training works, there is a four week cycle where each 4 weeks has a different emphasis. this based on the thp strength training. I basically want my friends to be able to click on the web applicaiton link, click on their name since there is only 2 other friends, and then be able to see their workout and the workouts for the week. I want one feature where I can vooice record myself for a particular workout, lets say squats and then for everything that squats are assigned, that voice recording is associated to that particular lift. mainly, we want to keep track of peoples maxes, right now it is me ([me]), [friend 1], and [friend 2]. this web applicaiton will be access through a web link on the phone, i think we all have iphones. I do want the feature to move workouts around if people request it. Right now, we are getting the workouts from the pdfs and every four week is a cycle. It would be nice to see the cycles at a high level but do notes of what I haven't captured. This dated back to the first week of july but you can gather that from context.

Two answers given in the planning session (2026-11-23), recorded in `DECISIONS.md`:

- Shared data: "Yes, Supabase (Recommended)". A free Supabase project, which the person creates, is the shared database.
- Visibility: "Group passcode (Recommended)". Program text, maxes and recordings stay out of the public repository and are readable only with a shared passcode.

## The tool

A phone web page for three athletes following the same jump-training program. An athlete opens the link, types the group passcode once, taps their name, and sees today's session and the rest of the week, with working weights worked out from their own maxes. Each athlete records their maxes. The coach (the person, who is also one of the three athletes) records voice notes against a lift, and a note is offered wherever that lift is assigned. Sessions can be moved to another day. A cycles screen shows each four-week cycle, its emphasis, and what has not been captured yet.

Device: iPhone Safari first. It must also work in a desktop browser for the coach.

The program comes from TeamBuildr PDFs the person supplies. Five weeks are transcribed in `private/program/program.json` (Cycle 1 weeks 1 to 4, Cycle 2 week 1; originally dated 1 July to 3 August 2024). That file is not committed.

## Done means

Each is checked by the person on their own phone unless a test is named in `PLAN.md`.

1. Opening the link on an iPhone shows a passcode screen and makes no request to another origin until a passcode is submitted.
2. With the right passcode, three names appear. Tapping one shows that athlete's session for today; the phone remembers the passcode and the name on the next visit.
3. The week screen shows Monday to Sunday for the athlete's current program week, and earlier and later weeks can be reached.
4. A lift prescribed as a percentage shows a weight from that athlete's own max, rounded to the nearest 5 lb, beside the coach's original note. Worked case: back squat max 305, "last 2 @ 75%" shows 230.
5. An athlete can add a max for a lift with a date; the maxes screen shows the current max and the history for all three athletes; a new max changes the weights shown from that date on.
6. The coach can record a voice note on a lift from the phone. Every session, for every athlete, that assigns that lift offers the note for playback. The coach can delete a note.
7. A session can be moved to another date. The athlete's week then shows it on the new date, and shows that it was moved.
8. The cycles screen shows each cycle's name, emphasis and four weeks, marks weeks with no data as not loaded, and lists the `not_captured` notes from the program file.
9. Nothing from the program text, no max, no recording and no passcode is in the public repository or in the published static files.
10. `./scripts/check.sh` passes.

## Decisions that stay with the person

- The program content and whether it may be shared with the two friends. It is paid coaching material; the passcode keeps it off the public web, and the decision to share it is the person's.
- The Supabase account, project and both passcodes (group and coach). The person creates them and types the passcodes; the agent never chooses, sees in chat, or commits a passcode.
- Each athlete's start date for Cycle 1.
- Whether Cycle 1 week 3 as transcribed is the standard program or a one-off (see `not_captured` in the program file).
- Which max the Cycle 2 front-squat percentage refers to.
- Recordings are the person's own voice; friends' data is first name and lifting numbers only.
- Creating the public repository and publishing.

## Accessibility

Session kind (jump, lift, general, isometric, rest) is carried by a text label, not by colour alone. Touch targets are at least the size in `data/` (44 CSS pixels). Text is readable at arm's length in a gym: no information smaller than body size. Works with touch and with a keyboard and mouse. Audio notes show their length and a text title, since a note cannot be heard in every gym.

## Style

`default`

## References

none

## Words

Defined in `GLOSSARY.md` before any string uses them: cycle, week, session, lift, max, working weight, test, voice note, athlete, coach, isometric hold, moved session.

## Autonomy

The plan in `PLAN.md` was written in advance by the planning session and the person asked for a different model to build from it; the person reads `PLAN.md` before the build starts. Where the brief and plan are silent, the agent decides, records the decision in `SECOND-ORDER.md`, and continues. The agent may not loosen a Builder requirement or a boundary test. The one declared off-origin feature is the Supabase project named in `web/net/config.js`; the agent may not add any other server, account, analytics or origin. The agent may install the pinned Rust toolchain and Playwright after asking once. The agent may not create an account anywhere, may not handle a passcode, and may not commit anything from `private/`.

- Deploys: `on my word`
- Subagents: `none`

## Journal

`default`

## Allowed core dependencies

`serde`, `serde_json` (reference default).
