# R0 round 0 — FOLD report for `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md`

**Artifact folded:** `design/IMPLEMENTATION_PLAN_hashlock_H2_device.md` (engrave master
`02abee6`, working tree; plan text verified identical to `02abee6` before the first edit).
**Contract:** the deduplicated CONFIRMED + PARTIAL list in
`design/agent-reports/hashlock-H2-plan-R0-r0-refute.md` §3 — **16 distinct defects** (15
CONFIRMED, 1 PARTIAL), plus its three flagged severity disputes.
**Tree edited identically:** `/scratch/code/shibboleth/.tmp/h2-gate` (fork `c4a64fc` + the
plan). Go: `/scratch/code/shibboleth/.toolchain/go/bin/go`, 1.26.7.
**Not touched:** `design/SPEC_hashlock_H2_device.md`, any report, the fork checkout
(`git -C /scratch/code/shibboleth/seedhammer status --porcelain` = empty at the end).
**Committed:** nothing. **Sub-agents:** none. **`.jsonl` read:** none.
**Engrave working tree at the end:** `M design/IMPLEMENTATION_PLAN_hashlock_H2_device.md`
and nothing else (+1340 / −180).

---

## 1. Per confirmed finding — the plan change, and the RED/GREEN evidence

Every mutation below was applied to the gated tree, run, reverted, and re-run green. The
quoted text is the actual failure output, not a prediction.

### 1. Hardened derivation stalls under the screensaver — adversarial C-1 (Critical)

**Change.** `hashlockDeriveFlow`'s per-frame closure now calls `ctx.KeepAwake()` then
`ctx.WakeupAt(time.Now())` immediately before `ctx.Frame`, mirroring `unlockDerive`
(`gui/unlock_kdf.go:334-335`, F-93), with the mechanism in the closure's own comment.
New test `TestHashlockDeriveKeepsAwakeUnderTheScreensaver` in `gui/composer_hashlock_test.go`,
built on the fork's Run-level harness (`newDeadlinePlatform()`, `mustFinish`, `synctest`)
at `p.tickFloor = 1 * time.Second` — 201 frames of bubble time against `idleTimeout`'s
180 s.

**Why not the touch harness.** `runUITouch` sets `ctx.FrameCallback` directly and never
runs Run's idle loop, so it is structurally blind to this class, exactly as the reviewers
said. The brief offered the emulator walk as a fallback; it was not needed — **the Critical
is closed by a test that runs in CI**, and the plan says so in Self-review item 2.

**RED — delete `ctx.KeepAwake()`:**

    --- FAIL: TestHashlockDeriveKeepsAwakeUnderTheScreensaver (0.13s)
        composer_hashlock_test.go:806: Run exceeded 100000 ticks without terminating --
        flow is probably parked (screensaver?). 180 frames drawn,
        last = "89%About21secondsleft.Deriving"

**A second RED the first version MISSED, found by mutating the other half.** Deleting
`ctx.WakeupAt(time.Now())` and keeping `KeepAwake` **passed** the first draft of the test
(`ok  seedhammer.com/gui  0.051s`): KeepAwake alone keeps the saver off, so the park check
cannot see it — while on the device every `AppendEvents` then waits out Run's own
`ctx.WakeupAt(idleWakeup)`, three minutes per 500-iteration slice. The test was strengthened
with a bubble-clock bound before being accepted:

    --- FAIL: TestHashlockDeriveKeepsAwakeUnderTheScreensaver (0.05s)
        composer_hashlock_test.go:831: the derivation took 9h57m1s of device time; at a 1s
        tick floor and 200 frames it should take about 3m20s. A frame that omits
        ctx.WakeupAt(time.Now()) waits out Run's idle deadline (3 min) instead of the next
        500-iteration slice

**GREEN:** `ok  seedhammer.com/gui  0.055s`.

### 2. Decoder off-by-one and a vacuous digest test — adversarial C-2 = fidelity I-6 = tests C-1

**Both false PASSes were reproduced before being fixed.**

