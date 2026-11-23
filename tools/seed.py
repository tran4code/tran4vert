#!/usr/bin/env python3
"""Loads the program into the database. Run by the person, on their own computer.

    python3.13 tools/seed.py [PATH]      PATH defaults to private/program/program.json

It reads the program file, drops what must not reach a friend's phone (the
private maxes from the PDFs and each day's original date), prints a summary of
every week, asks to continue, then asks for the coach passcode with getpass
(typed, never echoed, never taken from an argument or a file) and calls
`set_program`. It finishes by fetching the state back and printing how many
weeks the database holds as loaded. The address and publishable key come from
web/net/config.js.
"""
import getpass
import json
import pathlib
import re
import sys
import urllib.error
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[1]


def config():
    text = (ROOT / "web/net/config.js").read_text()
    url = re.search(r'url: "([^"]*)"', text).group(1)
    key = re.search(r'key: "([^"]*)"', text).group(1)
    if not url or not key:
        sys.exit("web/net/config.js has no address and key yet: run tools/set_project.py first")
    return url, key


def rpc(url, key, name, body):
    request = urllib.request.Request(
        f"{url}/rest/v1/rpc/{name}",
        data=json.dumps(body).encode(),
        headers={"apikey": key, "Authorization": f"Bearer {key}", "Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return response.read().decode()
    except urllib.error.HTTPError as e:
        try:
            reason = json.loads(e.read().decode()).get("message", "server")
        except Exception:
            reason = "server"
        sys.exit(f"the database refused {name}: {reason}")


def clean(program):
    """The program as the database should hold it."""
    program = {k: v for k, v in program.items() if k not in ("maxes_from_pdf", "blocks_legend", "_about")}
    for cycle in program.get("cycles", []):
        cycle.pop("original_start", None)
        for week in cycle.get("weeks", []):
            for day in week.get("days", []):
                day.pop("original_date", None)
    return program


def summary(program):
    lines = []
    for cycle in program["cycles"]:
        for week in cycle["weeks"]:
            if week.get("missing"):
                lines.append(f"  {cycle['id']} week {week['week']}: not loaded")
            else:
                kinds = ", ".join(d["kind"] for d in week["days"])
                lines.append(f"  {cycle['id']} week {week['week']}: {len(week['days'])} days ({kinds})")
    return lines


def main(argv):
    path = pathlib.Path(argv[0]) if argv else ROOT / "private/program/program.json"
    program = clean(json.loads(path.read_text()))
    print(f"Program from {path.name}:")
    print("\n".join(summary(program)))
    print(f"{len(program.get('not_captured', []))} not-captured notes will go with it.")
    if input("Send it? [y/N] ").strip().lower() != "y":
        sys.exit("not sent")
    url, key = config()
    code = getpass.getpass("Coach passcode: ")
    rpc(url, key, "set_program", {"p_code": code, "p_body": program})
    state = json.loads(rpc(url, key, "get_state", {"p_code": code}))
    weeks = [w for c in state["program"].get("cycles", []) for w in c["weeks"]]
    loaded = sum(1 for w in weeks if not w.get("missing"))
    print(f"The database now holds {loaded} loaded weeks of {len(weeks)}; role {state['role']}.")


if __name__ == "__main__":
    main(sys.argv[1:])
