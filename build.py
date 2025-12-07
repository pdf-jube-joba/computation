#!/usr/bin/env python3
"""
Build `web_builder` WASM bundle using `wasm-pack`.
And locates the built files in `assets/`.
This files doen not serve any HTML or JS files (just builds the WASM bundle).
"""

import shutil
import subprocess
import sys
from pathlib import Path

WORKSPACE_DIR = Path(__file__).resolve().parent
WEB_BUILDER_DIR = WORKSPACE_DIR / "web_builder"
ASSETS_DIR = WORKSPACE_DIR / "assets" / "wasm_bundle"
FEATURES: list[str] = ["example"]

def ensure_wasm_pack() -> None:
    if shutil.which("wasm-pack") is None:
        sys.exit("wasm-pack is required but not found on PATH. Install it via `cargo install wasm-pack` or from https://rustwasm.github.io/wasm-pack/installer/ .")

def build_wasm(feature: str, release: bool) -> None:
    cmd = [
        "wasm-pack",
        "build",
        "--out-name",
        feature,
        "--target",
        "web",
        "--mode",
        "no-install",
        "--no-typescript",
        "--no-pack",
    ]

    if release:
        cmd.insert(2, "--release")

    cmd.extend(["--features", feature])

    print(f"[build] feature={feature}")
    run(cmd, cwd=WEB_BUILDER_DIR)

def run(cmd: list[str], cwd: Path) -> None:
    result = subprocess.run(cmd, cwd=cwd)
    if result.returncode != 0:
        sys.exit(result.returncode)

def rename_and_move(label: str) -> None:
    src_dir = WEB_BUILDER_DIR / "pkg"
    dest_dir = ASSETS_DIR
    dest_dir.mkdir(parents=True, exist_ok=True)

    wasm_src = src_dir / f"{label}_bg.wasm"
    wasm_dest = dest_dir / f"{label}_bg.wasm"
    js_src = src_dir / f"{label}.js"
    js_dest = dest_dir / f"{label}.js"

    shutil.copy2(wasm_src, wasm_dest)
    shutil.copy2(js_src, js_dest)

def main() -> None:
    ensure_wasm_pack()

    # Reset output dir once before building all targets so artifacts for each
    # feature accumulate instead of being overwritten on every iteration.
    if ASSETS_DIR.exists():
        shutil.rmtree(ASSETS_DIR)
    ASSETS_DIR.mkdir(parents=True, exist_ok=True)

    for feature in FEATURES:
        build_wasm(feature, release=True)
        rename_and_move(feature)

if __name__ == "__main__":
    main()
