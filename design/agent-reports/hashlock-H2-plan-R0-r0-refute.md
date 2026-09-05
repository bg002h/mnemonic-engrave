# R0 round 0 — refute pass over the five landed lens reports

**Artifact under review:** `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md` at engrave master `02abee6`
(verified identical to the working tree and to `git show 02abee6:...`).
**Fork baseline read:** `/scratch/code/shibboleth/seedhammer` main, confirmed at `c4a64fc`.
**Gated tree read (read-only, never written):** `/scratch/code/shibboleth/.tmp/h2-gate`.
**Own scratch used for execution:** `/scratch/code/shibboleth/.tmp/h2-refute/tree` — a `cp -a` of
`.tmp/h2-gate` (never `.tmp/h2-gate` or `.tmp/h2-tests` themselves), used to run one probe test
(§2.8). Read-only elsewhere; nothing committed; no sub-agents; no `.jsonl` read.
**Reports processed:** `hashlock-H2-plan-R0-r0-fidelity.md` (0C/6I), `-journey.md` (0C/5I),
`-adversarial.md` (2C/4I), `-coverage.md` (0C/1I), and `-tests.md` (1C/6I), which landed mid-session
(checked absent at the start of this review, then present after the coordinator's message; read in
full and folded in below before this report was finalized).

**Method:** every Critical/Important finding below was checked against the plan text, the fork at
`c4a64fc`, and (where a finding needed it) direct execution in the refute scratch. Line citations were
re-verified by `sed`/`grep` against the actual files, not taken from any report's quotation.

---

## 1. Finding-by-finding table

25 C/I findings across the five reports (2C+16I from the original four, +1C+6I from `-tests.md`)
resolve to **16 distinct underlying defects** (9 report-entries are duplicates or tight corroborations
of another entry).

| report | id | title (short) | verdict | duplicate-of |
| --- | --- | --- | --- | --- |
| adversarial | C-1 | hardened derivation stalls 3 min, screensaver parks it, `ctx.Frame` never returns | **CONFIRMED** | unique |
| adversarial | C-2 | decoder off-by-one ships green; kind-row lockstep unimplemented | **CONFIRMED** | overlaps fidelity I-6 and tests C-1 (severity C/I split; tests C-1 independently executes the digest-test half) |
| adversarial | I-1 | reconciliation line moved behind `composerEveryPathHashed`, unreachable for a mixed policy | **CONFIRMED** | = fidelity I-2 = journey I-3 |
| adversarial | I-2 | `st.hashByPhrase` set once, never cleared | **CONFIRMED** | = fidelity M-2 = journey M-1 (both Minor, outside their own C/I scope); compounded by tests I-4 (a distinct claim — assignment itself untested) |
| adversarial | I-3 | `Type 64 hex`'s Back behaviour changed, untested, plan's claim of coverage is false | **CONFIRMED** | = fidelity I-4 = journey I-4 |
| adversarial | I-4 | `Deriving` zero-state lead can never render | **CONFIRMED** | = fidelity M-1 = journey M-2 (both Minor, outside their own C/I scope) |
| fidelity | I-1 | C-4 regression test never drives `composerHashEdit`'s dispatch; plan's mutation claim is false | **CONFIRMED** | corroborated + refined by tests I-2 |
| fidelity | I-2 | reconciliation line lost for the common (mixed) policy shape | **CONFIRMED** | = adversarial I-1 = journey I-3 |
| fidelity | I-3 | 3 of 5 fit-gate rows measured on the wrong renderer; drop-order decision rests on wrong numbers | **PARTIAL** | = journey I-2; label-mismatch half corroborated by tests's own finding (c); consequence half refuted |
| fidelity | I-4 | `Type 64 hex` Back untested, false claim | **CONFIRMED** | = adversarial I-3 = journey I-4 |
| fidelity | I-5 | relation line's no-match branch never driven; `match:=0` mutation passes | **CONFIRMED** | unique |
| fidelity | I-6 | §7.4's two strongest cases missing; `TestKindRowPreimageDigest` near-vacuous | **CONFIRMED** | overlaps adversarial C-2 and tests C-1 |
| journey | I-1 | two paths, two different phrases, device never cross-checks or warns | **CONFIRMED** | unique |
| journey | I-2 | 3 of 5 fit-gate rows measured on the wrong screen | **PARTIAL** | = fidelity I-3 |
| journey | I-3 | reconciliation line unreachable for any policy with one un-hashed path | **CONFIRMED** | = adversarial I-1 = fidelity I-2 |
| journey | I-4 | `Type 64 hex` Back changes behaviour, no test, plan says one does | **CONFIRMED** | = adversarial I-3 = fidelity I-4 |
| journey | I-5 | §8i rule modal fires in front of the route it was written to warn against | **CONFIRMED** | unique |
| coverage | I-1 | Task 3 Step 5's "one-line stub" instruction under-specifies what must compile | **CONFIRMED** | unique |
| tests | C-1 | `TestKindRowPreimageDigest` is a false PASS relative to its own stated purpose (executed: double-hash mutation survives it) | **CONFIRMED** | overlaps adversarial C-2 / fidelity I-6 (the digest-test half specifically; adversarial C-2 additionally covers the codex32 decoder off-by-one, which tests did not separately mutate) |
| tests | I-1 | Task 4 Step 2's "does not compile" RED claim does not reproduce; compiles and fails at runtime instead | **CONFIRMED** | unique |
| tests | I-2 | `TestWhichHashRowsAreLabelKeyed`'s mutation-credit comment is wrong for that specific test; the class IS caught elsewhere once Task 4 is fully wired | **CONFIRMED (with a corrective nuance)** | duplicate-of / refines fidelity I-1 |
| tests | I-3 | `DeriveHardened`'s own progress-abandon contract (`ok=false` on Back) untested anywhere; GUI wrapper's redundant flag masks it | **CONFIRMED** | unique |
| tests | I-4 | `hashByPhrase`'s real assignment (inside `hashlockPhraseRoute`) is never exercised by any test — only a manual struct literal touches the field | **CONFIRMED** | related to, but a distinct claim from, adversarial I-2 / fidelity M-2 / journey M-1 (those are about the field never being CLEARED; this is about the assignment never being VERIFIED to occur) |
| tests | I-5 | `hashlock.minMS1Len = 48`'s own boundary (47 vs 48) has zero corpus/test coverage | **CONFIRMED** | unique |
| tests | I-6 | `IsMS1Shaped`'s `strings.TrimSpace` call is untested-as-such and provably redundant with the stripping loop that follows it | **CONFIRMED** | unique (topically adjacent to journey N-1 / adversarial M-2, both Minor, about the SEPARATOR SET rather than this redundancy) |

