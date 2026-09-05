# Hashlock H0 plan — R0 round 1, fold verification

**Reviewer:** independent sonnet fold-verification reviewer (targeted).
**Artifact under review:** the fold commit `fdfb040` over
`design/IMPLEMENTATION_PLAN_hashlock_H0_reader_guards.md` (gate-green draft
`b0af794`), responding to round 0's two persisted reports: fidelity (opus)
`design/agent-reports/hashlock-H0-plan-R0-r0-fidelity.md` (`1b254c9`,
2C/5I/2M/0N) and tests (sonnet) `design/agent-reports/hashlock-H0-plan-R0-r0-tests.md`
(`a7aebdc`, 0C/1I/3M).

**ONE QUESTION:** did the fold address every Critical and Important from both
reports, without introducing a contradiction or a false claim of its own?

**Answer: mostly yes, with one new false claim.** All 8 C/I rows from round 0
are genuinely FIXED in the plan text, verified by executing the plan's own
code blocks in scratch copies (never the controller's `.tmp/h0-fork` or
`me-worktrees/h0-gate2`). But the fold's own gate-output paragraph in the
commit message claims **"whole crate 615/616"** GREEN for the Rust side; the
true count, reproduced independently below, is **610/616 (6 failures)** — the
plan's own Task 1 Step 1 corpus edit silently breaks three tests in a THIRD
file (`crates/me-cli/tests/record_corpus.rs`) that neither round-0 report
caught and that Task 1 never mentions or updates. That is a new, unreproducible
gate claim (Important, per this round's severity rule).

**Closing counts: 0 Critical / 1 Important (new) / 0 Minor beyond wording —
NOT GREEN.**

---

## 1. Finding table

### Criticals and Importants (8, both reports)

| # | Report | Finding | Fold's claim | Verdict |
| --- | --- | --- | --- | --- |
| C-1 | fidelity | Two engrave doors (NFC scan, typed `M*1 STRING`) unguarded, both reach `engraveCodex32`/`EngraveSeedString` | FIXED — Task 2 Step 8 narrows `gui/scan.go:89`'s codex32 arm and guards the `engraveCodex32` choke point; two new tests | **FIXED.** Plan text quotes both edits verbatim (Step 8a: `} else if s, err := codex32.New(string(buf)); err == nil && !codex32.IsPreimage(s) {`; Step 8b: `if codex32.IsPreimage(scan) { ... showError(ctx, th, "Hashlock preimage", ...) }`). Applied both to a fork scratch copy; `TestEngraveCodex32RefusesAPreimagePlate` and `TestScanDoesNotHandAPreimagePlateToEngrave` PASS; removing (b) reproduces the claimed mutation output verbatim (`last frame "ConfirmCodex32SecretidHASHUnsharedsecret(S)75chars"`); removing (a) reproduces `Scan(preimage plate) = codex32.String, <nil>; want errScanUnknownFormat`. Traced every `codex32.New(`/`engraveObjectFlow(`/`engraveCodex32(` call site in `gui/*.go`: a third path exists (`gui/gui.go:2869`, `syswOffer(ClassCodex32Secret)` inside `newInputFlow`) but it is guarded upstream by `isStrictMs1` (Step 4) and, even if it weren't, converges on the same `engraveObjectFlow` call at `gui.go:2151` → the same guarded choke point. Only two non-test callers of `backup.EngraveSeedString` exist in the whole `gui`/`backup` tree (`gui.go:2816` inside the choke point, `unlock_session.go:203` behind Step 7's guard) — no third door found. |
| C-2 | fidelity | `IsPreimage` read any string's first payload byte, misclassifying ~1/256 of shares and plain BIP-93 secrets, including a whole sealed payload as "Payload unreadable." | FIXED — singles-only, shape-exact predicate (`ParsePrefix(...).Unshared && len(Seed())==33 && Seed()[0]==0x03`); two `device_admits:true` corpus rows | **FIXED.** Applied the predicate to a fork scratch copy; built the three `NewSeed`-derived corpus rows and confirmed each decodes to exactly what its `source` field claims: `bip93-plain-payload-0x03` (id=`test`, Unshared=true, 16-byte payload, byte0=0x03) → `IsPreimage=false`; `bip93-share-payload-0x03` (id=`test`, threshold=2 i.e. NOT unshared, 33-byte payload, byte0=0x03) → `IsPreimage=false`; `preimage-shape-entr-id` (id=`entr`, Unshared=true, 33-byte payload, byte0=0x03) → `IsPreimage=true`, so the device guard still refuses it. All three match their corpus `host_admits`/`device_admits` values. |
| I-1 | fidelity | Three anchors wrong (`codex32_seam.rs:15-16`, `codex32_seam_test.go:11`, `seal/record.go:214-216`) | FIXED — re-cited (`:25-26`, `:30`, `:212-214`), every Modify now quotes anchor text | **FIXED.** All three, plus every other anchor cited anywhere in the fold (`gui/scan.go:89`, `seal/record_test.go`'s `{d.Public[2], ClassMDMK}`, `sysw/mod.rs`'s `Bip93OutsideTheProfile(usize),` / profile-arm line / `THE CONTROL` comment, `main.rs`'s `Bip93OutsideTheProfile(len)` arm, `record.rs`'s `bip93_outside_the_profile`, `codex32/mspayload.go`'s const block, `gui/codex32_polish.go`'s `engraveCodex32`/`for {`), verified to exist **exactly once**, at the exact cited text, in the fork at `839fa5aa` and engrave at `e06e29d` (see §4 Anchors below). |
| I-2 | fidelity | The `IsPreimage` unit test's own MUTATION comment named a mutation the shipped test could not catch | FIXED — six-population table, each mutation measured against exactly one row; a 33-byte `0x31` row added | **FIXED.** Ran `TestIsPreimageReadsThePrefixByteOnly` (PASS). Ran all four declared mutations against the fork scratch copy and reproduced each exactly: drop `!f.Unshared` → share row `true, want false`; drop `len(d)==33` → 16-byte row `true, want false`; `d[0] != msPrefixEntr` → 0x31 row `true, want false`; id-keyed (`id, _, _ := s.Split(); return id == "hash"`) → entr-id row `false, want true` (reproduced independently in a second scratch copy). |
| I-3 | fidelity | `me`'s refusal message calls a preimage plate a BIP-93 secret, contradicts itself on length, tells the operator to re-encode it as entropy | FIXED — `preimage_plate` + `UnknownReason::PreimagePlate` + Display text, arm before the profile arm; unit test + binary test | **FIXED.** Applied Task 1 Step 7 to an engrave worktree at `fdfb040`; built `me`; ran `echo <plate> \| me sysw pack`: stderr now reads *"record 0 ... is a hashlock PREIMAGE plate (kind 0x03, id `hash`), not a seed record ... do not re-encode it as entropy"* — no "outside", no "re-encode the entropy", record not echoed. Ran the unit test `a_preimage_plate_is_named_not_misdiagnosed` (PASS) and the mutation (swap the two arms in `unknown_reason`) → reproduced `left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))` exactly. Confirmed the **non-widening** requirement separately: `me sysw pack` on the corpus's genuine `entr-id-but-off-profile-length-90` string (90 chars, id `entr`) still gets the unchanged profile message, not the new arm. |
| I-4 | fidelity | A cheaper device acceptance exists today (the two direct doors need no payload) and was declined | FIXED — Task 3 Step 2 rewritten: acceptance before the flash (Step 8's two tests + Step 7's test, plus an emulator walk of both doors); only the sealed half still waits for H2 | **FIXED** (as a plan-text/process change — not independently re-executable without the physical device/operator, per the brief's scope). Text matches the fidelity report's own SUGGESTION almost verbatim: the executable half is exactly Step 7+Step 8 (which I ran and confirmed PASS above), and the walked half names the expected screens ("Hashlock preimage" — matches the actual `showError` title string in Step 8b's code exactly; "Unknown format" — matches `gui/gui.go`'s `scanUnknownFormat → "Unknown format"` rendering exactly, confirmed by reading the source). |
| I-5 | fidelity | "Payload unreadable." (the operator-visible outcome of the sealed refusal) was never recorded | FIXED as records — one sentence in Global Constraints, a Task 4 follow-up owned by H2 | **FIXED**, exactly as declared (a record, not a behavior change, matching the fidelity report's own "not a behaviour change for H0" instruction). Global Constraints now states both containers' fates including "Payload unreadable."; Task 4 item (3) files the H2-owned follow-up for a dedicated arm. |
| Tests I-1 | tests | The whole corpus/test surface cannot distinguish "keyed on the prefix byte" from "keyed on the id `hash`" | FIXED — `preimage-shape-entr-id` row (unshared, 33 bytes, 0x03, id `entr`) in the corpus and the table, id-keyed mutation measured | **FIXED.** Row present in the corpus with `host_admits:false, device_admits:false`; present in the six-population unit-test table with `want:true`; the id-keyed mutation (constructed independently, not copy-pasted from the plan) returns `false` on this exact row, reproducing the claimed `entr-id row false, want true` output. |

### Minors

| # | Report | Finding | Fold's claim | Verdict |
| --- | --- | --- | --- | --- |
| M-1 | fidelity | `record.rs:177` cited, actual call at `:176` | FIXED (`:176`) | **FIXED.** Global Constraints now cites `:176`; confirmed `record.rs:176` at `e06e29d` is `ms_codec::decode(s)`. |
| M-2 | fidelity | Plan describes only the `seal` container's fate for an unknown record, not `sysw`'s | FIXED (both containers' behaviour stated) | **FIXED.** Global Constraints' "No new class, and what the operator sees" paragraph names both: `sysw` (per-record inert, counted "inert") and `seal` (whole-section refusal, "Payload unreadable."). |
| Tests M-1 | tests | Two stale line citations (`codex32_seam_test.go:11`→30, `seal/record.go:214-216`→212-214) | FIXED with fidelity I-1 | **FIXED**, same edit as fidelity I-1; confirmed. |
| Tests M-2 | tests | The `d[0] != msPrefixEntr` mutation comment overstated what the test actually caught ("the entr and mnem seams below") | Message says "recorded" (not claimed fixed) | **Incidentally fixed anyway.** The rewritten Step 3 comment (driven by I-2's table rewrite) no longer contains the overstated phrase; it now states plainly "The mnem row is 17 bytes and is refused by the length test alone" — the exact gap the tests report flagged. A positive discrepancy (fold undersold what it fixed), not a defect. |
| Tests M-3 | tests | The Rust pin test's `MsTooLong` arm is unreachable/dead for this fixture | Message says "recorded" (not claimed fixed) | **Not fixed, as declared.** The pin test's comment is unchanged context in the diff; matches the fold's own claim of leaving it as a record. No defect. |

---

## 2. The two Criticals, executed (brief item 2)

Both executed end-to-end in a fork scratch copy (`/scratch/code/shibboleth/.tmp/h0-fork-verify`, a tar copy of `seedhammer` at `839fa5aa`, removed at the end) with the full Task 2 diff applied (Steps 2, 4, 5, 6, 7, 8), plus the vendored 12-row corpus (sha256 `f1f2fa6b…391c`, recomputed and matched — see §3).

**(a) C-1.** `go test -count=1 -run 'TestEngraveCodex32RefusesAPreimagePlate|TestScanDoesNotHandAPreimagePlateToEngrave|TestUnlockEngraveCodex32RefusesAPreimagePlate' ./gui/` → all PASS. Removing the `engraveCodex32` guard: `TestEngraveCodex32RefusesAPreimagePlate` fails with `never reached "hashlock preimage"; last frame "ConfirmCodex32SecretidHASHUnsharedsecret(S)75chars"` (verbatim match to the fold's claim). `grep -n "engraveObjectFlow(\|engraveCodex32(\|codex32.New(" gui/*.go` traced: a third non-test path (`gui/gui.go:2869`, the `syswOffer(ClassCodex32Secret)` recovery branch inside `newInputFlow`) exists, but (i) `isStrictMs1` (Step 4) already keeps a preimage plate out of the `ClassCodex32Secret` offer set, and (ii) even a hypothetical bypass converges on the same `engraveObjectFlow` call at `gui.go:2151` → the same guarded `engraveCodex32` choke point. `grep -rn "EngraveSeedString(" gui/*.go backup/*.go` (excluding tests) found exactly two non-test callers — `gui.go:2816` (inside the guarded choke point) and `unlock_session.go:203` (behind Step 7's guard) — no unguarded third caller.

**(b) C-2.** `go test -count=1 -run TestIsPreimageReadsThePrefixByteOnly ./codex32/` → PASS. Confirmed both `bip93-plain-payload-0x03` and `bip93-share-payload-0x03` classify `ClassCodex32Secret` in both `sysw.Classify` (`isStrictMs1`) and `seal.Classify` after the guard, via `go test -count=1 ./sysw/ ./seal/` (both green, seam test's `device_admits:true` assertion for those two rows holds) plus a throwaway unit test (deleted after) confirming `s.Seed()`, `s.Split()` and `IsPreimage(s)` for all three `NewSeed`-constructed rows match their `source` field claims exactly (values quoted in the table above).

---

## 3. I-3, executed (brief item 3)

Applied Task 1 Step 7 to an engrave worktree (`me-worktrees/h0-verify`, detached at `fdfb040`, removed at the end), plus Step 1 (the 4 corpus rows — sha256 recomputed as `f1f2fa6bbbf27e3697ee496636de49be2f25787deff7b3bc4a2c5e16854e391c`, matching the plan exactly) and Step 3 (re-pin).

- `cargo test --locked -p mnemonic-engrave --test codex32_seam --test preimage_plate_is_not_a_seed`: 3/3 PASS, including `sysw_pack_names_a_preimage_plate_and_never_echoes_it`.
- Swapped the two arms in `unknown_reason`: `a_preimage_plate_is_named_not_misdiagnosed` FAILS with `left: Err(Unclassifiable(0, Bip93OutsideTheProfile(75)))` — reproduced verbatim, then reverted (`touch`ed after restoring).
- `echo "<75-char plate>" | me sysw pack` stderr: *"record 0 (records count from 0) is a hashlock PREIMAGE plate (kind 0x03, id `hash`), not a seed record; this container cannot place one yet... do not re-encode it as entropy."*
- Non-widening check: `echo "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqutd7mdh2lc8h2" | me sysw pack` (the corpus's genuine `entr-id-but-off-profile-length-90` row, 90 chars, id `entr`) — stderr unchanged: *"is a VALID BIP-93 codex32 string ... not a constellation `ms1` record ... This one is 90 characters ... re-encode the entropy as `ms1`..."* — still the profile message, confirming the new arm did not widen.

---

## 4. New contradictions (brief item 4) — **one new Important finding**

**The corpus shape counts (2/6/4).** Recomputed directly from the 12-row JSON after applying Task 1 Step 1: `both=2, device_only=6, neither=4, host_only=0`. Matches every citation of "12 rows: 2 both / 6 device-only / 4 neither" in Global Constraints, Task 1 Step 4, and Task 2 Step 4.

**Task 2 Step 1's RED claim.** Reverted the `sysw.isStrictMs1` guard against the 12-row corpus: `TestCodex32SeamDeviceAdmitsEverythingTheHostDoes` fails on **exactly** `preimage-plate-0x03` and `preimage-shape-entr-id` (`device admits = true, want false (Classify = 2)`), the two `0x03`-leading `device_admits:true` rows pass. Matches the plan's claim exactly.

**`errScanUnknownFormat` vs. spec §12 item 7 / "named refusal."** Not a contradiction. §12 item 7's exact text (`mnemonic-secret/design/SPEC_ms_hashlock.md:837-839`) requires only that "`sysw.Classify` is not `ClassCodex32Secret`" and "no engrave path offers it" — it does not require a *named* refusal. The plan's own text (Task 2 Step 8a) explicitly says the Scan arm's fallthrough is "a refusal, not a named one. The named refusal is at the choke point" — the plan states this distinction itself rather than obscuring it, and it is consistent with the spec's weaker requirement. No defect.

**NEW (Important): the fold's own gate-output claim ("whole crate 615/616") is false — the plan's Task 1 Step 1 corpus edit breaks a third, unmentioned test file.**

Applying exactly Task 1 Step 1 (the four new rows in `crates/me-cli/testdata/codex32_seam_vectors.json`) to the engrave worktree at `fdfb040` and running the **whole crate** with `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast`:

```
Summary [0.484s] 616 tests run: 610 passed, 6 failed, 2 skipped
```

Six failures, not the one (`history_purge::the_harness_records_history_at_all`) the fold commit message names. Three are the pre-existing, environment-only `history_purge` failures (no `/usr/bin/zsh` on this box — matches the tests report's own finding, unrelated to H0). The other **three are new**, all in `crates/me-cli/tests/record_corpus.rs` — a file **not named anywhere in the plan's File Structure table, Task 1, or Task 4**:

```
FAIL (447/616) record_corpus  every_corpus_record_classifies_as_it_did_before_s2
FAIL (448/616) record_corpus  the_descriptor_gate_stays_shut_on_every_corpus_record
FAIL (449/616) record_corpus  the_capture_is_the_whole_corpus
  assertion `left == right` failed: testdata/record_corpus_pre_s2.json is not the enumerated corpus
  assertion `left == right` failed: class assertions run   (left: 33, right: 37)
  assertion `left == right` failed: gate assertions run    (left: 33, right: 37)
```

`record_corpus.rs`'s own doc comment explains why: it is "Invariant 2, as a gate" from the prior S2 cycle — a whole-corpus snapshot (`testdata/record_corpus_pre_s2.json`, 33 records) built by enumerating `sysw_vectors.json`, `codex32_seam_vectors.json` and some inline literals, deliberately captured so that "a change to that file is a change to invariant 2 and has to be argued for in the diff." Task 1 Step 1 grows `codex32_seam_vectors.json` from 8 to 12 rows (33→37 total records) but never touches `record_corpus_pre_s2.json`, so the capture goes stale and the gate — working exactly as designed — fails.

**Root cause of why round 0 and the fold's gate both missed this:** `cargo nextest run --locked -p mnemonic-engrave` **without** `--no-fail-fast` (confirmed: this is nextest's behavior in this repo, with no `.config/nextest.toml` present) stops after accumulating the pre-existing `history_purge` failures and never reaches `record_corpus` (alphabetically later) — reproduced directly: the identical command without `--no-fail-fast` reports `Summary [...] 457/616 tests run: 454 passed, 3 failed, 2 skipped` and exits before `record_corpus` runs at all. That is very likely the exact command that produced both the tests report's "3 pre-existing... failures" claim and the fold's "615/616" claim — both are undercounts of the true, whole-crate state, for the same masking reason. Verified by reverting the corpus edit alone (`git stash` in the worktree, everything else unchanged): `record_corpus.rs`'s 6 tests all pass; reapplying the edit reproduces the 3 failures every time.

This is not a design flaw in the plan's guard logic — C-1 and C-2 are correctly fixed, as shown above — but it is a genuine, mechanical, reproducible defect in Task 1 as written: following it exactly, as the "Whole crate, then commit" step instructs, ships a broken invariant-2 capture and a false GREEN claim in the fold commit's own message. Per this round's severity rule ("a mutation claim you cannot reproduce = Important"), this is **Important**, not Minor: the specific number cited (`615/616`, one named failure) does not reproduce under a full run, and the true state includes a regression to a test the plan never names or updates.

**Recommended fix for the next fold (not performed here — read-only round):** add `testdata/record_corpus_pre_s2.json` to Task 1's File Structure table and Step 9's `git add`/commit, with a step that regenerates the capture and argues for the 33→37 change in the diff (per that test's own documented contract); and change every whole-crate gate invocation in this plan (and ideally the reusable gate script) to `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast`, since the default silently hides real regressions behind unrelated pre-existing failures.

---

## 5. Anchors (brief item 5)

Every anchor cited in a `Modify` block, checked against the fork at `839fa5aa` and engrave at `e06e29d`, confirmed to exist **exactly once** at the cited text:

| File @ pin | Anchor text | Exists once? |
| --- | --- | --- |
| fork `gui/scan.go` | `} else if s, err := codex32.New(string(buf)); err == nil {` | yes (line 89) |
| fork `seal/record.go` | `if _, err := codex32.New(s); err == nil {` … `return ClassCodex32Secret` | yes (lines 212-214) |
| fork `sysw/codex32_seam_test.go` | `const seamVectorsSHA256 = ` | yes (line 30) |
| fork `gui/codex32_polish.go` | `func engraveCodex32(ctx *Context, th *Colors, scan codex32.String) bool {` / `for {` | yes (line 218 / the following `for {`) |
| fork `seal/record_test.go` | `{d.Public[2], ClassMDMK}, // md1` | yes (line 413) |
| fork `sysw/classify.go` | `_, err := codex32.New(record)` / `return err == nil` (as a pair, `isStrictMs1`'s last two lines) | yes (lines 123-124) |
| fork `gui/unlock_session.go` | `id, _, _ := s.Split()` (the insertion point) | yes (unique in `unlockEngraveCodex32`) |
| fork `gui/unlock_session_test.go` | `func runUnlockEngraveMnemonic(t *testing.T, pf Platform, rec []byte) *sessionHarness {` | yes (line 714) |
| fork `codex32/mspayload.go` | the const block (`msPrefixEntr`/`msPrefixMnem`/`msMaxLanguage`) | yes (lines 8-12) |
| engrave `crates/me-cli/tests/codex32_seam.rs` | `const SEAM_VECTORS_SHA256: &str =` | yes (line 25) |
| engrave `crates/me-cli/src/seal/record.rs` | `pub fn bip93_outside_the_profile` | yes (line 204) |
| engrave `crates/me-cli/src/seal/record.rs` | `ms_codec::decode(s)` (the M-1 fix target, `:176`) | yes (line 176) |
| engrave `crates/me-cli/src/sysw/mod.rs` | `Bip93OutsideTheProfile(usize),` | yes (line 145) |
| engrave `crates/me-cli/src/sysw/mod.rs` | `if crate::seal::record::bip93_outside_the_profile(record) {` | yes (line 183) |
| engrave `crates/me-cli/src/sysw/mod.rs` | `/// **THE CONTROL, in both directions.**` | yes (line 832) |
| engrave `crates/me-cli/src/main.rs` | `U::Bip93OutsideTheProfile(len) => format!(` | yes (line 2799) |

No anchor drift found anywhere in the fold — this fully closes fidelity I-1.

---

## Closing counts

- **Critical: 0.** Both round-0 Criticals (C-1, C-2) are genuinely fixed and independently re-executed above.
- **Important: 1 (new).** The fold's Rust gate-output claim ("whole crate 615/616") does not reproduce; the true whole-crate state is 610/616 with 3 new failures in `record_corpus.rs`, caused by Task 1 Step 1's corpus edit and never mentioned anywhere in the plan. All 8 of round 0's own C/I findings are FIXED as claimed (table in §1) — this is a new finding from this round's own execution (brief item 4), not a reopened round-0 item.
- **Minor: 0 beyond wording.** All round-0 Minors are either FIXED (fidelity M-1, M-2, tests M-1) or left as records exactly as declared (tests M-3), with tests M-2 incidentally resolved as a side effect.
- **Anchors: 16/16 confirmed**, no drift (§5).

**GREEN / NOT GREEN: NOT GREEN.** One new Important finding (the false whole-crate gate claim / missing `record_corpus_pre_s2.json` update) must be folded and the gate re-run with `--no-fail-fast` before this plan can close R0.
