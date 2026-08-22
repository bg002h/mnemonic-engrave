#!/usr/bin/env python3
"""Drive the HASHLOCK VAULT journey's device half -- both of its arms.

The host half (transcript_hashvault.sh) built a four-tier degrading vault from
six seeds and engraved a KEYLESS template plus six mk1 key cards. This drives
that card set into the emulator over NFC and reads what SeedHammer II does with
it. There are two arms, and BOTH are findings rather than photo shoots:

  --seating (default)  the engraved set: template + six mk1 cards.
                       EXPECTS A REFUSAL. Every key sits at the same path from
                       a different seed, so all six cards match all six slots
                       and the template declares no fingerprints to break the
                       tie. gui/key_card_seating.go calls that errSeatSlot-
                       Contested. This arm asserts the device says so IN WORDS
                       and shows NO address -- guessing here would be invisible.

  --keyed              the keyed card alone, 15 chunks, no seating involved.
                       EXPECTS ADDRESSES, and compares them to the host's.

The second arm is what makes the first one mean something. On its own, "the
device refused" is also what a device that simply cannot handle this wallet
would do -- four tiers, two sha256 hashlocks, both timelock flavours, multi_a
and a NUMS internal key. The keyed arm derives real addresses from the same six
xpubs, so a refusal in the seating arm can only be about the SEATING.

    python3 capture_hashvault.py [--keyed] [--port 8799] [--shot-port 8740]

Exits non-zero unless the arm's expectation held, so neither a partial capture
nor a disagreeing one can be mistaken for a complete one.
"""
import argparse, asyncio, functools, http.server, json, os, socketserver
import subprocess, sys, threading, time

W = os.path.dirname(os.path.abspath(__file__))
EMU = os.path.abspath(os.path.join(W, "..", "..", "..", "seedhammer", "cmd", "emu"))
SHOTS = os.path.join(W, "shots")
OUT = os.path.join(W, "out", "hashvault")

# The host's ids, measured with `md inspect` on the very strings below rather
# than copied from a doc. The template and the keyed card share a
# TEMPLATE id and have different POLICY ids -- one wallet, two stubs.
TEMPLATE_ID = "68a1a888385797337ce5debc90fcfb1e"
KEYED_POLICY_ID = "a0b128ceaef3155a40af6f8e88765ecb"

# The seating arm's refusal, from gui/wallet_policy.go seatRefusalMessage():
#   "Two different key cards claim the same slot, and this template can't tell
#    them apart. It declares no fingerprints."
# Matched on a distinctive FRAGMENT, because the screen wraps and waitFor
# squashes whitespace -- but asserting the whole sentence would still be
# brittle against a line break the renderer chooses.
REFUSAL_NEEDLE = "declares no fingerprints"

SEATING_EXPECTED = ["h00-boot.png", "h01-carousel.png", "h02-gather-empty.png",
                    "h03-gather-full.png"]


def read_artifacts():
    """Load what transcript_hashvault.sh wrote. A MISSING file is fatal.

    F-210's failure was a journey step reading an intermediate that nothing
    committed produced, so the run only ever worked when a previous session had
    left the file behind. Every path here is written by transcript_hashvault.sh,
    and if it is absent the answer is to run that script, not to default.
    """
    def need(name):
        p = os.path.join(OUT, name)
        if not os.path.exists(p):
            sys.exit(f"missing {p}\nRun ./transcript_hashvault.sh first -- it writes this.")
        return p

    template = [l.strip() for l in open(need("md1-template.txt")) if l.strip()]
    keyed = [l.strip() for l in open(need("md1-keyed.txt")) if l.strip()]
    with open(need("mk-encode.json")) as f:
        enc = json.load(f)
    key_cards = [c["mk1_strings"] for c in enc["cards"]]
    recv = [l.strip() for l in open(need("receive.txt")) if l.strip()]
    chg = [l.strip() for l in open(need("change.txt")) if l.strip()]

    # THE PREMISE, ASSERTED. This journey's whole finding is that the six cards
    # are indistinguishable to the template. If a future edit gives them
    # distinct origins, the seating arm would start passing for a reason that
    # has nothing to do with what it was written to show -- so check it here
    # and say so loudly rather than quietly documenting a different wallet.
    paths = {c["origin_path"] for c in enc["cards"]}
    fps = {c["origin_fingerprint"] for c in enc["cards"]}
    if len(paths) != 1:
        sys.exit(f"PREMISE BROKEN: the six cards no longer share one origin path: {sorted(paths)}\n"
                 f"The seating arm tests slot AMBIGUITY; distinct paths make it decidable, and "
                 f"this walk would then be asserting a refusal that should no longer happen.")
    if len(fps) != len(enc["cards"]):
        sys.exit(f"PREMISE BROKEN: expected {len(enc['cards'])} distinct fingerprints, got {len(fps)}")

    # The device shows two per chain; the host derived two per chain. Asserting
    # on MORE than the screen can hold would fail for the wrong reason.
    return template, keyed, key_cards, recv[:2] + chg[:2], sorted(paths)[0]


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


