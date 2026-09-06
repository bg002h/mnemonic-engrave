# H6 plan R0 round 0 — independent tests/mutation review (sonnet)

Reviewing `design/IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md` at engrave
master `e6d84d9c`, spec `a2a031fa`. Method: copied the three gated trees
(`.tmp/h6-ms`, `.tmp/h6-me`, `.tmp/h6-gate`) via `cp -a` into my own
`.tmp/h6-tests-{ms,me,gate}`; every mutation below was applied to my copies,
run, quoted, then reverted and diff-checked against the pristine tree before
moving on. Never touched the gated trees themselves. Go binary
`/scratch/code/shibboleth/.toolchain/go/bin/go`; Cargo under
`CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/h6-tests-{ms,me}-target`,
`TMPDIR=/scratch/code/shibboleth/.tmp`. No sub-agents; no `.jsonl` read; nothing
committed.

One tree-isolation fix, kept (not a mutation): `h6-tests-me/Cargo.toml`'s
`[patch.crates-io]` pointed at `../h6-ms` (the gated tree, sibling-relative); I
repointed it to `../h6-tests-ms` so builds never touch the gated tree. This is
the only non-mutation diff against `h6-me`. `h6-tests-gate/go.mod` shows one
line moved between the direct/indirect blocks — a `go build`/`go test`
auto-adjustment (`GOFLAGS=-mod=mod`) touching an unrelated dependency
(`chainhash/v2`), not anything I edited.

## RED quotes (mutations applied to my copies, output quoted, then reverted)

