# composer S4 — fold r1: the W-3 merge into the driver branch

**Brief:** `design/agent-briefs/composer-S4-fold-r1-brief.md`. **Implementer:**
the same single opus agent, resumed. W-3 merged into fork `main` as
`1ae0ffcb3cd61ddc176eb2f1b9b365558185d982`.

**Outcome: merged, verified, nothing to re-pin.** The predictable consequence I
named in the W-3 fix report (item 4) — that the stub screens would page one more
time and the driver's two `pages.length !== 2` assertions would fail — **did not
materialise. My prediction was wrong.** The page counts are unchanged, every
gate is green, and the only edit anywhere is the transcript's fork-rev line.

## The merge

```
$ git -C /scratch/code/shibboleth/seedhammer rev-parse main
1ae0ffcb3cd61ddc176eb2f1b9b365558185d982

$ git merge --no-ff main -F <msg>
MERGE EXIT=0
Merge made by the 'ort' strategy.
 gui/composer_measure_test.go        |  15 +++
 gui/composer_paged.go               |  41 ++++--
 gui/composer_paged_geometry_test.go | 240 ++++++++++++++++++++++++++++++++++++
 3 files changed, 287 insertions(+), 9 deletions(-)
```

**No conflict**, as predicted — W-3 touched `gui/` only, this branch `cmd/emu/`
only. `git status --porcelain` empty after (exit 0).

## New tips

```
$ git -C /scratch/code/shibboleth/wt-composer-s4-emu log --oneline main..HEAD
b481be7 Merge main into composer-s4-emu: W-3, paged lines wrap clear of the nav column
a6eb44e emu: fold the S4 whole-diff review -- M-1, M-2, N-1, N-4 (composer-S4-exec-review-r0)
86cec95 emu: the composer journey's walk -- shots_composer.js, and shTargets to tap rows by (S4 Task 3)
a79a454 Merge main into composer-s4-emu: W-2, the pick screen's rows are touch targets (S4 Task 3 needs it)
05d903b emu: a THIRD test payload carrying the composer's own record classes (S4 Task 1)

$ git -C /scratch/code/shibboleth/wt-engrave-s4-emu log --oneline master..HEAD
55db8e5 journeys: regenerate transcript_composer.txt at the W-3 merge tip (S4 fold r1)
651fa0e journeys: regenerate transcript_composer.txt at the fork tip (S4 review N-3)
05a066a journeys: fold the S4 whole-diff review -- I-1, M-3, M-4 (composer-S4-exec-review-r0)
c6adac2 journeys: the composer's device half -- capture_composer.py (S4 Task 3)
5040bb2 journeys: the composer's host half -- transcript_composer.sh (S4 Task 2)
```

Both trees clean. Nothing pushed, nothing flashed, no sub-agent, no `.jsonl`
read. Mutations ran in `cp -r` copies under `/scratch/code/shibboleth/.tmp/`
with no `.git` reachable.

## The capture run the brief asked for FIRST — it passed

```
$ EMU=…/wt-composer-s4-emu/cmd/emu python3 capture_composer.py --arm both --no-build --port 8803 --shot-port 8744
ARM BOTH (merged, before any pin) EXIT=0
  keyed-A 21 shots, census 2 | keyed-B 21 shots, census 4 | keyless 8 shots, census 1
  all legs matched the host.
```

**There is no failure to paste.** The measured page counts, against the values
recorded before the merge:

| leg | stubPages | mapping | consent | shots | census | before |
| --- | --- | --- | --- | --- | --- | --- |
| keyed-A | `[2, 3]` | 2 | 4 | 21 | 2 | identical |
| keyed-B | `[2, 3]` | 2 | 4 | 21 | 4 | identical |
| keyless | `2` | – | 2 | 8 | 1 | identical |

