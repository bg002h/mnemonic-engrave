# Continuity — F-531 and F-530 both DONE; the flash is next

**Updated 2026-09-13.** Everything below is measured, not recalled.
Supersedes the pre-F-531 version of this file (see `git log` on this path).

## Where the repos are

| repo | branch | tip | state |
| --- | --- | --- | --- |
| `mnemonic-engrave` | `master` | `eb896ddb` | F-531 pushed at `ab1f9f10`; F-530 commits after it not yet pushed |
| `seedhammer` (fork) | `main` | `0562e81` | F-531 pushed at `9b36ed7` (CI green); F-530 commits after it not yet pushed |
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

## F-530 — CLOSED, GREEN

The device derives no address for a descriptor that puts the same public key at
two seats. All three `descriptorFlow` callers, both branches of the Addresses
button, and the screen says why rather than silently withholding.

Fork `0562e81`, engrave `3fbc4a02`. Full detail is in FOLLOWUPS; the three
reports are `design/agent-reports/f530-descriptor-rule-review.md`,
`f530-fold-verification.md` and `f530-parity-verification.md`.

**What a future reader most needs from this one.** Three review rounds found the
SAME defect class three times, and it was never arithmetic — it was a rule kept
as a **second copy**:

1. `gui/` copied the deriver's normalisations, so the predicate compared the
   *spelling* of a key expression. Four spellings of one key walked the
   refusal; a one-token edit to the input opened the Addresses choice and paid
   out the byte-identical address the refused fixture produces.
2. `sysw/` copied the primary's admission, so on the payload route the new gate
   fired only on descriptors admission had already rejected.
3. The two languages then held two *different* well-reasoned versions of the
   same sentence — receive-only in Rust, either-chain in Go — and disagreed
   about a key that collides only on the change chain.

`address.DerivesSameKey` and `derive::derives_same_key` are now one rule in two
languages, cross-checked over 324 pairs against ground-truth CKDpub derivation
and pinned by a shared vector (`gate/duplicate-key-change-chain-only`) that
reds four named assertions if they drift.

**And two findings were about the screen, not the rule** — both the silence
class, both worth remembering before adding any warning to a device screen:

- An unbounded **scanned** Title pushed the warning off a screen that does not
  scroll. At 200 characters the funds sentence itself was cut mid-clause,
  leaving "2-of-3 multisig", a fragment and an empty button — the exact silent
  refusal the code cites F-531 to justify preventing, restored by a field the
  artefact controls. The warning is drawn FIRST now; nothing can push it down
  because nothing is above it, which is a guarantee no length budget gives.
- The fit gate meant to catch that measured in narrow non-wrapping `x` glyphs
  and certified 44 characters of slack while a real 25-character title
  overflowed. Measure in the unit the thing is actually made of.

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
