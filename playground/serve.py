from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import shutil
import sys
import subprocess

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
    handler = partial(SimpleHTTPRequestHandler, directory=str(PLAYGROUND_DIR / "playground_files"))
    server = ThreadingHTTPServer(("127.0.0.1", port), handler)
    print(f"[serve] http://127.0.0.1:{port}/ (serving {PLAYGROUND_DIR})")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n[serve] stopped")

def main() -> None:
    call_build_script()
    pull_assets()
    serve(port=8000)

if __name__ == "__main__":
    main()