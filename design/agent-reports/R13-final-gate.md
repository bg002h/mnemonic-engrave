# R13 — final pre-execution gate, `mt` v0.1

**Artifact:** `design/IMPLEMENTATION_PLAN_mt_v0_1.md` and `design/SPEC_mt_v0_1.md`
@ `72ec868` (the fold responding to `design/agent-reports/R12-gate-rerun.md`,
which returned NOT SAFE TO EXECUTE — 0 Critical, 1 Important blocking, 4 Minor).

**Question asked:** did `72ec868` close each of R12's five findings, and is the
plan now safe to execute unattended tonight, with no operator available?

**Scope, per the dispatch brief:** mechanical verification only — no re-audit
of settled ground (mt qr deferred, fork-per-codec, no script engine, zero
redundancy, cross-format verification abandoned, four verbs, `--elide-prefix`,
the 55-bit header, operator rulings). Environment and the two already-built
regtest vectors taken as given, not re-derived. Both gate scripts (`plan-cite-check.sh`,
`spec-structure-check.sh`) re-run directly rather than trusted from the commit
message.

---

## Section A — per-finding status

| # | R12 finding | Status | Evidence |
| --- | --- | --- | --- |
| B1 | P0's own deliverable list never named the JSON copy P1's SHA-256 pin reads | **FIXED** | `design/IMPLEMENTATION_PLAN_mt_v0_1.md:289`, inside P0's own section (283–332, confirmed by heading grep: P0 starts 283, P1 starts 334): `` **`design/vectors/mt1_v1_vectors.json` → `crates/mt-codec/src/test_vectors/mt1_v1.json`** — the machine-readable form, at `mk`'s location shape. **This is the exact file P1's SHA-256 pin test reads**, so P1 fails on a missing file without it (R12 B1) ``. `grep -n "mt1_v1.json"` now returns 3 hits total; one (line 289) is inside P0's operative bullet list, not only inside S0's explanatory blockquote (lines 178/187). |
| B2 | Status line cited R11 (the finding report, verdict "NOT SAFE TO EXECUTE") as having closed the gate | **FIXED** | Plan lines 3–9 now read: *"The pre-implementation gate ran twice: `R11-pre-implementation-gate.md` found the blockers (3C/6I/10m) and `R12-gate-rerun.md` verified the fold. **Cite the verifying report, not the finding report** — an earlier version of this line named R11 as having closed the gate, and R11's own recorded verdict is 'NOT SAFE TO EXECUTE' (R12 B2)."* Correctly distinguishes the two reports' roles. |
| B5 | Spec's top-of-file status line still read "DRAFT, in R0 … no code may be written" | **FIXED** | `design/SPEC_mt_v0_1.md:3`: *"Status: **GREEN — 0 Critical / 0 Important, 2026-08-23.** Closed after R6 … and R7's fold verification, then held green through three live journey walks, two out-of-scope sweeps and the pre-implementation gates R11/R12."* A blockquote directly below names the stale line explicitly and explains why it survived (predated R6/R7, outside every intervening grep). Zero remaining hits for "DRAFT, in R0" or "no code may be written" anywhere in the spec. |
| B4 | S0's transaction-shape constraints (segwit input, past nLockTime) silently dropped by the regtest fold | **FIXED** | Plan lines 196–201, immediately after the "produced on regtest" requirement: *"**Shape, restored after the regtest fold dropped it (R12 B4):** at least one **segwit** input … and a **non-zero `nLockTime` set to a PAST height** … Core's regtest wallet defaults to native segwit, so the first is satisfied without asking; the locktime is not, and must be passed to `walletcreatefundedpsbt`."* Confirmed no downstream phase depended on the more specific pre-fold wording (P2WPKH/P2TR) that was not restored — `grep -n "P2WPKH\|P2TR"` returns zero hits anywhere in the plan, so nothing is left dangling on the more specific (now-generalized) shape. |
| B3 | Three stale character-count prose sites (90-chars aside, 88/89 error example, untouched historical box) | **FIXED**, with one new arithmetic slip inside the fix — see Section B.1 | Site 1 (spec ~948, grouping aside): "90 characters" → "~90 characters" (now an approximation, consistent with the table's range of 80–91). Site 2 (spec ~1015, error example): "string 7: 88 characters (expected 89)" → "string 7: 89 characters (expected 90)" — 90 now matches a real table value (535-byte case, full string); the corrupted "89" is intentionally one less than a valid length (that's the point of the example) and need not itself appear in the table. Site 3 (spec ~1268–1277, historical box): now labelled as historical ("at the 49-bit header this box was written under") and extended with a recomputed 55-bit-header figure — but that added figure is itself wrong; see Section B.1. |

