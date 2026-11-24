**⚠ Two me-side gate steps did not pass clean, both outside this change (details below); tinygo is owed to CI.**

# Build report: sh2 bug-hunt fixes

Heads: me `6c3331db` (branch `bughunt/sh2`, before this report commit); fork `91fb004` (branch `bughunt/sh2-fix`). Local only, nothing pushed.

Fork commits after the item 0 merges (`319ed85`): `648250f` seam scaffold, `918b5c4` RED rows, `958f829` item 1, `9662b95` item 2, `02dd625` item 4, `33dcb5c` item 5, `7816b8c` item 6, `91fb004` item 9. me: `bc2d77b5` items 1 and 2 (me side), `18b17c78` item 3, `97b23e8d` item 11, `6c3331db` item 10. Items 7 and 8 need no file change.

## RED (fork `918b5c4`, log `/scratch/cache/bughunt/sh2/build/red.log`)
24 of 25 rows failed by the FIX's stated reason; 21b is a guard that passes on base by design.
- 02, 03, 31a, 31b, 32: Back on the passphrase keyboard or question carried on with passphrase "" (to engrave mode, child picker, "Your seed is at slots", or a recovered SLIP-39 seed).
- 04: Back after 3 typed words returned the 12-slot mnemonic with the words CONFIRMED.
- 21a: self-check accepted unseated slots declaring m/84'/0'/0' and a non-lowest account. 21b passes.
- 22: Button3 returned row 2 on a page showing rows 0..1.
- 01 (bip380) and 01bw (nonstandard): xpub depth/child re-encoded differently.
- 06a, 06b, 06c: thresholds 0, -1, 3-of-2 and 5-of-2 accepted; wpkh/pkh/tr(sortedmulti) and wsh single key accepted.
- 50a, 50b, 50c: name-only file accepted; file without Format accepted; Derivation after key lines gave origin m.
- 11: codex32 New panicked, slice bounds [6:5].
- 7: picobin HashData wrong for zero-fill entries.
- 13, 14: white-label string overlapped the UTF-16 string at rows 16..25; entry with valid bit clear read as "SHII".
- 30c, 30d: repeated S control point {0 0}, want {3 3}; repeated relative commands ended at the wrong point.
- 49: optimized path dropped the backtrack to (6,5).
- 39: after a failed label address write, 8 string writes and 1 boot key write, want 0 and 0.

## GREEN
- All 25 rows by exact name (`g-rows.log`): 25 run, 25 PASS, 0 fail.
- Fork `go test -timeout 20m ./...`: rc=0, 61 packages `ok`, none FAIL (`gate-go.log`).
- 38 `t.Skip("deferred-minor: <slug>")` lines in 20 files (grep count).

## Gate
Fork:
- `go test -timeout 20m ./...`: PASS (rc=0).
- `go test -tags oraclelive -run '^$' ./...`: PASS (rc=0).
- `GOOS=js GOARCH=wasm go vet ./cmd/emu/` and with `-tags refugium`: PASS (rc=0 each).
- 25 rows by exact name: PASS, matched 25.
- Item 8 font regeneration (`fontgen.log`): no diff.
- tinygo `./cmd/controller` build: NOT RUN, tinygo not installed; owed to CI (item 7).

me (`gate-*.log`):
- `cargo nextest run --locked --no-fail-fast`: 691 run, 689 passed, **2 failed**, 2 skipped. The failures are `cross_lang::rust_ndef_parses_in_seedhammer_go_reader` and `preview_cross_lang::real_sidecar_renders_public_plates_only`. Both need the `third_party/seedhammer` submodule, which is not initialised in this worktree (`git submodule status` shows `-7b6f2fbb`); the error is "open .../third_party/seedhammer/go.mod: no such file". Neither touches files this change edits. Not run with the submodule initialised.
- `cargo test --locked --doc`: PASS (rc=0).
- `cargo fmt --check`: PASS (rc=0).
- `cargo clippy --locked --all-targets -- -D warnings`: **FAIL**, one error, `crates/me-cli/tests/f449_stage4a.rs:179` (`cloned_ref_to_slice_refs`). That file has no commit in `origin/master..HEAD`, so the lint predates this change (newer toolchain lint); left alone. `--keep-going` showed it is the only error.
- Seam filter `-E 'binary(descriptor_seam)'`: 19 run, 19 passed. The FIX has no "How checked" heading; I used the seam test binary.

## Judgement on code
- Used the local `bughunt/triage*` branches because `origin/bughunt/triage` does not exist.
- Committed the fix-39 seam scaffold (`648250f`, base behaviour) before the RED rows so they compile.
- Fork commits use `git commit -s`.
- 21b is a guard that passes on base.
- The item 1 `nonstandard/parse.go` edit (2.7) was done by hand, not from the patch.
- TestDescriptors per-key metadata values were measured: depth 3 / 0x80000000, and depth 4 / 0x80000002 for the multisig keys.
- 30d: adjusted the `rel` handling only, and added the `s` loop by hand because the reference patch lacked it.
- fx5 patch: applied only its `main.go` hunk (the patch also held a test file).
- fix-21 error string normalised to the FIX text (the probe had "§4f's").
- sysw comment tidied; confinement comment edit covers point 1 only.
- Item 11: spec anchors located by content plus a +12 line offset with asserts; two anchors (1671, 1714) were one line later than quoted, as the FIX itself notes. One line-984 wrap edit.
- Item 10: entries generated from the FIX tables with the 9.2 template. Fields the template does not cover: `sh-supply-reask` and `sh-needs-seam-tinygo-drivers` get "RED row: none; no host row."; the latter says "graded UNRESOLVED."; the S2/S3 and r1 minors omit Where and RED row and take repo mnemonic-engrave for `sh2-me-refusal-device-claim`, else seedhammer fork; the two architect notes say "graded" as "ungraded" and use "applies the fix" because no fix number exists.
- Pre-existing noise left alone: gofmt-unclean `gui/transaction*.go`; the `go vet ./gui` ArtifactDir/go1.26 message.

## Skipped or discrepant
- tinygo build (CI).
- The two submodule-dependent me tests and the pre-existing clippy error above.
- FIX count line says 37 skipped; its 6.2 table lists 38 test names (27a counts two). I skipped all 38.
