// The one page file that may touch browser storage (kits/saves, D17 changed).
// It keeps text under a name and hands it back; it never looks inside. What a
// kept choice means, and what to do with one that cannot be read, is the
// core's to decide.

const prefix = "tran4vert:";

export function read(name) {
  try {
    return localStorage.getItem(prefix + name) || "";
  } catch (e) {
    return "";
  }
}

export function write(name, text) {
  try {
    localStorage.setItem(prefix + name, text);
    return true;
  } catch (e) {
    return false;
  }
}

export function remove(name) {
  try {
    localStorage.removeItem(prefix + name);
  } catch (e) {
    // Nothing was kept, so nothing needs removing.
  }
}