**All five FIXED as R12 scoped them.** One new defect was introduced by the B3 fold itself (Section B.1, Minor, non-blocking).

### Gate scripts re-run directly (not trusted from the commit message)

```
./scripts/spec-structure-check.sh design/SPEC_mt_v0_1.md   → sections: 17 ; cross-refs checked: 59 ; STRUCTURE OK (exit 0)
./scripts/plan-cite-check.sh design/IMPLEMENTATION_PLAN_mt_v0_1.md design/SPEC_mt_v0_1.md
   → design/IMPLEMENTATION_PLAN_mt_v0_1.md: 5 / 5 resolved, 0 dangling, 0 ambiguous
   → design/SPEC_mt_v0_1.md:                34 / 34 resolved, 0 dangling, 0 ambiguous
```
Matches the commit message's "34/34 and 5/5, 0 dangling" claim exactly — machine-checked, not read.

### Arithmetic recomputation (independent, formula from the brief)

`count = ceil(len/40)`, `bytes_per_chunk = ceil(len/count)`, `last = len − (count−1)·bytes_per_chunk`,
`strlen(b) = 3 + ceil((55 + 8b)/5) + 13`:

- **162 bytes:** count = ceil(162/40) = 5; bytes_per_chunk = ceil(162/5) = 33; last = 162 − 4×33 = 30.
  strlen(33) = 3 + ceil((55+264)/5) + 13 = 3 + ceil(63.8) + 13 = 3+64+13 = **80**.
  strlen(30) = 3 + ceil((55+240)/5) + 13 = 3 + ceil(59) + 13 = 3+59+13 = **75**.
  Total = 4×80 + 75 = **395**. **Matches the fold's claim exactly.**
- **535 bytes:** count = ceil(535/40) = 14; bytes_per_chunk = ceil(535/14) = 39; last = 535 − 13×39 = 28.
  strlen(39) = 3 + ceil((55+312)/5) + 13 = 3 + ceil(73.4) + 13 = 3+74+13 = **90**.
  strlen(28) = 3 + ceil((55+224)/5) + 13 = 3 + ceil(55.8) + 13 = 3+56+13 = **72**.
  Total = 13×90 + 72 = **1,242**. **Matches "90/72, total 1,242" exactly.**

Both spot-checks agree with the fold's arithmetic. **One disagreement found elsewhere** — reported in Section B.1, not in either of the two cases the brief named for spot-checking.

---

## Section B — new defects

### B.1 (Minor, non-blocking) The B3 fold's own added sentence in the historical box is arithmetically wrong

`design/SPEC_mt_v0_1.md:1274–1276` (new text added by this fold):

