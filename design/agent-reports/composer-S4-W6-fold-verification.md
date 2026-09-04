# Independent verification — round 2, the FOLD (`composer-s4e` @ `818220d8`)

Reviewer: independent fold-verification agent (opus). Base `70008da`, reviewed tip
`05466727`, fold `818220d8991e084ab6c8a4a3a6c44ebc7ff310a7`, worktree
`/scratch/code/shibboleth/wt-composer-s4e` (left clean, `git status --short` empty at
`818220d8`; nothing committed). Every mutation ran in `cp -r` copies at
`/scratch/code/shibboleth/.s4e-verify2`, `.s4e-mut` and `.s4e-prefold`; the `70008da`
baseline is a `git archive` extraction at `.s4e-base70008da`. No `.jsonl` read, no
sub-agent spawned.

**VERDICT: DO NOT MERGE AS IS. 0 Critical / 2 Important / 2 Minor / 1 Nit.**

**Round 1's three findings are all FIXED, and the root cause is closed structurally** —
over 4,828 composable path lists there is not one pair that shares a shape signature and
numbers its slots differently, so every door that swaps `st.list` under
`composerApplyShapeEdit` is now safe by construction. C-1's whole class is gone.

What is not closed is the **precision of the one new function**. `composerEditCanRenumber`
is the guard's entire condition on two arms, nothing outside its own tests exercises it,
and it is **wrong in both directions** — measured exhaustively over 14,092 composable
`(list, idx)` pairs: **1,200 false negatives** (§8j skipped and every seat destroyed) and
**288 false positives** (§8j fires, and lies, where nothing can move). Both are regressions
against `05466727`: pre-fold, neither walk lost a seat or drew a screen. Both have **one**
root cause, and the measurement that shows it is in I-3 below.

---

## I-2 (IMPORTANT) — the hash arm skips §8j and then discards EVERY seat. Introduced by this fold.

`composerEditCanRenumber` probes with `Hash = nil` in **both** of its variants. On a
KEY-LESS path (`Keys == nil`, the wsh-only §4b shape) that makes both variants the same
refused empty-path list, so the probe compares two structural-only signatures, finds them
equal, and answers **false**. The hash arm therefore asks nothing — and the
`composerApplyShapeEdit` wrapper this fold newly added then sees the mapping vanish and
clears every seat.

Reproduced end to end through the production function `composerPathEdit`, real screens,
on `818220d8`:

```
BEFORE: sig=w1/2,0,|0.0/0.1/ slots=[{Index:0 Path:0 Ordinal:0} {Index:1 Path:0 Ordinal:1}] assigned=[0 1]
NO §8j drawn on the way into the hash editor.
  Last frame: "Path2hashWhichhash?Type64hexNohashlock"
AFTER : sig=w1/2,0, composeErr=md: compose: a path with neither keys nor a hash is not a
        spend path: path 2   assigned=[-1 -1]

FINDING: 2 of 2 seats were DISCARDED by a hash edit and §8j was never drawn.
```

