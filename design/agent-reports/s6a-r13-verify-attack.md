# S6a R13 — VERIFY THE FOLD'S CLAIMS + ADVERSARIAL ATTACK

**Artifact:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
**Fold under review:** `git diff 2453561~1..2453561` (R12's fold, "the record could not tell
single-sig from multisig")
**Code:** `/scratch/code/shibboleth/seedhammer`, `main` = `2b9a12805f6b1b71d71137354971d34394b2364a`
**Prior report checked against:** `design/agent-reports/s6a-r12-closing.md` (1C/2I)

## VERDICT: RED — 1 Critical, 1 Important (+ 0 filed)

---

### C-1 — R12's C-1 is dressed as closed but is not: `suppliedCosigners` is declared,
never written, never read by the line-generation rule, and never tested [MECHANICAL]

**Where:** §4.7b-seam's `passRecord` struct (the `suppliedCosigners int` field and its
comment, current lines 825–842), against §4.7c "THE FOUR LINES" (lines 880–905, the sole
stated authority for what `buildVerifyStatusLine` prints) and the test table T20–T26
(lines 1086–1104) and P6 (§4.7g, lines 969–987).

**The defect.** `grep -n -i supplied design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
over the whole 1427-line document returns exactly 3 lines — all three inside the single
struct-comment block that declares the field (lines 831, 839, 841). Nowhere else in the
document does `suppliedCosigners` appear. Specifically absent:

- **A write-site.** No line says what value is stored into `suppliedCosigners` at the
  multisig success return (`gui/multisig_verify.go:986-987`) or at any of single-sig's
  eleven exits (build-order step 1). Contrast with `full` and `legs`, the struct's other
  two fields: both are *already-named, in-scope values* at the cited line (`full` is a
  function parameter of `multisigVerifyFlow`; `legs` is the local slice `verifyMultisigLegs`
  already computed), so citing the line is a sufficient specification — there is nothing to
  compute, only to capture. `suppliedCosigners` has no such existing named value anywhere
  the plan cites; producing one requires **new** computation (see below), and the plan does
  not do it or point at what it would use.
- **A generation rule.** §4.7c's table is the one place the plan says what each of the
  four rendered lines actually contains, and its "pass, no adverse" cell reads only:
  *"generated from the pass record — names exactly the comparisons this mode ran, and
  states what was not read."* No mention of a cosigner clause, a path, or `suppliedCosigners`.
- **A test.** T22 is explicitly captioned *"the pass line is GENERATED PER MODE (R9 C-1)"*
  and tests only the **mode** axis (full vs. watch-only ms1 clause) — not the **path** axis
  R12's C-1 was actually about. T26 (P6's per-claim, per-mode audit) likewise names no
  cosigner clause. No T-row anywhere asserts the clause's presence on a multisig document or
  its absence on a single-sig one — i.e. **the fix for a Critical again has nothing that
  could fail if it regressed**, the exact pattern §5's own preamble (lines 1177–1181) says
  was already once true of this cycle's original Critical.

**The harm.** Go zero-values an unassigned `int` field to `0`. Since nothing in the plan
writes `suppliedCosigners` and §4.7c's generation rule never reads it, a literal
implementation of the plan as written leaves it `0` on **every** run — single-sig and
multisig alike. The plan's own stated rendering rule (*"the clause renders iff
suppliedCosigners > 0"*) is then never true anywhere, so the cosigner-scoping clause never
renders on **any** multisig document. That reproduces R12's C-1 exactly on the arm R12 called
the worse one: *"Omit it → the multisig restore document asserts a verification with none of
the scoping the shipped screen was fixed to carry. G2 — the device claims a check it did not
perform."* The fold's commit message ("the record could not tell single-sig from multisig")
is not achieved by the text as written; a field was added, nothing connects to it.

**A landmine for whichever formula an implementer picks, found while checking whether "a
count" can even work.** `gui/multisig_verify.go:721` already decodes the readback policy —
`_, keys, err := md.ExpandWalletPolicyChunks(readbackMd1)` — so `len(keys)` (total wallet
participants) is in scope at the success return, and the obvious formula is
`len(keys) - len(legs)` (participants minus the legs this run itself verified). But
`gui/multisig_build.go:50` carries its own comment: *"an operator CAN now hold every slot, so
`open` can be 0"* (`open := p.N - len(p.SelfSlots)`, `gui/multisig_build.go:96`) — i.e. a
build where the operator holds and verifies every key in the policy is a real, already-coded,
already-named scenario (I-6). On that path `len(keys) - len(legs)` evaluates to `0` for a
**genuine multisig success**, colliding with single-sig's `0` again — the identical defect
shape R12 demonstrated, on a different edge. This is not proof the eventual write-site will
get it wrong; it is proof "is a count sufficient" is not answerable from the plan as written,
and that the answer is not obviously yes.

**Remedy — UNVERIFIED**, per instruction; not resolved against the call graph here. The shape
above (`len(keys) - len(legs)`, plus a §4.7c update and a new T-row covering the path axis) is
one candidate, not a prescription.

---

### I-1 — Step 7 breaks 8 more `multisigVerifyFlow` call sites and 1 stub assignment the
plan neither cites nor counts [MECHANICAL]

**Where:** §4.8 step 7 and §5's "four source assertions" table (current lines 1142–1157),
against §4.7b-seam's `func multisigVerifyFlow(..., rec *verifyRecord) multisigVerifyResult`.

**The defect.** R12's I-1 (folded, and independently re-verified below under PART 1 §2) found
four `strings.Contains` needles pinning `multisigVerifyFn(ctx, th, full, engravedSlots,
<x>Md1)` verbatim, and the fold added a table naming exactly those four. But `multisigVerifyFlow`
— the actual function step 7 re-signs — is also called **directly**, bypassing the
`multisigVerifyFn` seam var entirely, at 8 more sites, all with today's 5-argument shape:

```
$ grep -rn "multisigVerifyFlow(" --include="*.go" gui/ | grep -v "func multisigVerifyFlow\|var multisigVerifyFn"
gui/multisig_verify_policy_test.go:177
gui/multisig_supply_multislot_test.go:271
gui/multisig_verify_flow_test.go:118
gui/multisig_verify_flow_test.go:224
gui/multisig_verify_flow_test.go:250
gui/multisig_verify_report_test.go:38
gui/multisig_verify_report_test.go:348
gui/multisig_verify_report_test.go:576
```

None of these 8 lines, and none of their 4 host files (`multisig_verify_policy_test.go`,
`multisig_supply_multislot_test.go`, plus the 2 already partly cited for other reasons), are
named anywhere in the plan (grepped by filename; zero hits for the first two). These are not
needle assertions that merely go red — they are direct Go function calls with too few
arguments, so adding the `rec *verifyRecord` parameter is a straight `not enough arguments in
call to multisigVerifyFlow` **compile error** at each site, which fails the whole `gui`
package's build, not just one named test. Five of the eight sites are inside shared
`t.Helper()` drivers (`s5DriveVerifyStopAfterOneSeed`, `s5DriveVerifyTolerant` — 3 callers
across 3 more files including the uncited `multisig_supply_dupslot_test.go` —
`s5DriveVerifyTwoSeeds`, `s5DriveVerifyFirstSeedRefused`, `s5DriveVerifyFullTwoSeeds`), so the
true compile blast radius is upward of a dozen test functions across at least 6 files, not 8
lines.

Separately, and smaller: the plan's own sentence (line ~1155) says step 7 must update *"the
`multisigVerifyFn` stub assignment those tests set."* None of the four cited tests sets that
stub. The assignment is a fifth, uncited file — `gui/multisig_engrave_tail_walk_test.go:105`,
`s5StubVerifyFn`'s `multisigVerifyFn = func(ctx *Context, th *Colors, full bool, expectedSlots
[]int, engravedMd1 []string) multisigVerifyResult {...}` — which will *also* fail to compile
once the var's inferred type gains the `rec` parameter (a type mismatch, not a needle miss).
The sentence gestures at the right problem but attributes it to the wrong tests and cites no
file:line, unlike every other claim in this section.

**Checked and cleared, for contrast — this is not a systemic gap in the plan's blast-radius
accounting.** The two other signature changes in this build order are fully accounted for:
`restoreDocFlow(` has exactly 1 call site, `gui/singlesig.go:136`, 0 test call sites — matches
§4.2 exactly. `multisigRestoreDocFlow(` has exactly 3 — `gui/multisig.go:361`,
`gui/multisig_build.go:478`, `gui/multisig_nested_name_test.go:230` — matches §4.2/§4.8's "all
three call sites" exactly, and the test site is cited by name. `singleSigVerifyFlow(` has
exactly 1 call site, `gui/singlesig.go:132`. Only `multisigVerifyFlow`'s blast radius is
undercounted.

**Which goal it breaks:** none directly; makes step 7 unexecutable as literally scoped — the
same class R12 rated Important (I-2) for a different cause, recurring one round later on the
very fold meant to close it.

**Remedy — UNVERIFIED.** Presumably each of the 8 sites needs a `nil` or `&verifyRecord{}`
sixth argument and the :105 stub literal needs the added parameter; not resolved against the
call graph.

---

## PART 1 — CLAIMS VERIFIED

**1. `suppliedCosigners` — the four multisig notices, the single-sig omission, the pinning
test.**

- `gui/multisig_verify.go:32` — `multisigVerifyOKBody = "Operator key and secret verified.
  Other cosigners' keys are taken as supplied."` — confirmed, ends with the clause.
- `gui/multisig_verify.go:1053` — `!full` single-leg arm — `"Operator key verified. Other
  cosigners' keys are taken as supplied."` — confirmed.
- `gui/multisig_verify.go:1059` — `full`, multi-leg arm — `"...for each seed. Other cosigners'
  keys are taken as supplied.", legs)` — confirmed.
- `gui/multisig_verify.go:1062` — `!full`, multi-leg arm — `"...are taken as supplied.",
  legs)` — confirmed. **All four cited lines verified verbatim; all four end with the clause.**
- `gui/singlesig_verify.go:148` — `showNotice(ctx, th, "Verify OK", "The engraved bundle
  matches the seed.")` — confirmed, no cosigner clause; this is the whole function's only
  success screen.
- `gui/multisig_verify_test.go:171` — `func TestMultisigVerifyNoticeIsHonest(t *testing.T)` —
  confirmed at that exact line; the body asserts `uiContains(content, "taken as supplied")` and
  `!uiContains(content, "matches the seed")`, i.e. it pins the split precisely as the plan
  describes.
- **Is a count sufficient?** Not verifiably — see **C-1** above. Checked
  `gui/multisig_build.go` (`p.SelfSlots`, `open := p.N - len(p.SelfSlots)`, line 96) and
  `gui/multisig.go` (no equivalent multi-slot-holding machinery on the supply path). Found a
  real, codebase-documented edge case (self-multisig, `open == 0`, `gui/multisig_build.go:50`
  comment "I-6") where a natural formula for "count of un-verified cosigners" is 0 for a
  genuine multisig success. The plan never specifies the formula, so this cannot be resolved
  either way from the artifact — which is itself the finding.

**2. The four source assertions (R12 I-1) and whether the plan still misses others.**

All four confirmed exact, needle text including the closing paren:

```
gui/multisig_verify_report_test.go:1079: "multisigVerifyFn(ctx, th, full, engravedSlots, suppliedMd1)"},
gui/multisig_verify_report_test.go:1081: "multisigVerifyFn(ctx, th, full, engravedSlots, engraveMd1)"},
gui/multisig_verify_flow_test.go:373:    if !strings.Contains(body, "multisigVerifyFn(ctx, th, full, engravedSlots, engraveMd1)") {
gui/multisig_verify_flow_test.go:394:    if !strings.Contains(body, "multisigVerifyFn(ctx, th, full, engravedSlots, suppliedMd1)") {
```

`grep -rn "multisigVerifyFn(" --include="*_test.go" gui/` returns exactly these 4 lines — no
fifth needle against the seam var was missed. **But the plan asks the wrong question by
scoping to the seam alone** — see **I-1** above: 8 more sites call the underlying function
directly and are missed.

**3. The build order after R12's reshuffle (steps 1–9).**

Walked all nine rows. Steps 1–4 add pure functions / update fully-enumerated call sites (see
I-1's "checked and cleared" paragraph for `restoreDocFlow`/`multisigRestoreDocFlow`) and leave
the tree green; nothing in steps 1–4 depends on anything created later. Steps 5+6+7 land as one
commit, and the ordering logic within that bundle (5 alone red on the three walks; 5+6 without
7 green but reproduces C-1's harm) is sound and unchanged by this fold except for the T20/T7c
relocations, which are internally consistent with §5's cited reasons. Steps 8–9 are independent
and correctly sequenced last. **One thing checked and NOT flagged:** step 3's cell says
"updating the six existing call sites"; §4.3 measures 2 production + 6 test = 8 existing sites.
R12's own report uses the identical "six" shorthand for the test sites only
(`design/agent-reports/s6a-r12-closing.md:228`, "All six existing test call sites..."), so this
is established, consistent usage in this document, not a new ambiguity — not filed.

**The one genuine unexecutability in the build order is I-1 above** (step 7's blast radius),
which the "why here" reasoning of row 7 does not anticipate.

## PART 2 — ATTACK FINDINGS

**4. The seam, end to end.** Traced single-sig (declare local `rec verifyRecord`before the
verify-offer choice in `gui/singlesig.go`; `singleSigVerifyFlow` conditionally writes it if
"Verify now" is chosen; `restoreDocFlow(..., buildVerifyStatusLine(rec), ...)` always reads it,
whether or not verify ran) and multisig build/supply (equivalent pattern via
`multisigVerifyFn`/`multisigRestoreDocFlow`). The **out-parameter cannot be nil** in production:
`rec` is a caller-local value (`verifyRecord`, not a pointer) passed by address, and a skipped
verify leaves it at its zero value, which the `default:` switch arm handles safely — this
matches R12's own note ("A skipped verify is structurally safe") and is not a new finding. **The
one field that is written nowhere and read nowhere is `suppliedCosigners`** — see C-1; this *is*
the "record written but not read" (more precisely: neither) case the prompt asked to hunt for.

**5. Consistency across the whole file.** `grep -n "passRecord{"` → 1 hit, the struct
declaration only; no stale `passRecord{full, legs}` literal survives. `grep -n "T9\b\|T13a\|T13b"`
→ 1 hit, and it is the corrective sentence itself ("An earlier draft justified this step with
T9/T13a/T13b, tests that exist nowhere in this plan") — not a live citation. No description of
the record as two-fielded survives outside the historical framing at line ~820-824, which is
explicitly narrating the *prior* defect. Clean.

**6. Anything making step 1 impossible or the first commit red.** Step 1 is a review artifact
(a table), not code — it cannot be "red" in a build-gate sense. R12's M-1 (step 1's brief still
says "verifyStatus mapping" rather than "record-write mapping") is untouched by this fold and
remains open, but is a wording Minor, not gating, and was already filed. No new step-1 blocker
found.

## GATES — ACTUAL OUTPUT

```
$ export PATH="/nix/var/nix/profiles/default/bin:$PATH"
$ cd /scratch/code/shibboleth/mnemonic-engrave
$ for g in verify-returnsite-sweep plan-cite-check plan-glyph-check plan-table-check; do
    ./scripts/$g.sh design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md; echo "$g exit=$?"
  done
```

- `verify-returnsite-sweep.sh` → **15 verdict return sites; 0 unrowed. exit=0**
  (scope warning printed, as documented: single-sig contributes zero sites, multisig-only
  coverage — this is the gate's declared, honest limitation, not a defect)
- `plan-cite-check.sh` → **102 / 102 citations resolved, 0 dangling. exit=0**
- `plan-glyph-check.sh` → **57 operator strings scanned, 0 undrawable. exit=0**
- `plan-table-check.sh` → **76 table rows checked, 0 malformed. exit=0**

All four gates GREEN, stdout/stderr captured separately (no "Git tree is dirty" contamination
observed in either stream). These gates corroborate PART 1's line-citation checks
mechanically; they do not and cannot check whether a declared field is actually connected to
anything, which is what C-1 and I-1 above are about.

## WHAT WAS NOT DONE

No codebase audit for pre-existing defects. No prose/markdown review. No re-litigation of the
four-state design, the goals, or NG1/NG2. Remedies for both findings above are marked
UNVERIFIED and not resolved against the call graph, per instruction.
