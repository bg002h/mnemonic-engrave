# R0 round 2 — IMPLEMENTATION_PLAN_multisig_build_repair.md

Reviewer: independent verification pass (sonnet), 2026-08-13. Plan reviewed at
`design/IMPLEMENTATION_PLAN_multisig_build_repair.md` (current HEAD, commit
`1419a25`); fold isolated via `git diff de03e52..1419a25` (`de03e52` is round
1's persisted-report commit, a no-op on the plan file, so this is identical to
`git diff d671d01..1419a25`); round 1's report at
`design/agent-reports/multisig-build-repair-plan-R0-round1.md`; round 0's at
`design/agent-reports/multisig-build-repair-plan-R0-round0.md`; source at
`/scratch/code/shibboleth/seedhammer` @ `a10d007`. `./scripts/plan-cite-gate.sh`
re-run this session: **every citation resolves**, including `mk/mk.go:5`. The
gate script itself states it does not cover whether a line SAYS what the plan
claims — that gap is exactly where this round's finding lives.

## Verdict

**NOT GREEN — 0 Critical, 1 Important, 0 Minor.** Both of round 1's Importants
(R1-I1, R1-I2 on the ownership axis) are cleanly closed. But this fold's newest
text — S0 deliverable 4, entirely fold-added — carries a factual claim about
the codebase that a machine check (a round-trip test run against the actual
source this session) proves **false**: the fork's `mk` package already decodes
a depth-0 mk1 card and produces `Path == "m"` today, with no re-pin. Round 0
and round 1 both treated `mk/mk.go:5`'s "family_token mk-codec 0.2" comment as
proof of a decode-capability gap; nobody traced the decode path itself. That is
the new Important (R2-I1).

---

## The implementer-judgement table — rebuilt from scratch against current HEAD

| # | location (stage / section) | the decision left open | verdict |
| --- | --- | --- | --- |
| 1 | §1a / S0 oracle table, mk1 relation (a) | Exact CLI success shape for "the primary `mk decode`/`mk inspect` accepts the chunks" (exit code vs. stderr/stdout). | DETAIL — walk-script plumbing, doesn't change what's engraved. |
| 2 | S0 deliverable 3 | Cite `address_test.go`'s existing fixtures' provenance, or replace with BIP-382 vectors. | DETAIL — `TestBip382WshMultiAddressesMatchPublishedVectors` (S0) independently proves address correctness either way. |
| 3 | S0, "prints the resolved oracle versions" | Print-only visibility vs. hard refusal on a stale (non-vendored) primary. | SETTLED, not open — repeated twice verbatim; only vendored data gets the hard refusal + dedicated test. Unchanged since round 1. |
| **4** | **S0 deliverable 4** (`mk 0.2 → 0.4.x including V19`) and **S5 test 6's "pin seam" note** | **Whether the fork's `mk` decoder actually needs a wire re-pin to decode a depth-0 mk1 card and see `Path == "m"`.** | **RULE IT — Important. FALSE as written; see R2-I1.** Machine-verified this session: the fork decodes depth-0 today. |
| 5 | S0 deliverable 4's escape clause ("if larger than S0 should carry, becomes its own stage before S5") | Whether this is a real, owned constraint or an unowned assumption in disguise. | Real constraint **for the ownership question** — it correctly preserves "must not become unowned again" if the work turns out large. But it is scoped against a false premise (row 4): once the mk-side claim is dropped, the only work deliverable 4 actually carries is the (separately legitimate, already-settled) md vector re-pin, which is very unlikely to overflow S0. Not a hole on its own; downstream of row 4. |
| 6 | S0 — no test or Gate criterion names deliverable 4 at all | Unlike every other deliverable (1–3 each map to a named test in "Tests first"), deliverable 4 has none, and S0's "Gate" paragraph checks only the three BIP tests + harness printing + refusal test. | Minor, subsumed into R2-I1's fix — once the false mk claim is removed, what remains (md vector re-pin) should get the same treatment: a named test or an explicit Gate line, so S0 can't be declared closed while deliverable 4 (whichever part survives) is silently undone. |
| 7 | S2 test 3, `TestBuildRefusesForeignOriginCardBeforeS5` | Refuse vs. warn. | **SETTLED, no longer open.** Fold picked REFUSE explicitly; name and body now agree. (Was RULE IT/Minor in round 1 — closed.) |
| 8 | S4 gate, "emulator walk of… one loud failure" | Which of S4's 5 failing-row tests gets the visual walk. | DETAIL — tests are exhaustive; walk is a smoke check. |
| 9 | S4 test 5, `TestGateAcceptsSameSeedAtDistinctOrigins` | Exact notice text shown to the operator. | DETAIL/Minor — informational only. |
| 10 | S4 test 8 / S5 test 7 (`TestGateDerivesAtTheCardsOwnOrigin`, `TestGateStillFiresAfterOriginsDiverge`) | Fixture, PROCEED/FAIL split, mutation mandate. | **SETTLED, no longer open.** Both now name a concrete fixture (`m/48'/0'/1'/2'`), a PROCEED/FAIL pair, and an explicit mutation ("derive at `multisigSharedOrigin()` instead of the card's declared origin"). Verified this session: `multisigSharedOrigin()` is the fixed constant `m/48'/0'/0'/2'` (`gui/multisig_build.go:421-424`) — cryptographically unrelated to the fixture's `…/1'/2'` under hardened BIP-32 derivation, so the named mutation genuinely flips the PROCEED case to FAIL. (Was RULE IT/Important in round 1 as R1-I1 — closed.) |
| 11 | S6 gate sentence | Whether the ms1 readback (item 3) is one of the gate's own pass/fail conditions. | **SETTLED, no longer open.** Gate sentence now reads "…and master B's mnemonic restores from its ms1 plate." (Was RULE IT/Minor in round 1 — closed.) |
| 12 | S3, "Delete or correct the four `TYPED-ONLY` comments" | Delete vs. correct, per site. | DETAIL — either satisfies "a future reader finds nothing misleading." |

