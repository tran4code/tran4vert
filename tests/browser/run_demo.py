#!/usr/bin/env python3
"""Runs the demo interactions on a copy of dist/ with the project address cleared.

The page in dist/ points at the person's database, so its first screen is the
passcode screen. The demo interactions need the invented program and made-up
athletes, so this copies dist/, makes the same edit `tools/set_project.py --clear`
makes, and runs check.py on the copy (BUILDER_DIST).
"""
import importlib.util
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]


def main():
    spec = importlib.util.spec_from_file_location("set_project", ROOT / "tools/set_project.py")
    tools = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(tools)
    with tempfile.TemporaryDirectory(prefix="tran4vert-demo-") as work:
        site = pathlib.Path(work) / "dist"
        shutil.copytree(ROOT / "dist", site)
        tools.apply(site, "", "", web=".")
        env = dict(os.environ, BUILDER_DIST=str(site))
        sys.exit(subprocess.run([sys.executable, str(ROOT / "tests/browser/check.py")], env=env).returncode)


if __name__ == "__main__":
    main()