*The digest half.* With `Digest` mutated to double-hash, the round-0 body
(`if h := Digest(&x); h == x { t.Fatalf("Digest is the identity") }`) reported
`ok  seedhammer.com/hashlock  0.002s`. **Change:** a `Digest` field joined the `Kind`
struct and the test now compares against the corpus constant. Same mutation, RED:

    --- FAIL: TestKindRowPreimageDigest (0.00s)
        hashlock_test.go:201: kind[0] digest: got 88b8f02ce56abce1d453e0610318130f4d0a13067549e804af1f5186f81a2691
        want 9a2db2e23f1504cd056606553ac049c5e718e8f9ce9233876df1a7a1821af885

*The decoder half.* With `copy(preimage[:], d[:32])` in place of `d[1:]`, the round-0
`TestDecodeMS1PreimageIsShapeExact` reported `ok  seedhammer.com/codex32  0.001s`.
**Change:** the test reads the vendored corpus by relative path instead of a transcribed
literal (its import block gains `encoding/json` and `os`), compares all 32 bytes to
`preimage_hex`, adds §7.4's acceptance-record plate
(`ms10hashsq0p7jaf…`, `ms-hashlock-H1-acceptance.md` item 3) → the anchor row's
`hardened_x`, and drives §7.1's "entr32 pair" clause in BOTH directions
(`DecodeMS1Preimage(entr32_pair)` → `errMSBadPrefix`; `DecodeMS1(entr32_pair)` → prefix
`msPrefixEntr`, language 0, and the same 32 bytes as the hash plate). Same mutation, RED:

    --- FAIL: TestDecodeMS1PreimageIsShapeExact (0.00s)
        mspayload_test.go:185: preimage = 03ababab…abab,
        want the corpus's preimage_hex ababab…abab

**GREEN both.** The declared `!f.Unshared` mutation was re-run and still fires:
`DecodeMS1Preimage(a 2-of-N share beginning 0x03) err = <nil>, want codex32: not an
m-format secret payload`.

### 3. Reconciliation line unreachable on a mixed policy — adversarial I-1 = fidelity I-2 = journey I-3

**Change.** New `composerCopyHashlockReconcile()`, shown by its own `showError` immediately
after HOLD in `hashlockPhraseRoute`; `composerCopyHashEveryPathPhrase` returns to §4.7's
text verbatim. Resolved INSIDE the spec's intent rather than its literal step: §4.5's
drop-order step 2 names the phrase-route §8h at Done as the destination, and §8h is guarded
by `composerEveryPathHashed` (`gui/composer_state.go:239` at `c4a64fc`), false for any
policy with one un-hashed path. The removal from the modal STANDS; only the destination
moved. **The spec is not edited** — the exact replacement sentence is an H3 record item in
the plan.

New test `TestHashlockReconcileScreenIsReachableOnAMixedPolicy` builds the ordinary mixed
shape (path 0 hashed with a different digest, path 1 taking the phrase route) and
**asserts `composerEveryPathHashed(st.list) == false` before walking**, so it fails loudly
if it ever stops being the case §8h's guard rejects.

**RED — delete the `showError(…, composerCopyHashlockReconcile())` call:**

    --- FAIL: TestHashlockReconcileScreenIsReachableOnAMixedPolicy (2.03s)
        composer_hashlock_test.go:891: never reached "run ms hashlock with this phrase";
        last frame "hashb867db87..edbc96cbmethod:sha256chars:28anotherpathhasadifferenthash:…"

**Re-measured fit** (`assertModalBodyFits`, margin 80): confirm modal **337 drawn /
headroom 107**, reconciliation screen **94 / 455**, §8h phrase form **160 / 378**, hardened
warning wrapped **189 / 302**, SHA-256 warning wrapped **226 / 302**, ms1-plate refusal
**91 / 476**. The confirm variant is the longest one — relation line AND other-path line,
`chars: 100` — and clears the margin by 27.

### 4. `hashByPhrase` never cleared, and its assignment never verified — adversarial I-2 (+ fidelity M-2 / journey M-1 / tests I-4)

