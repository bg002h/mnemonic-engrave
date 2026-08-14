# RECON — what the published BIPs actually contain (S0 deliverable 6)

Date: 2026-08-14. Prerequisite for S0 D6/D7 per the plan's §1a instruction:
**open the sources and inventory them BEFORE writing the test list**, because
"the previous list followed an author's memory and two of its three tests were
unwritable."

It happened again. **One of the three tests in the current list is unwritable
from the BIP it cites**, and the citation is not merely imprecise — the document
contains no vector of any kind.

## Provenance

All text read from `bitcoin/bips`, pinned at commit
**`60f5b33b0a7be3cf09b933d97b78071d684db7d1`**, fetched as
`https://raw.githubusercontent.com/bitcoin/bips/<sha>/bip-XXXX.mediawiki`.
Every count below is a `grep -c` against that text, not a recollection.

## Inventory

### BIP-383 — `multi()` / `sortedmulti()`

Publishes **descriptor → script hex** pairs. Descriptors with derived child keys
list the 0th, 1st and 2nd scripts.

- **Addresses published: 0** (`grep -cE '(bc1|tb1|[13][a-km-zA-HJ-NP-Z1-9]{25,34})'` → 0).
  The plan's "not BIP-382, which publishes no addresses" is right about 382, and
  the same is true of 383 — the plan already says so, and it holds.
- `wsh(multi(...))` vectors exist with `0020…` scriptPubKeys. **Comparing at
  scriptPubKey is therefore correct and writable.**
- Bare `sortedmulti(...)` vectors exist, including one over two xpubs with three
  derived child scripts — useful, because it exercises sorting *after*
  derivation.

**Gap, and it matters:** `grep -c 'wsh(sortedmulti'` → **0**. BIP-383 publishes
no `wsh(sortedmulti(...))` vector, and that is precisely the shape this device
builds (`gui.TestExpandedToDescriptorWshSortedmultiRoundTrip`). The composed
shape can be covered only by *joining* sources — BIP-67 for the ordering,
BIP-383 for `wsh()` wrapping and for sorted-key script construction — never by
quoting one published vector. **A test that claims to check
`wsh(sortedmulti(...))` against BIP-383 would be claiming more than the document
supports.**

### BIP-67 — deterministic key ordering

Richer than the plan assumes. Each vector publishes **four** fields:

    List (unsorted) · Sorted · Script · Address

- **Addresses published: 5**, P2SH (e.g. `39bgKC7RFbpoCRbtD5KEdkYKtNyhpsNa3Z`,
  `3CKHTjBKxCARLzwABMu9yD85kvtm7WnMfH`).
- Vector 2 is already sorted ("no action required") — a genuine no-op case.
- Vector 3 sorts keys differing only in the final byte and in the `02`/`03`
  prefix, which is the case a naive comparator gets wrong.

So this supports more than "ordering vectors": **sorted order → script →
P2SH address, end to end, all quoted**. Keys are raw pubkeys, not xpubs, so it
tests the comparator and script construction rather than derivation.

### BIP-141 — the citation that does not hold

**BIP-141 publishes NO test vectors.** Every example in §Examples is a
structural template:

    scriptSig:    <0 <32-byte-hash>>
                  (0x220020{32-byte-hash})
    scriptPubKey: HASH160 <20-byte-hash> EQUAL
                  (0xA914{20-byte-hash}87)

`grep -cE '[0-9a-f]{40,}'` over the whole document → **0**. No concrete hash, no
concrete key, no scriptPubKey, no redeemScript, no address.

Therefore:

- **`TestBip141NestedSegwitScriptDiffersFromLegacy` is unwritable as specified.**
- The plan's line "the address is **derived from** a published vector, not
  quoted from one" is **false in a way that reads as careful**: it is not that
  141 omits the address, it is that there is nothing to derive one *from*.
  Recording it at "that weaker, honest level" still overstates what exists.

### BIP-143 — the replacement, and it is strictly better

§P2SH-P2WSH publishes a concrete **6-of-6 multisig** with all three layers:

    scriptPubKey : a9149993a429037b5d912407a71c252019287b8d27a587
    redeemScript : 0020a16b5755f7f6f96dbd65f5f0d6ab9418b89af4b1f14a1bb8a09062c35f0dcb54
    witnessScript: 5621…56ae   (6-of-6)

**Machine-checked here, not assumed** — the whole chain is self-consistent:

    sha256(witnessScript)  = a16b5755f7f6f96dbd65f5f0d6ab9418b89af4b1f14a1bb8a09062c35f0dcb54
                             == the witness program inside redeemScript   MATCH
    hash160(redeemScript)  = 9993a429037b5d912407a71c252019287b8d27a5
                             == the hash inside scriptPubKey              MATCH

