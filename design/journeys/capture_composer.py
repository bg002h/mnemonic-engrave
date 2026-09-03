#!/usr/bin/env python3
"""Capture the WALLET POLICY COMPOSER journey's device half, and CHECK it against the host.

The host half (transcript_composer.sh) composed the same two wallets in Rust and
wrote every id, address and engraved string into out/composer/. This drives the
emulator through the composer's own screens -- building the policy on the device,
from a payload the device loaded -- and reads back what SeedHammer II shows.

It is a comparison, not a photo shoot. cmd/emu/shots_composer.js is handed the
host's digest, ids, addresses and md1/mk1 strings and throws if the screen or the
engraved plate disagrees, so a green run means the device and the host agree
across the air gap -- through two implementations, in two languages, from the
same two BIP-39 vectors.

THREE COMPARISONS, and the third is the one nothing else can make:

  1. the consent screen's Policy-ID and all four addresses;
  2. the census screen's plate claim against the plates actually cut;
  3. THE ENGRAVED STRINGS, BYTE FOR BYTE. The keyless template encodes on the
     host as a 47-character UNCHUNKED string and a 56-character CHUNKED one, and
     `md verify`, `md inspect` and `md decode` accept both identically while the
     DEVICE is chunk-form-always. No verify step can tell them apart.

    python3 capture_composer.py [--arm keyed|keyless|both] [--port 8793] [--shot-port 8734]

Exits non-zero unless every expected shot arrived AND every comparison passed, so
a partial or disagreeing capture cannot be mistaken for a complete one.
"""
import argparse, asyncio, functools, http.server, json, os, socketserver
import subprocess, sys, threading, time

W = os.path.dirname(os.path.abspath(__file__))
# The fork's emulator. Overridable because this script is run from a WORKTREE at
# least as often as from the main checkout, and the relative default resolves to
# the main one -- which is the wrong tree while a fork change is on a branch.
EMU = os.environ.get("EMU") or os.path.abspath(
    os.path.join(W, "..", "..", "..", "seedhammer", "cmd", "emu"))
SHOTS = os.path.join(W, "shots")
OUT = os.path.join(W, "out", "composer")

# The fixed shots per arm. The paged screens contribute a shot per page and the
# count is not known ahead of time, so the driver's own return value carries
# those -- but the number of PAGES is asserted inside the driver, never here.
EXPECTED = {
    "keyed": ["c00a-boot-offer.png", "c01-payload-digest.png", "c02-door.png",
              "c03-start-from.png", "c04-lock-echo-p0.png", "c05-hash-rule.png",
              "c07-seat-slot0.png", "c08-seat-slot2-seed.png"],
    "keyless": ["c00a-boot-offer.png", "k01-door.png", "k04-census.png"],
}


def need(name):
    """Load one artifact the host half wrote. A MISSING file is fatal, deliberately.

    F-210's failure was a journey step reading an intermediate that nothing
    committed produced, so the run only ever worked when a previous session had
    left the file behind. Every path here is written by transcript_composer.sh in
    the same directory, and if it is absent the answer is to run that script, not
    to carry on with a default.
    """
    p = os.path.join(OUT, name)
    if not os.path.exists(p):
        sys.exit(f"missing {p}\nRun ./transcript_composer.sh first -- it writes this.")
    return [l.strip() for l in open(p) if l.strip()]


def field(lines, key):
    for line in lines:
        if line.startswith(key + ":"):
            return line.split(":", 1)[1].strip()
    sys.exit(f"no {key} in the host's id file")


