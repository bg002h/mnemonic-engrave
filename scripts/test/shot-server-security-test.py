#!/usr/bin/env python3
"""D4(b) -- prove shot_server.py's three claimed security properties hold, in
the real code, not its docstring.

design/journeys/shot_server.py's module docstring asserts: CORS pinned to one
origin, flat filenames only, and a resolved-path re-check that catches
symlinks. Those are three separate claims and the docstring is exactly the
thing under suspicion (F-165 / D4b) -- this exercises the real functions and
the real request handler, not a description of them.

The module can't be imported normally: its top level reads sys.argv[1:2] and
then blocks forever in socketserver.TCPServer(...).serve_forever(). So this
loads it by exec()'ing its source up to (but not including) the
"socketserver.TCPServer.allow_reuse_address" line -- everything needed
(NAME_RE, safe_path, class H, OUT, ALLOWED_ORIGIN) is defined before that
line, and sys.argv is set to a scratch temp dir + a test origin first, using
the module's own os.path.realpath(sys.argv[1]) logic rather than
reimplementing it.

The request handler (class H) is exercised directly -- do_POST/do_OPTIONS are
called on a real H instance with a real email.message.Message for .headers
and real io.BytesIO for .rfile/.wfile -- rather than by binding a socket.
do_POST/do_OPTIONS only ever touch self.headers, self.rfile, self.wfile, and
inherited send_response/send_header/end_headers machinery (which need only
self.request_version and self.protocol_version, both set here); no part of
the real request line parsing or socket I/O is bypassed in a way that changes
their behaviour. This is the real do_POST, calling the real safe_path,
running against a real temp-directory filesystem.

Run:
    python3 scripts/test/shot-server-security-test.py
    python3 scripts/test/shot-server-security-test.py --target /path/to/mutated/copy.py

Exits non-zero (uncaught AssertionError) on the first failure. No arguments
required for normal use, no real network socket ever opened.
"""

import argparse
import base64
import email.message
import io
import json
import os
import pathlib
import shutil
import sys
import tempfile

DEFAULT_TARGET = (
    pathlib.Path(__file__).resolve().parent.parent.parent
    / "design" / "journeys" / "shot_server.py"
)
CUT_MARKER = "socketserver.TCPServer.allow_reuse_address = True"


def load_module(target_path, out_dir, allowed_origin):
    """exec() shot_server.py's definitions (NAME_RE, safe_path, H, OUT,
    ALLOWED_ORIGIN) without running its blocking serve_forever() tail."""
    src = pathlib.Path(target_path).read_text()
    if CUT_MARKER not in src:
        raise RuntimeError(
            f"cut marker not found in {target_path} -- shot_server.py's "
            "structure changed; update CUT_MARKER"
        )
    defs_src = src.split(CUT_MARKER, 1)[0]
    old_argv = sys.argv
    sys.argv = ["shot_server.py", str(out_dir), "0", allowed_origin]
    ns = {"__name__": "shot_server_under_test", "__file__": str(target_path)}
    try:
        exec(compile(defs_src, str(target_path), "exec"), ns)
    finally:
        sys.argv = old_argv
    return ns


def _invoke(ns, method_name, origin, body=b""):
    """Call do_POST/do_OPTIONS directly on a real H instance -- no socket."""
    H = ns["H"]
    h = H.__new__(H)  # bypass socketserver.__init__, which wants a real conn
    h.client_address = ("127.0.0.1", 55555)
    h.request_version = "HTTP/1.1"
    h.protocol_version = getattr(H, "protocol_version", "HTTP/1.0")
    h.requestline = f"{method_name[3:]} / HTTP/1.1"
    h.close_connection = True
    msg = email.message.Message()  # case-insensitive .get(), like the real
    if origin is not None:         # http.client.HTTPMessage self.headers
        msg["Origin"] = origin
    if body:
        msg["Content-Length"] = str(len(body))
    h.headers = msg
    h.rfile = io.BytesIO(body)
    h.wfile = io.BytesIO()
    getattr(h, method_name)()
    raw = h.wfile.getvalue()
    head, _, rest = raw.partition(b"\r\n\r\n")
    lines = head.split(b"\r\n")
    status = int(lines[0].split()[1])
    headers = {}
    for line in lines[1:]:
        if b":" in line:
            k, v = line.split(b":", 1)
            headers[k.decode().strip()] = v.decode().strip()
    return status, headers, rest


def _data_url(raw_bytes):
    return "data:image/png;base64," + base64.b64encode(raw_bytes).decode()


