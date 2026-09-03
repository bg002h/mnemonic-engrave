# composer S3 — independent fold-verification review, round 1 (targeted)

**Subject:** fork branch `composer-s3`, fold commits `7edc863`, `83e932a`, `27afa9f`
(diff `a63fd1e..27afa9f`), plus mnemonic-engrave master `db53513` (spec M-2/M-3) and
`b1a1985` (F-461). Reviewed report: `design/agent-reports/composer-S3-exec-review-r0.md`
(1C/2I/5M/3N). Fold report under verification: `design/agent-reports/composer-S3-fold-r0-report.md`.

**One question:** did the fold fix every Critical/Important exactly as filed, can each new
or changed test fail, and did nothing else move.

**Environment.** Go `/scratch/code/shibboleth/.toolchain/go/bin/go` (go1.26.7 linux/amd64),
`CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local TMPDIR=/scratch/code/shibboleth/.tmp`,
`-mod=readonly`. Nix at `/nix/var/nix/profiles/default/bin` (2.35.2), TinyGo via
`nix develop -c`. All work done in `cp -r` copies under `/scratch/code/shibboleth/.s3-verify/`
(deleted at the end of this review) plus one throwaway baseline copy under `.tmp/`
(also deleted). No sub-agents were spawned. No `.jsonl` file was read.

**Counts: 0 Critical / 0 Important remaining. All 5 Minors and 3 Nits accounted for.**
One Minor found in this round: a numeric transcription error in the persisted reports
(M-4's measured character count). See "New finding" below.

**A process incident, found and fully repaired in this session — see "Worktree metadata
incident" at the end.** It affected only shared git metadata (HEAD/index) in the original
fork worktree, never its files or the reviewed history; it is disclosed for the record
and does not bear on any verdict below.

---

## 0. The second Critical the fold found (composerSelfCheck's K/N domain error)

**VERIFIED.**

Fold commit `83e932a` changes `gui/composer_selfcheck.go`'s `p.Keys.N >= 2` arm from an
unconditional `b.K`/`b.N` compare to: compare the threshold only where the codec reports
one (`b.K != 0 || b.N != 0`), else compare `b.Keys` (the key count) against `p.Keys.N`.

- **Control test exists and passes on the fold.** `TestComposerSelfCheckAcceptsEveryOfferedPresetsHonestBuild`
  walks all 12 offered (wrapper, preset) pairs (4 wrappers × up to 6 presets) and asserts
  `composerSelfCheck` accepts each honest build. Ran it: **12/12 PASS**.
- **Reverting the K/N read reproduces exactly the four named failures.** Restored the
  unconditional `if b.K != int(p.Keys.K) || b.N != int(p.Keys.N) { ... }` read (removing the
  `switch` added in the fold) and re-ran the same test:
  ```
  --- FAIL: .../wsh/tiered-recovery      self-check: path 2 is 1-of-2 in the shape and 0-of-0 decoded
  --- FAIL: .../wsh/decaying-multisig    self-check: path 1 is 2-of-2 in the shape and 0-of-0 decoded
  --- FAIL: .../tr/tiered-recovery       self-check: path 2 is 1-of-2 in the shape and 0-of-0 decoded
  --- FAIL: .../tr/decaying-multisig     self-check: path 1 is 2-of-2 in the shape and 0-of-0 decoded
  ```
  Exactly the four pairs and the exact messages the fold report cites. Reverted; tree
  clean afterward.
- **The other half — the fix does not turn a real disagreement into a pass.**
  `TestComposerSelfCheckStillComparesKeyCountsUnderALock` builds an honest locked 2-of-3,
  then perturbs the shape to claim 4 keys against 3 decoded, on a locked path. Ran it: PASS
  (refuses, names "keys" in the error).

**Verdict: exactly as filed. Both directions verified by direct re-execution, not by
reading the report.**

---

## 1. Per Critical/Important — fold hunk, reproduction, and named-mutation kill

### C-1 — seating re-entry offered zero sources (commit `7edc863`)

**VERIFIED.**

- Hunks: `gui/composer_seat.go` (`composerReleaseSeat`, `composerReleaseLastSeat`, the
  resume-skip in `composerSeatFlow`), `gui/composer_flow.go` (`composerSeatingStep`'s new
  loop). Both are exactly the two halves the fold report describes.
- **Named mutation ("restore the re-ask from 0"):** removed the `if st.assigned[i].src >= 0
  { i++; continue }` skip in `composerSeatFlow`. Ran
  `TestComposerBackAtTheMappingReviewKeepsTheSeatedKeys`:
  ```
  composer_join_test.go:377: Back at the mapping review did not land on the last seated
  slot: seating re-asked from slot @0 instead of resuming at @1 ...
  Frame: "SeatkeysSlot@0,Path1key1of2:chooseakey73c5da0am/48h/0h/1h/2hTypeaseedLeaveunseated"
  ```
  Byte-identical to the fold report's pasted output. Reverted; tree clean.
- **Confirmed the fold's own "I had to strengthen the assertion" claim as a side effect.**
  With only the resume-skip reverted (release logic intact), the reproduced frame shows
  `Slot @0` **and still contains `73c5da0a`** — i.e. an assertion that only checked "a
  fingerprint is present" would have missed this exact mutation. The test's actual
  assertion (`Slot @1`, not just presence) is what catches it. This is not asserted from
  the report; it fell out of the reproduction above.
- **Unit half:** reverted the source-release inside `composerReleaseSeat` (removed the
  `st.sources[src].used = false` line). Ran
  `TestComposerSeatingReleasesASourceWhenItsAssignmentIsDropped`:
  ```
  composer_join_test.go:414: source 1 is still marked used after its only assignment was
  released, so no later pick list will offer it while nothing holds it
  ```
  Reverted; tree clean.

### I-1 — Move up discarded nothing on equal-key-count reorder (commit `7edc863`)

**VERIFIED.**

- Hunk: `gui/composer_shape.go`'s `composerMoveUp` (unconditional
  `composerDiscardAssignments(st)`, bypassing `composerApplyShapeEdit`'s signature check).
- **Named mutation:** removed the `composerDiscardAssignments(st)` call. Ran
  `TestComposerMoveUpDiscardsUnconditionally`:
  ```
  composer_join_test.go:463: slot @0 still holds source 0 after a Move up
  ... (all six slots)
  composer_join_test.go:468: source 0 is still marked used after a Move up discarded every seat
  ```
  Matches the fold report's pasted output. Reverted; tree clean.
- The controller's chosen fix (option 3 of the review's hypothesis — discard
  unconditionally, one line) is exactly what landed; §8j's copy ("Every key you seated
  will be cleared") is honored by construction now.

