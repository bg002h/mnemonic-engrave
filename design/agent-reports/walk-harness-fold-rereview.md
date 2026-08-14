# Walk-harness fold re-review — `740888d..015150a` (3 commits)

**Reviewer:** independent sonnet re-review, repo `/scratch/code/shibboleth/seedhammer`,
branch `main`, HEAD `015150a`. All builds/tests/mutations run in the dedicated
worktree `/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/3985bd41-08d3-42b8-a967-1493b588d215/scratchpad/wt-rereview`
(checked out at `015150a`). Worktree confirmed clean (`git status --short`,
`git diff --stat`, both empty) before, throughout (after every mutation), and
at the end.

**The one question, per commit:** did the fold actually fix what it claims to
fix, and did the fold introduce a new defect? Scope is the folds only — no
fresh audit of the underlying feature.

**Toolchain:** `export PATH="/nix/var/nix/profiles/default/bin:$PATH"`, then
`nix develop --command go test ...`. All mutation testing below is live
execution, not just reading.

---

## Verdict

**0 Critical.** **1 Important** (commit A — a doc-narrowing fold that fixed
five of five cited overclaim sites but left two sibling comments in the same
file inconsistent with its own new rule). **1 Minor** (commit C — a narrow,
practically-inert truncation-then-parse path opened by the new mutable shared
reader state that the old one-shot design could not exhibit). Commit B's fold
is clean: all four re-run older mutations behave correctly, no new escape
found.

---

## A. `0307aff` — the census doc-narrowing fold

### Is every place that made the overclaim now accurate?

Searched the whole tree for the literal phrase and its variants:

```
grep -rn "md1/mk1/ms1\|md1, mk1, ms1\|md1|mk1|ms1" --include="*.go" .
```

