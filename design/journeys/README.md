# Operator journeys

End-to-end walkthroughs of backing up a wallet with the constellation: host CLIs,
the firmware GUI in the emulator, and the plates that come out.

**Nothing in these documents is illustrative.** Every CLI block is the
transcript script's real stdout+stderr with its actual exit code, every
screenshot is the emulator's own 480×320 framebuffer, and every plate image is a
real render — from `me bundle --preview`, or from the engrave overlay the
emulator draws from the driver's step stream.

## The documents

| file | wallet | delivery |
| --- | --- | --- |
| `SeedHammer-II-pathological-wallet-journey.pdf` | the constellation's **pathological example** — 11 keys, all four Bitcoin timelock kinds, a sha256 hashlock | typed on the device, **no NFC** |
| `SeedHammer-II-operator-journey.pdf` | a 5-of-12 `wsh(multi(…))` | NFC attempted; see below |

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

## Reproducing

```sh
bash transcript_pathological.sh > transcript_pathological.txt 2>&1
python3 build_pdf_pathological.py
```

The build scripts expect the artifacts beside them (`out/`, `shots/`), which are
not committed — the PDFs are the deliverable. `shot_server.py` is the receiver
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
