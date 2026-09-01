#!/usr/bin/env python3
"""Drive the device-csid-warning cycle's mk1-INSPECT flow in the emulator.

Reads the pinned/clean-twin rows DIRECTLY from the fork's vendored corpus
(seedhammer/mk/testdata/csid_ext_v0.1.json) -- never hand-minted, per
mnemonic-engrave/design/SPEC_device_csid_warning.md's own rule for these
fixtures -- and presents each row's two mk1 chunk strings over simulated NFC
at the emulator's home screen. cmd/emu/shots_csid_warning.js asserts the
pinned row shows the Contract-2 warning modal and the clean twin does not, so
a green run means the on-device comparison is actually wired, not merely that
some screenshots exist.

    python3 capture_csid_warning.py [--port 8798] [--shot-port 8739] [--fork-dir DIR]

--fork-dir defaults to the SHIBBOLETH_SEEDHAMMER_DIR env var, or the sibling
"../../../seedhammer" checkout the other capture_*.py scripts here assume;
pass it explicitly to point at a worktree (e.g. this cycle's
sh-worktrees/dev-warn) instead of the default clone.

Exits non-zero unless every expected shot arrived AND both assertions
(warns on mismatch, silent on the clean twin) held.
"""
import argparse, asyncio, functools, http.server, json, os, socketserver
import subprocess, sys, threading, time

W = os.path.dirname(os.path.abspath(__file__))
DEFAULT_FORK_DIR = os.environ.get(
    "SHIBBOLETH_SEEDHAMMER_DIR",
    os.path.abspath(os.path.join(W, "..", "..", "..", "seedhammer")))
SHOTS = os.path.join(W, "shots")
OUT = os.path.join(W, "out", "csid-warning")
DEST_DIR = W  # design/journeys/ -- the modal PNG's required destination

# The fixed shots; the bonus bundle-review shots are best-effort and checked
# separately (a missing one does not fail the capture -- see main()).
EXPECTED = [
    "csid00-boot.png",
    "csid01-chooser.png",
    "csid02-gather-1of2.png",
    "csid-warning-modal.png",
    "csid03-card-display.png",
    "csid04-clean-card-display.png",
]
BONUS = ["csid05-carousel-bundle.png", "csid06-bundle-modal.png", "csid-warning-bundle-review.png"]


def read_rows(corpus_path):
    """Read the PINNED and CLEAN TWIN rows straight from the vendored corpus
    -- the fixture pair SPEC_device_csid_warning.md's R0 rounds pinned:
    SEED_pinned_12345_ef12f (declared 12345, derived ef12f) and its clean
    twin SEED_plate_b_ef12f (declared == derived == ef12f), same key
    content."""
    if not os.path.exists(corpus_path):
        sys.exit(f"missing vendored corpus {corpus_path} -- is the seedhammer checkout/worktree present?")
    with open(corpus_path) as f:
        corpus = json.load(f)
    rows = {r["name"]: r for r in corpus["rows"]}
    for name in ("SEED_pinned_12345_ef12f", "SEED_plate_b_ef12f"):
        if name not in rows:
            sys.exit(f"corpus row {name!r} not found in {corpus_path}")
    pinned, clean = rows["SEED_pinned_12345_ef12f"], rows["SEED_plate_b_ef12f"]
    if not pinned["expect_mismatch_warning"]:
        sys.exit(f"{pinned['name']}: expect_mismatch_warning=false, want true")
    if clean["expect_mismatch_warning"]:
        sys.exit(f"{clean['name']}: expect_mismatch_warning=true, want false")
    return pinned, clean


def build_wasm(emu_dir):
    print("building emu.wasm ...", flush=True)
    r = subprocess.run(["sh", "build.sh"], cwd=emu_dir, capture_output=True, text=True)
    if r.returncode != 0:
        sys.exit(f"emu build failed:\n{r.stdout}\n{r.stderr}")
    print((r.stdout.strip().splitlines() or ["built"])[-1], flush=True)


def serve(directory, port):
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


