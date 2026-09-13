# Continuity — F-531 DONE, F-530 next, before the flash

**Updated 2026-09-13.** Everything below is measured, not recalled.
Supersedes the pre-F-531 version of this file (see `git log` on this path).

## Where the repos are

| repo | branch | tip | state |
| --- | --- | --- | --- |
| `mnemonic-engrave` | `master` | `7ea49ac3` | see push status below |
| `seedhammer` (fork) | `main` | `9b36ed7` | see push status below |
| `descriptor-mnemonic` | `main` | `40c400de` | pushed, unchanged this session |

## F-531 — CLOSED, GREEN

Two address routes returned different addresses for a repeated-slot multisig.
**The device now derives no address for a policy that repeats a key slot inside
one script expression**, on either route.

**Measured before deciding.** Bitcoin Core 25.0.0, throwaway regtest datadir,
`getdescriptorinfo` + `deriveaddresses`, keys version-swapped to tpub, datadir
deleted after. Core ACCEPTS all four — the shape imports and funds:

| descriptor | address | which route |
| --- | --- | --- |
| `wsh(sortedmulti(1,A,A,B))` | `bcrt1qvljqpug…qqqkckr` | the emitter |
| `wsh(sortedmulti(1,A,B))` | `bcrt1q2gu6t4m…s0ufenu` | **the flat route showed this** |
| `wsh(sortedmulti(2,A,A,B))` | `bcrt1qaej2r8z…qs9h244` | the emitter |
| `wsh(sortedmulti(2,A,B))` | `bcrt1qsl0stsx…qqz65am` | **the flat route showed this** |

So the flat route was not approximately wrong — it was answering the two-seat
question. A faithful flat descriptor **was** measured to work (repeat the key in
`Keys`, Core's address comes back) and was deliberately not shipped: the
operator's ruling is that md does not serve BIP-388-forbidden wallets.

**The primary already agreed** — `md address` refuses, `md decode` reads and
warns. The corpus gate recorded the convergence itself, moving
`keyed_wsh_timelock_hashlock` from `{device: ok, rust: refused}` to
`{device: source}`.

Commits, in order: fork `a4760e1` (fix) → engrave `a832433b` (spec + baseline)
→ `584051f0` (follow-ups) → `cea4a63b` (review, verbatim, NOT GREEN 0C/3I) →
fork `9b36ed7` (fold) → engrave `9db77cf5` (records fold) → `81ca436d`
(verification, verbatim, GREEN) → `7ea49ac3` (M-5 residue).

**The two findings worth remembering**, both from the adversarial review:

- **I-2 would have reached steel.** The reuse warning was an *arm* of
  `noAddressLines` placed after the keyless ones, so a KEYLESS repeated-seat
  template — which `TemplateEngraveShapeGuardChunks` admits — reached the
  Engrave consent saying "Template has no keys - no addresses." and nothing
  about the reuse, under a "1-of-3" label with two slots beneath it. Reuse is a
  fact about the CARD; keylessness is a fact about what can be derived from it.
  The warning is a PREFIX now, and the inspect announcement is hoisted above the
  routing because `expandTemplateOnly` had no modal at all.
- **I-1: the fix made its own gate unfailable.** All three F-514 warning blocks
  became unreachable (`expandOK ⟹ DuplicateNone`), while the comment written in
  the same commit asserted they were covered. Deleting all three left 1312/1312
  green. They are gone, each site noting that F-533's remedy would make them
  live again with no coverage.

## F-530 — next, and smaller than it looked

`descriptorFlow` → `DescriptorScreen.Confirm` → `descriptorAddressFlow` shows
addresses with no duplicate-key warning.

**Measured this session** (read-only, no code written yet):

- **Three callers, and TWO of them carry no md1** — not one, as the earlier
  version of this file said. `gui/gui.go:2599` (a scanned `*bip380.Descriptor`),
  `gui/wallet_policy.go:122` (`nonstandard.OutputDescriptor` over a payload
  record), and `gui/md1_gather.go:177` (the only chunk-bearing one, and already
  covered by F-531's gates). So the rule must be over a `*bip380.Descriptor`.
- **`bip380.Parse` has no duplicate check** (`bip380/bip380.go:295-350`); nor do
  the BlueWallet / JSON / bare-key arms of `nonstandard.OutputDescriptor`.
- **One choke point**: `supported := address.Supported(s.Descriptor)` at
  `gui/gui.go:3238` gates the Addresses button for all three callers.

**Proposed shape** (not yet reviewed): a predicate over `*bip380.Descriptor` —
two `Keys` entries equal in `KeyData` + `ChainCode` + `Children`. Origin metadata
(`MasterFingerprint`, `DerivationPath`) is deliberately EXCLUDED: only the
derived pubkey reaches the script. Including `Children` is what keeps a BIP-388
-legal disjoint multipath (`/0/*` vs `/1/*`) from being called a duplicate.

**One thing to settle first**: `verifyAddressFlow` (`gui/gui.go:3251`) is a
second consumer of the same descriptor. Left alone it would confirm an address
the device has just declined to derive.

`scriptForTemplate`'s doc comment already states what F-531 left of F-530 —
read it before starting.

## Also open

- **F-533** (new this session): `keyed_tr_multi_a` and `keyed_tr_sortedmulti_a`
  still derive on-device while the primary refuses them — the corpus gate's
  remaining `ok/refused 2`. They survive because the refusal rides
  `md.DuplicateKeySlot`, which answers CORE's question by design. The operator's
  stated reason for refusing fits these HARDER than the shape F-531 fixed: both
  seats of a repeated-seat multisig sign the same sighash, whereas a key that is
  both internal key and leaf key signs a key-path sighash and a script-path
  sighash. Closing it needs a SECOND predicate for BIP 388's rule, not a wider
  read of the Core one.
- **F-529**, **F-532** (M-7 and M-4 remain; M-3 and the in-file half of M-8 were
  taken during F-531 because they sat inside the comment block it rewrote).

Operator-owned and unchanged: the flash, the H4 walk, ACCEPTANCE item 8.

## Gates to run

```sh
# fork
scripts/gui-shard-test.sh ./gui/ 24        # from mnemonic-engrave, run in the fork
go test $(go list ./... | grep -v "/gui$")
scripts/fork-vet-gate.sh
nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 \
  -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller

python3 scripts/policy-generate.py --corpus
```

Last measured at fork `9b36ed7`: gui **1314/1314** across 24 shards (partition
verified exhaustive), all 75 non-gui packages, vet clean beyond the known TinyGo
gap, tinygo **1,651,004 flash / 63,304 ram**, corpus 67/67 matching, `gofmt -l`
the known five-file baseline.