**Change.** `composerHashByPhraseSync(st)` in `gui/composer_hash.go`, beside its only
caller, drops the flag from the `noneRow` arm once NO path carries a hash. It is
deliberately not cleared on the narrower events — see §3 of this report. The assignment is
now driven through the real route and asserted; the clear is asserted too.

**RED — delete the sync call:**

    --- FAIL: TestComposerHashEditDispatchesByRowLabel/none_row_clears_without_the_rule_modal
        composer_hashlock_test.go:706: st.hashByPhrase survived the last hash being cleared

**RED — delete `st.hashByPhrase = true` in `hashlockPhraseRoute`:**

    composer_hashlock_test.go:899: the phrase route did not record that this hash was set by phrase

### 5. `Type 64 hex` Back untested, false coverage claim — adversarial I-3 = fidelity I-4 = journey I-4

**Change.** New `TestHashlockHexRowBackKeepsThePath` drives it through `composerAddPath`
(the creation entry point, where `false` deletes the path) and asserts the path survives
with `Hash == nil`. Task 3's false sentence ("Task 4's harness tests do") is corrected in
place and now names the test.

**RED — `return false` in place of `continue` in the hex arm:**

    --- FAIL: TestHashlockHexRowBackKeepsThePath (1.01s)
        composer_hashlock_test.go:733: never reached "Type a hashlock phrase";
        last frame "0123456789ABCDEF0of64hexHashlock"

It fails EARLIER than at the path-count assertion — the false unwinds `composerAddPath`,
which deletes the path and leaves the screen. The test's comment records that, so a future
reader is not surprised.

### 6. `Deriving` zero-state lead unreachable — adversarial I-4 (Important) / fidelity M-1 / journey M-2 (Minor)

**Change.** The lead is now the pure function `hashlockDerivingLead(done, total, elapsed)`,
and `hashlockDeriveFlow` draws a zero-state frame BEFORE the first `Step(500)` — which is
the actual fix, since the guard's spelling was never the defect (`done > 0 && elapsed > 0`
and `done <= 0 || elapsed <= 0` are the same predicate; the problem was that it was only
ever evaluated inside a callback whose first call arrives at `done = 501`).

**RED — return the estimate unconditionally:**

    --- FAIL: TestHashlockDerivingLead (0.00s)
        composer_hashlock_test.go:770: the zero-state frame: hashlockDerivingLead(0, 100000, 0s)
        = "About -9223372036 seconds left.", want "Deriving. This takes about 10 seconds."
        (and two further rows)

**RED — delete the hoisted `frame(0, hashlock.Iterations)`:**

    composer_hashlock_test.go:814: only 199 frames drawn; 100,000 iterations in 500-step slices is 201

### 7. C-4 regression protection real but mis-attributed — fidelity I-1, refined by tests I-2

**Change, both halves.** (a) `TestWhichHashRowsAreLabelKeyed`'s comment no longer claims to
catch a dispatch mutation it structurally cannot see, and names the test that does; its own
mutation is now one it CAN see (swap the phrase and hex appends → `n=0: indices 1/0/2`).
(b) New `TestComposerHashEditDispatchesByRowLabel` drives `composerHashEdit` through the
screen with **two payload digests loaded**, four subtests: payload row 2 assigns payload
digest 2, hex row opens hex entry and does not clear, phrase row opens the phrase screen,
none row clears without the rule modal.

**RED — the surgical index-arithmetic reversion** (`case sel == len(rows.digests)` for the
phrase row, `default` clears):

    --- FAIL: TestComposerHashEditDispatchesByRowLabel/hex_row_opens_hex_entry_and_does_not_clear
        composer_hashlock_test.go:663: never reached "0 of 64 hex"; last frame
        "ThehashmustbeSHA-256ofa32-bytevalue.…Path1hash"

**Fidelity I-1's core claim reproduced, by measurement:** under that same mutation
`TestWhichHashRowsAreLabelKeyed` and `TestHashlockPhraseRouteSetsTheCorpusDigest` both
stayed **GREEN**. This is now recorded in the plan's Task 4 mutation table.

### 8. Fit-gate renderer mismatch — fidelity I-3 = journey I-2 (PARTIAL)

