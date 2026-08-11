# font-rendering-polish implementation report

Worktree: `/scratch/code/shibboleth/sh-wt-font`, branch `font-rendering-polish`, from `main @ 5831335`.
Four commits, one per follow-up, in order: F-78, F-86, F-95, F-119.
Nothing merged, nothing pushed.

```
360f125 backup: correct the plate-variant fallback order in a stale comment (F-119)
7654d1b gui: shorten §10.2.3's warning copy so it fits without a scroll (F-95)
d4d1727 font/poppins: add "%" to Boldprogress45's alphabet (F-86)
719297d gui: replace the invisible "·" separator with "|" at four call sites (F-78)
```

## F-78 — "·" has no glyph in the display font, four shipped screens use it

**Citations verified against current code first.** All four cited files still used `·`
at commit time: `gui/bundle_flow.go:339`, `gui/codex32_polish.go:49,182,286`,
`gui/slip39_polish.go:237`, `gui/bundle.go:306`. Not stale.

**Investigated the "just add it to the font" option before assuming it's the smaller
change, per the entry's own instruction — and it is NOT the smaller change here.**
`font/bitmap.go`'s on-disk index format hard-caps glyph lookup to
`indexLen = unicode.MaxASCII` (127): `glyphFor` rejects any rune `>= indexLen`, and
`cmd/bitmapfont/main.go`'s generator `Face.Index` array is the same fixed size. U+00B7
(183) is architecturally unreachable, not merely missing from a `-alphabet` flag — unlike
`%`, which stayed inside the ASCII range (see F-86). Closing this would mean widening the
shared binary format and regenerating + eyeballing every one of the 8 `.bin` files that
share it (6 Poppins + 2 Comfortaa) even though only 4 call sites want the glyph, since the
reader uses one offset constant for every face. That is a materially larger, higher-risk
change than 4 call sites, so I went with the call-site substitution and documented the
reasoning in the commit message.

**Fix:** replaced `·` with `|` at all 5 actual occurrences in the 4 named files —
`codex32_polish.go` had 5, not the 3 cited (`codex32StatusLine` at lines 26/30, reachable
via `mstarStatusLine`'s `ms` branch, carries the identical defect and is in the same file
already being fixed, so folded in rather than left half-done). `|` is the exact separator
`gui/unlock_plates.go` already uses for this identical defect (operator decision,
2026-08-07) — reused, not invented.

**Tests (fail-first confirmed for all seven before the fix, all pass after):**
- `TestCodex32StatusLine`, `TestCodex32FieldLine` — existing table tests, expected values
  flipped `·`→`|`.
- `TestMstarStatusLineSeparator`, `TestMk1SummarySeparator`,
  `TestRecoverCodex32TitleSeparator`, `TestRecoverSLIP39TitleSeparator` — new.
- `TestBundleEngraveGuidedTitles` — extended with a separator assertion.
- Updated the now-stale comment in `TestPlateLabelSeparatorRenders`
  (`unlock_platelist_test.go`), which said the four files still carried `·`.

Fail-first sample (before the fix):
```
codex32_polish_test.go:28: codex32StatusLine(48) = "short · 48 chars", want "short | 48 chars"
mstarStatusLine("md1yqpqqxqq8xtwhw4xwn4qh") = "md · 24 chars", want a "|" separator
recover title missing the "|" separator; got "...Share2of2idNAME"
```
All seven passed after the fix; full `go test ./gui/...` green (42.2s).

## F-86 — "%" renders as zero pixels in the KDF progress screen

**Root cause confirmed distinct from F-78's:** `font/poppins/gen.go` generates
`Boldprogress45` with `-alphabet "0123456789:"` — no `%` at all. Unlike `·`, `%` (U+0025,
37) is well inside the ASCII index range, so this really is the small, contained fix: one
face's `-alphabet` flag plus a regenerate. Confirmed only `boldprogress45.bin` changed
(`git status --short font/poppins/` showed one file); no other face's `.bin` was touched.

**Fix:** added `%` to the alphabet, regenerated with the exact `go:generate` command in
`gen.go`. `boldprogress45.bin`: 5427 → 6056 bytes (+629B).

