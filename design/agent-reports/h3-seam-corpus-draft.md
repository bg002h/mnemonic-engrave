# H3 records draft — F-475: the seam corpus's 33-byte collision row named the wrong ms-codec 0.8 error

**Item:** F-475 (`seam-corpus-33-byte-collision-row-names-the-wrong-0.8-error`), owning phase H2, drafted here as a stage-H3 RECORDS-ONLY change.
**Scope:** one `source` parenthetical in one corpus row, the two sha256 pins that editing the corpus forces, and the F-475 closure in `design/FOLLOWUPS.md`. No code, no test logic, no device behaviour, no verdict changed.

## Branches and tips

| repo | branch | base | tip |
| --- | --- | --- | --- |
| engrave (`/scratch/code/shibboleth/mnemonic-engrave`) | `h3-seam-corpus`, worktree `/scratch/code/shibboleth/me-worktrees/h3-seam-corpus` | `master` **`68aae89`** | **`29499b148f65bc6ec834c6bc0864a8d671afc266`** |
| fork (`/scratch/code/shibboleth/seedhammer`) | `h3-seam-corpus`, worktree `/scratch/code/shibboleth/.tmp/seedhammer-h3-seam-corpus` | fork main `c4a64fc` | **`245eee1a53fd16a56e2c06cd9bfcd7dd7568282e`** |

Engrave commits, in order:

- `743da17259e9edecef30376d710376bd14c390ba` — the prose fix + `SEAM_VECTORS_SHA256`.
- `29499b148f65bc6ec834c6bc0864a8d671afc266` — F-475 marked CLOSED in `design/FOLLOWUPS.md`.

**One base drift worth flagging:** the brief names engrave master as `d81714e`; at the moment the worktree was created `master` was **`68aae89`** ("continuity: ultracode lenses landed; lens fold at fork a1fd139; records e879123 on the branch"), and that is what this branch is based on. Nothing was pushed and no commit was made on `master` or `main`.

## What changed, and why

### 1. The prose (`crates/me-cli/testdata/codex32_seam_vectors.json`, row at line 149)

The row `bip93-plain-33-byte-payload-0x03` argues that its refusal is CONVERGENCE rather than a device narrowing, because the host `me` refuses the same string. It cited the mechanism as:

```
(ms-codec 0.7 at the prefix gate, 0.8 as a TagKindMismatch)
```

The 0.8 clause named the wrong error. Replaced, exactly as F-475 proposed:

```
(ms-codec 0.7 at the prefix gate; 0.8 as an `UnknownTag` -- the shape predicate, not the codec error, is what names it)
```

ASCII `--`, not the em dash F-475 renders the suggestion with: the corpus file contains no non-ASCII byte (`LC_ALL=C grep -n '[^ -~]'` returns nothing; `file` reports "JSON text data", not "Unicode text").

**Re-measured before it was written down**, rather than carried across from F-475's table. A throwaway crate (`/scratch/code/shibboleth/.tmp/f475-check`, carrying `mnemonic-secret`'s `rust-toolchain.toml`) calling `ms_codec::decode` on the row's own string, against ms-codec **0.8.0** in `mnemonic-secret` at `504ff46`:

```
$ cargo run --quiet
bip93-plain-33-byte-payload-0x03 (id test): Err(Error("unknown tag "test"; not a member of RESERVED_TAG_TABLE")) -- Display: unknown tag "test"; not a member of RESERVED_TAG_TABLE
```

And it is the only reachable outcome in that tree, by inspection of `mnemonic-secret/crates/ms-codec/src/`:

- the string is 75 characters, which is in **both** `VALID_STR_LENGTHS` (`consts.rs:33` — `&[50, 56, 62, 69, 75]`) and `VALID_PREIMAGE_STR_LENGTHS` (`consts.rs:45` — `&[75]`), so neither length gate fires (`decode.rs:48` pre-dispatch, `decode.rs:63` bound-to-kind);
- `test` is not in `RESERVED_NOT_EMITTED_V01` (`consts.rs:71` — `&[*b"seed", *b"xprv", *b"prvk"]`), so the rule-7 gate at `decode.rs:71` does not fire;
- the tag match at `decode.rs:86` routes anything that is neither `TAG_ENTR` nor `TAG_HASH` to `_ => Error::UnknownTag` at **`decode.rs:126`**, past the rule-6b `TagKindMismatch` arm at **`decode.rs:91`**, whose guard is `x if (x == TAG_ENTR || x == TAG_HASH) && tag != payload.kind().single_tag()` — it only tests tags that ARE `entr` or `hash`. The match spans `decode.rs:86-130`.

`TagKindMismatch` (`error.rs:67`, Display at `error.rs:220`) is therefore the error of the row **one over** — `preimage-shape-entr-id`, whose id IS `entr`. `UnknownTag` is `error.rs:51`, Display at `error.rs:206`.

**The 0.7 clause was checked too and kept unchanged.** ms-codec 0.7.0 spans `853a6ed`..`bd76cec^` in `mnemonic-secret` (found with `git log -S'version = "0.7.0"' -- crates/ms-codec/Cargo.toml`). At `853a6ed`, `src/envelope.rs` documents `any other prefix → Err(Error::ReservedPrefixViolation)` at `:117` and raises it at `:216`, so a `0x03` prefix byte is refused at the prefix gate.

**The row's VERDICT never moved** and was never wrong: `host_admits: false`, `device_admits: false`.

The host-side half of the new wording was checked against the code, not just against F-475: `seal::record::preimage_plate` (`crates/me-cli/src/seal/record.rs:287`) short-circuits on an id/kind mismatch and on `PreimageLengthMismatch` — neither of which this string produces — and then names it by SHAPE alone: unshared header characters plus `d.len() == 33 && d[0] == 0x03`, at `record.rs:310-320`. Its doc comment at `record.rs:284-286` independently corroborates the 0.7 clause: *"0.7 refused the kind with `ReservedPrefixViolation { got: 3 }`, 0.8 decodes it or names its length; both answer `true` here"*. The test F-475 cites is `crates/me-cli/src/sysw/mod.rs:869`, `a_preimage_plate_is_named_not_misdiagnosed`.

### 2. The re-pin, in both repos

Following the recipe in the corpus's own header (`codex32_seam_vectors.json:40-46`, steps 1-4 at `:41`, `:42`, `:43`, `:46`):

```
bb703f608215bb00ccc677de4a282772016e774dd2d1d0f5c828ea38f5eac78b
->
2c2fbb3fa4d38c8858b9de4769d876d275478956c76ca491005c70d9f6bd541b
```

- engrave: `SEAM_VECTORS_SHA256` at **`crates/me-cli/tests/codex32_seam.rs:26`**.
- fork: `seamVectorsSHA256` at **`sysw/codex32_seam_test.go:30`**, plus the byte-identical vendored copy at `sysw/testdata/codex32_seam_vectors.json` (step 4: copied, then both re-hashed).

```
$ sha256sum <engrave copy> <fork copy>
2c2fbb3fa4d38c8858b9de4769d876d275478956c76ca491005c70d9f6bd541b  .../crates/me-cli/testdata/codex32_seam_vectors.json
2c2fbb3fa4d38c8858b9de4769d876d275478956c76ca491005c70d9f6bd541b  .../sysw/testdata/codex32_seam_vectors.json
$ diff -q <engrave copy> <fork copy>
IDENTICAL
```

JSON still parses, 13 rows (`python3 -c "json.load(...)"` → `json ok, rows = 13`).

**Every occurrence of the old hash was enumerated before editing** (`grep -rn bb703f60…ac78b` over both worktrees, the two main checkouts and the H2 worktree). Exactly **two** are live pins and both are updated. The other four are HISTORICAL RECORDS of what the corpus was at H0 and are deliberately left alone:

