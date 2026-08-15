# S0b fold — independent re-review — 2026-08-15

**Scope, as briefed and as executed:** two questions only — did the fold fix
C-1, C-2, C-3, C-4, I-1, I-2, and did the fold introduce a new defect. Not a
fresh audit. S0b's pre-fold state, S1/S2/S3 and the original findings' derivation
were not re-litigated.

**Repo:** `/scratch/code/shibboleth/seedhammer`, branch `main`, HEAD
`05c5a736ca3af07b3d8d01a904ebf1e330529826`. Diffed against `4b8488e`.
**Reviewer:** independent context; author ≠ reviewer.

**Verdict: 0 Critical / 1 Important / 3 Minor / 3 Nit.**

The Important is not a regression — every one of the six findings' mechanisms is
repaired, and the untagged suite now enforces byte-identity on the deciding
machine, which is what the fold was for. The Important is that C-2's *headline
sentence* — "a fabricated gate record passes the entire suite" — still reproduces
through a door the fold closed only in prose, and the one check that would catch
it is behind `oraclelive`.

---

## FINDING-BY-FINDING VERDICT

| finding | verdict |
| --- | --- |
| **C-1** derived-census gate skipped without the oracles | **FIXED** |
| **C-2** comparison hardwired to one filename; a fabricated record passed | **FIXED BUT** — see I-1 below |
| **C-3** S2's md1 byte-identity gate skipped by the same construct | **FIXED** |
| **C-4** sysw vectors read from a sibling repo CI never checks out | **FIXED** |
| **I-1** `ok` computed from a driver-supplied `plates` | **FIXED** |
| **I-2** `NEEDLE_*` literals bound to the pinned list by a comment | **FIXED** |

---

## ENVIRONMENT USED FOR EVERY PROBE

All mutations ran in a **detached scratch worktree** at `05c5a73`
(`git worktree add --detach`), never in the main checkout, at a path where
`../../mnemonic-engrave` does not resolve. Go invoked as:

```
nix develop --command env HOME=<empty scratch dir> CGO_ENABLED=0 \
  GOPATH=/home/bcg/go GOMODCACHE=/home/bcg/go/pkg/mod \
  GOCACHE=/home/bcg/.cache/go-build go test …
```

so the run reproduces CI (no `~/.cargo/bin`, no sibling repo) while keeping the
module cache real. Every `$status` below is a **true exit code**, read directly
and never through a pipe. Every mutation was reverted with `git checkout --` and
the tree re-verified.

**Restoration confirmed.** `git status --porcelain` in
`/scratch/code/shibboleth/seedhammer` is empty; HEAD is still `05c5a73`; the
scratch worktree was removed and `git worktree prune` run; `git worktree list`
shows only `main` at `05c5a73` and `seedhammer-s3` at `db6486c` (untouched).

---

## BASELINE I RAN MYSELF (not inherited)

Full suite, verbose, in the no-oracle / no-sibling environment:

```
$ go test ./... -count=1 -v
ok pkgs: 51
FAILpkgs: 0
--- SKIP lines ---
--- SKIP: TestIdleTimerUnderSH2ShapedEventLoop (0.00s)
```

Exactly one skip, the pre-existing `SH2_REALCLOCK` diagnostic. Every gate the
fold introduces **executed**:

```
--- PASS: TestEveryGateRecordCensusMatchesItsCommittedExpectation (0.00s)
--- PASS: TestEveryCommittedExpectationBelongsToARecord (0.00s)
--- PASS: TestVendoredExpectationsWereDerivedFromThePinnedToolchain (0.00s)
--- PASS: TestPlateCountIsDerivedFromTheInputs (0.00s)
--- PASS: TestCommittedFingerprintsAreRealAndDistinct (0.00s)
--- PASS: TestAssembledMd1MatchesTheCommittedGolden (0.01s)
--- PASS: TestConformance (0.04s)      [8 subtests S-A … S-J, all PASS]
--- PASS: TestVendoredVectorsMatchTheirProvenancePin (0.00s)
--- PASS: TestWalkNeedleLiteralsAreAllPinned (0.00s)
--- PASS: TestWalkOkContainsNoDriverSuppliedPlateCount (0.00s)
```

