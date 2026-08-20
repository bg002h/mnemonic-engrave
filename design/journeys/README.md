# Operator journeys

End-to-end walkthroughs of backing up a wallet with the constellation: host CLIs,
the firmware GUI in the emulator, and the plates that come out.

**Nothing in these documents is illustrative.** Every CLI block is the
transcript script's real stdout+stderr with its actual exit code, every
screenshot is the emulator's own 480×320 framebuffer, and every plate image is a
real render — from `me bundle --preview`, or from the engrave overlay the
emulator draws from the driver's step stream.

## The documents

| file | subject | delivery |
| --- | --- | --- |
| `SeedHammer-II-pathological-wallet-journey.pdf` | the constellation's **pathological example** — 11 keys, all four Bitcoin timelock kinds, a sha256 hashlock | typed on the device, **no NFC** |
| `SeedHammer-II-operator-journey.pdf` | a 5-of-12 `wsh(multi(…))` | NFC attempted; see below |
| `SeedHammer-II-load-payload-journey.pdf` | not a wallet — the **systemwide payload**: pack it, write it to `0x10D00000`, compare the digest across the air gap, use it, unload it, wipe it | host CLI + emulator |

Each has a matching `transcript*.sh` (regenerates every CLI block),
`build_pdf*.py` (regenerates the PDF from the artifacts) and `inputs*/` (the
files the operator supplies).

### Which wallet is "pathological"

The 5-of-12 document was written first, against the wrong wallet. `CORPUS.md`'s
C6 "pathological deeply-nested miniscript" entry marks *itself* a placeholder,
and the 12-key `multi(5,…)` policy taken instead encodes to **13 bytes and one
md1 string** — a keyless BIP-388 template does not pay per key, so it never
forces chunking.

The real one is `mnemonic-toolkit` Examples §5, *"Custom degrading-miniscript
wallet — the pathological example"*: a four-tier vault whose two 32-byte hash
literals and four timelock arguments come to **182 data symbols against a
single-string cap of 80**. That is the first policy in this work that genuinely
needs a chunk set.

The 5-of-12 document is kept because it is still a real journey and still the
one that exercises NFC. It remains pathological in *operator effort* — 26 plates.

### Corrections already folded in

The pathological document's first draft reported `me bundle`'s refusal as a tool
defect and rendered plates by calling the sidecar directly. That was wrong: the
refusal followed from omitting `--path`, which `md` had warned about at encode
time. The published version supplies `--path`, and its plates and checklist come
from `me bundle --preview`.

## Corrections to the published documents

**The two wallet PDFs state things that later measurement contradicted.** They
are kept as published — a journey is a record of a run — and the corrections
live here, because this is where a reader arrives first. Each is a follow-up in
`design/FOLLOWUPS.md` with the measurement behind it, and each is restated in
the Load Payload document's "Corrections to the two earlier journeys" page.

| # | what the PDFs say or imply | what is true |
| --- | --- | --- |
| F-131 | "Recovery: any 3 of 11 signing keys + md1" | Not a 3-of-11. Four tiers, **eight** distinct minimal key-sets, each with its own timelock, two also needing a hash preimage. False in both directions. |
| F-132 | the engraved set is a complete backup | Tiers 1–2 need the 32-byte preimage `X` to spend. Measured: **0** backup strings carry it, and nothing in the checklist mentions it. |
| F-133 | the tiers degrade weakest-last | INVERTED. Tier 4 (1-of-3) matures at 365.00 d; tier 3 (2-of-2) at ≈455 d — the weakest key-set unlocks ~90 days FIRST. |
| F-134 | the wallet is 26 plates | 26, 38 or 58 depending on an md1-form flag the operator is never shown. The default is the 58 one. |
| F-130 | a restored wallet reproduces the descriptor | Keys byte-identical, addresses unaffected — but restored xpubs lose depth/parent/child, so the descriptor string and its checksum differ. |
| F-136 | `md encode` auto-chunks | It does not; it fails and tells you to retry with `--force-chunked`. Two places say otherwise, including the flag's own help. |
| F-137 | — | Encoder has no depth guard where the decoder does, so an encodable-but-undecodable card may be expressible. **Unconfirmed** — carried on the codec lens's authority. |
| F-139 | `CORPUS.md` §C6 is TBD | This wallet is the answer: 182 data symbols against a cap of 80. |
| F-140 | `compare-cost`: taproot is +127..+131 vB | True delta **+1..+6 vB**. One side counts its script and the other does not, so it points the wrong way. |

## Findings these runs produced

- **F-126** — presenting an NFC tag to a gathering flow freezes the emulator.
  All five scan loops yield only on `scanFailed`, so a reader at EOF spins and
  starves Go/wasm's single thread.
- **F-127** — `mk encode --from-md1` cannot read a chunked md1; `mk` vendors
  md-codec 0.34.0 against the primary's 0.42.0.
- **F-128** — the stub's spec sentence names `WalletPolicyId`; `mk` uses the
  template-id.
- **F-129** — `--path` is mandatory for a non-canonical wrapper and flattens
  divergent origins; precedence against the mk1 cards' own origins is unpinned.

From the Load Payload run:

- **F-153** — `me sysw pack`'s refusal named a cause that had not occurred; the
  message is fixed, the 0-based record index is still filed.
