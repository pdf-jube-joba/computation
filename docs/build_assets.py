#!/usr/bin/env python3
"""
mdBook preprocessor:
- `supports <renderer>` -> prints "true"
- otherwise: read book JSON from stdin, build workspace wasm bundles, and echo the JSON.
All logs go to stderr; stdout is JSON only.
"""

import json
import shutil
import subprocess
import sys
from pathlib import Path

BOOK_DIR = Path(__file__).resolve().parent
WORKSPACE_DIR = BOOK_DIR.parent
MODELS_DIR = WORKSPACE_DIR / "models"
WASM_ASSETS_DIR = BOOK_DIR / "src" / "_assets" / "wasm_bundle"

def ensure_wasm_bindgen() -> None:
    if shutil.which("wasm-bindgen") is None:
        sys.exit(
            "wasm-bindgen is required but not found on PATH. "
            "Install it via `cargo install wasm-bindgen-cli`."
        )

def find_packages(root_dir: Path) -> list[tuple[str, Path]]:
    packages: list[tuple[str, Path]] = []
    if not root_dir.exists():
        return packages
    for cargo_toml in root_dir.glob("*/Cargo.toml"):
        name = read_package_name(cargo_toml)
        if name:
            packages.append((name, cargo_toml.parent))
    return packages

def read_package_name(cargo_toml: Path) -> str | None:
    in_package = False
    for line in cargo_toml.read_text().splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if stripped == "[package]":
            in_package = True
            continue
        if in_package and stripped.startswith("["):
            break
        if in_package and stripped.startswith("name"):
            _, value = stripped.split("=", 1)
            return value.strip().strip('"')
    return None

def read_bins(cargo_toml: Path) -> list[dict]:
    bins: list[dict] = []
    in_bin = False
    current: dict = {}
    for line in cargo_toml.read_text().splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        if stripped == "[[bin]]":
            if in_bin and current:
                bins.append(current)
            current = {}
            in_bin = True
            continue
        if stripped.startswith("["):
            if in_bin and current:
                bins.append(current)
            in_bin = False
            continue
        if in_bin and "=" in stripped:
            key, value = stripped.split("=", 1)
            key = key.strip()
            value = value.strip().strip('"')
            if key in ("name", "path"):
                current[key] = value
    if in_bin and current:
        bins.append(current)
    return bins

def resolve_bin_name(crate_dir: Path, package_name: str) -> str | None:
    if (crate_dir / "src" / "main.rs").exists():
        return package_name
    cargo_toml = crate_dir / "Cargo.toml"
    if not cargo_toml.exists():
        return None
    for bin_table in read_bins(cargo_toml):
        name = bin_table.get("name")
        path = bin_table.get("path")
        if not name:
            continue
        if path:
            if (crate_dir / path).exists():
                return name
            continue
        if (crate_dir / "src" / "bin" / f"{name}.rs").exists():
            return name
    return None

def run(cmd: list[str], cwd: Path | None = None) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, cwd=cwd)

def build_model_wasm(package_name: str, crate_dir: Path, release: bool) -> bool:
    bin_name = resolve_bin_name(crate_dir, package_name)
    if not bin_name:
        print(f"[skip] {package_name}: no web entry (bin target not found)", file=sys.stderr)
        return False

    cmd = ["cargo", "build", "--package", package_name, "--target", "wasm32-unknown-unknown", "--bin", bin_name]
    if release:
        cmd.append("--release")
    print(f"[build] package={package_name} bin={bin_name}", file=sys.stderr)
    result = run(cmd, cwd=WORKSPACE_DIR)
    if result.returncode != 0:
        print(f"[error] cargo build failed for {package_name} (bin={bin_name})", file=sys.stderr)
        return False

    profile = "release" if release else "debug"
    wasm_path = WORKSPACE_DIR / "target" / "wasm32-unknown-unknown" / profile / f"{bin_name}.wasm"
    if not wasm_path.exists():
        print(f"[error] wasm output missing for {package_name}: {wasm_path}", file=sys.stderr)
        return False

    WASM_ASSETS_DIR.mkdir(parents=True, exist_ok=True)
    bindgen_cmd = [
        "wasm-bindgen",
        "--target",
        "web",
        "--out-dir",
        str(WASM_ASSETS_DIR),
        "--no-typescript",
        "--out-name",
        package_name,
        str(wasm_path),
    ]
    print(f"[bindgen] package={package_name} -> {WASM_ASSETS_DIR}", file=sys.stderr)
    result = run(bindgen_cmd)
    if result.returncode != 0:
        print(f"[error] wasm-bindgen failed for {package_name}", file=sys.stderr)
        return False
    return True

# this function does nothing, but we'll keep this code.
def process_item(item):
    # print to stderr for debugging
    if 'Chapter' not in item:
        return

    chapter = item['Chapter']
    # Adjust the asset path based on the chapter depth so nested chapters resolve correctly
    name = chapter['name']
    print(f'Processing chapter {name}', file=sys.stderr)

    # draft chapters does not have a path
    if chapter.get('path') is None:
        print(f'  Warning: Chapter {name} has no path', file=sys.stderr)
        return

    for sub in chapter.get('sub_items', []):
        process_item(sub)

def preprocess() -> None:
    context, book = json.load(sys.stdin)
    build_flag = context["config"]["preprocessor"]["build-assets"]["build"]
    release_flag = context["config"]["preprocessor"]["build-assets"]["release"]
    print(f"[config] build={build_flag} release={release_flag}", file=sys.stderr)
    if build_flag:
        ensure_wasm_bindgen()
        if WASM_ASSETS_DIR.exists():
            shutil.rmtree(WASM_ASSETS_DIR)
        WASM_ASSETS_DIR.mkdir(parents=True, exist_ok=True)

        for package_name, crate_dir in find_packages(MODELS_DIR):
            build_model_wasm(package_name, crate_dir, release=release_flag)

    for top in book['items']:
        process_item(top)
    json.dump(book, sys.stdout)

def main() -> None:
    if len(sys.argv) > 1 and sys.argv[1] == "supports":
        renderer = sys.argv[2] if len(sys.argv) > 2 else "html"
        print("true" if renderer == "html" else "false")
        return
    preprocess()

if __name__ == "__main__":
    main()