Four remaining hits, all self-referential ("...because saying 'md1/mk1/ms1'
was an overclaim...") or a design-rationale aside (`gui/gui.go:589`, arguing
*why* `Plate` doesn't carry the string — not a coverage claim). No live
overclaim remains under that exact phrasing.

Cross-checked against the false-pass review's own count: it named four doc
sites explicitly (`cmd/emu/toolpath_js.go:28`, `cmd/emu/engraved.go:6`,
`gui/engraved_hook.go:3`, `gui/engraved_hook.go:47`) plus said "five doc
comments" in the fold's own commit message. The fifth is
`cmd/emu/platform.go`'s `engraved` field comment ("the census of md1/mk1/ms1
strings..."), also diffed in the fold. All five are corrected in the diff and
confirmed by re-reading the current files — this part of the fold is complete
and accurate.

**But two sibling comments in the same file the fold touched were not
updated, and they now contradict the fold's own new text.** See Important
finding below.

### Is the new claim TRUE — does an ms1 bundle card actually pass through `validateMdmk`?

**Yes, verified by execution, not just reading.**

- `gui/singlesig_engrave.go:26`: `strings: []string{b.MS1}` — the ms1 secret
  goes into a `bundleCard{kind: cardMS1}` verbatim.
- `gui/bundle_flow.go:271-286` (`bundlePlatePlan`): every string of every
  card, unconditionally, becomes a `bundlePlate{str: s}` entry in `plan`.
- `gui/bundle_flow.go:295-304` (`bundleEngrave`): `for _, p := range plan {
  labels, plates, err := validateMdmk(ctx.Platform, p.str) ... }` — no
  format branch on `p.str`. `validateMdmk` (`gui/gui.go:2258`) itself is
  content-agnostic: it QR-encodes `s`, builds three paragraph variants, and
  assigns `plateTextSeq`-derived ids to whatever fits a plate, regardless of
  whether `s` is md1/mk1/ms1.

Added a temporary probe (`gui/engraved_hook_test.go`, reverted immediately
after):

```go
func TestScratchMS1ThroughValidateMdmkIsAnnounced(t *testing.T) {
	p := newEngravedAwarePlatform()
	labels, plates, err := validateMdmk(p, ms1Fixture)
	...
}
```

```
$ nix develop --command go test ./gui/ -run TestScratchMS1ThroughValidateMdmkIsAnnounced -v
--- PASS: TestScratchMS1ThroughValidateMdmkIsAnnounced (0.00s)
```

Confirms: the ms1 fixture gets one id per variant and is correctly recorded
in `p.candidates`, exactly like the md1 case `TestValidateMdmkAnnouncesOneIdPerVariant`
already pins. The fold's new claim — "the ms1 that `bundleEngrave` cuts as a
bundle card" is covered — is true.

### Does any doc still imply a gate can ignore `unattributed`?

**Yes — one Important finding.**

`cmd/emu/engraved.go`'s new package comment (lines 9-26, added by this fold)
states plainly: *"A GATE MUST THEREFORE TREAT unattributed > 0 AS 'something
was cut that this census cannot name', not as noise."* That is correct and is
exactly the fix the Critical called for.

But two other comments **in the same file**, on the two fields that actually
carry this data, were not touched by the fold and still say the opposite in
substance:

- `cmd/emu/engraved.go:60-64` (the `unknown` field on `engravedRecorder`):
  *"unknown counts engraved ids nobody announced -- every seed, passphrase
  and free-text plate, which carry id 0."* Still enumerates only the three
  intentionally-excluded categories; does not mention the ms1-standalone
  codex32 case this very fold's Critical established.
- `cmd/emu/engraved.go:127-130` (the `Unattributed` field of the JSON struct
  `StringsJSON()` marshals): *"Unattributed counts finished plates that never
  came from validateMdmk -- seeds, passphrases, free text. Expected to be
  non-zero on any walk that cuts one, and NOT an error."*

The second one is the one that matters most: it is the doc comment sitting
directly on the actual `json:"unattributed"` field — the thing a gate author
integrating against `shToolpath.strings()`'s JSON output is most likely to
read, arguably more likely than the 25-line package-comment prose 100 lines
above it in the same file. Read in isolation (which `go doc` will happily
show, and which is exactly where someone writing a gate check would look),
it says unattributed is "NOT an error" with no qualification — precisely the
"a gate can ignore this" reading the fold's own new package comment exists to
foreclose.

**This is not a code defect** — the underlying behavior is unchanged and
correct (ms1-standalone plates really do carry id 0 and land in
`unattributed`, exactly as designed). It is a documentation completeness gap:
the fold's commit message claims the overclaim was fixed "in every place that
made it," and that is not quite true — two sibling comments in the file it
edited, describing the exact same field this fold is about, were left
internally inconsistent with the fold's own corrective text three lines
above them (`unknown`) and a hundred lines above them (`Unattributed`).
Given the whole point of this fold's Critical was that a gate author could
be misled by stale documentation into treating a genuine ms1 mis-cut as
"noise," leaving two more instances of exactly that framing in the same file
is a real, if narrower, recurrence of the same class.

**Severity: Important.** Blocks per project policy. Straightforward one-line
fix: add the same "except an ms1 cut through the standalone codex32 flows"
caveat to both comments, or point them at the package comment instead of
re-stating a now-incomplete rationale.

### Minor/Nit noticed in passing (not blocking, recorded for completeness)

- `cmd/emu/toolpath_js.go:34` — the doc-comment edit left one line unusually
  long (127 chars) where the surrounding lines wrap around 80: *"// The two
  counts exist so an EMPTY census can be told apart from a BROKEN one --
  announced=0 on a walk that reached an engrave"*. Cosmetic only; `gofmt`
  does not reflow comment prose, so this does not fail the build gate. **Nit.**

---

## B. `015150a` — the guard fold (`gui/tinygo_split_test.go`)

### Does the new stub check catch the original escape, and does the recursive walk work?

**Not re-derived — both explicitly marked ALREADY SETTLED** in the brief:
"declaring an exported interface inside a tinygo stub" and "relocating a hook
pair into `gui/widget` with an untagged host" were already run against the
current (folded) guard and killed. Re-deriving these was explicitly out of
scope for this pass.

### The four older mutations, re-run against the current (post-fold) guard

Baseline confirmed first:

```
$ nix develop --command go test ./gui/ -run TestBuildTaggedHooksAreAbsentFromTheFirmwareImage -v
--- PASS
```

All mutations applied to `gui/frame_hook.go` / `gui/frame_hook_tinygo.go` /
`gui/gui.go` (a pair untouched by either fold, so results aren't confounded
by commit A's edits), each reverted via `git checkout --` immediately after
observing the result, worktree confirmed clean after each.

1. **Drop `//go:build !tinygo` from a host file** (`gui/frame_hook.go`):
   ```
   --- FAIL
       tinygo_split_test.go:123: frame_hook.go carries "", want "//go:build !tinygo"
   ```
   Correctly caught.

2. **Delete a stub** (`rm gui/frame_hook_tinygo.go`):
   ```
   --- PASS
   ```
   **This unit test does NOT catch it** — pairs are discovered "from the stub
   side" (`for _, stub := range files { if !strings.HasSuffix(stub,
   "_tinygo.go") { continue } ... }`), so a deleted stub simply removes that
   pair from consideration; `FrameAware` never enters `owner` and is never
   checked. **This is not a regression.** Extracted the pre-fold version
   (`git show 740888d:gui/tinygo_split_test.go`) and confirmed the identical
   stub-side pairing logic was already there, verbatim, before this fold —
   and the pre-fold file's own header comment already said so explicitly:
   *"WHY IT IS A TEST AND NOT THE COMPILER. CI does build the device image
   ..., so the loud mutations -- delete a stub, drop a constraint, call the
   hook from shared code -- fail there."* That sentence is retained unchanged
   in the post-fold file (`gui/tinygo_split_test.go:29-34`). Confirmed the
   "fails there" claim mechanically:
   ```
   $ nix develop --command go build -tags tinygo ./gui/...
   gui/run_flow.go:264:5: undefined: notifyFrame
   ```
   So "delete a stub" is caught — by the separate `tinygo-device-build` CI
   job / a plain `-tags tinygo` build failure, exactly as documented, both
   before and after this fold. No property was lost.

3. **Name a hook interface in code from an untagged file** (added
   `var _ FrameAware` to `gui/gui.go`):
   ```
   --- FAIL
       tinygo_split_test.go:224: gui.go uses FrameAware in code but is not frame_hook.go
   ```
   Correctly caught.

4. **Move a hook interface declaration into an untagged file** (cut `type
   FrameAware interface {...}` out of `gui/frame_hook.go`, pasted into
   `gui/gui.go`):
   ```
   --- FAIL
       tinygo_split_test.go:202: frame_hook.go declares no exported interface, so
       nothing about it is checked below -- if this pair is not an interface
       hook, say so here rather than leaving the scan silently vacuous
   ```
   Correctly caught, via the pre-existing per-pair `found == 0` floor (this
   floor predates this fold too — confirmed identical in the pre-fold file).

**Summary: 3 of 4 fail the guard test directly; the 4th ("delete a stub") is,
by design and unchanged across the fold, caught by a different gate (the
actual `-tags tinygo` build) rather than by this unit test.** No regression.

### Is there a NEW escape the fold opened?

Checked specifically for hazards the switch from `os.ReadDir(".")` to
`filepath.WalkDir(".", ...)` could introduce:

- **Non-Go / testdata directories under `gui/`** (`gui/assets`,
  `gui/op/testdata`, `gui/testdata`, `gui/testdata/fuzz/...`): all filtered
  by `filepath.Ext(path) != ".go"`; no `.go` files exist under any testdata
  directory except one `_test.go` (excluded). No spurious parse attempts.
- **Build-tagged files in subpackages that could interact oddly with the
  recursive scan**: `grep -rl "^//go:build"` across `gui/**/*.go` (excluding
  `_test.go`) returns only the existing top-level pairs
  (`plate_hook*`, `frame_hook*`, `engraved_hook*`) plus `gui/preview.go`,
  `gui/debug.go`, `gui/nodebug.go` — none in a subpackage, so the recursive
  walk currently has no tagged files to trip over below the top level.
- **Path-comparison correctness for subpackages** (`host != name` with
  `filepath.ToSlash` paths): not independently re-derived here, since the
  brief marks the `gui/widget` relocation mutation (which exercises exactly
  this) as already run and killed against the current guard.
- **Cross-package identifier-name collisions in the flat `owner` map**:
  considered as a theoretical edge case (two different subpackages
  legitimately declaring an identically-named exported interface would
  make the last-scanned one win in `owner`, causing the scan to
  mis-attribute the other pair's own file as a violation). This is a
  pre-existing structural property of using a name-keyed map rather than a
  regression from the walk becoming recursive, it fails *loud* in every
  case reasoned through (a false `t.Errorf`, never a silent pass), and there
  are currently only 3 real pairs with distinct names, so it is not live.
  Not filed as a finding.

**No new escape found.**

---

## C. `5374255` — `cmd/emu/nfc.go` queue rewrite (unreviewed, small, adjacent)

### Does `Read` correctly delimit tags for `gui/scan.go`'s contract?

`gui/scan.go`'s `scanner.Scan(r)` calls `r.Read` once per `Scan` call,
accumulating into a persistent `s.buf`/`s.n` across calls, and treats
`io.EOF` as "the accumulated bytes are one complete record" (`err == nil` →
`errScanInProgress`, i.e. "more is coming").

- **Record larger than the caller's buffer**: `nfcSource.Read` copies
  `min(len(p), remaining)` bytes and returns `(n, nil)` if the record isn't
  fully drained, `(n, io.EOF)` once it is. Existing test
  `TestTagsCrossTheReaderInOrder` and the doc's own reasoning confirm this
  matches the "several Reads, reported as progress" contract. Correct.
- **Empty queue**: `n.cur == nil && len(n.queue) == 0` → `(0, io.EOF)`
  immediately — the documented "idle reader" signal `gui/nfc_scan.go`
  backs off on. Pinned by `TestAnIdleReaderIsNotAnAbsentReader`. Correct.
- **Close mid-record**: `Close()` drops `cur`/`off`, pinned by
  `TestCloseDropsAHalfReadTag` — but that test exercises a *fresh* reader
  handle after `Close`, i.e. the "next flow entry" scenario the doc
  describes. I additionally checked what happens if `Close()` (or `set("")`,
  which does the same reset) lands **between two `Scan()` calls of the SAME
  in-progress multi-part record**, on the SAME `gui.scanner` instance whose
  `s.n` already holds a partial accumulation from before the interruption —
  a scenario the new mutable, persistent, cross-goroutine `nfcSource` can
  produce that the old one-shot `bytes.Reader`-per-handout design could not
  (a handed-out `bytes.Reader` was an independent, immutable snapshot; a
  later `set()`/would-be-`Close()` on the source could never reach back into
  bytes already handed to a reader). Wrote a synchronous, deterministic
  probe (temporary, reverted) reproducing exactly the byte-level effect of
  that interleaving without needing to win an actual goroutine race:
  ```go
  // scratchTruncatingReader: first Read returns a few bytes + nil (progress);
  // second Read returns (0, io.EOF), exactly what nfcSource.Read reports once
  // cur is reset to nil by an external Close()/set("").
  ```
  ```
  $ nix develop --command go test ./gui/ -run TestScratchScanTreatsAnExternallyTruncatedStreamAsAWholeRecord -v
      scan_test.go:180: second Scan (after simulated Close-interrupt): obj=<nil> err=scan: unknown format
  --- PASS
  ```
  **Confirmed mechanism**: gui/scan.go treats the leftover accumulated
  prefix as "the whole record" and attempts to parse it. **Confirmed
  consequence is benign**: every format branch in `gui/scan.go` (sysw hex,
  bip39, descriptor, codex32, md/mk BCH, address) is checksummed, so a
  truncated prefix essentially cannot coincidentally validate as a
  different, wrong-but-plausible record — it reports `errScanUnknownFormat`,
  the same as scanning garbage. The scan result also lands on a
  goroutine/channel that is mid-teardown at exactly this point (Close is
  only ever invoked from `startScanner`'s `stop()`, called when the owning
  screen exits), so in the realistic case nothing downstream ever reads it.
  **Severity: Minor.** Not exploitable in practice (checksums save it, and
  the channel is orphaned), but it is a genuinely new behavior this commit
  introduces (the old design could not truncate a handed-out reader
  mid-stream at all) and is undocumented. Worth a one-line comment on
  `Close`/`set("")` noting that an interrupted multi-part record can
  surface as a transient "unknown format" scan result rather than silently
  vanishing — not worth more than that.
- **Concurrent `set` during `Read`**: both take `n.mu`, fully serialized;
  `set` only appends to `queue` or resets `cur`/`off`/`queue` (the `rec==""`
  clear case, covered above) — no data race, no interference with an
  in-flight `cur` delivery from a concurrent non-clearing `set`.

### Can a tag be delivered twice, or silently dropped?

- **Twice**: no. `n.queue[0]` is popped exactly once (`n.queue =
  n.queue[1:]`) and never re-appended; nothing re-queues a delivered `cur`.
  Pinned by `TestATagCrossesOnce`.
- **Silently dropped**: only the two *documented and tested* cases —
  `Close()`/`set("")` interrupting a record mid-flight (drops the
  remainder, `TestCloseDropsAHalfReadTag`), and `set("")` clearing the whole
  queue (`TestClearRemovesQueuedTags`). Both are intentional walk-reset
  operations, not accidental loss paths.

### Is `detached` mode coherent — can a walk reach a genuinely-no-reader machine?

Yes. `shNFC.detach()`/`.attach()` (`cmd/emu/nfc_js.go`) call
`nfcSource.detach(bool)`, which `reader()` checks before returning `n` vs
`nil`. Every consuming screen fetches `ctx.Platform.NFCReader()` exactly once
at entry (`gui/bundle_flow.go:110`, `gui/md1_gather.go:85`,
`gui/derive_xpub.go:231`, `gui/verify_address.go:77`,
`gui/mk1_inspect.go:160`, `gui/gui.go:1892`), matching the documented "takes
effect on the next flow entry" contract, and `startScanner(ctx, nil)`
(`gui/nfc_scan.go:45-49`) returns a channel that never delivers without a
goroutine — the existing Back-only behavior. `platform.go`'s `Features()`
(unchanged by this commit) reports a constant `FeatureNFC` capability bit
independent of `detached` state, which is correct and intentional per its
own comment (capability vs. current-reader-state are different questions,
and `derive_xpub.go:156` deliberately probes the former). Confirmed
`TestAnIdleReaderIsNotAnAbsentReader`'s detach/attach round-trip is present
and passing in the new test file. Coherent.

---

## Test suite sanity

Ran at a clean, unmutated `HEAD` (`015150a`) after all mutation probes were
reverted:

```
$ nix develop --command go test ./gui/... ./cmd/emu/...
ok  	seedhammer.com/gui	60.126s
ok  	seedhammer.com/gui/assets	0.004s
?   	seedhammer.com/gui/layout	[no test files]
ok  	seedhammer.com/gui/op	1.510s
ok  	seedhammer.com/gui/saver	4.314s
ok  	seedhammer.com/gui/text	3.106s
ok  	seedhammer.com/gui/widget	1.021s
ok  	seedhammer.com/cmd/emu	1.237s
```

`git status --short` and `git diff --stat` in the worktree: both empty,
confirmed at the end.

---

## Findings summary

| # | Commit | Severity | Location | What |
|---|---|---|---|---|
| 1 | `0307aff` | **Important** | `cmd/emu/engraved.go:60-64`, `:127-130` | Two doc comments on the exact fields that carry the census's "unattributed" signal were not updated by the fold and still read as flatly benign ("NOT an error"), contradicting the fold's own new package-comment directive three/~100 lines above in the same file. Not a code defect; a documentation completeness gap in the same class the fold's own Critical was about. |
| 2 | `0307aff` | Nit | `cmd/emu/toolpath_js.go:34` | One comment line left at 127 chars vs. ~80 for its neighbors after the edit. Cosmetic; `gofmt` doesn't reflow comments. |
| 3 | `015150a` | — | (no finding) | All 4 re-run older mutations behave correctly; "delete a stub" is, unchanged across the fold, caught by the separate `-tags tinygo` build rather than this unit test — confirmed both pre- and post-fold source carry identical stub-side pairing logic, and confirmed the actual build fails on it. No new escape found in the recursive walk. |
| 4 | `5374255` | **Minor** | `cmd/emu/nfc.go` `Read`/`Close`/`set` | A `Close()` or `set("")` landing between two `Scan()` calls of the same in-progress multi-part record can truncate it; the leftover bytes get parsed as a "complete" record attempt by `gui/scan.go`. Verified benign in practice (checksums on every format branch mean this reports `errScanUnknownFormat`, never a phantom valid or wrong tag) and the result lands on an already-tearing-down channel. Genuinely new behavior vs. the old one-shot design (which could not be truncated post-handout); undocumented. Worth a one-line comment, not a blocking fix. |

**1 Important finding blocks per project policy** (`0307aff`'s incomplete doc
fold). Recommend: extend the same one-sentence ms1 caveat already added to
the package comment to `cmd/emu/engraved.go`'s `unknown` field doc (line
60-64) and `Unattributed` JSON field doc (line 127-130); this is a
comment-only fold and per the project's own re-review-scope rule does not
need to re-trigger a full round — a mechanical confirmation that the two
comments now match the package comment's framing is sufficient to close it.

Commits `015150a` (guard fold) and `5374255` (nfc.go rewrite) are otherwise
clean: 0 Critical, 0 Important. `5374255`'s one Minor and `0307aff`'s one Nit
are recorded, not blocking.
