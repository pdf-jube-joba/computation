#!/usr/bin/env python3
"""
Build and serve per-feature WASM bundles for web_builder using wasm-pack.
Also generates HTML scaffolding that loads the right bundle based on
data-model attributes and per-feature view.js files.

Usage:
  python build_and_serve.py build
  python build_and_serve.py serve --port 8080
  python build_and_serve.py build --features model_a model_b --release
"""

import argparse
import shutil
import subprocess
import sys
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Iterable

FEATURES = ["model_b"]
PLAYGROUND_DIR = Path(__file__).resolve().parent
WEB_BUILDER_DIR = PLAYGROUND_DIR.parent / "web_builder"
ASSETS_DIR = PLAYGROUND_DIR / "assets"


def main() -> None:
    parser = argparse.ArgumentParser(description="Build/serve web_builder WASM bundles for each feature.")
    parser.add_argument("command", choices=["build", "serve"], nargs="?", default="build")
    parser.add_argument("--features", nargs="+", default=FEATURES, help="Features to build (default: %(default)s)")
    parser.add_argument("--port", type=int, default=8080, help="Port for `serve` (default: %(default)s)")
    parser.add_argument("--release", action="store_true", help="Pass --release to wasm-pack")
    args = parser.parse_args()

    features = args.features
    ensure_wasm_pack()
    build_all(features, release=args.release)

    if args.command == "serve":
        serve(args.port)


def ensure_wasm_pack() -> None:
    if shutil.which("wasm-pack") is None:
        sys.exit("wasm-pack is required but not found on PATH. Install it via `cargo install wasm-pack` or from https://rustwasm.github.io/wasm-pack/installer/ .")


def build_all(features: Iterable[str], release: bool) -> None:
    ASSETS_DIR.mkdir(parents=True, exist_ok=True)

    write_loader()

    for feature in features:
        build_feature(feature, release=release)
        ensure_view_js(feature)
        write_feature_page(feature)

    write_index(features)


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


def write_index(features: Iterable[str]) -> None:
    links = "\n".join(f'<li><a href="/{feature}/">{feature}</a></li>' for feature in features)
    html = f"""<!doctype html>
<meta charset="utf-8">
<title>Playgrounds</title>
<h1>Model playgrounds</h1>
<p>Bundles built from web_builder for each feature.</p>
<ul>
{links}
</ul>
"""
    (ASSETS_DIR / "index.html").write_text(html, encoding="utf-8")


def write_feature_page(feature: str) -> None:
    html = f"""<!doctype html>
<meta charset="utf-8">
<title>{feature} playground</title>
<h1>{feature}</h1>
<div data-model="{feature}">
  <canvas width="640" height="480"></canvas>
</div>
<script type="module">
  import {{ boot }} from "../loader.js";
  boot(document.querySelector('[data-model]'));
</script>
"""
    (ASSETS_DIR / feature / "index.html").write_text(html, encoding="utf-8")


def write_loader() -> None:
    """Shared script that loads wasm + view.js based on data-model attribute."""
    js = r"""export async function boot(container) {
        if (!container) throw new Error("container not found");
        const model = container.dataset.model;
        if (!model) throw new Error("data-model attribute missing");

        const wasmModule = await import(new URL(`./${model}/${model}.js`, import.meta.url));
        await wasmModule.default();

        const view = await import(new URL(`./${model}/view.js`, import.meta.url));
        if (typeof view.render === "function") {
          view.render(container, wasmModule);
        } else {
          console.warn("view.js missing render(container, wasmExports)");
        }
        }
        """
    (ASSETS_DIR / "loader.js").write_text(js, encoding="utf-8")


def ensure_view_js(feature: str) -> None:
    """Create a stub view.js for a feature if none exists."""
    view_path = ASSETS_DIR / feature / "view.js"
    if view_path.exists():
        return
    stub = f"""// Customize rendering for {feature} here.
export function render(container, wasm) {{
  const canvas = container.querySelector("canvas");
  const ctx = canvas.getContext("2d");
  ctx.fillStyle = "#101010";
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  ctx.fillStyle = "#21c55d";
  ctx.font = "18px sans-serif";
  ctx.fillText(`model: {feature}`, 16, 28);
  ctx.fillText("wasm exports available in console", 16, 52);
  console.log("{feature} wasm exports", wasm);
}}
"""
    view_path.write_text(stub, encoding="utf-8")


def serve(port: int) -> None:
    handler = partial(SimpleHTTPRequestHandler, directory=str(ASSETS_DIR))
    server = ThreadingHTTPServer(("127.0.0.1", port), handler)
    print(f"[serve] http://127.0.0.1:{port}/ (serving {ASSETS_DIR})")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n[serve] stopped")


def run(cmd: list[str], cwd: Path) -> None:
    result = subprocess.run(cmd, cwd=cwd)
    if result.returncode != 0:
        sys.exit(result.returncode)


if __name__ == "__main__":
    main()
