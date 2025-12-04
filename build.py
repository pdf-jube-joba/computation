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

FEATURES = [] # currently no per-feature builds
WORKSPACE_DIR = Path(__file__).resolve().parent
WEB_BUILDER_DIR = WORKSPACE_DIR / "web_builder"
ASSETS_DIR = WORKSPACE_DIR / "assets" / "wasm_bundle"

def ensure_wasm_pack() -> None:
    if shutil.which("wasm-pack") is None:
        sys.exit("wasm-pack is required but not found on PATH. Install it via `cargo install wasm-pack` or from https://rustwasm.github.io/wasm-pack/installer/ .")

def build_feature(feature: str, release: bool) -> None:
    out_dir = ASSETS_DIR / feature
    if out_dir.exists():
        shutil.rmtree(out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    cmd = [
        "wasm-pack",
        "build",
        "--out-dir",
        str(out_dir),
        "--out-name",
        feature,
        "--target",
        "web",
        "--mode",
        "no-install",
        "--no-typescript",
        "--no-pack",
        "--features",
        feature,
    ]
    if release:
        cmd.insert(2, "--release")

    print(f"[build] feature={feature}")
    run(cmd, cwd=WEB_BUILDER_DIR)

def run(cmd: list[str], cwd: Path) -> None:
    result = subprocess.run(cmd, cwd=cwd)
    if result.returncode != 0:
        sys.exit(result.returncode)

def rename_and_move(feature: str) -> None:
    src_dir = WEB_BUILDER_DIR / "pkg"
    dest_dir = ASSETS_DIR / feature
    if dest_dir.exists():
        shutil.rmtree(dest_dir)
    dest_dir.mkdir(parents=True, exist_ok=True)

    wasm_src = src_dir / "web_builder_bg.wasm"
    wasm_dest = dest_dir / f"{feature}_bg.wasm"
    shutil.move(str(wasm_src), str(wasm_dest))

    js_src = src_dir / "web_builder.js"
    js_dest = dest_dir / f"{feature}.js"
    shutil.move(str(js_src), str(js_dest))

def main() -> None:
    ensure_wasm_pack()

    for feature in FEATURES:
        build_feature(feature, release=True)
        rename_and_move(feature)