**Could a competent implementer build this without asking a question that
changes the result? Not quite — one point remains, and it is exactly the shape
of judgement this gate exists to close.** Row 4: the plan directs the
implementer to complete a wire-format re-pin before S5 test 6's premise is
"sound." That directive is false. An implementer who tries writing S5 test 6
against current `mk` will find it already works and be left choosing, unaided
by the plan, between (a) doing unnecessary — and risky, since `mk/mk.go` is
funds-adjacent decode code — work anyway because the plan says to, or (b)
silently deviating from written scope without it ever being decided or
recorded. That is a real open decision; every other row is DETAIL or already
settled by this fold.

---

## Did round 1's fold work?

| finding | fixed? | new defect? |
| --- | --- | --- |
| **R1-I1** — S5's gate-reproof test underspecified; S4-side companion missing | **YES.** S4 gained test 8 (`TestGateDerivesAtTheCardsOwnOrigin`): named fixture (`m/48'/0'/1'/2'`), explicit PROCEED/FAIL split, explicit mutation ("derive at `multisigSharedOrigin()` instead of the card's declared origin — the PROCEED case must go red"). S5 test 7 rewritten to reuse "S4 test 8's fixture," same PROCEED/FAIL split, "mutation-checked the same way." Verified the mutation is genuinely kill-capable (see table row 10). | No. |
| **R1-I2** — oracle re-pin ownership sentence dropped by the previous fold | **YES, on its own terms.** New S0 deliverable 4 states "**S0 owns this**" explicitly, names both halves (mk 0.2→0.4/V19, md vectors 0.36.0→current), and the S5 test 6 cross-reference was corrected from ambiguous "S0's re-pin" to "S0 deliverable 4's re-pin." The ownership question R1-I2 asked about is answered. | **Yes — see R2-I1.** The restored ownership sentence is attached to a mk-side justification that is factually wrong. Ownership was fixed; what's owned was not checked. |
| **R1-M1** — S2 test 3 name/body disagree (refuse vs. warn) | **YES.** Fold picks REFUSE explicitly; "must not be silently stamped… **this plan picks REFUSE**, so the test's name matches its body and the assertion has one arm." | No. |
| **R1-M2** — S6 gate sentence doesn't cite the ms1 readback | **YES.** One clause added: "…and master B's mnemonic restores from its ms1 plate." | No. |

---

## New findings

### R2-I1 (Important) — S0 deliverable 4's mk-decoder justification is machine-verified false; the fork already decodes a depth-0 mk1 card today

**Where.** S0 Deliverable 4 (fold-added, entirely new text): "Without V19 the
fork cannot decode a depth-0 mk1 far enough to see `Path == \"m\"`, so that
test cannot be written as specified…". S5 test 6's fold-touched note: "…this
test's premise — that the fork decodes the card far enough to see `Path ==
\"m\"` — is only sound once **S0 deliverable 4's** re-pin includes V19."

**The claim's origin.** Round 0 (fable) first asserted this in M5, reasoning
from `mk/mk.go:5`'s comment ("family_token \"mk-codec 0.2\"") and the primary's
CHANGELOG line "Added a depth-0 / no-path test vector (V19)." Round 1 (sonnet)
listed the `mk/mk.go:5` pin as an already-settled fact and did not re-derive
it, per its own scope discipline. This fold then built S0 deliverable 4's
entire justification — and part of its escape clause — on that inherited
claim. At no point did any round trace the actual decode path.

