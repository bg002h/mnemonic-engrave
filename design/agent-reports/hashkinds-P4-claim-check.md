# Hashlock kinds phase 4 — mechanical claim-check (hashkinds-p4, 6 commits, `main` 0562e811..285f7a9)

**VERDICT: NOT CLEAN.** Every mutation claim checked (8 of 8) applied and reproduced its stated
red output byte-for-byte, and every build/test/gofmt/vet/hash/row-count claim checked (14 of 14)
matched exactly — but three numeric claims in commit messages are demonstrably wrong (Important),
one function shipped by the diff has zero callers anywhere (Important), one emulator-walk
assertion was left checking retired text by the same commit that retired it (Important), and one
out-of-repo spec blockquote is now stale (Minor). No false-PASS test was found among the mutations
actually run.

Toolchain used throughout: `export PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH`,
`go version go1.26.7`. An untracked `engrave/zz_review_probe_test.go` present at session start was
moved aside before every build/test run (it is not part of this diff) and restored byte-identical
afterward; all worktrees created for comparison were removed; a stray `cmd/emu` build binary was
deleted. `git status --short` at end of session == at start (only that one pre-existing untracked
file).

## Claims checked

| # | Claim | Command | Result |
|---|---|---|---|
| 1 | `go build ./...` clean | `go build ./...` | PASS (exit 0) |
| 2 | `GOOS=js GOARCH=wasm go build ./...` ok | same w/ env | PASS (exit 0) |
| 3 | "56 packages ok, 0 FAIL" (all 6 commit msgs) | `go test -count=1 ./...` | PASS — exactly 56 `ok`, 0 `FAIL`, 20 `?` (76 total) |
| 4 | "gofmt at the five-file baseline" | `gofmt -l .` | PASS — exactly `gui/transaction.go, gui/transaction_golden_test.go, gui/transaction_txrecord_test.go, mt/mt.go, mt/mt_test.go` |
| 5 | "vet at its 10 pre-existing go1.26 lines" | `go vet ./...` | PASS — exactly 10 lines, all `testing.ArtifactDir requires go1.26` |
| 6 | (3f9771f) "vet ... verified byte-identical to HEAD" | `go vet` on a `main` worktree vs. a worktree at 3f9771f itself, diffed | **FAIL** — 1 of 10 lines differs: `backup/hashlock_test.go:498` (main) vs `:507` (3f9771f), because that same commit added lines above it. Same 10 warning messages/files, but not byte-identical as claimed. |
| 7 | sysw `record_class_vectors.json`: 68→81 rows, +13 rows, sha256 `d6766fdd..` | `python3 -c "len(json.load(...))"`, `sha256sum` on main-worktree and hashkinds-p4 | PASS — 68→81 confirmed, sha256 `d6766fdd7308ce28a23ef954f227d8e4e8431f841b64ea336ea9ea9a09b928dd` matches exactly |
| 8 | commit `40e1962e` exists in mnemonic-engrave, records the spec fold | `git -C mnemonic-engrave log --format -1 40e1962e...` and `git show --stat` | PASS — commit exists, titled "spec: the device screens name the hash kind (§13.1, §13.2)"; `file_commit` `73f3e7d8` also exists |
| 9 | `hashlock-v0.8.json` sha256 `0a911f78..`, 11 derivation rows | `sha256sum`, `python3 json` | PASS — sha256 exact match; provenance.json's own `derivation_rows: 11` matches |
| 10 | "22 row/kind pairs" in the digest KAT | Added a temporary `t.Logf` in `TestDigestKAT`, ran `go test -run TestDigestKAT -v`, reverted | **FAIL** — actual is 11 rows × 4 kinds = **44** row/kind pairs, both at HEAD and at commit 3f9771f itself (checked via a worktree at that commit). Every derivation row has all four kind columns populated. |
| 11 | "SIX map-index errors and THREE equality errors, which is spec §5's 'eleven ... sites'" | Arithmetic: 6+3=9 | **FAIL** — 9 ≠ 11. The commit equates its own breakdown to the spec's "eleven," but the breakdown sums to nine. (Historical compiler-error count from development is not independently re-derivable; this flags the internal arithmetic inconsistency, not a claim about which number is "true.") |
| 12 | "one round short on hash256 reds both and produces `3cf5d421..`, the SHA256 digest" | Mutated `DigestHash256` to drop the second SHA-256 round, ran `TestDigestKAT -v`, restored | PASS — for phrase "correct horse battery staple" the mutated function returns `3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12`, which equals the corpus's own `hardened_h` (the sha256, non-hardened digest) for that phrase, exactly as claimed. Both the function assertion and the `DigestOf` assertion fail. |
| 13 | "mis-wiring one DigestOf arm reds ONLY the dispatch assertion" | Mutated `DigestOf`'s `KindRipemd160` arm to call `DigestHash160`, ran `TestDigestKAT -v`, restored | PASS — 11 `DigestOf gave` failures, 0 `function gave` failures |
| 14 | "all four kinds encode to 53 modules" / `TestQRStaysAtFiftyThreeModules` is now a test | `go test ./hashlock/ -run TestQRStaysAtFiftyThreeModules -v` | PASS |
| 15 | "ONE golden moved, hashlock-phrase-qr-v9, and the other three plates are untouched" | `git diff main..hashkinds-p4 --stat -- backup/testdata/` | PASS — only that one `.bin` changed; `hashlock-phrase-noqr.bin`, `hashlock-phrase-space-legend.bin`, `hashlock-string-6mm.bin` untouched |
| 16 | SPEC §13.1 table: locator row 35 chars / 10 rows / 1.20mm spare | Read `SPEC_hashlock_kinds.md` §13.1 table | PASS — matches commit ac452c2's citation exactly |
| 17 | H6 §6.5 pins method line at 73 chars | Read `SPEC_hashlock_H6_preimage_plates.md` §6.5 | PASS |
| 18 | §10's "six hardcoded `[56:]` sites" / composerDigestShort panics on 40-hex, now slices from the end, ONE implementation | Read `composer_consent.go:67` (`h[len(h)-8:]`) | PASS |
| 19 | "TestComposerEveryScreenFunctionHasAProductionCaller ... reported the original had no caller left" (dead-code gate exists and passes) | `go test ./gui/ -run TestComposerEveryScreenFunctionHasAProductionCaller -v` | PASS (gate runs, passes, only checks `composer*`-prefixed funcs in `gui/`) |
| 20 | §7.4 "no default" tag switch in policy_shape.go / D-1,D-2 panics in `DigestLen`/`Token`/`tag`/`DigestOf` on unknown kind | Read `md/policy_shape.go`, `md/compose.go`, `hashlock/hashlock.go` | PASS — all four panic on an unreachable/unknown kind as claimed |
| 21 | "measured margin: 61-char lead draws 4 rows, +40→4 rows, +60→3 rows" | Parametrized padding (0/20/40/60/80/100 unbreakable chars) on `composerCopyHashKindLead`, ran `TestComposerHashKindScreenDrawsAllFourRows` each time, reverted | PASS — +20/+40 pass (4 rows), +60/+80/+100 fail (3 rows), reproducing the exact threshold. (A natural-language sentence of similar length did *not* reproduce this, because word-wrap packs differently than fixed padding — not a discrepancy, just a methodology note.) |
| 22 | node --check passes on both touched walk files | `node --check cmd/emu/walk_hashlock_phrase.js`, `cmd/emu/shots_composer.js` | PASS |
| 23 | "go build ./... does not reach cmd/emu[js-tagged files]" | `go list ./...`, checked build tags | Roughly true / Minor nuance — `cmd/emu` itself is listed and builds (via `main_notjs.go`, tag `!js`), but the specific file with the cited hunks (`composer_js.go`) carries `//go:build js` and is compiled only under `GOOS=js`, so the claim's substance holds |

