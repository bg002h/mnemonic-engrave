# Hashlock H0 — round-1 post-fold verification

**Verdict: GREEN.** 0 Critical / 0 Important / 0 Minor / 0 Nit.

Reviewer: independent sonnet fold-verification reviewer, brief
`design/agent-briefs/hashlock-H0-post-impl-r1-fold-verification-brief.md`.
Tips verified: mnemonic-engrave `hashlock-h0` **95cd48a**, seedhammer fork
`hashlock-h0` **83fbc17**, checked against the "Post-review fold" section of
`design/agent-reports/hashlock-H0-implementation-report.md` (engrave branch).

Own copies (removed after the run): engrave
`git worktree add --detach me-worktrees/h0-verify3 95cd48a`,
`CARGO_TARGET_DIR=.tmp/h0-verify3-target`, `TMPDIR=.tmp`; fork
`git ls-files -z | tar` export into `.tmp/h0-verify3-fork`, Go 1.26.7 at
`.toolchain/go`. Neither branch worktree, neither repo, and nothing under
`.tmp/seedhammer-hashlock-h0` was written — every mutation applied to a copy,
observed, restored from a byte backup and `touch`ed; `git status --porcelain`
empty on the engrave worktree at the end, and the fork copy diffs from its
source only in two untracked build artifacts (`cmd/emu/emu.wasm`,
`wasm_exec.js`) that `git ls-files` never copied in the first place.

## The one question

> Did the fold fix the Critical and both Importants — FIXED / PARTIAL / NOT
> FIXED / DECLINED-with-reason — with a test that can FAIL for each, without a
> regression or a false claim of its own?

**FIXED, FIXED, FIXED.** All three defects are closed, each by a test that
demonstrably fails on the reintroduced defect, no regression, and every count
in the fold report's "Post-review fold" section reproduces exactly.

## Findings

| # | finding | verdict | evidence |
| --- | --- | --- | --- |
| C-1 | Recover reaches Confirm/Engrave for a preimage recovered from shares | **FIXED** | guard now at the TOP of `engraveCodex32`'s `for` body (fork `gui/codex32_polish.go:218-235`); `TestEngraveCodex32RefusesAPreimageRecoveredFromShares` passes at the tip; moving the guard back outside the loop reproduces the exact pre-fold failure verbatim (below); the two older door tests stay green; grep of `scan = `, `engraveCodex32(`, `backupSeedStringFlow(` finds exactly one reassignment (`scan = secret`) and one caller each, all inside the guarded loop — `unlockEngraveCodex32` is a separate, already-guarded, non-looping function (its own `IsPreimage` check, no reassignment) |
| I-1 | the 0x03/33-byte plain-BIP-93 vs. preimage-plate collision, and the records that called it impossible | **FIXED** | corpus 13 rows (2 both/6 device-only/5 neither, both counted directly), sha256 `bb703f60…` identical byte-for-byte on both copies and matching both pinned literals; new row `bip93-plain-33-byte-payload-0x03` is not-secret under both `sysw.Classify` and `seal.Classify`; the `0x31` control row (`ms10testsxy0qq…`) stays a secret under both; flipping `device_admits`→true fails the fork test with the exact quoted message, and (after re-pinning the sha to match, since the pin gate runs first) flipping `host_admits`→true fails the Rust test at `codex32_seam.rs:54`, while flipping `device_admits`→true on the Rust side passes trivially, exactly as the report explains structurally; doc comment (`codex32/mspayload.go`) and F-472's correction both state the narrowed claim (16/20/24/28/32-byte seeds and every share untouched) with no residual "33-byte plain seeds are untouched" claim anywhere; `record_corpus` 38 records, 6/6 tests green |
| I-2 | `me seal` echoed the raw codec error instead of naming the hashlock kind | **FIXED** | `printf '%s\n' "$PLATE" \| me seal --seal-secret --out a.uf2` → exit 4, stderr names "hashlock PREIMAGE plate", no "reserved-prefix byte", no plate text; same for the entr-32-then-plate multi-record case; `me sysw pack` on the plate still shows its (pre-existing) "record 0 (records count from 0) is a hashlock PREIMAGE plate" message, on a separate call site (`sysw/mod.rs:198`) untouched by this fold; mutating `validate_record`'s `if preimage_plate(s)` arm to never fire fails exactly the two I-2 tests (`a_preimage_plate_is_not_a_seed_record`, `seal_names_a_preimage_plate_and_never_echoes_it`) and leaves `sysw_pack_names_a_preimage_plate_and_never_echoes_it` green — matching the fold report's own R4 mutation row ("both I-2 tests") rather than all three |

