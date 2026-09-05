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

---

# Post-review fold

The post-implementation review (`design/agent-reports/hashlock-H0-post-impl.md`,
engrave master `87771f6`) returned **NOT GREEN — 1C / 2I / 4M / 1N** against
engrave `265dc8e` and fork `14afdff`. All eight findings are folded below, one
commit per finding or tightly related pair, on the same two branches. **Every
defect was reproduced on the branch tip before it was fixed** — the review's
suggested fixes were treated as suggestions, and the reproductions are quoted
per finding.

The fork branch had gained one controller commit, `45f3d4c` (the emulator walk
`cmd/emu/walk_h0_preimage.js`); this fold builds on top of it and never touches
that file.

## Finding → commit → proof

| finding | sev | repo | commit | proof it is closed |
| --- | --- | --- | --- | --- |
| C-1 Recover reaches Confirm/Engrave | **C** | fork | `52336b0` | new test drives the real Recover screens; mutation (guard back outside the `for`) reaches `Confirm Codex32 Secret` |
| I-1 / M-2 the `0x03`/33-byte collision | **I/M** | engrave | `8018873` | corpus row + capture 37→38; `host_admits` flip fails `host <= device` |
| I-1 / M-2 device half + the words | **I/M** | fork | `72506fb` | corpus re-vendored `cmp`-identical; `device_admits` flip fails with the row named |
| I-2 / N-1 `me seal`'s raw codec error | **I/N** | engrave | `2089282` | `me seal` now names the kind; mutation restores the raw string and kills both tests |
| M-1 / M-4 doc comments and "48 or 74" | **M** | engrave | `6e84280` | wording only; 77-char case measured and described |
| M-3 godoc says NFC, is keypad | **M** | fork | `83fbc17` | comment only; the C-1 test drives that branch with `runes()` |

### C-1 — the guard ran once, before a loop that reassigns what it guards

**Reproduced first.** The review's own construction regenerated with the fork's
`codex32.NewSeed` + `Interpolate` (2-of-N over a 33-byte payload beginning
`0x03`, id `hash`) produced **the review's exact secret**,
`ms12hashsqvqsyqcyq5rqwzqfpg9scrgwpugpzysnzs23v9ccrydpk8qarc0jq9get7tzc6sn5y`
— an independent confirmation, not a transcription. The new test then failed on
the shipped tip:

```
--- FAIL: TestEngraveCodex32RefusesAPreimageRecoveredFromShares (0.00s)
    codex32_polish_test.go:534: a preimage recovered from shares reached the SECRET confirm screen: "ConfirmCodex32SecretidHASHUnsharedsecret(S)75chars"
```

byte-identical to the frame the review reported.

