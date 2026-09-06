# F-503 + F-506 post-implementation check (independent, sonnet tier)

Brief: `design/agent-briefs/f503-post-impl-check-brief.md` (including its
Dispatch addendum, which supersedes the plan's seam sha and inverts mutation
2(a)). Scope: F-503 AND F-506, per the addendum. Branches checked at their
stated tips: me `f503` `44ae864f` (base cited `cc88001b`), fork `f503`
`d5d331d0` (base `main` `3cadffa8`), ms `f503-records` `81d67b85` (base
`a994a99`). Read-only on all three source repos; all work done in detached
worktrees (`me-worktrees/f503-check`, `.tmp/seedhammer-f503-check`,
`.tmp/ms-f503-records-check`), all three restored to clean (`git status
--porcelain` empty) and removed after this check. No `.jsonl` read. No commits.

## 1. Diff vs plan's File structure table

**me and ms's cited "base" commits are not actual ancestors of the branch
tips** — `cc88001b` is a sibling of the `f503` branch (both descend from the
true common ancestor `4df46e22`, confirmed by `git merge-base`), and ms's
`a994a99..81d67b85` crosses one intervening master commit (`cacf5da`, a push
record, also an ancestor of current ms `master`). Diffing against the stated
bases therefore picks up unrelated files (`design/agent-briefs/*.md` on me;
`design/agent-reports/push-ms-a994a99.md` on ms) that master/master
independently gained — not implementation scope creep. **Diffing against the
true branch points instead:**

- me `4df46e22..44ae864f`: `crates/me-cli/{CHANGELOG.md,src/main.rs,
  src/sysw/mod.rs,testdata/codex32_seam_vectors.json,
  testdata/record_corpus_pre_s2.json,tests/codex32_seam.rs,
  tests/sysw_pack_preimage.rs}`, `design/{CONTINUITY_composer_2026-09-01.md,
  FOLLOWUPS.md}` — **exactly** the plan's Task 1 (7 files) + Task 3 engrave row
  (2 files), 9/9, none missing, none extra.
- fork `3cadffa8..d5d331d0` (ancestry confirmed clean): `codex32/mspayload.go`,
  `codex32/mspayload_test.go`, `sysw/classify.go`, `sysw/codex32_seam_test.go`,
  `sysw/testdata/codex32_seam_vectors.json` — **exactly** the plan's Task 2
  table, 5/5.
- ms `cacf5da..81d67b85` (ancestry confirmed clean): `design/SPEC_ms_hashlock.md`
  — **exactly** the plan's Task 3 ms row, 1/1.

No file outside any table; no table file untouched. (Minor/Nit: the dispatch's
cited "base" SHAs for me and ms are stale citations, not diff defects — noted
so a future check doesn't repeat the raw `cc88001b`/`a994a99` diff and mistake
its noise for scope creep.)

## 2. Mutation table — every mutation re-run on the real branch tips, then reverted

