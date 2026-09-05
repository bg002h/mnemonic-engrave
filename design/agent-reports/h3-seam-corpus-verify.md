# H3 records verify — `seam-corpus` (F-475 closure)

**Role:** sonnet, read-only verification. No branches created, no commits made, nothing pushed. Verified against the drafter's own worktrees (did not create new ones): engrave `/scratch/code/shibboleth/me-worktrees/h3-seam-corpus`, fork `/scratch/code/shibboleth/.tmp/seedhammer-h3-seam-corpus`.

**Draft under review:** `design/agent-reports/h3-seam-corpus-draft.md` (engrave branch `h3-seam-corpus` tip `29499b148f65bc6ec834c6bc0864a8d671afc266` on master `68aae89`; fork branch `h3-seam-corpus` tip `245eee1a53fd16a56e2c06cd9bfcd7dd7568282e` on fork main `c4a64fc`).

## Tips and scope — confirmed exact

```
$ git -C .../me-worktrees/h3-seam-corpus rev-parse HEAD
29499b148f65bc6ec834c6bc0864a8d671afc266
$ git -C .../.tmp/seedhammer-h3-seam-corpus rev-parse HEAD
245eee1a53fd16a56e2c06cd9bfcd7dd7568282e
```
Both match the brief exactly. `git diff 68aae89..HEAD --stat` (engrave): 3 files, 64(+)/3(-) — `crates/me-cli/testdata/codex32_seam_vectors.json`, `crates/me-cli/tests/codex32_seam.rs`, `design/FOLLOWUPS.md`. `git diff c4a64fc..HEAD --stat` (fork): 2 files, 2(+)/2(-) — `sysw/codex32_seam_test.go`, `sysw/testdata/codex32_seam_vectors.json`. Read every changed line in both diffs (full `git diff` above, not just stat) — nothing outside the claimed scope.

## Table — claim, true/false, evidence

