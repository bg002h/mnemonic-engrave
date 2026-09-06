# F-503 implementation report — the preimage KIND under the id `hash` is inert at every length

Written by the ONE implementer dispatched from
`design/agent-briefs/f503-implementer-brief.md` against
`design/IMPLEMENTATION_PLAN_hashlock_F503_hash_kind_inert.md`. Tasks 1, 2, 3 in
order (Rust first, then the fork as a CONVERGENCE port, then records). Nothing
pushed; no commits on `master` or `main`. Every number below is from a run at
the branch tip, captured under `/scratch/code/shibboleth/.tmp/f503-runs/`.

## 1. Branch tips

| repo | branch | tip | baseline |
| --- | --- | --- | --- |
| mnemonic-engrave | `f503` | **`cb0628e70e0c8e579c98214d3e2ab31e5872f29c`** | master `4df46e22` |
| seedhammer (fork) | `f503` | **`b32ff08ab0f6f025c3c3242a6546a522ffcc80ed`** | main `3cadffa8` |
| mnemonic-secret | `f503-records` | **`81d67b85be98de3d25246099f02e6ef4bb949240`** | master `cacf5da` |

Worktrees: `/scratch/code/shibboleth/me-worktrees/f503`,
`/scratch/code/shibboleth/.tmp/seedhammer-f503`,
`/scratch/code/shibboleth/ms-worktrees/f503-records`. All three trees are clean
at their tips (`git status --porcelain` empty).

### Commits

| SHA | task | subject |
| --- | --- | --- |
| `277c6336` | 1 | F-503 (me): the preimage-plate refusal names the conjunct that failed; four seam rows pin the hash-id kind at every length |
| `b32ff08` | 2 | sysw: the preimage KIND under the id hash is inert at every length; codex32.IsPreimageKind (F-503) — `-s` signed off |
| `81d67b85` | 3 | spec: SPEC_ms_hashlock states F-503's precise device rule … |
| `6ea358db` | 3 | records: F-503 CLOSED — me f503 277c6336, fork f503 b32ff08, ms f503-records 81d67b85 |
| `cb0628e7` | — | followup: F-506 — the UPPERCASE spelling of the F-503 string is still a device seed class (deviation D6, §6) |

Every commit carries `Co-Authored-By: Claude Opus 5 <noreply@anthropic.com>` and
`Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA`;
backticks verified intact with `git log -1 --format=%B`.

## 2. Boundary numbers, measured at the tips

**me** (`cargo nextest run --locked -p mnemonic-engrave --no-fail-fast`, at
`6ea358db`, re-run at the tip; `final-me-suite.txt`):

```
Summary [   0.366s] 634 tests run: 631 passed, 3 failed, 2 skipped
```

The three failures are the `history_purge` zsh trio (F-500, no `/usr/bin/zsh` on
this box), named exactly: `the_harness_records_history_at_all`,
`editing_the_file_alone_is_the_trap_the_message_warns_about`,
`the_emitted_zsh_recipe_actually_purges_the_entry`. **This is the plan's
expected figure, 634 / 631 / 3 / 2.**

- `cargo fmt --all -- --check` → exit 0, no output (`t1s9-fmt.txt`).
- `cargo clippy --locked --all-targets -p mnemonic-engrave` → exit 0, ONE
  warning, the pre-existing `manual implementation of .is_multiple_of()` at
  `crates/me-cli/src/sysw/composer_records.rs:177` (`t1s9-clippy.txt`). Nothing
  new.

**fork** (at `b32ff08`):

- `gofmt -l codex32/ sysw/` → empty (`t2s4-gofmt.txt`).
- `go test $(go list ./... | grep -v /gui$) -count=1` → exit 0, **54 packages
  `ok`**, no failures (`t2s4-nongui.txt`).
- `scripts/gui-shard-test.sh ./gui/ 24` → exit 0,
  `RESULT: ok -- all 1289 tests ran across 24 shards`, wall 25s
  (`t2s4-gui.txt`). **The plan's expected 1289.**
- `go test ./codex32/ ./sysw/ -count=1` re-run at the tip → both `ok`
  (`final-fork.txt`).

