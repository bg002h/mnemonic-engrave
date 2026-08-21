#!/usr/bin/env python3
"""Capture the TAPROOT PATHOLOGICAL journey's device half, and CHECK it against the host.

The host half (transcript_tr_pathological.sh) produced a card set and, separately,
the wallet id and addresses that card set must prove to. This drives the same
cards into the emulator over NFC and reads what SeedHammer II shows.

It is a comparison, not a photo shoot. cmd/emu/shots_tr_pathological.js is handed
the host's id and addresses and throws if the screen disagrees, so a green run
means the device and the host agree across the air gap -- through two
implementations, in two languages, from the same four xpubs.

    python3 capture_tr_pathological.py [--port 8793] [--shot-port 8734]

Exits non-zero unless every expected shot arrived AND the comparison passed, so
a partial or disagreeing capture cannot be mistaken for a complete one.
"""
import argparse, asyncio, functools, http.server, json, os, socketserver
import subprocess, sys, threading, time

W = os.path.dirname(os.path.abspath(__file__))
EMU = os.path.abspath(os.path.join(W, "..", "..", "..", "seedhammer", "cmd", "emu"))
SHOTS = os.path.join(W, "shots")
OUT = os.path.join(W, "out", "tr-pathological")

# The consent screen pages, so the count is not known ahead of time -- only the
# fixed shots are listed here, and the driver's own return value carries the
# consent pages it actually took.
EXPECTED = ["t00-boot.png", "t01-carousel.png", "t02-gather-empty.png",
            "t03-gather-full.png", "t04-consent-p0.png"]


def read_artifacts():
    """Load what the host half wrote. A MISSING file is fatal, deliberately.

    F-210's failure was a journey step reading an intermediate that nothing
    committed produced, so the run only ever worked when a previous session had
    left the file behind. Every path here is written by transcript_walletpolicy.sh
    in the same directory, and if it is absent the answer is to run that script,
    not to carry on with a default.
    """
    def need(name):
        p = os.path.join(OUT, name)
        if not os.path.exists(p):
            sys.exit(f"missing {p}\nRun ./transcript_tr_pathological.sh first -- it writes this.")
        return [l.strip() for l in open(p) if l.strip()]

    md1 = need("md1-keyed.txt")
    ids = need("tr.id.txt")
    recv = need("tr.receive.txt")
    chg = need("tr.change.txt")

    policy_id = None
    for line in ids:
        if line.startswith("wallet-policy-id:"):
            policy_id = line.split(":", 1)[1].strip()
    if not policy_id:
        sys.exit("no wallet-policy-id in tr.id.txt")
    # The device shows two per chain (plan D6); the host derived two per chain.
    # Asserting on MORE than the screen can hold would fail for the wrong reason.
    return md1, policy_id, recv[:2] + chg[:2]


def build_wasm():
    print("building emu.wasm ...", flush=True)
    r = subprocess.run(["sh", "build.sh"], cwd=EMU, capture_output=True, text=True)
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


async def drive(port, shot_port, md1, policy_id, addresses):
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
                """async ({shotURL, md1, expect}) => {
                     const m = await import("./shots_tr_pathological.js");
                     return await m.run({ shotURL, md1, expect });
                   }""",
                {"shotURL": f"http://127.0.0.1:{shot_port}/",
                 "md1": md1,
                 "expect": {"walletPolicyId": policy_id, "addresses": addresses}},
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
    # THE NEGATIVE CONTROL, AS A COMMAND (G6). The sibling journey's was run
    # once by hand and remembered; a comparison nobody has made fail is not
    # evidence it can. This corrupts ONE character of ONE expected address and
    # requires the walk to notice — exit 0 only if the capture FAILED.
    ap.add_argument("--prove-it-can-fail", action="store_true",
                    help="corrupt one expected address; succeed only if the walk catches it")
    ap.add_argument("--port", type=int, default=8795)
    ap.add_argument("--shot-port", type=int, default=8736)
    ap.add_argument("--no-build", action="store_true")
    a = ap.parse_args()

    md1, policy_id, addresses = read_artifacts()
    if a.prove_it_can_fail:
        # Flip the LAST character of the first address: a one-character lie is
        # the smallest thing the comparison must still catch.
        bad = addresses[0]
        addresses = [bad[:-1] + ("q" if bad[-1] != "q" else "p")] + addresses[1:]
        print(f"NEGATIVE CONTROL: expecting {addresses[0]} (corrupted)")
    print(f"host: {len(md1)} md1 chunks, policy id {policy_id}")
    for x in addresses:
        print(f"      {x}")

    if not a.no_build:
        build_wasm()
    os.makedirs(SHOTS, exist_ok=True)

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
        res = asyncio.run(drive(a.port, a.shot_port, md1, policy_id, addresses))
    finally:
        httpd.shutdown()
        shot.terminate()
        try:
            out, _ = shot.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            shot.kill(); out = ""
        if out:
            print(out.strip())

    if a.prove_it_can_fail:
        if res is None:
            print("\nNEGATIVE CONTROL PASSED: the walk refused the corrupted address.")
            sys.exit(0)
        sys.exit("NEGATIVE CONTROL FAILED: the walk accepted an address the host never derived. "
                 "The comparison proves nothing.")

    if res is None:
        sys.exit("capture failed: the driver did not return")

    # SIZE, NOT EXISTENCE. A canvas that fails to rasterise yields "data:," which
    # the receiver writes as a zero-byte PNG with a 200 OK, so a driver can
    # report success for files holding no image.
    def bad(n):
        p = os.path.join(SHOTS, n)
        return not os.path.exists(p) or os.path.getsize(p) < 512

    want = list(EXPECTED) + [n for n in res.get("shots", []) if n not in EXPECTED]
    missing = [n for n in want if bad(n)]
    if missing:
        sys.exit(f"capture INCOMPLETE -- missing or empty: {missing}")

    with open(os.path.join(SHOTS, "tr-pathological-result.json"), "w") as f:
        json.dump(res, f, indent=2)

    print(f"\ncaptured {len(want)} shots into {SHOTS}")
    print(f"chunks presented: {res['chunksPresented']}, cards gathered: {res['cardsGathered']}")
    print(f"consent pages read: {len(res['consentPages'])}")
    print("MATCHED against the host:")
    print(f"  wallet id  {res['matched']['walletPolicyId']}")
    for x in res["matched"]["addresses"]:
        print(f"  address    {x}")


if __name__ == "__main__":
    main()
