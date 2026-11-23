#!/usr/bin/env python3
"""Fails if anything from private/ or .env has reached a file that could be published.

The program text is paid coaching material and the maxes belong to the athletes
(BRIEF.md, "Decisions that stay with the person"). This reads
private/program/program.json and .env when they exist and checks that:

  1. no path under private/, and no .env file, is tracked by git;
  2. no tracked text file, no file in dist/ and no page of the design journal
     holds a line of the private program's text (a coach note, a note or text on
     a block, a circuit's items, a theme or a line of not_captured, 20 or more
     characters);
  3. none of them holds a max from the PDFs next to its lift id (the id, a colon or
     a space, then the number);
  4. none of them names an athlete the PDFs name (the keys of maxes_from_pdf), as
     a word;
  5. none of them holds a value of .env (8 or more characters).

It prints the file and line of each problem and never the text it matched.
With no private/ (on CI) the real check is skipped and says so; `--self-test`
plants each kind of leak in a temporary project and shows each is caught, and
needs no private/ at all.

    python3 scripts/leak_check.py
    python3 scripts/leak_check.py --self-test
"""
import json
import pathlib
import re
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[1]
MIN_LINE = 20
MIN_ENV = 8


def strings_of(program):
    """Lines of the program's text that must not be published."""
    out = set()

    def add(text):
        for line in str(text).splitlines():
            line = line.strip()
            if len(line) >= MIN_LINE and not line.startswith("@"):
                out.add(line)

    for text in program.get("notes", {}).values():
        add(text)
    for item in program.get("library", {}).values():
        for text in item.get("items", []):
            add(text)
        if item.get("dose"):
            add(item["dose"])
    for text in program.get("not_captured", []):
        add(text)
    for cycle in program.get("cycles", []):
        add(cycle.get("cycle_note", ""))
        add(cycle.get("emphasis", ""))
        for theme in cycle.get("week_themes", []):
            if theme:
                add(theme)
        for week in cycle.get("weeks", []):
            for day in week.get("days", []):
                for block in day.get("blocks", []):
                    for field in ("note", "text", "detail", "dose"):
                        if isinstance(block.get(field), str):
                            add(block[field])
    return out


def maxes_of(program):
    """(athlete, lift id, number) for each max the PDFs held."""
    out = []
    for athlete, lifts in program.get("maxes_from_pdf", {}).items():
        if athlete.startswith("_") or not isinstance(lifts, dict):
            continue
        for lift, number in lifts.items():
            if isinstance(number, int):
                out.append((athlete, lift, number))
    return out


def env_values(text):
    values = set()
    for line in text.splitlines():
        line = line.strip()
        if line and not line.startswith("#") and "=" in line:
            value = line.split("=", 1)[1].strip().strip("'\"")
            if len(value) >= MIN_ENV:
                values.add(value)
    return values


def scan(files, program, env):
    """Problems as (path, line number, what). `files` maps a path to its text."""
    lines = strings_of(program)
    maxes = maxes_of(program)
    names = {a for a, _, _ in maxes}
    values = env_values(env)
    problems = []
    for path, text in files.items():
        for n, line in enumerate(text.splitlines(), 1):
            for secret in lines:
                if secret in line:
                    problems.append((path, n, "a line of the private program"))
                    break
            for _, lift, number in maxes:
                if re.search(rf"\b{re.escape(lift)}\b[\W_]{{1,6}}{number}\b", line):
                    problems.append((path, n, "a max from the PDFs beside its lift id"))
            for name in names:
                if re.search(rf"\b{re.escape(name)}\b", line, re.I):
                    problems.append((path, n, "the name of an athlete from the PDFs"))
            for value in values:
                if value in line:
                    problems.append((path, n, "a value from .env"))
    return problems


def read_files(root):
    """Tracked text files, dist/ and the journal's pages, by path."""
    listed = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard"], cwd=root, capture_output=True, text=True
    ).stdout.split("\n")
    paths = [root / p for p in listed if p]
    dist = root / "dist"
    if dist.exists():
        paths += [p for p in dist.rglob("*") if p.is_file()]
    journal = root.parent / f"{root.name}-journal" / "index.html"
    if journal.exists():
        paths.append(journal)
    files = {}
    for p in paths:
        if p.is_file() and not str(p.relative_to(root.parent) if root.parent in p.parents else p).startswith("private"):
            try:
                files[str(p)] = p.read_text()
            except (UnicodeDecodeError, OSError):
                continue
    return files


def tracked_problems(root):
    listed = subprocess.run(["git", "ls-files"], cwd=root, capture_output=True, text=True).stdout.split("\n")
    return [p for p in listed if p.startswith("private/") or p == ".env" or p.startswith(".env.")]


def self_test():
    program = {
        "notes": {"a": "Keep the bar close and finish tall every rep."},
        "cycles": [{"emphasis": "Strength before speed in the base cycle", "weeks": []}],
        "maxes_from_pdf": {"_note": "x", "pat": {"squat_x": 305}},
    }
    env = "SOME_KEY=abcdefgh12345678\n"
    clean = {"a.md": "Nothing private here.\nThe squat_x lift has a max field.\n"}
    assert scan(clean, program, env) == [], "a clean project was flagged"
    planted = {
        "line": "x = 'Keep the bar close and finish tall every rep.'",
        "emphasis": "Strength before speed in the base cycle",
        "max": '"squat_x": 305',
        "max spaced": "squat_x 305",
        "name": "Thanks, Pat.",
        "env": "key=abcdefgh12345678",
    }
    for kind, text in planted.items():
        found = scan({"data/strings.json": text}, program, env)
        assert found, f"a planted {kind} leak was not caught"
    # A number alone, or beside another lift, is not a leak.
    assert scan({"a": "305 at 75%\nrope_clean 305"}, program, env) == []
    with tempfile.TemporaryDirectory() as tmp:
        repo = pathlib.Path(tmp)
        subprocess.run(["git", "init", "-q"], cwd=repo, check=True)
        (repo / "private").mkdir()
        (repo / "private/p.json").write_text("{}")
        subprocess.run(["git", "add", "-f", "private/p.json"], cwd=repo, check=True)
        assert tracked_problems(repo) == ["private/p.json"], "a tracked private file was not caught"
    print("leak check self-test: every planted leak was caught and a clean project passed")


def main(argv):
    if "--self-test" in argv:
        self_test()
        return
    private = ROOT / "private/program/program.json"
    problems = [(p, 0, "tracked by git") for p in tracked_problems(ROOT)]
    if not private.exists():
        print("leak check: private/program/program.json is absent, so the text checks are skipped")
    else:
        program = json.loads(private.read_text())
        env_file = ROOT / ".env"
        env = env_file.read_text() if env_file.exists() else ""
        files = read_files(ROOT)
        problems += scan(files, program, env)
        print(f"leak check: {len(files)} files read, {len(strings_of(program))} private lines, "
              f"{len(maxes_of(program))} private maxes")
    if problems:
        for path, n, what in problems:
            where = pathlib.Path(path)
            try:
                where = where.relative_to(ROOT.parent)
            except ValueError:
                pass
            print(f"LEAK {where}:{n}: {what}")
        sys.exit(1)
    print("leak check: nothing from private/ or .env is in a tracked or published file")


if __name__ == "__main__":
    main(sys.argv[1:])