async def drive(port, shot_port, md1, key_cards, policy_id, addresses, refusal, prefix):
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
            # REUSES shots_seating.js. Its run() already takes a template, key
            # cards and an optional expectRefusal, which is exactly both arms
            # here -- a second near-identical walk file would be one more place
            # for the two to drift apart.
            res = await page.evaluate(
                """async ({shotURL, md1, keyCards, expect, expectRefusal, prefix}) => {
                     const m = await import("./shots_seating.js");
                     return await m.run({ shotURL, md1, keyCards, expect, expectRefusal, prefix });
                   }""",
                {"shotURL": f"http://127.0.0.1:{shot_port}/",
                 "md1": md1,
                 "keyCards": key_cards,
                 "expect": {"walletPolicyId": policy_id, "addresses": addresses},
                 "expectRefusal": refusal,
                 # Per-ARM, not merely per-journey: "h" seating, "k" keyed. Both
                 # arms of this walk shoot the same screens, so one prefix for the
                 # journey would have had the second run silently overwrite the
                 # first -- the same defect as sharing "s" with the seating walk,
                 # one level down.
                 "prefix": prefix},
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
    ap.add_argument("--keyed", action="store_true",
                    help="the control arm: the 15-chunk keyed card alone, expecting addresses")
    ap.add_argument("--port", type=int, default=8799)
    ap.add_argument("--shot-port", type=int, default=8740)
    ap.add_argument("--no-build", action="store_true")
    # THE NEGATIVE CONTROL, AS A COMMAND. The seating arm passes when a needle
    # APPEARS, and a needle that has never failed to appear is not evidence --
    # a walk that silently stopped driving the device would pass it too. This
    # feeds the KEYED card (which derives, as the control arm proves) while
    # still demanding the refusal, so the needle cannot legitimately appear.
    # Exit 0 only if the walk FAILED.
    ap.add_argument("--prove-it-can-fail", action="store_true",
                    help="expect the refusal from a card that derives; succeed only if the walk catches it")
    a = ap.parse_args()

    template, keyed, key_cards, addresses, shared_path = read_artifacts()

    if a.prove_it_can_fail:
        md1, cards, refusal, policy_id, prefix = keyed, [], REFUSAL_NEEDLE, None, "n"
        print("NEGATIVE CONTROL: feeding the KEYED card while demanding the seating refusal.")
        print(f"  {len(md1)} chunks, no key cards, so {REFUSAL_NEEDLE!r} must NOT appear.")
    elif a.keyed:
        md1, cards, refusal, policy_id, prefix = keyed, [], None, KEYED_POLICY_ID, "k"
        print(f"KEYED ARM: {len(md1)} chunks, no seating. Expecting the device to derive.")
        print(f"  policy id {policy_id}")
        for x in addresses:
            print(f"  address   {x}")
    else:
        md1, cards, refusal, policy_id, prefix = template, key_cards, REFUSAL_NEEDLE, None, "h"
        print(f"SEATING ARM: {len(md1)} template chunks + {len(cards)} mk1 cards, "
              f"all at {shared_path}")
        print(f"  template id {TEMPLATE_ID}")
        print(f"  EXPECTING A REFUSAL containing {REFUSAL_NEEDLE!r}, and no address.")

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
        res = asyncio.run(drive(a.port, a.shot_port, md1, cards, policy_id, addresses,
                                refusal, prefix))
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
            print("\nNEGATIVE CONTROL PASSED: the walk refused to report a refusal that "
                  "never happened.")
            sys.exit(0)
        sys.exit("NEGATIVE CONTROL FAILED: the walk reported the seating refusal for a card "
                 "that derives addresses. The seating arm proves nothing.")

    if res is None:
        sys.exit("capture failed: the driver did not return -- see DRIVER FAILED above")

    # SIZE, NOT EXISTENCE. A canvas that fails to rasterise yields "data:,"
    # which the receiver writes as a zero-byte PNG with a 200 OK, so a driver
    # can report success for files holding no image.
    def bad(n):
        p = os.path.join(SHOTS, n)
        return not os.path.exists(p) or os.path.getsize(p) < 512

    want = list(res.get("shots", []))
    if not a.keyed:
        want = SEATING_EXPECTED + [n for n in want if n not in SEATING_EXPECTED]
    missing = [n for n in want if bad(n)]
    if missing:
        sys.exit(f"capture INCOMPLETE -- missing or empty: {missing}")

    tag = "hashvault-keyed" if a.keyed else "hashvault-seating"
    with open(os.path.join(SHOTS, f"{tag}-result.json"), "w") as f:
        json.dump(res, f, indent=2)

    print(f"\ncaptured {len(want)} shots into {SHOTS}")
    if a.keyed:
        print(f"chunks presented: {res['chunksPresented']}, "
              f"consent pages read: {len(res['consentPages'])}")
        print("MATCHED against the host:")
        print(f"  wallet id  {res['matched']['walletPolicyId']}")
        for x in res["matched"]["addresses"]:
            print(f"  address    {x}")
        print("\nThe device derives this four-tier vault. So the seating arm's refusal,")
        print("if it holds, is about the SEATING and not about the wallet's shape.")
    else:
        print(f"template chunks: {res['chunksPresented']}, "
              f"key cards gathered: {res.get('keyCardsGathered', 0)}")
        print(f"\nREFUSED, in words: {res['refused']!r}")
        print("No address was shown. The device declined to guess which of 720")
        print("assignments the operator meant -- which is the correct answer, and")
        print("the reason the engraved set as it stands cannot be restored from.")


if __name__ == "__main__":
    main()
