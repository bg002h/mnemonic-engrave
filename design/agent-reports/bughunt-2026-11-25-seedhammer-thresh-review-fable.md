# seedhammer PR #29, final-gate review (whole PR, head 017357b)

Reviewer: final gate, read-only. Repo `/home/user/seedhammer`, branch `claude/project-thread-utm1aa`, head `017357b`, merge base `e4fa676` (18 commits, 53 files). Nothing was edited, committed or pushed in either repository; `git status` is clean in seedhammer and descriptor-mnemonic.

**Verdict: 0 Critical / 1 Important / 5 Minor / 3 Nit.**

- I found no case where the Go code gives a summary Rust refuses, and no wrong "NO KEY NEEDED".
- The Important item is a merge gate, not a code defect: this head has never been compiled by TinyGo.
- The "any k of n" text is not reachable from any device flow today, which bounds the severity of every display finding below.

Toolchain: go1.26.7 from the module cache (`GOTOOLCHAIN=local`), cargo 1.85.0. TinyGo is unreachable here (`nix develop` gets HTTP 403 on the flake inputs).

## What I ran

- **Suites.** `go test -count=1 ./md/ ./mk/ ./mt/ ./address/ ./nonstandard/` pass. `bash scripts/test-32bit.sh` passes (386 tests including md, arm build). `GOARCH=arm go vet ./md/` and `GOARCH=arm go build ./md/ ./gui/ ./cmd/...` are clean. `gofmt -l .` is the five-file baseline.
- **gui, targeted.** The any-of, composer-guard, no-change-chain, legacy-Check and consent tests pass. I did not re-run the full gui package.
- **Counts.** md has 201 top-level tests; gui lists 1466 tests plus 7 fuzz targets, which is the PR body's 1473.
- **Equivalence at depth.** `MD_EQUIV_ROUNDS=300000` on the four iterative-versus-recursive tests and `TestMaxDepthCardsFitTheStack`: all pass.
- **Go against Rust, full shape.** I built `shape_probe` from descriptor-mnemonic HEAD `65a0162a` (which is ahead of the vendored pin `1dac0e10`) and compared every decodable card on `typed` and on the whole shape with the repo's own `shapeAgrees`. Earlier differentials scored only `typed` and `complete`.

  | Corpus | Cards | Disagreements |
  |---|---|---|
  | Thresh-centred generator: repeated slots, off-list children, every placement, all three internal-key kinds | 144,492 (7,346 any-of entries, 4,116 keyless) | 1,907, all finding M-1 |
  | The implementer's generator, new seed | 76,935 | 0 |
  | Deep typed trees (depth 4 to 9) | 120,000 | 0 |
  | Resource-limit families in wsh, sh, sh(wsh) and tap | 218,447 | 0 |
  | sh scripts with scriptSig sizes 1630 to 1720, script within 520 bytes | 376 | 0 |

- **Do the limit corpora straddle the limits?** Yes for six of them: off-by-one mutants of the 100-item, 201-op, 520-byte, 3600-byte and 1650-byte limits, and removal of the tap 1000-element limit, each produce disagreements with Rust (18, 2, 29, 4, 10, 5). In the scriptSig corpus Rust accepts 13 cards at exactly 1650 and refuses 13 at 1651.
- **Composer regression check.** 123,562 composed path lists (all four wrappers, up to 8 paths, every lock and hash kind): every one is still `Complete` after the type rule, from the tree and from the card.
- **Mutants against the committed tests.** 13 in md and 2 in gui; results are under M-3 and "What held".

Generator sources and the probe are kept in `/tmp/claude-0/-home-user/efb9fedd-ab0a-5019-871b-00d31aeaaeb6/scratchpad/fable-sh/keep/`. The build trees and corpora are deleted.

## Important

### I-1. This head has never been compiled for the device, and merging is what first compiles it
- **Where:** `.github/workflows/test.yml:2-5` and `.github/workflows/image.yml:2-6` (both trigger on push to `main` and on `workflow_dispatch` only).
- **Scenario:** the PR rewrites every tree walk on the md1 load path (decoder, validators, canonicaliser, re-encoder, script emitter, tap-leaf walks) and adds about 1,000 lines of type checker to firmware code. The first TinyGo compile of all of it is the push to `main`, in the same event that builds the release image.
- **Evidence:**
  - `nix develop -c tinygo version` fails here with HTTP 403 on the flake inputs, so there are no flash, RAM or stack numbers for this head. The PR body says the same.
  - This class already happened inside this PR: review r2 C-1 was a host-green change that broke the 32-bit build.
  - The stack evidence is gc frames on 386 under `debug.SetMaxStack`, which is a proxy for TinyGo's Thumb frames.