`variantRowsTaken` (`['TEXT ONLY','TEXT ONLY']`, four `TEXT ONLY`, `['TEXT + QR']`),
the engraved string counts (7 / 9 / 1) and the set of 30 shot names are
identical too. **Nothing the merge moved is measurable in the driver's surface,
so nothing was re-pinned.** The two assertions already sit at the measured
value:

```
cmd/emu/shots_composer.js:514   if (stub.pages.length !== 2)     // keyless
cmd/emu/shots_composer.js:690   if (stub1.pages.length !== 2)    // keyed
```

They were left exactly as they are — a pin at 2, which is what the screens do.
Nothing was loosened, and nothing needed tightening.

## Why my prediction was wrong

I reasoned from the §13 measurement, where the **32-slot** stub screen's
capacity fell from 7 rows per frame to 6, and assumed the fixture's stub screens
would page once more. Two things I did not check:

1. **A wrapped line is still ONE row.** `composerPageLines` lays out each input
   string as one row whose *height* may span two visual lines. Wrapping the
   Template-ID grows the row, it does not add one — so the body's row count is
   unchanged and only the per-frame budget moves.
2. **The fixture's stub screens are short.** They hold a handful of rows, not 42,
   so losing one row of budget while gaining one row of height lands on the same
   page count. Measured on the emulator, before and after: page 0's last
   paragraph (`A wallet built here is its own wallet…`) simply moved to page 1.
   The **distribution** changed; the **count** did not.

The honest summary is that a page-count pin could never have caught W-3, and did
not. **The shots caught it, and only because someone looked at them.** That is
worth carrying forward: `ExtractText` sees text under a button, and so does a
page count.

## The 32 hex digits, on the merged build — looked at, not inferred

```
/scratch/code/shibboleth/wt-engrave-s4-emu/design/journeys/shots/c06-stub-p0.png
/scratch/code/shibboleth/wt-engrave-s4-emu/design/journeys/shots/c06-stub-p1.png
/scratch/code/shibboleth/wt-engrave-s4-emu/design/journeys/shots/c10-stub2-p0.png
/scratch/code/shibboleth/wt-engrave-s4-emu/design/journeys/shots/c10-stub2-p1.png
/scratch/code/shibboleth/wt-engrave-s4-emu/design/journeys/shots/c10-stub2-p2.png
/scratch/code/shibboleth/wt-engrave-s4-emu/design/journeys/shots/k02-stub-p0.png
/scratch/code/shibboleth/wt-engrave-s4-emu/design/journeys/shots/k02-stub-p1.png
```

| shot | what I see |
| --- | --- |
| `k02-stub-p0.png` | `Template-ID:` on its own line, then `e0863d3ccac31a64d3b5e14b85ccd6c0` complete, ending `c0` well clear of Back; both `mk encode` lines end clear of the pager |
| `c06-stub-p0.png` | `Template-ID: 531ab9e1777f018ae53694387dd0d128` on ONE line, all 32 digits, ending `8` clear of Back |
| `c10-stub2-p0.png` | `Template-ID: 531ab9e1…d0d128` **and** `Policy-ID: 4dd749a8372af515a61d7104faf944ef` both complete and clear; `--policy-id-stub 531ab9e1 --policy-id-stub` wraps, with `4dd749a8` on the next line, clear of the pager |

**The two ids render differently and both are now safe**, which is itself a
measurement rather than luck. Rendered through the widget, before and after:

```
                                                       pre-W-3   merged
Template-ID: 531ab9e1777f018ae53694387dd0d128          UNDER      clear
Template-ID: e0863d3ccac31a64d3b5e14b85ccd6c0          UNDER      clear
--policy-id-stub 531ab9e1 --policy-id-stub 4dd749a8    UNDER      clear
mk encode --xpub <xpub> --origin-fingerprint <fp>      UNDER      clear
Policy-ID: 4dd749a8372af515a61d7104faf944ef            clear      clear
```

