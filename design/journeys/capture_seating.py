#!/usr/bin/env python3
"""Drive D3's FIRST HALF on the device: a keyless template plus its mk1 key cards.

The host half (transcript_walletpolicy.sh) produced a card set and, separately,
the wallet id and addresses that card set must prove to. This drives the same
cards into the emulator over NFC and reads what SeedHammer II shows.

It is a comparison, not a photo shoot. cmd/emu/shots_seating.js is handed
the host's id and addresses and throws if the screen disagrees, so a green run
means the device and the host agree across the air gap -- through two
implementations, in two languages, from the same four xpubs.

    python3 capture_seating.py [--port 8793] [--shot-port 8734]

Exits non-zero unless every expected shot arrived AND the comparison passed, so
a partial or disagreeing capture cannot be mistaken for a complete one.
"""
import argparse, asyncio, functools, http.server, json, os, socketserver
import subprocess, sys, threading, time

W = os.path.dirname(os.path.abspath(__file__))
EMU = os.path.abspath(os.path.join(W, "..", "..", "..", "seedhammer", "cmd", "emu"))
SHOTS = os.path.join(W, "shots")
OUT = os.path.join(W, "out", "seating")

# The consent screen pages, so the count is not known ahead of time -- only the
# fixed shots are listed here, and the driver's own return value carries the
# consent pages it actually took.
EXPECTED = ["s00-boot.png", "s01-carousel.png", "s02-gather-empty.png",
            "s03-gather-full.png", "s04-consent-p0.png"]


def read_artifacts():
    """Load the fixture: a KEYLESS template, its mk1 key cards, and what the
    host derives for the seated wallet.

    Written by `make_seating_fixture.py`, which mints everything from the
    conformance corpus so the expected addresses have Rust behind them.
    """
    p = os.path.join(OUT, "seating-fixture.json")
    if not os.path.exists(p):
        sys.exit(f"missing {p}\nRun ./make_seating_fixture.py first -- it writes this.")
    with open(p) as f:
        fx = json.load(f)
    return fx["template"], fx["keyCards"], fx["policyId"], fx["addresses"], fx.get("wrongStubCard")


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


async def drive(port, shot_port, md1, key_cards, policy_id, addresses, refusal=None):
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
                """async ({shotURL, md1, keyCards, expect, expectRefusal}) => {
                     const m = await import("./shots_seating.js");
                     return await m.run({ shotURL, md1, keyCards, expect, expectRefusal });
                   }""",
                {"shotURL": f"http://127.0.0.1:{shot_port}/",
                 "md1": md1,
                 "keyCards": key_cards,
                 "expect": {"walletPolicyId": policy_id, "addresses": addresses},
                 "expectRefusal": refusal},
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
    ap.add_argument("--port", type=int, default=8797)
    ap.add_argument("--shot-port", type=int, default=8738)
    ap.add_argument("--no-build", action="store_true")
    # A4's refusal arm, as a command. Corrupts one key card's stub so it belongs
    # to no policy, and requires the device to say so IN WORDS — the sentence
    # naming the full-policy-vs-template stub difference, which is the likeliest
    # real cause and the one an operator cannot otherwise diagnose.
    ap.add_argument("--prove-refusal", action="store_true",
                    help="corrupt a key card's stub; succeed only if the device refuses in words")
    a = ap.parse_args()

    md1, key_cards, policy_id, addresses, wrong_stub = read_artifacts()
    refusal = None
    if a.prove_refusal:
        if not wrong_stub:
            sys.exit("fixture has no wrongStubCard; re-run make_seating_fixture.py")
        # A VALID card carrying the wrong stub — a card made for the full-policy
        # form of this same wallet. Not a corrupted one: mk1 is BCH-checksummed,
        # so a mangled card is dropped at the scanner and never reaches seating.
        key_cards = [wrong_stub] + key_cards[1:]
        refusal = "different stub"
        print("REFUSAL ARM: key card 1 replaced with one stubbed on the POLICY id")
    print(f"host: {len(md1)} template chunk(s), {len(key_cards)} key card(s), policy id {policy_id}")
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
        res = asyncio.run(drive(a.port, a.shot_port, md1, key_cards, policy_id, addresses, refusal))
    finally:
        httpd.shutdown()
        shot.terminate()
        try:
            out, _ = shot.communicate(timeout=5)
        except subprocess.TimeoutExpired:
            shot.kill(); out = ""
        if out:
            print(out.strip())

    if a.prove_refusal:
        if res is None:
            sys.exit("REFUSAL ARM FAILED: the device did not refuse in the expected words.")
        print(f"\nREFUSAL ARM PASSED: the device refused, saying {res['refused']!r}, "
              f"and showed no address.")
        sys.exit(0)

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

    with open(os.path.join(SHOTS, "seating-result.json"), "w") as f:
        json.dump(res, f, indent=2)

    print(f"\ncaptured {len(want)} shots into {SHOTS}")
    print(f"template chunks: {res['chunksPresented']}, cards: {res['cardsGathered']}, "
          f"key cards: {res.get('keyCardsGathered', 0)}")
    print(f"consent pages read: {len(res['consentPages'])}")
    print("MATCHED against the host:")
    print(f"  wallet id  {res['matched']['walletPolicyId']}")
    for x in res["matched"]["addresses"]:
        print(f"  address    {x}")


if __name__ == "__main__":
    main()
