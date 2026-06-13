#!/usr/bin/env python3

from __future__ import annotations

import argparse
import re
from pathlib import Path


TOP_LEVEL_PREFIX = "- "
ARCHIVE_SIZE = 50
ARCHIVE_NAME_RE = re.compile(r"archive-(\d{3})\.md$")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Archive top-level listings from docs/inbox.md into chunked files."
    )
    parser.add_argument(
        "--source",
        type=Path,
        default=Path("docs/inbox.md"),
        help="Source inbox markdown file.",
    )
    parser.add_argument(
        "--archive-dir",
        type=Path,
        default=Path("docs/inbox"),
        help="Directory where archive files are written.",
    )
    parser.add_argument(
        "--chunk-size",
        type=int,
        default=ARCHIVE_SIZE,
        help="Number of top-level listings per archive file.",
    )
    return parser.parse_args()


def split_entries(text: str) -> tuple[list[str], list[str]]:
    preamble: list[str] = []
    entries: list[list[str]] = []

    for line in text.splitlines(keepends=True):
        if line.startswith(TOP_LEVEL_PREFIX):
            entries.append([line])
            continue

        if entries:
            entries[-1].append(line)
        else:
            preamble.append(line)

    return preamble, ["".join(entry) for entry in entries]


def next_archive_index(archive_dir: Path) -> int:
    max_index = 0
    for path in archive_dir.glob("archive-*.md"):
        match = ARCHIVE_NAME_RE.fullmatch(path.name)
        if match:
            max_index = max(max_index, int(match.group(1)))
    return max_index + 1


def ensure_trailing_newline(text: str) -> str:
    if not text:
        return ""
    return text if text.endswith("\n") else f"{text}\n"


def main() -> None:
    args = parse_args()
    if args.chunk_size <= 0:
        raise SystemExit("--chunk-size must be positive")

    source_text = args.source.read_text(encoding="utf-8")
    preamble, entries = split_entries(source_text)

    archiveable_count = len(entries) // args.chunk_size * args.chunk_size
    archive_entries = entries[:archiveable_count]
    remaining_entries = entries[archiveable_count:]

    args.archive_dir.mkdir(parents=True, exist_ok=True)
    archive_index = next_archive_index(args.archive_dir)

    for offset in range(0, len(archive_entries), args.chunk_size):
        chunk = archive_entries[offset : offset + args.chunk_size]
        archive_path = args.archive_dir / f"archive-{archive_index:03}.md"
        archive_path.write_text("".join(chunk), encoding="utf-8")
        archive_index += 1

    remaining_text = "".join(preamble) + "".join(remaining_entries)
    args.source.write_text(ensure_trailing_newline(remaining_text), encoding="utf-8")

    archived_files = len(archive_entries) // args.chunk_size
    print(
        f"archived {len(archive_entries)} entries into {archived_files} files; "
        f"kept {len(remaining_entries)} entries in {args.source}"
    )


if __name__ == "__main__":
    main()
