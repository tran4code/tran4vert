//! A local-only test: when `private/program/program.json` exists (it is not in
//! the repository), the model must read it. On a machine without it, and on
//! CI, the test passes without reading anything.

use std::fs;
use std::path::Path;

use tool_core::program::Program;

#[test]
fn the_private_program_parses_when_it_is_there() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../private/program/program.json");
    let Ok(text) = fs::read_to_string(&path) else {
        eprintln!("private/program/program.json is absent: skipped");
        return;
    };
    let program = Program::parse(&text).unwrap_or_else(|e| panic!("private program: {e}"));
    assert!(!program.cycles.is_empty(), "the private program has no cycles");
    assert!(!program.lifts.is_empty(), "the private program has no lifts");
}
