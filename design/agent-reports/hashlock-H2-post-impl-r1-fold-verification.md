# hashlock H2 post-implementation fold — R1 fold-verification (sonnet, independent)

**Scope.** Verify fork commit `26fd1dd` (branch `hashlock-h2`) against the opus
post-impl review (`design/agent-reports/hashlock-H2-post-impl.md`, 0C/2I/3M/2N
at fork `17b3979`), and engrave records commit `67f9fe9` (branch `hashlock-h2`).
Own detached worktree: `/scratch/code/shibboleth/.tmp/h2-fold-verify` (removed
at the end of this run). Read-only on both branch worktrees. No sub-agents. No
`.jsonl` read.

**Verdict: GREEN.** Every claim in the fork commit message and the
implementation-report addendum reproduced exactly when executed. No new
defect found. One Minor observation (gofmt scope wording), not blocking.

---

## 1. `git diff 17b3979..26fd1dd --stat`, read whole

```
 gui/composer_hashlock.go      |  4 +++-
 gui/composer_hashlock_test.go | 53 ++++++++++++++++++++++++++++++++++++++++++-
 hashlock/hashlock.go          | 22 +++++++++++++++---
 3 files changed, 74 insertions(+), 5 deletions(-)
```

Read in full (`git diff 17b3979..26fd1dd -- gui/composer_hashlock.go
hashlock/hashlock.go gui/composer_hashlock_test.go`). Three changes:
`hashlockPhraseFlow` no longer cuts 8px off the bottom of `content` (F-481);
`PreimageHardened`/`DeriveHardened` fail closed on a nil `Deriver.Key()`
(M-2); `IsMS1Shaped` case-folds ASCII-only via `strings.Map` instead of
`strings.ToLower` (N-2). Two new tests
(`TestHashlockPhraseScreenDrawsTheMaskedReadout`,
`TestHashlockRefusalCopyCoversEverySentinel`) plus two new assertions inside
the existing `TestHashlockPhraseRouteSetsTheCorpusDigest` (I-1).

## 2. I-1 mutations (`hashlockFirst8Last8` / `chars:` count)

Baseline: `TestHashlockPhraseRouteSetsTheCorpusDigest` PASS (5.11s, both
subtests).

**Mutation A** — `s[len(s)-8:]` → `s[:8]` in `hashlockFirst8Last8`:
```
composer_hashlock_test.go:371: the confirm modal drew "hash3cf5d421..3cf5d421...",
  want it to contain "hash 3cf5d421..b70a4c12"
composer_hashlock_test.go:371: the confirm modal drew "hashb867db87..b867db87...",
  want it to contain "hash b867db87..edbc96cb"
--- FAIL: TestHashlockPhraseRouteSetsTheCorpusDigest (both subtests)
```
Matches the addendum's cited failure (`hashb867db87..b867db87`) verbatim.
Reverted; tree confirmed clean.

**Mutation B** — `len(phrase)` → `len(phrase)+1` at the
`composerCopyHashlockConfirm` call:
```
composer_hashlock_test.go:375: the confirm modal drew "...chars:29...",
  want it to contain "chars: 28"
--- FAIL: TestHashlockPhraseRouteSetsTheCorpusDigest (both subtests)
```
Matches the addendum's `chars:29 ... want "chars: 28"` verbatim. Reverted;
tree confirmed clean.

## 3. I-2 / F-481 mutation, then geometry

**Mutation** — restored `content, _ = content.CutBottom(8)` after
`_, content := screen.CutTop(leadingSize)`:
```
composer_hashlock_test.go:979: the phrase screen drew 0 asterisks for 10
  typed characters; the readout is not drawn (F-481).
--- FAIL: TestHashlockPhraseScreenDrawsTheMaskedReadout
```
Exactly the addendum's claim ("0 with the cut restored"). Reverted; tree
confirmed clean.

**Geometry, not just text.** Built a scratch measurement in the worktree
(never committed, deleted before the whole-suite gate run) that:

1. Reproduces `hashlockPhraseFlow`'s own arithmetic at the real panel size
   (`sh2DisplaySize = (480,320)`; the default 240×240 test display is, per
   `gui/gui_test.go`'s own comment, "a fiction that no shipped device has"):

   | quantity | value |
   | --- | --- |
   | `dims` | (480, 320) |
   | after `CutTop(leadingSize=44)` | Dy=276 |
   | after lead-band cut (`leadSz.Y`=44 for the real lead string) | Dy=232 |
   | after counter-band cut (`cntSz.Y`=23 for `"0/100"`) | Dy=209 |
   | **PRE-FIX** `MaxHeight` (extra `CutBottom(8)`) | **201** |
   | **POST-FIX** `MaxHeight` (no cut) | **209** |
   | grid height `k.size[page].Y` | 182 |
   | readoutGap | 8 |
   | **PRE-FIX avail** = MaxHeight − grid.Y − gap | **11** |
   | **POST-FIX avail** = MaxHeight − grid.Y − gap | **19** |
   | one masked line's height (word style, width 340, `"*"`) | **19** |

   Pre-fix: 11 < 19 by 8px — a masked line categorically cannot fit (matches
   the new test's own comment: "readout budget at 11 px (one line needs
   19)"). Post-fix: 19 ≥ 19 — fits, with **zero pixels of margin**. This is a
   real observation: any future change that grows the lead or counter band by
   even 1px, or changes `readoutGap`, regresses the readout to zero lines
   again with no test currently pinning the margin. Not a defect in `26fd1dd`
   (F-481 as filed is fixed), so **not blocking** — recorded as a Minor
   headroom note for whoever next touches this screen's layout.

2. Rasterizes `PassphraseKeyboard.Layout`'s own returned op (not the full
   screen, not `ExtractText`) via `op.Drawer.Draw` into an `image.RGBA`, with
   `Fragment = "abcdefghij"` (10 chars, masked), and counts ink **per row
   band**: rows above the grid's top (`gridTop = combined.Y − grid.Y`) vs.
   rows at/after it — a genuine pixel-level geometric check, independent of
   any text extraction:

   ```
   PRE-FIX:  widget size=(340,209) gridTop(y)=27 readout-band ink=0     grid-band ink=12618
   POST-FIX: widget size=(340,209) gridTop(y)=27 readout-band ink=260   grid-band ink=12618
   ```

   Pre-fix the readout band (rows 0–26) draws **zero** ink for a 10-character
   fragment — reproduces F-481's symptom at the pixel level, not merely via a
   string match. Post-fix it draws 260 ink pixels in exactly that band, with
   the grid band unchanged (12618 both times, as expected — the grid itself
   never moved).

   Composing this with the full-screen offsets (`content.Min.Y=111` after the
   top/lead/counter cuts, south-aligned placement of the combined
   readout+grid block against `content`, whose `Dy()=209` post-fix equals the
   combined block's own height exactly): the readout occupies absolute
   `y∈[111,138)`, i.e. **below** the counter band (`y∈[88,111)`) and **above**
   the grid (`y∈[138,320)`), with zero slack on both seams — geometrically
   exactly where the review's caveat asked it be confirmed.

   The harness does not expose per-glyph drawn-text positions:
   `op.Drawer.ExtractText` (`gui/op/op.go:617`) discards the rasterized image
   after collecting only the concatenated rune string (`d.text`), and
   `TagBounds`/`Hit` (`op.go:630-646`) report bounds only for **input** tags
   (buttons), not text runs. So the raster/ink approach above — already used
   in-repo by `gui/raster_test.go`'s `countInk` for a different regression
   (F-151) — was the available geometric instrument, and it directly answers
   the review's ask.

## 4. N-1, M-2, N-2

**N-1 mutation** — deleted the `hashlock.ErrHex64` case in
`composerCopyHashlockRefusal`:
```
composer_hashlock_test.go:989: composerCopyHashlockRefusal(hashlock: that is
  a preimage in hex, not a phrase) = "hashlock: that is a preimage in hex,
  not a phrase": fell through to the Go error
--- FAIL: TestHashlockRefusalCopyCoversEverySentinel
```
Matches the claim. Reverted; tree confirmed clean.

**M-2** — read `seal/pbkdf2.go` source directly:
```go
func (d *Deriver) Key() []byte {
    if d.dead || d.total == 0 || d.done < d.total {
        return nil
    }
    return append([]byte(nil), d.acc[:]...)
}
func (d *Deriver) Wipe() {
    d.dead = true
    ...
}
```
`Wipe` sets `d.dead = true` unconditionally, and `Key()`'s first clause
returns nil whenever `d.dead`. So a post-`Wipe` `Key()` is nil in every case,
confirming the guard is real. In `PreimageHardened`, `defer d.Wipe()` runs
only at return, after `d.Key()` is read, so the guard there is unreachable
today exactly as the code comment and addendum say — kept for the contract.
Ran the lockstep/corpus tests to confirm the guard changed no live value:
```
=== RUN   TestDerivationRowsLockstep
--- PASS (0.23s)
=== RUN   TestCorpusCarriesTheNonFixedPointRows
--- PASS (0.00s)
=== RUN   TestLockstepListIsTheOneWeDrive
--- PASS (0.00s)
```

**N-2** — ran the whole `hashlock` package (9 tests, all PASS, including
`TestRefusalRowsMatchTheHost`, which iterates every corpus `Refusals` row —
plain, uppercase, lowercase, grouped-by-5, grouped-by-2, two-leading/trailing
spaces — through `ValidatePhrase`, i.e. through `IsMS1Shaped`). No new
acceptance, no regressed refusal. Host equivalence checked against source,
not assumed: `mnemonic-secret/crates/ms-cli/src/argv_guard.rs:149` calls
`is_ms1_shaped(&raw.trim().to_ascii_lowercase())` — Rust's
`to_ascii_lowercase()` folds only ASCII `A`-`Z`, leaving every other rune
untouched, which is exactly what the fork's new `strings.Map` fold does
(`if r >= 'A' && r <= 'Z' { return r + ('a'-'A') }; return r`), unlike the
superseded `strings.ToLower` it replaced (which folds non-ASCII case pairs
the host does not).

## 5. Whole gates at `26fd1dd`

Four packages named in the fold commit message:
```
ok  	seedhammer.com/hashlock	0.239s
ok  	seedhammer.com/codex32	0.002s
ok  	seedhammer.com/seal	11.832s
ok  	seedhammer.com/sysw	0.037s
```

gui, 24 shards via `scripts/gui-shard-test.sh ./gui/ 24`:
```
1224 top-level tests
partition verified exhaustive: 1224 == 1224
=== running 24 shards in parallel ===
[all 24 shards: ok, 51 tests each]
RESULT: ok -- all 1224 tests ran across 24 shards
```
Exactly the 1224 the brief expected.

`gofmt -l`, scoped to the packages this fold actually touched (hashlock,
codex32, seal, sysw, gui):
```
gui/transaction.go
gui/transaction_golden_test.go
gui/transaction_txrecord_test.go
```
Matches the fold commit's claim exactly. **Observation (Minor, not
blocking):** a whole-tree `gofmt -l .` also names `mt/mt.go` and
`mt/mt_test.go` — pre-existing at `17b3979` *and* at fork main `c4a64fc`
(verified both), so this is not new and not in a touched package; the fold
commit's phrase "gofmt clean on the touched packages" is technically correct,
but the brief's "only the three pre-existing transaction*.go" undercounts
the whole-tree pre-existing set by two files. Worth a one-line correction if
this brief's wording is reused.