### I-2 — six more self-check fault rows, one filed as F-461 (commit `83e932a`)

**VERIFIED — all six arms individually mutation-tested against the real source.**

Applied `if false {` to each of the six arms in `gui/composer_selfcheck.go` (one at a
time, each reverted before the next) and ran
`TestComposerSelfCheckRefusesAFaultInjectedBuilderOutput`:

| arm (line in current file) | result |
| --- | --- |
| lock VALUE compare (`:126`) | `FAIL .../a_path's_lock_VALUE_moves` |
| sha256 digest compare (`:138`) | `FAIL .../a_path's_sha256_digest_moves` |
| unseated-slot fingerprint presence (`:160`) | `FAIL .../an_UNSEATED_slot_declares_a_fingerprint` |
| seated-slot fingerprint presence (`:169`) | `FAIL .../a_seated_slot's_fingerprint_PRESENCE_differs` |
| §4f invariant on decoded md1 (`:196`) | `FAIL .../the_decoded_md1_puts_two_slots_at_ONE_origin_with_no_fingerprints` |
| use-site dispatch (`:153`) | **whole `^TestComposer...FaultInjected...` suite still PASSES** |

Five of six kill their own named row exactly as claimed; the sixth (use-site dispatch)
genuinely survives, confirming the report's own "SURVIVED" claim rather than merely
repeating it.

**F-461's predicate test and reproduction, independently re-run:**
`TestComposerUseSiteGuardRefusesEveryShapeButTheFixedOne` drives `composerUseSiteIsFixed`
over seven shapes — ran it, 7/7 PASS. Then ran F-461's own stated reproduction verbatim —
mutate the use-site dispatch to `if false {` and run
`go test -run '^TestComposerSelfCheck|^TestComposerUseSiteGuard|^TestComposerConsentRefuses' ./gui/`
— result: `ok`, exactly as the followup states. Reverted; tree clean.

