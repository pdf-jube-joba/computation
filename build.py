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
from typing import Iterable

WORKSPACE_DIR = Path(__file__).resolve().parent
WEB_BUILDER_DIR = WORKSPACE_DIR / "web_builder"
WEB_COMPILER_DIR = WORKSPACE_DIR / "web_compiler"
ASSETS_DIR = WORKSPACE_DIR / "assets" / "wasm_bundle"
RELEASE = False

def ensure_wasm_pack() -> None:
    if shutil.which("wasm-pack") is None:
        sys.exit("wasm-pack is required but not found on PATH. Install it via `cargo install wasm-pack` or from https://rustwasm.github.io/wasm-pack/installer/ .")


def load_features(cargo_dir: Path) -> list[str]:
    """Read feature keys from the `[features]` section of Cargo.toml without full TOML parsing."""
    features: list[str] = []
    in_features = False
    cargo_toml = (cargo_dir / "Cargo.toml").read_text().splitlines()
    for line in cargo_toml:
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if stripped == "[features]":
            in_features = True
            continue
        if in_features and stripped.startswith("["):
            break  # reached the next section
        if not in_features:
            continue
        if "=" in stripped:
            name = stripped.split("=", 1)[0].strip()
            if name and name not in ("default",):
                features.append(name)
    return features


def build_wasm(crate_dir: Path, feature: str, release: bool) -> None:
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
    else:
        cmd.extend(["--no-opt"])

    cmd.extend(["--features", feature])

    print(f"[build] crate={crate_dir.name} feature={feature}")
    run(cmd, cwd=crate_dir)


def run(cmd: list[str], cwd: Path) -> None:
    result = subprocess.run(cmd, cwd=cwd)
    if result.returncode != 0:
        sys.exit(result.returncode)


def rename_and_move(crate_dir: Path, label: str) -> None:
    src_dir = crate_dir / "pkg"
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
    builder_features = [f for f in load_features(WEB_BUILDER_DIR)]
    compiler_features = [f for f in load_features(WEB_COMPILER_DIR)]

    # Reset output dir once before building all targets so artifacts for each
    # feature accumulate instead of being overwritten on every iteration.
    if ASSETS_DIR.exists():
        shutil.rmtree(ASSETS_DIR)
    ASSETS_DIR.mkdir(parents=True, exist_ok=True)

    for feature in builder_features:
        build_wasm(WEB_BUILDER_DIR, feature, release=RELEASE)
        rename_and_move(WEB_BUILDER_DIR, feature)

    for feature in compiler_features:
        build_wasm(WEB_COMPILER_DIR, feature, release=RELEASE)
        rename_and_move(WEB_COMPILER_DIR, feature)

if __name__ == "__main__":
    main()