| # | Repo:file | Mutation | Result | Match |
| --- | --- | --- | --- | --- |
| M1 | me `sysw/mod.rs` (Task 1 Step 5): `PreimageLengthMismatch{got}=>Some(got)` → `{got:_}=>None` | 2 tests run, 0 passed, 2 failed — unit test at `sysw/mod.rs:984` (`left: ...x_len: None` vs `right: ...x_len: Some(16)`) and integration test at `sysw_pack_preimage.rs:470` (quoted wrong-id body) | Exact match to plan/report |
| M2 | me `tests/codex32_seam.rs` (Task 1 Step 6): old sha `2c2fbb3f…` restored | 1 test run, 1 failed: `assertion \`left == right\` failed: testdata/codex32_seam_vectors.json is not the file the fork's copy is pinned to; re-pin BOTH literals` | Exact match |
| M3 | fork `codex32/mspayload.go` (Task 2 Step 2): `len(d) > 0` → `len(d) == 33` in `IsPreimageKind` only (confirmed `IsPreimage` untouched, before and after) | 4 rows FAIL at `mspayload_test.go:290`: 17/32/34-byte-under-hash and 17-byte-under-test, each "IsPreimageKind = false, want true" | Exact match |
| M4 | fork `sysw/classify.go` (Task 2 Step 3): drop `&& !isHashIdPreimageKind(c)` | 4 rows FAIL (the seam corpus's 3 original F-503 rows **plus** the F-506 uppercase row, all `Classify = 2`) — one more row than the plan's 3 because the corpus has grown by the F-506 fold; same mechanism, expected | Superset of plan's 3, correctly explained |
| a | fork `sysw/classify.go`: addendum's inverted check — `strings.EqualFold(id,"hash")` (SHIPPED) → `id == "hash"` | Reds **exactly** `hash-kind03-16-byte-x-uppercase`, nothing else | Exact match to addendum |
| b | fork `codex32/mspayload.go`: `IsPreimageKind` drops the `Unshared` test (`f,err:=ParsePrefix(...); if err!=nil{...}`) | 1 row FAILs: "a 2-of-N share beginning 0x03: IsPreimageKind = true, want false" | Matches check item 2(b) |
| c | me `sysw/mod.rs`: `id_is_hash` hard-coded `true` | 1 test FAILs — `a_preimage_plate_is_named_not_misdiagnosed` at `sysw/mod.rs:953` (the wrong-id 33-byte unit-test row: `left: ...id_is_hash:true...` vs `right: ...id_is_hash:false...`); the F-503 integration test does NOT catch it (its own string's true id IS `hash`, so the mutation is a no-op there) | Matches check item 2(c); caught by a different test than the F-503 one, correctly |

Every mutation reverted; each repo's own suite (below) re-confirmed green
after every revert.

## 3. Seam corpus parity

```
sha256sum crates/me-cli/testdata/codex32_seam_vectors.json (me f503-check)
  a669e10f7936478f4ec1d17f417864aab6cf5c48e3824591d4c73f9c1f3b7bd9
sha256sum sysw/testdata/codex32_seam_vectors.json (fork f503-check)
  a669e10f7936478f4ec1d17f417864aab6cf5c48e3824591d4c73f9c1f3b7bd9
cmp <me file> <fork file>  ->  IDENTICAL (no output, exit 0)
```

Both pinned literals (`SEAM_VECTORS_SHA256` in `crates/me-cli/tests/codex32_seam.rs:26`,
`seamVectorsSHA256` in `sysw/codex32_seam_test.go:30`) equal the same 64-hex-char
value above — matches the addendum's superseding value, not the plan's stale
`f53a17dc…`.

Row check (measured `len(string)` vs the `chars` field), all five F-503/F-506
rows:

| row | chars field | actual length | host_admits | device_admits |
| --- | --- | --- | --- | --- |
| `hash-kind03-16-byte-x` | 50 | 50 | false | false |
| `hash-kind03-31-byte-x` | 74 | 74 | false | false |
| `hash-kind03-33-byte-x` | 77 | 77 | false | false |
| `hash-id-kind00-16-byte` (control) | 50 | 50 | false | **true** |
| `hash-kind03-16-byte-x-uppercase` | 50 | 50 | false | false |

`bip93-plain-payload-0x03` (the seam row the rule must NOT flip) still reads
`device_admits: true`. Corpus total: 18 rows (was 17 pre-F-506).

Capture (`record_corpus_pre_s2.json`): 43 records (was 42 pre-F-506, 38
pre-F-503); the five `codex32_seam/hash-*` origin entries are present, in
order, immediately after `codex32_seam/bip93-plain-33-byte-payload-0x03`.
`cargo nextest run -E 'test(the_capture_is_the_whole_corpus)'` → PASS.

## 4. Refusal texts, from the built binary at me tip `44ae864f`

`cargo build --locked -p mnemonic-engrave --bin me`; every body below is a
byte-for-byte match (verified with a whitespace-normalized string-containment
check against the plan's `format!` templates) to what the plan's Task 1 Step 4
block specifies for its `(id_is_hash, x_len)` arm.

| string | via | exit | first stderr line (paraphrased where long) |
| --- | --- | --- | --- |
| F-503 string, 50 chars (`ms10hashsqvqq…s5c`) | argv | 4 | "...under the id `hash` whose X is 16 bytes, not 32.... re-encode it with `ms hashlock`..." — arm `(true, Some(16))` |
| `hash-kind03-31-byte-x`, 74 chars | argv | 4 | **Different arm than expected**: this string is NOT one of the `ms1` profile's valid widths ([50,56,62,69,75]/[51,58,64,70,77] — 74 is absent), so it never reaches `preimage_plate()`; it is refused earlier by the profile gate: "is a VALID BIP-93 codex32 string — the checksum is good — but not a constellation `ms1` record ... This one is 74 characters." No false id claim, no 1-in-256 sentence, one refusal (`records count from 0` occurs once). Consistent with the plan's own row commentary ("one byte short of a plate, at the 74-character width of a plain BIP-93 256-bit secret") — the plan picked this width deliberately, and both host and device still refuse it (device `Classify=0`, confirmed in §5) — see note below. |
| `hash-kind03-33-byte-x`, 77 chars | argv | 4 | "...under the id `hash` whose X is 33 bytes, not 32...." — arm `(true, Some(33))` |
| wrong-id 33-byte row (`ms10testsqvrs…lvvp`) | argv | 4 | "...whose 4-character id is not `hash`.... roughly 1 in 256 of them look like this." — arm `(_, None)`, the ONLY row where the 1-in-256 sentence appears |
| UPPERCASE plate | argv | 3 | Intercepted by the **argv secret guard** (a genuine 33-byte preimage plate is bearer material and refused before parse, regardless of case) — re-run via `--in` to reach the classify path: exit 4, "...whose 4-character id is not `hash`.... roughly 1 in 256..." — arm `(_, None)`. This is the KNOWN, OPEN F-504 wording defect (the id argument is case-sensitive-true here even though bech32 is case-insensitive); explicitly out of scope per the addendum, not a new finding. |
| well-formed plate (lowercase) | argv | 3 → via `--in`: 0 | Argv-guard-refused on argv (same as above, correct — real secret material); via `--in`, **admitted** (packs, prints a digest) — confirms the fix did not newly refuse or newly admit anything it shouldn't |

No body ever claims the id is not `hash` when it IS; the 1-in-256 sentence
appears in exactly the one case named above; every run produced exactly one
`records count from 0` occurrence (checked programmatically for all four
argv-reachable strings and the UPPERCASE `--in` run).

**Note on `hash-kind03-31-byte-x`:** this is a genuine observation from
running the check, not in the implementer's own report (which tested only
three strings against the built binary). It does not violate the rule or the
checklist's assertions — the string is refused on both host and device, just
via a different (also-true) message. Not a finding; flagged for the next
reader's benefit only.

## 5. Device classification — six strings, fork tip `d5d331d0`

Via a temporary `go test` (`sysw/zzf503check_test.go`, written, run, and
deleted; tree confirmed clean after):

| string | Classify | meaning | expected |
| --- | --- | --- | --- |
| F-503 string (hash-id, 16-byte X) | 0 | ClassUnknown | ClassUnknown ✓ |
| hash-kind03-31-byte-x (74 chars) | 0 | ClassUnknown | ClassUnknown ✓ |
| hash-kind03-33-byte-x (77 chars) | 0 | ClassUnknown | ClassUnknown ✓ |
| wrong-id 33-byte / kind-0x00-under-hash control | 2 | ClassCodex32Secret | ClassCodex32Secret ✓ |
| well-formed plate | 13 | ClassPreimage | ClassPreimage ✓ |
| H0 row (`ms10testsqv0…`) | 2 | ClassCodex32Secret | ClassCodex32Secret, STILL, unchanged ✓ |

All six match. `codex32.IsPreimagePlate` confirmed unchanged and still
case-sensitive (`id == "hash"`, `codex32/mspayload.go:178`) — the UPPERCASE
plate is not admitted anywhere (host: argv-guard/`--in` refusal above; device:
`Classify=0`, not `ClassPreimage`/`ClassCodex32Secret`). No Critical of the
"uppercase plate now admitted" shape found.

## 6. Suites

**me**, `f503-check` worktree, `CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/f503-check-target`:
- `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast` → `634 tests
  run: 634 passed, 2 skipped` (exit 0) — matches the addendum's controller
  measurement exactly, zsh trio included and green.
- `cargo fmt --all -- --check` → exit 0, no output.
- `cargo clippy --locked --all-targets -p mnemonic-engrave` → exactly one
  distinct warning, the pre-existing `manual_is_multiple_of` at
  `crates/me-cli/src/sysw/composer_records.rs:177` (grep count of `^warning:`
  lines is 3, but two of those are "generated N warning" summary lines for the
  same single warning across lib/lib-test builds — confirmed by reading the
  full output, not just the count).

**fork**, `.tmp/seedhammer-f503-check` worktree, Go 1.26.7 from
`/scratch/code/shibboleth/.toolchain/go/bin`:
- `gofmt -l codex32/ sysw/` → empty.
- `go test $(go list ./... | grep -v /gui$) -count=1` → 54 `ok` lines (counted:
  `grep -c '^ok'` = 54), no failures.
- `scripts/gui-shard-test.sh ./gui/ 24` → `partition verified exhaustive: 1289
  == 1289`; `RESULT: ok -- all 1289 tests ran across 24 shards`, wall 24s.
- `go vet ./...` → red at baseline (backup/gui `testing.ArtifactDir requires
  go1.26 or later`) — F-494, settled, not a finding.

**ms**, `.tmp/ms-f503-records-check` worktree, tip `81d67b85`:
- `cargo nextest run --locked` → `562 tests run: 562 passed, 11 skipped` (a
  first attempt hit a transient `SIGKILL` during compile of one test binary —
  re-run succeeded cleanly; consistent with box memory pressure from
  concurrent builds during this check, not a code defect).

## 7. Records

- ms spec: the plan's Task 3 Step 1 sentence is present in
  `design/SPEC_ms_hashlock.md`, verbatim modulo markdown line-wrapping
  (confirmed programmatically with whitespace-normalized substring match); §12
  item 7's addition `(at every length under the id `hash`, F-503)` is present
  verbatim, immediately after "`sysw.Classify` is not `ClassCodex32Secret`".
- me `design/FOLLOWUPS.md`: F-503 entry reads `CLOSED 2026-09-06: me 277c6336,
  fork b32ff08, ms 81d67b85; rule = kind 0x03 under `hash` at every length
  (operator ruling), control row `hash-id-kind00-16-byte`.` — the three SHAs
  are the real per-task commits (Task 1/2/3), matching git log. F-506 entry is
  present and marked `CLOSED 2026-09-06 by the operator's ruling — the id is
  compared case-insensitively`, consistent with fork `d5d331d0` / me `10faf109`.
- me CHANGELOG: the F-503/F-506 entry is the first item under `## [Unreleased]`
  / `### Changed`, and its text now says "Five seam corpus rows" (updated from
  four for the F-506 fold) — consistent with the corpus's actual 18-row /
  43-record counts.
- Every count in the implementer's report that I could independently recompute
  matched: 634/634/2 (me suite), 54 non-gui packages, 1289 gui tests across 24
  shards, corpus 42→43 (F-506 added one more on top of the report's own 38→42),
  seam corpus 17→18 rows. No false count found.

## Findings

**0 Critical, 0 Important, 0 Minor requiring action, 2 Nit (informational only).**

- N-1 — the dispatch brief's cited "base" SHAs for me (`cc88001b`) and ms
  (`a994a99`) are not the branches' true ancestors/points of divergence;
  diffing against them picks up unrelated master-side files. Diffing against
  the true merge-base (me: `4df46e22`; ms: `cacf5da`) shows an exact,
  clean match to the plan's File structure table. No action needed; noted so
  a future reviewer diffs against the right point.
- N-2 — the seam row `hash-kind03-31-byte-x` (74 chars) does not exercise the
  new `PreimagePlate`-arm refusal text at all; it is refused earlier, by the
  `ms1`-profile-length gate, because 74 is not a valid profile width. Still
  correctly refused on both host and device (§4, §5); not a defect, just an
  observation for anyone expecting all four/five seam rows to share one
  refusal mechanism.

## GREEN

The F-503 and F-506 implementation across all three branches (me `44ae864f`,
fork `d5d331d0`, ms `81d67b85`) does exactly what the plan and the operator's
rulings specify. Every test the diffs add reds under its named mutation
(including the addendum's inverted EqualFold check and this reviewer's own
three additional mutations); the seam corpus is byte-identical in both repos
and both pins match; every number in the implementer's report that could be
independently recomputed was true. **GREEN — no Critical or Important
findings; ready to proceed to Ship (plan §4).**
