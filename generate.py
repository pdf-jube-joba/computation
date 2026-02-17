#!/usr/bin/env python3
"""
Generate dist/ from docs/, then overlay models/ into dist/src/.
"""

from __future__ import annotations

import shutil
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent
DOCS_DIR = REPO_ROOT / "docs"
DIST_DIR = REPO_ROOT / "dist"
BOOK_SRC_DIR = DOCS_DIR / "src"
DIST_SRC_DIR = DIST_DIR / "src"


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
    if DIST_DIR.exists():
        shutil.rmtree(DIST_DIR)
    shutil.copytree(DOCS_DIR, DIST_DIR)

    models_path = REPO_ROOT / "models"
    dist_models = DIST_SRC_DIR / "models"

    models_md = copy_md_tree(models_path, dist_models)
    write_navigation_json(dist_models, models_md)

    root_readme = REPO_ROOT / "README.md"
    if root_readme.exists():
        shutil.copy2(root_readme, DIST_SRC_DIR / "README.md")

    summary_path = DIST_SRC_DIR / "SUMMARY.md"
    md_files = []
    for path in DIST_SRC_DIR.rglob("*.md"):
        if not path.is_file():
            continue
        rel = path.relative_to(DIST_SRC_DIR).as_posix()
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