def read_keyed():
    """The keyed arm's oracle: the digest, both ids, four addresses, both forms."""
    ids = need("keyed.id.txt")
    template_id = field(ids, "wallet-descriptor-template-id")
    policy_id = field(ids, "wallet-policy-id")
    digest = " ".join(need("payload.digest.txt")[0].split()[1:])
    recv, chg = need("keyed.receive.txt"), need("keyed.change.txt")
    policy = need("keyed.md1.txt")
    template = need("keyed-template.md1.txt")
    cards = [need(os.path.join("cards", f"slot{i}.mk1.txt")) for i in (0, 1, 2)]

    base = {
        "digest": digest,
        "templateId": template_id,
        # THE STUBS ARE THE FIRST FOUR BYTES OF THE IDS, derived here rather than
        # pinned: a stub written down separately is a second source of truth for
        # a value that has exactly one.
        "templateStub": template_id[:8],
        "policyId": policy_id,
        "policyStub": policy_id[:8],
        "addresses": recv[:2] + chg[:2],
    }
    forms = {
        # Form A: the keyed policy alone. 7 chunks pack onto 2 plates (F-423).
        "A": dict(base, entries=2, strings=policy, censusLines=[
            "This engraves 2 plates.",
            "md1 policy: 2 plates (the wallet policy, with its keys)"]),
        # Form B: the key-less template plus one plate per key card.
        "B": dict(base, entries=4, strings=template + cards[0] + cards[1] + cards[2],
                  censusLines=[
                      "This engraves 4 plates.",
                      "md1 template: 1 plate (key-less wallet policy)",
                      "mk1 key @0: 1 plate (m/48'/0'/0'/2')",
                      "mk1 key @1: 1 plate (m/48'/0'/1'/2')",
                      "mk1 key @2: 1 plate (m/48'/0'/0'/2')"]),
    }
    return forms


def read_keyless():
    ids = need("keyless-tr.id.txt")
    template_id = field(ids, "wallet-descriptor-template-id")
    strings = need("keyless-tr.md1.txt")
    return {
        "templateId": template_id,
        "templateStub": template_id[:8],
        "entries": 1,
        "strings": strings,
    }


def build_wasm():
    print(f"building emu.wasm in {EMU} ...", flush=True)
    env = dict(os.environ)
    env.setdefault("TMPDIR", "/scratch/code/shibboleth/.tmp")
    r = subprocess.run(["sh", "build.sh"], cwd=EMU, capture_output=True, text=True, env=env)
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