**What the source and changelog actually say.**
- `mnemonic-key/CHANGELOG.md` (primary, `mk-codec` crate): the `[0.4.2]` entry
  reads **"No wire or runtime-behavior change"** — V19 is "one new test
  vector," and "V1–V18 byte-identical." `[0.4.1]` states wire format and
  corpus are "byte-identical to 0.4.0." Nothing in the changelog describes any
  wire-format change to path encoding, ever, across the 0.1→0.4.2 line.
- `seedhammer/mk/mk.go` `decodePath` (lines 308-336): the explicit-path branch
  reads a 1-byte `count`; `count == 0` is a completely ordinary case of the
  existing loop (`for i := 0; i < count; i++`), producing an empty `comps`
  slice — no version gating, no special-casing needed.
- `seedhammer/mk/mk.go` `pathString` (line 369): an empty `comps` renders as
  `"m"` — this is the generic base case, not new logic.
- `seedhammer/mk/mk.go` `reconstructXpub` (lines 401-408): already handles
  `len(comps) == 0` explicitly (`depth := uint8(len(comps))`; `childNum`
  defaults to 0), producing a valid depth-0 xpub.
- `seedhammer/mk/encode.go:100-101`'s own doc comment: `"m" (depth 0) yields
  an empty slice` — the capability is already documented in the code the
  citation-gate resolved but nobody read past the file name.

**Machine verification performed this session.** Added a scratch test to
`seedhammer/mk` (not committed; removed immediately after, `git status`
confirmed clean) that builds a `Card{Path: "m", Stubs: [...], Xpub: <a real
depth-0 master pubkey from `hdkeychain.NewMaster().Neuter()`>}`, calls the
package's own `Encode`, then `Decode` on the result:

```
=== RUN   TestScratchDepth0RoundTrip
    zzz_depth0_scratch_test.go:39: encoded 2 mk1 string(s)
    zzz_depth0_scratch_test.go:47: round-trip OK: decoded Path="m" Xpub=xpub661...
--- PASS: TestScratchDepth0RoundTrip (0.00s)
```

This exercises the full chunked encode/decode path (2 chunks, cross-chunk
integrity hash, header parsing) against the actual `a10d007` source, with
**zero code changes**. The fork decodes a depth-0 mk1 card correctly today.

**Why this matters.** `mk/mk.go:5`'s "mk-codec 0.2" comment is a stale
provenance label on the Go package's own **test-vector corpus**, not a claim
about decoder capability — the decoder's `count`-byte parsing was never
version-gated per path depth. Confusing "our vendored test vectors are old"
with "our decoder can't handle new wire content" is exactly the class of error
this project's own standing rule targets ("never describe code from its doc
comment… measure it, then quote the number").

**Failure scenario if left uncorrected.** An implementer reaches S0, reads
deliverable 4, and either (a) spends real effort "re-pinning" a decoder that
needs no functional change — and in doing so, edits `mk/mk.go`'s path-decoding
logic, which is exactly the kind of edit to currently-correct, funds-adjacent
wire-decode code that can introduce a **real** regression (a subtly wrong
depth or child-index handling would misattribute a cosigner's origin — the
precise C2 failure class this whole plan exists to close); or (b) tries writing
S5 test 6 directly, finds it passes without any re-pin, and is left silently
deviating from written plan text with no record of why. Neither outcome is
acceptable from a plan that is otherwise this rigorous about tracing claims to
source.

**Fix (bounded, no spec change).** Two edits:
1. S0 deliverable 4: drop the mk/V19 clause and its justification sentence
   entirely. State plainly (one sentence) that the fork's `mk` package already
   decodes depth-0 (V19-shape) cards correctly as of `a10d007` — verified by
   round-trip — so no functional re-pin is required there. Keep only the md
   vendored-vector re-pin (0.36.0 → current), which remains real and
   S0-owned per round 0's I2.
2. S5 test 6's note: replace "…is only sound once S0 deliverable 4's re-pin
   includes V19" with a statement that the premise already holds today, citing
   `mk/encode.go:100-101`'s own "m (depth 0) yields an empty slice" comment
   in place of the retired pin-seam claim.

Optionally (not blocking): if the team still wants `mk/mk.go:5`'s stale
"mk-codec 0.2" comment updated to reflect the current corpus vintage, that is
a one-line documentation fix, separable from any decode-path work, and can
ride with the md vector re-pin.

---

## Disposition

**Not 0C/0I.** R1-I1 and R1-I2 are cleanly closed on their own terms — no
re-review needed for the origin-gate specificity work or for the ownership
sentence itself. The one open Important (R2-I1) is a factual correction, not a
design change: delete a false claim and its downstream cross-reference, keep
the (real) md vector re-pin under S0's existing ownership. It touches no
operator ruling, no test behavior already specified, and requires no code —
only plan text. Re-review after this fold should scope to: S0 deliverable 4's
rewritten justification and S5 test 6's corrected note — nothing else in this
plan needs re-touching.