- **Why Important:** the logged follow-up `sh-typerule-r3-n2-workflow-skips-pr-branches` is marked "record only". I disagree for this PR, because the rewritten walks run on every card load.
- **Required before merge:**
  - dispatch `test.yml` and `image.yml` on the branch and record flash and RAM;
  - on the emulator or a device, load one max-depth card and one legacy `Check(PkK)` card, and compare the address shown with Rust's.

## Minor

### M-1. A multi that repeats a slot is shown as "k-of-n" where Rust reports none, and the follow-up says the two agree
- **Where:** `md/policy_shape.go:379-380` (no `nkeys == br.Keys` guard on the `plainMulti` arm); Rust has the guard at `crates/md-codec/src/policy_shape.rs:628-633`.
- **Scenario:** `wsh(multi(2,@0,@0,@1))`, card `md1faekzqqpq2tvyyy4qqxppsg3qqkcfs3z36psgk`. Go gives `Complete`, K/N = 2/3, Keys = 2, which `policySummaryLines` prints as `1: 2-of-3`. Rust gives k = 0, n = 0 and slots [0,1]. One signature from @0 satisfies both @0 positions, so @0 spends alone.
- **Evidence:** all 1,907 disagreements in the first corpus are this, for `multi`, `sortedmulti`, `multi_a` and `sortedmulti_a`, bare or as an `or_*` alternative. With a lock beside the multi (`soleMulti` arm) the two agree.
- **What is wrong beyond the lag:**
  - `scratchpad/followups/seedhammer-thresh.md:116-123` says "Rust parity today; fix lands in md-codec first". That is false for the bare spelling. `md/bits.go:34-37` and the PR body do state the lag correctly.
  - No vendored case has a repeated-slot multi, so `TestPolicyShapeAgreesWithRustCases` cannot see it.
- **Why Minor:** no device flow reaches it. The only summary input is the Build Policy template, which is `sortedmulti` over distinct slots and takes the classifiable branch anyway.
- **Fix:** the guard is one line and is pure convergence. With it applied in a scratch copy, md and the gui summary tests pass except `TestIterativeWalkMatchesRecursive` (see M-4). Correct the follow-up text either way.

### M-2. A path that needs no key prints "0 key(s), custom" beside the new "NO KEY NEEDED"
- **Where:** `gui/template_engrave.go:196`.
- **Scenario:** `wsh(or_d(pk(@0),1))` (card `md1f40mnqqpqztvyyy4qqxpxp2xq26lf6j5375pn5`) is typed and complete in both languages, and its second path, which anyone can spend, prints `2: 0 key(s), custom`. `wsh(or_d(pk(@0),older(10)))` prints `2: 0 key(s), custom +timelock`.
- **Evidence:** both cards run through Go and the Rust probe; the shapes agree (`slots: []`).
- **Why it matters now:** before this PR the device had one wording for a keyless path. It now has a loud one for any-of entries and a quiet one for conjunctions, so the quiet one reads as the lesser warning.
- **Status:** latent, as M-1. `sh-thresh-r1-m2-or-i-constant-arm-phantom-entry` covers the `or_i(0|1, thresh)` instance and grades it Minor. I agree with the grade only while no flow renders an outside card; make it a precondition of any such flow.

### M-3. Several resource-limit rules pass the committed tests when broken
- **Where:** `md/policy_typecheck.go:232` (201 ops), `:237` (1650-byte scriptSig), `:240` (100 witness items), `:245` (tap 1000-element stack), `:373` (520-byte Legacy script), `:1023-1028` (thresh exec-stack fold).
- **Evidence:** each of these mutants leaves `go test ./md/` green:
  - 100 items → 101; 201 ops → 202; 1650 → 1651; 520 → 521;
  - the tap stack check removed entirely;
  - the `+1` in thresh's exec-stack fold removed.
- **The code is right.** Against Rust, the first four mutants and the tap removal are each caught by my corpora. Only the exact tap edge (1000 against 1001) and the thresh exec-stack fold are unproven by anything; both need a tap leaf near 1,000 stack elements.
- **Fix:** add edge vectors to Rust's `cases.json` first, then re-vendor. The tap stack rule has no vector at all today.