def test_origin_pinning(ns):
    print("== property 1: pinned to one origin ==")
    ORIGIN = ns["ALLOWED_ORIGIN"]
    png = b"\x89PNG\r\n\x1a\nFAKE"
    payload = json.dumps({"name": "frame001.png", "png": _data_url(png)}).encode()

    # Legitimate request: proves the suite can PASS, not just refuse.
    status, headers, body = _invoke(ns, "do_POST", ORIGIN, payload)
    assert status == 200, f"expected 200 for legit request, got {status}"
    assert body == b"ok", f"expected b'ok' body, got {body!r}"
    written = pathlib.Path(ns["OUT"]) / "frame001.png"
    assert written.read_bytes() == png, "written file content mismatch"
    assert headers.get("Access-Control-Allow-Origin") == ORIGIN
    print(f"  [legit]           200, wrote {written}")

    # Wrong origin.
    status, headers, _ = _invoke(ns, "do_POST", "http://evil.example", payload)
    assert status == 403, f"expected 403 for wrong origin, got {status}"
    assert "Access-Control-Allow-Origin" not in headers, "ACAO leaked on reject"
    print("  [wrong origin]    403, no ACAO header leaked")

    # Missing Origin header entirely -- the one people forget to test.
    status, headers, _ = _invoke(ns, "do_POST", None, payload)
    assert status == 403, f"expected 403 for missing origin, got {status}"
    assert "Access-Control-Allow-Origin" not in headers, "ACAO leaked on reject"
    print("  [missing origin]  403, no ACAO header leaked")

    # Preflight (OPTIONS), both directions.
    status, headers, _ = _invoke(ns, "do_OPTIONS", ORIGIN)
    assert status == 204, f"expected 204 for matching-origin OPTIONS, got {status}"
    assert headers.get("Access-Control-Allow-Origin") == ORIGIN
    status, headers, _ = _invoke(ns, "do_OPTIONS", "http://evil.example")
    assert status == 403, f"expected 403 for wrong-origin OPTIONS, got {status}"
    assert "Access-Control-Allow-Origin" not in headers, "ACAO leaked on reject"
    print("  [OPTIONS]         204 for matching origin, 403 + no ACAO for mismatch")
    print("PASS: origin pinning\n")


def test_flat_filenames(ns):
    print("== property 2: flat filenames only ==")
    safe_path = ns["safe_path"]
    OUT = ns["OUT"]

    ok = safe_path("frame001.png")
    assert ok == os.path.join(OUT, "frame001.png"), f"unexpected resolution: {ok}"
    print(f"  [legit name]              accepted -> {ok}")

    bad_names = {
        "path traversal":         "../../etc/passwd.png",
        "leading slash":          "/etc/passwd.png",
        "backslash":              "..\\..\\win.ini.png",
        "url-encoded traversal":  "%2e%2e%2f.png",
        "nested path":            "a/b.png",
        "double-dot, no separator (matches NAME_RE; caught only by the "
        "explicit '..' in name check)": "x..svg",
        "dotfile":                ".hidden.png",
    }
    for label, name in bad_names.items():
        try:
            p = safe_path(name)
        except ValueError:
            print(f"  [{label}] rejected: {name!r}")
        else:
            raise AssertionError(f"{label} should have been rejected, got {p!r}")
    print("PASS: flat filenames\n")


def test_resolved_path_recheck(ns):
    print("== property 3: resolved-path re-check ==")
    safe_path = ns["safe_path"]
    OUT = pathlib.Path(ns["OUT"])
    outside = pathlib.Path(tempfile.mkdtemp(prefix="shot-server-outside-"))
    target = outside / "pwned.png"  # deliberately outside OUT; need not exist
    link = OUT / "evil.png"
    link.symlink_to(target)
    try:
        try:
            p = safe_path("evil.png")
        except ValueError as e:
            print(f"  [symlink escape, safe_path()] rejected: {e}")
        else:
            raise AssertionError(f"symlink escape should have been rejected, got {p!r}")

        # Same attack through the full HTTP path: confirm nothing gets written
        # anywhere, not just that safe_path() raises in isolation.
        payload = json.dumps({"name": "evil.png", "png": _data_url(b"whatever")}).encode()
        status, _, _ = _invoke(ns, "do_POST", ns["ALLOWED_ORIGIN"], payload)
        assert status == 400, f"expected 400 for symlink-escaping name via POST, got {status}"
        assert not target.exists(), f"attacker write path {target} was created"
        print(f"  [symlink escape, full POST]  400, {target} not created")
    finally:
        shutil.rmtree(outside, ignore_errors=True)
    print("PASS: resolved-path re-check\n")


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--target", default=str(DEFAULT_TARGET),
                     help="path to shot_server.py (or a mutated scratch copy)")
    ap.add_argument("--origin", default="http://127.0.0.1:9797",
                     help="ALLOWED_ORIGIN to configure the module under test with")
    args = ap.parse_args()

    tmp = tempfile.mkdtemp(prefix="shot-server-out-")
    try:
        ns = load_module(args.target, tmp, args.origin)
        test_origin_pinning(ns)
        test_flat_filenames(ns)
        test_resolved_path_recheck(ns)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    print("ALL PASS")


if __name__ == "__main__":
    main()
