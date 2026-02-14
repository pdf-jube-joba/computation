from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path

WORKSPACE_DIR = Path(__file__).resolve().parent


def _get_build_options(config) -> tuple[bool, bool]:
    extra = config.get("extra", {})
    build_cfg = extra.get("rust-build", True)
    if isinstance(build_cfg, dict):
        enabled = bool(build_cfg.get("enabled", True))
        release = bool(build_cfg.get("release", False))
        return enabled, release
    return bool(build_cfg), False


def _ensure_wasm_bindgen() -> None:
    if shutil.which("wasm-bindgen") is None:
        sys.exit(
            "wasm-bindgen is required but not found on PATH. "
            "Install it via `cargo install wasm-bindgen-cli`."
        )


def _find_packages(root_dir: Path) -> list[tuple[str, Path]]:
    packages: list[tuple[str, Path]] = []
    if not root_dir.exists():
        return packages
    for cargo_toml in root_dir.glob("*/Cargo.toml"):
        name = _read_package_name(cargo_toml)
        if name:
            packages.append((name, cargo_toml.parent))
    return packages


def _read_package_name(cargo_toml: Path) -> str | None:
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


def _resolve_bin_name(crate_dir: Path, package_name: str) -> str | None:
    if (crate_dir / "src" / "main.rs").exists():
        return package_name
    return None


def _run(cmd: list[str], cwd: Path | None = None) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, cwd=cwd)


def _build_model_wasm(
    workspace_dir: Path,
    assets_dir: Path,
    package_name: str,
    crate_dir: Path,
    release: bool,
) -> bool:
    bin_name = _resolve_bin_name(crate_dir, package_name)
    if not bin_name:
        print(f"[skip] {package_name}: no web entry (main.rs not found)")
        return False

    cmd = [
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
        cmd.append("--release")
    print(f"[build] package={package_name} bin={bin_name}")
    result = _run(cmd, cwd=workspace_dir)
    if result.returncode != 0:
        print(f"[error] cargo build failed for {package_name} (bin={bin_name})")
        return False

    profile = "release" if release else "debug"
    wasm_path = (
        workspace_dir
        / "target"
        / "wasm32-unknown-unknown"
        / profile
        / f"{bin_name}.wasm"
    )
    if not wasm_path.exists():
        print(f"[error] wasm output missing for {package_name}: {wasm_path}")
        return False

    assets_dir.mkdir(parents=True, exist_ok=True)
    bindgen_cmd = [
        "wasm-bindgen",
        "--target",
        "web",
        "--out-dir",
        str(assets_dir),
        "--no-typescript",
        "--out-name",
        package_name,
        str(wasm_path),
    ]
    print(f"[bindgen] package={package_name} -> {assets_dir}")
    result = _run(bindgen_cmd)
    if result.returncode != 0:
        print(f"[error] wasm-bindgen failed for {package_name}")
        return False
    return True


def _build_wasm_bundles(workspace_dir: Path, release: bool) -> None:
    _ensure_wasm_bindgen()
    models_dir = workspace_dir / "models"
    compilers_dir = workspace_dir / "compilers"
    assets_dir = workspace_dir / "assets" / "wasm_bundle"

    if assets_dir.exists():
        shutil.rmtree(assets_dir)
    assets_dir.mkdir(parents=True, exist_ok=True)

    for package_name, crate_dir in _find_packages(models_dir):
        _build_model_wasm(workspace_dir, assets_dir, package_name, crate_dir, release=release)

    for package_name, crate_dir in _find_packages(compilers_dir):
        _build_model_wasm(workspace_dir, assets_dir, package_name, crate_dir, release=release)


def _resolve_docs_dir(config) -> Path:
    docs_dir = config.get("docs_dir", "docs")
    return (WORKSPACE_DIR / docs_dir).resolve()


def _sync_assets(workspace_dir: Path, docs_dir: Path) -> None:
    src = workspace_dir / "assets"
    dest = docs_dir / "assets"
    if dest.exists():
        shutil.rmtree(dest)
    if src.exists():
        shutil.copytree(src, dest)


def _sync_markdown_tree(src_root: Path, dest_root: Path) -> None:
    if dest_root.exists():
        shutil.rmtree(dest_root)
    if not src_root.exists():
        return
    for path in src_root.rglob("*.md"):
        rel = path.relative_to(src_root)
        target = dest_root / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, target)


def on_pre_build(config) -> None:
    docs_dir = _resolve_docs_dir(config)
    _sync_assets(WORKSPACE_DIR, docs_dir)
    _sync_markdown_tree(WORKSPACE_DIR / "models", docs_dir / "models")
    _sync_markdown_tree(WORKSPACE_DIR / "compilers", docs_dir / "compilers")
    enabled, release = _get_build_options(config)
    if not enabled:
        print("[build] skipped (extra.build is false)")
        return
    _build_wasm_bundles(WORKSPACE_DIR, release=release)
