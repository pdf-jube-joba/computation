#!/usr/bin/env python3
"""
Build WASM bundles for each model crate and copy them into assets/wasm_bundle.
"""

import shutil
import subprocess
import sys
from pathlib import Path

WORKSPACE_DIR = Path(__file__).resolve().parent
MODELS_DIR = WORKSPACE_DIR / "models"
ASSETS_DIR = WORKSPACE_DIR / "assets" / "wasm_bundle"
RELEASE = False

def ensure_wasm_bindgen() -> None:
    if shutil.which("wasm-bindgen") is None:
        sys.exit(
            "wasm-bindgen is required but not found on PATH. "
            "Install it via `cargo install wasm-bindgen-cli`."
        )


def find_model_packages() -> list[tuple[str, Path]]:
    packages: list[tuple[str, Path]] = []
    if not MODELS_DIR.exists():
        return packages
    for cargo_toml in MODELS_DIR.glob("*/Cargo.toml"):
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


def resolve_bin_name(crate_dir: Path, package_name: str) -> str | None:
    if (crate_dir / "src" / "main.rs").exists():
        return package_name
    return None


def run(cmd: list[str], cwd: Path | None = None) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, cwd=cwd)


def build_model_wasm(package_name: str, crate_dir: Path, release: bool) -> bool:
    bin_name = resolve_bin_name(crate_dir, package_name)
    if not bin_name:
        print(f"[skip] {package_name}: no web entry (main.rs or bin/web.rs not found)")
        return False

    cmd = ["cargo", "build", "--package", package_name, "--target", "wasm32-unknown-unknown", "--bin", bin_name]
    if release:
        cmd.append("--release")
    print(f"[build] package={package_name} bin={bin_name}")
    result = run(cmd, cwd=WORKSPACE_DIR)
    if result.returncode != 0:
        print(f"[error] cargo build failed for {package_name} (bin={bin_name})")
        return False

    profile = "release" if release else "debug"
    wasm_path = WORKSPACE_DIR / "target" / "wasm32-unknown-unknown" / profile / f"{bin_name}.wasm"
    if not wasm_path.exists():
        print(f"[error] wasm output missing for {package_name}: {wasm_path}")
        return False

    ASSETS_DIR.mkdir(parents=True, exist_ok=True)
    bindgen_cmd = [
        "wasm-bindgen",
        "--target",
        "web",
        "--out-dir",
        str(ASSETS_DIR),
        "--no-typescript",
        "--out-name",
        package_name,
        str(wasm_path),
    ]
    print(f"[bindgen] package={package_name} -> {ASSETS_DIR}")
    result = run(bindgen_cmd)
    if result.returncode != 0:
        print(f"[error] wasm-bindgen failed for {package_name}")
        return False
    return True

def main() -> None:
    ensure_wasm_bindgen()

    # Reset output dir once before building all targets so artifacts for each
    # feature accumulate instead of being overwritten on every iteration.
    if ASSETS_DIR.exists():
        shutil.rmtree(ASSETS_DIR)
    ASSETS_DIR.mkdir(parents=True, exist_ok=True)

    for package_name, crate_dir in find_model_packages():
        build_model_wasm(package_name, crate_dir, release=RELEASE)

if __name__ == "__main__":
    main()
