# R0 round 3 — IMPLEMENTATION_PLAN_multisig_build_repair.md

Reviewer: independent verification pass (sonnet), 2026-08-13. Plan reviewed at
`design/IMPLEMENTATION_PLAN_multisig_build_repair.md` (current HEAD, commit
`e12047d`); fold isolated via `git diff 518cc6b..e12047d` (`518cc6b` is round
2's persisted-report commit); round 2's report at
`design/agent-reports/multisig-build-repair-plan-R0-round2.md`; source at
`/scratch/code/shibboleth/seedhammer` @ `a10d007`.
`./scripts/plan-cite-gate.sh design/IMPLEMENTATION_PLAN_multisig_build_repair.md`
re-run this session: **every citation resolves.**

## Verdict

**GREEN — 0 Critical, 0 Important, 0 Minor (new).** The fold closes R2-I1
cleanly and I could not break the replacement claim. Gate closes; implementation
begins at S0.

## 1. The fold, verified

`git diff 518cc6b..e12047d --stat` — one file, 23 insertions / 15 deletions,
all in `design/IMPLEMENTATION_PLAN_multisig_build_repair.md`; nothing else
moved. Two hunks, exactly matching round 2's prescribed fix:

- **S0 deliverable 4** now states only the md vendored-vector re-pin
  (0.36.0 → current) as S0-owned, and adds a "NOT included, and the reason is
  worth keeping" paragraph retracting the `mk 0.2 → 0.4.x/V19` claim, citing
  the round-trip and the changelog, and naming the failure mode (a stale
  comment mistaken for the mechanism).
- **S5 test 6's note** drops "only sound once S0 deliverable 4's re-pin
  includes V19" and replaces it with "the premise is already sound... so the
  flow reaches `errMultisigEmptyDivergent`... No re-pin gates it."

No new claim was smuggled in beyond what round 2 licensed. **No new defect.**

## 2. Adversarial check on the new positive claim — machine-run, not read

Two things had to hold for the fold's new claim to be true: (a) the mk
round-trip (round 2 already ran this — not re-derived), and (b) that the
*build flow*, once S5 wires `cosignerFromCard` to stop discarding
`card.Origin`, actually reaches `errMultisigEmptyDivergent` rather than
failing/succeeding differently earlier. Nobody had run (b). I did:

- Read `md/encode_multisig.go:96-109`: in `OriginDivergent` mode, `EncodeMultisig`
  is the **first** thing that runs (before script-tree or pubkey work) and
  loops cosigners checking `len(c.Origin) == 0` per-cosigner — the exact
  mechanism the plan cites at `md/encode_multisig.go:104-106`. Lines 104-106
  say exactly what's claimed (the `if`, the `return ... errMultisigEmptyDivergent`,
  the closing brace). Note: this is a *range* citation (`104-106`); the
  cite-gate's regex only resolves single-line `path:NNN` citations, so this one
  has never been machine-checked in any round — I checked it by hand this
  round. Pre-existing since round 0, unchanged by this fold, not a new gap.
- Ran the existing `TestEncodeMultisigRefuse/divergent-empty-origin` subtest
  (`go test ./md/... -run TestEncodeMultisigRefuse -v`, via
  `/home/bcg/.local/go/bin/go` — `go` is not on `$PATH` in this session) —
  PASS. Confirms `errMultisigEmptyDivergent` is live, real, reachable code
  today, not just a cited string.