**Change.** The three `ConfirmWarningScreen` bodies moved out of
`TestModalsThisBlockTouchesAreDrawnInFull` into a new
`TestConfirmScreensThisBlockTouchesAreDrawnInFull`, measured through `confirmWarningBody`
and wrapped in `composerConfirmBody` as production draws them. A separate table rather than
a renderer column, because the existing table's rows are positional composite literals and
a third field would have had to be added to every pre-existing row — attempted first, and
`vet: gui/modal_fits_test.go:321:3: too few values in struct literal` is why it was not
kept. Journey's further step is declined (§3 below). As the refute predicted, **no number
moved**: the headroom values are the same ones it measured.

### 9. Relation line's no-match branch untested — fidelity I-5

**Change.** `hashlockRelationLine(payload, h)` extracted as a pure function, and
`TestHashlockConfirmRelationLine` parameterised over three cases: the SECOND record matches
(pins the 1-based index), neither matches (reaches the no-match arm), and no records at all
(asserted directly on the pure function).

**RED — `match := 0`:** the no-match case reports `matches hash 1 in the payload`.
**RED — `%d` on `i` rather than `i+1`:** the second-record case reports `matches hash 1`
where `matches hash 2` is wanted. **RED — drop the `len(payload) == 0` arm:**
`no payload records drew the relation line "no hash: record in the payload has this digest"`.
Each mutation fails a DIFFERENT case; round 0's single case could distinguish none of them.

### 10. Two paths, two phrases, no cross-check — journey I-1

**Change.** `hashlockOtherPathLine(st, idx, h)` compares the new digest against every OTHER
path's `*p.Hash` and adds a second §4.5 relation line,
`composerCopyHashlockOtherPath()` = "another path has a different hash: two phrases to back
up". It reads the live hashes, not `hashByPhrase`, so it is unaffected by that flag's
staleness. `TestHashlockOtherPathLineIsSilentOnAnEqualHash` covers equal, different, self
and none.

**RED — return `""` always:** `never reached "two phrases to back up"` (screen level) and
`a DIFFERENT hash on another path drew "", want the warning` (unit level).
**RED — drop the `*p.Hash != h` comparison:** `an EQUAL hash on another path drew "another
path has a different hash: two phrases to back up", want silence`.

### 11. §8i rule modal confusing ahead of the phrase route — journey I-5

**Change, copy only.** `composerCopyHashlockPhraseLead()` now opens "This screen does that
hashing for you.", answering the modal the operator has just dismissed
(`composerCopyHashRule`: "A passphrase must be hashed to 32 bytes first, then hashed
again"). The cheaper of the two remedies the refute named — no new gate row, no new screen.
Its `composerCopyTable` row moved with it, and the layout was checked by running the phrase
screen's own tests (the longer lead shortens the keyboard's `MaxHeight`; every
`TestHashlock*` test that types a phrase still passes, so no key was pushed off-canvas).

### 12. Task 3 Step 5 stub under-specified — coverage I-1

**Change.** Step 5 now SHOWS the stub — the `hashlockOutcome` type and both constants, not
"a one-line stub" — and the block was **compiled**, not asserted: dropped into a copy of the
gated tree with Task 4's copy bodies and `composer_hashlock_test.go` removed,
`go build ./gui/` exited 0.

**A defect of my own fold, caught by that compile.** My first fix placed
`composerHashByPhraseSync` in `gui/composer_hashlock.go` — the file Task 3's stub replaces —
so `composerHashEdit`'s call to it would not have compiled at Task 3. The function was moved
to `gui/composer_hash.go`, beside its only caller. Coverage's M-1 companion is folded
alongside: Task 3's `Files:` header now names `gui/composer_hashlock.go`.

### 13. Task 4 Step 2's RED claim does not reproduce — tests I-1

**Change, documentation.** The Expected line now says the package COMPILES (Task 3's stub
already declares every symbol the new test file names) and the failures are at RUNTIME, in
`tapPassphraseKey`, with `no *PassphraseKeyboard was registered for this harness` — and says
why that still is the RED checkpoint.

