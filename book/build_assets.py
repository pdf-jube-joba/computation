#!/usr/bin/env python3
"""
mdBook preprocessor:
- `supports <renderer>` -> prints "true"
- otherwise: read book JSON from stdin, run workspace build.py, copy assets into book/assets, and echo the JSON.
All logs go to stderr; stdout is JSON only.
"""

import json
import shutil
import subprocess
import sys
from pathlib import Path

BOOK_DIR = Path(__file__).resolve().parent
WORKSPACE_DIR = BOOK_DIR.parent
ASSETS_SRC = WORKSPACE_DIR / "assets"
ASSETS_DEST = BOOK_DIR / "src" / "assets"
BUILD_SCRIPT = WORKSPACE_DIR / "build.py"

def call_build_script() -> None:
    result = subprocess.run(
        [sys.executable, str(BUILD_SCRIPT)],
        cwd=WORKSPACE_DIR,
        capture_output=True,
        text=True,
    )
    if result.stdout:
        sys.stderr.write(result.stdout)
    if result.stderr:
        sys.stderr.write(result.stderr)
    if result.returncode != 0:
        sys.exit(result.returncode)


def copy_assets() -> None:
    if ASSETS_DEST.exists():
        shutil.rmtree(ASSETS_DEST)
    shutil.copytree(ASSETS_SRC, ASSETS_DEST)


def preprocess() -> None:
    context, book = json.load(sys.stdin)
    call_build_script()
    copy_assets()
    json.dump(book, sys.stdout)


def main() -> None:
    if len(sys.argv) > 1 and sys.argv[1] == "supports":
        return
    preprocess()

if __name__ == "__main__":
    main()