## Executed checks, with output

**C-1 — mutation (guard moved back outside the `for` body):**
```
--- FAIL: TestEngraveCodex32RefusesAPreimageRecoveredFromShares (0.00s)
    codex32_polish_test.go:534: a preimage recovered from shares reached the SECRET confirm screen: "ConfirmCodex32SecretidHASHUnsharedsecret(S)75chars"
```
byte-identical to the fold report's quoted reproduction. Reverted; all three
door tests (`TestEngraveCodex32RefusesAPreimagePlate`,
`TestScanDoesNotHandAPreimagePlateToEngrave`,
`TestEngraveCodex32RefusesAPreimageRecoveredFromShares`) pass again.

**C-1 — reassignment/re-entry sweep:**
```
$ grep -rn "scan = " gui/*.go
gui/codex32_polish.go:244:  scan = secret // recovered unshared secret; loop re-confirms it
$ grep -rn "engraveCodex32(" gui/*.go
gui/codex32_polish.go:218:func engraveCodex32(...)
gui/gui.go:2556:  return engraveCodex32(ctx, th, scan)
$ grep -rn "backupSeedStringFlow(" gui/*.go
gui/codex32_polish.go:249:  backupSeedStringFlow(ctx, th, s)   # inside the guarded loop
gui/gui.go:2814:func backupSeedStringFlow(...)
```
One reassignment, one caller of `engraveCodex32`, one caller of
`backupSeedStringFlow` (from inside the guarded loop). `unlockEngraveCodex32`
(`gui/unlock_session.go`) is a separate single-shot function with its own
`IsPreimage` guard and no loop that reassigns its argument — already covered by
round 0's G7 mutation, not a new path.

**I-1 — corpus:**
```
$ sha256sum crates/me-cli/testdata/codex32_seam_vectors.json sysw/testdata/codex32_seam_vectors.json
bb703f608215bb00ccc677de4a282772016e774dd2d1d0f5c828ea38f5eac78b  crates/me-cli/testdata/codex32_seam_vectors.json
bb703f608215bb00ccc677de4a282772016e774dd2d1d0f5c828ea38f5eac78b  sysw/testdata/codex32_seam_vectors.json
```
13 rows: 2 both / 6 device-only / 5 neither (counted from the JSON directly).
Both pinned `SEAM_VECTORS_SHA256`/`seamVectorsSHA256` literals equal the
measured hash.

New row, both classifiers:
```
sysw.Classify: 0 == ClassCodex32Secret? false
seal.Classify: unknown format == ClassCodex32Secret? false
```
`0x31` control row (`ms10testsxy0qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5dayejmh0wrfk`):
```
IsPreimage: false
sysw.Classify: 2 == ClassCodex32Secret? true
seal.Classify: codex32 secret == ClassCodex32Secret? true
```

Fork flip, `device_admits`→true (re-pinning the sha so the assertion under
test runs rather than the pin gate):
```
codex32_seam_test.go:66: bip93-plain-33-byte-payload-0x03: device admits = false, want true (Classify = 0)
```
Rust flip, `host_admits`→true:
```
thread '...' panicked at crates/me-cli/tests/codex32_seam.rs:54:9:
bip93-plain-33-byte-payload-0x03: the HOST admits what the DEVICE refuses
```
Rust flip, `device_admits`→true: `test ... ok` (passes trivially — the safe
implication `host<=device` is satisfied by `false/true`, exactly as the fold
report explains; this is structural, not a defect).

Doc comment (`codex32/mspayload.go:81-91`) states "16-, 20-, 24-, 28- and
32-byte seeds are untouched, and so is every share" — no claim that 33-byte
plain seeds are untouched. F-472's correction paragraph in
`design/FOLLOWUPS.md` states the same narrowed inventory. No stale claim found.

`record_corpus_pre_s2.json`: 38 records (measured directly). `record_corpus`
tests:
```
running 6 tests
test the_capture_covers_every_class_s2_must_not_move ... ok
test the_capture_is_the_whole_corpus ... ok
test the_descriptor_gate_stays_shut_on_every_corpus_document ... ok
test every_corpus_record_classifies_as_it_did_before_s2 ... ok
test the_descriptor_gate_stays_shut_on_every_corpus_record ... ok
test expect_resolves_before_the_descriptor_gate ... ok
test result: ok. 6 passed; 0 failed
```

