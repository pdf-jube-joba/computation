#!/usr/bin/env python3
"""
Generate docs/src/SUMMARY.md by recursively listing markdown files,
then copy the repo root README.md to docs/README.md.
"""

from __future__ import annotations

import shutil
from pathlib import Path


def main() -> int:
    docs_dir = Path(__file__).resolve().parent
    repo_root = docs_dir.parent
    src_dir = docs_dir / "src"
    summary_path = src_dir / "SUMMARY.md"

    root_readme = repo_root / "README.md"
    docs_readme = src_dir / "README.md"
    if root_readme.exists():
        shutil.copy2(root_readme, docs_readme)

    md_files = []
    for path in src_dir.rglob("*.md"):
        if not path.is_file():
            continue
        rel = path.relative_to(src_dir).as_posix()
        md_files.append(rel)

    md_files.sort()

    lines = ["[README](README.md)"]
    for rel in md_files:
        if rel in ("README.md", "SUMMARY.md"):
            continue
        lines.append(f"- [{rel}]({rel})")

    summary_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