and C-1's three mutation proofs now run **without the toolchain**, which was
C-1's sharpest consequence:

```
--- PASS: TestCompareCensusCatchesAMutatedString (0.00s)
--- PASS: TestCompareCensusCatchesAShortCensus (0.00s)
--- PASS: TestCompareCensusCatchesReorderedPlates (0.00s)
--- PASS: TestCompareCensusRefusesToPassOnNothing (0.00s)
--- PASS: TestLoadExpectRefusesAnEmptyExpectation (5 subtests, all PASS)
```

Grep for the four tag-only test names in that whole verbose log: **0 hits** — they
genuinely do not exist in a normal build.

Tree-wide `t.Skip` inventory (excluding `third_party`), re-run by me: 10 hits, of
which 8 are comments or the pre-existing fixture-shape / symlink / env-gated
skips the rulings excluded by name. **No member of the class remains.**

---

## IMPORTANT

### I-1 — A hand-authored gate record with six invented strings passes the entire untagged suite, and the only check that catches it lives behind `oraclelive`. C-2's headline sentence still reproduces.

**Severity: Important (blocks).**

`oracle/expect_test.go:109-134` — `TestEveryCommittedExpectationBelongsToARecord`
is the only untagged check that binds an expectation to anything, and it looks at
`Expect.Record` and `Expect.Stage` only; it never reads `Expect.Inputs`.
`oracle/expectfile.go:35-39` and `:43-44` — the three mechanisms that are said to
make a vendored expectation "a cached primary output, not a fork-authored
vector". `cmd/gaterecord/main.go:27-31` — "THERE IS NO WAY TO TURN THIS OFF".

**Failure scenario, in one sentence.** A stage author whose record `cmd/gaterecord`
refuses to mint (which is *every* stage from S3 on — see the fold's own F-a: no
`ExpectKind` names md1 policy chunks) writes the record, walk and expectation JSON
by hand, and the full suite reports green while logging "6 committed artifact(s)
matched the engraved census byte for byte" about six invented strings.

**Evidence I ran.** I planted `S9-hand.{record,walk,expect}.json` in
`oracle/gaterecords/` — census `md1FAKE0…md1FAKE5` in both the record and the
expectation, provenance block copied verbatim from the honest S0 expectation,
walk file sha256 recomputed so the record is self-consistent. No tool involved.

```
$ go test ./... -count=1                 # no-oracle env
HANDFAB2_TRUE_EXIT=0
ok  	seedhammer.com/oracle	0.106s
ok  	seedhammer.com/cmd/emu	1.965s

--- PASS: TestEveryGateRecordCensusMatchesItsCommittedExpectation (0.00s)
--- PASS: TestEveryCommittedExpectationBelongsToARecord (0.00s)
--- PASS: TestVendoredExpectationsWereDerivedFromThePinnedToolchain (0.00s)
--- PASS: TestPlateCountIsDerivedFromTheInputs (0.00s)
--- PASS: TestCommittedFingerprintsAreRealAndDistinct (0.00s)
--- PASS: TestS0GateHasARecord (0.00s)
--- PASS: TestEveryGateRecordOnDiskVerifies (0.00s)
--- PASS: TestGateRecordStringsAreRecordsOfTheCardsPayload (0.00s)

    expect_test.go:101: S9-hand.record.json: 6 committed artifact(s) matched the
                        engraved census byte for byte
    expect_test.go:154: S9-hand.record.json: derived by ms@ddfa4970 mk@a38a908e,
                        all still pinned
    record_test.go:395: verified 2 gate record(s):
                        [S0-trace-a.record.json S9-hand.record.json]
    gaterecord_anchor_test.go:79: S9-hand.record.json plate 0 is not an mk1
                        (md1FAKE0…) — not anchored by the payload   [× 6, then PASS]
```

`record_test.go:395: verified 2 gate record(s)` is the **same log line the
original review quoted as C-2's proof**, about the same class of fiction.

And the check that does catch it — tagged, so it does not exist in CI or in any
routine local run:

```
$ go test -tags oraclelive -count=1 -v \
    -run TestLiveDerivationReproducesEveryCommittedExpectation ./oracle/
LIVE_TRUE_EXIT=1
--- FAIL: TestLiveDerivationReproducesEveryCommittedExpectation (0.08s)
    live_test.go:128: S9-hand.record.json: loading S9-honest.inputs.json:
                      open gaterecords/S9-honest.inputs.json: no such file or directory
```

**This is the brief's probe 1 answered affirmatively:** a load-bearing property —
"a committed expectation must be re-derivable, i.e. tied to a committed inputs
file the package knows how to derive from" — is enforced **only** under
`-tags oraclelive`, and nothing routinely runs that tag (CI compiles it and runs
no test; `scripts/oracle-live.sh` has no caller).

**The claims that make the gap invisible are false as written**, and this is the
part that turns a design trade into a finding:

- `oracle/expectfile.go:35-39` — "NOTHING CAN MINT ONE BUT A LIVE DERIVATION …
  An expectation cannot exist except as the output of a live run." `git add` is a
  code path; the probe above is the counter-example.
- `oracle/expectfile.go:43-44` — "THE LIVE ARM STILL RUNS on every machine that
  could mint, which is exactly the population that could mint a wrong one." This
  was true of the pre-fold arrangement (the live arm ran in `go test ./...`
  wherever the oracles existed). After `05c5a73` the live arm runs on **no**
  machine unless a human types the tag. Same sentence recurs at
  `oracle/live_test.go:117-118` and
  `gui/multisig_build_oracle_live_test.go:87-88`.
- `cmd/gaterecord/main.go:27-31` — "THERE IS NO WAY TO TURN THIS OFF … it is what
  makes 'a committed expectation' mean 'something the primary produced' rather
  than 'something somebody wrote down'." True of the tool; not true of the
  directory.

**Minimal fix** (toolchain-free, ~15 lines, no new file, closes the probe above):
in the untagged `TestEveryCommittedExpectationBelongsToARecord`, additionally
require, for every record:

1. `Expect.Inputs != ""`;
2. `filepath.Join(GateRecordsDir, Expect.Inputs)` exists and `LoadInputsFile`
   parses it;
3. `inf.Expect != nil` and `inf.Expect.Kind` is a kind `DeriveExpected` knows
   (export the kind set as e.g. `oracle.KnownExpectKind(k) bool` — the switch
   already exists at `oracle/expect.go:163`);
4. optionally, that every `Artifact.Kind` in the expectation is the kind that
   `inf.Expect.Kind` produces (`cosigner-cards` ⇒ `mk1`) — my fabrication set
   `md1` against a `cosigner-cards` inputs block and nothing noticed.

That does not make hand-authoring impossible (nothing toolchain-free can), but it
forces a fabricated expectation to also be a *self-describing, one-command
re-derivable* artifact, which is the honest bound of the vendored layer. Then
correct the four claims above to state what is actually enforced.

**Second half of the fix, cheap:** because the fold's F-a means no stage from S3
on can mint through the tool, the hand-authoring path is not hypothetical — it is
the path of least resistance. Raise F-a from "Important, owned by S3" to a
prerequisite that must land before any S3 record is committed, and say in
`design/` that a record whose expectation the untagged layer cannot re-link is
not evidence.

---

## MINOR

### M-1 — Four sources cite `go vet -tags oraclelive ./...` as the workflow's type-check of the tagged files. The workflow does not run it, and it exits 1.

`oracle/live_test.go:56-57`, `gui/multisig_build_oracle_live_test.go:25`,
`sysw/vendored_vectors_live_test.go:24`, `scripts/oracle-live.sh:49`.

The workflow step is `go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/`
(`.github/workflows/test.yml:68`). The fold's own judgement call 11 explains why
vet was rejected — the comments were simply not updated with it. Measured, cold
`GOCACHE`:

```
$ go vet -tags oraclelive ./...
VET_TAGGED_TRUE_EXIT=1        findings: 40
gui/op/draw_test.go:176:24: testing.ArtifactDir requires go1.26 or later …

$ go vet -tags oraclelive ./oracle/ ./gui/ ./sysw/
VET_TAGGED_NARROW_TRUE_EXIT=1
gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 …

$ go test -tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/    # what CI runs
TAGSTEP_TRUE_EXIT=0
ok  seedhammer.com/oracle  0.001s [no tests to run]   (× 3)
```

