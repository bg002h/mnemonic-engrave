#!/usr/bin/env python3
"""Drive the REASONABLY COMPLEX WALLET's device half — both wrappers, both routes.

The host half (transcript_rcw.sh) built two wallets from six seeds: a taproot
form at m/270028'/0'/8'/0' and a wsh form at m/270028'/0'/9'/1'. This carries
each one's cards into the SeedHammer II emulator over NFC and requires the
screen to agree with what `md` derived in Rust.

FOUR ARMS, two per wrapper:

  --route keyed    the 15-chunk KEYED card alone. No seating; the card carries
                   its own keys. Proves the device can DERIVE this wallet's
                   addresses at all.
  --route seating  the 5-chunk keyless template plus its six mk1 key cards, which
                   is what actually gets engraved. Proves the device can JOIN
                   them — the fingerprints are declared (F-227), so the six cards
                   seat uniquely rather than being refused as contested.

Both arms must produce the SAME addresses, because they are the same wallet
reached two ways. That equality is the point: it is the air gap crossed twice,
through two implementations in two languages.

    python3 capture_rcw.py --wrapper tr --route seating [--prove-it-can-fail]

Exits non-zero unless every expected shot arrived AND the comparison passed.
"""
import argparse, asyncio, functools, http.server, json, os, socketserver
import subprocess, sys, threading, time

W = os.path.dirname(os.path.abspath(__file__))
EMU = os.path.abspath(os.path.join(W, "..", "..", "..", "seedhammer", "cmd", "emu"))
SHOTS = os.path.join(W, "shots")
OUT = os.path.join(W, "out", "rcw")

# Shot prefixes, one per (wrapper, route). shots/ is one flat directory shared
# by every journey, so a collision here would silently overwrite another arm's
# frames — the defect that made shots_seating.js take a prefix in the first
# place. Two characters, because four arms need four names.
PREFIX = {("tr", "keyed"): "tk", ("tr", "seating"): "ts",
          ("wsh", "keyed"): "wk", ("wsh", "seating"): "ws"}

# TWO PER CHAIN IS SUFFICIENT — operator ruling 2026-08-22: "Two addresses is
# fine, we don't need 5." This is settled; do not re-open it as a coverage gap.
#
# The device shows addrProofPerChain = 2 (gui/wallet_policy.go:184, plan D6),
# pinned by a test (gui/wallet_policy_test.go:33, "want 2 (plan D6)"), and the
# firmware states the reason at the constant: a chain mismatch silently loses
# funds, so proving BOTH CHAINS derive beats proving one chain five times. A
# backdoored screen defeats five addresses exactly as easily as two; the extra
# three buy reading fatigue on a consent surface and no extra evidence.
#
# The host still derives five, deliberately: the device proves a SAMPLE and the
# host proves the RANGE. The comparison asserts the device's four values are
# byte-equal to host indices 0-1 per chain, rather than quietly comparing fewer
# and calling it agreement.
DEVICE_PER_CHAIN = 2


def read_artifacts(wrapper):
    """Load what transcript_rcw.sh wrote for this wrapper. Missing is fatal.

    F-210's failure was a journey step reading an intermediate that nothing
    committed produced, so it only ever worked when a previous session had left
    the file behind. Every path here is written by transcript_rcw.sh.
    """
    d = os.path.join(OUT, wrapper)

    def need(name):
        p = os.path.join(d, name)
        if not os.path.exists(p):
            sys.exit(f"missing {p}\nRun `bash transcript_rcw.sh` first -- it writes this.")
        return p

    template = [l.strip() for l in open(need("md1-template.txt")) if l.strip()]
    keyed = [l.strip() for l in open(need("md1-keyed.txt")) if l.strip()]
    with open(need("mk-encode.json")) as f:
        cards = [c["mk1_strings"] for c in json.load(f)["cards"]]
    recv = [l.strip() for l in open(need("receive.txt")) if l.strip()]
    chg = [l.strip() for l in open(need("change.txt")) if l.strip()]

    # THE PREMISE, ASSERTED. This wallet's six slots share one origin path and
    # are told apart ONLY by their declared fingerprints. If a future edit drops
    # them, the seating arm would start failing for a reason that has nothing to
    # do with what it tests — so say so here rather than debugging a refusal.
    ins = subprocess.run(
        ["/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md", "inspect", *template],
        capture_output=True, text=True)
    slots = [l.strip() for l in ins.stdout.splitlines() if l.strip().startswith("@")]
    if slots and not all("[" in s for s in slots):
        sys.exit(f"PREMISE BROKEN: {wrapper}'s template declares no fingerprint on some slot.\n"
                 f"Every slot shares one origin path here, so without fingerprints the six "
                 f"cards are indistinguishable and seating MUST refuse (F-227). Re-run the "
                 f"transcript, which passes --fingerprint per slot.")

    if len(recv) < DEVICE_PER_CHAIN or len(chg) < DEVICE_PER_CHAIN:
        sys.exit(f"{wrapper}: need at least {DEVICE_PER_CHAIN} addresses per chain, "
                 f"got {len(recv)} receive / {len(chg)} change")
    return template, keyed, cards, recv[:DEVICE_PER_CHAIN] + chg[:DEVICE_PER_CHAIN]


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