> "at the ruled 55-bit header **the same string is 90** and the 162-byte
> five-chunk artifact totals **395** characters (four strings of 80 plus a last
> of 75 — §10.13 a2's table, recomputed)."

The "same string" is the box's running example of a **maximal 40-byte chunk**
(`b = 40`, i.e. the 320-bit budget the box's untouched sentence two lines above
still cites: `41 + 320 = 361`, `49 + 320 = 369`). Recomputing `strlen(40)` at the
**current, ruled 55-bit header**, using the exact formula given and already
validated above: `3 + ceil((55 + 320)/5) + 13 = 3 + ceil(75) + 13 = 3+75+13 =
91`, not 90. This is not a guess — it matches the spec's own already-verified
table three rows independently (742/40/**91**/63; 560/40/**91**/91;
2,498/40/**91**/56 — every `b = 40` row in the table reads 91). Growing the
header from 49 to 55 bits (+6 bits) crosses a `ceil()` boundary at `b = 40`
(369→375, `ceil(369/5)=74` vs `ceil(375/5)=75`) that it does not cross at
smaller `b` (e.g. `b = 33`: `ceil(319/5)=64` vs `ceil(375-... )`... — the
162-byte case's own 80/75 figures, checked above, are unaffected), so "the
widening costs nothing further" does not generalize to the very case ("the
same string") the sentence names.

This is the identical defect class R11 and R12 both caught repeatedly this
cycle (stale/unrecomputed character counts) — reintroduced inside the same
sentence whose neighbor clause *was* correctly recomputed (395, verified
above), and inside the commit whose own message says "recompute, do not
assert." **Not blocking**: this text is in a historical, non-normative aside
(§10.23's box on the header-width ruling); R12 already established that none
of I3's three sites are read by any gate or test (P1/P3 read the generator's
computed values, not this prose), and the box's own headline claims — the
`45+320`/`49+320` bit-budget arithmetic, the 15-bit-per-field ruling, the
13.1× headroom figure — are untouched by this fold and remain correct. Also
confirmed: the "162-byte five-chunk artifact" is the **abandoned mainnet
candidate's size** (plan line 216, kept as a labelled aside, not the real S0
regtest vector — the actually-built vectors are 222 B and 284 B per the
controller's already-verified work), so the 395 figure is illustrative only
and not load-bearing anywhere. Recommend fixing "90" → "91" whenever
convenient; does not gate tonight's implementation.

### No other new defects found

Checked and clear:
- **Spec/plan status-line contradiction** — spec now claims "GREEN — 0
  Critical / 0 Important" (line 3); plan's headline claims only "GREEN — 0
  Critical" (dropping the "/0 Important" the pre-fold broken version had).
  Not a contradiction: the plan's own text never asserts 0 Important as a
  self-verified fact, and B1 (the one open Important from R12) is fixed by
  this same commit, so both are consistent with the true post-fold state.
  Reads as intentional restraint (not asserting a count only an independent
  reviewer should bless) rather than an error; not flagged as a defect.
- **P0's expanded three-file list vs. anywhere else enumerating those files**
  — grepped for `SPEC_mt_v0_1.md\` and`, `two files`, and the bare
  `mt1_v1_vectors.md` filename outside P0/S0: no stale two-file enumeration
  survives elsewhere. S0's own deliverable list (two vector forms, `.md` +
  `.json`, plus the generator) is a different, still-correct list — it is not
  P0's three-file *copy* list and does not contradict it.
  P1's pinning language ("P1 pins **that** file's hash", S0 blockquote line
  188) correctly refers to the path P0 now names.
- **B4's generalized shape vs. downstream phases** — grepped the whole plan
  for `P2WPKH`/`P2TR`: zero hits anywhere, so no downstream phase (P2, P4) was
  relying on the more specific pre-fold wording that was not restored.
- **`plan-cite-check.sh` / `spec-structure-check.sh`** — both re-run directly
  above, both clean, matching the commit message's claimed counts exactly.

---

## Section C — executability walk (S0 → P6)

Walked briefly as an implementer, given eight prior lenses have already run
and this fold touched only the opening status block, P0's deliverable list,
S0's constraints, and four spec prose sites (three of them non-normative
prose, the fourth — B3's historical box — is also non-normative and now
carries the one Minor above).

- **S0**: vector provenance (regtest, never broadcast), all four recorded
  forms, shape constraints (segwit + past locktime), and the JSON/`.md`
  dual-output are all now stated in one place an implementer reads before P0.
  Nothing left to guess.
- **P0**: three-file deliverable list is now complete and matches what P1
  needs; license/toolchain pins present; gate (`build`+`clippy`+`fmt`,
  deliberately no `nextest`) is satisfiable as stated (0.9.140's
  `--no-tests=fail` behavior already reasoned through and cited).
- **P1**: pin test target path now has a source (P0's copy step); five
  tests-first items and the gate are unambiguous; no dependency on the fixed
  historical-box sentence (B.1) anywhere in P1's own text.
- **P2–P6**: untouched by this fold; already walked clean by R11/R12 and not
  re-derived here per the brief's efficiency instruction. Spot-checked P4's
  node-fixture/offline gate and P5's refusal table for any dependency on the
  four edited spec prose sites — none found.

**Nothing found that would halt an unattended implementer or force a guess.**

---

## Section D — verdict

**SAFE TO EXECUTE.** All five of R12's findings (B1 blocking, B2/B3/B4/B5
Minor) are FIXED, verified by direct grep/read against the current file
state and independent arithmetic recomputation (both spot-checks in the
brief match exactly, plus the two named 6-row tables from R12 remain
internally consistent). Both gate scripts re-run clean (34/34, 5/5, 0
dangling; STRUCTURE OK). One new Minor was found — B.1, a stale/wrong
character count ("90" should be "91") inside the very sentence added to fix
a *different* stale character count, in a non-normative historical aside not
read by any gate or test. It does not block: nothing in P0–P6 depends on it,
and it carries none of the properties (Critical: wrong result / data loss /
security / unmet guarantee; Important: real defect, missing case, unsound
assumption reachable by an implementer) that would gate tonight's run.

**0 Critical, 0 Important, 1 Minor (new, non-blocking).**
