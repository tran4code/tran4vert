#!/usr/bin/env python3
"""Screenshots of the built page in demo mode at the size a phone shows it
(390 by 844 CSS pixels), for the design journal. Uses WebKit with the iPhone
profile, headless, so no window opens. The clock is fixed at Tuesday
2026-12-01 so the shots show the same day each run.

    .venv/bin/python tools/screenshots.py OUTPUT_DIR

Run ./scripts/build.sh first. Prints each file it writes.
"""
import functools
import http.server
import importlib.util
import pathlib
import shutil
import sys
import tempfile
import threading

from playwright.sync_api import sync_playwright

ROOT = pathlib.Path(__file__).resolve().parents[1]
TODAY = "2026-12-01T09:00:00"


class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        ".wasm": "application/wasm",
        ".js": "text/javascript",
        ".json": "application/json",
    }

    def log_message(self, *args):
        pass


def demo_copy(work):
    """dist/ with the project address cleared, so the page runs as the demo."""
    spec = importlib.util.spec_from_file_location("set_project", ROOT / "tools/set_project.py")
    tools = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(tools)
    site = pathlib.Path(work) / "dist"
    shutil.copytree(ROOT / "dist", site)
    tools.apply(site, "", "", web=".")
    return site


def main(out):
    out = pathlib.Path(out)
    out.mkdir(parents=True, exist_ok=True)
    work = tempfile.mkdtemp(prefix="tran4vert-shots-")
    server = http.server.ThreadingHTTPServer(
        ("127.0.0.1", 0), functools.partial(Handler, directory=str(demo_copy(work)))
    )
    threading.Thread(target=server.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{server.server_address[1]}/index.html"
    with sync_playwright() as p:
        profile = dict(p.devices["iPhone 15"])
        profile["viewport"] = {"width": 390, "height": 844}
        browser = p.webkit.launch()
        context = browser.new_context(**profile)
        page = context.new_page()
        page.clock.install(time=TODAY)
        page.goto(base)
        page.wait_for_selector("body[data-ready='true']", state="attached")

        def shot(name, top=True):
            if top:
                page.evaluate("window.scrollTo(0, 0)")
            path = out / f"{name}.png"
            page.screenshot(path=str(path))
            print(path)

        def go(label):
            page.click(f"#nav button:text-is('{label}')")

        shot("1-who")
        page.evaluate("window.scrollTo(0, 420)")
        shot("1b-who-tiles", top=False)
        page.click("[data-row='athlete-1'] button")
        page.wait_for_selector("body[data-screen='today']", state="attached")
        shot("2-today")
        page.evaluate("window.scrollTo(0, 560)")
        shot("3-today-lift", top=False)
        go("Week")
        page.wait_for_selector("body[data-screen='week']", state="attached")
        shot("4-week")
        go("Today")
        page.click("#session-c1-1-1 button:text-is('Move this session')")
        page.fill("#move-to", "2026-12-03")
        page.click("#move button:text-is('Move')")
        page.wait_for_selector("#session-c1-1-1 .marker", state="attached")
        go("Week")
        shot("5-week-moved")
        go("Maxes")
        shot("6-maxes")
        go("Cycles")
        shot("7-cycles")
        page.click("#who button")
        page.click("#demo-coach button")
        # Jo, whose Tuesday session has not been moved.
        page.click("[data-row='athlete-2'] button")
        page.click("[data-row='session-c1-1-1-b1'] button:text-is('Open lift')")
        page.wait_for_selector("body[data-screen='lift']", state="attached")
        shot("8-lift-coach")
        browser.close()
    server.shutdown()
    shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    main(sys.argv[1])