**I-2:**
```
$ printf '%s\n' "$PLATE" | me seal --seal-secret --out a.uf2
me: this record is a hashlock PREIMAGE plate (kind 0x03), not a seed record; this container
    cannot place one yet. A preimage backs a hashlock spend path, not a wallet -- keep it with
    the policy it unlocks, and do not re-encode it as entropy.
exit=4
```
No "reserved-prefix byte", no plate substring, in either the single-record or
the entr-32-then-plate multi-record case (both checked; both exit 4, same
message). `me sysw pack "$PLATE"` unchanged: still names the kind via its own,
separate `unknown_reason`/`preimage_plate` call site
(`crates/me-cli/src/sysw/mod.rs:198`).

Mutation (`validate_record`'s `if preimage_plate(s)` forced false):
```
test a_preimage_plate_is_not_a_seed_record ... FAILED
  refused as Invalid("reserved-prefix byte was 0x03, expected 0x00"), not as a preimage plate
test seal_names_a_preimage_plate_and_never_echoes_it ... FAILED
  stderr does not name the kind:
  me: invalid record: reserved-prefix byte was 0x03, expected 0x00
test sysw_pack_names_a_preimage_plate_and_never_echoes_it ... ok
```
Exactly the fold report's own R4 row ("both I-2 tests"), not the sysw one —
correct, since `me sysw pack`'s diagnosis is a separate call site. Reverted;
all 3 tests in the file pass again.

## Regressions

Engrave, whole crate, nextest, clean tree:
```
Summary [0.563s] 617 tests run: 614 passed, 3 failed, 2 skipped
FAIL mnemonic-engrave::history_purge editing_the_file_alone_is_the_trap_the_message_warns_about
FAIL mnemonic-engrave::history_purge the_harness_records_history_at_all
FAIL mnemonic-engrave::history_purge the_emitted_zsh_recipe_actually_purges_the_entry
```
Only the box-local `history_purge` trio fails (secret-handling never gates;
matches the report). `cargo clippy --all-targets --locked -- -D warnings`:
exit 101, same single pre-existing lint at `crates/me-cli/src/sysw/composer_records.rs:114`
(`git blame` shows that line last touched by `d01e7a1`, 2026-09-02, unrelated
to this fold). `cargo fmt --check`: exit 0.

Fork:
```
vet exit=0
ok  seedhammer.com/codex32  0.003s
ok  seedhammer.com/sysw     0.040s
ok  seedhammer.com/seal     12.506s
```
Targeted gui filter (`TestEngraveCodex32|TestScan|TestUnlockEngrave|TestConfirmCodex32|TestClassify|TestAdmit|TestRecover`):
29 subtests, 0 failures, `PASS`.
`gui-shard-test.sh ./gui/ 24`:
```
1205 top-level tests
partition verified exhaustive: 1205 == 1205
RESULT: ok -- all 1205 tests ran across 24 shards
```
`gofmt -l` on the six Go files the fold touched (`codex32/mspayload.go`,
`codex32/mspayload_test.go`, `gui/codex32_polish.go`,
`gui/codex32_polish_test.go`, `gui/unlock_session.go`,
`sysw/codex32_seam_test.go`): empty, exit 0.

Firmware, fold tip:
```
   code    data     bss |   flash     ram
1551336   31796   31004 | 1583132   62800
```
`1,583,132 / 62,800` — an exact match to the fold report's own re-measurement
(delta +32 B flash / +0 RAM vs. pre-fold `14afdff`), well inside "a few
hundred bytes".

## The report's own counts

Every count checked against a direct measurement: seam-vector shapes (13:
2/6/5), corpus size (38), `record_corpus` (6/6), engrave suite (617/614/3
box-local/2 skipped), gui shard total (1205, exhaustive), firmware
(1,583,132/62,800), clippy's single pre-existing lint site, and the 6-commit
finding→commit table (verified against `git log 265dc8e..95cd48a` and
`git log 14afdff..83fbc17` on the fork). All reproduce exactly; no false
count found.

## Severity

No Criticals, no Importants: every FIXED verdict is backed by a test that
demonstrably fails on the reintroduced defect, no regression anywhere, and
every quoted count/hash reproduces. Nothing rises even to Minor — the one
apparent wrinkle (flipping `device_admits` on the Rust side "passing" and
needing a sha re-pin to exercise the Go-side flip past its own pin gate) is
the fold report's own documented, structurally-correct behavior, reproduced
here rather than contradicted.

## GREEN / NOT GREEN

**GREEN.** 0 Critical / 0 Important / 0 Minor / 0 Nit. No further fold or
re-review is warranted on this round's one question.
