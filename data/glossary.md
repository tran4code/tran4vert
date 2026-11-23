# Glossary

The source of truth for what each word in this tool means. Strings, tests, the plan and the README use a term in the sense given here. The page shows a definition when a reader hovers the term.

Write each entry as a `### Term` heading, then one or two plain sentences, then an optional `Not:` line listing words that must not be used for it. A number a rule decides is a `{placeholder}` the core fills, so the glossary cannot disagree with the rules. Keep entries in alphabetical order. `crates/core/tests/copy.rs` checks the format, the placeholders and the `Not:` words; `TONE.md` governs the prose.

<!--
An example entry. It sits inside this comment, so it does not count.

### Spout

Where slush comes out. Each line has up to {line.spouts} spouts.

Not: tap, dispenser, nozzle
-->

### Athlete

One of the people following the program, who signs in by tapping their name. Each athlete has a start date, maxes and moved sessions of their own.

Not: user, client, member

### Category

The group a lift belongs to: power, strength, calf, accessory or isometric. Each has its own colour and its own word, so the colour is never the only cue.

Not: classification, tier

### Coach

The person who sets the program and records voice notes, and who is also an athlete. The coach can move any athlete's session.

Not: admin, trainer, owner

### Cycle

A block of {cycle.weeks} weeks with one emphasis. Cycles follow each other without a gap.

Not: block, phase, mesocycle

### Isometric hold

A lift held still for a number of seconds instead of repeated. The program lists its sets and seconds.

Not: static hold, plank

### Lift

A named movement in the program, such as a squat or a clean. A voice note belongs to one lift and shows wherever that lift is assigned.

Not: exercise, movement, drill

### Max

The heaviest weight an athlete has recorded for a lift, with the date. The latest max on or before a date is the one that date uses.

Not: PR, personal best, one rep max

### Moved session

A session placed on a different date from the one the cycle gives it. It shows on the new date and says where it came from, and the old date says where it went.

Not: rescheduled, postponed, swapped

### Session

The work set for one day: a kind (jump, lift, general, isometric or rest) and its blocks in order.

Not: workout, practice, class

### Test

A prescription to find a new max instead of working from an old one. The page offers to enter the result as a max.

Not: assessment, trial

### Voice note

A short spoken recording from the coach, with a title, attached to one lift. It plays on every session that assigns the lift, for every athlete.

Not: memo, audio

### Week

Seven days from Monday to Sunday, counted within a cycle. A week with no program is marked not loaded.

Not: microcycle

### Working weight

The weight to lift now: the athlete's max for the lift times the prescribed percentage, rounded to the nearest {working_weight.step} lb.

Not: load, target, prescription weight