async def drive(port, shot_port, legs):
    """Run each leg on a FRESH PAGE.

    No leg inherits another's device state: the composer keeps a loaded payload
    and a built shape in the session, and a second walk starting from the first
    one's door would be walking a machine no operator has.
    """
    from playwright.async_api import async_playwright
    url = f"http://127.0.0.1:{port}/index.html"
    results = {}
    async with async_playwright() as pw:
        browser = await pw.chromium.launch()
        for name, arm, form, expect in legs:
            page = await browser.new_page()
            errors = []
            page.on("console", lambda m: errors.append(m.text) if m.type == "error" else None)
            page.on("pageerror", lambda e: errors.append(str(e)))
            await page.goto(url)
            await page.wait_for_function("window.shScreen !== undefined", timeout=60_000)
            await page.wait_for_function("window.shTargets !== undefined", timeout=60_000)
            await page.wait_for_timeout(2500)
            print(f"--- leg {name} ---", flush=True)
            try:
                res = await page.evaluate(
                    """async ({shotURL, arm, form, expect}) => {
                         const m = await import("./shots_composer.js");
                         return await m.run({ shotURL, arm, form, expect });
                       }""",
                    {"shotURL": f"http://127.0.0.1:{shot_port}/",
                     "arm": arm, "form": form, "expect": expect},
                )
            except Exception as e:
                print(f"DRIVER FAILED on leg {name}:", e, file=sys.stderr)
                if errors:
                    print("page errors:", errors[:5], file=sys.stderr)
                await page.close()
                await browser.close()
                return None
            await page.close()
            if errors:
                print("page errors (non-fatal):", errors[:5], file=sys.stderr)
            results[name] = res
        await browser.close()
    return results


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--arm", choices=["keyed", "keyless", "both"], default="both")
    # THE NEGATIVE CONTROL, AS A COMMAND. A comparison nobody has made fail is
    # not evidence it can. This corrupts ONE character of ONE expected address
    # and requires the walk to notice -- exit 0 only if the capture FAILED.
    ap.add_argument("--prove-it-can-fail", action="store_true",
                    help="corrupt one expected address; succeed only if the walk catches it")
    ap.add_argument("--port", type=int, default=8797)
    ap.add_argument("--shot-port", type=int, default=8738)
    ap.add_argument("--emu", default=None, help="the fork's cmd/emu (default: the sibling checkout)")
    ap.add_argument("--no-build", action="store_true")
    a = ap.parse_args()

    global EMU
    if a.emu:
        EMU = os.path.abspath(a.emu)
    if not os.path.isdir(EMU):
        sys.exit(f"no emulator at {EMU}\nPass --emu <fork>/cmd/emu or set EMU=.")

    if a.prove_it_can_fail and a.arm != "keyed":
        sys.exit("--prove-it-can-fail corrupts an ADDRESS, which only the keyed arm has; "
                 "run it with --arm keyed.")

    legs = []
    if a.arm in ("keyed", "both"):
        forms = read_keyed()
        if a.prove_it_can_fail:
            # Flip the LAST character of the first address: a one-character lie
            # is the smallest thing the comparison must still catch.
            bad = forms["A"]["addresses"][0]
            forms["A"] = dict(forms["A"], addresses=[
                bad[:-1] + ("q" if bad[-1] != "q" else "p")] + forms["A"]["addresses"][1:])
            print(f"NEGATIVE CONTROL: expecting {forms['A']['addresses'][0]} (corrupted)")
            legs.append(("keyed-A", "keyed", "A", forms["A"]))
        else:
            legs.append(("keyed-A", "keyed", "A", forms["A"]))
            legs.append(("keyed-B", "keyed", "B", forms["B"]))
        print(f"host: policy id {forms['A']['policyId']}, template id {forms['A']['templateId']}")
        for x in forms["A"]["addresses"]:
            print(f"      {x}")
    if a.arm in ("keyless", "both"):
        kl = read_keyless()
        legs.append(("keyless", "keyless", None, kl))
        print(f"host: key-less template id {kl['templateId']}, "
              f"{len(kl['strings'])} md1 string(s), {len(kl['strings'][0])} chars")

    print(f"emulator: {EMU}")
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
        res = asyncio.run(drive(a.port, a.shot_port, legs))
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

    missing = []
    for name, r in res.items():
        want = list(EXPECTED[r["arm"]]) + [n for n in r.get("shots", []) if n not in EXPECTED[r["arm"]]]
        missing += [f"{name}:{n}" for n in want if bad(n)]
    if missing:
        sys.exit(f"capture INCOMPLETE -- missing or empty: {missing}")

    with open(os.path.join(SHOTS, "composer-result.json"), "w") as f:
        json.dump(res, f, indent=2)

    for name, r in res.items():
        print(f"\n===== leg {name} ({r['elapsedSec']}s) =====")
        print(f"  shots: {len(r['shots'])}   census claim: {r['censusClaim']} plate(s)")
        m = r["matched"]
        if r["arm"] == "keyed":
            print(f"  payload digest  {m['digest']}")
            print(f"  Template-ID     {m['templateId']}   stub {m['templateStub']}")
            print(f"  Policy-ID       {m['policyId']}   stub {m['policyStub']}")
            for x in m["addresses"]:
                print(f"  address         {x}")
        else:
            print(f"  Template-ID     {m['templateId']}   stub {m['templateStub']}")
        print(f"  ENGRAVED, byte for byte against the host ({len(m['strings'])} string(s)):")
        for s in m["strings"]:
            print(f"    {s}")
    print("\nall legs matched the host.")


if __name__ == "__main__":
    main()
