#!/usr/bin/env python3
"""The live path in the browser, against a pretend database.

check.py runs the page in demo mode. This script copies dist/, sets a pretend
project address in the copy (the same edit tools/set_project.py makes), and
answers that address from Playwright's router with a small in-memory version
of the database's functions. It shows, in each engine:

  - first load and idle send nothing to the database, and the passcode screen shows;
  - a wrong passcode is refused in words, and nothing is kept;
  - the right passcode shows three names; a tap shows that athlete's session;
  - after a reload the passcode and the name are not asked for again;
  - a new max is sent as a call with the athlete, lift, weight and date, and the
    data is fetched again;
  - with the database unreachable, a reload still draws the last session, says it
    is offline, and turns the change buttons off;
  - "Forget this phone" brings back the passcode screen;
  - no request goes anywhere except the page's own origin and the pretend address.

The passcodes here are made up for this test and belong to no real project.

    python3 tests/browser/live_mock.py     (BUILDER_ENGINES as for check.py)
"""
import functools
import http.server
import importlib.util
import json
import os
import pathlib
import shutil
import sys
import tempfile
import threading

from playwright.sync_api import sync_playwright

ROOT = pathlib.Path(__file__).resolve().parents[2]
ENGINES = os.environ.get("BUILDER_ENGINES", "chromium firefox webkit").split()
MOCK = "https://mock.supabase.test"
KEY = "pretend-publishable-key-0000000000"
GROUP, COACH = "mock-group-code", "mock-coach-code"
TODAY = "2026-12-01T09:00:00"
failures = []


def check(ok, what):
    print(("  PASS " if ok else "  FAIL ") + what)
    if not ok:
        failures.append(what)


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        ".wasm": "application/wasm",
        ".js": "text/javascript",
        ".json": "application/json",
    }

    def log_message(self, *args):
        pass


class Database:
    """Just enough of the functions in supabase/schema.sql for the page."""

    def __init__(self):
        program = json.loads((ROOT / "tests/fixtures/sample_program.json").read_text())
        self.state = {
            "program": program,
            "athletes": [{"id": i, "name": n, "start": "2026-11-30"} for i, n in ((1, "Sam"), (2, "Jo"), (3, "Pat"))],
            "maxes": [{"id": 1, "athlete": 1, "lift": "anvil_squat", "weight": 305, "date": "2026-11-02"}],
            "moves": [],
            "notes": [],
            "cycle_notes": [],
            "as_of": "2026-11-23",
        }
        self.up = True
        self.calls = []

    def answer(self, name, body):
        code = body.get("p_code")
        role = "coach" if code == COACH else "athlete" if code == GROUP else None
        if role is None:
            return 400, {"message": "bad_passcode"}
        if name == "get_state":
            return 200, {**self.state, "role": role}
        if name == "add_max":
            rows = self.state["maxes"]
            rows.append({"id": len(rows) + 1, "athlete": body["p_athlete"], "lift": body["p_lift"],
                         "weight": body["p_weight"], "date": body["p_date"]})
            return 200, len(rows)
        return 404, {"message": "unknown"}


def start_site():
    work = pathlib.Path(tempfile.mkdtemp(prefix="tran4vert-live-"))
    site = work / "dist"
    shutil.copytree(ROOT / "dist", site)
    tools = load("set_project", ROOT / "tools/set_project.py")
    tools.check(MOCK.replace("mock.supabase.test", "mock.supabase.co"), KEY)  # the check the real tool runs
    tools.apply(site, MOCK, KEY, web=".")
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Handler, directory=str(site)))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return work, server