Every step is locally reproducible, an address is derivable from the
scriptPubKey, and — unlike 141 — it is a **multisig**, which is what this device
cuts.

## The second unwritable test, and this one is about OUR code

`TestBip383WshMultiScriptPubKeyMatchesPublishedVectors` says to run BIP-383's
`wsh(multi(…))` vectors "through `bip380`". It cannot be done:

- `bip380.MultisigType` has exactly two values, `Singlesig` and `SortedMulti`,
  and `Parse` accepts only the literal `"sortedmulti"` (`bip380.go:335`). There
  is no unsorted `multi`. `gui.TestExpandedToDescriptorUnsortedMultiUnsupported`
  already pins that as deliberate.
- `address.go` reinforces it: the multisig arm sorts unconditionally
  (`slices.SortFunc`) before `txscript.MultiSigScript`.
- **Every `wsh(...)` vector in BIP-383 is `multi`, not `sortedmulti`** — and 383
  publishes no `wsh(sortedmulti)` at all. So the vectors the test names are
  exactly the ones this codebase is designed never to accept.

So two of the three named tests are unwritable, for two unrelated reasons: 141
has no vectors, and 383's `wsh` vectors are a descriptor type we refuse by
design. §1a's warning recurred at precisely the rate it predicted.

### The writable form — BIP-383 still earns its place

BIP-383's **bare `sortedmulti(...)`** vectors publish the multisig script hex,
and that script is exactly the **witnessScript** of the `wsh(sortedmulti(...))`
this device builds. That gives an anchor in published bytes:

1. Take a published `sortedmulti(k, keys…)` vector and its script hex.
2. Compute the expected address locally: `0020 || sha256(script)` → bech32 P2WSH.
3. Build `wsh(sortedmulti(k, keys…))` through `bip380.Parse` +
   `address.Receive`, and compare.

The published bytes anchor the key ordering and the script encoding — the two
things a wrong implementation gets wrong — while only the bech32 wrapping is
locally derived, and the test says so. The vector over two xpubs with three
derived child scripts is the most valuable of these, because it exercises
sorting *after* derivation, which is our actual path.

## Verdict on the plan's three tests

| plan's test | verdict |
| --- | --- |
| `TestBip383WshMultiScriptPubKeyMatchesPublishedVectors` | **UNWRITABLE as stated** — `bip380` has no unsorted `multi`, by design, and every `wsh` vector in 383 is `multi`. Rewrite against 383's bare `sortedmulti` scripts as the witnessScript anchor (above). |
| `TestBip67SortedMultiKeyOrder` | **Writable, and can be stronger** — 67 publishes Script and Address too, so assert sorted → script → address rather than ordering alone. |
| `TestBip141NestedSegwitScriptDiffersFromLegacy` | **UNWRITABLE.** BIP-141 contains no vectors. Re-point at **BIP-143 §P2SH-P2WSH**, whose chain is verified above. |

## Recommended list

1. `TestBip383SortedMultiScriptMatchesPublishedVectors` — BIP-383's bare
   `sortedmulti` script hex as the witnessScript anchor, through
   `bip380.Parse` + `address.Receive`, with the bech32 wrapping derived locally
   and labelled as derived. NOT the `wsh(multi)` vectors: `bip380` refuses
   unsorted `multi` by design.
2. `TestBip67SortedMultiKeyOrderScriptAndAddress` — all four published fields.
3. `TestBip143NestedP2wshScriptPubKeyMatchesPublishedVector` — replaces the
   BIP-141 test; asserts the witnessScript → redeemScript → scriptPubKey chain
   against quoted values.
4. **Owed, and not satisfiable by quotation:** `wsh(sortedmulti(...))` is the
   device's actual output shape and no BIP publishes a vector for it. Cover it
   by composition and **say in the test that it is composed**, not published —
   an unattributed expected-value here would be self-agreement wearing the
   costume of a test, which is exactly what D7 exists to stamp out.

## For D7 (`address/address_test.go` provenance)

The plan offers "cite where they came from, or replace them with BIP-383
scriptPubKey vectors." **Replacement cannot be a like-for-like swap**: 383
publishes scriptPubKeys and zero addresses, so any fixture that is an *address*
must either keep a local derivation (documented as derived) or move to BIP-67,
which does publish P2SH addresses. Decide per fixture; do not assume 383 can
supply what those fixtures currently assert.
