"""The project's own browser interactions, run by check.py in every engine.

check.py has already loaded the page (in demo mode: no database address is
set, so the page loads the invented sample program and keeps changes in
memory), waited for it to report ready, and let it idle. Drive the tool here
the way a person would, and assert what the page shows with
`check(condition, "what this shows")`.

Expected values are worked out by hand from the sample program, never copied
from the tool's own output. The clock is fixed at Tuesday 2026-12-01, so the
demo's athletes started on Monday 2026-11-30 and today is cycle 1 week 1 Tuesday,
the anvil squat lift session. Sam's demo anvil squat max is 305, and 305 at 75%
is 228.75, which rounds to 230. Waits are on selectors and data-* attributes,
never on a fixed time.
"""

TODAY = "2026-12-01T09:00:00"
PHONE_MIN = 44


def tiny_png():
    """A valid 1 by 1 PNG, built here so its checksums are right."""
    import struct
    import zlib

    def chunk(kind, data):
        body = kind + data
        return struct.pack(">I", len(data)) + body + struct.pack(">I", zlib.crc32(body) & 0xFFFFFFFF)

    header = struct.pack(">IIBBBBB", 1, 1, 8, 6, 0, 0, 0)
    pixel = zlib.compress(b"\x00\xc6\xff\x3d\xff")
    return b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header) + chunk(b"IDAT", pixel) + chunk(b"IEND", b"")


def text(page, selector):
    return (page.text_content(selector) or "").strip()


def dismiss_all(page):
    """Reads every message on the screen; a message covers the page until it is read."""
    while page.query_selector("#popup button"):
        page.click("#popup button:text-is('Got it')")


def screen(page, name):
    page.wait_for_selector(f"body[data-screen='{name}']", state="attached")


def nav(page, label):
    page.click(f"#nav button:text-is('{label}')")


def fake_microphone(page):
    page.evaluate(
        """() => {
          const context = new AudioContext();
          const dest = context.createMediaStreamDestination();
          const tone = context.createOscillator();
          tone.connect(dest);
          tone.start();
          navigator.mediaDevices.getUserMedia = async () => dest.stream;
        }"""
    )


