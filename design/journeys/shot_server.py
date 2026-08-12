"""Receives canvas frames from the emulator page and writes them to disk.

The emulator renders into a single 480x320 canvas, so a POSTed data URL is the
exact device frame -- no browser chrome, no CSS scaling, no cropping guesswork.
"""

import base64
import http.server
import json
import os
import socketserver
import sys

OUT = sys.argv[1]
PORT = int(sys.argv[2])
os.makedirs(OUT, exist_ok=True)


class H(http.server.BaseHTTPRequestHandler):
    def _cors(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Headers", "content-type")
        self.send_header("Access-Control-Allow-Methods", "POST, OPTIONS")

    def do_OPTIONS(self):
        self.send_response(204)
        self._cors()
        self.end_headers()

    def do_POST(self):
        n = int(self.headers.get("content-length", 0))
        body = json.loads(self.rfile.read(n))
        raw = base64.b64decode(body["png"].split(",", 1)[1])
        path = os.path.join(OUT, body["name"])
        with open(path, "wb") as f:
            f.write(raw)
        print(f"{body['name']}: {len(raw)} bytes", flush=True)
        self.send_response(200)
        self._cors()
        self.end_headers()
        self.wfile.write(b"ok")

    def log_message(self, *a):
        pass


socketserver.TCPServer.allow_reuse_address = True
with socketserver.TCPServer(("127.0.0.1", PORT), H) as s:
    s.serve_forever()
