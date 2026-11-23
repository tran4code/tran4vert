// The one page file that calls the database (R7). It sends what the core asked
// for and returns what came back; it decides nothing about the program.
//
// Every call is a POST to a function in the database that checks the passcode.
// Nothing is sent until a passcode has been typed (or was kept by this phone
// after it was accepted once).
import { config } from "./config.js";

export function configured() {
  return config.url !== "";
}

let codes = { group: "", coach: "" };

export function setCodes(next) {
  codes = { group: next.group || "", coach: next.coach || "" };
}

export function currentCodes() {
  return codes;
}

export function hasCode() {
  return codes.group !== "" || codes.coach !== "";
}

async function call(name, body, wantBytes) {
  const response = await fetch(config.url + "/rest/v1/rpc/" + name, {
    method: "POST",
    headers: {
      apikey: config.key,
      Authorization: "Bearer " + config.key,
      "Content-Type": "application/json",
      Accept: wantBytes ? "application/octet-stream" : "application/json",
    },
    body: JSON.stringify(body),
  });
  if (!response.ok) {
    let reason = "server";
    try {
      reason = (await response.json()).message || reason;
    } catch (e) {
      // The body was not JSON; the reason stays "server".
    }
    throw new Error(reason);
  }
  return wantBytes ? response.blob() : response.text();
}

// A request's arguments become the function's p_ parameters.
function params(args, code) {
  const body = { p_code: code };
  for (const [key, value] of Object.entries(args)) {
    body["p_" + key] = value;
  }
  return body;
}

// The coach passcode first, when one was typed: it answers everything the
// group passcode does, and the data says which role it opened.
async function withCodes(name, args, wantBytes) {
  const order = [codes.coach, codes.group].filter((c) => c !== "");
  let last = new Error("bad_passcode");
  for (const code of order) {
    try {
      return await call(name, params(args, code), wantBytes);
    } catch (e) {
      last = e;
      if (e.message !== "bad_passcode") {
        throw e;
      }
    }
  }
  throw last;
}

// The names to tap, before anyone is signed in: names and ids, no data.
export function fetchNames() {
  return call("get_names", {}, false);
}

export function fetchState() {
  return withCodes("get_state", {}, false);
}

export function run(request) {
  return withCodes(request.rpc, request.args, false);
}

export function fetchAudio(id) {
  return withCodes("get_note_audio", { id }, true);
}

// Bytes travel as base64 text and are decoded in the database.
async function toBase64(blob) {
  const text = await new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result);
    reader.onerror = () => reject(new Error("server"));
    reader.readAsDataURL(blob);
  });
  return text.split(",")[1];
}

export async function addNote(args, blob) {
  return withCodes("add_note", { ...args, mime: blob.type, audio: await toBase64(blob) }, false);
}

// An athlete's picture shows before anyone is signed in, so reading it needs no sign-in.
export function fetchAvatar(id) {
  return call("get_avatar", { p_athlete: id }, true);
}

export async function setAvatar(args, file) {
  const mime = file.type || "image/jpeg";
  return withCodes("set_avatar", { ...args, mime, image: await toBase64(file) }, false);
}
