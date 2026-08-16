# S5 verify fold — highest-stakes adversarial review of `f0006b7`

**Reviewer:** independent adversarial reviewer (fable tier; did not author the fold).
**Subject:** `f0006b7` ("S5 fold: fix the reused-key verify regression WHERE IT IS")
on `/scratch/code/shibboleth/wt-s5`, folding the 0C/2I review of `4b10319` + `40eb3bc`.
**Question:** is the fold correct — and did fixing a false RED reopen any path to a
false GREEN?

**Counts: 0 Critical / 2 Important / 1 Minor / 0 Nit.**

**Headline answers.**
- **No false GREEN was reopened.** Confirmed by structure and probing (details
  under Q1-Q3 below): the bijection in `verifyMultisigLegs` is untouched, every
  read-back plate must still be claimed and field-compared, and the dedupe can
  only *shrink* `legs`, which moves every outcome toward Incomplete/Failed —
  never toward OK.
- **The refusal of the earlier review's recommendation was CORRECT** (verdict
  section at the end).
- **But the fold does not fix I-1.** The dedupe fires only on byte-identical
  mk1s, and the reuse shape the supply path's one-plate engrave actually
  produces — the same seed at *distinct* origins, which is `TestFindUserSlot`'s
  ambiguous fixture and Trace B's own `@0`/`@1` — derives byte-*different* mk1s.
  The false RED the fold claims fixed still shows, confirmed by execution.

Worktree state: probe and mutation reverted; `git status --porcelain` empty at
finish.

---

## Important

### I-1 — The fold fixes only the byte-identical sub-case of the reused-key false RED; the shape the supply path's notice actually fires on still reports "Verify Failed". CONFIRMED (executed).

`gui/multisig_verify.go:361-364` (the dedupe), `gui/multisig_verify.go:355-357`
(the false premise), against `gui/multisig.go:141-149` (the one-plate engrave).

**The defect.** `verifyLegWithSameKey` dedupes on `slices.Equal(l.B.MK1, b.MK1)`
— byte-identical chunks, which requires the same origin path AND the same xpub.
But the supply path's "This key is reused at slots @0 and @1; engraving the
first (@0)" notice fires on `len(reused) >= 2` from `findUserSlot`, whose match
set is `allUserSlots` — **the same SEED at several slots**, and in every fixture
in this tree those slots sit at **distinct origins with distinct account xpubs**:

- `gui/multisig_match_test.go:90-96` — the ambiguous fixture the notice's
  behaviour is pinned on is literally commented *"The SAME seed at two DISTINCT
  origins (legitimate reused key)"*, `@0` at `m/48h/0h/0h/2h`, `@2` at
  `m/48h/0h/2h/2h`, each xpub derived at its own origin.
- Trace B's `@0`/`@1` (master A, accounts 0 and 1) is the same shape.

Distinct origins ⇒ distinct xpubs ⇒ distinct mk1s ⇒ the dedupe **never fires**.

**Executed proof** (probe test, deleted after; replicated the flow's derive loop
verbatim against the supply path's own pipeline with Trace B's md1 + master A):

```
SUPPLY: idx=0 reused=[0 1] (notice fires: len(reused)>=2 is true)
VERIFY: allUserSlots=[0 1]
VERIFY: len(legs)=2 len(readbackMk1s)=1
VERIFY: verifyMultisigLegs = verify: no read-back key plate carries slot @1's key
=> FLOW SHOWS: "Verify Failed. The read-back bundle does NOT match the seed."
```

