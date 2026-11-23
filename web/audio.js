// The recorder: the microphone, a visible timer, listen, and a note to save.
// It asks for the microphone only when Record is pressed. It decides nothing
// about the note: the core checks the lift, the title and the length when the
// page passes them on, and the limits come from the core's panel.

function make(tag, text) {
  const node = document.createElement(tag);
  if (text !== undefined) node.textContent = text;
  return node;
}

export function mount(box, panel, labels, cfg, handlers) {
  const lift = panel.fields.find((f) => f.name === "lift");
  const title = panel.fields.find((f) => f.name === "title");
  const label = (name) => (panel.actions.find((a) => a.name === name) || { label: name }).label;

  const pickLift = make("select");
  pickLift.id = "rec-lift";
  for (const c of lift.choices) {
    const option = make("option", c.label);
    option.value = c.value;
    pickLift.append(option);
  }
  pickLift.value = lift.value;
  const pickTitle = make("input");
  pickTitle.id = "rec-title";
  pickTitle.type = "text";
  if (title.max) pickTitle.maxLength = title.max;
  const lift_label = make("label", lift.label);
  lift_label.htmlFor = pickLift.id;
  const title_label = make("label", title.label);
  title_label.htmlFor = pickTitle.id;

  const status = make("p", labels.page_rec_ready);
  status.id = "rec-status";
  status.setAttribute("role", "status");
  const timer = make("p", "");
  timer.id = "rec-timer";

  const buttons = {};
  const bar = make("div");
  bar.className = "actions";
  for (const name of ["rec.start", "rec.stop", "rec.listen", "rec.save", "rec.discard"]) {
    const b = make("button", label(name));
    b.type = "button";
    b.dataset.action = name;
    buttons[name] = b;
    bar.append(b);
  }

  let stream = null;
  let recorder = null;
  let chunks = [];
  let startedAt = 0;
  let elapsed = 0;
  let ticker = null;
  let stopper = null;
  let blob = null;

  function show() {
    const recording = recorder !== null && recorder.state === "recording";
    buttons["rec.start"].disabled = recording;
    buttons["rec.stop"].disabled = !recording;
    buttons["rec.listen"].disabled = blob === null;
    buttons["rec.save"].disabled = blob === null;
    buttons["rec.discard"].disabled = blob === null && !recording;
    box.dataset.state = recording ? "recording" : blob === null ? "ready" : "recorded";
  }

  function tick() {
    const ms = Date.now() - startedAt;
    timer.textContent = handlers.format(String(ms));
  }

  function release() {
    clearInterval(ticker);
    clearTimeout(stopper);
    if (stream) {
      stream.getTracks().forEach((t) => t.stop());
      stream = null;
    }
  }

  async function begin() {
    if (typeof MediaRecorder === "undefined" || !navigator.mediaDevices) {
      status.textContent = labels.page_rec_unsupported;
      return;
    }
    try {
      stream = await navigator.mediaDevices.getUserMedia({ audio: true });
    } catch (e) {
      status.textContent = labels.page_rec_denied;
      return;
    }
    const mimeType = cfg.mimes.find((m) => MediaRecorder.isTypeSupported(m)) || "";
    const options = { audioBitsPerSecond: cfg.bits_per_second };
    if (mimeType !== "") options.mimeType = mimeType;
    recorder = new MediaRecorder(stream, options);
    chunks = [];
    blob = null;
    recorder.ondataavailable = (e) => chunks.push(e.data);
    recorder.onstop = () => {
      elapsed = Date.now() - startedAt;
      blob = new Blob(chunks, { type: recorder.mimeType || mimeType });
      release();
      timer.textContent = handlers.format(String(elapsed));
      status.textContent = labels.page_rec_ready;
      show();
    };
    startedAt = Date.now();
    recorder.start();
    ticker = setInterval(tick, cfg.tick_ms);
    stopper = setTimeout(end, panel.limit * cfg.ms_per_second);
    status.textContent = labels.page_rec_recording;
    show();
  }

  function end() {
    if (recorder && recorder.state === "recording") recorder.stop();
  }

  function discard() {
    end();
    release();
    blob = null;
    chunks = [];
    timer.textContent = "";
    status.textContent = labels.page_rec_ready;
    show();
  }

  buttons["rec.start"].addEventListener("click", begin);
  buttons["rec.stop"].addEventListener("click", end);
  buttons["rec.discard"].addEventListener("click", discard);
  buttons["rec.listen"].addEventListener("click", () => {
    const url = URL.createObjectURL(blob);
    const player = new Audio(url);
    player.onended = () => URL.revokeObjectURL(url);
    player.play().catch(() => {
      status.textContent = labels.page_play_failed;
    });
  });
  buttons["rec.save"].addEventListener("click", async () => {
    if (pickTitle.value.trim() === "") {
      status.textContent = labels.page_note_missing;
      return;
    }
    try {
      await handlers.save({ lift: pickLift.value, title: pickTitle.value, ms: elapsed, blob });
      status.textContent = labels.page_rec_saved;
      blob = null;
      pickTitle.value = "";
      timer.textContent = "";
      show();
    } catch (e) {
      status.textContent = labels.page_save_failed;
    }
  });

  box.append(lift_label, pickLift, title_label, pickTitle, status, timer, bar);
  show();
}
