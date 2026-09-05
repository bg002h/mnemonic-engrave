# Hashlock H0 — implementation report (the ONE implementer)

**Plan:** `design/IMPLEMENTATION_PLAN_hashlock_H0_reader_guards.md` (STATUS R0
GREEN). **Spec:** `mnemonic-secret/design/SPEC_ms_hashlock.md` §1, §9.
**Brief:** `design/agent-briefs/hashlock-H0-implementer-brief.md`.

**Branch tips (nothing pushed; no commit on `master` or `main`):**

| repo | worktree | branch | tip |
| --- | --- | --- | --- |
| mnemonic-engrave | `/scratch/code/shibboleth/me-worktrees/hashlock-h0` | `hashlock-h0` | `19b2a58` (report commit on top) |
| seedhammer fork | `/scratch/code/shibboleth/.tmp/seedhammer-hashlock-h0` | `hashlock-h0` | `14afdff` |

Base revisions: engrave `hashlock-h0` branched from `master` = `60b4cfb`; fork
`hashlock-h0` branched from `main` = `839fa5a`. Every anchor the plan cites
resolved **exactly once** at those tips — each edit was applied by a script that
asserted `count == 1` on the quoted anchor text before writing, so no anchor had
drifted since the plan's baselines.

**`master` moved during implementation.** `hashlock-h0` is based on engrave
`master` = `60b4cfb`; while this work ran, the controller advanced `master` to
`4973140` (three continuity/report commits, none touching `crates/me-cli`).
`git merge-base master hashlock-h0` is `60b4cfb`, and `git log master..hashlock-h0`
lists exactly this task's three commits — no commit of mine is on `master`, and
fork `main` is unmoved at `839fa5aa`. The controller should expect to merge, not
fast-forward.

Commits, in order:

| # | repo | SHA | subject |
| --- | --- | --- | --- |
| 1 | engrave | `be72e75` | seam corpus: preimage-plate rows … + host pin + preimage-plate diagnosis (hashlock H0) |
| 2 | fork | `14afdff` | hashlock H0: a kind-0x03 preimage single is inert … |
| 3 | engrave | `19b2a58` | records: hashlock H0 follow-ups F-472/F-473/F-474 + CHANGELOG (unreleased) |
| 4 | engrave | this report | — |

---

## Task 1 — the corpus rows and the host half (mnemonic-engrave, `be72e75`)

Files changed (7): `crates/me-cli/testdata/codex32_seam_vectors.json`,
`crates/me-cli/testdata/record_corpus_pre_s2.json`,
`crates/me-cli/tests/codex32_seam.rs`,
`crates/me-cli/tests/preimage_plate_is_not_a_seed.rs` (new),
`crates/me-cli/src/seal/record.rs`, `crates/me-cli/src/sysw/mod.rs`,
`crates/me-cli/src/main.rs`. 186 insertions, 1 deletion.

### Step 1 — the four rows, and the sha256

The four row bodies were extracted **byte-for-byte out of the plan file** (`sed
-n '178,209p'`) rather than retyped, so the corpus text is the reviewed text.

```
$ sha256sum crates/me-cli/testdata/codex32_seam_vectors.json
f1f2fa6bbbf27e3697ee496636de49be2f25787deff7b3bc4a2c5e16854e391c
```

That is the plan's Global-Constraint hash **exactly**, first try — the rows are
byte-identical to the reviewed text. Row split, measured from the file:

```
12 ['preimage-plate-0x03', 'bip93-plain-payload-0x03', 'bip93-share-payload-0x03', 'preimage-shape-entr-id']
both 2 device-only 6 neither 4
```

which is the plan's "12 rows: 2 both / 6 device-only / 4 neither".

### Steps 1b–2 — RED, verbatim

Run once at the post-Step-1 tree, captured to
`/scratch/code/shibboleth/.tmp/h0-red-step1.txt`:

```
thread 'the_host_never_admits_what_the_device_would_refuse' panicked at crates/me-cli/tests/codex32_seam.rs:33:5:
assertion `left == right` failed: testdata/codex32_seam_vectors.json is not the file the fork's copy is pinned to; re-pin BOTH literals
  left: "f1f2fa6bbbf27e3697ee496636de49be2f25787deff7b3bc4a2c5e16854e391c"
 right: "3d53ef88a474f02c15aa60a839f4a31071598a26c853463122a847515926eb6a"

thread 'the_capture_is_the_whole_corpus' panicked at crates/me-cli/tests/record_corpus.rs:137:5:
assertion `left == right` failed: testdata/record_corpus_pre_s2.json is not the enumerated corpus

thread 'every_corpus_record_classifies_as_it_did_before_s2' panicked at crates/me-cli/tests/record_corpus.rs:155:5:
assertion `left == right` failed: class assertions run
  left: 33
 right: 37

Summary 7 tests run: 3 passed, 4 failed
```

The failure text is the plan's Step 2 expectation word for word, and the
record-capture reds are the three the plan's Step 1b predicts (the third is
`the_descriptor_gate_stays_shut_on_every_corpus_record`, same 33-vs-37 shape).

### Steps 1b/3/4 — GREEN

The four capture entries were inserted directly after the
`codex32_seam/bip93-bad-checksum` entry (its trailing `},` kept), the pin
re-written to the measured hash, and the same two binaries re-run:

```
Summary [   0.005s] 7 tests run: 7 passed, 0 skipped
```

`record_corpus` is 6/6 with **37** records (measured from the file:
`len(records) == 37`), `codex32_seam` 1/1. Matches the plan's Step 1b and
Step 4 expectations.

### Steps 5–6 — the pin test, and its RED

`crates/me-cli/tests/preimage_plate_is_not_a_seed.rs` was created from the
plan's block verbatim. First run
(`/scratch/code/shibboleth/.tmp/h0-step6-red.txt`):

```
PASS (1/2) a_preimage_plate_is_not_a_seed_record
FAIL (2/2) sysw_pack_names_a_preimage_plate_and_never_echoes_it

thread 'sysw_pack_names_a_preimage_plate_and_never_echoes_it' panicked at crates/me-cli/tests/preimage_plate_is_not_a_seed.rs:54:5:
stderr does not name the kind:
me: record 0 (records count from 0) is a VALID BIP-93 codex32 string — the checksum is good — but not a constellation `ms1` record, so this container cannot place it.
      `ms1` is a two-gate PROFILE over BIP-93: the whole string must be [50, 56, 62, 69, 75] characters (entropy) or [51, 58, 64, 70, 77] (mnemonic), and the 4-character id must be `entr`. This one is 75 characters.
      Plain BIP-93 secrets are 48 or 74 characters and BIP-93 SHARES carry their own id, so neither is a constellation record — re-encode the entropy as `ms1` rather than editing the string.
```

This is fidelity I-3 reproduced live: the message lists a set **containing 75**,
then gives 75 as the reason, calls a constellation record not one, and tells the
operator to re-encode a hashlock preimage as seed entropy.

### Steps 7–8 — the diagnosis, and its two mutations

`seal::record::preimage_plate`, `UnknownReason::PreimagePlate`, the arm ordered
**before** the profile arm, the `main.rs` Display text, and the unit test — all
applied from the plan's blocks verbatim. Then:

```
$ cargo fmt -p mnemonic-engrave && cargo nextest run --locked -p mnemonic-engrave -E 'test(/preimage|codex32_seam|outside_the_profile/)'
Summary [   0.019s] 5 tests run: 5 passed, 613 skipped
```

5 PASS, as the plan measured (the two integration tests, the two existing
profile tests, the new unit test).

**MUTATION 1 — swap the two arms in `unknown_reason`.** Re-run (mutate, observe,
restore from backup, `touch`):

```
FAIL mnemonic-engrave sysw::tests::a_preimage_plate_is_named_not_misdiagnosed
  left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))
 right: Err(Unclassifiable(0, PreimagePlate))
```

Exactly the plan's claim: with the arms swapped the profile arm claims the plate
and reports 75.

**MUTATION 2 — `if s.starts_with("ms10hash") { return Ok(RecordKind::Ms); }`
before `record.rs`'s `ms_codec::decode(s)`.** Both witnesses fired:

