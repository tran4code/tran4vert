#!/usr/bin/env python3
"""Checks the live database refuses what it should (PLAN decisions 13 and 14).

    python3.13 tools/remote_check.py [--write]

Uses only the address and the publishable key from web/net/config.js, plus
passcodes it asks for with getpass (typed by the person, never echoed, never
read from an argument or a file; press Enter to skip one). It checks that:

  1. a direct read of each table with the publishable key returns no rows;
  2. every function refuses a wrong passcode with `bad_passcode`;
  3. `set_passcode` cannot be called with the publishable key at all;
  4. with the group passcode, `get_state` works and every coach function is
     refused with `coach_only`;
  5. with the coach passcode, `get_state` says role `coach`.

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
    status, body = rpc(url, key, "set_passcode", {"p_kind": "group", "p_code": WRONG * 2}, "")
    check(status != 200, f"set_passcode cannot be called with the publishable key (status {status})")
    group = getpass.getpass("Group passcode (Enter to skip): ")
    coach = getpass.getpass("Coach passcode (Enter to skip): ")
    if group:
        print("With the group passcode:")
        status, body = rpc(url, key, "get_state", {}, group)
        check(status == 200 and json.loads(body)["role"] == "athlete", "get_state works and says athlete")
        for name, args in COACH.items():
            status, body = rpc(url, key, name, args, group)
            check(status != 200 and message(body) == "coach_only", f"{name}: coach_only (status {status}, {message(body)!r})")
    if coach:
        print("With the coach passcode:")
        status, body = rpc(url, key, "get_state", {}, coach)
        check(status == 200 and json.loads(body)["role"] == "coach", "get_state says coach")
        if write:
            state = json.loads(body)
            lifts = list(state["program"].get("lifts", {}))
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