Walk: wsh, `[{2-of-2}, {key-less + hash}]`, both slots seated → `Path 2` → `Hash lock` →
`No hash lock` (the picker's last row). No confirm, no chance to decline, all seating gone.

**It is a regression.** The identical walk against `05466727`, run in `.s4e-prefold` with
`gui/composer_shape.go` and `gui/composer_discard.go` restored from that commit:

```
PRE-FOLD (05466727) after the same walk: assigned=[0 1]
```

Pre-fold both seats survived (correctly — a key-less path contributes no slots, so nothing
moved); post-fold both are destroyed in silence. §7d/§8j promise the operator is told
before an edit that clears their seating is accepted. Here it is not told, and cannot
decline.

Not Critical: the discard is the *safe* direction, so no key is seated onto a path the
operator did not choose and no wrong wallet is derived. What is lost is the operator's
seating work and §8j's guarantee.

Census over the reachable set (base composable, i.e. seats can actually be held):
**1,200** `(list, idx)` pairs move the signature with no §8j, all of them `hash -> REFUSED`,
and in **all 1,200** the edited path is key-less — one shape, exactly as above.

---

## I-3 (IMPORTANT) — §8j fires, and lies, on a `tr` lock edit that cannot move a slot. Introduced by this fold.

Same probe, other direction. Because it clears the **hash** as well as the lock, it answers
a question about a path it has already changed. On a `tr` path that carries a hash, no lock
value can ever affect `isBareSingle()` — the hash already disqualifies it — yet the probe
reports the edit can renumber.

Reproduced end to end on `818220d8` (`composerPathEdit`, real screens), tr,
`[{1 key + hash lock}, {1 key}]`, slot @0 seated:

```
BEFORE: sig=w0/1,1,|1.0/0.0/ slots=[{Index:0 Path:1 Ordinal:0} {Index:1 Path:0 Ordinal:0}]
composerEditCanRenumber(list,0) = true
§8j drew: "EDITING THE SHAPE / CLEARS THE KEYS / Slot numbers change with the shape.
           Every key you seated will be cleared. Continue? Hold button to confirm."
AFTER : sig=w0/1,1,|1.0/0.0/ moved=false assigned=[0 -1]
```

Nothing moved and nothing was cleared. The screen's claim is false for the edit the
operator is making, and **declining it — the only response to a screen that threatens
every seat — returns `continue` and leaves the lock uneditable at all.** That is verbatim
the failure `composerEditCanRenumber`'s own doc comment says the function exists to
remove:

> Asking it before every lock editor told an operator who wanted to change a lock that
> every key would be cleared — false for the edit they intended, and declining it left the
> lock uneditable at all (§7g classifies a lock edit DEFAULT).

Census: **288 of the 1,156** §8j firings over composable lists are for nothing. All 288 are
under `tr` (wrapper 0), and in **all 288** the edited path carries a hash.

Reachability, stated honestly: not from a preset in one step. `hashlock-gated` under tr is
`[{1 key, sha256}, {1 key, older(26280)}]`, and there the probe correctly answers *false*
(putting @0 on path 0 changes nothing, because path 0 is already first) — I checked and it
draws no confirm. It needs a tr list where the hashed path has a **later** bare single,
e.g. hand-built `[{1 key + hash}, {1 key}]`, or `hashlock-gated` with path 2's lock cleared
and the slots re-seated.

### Both findings are ONE root cause — measured, not asserted

The probe uses **one** answer for **two** arms, and varies the wrong field for each. A
probe that varies only the field its own arm edits, holding the other as the operator left
it, is exact. Over the same 14,092 composable pairs:

```
shipped composerEditCanRenumber          : 1200 false negatives, 288 false positives
per-arm probe varying ONLY its own field :    0 false negatives,   0 false positives
```

Reported so the shared cause is measured. **Not a prescribed remedy** — the finding is the
defect, and the author should reproduce it before choosing a fix.

---

## M-2 (MINOR) — `composerShapeFlow`'s doc comment now contradicts the code

`gui/composer_shape.go:375-377`, untouched by this fold:

```
// THE DISCARD RULE HAS ONE PLACE TO LIVE, and it is composerPathEdit's Keys,
// Remove and Move arms plus composerAddPath -- the four that can move slot
// NUMBERING. A lock or a hash edit moves none (§7d) and is not guarded.
```

The lock and hash arms **are** now guarded, and a lock edit under tr **does** move slots —
that is the whole of I-1. The next reader of this function is told the opposite of what
the file two hundred lines up says.

## M-3 (MINOR) — `composerMoveUp`'s stated premise is stale, though its conclusion holds

`gui/composer_shape.go:447-449` still says the signature "carries the wrapper, the path
count and each path's key count". It now also carries `md.Composed.Slots()`. The
conclusion is unaffected and I measured it rather than assuming: swapping two equal-count
paths leaves the signature identical (`w1/1,1,|0.0/1.0/` before and after), so the
unconditional discard is still load-bearing, and it does clear both seats and both
`sources[i].used` flags.

## N-1 (NIT) — `composerSizeAssignments` does not release `sources[i].used`

Unlike `composerDiscardAssignments`, it rebuilds `st.assigned` without clearing the source
flags, so a resize with seats held would leak sources as `used` and never offer them again.
I enumerated **every** production write to `st.list` (below) and found no route that
changes the slot count outside `composerApplyShapeEdit` or `composerMoveUp`, so this is
unreachable today — recorded as hardening, not a defect.

---

## Round 1's findings

### C-1 — **FIXED**

The signature now asks the codec. Two independent proofs.

**Structural (the closure theorem).** If two lists share a signature, `md` numbers their
slots identically — so *any* door that swaps `st.list` under `composerApplyShapeEdit` is
safe, not just the ones tested:

```
TestSignatureEqualityImpliesEqualNumbering:
  4828 composable lists, 145 distinct signatures, 0 equal-signature pairs that renumber
```

Restated on C-1's own door, every preset against every hand-built shape, all four wrappers:

```
TestEveryPresetAgainstEveryHandBuiltShape:
  wrapper 0 offers 6 presets / wrapper 1: 6 / wrapper 2: 1 / wrapper 3: 1
  preset x hand-built pairs: 28948 checked, 28568 renumber,
                             0 of those invisible to the signature
```

**Behavioural (the brief's item 1, extended).** 39 sub-tests driving the production
`composerStartStep(…, fromPaths=true)` on real screens: the blank row and every preset
row, under every wrapper, accepted and declined, against a fully seated composition.
0 failed.

```
blank row, any wrapper : asked=false sigMoved=false seated=4/4   <- §7b, nothing lost
all 6 tr presets accept: asked=true  sigMoved=true  seated=0/N   <- §8j, every seat cleared
all 6 tr presets decline: asked=true sigMoved=false seated=4/4   <- shape AND seats intact
```

Assertions enforced per case: nothing may move without §8j; a decline may change neither
shape nor seats; an accepted move must clear every seat; no confirm where nothing is at
stake. Round 1's exact reproduction (`[2-of-2, 1 key, 1 key]` under tr → §8p → "Back to
the paths" → Back → `Start from?` → `decaying-multisig`) is the shipped
`TestComposerBackLegPresetAsksBeforeDiscardingSeats`, which passes and which mutation 1
and mutation 5 both break.

### I-1 — **FIXED**, and not over-closed under wsh

`TestComposerLockEditUnderTrDiscardsTheSeatsItMoves` passes; mutations 1, 2 and 4 each
break it with its own message. The quiet direction holds where it was tested: the three
"cannot matter" cases ask nothing and keep every seat —

```
tr, edited path is a 2-of-2 (never a bare single)  canRenumber=false  seated 4/4, no §8j
tr, an EARLIER path is already the bare single      canRenumber=false  seated 2/2, no §8j
wsh, any lock edit                                  canRenumber=false  seated 2/2, no §8j
```

Round 1's M-C guard (§8j must not fire where nothing is at stake) still bites: mutation 3
makes the arm always ask, and the shipped
`TestComposerLockAndHashEditsAreNotGuardedByTheDiscardConfirm` fails on both its `time
lock` and `hash lock` sub-tests. **I-3 above is exactly this class escaping that guard, on
the tr-with-hash case the shipped test does not cover.**

### M-1 — **FIXED**

`TestComposerBackAtTheWrapperPickerLeavesTheComposer` exists and passes. It can fail, and
the mutated line provably runs (mutation 6).

---

## Per-item results against the brief

### 1. C-1 closed — **VERIFIED**. See above (39 sub-tests + 28,948 pairs).

### 2. I-1 closed, not over-closed — **VERIFIED for the tested cases; the over-close guard has a gap (I-3).**

### 3. THE THIRD DOOR — **found, and it is I-2.**

I enumerated **every** production assignment under `st.list` in `gui/` (excluding
`_test.go`) rather than testing a list of suspects:

| site | route | wrapped? |
| --- | --- | --- |
| `composer_lock.go:194,266` | lock set / clear | now inside `composerApplyShapeEdit`, conditional guard |
| `composer_hash.go:163,170,173` | hash set / clear | now inside `composerApplyShapeEdit`, conditional guard — **I-2 lives here** |
| `composer_flow.go:165` | the Back leg | guard + `composerApplyShapeEdit` ✓ |
| `composer_shape.go:185` | `composerKeysEdit` | reached only via the keys arm and `composerAddPath`, both guarded ✓ |
| `composer_shape.go:210,217` | `composerKeyOrderStep`'s `Keys.Sorted` | **neither guarded nor wrapped** — see below |
| `composer_shape.go:231–279` | `composerAddPath` | guard + `composerApplyShapeEdit` ✓ |
| `composer_shape.go:326` | keys-edit restore | inside the same `composerApplyShapeEdit` ✓ |
| `composer_shape.go:353` | "Remove path" | guard + `composerApplyShapeEdit` ✓ |
| `composer_shape.go:424` | "Change the script" | guard + `composerApplyShapeEdit` ✓ |
| `composer_shape.go:465` | `composerMoveUp` | guard + unconditional discard ✓ |

The one unwrapped write is `Keys.Sorted`, and I did not leave it at inspection.
`composerSortedIsLegal` confines it to a single-path list with `N >= 2`, no lock and no
hash, under tr or wsh; over that whole reachable domain the write is mapping-neutral:

```
TestSortedIsMappingNeutral: Sorted checked over 4 reachable lists — 0 signature moves
```

I also checked the two acceptance predicates cannot diverge, since the signature keys on
`md.Compose` while Done keys on `md.ValidatePathList` — a disagreement would let seats be
held on a list whose signature has no mapping:

```
TestComposeAndValidateAgree: Compose/Validate disagreements: 0   (over 17,472 lists)
```

`composerMoveUp` was tested, not reasoned about (M-3). So: **the third door is I-2, and it
is inside the fold's own new wrapper, not in an untouched arm.**

### 4. `composerEditCanRenumber`'s probe — **NOT VERIFIED. This is I-2 and I-3.**

Attacked exhaustively rather than by cases, over wrappers × 1..3 paths × `{nil, 1-of-1,
2-of-2, 2-of-3}` × `{no lock, older(7)}` × `{no hash, digest}` = 17,472 lists, against
every reachable post-state of each arm (`Lock ∈ {nil, older-blocks, older-units,
after-height, after-time}` from `composerLockEdit`; `Hash ∈ {nil, d1, d2}` from
`composerHashEdit`):

```
probe soundness : 51264 (list,idx) pairs examined, 3600 false negatives
                  restricted to lists that COMPOSE (i.e. can hold seats):
                  14092 pairs, 1200 unguarded signature moves, kinds = [hash -> REFUSED:1200]
probe precision : 14092 composable pairs, 1156 fire §8j, 288 fire with nothing at stake
```

The brief's four named attacks, answered: a path with `Keys == nil` and a hash → **I-2**;
a path already at the slot cap → the probe changes no key count, so it cannot cross the cap
(no case found); a shape whose probe variants fail to compose while the real edit would not
→ **I-2**, exactly that; anything where the probe answers false and the real edit still
moves the mapping → **I-2**, 1,200 of them, one shape.

### 5. The signature's fallback — **VERIFIED**

```
TestFallbackCollisions: refused shapes: 56 distinct signatures, 668 collisions
                        (all same slot count)
```

Two different refused shapes can share a signature, but never with different slot counts,
so a refused→refused edit can neither carry a seat onto a moved slot nor desynchronise
`len(st.assigned)`. And a refused signature can never equal a composed one, because the
composed form always appends `|…`. Seats cannot in any case be held on a refused list: the
valid→refused transition itself moves the signature (which is what I-2 exploits).

### 6. Gates, as CI runs them — **VERIFIED, every one reproduced**

| gate | claim | measured | |
| --- | --- | --- | --- |
| `gofmt -l cmd/` | clean | clean, exit 0 | ✓ |
| `gofmt -l gui/` | 3 pre-existing | `transaction.go`, `transaction_golden_test.go`, `transaction_txrecord_test.go` | ✓ |
| `go vet ./gui/ ./cmd/...` | only 2 `ArtifactDir` lines | exactly those 2 — **and byte-identical on `70008da`, which I ran** | ✓ |
| `go test ./...` | 0 FAIL | exit 0, 0 FAIL, 74 ok/no-test-files | ✓ |
| sharded gui (24) | all 1199 ran | `partition verified exhaustive: 1199 == 1199`, `RESULT: ok -- all 1199 tests ran across 24 shards`, wall 41s | ✓ |
| `./scripts/test-32bit.sh` | exit 0 both | `GOARCH=386 test: exit 0`, `GOARCH=arm build: exit 0` | ✓ |
| `go build ./cmd/...` | exit 0 | exit 0 | ✓ |

**Shard count accounted for:** `git diff 05466727..818220d8 | grep -c "^+func Test"` = **4**,
all four in `composer_backleg_test.go`. 1195 + 4 = **1199**. ✓

**Firmware — I measured BOTH ends myself** (round 1 took the baseline on trust; I extracted
`70008da` with `git archive` into a tree of its own and built it):

```
70008da  code 1549408  data 31796  bss 31004 | flash 1581204  ram 62800
818220d8 code 1550768  data 31796  bss 31004 | flash 1582564  ram 62800
```

**+1,360 B flash, +0 B RAM.** Both figures match the fold commit's message exactly. RAM
unmoved is the right answer: the fold adds two pure functions and one `append` on a
stack-local slice copy, and allocates nothing statically.

### 7. The four capture drivers — **VERIFIED**

Against `EMU=/scratch/code/shibboleth/.s4e-verify2/cmd/emu`:

```
RC_capture_composer=0          ("all legs matched the host"; ENGRAVED byte for byte
                                across the air gap, template-ID e0863d3c…, 8 shots)
RC_capture_walletpolicy=0
RC_capture_seating=0
RC_capture_tr_pathological=0
```

No journey regression.

### 8. The spec fold — **NOT VERIFIED (it describes the intent, not the code)**

§7d's measured claim reproduces exactly, so the *evidence* in it is sound:

```
hand   wrapper=0 paths=3 counts=211 slots=[{0 p1 o0} {1 p0 o0} {2 p0 o1} {3 p2 o0}]
preset wrapper=0 paths=3 counts=211 slots=[{0 p0 o0} {1 p0 o1} {2 p1 o0} {3 p2 o0}]
slots total=4 moved=3   (spec claims "three of their four")   signatures now differ: true
```

The three terms agree, three of four slots disagree — verbatim as written. §7b's refined
Back sentence is accurate.

But the new §7d sentence —

> §8j is asked wherever that mapping would move — for a lock or hash edit, **exactly when
> the codec says that edit can renumber under this wrapper**. A lock or hash edit that
> moves no slot — every such edit under wsh, and most under tr — keeps assignments, asks
> nothing…

— is false in both halves as implemented. I-2 is a hash edit whose mapping moves and which
asks nothing; I-3 is a lock edit that moves no slot and asks anyway. The brief's rule for
this item is that §7d must describe what the code does. It currently describes what the
code will do once I-2 and I-3 are closed, so this resolves with them rather than needing
its own edit.

---

## Every mutation run

All in `/scratch/code/shibboleth/.s4e-mut` (a copy, restored between each; the tree was
re-verified green at the end).

| # | mutation | caught by | its own message |
| --- | --- | --- | --- |
| 1 | `composerShapeSignature` back to structural-only | `…ShapeSignatureSeesTheCodecsNumbering`, `…BackLegPresetAsksBeforeDiscardingSeats`, `…LockEditUnderTrDiscardsTheSeatsItMoves` | "two shapes whose slots the codec numbers DIFFERENTLY share a shape signature…"; "a preset replaced a seated shape with no §8j…"; "§8j did not fire before a lock edit that CAN renumber under tr…" |
| 2 | NEVER ask on the lock arm (`if false &&`) | `…LockEditUnderTrDiscardsTheSeatsItMoves` | "§8j did not fire before a lock edit that CAN renumber under tr…" |
| 2a | panic proof on the same line | — | `panic: MUT2-LOCK-ARM-RAN` — the mutated arm provably executes |
| 3 | ALWAYS ask on the lock and hash arms (`if true &&`, 2 sites) | `…LockAndHashEditsAreNotGuardedByTheDiscardConfirm` | both sub-tests (`time lock`, `hash lock`) FAIL — the over-fire guard bites |
| 4 | lock arm applies OUTSIDE `composerApplyShapeEdit` | `…LockEditUnderTrDiscardsTheSeatsItMoves` | "slot @0 is still seated (src=0) after a lock edit that moved it from path 1 to path 0…" |
| 5 | the Back leg drops its §8j condition (`if false &&`) | `…BackLegWrapperChangeAsks…`, `…BackLegPresetAsksBeforeDiscardingSeats` | "the wrapper changed on the Back leg without asking §8j…"; "a preset replaced a seated shape with no §8j…" |
| 6 | Back at the wrapper picker `continue`s instead of returning false | `…BackAtTheWrapperPickerLeavesTheComposer` | "Back at the wrapper picker did not leave the composer; the flow is still drawing, so the leg has no exit." |
| 6a | panic proof on the same line | — | `panic: MUT6-WRAPPER-BACK-RAN` |

**All four new tests can fail, and each fails for its own named reason.** Unmutated, all
140 `TestComposer*` tests pass with 0 FAIL.

Regression control (not a mutation of the fold, a restoration of `05466727`): I-2's walk in
`.s4e-prefold` returns `assigned=[0 1]` where `818220d8` returns `[-1 -1]`.

---

## Diff scope

```
$ git diff 05466727..818220d8 --stat
 gui/composer_backleg_test.go | 298 +++++++++++++++++++++++++++++++++++++++++++
 gui/composer_discard.go      |  73 ++++++++++-
 gui/composer_shape.go        |  35 +++--
 3 files changed, 392 insertions(+), 14 deletions(-)
```

Three files, two of them production, both read in full. **No hunk touches seating, the
codec, the stub screen, the census or the engrave path.** Nothing outside the fold.

---

## Reproduction (inlined — the scratch copies do not survive)

Drop into `gui/` of a copy of the worktree. Environment:
`PATH=/scratch/code/shibboleth/.toolchain/go/bin:$PATH`,
`TMPDIR=/scratch/code/shibboleth/.tmp`, `CGO_ENABLED=0 GOPROXY=off GOTOOLCHAIN=local`,
`go test -mod=readonly ./gui/ -run TestVerify2 -v`.

```go
package gui

import (
	"testing"
	"testing/synctest"

	"seedhammer.com/md"
)

// I-2: the hash arm's "No hash lock" row on a KEY-LESS path discards every
// seat with no §8j. Pre-fold (05466727) the same walk kept both seats.
func TestVerify2_I2(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		d := [32]byte{0xAB}
		st := &composerState{reg: &seedRegistry{}, list: md.PathList{
			Wrapper: md.ComposeWsh,
			Paths: []md.SpendPath{
				{Keys: &md.KeySet{K: 2, N: 2, Sorted: true}},
				{Hash: &d},
			},
		}}
		composerSizeAssignments(st)
		st.sources = []composerSource{{seedID: -1}, {seedID: -1}}
		for i := range st.assigned {
			st.assigned[i].src = i
			st.sources[i].used = true
		}
		frame, quit := runUI(ctx, func() { composerPathEdit(ctx, &descriptorTheme, st, 1) })
		defer quit()
		pumpUntil(frame, "Path 2:", 16)
		click(&ctx.Router, Down, Down) // Keys -> Time lock -> Hash lock
		click(&ctx.Router, Button3)
		if got, ok := pumpUntil(frame, "CLEARS THE KEYS", 16); ok {
			t.Fatalf("§8j fired (would be correct): %q", got)
		}
		pumpUntil(frame, "Which hash?", 24)
		click(&ctx.Router, Down) // "Type 64 hex" -> "No hash lock"
		click(&ctx.Router, Button3)
		for range 24 {
			frame()
		}
		for i, a := range st.assigned {
			if a.src < 0 {
				t.Errorf("seat @%d discarded by a hash edit that drew no §8j", i)
			}
		}
	})
}

// I-3: §8j fires, and lies, on a tr lock edit that cannot move a slot.
func TestVerify2_I3(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		d := [32]byte{0xAB}
		st := &composerState{reg: &seedRegistry{}, list: md.PathList{
			Wrapper: md.ComposeTr,
			Paths: []md.SpendPath{
				{Keys: &md.KeySet{K: 1, N: 1, Sorted: true}, Hash: &d},
				{Keys: &md.KeySet{K: 1, N: 1, Sorted: true}},
			},
		}}
		composerSizeAssignments(st)
		st.sources = []composerSource{{seedID: -1}}
		st.assigned[0].src = 0
		before := composerShapeSignature(st.list)
		frame, quit := runUI(ctx, func() { composerPathEdit(ctx, &descriptorTheme, st, 0) })
		defer quit()
		pumpUntil(frame, "Path 1:", 16)
		click(&ctx.Router, Down) // -> Time lock
		click(&ctx.Router, Button3)
		got, asked := pumpUntil(frame, "CLEARS THE KEYS", 16)
		if !asked {
			return
		}
		click(&ctx.Router, Button1) // decline
		for range 20 {
			frame()
		}
		if g, ok := pumpUntil(frame, "What kind of time lock?", 12); ok {
			t.Fatalf("lock editor drew after a decline: %q", g)
		}
		if composerShapeSignature(st.list) == before {
			t.Errorf("§8j drew (%q) for an edit that cannot move a slot, and declining it "+
				"left the lock uneditable", got)
		}
	})
}
```

The exhaustive harnesses (`venumerate` over 17,472 lists; the soundness, precision,
fallback, closure-theorem, preset-sweep, Sorted-neutrality and root-cause tests) are
reconstructible from the counts quoted above; each is a straight loop over that alphabet
comparing `composerShapeSignature` before and after a single-field edit.

---

## Counts

| severity | n | |
| --- | --- | --- |
| Critical | 0 | — |
| Important | 2 | **I-2**: the hash arm skips §8j and discards every seat (key-less path, "No hash lock") — **introduced by this fold**. **I-3**: §8j fires and lies on a tr lock edit that cannot renumber, and declining leaves the lock uneditable — **introduced by this fold**. One root cause. |
| Minor | 2 | M-2: `composerShapeFlow`'s doc comment now contradicts the code. M-3: `composerMoveUp`'s stated premise is stale (its conclusion re-measured and still true). |
| Nit | 1 | N-1: `composerSizeAssignments` does not release `sources[i].used` (no reachable route; hardening). |

**C-1: FIXED. I-1: FIXED. M-1: FIXED.** The signature's root cause is closed structurally —
0 equal-signature renumbering pairs over 4,828 composable lists, 0 over 28,948 preset ×
hand-built pairs. Every CI gate reproduced, both firmware figures measured at both ends, all
four capture drivers exit 0, no hunk outside the fold, and all six mutations caught by their
own named assertions with the two riskiest lines proved to execute.

The merge should wait on I-2 and I-3, which are one line in one new function and are both
regressions against `05466727`.

*F-470 not re-opened. No secret-handling defect observed.*