A maintainer following the comment runs a command that fails for reasons
unrelated to the tagged files. **Fix:** replace the four citations with the
command the workflow actually runs.

### M-2 — The stale-pin backstop now runs on no machine by default, and nothing invokes the script that would run it.

`oracle/live_test.go:182` (`TestRealPinsResolveTheInstalledOracles`),
`scripts/oracle-live.sh`.

Pre-fold this test skipped in CI but **ran automatically** on any machine with the
oracles installed — i.e. the maintainer's. Post-fold it exists only under the tag,
and `grep -rn 'oracle-live.sh'` finds no caller: not in `.github/`, not in a
`Makefile`, not in any stage-close checklist. That is the brief's "a build tag
nothing runs" applied to a drift check. It is not load-bearing for byte-identity
(the vendored layer carries that, verified above), so Minor rather than
Important — but the drift question is now answered by nobody.

**Fix:** the fold's own F-j, upgraded — name `./scripts/oracle-live.sh` in the
repo `CLAUDE.md` **and** make its exit code a recorded line in each stage's
gate/close report, so "the maintainer's machine answered the drift question on
date X" is on disk rather than assumed.

### M-3 — `TestCommittedFingerprintsAreRealAndDistinct` applies a cosigner-card property to every record in the directory, and will hard-fail on the first md1-chunk record.

`oracle/expect_test.go:183-198`. It loops `loadExpectations(t)` — every record —
and requires each artifact to carry a non-empty `Fingerprint` and the census to
hold ≥2 distinct ones. An S3/S4/S5 record of built md1 policy chunks has no
per-artifact fingerprint at all, so it fails on the first artifact.

Fail-closed, so not blocking, and arguably the correct forcing function — but it
is a **second** undiscovered blocker stacked on F-a, and F-a does not mention it.
**Fix:** one line in F-a, and scope the assertion to expectations whose
`inputs.expect.kind` is `cosigner-cards` when the new kinds land.

---

## NIT

### N-1 — `TestCompareCensusCatchesReorderedPlates` has no length guard, unlike its sibling.

`oracle/expect_test.go:265` does `got[0], got[2] = got[2], got[0]` with no check,
while `TestCompareCensusCatchesAMutatedString` at `:220-223` guards the same index
with an explicit `INCONCLUSIVE` fatal. A future first-in-directory record with
fewer than 3 artifacts panics instead of reporting. Copy the guard.

### N-2 — `TestWalkOkContainsNoDriverSuppliedPlateCount` prints a green-sounding summary on a failing run.

`cmd/emu/needle_test.go:324` increments `checked` before the `plates` check, and
`:337` logs unconditionally. Observed verbatim in my I-1 mutation probe:

```
    needle_test.go:326: walk_trace_a.js's `ok` contains `plates`, which the CALLER supplies …
    needle_test.go:337: 2 walk script(s) checked; no driver-supplied plate count in any `ok`
```

Both lines, same run. Move the `Logf` behind `if !t.Failed()`.

### N-3 — the tagged compile step names three packages; a tagged file in a fourth would not be compiled. Broadening it is measured-safe.

`.github/workflows/test.yml:68`. Measured, cold `GOCACHE`:

```
$ go test -tags oraclelive -run '^$' ./...
TAG_ALL_TRUE_EXIT=0        (51 packages ok)
```

so `./...` costs nothing and removes the maintenance claim. (Related, sub-nit:
`sysw/vendored_vectors_test.go:92-93` slices `p.SHA256[:16]`, `p.Commit[:8]` and
`p.FileCommit[:8]`, but `loadVectorPin` at `:53-58` only checks `SHA256`,
`Commit` and `Path` for emptiness — a pin with a short or absent `file_commit`
panics rather than reporting.)

---

## PROBE-BY-PROBE, INCLUDING EVERY CLEAN ONE

### Probe 1 — the `oraclelive` split: does anything load-bearing live only inside it?

