# Plan B Phase B1 — whole-diff review, round 0 (opus)

**Artifact:** `git diff main...HEAD` on `feat/encrypted-payload-phaseB1` @ `8778bb1`
(12 files, +1831/−43, 6 commits), in the fork worktree `/scratch/code/shibboleth/sh-wt-sealui`.

**Brief:** one question — does the diff do what it claims, safely, and what did the
IMPLEMENTATION introduce that plan review structurally could not catch. Plan
correctness was already gated (3 R0 rounds, closed 0C/0I), so the plan was declared
out of scope. The brief listed as already machine-verified: the full suite (47
packages ok, exactly two sanctioned setup failures), gofmt state including the five
pre-existing dirty files on main, that no Phase A test file was edited, and the
complete hardware pass. It named the highest-stakes question explicitly — can
anything secret reach the screen or the steel.

**Verdict: 0C / 2I / 3M / 6N.**

**Both Importants independently re-verified by the controller before folding**, by
applying each named mutant to a backed-up copy and running `go test ./gui/ ./seal/`:
with `unlockShape` hardcoded to `"UNSEALED"` AND `sel = start` deleted, both
packages report `ok`. The review's measurements are accurate.

---

## Report, verbatim

### [I] The "SEALED" assertion on the hash screen can never fail — `"SEALED"` is a substring of `"UNSEALED"`
**Where:** `gui/unlock_flow_test.go:70-71`, against `gui/unlock_flow.go:94-99`
**What the code does:**
```go
// unlock_flow_test.go, subtests {"D","SEALED"} and {"E","UNSEALED"}
if !uiContains(content, tc.shape) {
    t.Errorf("the hash screen does not show the %s shape; got %q", tc.shape, content)
}
```
`uiContains` is a plain `strings.Contains` on the lowercased, space-stripped frame text, so the `D`/`SEALED` subtest is satisfied by a screen reading `UNSEALED`.
**Why it is wrong:** §10.2 step 3 requires the shape displayed, and §11.3 makes an unkilled mutant a gap rather than a pass. This is the only assertion in the diff covering the sealed/unsealed word.
**Failure scenario:** Measured, not argued. Replacing the whole of `unlockShape` with `return "UNSEALED"` leaves `go test ./gui/ ./seal/` **fully green** (both packages `ok`). The opposite mutant (`return "SEALED"`) *is* caught, so the gap is one-directional and invisible to a reader. Shipped, that mutant labels a mixed payload `UNSEALED` while displaying the sealed digest, sending the operator to compare against the wrong recorded value — §6.6's own "teaches the operator that mismatches are normal".
**Fix:** anchor the needle so it cannot match the other word, e.g. assert `", " + tc.shape + "):"` (the format string already emits it) or assert the negative for the other shape in each subtest.

### [I] `sel = start` — "the selection follows the page" — is unfalsifiable, and the commit message claims it was mutation-checked
**Where:** `gui/unlock_platelist.go:141-144`; claim in commit `5e2ac7b`
**What the code does:**
```go
			// The selection follows the page. Leaving it behind would let OK
			// engrave a record the operator cannot see.
			sel = start
```
Commit `5e2ac7b`: *"The selection follows the page. Leaving it behind would let OK engrave a record the operator cannot see -- mutation-checked."*
**Why it is wrong:** the mutant the message names is not killed. Deleting exactly that line — "leaving it behind" — leaves `go test ./gui/ ./seal/` green, and every one of the seven `TestPlateList*` tests passes individually. `TestPlateListReturnsToTheSamePageAfterEngrave` pages to 2 and presses OK, but only asserts that *some* variant screen appeared; it never checks *which* record. So the record asserts a verification that measurement contradicts, and the hazard the code comment names has no test.
**Failure scenario:** with the line removed, an operator on page 3 of a 15-record 2-of-3 presses OK and cuts record 0 — ~21 minutes on the wrong plate, with the correct plate still showing as uncut. In B2 the same shape selects the wrong record from a set that includes the secrets.
**Fix:** extend `TestPlateListReturnsToTheSamePageAfterEngrave` (or add one case) to page once, press OK with `unlockEngraveHook` installed, and assert `gotRecord == recs[shownOnPage2First].Record` — the hook already exists and `TestPlateListOKEngravesTheSelectedRecord` shows the pattern. Correct the commit-message claim in the follow-up commit rather than leaving it standing.

