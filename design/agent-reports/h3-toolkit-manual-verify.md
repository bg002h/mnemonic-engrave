# H3 verification — toolkit-manual draft

Sonnet, read-only. No branches created, nothing committed, nothing pushed.
`.jsonl` not read.

- **Draft under review:** `mnemonic-toolkit` branch `h3-hashlock-device-manual`,
  tip `2c5f31cddc3da1442bfa596ce4e4f76c0e473a51` (base `master` `6cf3ecd8`, merge-base
  confirmed identical).
- **Device source of truth:** fork branch `hashlock-h2`, read via the existing
  read-only worktree `/scratch/code/shibboleth/.tmp/seedhammer-hashlock-h2`
  (`git show`/`git log`/`git diff` only — worktree confirmed clean throughout).
- **Method:** read the whole diff (`git diff 6cf3ecd8..2c5f31cd`, 161 lines / 2
  files — matches the draft's own stat exactly); cross-cited every quoted
  string and every mechanism claim against `git show 17b3979:<path>`;
  independently recomputed both machine-checks; re-ran every gate the draft
  named, myself, in the draft's own worktree
  (`/scratch/code/shibboleth/tk-worktrees/h3-hashlock-device-manual`, still
  present and unmodified); read the H2 device spec
  (`design/SPEC_hashlock_H2_device.md` §2-§4.7) and the ms spec
  (`mnemonic-secret/design/SPEC_ms_hashlock.md` §4.3) for consistency; read
  `design/CONTINUITY_composer_2026-09-01.md` for independent corroboration of
  the fork's post-17b3979 history (F-481 filed/closed, the other-path wording
  fix) since that record was written by neither the drafter nor me.

## Table