### M-4. The frozen recursive references pin known lags, so the next convergence fix fails an unrelated test
- **Where:** `md/recursive_ref_test.go:5-11` ("keep the bodies as they were").
- **Scenario:** applying M-1's one-line guard fails `TestIterativeWalkMatchesRecursive`, because `branchOfRec` is the pre-fix code. Every later behaviour change to a walk has the same cost.
- **Fix:** say in the header that a behaviour change edits both copies in one commit, or give the references an end date (for example, delete them after the first release that ships the iterative walks).

### M-5. The PR body reads as a change log, and four statements are untrue of the head
- **Where:** `scratchpad/prbodies/seedhammer.md`.
  - Line 23: "the device showed `thresh(k<n)` as if every child were required". No device flow shows an outside card's summary. `md.PolicyShapeChunks` has four callers: three composer readers (the composer cannot build a thresh) and `gui/multisig_build.go:808` (a device-built `sortedmulti` template). The function would have printed it; the device never did.
  - Line 6: the merge order says the thresh commits vendor from `2d5c9935`. The pin is `1dac0e10`, and the Rust branch is now at `65a0162a`.
  - Line 51: "Go leads it on exactly those trees". Not true at head: Rust refuses non-B tap leaves, and the differential shows 0 disagreements.
  - Line 27: "67 cases … 64 that have a card". The head has 106 and 102.
- **Fix:** rewrite the body as a description of the head, say plainly that the any-of text is not yet reachable on the device, and move the history below it.

## Nit

- **N-1. Four commit ids for one rule.** `md/policy_typecheck.go:4-6` cites `f8b4a675` and `5ad5a9b2`, `md/policy_shape.go:181-182` cites `ca48a05e`, and the pin is `1dac0e10`. All are on an unmerged branch and will dangle after a squash merge. Fold this into the logged re-pin (`sh-typerule-r3-m2-repin-provenance-to-merged-commit`).
- **N-2. `AnyOfMinKeys` returns 0 for a conjunction** (`md/policy_shape.go:144-153`). The doc says callers must test `AnyN` first; a `(n int, ok bool)` return would make the misuse a compile error.
- **N-3. Commits are unsigned and authored as Claude.** The PR body discloses it; it blocks only an upstream PR.

## What held

- **Go matches Rust at HEAD on every card I could generate**, except M-1: about 560,000 cards, compared on `typed` and on the whole shape (key path, tap depth, entry order, K/N, key counts, locks with kinds and values, hashlock counts, depths, any-of counts, fewest keys).
- **"NO KEY NEEDED" appears only when the timelocks alone reach k.** All 4,116 keyless any-of entries in the corpus agree with Rust's `min_keys == 0`.
- **The vendored cases are current.** `md/testdata/policy_shape_cases.json` is byte-identical to the file at descriptor-mnemonic HEAD (sha256 `325792c5…fc3e`). The one md-codec source change since the pin (`006d061b`) only gates the rule on the `derive` feature.
- **The type rule causes no composer regression.** No composable policy fails it.
- **The rest of the type rule is ported as written.** Tap's stack check, the "some satisfaction" rule per leaf, and thresh's `i <= k` fold match the vendored rust-miniscript source I read, and the mutants for `i < k`, the 3600-byte limit, `s:pkh`, the duplicate-slot refusal, the k == n allowlist and the type rule itself are each caught by committed tests.
- **The gui tests bite.** Disabling `composerShapeHasAnyOf` fails all three composer tests; disabling the any-of arm fails the goldens.
- **All readers of `Branches` are guarded.** The three composer readers test `composerShapeHasAnyOf` before first use, and `policySummaryLines` handles the any-of entry first. No other package reads a shape.
- **SH#3 and SH#4 match Rust line for line.** `validate_use_site_overrides_canonical`, `validate_no_empty_origin_overrides`, `reject_non_zero_pad` at both break points and in Rust's order, and mk-codec's `check_key_rules` (R2, R3 with its one exemption, R4).
- **SH#1 matches Rust.** The emitter and `describeTapLeaf` unwrap `Check` only over a bare key with a key argument.
- **The iterative decoder and emitter read correctly.** The tr frame's tree pointer points into the children slice, not the frame; the `v:` fold runs after its child; thresh's tail is queued last.
- **32-bit.** No new int holds a wire value wider than 31 bits; `msScriptNumSize` takes `uint32`; the suite runs at 386.
- **No new dependency, no `go.mod` change, no stray debug output, `git diff --check` clean.**

## Not covered

- Decode-refusal parity on malformed bitstreams was not differential-tested against Rust (every card I generated decodes in both). The committed SH#3 tests and the iterative-versus-recursive decode test are the evidence there.
- The full gui package was not re-run.
- Nothing was measured on TinyGo or a device (I-1).