- That existing test only covers **all-nil** origins, not the mixed case S5
  test 6 actually needs (one depth-0 card among real-origin cosigners, which
  is what makes `OriginDivergent` get selected in the first place per the
  plan's own rule "divergent when origins are not all equal"). Wrote a scratch
  test (`md/zzz_scratch_mixed_origin_test.go`, not committed, removed
  immediately after — `git status --short` confirmed clean) with three
  cosigners: two carrying real, distinct 4-component origins and one carrying
  `Origin: nil` (what a decoded `Path == "m"` card yields), `OriginMode:
  OriginDivergent`. Ran it:

  ```
  === RUN   TestScratchMixedOriginDepthZeroTripsNamedRefusal
      zzz_scratch_mixed_origin_test.go:37: mixed-origin depth-0 card correctly
      named-refused: md: EncodeMultisig OriginDivergent requires a non-empty
      Origin for every cosigner
  --- PASS: TestScratchMixedOriginDepthZeroTripsNamedRefusal (0.00s)
  ```

- Traced the remaining link — does a decoded `Path == "m"` card actually
  become `Origin: nil`/empty in the not-yet-written `cosignerFromCard`? Every
  primitive it needs already exists and already behaves this way:
  `bip32.ParsePath("m")` (`bip32/bip32.go:86-97`) splits on `/`, drops the
  leading `"m"`, and appends nothing for the remaining (empty) parts — returns
  an empty `Path`, no error. `originComponents` (`gui/singlesig_derive.go:129`,
  already used today for single-sig) maps an empty `bip32.Path` to
  `make([]md.PathComponent, 0)` — length 0. `decodeXpubBytes`
  (`gui/singlesig_derive.go:100`) parses via `hdkeychain.NewKeyFromString`,
  which does not gate on depth — a depth-0 xpub parses fine, so nothing fails
  earlier than the origin check.

**Conclusion: the positive claim holds, mechanically confirmed at every link,
not just re-read.** A false claim was not replaced by another false claim.

## 3. Rebuilt implementer-judgement table (final)

| # | location | the decision left open | verdict |
| --- | --- | --- | --- |
| 1 | S0 oracle table, mk1 relation (a) | CLI success shape for the walk script. | DETAIL. |
| 2 | S0 deliverable 3 | `address_test.go` fixture provenance. | DETAIL — `TestBip382…` proves address correctness independently. |
| 3 | S0, oracle-version printing | Print-only vs. hard refusal on stale primary. | SETTLED. |
| 4 | S0 deliverable 4 / S5 test 6 | Whether `mk` needs a re-pin to decode depth-0. | **SETTLED, closed this round.** False claim retracted; replacement verified independently (§2 above), not just re-read. No re-pin gates S5 test 6. |
| 5 | S0 deliverable 4 escape clause | Real constraint vs. unowned-assumption risk if the re-pin proves large. | Real, and now scoped correctly — only the md vector re-pin remains, unlikely to overflow S0. |
| 6 | S0 — no test/Gate line names deliverable 4 | Same treatment (named test or Gate criterion) as deliverables 1-3. | Minor, **still open**, non-blocking (round 2 scoped it out of this fold; record only). |
| 7 | S2 test 3 | Refuse vs. warn. | SETTLED (round 1). |
| 8 | S4 gate walk | Which failing row gets the visual walk. | DETAIL. |
| 9 | S4 test 5 | Exact notice text. | DETAIL/Minor. |
| 10 | S4 test 8 / S5 test 7 | Fixture, PROCEED/FAIL split, mutation. | SETTLED (round 1). |
| 11 | S6 gate | ms1 readback as a gate condition. | SETTLED (round 1). |
| 12 | S3 | Delete vs. correct `TYPED-ONLY` comments. | DETAIL. |

**Could a competent implementer build this without asking a question that
changes the result? Yes.** Row 4 — the one point round 2 found blocking — is
now closed and independently re-verified, not just trusted. The only open item
(row 6) is Minor and record-only per this project's severity rule: it costs
nothing to leave open, and doing so doesn't change what gets built or what's
engraved. Every other row is DETAIL or already SETTLED.

## Disposition

**0 Critical / 0 Important. Gate closes.** Four rounds across spec and plan;
this one found no new defect and closed the last open Important with
independent, run (not read) verification of the replacement claim, down to
the exact code path (`OriginDivergent` → per-cosigner empty check →
`errMultisigEmptyDivergent`) the plan's test depends on. Implementation may
begin at S0.
