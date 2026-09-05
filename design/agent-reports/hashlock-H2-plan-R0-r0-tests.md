# Hashlock H2 device plan — R0 round 0, independent tests/mutation review

Reviewer: sonnet tier, independent tests/mutation reviewer (per
`design/agent-briefs/hashlock-H2-plan-R0-r0-tests-brief.md`).
Plan under review: `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md` at engrave
`02abee6`. Spec: `design/SPEC_hashlock_H2_device.md` (GREEN `55ee7a4`). Fork:
`/scratch/code/shibboleth/seedhammer` main `c4a64fc` (read-only; verified
`git rev-parse HEAD` == `c4a64fc0fd334a7943cb0a22ad290c26c96c687f`, clean tree).
Corpus: ms `crates/ms-codec/tests/vectors/hashlock-v0.8.json` at `cd0a60f`,
sha256 `a46c197a…1d30` (verified byte-identical to the plan's pinned literal).

Work performed entirely in `/scratch/code/shibboleth/.tmp/h2-tests`, a fresh
tar-extracted copy of the fork, per the brief. `/scratch/code/shibboleth/.tmp/h2-gate`
was read twice for reference only (to confirm which files the gate had already
wired, and to resolve one naming ambiguity), never copied from wholesale — every
file below was typed from the plan's own text and run RED before GREEN. Go used
throughout: `/scratch/code/shibboleth/.toolchain/go/bin/go` (`go1.26.7`). Nothing
committed. No sub-agents used. No `.jsonl` read.

**ONE QUESTION answered:** can every test the plan adds actually FAIL on the
defect it names, does the plan's RED/GREEN/mutation story hold when wired from
the plan's text, and which mutations of the guards survive every test?

**Answer:** mostly yes for the plan's own eleven declared mutations across Tasks
1/2/4 — all eleven reproduce the plan's (or the gate's corrected) claim exactly,
quoted below. But the review found one test that is a false PASS relative to
its own stated purpose (Critical), one RED that does not reproduce as claimed
(Important), and five guard/contract mutations that survive the ENTIRE narrow
test selection with zero coverage anywhere in the plan (Important) — plus two
Minor/Nit items. See counts at the end.

---

## 1. RED steps, verbatim

**Task 1 Step 3** — `go test -count=1 ./hashlock/` before `hashlock.go` existed:

```
hashlock/hashlock_test.go:82:8: undefined: PreimageHardened
hashlock/hashlock_test.go:86:17: undefined: Digest
hashlock/hashlock_test.go:89:9: undefined: PreimageSHA256
hashlock/hashlock_test.go:97:15: undefined: DeriveHardened
hashlock/hashlock_test.go:162:10: undefined: ValidatePhrase
hashlock/hashlock_test.go:170:13: undefined: ErrEmpty
...
FAIL	seedhammer.com/hashlock [build failed]
```
Matches the plan's claim ("does not compile … undefined") exactly.

**Task 2 Step 2** — `go test -count=1 -run TestDecodeMS1PreimageIsShapeExact ./codex32/`
before `DecodeMS1Preimage` existed:

```
codex32/mspayload_test.go:135:12: undefined: DecodeMS1Preimage
codex32/mspayload_test.go:154:16: undefined: DecodeMS1Preimage
codex32/mspayload_test.go:165:15: undefined: DecodeMS1Preimage
FAIL	seedhammer.com/codex32 [build failed]
```
Matches the plan's claim exactly.