async def drive(port, shot_port, md1, key_cards, addresses, prefix):
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
            # Reuses shots_seating.js: its run() already models a template plus
            # optional key cards, which is exactly both routes here.
            res = await page.evaluate(
                """async ({shotURL, md1, keyCards, expect, prefix}) => {
                     const m = await import("./shots_seating.js");
                     return await m.run({ shotURL, md1, keyCards, expect, prefix });
                   }""",
                {"shotURL": f"http://127.0.0.1:{shot_port}/",
                 "md1": md1,
                 "keyCards": key_cards,
                 "expect": {"addresses": addresses},
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
    ap.add_argument("--wrapper", choices=["tr", "wsh"], required=True)
    ap.add_argument("--route", choices=["keyed", "seating"], required=True)
    ap.add_argument("--port", type=int, default=8801)
    ap.add_argument("--shot-port", type=int, default=8742)
    ap.add_argument("--no-build", action="store_true")
    # THE NEGATIVE CONTROL. The comparison passes when addresses APPEAR on the
    # screen, and an appearance check nobody has broken is not evidence -- a
    # walk that stopped driving the device would pass it too. This corrupts ONE
    # character of ONE expected address and requires the walk to notice.
    ap.add_argument("--prove-it-can-fail", action="store_true",
                    help="corrupt one expected address; succeed only if the walk catches it")
    a = ap.parse_args()

    template, keyed, cards, addresses = read_artifacts(a.wrapper)
    md1 = keyed if a.route == "keyed" else template
    key_cards = [] if a.route == "keyed" else cards
    prefix = PREFIX[(a.wrapper, a.route)]

    if a.prove_it_can_fail:
        bad = addresses[0]
        addresses = [bad[:-1] + ("q" if bad[-1] != "q" else "p")] + addresses[1:]
        prefix = "n" + prefix[0]
        print(f"NEGATIVE CONTROL: expecting {addresses[0]} (corrupted)")

    print(f"{a.wrapper} / {a.route}: {len(md1)} md1 chunk(s), {len(key_cards)} key card(s)")
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
        res = asyncio.run(drive(a.port, a.shot_port, md1, key_cards, addresses, prefix))
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
            print("\nNEGATIVE CONTROL PASSED: the walk refused an address the host never derived.")
            sys.exit(0)
        sys.exit("NEGATIVE CONTROL FAILED: the walk accepted an address the host never "
                 "derived. The comparison proves nothing.")

    if res is None:
        sys.exit("capture failed: the driver did not return -- see DRIVER FAILED above")

    # SIZE, NOT EXISTENCE. A canvas that fails to rasterise yields "data:," which
    # the receiver writes as a zero-byte PNG with a 200 OK, so a driver can
    # report success for files holding no image.
    def bad(n):
        p = os.path.join(SHOTS, n)
        return not os.path.exists(p) or os.path.getsize(p) < 512

    missing = [n for n in res.get("shots", []) if bad(n)]
    if missing:
        sys.exit(f"capture INCOMPLETE -- missing or empty: {missing}")

    tag = f"rcw-{a.wrapper}-{a.route}"
    with open(os.path.join(SHOTS, f"{tag}-result.json"), "w") as f:
        json.dump(res, f, indent=2)

    print(f"\ncaptured {len(res.get('shots', []))} shots into {SHOTS}")
    print(f"chunks presented: {res['chunksPresented']}, "
          f"key cards gathered: {res.get('keyCardsGathered', 0)}")
    print("MATCHED against the host:")
    for x in res["matched"]["addresses"]:
        print(f"  {x}")


if __name__ == "__main__":
    main()
