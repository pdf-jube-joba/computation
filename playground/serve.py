from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

PLAYGROUND_DIR = Path(__file__).resolve().parent
ASSETS_DIR = PLAYGROUND_DIR / "generated"

def serve(port: int) -> None:
    handler = partial(SimpleHTTPRequestHandler, directory=str(ASSETS_DIR))
    server = ThreadingHTTPServer(("127.0.0.1", port), handler)
    print(f"[serve] http://127.0.0.1:{port}/ (serving {ASSETS_DIR})")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n[serve] stopped")