---

## 2. Evidence

### 2.1 Adversarial C-1 — derivation stall (CONFIRMED)

Read `hashlockDeriveFlow`'s progress callback in the plan (Task 4 Step 3) and in the gated tree
(`gui/composer_hashlock.go:172-206`): it calls `ctx.Frame(op.Layer(pctOp, leadOp, nav, titleOp,
op.Color(&ctx.B, th.Background)))` and **nothing else** — no `ctx.WakeupAt`, no `ctx.KeepAwake()`.
Confirmed against the fork's own equivalent, `unlockDerive` (`gui/unlock_kdf.go:295-336`, `c4a64fc`),
which calls exactly `ctx.KeepAwake()` then `ctx.WakeupAt(time.Now())` immediately before its own
`ctx.Frame`, with a comment naming F-93 and explaining why the order is load-bearing. `idleTimeout =
3 * time.Minute` (`gui/gui.go:3584`). Traced `gui/run_flow.go:280-410` directly: the branch
`if a.idle.active { ...; ctx.WakeupAt(now.Add(minFrameTime)); continue }` (screensaver branch,
`:401-406`) `continue`s the frame-delivery loop rather than `break`ing, so it never returns control to
the caller of `ctx.Frame` until real input arrives. Confirmed `runUITouch`
(`gui/start_screen_touch_test.go:29`, the harness the plan's Task 4 Step 2 uses) sets
`ctx.FrameCallback` directly and never calls `runWithFlow`'s idle/screensaver loop at all — so the
plan's own gate is structurally blind to this class. Confirmed
`TestRunKeepAwakeDuringDerivationDoesNotParkUnderTheScreensaver` (`gui/run_flow_test.go:671`) drives
`unlockDerive` **by name**, not `hashlockDeriveFlow`, so the one existing regression test for this
exact class does not cover the new screen. Confirmed `seal.NewDeriver` sets `d.done = 1` and
`DeriveHardened`'s `for !d.Step(500) { if !progress(...) {...} }` calls `progress` only after the
first `Step(500)`, matching the stall's exact timing (first frame at `done=501`, then blocked). All
citations check out exactly as the report states. **CONFIRMED**, no scope/severity disagreement.

### 2.2 Adversarial C-2 / Fidelity I-6 / Tests C-1 — decoder off-by-one + vacuous digest test (CONFIRMED, three-way, severity split)

Read the plan's Task 2 Step 1 test verbatim: the only assertion on the decoded preimage is
`if x[0] == 0 && x[31] == 0 { t.Fatalf(...) }`. Verified the corpus (`hashlock-v0.8.json`, kind row
0) directly with Python: `preimage_hex` = `"ab"*32` (64 hex chars, 32 bytes), `digest` = SHA-256 of
that preimage (independently recomputed and matched), `ms1` = 75 chars. Under the stated mutation
(`copy(preimage[:], d[:32])` instead of `d[1:]`), `x[0] = 0x03` (the kind byte) and `x[31] = 0xab` —
neither zero, so the assertion's `&&` is false and the mutated decoder reports PASS. Confirmed the
plan's `corpus` struct (plan lines 182-185) declares only `PreimageHex` and `MS1` for `Kind` — no
`Digest` field, no `entr32_pair_ms1` — and confirmed by `grep -rn "entr32_pair_ms1"` over the gated
tree that the string appears **only** inside the vendored JSON, never in any Go test. Read
`TestKindRowPreimageDigest` (plan Task 1 Step 2) verbatim: its entire body is
`if h := Digest(&x); h == x { t.Fatalf("Digest is the identity") }` — an identity-function check, not
a comparison against the corpus's real `digest` constant.

**Tests C-1 independently executed this exact claim** (in `/scratch/code/shibboleth/.tmp/h2-tests`,
not this review's scratch): mutated `Digest` to double-hash
(`sha256.Sum256(sha256.Sum256(x[:])[:])`) and confirmed `TestKindRowPreimageDigest` **still PASSES**,
while the sibling `TestDerivationRowsLockstep` catches it (its `sha256 H` assertions fail for every
non-anchor row) — so the SUITE as a whole is not fooled, but the specific test named for this purpose
is. This is independent, executed corroboration of the static claim I verified above by inspection.
All three claims **CONFIRMED** exactly.

**Severity note (not adjudicated, flagged for the fold):** adversarial and **tests** both rate the
digest-test half Critical; fidelity rates the overlapping content Important — **2 votes to 1** for
Critical on this sub-claim, now with execution evidence behind the Critical side. Project policy
(`CLAUDE.md`, severity section) explicitly keeps "a test that reports a false PASS" in the blocking
class regardless of immediate operator-visible impact, which supports the Critical framing. Fidelity
I-6 additionally names one clause the other two do not: the acceptance record's plate
(`ms10hashsq0p7jaf…`, `ms-hashlock-H1-acceptance.md`, cited in the actual spec at §7.4) → corpus
anchor's `hardened_x`, tying the decoder to a host-produced artifact; confirmed that file and citation
exist. Adversarial C-2 additionally covers the codex32 **decoder** off-by-one (`d[1:]` vs `d[:32]`),
which tests did not separately construct as one of its own mutations (its Task 2 mutation table only
reproduces the plan's own declared `!f.Unshared` mutation). So: one root pattern (an assertion that
cannot fail on the property it names), three reports, overlapping but not 100% identical scope, and a
real severity disagreement to resolve at fold time.

### 2.3 Adversarial I-1 = Fidelity I-2 = Journey I-3 — reconciliation line unreachable (CONFIRMED, triplicate, consistent severity)

Read `composerEveryPathHashed` in the fork (`gui/composer_state.go:239-248`): returns false the
moment ANY path has `Hash == nil`. Read the gated tree's `gui/composer_shape.go:443`: still
`if composerEveryPathHashed(st.list) { showError(ctx, th, "Spend paths",
composerCopyHashEveryPathFor(st)) }` — same guard, unmodified by the plan. The reconciliation
sentence ("Before you fund this wallet, run ms hashlock with this phrase and method on the host and
check the digest matches.") lives, per the plan's own Task 4 Step 1 comment, **only** inside
`composerCopyHashEveryPathPhrase`, reached only through that guarded call. For the ordinary
mixed-shape wallet (one keyed path, one hashlocked-by-phrase path) the guard is false and the
sentence is drawn nowhere. **CONFIRMED**, exactly as all three reports independently trace it, with
matching (Important) severity across all three — no conflict.

### 2.4 Adversarial I-2 = Fidelity M-2 = Journey M-1 (+ Tests I-4) — `hashByPhrase` never cleared, and never verified as set (CONFIRMED, severity split, one compounding claim)

`grep -rn "hashByPhrase" gui/*.go` (excluding `_test.go`) in the gated tree returns exactly one write
site (`gui/composer_hashlock.go:80`, `st.hashByPhrase = true`, inside the phrase route's confirm
arm) and one read site (`gui/composer_copy.go:438`). No clear exists anywhere in production code —
not in the `noneRow` arm, not in the hex arm, not in `composerAddPath`'s two rollback sites, not in
`composerStartStep`'s preset-replace path. **CONFIRMED** as stated in all three reports.

**Severity note:** adversarial rates this Important; fidelity and journey both rate it Minor, and
fidelity's stated reason — "the dangerous direction (a phrase-set hash with `hashByPhrase` false) is
not reachable, so this is a copy-accuracy defect, not a safety one" — has support I checked directly:
`composerCopyHashEveryPathPhrase`'s actual text is *"Back up the phrase and its method, **or** the
preimage plate, separately"* (gated tree, `gui/composer_copy.go`) — it names both possible backup
artifacts, so the over-sticky flag causes an operator to be told to back up something extra, not to
be denied the correct instruction. That weakens (without refuting) adversarial's Important framing.

**Tests I-4 is a distinct claim about the same field, verified separately:** grepped every `_test.go`
in `gui/` for `hashByPhrase` in the gated tree — it appears in exactly one place besides production
code, `composer_copy_test.go`'s `composerCopyTable` row, which builds `&composerState{hashByPhrase:
true}` as a manual struct literal (confirmed at plan line 1003), never by driving the real
`hashlockPhraseRoute` assignment. None of the seven `TestHashlock*` tests asserts on
`st.hashByPhrase` after completing the flow. So not only is the field never cleared (the
already-known defect), its assignment to `true` is *also* never exercised by any test that drives
the real route — a second, independent gap in the same field's test coverage. **CONFIRMED**, and
noted as its own line in the fold rather than silently absorbed into the "never cleared" wording,
since a test for provenance-tracking (the fix these reports propose) would need to prove BOTH the set
and the clear, not just the clear.

### 2.5 Adversarial I-3 = Fidelity I-4 = Journey I-4 — `Type 64 hex` Back untested (CONFIRMED, triplicate, consistent severity)

Read the fork's `gui/composer_shape.go:266-269` (`c4a64fc`): `if !composerHashEdit(ctx, th, st, idx)
{ st.list.Paths = st.list.Paths[:idx]; return }` — confirms today's `false` from `composerHashEdit`
deletes the path at creation. Confirmed the plan's Task 3 Step 2 hex-row arm `continue`s instead
(`case sel == rows.hexRow: ... if !ok { continue }`) — a real behaviour change. Grepped the gated
tree's `gui/composer_hashlock_test.go` for `hex`/`Type 64 hex`/`hexRow`: the only hits are the
`encoding/hex` import, the `hashlockHashHex` helper, and one *phrase-refusal* row (`{"64 hex",
hashlockAnchorHardH, "Use the Type 64 hex row"}` — this types 64 hex characters **into the phrase
screen**, never selects the hex row). No test anywhere selects `rows.hexRow` or presses Back at the
hex pad. Confirmed `gui/composer_gates_test.go:906-951` (the fork's only tests touching
`composerHexEntry`) call it **directly**, never through `composerHashEdit`. The plan's own text
("Task 4's harness tests do [cover it]") is therefore false. **CONFIRMED** exactly, all three
reports, matching (Important) severity.

### 2.6 Adversarial I-4 = Fidelity M-1 = Journey M-2 — `Deriving` zero-state lead unreachable (CONFIRMED, severity split)

Read `seal.NewDeriver` (`seal/pbkdf2.go:85-102`, `c4a64fc`): sets `d.done = 1` after computing `U_1`.
Read the plan's `DeriveHardened` (Task 1 Step 4): `for !d.Step(500) { if !progress(d.Done(),
d.Total()) { return x, false } }` — `progress` is called only *after* a `Step(500)` returns false, so
the first call arrives with `done = 501`. Read the plan's callback guard: `if elapsed :=
time.Since(start); done > 0 && elapsed > 0 { lead = fmt.Sprintf("About %d seconds left.", ...) }` —
both conditions are true on every reachable call, so `composerCopyHashlockDerivingLead()` ("Deriving.
This takes about 10 seconds.") is assigned and then immediately overwritten on every frame; it is
never drawn. **CONFIRMED** by direct arithmetic, matching all three reports.

**Severity note:** adversarial rates Important, fidelity/journey rate Minor. No factual dispute —
purely a severity disagreement, left to the fold.

### 2.7 Fidelity I-1 (+ Tests I-2) — C-4 regression test doesn't drive the dispatch (CONFIRMED, corroborated with a corrective nuance)

Read `TestWhichHashRowsAreLabelKeyed` (plan Task 3 Step 1) in full: it calls only
`composerHashRows(s)` and asserts on `.labels`, `.phraseRow`, `.hexRow`, `.noneRow`, `.lead` — it
never calls `composerHashEdit`, so it cannot observe the dispatch switch at all. Read all seven
`TestHashlock*` tests in Task 4: every one uses `composerSessionWith(nil, nil)` (0 payload digests),
where `rows.phraseRow == 0`. Under fidelity's stated mutation (index arithmetic:
`case sel == len(rows.digests): // phrase; default: clear`), with 0 payload digests the mutated
`sel == len(rows.digests)` (== 0) is *coincidentally* still the phrase row, so every existing test
still passes; the mutation is only distinguishable by selecting the hex row or the none row through
`composerHashEdit`, which no test in the plan ever does. **CONFIRMED**: the plan's own comment
claiming this mutation is caught ("MUTATION: restore the index arithmetic ... this fails") is false
as written.

**Tests I-2 independently ran this exact investigation and adds a real nuance.** It confirmed, by
execution, that `TestWhichHashRowsAreLabelKeyed` itself never drives the switch (matching fidelity
exactly — "with only Task 3 wired," the mutated test still PASSES). But it also tried a **different,
cruder** reversion — literally the pre-H2 shipped 2-arm switch reapplied verbatim (no phrase-row
concept at all) rather than fidelity's more surgical 3-arm reconstruction that preserves the phrase
row's correct index and only collapses hex+none. Under that cruder mutation, **with Task 4 fully
wired**, 10 of the plan's own `TestHashlock*` tests fail (`never reached "Hashlock phrase"`, because
tapping the phrase row now incorrectly opens hex entry). So: the plan's SUITE, taken as a whole, is
not blind to every possible "revert to index arithmetic" mutation — it catches the crude one. What it
does not catch — confirmed independently by both fidelity's static counterexample and my own
citation-check in §2.7 above, and left unaddressed by tests' own execution, which tried a different
mutation — is the more surgical variant that keeps the phrase row's index correct and only merges
hex+none into one clearing arm, which is also the exact outcome ("`Type 64 hex` lands in the clearing
arm") the plan's own mutation-table comment predicts. **Net: fidelity I-1's core claim stands
(the regression the comment specifically describes is genuinely uncaught by anything in the plan),
refined rather than contradicted by tests I-2's finding that a blunter reversion is caught elsewhere.**
Fold should keep both nuances: the plan's own named test doesn't do what its comment claims, AND a
more surgical reversion of the same general shape remains fully uncaught by any test in the plan.

### 2.8 Fidelity I-3 / Journey I-2 (+ Tests finding (c)) — fit-gate renderer mismatch (PARTIAL — label mismatch TRUE, capacity-difference claim REFUTED by direct measurement)

**What is true.** Read `gui/modal_fits_test.go:359` (`TestModalsThisBlockTouchesAreDrawnInFull`)'s
single call site: `assertModalBodyFits(t, tc.what, errorScreenBody, tc.body)` for every row,
including the plan's three `ConfirmWarningScreen` bodies (both method warnings, the §4.5 confirm
modal), which in production are drawn via `composerConfirmScreen` → `ConfirmWarningScreen`, not
`showError`. This label/rendering-shape mismatch in the test table is real, and the two method-warning
rows are additionally measured as raw copy (`composerCopyHashlockHardenedWarning()`) rather than
wrapped through `composerConfirmBody(...)` as production actually draws them — measured directly (see
below) at 20 characters short (169 raw vs 189 wrapped for the hardened warning; 206 vs 226 for
SHA-256). **Tests's own finding (c) independently confirms this same mismatch and reaches the same
"numerically inert today" conclusion** (its own headroom numbers, 397/360/186, match mine below
exactly), rating it Minor (M-1 in that report) for the same reason my measurement gives: the two
renderers share `Warning.Layout`.

**What is false — checked by direct execution, not inference.** I built a probe test
(`/scratch/code/shibboleth/.tmp/h2-refute/tree/gui/zzz_probe_test.go`, my own scratch, a `cp -a` of
`.tmp/h2-gate`) that measures `modalHeadroom` for the SAME body under both `errorScreenBody` and
`confirmWarningBody`, for all three of the plan's rows plus fidelity's own two cited pre-existing
examples (`composerCopyKeylessPath()`, `composerCopyUnsortedKeys()`). Ran it with the pinned toolchain
Go (`/scratch/code/shibboleth/.toolchain/go/bin/go test -v -run TestZZZProbeRendererCapacity ./gui/`):

```
hardened warning RAW          errorScreenBody: drawn=169 headroom=397 | confirmWarningBody: drawn=169 headroom=397 | delta=0
hardened warning WRAPPED      errorScreenBody: drawn=189 headroom=302 | confirmWarningBody: drawn=189 headroom=302 | delta=0
sha256 warning RAW            errorScreenBody: drawn=206 headroom=360 | confirmWarningBody: drawn=206 headroom=360 | delta=0
sha256 warning WRAPPED        errorScreenBody: drawn=226 headroom=302 | confirmWarningBody: drawn=226 headroom=302 | delta=0
confirm body WRAPPED          errorScreenBody: drawn=290 headroom=186 | confirmWarningBody: drawn=290 headroom=186 | delta=0
§8a key-less confirm (fidelity's own example) errorScreenBody: drawn=166 headroom=339 | confirmWarningBody: drawn=166 headroom=339 | delta=0
§8b unsorted keys confirm (fidelity's own example) errorScreenBody: drawn=173 headroom=339 | confirmWarningBody: drawn=173 headroom=339 | delta=0
```

**Zero delta on every one of 7 bodies**, including fidelity's own cited examples. This is explained
structurally, not just empirically: `warningBodyClip(dims image.Point) image.Rectangle`
(`gui/gui.go:595-600`) — the clip rectangle both `ErrorScreen.Layout` and `ConfirmWarningScreen.Layout`
pass to the shared `(*Warning).Layout` — depends **only on `dims`** (the display size), not on which
screen type calls it, not on the nav buttons, not on the icon. There is no code path by which the two
renderers could differ in capacity for a fixed display size. Fidelity's inferred "capacity"
(drawn+headroom, compared across *different* bodies of different lengths) was confounded by where
each body's own word-wrap lands, not by an actual renderer difference — it is not a valid way to
compare capacity across bodies, only within one fixed body via search (which is what `modalHeadroom`
already does, and which I ran directly for the SAME body both ways, and which tests's own finding (c)
also did for these same three bodies with identical numbers).

**Consequence:** the specific downstream claim in fidelity/journey — that "the drop order... rests on
a measurement of the wrong surface" (journey I-2) and the implied invitation to re-run the drop order
and possibly reverse it (journey's own suggestion: *"If the unshortened body now fits with 80
characters to spare, the reconciliation line goes back"*) — is **refuted**. Re-measuring the §4.5
confirm body via `confirmWarningBody` gives the identical 290 drawn / 186 headroom fidelity itself
already reproduced via `errorScreenBody`; the drop-order decision would not change. **Verdict: PARTIAL**
— the cosmetic/labeling defect (wrong renderer named in the test table, two rows measured unwrapped)
is real and worth fixing for correctness of the test's own claims, but the reasoning that this defect
caused the reconciliation line to be cut on faulty numbers, and journey's specific suggested remedy
contingent on that reasoning, do not hold. Tests's own M-1 (Minor) is the correctly-scoped version of
this same finding.

### 2.9 Fidelity I-5 — relation line no-match branch untested (CONFIRMED, unique)

Read `TestHashlockConfirmRelationLine` (plan Task 4, the only test with any payload digests):
`composerSessionWith([]string{"hash:" + hashlockAnchorSHA_H, "hash:" + strings.Repeat("ab", 32)},
nil)` — record 0 IS the anchor's own SHA-256 digest, so a true match exists at index 0. Under
fidelity's stated mutation (`match := 0` instead of `match := -1`), the loop still finds a real match
at index 0 in this exact test (since it IS the anchor's digest), so `match` ends at 0 either way and
the test cannot distinguish correct code from the mutant. Confirmed every other `TestHashlock*` test
uses `composerSessionWith(nil, nil)` (0 payload records), so the relation line is never reached by
them at all. **CONFIRMED** exactly as stated. (Distinct from tests's own finding (v), which mutates
the relation *comparison set*, not the initial match value — same test, different mutation, both
land; no conflict.)

### 2.10 Journey I-1 — two paths, two phrases, no cross-check (CONFIRMED, unique)

Read the plan's relation-line code in `hashlockPhraseRoute` (Task 4 Step 3): the comparison set is
`payload` (`= rows.digests = composerPayloadDigests(ctx.sysw)`), i.e. only the payload's `hash:`
records — never `st.list.Paths`, though `st` is in scope on that exact line. Read `md.ValidatePathList`
(`md/compose.go:299-334`, `c4a64fc`) in full: it checks thresholds, slot counts, lock ranges, the
key-less/`tr` rule, and the legacy shape — no clause requires or forbids two paths' `Hash` values
being equal. So two different phrase-set hashes on two paths of the same policy are never compared or
flagged, and the copy's "One phrase per policy" is advisory only. **CONFIRMED** exactly as stated.

### 2.11 Journey I-5 — §8i modal confusing ahead of the phrase route (CONFIRMED, unique)

Read `composerCopyHashRule()` (`gui/composer_copy.go:175-179`, `c4a64fc`) verbatim: *"The hash must
be SHA-256 of a 32-byte value. A passphrase must be hashed to 32 bytes first, then hashed again. A
hash of the passphrase itself can never be spent."* Read the fork's header comment this stage rewrites
(`gui/composer_hash.go:27-28`): *"THE COMPOSER NEVER DERIVES, STORES OR ENGRAVES A PREIMAGE this
cycle."* Confirmed the plan's `taking` predicate (`sel < len(rows.digests) || sel == rows.phraseRow ||
sel == rows.hexRow`) fires this modal on the phrase row, ahead of a route that (as of this plan) DOES
derive a preimage via SHA-256/hardened-then-hash. Neither the rule modal nor the SHA-256 method
warning states that the phrase route performs the 32-byte hashing itself. The underlying facts are all
verified true; the "harm" (an operator distrusting the digest) is a judgment call, not something
mechanically confirmable, but nothing in the finding's factual claims is false. **CONFIRMED** (facts),
unique.

### 2.12 Coverage I-1 — Task 3 Step 5 stub under-specified (CONFIRMED, unique)

Read Task 3's whole text (`awk` extract): `hashlockAssigned` appears exactly once, inside the
`composerHashEdit` switch shown in Step 2 — a case value that requires `hashlockOutcome` and both its
constants to already exist for the package to compile. Step 5's text says only: *"add a one-line stub
in `gui/composer_hashlock.go` returning `hashlockBackToWhichHash`"* — no code block anywhere in Task 3
declares the type or either constant. **CONFIRMED**: an implementer following Step 5 literally hits an
undefined-identifier compile error the step's text does not anticipate or explain (self-correcting via
the RED step, but exactly the "step that describes without showing the code" class the coverage
report's own placeholder scan was built to catch).

### 2.13 Tests I-1 — Task 4 Step 2's RED claim does not reproduce (CONFIRMED, unique)

Read the plan's exact RED instruction at Task 4 Step 2's end (plan line 1674): *"Run: `go test
-count=1 -run TestHashlock ./gui/` — Expected: does not compile."* Confirmed this checkpoint sits
right before Step 3 replaces Task 3's stub — so at this point `gui/composer_hashlock.go` still holds
only the unconditional `return hashlockBackToWhichHash` stub, while all seven `TestHashlock*`
functions (created earlier in this same Step 2, plan lines 1372-1650-ish) already reference
`hashlockOutcome`/`hashlockAssigned`/`hashlockBackToWhichHash` — symbols the stub already defines.
Confirmed the exact string `"no *PassphraseKeyboard was registered for this harness"` exists at plan
line 1261, inside the harness helper that reads `hashlockKbdFor[h]` — the map populated only when
`hashlockPhraseFlow` actually registers a keyboard via `passphraseWidgetHook`, which the stub never
calls. So the package compiles cleanly (every symbol Step 2's new test file needs already resolves)
and fails at runtime with that exact message instead of failing to compile. **CONFIRMED**: the plan's
own predicted failure mode for this checkpoint is wrong, though the net effect (RED, as intended) is
unaffected.

### 2.14 Tests I-3 — `DeriveHardened`'s own abandon-path contract untested (CONFIRMED, unique)

Read the plan's only `hashlock`-package-level call to `DeriveHardened`, inside
`TestDerivationRowsLockstep` (plan line 247): `DeriveHardened(phrase, func(int, int) bool { return
true })` — an always-true progress function that never exercises `DeriveHardened`'s own
`if !progress(...) { return x, false }` early-return path (Task 1 Step 4, line 434). Read the GUI's
own call (`hashlockDeriveFlow`, Task 4 Step 3): `abandoned` is set directly by
`backBtn.Clicked(ctx)` inside the same closure that also returns `false` to `DeriveHardened`, and the
outer check is `if !ok || abandoned { return x, false }` — so even if `DeriveHardened` itself ignored
the callback's return value entirely (a broken contract at the package level), the GUI wrapper's own
independently-tracked `abandoned` flag would still correctly short-circuit the flow, masking the
underlying defect from every GUI-level test too. **CONFIRMED** by direct reading of both call sites;
consistent with the report's stated executed mutation (iv), which found 6/6 `hashlock` package tests
and the full narrow `gui` selection all still PASS under this exact mutation.

### 2.15 Tests I-5 — `minMS1Len` 47/48 boundary untested (CONFIRMED, unique)

Independently dumped the corpus's `refusals` array with Python and located every `ms1-shaped` row
(indices 9-13): all five are placeholder-substituted forms of the real `kind[0].ms1` plate (75 chars
stripped), the shortest being the plain lowercase/uppercase forms at 75 characters — none anywhere
near a 47/48-character stripped length. Confirmed no test in `composer_hashlock_test.go`'s
screen-level refusal tests uses a boundary-length input either (`TestHashlockPhraseRefusalsOnScreen`'s
`plate` and grouped variants are all derived from the same 75-char plate). **CONFIRMED**: the
boundary genuinely has zero coverage anywhere the plan touches.

### 2.16 Tests I-6 — `IsMS1Shaped`'s `TrimSpace` is redundant and untested-as-such (CONFIRMED, unique)

Read `IsMS1Shaped` (plan Task 1 Step 4, lines 486-505) verbatim: `t := strings.ToLower(
strings.TrimSpace(s))`, then a loop over every rune of `t` that skips `' '`, `'\t'`, `'\n'`, `'\r'`,
`'-'`, `','` **at every position**, not just the boundaries, before rebuilding `t`. Since the loop
already strips every character `TrimSpace` would have stripped (space and the ASCII control
whitespace forms it covers) at every position including the string's ends, `TrimSpace`'s effect is a
strict subset of the loop's — removing the `TrimSpace` call changes the function's behaviour for no
input. **CONFIRMED** by direct code reading, matching the report's own executed mutation (i), which
found zero test failures across the whole `hashlock` package with `TrimSpace` removed.

---

## 3. Deduplicated list — what the fold must address

**16 distinct defects**, CONFIRMED or PARTIAL, after removing 9 duplicate/corroborating report-entries:

1. **Hardened derivation stalls / screensaver freeze** (adversarial C-1). Remedy: add
   `ctx.KeepAwake(); ctx.WakeupAt(time.Now())` immediately before `ctx.Frame` in the progress
   callback, mirroring `unlockDerive`; add a `synctest`-based regression test on `newDeadlinePlatform()`
   analogous to `TestRunKeepAwakeDuringDerivationDoesNotParkUnderTheScreensaver`, driving
   `hashlockDeriveFlow` by name. **Remedy is sound** — it is the exact fix the fork already shipped
   once for the identical bug in `unlockDerive`.

2. **Decoder off-by-one / vacuous digest test** (adversarial C-2 = fidelity I-6 = tests C-1;
   severity split C×2 vs I×1 — see §2.2). Remedy: read the vendored corpus in
   `codex32/mspayload_test.go` rather than a literal; assert
   `DecodeMS1Preimage(New(kind[0].ms1)) == kind[0].preimage_hex` and
   `DecodeMS1Preimage(New(kind[0].entr32_pair_ms1)) == errMSBadPrefix`; add the acceptance-record
   plate → anchor `hardened_x` case (fidelity's extra clause); add a `Digest` field to the `Kind`
   struct and compare `Digest(&x)` against it instead of the identity check. **Remedy is sound.**
   Fold should settle the severity disagreement explicitly (2 of 3 reports, including one with
   execution evidence, call this Critical) rather than silently keeping the lower rating, given the
   project's "a test that reports a false PASS" is-blocking rule.

3. **Reconciliation line unreachable for a mixed policy** (adversarial I-1 = fidelity I-2 =
   journey I-3, triplicate, consistent Important). Remedy (converged across all three): keep the
   line reachable independent of `composerEveryPathHashed` — a separate `showError` right after HOLD
   in the phrase route, gated on `st.hashByPhrase`, added to both copy gates; add a test that builds a
   mixed-shape policy (one keyed path, one phrase-hashed path) and asserts the line is drawn.
   **Remedy is sound**, but see item 4: gating on `hashByPhrase` inherits that field's own defects
   (over-fires when stale-true; its assignment is itself untested), which are safe-direction failures
   (extra reminder, not a missing one), worth a one-line note in the fold rather than a blocker.

4. **`hashByPhrase` never cleared, and its assignment never verified** (adversarial I-2, Important;
   fidelity M-2 / journey M-1, Minor; plus tests I-4, a distinct companion claim — see §2.4). Remedy:
   clear the field wherever a hash leaves the phrase route (`noneRow`, hex arm, payload-row arm) and
   on `composerAddPath`'s two rollback sites and `composerStartStep`'s replace branch, or (the
   reviewers' preferred, more robust option) store per-path provenance instead of one
   composition-wide flag; **and** add a test that drives the real `hashlockPhraseRoute` assignment
   and asserts `st.hashByPhrase` becomes true (tests I-4's gap — today only a manual struct literal
   touches the field). **Remedy direction is sound**; the per-path-array variant needs the same
   splicing discipline `composerAddPath`/"Remove path" already applies to `Paths`, or it reintroduces
   C16 rather than avoiding it — flagged, not fatal.

5. **`Type 64 hex` Back untested, false coverage claim** (adversarial I-3 = fidelity I-4 = journey
   I-4, triplicate, consistent Important). Remedy: add the six-line harness test the plan claims
   exists — tap the hex row, Back at the pad, assert the path survives with `Hash == nil` and the
   frame is back at `Which hash?` — and delete the false sentence, or correct it. **Remedy is sound
   and specific**; all three reviewers converge on essentially the same test shape.

6. **`Deriving` zero-state lead unreachable** (adversarial I-4, Important; fidelity M-1 / journey
   M-2, Minor — see §2.6). Remedy: either hoist a zero-state frame before the first `Step`, or gate
   the estimate on `done <= 1` rather than `done > 0`, and pull the lead into a testable pure
   function on the `unlockKDFLead` model. **Remedy is sound**; independent of item 1's fix (fixing the
   stall does not fix this, and vice versa — both need addressing).

7. **C-4 regression protection is real but mis-attributed** (fidelity I-1, refined by tests I-2 —
   see §2.7). Two things must both be true after the fold: (a) `TestWhichHashRowsAreLabelKeyed`
   itself must not claim to catch a dispatch-switch mutation it structurally cannot see — reword or
   remove that claim; (b) a **behavioural** test must drive `composerHashEdit` (not just
   `composerHashRows`) once per row label with 1-2 payload digests, asserting the hex row reaches hex
   entry (not clear) and the none row clears — this is the specific, more surgical index-arithmetic
   reversion that tests I-2 confirmed remains uncaught by everything in the plan, even though a
   cruder reversion happens to be caught elsewhere. **Remedy is sound** once both halves are done;
   doing only (a) without (b) would leave the actual regression class unprotected.

8. **Fit-gate renderer mismatch** (fidelity I-3 = journey I-2, PARTIAL — see §2.8). **Fidelity's own
   remedy is sound as stated** (move the three confirm-screen rows to `confirmWarningBody` calls in
   `gui/composer_hashlock_test.go`, wrap the two warning bodies in `composerConfirmBody(...)` as
   production draws them, fix the cosmetic mislabeling) — worth doing for the table's own internal
   correctness, even though it will not change any number (confirmed by direct measurement, and
   independently by tests's own finding (c)). **Journey's remedy is UNSOUND as stated**: it invites
   re-running the §4.5 drop order "from step 0" with the implication that the reconciliation line
   "goes back" if the unshortened body "now fits with 80 characters to spare" — this is empirically
   false (§2.8: delta = 0 measured directly, for 7 different bodies), so that step should be dropped
   from whatever the fold does with this finding; re-measuring will reproduce the same 290/186
   numbers, and the reconciliation line stays cut on capacity grounds regardless of which renderer is
   nominally used. Item 3's fix (a separate `showError` outside the confirm modal) is the correct
   remedy for the actual loss, independent of this finding.

9. **Relation line's no-match branch untested** (fidelity I-5, unique, Important). Remedy:
   parameterize `TestHashlockConfirmRelationLine` over (a) 2 records, second matching → pins the
   1-based index against an off-by-one; (b) 2 records, neither matching → asserts the "no hash: record"
   text; (c) 0 records → asserts neither string appears. **Remedy is sound and closes the gap
   completely** (case (a) specifically catches what a single always-index-0-passing case cannot).

10. **Two paths, two phrases, no cross-check** (journey I-1, unique, Important). Remedy: widen the
    comparison at the same line to loop over `st.list.Paths` (excluding `idx`) before/alongside the
    payload loop, and say "this policy will need TWO phrases" when another path already carries a
    different phrase-set hash. **Remedy is sound** and cheap (one loop, one copy body, two gate rows);
    it does not need item 4's fix first since it operates on live `*p.Hash` values directly rather than
    the `hashByPhrase` flag.

11. **§8i modal confusing ahead of the phrase route** (journey I-5, unique, Important-as-UX).
    Remedy: one added clause, either in the §8i modal when `sel == rows.phraseRow`, or at the head of
    `composerCopyHashlockPhraseLead()` (the cheaper option, no new gate row). **Remedy is sound** and
    is documentation-only in the sense that no code path changes, only copy.

12. **Task 3 Step 5 stub under-specified** (coverage I-1, unique, Important). Remedy: show the
    stub's real content (the `hashlockOutcome` type plus both constants) inline in Step 5, or fold the
    type/constant declarations into Step 2's own code block instead of describing them in prose.
    **Remedy is sound**; coverage's own M-1 (Task 3's `Files:` header omitting
    `gui/composer_hashlock.go`) is a matched, trivially-fixed companion (Minor, not in this report's
    required scope, but free to fold alongside #12 since it is the same task section).

13. **Task 4 Step 2's RED claim does not reproduce** (tests I-1, unique, Important). Remedy: correct
    the plan's Expected line to describe the actual failure mode (10 runtime failures, "no
    *PassphraseKeyboard was registered") rather than "does not compile," so a future implementer
    isn't confused when the RED step doesn't produce a compile error. **Remedy is sound and cheap**
    (a documentation correction; the checkpoint still functions as a RED gate either way).

14. **`DeriveHardened`'s own abandon-path contract is untested** (tests I-3, unique, Important).
    Remedy: add a `hashlock`-package-level test that passes a `progress` func returning `false` after
    N calls and asserts `DeriveHardened` returns `ok=false` promptly (not after completing all
    iterations) — independent of the GUI wrapper's own redundant tracking. **Remedy is sound** and
    closes a real gap: today a broken `DeriveHardened` contract would ship silently since the GUI
    masks it.

15. **`minMS1Len` 47/48 boundary untested** (tests I-5, unique, Important). Remedy: add one corpus or
    plan-level row at the boundary (47-char stripped input, `ms1` prefix, refused as too-short;
    48-char, accepted as ms1-shaped if bech32-valid). **Remedy is sound and cheap.**

16. **`IsMS1Shaped`'s `TrimSpace` is dead weight, untested-as-such** (tests I-6, unique, Important as
    a coverage gap though the code itself is harmless). Remedy: either delete the redundant
    `TrimSpace` call (simplifies the function, changes nothing), or add a comment recording the
    redundancy so a future reader does not assume it does load-bearing work. **Remedy is sound**;
    this is the one item on this list where "delete the code" is itself a complete, safe fix rather
    than "add a test."

---

## 4. Closing counts

- **Findings processed:** 25 (3 Critical + 22 Important across five reports: adversarial 2C/4I,
  fidelity 0C/6I, journey 0C/5I, coverage 0C/1I, tests 1C/6I).
- **Distinct underlying defects:** 16.
- **CONFIRMED:** 15 of 16 (all except #8).
- **PARTIAL:** 1 of 16 (#8 — fit-gate renderer mismatch: labeling defect real and now corroborated by
  three independent measurements — fidelity's own numbers, tests's finding (c), and my own probe —
  but the capacity-difference claim and its consequence are refuted by direct measurement).
- **REFUTED (no defect):** 0 — every finding that named a real code location and mechanism, on
  inspection or execution, described something actually present in the plan/gated tree. The only
  claim refuted outright was the *magnitude/consequence* portion of finding #8, not the existence of
  any finding as a whole.
- **Duplicate/corroborating report-entries folded away:** 9 — adversarial C-2 ≈ fidelity I-6 ≈ tests
  C-1 (2 folded); adversarial I-1 = fidelity I-2 = journey I-3 (2 folded); adversarial I-3 = fidelity
  I-4 = journey I-4 (2 folded); fidelity I-3 = journey I-2 (1 folded); tests I-2 refining fidelity I-1
  (1 folded, with a corrective nuance kept); tests I-4 noted alongside adversarial I-2 as a distinct
  but tightly-coupled companion claim (1 folded, distinct wording kept). (adversarial I-2/I-4's
  duplicate mentions in fidelity M-2/M-1 and journey M-1/M-2 are Minor in those reports and were
  outside their own C/I scope, so they are noted for cross-reference but not counted toward this
  25/16 tally.)
- **Severity disagreements the fold must explicitly resolve, not silently drop to the lower rating:**
  three — (a) the decoder/digest test gap (#2, Critical×2 vs Important×1 — strengthened since the
  last draft of this report by tests C-1's independent execution), (b) `hashByPhrase` never cleared
  (#4, Important vs Minor twice), (c) the `Deriving` zero-state lead (#6, Important vs Minor twice).
- **Unsound suggestion flagged:** journey I-2's specific remedy step ("re-run the drop order from
  step 0; the reconciliation line goes back if it now fits") — refuted by direct measurement, see
  item 8 above. No other reviewer suggestion was found to introduce a defect of its own on inspection.
- **`-tests.md` status:** landed mid-session (coordinator notified); read in full and folded in above
  (§2.13-§2.16, plus corroborations at §2.2, §2.7, §2.8, §2.4).

**Not GREEN** — of the 16 distinct defects: 2 are named Critical by at least one report (item 1,
unique; item 2, named Critical by two reports and Important by a third, the Critical side now
carrying independent execution evidence from a fourth) and the remaining 14 are Important-severity
(after dedup), of which 1 (item 8) is only PARTIALLY confirmed. None were refuted outright — only
one finding's magnitude/consequence claim (item 8) did not hold up under direct measurement.
