#!/usr/bin/env python3
"""
Generate docs/src/SUMMARY.md by recursively listing markdown files,
then copy the repo root README.md to docs/src/README.md.
"""

from __future__ import annotations

import shutil
from pathlib import Path

DOCS_DIR = Path(__file__).resolve().parent
REPO_ROOT = DOCS_DIR.parent
BOOK_SRC_DIR = DOCS_DIR / "src"


def copy_md_tree(src_root: Path, dest_root: Path) -> list[str]:
    rel_paths: list[str] = []
    for path in src_root.rglob("*.md"):
        if not path.is_file():
            continue
        rel = path.relative_to(src_root)
        dest_path = dest_root / rel
        dest_path.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, dest_path)
        rel_paths.append(rel.as_posix())
    return rel_paths


def write_navigation_json(dest_root: Path, rel_paths: list[str]) -> None:
    items = []
    for rel in rel_paths:
        href = rel[:-3] + ".html" if rel.endswith(".md") else rel
        items.append({"title": rel, "href": href})
    nav_path = dest_root / "navigation.json"
    nav_text = "{\n  \"items\": [\n"
    for i, item in enumerate(items):
        comma = "," if i + 1 < len(items) else ""
        nav_text += f'    {{"title": "{item["title"]}", "href": "{item["href"]}"}}{comma}\n'
    nav_text += "  ]\n}\n"
    nav_path.write_text(nav_text, encoding="utf-8")


def main() -> int:
    # copying models/ and compilers/
    models_path = REPO_ROOT / "models"
    compilers_path = REPO_ROOT / "compilers"
    docs_models = BOOK_SRC_DIR / "models"
    docs_compilers = BOOK_SRC_DIR / "compilers"

    models_md = copy_md_tree(models_path, docs_models)
    compilers_md = copy_md_tree(compilers_path, docs_compilers)
    write_navigation_json(docs_models, models_md)
    write_navigation_json(docs_compilers, compilers_md)

    # copying readme.md to src
    root_readme = REPO_ROOT / "README.md"
    if root_readme.exists():
        shutil.copy2(root_readme, BOOK_SRC_DIR / "README.md")

    # generate entire
    summary_path = BOOK_SRC_DIR / "SUMMARY.md"

    md_files = []
    for path in BOOK_SRC_DIR.rglob("*.md"):
        if not path.is_file():
            continue
        rel = path.relative_to(BOOK_SRC_DIR).as_posix()
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