async def drive(port, shot_port, pinned, clean):
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
        await page.wait_for_function("window.shNFC !== undefined", timeout=60_000)
        await page.wait_for_timeout(2500)
        try:
            res = await page.evaluate(
                """async ({shotURL, pinned, clean}) => {
                     const m = await import("./shots_csid_warning.js");
                     return await m.run({ shotURL, pinned, clean });
                   }""",
                {"shotURL": f"http://127.0.0.1:{shot_port}/", "pinned": pinned, "clean": clean},
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
    ap.add_argument("--port", type=int, default=8798)
    ap.add_argument("--shot-port", type=int, default=8739)
    ap.add_argument("--no-build", action="store_true")
    ap.add_argument("--fork-dir", default=DEFAULT_FORK_DIR,
                     help="the seedhammer checkout/worktree to build+serve cmd/emu from "
                          "(default: $SHIBBOLETH_SEEDHAMMER_DIR or ../../../seedhammer)")
    a = ap.parse_args()

    fork_dir = os.path.abspath(a.fork_dir)
    emu_dir = os.path.join(fork_dir, "cmd", "emu")
    corpus_path = os.path.join(fork_dir, "mk", "testdata", "csid_ext_v0.1.json")
    if not os.path.isdir(emu_dir):
        sys.exit(f"no cmd/emu under --fork-dir {fork_dir!r} ({emu_dir})")
    print(f"fork dir: {fork_dir}")

    pinned, clean = read_rows(corpus_path)
    print(f"pinned:  {pinned['name']} declared {pinned['declared_csid']} derived {pinned['derived_csid']}")
    print(f"clean:   {clean['name']} declared {clean['declared_csid']} derived {clean['derived_csid']}")

    if not a.no_build:
        build_wasm(emu_dir)
    os.makedirs(SHOTS, exist_ok=True)
    os.makedirs(OUT, exist_ok=True)

    shot = subprocess.Popen(
        [sys.executable, os.path.join(W, "shot_server.py"), SHOTS, str(a.shot_port),
         f"http://127.0.0.1:{a.port}"],
        stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    httpd = serve(emu_dir, a.port)
    time.sleep(1.0)
    if shot.poll() is not None:
        sys.exit(f"shot_server.py exited immediately ({shot.returncode}); "
                 f"is 127.0.0.1:{a.shot_port} already in use?")

    try:
        res = asyncio.run(drive(a.port, a.shot_port, pinned["strings"], clean["strings"]))
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
        sys.exit("capture failed: the driver did not return (see DRIVER FAILED above)")

    # SIZE, NOT EXISTENCE (shots_seating.js's own lesson): a canvas that fails
    # to rasterise yields "data:," which the receiver writes as a zero-byte
    # PNG with a 200 OK.
    def bad(n):
        p = os.path.join(SHOTS, n)
        return not os.path.exists(p) or os.path.getsize(p) < 512

    missing = [n for n in EXPECTED if bad(n)]
    if missing:
        sys.exit(f"capture INCOMPLETE -- missing or empty required shots: {missing}")

    if not res.get("modalText") or "wasnotderivedfrom" not in res["modalText"]:
        sys.exit(f"the pinned card's modal text does not read as the csid warning: {res.get('modalText')!r}")
    if not res.get("cleanSilent"):
        sys.exit("the clean twin was NOT silent -- a false-positive warning fired")

    bonus_missing = [n for n in BONUS if bad(n)]
    if bonus_missing:
        print(f"bonus bundle-review shots not captured (best-effort): {bonus_missing}"
              + (f"; driver reported: {res.get('reviewError')}" if res.get("reviewError") else ""))

    with open(os.path.join(OUT, "csid-warning-result.json"), "w") as f:
        json.dump(res, f, indent=2)

    # Copy the required deliverable(s) to design/journeys/ under the names
    # the implementation report and the operator screenshot gate expect.
    import shutil
    copied = []
    for n in ["csid-warning-modal.png", "csid-warning-bundle-review.png"]:
        src = os.path.join(SHOTS, n)
        if os.path.exists(src) and os.path.getsize(src) >= 512:
            dst = os.path.join(DEST_DIR, n)
            shutil.copyfile(src, dst)
            copied.append(dst)

    print(f"\ncaptured {len(res.get('shots', []))} shots into {SHOTS}")
    print(f"modal text: {res['modalText'][:120]}...")
    print(f"clean twin silent: {res['cleanSilent']}")
    print("copied to design/journeys/:")
    for c in copied:
        print(f"  {c}")
    if not os.path.exists(os.path.join(DEST_DIR, "csid-warning-modal.png")):
        sys.exit("csid-warning-modal.png was not copied to design/journeys/ -- capture is incomplete")


if __name__ == "__main__":
    main()