**Rust — `preimage_plate_admissible` id conjunct dropped** (me-cli
`seal/record.rs`):
```
thread 'sysw::tests::a_preimage_plate_is_named_not_misdiagnosed' panicked at crates/me-cli/src/sysw/mod.rs:943:9:
assertion `left == right` failed
  left: Err(PreimageNotAdmitted(0, Preimage))
 right: Err(Unclassifiable(0, PreimagePlate))
```
(Reds one assertion earlier than the plan's own comment names, because
dropping the id test also widens the FIRST id-`test` row to `Preimage`; the
plan's own targeted row still catches it too — see mutation table.)

**Rust — same id check made case-insensitive** (`s.to_ascii_lowercase()`
before the `hash` compare): identical failure at the same line — the uppercase
plate row.

**Rust — `admit_check`'s second refusal rule dropped** (me-cli `sysw/mod.rs`):
```
thread 'no_flag_refuses_both_carriers_by_index' panicked at crates/me-cli/tests/sysw_pack_preimage.rs:81:9:
carrier 0 packed without the flag
```
Exact match to the plan's quoted failure.

**Rust — `Preimage`/`Phrase` dropped from `Class::is_secret`** (me-cli
`sysw/record.rs`):
```
thread 'the_warnings_print_in_the_f246_order' panicked at crates/me-cli/tests/sysw_pack_preimage.rs:220:10:
the passphrase ceremony did not run
```
Exact match, plus two more tests (`classification_is_unconditional`,
`the_flag_seals_by_default_and_names_the_class`) also red — broader coverage
than the plan's own one-line quote implied.

**Go — `codex32.EncodeMS1Preimage` minted under id `entr`**:
```
msencode_preimage_test.go:34: EncodeMS1Preimage = ms10entrsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kp9wv63u5a0u7q, want the corpus ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c
msencode_preimage_test.go:106: EncodeMS1Preimage emitted id "entr" for x=ee684868acd9adc4ae1a6569f69a786ec1ad4903fa2bac4a029d096eb3eec040
```
Byte-exact match to the plan's quoted campaign output.

**Go — `codex32.IsPreimagePlate` id test dropped**:
```
msencode_preimage_test.go:139: IsPreimagePlate(ms10entrsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kp9wv63u5a0u7q) = true: a kind-0x03 payload under id "entr" was admitted
msencode_preimage_test.go:168: IsPreimagePlate admitted the UPPERCASE spelling of a plate
```
Exact match.

**Go — budget pin `1379 + 20` changed to `1378`** (fork `engrave/engrave.go`):
```
h6_qr_test.go:242: constantTimeQRModules(53) = 1378, want 1399 (1379 observed + 20 buffer). The original was derived by fuzzing: 7,759,282 in 32 min, converged with 22.6 min of quiet. A different number needs its own campaign.
```
Exact match. **Independently confirmed the plan's own "retired mutation"
claim**: with the SAME budget of 1378 (observed max − 1), the fuzzed row
(`TestConstantTimeQRBudgetBoundsEveryPayload`, 400 in-suite samples,
`rand.NewSource(0x4831)`) still PASSED — `headroom 56`, observed only 1322 —
proving that mutation genuinely cannot fire from an in-suite sample, exactly as
the plan states, and that the PIN test is what actually guards it.

**Go — `engraveModule` scale-2 arm made symmetric** (mirrored offset instead
of the documented asymmetric one):
```
h6_qr_test.go:418: scale 2 p={0 0} axis x: ink [1920,5760], want [0,3840]
h6_qr_test.go:421: scale 2 p={0 0} axis y: ink [1920,5760], want [0,3840]
... (all four probed points fail on both axes)
```
**Go — `engraveModule`'s `case 2` arm removed entirely**:
```
panic: unsupported module scale [recovered, repanicked]
seedhammer.com/engrave.engraveModule(...) engrave.go:792
```
Matches the plan's stated `panic: unsupported module scale`.

**Go — `hashlockDerivedDigest` made to derive eagerly** (ignoring the lazy
`hashlockHeld` cache) — fork `gui/composer_hash.go`:
```
composer_hashlock_test.go:1302: one hardened derivation 10.550252ms; composerHashRows 10.264876ms
composer_hashlock_test.go:1307: the underived row does not say so: "phrase 1  3cf5d421..b70a4c12"
composer_hashlock_test.go:1310: composerHashRows took 10.264876ms against a 10.550252ms derivation: the row set is deriving, which §5.1 refuses -- three records would be a 30 s stall before a list could be drawn
composer_hashlock_test.go:1325: before derivation the row reads "phrase 1  b867db87..edbc96cb"
```
Both `TestWhichHashDoesNotDeriveAtRowBuildTime` and
`TestWhichHashPhraseRowShowsTheDigestOnceDerived` red.

**Go — the accepted preimage plate added as a `bundleCard`** (fork
`gui/composer_flow.go`, `composerEngraveStep`):
```
composer_preimage_plate_test.go:778: the shipped plate count changed when a preimage plate was accepted.
Frame: "PlatesToCutThisengraves2plates.md1template:1plate(key-lesswalletpolicy)mutation-fake:1plate(x)Eachplatetakesminutestocut...."
```
Matches the build-gate report's quoted failure (`hashlock-H6-plan-gate-8b-12.md`
mutation 9-4: *"the shipped plate count changed when a preimage plate was
accepted"*) verbatim.

**Go — §9's predicate swapped from `hashlock.IsMS1Shaped` to
`codex32.IsPreimage`** (fork `gui/sysw_session.go`, `syswWarnMS1Shaped`):
```
sysw_session_test.go:94: declining did not keep the operator on the screen (drawn=false, out=true)
sysw_session_test.go:100: accepting did not continue (drawn=false, out=true)
sysw_session_test.go:111: editing the text did not re-arm §9's warning
```
`TestSyswWarnMS1ShapedFiresOnceAndIsReArmedByAnEdit` catches it — the warning
goes silent for the fixture entirely, matching *"the warning is silent in BOTH
programs"*.

**Go — `hashlock.MethodLine`/`QRText` parameter order swapped** (fork
`hashlock/hashlock.go`, exploratory mutation not named by the plan, run to
false-PASS-hunt the F8 wiring — see below):
```
methodline_h6_test.go:8: MethodLine(true) = "method: pbkdf2-hmac-sha256 iterations=32 salt=ms-hashlock-v1 dklen=100000", want "method: pbkdf2-hmac-sha256 iterations=100000 salt=ms-hashlock-v1 dklen=32"
```
Caught by `TestH6MethodLineAndQRTextMatchTheRustPrimary`, a hardcoded-literal
lockstep — genuinely independent, not self-referential (see false-PASS
section).

**Go — the §11.4 "add one character" mutation, actually run** (fork
`backup/hashlock_test.go`, worst-case QR'd plate, `Method` padded to N chars,
73–79): confirmed **74–78 characters all still fit** (`rows=10 total=408320
budget=416000`) and **79 reds**:
```
method 79 chars: fits=false rows=11 fontMM=3.0 total=427520 budget=416000 qrDim=53
 -> EngraveHashlock err=backup: the hashlock plate does not fit at any font size: 11 rows at 3.0mm need 427520 units against a budget of 416000
```
Exact match to the plan's §11.4 finding and its quoted message. (My first
attempt at this used the QR-less plate and hit a `sed`/`python` replace typo
that touched an unrelated call site — worth recording since it shows how easily
this exact class of mistake — mutating the wrong occurrence — happens; caught
by re-grepping before trusting the result.)

