// The page draws what the core returns and decides nothing.
//
// The core describes the screen (panels of rows, fields and actions) and
// this file builds the DOM from that description without knowing what any
// panel means. Every button sends its action name and value back through
// `act`, the core's one door. When the core asks for a call to the database,
// the page makes it (web/net/client.js), fetches the data again and hands it
// to the core. The phone keeps a few things (web/save/store.js) so a name and
// a passcode are typed once and the last session shows without a signal.
import init, { start, act, length } from "./pkg/tool_shim.js";
import * as audio from "./audio.js";
import * as net from "./net/client.js";
import * as store from "./save/store.js";

const el = (id) => document.getElementById(id);
const make = (tag, text) => {
  const node = document.createElement(tag);
  if (text !== undefined) node.textContent = text;
  return node;
};

let state = null;
let glossary = "";
let strings = "";
let hints = [];
let labels = {};
let cfg = {};
let kept = "";
const live = net.configured();
// Voice notes recorded in the demo stay in memory; the last one plays.
let demoNote = null;

function call(fn, ...args) {
  return JSON.parse(fn(...args));
}

// A label whose text is a glossary term shows the definition on hover.
function mark(node) {
  const hint = hints.find((h) => h.term === node.textContent);
  if (hint) {
    node.dataset.term = hint.term;
    node.title = hint.definition;
  }
}

function say(text) {
  el("message").textContent = text;
}

function button(a, onclick) {
  const b = make("button", a.label);
  b.type = "button";
  b.disabled = !a.enabled;
  b.dataset.action = a.name;
  b.addEventListener("click", onclick);
  return b;
}

function fieldNode(f, form) {
  const wrap = make("div");
  wrap.className = "field";
  const label = make("label", f.label);
  let input;
  if (f.kind === "select") {
    input = make("select");
    for (const c of f.choices) {
      const option = make("option", c.label);
      option.value = c.value;
      input.append(option);
    }
  } else {
    input = make("input");
    input.type = f.kind;
    if (f.kind === "number") input.inputMode = "numeric";
    if (f.kind === "password") input.autocomplete = "off";
  }
  input.value = f.value;
  input.name = f.name;
  input.id = form + "-" + f.name;
  label.htmlFor = input.id;
  wrap.append(label, input);
  return wrap;
}

// What a form sends: the action's own value and each field's value by name.
function formValue(box, a) {
  const fields = {};
  box.querySelectorAll("[name]").forEach((input) => {
    fields[input.name] = input.value;
  });
  return JSON.stringify({ value: a.value, fields });
}

function rowNode(r) {
  const li = make("li");
  li.dataset.row = r.id;
  const head = make("div");
  head.className = "row-head";
  if (r.tag) {
    const tag = make("span", r.tag);
    tag.className = "tag";
    head.append(tag);
  }
  if (r.label) head.append(make("strong", r.label));
  if (r.marker) {
    const m = make("span", r.marker);
    m.className = "marker";
    head.append(m);
  }
  const body = make("div");
  if (r.detail) {
    const d = make("p", r.detail);
    d.className = "detail";
    body.append(d);
  }
  if (r.note) {
    const q = make("blockquote", r.note);
    q.className = "note";
    body.append(q);
  }
  const children = r.rows || [];
  const actions = make("div");
  actions.className = "actions";
  (r.actions || []).forEach((a) => actions.append(button(a, () => perform(a.name, a.value))));
  if (children.length && !r.open) {
    const details = make("details");
    const summary = make("summary");
    summary.append(head, body);
    details.append(summary, list(children));
    li.append(details);
    if (actions.children.length) li.append(actions);
    return li;
  }
  li.append(head, body);
  if (children.length) li.append(list(children));
  if (actions.children.length) li.append(actions);
  return li;
}

function list(rows) {
  const ul = make("ul");
  rows.forEach((r) => ul.append(rowNode(r)));
  return ul;
}

function panel(p) {
  const box = make("section");
  box.id = p.id;
  box.dataset.kind = p.kind;
  if (p.heading) box.append(make("h2", p.heading));
  if (p.marker && p.kind !== "recorder") {
    const m = make("p", p.marker);
    m.className = "marker";
    box.append(m);
  }
  if (p.text) box.append(make("p", p.text));
  if (p.kind === "recorder") {
    if (p.marker) box.append(make("p", p.marker));
    audio.mount(box, p, labels, cfg.audio, {
      format: (ms) => length(ms),
      save: (v) => saveNote(v),
    });
    return box;
  }
  if (p.rows && p.rows.length) box.append(list(p.rows));
  const form = p.kind === "form" ? make("form") : box;
  if (p.kind === "form") {
    form.addEventListener("submit", (e) => e.preventDefault());
    (p.fields || []).forEach((f) => form.append(fieldNode(f, p.id)));
    box.append(form);
  }
  const bar = make("div");
  bar.className = "actions";
  (p.actions || []).forEach((a) => {
    bar.append(button(a, () => perform(a.name, p.kind === "form" ? formValue(form, a) : a.value)));
  });
  if (bar.children.length) box.append(bar);
  return box;
}