### 14. `DeriveHardened`'s own abandon contract untested — tests I-3

**Change.** New `TestDeriveHardenedAbandonsWhenProgressSaysStop` in `hashlock`.

**RED — `progress(d.Done(), d.Total())` without the early return:**

    --- FAIL: TestDeriveHardenedAbandonsWhenProgressSaysStop (0.01s)
        hashlock_test.go:242: DeriveHardened returned ok=true after progress abandoned it
        hashlock_test.go:245: progress was called 199 times; abandoning must stop the KDF at
        the third call, not run it to completion (200 calls)

The third assertion (the result must be the zero value) deliberately does **not** log the
bytes.

### 15. `minMS1Len` 47/48 boundary untested — tests I-5

**Change.** New `TestIsMS1ShapedMinLengthBoundary`, with **literal** 47- and 48-character
inputs and their display-grouped forms. The literals matter: a first draft derived them from
`minMS1Len` itself, and the mutation then only tripped the constant's own pin rather than a
boundary row. Rewritten so the rows fail.

**RED — `minMS1Len = 47`:** `47 characters must be BELOW the ms1 shape bound` plus the
grouped row. **RED — `minMS1Len = 49`:** the two 48-character rows instead.

### 16. `IsMS1Shaped`'s `TrimSpace` — tests I-6

**Change.** The coverage gap is closed by `TestIsMS1ShapedTrimsWhatTheStripLoopCannot`.
The reviewer's remedy (delete the call) is **declined** — see §3.

**RED — remove `strings.TrimSpace`:** ten failures,

    hashlock_test.go:316: "\v" + the plate is not ms1-shaped -- the host trims this
    character before its own shape test, so the port must too
    (and \f, U+0085, U+00A0, U+2003; leading and trailing)

---

## 2. Severity disputes — resolved, with reasons

| dispute | votes | resolution | reason |
| --- | --- | --- | --- |
| (a) decoder / digest-test gap | C×2 (one executed) vs I×1 | **CRITICAL** | Project policy keeps "a test that reports a false PASS" in the blocking class regardless of operator-visible impact. Both false PASSes were re-executed here before being fixed, so the Critical side now has two independent executions behind it. |
| (b) `hashByPhrase` never cleared | I×1 vs M×2 | **MINOR** | The failure is additive copy: `composerCopyHashEveryPathPhrase` names "the phrase and its method, **or** the preimage plate", so a stale-true flag tells the operator to back up one artifact too many, never one too few. The dangerous direction (a phrase-set hash with the flag false) is unreachable while the flag is only ever set. Folded anyway where the fix is cheap and correct; the rest is an H3 follow-up. |
| (c) `Deriving` zero-state lead | I×1 vs M×2 | **IMPORTANT** | §4.4 states the zero-state lead normatively. Dead normative copy is an unmet spec guarantee, not a cosmetic defect — the same class the project keeps blocking. Folded with a test either way, so the rating changes no outcome; it is recorded because the refute asked for it not to be silently dropped. |

---

## 3. Declined, with reasons

1. **tests I-6's remedy, "delete the redundant `TrimSpace`" — DECLINED; the finding is
   folded, the remedy is not.** The premise is **false, measured**: the strip loop skips
   exactly `' '`, `'\t'`, `'\n'`, `'\r'`, `'-'`, `','`, while `TrimSpace` removes everything
   `unicode.IsSpace` reports at the ends. Removing it flips `IsMS1Shaped` from true to false
   for `'\v'`, `'\f'`, U+0085, U+00A0 and U+2003 — probed before and after, ten rows.
   It would also have **diverged from the Rust primary**: `looks_like_ms1` is
   `is_ms1_shaped(&raw.trim().to_ascii_lowercase())`
   (`mnemonic-secret/crates/ms-cli/src/argv_guard.rs:148-149`, read directly), and Rust's
   `str::trim` uses the White_Space property, covering all of them. Deleting the call would
   have weakened a refusal rule on a funds-relevant input. The coverage gap the finding
   actually names is closed by a test that pins the behaviour instead.