## Mutation table

| # | mutation | caught-by / SURVIVED | quoted assertion |
| --- | --- | --- | --- |
| 1 | Rust: drop id conjunct from `preimage_plate_admissible` | caught — `a_preimage_plate_is_named_not_misdiagnosed` | `left: Err(PreimageNotAdmitted(0, Preimage))` at an EARLIER assertion (`mod.rs:943`) than the plan's own comment names (`mod.rs:979` MUTATION note); both id-narrowing rows in the test are affected |
| 2 | Rust: make the id compare case-insensitive | caught — same test, same line | identical panic |
| 3 | Rust: drop `admit_check`'s second rule | caught — `no_flag_refuses_both_carriers_by_index` | `carrier 0 packed without the flag` |
| 4 | Rust: drop `Preimage`/`Phrase` from `is_secret` | caught — `the_warnings_print_in_the_f246_order` (+2 more) | `the passphrase ceremony did not run` |
| 5 | Go: `EncodeMS1Preimage` mints under `"entr"` | caught — `TestEncodeMS1PreimageMatchesTheCorpusKindRow`, `TestEncodeMS1PreimageCanNeverEmitIDEntr` | byte-exact corpus-mismatch and id quotes |
| 6 | Go: drop id test from `IsPreimagePlate` | caught — `TestIsPreimagePlateNarrowsByID` | `a kind-0x03 payload under id "entr" was admitted`; uppercase row too |
| 7 | Go: budget pin `1379+20` → `1378` | caught — `TestConstantTimeQRBudgetEntriesAreTheFuzzedOnes` | quoted above, exact |
| 7b | Go: SAME budget (1378) against the 400-sample fuzzed row | **SURVIVED, as documented** — `TestConstantTimeQRBudgetBoundsEveryPayload` PASSES | `observed max 1322, budget 1378, headroom 56` — independently reproduces the plan's own disclosed retired-mutation finding; the PIN test is the only guard |
| 8 | Go: `engraveModule` case-2 arm made symmetric (mirrored) | caught — `TestEngraveModuleScale2` | ink-extent mismatch on all 4 probed points, both axes |
| 9 | Go: `engraveModule` case-2 arm removed | caught — same test | `panic: unsupported module scale` |
| 10 | Go: `hashlockDerivedDigest` derives eagerly (ignores lazy cache) | caught — `TestWhichHashDoesNotDeriveAtRowBuildTime`, `TestWhichHashPhraseRowShowsTheDigestOnceDerived` | timing (10.26ms vs 10.55ms KDF) and stale-row-text quotes |
| 11 | Go: accepted preimage plate appended to `cards` (`bundleCard`) | caught — `TestComposerCutsPreimagePlatesBeforeThePolicySet` | `This engraves 2 plates.` where the shipped count must stay 1 |
| 12 | Go: §9 predicate swapped to `codex32.IsPreimage` | caught — `TestSyswWarnMS1ShapedFiresOnceAndIsReArmedByAnEdit` | warning goes silent (decline/accept/re-arm all fail) |
| 12b | same mutation, checked against `TestHashlockMS1WarningFiresOnAGroupedPlate` | **does not exercise the mutated code path** — see false-PASS note below | test still passes, but it never calls `syswWarnMS1Shaped` |
| 13 | Go: `hashlock.MethodLine`'s `Iterations`/`Salt`/`PreimageLen` args reordered | caught — `TestH6MethodLineAndQRTextMatchTheRustPrimary` | quoted above, exact |
| 14 | Rust: `HASHLOCK_ITERATIONS` bumped without the corpus | caught — `qr_text_matches_every_corpus_row`; **SURVIVED** by `parameters_come_from_the_constants` (documented, see false-PASS) | corpus-string mismatch quote |
| 15 | Go: §11.4 — method line grown 73→79 chars (worst-case QR'd plate) | caught (RUN directly, not via a shipped test — no automated 73-79 sweep exists in the suite) | `11 rows at 3.0mm need 427520 units against a budget of 416000`; 74-78 all confirmed still fitting (`total=408320`) |

## False-PASS hunting

**Does any test recompute its expectation with the code under test?**
Two real instances found, both already disclosed by the plan and both backed
by an independent second test:
- `parameters_come_from_the_constants` (ms-codec `hashlock_qr_text.rs`) builds
  its "want" string from the SAME `HASHLOCK_ITERATIONS`/`HASHLOCK_SALT`/
  `HASHLOCK_DKLEN` constants `qr_text` reads, so it cannot catch a hard-coded
  literal drifting from those constants — confirmed by mutation 14 above: it
  stayed green while `qr_text_matches_every_corpus_row` (an independent,
  fixed-literal JSON corpus) reddened. The plan's own doc comment says exactly
  this ("this test still passes today").
- `gui/composer_preimage_plate_test.go`'s two `hashlock.MethodLine`/
  `hashlock.QRText` checks (lines 632, 638) are similarly self-referential —
  they verify `composerBuildHashlockPlate` correctly PLUMBS whatever
  `hashlock.MethodLine`/`QRText` return, not that the return value is right.
  I went looking for independent coverage of `hashlock.MethodLine`/`QRText`
  themselves (Task 9's F8 finding: *"a literal here could drift... without a
  single test noticing"*) and initially found none in `backup/hashlock_test.go`
  (its `TestHashlockQRTextMatchesTheMSCorpus` checks a **package-test-local
  literal** `h6HardenedMethodLine`, not the production `hashlock.MethodLine`,
  because `backup` deliberately has no dependency on `hashlock`). The actual
  guard is a separate file I'd missed on first grep:
  `hashlock/methodline_h6_test.go`'s `TestH6MethodLineAndQRTextMatchTheRustPrimary`,
  which hardcodes its own "want" string independent of the constants —
  confirmed by mutation 13 above. **No gap: the guard exists, just in a file
  whose name doesn't match `hashlock_test.go` and is easy to miss on a single
  grep pass.**

**Do the QR goldens compare module counts or images?** Neither, and that's a
non-finding in the plan's favour: `golden.CompareBSpline`
(`fork/internal/golden/*.go`) does a knot-by-knot comparison of the actual
b-spline toolpath (`Engrave` flag, parametric `T`, and `Ctrl` point within
epsilon 1 unit) against a stored gzip blob, for both `TestH6ConstantQRGoldens`
and `backup`'s `TestHashlockGoldens`. This is real geometry, not a count or a
raster image — a bug that moved a knot by 2 units would fail it.

**Does the constant-time test compare command sequences across payloads, or
only lengths?** Only lengths, and the plan discloses this accurately rather
than hiding it: `TestConstantQRMoveCountIsAFunctionOfDimAlone` counts
`cmd.Engrave(...)` commands for two random same-dimension payloads and asserts
equal COUNTS — its own doc comment says outright *"this row passes on a budget
of 0, on 700, and on a correct one"* and is labelled a regression guard, "NOT
THE BUDGET PROOF". Confirmed accurate: nothing in this test would catch a
content-dependent command SEQUENCE (only a content-dependent command COUNT).
No hidden claim here — the plan is explicit that this test cannot substitute
for the budget proof, and it doesn't.

## Other findings (not requested by the ONE QUESTION, surfaced while working it)

- **A genuine documentation inconsistency inside the plan itself, machine-
  checked.** Task 3's inline "Boundary gate (RUN)" (line ~1212) states *"621
  tests run: 618 passed, 3 failed... 2 skipped"* — identical to Task 2's
  number, even though Task 3 adds the 9-test `sysw_pack_preimage.rs` file. My
  own `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast` on the
  final (all-tasks-wired) tree measured **630 run: 627 passed, 3 failed
  (`history_purge`, box-local), 2 skipped** — which matches the plan's OWN
  later `## Build gate` per-task table (row 3: *"630 run, 627 passed..."*)
  exactly, but contradicts the inline Task 3 text earlier in the same
  document. Minor — the correct number is recorded elsewhere in the same file
  and my own measurement confirms it, but the inline claim is stale/wrong.
- **Test-name drift between the plan's aspirational tables and the shipped
  file.** Task 3's "test | asserts | MUTATION" table names tests
  `the_flag_admits_both`, `warning_order_against_the_ceremony`, and
  `the_orphan_warning_covers_both_carriers`; the actual file
  (`sysw_pack_preimage.rs`) has `the_flag_admits_both_carriers`,
  `the_warnings_print_in_the_f246_order`, and no test of the third name (its
  ground covered by `a_space_after_the_comma_derives_a_different_preimage_and_warns`
  and `no_hash_record_at_all_is_a_note_not_a_warning`). The plan's OWN later
  "MUTATIONS, all five executed" list uses the correct real names throughout —
  the drift is confined to the earlier, apparently-pre-implementation table.
  Minor, cosmetic, does not affect any test's correctness.
- **Corpus and provenance pins independently re-verified by hash**, not just
  trusted from the plan's prose: `me-cli/testdata/record_class_vectors.json`
  → `sha256 3575ccb0e12d12646c45dde583380199170cff815ea5e8d86d4d37d4a1c4abaf`
  (68 rows); fork's vendored copy `sysw/testdata/record_class_vectors.json` —
  identical hash; fork's vendored `hashlock/testdata/hashlock-v0.8.json` →
  `4f1819cdd0862b101afd48d0478e8f0b218f933dd3da449915fa3c5eaaba21d4`, matching
  `hashlock_test.go`'s `corpusSHA256` constant. All match the plan's claims.

## Full-suite runs (once, at the final trees, tails quoted)

**ms — `cargo nextest run --locked`** (tree confirmed byte-identical to
`.tmp/h6-ms` before and after all Rust mutation work):
```
     Summary [   0.246s] 562 tests run: 562 passed, 11 skipped
```

**me — `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast`** (tree
identical to `.tmp/h6-me` except the deliberate patch-path fix):
```
     Summary [   0.380s] 630 tests run: 627 passed, 3 failed, 2 skipped
        FAIL (437/630) mnemonic-engrave::history_purge editing_the_file_alone_is_the_trap_the_message_warns_about
        FAIL (440/630) mnemonic-engrave::history_purge the_emitted_zsh_recipe_actually_purges_the_entry
        FAIL (441/630) mnemonic-engrave::history_purge the_harness_records_history_at_all
```
The three failures are the pre-existing box-local `history_purge` reds named in
the plan's Global Constraints (missing `/usr/bin/zsh`), not H6 regressions.

**fork — `scripts/gui-shard-test.sh ./gui/ 24`** (tree identical to
`.tmp/h6-gate` except an unrelated go.mod direct/indirect line moved by
`go build`/`go test` itself):
```
=== enumerating tests in ./gui/ ===
    1285 top-level tests
    partition verified exhaustive: 1285 == 1285
=== running 24 shards in parallel (timeout 20m each) ===
  shard 0: ok    54 tests ... shard 23: ok    53 tests
=== wall: 24s ===
RESULT: ok -- all 1285 tests ran across 24 shards
```
Matches the plan's final claimed count and shard-18's closure (the shard that
failed before Task 8b landed, per the plan's own diagnosis, now passes).

**fork — final sweep**, `go test ./engrave/ ./backup/ ./codex32/ ./sysw/
./hashlock/ ./cmd/emu/`: all `ok`.

**fork — `gofmt -l gui/ hashlock/ backup/ codex32/ sysw/ engrave/ cmd/emu/`**:
prints exactly the three pre-existing files (`gui/transaction.go`,
`gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`), no
fourth.

**fork — `go vet ./gui/`**: exits 1 with exactly the two pre-existing
`testing.ArtifactDir requires go1.26 or later` diagnostics
(`freetext_sizeproof_golden_test.go`, `transaction_golden_test.go`).

**fork — `GOOS=js GOARCH=wasm go vet ./cmd/emu/`**: exit 0. `go test
./cmd/emu/`: ok. (The browser-driven walk itself, Task 12, was not re-run here —
it requires a browser against the real emulator, outside this review's
Go/Cargo-only method; the plan's author + build-gate agent already ran it four
times and I have no way to independently re-execute it in this environment.)

## Closing counts

- 8 of 8 items named in the brief's ONE QUESTION independently reproduced by
  actually running the mutation (hash-id checks on both sides; the never-`entr`
  encoder test; the admission gate for `--pack-preimage`; the budget check; the
  scale-2 arm; the census exclusion; the lazy derive; the free-text warning).
  All caught. Zero survived undetected.
- 2 additional plan-documented "mutation cannot fire as originally stated"
  claims independently reproduced (the budget max-minus-one; the §11.4
  74-78-vs-79-character threshold) — both confirmed accurate, both properly
  guarded by a different, working test.
- 2 self-referential ("recomputes with the code under test") patterns found;
  both already disclosed by the plan and both backed by a genuine independent
  second test, confirmed by mutation.
- 0 false-PASS defects found in the QR goldens (genuine knot-level geometry
  comparison) or in the constant-time move-count test (its count-only scope is
  honestly labelled, not oversold).
- 1 test-attribution nuance: `TestHashlockMS1WarningFiresOnAGroupedPlate` does
  not itself exercise the mutated `syswWarnMS1Shaped` call site (it is a
  standalone property test); the actual regression catch for that mutation is
  `TestSyswWarnMS1ShapedFiresOnceAndIsReArmedByAnEdit`. Not a defect — the
  mutation is still caught — but the plan's prose attributes the catch to the
  wrong test.
- 1 Minor: Task 3's inline boundary-gate count (621/618/3/2) is stale/wrong —
  contradicted by the plan's own later per-task table (630/627/3/2) and by my
  own measurement (630/627/3/2, matching).
- 1 Minor: three test names in Task 3's aspirational table don't match the
  shipped file; the plan's own later mutation list uses the correct names.
- Full-suite gates, run once at the final (unmutated) trees: ms 562/562
  passed·11 skipped; me 630/627 passed·3 failed(pre-existing)·2 skipped; fork
  gui-shard 1285/1285 exhaustive, 24/24 shards ok; fork final sweep (engrave,
  backup, codex32, sysw, hashlock, cmd/emu) all ok; gofmt/go vet baseline reds
  unchanged from the plan's stated 3-file/2-diagnostic baseline.

**No Critical, no Important findings against the plan's tests.** Every RED and
MUTATION I could execute reproduced as claimed (with two harmless naming/count
documentation slips, filed as Minor above). The plan's own two disclosed
"mutation as first written could not fire" cases are real, correctly
diagnosed, and correctly re-guarded by a different test in both cases.