**Concrete operator scenario.** Supply a policy in which the operator's master
fills two slots at two account paths (Trace B's md1 is exactly this). The
device announces "This key is reused at slots @0 and @1; engraving the first
(@0)", cuts one mk1 plate + the md1, and offers "Verify now". The operator
scans back exactly what was cut and re-types their seed. Final screen: **"Verify
Failed. The read-back bundle does NOT match the seed. Check the engraved
plates."** — the earlier review's I-1 screen, verbatim, on the fold that claims
to have removed it.

**The fold's justifying comment is factually false.**
`gui/multisig_verify.go:355-357`: *"Reused slots derive byte-identically (same
key implies same origin, and the origin is what the leg is derived at)"*. The
reuse the cited notice fires on is seed-reuse, not key-reuse; the matched slots
carry different keys at different origins and derive byte-differently (proved
above by the probe deriving two different mk1s for `reused=[0 1]`). The shape
the fold *does* fix — the same xpub declared verbatim at two slots at one
origin — is reachable only from a degenerate supplied descriptor that repeats a
key inside its own k-of-n.

**Not Critical** because it fails loud (no funds move on a Failed screen); it is
the same severity and direction the original review assigned. But it is an open
Important: the fold's headline claim ("the false RED is fixed") is false for the
shape the codebase's own fixtures and notice define as reuse.

**Note toward the real fix (not prescribed):** the verify cannot distinguish
"the supply path chose not to cut @1's plate" from "@1's plate is lost" using
only the readback — that information is not in the plates. The mismatch is
between the SUPPLY path's engrave rule (one plate for the first matched slot,
even when the other matched slots hold *different account keys*, which then
exist on no steel at all) and the verify's rule (a plate per distinct key). Any
sound resolution reconciles those two rules; deduping byte-identical legs
cannot, because the conflicting shape is never byte-identical.

### I-2 — The flow-level fix is pinned by nothing: deleting the dedupe from `multisigVerifyFlow` leaves the entire gui suite green. CONFIRMED (mutation executed).

`gui/multisig_verify_legs_test.go:373-415` (`TestReusedKeyVerifiesAgainstItsONEPlate`)
vs `gui/multisig_verify.go:361-364` (the fix site).

**The defect.** The fold's commit message claims the new test "pins BOTH
halves". Mutation result: removing the entire
`if i := verifyLegWithSameKey(legs, b); i >= 0 { covered[s] = true; continue }`
block from the flow — i.e. deleting the fold's actual fix — runs
`go test ./gui/ -count=1` to **`ok`, exit 0**. The test exercises
`verifyLegWithSameKey` and `verifyMultisigLegs` as units and never drives the
derive loop where the defect lived and where the earlier review CONFIRMED it.
Its first assertion (`verifyLegWithSameKey(legs, legs[0].B)` finding index 0) is
a tautology — a leg matched against itself in the list — not the two-slots-one-
key shape; and the subtest comment repeats I-1's false premise ("Reused slots
hold one key at one origin").

**Failure scenario.** A later edit reorders or drops the dedupe call (or changes
`fresh` handling) and every test stays green; the reused-key false RED silently
returns — or, worse, a future "fix" to I-1 above is written against these same
tests and ships unverified at the flow level, on the screen that decides whether
an operator trusts their steel. Under this project's own standard (a fix whose
removal no test notices; "prove the mutated line RAN"), this blocks.

---

## Minor

### M-1 — The I-2 (ms1-Back) fix also has no regression test; its comment's claim is nevertheless TRUE. CONFIRMED by trace.

`gui/multisig_verify.go:320-334`. I verified the parenthetical claim in the
comment: between the loop guard's evaluation (`len(legs) < len(readbackMk1s)`,
line 276) and the `break` at line 331, `legs` is not modified on any path
(lines 277-330 touch only `typed`, `passphrase`, `slots`, `fresh`), so at the
break either `len(legs) == 0` (first-seed abandon — the documented shipped
silent return) or `0 < len(legs) < len(readbackMk1s)` and the "Verify
Incomplete" screen shows ("Checked N of the M key plates read back"). No PASS
screen is reachable after cancelling. The one-line fix is correct; it simply
has no test driving Back-at-second-ms1, so the same mutation blindness as I-2
applies to it. Recorded, does not gate on its own.

---

## The brief's five questions, answered