- `design/IMPLEMENTATION_PLAN_hashlock_H0_reader_guards.md:3`
- `design/CONTINUITY_composer_2026-09-01.md:1918`
- `design/agent-reports/hashlock-H0-post-impl-r1-fold-verification.md:72,73`
- `design/agent-reports/hashlock-H0-implementation-report.md:599`

`crates/me-cli/tests/descriptor_seam.rs` also declares a constant named `SEAM_VECTORS_SHA256` (`:45`), but it pins a **different** file — `testdata/descriptor_seam_vectors.json` (`:48`), `e7a4160ce064a6cb7ca31dc530e079c861cf2c8a075d75f793ef0d935f583758` — and is untouched. The fork's counterpart is `nonstandard/descriptor_seam_test.go:42`, likewise untouched.

### 3. F-475 marked CLOSED (`design/FOLLOWUPS.md`)

Marked in the **heading** (`design/FOLLOWUPS.md:15742`), which is what this file's own convention section demands ("a follow-up's STATUS lives in its heading" — status gets counted far more often than it gets read; a heading-only grep once said 24 open where the real number was ~16). The slug is kept under a strikethrough so `grep seam-corpus-33-byte-collision-row-names-the-wrong-0.8-error` still finds it, and the original claim is kept in the past tense so the entry still says what was wrong. A closure block below the entry carries the measurement, the code citations, both commit SHAs, the gate outputs and the merge note.

## Gates

### engrave — `cargo nextest run --locked -p mnemonic-engrave --no-fail-fast`

Run in the worktree; full output captured once to `/scratch/code/shibboleth/.tmp/h3-engrave-gate.txt`.

```
     Summary [   0.361s] 619 tests run: 616 passed, 3 failed, 2 skipped
        FAIL [   0.003s] (437/619) mnemonic-engrave::history_purge editing_the_file_alone_is_the_trap_the_message_warns_about
        FAIL [   0.003s] (440/619) mnemonic-engrave::history_purge the_harness_records_history_at_all
        FAIL [   0.004s] (442/619) mnemonic-engrave::history_purge the_emitted_zsh_recipe_actually_purges_the_entry
error: test run failed
```

**The seam test PASSES on the new pin** (line 434 of the capture):

```
        PASS [   0.003s] (312/619) mnemonic-engrave::codex32_seam the_host_never_admits_what_the_device_would_refuse
```

Re-run at the final branch tip to confirm the FOLLOWUPS commit changed nothing:

```
$ cargo nextest run --locked -p mnemonic-engrave --no-fail-fast -E 'test(the_host_never_admits_what_the_device_would_refuse)'
        PASS [   0.003s] (1/1) mnemonic-engrave::codex32_seam the_host_never_admits_what_the_device_would_refuse
     Summary [   0.004s] 1 test run: 1 passed, 620 skipped
```

**The three failures are box-local and pre-existing**, named as the brief anticipated. All three panic at the same precondition, `crates/me-cli/tests/history_purge.rs:35`:

```
/usr/bin/zsh is required: F-264's gate is 'the emitted recipe, RUN under a real interactive zsh,
actually removes the entry' ... This is deliberately a FAILURE and not a skip -- a skipped gate
prints ok and exit 0.
```

and `ls /usr/bin/zsh` → `No such file or directory` on this machine. That assert is unrelated to the corpus and fires on any tree here.

Caveat on environment: the Bash tool executed **bash**, not fish, so the `set -x` lines intended as fish `set -x` were bash's trace flag and `CARGO_TARGET_DIR` was **not** exported — the engrave build used the worktree-local `target/` (`/scratch/code/shibboleth/me-worktrees/h3-seam-corpus/target`), which is gitignored, per-worktree isolated, and was never staged (`git status --short` clean after both commits). No shared target dir was involved, so no baked paths.

### fork — `go test ./codex32/... ./sysw/...`

