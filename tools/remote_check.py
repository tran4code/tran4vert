#!/usr/bin/env python3
"""Checks the live database refuses what it should (PLAN decisions 13 and 14).

    python3.13 tools/remote_check.py [--write]

Uses only the address and the publishable key from web/net/config.js, plus
secrets it asks for with getpass (typed by the person, never echoed, never
read from an argument or a file; press Enter to skip one): an athlete's
sign-in written `<athlete id>:<goal>` (for example 1:40) and the coach
passcode. It checks that:

  1. a direct read of each table with the publishable key returns no rows;
  2. `get_names` answers without a sign-in and returns only ids and names;
  3. every function refuses a wrong sign-in with `bad_passcode`;
  4. five wrong tries lock an athlete out with `too_many` (tried on an athlete
     number that does not exist, so no real athlete is locked out);
  5. `set_passcode` cannot be called with the publishable key at all;
  6. with an athlete's sign-in, `get_state` works and names that athlete, shows
     no goals, every coach function is refused with `coach_only`, and another
     athlete's data cannot be changed (`not_yours`);
  7. with the coach passcode, `get_state` says role `coach` and shows the goals.

With --write (and the coach passcode) it also records a one-second voice note,
checks a 3 MB one is refused, and deletes what it made. Prints PASS or FAIL for
each check and exits non-zero on any failure.
"""
import base64
import getpass
import json
import pathlib
import re
import sys
import urllib.error
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[1]
TABLES = ["program", "athletes", "maxes", "moves", "voice_notes", "cycle_notes", "secrets"]
WRONG = "this-is-not-a-passcode"
WRONG_PICK = "999999:30"
ANY = {
    "get_state": {},
    "get_note_audio": {"p_id": 1},
    "add_max": {"p_athlete": 1, "p_lift": "x", "p_weight": 100, "p_date": "2026-01-05"},
    "delete_max": {"p_id": 1},
    "move_session": {"p_athlete": 1, "p_cycle": "c1", "p_week": 1, "p_dow": 1, "p_to": "2026-01-07"},
    "undo_move": {"p_athlete": 1, "p_cycle": "c1", "p_week": 1, "p_dow": 1},
}
COACH = {
    "add_note": {"p_lift": "x", "p_title": "t", "p_seconds": 1, "p_date": "2026-01-05", "p_mime": "audio/mp4", "p_audio": "AAAA"},
    "delete_note": {"p_id": 1},
    "set_start_date": {"p_athlete": 1, "p_start": "2026-01-05"},
    "set_cycle_note": {"p_cycle": "c1", "p_text": "x"},
    "set_program": {"p_body": {}},
    "set_goal": {"p_athlete": 1, "p_goal": 40},
}
failures = []


def check(ok, what):
    print(("  PASS " if ok else "  FAIL ") + what)
    if not ok:
        failures.append(what)


def config():
    text = (ROOT / "web/net/config.js").read_text()
    url = re.search(r'url: "([^"]*)"', text).group(1)
    key = re.search(r'key: "([^"]*)"', text).group(1)
    if not url or not key:
        sys.exit("web/net/config.js has no address and key yet")
    return url, key


