# H5 device polish — R0 round-2 fold verification

**Scope**: verify fold commit `44b1690` (over `d36ede5`) against round 1's four Importants
(`hashlock-H5-spec-R0-r1-fold-verification.md`, persisted `77621fb`). Round 0 not re-reviewed.
**Ground**: fork main `b9a9a30`, detached worktree `/scratch/code/shibboleth/.tmp/h5-r2`
(removed after use), Go 1.26.7 (`/scratch/code/shibboleth/.toolchain/go`). Read-only on both
repos; nothing committed; no sub-agents; no `.jsonl` read. Diff reviewed:
`git diff d36ede5..44b1690 -- design/SPEC_hashlock_H5_device_polish.md`.

**Counts: 0 Critical / 0 Important / 1 new Minor (see §3). GREEN.**

---

## 1. R1-I-1..R1-I-4 — fold change and verdict

| Finding | Fold change (`44b1690`) | Verdict |
| --- | --- | --- |
| R1-I-1 (§1.1 headroom 205/320, stale) | Text now reads "186 characters drawn in full, headroom 339" | **FIXED** — re-measured myself (see §2): **186/339, exact match** |
| R1-I-2 (§2.5 "the new sentence is shorter", false) | Text now reads "165 drawn (was 160), headroom unchanged at 378" | **FIXED** — re-measured myself (see §2): **old 160/378, new 165/378, exact match** |
| R1-I-3 (§1.4 sent reconcile clause to H2 §4.7, contradicting §2.5) | §1 item 4 now reads "...§4.5's post-HOLD reconcile clause quotes item 1's body (the reconcile text lives only in §4.5; §4.7 changes only through §2.5's phrase-form sentence)" | **FIXED** — confirmed against `SPEC_hashlock_H2_device.md`: the reconcile sentence ("Before you fund this wallet, run ms hashlock...") is part of §4.5's confirm-modal body; §4.7's only body text is the §8h phrase form §2.5 edits. No more contradiction; `grep -n "and §4.7"` on the live spec returns nothing |
| R1-I-4 (`composer_hashlock_test.go:916` unlisted, compile break unacknowledged) | §2 item 3 now lists "the six sites" incl. `composer_hashlock_test.go:916`; §6 now reads "...has its :916 assertion rewritten to `composerAnyPathByPhrase(st)` in Task 1 (it does not compile otherwise)" | **FIXED** — confirmed `:916` is `if !st.hashByPhrase {` inside `TestHashlockReconcileScreenIsReachableOnAMixedPolicy` (function starts `:882`, per `awk` function-boundary check) at `b9a9a30`; both the site list and the compile-dependency are now explicit |

## 2. Re-measurement (own detached worktree, `assertModalBodyFits` on `errorScreenBody` at `sh2DisplaySize`)

Built the exact fenced/quoted bodies from the folded spec text (not retyped from the report)
into a temporary, uncommitted `gui/zzz_h5_r2_measure_test.go`, ran under
`/scratch/code/shibboleth/.toolchain/go/bin/go test ./gui/ -run TestH5R2Measure -v`, then
deleted the file and removed the worktree:

```
H5-R2 fold reconcile body (hardened, chars: 100): 186 chars drawn in full, headroom 339 chars (margin 80)
H5-R2 OLD every-path-phrase body: 160 chars drawn in full, headroom 378 chars (margin 80)
H5-R2 NEW every-path-phrase body (fold): 165 chars drawn in full, headroom 378 chars (margin 80)
```

Matches the folded spec's claims exactly: **186/339** (§1 item 1) and **165 drawn (was 160),
headroom unchanged at 378** (§2 item 5).

## 3. New defect / new looseness check

Re-grepped the live spec for every superseded number/phrase — `205 characters`, `headroom
320`, `the new sentence is shorter`, `five sites that reference` — all absent. `186`, `339`,
`165 drawn`, `headroom unchanged at 378`, `six sites` all present and consistent; `:916`
appears at three places (§2 item 3, §6, and the new recap section) with no conflicting claim.

One new, non-blocking item: §6's fixed text adds *"...rewritten to `composerAnyPathByPhrase(st)`
in Task 1..."* — "Task 1" is a forward reference to an implementation-plan task breakdown that
does not exist anywhere yet (no `IMPLEMENTATION_PLAN_hashlock_H5*` file exists in `design/`,
and no other `SPEC_*.md` in this repo cites a bare "Task N"). It isn't a false claim (the
constraint it states — the rewrite must land with the field deletion or the build breaks — is
correct regardless of which task number does it) but it presupposes a plan structure this spec
doesn't define. **New Minor**, same class as r1's §3 citation-looseness findings, not graded
above Minor per the brief.

The four r1 §3 Minors (`:342`/`:388` "confirm rows" mislabel; §2 items numbered 1,2,3,5,4;
`composer_copy_test.go:130-137` spanning two rows; `unlock_kdf.go:391-393` vs `:388-393`) are
all **still present, unchanged, at `44b1690`** — expected, since the brief states the controller
folds these separately from this Important-only fold. (Informational, out of my review scope:
they have since been folded in a later commit, `d206a2e`, on top of `44b1690` — not reviewed
here.)

## Closing

**0 Critical, 0 Important.** All four round-1 Importants are fixed exactly as the fold's own
recap claims, re-measured independently and matching to the character; the §4.7/§4.5
contradiction is resolved and checked against H2's actual section content; the `:916` site is
now listed with its rewrite and the compile dependency stated. No new false number or
contradiction was introduced; one new Minor (an ungrounded "Task 1" forward reference) is
recorded but does not block.

**GREEN.**