def run(page, check, engine):
    # With a project address in the page, the first screen is the passcode screen and
    # nothing has been sent. The rest of the live path is tests/browser/live_mock.py;
    # the demo interactions below run on a demo copy of dist/ (tests/browser/run_demo.py).
    if page.get_attribute("body", "data-screen") == "passcode":
        check(page.query_selector("#passcode-group") is not None, "the passcode screen asks for the group passcode")
        check(page.query_selector("#nav button") is None or page.is_hidden("#nav"), "no screens are offered before a passcode")
        return
    # --- the first screen, with a date we know ---
    page.clock.install(time=TODAY)
    page.reload()
    page.wait_for_selector("body[data-ready='true']", state="attached")
    screen(page, "who")
    check(text(page, "#title") == "Who are you?", "the name screen is the first screen in the demo")
    names = [text(page, f"[data-row='athlete-{n}'] strong") for n in (1, 2, 3)]
    check(names == ["Sam", "Jo", "Pat"], f"three names are listed {names}")
    check(text(page, "#banner") == "Demo with made-up data", "the demo says it is a demo")
    check(text(page, "#hero h2") == "Reach 50 inches.", "the first screen's headline")
    check(text(page, ".side.before .inches") == "20 inches" and text(page, ".side.after .inches") == "50 inches", "the two jump heights are labelled in inches")
    check(len(page.query_selector_all(".lines p")) == 3, "three motivating lines")
    still = page.evaluate("() => getComputedStyle(document.querySelector('.side.after .jumper')).transform")
    page.wait_for_function(
        "(before) => getComputedStyle(document.querySelector('.side.after .jumper')).transform !== before", arg=still, polling=50
    )
    check(True, "the character on the right is jumping (its transform changes)")
    check(page.query_selector("[data-row='athlete-1'] .avatar b") is not None, "a tile shows an initial when there is no picture")

    # --- today ---
    page.click("[data-row='athlete-1'] button")
    screen(page, "today")
    check(text(page, "#title") == "Today · Tuesday 1 Dec 2026", f"today's date is the title {text(page, '#title')!r}")
    check(text(page, "#session-c1-1-1 h2") == "Lift", "the session kind is a text label")
    weight = text(page, "[data-row='session-c1-1-1-b1-weight']")
    check("230 lb" in weight and "305 lb" in weight, f"305 at 75% shows 230 beside the max {weight!r}")
    cats = page.evaluate(
        """() => [...document.querySelectorAll("#session-c1-1-1 li[data-category] > .row-head .tag")]
              .map(t => t.textContent + ":" + t.parentElement.parentElement.dataset.category)"""
    )
    check(cats == ["Strength:strength", "Accessory:accessory", "Accessory:accessory", "Calf:calf"], f"each lift block has its category as a word and a colour key {cats}")
    colours = page.evaluate(
        """() => ["power", "strength", "calf", "accessory", "isometric"].map(c => {
              const probe = document.createElement("span"); probe.dataset.category = c; probe.className = "chip";
              document.body.append(probe); const bg = getComputedStyle(probe).backgroundColor; probe.remove(); return bg; })"""
    )
    check(len(set(colours)) == 5, f"the five categories have five different colours {colours}")
    check(
        text(page, "[data-row='session-c1-1-1-b1-note'] blockquote") == "Slow down, fast up.",
        "the coach's note is shown word for word",
    )
    names_in_nav = [(b.text_content() or "").strip() for b in page.query_selector_all("#nav button")]
    check(names_in_nav == ["Today", "Week", "Maxes", "Cycles"], f"the bottom bar {names_in_nav}")

    # --- touch targets and text size (nothing smaller than body text) ---
    smallest = page.evaluate(
        """() => Math.min(...[...document.querySelectorAll('button')]
              .filter(b => b.offsetParent !== null)
              .map(b => b.getBoundingClientRect().height))"""
    )
    check(smallest >= PHONE_MIN, f"every button is at least {PHONE_MIN} CSS pixels tall (smallest {smallest})")
    smallest_text = page.evaluate(
        """() => Math.min(...[...document.querySelectorAll('#panels *, header *, nav *')]
              .filter(e => e.childNodes.length && [...e.childNodes].some(n => n.nodeType === 3 && n.textContent.trim()))
              .map(e => parseFloat(getComputedStyle(e).fontSize)))"""
    )
    body_text = page.evaluate("() => parseFloat(getComputedStyle(document.body).fontSize)")
    check(smallest_text >= body_text * 0.99, f"no text is smaller than body text ({smallest_text} against {body_text})")
    touch = page.evaluate("() => getComputedStyle(document.documentElement).touchAction")
    check(touch == "manipulation", f"touch-action is manipulation {touch!r}")

    # --- the week ---
    nav(page, "Week")
    screen(page, "week")
    tags = [text(page, f"[data-row='day-2026-{d}'] .tag") for d in ("11-30", "12-01", "12-02", "12-03", "12-04", "12-05", "12-06")]
    check(tags == ["Jump", "Lift", "General", "Jump", "Lift", "Isometric", "Rest"], f"Monday to Sunday {tags}")
    page.click("button[data-action='week.shift']:text-is('Later week')")
    check("week 2" in text(page, "#title"), f"a later week can be reached {text(page, '#title')!r}")
    page.click("button[data-action='week.shift']:text-is('This week')")
    check("week 1" in text(page, "#title"), "this week comes back")

    # --- move a session, see the markers, put it back ---
    nav(page, "Today")
    page.click("#session-c1-1-1 button:text-is('Move this session')")
    page.wait_for_selector("#move", state="attached")
    page.fill("#move-to", "2026-12-03")
    page.click("#move button:text-is('Move')")
    screen(page, "today")
    page.wait_for_selector("#session-c1-1-1 .marker", state="attached")
    check(text(page, "#title") == "Thursday 3 Dec 2026", "the moved session's new date is shown")
    check(text(page, "#session-c1-1-1 .marker") == "Moved from Tue 1 Dec", "the moved session says where it came from")
    check(page.query_selector("#session-c1-1-3") is not None, "the new date also keeps its own session")
    nav(page, "Week")
    check(text(page, "[data-row='day-2026-12-01'] .marker") == "Moved to Thu 3 Dec", "the old date says where it went")
    check(text(page, "[data-row='day-2026-12-03'] .marker") == "Moved from Tue 1 Dec", "the new date says where it came from")
    page.click("[data-row='day-2026-12-03'] button")
    page.click("#session-c1-1-1 button:text-is('Put it back')")
    nav(page, "Week")
    check(page.query_selector("[data-row='day-2026-12-01'] .marker") is None, "undoing a move clears the markers")
    day_chips = [text(page, f"[data-row='day-2026-12-01'] .chip:nth-child({n})") for n in (1, 2, 3)]
    check(day_chips == ["Strength", "Calf", "Accessory"], f"the week shows each day's categories {day_chips}")
    check(len(page.query_selector_all("#legend .chip")) == 5, "the week has a colour key with five categories")

    # --- maxes ---
    nav(page, "Maxes")
    screen(page, "maxes")
    check("305 lb" in text(page, "[data-row='max-anvil_squat-1']"), "Sam's current anvil squat max is shown")
    check("No max yet" in text(page, "[data-row='max-anvil_squat-3']"), "an athlete with no max says so")
    page.select_option("#max-form-lift", "anvil_squat")
    page.fill("#max-form-weight", "315")
    page.fill("#max-form-date", "2026-12-01")
    page.click("#max-form button:text-is('Save max')")
    page.wait_for_selector("[data-row='max-anvil_squat-1']:has-text('315 lb')", state="attached")
    check("315 lb" in text(page, "[data-row='max-anvil_squat-1']"), "a new max becomes the current max")
    nav(page, "Today")
    check(
        "235 lb" in text(page, "[data-row='session-c1-1-1-b1-weight']"),
        "the working weight follows the new max: 315 at 75% is 236.25, which rounds to 235",
    )

    # --- checking things off, finishing, and the message that follows ---
    nav(page, "Today")
    page.fill("#session-c1-1-1-b1-weight", "235")
    page.click("[data-row='session-c1-1-1-b1'] button.check:text-is('Set 1')")
    page.wait_for_selector("[data-row='session-c1-1-1-b1'] button.check[aria-pressed='true']", state="attached")
    check("1 of 15 checked" in text(page, "#session-c1-1-1 > p"), f"checking a set updates the progress {text(page, '#session-c1-1-1 > p')!r}")
    check(page.input_value("#session-c1-1-1-b1-weight") == "235", "the weight typed stays after checking a set")
    page.click("#session-c1-1-1 button:text-is('Finish session')")
    page.wait_for_selector("#popup .card", state="attached")
    check(
        text(page, "#popup p") == "Sam finished. Nice work, 1 this week. You beat Jo to it.",
        f"finishing shows an encouraging message that names who was beaten {text(page, '#popup p')!r}",
    )
    page.click("#popup button:text-is('Got it')")
    check(page.query_selector("#popup") is None, "the message goes away once read")
    check(text(page, "#session-c1-1-1 .marker") == "Finished", "the finished session is marked")

    # --- cycles ---
    nav(page, "Cycles")
    screen(page, "cycles")
    check(text(page, "[data-row='cycle-c1-w3'] .tag") == "Not loaded", "a week with no data is marked not loaded")
    check(text(page, "[data-row='cycle-c1-w1'] .tag") == "Loaded", "a loaded week is marked loaded")
    gaps = page.query_selector_all("#not-captured li")
    check(len(gaps) == 3, f"the not-captured notes are listed, three in the sample ({len(gaps)})")

    # --- the coach: a recorder on the lift screen, a note that shows on the lift ---
    page.click("#who button")
    screen(page, "who")
    page.click("#demo-coach button")
    page.click("[data-row='athlete-1'] button")
    screen(page, "today")
    check("Coach" in [(b.text_content() or "").strip() for b in page.query_selector_all("#nav button")], "the coach has a Coach screen")
    page.click("[data-row='session-c1-1-1-b1'] button:text-is('Open lift')")
    screen(page, "lift")
    # a rival finishes first: Jo (the coach can log for anyone) finishes, then Pat is nudged
    page.click("#who button")
    page.click("[data-row='athlete-2'] button")
    screen(page, "today")
    # Sam finished first today, so Jo is nudged before doing anything else
    page.wait_for_selector("#popup .card", state="attached")
    check(text(page, "#popup p").startswith("Sam"), f"the one who finished second is nudged {text(page, '#popup p')!r}")
    page.click("#popup button:text-is('Got it')")
    # a second message may follow (Sam's weight); read them all before the page is free to tap
    dismiss_all(page)
    page.click("#session-c1-1-1 button:text-is('Finish session')")
    page.wait_for_selector("#popup", state="attached")
    dismiss_all(page)
    page.click("#who button")
    page.click("[data-row='athlete-3'] button")
    page.wait_for_selector("#popup .card", state="attached")
    page.click("#popup button:text-is('Got it')")
    page.wait_for_selector("#popup .card", state="attached")
    check(
        text(page, "#popup p") == "Jo already finished. Your move.",
        f"when a rival finishes first the other athlete is nudged, one message at a time {text(page, '#popup p')!r}",
    )
    dismiss_all(page)
    page.click("#who button")
    page.click("[data-row='athlete-1'] button")
    screen(page, "today")
    dismiss_all(page)
    page.click("[data-row='session-c1-1-1-b1'] button:text-is('Open lift')")
    screen(page, "lift")
    check(page.query_selector("#recorder") is not None, "the coach sees the recorder on a lift")
    # a picture: chosen on the Coach screen, shown on the tile
    nav(page, "Coach")
    page.select_option("#avatar-athlete", "2")
    pixel = tiny_png()
    page.set_input_files("#avatar-file", {"name": "a.png", "mimeType": "image/png", "buffer": pixel})
    page.click("#who button")
    page.wait_for_selector("[data-row='athlete-2'] img", state="attached")
    check(page.query_selector("[data-row='athlete-3'] img") is None, "only the athlete who got a picture shows it")
    page.click("[data-row='athlete-1'] button")
    screen(page, "today")
    page.click("[data-row='session-c1-1-1-b1'] button:text-is('Open lift')")
    screen(page, "lift")
    check(
        "No voice note" in text(page, "#lift-notes"),
        "a lift with no voice note says so",
    )
    if engine.startswith("chromium"):
        fake_microphone(page)
        page.select_option("#rec-lift", "anvil_squat")
        page.click("#recorder button[data-action='rec.start']")
        page.wait_for_selector("#recorder[data-state='recording']", state="attached")
        page.wait_for_function("() => document.getElementById('rec-timer').textContent.length > 0", polling=100)
        page.click("#recorder button[data-action='rec.stop']")
        page.wait_for_selector("#recorder[data-state='recorded']", state="attached")
        page.fill("#rec-title", "Knees out")
        page.click("#recorder button[data-action='rec.save']")
        page.wait_for_selector("#lift-notes [data-row^='note-']", state="attached")
        check(text(page, "#lift-notes [data-row^='note-'] strong") == "Knees out", "a saved voice note shows on its lift")
        nav(page, "Today")
        page.wait_for_selector("[data-row^='session-c1-1-1-b1-voice']", state="attached")
        check(
            text(page, "[data-row^='session-c1-1-1-b1-voice'] strong") == "Knees out",
            "the voice note is offered on the session that assigns the lift",
        )
    else:
        print(f"  (the recorder is driven in chromium only; {engine} has no fake microphone here)")