- **F-154** — the tenth program's carousel dot is drawn under the firmware
  version line. Measured on the framebuffer: dots run to x≈322, the text starts
  at x≈305.
- **F-155** — the home screen is byte-for-byte identical whether a payload is
  loaded, was unloaded, or was never offered.
- **F-156** — the break below.

## These scripts did not run for a while, and nothing said so

`transcript_pathological.sh` and `build_pdf_pathological.py` both read
`inputs/`, which holds the *other* document's twelve cosigners; the pathological
wallet's files were moved to `inputs-pathological/` when the second document was
added and the scripts were never repointed. `build_pdf_pathological.py` also
read its CSS from `design/journey/build_pdf.py` — a directory that has never
existed — and opened a `keys.json` that was never committed.

All four are fixed (2026-08-12, F-156), and the key captions now come from the
committed `inputs-pathological/keys/*.xpub` headers rather than from an
uncommitted artifact. The lesson is the one the documents themselves are about:
**a claim that nothing here is illustrative decays into a promise the moment the
regeneration path stops being exercised.**

## ⚠ The shipped PDFs are STALE — 2026-08-19

**The two `.pdf` files in this directory predate DoNextList item 5 and no longer
describe the wallet the scripts now produce.** They embed the **retired
depth-3 xpubs** (`m/84'/0'/N'`) and contain **no addresses at all**, because at
the time they were built no address could be derived for this wallet.

They were **not** rebuilt, deliberately: `shots/` has **zero tracked files**, so
the screenshots these documents embed exist nowhere in the repo and a rebuild
produces pages with missing-image placeholders — and **since 2026-08-19 such a
build FAILS with exit 1** rather than reporting success (see "The builds refuse
to lie" below). Regenerating them faithfully
needs the emulator re-walked, which is separate work.

Until then, **the HTML built by the commands below is the current document and
the PDF is not.** This is exactly the decay the paragraph above warns about,
recorded rather than left for a reader to discover.

## The builds refuse to lie — 2026-08-19

`build_pdf.py`, `build_pdf_pathological.py` and `build_pdf_payload.py` **exit 1
when any referenced screenshot or plate is absent**, listing every one. They
used to render a `missing:` placeholder and exit 0.

The constellation recon measured what that cost: **both wallet journeys were
building with 100 % of their screenshots missing — 19/19 and 13/13 — and both
reported success.** A document that silently asserts less than it claims is the
exact decay this README is otherwise about.

Enforcement is the **default**. `--allow-missing` is an explicit opt-out for a
deliberate draft; it still prints the full list, then exits 0. There is no flag
or environment variable that switches the gate *on* — a gate something has to
enable is a gate that is off whenever someone forgets it.

So the commands below are honest now: if they exit 0, every image in the
document is real.

## Reproducing

The pathological journey regenerates END TO END — transcript, screenshots and
PDF — with three commands:

```sh
bash transcript_pathological.sh > transcript_pathological.txt 2>&1
python3 capture_pathological.py     # rebuilds emu.wasm, drives it, writes shots/
python3 build_pdf_pathological.py   # writes the PDF
```

`capture_pathological.py` is what F-210 was missing. This README described
`shot_server.py` as the receiver the emulator posts frames to, and **nothing
posted them**: the capture was console code in a session that no longer exists,
so the PDFs were committed while the process behind their screenshots was not.
The driver is now `cmd/emu/shots_pathological.js` in the `seedhammer` fork,
beside the walk drivers, and the runner above drives it in a real browser.

It rebuilds `emu.wasm` first on purpose — a capture against a stale binary
documents the stale binary — and exits non-zero unless all 13 shots arrive
with content, so a partial capture cannot pass for a whole one.

The operator journey still stops at HTML:

```sh
bash transcript.sh > transcript.txt 2>&1
python3 build_pdf.py                # writes out/journey.html; 19 shots still missing
```

**`build_pdf.py` writes HTML, not PDF**, and has no capture driver yet — the
remaining half of F-156. `build_pdf_payload.py` and `build_pdf_pathological.py`
both print their own:

```sh
mkdir -p out shots
bash transcript_payload.sh > out/transcript_payload.txt 2>&1
python3 build_pdf_payload.py               # writes the PDF, via $CHROME
```

The build scripts expect the artifacts beside them (`out/`, `shots/`), which are
gitignored — the PDFs are the deliverable. `shot_server.py` is the receiver
the emulator page POSTs `canvas.toDataURL()` frames to, which is how the
screenshots are the device framebuffer exactly rather than a cropped browser
window:

```sh
python3 shot_server.py <out-dir> <port> [allowed-origin]   # origin defaults to
                                                           # http://127.0.0.1:8731
```

It accepts frames only from that one origin, and only flat `.png`/`.svg` names
resolving inside `<out-dir>`. Both restrictions matter — see the module docstring
for what the first version of it did instead.

Key derivation for the 5-of-12 document uses `cmd/journeykeys` in the
`seedhammer` fork, which runs the device's own
`bip39.MnemonicSeed → hdkeychain.NewMaster → bip32.Derive → Neuter` path.

## Test material

Every seed in both documents is public by construction — BIP-39's own published
test vectors, or derived deterministically from a published string. **Never put
funds behind them.**