`go vet` on the touched packages, same three pre-existing warnings present
at both `17b3979` and `c4a64fc` (Go-version directive on `testing.ArtifactDir`
in three unrelated `_test.go` files), none introduced by this fold:
```
gui/op/draw_test.go:176:24: testing.ArtifactDir requires go1.26 or later (file is go1.25)
gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)
gui/transaction_golden_test.go:104:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)
```

## 6. Records commit `67f9fe9`

```
commit 67f9fe9bf61e13afb873c01496a7326a26c13f80
records: F-481 CLOSED by fork 26fd1dd; F-482 (two unrecorded §4 copy
departures, H3) and F-483 (phrase in an unwipeable string; secret-handling)
filed; post-impl fold addendum in the implementation report

 design/FOLLOWUPS.md                                            | 26 +++++++++++++++++++++-
 .../hashlock-H2-implementation-report.md                       | 16 +++++++++++++
 2 files changed, 41 insertions(+), 1 deletion(-)
```

- F-481 header: struck through and marked **CLOSED 2026-09-05 by fork
  `26fd1dd`** — confirmed correct SHA, and the fix it names
  (`TestHashlockPhraseScreenDrawsTheMaskedReadout`, the 8px `CutBottom`
  removal) is exactly what was verified in §3 above.
- F-482 present, both citations checked against `17b3979` and true:
  - `gui/composer_hash.go:169-171` — `composerHashRows` sets
    `r := composerHashRowSet{..., lead: "Which hash?"}` then, at exactly
    lines 169-171, `if len(digests) == 0 { r.lead =
    composerCopyHashlockNoPayloadLead() }` — an overwrite, confirming the
    "REPLACES... instead of adding a second lead line" claim verbatim.
  - `gui/composer_copy.go:369` — `composerCopyHashlockPhraseLead` returns
    `"This screen does that hashing for you. Use a phrase you have never "
    + "used anywhere else."` — confirms the "prefixed with..." claim
    verbatim.
- F-483 present, content matches the M-3 finding it's filed from (phrase in
  an unwipeable Go string), correctly scoped to secret-handling / never
  gating per the standing operator ruling.
- Implementation report's `## Post-implementation fold` addendum: every
  number in it (`hashb867db87..b867db87`, `chars:29`/`chars: 28`, "10
  asterisks... 0 with the cut restored", "four packages ok", "1224 tests /
  24 shards ok", "gofmt and vet unchanged from `c4a64fc`") reproduced exactly
  as executed above.

## Per-item verdicts

| item | claim | verified | verdict |
| --- | --- | --- | --- |
| I-1 | two new confirm-frame assertions, both mutation-killed | yes (§2) | closed |
| I-2 / F-481 | readout draws; text AND geometry (raster) confirmed | yes (§3) | closed |
| M-1 → F-482 | filed, citations true at `17b3979` | yes (§6) | closed |
| M-2 | nil-`Key()` fails closed; no live value changed | yes (§4) | closed |
| M-3 → F-483 | filed, correctly scoped secret-handling | yes (§6) | closed |
| N-1 | every refusal sentinel has copy; mutation-killed | yes (§4) | closed |
| N-2 | ASCII-only fold matches host; corpus refusals still refused | yes (§4) | closed |
| gates | 4 pkgs / 1224 gui tests / gofmt / vet | yes (§5) | closed |

**Closing counts: 0 Critical / 0 Important / 0 new Minor / 1 recorded Minor
(gofmt-scope wording, informational) / 1 recorded observation (zero-margin
F-481 headroom, informational).**

## GREEN