1. **Is "one leg per distinct key" ever wrong?** In the false-GREEN direction:
   no. Equal MK1 requires equal path + xpub + fingerprint + stub, i.e. the two
   slots' plates would be byte-identical steel; dropping the duplicate leg loses
   nothing the kept leg does not verify. No construction exists where equal-MK1
   slots legitimately need distinct plates. The build path cannot even cut two
   identical seed-slot plates: `nextAccount` (`gui/multisig_build.go:404-412`)
   assigns strictly increasing accounts per master, so held slots always get
   distinct origins. mk1 encodes network label, origin path, master
   fingerprint, policy stub, xpub (`gui/multisig_derive.go:48-54`) — **no slot
   index** — so the dedupe cannot drop a slot-distinguished plate. In the false-
   RED direction the rule is wrong by omission: it does not cover the reuse
   shape the engrave paths actually produce (I-1 above).
2. **`covered[s] = true` on the dropped duplicate.** Safe. `covered` feeds only
   the `fresh` filter for later seeds and the choice between the two error
   messages (`gui/multisig_verify.go:294-313`); it is not consulted by the
   completeness arithmetic or the OK decision. The dropped slot's key IS
   verified via the byte-identical kept leg. Cross-seed dedupe (which could
   mismatch `MS1Readback`) is unreachable: it would need two different
   (mnemonic, passphrase) pairs deriving the same xpub at the same path, and a
   re-typed identical seed exits earlier at the `len(fresh) == 0` refusal.
3. **The arithmetic.** Dedupe only ever reduces `len(legs)`, and OK requires
   `len(legs) == len(readbackMk1s)` plus a passing bijection in which every
   plate is claimed exactly once and field-compared. So dedupe can push an
   outcome toward Incomplete or Failed (both RED), never toward OK: **a partial
   verify cannot look complete.** A complete one CAN look partial/failed —
   that is exactly I-1. (Duplicate physical plates in the readback — e.g. a
   scanned spare — yield Incomplete pre- and post-fold alike; not a fold
   regression.)
4. **I-2's `break`.** The comment's claim holds — verified by trace (M-1
   above). No PASS after cancelling; correct fix, untested.
5. **`slices.Equal` on `[]string`.** Correct: length check then elementwise
   `==`. nil-vs-empty compare equal (both length 0), which is unreachable here
   — `mk.Encode` output on the success path is never empty (probed chunks are
   non-empty), and a derive error returns before the compare. Nothing was
   "replaced": the helper is new in this fold. No finding.

## Verdict on the refusal

**Refusing the earlier review's recommendation was correct, and the fold's
demonstration of why is sound.** Dropping the leg→plate requirement makes a
readback missing a plate PASS (three distinct legs, two plates: both plates
claimed, third leg skipped, sweep satisfied) — a funds-losing false GREEN, and
the fold proved it by running it. The review's suggested companion change
(count *claimed plates* rather than legs in the Incomplete gate) does not
rescue it: with a lost plate, claimed == read-back and the verify still reports
success over steel that is gone. The recommendation was unsound; the refusal
protected the one direction that loses funds.

**But the alternative implemented is incomplete.** It repairs the reused-key
false RED only for byte-identical slot declarations — a shape no fixture in the
tree produces — while the shape the supply-path notice, `TestFindUserSlot`'s
ambiguous fixture, and Trace B's `@0`/`@1` all define as "reused" still lands on
"Verify Failed" (I-1), and the flow-level fix that was landed is pinned by no
test (I-2). The loop does not close on this fold.

## Verification appendix

- Probe (`TestProbeSupplyOnePlateDistinctOrigins`, since deleted): supply
  pipeline `findUserSlot` → one `deriveMultisigLeg` plate, then the flow's
  derive loop verbatim; output quoted in I-1. Run via
  `nix develop -c go test ./gui/ -run TestProbeSupplyOnePlateDistinctOrigins -count=1 -v`, PASS (it asserts nothing; it logs the trace).
- Mutation: dedupe block removed from `multisigVerifyFlow`;
  `nix develop -c go test ./gui/ -count=1` → `ok seedhammer.com/gui`,
  `MUTATION_TEST_EXIT=0`. Reverted.
- Cleanup: probe deleted, mutation reverted, `git status --porcelain` empty.
