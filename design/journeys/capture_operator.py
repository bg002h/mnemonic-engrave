#!/usr/bin/env python3
"""Capture the operator (5-of-12 wsh) journey's screenshots, end to end.

F-210: the PDFs were committed while the process that produced their
screenshots was not -- it was console code in a session that no longer exists.
This is that process, committed. It:

  1. REBUILDS emu.wasm first. A capture against a stale binary documents the
     stale binary; the walk-driver record of 2026-08-19 made the same point.
  2. serves cmd/emu on a loopback port,
  3. starts shot_server.py (the receiver README.md already documented) pointed
     at design/journeys/shots,
  4. drives cmd/emu/shots_operator.js in a real browser,
  5. writes the plate captions the overlay rendered, AND the carousel entries
     it walked, beside the PNGs.

The carousel list is captured because build_pdf.py hardcoded "8 entries" and
named eight programs; there are TEN (Load Payload and Sealed Payload arrived
later). A list that is rendered from what the walk saw cannot drift again.

The captions are written as data because build_pdf_pathological.py used to
carry them as prose typed next to the image ("head 35.7, 13.0 mm"). A typed
caption can drift from the picture it labels while both still look right.

    python3 capture_operator.py [--port 8791] [--shot-port 8732]
    python3 capture_operator.py --measure     # just report the cut's step total

Exits non-zero unless every expected shot arrived, so a partial capture cannot
be mistaken for a complete one.
"""
import argparse, asyncio, json, os, subprocess, sys, threading, time
import http.server, socketserver, functools

W = os.path.dirname(os.path.abspath(__file__))
EMU = os.path.abspath(os.path.join(W, "..", "..", "..", "seedhammer", "cmd", "emu"))
SHOTS = os.path.join(W, "shots")

EXPECTED = [
    "j00-boot.png", "menu-4.png", "j06-bundle-enter.png",
    "j01-backup-enter.png", "j02-sel-mstring.png", "j03-mstring-prompt.png",
    "k01-engrave-text-enter.png", "k04-font.png", "k06-size.png",
    "k08-typed.png", "k09-revealed.png", "k13-after-footer.png",
    "k14-engrave-start.png",
    "p0-plate.png", "p2-screen.png", "p2-plate.png", "p5-plate.png",
    "p11-plate.png", "p17-settled.png",
]


def build_wasm():
    go = os.environ.get("GO", "go")
    env = dict(os.environ)
    if "/nix/store" not in env.get("PATH", "") and os.path.isdir("/nix/store"):
        pass  # caller's PATH decides; build.sh calls `go` itself
    print("building emu.wasm ...", flush=True)
    r = subprocess.run(["sh", "build.sh"], cwd=EMU, capture_output=True, text=True, env=env)
    if r.returncode != 0:
        sys.exit(f"emu build failed:\n{r.stdout}\n{r.stderr}")
    print(r.stdout.strip().splitlines()[-1] if r.stdout.strip() else "built", flush=True)


def serve(directory, port):
    """Serve `directory` on loopback, and DIE if the port is taken.

    A bind failure used to surface as a traceback on stderr while the capture
    carried on -- against whatever server already held the port. It happened to
    be serving the same directory, so the run "succeeded" by accident; a stale
    server pointed anywhere else would have captured someone else's emulator.
    """
    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=directory)
    class Quiet(socketserver.TCPServer):
        allow_reuse_address = True
    try:
        httpd = Quiet(("127.0.0.1", port), handler)
    except OSError as e:
        sys.exit(f"cannot serve {directory} on 127.0.0.1:{port}: {e}\n"
                 f"Something else holds that port -- stop it, or pass --port.")
    httpd.RequestHandlerClass.log_message = lambda *a, **k: None
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd


async def drive(port, shot_port, timeout_s, measure=False):
    from playwright.async_api import async_playwright
    url = f"http://127.0.0.1:{port}/index.html"
    async with async_playwright() as pw:
        browser = await pw.chromium.launch()
        page = await browser.new_page()
        errors = []
        page.on("console", lambda m: errors.append(m.text) if m.type == "error" else None)
        page.on("pageerror", lambda e: errors.append(str(e)))
        await page.goto(url)
        await page.wait_for_function("window.shScreen !== undefined", timeout=60_000)
        await page.wait_for_timeout(2500)
        try:
            res = await page.evaluate(
                """async ([shotURL, measure]) => {
                     const m = await import("./shots_operator.js");
                     return await m.run({ shotURL, measure });
                   }""",
                [f"http://127.0.0.1:{shot_port}/", measure],
            )
        except Exception as e:
            print("DRIVER FAILED:", e, file=sys.stderr)
            if errors:
                print("page errors:", errors[:5], file=sys.stderr)
            await browser.close()
            return None
        await browser.close()
        if errors:
            print("page errors (non-fatal):", errors[:5], file=sys.stderr)
        return res


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, default=8791)
    ap.add_argument("--shot-port", type=int, default=8732)
    ap.add_argument("--timeout", type=int, default=600)
    ap.add_argument("--no-build", action="store_true")
    ap.add_argument("--measure", action="store_true",
                    help="cut the plate and report its step total; captures nothing")
    a = ap.parse_args()

    if not a.no_build:
        build_wasm()
    os.makedirs(SHOTS, exist_ok=True)

    # shot_server.py pins CORS to ONE origin, and the emulator page is that
    # origin -- so it must be told the page's port, not its own.
    # The shot receiver, checked the same way: a Popen that dies still returns a
    # process object, so its exit is polled rather than assumed.
    shot = subprocess.Popen(
        [sys.executable, os.path.join(W, "shot_server.py"), SHOTS, str(a.shot_port),
         f"http://127.0.0.1:{a.port}"],
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    httpd = serve(EMU, a.port)
    time.sleep(1.0)
    if shot.poll() is not None:
        sys.exit(f"shot_server.py exited immediately ({shot.returncode}); "
                 f"is 127.0.0.1:{a.shot_port} already in use?")

    try:
        res = asyncio.run(drive(a.port, a.shot_port, a.timeout, a.measure))
    finally:
        httpd.shutdown()
        shot.terminate()
        try:
            out, _ = shot.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            shot.kill(); out = ""
        if out:
            print(out.strip())

    if res is None:
        sys.exit("capture failed: the driver did not return")

    # EXISTENCE IS NOT ENOUGH. A canvas that fails to rasterise yields the empty
    # data URL "data:,", which shot_server.py writes as a zero-byte PNG with a
    # 200 OK -- so the driver can report success for four files that hold no
    # image. Size is what distinguishes a capture from a POST.
    def bad(n):
        p = os.path.join(SHOTS, n)
        return not os.path.exists(p) or os.path.getsize(p) < 512
    if a.measure:
        print(f"planSteps={res.get('planSteps')}  (put this in shots_operator.js)")
        print(f"carousel: {len(res.get('entries', []))} entries")
        return
    missing = [n for n in EXPECTED if bad(n)]
    # The carousel list rides in the same file: it is capture output too.
    if res.get("entries"):
        with open(os.path.join(SHOTS, "carousel-operator.json"), "w") as f:
            json.dump({"entries": res["entries"], "text": res.get("text", "")}, f, indent=2)
        print(f"wrote carousel-operator.json ({len(res['entries'])} entries)")

    if res.get("captions"):
        with open(os.path.join(SHOTS, "captions-operator.json"), "w") as f:
            json.dump(res["captions"], f, indent=2)
        print(f"wrote captions-operator.json ({len(res['captions'])} plate caption(s))")
        for c in res["captions"]:
            print(f"  {c['name']}: {c['caption']}")

    print(f"captured {len(res.get('taken', []))} shot(s); final screen: {res.get('screen','')[:90]!r}")
    if missing:
        sys.exit("INCOMPLETE -- missing or empty: " + ", ".join(missing))
    print(f"all {len(EXPECTED)} expected shots present")


if __name__ == "__main__":
    main()