**Task 3 Step 1** — `go test -count=1 -run TestWhichHashRowsAreLabelKeyed ./gui/`
before `composerHashRows`/`composerHashRowSet` existed (verified by temporarily
swapping in the untouched fork's original `composer_hash.go` and re-running):

```
gui/composer_hash_test.go:79:11: undefined: composerHashRows
gui/composer_hash_test.go:83:37: undefined: composerHashRowPhrase
FAIL	seedhammer.com/gui [build failed]
```
Matches the plan's claim exactly.

**Task 4 Step 2** — `go test -count=1 -run TestHashlock ./gui/` with Task 3's own
prescribed stub (`gui/composer_hashlock.go` returning `hashlockBackToWhichHash`
unconditionally — Task 3 Step 5 directs exactly this stub) already in place, but
before Task 4's real route:

```
--- FAIL: TestHashlockPhraseRouteSetsTheCorpusDigest (2.04s)
    --- FAIL: .../hardened_anchor (1.01s)
        composer_hashlock_test.go:305: no *PassphraseKeyboard was registered for this harness
    [... 9 more of the same across TestHashlockPhraseRouteDoesNotNormalise,
     TestHashlockBackContractKeepsThePath, TestHashlockDeclineThenHardenedTypesOnce,
     TestHashlockPhraseRefusalsOnScreen (x4), TestHashlockMethodModalsFireOnCondition,
     TestHashlockConfirmRelationLine]
FAIL	seedhammer.com/gui	11.096s
```

**This does NOT match the plan's claim of "does not compile."** See Important
finding I-1 below: with the stub the plan itself directs building in Task 3, all
symbols `composer_hashlock_test.go` needs already resolve, so the file compiles
cleanly and every new test fails at RUNTIME instead, for the same reason
(the stub never calls `hashlockPhraseFlow`, so no keyboard is ever registered).

---

## 2. Declared mutations

### Task 1 Step 5 (six, all confirmed exactly)

| # | Mutation | Plan/gate claim | Measured (this review) |
| --- | --- | --- | --- |
| 1 | `Salt = append(Salt, 0, 0)` | 22 failures (11 rows × hardened X+H) | **Confirmed.** 22 failure lines, all `hardened X`/`hardened H`, 0 sha256 lines. Quoted: `"correct horse battery staple" hardened X: got 81b38099… want c3e97525…` |
| 2 | `Iterations = 99999` | 22 failures, same shape | **Confirmed.** 22 failure lines, identical shape. Quoted: `"correct horse battery staple" hardened X: got bd2905fe… want c3e97525…` |
| 3 | `NormalisePassphrase` at top of `PreimageHardened` | 4 failures: `Correct Horse Battery Staple` + `  a  b ` (X+H each) | **Confirmed exactly.** Only those two rows failed (4 lines total), quoted: `"  a  b " hardened X: got 4e02d909… want cae9f566…`, `"Correct Horse Battery Staple" hardened X: got c3e97525… want 865125fb…` |
| 4 | strip `-`/`,` from phrase first | gate-corrected: 4 rows fail (`correct-horse,battery staple`, `a-b,c`, and both 64/65-char rows containing `-`/`,`), not 1 | **Confirmed exactly** — reproduced the gate's correction, not round 0's original claim. 8 failure lines across exactly those 4 phrases. |
| 5 | `IsMS1Shaped` via `codex32.New` (checksum parse) | rows 11, 12, 13 fail (grouped-by-5, leading/trailing spaces, grouped-by-2) | **Confirmed exactly.** `row 11 rule ms1-shaped: got <nil> want …`, `row 12 … got <nil> want …`, `row 13 … got …TooLong… want …MS1Shaped…` — same three rows, no others. |
| 6 | cap literal `99` | only `TestPhraseMaxCharsIsTheCap` fails (corpus's sole `too-long` row is 101 chars, refused either way) | **Confirmed exactly.** All 5 other tests PASS; only `TestPhraseMaxCharsIsTheCap` FAILs. |

### Task 2 Step 4 (one, confirmed)

| Mutation | Plan claim | Measured |
| --- | --- | --- |
| drop `!f.Unshared` clause | the 2-of-N share case returns a value where `errMSBadPrefix` is wanted | **Confirmed exactly.** `DecodeMS1Preimage(a 2-of-N share beginning 0x03) err = <nil>, want codex32: not an m-format secret payload` |

### Task 4 Step 4 (five, all confirmed; one wording note)

| Mutation | Plan/gate claim | Measured |
| --- | --- | --- |
| fold `phrase` through `seal.NormalisePassphrase` in `hashlockPhraseFlow` | `TestHashlockPhraseRouteDoesNotNormalise`: `Correct Horse Battery Staple` fails | **Confirmed exactly.** `"Correct Horse Battery Staple": path hash = &[184 103 219 …], want 95d4447031cdc4117f797040c1a9e32367af2a8d97554e442c7bfd002297a7ff` |
| confirm's Back returns `hashlockBackToWhichHash` | `TestHashlockBackContractKeepsThePath`: never reached "Which method?" | **Confirmed exactly.** `never reached "Which method?"; last frame "Path1hash…"` |
| `composerHashEdit` returns `false` from the phrase route's Back | gate-corrected: fails EARLIER, at never-reached "Type a hashlock phrase" | **Confirmed exactly** — reproduced the gate's correction. `never reached "Type a hashlock phrase"; last frame "qwertyuiop…"` |
| remove the relation line | `TestHashlockConfirmRelationLine`: never reached "matches hash 1 in the payload" | **Confirmed exactly.** Same quoted failure. |
| delete the release event from `holdConfirm` | "every test with 2+ holds hangs at its second one" | **Confirmed the mechanism, one wording nit (N-1 below).** The test does not hang forever; it fails cleanly via the harness's own pump-timeout (`h.next`/`mustReach` give up after a bounded number of frames). `TestHashlockBackContractKeepsThePath` (3 sequential holds) failed at `never reached "Which method?"` after the second hold's press was mis-routed to the first hold's stale `Clickable`, matching the described mechanism exactly — just not a literal infinite hang inside `go test`. |

All eleven declared mutations were reverted and GREEN re-confirmed individually.

---

## 3. My own mutations

| # | Mutation | Result | Detail |
| --- | --- | --- | --- |
| (i) | `IsMS1Shaped` without `TrimSpace` | **SURVIVED** | The character-stripping loop that follows already removes space/tab/CR/LF/`-`/`,` at EVERY position, not just the string's boundaries, so `TrimSpace` is provably redundant in this implementation — a leading/trailing-space phrase is already de-spaced by the loop regardless. Zero test failures across the whole `hashlock` package. |
| (ii) | `IsMS1Shaped` with `minMS1Len = 47` | **SURVIVED** | No corpus or plan test row sits at the 47/48 boundary — see corpus-sufficiency §5. Zero test failures. |
| (iii) | `ValidatePhrase` checking the cap BEFORE the shape test | **CAUGHT** | `TestRefusalRowsMatchTheHost`, row 13 (`grouped by 2, 112 chars`): `got …TooLong… want …MS1Shaped…` — the corpus's own row exists precisely to pin this order (per spec §2). |
| (iv) | `DeriveHardened` ignoring `progress`'s `false` (Back no longer abandons the driver itself) | **SURVIVED** | `hashlock` package: 6/6 still PASS (the only `DeriveHardened` unit test passes an always-`true` progress func, so it never exercises the abandon path at all). GUI: `TestHashlockBackContractKeepsThePath` still PASSES, because `hashlockDeriveFlow` tracks its OWN `abandoned` bool, set directly from `backBtn.Clicked(ctx)` inside the same progress callback, independent of what `DeriveHardened` does with the callback's return value — so the GUI's outer check (`if !ok \|\| abandoned`) still returns `false` regardless. `DeriveHardened`'s own early-return contract has no test anywhere in the plan that depends on it. |
| (v) | relation line computed against `st.list`'s own path digests instead of the payload's | **CAUGHT** | `TestHashlockConfirmRelationLine`: `never reached "matches hash 1 in the payload"` (the path being created has no `Hash` yet at that point, so `st.list` digests are empty). |
| (vi) | confirm modal's HOLD assigning BEFORE the screen returns (assign, then confirm) | **CAUGHT** | `TestHashlockBackContractKeepsThePath`'s explicit assertion: `hash assigned before HOLD`. |
| (vii) | `hashByPhrase` never set | **SURVIVED** | Grepped every `_test.go` in `gui/`: the field is referenced in exactly ONE place, `composer_copy_test.go`'s `composerCopyTable` row, which constructs `&composerState{hashByPhrase: true}` MANUALLY rather than driving the real route. Zero test in the narrow Task 1-4 selection exercises the real assignment (`st.hashByPhrase = true` in `hashlockPhraseRoute`). Deleting that one line: all tests in the narrow selection still PASS. |
| (viii) | `composerHashEdit`'s `default:` arm clearing the lock again (instead of panicking) | **SURVIVED, but inert by construction** | `rows.labels` always has exactly `len(digests)+3` entries and `composerPickScreen` only ever returns an index into that slice, so `sel` is always one of the four named cases — `default:` is genuinely unreachable under normal operation. Not a live gap; noted for completeness only. |

### An additional, unplanned mutation this review ran to resolve a discrepancy

While applying (viii)'s sibling class, `TestWhichHashRowsAreLabelKeyed`'s own
comment claims: *"MUTATION: restore the index arithmetic with the new row
inserted -> 'Type 64 hex' lands in the clearing arm and this fails."* This
review reverted `composerHashEdit` to the pre-H2 index-arithmetic dispatch
(keeping `composerHashRows`' row construction untouched) and ran it two ways:

- **With only Task 3 wired** (Task 4's stub route in place): `TestWhichHashRowsAreLabelKeyed`
  and the rest of the Task 3 narrow selection all **PASS** — this specific test
  never drives `composerHashEdit`'s switch at all, only `composerHashRows`' row
  construction, so its own comment's credit is **wrong for this test in
  isolation**.
- **With Task 4 fully wired**, the same mutation produces 10 distinct failures
  across `TestHashlockPhraseRouteSetsTheCorpusDigest` (both subtests),
  `TestHashlockPhraseRouteDoesNotNormalise`, `TestHashlockBackContractKeepsThePath`,
  `TestHashlockDeclineThenHardenedTypesOnce`, `TestHashlockPhraseRefusalsOnScreen`
  (all 4 subtests), `TestHashlockMethodModalsFireOnCondition`, and
  `TestHashlockConfirmRelationLine` — all `never reached "Hashlock phrase"`,
  because tapping the phrase row now incorrectly opens hex entry (`sel == len(digests)`
  under the old arithmetic is the phrase row's actual index, not the hex row's).

So the *plan as a whole* does catch this regression once fully wired, but the
*specific test the comment credits* does not. Important (I-2 below).

---

## 4. False-PASS hunting

**(a) Does the keyboard itself normalise, so `TestHashlockPhraseRouteDoesNotNormalise`
passes for the wrong reason?** Read `gui/passphrase_keyboard.go` in full.
`commit()`'s `ppRune` case is `k.Fragment += string(key.r) // NO ToUpper — case
preserved`; `Clear()` only resets to `""`; there is no trim, case-fold, space
collapse, or Unicode normalisation anywhere in `Update`/`commit`/`Clear`. Every
character (lowercase page, uppercase page, symbols pages, the literal space key)
is appended verbatim. **Not a false-PASS**: the keyboard is confirmed byte-transparent,
so `ValidatePhrase` genuinely receives what was typed.

**(b) Does `mustReach("28/100")` prove the phrase survived, or would an empty
phrase screen also show a counter?** Verified empirically: mutated
`hashlockPhraseFlow` to ignore `initial` (`kbd.Fragment = ""` unconditionally).
Result: `TestHashlockBackContractKeepsThePath` fails at
`never reached "28/100"; last frame "…0/100Hashlockphrase"` — a dropped phrase
shows `0/100`, not `28/100`. **Not a false-PASS**: the assertion is a genuine
(if length-only, not byte-exact) proof that `initial` was threaded back into
the keyboard; byte-exact survival is separately proven elsewhere by the digest
match in `TestHashlockPhraseRouteSetsTheCorpusDigest`/`…DoesNotNormalise`.

**(c) Does `TestModalsThisBlockTouchesAreDrawnInFull`'s table use the same
modal renderer `composerConfirmScreen` actually draws?** No, for 2 of the 5 new
rows. `composerCopyHashlockHardenedWarning()` and `…SHA256Warning()` are drawn
in the real flow via `composerConfirmScreen` → `ConfirmWarningScreen` (the
hold-to-confirm shape) — and this same test file already defines a
`confirmWarningBody` renderer specifically for that shape (its own doc comment:
*"renders ConfirmWarningScreen — the hold-to-confirm shape"*) — but the plan's
new rows measure all three confirm-related bodies (both warnings and the final
confirm body) through `errorScreenBody` (the `showError`/`ErrorScreen`
renderer) instead. Measured both ways for all three bodies: headroom is
**identical** under either renderer (397/360/186 chars respectively), because
`ErrorScreen.Layout` and `ConfirmWarningScreen.Layout` both delegate body
layout to the same `Warning.Layout(ctx, th, dims, title, body)` widget behind
an identically-sized two-button nav row. So **not a live numeric false-PASS
today** — but the test does not exercise the modal shape it is nominally
checking for 2 of 3 rows, and a future change that makes `ConfirmWarningScreen`'s
chrome diverge from `ErrorScreen`'s (e.g. reserving space for the hold-progress
ring) would go undetected by this test. Minor (M-1 below).

**(d) Do the corpus tests recompute the expected value with the code under
test?** One instance found. In `TestDerivationRowsLockstep`, the final check —
`if d, ok := DeriveHardened(phrase, func(int, int) bool { return true }); !ok
|| d != x` — compares `DeriveHardened`'s stepwise output against `x`, which is
`PreimageHardened(phrase)`, **also code under test**, not a corpus constant.
Mitigated: `x` itself is independently checked against the corpus's
`r.HardenedX` two lines earlier in the same loop body, so a shared defect in
the underlying `seal.NewDeriver` call would already be caught there — but the
`DeriveHardened`-specific sub-assertion is, on its own, self-consistency only.
Minor (M-2 below).

Separately (surfaced while investigating (d)): `TestKindRowPreimageDigest`'s
own doc comment states its purpose is *"the digest of that preimage is what the
confirm modal must show for a --hex X"*, but its body is only
`if h := Digest(&x); h == x { t.Fatalf("Digest is the identity") }` — it rules
out exactly one wrong behaviour (the identity function) and checks nothing
else. Verified by mutating `Digest` to double-hash
(`sha256.Sum256(sha256.Sum256(x[:])[:])`): `TestKindRowPreimageDigest` **still
PASSES**. The corpus's own `kind[0]` object carries a `digest` field
(`9a2db2e2…af885`) that is exactly `sha256(kind[0].preimage_hex)` — verified
independently with `python3 hashlib` — but the plan's `Kind` struct in
`hashlock_test.go` never parses it (only `PreimageHex`/`MS1` fields are
declared) and the test never uses it. This is a genuine, fixable Critical
(C-1 below): reverted, the double-hash mutation IS caught by the sibling
`TestDerivationRowsLockstep` (whose `sha256 H` assertions fail for every
non-anchor row), so the SUITE is not fooled — but the specific test named for
this purpose is.

---

## 5. Corpus sufficiency (spec §2)

All five of spec §2's clauses have at least one corpus row:
non-empty (row 0, empty refused); printable-ASCII (rows 1-4: `café`, `0xFF`,
embedded TAB, `a`+`0x7F`, plus row 5's `0x20`/`0x7E` boundary ACCEPTED);
ms1-shaped (rows 9-13: lowercase/UPPERCASE/grouped-by-5/leading-trailing-spaces/
grouped-by-2 of the real `kind[0].ms1` plate); too-long (row 14, 101 chars
refused, PLUS the exactly-100-char row is present and ACCEPTED in the
`derivation` array); 64-hex (rows 6-7, both cases refused, PLUS `beef`
accepted as short non-64 hex in row 8).

The brief's own named examples: a phrase with a **TAB inside** — present (row
3, `a<TAB>b`, embedded not boundary). A **0x7F** — present (row 4, `617f` hex
= `a` + DEL). A **100-character phrase that is also 64-hex-shaped** — correctly
**impossible** (100 ≠ 64, contradictory lengths) and correctly absent.

One more impossible/moot case worth naming explicitly: a string that is
**simultaneously ms1-shaped and valid 64-hex** cannot exist, because
`IsMS1Shaped` requires the prefix `ms1` and `m`/`s` are not valid hex digits —
so the rule-3-before-rule-5 ordering (shape checked before the 64-hex check)
can never be exercised by one ambiguous string, and its absence from the
corpus is correct, not a gap.

**One genuine gap this review found (already noted at mutation (ii)):**
`hashlock.minMS1Len = 48` has no corpus or plan test row anywhere near its own
boundary — no phrase of stripped-length 47 or 48 appears in either the corpus
or `composer_hashlock_test.go`'s screen-level refusal tests. This is exactly
why mutating it to 47 survives the entire `hashlock` package suite.

The plan's own screen-level tests (`TestHashlockPhraseRefusalsOnScreen`) add UI
equivalents of three corpus categories using either the identical string
(`plate` is byte-identical to the corpus's `kind[0].ms1`, verified) or a
representative one (`strings.Repeat("k", 101)` for too-long, the real hardened
digest hex for the 64-hex case) — appropriate for a screen-level test, not a
duplicate vendoring.

---

## 6. Severity and closing counts

**Critical (1):**
- **C-1.** `TestKindRowPreimageDigest` is a false PASS relative to its own
  stated purpose: it cannot fail on a `Digest` defect other than the identity
  function, though the corpus already carries the ground-truth value
  (`kind[0].digest`) that would make it a real assertion. Verified by mutation
  (double-hash survives this test, caught only by a sibling test).

**Important (6):**
- **I-1.** Task 4 Step 2's RED claim ("does not compile") does not reproduce:
  with Task 3's own prescribed stub in place, `composer_hashlock_test.go`
  compiles cleanly and instead fails at runtime (10 failures, all "no
  *PassphraseKeyboard was registered").
- **I-2.** `TestWhichHashRowsAreLabelKeyed`'s own mutation-credit comment is
  wrong for that specific test (it never drives `composerHashEdit`'s switch);
  the regression it names is only caught once Task 4's harness tests exist.
- **I-3.** `DeriveHardened`'s own progress-abandon contract (`ok=false` on
  Back) is untested anywhere in the plan — survives the whole `hashlock`
  package suite and the full narrow `gui` selection, because the GUI wrapper
  tracks its own redundant abandon flag.
- **I-4.** `hashByPhrase`'s real assignment (`hashlockPhraseRoute`) is
  unexercised by any test — the field's only test reference constructs a
  `composerState` manually. Survives the whole narrow Task 1-4 selection.
- **I-5.** `hashlock.minMS1Len`'s own boundary (47 vs 48) has zero corpus/test
  coverage; mutation survives the whole `hashlock` package suite.
- **I-6.** `IsMS1Shaped`'s `strings.TrimSpace` call is untested-as-such and
  provably redundant with the character-stripping loop that follows it;
  mutation survives the whole `hashlock` package suite.

**Minor (2):**
- **M-1.** Two of five new `modal_fits_test.go` rows measure a
  `ConfirmWarningScreen` body through `errorScreenBody` rather than the file's
  own `confirmWarningBody`; numerically inert today (identical headroom,
  because both screens share the same `Warning.Layout` primitive) but not
  exercising the claimed renderer.
- **M-2.** `TestDerivationRowsLockstep`'s `DeriveHardened` sub-assertion
  compares against `PreimageHardened`'s own output rather than a corpus
  constant directly; mitigated by a sibling assertion in the same loop body.

**Nit (1):**
- **N-1.** The plan's mutation-table wording "every test … hangs" is
  imprecise — the affected test fails cleanly via the harness's own
  pump-timeout, not an actual infinite hang, though the underlying mechanism
  (stale global pointer state) is exactly as described.

**All eleven of the plan's own declared mutations (Task 1 Step 5's six, Task 2
Step 4's one, Task 4 Step 4's five) reproduce their claimed — or the gate's
already-corrected — outcome exactly, quoted above, with every mutation
reverted and GREEN re-confirmed.**

**Independent confirmation of Task 4 Step 5's claim:** ran
`scripts/gui-shard-test.sh ./gui/ 24` against this review's own independently
wired tree: `1213 top-level tests`, `partition verified exhaustive: 1213 ==
1213`, all 24 shards `ok`, 23 s wall — matching the plan's claimed shape
(1213 tests, exhaustive partition) on a tree this review built from the plan's
text, not copied from the gate.

Final state of `/scratch/code/shibboleth/.tmp/h2-tests`: every mutation
reverted; diffed byte-identical against this review's own post-wiring backups
at the time of writing this report; nothing committed.