```
thread 'a_preimage_plate_is_not_a_seed_record' panicked at crates/me-cli/tests/preimage_plate_is_not_a_seed.rs:23:21:
validate_record admitted a 0x03 preimage plate as Ms

thread 'the_host_never_admits_what_the_device_would_refuse' panicked at crates/me-cli/tests/codex32_seam.rs:58:9:
assertion `left == right` failed: preimage-plate-0x03: host verdict
```

Both mutations were reverted by restoring a backup **and `touch`ing** the file
(the plan's Global Constraint), and the post-revert runs are green below.

### Step 9 — whole crate

See "Final gates" below; 616 run, 613 passed, the 3 named `history_purge`
failures only.

---

## Task 2 — the device guard (seedhammer fork, `14afdff`)

Files changed (12): `codex32/mspayload.go`, `codex32/mspayload_test.go`,
`sysw/classify.go`, `sysw/testdata/codex32_seam_vectors.json`,
`sysw/codex32_seam_test.go`, `seal/record.go`, `seal/record_test.go`,
`gui/scan.go`, `gui/codex32_polish.go`, `gui/codex32_polish_test.go`,
`gui/unlock_session.go`, `gui/unlock_session_test.go`. 245 insertions, 10
deletions. `git commit -s` (DCO), author Brian Goss <goss.brian@gmail.com>.

### Step 1 — vendor, re-pin, RED

```
$ sha256sum sysw/testdata/codex32_seam_vectors.json
f1f2fa6bbbf27e3697ee496636de49be2f25787deff7b3bc4a2c5e16854e391c
```

Byte-identical to the primary. Then, with the pin updated and no guard yet
(`/scratch/code/shibboleth/.tmp/h0-fork-step1-red.txt`):

```
--- FAIL: TestCodex32SeamDeviceAdmitsEverythingTheHostDoes (0.00s)
    codex32_seam_test.go:66: preimage-plate-0x03: device admits = true, want false (Classify = 2)
    codex32_seam_test.go:66: preimage-shape-entr-id: device admits = true, want false (Classify = 2)
```

**Exactly the two rows the plan names, and only those two** — the two
`0x03`-leading `device_admits: true` control rows passed already and kept
passing. The spec's reader-table measurement as a failing test.

### Steps 2–3 — `IsPreimage`, and its four mutations

Predicate and six-population table applied verbatim; `gofmt` clean;
`go test -run TestIsPreimageReadsThePrefixByteOnly ./codex32/` → `ok`.

All four `MUTATION:` claims re-run once each, each restored from a backup
(`/scratch/code/shibboleth/.tmp/h0-fork-step3-mutations.txt`) — **each failed on
exactly one row, and a different row each time**, which is what makes the table
a real discriminator rather than a set of redundant cases:

| mutation | failing line, verbatim |
| --- | --- |
| drop `!f.Unshared` | `mspayload_test.go:115: IsPreimage(bip93-share-payload-0x03 (a 2-of-N share beginning 0x03)) = true, want false` |
| `len(d) > 0` for `len(d) == 33` | `mspayload_test.go:115: IsPreimage(bip93-plain-payload-0x03 (16-byte seed beginning 0x03)) = true, want false` |
| `d[0] != msPrefixEntr` | `mspayload_test.go:115: IsPreimage(bip93-plain-33-byte-payload-0x31 (unshared, 33 bytes, first byte 0x31)) = true, want false` |
| key on the id `hash` | `mspayload_test.go:115: IsPreimage(preimage-shape-entr-id (unshared, 33 bytes, 0x03, id entr)) = false, want true` |

`DecodeMS1` is unchanged and still returns `errMSBadPrefix` on the plate (the
test's last assertion).

### Step 4 — `isStrictMs1`

Guard applied; `go test -count=1 ./sysw/` → `ok  seedhammer.com/sysw 0.142s`.

**MUTATION — drop `!codex32.IsPreimage(c)`** (`_ = c; return err == nil`):

```
--- FAIL: TestCodex32SeamDeviceAdmitsEverythingTheHostDoes (0.00s)
    codex32_seam_test.go:66: preimage-plate-0x03: device admits = true, want false (Classify = 2)
    codex32_seam_test.go:66: preimage-shape-entr-id: device admits = true, want false (Classify = 2)
```

The plan's claim — "fails on exactly the Step 1 lines" — holds byte for byte.
Restored; re-run `ok`.

### Steps 5–6 — `seal.Classify` and the seal tests

Applied; `go test -count=1 ./seal/` → `ok  seedhammer.com/seal 12.816s`.

**MUTATION — drop `!codex32.IsPreimage(c)` from `Classify`:**

```
--- FAIL: TestClassifyMirrorsScanBranchOrder (0.00s)
    record_test.go:419: Classify("ms10hashsqw46h2at4w46h2a") = codex32 secret, want unknown format
--- FAIL: TestAdmitSectionRefusesAPreimagePlateAsUnknown (0.00s)
    record_test.go:467: AdmitSection(preimage plate, encrypted) err = <nil>, want ErrRecordNotPermitted
```

Both verbatim as the plan claims. Restored; re-run `ok`.

### Step 7 — the sealed engrave path

Guard + harness twin + test applied; `go test -run 'TestUnlock' ./gui/` → `ok`.

**MUTATION — drop the `IsPreimage` check in `unlockEngraveCodex32`:**

```
--- FAIL: TestUnlockEngraveCodex32RefusesAPreimagePlate (0.11s)
    unlock_session_test.go:1320: never reached "hashlock preimage"; last frame "Insertablankplateandclosethelock.Holdbuttontostarttheengravingprocess.Theprocessisloud,usehearingprotection.EngravePlate"
```

The device, handed a preimage plate through this path, would reach the engrave
screen. (The plan quoted the same frame with spaces and an elision for
readability; the harness's extracted frame text is space-stripped. Same frame.)

### Step 8 — the two direct doors

(a) `gui/scan.go`'s `codex32.New` arm narrowed with `!codex32.IsPreimage(s)`;
(b) the named refusal at the `engraveCodex32` choke point; both door tests
appended with `"errors"` added to the import block.

```
$ go test -count=1 -v -run 'TestEngraveCodex32RefusesAPreimagePlate|TestScanDoesNotHandAPreimagePlateToEngrave' ./gui/
--- PASS: TestEngraveCodex32RefusesAPreimagePlate (0.00s)
--- PASS: TestScanDoesNotHandAPreimagePlateToEngrave (0.00s)
ok  	seedhammer.com/gui	0.044s
```

(Run with `-v` deliberately: a `-run` filter that matches nothing also prints
`ok`, so the two `--- PASS` lines are the evidence the tests actually executed.)

**MUTATION — remove (b), the `engraveCodex32` guard:**

```
--- FAIL: TestEngraveCodex32RefusesAPreimagePlate (0.12s)
    codex32_polish_test.go:423: never reached "hashlock preimage"; last frame "ConfirmCodex32SecretidHASHUnsharedsecret(S)75chars"
```

The plate confirmed as a secret titled `HASH`, one button from the cut — the
plan's claim reproduced exactly.

**MUTATION — remove (a), the `scan.go` arm's guard:**

```
--- FAIL: TestScanDoesNotHandAPreimagePlateToEngrave (0.00s)
    codex32_polish_test.go:445: Scan(preimage plate) = codex32.String, <nil>; want errScanUnknownFormat
```

Both restored; re-run `ok`.

---

## Task 3 Step 1 — firmware size (Steps 2–3 are NOT mine)

Measured from the fork worktree at tip `14afdff`, nix on `PATH`
(`/scratch/code/shibboleth/.tmp/h0-fw-size.txt`):

```
$ nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller
   code    data     bss |   flash     ram
1551304   31796   31004 | 1583100   62800
```

| | flash | ram |
| --- | --- | --- |
| fork `main` `839fa5aa` (recorded baseline) | 1,582,628 | 62,800 |
| `hashlock-h0` `14afdff` (measured) | **1,583,100** | **62,800** |
| delta | **+472 B** | **0** |

Within the plan's "a few hundred bytes", and RAM is unchanged at exactly the
expected 62,800.

**Task 3 Step 2 (the emulator walk) and Step 3 (merge + flash) were NOT run** —
the brief reserves them. Nothing has been merged to fork `main`, nothing
flashed.

---

## Task 4 — records (engrave `19b2a58`)

Three follow-ups filed, continuing the file's sequence (previous max `F-471`):

- **F-472** `device-full-constellation-profile-convergence` — should the device
  refuse plain BIP-93 (48/74) and foreign-id shares as `me` does? Owning phase:
  **none — an operator decision**.
- **F-473** `ms-codec-0.8-bump-needs-a-preimage-refusal-arm` — owning phase
  **H1b**, and the gating one. It carries BOTH halves: `validate_record` needs a
  `Payload::Preimage` refusal, *and* `seal::record::preimage_plate` must be
  re-pointed, because it asks for `ReservedPrefixViolation { got: 3 }` **by
  name** and will silently answer `false` at `0.8`.
- **F-474** `unlock-kdf-names-the-refused-record` — owning phase **H2**.

`crates/me-cli/CHANGELOG.md` `[Unreleased]` gained an `### Added` section
naming the H0 diagnosis, the four corpus rows and the 33 → 37 capture. **The
fork keeps no CHANGELOG** (checked: no `CHANGELOG*` at its root), so the plan's
"Fork CHANGELOG if it keeps one" is a no-op.

**Not mine, not done:** the post-implementation review (Task 4 bullet 3), the
continuity entry, and any push.

---

## Deviations from the plan

Five, all minor; none changes behaviour.

1. **The vendoring path.** The plan's Task 2 Step 1 says
   `cp ../mnemonic-engrave/crates/me-cli/testdata/codex32_seam_vectors.json …`.
   The fork worktree lives at `/scratch/code/shibboleth/.tmp/seedhammer-hashlock-h0`,
   so `../mnemonic-engrave` does not resolve. I used the absolute path to the
   engrave **worktree's** just-edited copy
   (`/scratch/code/shibboleth/me-worktrees/hashlock-h0/crates/me-cli/testdata/…`)
   — i.e. the Rust primary as edited in Task 1, not the untouched `master` copy.
   Verified by hash: both files are `f1f2fa6b…391c`. The brief anticipated this
   class for the shard script.
2. **`gofmt` realignment in `seal/record_test.go`.** The plan's branch-order row
   `{sealPreimagePlate, ClassUnknown}, // H0: …` is longer than its neighbours,
   so `gofmt` re-aligns the two preceding rows' trailing comments:
   `{d.Public[0], ClassMDMK}, // mk1` and `{d.Public[2], ClassMDMK}, // md1`
   each gain padding spaces. I ran `gofmt -w seal/record_test.go`. Recorded
   because the commit touches two lines the plan did not name; the change is
   whitespace only.
3. **Trailer order on the fork commit.** `git commit -s` appends `Signed-off-by`
   *after* the message, which would have put it after the brief's two required
   trailers. I amended the commit so the message ends with
   `Co-Authored-By:` + `Claude-Session:`, with `Signed-off-by:` immediately
   before them. Both the DCO requirement and the brief's "every commit message
   ends with" are satisfied. (This is why the fork tip is `14afdff` and not the
   pre-amend `bc81a71`.)
4. **Step 7's mutation frame text.** The plan quotes the last frame with spaces
   and a `...` elision; the harness renders frame text space-stripped. Content
   identical — noting it so a future reader does not read the difference as a
   changed screen.
5. **Clippy is red on this box, as the plan predicted, and NOT from this work.**
   `cargo clippy --locked -p mnemonic-engrave --all-targets -- -D warnings`
   fails on `manual implementation of .is_multiple_of()` at
   `crates/me-cli/src/sysw/composer_records.rs:114` — a file this task never
   touched, flagged by the local nightly (`clippy 0.1.97 (52b6e2c208
   2026-04-27)`, `cargo 1.97.0-nightly`). The plan names this exact lint at this
   exact line as the local nightly's, green in CI at `917d4e3`. So Task 1
   Step 9's "green under CI's toolchain" is **unverified locally for clippy**;
   `nextest` and `fmt --check` are verified. Flagging rather than silently
   treating a red gate as green.

Nothing was silently diverged. Every plan block was applied by a script that
asserted its anchor text occurred **exactly once** before writing, and the four
JSON/Rust/Go bodies were extracted straight out of the plan file rather than
retyped.

---

## Final gates — verbatim tails, at the tips reported

### mnemonic-engrave, at `19b2a58` (clean tree)

Captured once to `/scratch/code/shibboleth/.tmp/h0-final-engrave-gate.txt`:

```
     Summary [   0.445s] 616 tests run: 613 passed, 3 failed, 2 skipped
        FAIL [   0.004s] (433/616) mnemonic-engrave::history_purge editing_the_file_alone_is_the_trap_the_message_warns_about
        FAIL [   0.004s] (440/616) mnemonic-engrave::history_purge the_harness_records_history_at_all
        FAIL [   0.005s] (441/616) mnemonic-engrave::history_purge the_emitted_zsh_recipe_actually_purges_the_entry
error: test run failed
nextest exit=100
    Checking mnemonic-engrave v0.8.0 (/scratch/code/shibboleth/me-worktrees/hashlock-h0/crates/me-cli)
error: manual implementation of `.is_multiple_of()`
   --> crates/me-cli/src/sysw/composer_records.rs:114:8
    |
114 |     if s.len() % 2 != 0
    |        ^^^^^^^^^^^^^^^^ help: replace with: `!s.len().is_multiple_of(2)`
    |
    = note: `-D clippy::manual-is-multiple-of` implied by `-D warnings`
error: could not compile `mnemonic-engrave` (lib) due to 1 previous error
error: could not compile `mnemonic-engrave` (lib test) due to 1 previous error
clippy exit=101
fmt exit=0
```

**616 run, 613 passed.** All three failures are the `history_purge` trio the
plan names as box-local, and I confirmed the cause mechanically rather than
taking the plan's word for it:

```
$ ls -la /usr/bin/zsh
ls: cannot access '/usr/bin/zsh': No such file or directory
```

Clippy: deviation 5 above. `fmt --check`: clean (exit 0).

### seedhammer fork, at `14afdff` (clean tree)

Captured once to `/scratch/code/shibboleth/.tmp/h0-final-fork-gate.txt`:

```
=== go vet ./codex32/ ./sysw/ ./seal/ ===
vet exit=0
=== go test -count=1 ./codex32/ ./sysw/ ./seal/ ===
ok  	seedhammer.com/codex32	0.003s
ok  	seedhammer.com/sysw	0.147s
ok  	seedhammer.com/seal	12.909s
test exit=0
=== gui-shard-test.sh ./gui/ 24 ===
=== enumerating tests in ./gui/ ===
    1204 top-level tests
    partition verified exhaustive: 1204 == 1204
=== running 24 shards in parallel (timeout 20m each) ===
=== wall: 52s ===
RESULT: ok -- all 1204 tests ran across 24 shards
shard exit=0
```

Fully green: `go vet` clean on the three packages, all three `ok`, and **all
1204** `gui` tests ran with the partition asserted exhaustive (so no test was
silently dropped).

`gofmt -l gui/` reports `gui/transaction.go`,
`gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go` — I ran the
same command in the untouched `main` checkout and it prints the identical three
files, so they are pre-existing and not this task's. Every file this task
touched is `gofmt`-clean. (`go vet ./gui/` was not run: the plan records two
pre-existing `testing.ArtifactDir requires go1.26 or later` complaints there on
`main` today, and the plan's Step 9 command deliberately vets only the three
packages.)

---

## Left undone, and why

| item | why |
| --- | --- |
| Task 3 Step 2 — the emulator walk | Reserved by the brief; not the implementer's. |
| Task 3 Step 3 — merge to fork `main`, flash | Reserved; and the flash needs the operator's word. |
| Task 4 — post-implementation review | The controller's, per the brief and the plan. |
| Task 4 — continuity entry, memory, pushes | The controller's. **Nothing was pushed.** |
| Clippy green under CI's toolchain | Cannot be verified on this box (deviation 5). CI at `917d4e3` is the plan's authority. |

Both `hashlock-h0` branches exist only locally, in their own worktrees, with
clean trees at the tips reported above.
