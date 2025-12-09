import argparse
import shutil
import subprocess
import sys
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

PLAYGROUND_DIR = Path(__file__).resolve().parent
WORKSPACE_DIR = PLAYGROUND_DIR.parent
# `./assets` may contains hand written files => include all of `./assets`
ASSETS_DIR = WORKSPACE_DIR / "assets"

def call_build_script() -> None:
    build_script = WORKSPACE_DIR / "build.py"
    result = subprocess.run([sys.executable, str(build_script)])
    if result.returncode != 0:
        sys.exit(result.returncode)

def pull_assets() -> None:
    # copy from `assets/` to `playground/assets/`
    dest_dir = PLAYGROUND_DIR / "assets"
    if dest_dir.exists():
        shutil.rmtree(dest_dir)
    shutil.copytree(ASSETS_DIR, dest_dir)

def serve(port: int) -> None:
    # Serve the entire playground folder so both playground_files/ and assets/ are reachable.
    handler = partial(SimpleHTTPRequestHandler, directory=str(PLAYGROUND_DIR))
    server = ThreadingHTTPServer(("127.0.0.1", port), handler)
    print(f"[serve] http://127.0.0.1:{port}/ (serving {PLAYGROUND_DIR})")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n[serve] stopped")

def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Serve the lambda calculus playground.")
    parser.add_argument(
        "--port",
        type=int,
        default=8000,
        help="Port to bind the HTTP server (default: 8000)",
    )
    parser.add_argument(
        "--no-build",
        action="store_true",
        help="Skip build.py execution and asset copy",
    )
    return parser.parse_args()

def main() -> None:
    args = parse_args()
    if not args.no_build:
        call_build_script()
        pull_assets()
    serve(port=args.port)

if __name__ == "__main__":
    main()
