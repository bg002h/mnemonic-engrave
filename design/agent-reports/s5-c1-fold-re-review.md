# S5 C1 fold — scoped re-review

Reviewer: independent agent, sonnet tier, mechanical/verification pass per the
brief. Subject: the two-file fold inside commit `7910e00`
(`gui/multisig_build_tail.go`, `gui/multisig_build_s5_test.go`) in
`/scratch/code/shibboleth/wt-s5`, as authored (already committed — nothing was
edited or run through the build gate; this is a re-review, not a fold-check that
mutates the tree).

**The one question: did the fold fix C1, and did it introduce a new defect?**

**Verdict: 0 Critical, 0 Important, 1 Minor. The fold is sound. This loop CLOSES.**

---

## Did the fold fix C1? Yes — CONFIRMED by trace, consistent with the settled
mutation proof

Traced `buildEngraveTail` (`gui/multisig_build_tail.go:945-1026`) end to end: the
`engraved` map is now `map[string]bool{}` keyed on `b.MS1` (the string
`deriveMultisigLeg` actually produced for this leg), decided **after** derivation
returns, not predicted from `s.SeedID` beforehand. The pre-fold defect — dedupe
never fires because `buildSeedForSlot` (`gui/multisig_build.go:194-201,510`)
mints a fresh `SeedID` per held slot even when two slots share one master — no
longer applies, because the new key is derived from the leg's own output, not
from registry bookkeeping. This matches the mutation-test proof already recorded
in the fold's commit message (reverting to never-fires reproduces the exact
pre-fold 3-plates-for-2-masters failure); I did not re-run it, per the brief.

## Q1 — Is the new key correct in both directions? CONFIRMED, both ways

**Same seed → same string (dedupes):** `deriveMultisigLeg`
(`gui/multisig_derive.go:64-72`) calls `codex32.EncodeMS1(m.Entropy())` with no
randomness in the path; two legs from identical entropy always produce identical
strings.

**Different seeds → never the same string (never merges):** `EncodeMS1`
(`codex32/msencode.go:17-29`) builds `payload := msPrefixEntr || entropy` and
passes it straight into `NewSeed("ms", 0, "entr", 's', payload)` — a direct
bech32-style embedding of the raw bytes, not a hash or truncation. `hrp`,
threshold, id (`"entr"`), and index (`'s'`) are fixed literals, identical for
every call, so they cannot introduce a collision. Valid BIP-39 entropy lengths
(16/20/24/28/32 bytes) each map to a distinct output length, so cross-length
collisions are structurally impossible, and same-length entropy differing in any
byte differs in at least one 5-bit group of the base32 conversion, hence in at
least one output character. No hash is in this path, so there is no
collision-search surface at all — the encoding is injective on entropy by
construction. I did not find, and do not believe there exists, an input pair
that violates either direction.

## Q2 — The passphrase question: verdict is NOT a data-loss defect

Traced the full chain: `deriveMultisigLeg` (`gui/multisig_derive.go:65-66`) calls
`codex32.EncodeMS1(m.Entropy())` — entropy only, passphrase never enters the
`ms1` payload, confirmed at the `EncodeMS1` signature
(`codex32/msencode.go:17`, `func EncodeMS1(entropy []byte)`). Meanwhile
`deriveAccountXpub` (`gui/derive.go:20`, `bip39.MnemonicSeed(m, passphrase)`)
**does** fold the passphrase into the BIP-32 seed, so two held slots on the same
words with different passphrases genuinely are two different masters with two
different xpubs/fingerprints/mk1s — but byte-identical ms1 strings. Under the
fold's dedupe, the second leg's `MS1` is cleared and only one physical ms1 plate
is cut.