2. **journey I-2's "re-run §4.5's drop order from step 0; the line goes back if the
   unshortened body now fits" — DECLINED.** Already refuted by direct measurement in the
   refute pass (delta 0 across seven bodies; `warningBodyClip` depends only on `dims`), and
   the labeling half of the finding is folded (item 8). The line's real loss was §8h's
   guard, and item 3 is the fix.
3. **Per-path hash provenance in place of `composerState.hashByPhrase` — DEFERRED, filed as
   a follow-up with owning phase H3** (recorded in the plan's Task 6). Clearing the flag
   whenever THIS path's hash is replaced would be wrong while another path is still
   phrase-set — the C16 shape the refute itself flagged — and a per-path array needs the
   same splicing discipline `composerAddPath` and "Remove path" already apply to `Paths`,
   which is a change to the composer's state model rather than to this route. The cheap,
   correct half (clear once no path carries a hash) is folded.

Nothing else on the refute's list was declined. No REFUTED item was acted on.

---

## 4. Spec departures — recorded in the plan, spec NOT edited

Two, both written out as exact replacement sentences in the plan's
`## R0 round 0 folded here` as H3 record items, and filed as follow-ups with owning phase
H3 in Task 6:

1. **§4.5's drop-order step 2** currently sends the reconciliation line to the phrase-route
   §8h at Done, whose guard makes it unreachable. Replacement sentence written out in full.
2. **§4.5's line list** gains the other-path line (journey I-1). Replacement line and
   replacement bullet written out in full.

---

## 5. Mutation re-runs of the plan's PRE-EXISTING declared mutations

Re-run because the code under them changed. All still fire.

| Declared mutation | Measured now |
| --- | --- |
| `seal.NormalisePassphrase` in `hashlockPhraseFlow` | `TestHashlockPhraseRouteDoesNotNormalise`: `"Correct Horse Battery Staple": path hash = …, want 95d4447031cdc411…` |
| the confirm's Back returns `hashlockBackToWhichHash` | `TestHashlockBackContractKeepsThePath`: `never reached "Which method?"` |
| `composerHashEdit` returns `false` from the phrase route's Back | `TestHashlockBackContractKeepsThePath`: `never reached "Type a hashlock phrase"` |
| remove the relation line | `TestHashlockConfirmRelationLine`, **both** cases now (round 0 had one) |
| drop `!f.Unshared` (Task 2) | `DecodeMS1Preimage(a 2-of-N share beginning 0x03) err = <nil>, want …` |
| `Salt` zero-padded / `Iterations = 99999` / normalise / separator-strip / `codex32.New` shape test / cap literal 99 (Task 1 table) | untouched by this fold; the table's measured outcomes stand, and four new rows were added beside them |

Not re-run: **delete the release event from `holdConfirm`** — the fold did not touch
`holdConfirm`, and the plan records that mutation as hanging rather than failing, so
running it costs a `go test` timeout for no new information.

---

## 6. Whole-package, shard and checker output

All from `/scratch/code/shibboleth/.tmp/h2-gate`.

    $ go test -count=1 ./hashlock/... ./codex32/... ./seal/... ./sysw/...
    ok  	seedhammer.com/hashlock	0.232s
    ok  	seedhammer.com/codex32	0.003s
    ok  	seedhammer.com/seal	14.955s
    ok  	seedhammer.com/sysw	0.038s

    $ scripts/gui-shard-test.sh ./gui/ 24
        1220 top-level tests
        partition verified exhaustive: 1220 == 1220
      shard 1: ok    51 tests  … shard 23: ok    50 tests
    === wall: 29s ===
    RESULT: ok -- all 1220 tests ran across 24 shards

1213 before the fold, 1220 after: the seven new `gui` top-level tests are
`TestComposerHashEditDispatchesByRowLabel`, `TestConfirmScreensThisBlockTouchesAreDrawnInFull`,
`TestHashlockDeriveKeepsAwakeUnderTheScreensaver`, `TestHashlockDerivingLead`,
`TestHashlockHexRowBackKeepsThePath`, `TestHashlockOtherPathLineIsSilentOnAnEqualHash`,
`TestHashlockReconcileScreenIsReachableOnAMixedPolicy`. `hashlock` went from 6 to 9 tests
(separate package, not in the 1220).

    $ scripts/h2-plan-blocks-vs-tree.sh
    26 blocks checked, 0 FAIL

26, not 25: one block was added (`codex32/mspayload_test.go`'s import block, which the new
corpus-reading test needs). The output embedded in the plan's "Build gate folded here"
section was re-captured after the last edit and **diffed against a fresh run**, so the
line numbers and line counts in the plan are current, not stale.

`go vet ./hashlock/... ./codex32/... ./seal/... ./sysw/... ./gui/` reports only the two
PRE-EXISTING `testing.ArtifactDir requires go1.26` complaints
(`gui/freetext_sizeproof_golden_test.go:111`, `gui/transaction_golden_test.go:104`).
`gofmt -l` over the five packages reports only the three PRE-EXISTING `gui/transaction*`
files. Both baselines were verified against the fork at `c4a64fc` rather than assumed.

---

## 7. Citations added by this fold — re-grepped against the fork at `c4a64fc`

Every new `file:line` was checked with `sed -n` against
`/scratch/code/shibboleth/seedhammer` at `c4a64fc`, and **three were wrong on the first
pass and corrected**:

| citation | first written | corrected to |
| --- | --- | --- |
| `composerEveryPathHashed` | `gui/composer_state.go:244` (the TREE's number — the `hashByPhrase` field shifts it) | `gui/composer_state.go:239`, qualified "at the fork baseline `c4a64fc`" |
| `modalRenderer` | `gui/modal_fits_test.go:109` (tree) | `gui/modal_fits_test.go:108` (fork) |
| the creation-time delete | `gui/composer_shape.go:266-269` (the refute's own number) | `gui/composer_shape.go:269-272`, qualified to the fork baseline |

A fourth correction is a **stale mechanism**, not a stale line number: the fork's own
`unlock_kdf.go` comment says "Run refreshes `a.idle.start` only on `len(evts) > 0`", and
`run_flow.go:350` now reads `if effective || (ctx.keepAwake && !armed)` — F-103 replaced
the arrival test with `effectiveInput`. Copying the fork's sentence would have propagated a
comment that has outlived its condition, so the new comment names the current predicate.

Verified correct as written: `gui/gui.go:110,119` (`WakeupAt`, `KeepAwake`),
`gui/gui.go:3584` (`idleTimeout = 3 * time.Minute`), `gui/gui.go:595-600`
(`warningBodyClip`), `gui/unlock_kdf.go:334-335`, `gui/run_flow.go:350-351`,
`gui/run_flow.go:401-406`, `gui/run_harness_test.go:58,183,220`,
`gui/run_flow_test.go:671`, `md/compose.go:299`,
`mnemonic-secret/crates/ms-cli/src/argv_guard.rs:148-149`.

---

## 8. What I could not fold, and what is still owed

- **Nothing on the refute's 16 was left unfolded.** The three declines above are decisions
  with stated reasons, not omissions; two of them decline a *remedy* while folding the
  *finding*, and the third is a scheduled follow-up with an owning phase.
- **The emulator walk (Task 5 Step 1) is still prose and still un-run** — out of scope for
  both the build gate and this fold, and it remains the plan's one un-gated executable
  artifact. It is no longer load-bearing for adversarial C-1.
- **The firmware size number is now stale.** The fold added **183 lines of production Go**
  (measured: `gui/composer_hashlock.go` +119, `gui/composer_copy.go` +39,
  `gui/composer_hash.go` +25; 78 of them non-comment), so Task 5 Step 2's 1,595,236 B /
  62,856 B predates it. The plan now says so at that step and in the fold section; the
  acceptance is the delta against `c4a64fc`, not the literal, so this is a re-measure before
  merge rather than an open defect.
- **Two spec departures are recorded but not applied to the spec** (§4 above), by
  instruction. Whoever folds the spec at H3 has both exact sentences.
- **`STATUS` is now** `DRAFT -- R0 round 0 folded; r1 fold verification pending.`