### [M] The page button's wrap has no test
**Where:** `gui/unlock_platelist.go:135-140`
**What the code does:**
```go
			if start+shown < len(labels) {
				start += shown
			} else {
				start = 0
			}
```
**Why it is wrong:** §10.3's nav table makes `Button2` "advance the paged list, **wrapping**". Deleting the `else` branch leaves the whole suite green — `collectPages` walks forward and `TestPlateListPagesThroughEveryRecord` breaks as soon as every label has been seen, so neither ever needs the last page to return to the first.
**Failure scenario:** without the wrap, an operator who pages past the plate they wanted has no way back except Back — which *is* Lock (§10.3). In B1 that is an annoyance; in B2 it discards the session and costs twelve words plus a 31 s KDF.
**Fix:** in `TestPlateListPagesThroughEveryRecord`, keep paging one step past the last page and assert the frame equals the first page's frame.

### [M] `layoutMainPager`'s new `lastNav` argument is never proved to be wired to `StartScreen.lastNav()`
**Where:** `gui/gui.go:1801`; `gui/unlock_program_test.go:103-113`
**What the code does:** `pagerDots` calls `layoutMainPager(&ctx.B, &descriptorTheme, backupWallet, lastNav)` **directly** with a constant, so it measures the function, not the screen.
**Why it is wrong:** the carousel-lap tests pin `lastNav()` into the *wrap* sites, but nothing pins it into the *draw* site. Changing line 1801 back to `layoutMainPager(&ctx.B, th, m.prog, bip85Derive)` leaves the suite green — measured.
**Failure scenario:** payload present, nine navigable programs, eight dots drawn; on the ninth page `i == int(page)` never matches so no dot is filled at all. Cosmetic, but it is exactly the const-to-runtime regression the conversion was supposed to be guarded against, and hardware would not have shown it (the 9-dot observation was made against the correct code).
**Fix:** in `TestUnlockPayloadVisibleWithAPayload`, count the dots on a frame the `StartScreen` actually drew, or have `pagerDots` take the `*StartScreen` and call `m.lastNav()`.

### [M] 64 KB of heap is retained for the GUI's whole lifetime; at most 16450 bytes of it can ever be meaningful
**Where:** `gui/gui.go:1541-1546`, with `seal/read_tinygo.go:41-52`
**What the code does:**
```go
	var payload []byte
	if r := ctx.Platform.PayloadReader(); r != nil {
		if b, err := r.Read(); err == nil {
			payload = b
		}
```
`XIPReader.Read` does `out := make([]byte, len(region))` over `clampRegion(RegionLen)` = **65 536 bytes**, and `uiFlow`'s `payload` holds it until the GUI exits.
**Why it is wrong:** §6.2's own caps make the largest legal blob `52 + 8191 + 8191 + 16 = 16450` bytes, so ~49 KB of the retained buffer is provably erased flash. Measured on this branch: `tinygo build -target pico-plus2 -size short ./cmd/controller` reports `ram 69300`, i.e. ~451 KB free of the RP2350B's 520 KB — so this is ~14 % of the free heap held permanently whenever a payload is present. §6.4 already treats a **transient** ~98 KB as a design hazard ("a fifth of the free heap"), and this allocation is neither transient nor measured anywhere in the plan.
**Failure scenario:** payload present + engrave started is the one configuration the hardware pass did not drive to completion (the recorded checks stop at the §10.2.3 warning). `validateMdmk` builds three full plate plans at once; if that pushes the TinyGo heap over, the failure is an out-of-memory during an engrave rather than at boot.
**Fix:** one line after `Inspect` succeeds — or in `uiFlow` after `ParseHeader` — trim the retained slice to `HeaderLen + PubLen + CtLen (+ TagLen)`. Alternatively hold the `seal.Reader` and re-read on selection.

### [N] `"Sealed Payload"` is a literal in `gui.go` while `unlockTitle` claims to be the single source
`gui/gui.go:1792` writes `titleTxt = "Sealed Payload"`, but `gui/unlock_flow.go:21` declares `const unlockTitle = "Sealed Payload"` as *"the operator-facing name of the whole feature, and the same string the menu entry carries"*. Two literals, one stated invariant, no compiler link. Use the const.

### [N] Back is drawn with `assets.IconBack`, which §10.3 says it should not read as
§10.3: *"The label shown to the operator should read as leaving the session, not as stepping back one screen."* `gui/unlock_platelist.go:149` uses the back arrow. There is no lock/exit glyph in `gui/assets` (checked: Back, Checkmark, Discard, Edit, Gear, Hammer, Info, Left, Progress, Right), so this cannot be fixed without an asset. Harmless in B1 — nothing is resident — but it is the affordance B2 relies on to make "every exit wipes" legible. Worth an owning-phase entry rather than silence.

