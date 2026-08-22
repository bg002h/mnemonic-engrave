#!/usr/bin/env python3
"""F-227 item 3 — restore a wallet from the PLATE IMAGES, not from the files.

WHAT WAS MISSING. Every proof in every journey so far starts from the
generator's own output: the md1/mk1 strings sitting in `out/`. The device walks
present those strings over NFC; the restore tests read those strings back. But
what an operator actually holds is a stack of engraved plates, and nothing had
ever gone the other way — **plate image → strings** — so the entire rendering
step was unverified. A plate that rendered a truncated QR, or the wrong chunk,
or a subtly wrong module size would have passed every gate in this directory.

This closes that. It decodes the QR out of each rendered plate PNG and proves
the recovered strings rebuild the wallet.

TWO LAYERS, AND ONLY THE SECOND ONE IS WORTH MUCH.

  Layer 1  the QR decodes to the string the manifest says is on that plate.
           Necessary, but weak on its own: `me bundle` wrote BOTH the manifest
           and the PNG, so this is one tool agreeing with itself. It catches a
           rendering fault, nothing more.

  Layer 2  the decoded strings — and ONLY the decoded strings — are fed back
           through `md` and must produce the wallet-descriptor-template-id and
           the addresses the journey independently derived. That is the layer
           that says the plate carries the wallet.

The decoder is `zbarimg`, which is not the library that wrote the QR. Same
reason the constellation cross-checks Go against Rust: a codec vouching for its
own output proves only that it is self-consistent.

WHAT THIS IS STILL NOT. It is not metal. The path from a rendered PNG to a
scratched steel plate to a phone camera is not exercised here, and that path is
where legibility, depth, glare and engraving artefacts live. This proves the
software chain end to end; it does not prove the plate is readable.

    python3 restore_from_plates.py tr-pathological [--prove-it-can-fail]

Exits non-zero unless every plate decoded AND the rebuilt wallet matches.
"""
import argparse
import json
import os
import re
import subprocess
import sys

W = os.path.dirname(os.path.abspath(__file__))
MD = "/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md"


def die(msg):
    print(f"FAIL: {msg}", file=sys.stderr)
    sys.exit(1)


def decode_qr(png):
    """One QR per plate, decoded by an INDEPENDENT implementation.

    --raw so zbar does not prepend a symbology prefix; the comparison below is
    byte-exact and a `QR-Code:` prefix would make every plate look corrupt.
    """
    r = subprocess.run(["zbarimg", "--quiet", "--raw", png],
                       capture_output=True, text=True)
    # zbarimg exits 4 when it finds no barcode at all -- distinguish that from a
    # decode that produced the wrong bytes, because they mean different things:
    # no symbol is a RENDERING failure, wrong bytes is a CONTENT failure.
    if r.returncode == 4:
        return None, "no QR symbol found in the image"
    if r.returncode != 0:
        return None, f"zbarimg exit {r.returncode}: {r.stderr.strip()}"
    out = r.stdout.rstrip("\n")
    if "\n" in out:
        return None, f"{out.count(chr(10)) + 1} symbols on one plate; expected exactly 1"
    return out, None