| # | Claim | T/F | Evidence |
|---|---|---|---|
| 1 | Row `bip93-plain-33-byte-payload-0x03`, `source` field, at `codex32_seam_vectors.json:154` (row's `name` at :149) | TRUE | `grep -n "THE COLLISION"` → `154:`; row's `"name"` line is `149:` |
| 2 | Old wording was `(ms-codec 0.7 at the prefix gate, 0.8 as a TagKindMismatch)` | TRUE | seen verbatim in the `-` side of the diff |
| 3 | New wording is exactly F-475's suggestion, ASCII `--` for the em dash | TRUE | `design/FOLLOWUPS.md:15742` entry's suggestion reads `... 0.8 as an \`UnknownTag\` — the shape predicate ...`; committed text at `:154` reads `... 0.8 as an \`UnknownTag\` -- the shape predicate ...` — same words, `--` on disk |
| 4 | Corpus file is pure ASCII (no non-ASCII byte) | TRUE | `LC_ALL=C grep -n '[^ -~]' codex32_seam_vectors.json` → no output (exit 0, empty); `file` → `JSON text data` |
| 5 | The string decodes at ms-codec 0.8.0 to `UnknownTag "test"` | TRUE — **independently reproduced**, not just re-derived from the report | Built a fresh throwaway crate (`/scratch/code/shibboleth/.tmp/verify-h3-seam-corpus`) depending on `path = ".../mnemonic-secret/crates/ms-codec"` (confirmed `version = "0.8.0"` in that Cargo.toml, at `mnemonic-secret` HEAD `504ff46a7acf31cca59e58c50fc9fd76ca3604b9`), fed it the row's exact 75-char string, ran `cargo run --quiet`: `Err(Error("unknown tag \"test\"; not a member of RESERVED_TAG_TABLE"))`, `Display: unknown tag "test"; not a member of RESERVED_TAG_TABLE`. len=75 confirmed independently in Python too. |
| 6 | `consts.rs:33`/`:45` hold `VALID_STR_LENGTHS`/`VALID_PREIMAGE_STR_LENGTHS` incl. 75 | TRUE | `grep -n`: `33:pub const VALID_STR_LENGTHS: &[usize] = &[50, 56, 62, 69, 75];`, `45:pub const VALID_PREIMAGE_STR_LENGTHS: &[usize] = &[75];` |
| 7 | `decode.rs:48`/`:63` are the two length gates, neither fires | TRUE | `:48` = `if !is_known_length(s.len()) {`, `:63` = `if !kind_allowed.contains(&s.len()) {` — 75 is in both allowed sets so neither returns |
| 8 | `consts.rs:71` = `RESERVED_NOT_EMITTED_V01`, doesn't contain `test`; `decode.rs:71` is that gate | TRUE | `71:pub const RESERVED_NOT_EMITTED_V01: &[[u8; 4]] = &[*b"seed", *b"xprv", *b"prvk"];` (no `test`); `decode.rs:71` = `if RESERVED_NOT_EMITTED_V01.contains(tag.as_bytes()) {` |
| 9 | `decode.rs:86` tag match, `:91` is the rule-6b `TagKindMismatch` guard (only fires for `entr`/`hash`), `:126` is the `_ => UnknownTag` catch-all | TRUE | `86: let payload = match *tag.as_bytes() {`; `91:` guard is `x if (x == TAG_ENTR \|\| x == TAG_HASH) && tag != payload.kind().single_tag() => {`; `126:` is `_ => { return Err(Error::UnknownTag { got: *tag.as_bytes() }); }` |
| 10 | `error.rs:51`/`:67` define `UnknownTag`/`TagKindMismatch`; `:206`/`:220` are their `Display` arms | TRUE | `grep -n`: `51:UnknownTag {`, `67:TagKindMismatch {`, `206: Error::UnknownTag { got } => write!(...)`, `220: Error::TagKindMismatch { tag, prefix } => write!(...)` |
| 11 | ms-codec 0.7.0 spans commit `853a6ed`; a `0x03` prefix hits `ReservedPrefixViolation`, documented `envelope.rs:117`, raised `:216` | TRUE | `git log -S'version = "0.7.0"'` → `853a6ed release(ms): P5 — ms-codec 0.7.0 …`; `git show 853a6ed:.../Cargo.toml` → `version = "0.7.0"`; that revision's `envelope.rs:117` = `/// - any other prefix → Err(Error::ReservedPrefixViolation)`; `:216` = `return Err(Error::ReservedPrefixViolation { got: other });` |
| 12 | Row's verdict never moved: `host_admits:false`, `device_admits:false` | TRUE | Both fields untouched in the diff — only the `source` line changed |
| 13 | Host claim checked against code: `record.rs:287` = `preimage_plate`, short-circuits on id/kind mismatch + `PreimageLengthMismatch`, then `d.len()==33 && d[0]==0x03` at `:310-320`; doc comment `:284-286` corroborates the 0.7 clause verbatim | TRUE | `grep -n`: `287:pub fn preimage_plate(s: &str) -> bool {`, `293: if id_kind_mismatch(s) {`, `317: d.len() == 33 && d[0] == 0x03` (inside `:310-320`); lines `284-286` read exactly *"(H1b, F-473): 0.7 refused the kind with `ReservedPrefixViolation { got: 3 }`, 0.8 decodes it or names its length; both answer `true` here."* |
| 14 | Test cited, `sysw/mod.rs:869`, `a_preimage_plate_is_named_not_misdiagnosed` | TRUE | `grep -n` → `869: fn a_preimage_plate_is_named_not_misdiagnosed() {` |
| 15 | Re-pin `bb703f60…ac78b` → `2c2fbb3f…d541b` at `codex32_seam.rs:26` and `sysw/codex32_seam_test.go:30`; both testdata copies byte-identical at the new hash | TRUE | Diff shows exactly this substitution at both line numbers; `sha256sum` on both worktrees' `testdata/codex32_seam_vectors.json` independently → `2c2fbb3fa4d38c8858b9de4769d876d275478956c76ca491005c70d9f6bd541b` for both; `diff -q` → files identical |
| 16 | Recipe steps 1-4 at `codex32_seam_vectors.json:41,42,43,46` | TRUE | `grep -n` → exactly those four line numbers for steps 1/2/3/4 |
| 17 | JSON still parses; 13 rows | TRUE | `python3 -c "json.load(...)"` → `len(d['vectors']) == 13` |
| 18 | Exactly 4 files carry the FULL old hash as a historical record (not a live pin), at the exact lines cited: `IMPLEMENTATION_PLAN_hashlock_H0_reader_guards.md:3`, `CONTINUITY_composer_2026-09-01.md:1918`, `agent-reports/hashlock-H0-post-impl-r1-fold-verification.md:72,73`, `agent-reports/hashlock-H0-implementation-report.md:599` | TRUE | `grep -rln` for the full 64-hex-char string over the post-edit worktree (excluding the draft's own report, which didn't exist pre-edit) returns exactly these 4 files; per-line `grep -n` matches the 4 cited line numbers exactly. (Two *other* files reference the old hash in **abbreviated** `bb703f60…` prose — `FOLLOWUPS.md:15806,15826` (the closure's own re-pin narrative, newly written) and `hashlock-H0-post-impl-r1-fold-verification.md:37` / `hashlock-H0-implementation-report.md:604` (pre-existing abbreviated mentions in the same two files already counted) — none of these is a full-length literal needing a re-pin, so the "exactly 4 historical, exactly 2 live" count is correct.) |
| 19 | `descriptor_seam.rs` declares its own `SEAM_VECTORS_SHA256` at `:45` pinning a different file (`e7a4160c…758`), untouched | TRUE | `grep -n` → `45:const SEAM_VECTORS_SHA256: &str =` / `48:const PATH: &str = "testdata/descriptor_seam_vectors.json";`, value `e7a4160ce064a6cb7ca31dc530e079c861cf2c8a075d75f793ef0d935f583758`; file absent from the diff |
| 20 | Fork's `nonstandard/descriptor_seam_test.go:42` likewise untouched, same different hash | TRUE | `grep -n` → `42:const seamVectorsSHA256 = "e7a4160c…758"`; file absent from the diff |
| 21 | F-475 marked CLOSED in the **heading** at `FOLLOWUPS.md:15742`, per the file's own stated convention | TRUE | `grep -n "F-475"` → `15742: ### F-475 — ~~\`seam-corpus-…\`~~ **CLOSED 2026-09-05** — …`; `design/FOLLOWUPS.md:5` = `## Convention — a follow-up's STATUS lives in its heading` |
| 22 | Merge note: branch `hashlock-h2` carries the OLD corpus + OLD pin at both `17b3979` (brief's cited rev) and its current tip `a1fd139` (2 commits later, neither touching the seam files) | TRUE | `git merge-base --is-ancestor 17b3979 HEAD` (in the H2 worktree) → true; `git log --oneline 17b3979..HEAD` → `26fd1dd`, `a1fd139`, neither's `--stat` mentions `seam`; `git show 17b3979:sysw/codex32_seam_test.go` line 30 and the worktree's current line 30 both read `bb703f60…ac78b`; both the historical blob and the live worktree file hash to `bb703f60…ac78b` |
| 23 | engrave gate: `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast` → 619 run / 616 passed / 3 failed / 2 skipped; `codex32_seam::…` PASSES; the 3 failures are the box-local `history_purge` trio, all at the same `/usr/bin/zsh is required` precondition | TRUE — **independently re-run**, own `CARGO_TARGET_DIR` | `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast` in the same worktree, `CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/verify-h3-seam-corpus-engrave-target` → `619 tests run: 616 passed, 3 failed, 2 skipped`; the 3 named failures are identical (`editing_the_file_alone_is_the_trap_the_message_warns_about`, `the_harness_records_history_at_all`, `the_emitted_zsh_recipe_actually_purges_the_entry`), each panicking at `history_purge.rs:35:5` with the same `/usr/bin/zsh is required` message; `ls /usr/bin/zsh` → No such file or directory on this box too; `codex32_seam::the_host_never_admits_what_the_device_would_refuse` → PASS |
| 24 | fork gate: `go test ./codex32/... ./sysw/...` → both `ok`; `go test ./sysw/ -run TestCodex32Seam -v -count=1` → PASS (uncached) | TRUE — **independently re-run** | `go1.26.7` first on PATH, in the same worktree: `go test ./codex32/... ./sysw/...` → `ok seedhammer.com/codex32`, `ok seedhammer.com/sysw`, exit 0; `go test ./sysw/ -run TestCodex32Seam -v -count=1` → `=== RUN TestCodex32SeamDeviceAdmitsEverythingTheHostDoes` / `--- PASS` / `ok` |
| 25 | Commits end with the two required trailer lines; fork commit carries exactly one `Signed-off-by` (a duplicate was caught and fixed) | TRUE | `git log -1 --format=%B` on all three commits (`743da17`, `29499b1`, `245eee1`) — both engrave commits end `Co-Authored-By: Claude Fable 5.1 <noreply@anthropic.com>` / `Claude-Session: https://claude.ai/code/session_01Fs3bg7TRfuSaFcCEkskwXA`; fork commit has `Signed-off-by: Brian Goss <goss.brian@gmail.com>` then the same two trailers; `grep -c "^Signed-off-by"` on the fork message → 1 |
| 26 | Nothing pushed; `master`/`main` untouched; worktrees clean | TRUE | `git ls-remote origin refs/heads/h3-seam-corpus` empty on both engrave and fork remotes; `git log -1 --oneline master` (engrave) → `68aae89`, `git log -1 --oneline main` (fork) → `c4a64fc` (both pre-branch tips, unchanged); `git status --short` empty in both worktrees |

## Gate re-runs (this agent, own scratch dirs)

Engrave (`CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/verify-h3-seam-corpus-engrave-target`):
```
Summary [0.363s] 619 tests run: 616 passed, 3 failed, 2 skipped
PASS (311/619) mnemonic-engrave::codex32_seam the_host_never_admits_what_the_device_would_refuse
FAIL mnemonic-engrave::history_purge editing_the_file_alone_is_the_trap_the_message_warns_about
FAIL mnemonic-engrave::history_purge the_harness_records_history_at_all
FAIL mnemonic-engrave::history_purge the_emitted_zsh_recipe_actually_purges_the_entry
  -> panicked at crates/me-cli/tests/history_purge.rs:35:5: "/usr/bin/zsh is required: ..."
```
(Test-index numbers in the PASS/FAIL lines differ trivially from the draft's capture — nextest runs in parallel and indices are execution-order artifacts, not a discrepancy in counts or identities.)

Fork (go1.26.7 first on PATH):
```
$ go test ./codex32/... ./sysw/...
ok  	seedhammer.com/codex32	(cached)
ok  	seedhammer.com/sysw	(cached)
EXIT=0
$ go test ./sysw/ -run TestCodex32Seam -v -count=1
=== RUN   TestCodex32SeamDeviceAdmitsEverythingTheHostDoes
--- PASS: TestCodex32SeamDeviceAdmitsEverythingTheHostDoes (0.00s)
PASS
ok  	seedhammer.com/sysw	0.002s
```

Independent throwaway-crate check (this agent's own, in `/scratch/code/shibboleth/.tmp/verify-h3-seam-corpus`, `ms-codec` path-dependency at `mnemonic-secret` `504ff46`):
```
bip93-plain-33-byte-payload-0x03 (id test) len=75
bip93-plain-33-byte-payload-0x03 (id test): Err(Error("unknown tag \"test\"; not a member of RESERVED_TAG_TABLE"))
Display: unknown tag "test"; not a member of RESERVED_TAG_TABLE
```

## Verdict

**GREEN.** Every claim checked in the draft (`h3-seam-corpus-draft.md`) and in F-475's closure text (`design/FOLLOWUPS.md:15742`) is TRUE of the branches at their stated tips — the prose replacement, every ms-codec/host-side file:line citation, the re-pin values and file-identity, the historical-vs-live hash enumeration, the FOLLOWUPS closure mechanics, the merge-conflict note against `hashlock-h2`, and both gates. Both gates were independently re-run in this agent's own scratch target dirs and reproduced exactly (619/616/3/2 with the identical 3 box-local `zsh`-absence failures on engrave; `ok`/`ok` plus an uncached PASS on the fork). The core empirical claim — that ms-codec 0.8.0 names this string `UnknownTag`, not `TagKindMismatch` — was independently reproduced from scratch, not merely re-derived from the report's numbers. No false claims found. Nothing pushed; `master`/`main` untouched; both worktrees clean.

No findings to fold. This item is safe to leave closed as the draft left it.
