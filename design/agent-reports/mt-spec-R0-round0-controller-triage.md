# R0 round 0 — controller triage

Artifact: `design/SPEC_mt_v0_1.md` at `099a516`. Four lens reports, persisted
verbatim in their own commits before this file was written:

| lens | model | reported | commit |
| --- | --- | --- | --- |
| 1 — design / determinism | opus | *still running at time of writing* | — |
| 2 — funds safety | opus | 6C / 10I / 5M / 1N | `52b91a3` |
| 3 — external facts | sonnet | 1C / 1I / 1M / 0N | `d68d454` |
| 4 — coverage | opus | 3C / 11I / 5M / 2N | `bc8b0c1` |

**The gate does not close.** That conclusion does not rest on the reported
totals — it rests on the subset below, which I verified myself.

## Independently CONFIRMED (controller-verified, not taken on a reviewer's word)

**T-1 (Critical) — §7's mitigation column is false for 2 of its 4 hazards.**
Verified against the artifact alone, no source needed. §7 promises *"legend
carries the input outpoints so a holder can check they are still unspent"* and
*"legend states rate and date so staleness is visible."* §5's legend has five
fields — `BEARER`, `FROM WALLET`, `SPENDABLE AFTER BLOCK`, `TO <addr> <amount>`,
`PLATE n OF m` — and §5's own *What was dropped* table lists **`input
outpoints`** and **`fee rate and date`** as cut. So §7 mitigates two hazards with
fields that do not exist. Same false promise recurs at §6c and §6d (*"the
outpoints go on the plate"*), four sites in total.

This is the `a-diff-falsifies-text-it-never-touches` failure mode exactly:
commit `0253b5c` rewrote §5's legend from ten fields to five, and nothing
re-read §7 — whose mitigations were written against the ten.

**T-2 (Critical) — §4's declared search space contradicts §4's own table.**
The search space reads `module size x QR version (1..40) x ECC (L,M,Q,H) x k*k
tiling`. Its results table lists configurations of **2 qr**, **3 qr** and
**6 qr**. None of 2, 3, 6 is a perfect square; only the `1 qr` rows are
consistent with `k*k`. Either the prose or the search that produced the numbers
is wrong, and §2's promise that selection is deterministic *"so two encoders
agree"* cannot be assessed until it is known which.

**T-3 (Critical) — `ur:bytes` is forbidden for production by the document §3
cites as its pedigree.** Verified against primary source, fetched directly:
BCR-2020-005 line 100 states the `bytes` type *"exists only for testing and
validation of UR implementations and MUST NOT be used for any other purpose."*
RFC-2119 keywords are declared normative in that same document (line 71). I then
enumerated the companion registry BCR-2020-006: the only Bitcoin-transaction
type is `psbt` (tag 40310, formerly `crypto-psbt`), which *"MUST be a valid
Partially Signed Bitcoin Transaction encoded in the binary format specified by
BIP174"* — not the finalized transaction §8 requires.

So §3 chose an envelope that its own standard forbids, and the registry offers
no compliant substitute for a raw signed transaction. This bites hardest on
**F-234**, whose whole premise is that a recoverer with no `mt`-aware software
can still read the plate: the type selected to serve that premise is the one no
wallet is obliged to decode.

**T-4 (Critical) — the required locktime is inert, and the spec never says the
word.** `nSequence` and `sequence`: **0 occurrences** in 534 lines (positive
control: `locktime`, 3 occurrences, so the query works). `nLockTime` is enforced
only when at least one input has `nSequence != 0xFFFFFFFF`. A transaction with
all inputs final ignores its locktime entirely — so §8.4's *"required future
locktime"*, which §7 names as the **only** mitigation for the bearer hazard,
can be satisfied on paper by a transaction that is broadcastable the moment the
plate is cut.

Also 0 occurrences of `SIGHASH_NONE` and `SIGHASH_SINGLE`; the spec inspects
sighash *flags* nowhere, and its one sighash discussion (§6a) covers only
`SIGHASH_ANYONECANPAY` in the amount-commitment argument.

## Found by me, outside every lens

**T-5 (Important) — §11 is false as written.** §11 says *"Everything measured is
in `design/measurements/`."* The §6c/§6d block figures — `nBits 17023cc1`, the
4,886-transaction block, the 538-byte proof, the header and MTP timestamps — are
backed by no `RESULTS_*` file. `grep -rl "17023cc1" design/` returns only the
spec and a lens report. The underlying block is real and locatable (height
**963650**, `nTime` 1787446609, matched against the live node) and the node's
difficulty `125807076547197.5` matches the spec's stated figure, so the *numbers*
are sound — but the evidence for them was never persisted, which is the one
place the cycle's own evidence discipline lapsed.

## NOT yet verified — do not fold on these

Lens 2 produced 6 Criticals from the spec's assertions rather than from source
(that was its brief; external facts were lens 3's). T-4 above is one of its
findings and it held up. The rest — legacy inputs committing to no amount, the
value-balance gap, a bare signed hex passing every §8 refusal, `include_mempool
false` inverting §6b's own argument — are individually plausible and individually
checkable, and none has been checked yet. **Prescribed fixes are not
authoritative**; each gets reproduced before anything is edited.

Lens 4's Criticals carry `file:line` anchors into the fork. Two spot-checks
resolved (`freeTextQRScale = 2` at `backup/fit.go:19`; `sysw/record.go` has zero
`Transaction`/`TxClass` occurrences). The rest are unresolved.

## What the fold cannot decide on its own

Three of these are not editorial repairs — they are design questions whose
answers change what gets built, and they belong to the operator:

1. **The envelope (T-3).** Use a fully-finalized PSBT under the compliant
   `ur:psbt`, and pay the size difference in plates? Or stay on `ur:bytes`
   knowingly off-label, and say so in the spec? The first is conformant and
   widely-read but costs bytes against §4's counts; the second keeps the counts
   and forfeits the compliance claim §3 currently makes.
2. **The bearer mitigation (T-4).** Requiring a non-final `nSequence` is the
   obvious repair, but it interacts with RBF signalling and with what signing
   devices will produce.
3. **§10.6 redundancy**, still open, and now entangled with a lens-2 observation
   worth its own check: that ECC protects against *marks on a symbol*, not
   against *loss of a whole plate*, so §4's objective may be optimising the
   wrong failure — and that any redundancy above zero makes `PLATE n OF m`
   misleading, since a holder then needs k of m.
