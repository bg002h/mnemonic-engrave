# Composer S1 plan — R0 round 1, FOLD VERIFICATION lens

**Artifact:** `design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md` (folded `3919f1f`,
citation fix `502a5df`); `design/SPEC_wallet_policy_composer.md` +
`design/BRAINSTORM_wallet_policy_composer.md` (folded `e20eae1`);
`design/STAGED_PLAN_wallet_policy_composer.md` (folded alongside the plan, no
separate commit).

**BEFORE/AFTER:** plan `108fd4c` → `502a5df` (= working tree); spec/brainstorm
`46fc91b` → `e20eae1`; staged plan `46fc91b` → `502a5df`.

**Lens:** mechanical fold verification against three inputs — the opus fidelity
review (`composer-S1-plan-R0-r0-fidelity.md`, 0C/6I/10M/3N, `85493a5`), the
sonnet tests/mutation review (`composer-S1-plan-R0-r0-tests.md`, 0C/3I/2M,
`343f17a`), and the fable stand-in operator ruling
(`composer-S1-decision-now-default.md`, `7612066`). For each of the 24 findings
and each of the ruling's "What changes" items: did the fold fix it AS STATED,
graded against the finding's own statement of the defect (not its suggested
remedy), and did the fold introduce anything new that is wrong.

**Read-only.** No repository was written to except this report file. No
`.jsonl` file was read.

