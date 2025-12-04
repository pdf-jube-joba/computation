from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
import shutil

PLAYGROUND_DIR = Path(__file__).resolve().parent
WORKSPACE_DIR = PLAYGROUND_DIR.parent
# `./assets` may contains hand written files => include all of `./assets`
ASSETS_DIR = WORKSPACE_DIR / "assets" 

def pull_assets() -> None:
    # copy from `assets/` to `playground/assets/`
    dest_dir = PLAYGROUND_DIR / "assets"
    if dest_dir.exists():
        shutil.rmtree(dest_dir)
    shutil.copytree(ASSETS_DIR, dest_dir)

def serve(port: int) -> None:
    handler = partial(SimpleHTTPRequestHandler, directory=str(ASSETS_DIR))
    server = ThreadingHTTPServer(("127.0.0.1", port), handler)
    print(f"[serve] http://127.0.0.1:{port}/ (serving {ASSETS_DIR})")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n[serve] stopped")