Tag inventory (`grep -rn oraclelive`): exactly three tagged files —
`oracle/live_test.go`, `gui/multisig_build_oracle_live_test.go`,
`sysw/vendored_vectors_live_test.go`. Contents: `resolveBins`,
`TestLiveDerivationReproducesEveryCommittedExpectation`,
`TestRealPinsResolveTheInstalledOracles`, `s2OracleMD`,
`TestAssembledMd1MatchesThePrimaryByteForByte` (+ the `-update` mint path),
`TestVendoredVectorsAreInSyncWithThePrimary`.

**Byte-identity itself is NOT tag-only — verified in both directions.** The
untagged gates execute and can fail (probes 3, 6 below). Only two ExpectFile
writers exist in the whole tree:

```
$ grep -rn '\.Write(' --include='*.go' . | grep -v third_party
cmd/gaterecord/main.go:258:      if err := exp.Write(expPath); err != nil {
gui/multisig_build_oracle_live_test.go:161:  if err := ef.Write(s2GoldenPath); err != nil {
```

so the "only a live run mints a golden" claim holds *for code in this repo*.

**What IS tag-only and load-bearing:** the re-derivability link
(expectation → inputs file → derivable kind). That is finding I-1 above.

**Also tag-only but correctly ceded:** primary-drift detection (sysw staleness,
pin-vs-installed-binary). Those are maintainer questions the ruling filed to S6;
they are recorded as M-2 only because nothing invokes the script.

### Probe 2 — `cmd/gaterecord`'s mint-time refusal: can a bad record still be minted?

Every attack path I could construct, run against a scratch mint dir. True exit
codes, no pipes:

```
A  fabricated census (the review's own S9-fake, md1FAKE0…5)   MINT_A_TRUE_EXIT=1
     gaterecord: REFUSING to mint this record: the walk's engraved census is not
     what the primary toolchain derives from these inputs.
     plate 0 (mk1, payload:masterA (card A@0)) differs: …
C  inputs file with NO `expect` block                          MINT_C_TRUE_EXIT=1
     …carries no `expect` block, so nothing says what these inputs REQUIRE…
D  `expect.kind: "md1-chunks"` (unknown kind)                  MINT_D_TRUE_EXIT=1
     unknown expectation kind "md1-chunks"; refusing to derive nothing and
     report it as a match
E  walk with ok:false, honest census                           MINT_E_TRUE_EXIT=1
     the walk did not finish green, so it cannot anchor a gate record
F  walk with an EMPTY census (nil slice on the wire)           MINT_F_TRUE_EXIT=1
     the census is empty, so nothing was engraved to anchor to
G  all six strings genuine, plates 0 and 2 SWAPPED             MINT_G_TRUE_EXIT=1
     REFUSING to mint this record: …plate 0 … differs
H  honest control                                              MINT_H_TRUE_EXIT=0
     6 artifact(s) derived live by [ms mk] and matched the walk's census
     wrote S9-honest.{record,walk,expect}.json
```

After A/C/D/E/F/G the mint directory held **no** `.record.json`, `.walk.json` or
`.expect.json` output — only my inputs. The refusal is genuinely unconditional:
there is no flag or environment variable that reaches it, and the empty/nil paths
fall out through `NewExpectFile`'s `len(arts) == 0` refusal before any write.
**Clean.** The residual is not a mint path at all — it is the directory, finding
I-1.

### Probe 3 — vendored expectations and the provenance-equality test: can it pass vacuously?

Vacuity floors read and exercised: `LoadExpect` (`oracle/expectfile.go:188-220`)
refuses absent / unparseable / wrong-schema / unknown-field / zero-artifact /
empty-string / zero-oracle, and `TestLoadExpectRefusesAnEmptyExpectation` covers
all five plus the absent case — all PASS untagged in the baseline above.
`loadExpectations` fatals on an empty directory and on any record without an
expectation.

Pin bumps, no toolchain, true exit codes:

```
3a  mk commit a38a908e… -> deadbeef…             P3A_TRUE_EXIT=1
    --- FAIL: TestVendoredExpectationsWereDerivedFromThePinnedToolchain
        oracle mk was derived at commit a38a908e…, pins.json now says deadbeef… —
        re-mint the expectation, do not edit it
    --- FAIL: TestEveryGateRecordOnDiskVerifies

3b  md commit 5a0a4f41… -> deadbeef…             P3B_TRUE_EXIT=1
    --- FAIL: TestAssembledMd1MatchesTheCommittedGolden
        testdata/s2_md1_golden.expect.json: … oracle md was derived at commit
        5a0a4f41…, pins.json now says deadbeef…

3c  md sha256 -> 000…0 (commit untouched)         P3C_TRUE_EXIT=1
    --- FAIL: TestAssembledMd1MatchesTheCommittedGolden
```

Byte flips in the committed data, no toolchain:

```
flip one char of expectation artifact 2          EXPECT_MUT_TRUE_EXIT=1
    --- FAIL: TestEveryGateRecordCensusMatchesItsCommittedExpectation
        expect_test.go:96: S0-trace-a.record.json: the engraved census is not what
        the primary toolchain derived (committed in gaterecords/S0-trace-a.expect.json)

flip one char of S2 golden chunk 3               GOLDEN_MUT_TRUE_EXIT=1
    plate 3 (md1, Trace A 2-of-3 wsh policy, chunk 3) differs:
      expected md1fxrvxzs…d9tq33dq
      engraved md1fxrvxzs…d9tq33dm
restored -> --- PASS: TestAssembledMd1MatchesTheCommittedGolden (0.04s)
    6 md1 chunk(s) byte-identical to the committed primary output (md @ 5a0a4f41)
```

`find . -name '*.expect.json'` returns exactly the two goldens, both covered.
**Clean.**

### Probe 4 — I-1's `ok`: is every term emulator-observed?

`cmd/emu/walk_trace_a.js:294` —
`ok: census.strings.length > 0 && census.unattributed === 0`. Both terms come off
the toolpath recorder.

`cmd/emu/walk_build_policy.js:548-553` — `proven.length === 7 &&
presentedAtEnd === 0 && cardsGathered > 0 && selected && ( … census !== null &&
census.strings.length > 0 && census.unattributed === 0 )`. `expect` and `engrave`
appear only as **mode selectors** choosing which observed arm applies, which is
what "`ok` NAMES ITS OUTCOME" means; `proven.length === 7` counts needle
*sightings*, not driver input. No caller-supplied value stands in for content in
either file. `plates` survives only as `runEngraveTail`'s loop bound
(`walk_trace_a.js:215`, `walk_build_policy.js:165`), documented as one.

The guard is real in both directions:

```
restore the pre-fold expression in walk_trace_a.js  PI1_TRUE_EXIT=1
--- FAIL: TestWalkOkContainsNoDriverSuppliedPlateCount
    walk_trace_a.js's `ok` contains `plates`, which the CALLER supplies (I-1/F-170)
```

and it fails closed on an unreadable `ok` (see probe 5b). **Clean** apart from
N-2.

### Probe 5 — I-2's needle binding: does it read the JS off disk, and does it glob?

`cmd/emu/needle_test.go:194` — `filepath.Glob("walk_*.js")`, not a named file,
with an `INCONCLUSIVE` fatal on an empty glob (`:198-201`) and a second fatal if
the files yield zero `NEEDLE_*` declarations between them (`:228-232`).

```
5a  repoint NEEDLE_TEMPLATE "Choose policy type" -> "First card from where?" (2 sites)
    P5A_TRUE_EXIT=1
    --- FAIL: TestWalkNeedleLiteralsAreAllPinned
        needle_test.go:219: walk_build_policy.js declares NEEDLE_TEMPLATE =
        "First card from where?", which is not in buildFlowNeedles.
        needle_test.go:226: walk_build_policy.js: 7 NEEDLE_* declaration(s), 1 unpinned

5b  add a NEW cmd/emu/walk_zzprobe.js declaring NEEDLE_S3_BOGUS = "Which md1?"
    P5B_TRUE_EXIT=1
    --- FAIL: TestWalkNeedleLiteralsAreAllPinned
        needle_test.go:219: walk_zzprobe.js declares NEEDLE_S3_BOGUS = "Which md1?" …
    --- FAIL: TestWalkOkContainsNoDriverSuppliedPlateCount
        needle_test.go:313: INCONCLUSIVE: walk_zzprobe.js has no `ok:` property
        this test can read …
```