**What I ran, beyond reading:** rebuilt BOTH sides of the plan from scratch,
independently of the controller's own build-gate measurement, rather than
trusting it: (1) `scripts/plan-build-gate-me.sh` on the current plan, then
hand-wired Task 2's and the RULED Task 4's fragments into the scratch copy
myself (`sysw/record.rs`'s three `Class` variants, `sysw/mod.rs`'s
`UnknownReason::Composer`/`SyswError::SecondNow`/classifier arm/`split` check,
`main.rs`'s `sysw_error`/`class_name` arms/`Pack{now,no_now}`/the two
auto-append insertion points/`print_composer_confirmation`,
`tests/record_corpus.rs`'s name-table arms) under `RUSTUP_TOOLCHAIN=1.85.0`;
(2) a second from-scratch copy of `mnemonic-secret` at `5f37b43` with Task 5's
`derive.rs` and `cli_derive_bip48.rs` edits hand-applied. Both were built,
tested (`cargo nextest run`, `cargo test`) and clippied independently of the
"already settled" figures in my brief, specifically to catch a stale number if
one existed — one was found (below). Also: decoded every hex body I needed to
check with Python, grepped every citation against the real files at the stated
revisions, and grepped the whole plan/spec/brainstorm for eleven specific
superseded phrasings.

## VERDICT: 29 FIXED / 1 PARTIAL / 0 NOT FIXED / 0 DECLINED — 0 regressions, 1 new defect

24 R0 findings (19 fidelity + 5 tests) plus 6 ruling "what changes" items = 30
graded items. The one PARTIAL and the one new defect are unrelated to each
other and to any of the 24 findings' substance — every substantive Important
finding (I-1..I-6 fidelity, I-1..I-3 tests) is FIXED and independently
reproduced against a live build, not just read. The new defect is a stale
test count left in prose; it does not affect the code, the fixture, or any
test assertion.

## Per-finding table

| id | title | fold's response (AFTER, brief) | verdict |
| --- | --- | --- | --- |
| fidelity I-1 | plan implemented option (a); ruling landed is (c)-narrowed, no `--now` | Header rewritten to state the (c)-narrowed ruling; Global Constraints lines 25/27 updated; Task 4 gains `now: bool` (`conflicts_with = "no_now"`) and the `if !*no_now && (*now \|\| composer_record_present) && …` predicate; Task 4 step 4's test list and the two CLI tests rewritten | **FIXED** |
| fidelity I-2 | default breaks descriptor-input "one record" invariant | "Under the landed ruling this vanishes" — confirmed: `descriptor_as::item_1…` packs no `key:`/`hash:`, so it is never a casualty; plan text says so explicitly at Task 4 step 4 | **FIXED** |
| fidelity I-3 | second-`now:` refusal fires only inside `split`, AFTER the passphrase ceremony (F-246) | New precheck block inserted directly after the `admit_check` block (`main.rs:1563-1566`) and before `decide_sealing`; new test `a_second_now_is_refused_before_the_passphrase_ceremony` asserts `!err.contains("write this down")` | **FIXED** — reproduced live: test passes in my independent rebuild |
| fidelity I-4 | new `sysw_error` arms don't open `record N (records count from 0)`, contradicting the plan's own Global Constraints claim | Both `U::Composer` and `E::SecondNow` arms now open `"record {i} (records count from 0) is …"` with the §8n line verbatim on the next line; Global Constraints line 28 rewritten to state this precisely | **FIXED** — reproduced live: `descriptor_seam::is_record_refusal`'s `err.contains("(records count from 0)")` invariant holds for both new arms |
| fidelity I-5 | fixture omits 11 named §6a malformations/valid shapes; a wrong Go port passes all 27 | All 11 rows added (`key-depth-3-valid`, `key-testnet-tpub-valid`, `key-depth-2-refused`, `key-depth-5-refused`, `key-fingerprint-uppercase`, `key-fingerprint-7-hex`, `key-origin-no-path`, `key-origin-longer-than-depth`, `key-body-not-utf8`, `key-origin-unterminated`, `now-body-uppercase-hex`); coverage test's required-name list updated to match | **FIXED** — 40 CASES rows, coverage list matches exactly (below); every row classifies as claimed (below) |
| fidelity I-6 | Task 5 leaves the ms-cli module doc's false "no taproot value" sentence standing | Task 5 step 1 retitled "…fix the module doc"; the two module-doc sentences (lines 18-21) now rewritten first, before the test-flip | **FIXED** — verified against the real `cli_derive_bip48.rs` at `5f37b43`; hand-applied and rebuilt, compiles and the new test passes |
| fidelity M-1 | Task 2 step 8 sends the implementer to a nonexistent `match` in `expect.rs`; File Structure table and commit also stage it | Task 2 step 8, the File Structure table and the `git add` line all corrected: "the repo has exactly TWO exhaustive matches on `Class`… `expect.rs` has no match on `Class`… and needs no change" | **FIXED** (its own three named sites) — but see the propagation sweep: one FOURTH site (Global Constraints' build-gate bullet, unchanged by this fold) still lists `sysw/expect.rs` as a fragment to hand-wire |
| fidelity M-2 | two drifted `main.rs` citations (2060/2117-2118) | Corrected to `:2063` / `:2088-2089` | **FIXED** — confirmed against the real file at current HEAD |
| fidelity M-3 | two different `script_type_label` strings; the label is unreachable for this template | Interfaces line rewritten to drop the "labelled…" claim and explain unobservability; only one `"3' p2tr (taproot multisig; Coldcard/Liana convention)"` string remains in the plan | **FIXED** |
| fidelity M-4 | `show`'s `now:` line claims provenance (`the pack time`) it cannot know | New comment ("`show` cannot tell an auto-appended pack time from an operator-supplied bound") and reworded output string that no longer asserts "the pack time" as fact | **FIXED** |
| fidelity M-5 | no well-formed 31-byte `hash:` fixture row | `hash-31-bytes` row added | **FIXED** |
| fidelity M-6 | Task 6 already executed but reads `- [ ]` throughout | Task 6 retitled "…DONE 2026-09-02" with a status paragraph naming the fold and both R0 rounds' commits | **FIXED** — all six cited commits (`12e0659`,`bb49953`,`44765d7`,`72ac66d`,`de34664`,`fdf7671`) exist in the repo |
| fidelity M-7 | four sentences elsewhere become false and the plan doesn't fold them | Task 7 gains Step 3 with all four edits (N9, `U::Unrecognised`, `Pack`'s `--help`, `record.rs` doc); `git add` extended to the two touched files | **FIXED** |
| fidelity M-8 | staged plan's S1 Exit vs Task 7's hand-off disagree about vendoring | Staged plan Exit rewritten: fixture "READY for S2, whose first act vendors it"; Task 7 hand-off text now says the same | **FIXED** |
| fidelity M-9 | uppercase `H` marker refused with no fixture row or note | Global Constraints line 25 states it explicitly ("out of scope on BOTH sides… pinned as a refused fixture row"); `key-uppercase-H-marker-out-of-scope` row added | **FIXED** |
| fidelity M-10 | Task 5's JSON hint points at a test that doesn't parse JSON | Corrected: "the existing test does substring matching instead… this one asserts the three keys `derive.rs:485-487` emits" | **FIXED** — confirmed against the real file |
| fidelity N-1 | Global Constraints implies path hardening is required (it isn't) | Rewritten: "components need not be hardened, but the xpub's own child number must equal…" | **FIXED** |
| fidelity N-2 | two of Task 5's three match arms are not compiler-enforced | Warning added: "BOTH of those matches end in a wildcard arm… so omitting either compiles and silently derives… only the new test's assertion catches it" | **FIXED** |
| fidelity N-3 | Task 5's snapshot hedge is a no-op | Replaced with a measurement: "neither `gen_man.rs` nor `gui_schema_emits_spec_v7_json.rs` enumerates template values… so no snapshot changes" | **FIXED** — reproduced live: both files' tests pass unchanged in my independent ms-cli rebuild |
| tests I-1 | "exactly six" pre-existing failures is false (seven), and the seventh's file isn't staged | Moot-but-correct under the ruling: Task 4 step 4 explains the true SEVEN-test history, states none is a casualty any more, and pins `a_payload_without_a_composer_record_packs_byte_identically_to_before` as the regression guard | **FIXED** |
| tests I-2 | origin-length-vs-depth rule has no isolating test | `key_origin_rules_are_each_enforced` gains two isolating assertions (`[73c5da0a/48'/2']…` and a 5-component origin), both with last component `2'` matching the xpub's own child number | **FIXED** — reproduced live: both assertions pass against the real parser |
| tests I-3 | depth-3-or-4 boundary rule has no test | `key-depth-2-refused` / `key-depth-5-refused` fixture rows added, exercised by `every_case_classifies_as_its_row_says_and_refuses_with_its_line` | **FIXED** — reproduced live |
| tests M-1 | ordering-before-sniffers claim is architecturally unexercised | Note added explaining exactly why (`me` has no free-text fallback so the claim is unobservable on the host; matters on the device) — the finding itself said "no action required beyond noting it" | **FIXED** |
| tests M-2 | same drifted citations as fidelity M-2 | Same fix | **FIXED** |
| ruling: spec §6a stmt 1 | unconditional default | Replaced verbatim with the ruled text (key:/hash: predicate, `--now`, byte-identical-to-today clause) | **FIXED** |
| ruling: spec §6a stmt 2 | unconditional default | Replaced with the ruled text plus the Build-lock-echo sentence | **FIXED** |
| ruling: spec §10 item 2 | unconditional default | Replaced verbatim, word-for-word match to the ruling's proposed text | **FIXED** |
| ruling: spec §7g row | missing seed/card-only row | New row added: "seed-only or card-only payload for Build… DEFAULT: no bound appended…" | **FIXED** |
| ruling: brainstorm §3.12 item 21 | missing supersession record | Item 21 added, dated, names items 9 and 19's `now:` clause as superseded, cites the ruling file | **FIXED** |
| ruling: plan Task 4 + header | plan implements the wrong option | Header rewritten; Task 4's flag, predicate, tests and commit message all carry the ruling | **FIXED** |

## Claims checked

1. **CASES has 40 rows; coverage list matches exactly; the two origin-length
   rows are refusable only by the count rule.** TRUE. `grep -c` and a Python
   set-diff of the 40 `Case { name: … }` entries against the coverage test's
   40-item `required` list: **zero difference either direction**. Decoded
   both hex bodies: `key-origin-shorter-than-depth` → `[73c5da0a/48'/2']xpub…`
   (2 components, last `2'`); `key-origin-longer-than-depth` →
   `[73c5da0a/48'/0'/0'/0'/2']xpub…` (5 components, last `2'`) — both share
   XPUB0's own child number `2'`, so the code's count check
   (`origin.len() != xpub.depth`) fires before the last-component check is
   ever reached for either. Confirmed live: `key_origin_rules_are_each_enforced`
   passes with exactly these two constructions.
2. **`now-body-uppercase-hex` contains an uppercase letter.** TRUE. The hex
   string `313735363638343830302C393130303030` (decodes to
   `1756684800,910000`) contains exactly one letter, `C` — uppercase, from
   encoding the comma byte `0x2C` — which is what `unhex_lower`'s
   `(b'a'..=b'f').contains(&b)` rejects.
3. **Second-`now:` precheck placement and the ceremony test.** TRUE. The
   precheck sits directly after the `admit_check` block cited at
   `main.rs:1563-1566` (verified against the real file — matches exactly) and
   before `decide_sealing`/the passphrase print (`:1655-1682`, also verified).
   `a_second_now_is_refused_before_the_passphrase_ceremony` asserts
   `!err.contains("write this down")`. Reproduced live: PASSES.
4. **`sysw_error` arms open `record {i} (records count from 0)` and carry the
   §8n line verbatim on a following line.** TRUE for both `U::Composer` and
   `E::SecondNow`. `E::SecondNow`'s second line, `"record {i}: a second now: \
   record; only one is allowed. Remove one."`, is byte-exact against the
   unwrapped §8n blockquote at `SPEC_wallet_policy_composer.md:703-704`.
   `descriptor_seam.rs:853-854`'s `is_record_refusal` still recognises both by
   `err.contains("(records count from 0)")` — reproduced live (both refusal
   tests pass; `two_operator_supplied_now_records_are_refused_naming_the_second`
   now also asserts the phrase directly).
5. **Task 4's four new tests exist and match the ruling; no stray `--no-now`
   edits to pre-existing tests remain.** TRUE. All four
   (`pack_appends_the_pack_time_when_a_composer_record_is_present_and_says_so`,
   `a_payload_without_a_composer_record_packs_byte_identically_to_before`,
   `now_forces_the_append_onto_any_payload_and_conflicts_with_no_now`,
   `no_now_suppresses_the_auto_append_so_a_fixture_is_a_pure_function_of_its_inputs`)
   present and reproduced live (PASS). `grep -n -- "--no-now"` over the whole
   plan shows every occurrence is either the flag's own definition/tests/docs
   or the unaffected historical sentence about the plan's original title — none
   is an instruction to add `--no-now` to a named pre-existing test. Task 4's
   `git add` line no longer stages `tests/sysw_cli.rs`.
6. **Spec §6a (both statements), §10 item 2, §7g carry the ruled words;
   brainstorm item 21 exists and names items 9/19 superseded; no unconditional
   default remains.** TRUE. `perl -0777` multi-line search for the OLD
   phrasing `hold none\n(an` (i.e. "hold none" running directly into the
   parenthetical with no qualifying clause) returns **zero matches** anywhere
   in the spec; every one of the three "contain none"/"hold none" occurrences
   in the file carries the `AND … key:/hash:` qualifier. Brainstorm item 21
   is present, dated 2026-09-02, and says "SUPERSEDES item 9 and the `now:`
   clause of item 19" verbatim.
7. **Task 5 module-doc edit, label claim removed, wildcard warning, JSON-hint
   correction, snapshot hedge replaced.** TRUE, all five — and reproduced
   live in an independent `mnemonic-secret` rebuild: `bip48_p2tr_derives_…`
   and `bip48_p2tr_json_names_…` both PASS (16/16 in `cli_derive_bip48.rs`);
   `gen_man`/`gui_schema_emits_spec_v7_json` (14/14) unaffected, confirming
   N-3's "no snapshot changes" claim.
8. **Task 6 reads DONE; Task 7 gained the four record edits and stages them;
   staged plan's Exit no longer claims S1 vendors the fixture.** TRUE, all
   three parts (table above; six commits verified to exist).
9. **Citations.** All FIVE resolve exactly as claimed, checked against the
   real files: `main.rs:2063` is `fn print_mdmk_confirmation`; `:2088-2089`
   are the `print_mt_confirmation`/`print_descriptor_confirmation` calls;
   `:1563-1566` is the `admit_check` block; mnemonic-secret (`5f37b43`)
   `derive.rs:485-487` is the three JSON keys the test asserts
   (`account_path`, `account_xpub`, `script_type_defaulted`); `:504-518` is
   the `script_type_defaulted`-gated label print the plan says makes the
   label unobservable for `Bip48P2tr`.
10. **Propagation sweep.** See below — ten of eleven phrasings are fully
    superseded; `expect.rs` fails the "only in the sentence saying it needs no
    change" test (one residual site).

## Propagation sweep

| phrasing | hits (line) | verdict |
| --- | --- | --- |
| `exactly six` | none | superseded |
| `` gain `--no-now` `` | none | superseded |
| `27 rows` | none | superseded |
| `27-row` | none | superseded |
| `2215285f` | none | superseded (old sha256 fully replaced by `a894e619…`) |
| `Open question for the operator` | none | superseded (header retitled) |
| `option (a)` | none | superseded |
| `expect.rs` | line 31 (Global Constraints, "Build gate before every fold" bullet) **and** line 715 (Task 2 step 8, "needs no change") | **FAILS the check** — two hits, only one compliant. Line 31 is unchanged by this fold (it predates it) and still lists `sysw/expect.rs` among the fragments "hand-wired in the gate's scratch copy by the controller before review" — stale, since `expect.rs` has no `Class` match to wire. Not one of M-1's three named sites (Task 2 step 8, the File Structure table, the commit), so M-1 itself is graded FIXED above, but this is an incomplete propagation of the same underlying fact and is worth a one-line fix. |
| `labelled "3' p2tr` | none | superseded |
| `three hand-picked` | none | superseded |
| `pack_appends_the_pack_time_when_no_now_record_is_given` | none | superseded (renamed) |

## New defects introduced by the fold

**One: a stale test count in "What the build gate covers, and does not"
(plan, final paragraph).** The fold's added sentence reads: *"Re-measured
2026-09-02 with the RULED Task 4 … hand-wired beside Task 2's fragments:
`cargo nextest run -p mnemonic-engrave --locked --no-fail-fast` → **611 tests
run, 611 passed, 2 skipped**"*.

This is **wrong by exactly one test**. Two independent measurements both say
612:

- The controller's own `design/CONTINUITY_composer_2026-09-01.md:52`: *"Task 4
  wired: 612/612, clippy clean, threaded runner clean; fixture 40 rows…"* —
  already handed to me as a settled fact in the brief.
- My own from-scratch, independently hand-wired rebuild (methodology above,
  done without reading the continuity doc first): `cargo nextest run -p
  mnemonic-engrave --locked --no-fail-fast` → **`612 tests run: 612 passed, 2
  skipped`**, clippy clean, threaded `cargo test` clean, regenerated fixture
  sha256 `a894e619580db8ca0e06ebfe45576cc45722f695913bf46e9285201c95f146c3` —
  matching `FIXTURE_SHA256` exactly.

`sysw_composer_cli.rs` has 9 test functions after the fold, one of which
(`a_second_now_is_refused_before_the_passphrase_ceremony`, the I-3 fix) was
added after this paragraph's number was apparently last measured; the
paragraph was never re-run to pick up the extra test. The row count (40), the
sha256, and every other figure in this paragraph are correct — only the two
"611" instances are stale. **Not a regression** (no behavior changed; the
suite is actually 612/612 green, better than the paragraph claims), but it is
new prose the fold wrote that is factually wrong, and an implementer running
Step 4 who sees 612 instead of 611 would have no way to know that is expected
rather than a red flag.

No other new defect found. In particular: the second-now precheck's placement
relative to the `--as` handling (verified against the real `main.rs`, `--as`
resolves at lines 1416-1449, well before `admit_check` at 1563) introduces no
ordering bug; the `composer_record_present` predicate classifies with
`sysw::classify`, which only returns `Class::Key`/`Class::Hash` for
already-well-formed records (a malformed one would have been refused by
`admit_check` moments earlier), so there is no path where a malformed
`key:`/`hash:` record silently triggers or skips the auto-append.

## What I ran

```
git diff 108fd4c..502a5df -- design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md
git diff 46fc91b..e20eae1 -- design/SPEC_wallet_policy_composer.md design/BRAINSTORM_wallet_policy_composer.md
git diff 46fc91b..502a5df -- design/STAGED_PLAN_wallet_policy_composer.md
git log --oneline 502a5df..HEAD -- <the four design files>   # confirmed unchanged since 502a5df
git rev-parse HEAD; git cat-file -t <six Task-6 commit hashes>   # all exist

# mnemonic-engrave: independent rebuild, NOT reusing the controller's scratch copy
TMPDIR=<scratchpad> CARGO_TARGET_DIR=<scratchpad>/gate-target \
  bash scripts/plan-build-gate-me.sh design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md
# hand-wired Task 2 (record.rs Class variants; mod.rs UnknownReason::Composer,
# SyswError::SecondNow, classify_with arm, unknown_reason arm, split check)
# and the RULED Task 4 (main.rs Pack{now,no_now}, both insertion points,
# print_composer_confirmation, sysw_error/class_name arms) plus
# record_corpus.rs's three name-table arms, myself, via targeted python
# string-replacement against the scratch copy's real source (not copy-paste
# of the plan's fenced blocks, to avoid reproducing a plan bug by construction)
RUSTUP_TOOLCHAIN=1.85.0 cargo build -p mnemonic-engrave --all-targets --locked
RUSTUP_TOOLCHAIN=1.85.0 cargo nextest run -p mnemonic-engrave --locked --no-fail-fast
  # → 612 tests run: 611 passed, 1 failed (fixture not yet generated), 2 skipped
RUSTUP_TOOLCHAIN=1.85.0 cargo test --locked -p mnemonic-engrave --test sysw_composer_records \
  regenerate -- --ignored --nocapture
  # → wrote 40 rows; sha256 a894e619580db8ca0e06ebfe45576cc45722f695913bf46e9285201c95f146c3
RUSTUP_TOOLCHAIN=1.85.0 cargo nextest run -p mnemonic-engrave --locked --no-fail-fast
  # → 612 tests run: 612 passed, 2 skipped
RUSTUP_TOOLCHAIN=1.85.0 cargo test --locked -p mnemonic-engrave   # threaded runner, clean
RUSTUP_TOOLCHAIN=1.85.0 cargo clippy -p mnemonic-engrave --all-targets --locked -- -D warnings
  # clean

# mnemonic-secret: independent second scratch copy at 5f37b43, Task 5 applied by hand
RUSTUP_TOOLCHAIN=1.85.0 cargo build -p ms-cli --all-targets --locked        # clean
RUSTUP_TOOLCHAIN=1.85.0 cargo nextest run -p ms-cli --locked --test cli_derive_bip48
  # → 16 tests run: 16 passed, 0 skipped (both new bip48_p2tr_* tests pass)
RUSTUP_TOOLCHAIN=1.85.0 cargo nextest run -p ms-cli --locked --no-fail-fast
  # → 310 tests run: 308 passed, 2 failed (the two named scratch artefacts:
  #   conformance_vectors_pass, the_display_grouping_conformance_pin_is_untouched),
  #   11 skipped -- matches the plan's own paragraph exactly
RUSTUP_TOOLCHAIN=1.85.0 cargo nextest run -p ms-cli --locked -E 'test(gen_man) + test(gui_schema)'
  # → 14/14 pass, unaffected (N-3)
RUSTUP_TOOLCHAIN=1.85.0 cargo clippy -p ms-cli --all-targets --locked -- -D warnings   # clean

# grep/python checks
grep -oP 'Case \{ name: "\K[^"]+' <plan> | sort            # 40 rows
python3 <extract the `required` list from the coverage test>  # 40 names, set-equal to CASES
python3 -c "bytes.fromhex(...)"  # decoded key-origin-{shorter,longer}-than-depth, now-body-uppercase-hex
grep -n <5 citations> against mnemonic-engrave HEAD and mnemonic-secret 5f37b43
perl -0777 -ne '.../hold none\s*\n\s*\(an/g...'  # 0 matches -- no unconditional default survives
grep -n -F <11 superseded phrasings> design/IMPLEMENTATION_PLAN_composer_S1_host_inputs.md
```

No `.jsonl` file was read. Nothing was committed in any repository; the only
file this review wrote is this one.