**Seam corpus parity.** Both copies sha256
`f53a17dc9d1ea5a4c0ff913786e82a5f19de004f4719536bb6d957bccdef1ee8` — the plan's
target value, reached without adjustment — and `cmp` reports them byte-identical.
Both literals re-pinned: `SEAM_VECTORS_SHA256` in
`crates/me-cli/tests/codex32_seam.rs:26` and `seamVectorsSHA256` in
`sysw/codex32_seam_test.go:30`. The corpus is 17 rows; the capture
`record_corpus_pre_s2.json` went 38 → 42 records.

## 3. REDs, quoted

**Task 1 Step 1** — `cargo nextest run -E 'test(a_hash_id_plate_with_a_short_x)'`,
1 test run: 0 passed, 1 failed (`t1s1-red.txt`), at
`crates/me-cli/tests/sysw_pack_preimage.rs:470`:

```
F-503: the length is not named: me: record 0 (records count from 0) is a kind-0x03 preimage payload whose 4-character id is not `hash`. A preimage plate is kind 0x03 under the id `hash` (SPEC_ms_hashlock rule 2), and --pack-preimage admits only that. Re-encode it with `ms hashlock` rather than editing the string. If this string is a 33-byte seed backup that happens to begin 0x03, it is not a preimage: roughly 1 in 256 of them look like this.
```

That is the plan's quoted expected body, verbatim — the two false things (the id
claim and the 1-in-256 sentence) both present.

**Task 1 Step 7** — the capture RED, taken deliberately by restoring the
pre-insert corpus, at `crates/me-cli/tests/record_corpus.rs:139`
(`t1s7-red.txt`):

```
assertion `left == right` failed: testdata/record_corpus_pre_s2.json is not the enumerated corpus
```

`left` (the enumerated corpus) held 42 entries including the four
`codex32_seam/hash-…` origins; `right` (the capture file) held 38 and stopped at
`codex32_seam/bip93-plain-33-byte-payload-0x03`.

**Task 2 Step 1** — after vendoring the corpus and re-pinning, before any
device change (`go test ./sysw/ -run TestCodex32Seam -count=1`, `t2s1-red.txt`):

```
--- FAIL: TestCodex32SeamDeviceAdmitsEverythingTheHostDoes (0.00s)
    codex32_seam_test.go:66: hash-kind03-16-byte-x: device admits = true, want false (Classify = 2)
    codex32_seam_test.go:66: hash-kind03-31-byte-x: device admits = true, want false (Classify = 2)
    codex32_seam_test.go:66: hash-kind03-33-byte-x: device admits = true, want false (Classify = 2)
```

Exactly three rows, exactly the plan's text; the control row
`hash-id-kind00-16-byte` (device_admits `true`) passed, which is the whole point
of that row.

## 4. Mutations — each run once, then reverted

**M1 (me, Task 1 Step 5): `x_len` always `None` in the naming arm.**
`Err(ms_codec::Error::PreimageLengthMismatch { got }) => Some(got),` →
`{ got: _ }) => None,`. Result: **2 tests run, 0 passed, 2 failed**
(`t1s5-mutation.txt`):

- `crates/me-cli/src/sysw/mod.rs:984`:
  `left: Err(Unclassifiable(0, PreimagePlate { id_is_hash: true, x_len: None }))`
  vs `right: Err(Unclassifiable(0, PreimagePlate { id_is_hash: true, x_len: Some(16) }))`
- `crates/me-cli/tests/sysw_pack_preimage.rs:470`: the first assertion, with the
  wrong-id body quoted in §3.

Reverted; both PASS.

**M2 (me, Task 1 Step 6): the old seam sha.** Restoring
`2c2fbb3fa4d38c8858b9de4769d876d275478956c76ca491005c70d9f6bd541b`. Result: 1
run, 0 passed, 1 failed at `crates/me-cli/tests/codex32_seam.rs:33`
(`t1s6-mutation.txt`):

```
assertion `left == right` failed: testdata/codex32_seam_vectors.json is not the file the fork's copy is pinned to; re-pin BOTH literals
```

Reverted; PASS.

**M3 (fork, Task 2 Step 2): `len(d) == 33 &&` in `IsPreimageKind`.** Edited BY
FUNCTION (the plan's named hazard: `IsPreimage` and `IsPreimageKind` end in
textually identical lines — the replacement was applied only to the slice of the
file starting at `func IsPreimageKind`, and `git diff` before restoring showed a
single `+` line inside the new function and no change to `IsPreimage`). Result
(`t2s2-mutation.txt`), four rows at `codex32/mspayload_test.go:290`:

```
17 bytes under hash (F-503): IsPreimageKind = false, want true
32 bytes under hash: IsPreimageKind = false, want true
34 bytes under hash: IsPreimageKind = false, want true
17 bytes under test (H0 keeps it a seed): IsPreimageKind = false, want true
```

Reverted, and both functions re-read to confirm the restore landed in the right
one: `IsPreimage` → `return len(d) == 33 && d[0] == msPrefixPreimage`;
`IsPreimageKind` → `return len(d) > 0 && d[0] == msPrefixPreimage`.

**M4 (fork, Task 2 Step 3): drop `&& !isHashIdPreimageKind(c)`.** Result: Step
1's three-row FAIL returns verbatim (`t2s3-mutation.txt`), same three names, same
`device admits = true, want false (Classify = 2)`. Reverted; `./codex32/`
and `./sysw/` both `ok`.

## 5. Inline verification beyond the plan's steps

**All three refusal bodies are reachable and each says only what is true**, run
against the built binary at the tip:

- `ms10hashsqvqqqqqqqqqqqqqqqqqqqqqqqqqqmv3lqlgkn6s5c` (the F-503 string) →
  `(true, Some(16))`: "…under the id `hash` whose X is 16 bytes, not 32. … This
  string is a damaged or hand-built plate: re-encode it with `ms hashlock` from
  the phrase or the 32-byte preimage…". No id claim, no 1-in-256 sentence.
- `MS10HASHSQVQQQQQQQQQQQQQQQQQQQQQQQQQQMV3LQLGKN6S5C` → `(false, Some(16))`:
  "…whose 4-character id is not `hash` and whose X is 16 bytes, not 32…". This
  proves the middle arm is not dead code — the id is read case-sensitively from
  bytes 4..8, so the uppercase spelling reaches it.
- `ms10testsqvrsu9guyv4rzwplgex4gkmzd9c8wl593jfe4gdg47mtm3xt6tv7qh3pm4xrfdlvvp`
  → `(_, None)`: the §4.3 collision case, keeping the 1-in-256 sentence, which is
  the only shape it is true of.

**F-504 is untouched and still open**, as intended: the UPPERCASE plate reads
`id_is_hash: false, x_len: None`, so its body is byte-identical to before.

## 6. Deviations

**D1 — baselines moved; branched from current heads as the brief directs.** The
plan's baselines are engrave `db37ec79` and ms `a994a99`; the actual heads were
`4df46e22` and `cacf5da`. Both plan baselines were confirmed ANCESTORS
(`git merge-base --is-ancestor`), the intervening commits are records only
(engrave: the F-503 plan/brief commits; ms: one push record). Fork main was
exactly the plan's `3cadffa8`. No content consequence.

**D2 — Task 1 Step 1's RED was 1 test, not 2.** The plan predicts "2 tests run,
0 passed, 2 failed — the unit test in Step 3 reds too", but at Step 1 the unit
test's I-3 row has not yet been rewritten, so the filter
`test(a_hash_id_plate_with_a_short_x)` matches one test. The plan's figure
describes the gate's measurement with Step 3 already in place; the 2-fail
observation is recorded here at M1, where both tests exist and both die.

**D3 — Task 1 Step 2's site counts are off by one in categorisation.** The plan
says "three `assert!(matches!(...))` sites … three `assert_eq!` sites". The file
has SIX `PreimagePlate` test sites: **two** `assert!(!matches!(…))` (which took
`{ .. }`) and **four** `assert_eq!` — the three concrete ones the plan names (the
`test`-id 33-byte row twice, and the UPPERCASE plate, all
`{ id_is_hash: false, x_len: None }`) plus the I-3 row, which Step 3 rewrites to
`{ id_is_hash: true, x_len: Some(16) }`. The end state is exactly what the plan
specifies; only the tally in the prose was wrong.

**D4 — Task 1 Step 7's left/right are the other way round.** The plan says "left
38, right 42"; measured, `left` is the enumerated corpus at 42 and `right` is the
capture file at 38. Direction only.

