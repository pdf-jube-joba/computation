#!/usr/bin/env python3
"""
Generate dist/ from docs/ and wasm_bundle/, then overlay models/ into dist/src/.
"""

from __future__ import annotations

import argparse
import filecmp
import os
import shutil
from pathlib import Path
from typing import Optional

REPO_ROOT = Path(__file__).resolve().parent
DOCS_DIR = REPO_ROOT / "docs"
DIST_DIR = REPO_ROOT / "dist"
DIST_SRC_DIR = DIST_DIR / "src"
ROOT_WASM_BUNDLE_DIR = REPO_ROOT / "wasm_bundle"
DIST_WASM_BUNDLE_DIR = DIST_SRC_DIR / "_assets" / "wasm_bundle"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate dist/ from docs/, models/, and wasm_bundle/."
    )
    parser.add_argument(
        "--mode",
        choices=("clean", "incremental"),
        default="incremental",
        help="clean recreates dist/ from scratch; incremental only adds/updates files.",
    )
    return parser.parse_args()

def copy_file_if_changed(src: Path, dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.exists() and filecmp.cmp(src, dest, shallow=False):
        return
    if dest.suffix == ".md" and dest.exists():
        dest.write_text(src.read_text(encoding="utf-8"), encoding="utf-8")
        return
    temp_path = dest.with_name(f".{dest.name}.tmp")
    shutil.copyfile(src, temp_path)
    os.replace(temp_path, dest)


def write_text_if_changed(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    if path.exists() and path.read_text(encoding="utf-8") == text:
        return
    temp_path = path.with_name(f".{path.name}.tmp")
    temp_path.write_text(text, encoding="utf-8")
    os.replace(temp_path, path)


def sync_tree(src_root: Path, dest_root: Path) -> None:
    dest_root.mkdir(parents=True, exist_ok=True)

    for src_path in src_root.rglob("*"):
        rel = src_path.relative_to(src_root)
        dest_path = dest_root / rel
        if src_path.is_dir():
            dest_path.mkdir(parents=True, exist_ok=True)
            continue
        copy_file_if_changed(src_path, dest_path)


def sync_selected_files(src_root: Path, dest_root: Path, relative_paths: list[Path]) -> None:
    dest_root.mkdir(parents=True, exist_ok=True)
    for rel in relative_paths:
        copy_file_if_changed(src_root / rel, dest_root / rel)


def collect_md_rel_paths(src_root: Path) -> list[Path]:
    rel_paths: list[Path] = []
    for path in src_root.rglob("*.md"):
        if path.is_file():
            rel_paths.append(path.relative_to(src_root))
    return rel_paths


def copy_md_tree(src_root: Path, dest_root: Path) -> list[str]:
    rel_paths: list[str] = []
    rel_md_paths = collect_md_rel_paths(src_root)
    sync_selected_files(src_root, dest_root, rel_md_paths)
    for rel in rel_md_paths:
        rel_paths.append(rel.as_posix())
    return rel_paths


def write_navigation_json(
    dest_root: Path,
    rel_paths: list[str],
    parent_item: Optional[dict[str, str]] = None,
) -> None:
    items = []
    if parent_item is not None:
        items.append(parent_item)
    for rel in rel_paths:
        href = rel[:-3] + ".html" if rel.endswith(".md") else rel
        items.append({"title": rel, "href": href})
    nav_path = dest_root / "navigation.json"
    nav_text = "{\n  \"items\": [\n"
    for i, item in enumerate(items):
        comma = "," if i + 1 < len(items) else ""
        nav_text += f'    {{"title": "{item["title"]}", "href": "{item["href"]}"}}{comma}\n'
    nav_text += "  ]\n}\n"
    write_text_if_changed(nav_path, nav_text)


def main() -> int:
    args = parse_args()

    if args.mode == "clean" and DIST_DIR.exists():
        shutil.rmtree(DIST_DIR)

    sync_tree(DOCS_DIR, DIST_DIR)

    if ROOT_WASM_BUNDLE_DIR.exists():
        sync_tree(ROOT_WASM_BUNDLE_DIR, DIST_WASM_BUNDLE_DIR)

    models_path = REPO_ROOT / "models"
    dist_models = DIST_SRC_DIR / "models"

    models_md = copy_md_tree(models_path, dist_models)
    write_navigation_json(
        dist_models,
        models_md,
        parent_item={"title": "Up", "href": "../"},
    )

    root_readme = REPO_ROOT / "README.md"
    if root_readme.exists():
        copy_file_if_changed(root_readme, DIST_SRC_DIR / "README.md")
    root_todo = REPO_ROOT / "TODO.md"
    if root_todo.exists():
        copy_file_if_changed(root_todo, DIST_SRC_DIR / "TODO.md")

    summary_path = DIST_SRC_DIR / "SUMMARY.md"
    md_files = []
    for path in DIST_SRC_DIR.rglob("*.md"):
        if not path.is_file():
            continue
        rel = path.relative_to(DIST_SRC_DIR).as_posix()
        md_files.append(rel)

    md_files.sort()

    lines = ["[README](README.md)"]
    if root_todo.exists():
        lines.append("[TODO](TODO.md)")
    for rel in md_files:
        if rel in ("README.md", "SUMMARY.md", "TODO.md"):
            continue
        lines.append(f"- [{rel}]({rel})")

    write_text_if_changed(summary_path, "\n".join(lines) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