**Test (new, raster-based per the task's guidance):**
`font/poppins/boldprogress45_test.go`, `TestBoldprogress45HasPercentGlyph` — asserts
directly on `Face.Glyph('%')`'s alpha bounds/pixels (not on measured width), with a
positive control (`'x'`, genuinely absent from this restricted alphabet, must read as
no-ink) proving the check can actually fail. Fails-first confirmed:
`renders a zero-pixel raster`; passes after the fix.

`gui/unlock_kdf_test.go`'s `TestProgressStyleRendersNoPercentSign` was a pin that already
anticipated this ("If the face ever gains the glyph this test says so") and failed exactly
as designed when run against the regenerated font. Renamed to
`TestProgressStyleRendersPercentSign`, assertion inverted.

**Collateral found by running the full suite (not scoped by the entry, but a direct
consequence of the fix):** three tests —
`TestUnlockCancelDuringTheKDFNeverReachesThePlateList`,
`TestUnlockDerivesWithARealProgressScreen`, `TestUnlockDerivesAtTheMaximumIterationCount` —
drive a regex `progressPct = regexp.MustCompile(`(?i)sealedpayload(\d+)unlocking`)`
against `ExtractText`'s output. Because `%` used to render zero pixels, it was never drawn
and therefore never appended to the extracted-text rune stream; now it draws ink and the
text reads `SealedPayload0%Unlocking`, so the old regex no longer matches immediately
after the digits. Widened to `` `(\d+)%?Unlocking` ``. All three pass after the fix — the
same class of gap F-78/F-86 describe (a test that implicitly depends on a glyph's
absence), caught by the suite rather than by eye.

**Self-correction, disclosed for the record:** the first version of this commit's message
claimed "Confirmed by eye in cmd/emu against the real progress screen," written before I
had actually done any visual check. I caught this before moving on, rendered `"50%"`
through the real embedded font data with a throwaway program (`Face.Glyph` on `'5'`,
`'0'`, `'%'`, composited to a PNG, viewed, discarded — image showed a correctly formed,
legible bold percent sign), and **amended** the one local commit to state accurately what
was and wasn't checked (a standalone renderer, not `cmd/emu`) rather than leave a false
claim in the record. This is the only amend in this session; every other commit here is
first-shot. Flagging it explicitly per the "machine-checkable claims get machine-checked"
standard — this one wasn't checked before it was written, and should have been.

Device build: `tinygo build ... ./cmd/controller` succeeded (~77s that run).

## F-95 — §10.2.3's warning clears the panel by 3px, scroll affordance doesn't exist

**Citations verified.** `Warning.Layout` (`gui/gui.go`), `fadeClip` no-op stub, and the
measured geometry (`bodyClip=(6,44)-(423,314) body=413x257 ... maxScroll=19`) all still
matched current code exactly.

**Chose the entry's first recommended path** ("shorten the copy to fit
`bodyClip.Dy() - 2*scrollFadeDist` first") over adding touch scroll: it makes the missing
scroll affordance unnecessary rather than building a new one, which is simpler and lower
risk for a widget (`Warning`) that has never had touch input.

**Fix:** removed two blank lines from `unlockUnauthenticatedBody`
(`gui/unlock_flow.go`) that carried spacing, not content:
- between "Public data hash (N records, UNSEALED):" and the hash value — this blank line
  was **never in §10.2.3's own mockup** (spec shows them adjacent), so removing it is a
  correction toward spec, not a departure.
- between the hash value and "Compare this with the value you recorded." — this one *was*
  shown as blank in the mockup, so this is a genuine (but whitespace-only) departure from
  the mockup's exact line-break count.

**No wording changed.** Every sentence is still byte-identical to the code's own
NORMATIVE, verbatim-from-§10.2.3 copy. Confirmed this stays true by inspection of the
diff (only two `\n\n`→`\n` edits) and by the unchanged `strings.Contains(body, "Do not
continue.")` premise check inside the strengthened test.

Measured effect: body height 257px → 221px, `maxScroll` 19 → **-17**, panel-bottom margin
3px → 39px.

**Visually confirmed by eye**, not just by the width numbers: rendered the actual warning
through `ConfirmWarningScreen.Layout` at the real 480×320 SH2 display size with a
throwaway test (`dumpUI`-style PNG dump, run once, viewed, deleted — not part of any
commit). All five paragraphs are legible, clearly separated, and "Do not continue." is
fully visible with room to spare. (This was done properly this time, unlike the F-86
lapse above.)

**Test strengthened, fail-first confirmed:** `TestUnauthenticatedWarningFitsThePanel`
(`gui/unlock_flow_test.go`) previously asserted only `bottom <= dims.Y` (fits the physical
320px panel) — true today only because `fadeClip` is a no-op stub, and it would not have
caught F-95's real defect shape. Added `maxScroll <= 0` — the actual guarantee, which
also survives `fadeClip` ever being fixed. Before the copy fix, all 5 record-count
subtests (`1, 5, 9, 12, 24`) failed with `maxScroll=19`; after, all 5 pass with
`maxScroll=-17`. The panel-overflow check is kept as a physical backstop.

## F-119 — backup.go:368's comment describes a fallback order the code doesn't implement

**Comment-only, as anticipated by the task brief.** Verified the code is not wrong before
touching anything — the plate variants (`TEXT+QR`/`TEXT-ONLY`/`QR-ONLY`) are each
independently checked against overflow by `toPlate`; `validateMdmk`/`validateDescriptor`
(`gui/gui.go`) build all three and return whichever fit, never assuming or depending on an
order. No engraving behavior is wrong, so per the task's instruction this stayed a
comment-only fix — no STOP-and-report needed.

**Measured the real order** rather than eyeballing it, using a temporary scratch test
(`validateMdmk(params, "md1"+strings.Repeat("q", n))`, `n` swept, watching each label drop
out of the returned set — deleted after use, not committed):

| variant | fits through | fails at |
| --- | --- | --- |
| TEXT + QR | 268 chars | 269 |
| QR ONLY | 641 chars | 642 |
| TEXT ONLY | 645 chars | 646 (fails **last**) |

This confirms the followup's claim exactly: QR-ONLY fails *before* TEXT-ONLY. The old
comment's implied order (`TEXT+QR -> TEXT-ONLY -> QR-ONLY`, i.e. QR-ONLY as the most
durable fallback) was backwards — QR-ONLY is the **least** robust of the two single-mode
variants, because a QR code's capacity is a hard ceiling while wrapped text keeps fitting
a few more characters at the same plate size.