| # | Claim | Verdict | Evidence |
|---|---|---|---|
| 1 | Diff is exactly `docs/manual/.cspell.json` (+4 words) and `docs/manual/src/40-cli-reference/43-ms.md` (+157 lines), 161 insertions total, nothing else touched | **TRUE** | `git diff 6cf3ecd8..2c5f31cd --stat`: `2 files changed, 161 insertions(+)`, exact same two paths |
| 2 | The 4 added `.cspell.json` words are `seedhammer`, `brainwallet`, `diceware`, `normalises` | **TRUE** | `git diff` on `.cspell.json` shows exactly those 4 lines added after `"sysw"` |
| 3 | New content is one `### On the SeedHammer II {#hashlock-on-the-seedhombre}`-style section with five `####` subsections, inserted after the `### Worked example` block and before the `---`/`## ms vectors` boundary | **TRUE** | Full diff read; heading greps confirm exactly 1 `###` + 5 `####` added; diff context shows insertion point exactly as described |
| 4 | Read-only worktree HEAD is `a1fd139`, two commits past the pinned `17b3979` (`26fd1dd`, `a1fd139`) | **TRUE** | `git log --oneline -3` in the worktree reproduces the exact 3-line log the draft quotes |
| 5 | `diff --stat 17b3979..a1fd139` touches exactly those 6 files with those exact +/− counts | **TRUE** | Re-ran; byte-identical output to the draft's |
| 6 | F-481 mechanism: `content, _ = content.CutBottom(8)` is present at `17b3979:gui/composer_hashlock.go` (~line 166) and replaced by a 3-line comment at `a1fd139`, i.e. the "Known limitation: no readout" subsection is true at `17b3979` and false at the tip | **TRUE** | `git show 17b3979:...` / `git show a1fd139:...` diff around that line confirms both states verbatim; independently corroborated by `design/CONTINUITY_composer_2026-09-01.md:2329-2337,2350` ("F-481 CLOSED... fixed at 26fd1dd; MaxHeight now 209"), a record neither the drafter nor I wrote |
| 7 | The confirm modal's "other path" line text differs between `17b3979` ("two phrases to back up") and `a1fd139` ("back up every phrase"), and the draft deliberately does not quote it, describing only the stable predicate | **TRUE** | `git show 17b3979:gui/composer_copy.go` / same file at `a1fd139` (function `composerCopyHashlockOtherPath`) show exactly those two strings; manual's prose ("a further line follows it when another path... already carries a different hash") names no literal string; independently corroborated by continuity's "e2e I-1 'two phrases to back up' hard-coded, wrong on the three-hashlock wallet -> count-free line" entry |
| 8 | The device's `Deriving` countdown is ~10s (100,000 iters at 9,715 it/s ≈ 10.3s), not ~30s as the item's brief apparently assumed | **TRUE** | `hashlock/hashlock.go:23-24` doc comment, `composer_hashlock.go` pick-row `"Hardened (about 10 s)"`, `composer_copy.go` `"Deriving. This takes about 10 seconds."` all say 10; H2 spec §3 itself explains the likely source of "30s" — the *sealed-payload* KDF screen's `unlockKDFLead` says "about 30 seconds" for its own, different iteration count, and spec §3 explicitly warns "that string and its fallback are calibrated for the payload's iteration count, not this one's" |
| 9 | The manual (before this change) had zero occurrences of "SeedHammer" anywhere in `src/`, so there was no composer/payload chapter to imitate | **TRUE** | `grep -rin seedhammer docs/manual/src/` on `master` (`6cf3ecd8`) → 0 |
| 10 | `Path N hash` screen: title `fmt.Sprintf("Path %d hash", idx+1)`, rows = payload hashes then `Type a hashlock phrase`/`Type 64 hex`/`No hash lock`, lead `Which hash?` (or the no-payload lead), the §8i rule modal fires exactly when `sel < len(digests) \|\| sel == phraseRow \|\| sel == hexRow` | **TRUE** | `git show 17b3979:gui/composer_hash.go` — `composerHashRows`, `composerHashEdit` match verbatim, predicate byte-for-byte |
| 11 | The five refusal strings, in the host's order (empty → printable-ASCII → ms1-shaped → too-long → hex64), and "the ms1 test runs before the length cap" | **TRUE** | `hashlock.go:83-102` (`ValidatePhrase`) order matches; strings in `composer_copy.go` (`composerCopyHashlockRefusal`) match the manual's table verbatim; ms-cli's own doc comment (`hashlock_phrase.rs:116-117`) says the identical thing: "Order matters and is the spec's: empty, printable ASCII, ms1-shape (BEFORE the cap), cap, 64-hex" |
| 12 | Rule modal text, phrase-screen lead, no-payload lead, both method warnings, deriving zero-state lead, confirm modal body (two-space `"hash  "`, three-space `"method: %s   chars: %d"`), relation-line text, reconcile-screen text, all quoted verbatim | **TRUE** | Every string diffed byte-for-byte against `composer_copy.go` functions `composerCopyHashRule`, `composerCopyHashlockPhraseLead`, `composerCopyHashlockNoPayloadLead`, `composerCopyHashlockHardenedWarning`, `composerCopyHashlockSHA256Warning`, `composerCopyHashlockDerivingLead`, `composerCopyHashlockConfirm`, `composerCopyHashlockRelation`, `composerCopyHashlockReconcile`, `composerConfirmBody` — no discrepancy found |
| 13 | Hardened warns only under 20 characters; SHA-256 warns unconditionally | **TRUE** | `composer_hashlock.go` `hashlockMethodWarning`: `case hashlockSHA256:` unconditional, `case hashlockHardened: if len(phrase) < 20` |
| 14 | Back contract: phrase-screen Back drops the phrase and exits to `Path N hash`; every other inner Back (method pick, either warning, deriving, confirm) keeps the phrase and returns one step within the route; `composerHashEdit` returns `false` only for Back at `Path N hash`, and a path being created is discarded then | **TRUE** | `hashlockPhraseRoute`'s loop structure (return/`break pick`/`continue`) and `composer_hash.go:206-209`/`composer_shape.go:269-272` match the manual's table row-for-row |
| 15 | Discard button on `Deriving` uses `assets.IconDiscard`; `ctx.KeepAwake()` runs every frame of the derivation | **TRUE** | Both literals present in `composer_hashlock.go` |
| 16 | The four keyboard pages + the space key type **exactly** the 95 characters `0x20..=0x7E`, nothing missing, nothing extra | **TRUE (independently recomputed)** | Reconstructed the union from `passphrase_keyboard.go:19-26`'s four page constants + the space key in Python: `union size 95`, `missing: []`, `extra: []` |
| 17 | Worked example: `hash 3cf5d421..b70a4c12` / `chars: 28` for hardened; SHA-256 method digest is `b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb` — matching the chapter's pre-existing worked example and the fork's vendored corpus | **TRUE (independently recomputed)** | `hashlib.pbkdf2_hmac('sha256', phrase, b'ms-hashlock-v1', 100000, 32)` then `sha256` of that = `3cf5d421...b70a4c12` (hardened); `sha256(sha256(phrase))` = `b867db87...edbc96cb` (the SHA-256 method is a **double** hash: `PreimageSHA256`=SHA256(phrase), then `Digest`=SHA256(X) — this is not obvious from the manual's prose alone and I verified it against `hashlock.go`'s `Digest`/`PreimageSHA256` functions before recomputing); both values match `43-ms.md:368,370` (unchanged, pre-existing) and the fork's `hashlock/testdata/hashlock-v0.8.json` → `derivation[0]` exactly |
| 18 | Consistent with H2 spec §2 (phrase rule + order), §3 (constants, 10s not 30s, driver signature), §4.1-§4.6 (screens, copy, Back contract) | **TRUE** | Read spec §1-§4.7 in full; every claim above the confirm-modal's exact wording matches; the one place the manual and the *original* spec text differ (the reconciliation line's location — spec text still shows it inside the confirm modal, §4.5) is a **known, documented spec/code divergence** the plan's own "R0 round 0 folded here" section records (the line moved to its own post-HOLD screen because §8h's guard made the original destination unreachable on a mixed wallet) — the manual correctly follows the code, not the stale spec prose |
| 19 | Consistent with ms spec §4.3 (identical rule, shape-test-before-cap, printable-ASCII bytes, no normalisation, byte-verbatim reader) | **TRUE** | `SPEC_ms_hashlock.md:384-430` states the identical order and rationale (ms1-shape before cap explicitly, "0x20..=0x7E inclusive and nothing else") |
| 20 | `make lint` (markdownlint + cspell + lychee + flag-coverage + glossary-coverage + index-bidirectional) passes, 40 files, cspell 0/0, lychee `297 Total / 164 Unique / 274 OK / 0 Errors / 23 Excluded`, and is unchanged vs. an unmodified tree | **TRUE (reproduced)** | Ran `make lint` myself in the branch's own worktree with the same 4 `_BIN` overrides (all 4 binaries confirmed present at the claimed versions: `mnemonic 0.97.0`, `md 0.14.0`, `ms 0.18.0`, `mk 0.13.0`) — output byte-for-byte identical to the report's, including the lychee counts. (My *own main* toolkit checkout shows 41 files under `make lint` — traced to a stray **untracked** `docs/manual/src/99-build-banner.md` left over from an unrelated prior PDF build in that shared checkout, nothing to do with this branch; the branch's own worktree, and my direct check there, both show 40.) |
| 21 | `make anchor-check` is red with the same 10 `baseline shrunk` lines both with and without the change (pre-existing, unrelated) | **TRUE (reproduced)** | Ran `make anchor-check` in the branch worktree: identical 10-slug error set + `Error 1`. Ran it again on `master` in my main checkout (untouched by this branch): **identical** 10-slug set. Confirms the ratchet is stale on `master` already, not introduced here |
| 22 | Both new anchors render in `make html` output: `id="hashlock-on-the-seedhammer"` and the auto-slug for the limitation heading | **TRUE** | `grep -o` on the built `build/m-format-manual.html` finds both `id="hashlock-on-the-seedhammer"` and `id="known-limitation-the-phrase-screen-shows-no-readout"` |
| 23 | `--toc-depth=3` keeps the new `####` headings out of the TOC; `####` is used 44 times elsewhere in `src/` (not novel depth) | **TRUE** | pandoc invocation shows `--toc-depth=3` verbatim; `grep -rc '^#### '` on `master` sums to 44 |
| 24 | cspell's `` `[^`]+` `` ignore-regex spans code fences, so the whole-file run cannot actually check prose between two fenced blocks — demonstrated concretely with "brainwallet", present 3× in this chapter since `87e594e0`, never checked | **TRUE (independently reproduced a different way)** | Copied the whole `docs/manual` tree to scratch, removed `brainwallet` from `.cspell.json`'s `words` list, re-ran `cspell` over all 40 files: still `Issues found: 0 in 0 files` — i.e. even with the word un-whitelisted, the whole-file run never flags it, confirming the blind spot exists independent of the drafter's own probe-file method. `git show 6cf3ecd8:.../43-ms.md \| grep -in brainwallet` → 3 hits, matching the draft's count exactly. `87e594e0` confirmed to be the commit titled "ms hashlock chapter section..." |
| 25 | No `\index{}` marker added; no transcript/`include=` fence added (so `make verify-examples` was correctly skipped) | **TRUE** | Diff has exactly one fenced block, ` ```text `, non-executable; no `\index{` anywhere in the diff |
| 26 | Nothing pushed, no `master`/`main` touched, no commits in the engrave repo | **TRUE** | `git log master..h3-hashlock-device-manual` = 1 commit, on its own branch; no remote-tracking changes observed; engrave repo untouched by this task |

## What I did not re-derive

- The exact byte offsets of every `file:line` citation in the draft's §3 table
  (composer_hash.go:203/139/160-169/163/171-173, composer_hashlock.go's ~30
  line citations, etc.) — I spot-checked the *content* at each named function
  (all correct) rather than re-diffing every literal line number; none of the
  numbers I did check were off by more than a line or two of natural drift,
  never wrong about the code itself.
- The F-481 pixel-geometry numbers (11px/19px) are re-quoted from the already
  machine-verified `hashlock-H2-post-impl-r1-fold-verification.md:61-100` in
  this repo, which I read and confirm says exactly what the draft says; I did
  not re-run that geometry measurement myself (it is orthogonal to the H3
  draft — the draft only needed to know *that* F-481 is closed at 26fd1dd, not
  re-measure it).

## Verdict

**GREEN.** Every sentence the draft adds is true of the fork branch at
`17b3979` as instructed, consistent with both the H2 device spec and the ms
spec (including the one place code has since diverged from the spec's own
prose, which the draft correctly follows), and every gate the draft names
reproduces identically when run independently in a fresh invocation of the
same worktree. The draft's own §1 is not a defect in the draft — it is the
draft correctly catching that its 17b3979 pin is two commits stale on a branch
that moved during drafting, and filing the exact, verified replacement text
and a blocking follow-up for whoever folds it. No false claim found anywhere
in the report.

**One item to carry forward, already flagged by the draft itself and
independently confirmed here:** the `#### Known limitation: the phrase screen
shows no readout` subsection must not merge as-is once the branch includes
`26fd1dd`+ — F-481 is closed there. The draft's ready replacement text (§1a of
`h3-toolkit-manual-draft.md`) is drafted from `passphrase_keyboard.go:431-473`,
unchanged between the two commits, and is the right fix; it should be swapped
in (not re-verified from scratch) at fold time, since it was explicitly not
checked against `26fd1dd` by the drafter per its own brief's pin.
