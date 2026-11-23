#!/usr/bin/env python3
"""Writes the Supabase project's address and publishable key into the page.

    python3.13 tools/set_project.py URL KEY [--root DIR]

Edits web/net/config.js and the connect-src of web/index.html together, so the
net fence test (crates/core/tests/net_fence.rs) matches them. It refuses an
address that is not https, an address with a path or a trailing slash, and a
key that is a service-role key (a JWT whose role is not anon) or that looks like
a secret key. The publishable key is meant to be public; the passcodes and the
service-role key are not, and never go in this repository.

    python3.13 tools/set_project.py --clear     empties both again (demo mode)
"""
import base64
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[1]


def refuse(why):
    sys.exit(f"refused: {why}")


def check(url, key):
    if not re.fullmatch(r"https://[a-z0-9-]+\.supabase\.co", url):
        refuse("the address must look like https://<project>.supabase.co, with no path or trailing slash")
    if not key or len(key) < 20 or re.search(r"\s", key):
        refuse("the key is empty, short or has a space")
    if key.startswith("sb_secret_") or "service_role" in key:
        refuse("that is a secret key; give the publishable (anon) key")
    parts = key.split(".")
    if len(parts) == 3:
        try:
            pad = "=" * (-len(parts[1]) % 4)
            role = json.loads(base64.urlsafe_b64decode(parts[1] + pad)).get("role")
        except Exception:
            role = None
        if role != "anon":
            refuse(f"the key's role is {role!r}; the page needs the anon key")


def apply(root, url, key, web="web"):
    config = root / web / "net/config.js"
    text = config.read_text()
    new = re.sub(r'(url: ")[^"]*(")', lambda m: m.group(1) + url + m.group(2), text, count=1)
    new = re.sub(r'(key: ")[^"]*(")', lambda m: m.group(1) + key + m.group(2), new, count=1)
    if new == text and (url or key):
        refuse("config.js was not changed; is its shape different?")
    config.write_text(new)
    page = root / web / "index.html"
    html = page.read_text()
    pattern = r"connect-src [^;\"]*;"
    if not re.search(pattern, html):
        refuse("index.html has no connect-src")
    source = "connect-src 'self'" + (f" {url}" if url else "") + ";"
    page.write_text(re.sub(pattern, lambda m: source, html, count=1))


def main(argv):
    root = ROOT
    if "--root" in argv:
        i = argv.index("--root")
        root = pathlib.Path(argv[i + 1])
        argv = argv[:i] + argv[i + 2 :]
    if argv == ["--clear"]:
        apply(root, "", "")
        print("cleared: the page runs as a demo again")
        return
    if len(argv) != 2:
        sys.exit(__doc__)
    url, key = argv
    check(url, key)
    apply(root, url, key)
    print(f"set {url} in web/net/config.js and in the connect-src of web/index.html")


if __name__ == "__main__":
    main(sys.argv[1:])