**Fix:** the `IsPreimage` test moved to the **top of the `for` body**, so it
runs on the object a door handed in *and* on every `scan` the `codex32Recover`
arm manufactures. The comment that carried the false assumption ("Both doors …
end here") is replaced by one that states why the test must be inside the loop.

**The test drives the real screens** — Confirm Codex32 SHARE → Recover
(`Button2`) → share 2 on the keypad (`runes`) → OK (`Button3`) — and treats
reaching **either** `Confirm Codex32 Secret` **or** `Engrave Plate` as fatal,
rather than merely looking for the refusal afterwards. It also asserts its own
premise (both shares are *not* preimages, so no upstream door refuses them, and
their interpolation *is* one), so it cannot pass vacuously.

One deviation, recorded: the shares are driven **UPPERCASE**. The first attempt
used the lowercase canonical strings and failed with `"mismatchedtypeInvalid
share"` — `codex32` requires one consistent case across a set, and the keypad
uppercases what it types, so a lowercase share plus a typed one is a mismatched
HRP. Uppercase is the form the device actually holds; the comment in the test
says so.

**Mutation, re-run at the final tip:** with the guard back outside the `for`
body, `TestEngraveCodex32RefusesAPreimageRecoveredFromShares` fails on the frame
quoted above **while the older `TestEngraveCodex32RefusesAPreimagePlate` still
passes** — which is precisely why the defect survived the first round.

### I-1 / M-2 — the collision, and the records that called it impossible

**Reproduced on both sides first.** `ms10testsqvrsu9…` (id `test`, unshared,
33-byte payload, first byte `0x03`):

```
ms10testsqvrsu len=75 seedlen=33 seed[0]=0x3 id="test" thr=1 idx=s IsPreimage=true
ms10testsxy0qq len=75 seedlen=33 seed[0]=0x31 id="test" thr=1 idx=s IsPreimage=false
```

and the built `me` on the identical string:

```
me: record 0 (records count from 0) is a hashlock PREIMAGE plate (kind 0x03), not a seed record; …
```

while the `0x31` string is still "outside the profile". So the host refuses it
too — **convergence, not a device narrowing**, which is the controller's ruling
verified rather than assumed.

**Behaviour stands; the words changed.** The doc comment's "A plain BIP-93
secret whose seed begins 0x03 has a 16..32-byte payload and is untouched" is
replaced by a paragraph that states the collision plainly, its rate (~1 in 256
of 33-byte seeds), what is untouched (16/20/24/28/32-byte seeds, every share),
and why it is accepted (`me` refuses the same string; keying on the id would
engrave a mistagged **real** preimage as a seed — a refusal costs a re-encode, a
wrong cut exposes a spend secret). The `mspayload_test.go` mutation comment is
corrected in the same way, and **F-472's inventory** and the **CHANGELOG**, both
of which had inherited the falsehood, now say the device had already lost one
plain-BIP-93 population rather than none.

**The fact became a test.** New seam row `bip93-plain-33-byte-payload-0x03`
(`host_admits: false`, `device_admits: false`); the `0x31` row keeps
`device_admits: true`.

```
codex32_seam_vectors.json  12 -> 13 rows (2 both / 6 device-only / 5 neither)
record_corpus_pre_s2.json  37 -> 38 records, class Unknown, consult record-refusal
sha256                     f1f2fa6b…391c -> bb703f608215bb00ccc677de4a282772016e774dd2d1d0f5c828ea38f5eac78b
```

RED first on both sides: the host showed `37` vs `38` in three `record_corpus`
tests plus the pin mismatch; the fork showed the vendoring drift named
verbatim (`hashes to bb703f60…, not the pinned f1f2fa6b…`). Both literals
re-pinned; `cmp` confirms the two copies byte-identical.

**A correction to the fold brief, measured.** The brief expected the new row
flipped to `device_admits: true` to "fail the seam test on both sides". It fails
on the **device side only**, and that is structural rather than a defect: the
Rust test asserts the HOST verdict and the safe direction `host <= device`
(`codex32_seam.rs:54`), and a `false`/`true` row satisfies that implication
trivially — the crate cannot call Go's `sysw.Classify`. So the row is pinned on
each side by the field that side can measure, and both were re-run:

| flip | side | result |
| --- | --- | --- |
| `device_admits` → `true` | fork | `bip93-plain-33-byte-payload-0x03: device admits = false, want true (Classify = 0)` |
| `device_admits` → `true` | engrave | **passes** — cannot fail, by design (see above) |
| `host_admits` → `true` | engrave | fails at `codex32_seam.rs:54`, `the HOST admits what the DEVICE refuses` |

### I-2 / N-1 — the second host verb

**Reproduced first**, on the branch tip:

```
$ printf '%s\n' "$PLATE" | me seal --seal-secret --out a.uf2
exit 4
me: invalid record: reserved-prefix byte was 0x03, expected 0x00
```

**Fix:** a new `RecordError::PreimagePlate`, returned from `validate_record`
before a decode error is mapped to `RecordError::Invalid`. A separate variant
for the same reason `MsTooLong` is one: the record is intact and correctly
checksummed — it is the wrong KIND, not corrupt — and §6.4 renders every other
record failure as "payload unreadable". After:

```
me: this record is a hashlock PREIMAGE plate (kind 0x03), not a seed record; this container
    cannot place one yet. A preimage backs a hashlock spend path, not a wallet — keep it with
    the policy it unlocks, and do not re-encode it as entropy.
```

`me sysw pack` is unchanged and keeps its record index — verified by running
both verbs after the change, not inferred.

**N-1 closed in the same edit:** the pin test's `Err(_) => {}` now asserts
`Err(RecordError::PreimagePlate)` exactly.

**RED was made behavioural rather than a compile error** — the variant and its
`Display` were added *unwired* first, so the two tests failed on conduct:

```
refused as Invalid("reserved-prefix byte was 0x03, expected 0x00"), not as a preimage plate
stderr does not name the kind
```

**Mutation after wiring:** dropping the arm reproduces both failures.
**F-473** now names both verbs and records that, since the diagnosis is shared,
the 0.8 re-pointing is one arm.

### M-1 / M-4 — wording, no behaviour change

**M-1 reproduced.** Strings built at three payload widths under id `hash`:

| payload | chars | `me sysw pack` says |
| --- | --- | --- |
| 32 B | 74 | outside the profile |
| 33 B | 75 | hashlock PREIMAGE plate |
| **34 B** | **77** | **hashlock PREIMAGE plate** |

So "id `hash`, 75 characters" and "75 characters is the only shape" were both
false: the predicate asks `ms_codec` about the PREFIX BYTE. Both comments now
say kind byte `0x03`, name the well-formed plate as the 75-character `hash`
case, and state that a malformed `0x03` string (§1's `PreimageLengthMismatch`)
is named the same way and why that is the wanted direction — which matters
because F-473 asks a future implementer to re-point this exact predicate using
these comments.

**M-4:** "Plain BIP-93 secrets **are** 48 or 74 characters" → "**are usually**
48 or 74 characters". The corpus's own `bip93-plain-33-byte-payload-0x31` row is
a 75-character plain BIP-93 secret.

### M-3 — the godoc

`recoverCodex32Flow` collects each further share through `inputCodex32Flow`,
which is `newCodex32Keyboard` — no NFC anywhere on that path. Measured rather
than read: the C-1 test drives that exact branch with `runes()`. The *reason*
`unlockEngraveCodex32` does not reuse `engraveCodex32` is unchanged and still
correct.

## Mutations — all 14, re-run at the fold tips. No survivors.

Every mutation was applied to the branch tree, observed, restored from a byte
backup and `touch`ed; `git status --porcelain` was empty after each batch.

| # | mutation | fails with |
| --- | --- | --- |
| R1 | swap the two arms in `unknown_reason` | `left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))`; `stderr does not name the kind` |
| R2 | `validate_record` admits `ms10hash` | `preimage-plate-0x03: host verdict` (`left: true right: false`) + the pin test |
| R3 | `preimage_plate` always `false` | the two sysw witnesses **and now** `refused as Invalid("reserved-prefix byte was 0x03, expected 0x00"), not as a preimage plate` — N-1's tightening turned R3 from a 2-witness into a 3-witness mutation |
| R4 | drop the `preimage_plate` arm from `validate_record` | both I-2 tests (raw codec string returns) |
| G1 | `IsPreimage`: drop `!f.Unshared` | `IsPreimage(bip93-share-payload-0x03 …) = true, want false` |
| G2 | `IsPreimage`: `len(d) > 0` | `IsPreimage(bip93-plain-payload-0x03 (16-byte seed …)) = true, want false` |
| G3 | `IsPreimage`: `d[0] != msPrefixEntr` | `IsPreimage(bip93-plain-33-byte-payload-0x31 …) = true, want false` |
| G4 | `IsPreimage`: key on the id `hash` | `IsPreimage(preimage-shape-entr-id …) = false, want true` |
| G5 | `isStrictMs1`: drop the guard | `preimage-plate-0x03` + `preimage-shape-entr-id`: `device admits = true, want false` |
| G6 | `seal.Classify`: drop the guard | `Classify(…) = codex32 secret, want unknown format` + `AdmitSection` |
| G7 | `unlockEngraveCodex32`: drop the guard | `never reached "hashlock preimage"; last frame "Insertablankplate… EngravePlate"` |
| G8 | `engraveCodex32`: drop the in-loop guard | **both** door tests, including the new Recover one |
| G9 | `gui/scan.go`: drop the guard | `Scan(preimage plate) = codex32.String, <nil>; want errScanUnknownFormat` |
| **G10** | **new — guard OUTSIDE the `for` body** | `a preimage recovered from shares reached the SECRET confirm screen: "ConfirmCodex32SecretidHASHUnsharedsecret(S)75chars"` |
| **G11** | **new — the collision row → `device_admits: true`** | `bip93-plain-33-byte-payload-0x03: device admits = false, want true (Classify = 0)` |

## Final gates — verbatim, at the fold tips

### mnemonic-engrave `6e84280` (clean tree)

```
     Summary [   0.615s] 617 tests run: 614 passed, 3 failed, 2 skipped
        FAIL [   0.003s] (432/617) mnemonic-engrave::history_purge editing_the_file_alone_is_the_trap_the_message_warns_about
        FAIL [   0.003s] (438/617) mnemonic-engrave::history_purge the_harness_records_history_at_all
        FAIL [   0.006s] (444/617) mnemonic-engrave::history_purge the_emitted_zsh_recipe_actually_purges_the_entry
error: test run failed
nextest exit=100
error: manual implementation of `.is_multiple_of()`
   --> crates/me-cli/src/sysw/composer_records.rs:114:8
clippy exit=101
fmt exit=0
```

**617 run, 614 passed** (was 616/613 — the fold adds one test, `me seal`'s).
The three failures are the same box-local `history_purge` trio (`/usr/bin/zsh`
absent). Clippy is red on the same **pre-existing, untouched**
`composer_records.rs:114` lint the plan names as the local nightly's, green in
CI at `917d4e3` — unchanged from the pre-fold report's deviation 5, and still
flagged rather than called green. `fmt --check` clean.

### seedhammer fork `83fbc17` (clean tree)

```
=== go vet ./codex32/ ./sysw/ ./seal/ ===
vet exit=0
=== go test -count=1 ./codex32/ ./sysw/ ./seal/ ===
ok  	seedhammer.com/codex32	0.004s
ok  	seedhammer.com/sysw	0.184s
ok  	seedhammer.com/seal	11.932s
test exit=0
=== gui-shard-test.sh ./gui/ 24 ===
    1205 top-level tests
    partition verified exhaustive: 1205 == 1205
RESULT: ok -- all 1205 tests ran across 24 shards
shard exit=0
=== gofmt -l (repo) ===
gui/transaction.go
gui/transaction_golden_test.go
gui/transaction_txrecord_test.go
```

Fully green. **1205** gui tests, up from 1204 — the one new test is C-1's — with
the partition asserted exhaustive, so nothing was silently dropped. The three
`gofmt` files are the same pre-existing ones present on untouched `main`; every
file this fold touched is clean.

## Firmware, re-measured at the fold tip

```
$ nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 \
    -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller
   code    data     bss |   flash     ram
1551336   31796   31004 | 1583132   62800
```

| | flash | ram |
| --- | --- | --- |
| fork `main` `839fa5aa` | 1,582,628 | 62,800 |
| pre-fold `14afdff` | 1,583,100 | 62,800 |
| **fold tip `83fbc17`** | **1,583,132** | **62,800** |
| delta vs `main` | **+504 B** | **0** |
| delta vs pre-fold | **+32 B** | **0** |

The fold costs 32 bytes of flash — the C-1 guard moving inside the loop — and no
RAM. Still inside "a few hundred bytes" of the plan's reference.

## Still not mine, still not done

The emulator walk beyond the controller's `45f3d4c`, the merge to fork `main`,
the flash, the re-review of this fold, the continuity entry, and any push.
**Nothing was pushed; no commit was made on `master` or `main`.**