function draw(ui) {
  state = ui.state;
  hints = ui.hints;
  labels = ui.labels;
  document.title = ui.title;
  el("title").textContent = ui.title;
  const who = el("who");
  who.replaceChildren();
  if (ui.header.who) {
    who.append(button({ name: "athlete.switch", label: ui.header.who, enabled: true }, () => perform("athlete.switch", "")));
    who.querySelector("button").title = labels.page_switch;
  }
  el("banner").textContent = ui.header.offline || (live ? "" : labels.page_demo);
  el("panels").replaceChildren(...ui.panels.map(panel));
  const nav = el("nav");
  nav.setAttribute("aria-label", labels.page_nav);
  nav.replaceChildren(
    ...ui.nav.map((n) => {
      const b = button({ name: n.name, label: n.label, enabled: true }, () => perform(n.name, n.value));
      if (n.current) b.setAttribute("aria-current", "page");
      return b;
    }),
  );
  nav.hidden = ui.nav.length === 0;
  document.querySelectorAll("#panels h2, #panels button, #panels .tag").forEach(mark);
  say(ui.message);
  document.body.dataset.screen = ui.screen;
  if (live && ui.keep !== kept && ui.keep !== "") {
    kept = ui.keep;
    store.write("choice", ui.keep);
  }
}

// Hands the core one action; draws the answer; makes the calls it asks for.
async function perform(name, value, extra) {
  if (name === "page.forget") {
    ["group", "coach", "choice", "state"].forEach(store.remove);
    location.reload();
    return true;
  }
  const result = call(act, JSON.stringify(state), glossary, name, value);
  if (result.error) {
    say(result.error);
    return false;
  }
  draw(result.ok);
  if (name === "passcode.submit") {
    const typed = JSON.parse(value).fields;
    net.setCodes({ group: typed.group, coach: typed.coach });
    say(labels.page_checking);
  }
  await runRequests(result.ok.requests, name, extra);
  return true;
}

async function runRequests(requests, name, extra) {
  if (requests.length === 0) return;
  if (!live) {
    for (const r of requests) {
      if (r.rpc === "get_note_audio") {
        await playDemoNote();
      } else {
        const result = call(act, JSON.stringify(state), glossary, "data.apply", JSON.stringify(r));
        if (result.error) say(result.error);
        else draw(result.ok);
      }
    }
    return;
  }
  say(labels.page_loading);
  try {
    for (const r of requests) {
      if (r.rpc === "get_state") {
        const text = await net.fetchState();
        remember(text);
        await perform("data.load", text);
        return;
      }
      if (r.rpc === "get_note_audio") {
        await play(await net.fetchAudio(r.args.id));
      } else if (r.rpc === "add_note") {
        await net.addNote(r.args, extra.blob);
      } else {
        await net.run(r);
      }
    }
    if (requests.some((r) => r.rpc !== "get_note_audio")) await sync();
    else say("");
  } catch (e) {
    await fail(e, name);
  }
}

async function fail(e, name) {
  const result = call(act, JSON.stringify(state), glossary, "net.failed", e.message);
  if (result.ok) draw(result.ok);
  if (name === "passcode.submit" || e.message === "bad_passcode") {
    store.remove("group");
    store.remove("coach");
    store.remove("state");
  }
}

function remember(text) {
  const codes = net.currentCodes();
  store.write("group", codes.group);
  store.write("coach", codes.coach);
  store.write("state", text);
}

// Fetches the shared data again; with no signal, the page keeps what it drew.
async function sync() {
  try {
    const text = await net.fetchState();
    remember(text);
    await perform("data.load", text);
  } catch (e) {
    if (e.message === "bad_passcode") {
      await fail(e, "sync");
    } else {
      const result = call(act, JSON.stringify(state), glossary, "net.offline", "");
      if (result.ok) draw(result.ok);
    }
  }
}

async function play(blob) {
  const url = URL.createObjectURL(blob);
  const player = new Audio(url);
  player.onended = () => URL.revokeObjectURL(url);
  try {
    await player.play();
    say(labels.page_playing);
  } catch (e) {
    say(labels.page_play_failed);
  }
}

async function playDemoNote() {
  if (demoNote) await play(demoNote);
  else say(labels.page_play_failed);
}

// A note's text goes through the core; its bytes go straight to the database.
async function saveNote(v) {
  const value = JSON.stringify({ value: "", fields: { lift: v.lift, title: v.title, ms: String(v.ms) } });
  if (!live) demoNote = v.blob;
  const ok = await perform("note.save", value, { blob: v.blob });
  if (!ok) throw new Error("note");
}

function today() {
  // The date on this phone, as YYYY-MM-DD.
  return new Date().toLocaleDateString("en-CA");
}

async function main() {
  await init();
  strings = await (await fetch("data/strings.json")).text();
  glossary = await (await fetch("data/glossary.md")).text();
  const ui = await (await fetch("data/ui.json")).json();
  cfg = { audio: await (await fetch("data/audio.json")).json() };
  const root = document.documentElement.style;
  root.setProperty("--tap", ui.tap_px + "px");
  root.setProperty("--text", ui.text_px + "px");
  root.setProperty("--gutter", ui.gutter_px + "px");
  const first = call(start, strings, glossary, today());
  if (first.error) {
    say(first.error);
    return;
  }
  draw(first.ok);
  if (live) {
    const group = store.read("group");
    const coach = store.read("coach");
    if (group !== "" || coach !== "") {
      net.setCodes({ group, coach });
      await perform("saved.load", store.read("choice"));
      const cached = store.read("state");
      if (cached !== "") await perform("data.load", cached);
      await sync();
    }
  } else {
    await perform("demo.start", await (await fetch("data/demo_program.json")).text());
  }
  footer();
  document.body.dataset.ready = "true";
}

// Under the screens: how to fill the screen on a phone, and a way to forget this phone.
function footer() {
  const box = el("footer");
  if (!navigator.standalone && matchMedia("(pointer: coarse)").matches) {
    box.append(make("p", labels.page_home_screen));
  }
}

main().catch((e) => {
  // A failure to start is an error the browser check must see.
  console.error(e);
  // The strings may be what failed, so read the one line this needs by hand.
  let line = "";
  try {
    line = JSON.parse(strings).page_start_failed;
  } catch (e) {
    line = "";
  }
  say(line);
});