**F-461's "61 vendored compose vectors" claim, independently checked.** `ls
md/testdata/vectors/*.bytes.hex | wc -l` = **61**, confirming this is the count of the
whole vendored golden-vector corpus (not literally all named "compose"). Wrote a throwaway
probe decoding every vector's `.phrase.txt` via `ExpandWalletPolicyChunks` and scanning
every key's `UseSite`: 61 total, 48 decode as wallet-policy chunks, **0 carry a non-fixed
`<0;1>/*` use-site**. Supports the claim that no fixture in this tree can drive the
unreachable arm. Probe file removed; tree clean.

**Verdict: I-2 closed as filed — five arms gated with a named-mutation kill, the sixth
correctly filed rather than faked, and F-461's own reproduction and vector-count claim
both reproduce exactly.**

---

## 2. Minors and Nits — each accounted for

| # | disposition | verified |
| --- | --- | --- |
| M-1 (three lock bands untestable) | folded, `27afa9f` | **VERIFIED** — pure extraction (`composerBlocksBandEcho`/`DaysBandEcho`/`HeightBandEcho`), no behavior change (diff confirms callers unchanged). Named mutations `n>65535→65536`, `n>388→389`, `n>499999999→500000000` each independently applied and each fails exactly `.../blocks_one_past`, `.../days_one_past`, `.../height_one_past` respectively (own runs, not transcribed). Reverted; clean. |
| M-2 (census ceiling refusal, described but absent) | folded (spec), `db53513` | **VERIFIED** — spec now says NOT IMPLEMENTED, points at F-457, §13 item 1's measured ceiling (596) retained. Gates re-run on the folded spec: `plan-cite-check.sh` 76/76 ok 0 dangling; `plan-glyph-check.sh` 113 scanned, 0 undrawable; `plan-table-check.sh` 137 checked, 0 malformed; `spec-structure-check.sh` 56 sections, 49 cross-refs, STRUCTURE OK. All four counts match the commit message exactly. |
| M-3 (three copy bodies not §8 blockquotes) | folded (spec), `db53513` | **VERIFIED** — §8c now blockquotes the blocks-echo and packed-height bodies, §8t gains the date ceiling. Independently re-ran the review's own spec-vs-table diff (extracted every `>` paragraph in §8, normalized, set-diffed against `composerCopyTable()`'s verbatim column via a throwaway dump test): **44 blockquotes, 41 table rows, exactly one non-blockquote row — `composerCopySameXpub` (§7d)** — which §11 explicitly admits as "a quoted string in its table." Matches the fold report's re-run precisely. |
| M-4 (same-xpub body outside the copy table) | folded, `27afa9f` | **VERIFIED with a numeric correction — see "New finding" below.** `composerCopySameXpub` now has its own table row; deleting the row makes `TestComposerCopyTableCoversTheSameXpubRefusal` fail exactly as claimed ("composerCopySameXpub is not in composerCopyTable ..."). The body fits the modal (headroom well above the 80-char margin) — but the measured character count is **55, not 57** as both the review r0 report and the fold report state. See below; this is a report-text defect, not a code or gate defect — the gate itself measures correctly. |
| M-5 (`ErrComposeIndistinguishableSlots` unmapped) | folded, `27afa9f` | **VERIFIED** — `composerRefusalBody` gains the arm. Deleting it makes `TestComposerRefusalBodyMapsTheIndistinguishableSlotsSentinel` fail with the exact named message ("has no §8m/§8v arm ..."). |
| N-1 (re-mint identity fields unasserted) | folded, `27afa9f` | **VERIFIED** — applied the exact named mutation `card.Fingerprint = "00000000"` after re-mint; `TestComposerReMintCarriesTheSourceCardsIdentityFields` fails with the exact quoted message. |
| N-2 (C29 groups in map order) | folded, `27afa9f` | **VERIFIED** — reverted `composerSharedSeedInPath` to the pre-fold map-ranging form (removed the `sort.Slice` and the two-pass group collection); `TestComposerC29GroupsRenderInSlotOrder` fails on run 0 with "render in a different order on run 0 (group 0 starts at slot @1, was @0)" — matches. |
| N-3 (`gofmt -l` lists five, not three) | no code change; the fold's "no change needed" reasoning verified | **VERIFIED** — `gofmt -l .` on the fold still lists exactly the same 5 files (`gui/transaction.go`, `gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go`); `git diff --stat 321acb56..27afa9f -- <those 5 files>` is empty, confirming none of them were touched by this cycle. |
| I-2 use-site arm | filed, `b1a1985` (F-461) | **VERIFIED** — see §1 above; filing's reproduction and vector-count claim both independently reproduce. |

### New finding — M-4's measured character count is wrong in both reports (Minor)

`composerCopySameXpub(0, 1)` returns `"Slots @0 and @1 hold the same key. Every slot
needs a different key."`. Both `composer-S3-exec-review-r0.md` (M-4) and
`composer-S3-fold-r0-report.md` (§6, M-4) state the modal-fits measurement as "57 chars
drawn in full, headroom 494." Running `TestComposerCopyTableCoversTheSameXpubRefusal`
directly prints:

```
composer_copy_test.go:224: the §7d same-xpub refusal: 55 chars drawn in full, headroom 494 chars (margin 80)
```

**55, not 57** — reproducibly, on both the fold tip and after undoing/redoing the row
deletion. `bodyDrawnFully` counts `normalizeDrawn` (lowercased, whitespace-stripped)
characters; a manual count of the normalized string confirms 55. This is a two-character
transcription error that was carried from the round-0 review into the fold report without
being re-measured at fold time — the fold's own "Machine-checkable claims get
machine-checked" standard would have caught it. It does not affect the gate's behavior:
the headroom (494) and the pass/fail outcome are unaffected, and the test still correctly
enforces the margin. **Severity: Minor** (a wrong number in a persisted record, not a
functional or coverage defect) — filed here rather than in `FOLLOWUPS.md` since it
requires no further work, only a note for whoever next edits those two report files.

---

## 3. Nothing else moved

- **File-level accounting.** Every touched file in all three fork commits maps to a named
  finding: `7edc863` → C-1/I-1 (`composer_flow.go`, `composer_join_test.go`,
  `composer_seat.go`, `composer_shape.go`); `83e932a` → I-2 + the second Critical
  (`composer_selfcheck.go`, `composer_selfcheck_test.go`); `27afa9f` → M-1/M-4/M-5/N-1/N-2
  (`composer_cards_test.go`, `composer_copy.go`, `composer_copy_test.go`,
  `composer_lock.go`, `composer_lock_test.go`, `composer_review.go`,
  `composer_review_test.go`, `composer_shape.go`, `composer_shape_test.go`). Master:
  `db53513` → `design/SPEC_wallet_policy_composer.md` (M-2/M-3); `b1a1985` →
  `design/FOLLOWUPS.md` (F-461). No stray file in any commit.
- `gofmt -l .` on the fold: same 5 pre-existing files, confirmed untouched by this diff
  (§2, N-3).
- `go vet ./gui/`: only the two pre-existing go1.25 `ArtifactDir` findings
  (`freetext_sizeproof_golden_test.go:111`, `transaction_golden_test.go:104`).
- `go test -count=1 -run '^TestComposer' -v ./gui/`: **127 top-level PASS, 0 FAIL, 150
  sub-test PASS** (counted directly from the run's own `--- PASS`/`--- FAIL` lines, not
  read from the fold report) — matches "127 top-level / 150 sub-test" exactly.
- `go test -count=1 -timeout 20m $(go list ./... | grep -v '/gui$')`: 53 ok, 0 FAIL.
  `go test -count=1 -timeout 20m ./gui/`: 1 ok (108.8s). Total **54 ok, 0 FAIL** — matches.
- `scripts/gui-shard-test.sh ./gui/ 24`: **1185 tests across 24 shards, all ok** — matches
  exactly.
- `-tags oraclelive -run '^$' ./oracle/ ./gui/ ./sysw/`: ok, no tests to run (all three).
- `scripts/test-32bit.sh`: `GOARCH=386 test: exit 0 ; GOARCH=arm build: exit 0`.
- `GOOS=js GOARCH=wasm go vet ./cmd/emu/`: clean.
- `go test -count=1 -run Needle -v ./cmd/emu/`: 7/7 PASS.
- **Firmware size — the step that "now runs."** Ran
  `nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 -stack-size
  16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller` on the fold tip in the
  worktree copy:
  ```
     code    data     bss |   flash     ram
  1548128   31796   31004 | 1579924   62800
  ```
  **1,579,924 B flash / 62,800 B RAM — byte-for-byte the fold report's own numbers.**
  Then independently checked out fork `321acb56` in a *separate* scratch copy and ran the
  identical command:
  ```
     code    data     bss |   flash     ram
  1475248   31636   30956 | 1506884   62592
  ```
  **1,506,884 B / 62,592 B — matches the stated baseline exactly.** Delta: **+73,040 B
  flash / +208 B RAM**, non-zero, satisfying the plan's "a zero delta is Critical" gate.
  Both numbers measured independently in this round, not copied from either report.

**Verdict: nothing outside the findings moved. Every gate reproduces the fold report's
own numbers exactly, measured independently rather than read.**

---

## Closing counts

- **0 Critical / 0 Important open.** The one Critical the review found (C-1) and the
  second Critical the fold found (composerSelfCheck's K/N domain error) are both fixed,
  both verified in both directions (mutation fails, fix passes, control test exists and
  passes on the fold).
- **5 Minors, 3 Nits: all accounted for** — folded, filed with an owning phase, or
  declined with a reason, each independently confirmed above.
- **1 new Minor filed by this round**: the M-4 character-count transcription error (55 vs.
  57), a report-text defect only — no code or gate change needed.
- **Every new or changed guard's named mutation was independently applied and killed the
  named test**, with output pasted above from this round's own runs.
- **The firmware size gate — the one gate that had never executed in the whole S3
  cycle — ran, and both its numbers (tip and baseline) were independently reproduced.**

**Gate reproduces:** `go test -count=1 -run '^TestComposer' -v ./gui/` → 127/127 top-level,
150/150 sub-test, 0 FAIL. `gui-shard-test.sh ./gui/ 24` → 1185/1185. `go test ./...`
(minus/plus gui) → 54/54 ok. `test-32bit.sh`, oraclelive, js vet, Needle → all clean.
`tinygo build -size short` → 1,579,924 B / 62,800 B (delta +73,040 / +208 vs. baseline
1,506,884 / 62,592). `git status --porcelain` empty in the reviewed worktree and in
mnemonic-engrave at the end of this review.

---

## Worktree metadata incident (disclosed, fully repaired, does not affect any finding above)

The brief's isolation instruction was to `cp -r` the fork worktree into
`/scratch/code/shibboleth/.s3-verify/` for every mutation and checkout. A git worktree's
`.git` is not a directory but a **gitlink file** (`gitdir:
/scratch/code/shibboleth/seedhammer/.git/worktrees/wt-composer-s3`) pointing at shared
metadata in the main repo — `cp -r` duplicates the working files but not that metadata,
so the copy and the original worktree share one HEAD ref and one index.

Read-only git commands (`status`, `diff <ref> <ref>`, `show`, `log`) are unaffected. One
step of this review — establishing the firmware-size baseline — made a second-generation
copy and ran `git checkout 321acb56` inside it to measure the pre-S3 baseline. Because
that copy's gitlink pointed at the same shared metadata, the checkout moved the **shared**
HEAD to a detached `321acb56` and rewrote the **shared** index to that tree. This did not
touch any file's contents anywhere (the branch ref `refs/heads/composer-s3` is a separate,
untouched ref, and every file on disk in the original worktree remained byte-identical to
`27afa9f` throughout — confirmed via `git hash-object` against the `27afa9f` blob before
any repair). Its only visible effect was that `git status` in the **original** worktree
briefly reported the diff between `321acb56` and the (unchanged, `27afa9f`) files on disk
as a wall of modified/untracked paths.

**Repair, performed in this session:** `git symbolic-ref HEAD refs/heads/composer-s3`
(re-attaches HEAD to the branch, which was never moved and was still at `27afa9f`), then
`git reset --mixed HEAD` (updates the index to match `27afa9f`, touching no working-tree
file). Verified after repair: `git worktree list` shows
`/scratch/code/shibboleth/wt-composer-s3  27afa9f [composer-s3]`; `git status --porcelain`
is empty in both the original worktree and every copy sharing its gitdir. The throwaway
baseline copy was then deleted, and all `.s3-verify/` copies were deleted at the end of
this review as planned.

No finding in this report depends on file content read during the affected window — every
diff and test result above was taken from files verified byte-identical to their intended
commit, and the fix restored the shared metadata to the exact state it should have carried
throughout.