```
$ go version
go version go1.26.7 linux/amd64        (/scratch/code/shibboleth/.toolchain/go/bin)
$ go test ./codex32/... ./sysw/...
ok  	seedhammer.com/codex32	0.003s
ok  	seedhammer.com/sysw	0.037s
EXIT=0
```

`sysw` is the package that pins the sha (`sysw/codex32_seam_test.go:30`). Proved the pin gate actually RAN rather than reporting a cached `ok`:

```
$ go test ./sysw/ -run TestCodex32Seam -v -count=1
=== RUN   TestCodex32SeamDeviceAdmitsEverythingTheHostDoes
--- PASS: TestCodex32SeamDeviceAdmitsEverythingTheHostDoes (0.00s)
PASS
ok  	seedhammer.com/sysw	0.003s
```

## Merge note — the H2 branch will conflict here, by design

Branch `hashlock-h2` carries the OLD corpus bytes and the OLD pin. Measured rather than assumed, at both revisions:

- at **`17b3979`** (the revision the brief names): `git show 17b3979:sysw/codex32_seam_test.go` → `seamVectorsSHA256 = "bb703f60…ac78b"` at line 30, and `git show 17b3979:sysw/testdata/codex32_seam_vectors.json | sha256sum` → `bb703f60…ac78b`;
- at its **CURRENT tip `a1fd1398f7189aae1b6fb62f80599122e982f06c`** — the read-only worktree `/scratch/code/shibboleth/.tmp/seedhammer-hashlock-h2` is at `a1fd139`, not `17b3979`. `17b3979` is an ancestor (`git merge-base --is-ancestor` → YES) and the two commits since (`26fd1dd`, `a1fd139`) touch neither file: the worktree's JSON still hashes to `bb703f60…ac78b` and its `codex32_seam_test.go:30` still carries the old literal.

So a merge of `h3-seam-corpus` and `hashlock-h2` touches `sysw/testdata/codex32_seam_vectors.json` and `sysw/codex32_seam_test.go:30` together. **Take this branch's bytes AND this branch's literal**, or the pin gate goes red on the merge commit. The same pairing applies on the engrave side (`testdata/codex32_seam_vectors.json` + `tests/codex32_seam.rs:26`).

## Rules observed

- Own branch and worktree in each repo, created exactly as the brief specified (the fork's off `c4a64fc`). Nothing pushed. No commit on `master` or `main`. The H2 worktree was read only.
- Only the files the item names were edited: the corpus JSON, the two pin sites, the vendored copy, `design/FOLLOWUPS.md`.
- Paths staged explicitly (`git add <path> …`, never `-A`); `git status --short` inspected before each commit.
- Commit messages written to files and applied with `git commit -F` — backticks survived (`git log -1 --format=%B | grep -c '`'` → 11 on the engrave corpus commit, 9 on the fork commit). Both engrave commits and the fork commit end with the two required trailer lines; the fork commit also carries `Signed-off-by: Brian Goss`. Note: `git commit -s -F <file>` appended a **duplicate** Signed-off-by because the message file already carried one in the fork's conventional order (`Signed-off-by` → `Co-Authored-By` → `Claude-Session`); amended with `-F` alone, leaving exactly one signoff in the conventional position.
- No sub-agents. No `.jsonl` read.

## Residue (not acted on — outside this item)

- The corpus header's re-pin recipe (`codex32_seam_vectors.json:40-46`) has no step for the vendored copy's own hash check beyond "copy the file to the fork", and nothing mechanically enforces the copy. It held here because both files were re-hashed and diffed by hand. A `scripts/` one-liner that copies and re-pins both literals would make the recipe a command rather than a discipline — the same argument the constellation standard makes for `plan-build-gate.sh`.
- F-475's own table (`design/FOLLOWUPS.md`, the rows above the closure) renders the suggested replacement with an em dash; the corpus is ASCII. Anyone folding a "verbatim" suggestion out of that file into an ASCII artifact has to make the same substitution. Recorded in the closure so it is not read back as a near-miss transcription later.
