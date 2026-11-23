# Measurements

Stage 4, recorded by the agent before building and again at handoff. `B2 Require step-by-step justification`.

The state measured rather than recalled, so later changes can be compared with it. A probe prints numbers and asserts loose bounds at most; record its output here with the command that produced it.

| What | Command | Result | When |
| --- | --- | --- | --- |
| Toolchain | `rustc --version; wasm-bindgen --version` | rustc 1.99.0 (b940084d7 2026-09-28); wasm-bindgen 0.2.127; Playwright 1.63.0 | 2026-11-23 |
| Core tests | `./scripts/count-tests.sh` | template baseline: 38 passed, 0 failed | 2026-11-23 |
| Full check | `./scripts/check.sh` | baseline on the copy as planned (before any build edit): steps 1 to 6 pass; step 7 fails because `HANDOFF.md` does not name the `person` rows R1.4 to R1.7; step 8 fails because the opening journal entry is blank and the journal does not name those rows. The browser check (step 9) run on its own passes in chromium, firefox, webkit and webkit-iphone. Cargo needed `DEVELOPER_DIR=/Library/Developer/CommandLineTools` because the Xcode license is not accepted (R2.1). | 2026-11-23 |
| WebAssembly size | printed by `./scripts/build.sh` | | |
| Breaks caught | `python3 scripts/break_tests.py` | | |

## Not measured

Devices, browsers and conditions no measurement here covers.