5b is the one that matters for the brief's concern: a walk added by a later stage
is bound **the moment it lands**. S3's worktree already carries
`cmd/emu/walk_s3_nested.js`, which the glob will pick up on rebase — and note that
it will also have to expose a readable `ok:` expression, or
`TestWalkOkContainsNoDriverSuppliedPlateCount` fails closed rather than skipping.
Blind spot is stated in the source (`:169-175`: bare `waitFor("literal")` is
invisible), which is correct disclosure. **Clean.**

### Probe 6 — C-4's vendored sysw vectors.

`sysw/conformance_test.go:26` — `defaultVectors = "testdata/sysw_vectors.json"`,
in-repo. `SYSW_REQUIRE_VECTORS` grep: zero hits anywhere in the tree.

```
6a  delete sysw/testdata/sysw_vectors.json
    P6A_SYSW_TRUE_EXIT=1   SKIP lines in sysw: 0
      --- FAIL: TestConformance / TestTheVectorSetIsMeaningful /
                TestConformanceMDMKDecode / TestPaddedRegionMatchesTheBareVector /
                TestVendoredVectorsMatchTheirProvenancePin
    P6A_GUI_TRUE_EXIT=1    SKIP lines in gui: 0
      --- FAIL: TestSyswLoadFlowBootSkipLoadsNothing /
                TestSyswLoadFlowBootDefaultsToLoad /
                TestSyswLoadFlowDecliningTheDigestUnloads

6b  replace the file with `[]`
    P6B_TRUE_EXIT=1        SKIP lines: 0
      conformance_test.go:91: INCONCLUSIVE: testdata/sysw_vectors.json holds no
      vectors, so this test checks nothing              [and :157, :192]
      --- FAIL: TestVendoredVectorsMatchTheirProvenancePin

6c  flip one nibble of vector S-A's blob
    P6C_TRUE_EXIT=1
      --- FAIL: TestConformance  --- FAIL: TestPaddedRegionMatchesTheBareVector
      --- FAIL: TestVendoredVectorsMatchTheirProvenancePin
```

Missing → fatal, empty → fatal, mutated → three independent gates fire (semantic
identity, the padding mirror, the provenance hash). Zero skips in every case.
The sync audit (`sysw/vendored_vectors_live_test.go`) cannot pass silently: a
missing sibling checkout and a missing vendored copy are both `t.Fatalf`, and only
the *commit-bookkeeping* tail is best-effort — explicitly and correctly labelled
as such at `:57-60`, after byte equality has already been asserted. **Clean.**

Residual, pre-existing and by ruling: `SYSW_VECTORS` still overrides the path in
both `sysw/conformance_test.go:65` and `gui/sysw_load_test.go:38`, so a developer
can point the conformance suite at a narrower set — but nothing under `.github/`
sets it, and `TestVendoredVectorsMatchTheirProvenancePin` reads `defaultVectors`
directly, so CI is unaffected. The ruling kept it deliberately; not a finding.

---

## WHAT I DID NOT DO

Did not re-derive the six findings, did not re-review pre-fold S0b, did not audit
S1/S2/S3, did not re-measure `go vet ./...`'s 40-finding baseline, `gofmt -l`,
`test-32bit.sh`, `GOOS=js go vet`, `cmd/emu/build.sh` or the tinygo flash size —
all stated as machine-verified in the brief and none touched by my probes. I did
pin `GOCACHE` on every comparison, per the caching trap in the brief.

## CLOSING

The fold does the thing it was written to do: on the machine whose verdict gates
a merge, the derived-census comparison, S2's md1 byte-identity gate, the whole
sysw conformance suite and both walk guards now **execute**, can **fail**, and
have been seen to fail — with one pre-existing, env-gated skip left in the tree
and no member of the C-1..C-4 class remaining. C-1, C-3, C-4, I-1 and I-2 close.

C-2 closes on both of its stated mechanisms and reopens on its stated
*consequence*: the fabricated record still passes, through the directory rather
than through the tool, and the check that notices is behind a tag nobody runs.
That is one Important finding with a fifteen-line, toolchain-free fix, plus three
Minors and three Nits.
