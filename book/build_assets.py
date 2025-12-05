#!/usr/bin/env python3
"""
Minimal mdBook preprocessor:
- `supports <renderer>` -> prints "true".
- Otherwise: read book JSON from stdin, run the workspace build, copy assets into book/assets, and echo the JSON.
"""

import json
import shutil
import subprocess
import sys
from pathlib import Path

BOOK_DIR = Path(__file__).resolve().parent
WORKSPACE_DIR = BOOK_DIR.parent
ASSETS_SRC = WORKSPACE_DIR / "assets"
ASSETS_DEST = BOOK_DIR / "assets"
BUILD_SCRIPT = WORKSPACE_DIR / "build.py"


def call_build_script() -> None:
    result = subprocess.run([sys.executable, str(BUILD_SCRIPT)])
    if result.returncode != 0:
        sys.exit(result.returncode)


def copy_assets() -> None:
    if ASSETS_DEST.exists():
        shutil.rmtree(ASSETS_DEST)
    shutil.copytree(ASSETS_SRC, ASSETS_DEST)

def main() -> None:
    call_build_script()
    copy_assets()

if __name__ == "__main__":
    main()
