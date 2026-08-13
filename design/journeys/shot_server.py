"""Receives canvas frames from the emulator page and writes them to disk.

The emulator renders into a single 480x320 canvas, so a POSTed data URL is the
exact device frame -- no browser chrome, no CSS scaling, no cropping guesswork.

Usage:
    python3 shot_server.py <out-dir> <port> [allowed-origin]

SECURITY. This listens on localhost and takes a filename and a payload from a
web page, which is precisely the shape that turns a scratch tool into an
arbitrary-file-write primitive. Two things are load-bearing:

  * The name is validated against a strict whitelist and then re-checked after
    resolution, so neither `../` nor an absolute path can escape <out-dir>.
    os.path.join() ALONE IS NOT A CONTAINMENT CHECK -- joining an absolute path
    silently discards the prefix.
  * CORS is pinned to one origin. The original version sent
    `Access-Control-Allow-Origin: *` and answered the preflight, so any site the
    operator happened to be browsing could drive this server while it ran.
"""

import base64
import binascii
import http.server
import json
import os
import re
import socketserver
import sys

OUT = os.path.realpath(sys.argv[1])
PORT = int(sys.argv[2])
ALLOWED_ORIGIN = sys.argv[3] if len(sys.argv) > 3 else "http://127.0.0.1:8731"
os.makedirs(OUT, exist_ok=True)

# Frames and plate dumps only, flat, no dotfiles, no separators of any kind.
NAME_RE = re.compile(r"\A[A-Za-z0-9][A-Za-z0-9._-]{0,63}\.(png|svg)\Z")
MAX_BODY = 16 * 1024 * 1024


def safe_path(name):
    """Resolve name under OUT, or raise. Both checks are required."""
    if not isinstance(name, str) or not NAME_RE.match(name) or ".." in name:
        raise ValueError(f"rejected name: {name!r}")
    path = os.path.realpath(os.path.join(OUT, name))
    # Re-check after resolution: catches symlinks planted in OUT, which the
    # regex cannot see.
    if os.path.dirname(path) != OUT:
        raise ValueError(f"escapes {OUT}: {name!r}")
    return path


class H(http.server.BaseHTTPRequestHandler):
    def _cors(self):
        if self.headers.get("Origin") == ALLOWED_ORIGIN:
            self.send_header("Access-Control-Allow-Origin", ALLOWED_ORIGIN)
            self.send_header("Access-Control-Allow-Headers", "content-type")
            self.send_header("Access-Control-Allow-Methods", "POST, OPTIONS")
            self.send_header("Vary", "Origin")

    def _fail(self, code, why):
        print(f"REJECTED {code}: {why}", flush=True)
        self.send_response(code)
        self._cors()
        self.end_headers()

    def do_OPTIONS(self):
        if self.headers.get("Origin") != ALLOWED_ORIGIN:
            return self._fail(403, f"origin {self.headers.get('Origin')!r}")
        self.send_response(204)
        self._cors()
        self.end_headers()

    def do_POST(self):
        origin = self.headers.get("Origin")
        # A same-origin fetch from the emulator page always sends Origin; a
        # missing one means something that is not that page.
        if origin != ALLOWED_ORIGIN:
            return self._fail(403, f"origin {origin!r}")
        try:
            n = int(self.headers.get("content-length", 0))
        except ValueError:
            return self._fail(400, "bad content-length")
        if n <= 0 or n > MAX_BODY:
            return self._fail(413, f"content-length {n}")
        try:
            body = json.loads(self.rfile.read(n))
            path = safe_path(body.get("name"))
            data = body["png"]
            raw = base64.b64decode(data.split(",", 1)[1], validate=True)
        except (ValueError, KeyError, TypeError, binascii.Error) as e:
            return self._fail(400, str(e))

        with open(path, "wb") as f:
            f.write(raw)
        print(f"{os.path.basename(path)}: {len(raw)} bytes", flush=True)
        self.send_response(200)
        self._cors()
        self.end_headers()
        self.wfile.write(b"ok")

    def log_message(self, *a):
        pass


socketserver.TCPServer.allow_reuse_address = True
with socketserver.TCPServer(("127.0.0.1", PORT), H) as s:
    print(f"receiving into {OUT} from {ALLOWED_ORIGIN} on :{PORT}", flush=True)
    s.serve_forever()
