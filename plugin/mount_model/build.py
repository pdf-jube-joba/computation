#!/usr/bin/env python3
"""
Build wasm component artifacts for model binaries.

Flow (wasm-only):
1. cargo build --target wasm32-unknown-unknown
2. wasm-tools component new
3. jco transpile

Output directory must be provided via WORKSPACE_FS_OUTPUT_DIRECTORY.
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPT_DIR.parent.parent
MODELS_DIR = REPO_ROOT / "models"
TARGET_DIR = REPO_ROOT / "target" / "wasm32-unknown-unknown"
STATIC_FILES = ("renderer.js", "script.js", "style.css")


def run(cmd: list[str], cwd: Path | None = None) -> None:
    print("[run]", " ".join(cmd))
    result = subprocess.run(cmd, cwd=cwd)
    if result.returncode != 0:
        raise RuntimeError(f"command failed ({result.returncode}): {' '.join(cmd)}")


def ensure_tools() -> None:
    required = ("cargo", "wasm-tools", "npx")
    missing = [tool for tool in required if shutil.which(tool) is None]
    if missing:
        raise RuntimeError(f"missing required tools: {', '.join(missing)}")
    jco_check = subprocess.run(
        ["npx", "--yes", "jco", "--version"],
        cwd=REPO_ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    if jco_check.returncode != 0:
        raise RuntimeError("`npx jco` is not available; install Node/npm and ensure npx can fetch jco")


def read_package_name(cargo_toml: Path) -> str | None:
    in_package = False
    for line in cargo_toml.read_text(encoding="utf-8").splitlines():
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


def read_bins(cargo_toml: Path) -> list[dict[str, str]]:
    bins: list[dict[str, str]] = []
    in_bin = False
    current: dict[str, str] = {}
    for line in cargo_toml.read_text(encoding="utf-8").splitlines():
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


def resolve_bin_names(crate_dir: Path, package_name: str) -> list[str]:
    bin_names: list[str] = []
    seen: set[str] = set()

    def push(name: str) -> None:
        if name not in seen:
            seen.add(name)
            bin_names.append(name)

    if (crate_dir / "src" / "main.rs").exists():
        push(package_name)

    cargo_toml = crate_dir / "Cargo.toml"
    if not cargo_toml.exists():
        return bin_names

    for table in read_bins(cargo_toml):
        name = table.get("name")
        path = table.get("path")
        if not name:
            continue
        if path:
            if (crate_dir / path).exists():
                push(name)
            continue
        if (crate_dir / "src" / "bin" / f"{name}.rs").exists():
            push(name)
    return bin_names


def find_packages(root_dir: Path) -> list[tuple[str, Path]]:
    packages: list[tuple[str, Path]] = []
    if not root_dir.exists():
        return packages
    for cargo_toml in sorted(root_dir.glob("*/Cargo.toml")):
        name = read_package_name(cargo_toml)
        if name:
            packages.append((name, cargo_toml.parent))
    return packages


def is_up_to_date(output: Path, inputs: list[Path]) -> bool:
    if not output.exists():
        return False
    out_mtime = output.stat().st_mtime
    for path in inputs:
        if not path.exists():
            return False
        if path.stat().st_mtime > out_mtime:
            return False
    return True


def build_bin(
    package_name: str,
    bin_name: str,
    profile: str,
    release: bool,
    out_names: list[str],
    output_dir: Path,
) -> None:
    build_cmd = [
        "cargo",
        "build",
        "--package",
        package_name,
        "--target",
        "wasm32-unknown-unknown",
        "--bin",
        bin_name,
    ]
    if release:
        build_cmd.append("--release")
    run(build_cmd, cwd=REPO_ROOT)

    core_wasm = TARGET_DIR / profile / f"{bin_name}.wasm"
    if not core_wasm.exists():
        raise RuntimeError(f"missing wasm output: {core_wasm}")

    output_dir.mkdir(parents=True, exist_ok=True)
    for out_name in out_names:
        component_wasm = output_dir / f"{out_name}.component.wasm"
        jco_js = output_dir / f"{out_name}.js"
        jco_core = output_dir / f"{out_name}.core.wasm"

        if not is_up_to_date(component_wasm, [core_wasm]):
            run(
                [
                    "wasm-tools",
                    "component",
                    "new",
                    str(core_wasm),
                    "-o",
                    str(component_wasm),
                ],
                cwd=REPO_ROOT,
            )
        else:
            print(f"[skip] component up-to-date: {component_wasm.name}")

        if not (is_up_to_date(jco_js, [component_wasm]) and is_up_to_date(jco_core, [component_wasm])):
            run(
                [   "jco",
                    "transpile",
                    str(component_wasm),
                    "-o",
                    str(output_dir),
                    "--name",
                    out_name,
                    "--no-typescript",
                    "-q",
                ],
                cwd=REPO_ROOT,
            )
        else:
            print(f"[skip] jco output up-to-date: {out_name}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Build wasm bundle with component + jco")
    parser.add_argument(
        "--package",
        action="append",
        dest="packages",
        default=[],
        help="Limit to specific package name(s). Repeatable.",
    )
    parser.add_argument(
        "--release",
        action="store_true",
        help="Use cargo --release",
    )
    parser.add_argument(
        "--clean",
        action="store_true",
        help="Remove root wasm_bundle before generation",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    ensure_tools()
    output_dir_env = os.environ.get("WORKSPACE_FS_OUTPUT_DIRECTORY")
    if not output_dir_env:
        raise RuntimeError("WORKSPACE_FS_OUTPUT_DIRECTORY is required")
    output_dir = Path(output_dir_env)

    if args.clean and output_dir.exists():
        shutil.rmtree(output_dir)

    output_dir.mkdir(parents=True, exist_ok=True)
    copy_static_assets(output_dir)

    profile = "release" if args.release else "debug"

    package_filter = set(args.packages)
    packages = find_packages(MODELS_DIR)
    if package_filter:
        packages = [pkg for pkg in packages if pkg[0] in package_filter]
        missing = sorted(package_filter - {name for name, _ in packages})
        if missing:
            raise RuntimeError(f"unknown package(s): {', '.join(missing)}")

    if not packages:
        print("[warn] no packages to build")
        return 0

    for package_name, crate_dir in packages:
        bin_names = resolve_bin_names(crate_dir, package_name)
        if not bin_names:
            print(f"[skip] {package_name}: no bin targets")
            continue
        for bin_name in bin_names:
            out_names = [bin_name]
            if len(bin_names) == 1 and package_name != bin_name:
                out_names.append(package_name)
            print(f"[build] package={package_name} bin={bin_name} out={','.join(out_names)}")
            build_bin(
                package_name,
                bin_name,
                profile=profile,
                release=args.release,
                out_names=out_names,
                output_dir=output_dir,
            )

    return 0


def copy_static_assets(output_dir: Path) -> None:
    for filename in STATIC_FILES:
        shutil.copy2(SCRIPT_DIR / filename, output_dir / filename)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except RuntimeError as exc:
        print(f"[error] {exc}", file=sys.stderr)
        raise SystemExit(1)