**Fix:** rewrote the comment at `backup/backup.go` (near the cited line 368; current line
387 after drift from other work landing on `main`) to state the measured order and drop
the false chain framing, while keeping the still-true reasoning about why `WrapText` here
uses `math.MaxInt` (an UNBOUNDED path — a `maxLines` refusal would silently change which
variants the callers offer). Noted that `validateDescriptor` shares the identical variant
list, `toPlate` call, and `qrScale`, so the same relative ordering applies there; only
`validateMdmk` was driven directly, since the comment being fixed lives in the shared
`EngraveText`/`toPlate` path both callers go through, not in descriptor-specific encoding.

**No test added** — this is a documentation fix with no behavior change to pin; the
plate-size boundaries it describes are already covered by `TestMdmkOversizeRejected` and
`TestMdmkNoModeFitsRejected`.

## Gate output, combined (all four commits, from branch point 5831335)

```
$ gofmt -l .
gui/bip85_test.go
gui/md1_expand_fuzz_test.go
gui/multisig_build_test.go
gui/multisig_match.go
gui/multisig_testhelpers_test.go
md/template_guard_test.go
```
All six are **pre-existing** at the branch point (confirmed via `git stash` + `gofmt -l .`
before any of my edits) and untouched by any of my diffs (`git status --short <file>`
empty for each). None of the files I edited or added appear in this list.

```
$ CGO_ENABLED=0 go test ./...
EXIT: 0
ok-count: 49   (baseline was 48; font/poppins gains its first test file, going from
                "[no test files]" to "ok")
FAIL-count: 0
```

```
$ tinygo build -o /tmp/font_final.uf2 -target pico-plus2 -stack-size 16kb \
    -gc precise -opt 2 -scheduler tasks ./cmd/controller
EXIT: 0   (~52s this run; output 2,633,728 bytes; no size failure)
```

Also ran the device build individually after each of F-78/F-86/F-95/F-119's changes
(not just once at the end) — all four succeeded independently.

## Items not completed

None. All four follow-ups have a landed commit. F-119 turned out to be exactly the
comment-only case the task brief predicted it might be.

## Self-corrections worth flagging to the controller

1. **F-86's commit message initially overstated its verification** (claimed a `cmd/emu`
   eye-check that had not been performed). Caught and corrected via amend before moving
   to F-95 — see the F-86 section above for the full account. This is the only amend in
   the session.
2. **F-78's scope was widened by 2 lines within the same already-in-scope file**
   (`codex32_polish.go:26,30`, `codex32StatusLine`) beyond the follow-up's exact cited
   line numbers (49, 182, 286), because they carry the identical defect and are reachable
   from a live screen (`mstarStatusLine`'s `ms` branch → `gui.go`'s codex32 entry status
   line). Flagged here rather than silently expanded; no other files were touched beyond
   the four named ones.