### [N] `unlockWarnUnauthenticated` renders the digest without consulting `p.HasHash`
`gui/unlock_flow.go:116` formats `seal.FormatHash(p.Hash)` unconditionally, while `unlockPayloadFlow:52` guards the notice screen with `if p.HasHash`. It is unreachable today only because `ParseHeader` rejects `pub_len == 0 && ct_len == 0` with `ErrEmpty` — so the warning is reached only when `pub_len > 0`. If that bound is ever relaxed, this screen prints the empty-set constant under a "compare this" instruction. A `HasHash` guard costs nothing.

### [N] `groupCards` now has no production caller
`seal/record.go:341` survives only as a wrapper for `seal/record_test.go:263,323`. Not wrong, but it now reads as production code that only tests exercise; either fold the two tests onto `groupRecords` or say in the doc comment that it is test-facing.

### [N] `PlateIndex` is positional, not the record's own `ChunkHeader.ChunkIndex`
`labelCards` (`seal/record.go:257-265`) counts occurrences in record order. `md.ChunkHeader` and `mk.Header` both carry `ChunkIndex`/`TotalChunks`, and a chunk-permuted public section is *admitted* — measured: reversing vector D's five records yields `err=nil, 5 records`. §6.6's "record order is plate order" makes the positional reading defensible, so this is a documentation gap rather than a defect; one line saying which of the two the label means would keep B2 from re-deriving it.

### [N] §10.2.2's "records already cut this session are marked" is neither implemented nor listed as deferred
The plan's *What B1 does NOT cover* (`IMPLEMENTATION_PLAN_…phaseB1.md:706-720`) defers §10.2.2's lifecycle, wiping and §10.2.4, but not this bullet — which is a property of the plate list, and the plate list is B1. Probably B2 by intent; record the owning phase so it is a grep rather than a recollection.

---

**Cleared by measurement, so a re-review need not re-derive these:**

- **No secret path to the plate list.** `unlockPlateListFlow` has exactly one call site (`gui/unlock_flow.go:73`), guarded by both the `p.Header.Sealed()` terminal return and the `ConfirmYes` gate. Disabling the sealed guard is caught by three tests; forcing the warning to confirm is caught by three more. `pub_len == 0 && ct_len == 0` is rejected by `ParseHeader` (`ErrEmpty`), so `p.Public` is never empty at the list, and `pub_len == 0 && ct_len > 0` terminates before it.
- **Task 4a changed no Phase A behaviour.** A 19-case malformed-public-section corpus run against **main's** `seal/record.go` and the branch's produced **byte-identical** sentinel + record-count output (`ErrNotLowercase` / `ErrRecordNotPermitted` / `ErrUndecodableCardSet`, `nrecs=0` on every failure). Instrumenting `cardKey`: exactly one call per record for D/E/G, **zero** for vector F's encrypted section.
- **The test-only seam is inert.** `unlockEngraveHook` is unexported, nil-initialised, assigned only in `gui/unlock_platelist_test.go:336` with a `t.Cleanup` reset, and has no exported setter — same shape as the sanctioned `bip85SeedHook`.
- **No panic path in the carousel conversion.** `m.prog` is bounded by `m.lastNav()` in both wrap directions; `layoutMainPlates`' case list includes `unlockPayload`; all three nav lists in the diff use exactly 3 (plate list) or 2 (`ConfirmWarningScreen`) slots against `ys [3]int`; `validateMdmk` returns label and plate slices of equal length by construction, so `plates[choice]` cannot over-index.
- **No clipped copy.** Rendered at 480×320, the §10.2.3 warning body (334 chars) and the hash notice (101 chars) are drawn in full on the first frame — `ExtractText` drops glyphs whose clip is empty, so this is a real visibility check, not a text-buffer check. This matters because `Warning` scrolls only on `Up`/`Down` button events and the SH2 is touch-only.
- **No nav shadowing.** Widest label the §6.4 cap can produce (`mk1 24/24 | 24/24`) is 138 px; an entry hit rect only reaches the nav column past 362 px. All three nav slots hit-test to their own `Clickable` on a drawn plate-list frame.
- Commit `4ee86bb`'s and `5e2ac7b`'s other mutation claims spot-checked and accurate: nil dispatch → caught; sealed falls through → caught; declining proceeds → caught; `recs[0]` instead of `recs[sel]` → caught; engrave resets the page → caught.

*Incidental:* running mutations from a `cp -a` of the worktree staged main's `seal/record.go` into the shared worktree index. Reverted with `git restore --staged`; `/scratch/code/shibboleth/sh-wt-sealui` is clean at `8778bb1` with no working-tree change.

VERDICT: 0C / 2I / 3M / 6N