(`UNDER` = the geometry probe found ink inside button `(427,44)-(480,97)`.) So
the keyed fixture's **own** Template-ID was being cut on the shipped build too,
not only the key-less one — the walk had been photographing a truncated id on
every keyed leg. The `Policy-ID` line was always clear, because its label is
shorter; that is why it never showed the symptom.

## Gates

| gate | exit | detail |
| --- | --- | --- |
| `capture_composer.py --arm both` (final, post-transcript) | **0** | keyed-A 62 s / keyed-B 74 s / keyless 28 s, 50 shots, `all legs matched the host.` |
| `--arm keyed --prove-it-can-fail` (honest) | **0** | `NEGATIVE CONTROL PASSED`, 39 s, naming `bc1q8cf5g5f…tp4q` |
| `--prove-it-can-fail` with the digest corrupted | **1** | `NEGATIVE CONTROL INCONCLUSIVE: the walk failed before the comparison …`, 8 s |
| the plan's unchunked-string mutation | **1** | `device: … (56 chars)` / `host: … (47 chars)` |
| `capture_walletpolicy.py` | **0** | wallet id `4e67c6fd…` |
| `capture_seating.py` | **0** | wallet id `c8fe87cd…` |
| `capture_tr_pathological.py` | **0** | wallet id `590f3abc…` |
| `gofmt -l cmd/` | **0** | no output |
| `CGO_ENABLED=0 go test -count=1 ./cmd/emu/` | **0** | `ok  seedhammer.com/cmd/emu  2.206s` |
| `GOOS=js GOARCH=wasm go vet ./cmd/emu/` | **0** | |
| the merged tree's new gui tests (W-2, W-3, N-4, digit pad) | **0** | `ok  seedhammer.com/gui  0.110s` |
| `transcript_composer.sh` | **0** | 27 GATE PASS, 0 GATE FAIL |

Every exit code above was recorded directly from the command, never through a
pipe.

## The transcript (regenerated last)

The fork tip moved `a6eb44e → b481be7`, so the rev line was stale. Regenerated
with `FORK=/scratch/code/shibboleth/wt-composer-s4-emu`. **The whole diff:**

```
24c24
< a6eb44e
---
> b481be7
462-483:  the `ls -la` mtimes
```

and nothing else. Substantively byte-identical:

```
$ diff <(strip before) <(strip after)          SUBSTANTIVE DIFF EXIT=0
```

Every artifact `cmp` EXIT=0 against its pre-regeneration copy —
`keyed.id.txt`, `keyed.receive.txt`, `keyed.change.txt`, `keyed.md1.txt`,
`keyed-template.md1.txt`, `keyless-tr.md1.txt`, `payload.digest.txt`,
`payload.bin` — and `payload.bin` is still byte-identical to
`cmd/emu/sysw_composer_payload.bin`.

## What I decided, and what I could not do

1. **I changed no assertion**, because none had moved. The brief's instruction
   was to pin the measured values and never loosen; the measured values are the
   pinned values. Editing them to say the same thing would have been churn, and
   relaxing them was forbidden — correctly.
2. **I recorded that my own prediction was wrong**, with the measurement that
   explains it, rather than quietly finding nothing. The W-3 report's item 4
   should be read as superseded by this one.
3. **An observation, not a change:** the stub screens' page counts (`stub2 = 3`,
   `mapping = 2`, `consent = 4`) are recorded but not asserted, and this episode
   shows a count is a weak pin for a layout change — it survived W-3 unchanged
   while every keyed leg was photographing a truncated Template-ID. The check
   that would have caught it is the geometry test now on `main`
   (`gui/composer_paged_geometry_test.go`), which runs in CI on every push. I did
   not add driver-side pins for those counts; if the controller wants the walk to
   assert legibility rather than presence, that is a new lens, not a re-pin.
4. **Not done, and not mine:** merging or pushing either branch, flashing, and
   Task 4's live device walk (which the plan gates on W-2 — and now W-3 — being
   flashed).
