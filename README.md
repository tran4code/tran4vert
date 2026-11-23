# Tran4Vert

A phone web page for three friends on the same four-week-cycle jump-training program. An athlete enters the group passcode, taps their name, and sees today's session and the week, with working weights worked out from their own maxes. The coach records voice notes on a lift, and any athlete can move a session or add a max.

The program text, the maxes, the voice notes and the passcodes are not in this repository. Static files hold the page. A Supabase project holds the data and checks the passcode in every call. Without a project address in `web/net/config.js`, the page runs as a demo with an invented program and made-up athletes.

## Build, test and serve

Set `DEVELOPER_DIR=/Library/Developer/CommandLineTools` first if the Xcode license is not accepted on this machine.

```bash
./scripts/setup.sh              # once: Python 3.13 venv with Playwright and its browsers
cargo test --workspace          # the core, boundary, copy and fence tests, in seconds
./scripts/build.sh              # release WebAssembly build into dist/
python3 scripts/serve.py        # http://127.0.0.1:8000/, the demo
./scripts/check.sh              # everything, before handing off
python3 scripts/break_tests.py  # each row of tests/breaks.json must make its test fail
python3.13 tools/set_project.py URL KEY   # put the project address and publishable key in the page
python3.13 tools/seed.py        # load the program into the database (asks for the coach passcode)
python3.13 tools/remote_check.py          # check the database refuses what it should (asks for passcodes)
```

Needs Rust (pinned in `rust-toolchain.toml`), `wasm-bindgen-cli` 0.2.127, and Python 3.13.

## Layout

| Path | Holds |
| --- | --- |
| `crates/core` | every rule: dates, the schedule, moved sessions, maxes and working weights, voice notes, the screens |
| `crates/shim` | three functions the page calls; each is one call into the core |
| `web/` | the static page; `net/` is the one place that talks to the database, `save/` the one place that keeps anything on the phone |
| `data/` | reader-visible text, presentation numbers, the app manifest's name and colours |
| `supabase/schema.sql` | the tables (closed to the public key) and the functions that check the passcode |
| `tools/` | the seed tool, the remote check, the project setter and the journal tools |
| `tests/fixtures/` | the invented sample program the tests and the demo use |
| `tests/browser/` | the browser check: the demo in four engines, then the live path against a pretend database |
| `private/` | the transcribed program; ignored by git |