## Mutations run (Job 2) — 8 of 8, all applied, all byte-diffed, all reverted, all reproduced the claimed result

1. `gui/composer_hash.go`: `want := kind.DigestLen()*2` → `want := 64` → **RED**: all `ripemd160` subtests fail, all `sha256` subtests pass. Matches claim exactly.
2. Same file: `valid := len(frag) == want` → `valid := len(frag) >= want-1` → **RED** at both kinds' `want-1` boundary. Matches.
3. `gui/composer_consent.go`: restored a `break` in the kind sweep → **RED**, and specifically the two-kind consent test names only `sha256` (path 2's `ripemd160` is missing from the scoped §8i line) — confirms both the mutation AND the earlier false-pass-scoping fix (searching the *whole* screen would have passed, since the miniscript branch line also contains the word "ripemd160"; the test is correctly scoped to the §8i line only).
4. `gui/composer_copy.go`: `composerCopyHashRule` reworded back to name SHA-256 → **RED** on the "entry body names no kind" subtest.
5. `gui/composer_copy.go`: padded `composerCopyHashKindLead` by 60+ unbreakable characters → **RED**, kind screen drops to 3 tappable rows (see row 21 above).
6. `gui/composer_hash.go`: hex-arm seed `kind := md.KindSha256` → `md.KindHash256` → **RED** (press-through now yields a `hash256` lock). Confirmed the *reorder* of `composerHashKinds` (the test's originally-named, now-retired mutation) is a genuine semantic no-op: re-ran with the slice reordered and the same test still **PASSES**, exactly as the code comment claims.
7. `gui/composer_hash.go`: `md.NewHashLock(kind, raw)` → `md.NewHashLock(md.KindSha256, raw)` → **RED** on `TestComposerCanBuildARipemd160HashlockOnTheDevice` (fails via a frame-timeout/no-return rather than the literal "path holds no hash" message quoted in the comment — same failure class, worded slightly differently; Minor).
8. `gui/composer_state_hook.go`: `out[i] = p.Hash` (aliasing) → **RED**, "hook handed out the POLICY's own pointer."
9. `md/compose.go`: `HashLock.Digest()` → return the whole padded array → **RED**: width assertion fails at `64 != 40`, suffix assertion prints `a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3` + 24 zeros (exact match to the comment's `a0a1..b0b1b2b3 followed by 24 zeros`), and — as claimed — the abbreviation-agreement assertion at the bottom does **not** fire (stays green under the mutation, the test's own stated point).

No false-pass or tautological test was found among these eight. Every named mutation actually
changes behavior and every test that claims to catch it does.

## Findings

### Important

1. **"22 row/kind pairs" (commit `3f9771f`) is wrong — actual is 44.** `TestDigestKAT` iterates all
   11 corpus rows (all 11 carry all four kind columns) × 4 kinds = 44 (row, kind) pairs, checked
   both at the diff tip and at commit `3f9771f` itself via a worktree. Confirmed with a temporary
   `t.Logf` instrumentation (`checked=11 pairs=44`), reverted. This is a wrong number in a commit
   message describing the KAT's own coverage.

2. **"SIX map-index errors and THREE equality errors, which is spec §5's 'eleven ... sites'" is
   arithmetically inconsistent.** 6 + 3 = 9, not 11. The commit (`3f9771f`) presents its own
   breakdown as identical to the spec's count; it is not. (The true historical compiler-error count
   from the refactor is not independently reproducible after the fact without replaying the exact
   development sequence — this finding is about the commit contradicting itself, not about which
   number is "correct.")

3. **"vet at its 10 pre-existing go1.26 lines, verified byte-identical to HEAD" (commit `3f9771f`)
   is false as stated.** `go vet` output differs by one line number (`backup/hashlock_test.go:498`
   on `main` vs. `:507` at that commit) because that same commit adds lines above it in that file.
   Same 10 warnings/files, same category, not byte-identical. Substance (10 pre-existing go1.26
   lines, none newly introduced) is unaffected; the stronger literal claim is wrong.

4. **`sysw.HashRecord` (added by `3f9771f`) has zero callers anywhere** — not in production code,
   not in any test, not in any `.js` walk file. `ParseHashRecord` (the decoder) is exercised
   throughout; its symmetric encoder is dead on arrival. It is not caught by
   `TestComposerEveryScreenFunctionHasAProductionCaller` because that gate only scans
   `composer`-prefixed functions in package `gui`; `HashRecord` lives in package `sysw`.

5. **`cmd/emu/shots_composer.js:667` still asserts the retired row label.** `composerHashRowHex`
   was renamed `"Type 64 hex"` → `"Type a digest"` by commit `285f7a9` (which also edited this same
   file, two lines below, to fix a *different* stale string, the §8i wording). Line 667 —
   `must(hashRows, "Type 64 hex", "the type-it row");` — was left unchanged and now checks for text
   the production screen no longer draws. The commit's claim ("shots_composer.js stopped asserting
   the retired §8i wording") is true only for the §8i line specifically and does not cover this one;
   it is not a false claim, but it is a real regression the same commit introduced and didn't catch,
   in the walk script most directly about this exact rename. `node --check` (syntax only) cannot see
   it, and per the commit's own text "Neither runs in CI." Job-3 finding, but rated Important because
   it is a functional break (the walk would fail immediately if run) rather than mere prose drift.

### Minor

6. **`mnemonic-engrave/design/SPEC_wallet_policy_composer.md` §8i's blockquote is now stale.** It
   still reads *"The hash must be SHA-256 of a 32-byte value..."* — the exact string commit `d96ade7`
   retired from `composerCopyHashRule()` (now *"The preimage must be a 32-byte value..."*). No
   document anywhere was found with the new text (`grep -rn "The preimage must be a 32-byte value"
   design/*.md` → no hits). This is a different, older spec file in the other repo that this branch
   does not touch; `SPEC_hashlock_kinds.md` §7.2 correctly describes the rewording, but the original
   quoted-copy table in `SPEC_wallet_policy_composer.md` was not updated to match.

7. **`TestComposerCanBuildARipemd160HashlockOnTheDevice`'s named mutation fails differently than
   the comment says.** Comment: "this test reports that the path holds no hash at all." Actual:
   the harness times out ("composerAddPath never returned after 256 frames") before reaching that
   assertion. Both are genuine failures (the mutation is not a no-op), just not the literal message
   quoted.

### Nit

8. Commit `d96ade7`'s "go build ./... does not reach cmd/emu" is imprecise: `cmd/emu` itself is in
   `go list ./...` and builds fine (via the `!js`-tagged stub file); what does not get compiled under
   a plain `go build ./...` is specifically the `//go:build js`-tagged files in that package
   (`composer_js.go` among them), which is what the surrounding hunk is actually about.

## Not independently re-derivable

- The exact historical count of compiler errors from making `HashLock` non-comparable (item 11
  above) — this requires replaying the refactor in the exact order it was done; only the commit's
  internal arithmetic could be checked, and that failed.
- The literal "64 characters of headroom" mid-development measurement in commit `ac452c2` (current
  measured headroom for that modal is 107 chars, `margin 80`, per `go test -run
  'TestConfirmScreensThisBlockTouchesAreDrawnInFull' -v`) — the claim describes a state *before* the
  same commit's body-shortening fix, which is plausible and not contradicted by the final number,
  but not independently checkable without the intermediate diff.