**Reasoning through it:** this does not lose recoverable information. Both
would-be ms1 plates are byte-for-byte identical (ms1 never encodes passphrase for
*any* seed, single-slot or multi-slot — this is a pre-existing, general property
of the format, not something this fold introduces). Cutting two identical
plates gives zero additional recovery capability over cutting one — exactly what
the fold's own comment argues ("a duplicate secret on steel with no recovery
benefit"). The information that actually distinguishes the two masters — the
distinct passphrases — was never captured in this flow's backup materials for
any seed (`gui/multisig_build_census.go`'s `buildPlateCensusLines` /
`buildPlateInventoryLines` mention no passphrase at all; verified by grep, no
hits). Passphrase custody is out-of-band by design elsewhere in the product (the
separate BIP-39 Password engraving program). The two masters remain
distinguishable at restore time via their **distinct mk1 fingerprints/xpubs** —
the dedupe never touches mk1s, only ms1s. SPEC 4.1 ("the (seed, passphrase) pair
is the derivation unit") supports treating this as intentional: the *seed*
(words) is genuinely singular here; only the derivation output differs.
**Verdict: correct behavior, not a defect.**

**One Minor found in the course of this reasoning**, below.

## Q3 — Clearing `b.MS1` on the duplicate leg: CONFIRMED no consumer breaks

Grepped every non-test read of `.MS1` in `gui/`. Exactly one production consumer
reads a multi-leg `legs` slice's `MS1`: `gui/multisig_build.go:355`,
`multisigVerifyFlow(ctx, th, legs[0], legs[0].MS1 != "")` — index **0** only
(the comment at `:340-350` already documents that `multisigVerifyFlow` is
single-leg and doesn't yet cover legs 1..n; that's a separately-tracked,
already-known gap per the brief, not new). Traced the loop in
`buildEngraveTail`: `legs` is appended in slot order, and the `engraved` map
starts empty, so the **first** append to `legs` is unconditionally the first
occurrence of its own ms1 (nothing can already be in `engraved` before the first
insert) — this holds regardless of which physical slot is first held, since
`legs[0]` is always the first *added* leg, not `sources[0]`. So `legs[0].MS1` can
never be the one the dedupe clears. `gui/multisig.go:172`
(`multisigEngraveCards(b.MS1, ...)`) is a different, single-leg code path
(`engraveMultisig`, calling `deriveMultisigLeg` directly) not fed by
`buildEngraveTail` at all — irrelevant to this fold. The restore doc
(`gui/multisig_build.go:371`, `multisigRestoreDocFlow`) is built from
`cardsOut`/`tpl`/`keys` (via `md.ExpandWalletPolicyChunks`), never from `legs`,
so it cannot observe a cleared `MS1` either. No consumer is affected.

## Q4 — Cost / entropy exposure: CONFIRMED no extra exposure

`gui/multisig_derive.go:64-72` (verified at the cited lines):
```go
if full {
    entropy := m.Entropy()
    ms1, err := codex32.EncodeMS1(entropy)
    wipeBytes(entropy)
    ...
}
```
`wipeBytes(entropy)` runs unconditionally immediately after `EncodeMS1`, on
**every** call to `deriveMultisigLeg`, regardless of whether the tail later keeps
or discards the resulting string. The dedupe decision happens entirely in the
caller (`buildEngraveTail`) after this buffer is already wiped, so deriving
`full` on every held slot (rather than only on one representative per master)
costs extra CPU/one more `EncodeMS1` call, not extra seed residency.

## Q5 — Did the fixture change weaken a sibling test? CONFIRMED no

Five tests call `s5TraceB`: `TestMultiSlotSelfAssembles`,
`TestLegDerivedAtHeldSlotOrigin`, `TestOneMk1PerHeldSlot`,
`TestFullModeEngravesMs1ForEveryMaster`, `TestReRunMintsByteIdenticalPlates`.
Read all five bodies. `buildSlotSources`'s account assignment is keyed on
**master fingerprint**, not `SeedID` (`gui/multisig_build.go:383-393`'s own
comment states this), so whether the fixture passes one registry entry reused
twice or two separate entries for the same master, the account numbers, origins,
and mk1 cardinality/distinctness assertions in the first three tests are
unaffected either way — none of them assert on `SeedID` identity. The
determinism check (`TestReRunMintsByteIdenticalPlates`) compares two runs of the
*same* fixture against each other and is agnostic to what the fixture's shape is.
Only `TestFullModeEngravesMs1ForEveryMaster` is sensitive to the fixture's
registry shape, and the fixture correction is what makes it actually exercise the
product's real shape instead of the shape in which C1 was invisible (this is the
fix working as intended, not a weakening). No sibling assertion became vacuous.

---

## MINOR

### M1 — the fold's justifying comment cites a plan requirement not yet implemented anywhere in the codebase

`gui/multisig_build_tail.go:72-75`:
> A passphrase does not split a plate: ms1 encodes the WORDS and never the
> passphrase (**which is why the plan requires the backup to say so out loud**),
> so one word-set engraved twice is a duplicate secret on steel with no recovery
> benefit.

The parenthetical refers to `design/IMPLEMENTATION_PLAN_multisig_build_repair.md:1208`:
*"Where a passphrase was used, the mode label and the restore doc MUST both say
the backup is incomplete without it."* Grepped `gui/` for this requirement's
implementation (`"incomplete without"`, `"requires a BIP-39 passphrase"`,
`"required spending factor"`) — **no hits anywhere in the tree**, including in
`gui/multisig_build_census.go` (`buildPlateCensusLines`/`buildPlateInventoryLines`,
which carry no passphrase text at all) and the mode-label `ChoiceScreen` at
`gui/multisig_build.go:311-315`. The requirement is real in the plan but is not
yet built. This is a **pre-existing gap**, not introduced or worsened by this
fold — it applies identically to a single held slot with one passphrase, with or
without S5's multi-slot model — and it does not change the correctness of the
dedupe itself (Q2's verdict stands independent of whether this line ever gets
built). But the comment reads as though the cited requirement is already
satisfied elsewhere in the product, which is not currently true. Fix: either cite
a FOLLOWUPS.md entry for this (none currently exists — checked), or reword to
state plainly that the "say so out loud" text does not exist yet.

**Not blocking.** Filed for whoever owns the passphrase-disclosure requirement,
not against this fold.

---

## Scope discipline

Did not re-derive: the build-gate result (`go test` 51/0, `gofmt` clean), the
mutation proof, the master-fingerprint-key rejection rationale, the `@S`
single-select limit, or the `legs[0]`-only verify limit — all taken as settled
per the brief. `go` was not available in this execution environment to re-run
the suite; every claim above was verified by direct source trace instead (line
numbers cited throughout, each read from the file at the time of this review).

## Worktree state

`git status --porcelain` in `/scratch/code/shibboleth/wt-s5` is empty before and
after this review — no files were edited, added, or removed.