**D5 — the ms spec sentence was placed after the (a)/(b) sentence, not inside
it.** The plan says append the sentence where §9 says `isStrictMs1` "makes a
`0x03` string INERT (…)". That clause is item (a) of an `(a) … ; (b) …` list, so
appending there splits the list mid-sentence (tried, and it read as a broken
enumeration). The sentence is verbatim as the plan writes it, placed immediately
after the (a)/(b) sentence ends and before "This reorders 4.5's sequence". §12
item 7 took the plan's parenthetical verbatim.

**D6 — one extra commit, `cb0628e7`, filing F-506.** While transcribing the
plan's `isHashIdPreimageKind` comment I measured its claim that the UPPERCASE
spelling "is refused elsewhere (H6 §4.3) and never reaches a seed class". It is
FALSE for the shape F-503 is about. Measured on the fork tip with a throwaway
probe in `sysw` (run, then deleted; the tree is unchanged and clean):

```
"ms10hashsqvqqqqqqqqqqqqqqqqqqqqqqqqqqmv3lqlgkn6s5c" -> Classify = 0   (fixed)
"MS10HASHSQVQQQQQQQQQQQQQQQQQQQQQQQQQQMV3LQLGKN6S5C" -> Classify = 2   (a SEED class)
"ms10hashsqw46h2at4w46…kzv2ncy60u7z9c"               -> Classify = 13  (ClassPreimage)
"MS10HASHSQW46H2AT4W46…KZV2NCY60U7Z9C"               -> Classify = 0
"ms10testsqv0qqqqqqqqqqqqqqqqqqqqqqq8mzk8tjfdnjn5"   -> Classify = 2   (H0's pinned row)
```

The claim is true of the uppercase PLATE (`0`, because `IsPreimage` reads
`Seed()` and is case-insensitive) and false of the uppercase SHORT-X form (`2`).
The host refuses that string by name (§5), so it is the same host/device
divergence F-503 closed, surviving in the other case.

**No code was changed for it, and it does not gate this plan:** the operator's
ruling is stated as the id `hash`, and the plan deliberately reads the id
case-sensitively to mirror `codex32.IsPreimagePlate`, so the BEHAVIOUR is what
was specified. What is wrong is the comment's justification. Minimum fix: correct
the comment. Wider fix, the operator's call: compare the id case-insensitively
(and decide whether `IsPreimagePlate` follows), Rust-first. Distinct from F-504,
which is host-side wording for the uppercase plate; this is a device-side seed
class. Filed rather than fixed because the fix is a judgement about the ruling's
scope, and amending `b32ff08` would have invalidated the SHA already recorded in
three other commits.

## 7. Files touched

Exactly the plan's File structure, plus `design/FOLLOWUPS.md` a second time for
D6:

- me `277c6336`: `crates/me-cli/src/sysw/mod.rs`, `crates/me-cli/src/main.rs`,
  `crates/me-cli/tests/sysw_pack_preimage.rs`,
  `crates/me-cli/testdata/codex32_seam_vectors.json`,
  `crates/me-cli/tests/codex32_seam.rs`,
  `crates/me-cli/testdata/record_corpus_pre_s2.json`,
  `crates/me-cli/CHANGELOG.md` (7 files).
- fork `b32ff08`: `codex32/mspayload.go`, `codex32/mspayload_test.go`,
  `sysw/classify.go`, `sysw/codex32_seam_test.go`,
  `sysw/testdata/codex32_seam_vectors.json` (5 files).
- ms `81d67b85`: `design/SPEC_ms_hashlock.md`.
- me `6ea358db`: `design/FOLLOWUPS.md`, `design/CONTINUITY_composer_2026-09-01.md`.
- me `cb0628e7`: `design/FOLLOWUPS.md`.

## 8. For the post-impl check (plan §4)

- Each new test reds under its named mutation on the real branches: §4, four
  mutations, each with its failing file:line.
- The seam file is byte-identical in both repos and both pins equal its sha256:
  §2, `cmp` plus two `sha256sum`s.
- The capture matches the enumerated corpus: `the_capture_is_the_whole_corpus`
  PASS at the tip, 42 records.
- **Read F-506 (§6/D6) before merging the fork branch** — it names a comment in
  `sysw/classify.go` that is measurably false as shipped.
