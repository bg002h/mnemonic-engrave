# B2b follow-up reconciliation sweep — F-87, F-89, F-93, F-96, F-105

**Date:** 2026-08-10. **Scope:** mechanical verification only, against code —
not the register's own claims. Firmware checked read-only at
`/scratch/code/shibboleth/seedhammer-b2b` @ `3de8aa1` (branch `b2b`) and
`/scratch/code/shibboleth/seedhammer-gate-orphan` @ `231a222` (branch
`b2b-residency`, = `b2b` + `89235db` + `231a222`, the F-107/F-108 residency
work). No file in either firmware worktree was modified. All `go test`
invocations below were run via
`/nix/var/nix/profiles/default/bin/nix develop /scratch/code/shibboleth/seedhammer --command go test ./gui/ ...`
from inside `seedhammer-b2b`, and their PASS/FAIL output is pasted verbatim,
not summarised from a comment.

## 1. Summary table

| item | verdict | satisfied on | justification |
| --- | --- | --- | --- |
| F-87 | **PARTIAL** | `b2b` (2 of 3), same on `b2b-residency` | Two of three early returns pinned and tested; the third (`masterFingerprintFor` error, `unlock_session.go:284-288`) has no test seam — confirmed by commit `920e1e1`'s own message and by grep (`grep -n "Couldn't derive the fingerprint" gui/*_test.go` → no hits). |
| F-89 | **SATISFIED** | `b2b` (and `b2b-residency`, unaffected by F-107/F-108) | Both halves done: the unwind mechanism (`run_flow.go:183-187`, `:201-207`) and `RecordsResident`'s corrected, narrow-reading contract (`seal/session.go:20-51`) with the timer keyed on the guard's own lifetime, not the predicate (`wipe_guard.go:37-60`). Pinned end-to-end by `TestWipeZeroesEveryPinnedBufferAtRunLevel` (`gui/wipe_inventory_audit_test.go:200`), which PASSED. |
| F-93 | **SATISFIED** | `b2b` (and `b2b-residency`) | `unlockDerive` calls `ctx.KeepAwake()` per slice (`gui/unlock_kdf.go:327`), and `run_flow.go:150`'s `(ctx.keepAwake && !armed)` term is what reconciles it with F-89. Two mutation-killed tests in `gui/run_flow_test.go` both PASSED when run. |
| F-96 | **SATISFIED (register is stale)** | `mnemonic-engrave` repo (not the firmware worktrees — this item is tooling, not gui code) | Phase report exists (`design/PHASE_REPORT_encrypted_payload_deviceB_phaseB2a_ii.md`, committed `09996e2`) and `scripts/mutation-run.py` is committed at `dd3d4b3` (Task 7) with a real, cited 16/16-KILLED run recorded in the commit message. The FOLLOWUPS.md entry's own text — "The runner half stays open until Task 7 commits" — is now false; Task 7 has committed. |
| F-105 | **PARTIAL** | software: `b2b`; hardware: **neither** | The software fix (`unlockPassphraseFlow`'s own bracket, `gui/unlock_kdf.go:135-137`, closing before `unlockAttemptOnce`/`unlockDerive`) is landed and pinned by 4 tests in `gui/unlock_passphrase_wipe_test.go`, all of which PASSED. Task 9.5's hardware validation ("type two words, stop, wait 3:30" on the real SH2) has not been run — no `HARDWARE_RESULT_*.md` exercises the passphrase-entry screen; all three existing hardware docs test row 1 (post-unlock Cut/Skip), and F-106 (open, CRITICAL, unfixed on `3de8aa1`) would likely confound a clean 9.5 read today since `unlockPassphraseFlow`'s guard install is subject to the same late-arm-edge mechanism. |

**OUTSTANDING count: 0** (none of the five is fully outstanding). **PARTIAL: 2** (F-87, F-105). **SATISFIED: 3** (F-89, F-93, F-96 — F-96's register needs a stale-text correction but the underlying work is done).

## 2. Per-item detail

### F-87 — nothing pins `unlockEngraveMnemonic`'s deferred wipe

**Demand:** drive each of the function's three early returns
(`!ss.Confirm`, `masterFingerprintFor` error, `engraveSeed` error) with
`unlockMnemonicParsedHook` set and assert `m`'s words are zero after the
flow returns — because a defer only runs on return, and only a test built
on the *right* hook (fired immediately after `defer clear(m)` is
registered) can discriminate deleting that defer from keeping it.

**Code, `gui/unlock_session.go:255-321`:**

```go
255 func unlockEngraveMnemonic(ctx *Context, th *Colors, rec []byte) {
256     m, err := bip39.Parse(rec)
257     if err != nil { showError(...); return }          // early return #1 — NOT F-87's scope (before defer clear(m) exists)
264     defer clear(m)
269     if unlockMnemonicParsedHook != nil { unlockMnemonicParsedHook(m) }
277     if !ss.Confirm(ctx, th, m) { return }               // early return #1 of the THREE named
284     mfp, err := masterFingerprintFor(m, &chaincfg.MainNetParams, "")
285     if err != nil { showError(...); return }            // early return #2 of the THREE named — UNTESTED
290     plate, err := engraveSeed(params, m, mfp)
291     if err != nil { showError(...); return }            // early return #3 of the THREE named
```

Two tests exist and both PASSED when run directly:

```
=== RUN   TestUnlockEngraveMnemonicZeroesMOnConfirmDiscard
--- PASS: TestUnlockEngraveMnemonicZeroesMOnConfirmDiscard (0.05s)
=== RUN   TestUnlockEngraveMnemonicZeroesMOnEngraveSeedError
--- PASS: TestUnlockEngraveMnemonicZeroesMOnEngraveSeedError (0.05s)
```

`TestUnlockEngraveMnemonicZeroesMOnConfirmDiscard` (`unlock_session_test.go:795`)
covers the `:277` return; `TestUnlockEngraveMnemonicZeroesMOnEngraveSeedError`
(`:845`) covers the `:291` return. Both call
`assertMnemonicZeroed`, which `t.Fatal`s if the hook never fired (so
neither is vacuous — the fired-guard is itself asserted, per the file's own
stated discipline).

**No test exists for the `:285` return** (`masterFingerprintFor` error).
`grep -n "Couldn't derive the fingerprint\|masterFingerprintFor" gui/*_test.go`
returns only unrelated fingerprint-derivation tests (`bip85_test.go`,
`gui_test.go`, `seedxor_polish_test.go`) — none drives this call site's
error branch.

This is not an oversight silently left in place: commit `920e1e1`'s own
message ("F-87's pins (2 of 3)") states it explicitly and gives an
exhaustive argument that the branch is structurally unreachable through any
real mnemonic/password (`bip39.MnemonicSeed` always returns exactly
`hdkeychain.MaxSeedBytes`; `hdkeychain.NewMaster`'s only other failure mode
is a scalar landing outside the curve order, "< 1 in 2^127" per its own doc
comment; `ECPubKey()` cannot fail on an already-validated scalar). Adding a
seam would mean modifying `masterFingerprintFor`/`deriveMasterKey`
themselves — shared funds-path code the phase's own comment (line ~245)
already scopes out, the same code F-94 (re-assigned to B2c) already owns.

**Verdict: PARTIAL.** 2 of 3 legs pinned and passing; the third is an
acknowledged, argued-unreachable gap, not a silent one — but the item as
worded ("nothing pins the deferred wipe") is not yet fully closed by the
letter of its own text.

**What would settle it, if the operator wants it fully closed rather than
accepted as-is:** either a seam inside `masterFingerprintFor`/
`deriveMasterKey` reachable without `unsafe` (F-94's territory, and it is
explicitly filed there — "shared funds-path code… widens the diff… this
phase does not otherwise touch"), or an explicit register decision to
accept the gap as structurally unreachable and close F-87 on 2-of-3 with
that reasoning recorded. This is a judgment call, not something this sweep
resolves.

### F-89 — B2b's idle wipe MUST unwind the flow, not just call `p.Wipe()` (+ `RecordsResident`'s contract)

**Demand, half 1 (the unwind):** the timer must make the flow *return* —
so a `defer clear(m)` on some early-returning function actually fires — not
wipe `p.Secret` in place while the flow is still parked mid-function.

**Demand, half 2 (the funds-relevant half):** `SecretsResident()`'s
contract must be corrected — it scans `p.Secret` only and goes false the
instant `clear(rec)` runs, while string copies of an `ms1` share are still
live — and B2b's timer must not be keyed on it.

**Code — the unwind, `gui/run_flow.go`:**

```go
183   if wipeNowHook != nil && wipeNowHook() {
184       wiping = true
185       ctx.Done = true
186       break // unwind, never exit
187   }
...
201       if armed { // §10.2.4's window: warn, then wipe
203           if now.Sub(wipeAt) >= 0 {
204               wiping = true
205               ctx.Done = true
206               break
207           }
```

`ctx.Done = true` is read by `Context.Frame` (`gui/gui.go:84-89`) via
`FrameCallback`, which returns immediately once `ctx.Done` is true
(`run_flow.go:73-75`) — so every one of the package's `for !ctx.Done { ...
ctx.Frame(...) ... }` loops (dozens, e.g. `gui.go:588,615,713,816,921,...`)
exits on its own next iteration, unwinding the whole call stack and firing
every registered `defer` on the way out, including `unlockEngraveMnemonic`'s
`defer clear(m)`.

**Code — `RecordsResident`'s corrected contract, `seal/session.go:20-51`:**
the doc comment explicitly disclaims the wide reading ("READ THAT
NARROWLY... this function was renamed from `SecretsResident` to
`RecordsResident`, so nobody builds a control on the wide reading again"),
and the implementation (`:51-63`) is unchanged — it still scans `p.Secret`
only, which is now documented as correct-but-narrow rather than
mis-described as "secrets are gone."

**Code — the timer does NOT key on it, `gui/wipe_guard.go:37-60`:**
`armed()` returns `g != nil` gated only by job state (`engraveRunning`,
`engraveStopping` disarm), never touching `RecordsResident`.
`run_flow.go:136-140`'s own comment: *"the timer keys on the SESSION
BRACKET's lifetime, never on seal.RecordsResident."*

**Test, `gui/wipe_inventory_audit_test.go:200-264`,
`TestWipeZeroesEveryPinnedBufferAtRunLevel`:** drives the REAL Run loop and
the REAL 3:00/30s timer (not a synthetic `p.Wipe()` call) to a park on
`SeedScreen.Confirm` with `bip39.Parse`'s live `m` local captured via
`unlockMnemonicParsedHook`, and asserts `allZeroWords(m)` after the wipe —
explicitly named in its own comment as "the exact F-89 shape." Ran:

```
=== RUN   TestWipeZeroesEveryPinnedBufferAtRunLevel
=== RUN   TestWipeZeroesEveryPinnedBufferAtRunLevel/vectorA-parked-on-seed-screen
=== RUN   TestWipeZeroesEveryPinnedBufferAtRunLevel/vectorF-parked-on-ms1-cutskip
--- PASS: TestWipeZeroesEveryPinnedBufferAtRunLevel (0.67s)
    --- PASS: TestWipeZeroesEveryPinnedBufferAtRunLevel/vectorA-parked-on-seed-screen (0.27s)
    --- PASS: TestWipeZeroesEveryPinnedBufferAtRunLevel/vectorF-parked-on-ms1-cutskip (0.40s)
```

This test discriminates the exact regression F-89 warns against: if the
timer instead called `p.Wipe()`/`WipeSecretAt` in place without unwinding,
`rec` would zero but the captured `m` slice (same backing array the test
holds a reference to) would still read the real words, and
`allZeroWords(m)` would fail. It is not a `p.Wipe()`-in-place design; it is
the unwind, and the test would catch a regression to the old shape.

**Verdict: SATISFIED**, both halves, on `b2b`. `b2b-residency`'s two extra
commits (`89235db`, `231a222`) add `ctx.B.Scrub()` calls for F-107 but do
not touch `wipe_guard.go`, `RecordsResident`, or the unwind mechanism —
same state.

### F-93 — the screensaver still PARKS a spec-legal derivation

**Demand:** treat an in-progress derivation as activity (Run-side), so a
derivation past `idleTimeout` does not permanently stall behind the
screensaver's non-breaking `continue`.

**Code, `gui/unlock_kdf.go:327`** (`unlockDerive`'s per-slice loop, just
before `ctx.WakeupAt`/`ctx.Frame`):

```go
327     ctx.KeepAwake()
        ctx.WakeupAt(time.Now())
        ctx.Frame(op.Layer(...))
```

**Code, `gui/run_flow.go:150`:**

```go
150     if len(evts) > 0 || (ctx.keepAwake && !armed) {
151         a.idle.start = now
152     }
```

The `&& !armed` term is what reconciles this with F-89/§10.2.4: `KeepAwake`
holds off the *screensaver* only, and is ignored whenever the wipe timer is
armed, so a derivation can never use it to postpone a wipe.

**Tests, both PASSED:**

```
=== RUN   TestRunKeepAwakeDuringDerivationDoesNotParkUnderTheScreensaver
--- PASS: TestRunKeepAwakeDuringDerivationDoesNotParkUnderTheScreensaver (0.06s)
=== RUN   TestRunKeepAwakeCannotPostponeAnArmedWipe
--- PASS: TestRunKeepAwakeCannotPostponeAnArmedWipe (0.01s)
```

Per commit `237c85f`'s message, these kill 3 mutation rows: `ctx.keepAwake`
→ `false`; `(ctx.keepAwake && !armed)` → `(ctx.keepAwake)`; and a hand-run
statement-reorder (`ctx.Reset()` moved before the `keepAwake` read) — all
3 confirmed KILLED at fold time (not re-verified by this sweep, since the
commit message already records the row-by-row result and the tests
currently pass unmutated).

**Verdict: SATISFIED**, on `b2b` (and unaffected on `b2b-residency`).

### F-96 — the §11.3 mutation runner is uncommitted

**Demand:** commit the mutation runner as `scripts/mutation-run.py`, deriving
its rows from the plan's own tables rather than transcribing them, and land
the missing B2a-ii phase report.

**Code/commits, `mnemonic-engrave` repo (this repo, not a firmware
worktree — the register's phase-report and script live here):**

- `design/PHASE_REPORT_encrypted_payload_deviceB_phaseB2a_ii.md` exists
  (7864 bytes), committed at `09996e2` — "write B2a-ii's missing phase
  report — F-96's second half."
- `scripts/mutation-run.py` exists and is committed at `dd3d4b3`
  ("tooling: mutation-run.py -- run the plan's own §11.3 rows (F-96's
  runner)"), confirmed via `git log --oneline -1 -- scripts/mutation-run.py`
  → `dd3d4b3`, and `git diff HEAD -- scripts/mutation-run.py` → empty (no
  uncommitted changes). The script imports `plan-mutation-anchors.py`'s row
  scanner rather than hand-copying the table (read directly, `mutation-run.py:12-25`).
- The commit's own message records a REAL run against `seedhammer-b2b`
  (branch `b2b`, `920e1e1`): 16/16 mechanically-applicable rows KILLED, 0
  SURVIVED, final unfiltered `go test ./gui/ ./gui/op/ ./seal/ ./bip39/`
  PASS, and a deliberately-wrong-mapping self-check (row 1141 pointed at
  row 1142's test) correctly reporting SURVIVED rather than a rubber-stamped
  KILLED.

**FOLLOWUPS.md's own text is stale.** The entry (`design/FOLLOWUPS.md:1133`)
still reads: *"The runner half stays open until Task 7 commits."* Task 7
committed at `dd3d4b3`, which post-dates the entry's own "second half
CLOSED 2026-08-09" annotation (`dd3d4b3` is dated the same day, after the
entry text was last edited). The record has not been updated to reflect
that Task 7 landed.

**Verdict: SATISFIED** (both the phase-report half and the runner half are
done); the FOLLOWUPS.md entry needs a text correction, not more work.

### F-105 — a typed passphrase is wiped by NOTHING until it is submitted

**Demand (post-ruling):** an in-flight passphrase is seed-equivalent; the
wipe guard must be armed during passphrase entry, closing before the KDF
ever runs (arming across the KDF is unsurvivable — a derivation that
reaches 3:00 under an armed guard freezes under the warning and is wiped
mid-derivation, permanently un-openable).

**Code, `gui/unlock_kdf.go:109-137`:**

```go
109 func unlockPassphraseFlow(ctx *Context, th *Colors) (bip39.Mnemonic, bool) {
135     prev := ctx.wipe
136     ctx.wipe = &wipeGuard{subject: wipeWarningSubjectPassphrase}
137     defer func() { ctx.wipe = prev }()
```

The bracket is the function's own lifetime, closing via `defer` on every
return path — before `unlockAttemptOnce`/`unlockDerive` ever run (verified
by reading the call graph: `unlockPassphraseFlow` returns `(m, true)` at
`:170`, and only then does the caller proceed to derive).

**Tests, `gui/unlock_passphrase_wipe_test.go`, all 4 PASSED:**

```
=== RUN   TestUnlockPassphraseWipeDuringPartialEntryZeroesTheWordBuffer
--- PASS: TestUnlockPassphraseWipeDuringPartialEntryZeroesTheWordBuffer (0.02s)
=== RUN   TestUnlockPassphraseBracketExcludesTheKDF
--- PASS: TestUnlockPassphraseBracketExcludesTheKDF (0.09s)
=== RUN   TestUnlockPassphraseWarningIsNotArmedOnTheHashScreens
--- PASS: TestUnlockPassphraseWarningIsNotArmedOnTheHashScreens (0.15s)
=== RUN   TestUnlockPassphraseWarningShowsTheRow4Subject
--- PASS: TestUnlockPassphraseWarningShowsTheRow4Subject (0.01s)
```

`TestUnlockPassphraseBracketExcludesTheKDF` in particular samples
`ctx.wipe` on *every* drawn "Unlocking" frame (not once), which is the
right discriminator for "closes before" vs. "reopens partway through" — a
regression that re-armed mid-KDF would fail it deterministically.
`TestUnlockPassphraseWarningShowsTheRow4Subject` drives the real Run-level
timer through `unlockPassphraseFlow` parked at word entry and asserts the
warning text says "partly typed passphrase" (row 4) and never "decrypted
seed material" (row 1) — a genuine end-to-end check, not a unit test of the
bracket alone.

**Software half: SATISFIED.**

**Hardware half (Task 9.5): OUTSTANDING, on neither branch.** The
follow-up's own re-verification note is accurate and not stale: *"FIX
LANDED — b2b `749fce7` (Task 9). Hardware validation (Task 9.5) is the only
part still owed."* Confirmed by absence: none of the three hardware result
docs (`HARDWARE_RESULT_2026-08-09_phaseB2b.md`,
`HARDWARE_RESULT_2026-08-10_f106.md`,
`HARDWARE_RESULT_2026-08-10b_f106_ROOT_CAUSE.md`) exercises the
passphrase-entry screen; all three run the post-unlock Cut/Skip screen
(row 1). No doc records "type two words, stop, wait 3:30" — the exact
discriminating experiment F-105's own entry names.

Worth flagging for whoever runs it: F-106 (LATE ARM EDGE, open, CRITICAL,
gates the phase, unfixed as of `3de8aa1`) applies to *any* `wipeGuard`
installation during a flow's own execution — which is exactly how
`unlockPassphraseFlow` installs its guard at `:136`. A Task 9.5 run today
would likely also exhibit the "warning at 6:00, not 3:00" symptom, which is
F-106 spilling over rather than an F-105 defect — but the entry itself
already anticipates this ("If that warning does NOT appear, it is an
F-105 defect in its own right and not merely F-106 spilling over"),
implying the expected near-term result is a *late* but *present* warning,
not a missing one. This sweep did not run hardware and cannot settle which
outcome actually occurs.

**Verdict: PARTIAL** — software satisfied and well-pinned; the named
hardware sub-task is not done, and the register's own text already says so
correctly.

## 3. Register corrections owed

1. **F-89** (`design/FOLLOWUPS.md:2180`) — carries no closure marker despite
   being fully satisfied and pinned by `TestWipeZeroesEveryPinnedBufferAtRunLevel`.
   Should get a "CLOSED" annotation (with the test name and commit, the
   project's own convention) so a future sweep does not re-derive this.
2. **F-93** (`design/FOLLOWUPS.md:994`) — same: fully satisfied by `237c85f`
   with two mutation-killed tests, no closure marker. Should be marked
   CLOSED.
3. **F-96** (`design/FOLLOWUPS.md:1133`) — the entry's embedded note ("The
   runner half stays open until Task 7 commits") is now false; Task 7
   committed at `dd3d4b3`, same day. Update to record both halves closed.
4. **F-87** (`design/FOLLOWUPS.md:2274`) — not wrong, but incomplete: the
   entry does not mention that `920e1e1` closed 2 of 3 legs and gave an
   exhaustive unreachability argument for the third. Worth a short amendment
   pointing at `920e1e1` so the next reader does not re-derive the
   unreachability argument from scratch, and an explicit operator decision
   on whether 2-of-3 + the argument closes the item or whether a seam is
   still wanted (deferred to F-94's shared funds-path review).
5. **F-105** — no correction needed; the entry's 2026-08-10 re-verification
   note matches the code exactly.

## 4. What would gate B2b's close, restricted to these five items

- **F-89, F-93, F-96: none.** All three are done in code, tested (F-89,
  F-93) or recorded with a real run (F-96), and none blocks the phase —
  only their register text is stale.
- **F-87: a judgment call, not a code gate.** The remaining leg is argued
  structurally unreachable through real inputs; whether that argument is
  sufficient to close the item as-is, or whether B2b still owes a seam
  (which the existing commit explicitly scopes to F-94/shared funds-path
  review instead), is for the operator to decide. It does not read as a
  live defect.
- **F-105: Task 9.5, the hardware validation.** This is the one item in
  the set with genuine unfinished, phase-owned work — a named subtask
  (Task 9.5) that has not been executed on real hardware, on a
  funds-adjacent control (arming the wipe across passphrase entry). It is
  entangled with F-106 (already the phase's gating CRITICAL): running 9.5
  before F-106's fix lands will likely produce a confounded read (late
  warning from F-106, not evidence about F-105's own bracket), so the
  practical order is F-106's fix first, then Task 9.5's hardware run as a
  clean discriminator.