def run_engine(p, engine, origin):
    db = Database()  # each engine starts from the same data
    print(f"\n[{engine}] {origin}/")
    launcher, _, device = engine.partition("-")
    profile = dict(p.devices["iPhone 15"]) if device == "iphone" else {}
    browser = getattr(p, launcher).launch()
    context = browser.new_context(**profile)
    hosts = set()
    errors = []
    db.calls.clear()
    db.up = True

    def handle(route):
        request = route.request
        hosts.add("mock")
        cors = {
            "access-control-allow-origin": origin,
            "access-control-allow-headers": "apikey, authorization, content-type, accept",
            "access-control-allow-methods": "POST, OPTIONS",
        }
        if request.method == "OPTIONS":
            route.fulfill(status=204, headers=cors)
            return
        if not db.up:
            route.abort("failed")
            return
        name = request.url.rsplit("/", 1)[-1]
        body = json.loads(request.post_data or "{}")
        db.calls.append((name, body, request.headers.get("apikey")))
        status, payload = db.answer(name, body)
        route.fulfill(status=status, headers={**cors, "content-type": "application/json"}, body=json.dumps(payload))

    def guard(route):
        if route.request.url.startswith(origin) or route.request.url.startswith("data:") or route.request.url.startswith("blob:"):
            route.continue_()
        else:
            hosts.add(route.request.url)
            route.abort()

    context.route("**/*", guard)
    # Routes registered later win, so the pretend database answers before the guard sees its address.
    context.route(MOCK + "/**", handle)
    page = context.new_page()
    page.on("pageerror", lambda e: errors.append(str(e)))
    page.on("console", lambda m: errors.append(m.text) if m.type == "error" and "Failed to load resource" not in m.text and "net::ERR" not in m.text and "CORS request did not succeed" not in m.text else None)
    page.goto(origin + "/index.html")
    page.wait_for_selector("body[data-ready='true']", state="attached")
    # The clock is installed after the first load and the page reloaded, as interactions.py does,
    # because Firefox and WebKit apply it only to a page loaded after it.
    page.clock.install(time=TODAY)
    page.reload()
    page.wait_for_selector("body[data-ready='true']", state="attached")
    page.wait_for_timeout(2000)
    check(db.calls == [] and "mock" not in hosts, f"first load and idle send nothing to the database {db.calls}")
    check(page.get_attribute("body", "data-screen") == "passcode", "the passcode screen is shown")

    def submit(code):
        page.fill("#passcode-group", code)
        page.click("#passcode button:text-is('Enter')")

    submit("nope-nope-nope")
    page.wait_for_selector("#passcode .marker", state="attached")
    check(page.text_content("#passcode .marker") == "That passcode was not accepted.", "a wrong passcode is refused in words")
    check(page.get_attribute("body", "data-screen") == "passcode", "a wrong passcode stays on the passcode screen")
    submit(GROUP)
    page.wait_for_selector("body[data-screen='who']", state="attached")
    names = [page.text_content(f"[data-row='athlete-{n}'] strong") for n in (1, 2, 3)]
    check(names == ["Sam", "Jo", "Pat"], f"the right passcode shows three names {names}")
    check(db.calls[-1][2] == KEY and db.calls[-1][1]["p_code"] == GROUP, "the call carries the publishable key and the passcode")
    page.click("[data-row='athlete-1'] button")
    page.wait_for_selector("body[data-screen='today']", state="attached")
    shown = page.text_content("#title")
    check("230 lb" in (page.text_content("[data-row='session-c1-1-1-b1-weight']") or ""), f"the session shows 305 at 75% as 230 ({shown!r})")

    before = len(db.calls)
    page.reload()
    page.wait_for_selector("body[data-ready='true']", state="attached")
    page.wait_for_selector("body[data-screen='today']", state="attached")
    check(page.query_selector("#session-c1-1-1") is not None, "after a reload the name and passcode are remembered")
    page.wait_for_function("() => document.getElementById('message').textContent === ''", polling=100)
    check(len(db.calls) > before, "the reload fetched the data again")

    page.click("#nav button:text-is('Maxes')")
    page.fill("#max-form-weight", "330")
    page.fill("#max-form-date", "2026-12-01")
    calls = len(db.calls)
    page.click("#max-form button:text-is('Save max')")
    page.wait_for_selector("[data-row='max-anvil_squat-1']:has-text('330 lb')", state="attached")
    sent = [c for c in db.calls[calls:] if c[0] == "add_max"]
    check(
        len(sent) == 1
        and sent[0][1]["p_athlete"] == 1
        and sent[0][1]["p_lift"] == "anvil_squat"
        and sent[0][1]["p_weight"] == 330
        and sent[0][1]["p_date"] == "2026-12-01",
        f"a new max is sent with athlete, lift, weight and date {sent}",
    )
    check(db.calls[-1][0] == "get_state", "the data is fetched again after a change")

    db.up = False
    page.reload()
    page.wait_for_selector("body[data-ready='true']", state="attached")
    page.wait_for_selector("#banner:has-text('Offline')", state="attached")
    check(page.text_content("#banner") == "Offline, as of 2026-11-23. Changes are off.", "offline, the page says so")
    page.click("#nav button:text-is('Today')")
    check(page.query_selector("#session-c1-1-1") is not None, "offline, the last session is still drawn")
    page.click("#nav button:text-is('Maxes')")
    check(page.is_disabled("#max-form button:text-is('Save max')"), "offline, the change buttons are off")

    db.up = True
    page.click("button:text-is('Forget this phone')")
    page.wait_for_selector("body[data-screen='passcode']", state="attached")
    check(True, "forgetting this phone brings back the passcode screen")
    check(not errors, f"no console error or page error {errors}")
    check(hosts <= {"mock"}, f"no request left for any other address {hosts}")
    browser.close()


def main():
    if not (ROOT / "dist/index.html").exists():
        sys.exit("dist/ is missing: run scripts/build.sh first")
    work, server = start_site()
    origin = "http://127.0.0.1:%d" % server.server_address[1]
    try:
        with sync_playwright() as p:
            for engine in ENGINES:
                run_engine(p, engine, origin)
    finally:
        server.shutdown()
        shutil.rmtree(work, ignore_errors=True)
    if failures:
        print(f"\nLIVE MOCK CHECK FAILED: {len(failures)} problem(s)")
        sys.exit(1)
    print(f"\nLIVE MOCK CHECK PASSED in {', '.join(ENGINES)}")


if __name__ == "__main__":
    main()