def call(url, key, method, path, body=None):
    request = urllib.request.Request(
        url + path,
        data=None if body is None else json.dumps(body).encode(),
        method=method,
        headers={"apikey": key, "Authorization": f"Bearer {key}", "Content-Type": "application/json"},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as r:
            return r.status, r.read().decode()
    except urllib.error.HTTPError as e:
        return e.code, e.read().decode()


def message(text):
    try:
        return json.loads(text).get("message", "")
    except Exception:
        return text[:80]


def rpc(url, key, name, args, code):
    return call(url, key, "POST", f"/rest/v1/rpc/{name}", {"p_code": code, **args})


def main(argv):
    url, key = config()
    write = "--write" in argv
    print("Tables closed to the publishable key:")
    for table in TABLES:
        status, body = call(url, key, "GET", f"/rest/v1/{table}?select=*")
        rows = json.loads(body) if status == 200 and body.startswith("[") else None
        check(status != 200 or rows == [], f"{table}: a direct read returns nothing (status {status})")
    print("A wrong passcode is refused everywhere:")
    for name, args in {**ANY, **COACH}.items():
        status, body = rpc(url, key, name, args, WRONG)
        check(status != 200 and message(body) == "bad_passcode", f"{name}: bad_passcode (status {status}, {message(body)!r})")
    status, body = rpc(url, key, "set_passcode", {"p_kind": "coach", "p_code": WRONG * 2}, "")
    check(status != 200, f"set_passcode cannot be called with the publishable key (status {status})")
    status, body = call(url, key, "POST", "/rest/v1/rpc/get_names", {})
    names = json.loads(body) if status == 200 else None
    check(
        names is not None and all(set(n) == {"id", "name"} for n in names),
        f"get_names answers without a sign-in and returns only ids and names (status {status})",
    )
    print("Five wrong tries lock an athlete out:")
    last = None
    for _ in range(6):
        status, body = rpc(url, key, "get_state", {}, WRONG_PICK)
        last = message(body)
    check(last == "too_many", f"the sixth wrong try is refused with too_many ({last!r})")
    pick = getpass.getpass("An athlete's sign-in as id:goal, for example 1:40 (Enter to skip): ")
    coach = getpass.getpass("Coach passcode (Enter to skip): ")
    if pick:
        print("With an athlete's sign-in:")
        status, body = rpc(url, key, "get_state", {}, pick)
        state = json.loads(body) if status == 200 else {}
        me = int(pick.split(":", 1)[0])
        check(status == 200 and state.get("role") == "athlete" and state.get("me") == me, "get_state works and names that athlete")
        check(all("goal" not in a for a in state.get("athletes", [])), "no goal is sent to an athlete")
        for name, args in COACH.items():
            status, body = rpc(url, key, name, args, pick)
            check(status != 200 and message(body) == "coach_only", f"{name}: coach_only (status {status}, {message(body)!r})")
        others = [a["id"] for a in state.get("athletes", []) if a["id"] != me]
        if others:
            status, body = rpc(url, key, "add_max", {"p_athlete": others[0], "p_lift": "x", "p_weight": 100, "p_date": "2026-01-05"}, pick)
            check(status != 200 and message(body) == "not_yours", f"another athlete's max cannot be added (status {status}, {message(body)!r})")
    if coach:
        print("With the coach passcode:")
        status, body = rpc(url, key, "get_state", {}, coach)
        coach_state = json.loads(body) if status == 200 else {}
        check(coach_state.get("role") == "coach", "get_state says coach")
        check(any("goal" in a for a in coach_state.get("athletes", [])) or not coach_state.get("athletes"), "the coach is sent the goals")
        if write:
            lifts = list(coach_state["program"].get("lifts", {}))
            if not lifts:
                check(False, "the program is not loaded, so a voice note cannot be tried (run tools/seed.py)")
            else:
                tiny = base64.b64encode(b"x" * 100).decode()
                status, body = rpc(url, key, "add_note", {"p_lift": lifts[0], "p_title": "remote check", "p_seconds": 1,
                                                           "p_date": "2026-01-05", "p_mime": "audio/mp4", "p_audio": tiny}, coach)
                check(status == 200, f"a small voice note is accepted (status {status})")
                if status == 200:
                    note_id = json.loads(body)
                    status, got = rpc(url, key, "get_note_audio", {"p_id": note_id}, coach)
                    check(status == 200, "its audio comes back")
                    rpc(url, key, "delete_note", {"p_id": note_id}, coach)
                big = base64.b64encode(b"x" * 3_000_000).decode()
                status, body = rpc(url, key, "add_note", {"p_lift": lifts[0], "p_title": "too big", "p_seconds": 1,
                                                           "p_date": "2026-01-05", "p_mime": "audio/mp4", "p_audio": big}, coach)
                check(status != 200, f"a 3 MB voice note is refused (status {status}, {message(body)!r})")
    if failures:
        print(f"\nREMOTE CHECK FAILED: {len(failures)} problem(s)")
        sys.exit(1)
    print("\nREMOTE CHECK PASSED")


if __name__ == "__main__":
    main(sys.argv[1:])