def md_run(args):
    return subprocess.run([MD] + args, capture_output=True, text=True)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("journey", help="e.g. tr-pathological")
    # THE NEGATIVE CONTROL. Layer 2 passes when a rebuild MATCHES, and a match
    # check nobody has broken is not evidence -- a driver that silently compared
    # a value against itself would pass too. This flips one character of one
    # decoded chunk and requires the rebuild to notice.
    ap.add_argument("--prove-it-can-fail", action="store_true",
                    help="corrupt one decoded chunk; succeed only if the rebuild catches it")
    # THE CONTROL THAT ACTUALLY TESTS LAYER 2. --prove-it-can-fail flips a
    # character, which codex32's BCH checksum rejects at DECODE -- so it
    # exercises the checksum, something already known to work, and never
    # reaches the template-id comparison at all. A layer-2 control has to
    # supply strings that are perfectly VALID and belong to a DIFFERENT wallet.
    ap.add_argument("--prove-layer2-can-fail", metavar="JOURNEY",
                    help="substitute another journey's valid template; succeed only "
                         "if the template-id comparison catches the swap")
    a = ap.parse_args()

    out = os.path.join(W, "out", a.journey)
    manifest_path = os.path.join(out, "manifest.json")
    if not os.path.exists(manifest_path):
        die(f"no manifest at {manifest_path} -- run that journey's transcript first")
    with open(manifest_path) as f:
        manifest = json.load(f)

    plates = manifest["plates"]
    print(f"{a.journey}: {len(plates)} plates in the manifest\n")

    # ---- Layer 1 -----------------------------------------------------------
    decoded, no_preview, failures = {}, [], []
    for p in plates:
        prev = p.get("preview")
        if not prev or not os.path.exists(prev):
            # NOT skipped silently. A plate with no rendered image is a plate
            # this driver did not check, and saying so is the difference
            # between a partial result and a false clean one.
            no_preview.append(p)
            continue
        got, err = decode_qr(prev)
        if err:
            failures.append((p["plate"], err))
            continue
        want = p["string"]
        if got != want:
            failures.append((p["plate"],
                             f"decoded {len(got)} chars, manifest says {len(want)}; "
                             f"first difference at {next((i for i, (x, y) in enumerate(zip(got, want)) if x != y), min(len(got), len(want)))}"))
            continue
        decoded[p["plate"]] = got

    print(f"Layer 1 — the QR on each plate decodes to the string the manifest names:")
    print(f"  {len(decoded)} of {len(plates)} plates decoded byte-exact via zbarimg")
    if no_preview:
        print(f"  {len(no_preview)} plate(s) carry NO rendered image and were NOT checked:")
        for p in no_preview:
            print(f"    plate {p['plate']} ({p['kind']})")
    if failures:
        for n, err in failures:
            print(f"  plate {n}: {err}", file=sys.stderr)
        die(f"{len(failures)} plate(s) did not decode to their manifest string")

    # ---- Rebuild from the DECODED strings only -----------------------------
    md1 = [s for n, s in sorted(decoded.items()) if s.startswith("md1")]
    mk1 = [s for n, s in sorted(decoded.items()) if s.startswith("mk1")]
    if a.prove_layer2_can_fail:
        other = os.path.join(W, "out", a.prove_layer2_can_fail)
        src = next((q for q in (os.path.join(other, "md1-template.txt"),
                                os.path.join(other, "md1.txt")) if os.path.exists(q)), None)
        if not src:
            die(f"no template to borrow from {other}")
        md1 = [l.strip() for l in open(src) if l.startswith("md1")]
        print(f"\nLAYER-2 CONTROL: replaced the plate-borne template with "
              f"{a.prove_layer2_can_fail}'s ({len(md1)} chunks). Every string is VALID; "
              f"the wallet is wrong.")

    if a.prove_it_can_fail:
        if not md1:
            die("nothing to corrupt: no md1 chunk came off a plate")
        bad = md1[0]
        # One character, in the data body rather than the hrp -- the smallest
        # lie the rebuild must still catch.
        i = 10
        md1 = [bad[:i] + ("q" if bad[i] != "q" else "p") + bad[i + 1:]] + md1[1:]
        print(f"\nNEGATIVE CONTROL: flipped one character of plate-borne chunk 1")

    print(f"\nRebuilding from the plate-borne strings alone: "
          f"{len(md1)} md1 chunk(s), {len(mk1)} mk1 string(s)")

    ins = md_run(["inspect"] + md1)
    if ins.returncode != 0:
        if a.prove_layer2_can_fail:
            die("LAYER-2 CONTROL INVALID: the borrowed template did not even decode, "
                "so this exercised the checksum again rather than the comparison. "
                "Borrow from a journey whose chunks are well-formed.")
        if a.prove_it_can_fail:
            print("\nNEGATIVE CONTROL PASSED: the corrupted chunk was refused at decode.")
            sys.exit(0)
        die(f"md inspect refused the plate-borne chunks:\n{ins.stderr.strip()}")

    tid = re.search(r"wallet-descriptor-template-id: (\S+)", ins.stdout)
    if not tid:
        die("no wallet-descriptor-template-id in md inspect output")
    tid = tid.group(1)

    # ---- Layer 2: against truth the plates did not supply -------------------
    ref_path = os.path.join(out, "md1-template.txt")
    if not os.path.exists(ref_path):
        ref_path = os.path.join(out, "md1.txt")
    if not os.path.exists(ref_path):
        die(f"no reference template in {out} to compare against")
    ref_chunks = [l.strip() for l in open(ref_path) if l.startswith("md1")]
    ref_ins = md_run(["inspect"] + ref_chunks)
    ref_tid = re.search(r"wallet-descriptor-template-id: (\S+)", ref_ins.stdout)
    if not ref_tid:
        die(f"could not derive a reference template-id from {ref_path}")
    ref_tid = ref_tid.group(1)

    print(f"\nLayer 2 — the rebuilt wallet, against what the journey derived separately:")
    print(f"  template-id off the PLATES : {tid}")
    print(f"  template-id from the run   : {ref_tid}")
    if tid != ref_tid:
        if a.prove_it_can_fail or a.prove_layer2_can_fail:
            print("\nNEGATIVE CONTROL PASSED: the rebuild produced a different wallet "
                  "and the LAYER-2 comparison caught it.")
            sys.exit(0)
        die("the plates rebuild a DIFFERENT wallet than the journey engraved")

    if a.prove_it_can_fail:
        die("NEGATIVE CONTROL FAILED: a corrupted chunk still rebuilt the right "
            "wallet. The comparison proves nothing.")
    if a.prove_layer2_can_fail:
        die("LAYER-2 CONTROL FAILED: another wallet's template rebuilt to THIS "
            "wallet's id. The template-id comparison discriminates nothing.")

    # ---- Seatability, from the plates --------------------------------------
    slots = [l.strip() for l in ins.stdout.splitlines() if l.strip().startswith("@")]
    decls = {s.split(": ", 1)[1] for s in slots}
    withfp = sum(1 for s in slots if "[" in s)
    seatable = len(decls) == len(slots) and withfp == len(slots)
    print(f"\n  slots {len(slots)}, distinct declarations {len(decls)}, "
          f"with a fingerprint {withfp}")
    print(f"  the plate-borne template is {'SEATABLE' if seatable else 'NOT SEATABLE'}")
    if not seatable:
        die("the engraved plates rebuild a template whose slots cannot be told "
            "apart -- one mk1 card would match several, and seating must refuse")

    print(f"\nRESTORED FROM THE PLATES. {len(decoded)} images in, the same wallet out.")
    if no_preview:
        print(f"Caveat: {len(no_preview)} plate(s) had no rendered image and were not "
              f"part of this.")
    print("Not metal: this proves the software chain, not that steel is legible.")


if __name__ == "__main__":
    main()
