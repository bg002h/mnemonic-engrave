# Hashlock H5 — Device Polish Implementation Plan (SeedHammer fork)

**STATUS: DRAFT 2026-09-05 — build gate RUN, R0 review pending.** Every code block
below was hand-wired into a scratch copy of fork main `b9a9a30` at
`/scratch/code/shibboleth/.tmp/h5-gate` and built and tested **at every task
boundary** (not once at the end — the H2 plan's gate wired all six tasks at once
and so could not see that its Task 3 used a field its Task 4 added). Each task's
RED, GREEN and every `MUTATION:` in it was executed and its measured failure is
quoted at the task. `scripts/h5-plan-blocks-vs-tree.sh` compares every block here
against that tree; its tail is in `## Build gate`.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development
> (recommended) or superpowers:executing-plans to implement this plan task-by-task.
> Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fold H2's five ruled follow-ups into the shipped device leg as ONE fork
branch: the reconcile screen carries the digest it asks the operator to compare
(F-487), hash provenance becomes per digest instead of one flag per policy
(F-480), the phrase screen's lead stops painting inside the Back button (F-484),
the emulator walk proves hold ORDER and stored-versus-displayed (F-485), and the
unlock refusal says what to do next (F-488).

**Architecture:** Four of the five are edits inside files H2 created.
The fifth adds the package's FOURTH `//go:build` pair — a composition-state seam
on the `frame_hook.go` model — plus a `//go:build js` glue file in `cmd/emu` and
a rewritten walk. `composerState.hashByPhrase` is replaced by a value set of
digests plus a predicate over the current paths, which deletes
`composerHashByPhraseSync` and both of its call sites rather than adding a third.

**Tech Stack:** Go 1.26 (`/scratch/code/shibboleth/.toolchain/go/bin/go`), the fork's
`gui` touch harness (`runUITouch`, `sessionHarness`, `runUITouchRaster`), TinyGo via
`nix develop -c tinygo build …` for size, `cmd/emu` (GOOS=js) for the walk,
`scripts/gui-shard-test.sh` (engrave) for the whole `gui` package.

**Spec:** `design/SPEC_hashlock_H5_device_polish.md` (R0 GREEN at engrave `e03d8e7`:
three lenses, one fold, two verifications) — §1 the reconcile screen, §2
provenance, §3 the lead's band, §4 the walk and its seam, §5 the refusal, §6
tests, §7 acceptance. Parent: `design/SPEC_hashlock_H2_device.md`.

**Baselines (for `scripts/plan-staleness-check.sh`):** seedhammer fork main
`b9a9a30`; mnemonic-engrave `e03d8e7`; mnemonic-toolkit `46b40bb`. Every
`file:line` below was re-grepped at `b9a9a30`.

---

## Global Constraints

- **The fork baseline is RED in one package, and Task 5 fixes it.** `CGO_ENABLED=0
  go test ./...` — CI's own command (`.github/workflows/test.yml:75`) — FAILS at
  `b9a9a30`: `TestWalkOkContainsNoDriverSuppliedPlateCount`
  (`cmd/emu/needle_test.go`) reports INCONCLUSIVE, which is a `t.Errorf`, for
  `walk_h0_preimage.js` and `walk_hashlock_phrase.js`. Its `okExprRe` reads only
  the object-literal form `ok: <expr>,`; both of those walks ASSIGN
  (`out.ok = <expr>;`) and have since `45f3d4c` (H0) and `e1bf137` (H2)
  respectively. **This was NOT introduced by H5** — measured on the pristine fork
  checkout, quoted in `## Build gate` — and H5 may not leave it standing, both
  because a red suite is itself a blocking finding and because spec §4.4 changes
  this walk's `ok` to exactly the shape the guard cannot read. Task 5 Step 6
  teaches the guard the assignment shape.
- **The fit gates are the arbiter of every copy change, and no character budget
  is asserted.** `assertModalBodyFits` (`gui/modal_fits_test.go`) renders the body
  and measures headroom against `modalBodyMargin = 80`; headroom is a LINE budget,
  not a character budget (`gui/modal_fits_test.go:30-32`). Every measured number
  quoted below came from a run of that gate on the wired tree and is reproduced
  in `## Build gate`.
- **Copy is ASCII and gated twice:** a new or changed `composerCopy*` body needs
  its `composerCopyTable()` row (`gui/composer_copy_test.go`), which
  `TestComposerCopyTableCoversEveryBody` AST-scans `composer_copy.go` to enforce
  along with the exact `declared` count, and a row in a fit table. **This stage
  adds NO new `composerCopy*` function**, so the `declared != 53` literal does not
  move — verified by running the test.
- **Rust-primary (CLAUDE.md):** nothing here touches a ported codec. The only
  normative host-facing text is the toolkit manual's quotes of device copy, which
  follow the device (Task 6), not the reverse.
- **Secret-handling defects never gate** (operator ruling 2026-08-27).
- **The composition-state seam READS and never DRIVES** (spec §4.1). It hands out
  copies of digests; driving stays with `shTap`/`shPress`/`shRelease`.
- **Fork commits** `git commit -s` (DCO), author Brian Goss, branch `hashlock-h5`
  off fork `main` `b9a9a30`; stage paths explicitly, never `git add -A`.
- **Flash only via `~/bin/sh/sh2-flash -y` at the operator's word**; never picotool
  by hand.

---

## File Structure

| File | Change | Task | Responsibility |
| --- | --- | --- | --- |
| `gui/composer_state.go` | Modify | 1 | `hashByPhrase bool` → `phraseDigests map[[32]byte]struct{}`; `composerNotePhraseDigest`, `composerAnyPathByPhrase` |
| `gui/composer_hash.go` | Modify (delete) | 1 | `composerHashByPhraseSync` and its `noneRow` call removed |
| `gui/composer_shape.go` | Modify (delete) | 1 | the Remove arm's sync call removed |
| `gui/composer_hashlock.go` | Modify | 1,2,3 | HOLD notes the digest; the reconcile call gains its arguments; `hashlockPhraseLead` |
| `gui/composer_copy.go` | Modify | 1,2 | §8h's "every … and every"; the reconcile body's arguments and mismatch sentence; the write-down sentence; the corrected headroom note |
| `gui/composer_provenance_test.go` | Create | 1 | §2's four tests |
| `gui/composer_copy_test.go` | Modify (rows) | 1,2 | three rows and one helper; the `declared` literal does NOT move |
| `gui/composer_hashlock_test.go` | Modify | 1,2 | six existing tests re-aimed; two new tests |
| `gui/modal_fits_test.go` | Modify (one row) | 2 | the reconcile row takes the longest variant |
| `gui/composer_paged.go` | Modify | 3 | `composerTextBand`, factored out of `composerPageLines` |
| `gui/passphrase_keyboard.go` | Modify | 3 | `readoutGap` → package-level `ppReadoutGap` |
| `gui/composer_hashlock_geometry_test.go` | Create | 3 | §3.2's three gates plus the probe's own proof |
| `gui/unlock_kdf.go` | Modify | 4 | the refusal's next step |
| `gui/unlock_preimage_test.go` | Modify | 4 | the frame assertion, three rows, the longest-noun fit row |
| `gui/composer_state_hook.go` | Create | 5 | `//go:build !tinygo`: `composerStateHook`, `setComposerStateHook`, `clearComposerStateHook`, `ComposerPathHashes` |
| `gui/composer_state_hook_tinygo.go` | Create | 5 | `//go:build tinygo`: the empty twin, with the measured sizes |
| `gui/composer_state_hook_test.go` | Create | 5 | the seam's lifetime and read contract |
| `gui/composer_flow.go` | Modify | 5 | `composerFlowExit`: ONE defer for the scrub and the hook |
| `gui/tinygo_split_test.go` | Modify | 5 | `nonInterfaceHookPairs`, told out loud, and still scanned |
| `cmd/emu/composer_js.go` | Create | 5 | `//go:build js`: `window.shComposerPathHashes()` |
| `cmd/emu/platform.go` | Modify (one line) | 5 | `installComposerAPI()` |
| `cmd/emu/walk_hashlock_phrase.js` | Modify | 5 | the order and stored-versus-displayed assertions; `ok` set, not recomputed |
| `cmd/emu/needle_test.go` | Modify | 5 | the `ok` guard reads the assignment shape (the baseline red) |
| engrave `design/SPEC_hashlock_H2_device.md` | Modify | 6 | §4.5 and §4.7 folds |
| engrave `design/FOLLOWUPS.md` | Modify | 6 | five closures with owning-phase reconciliation |
| toolkit `docs/manual/src/40-cli-reference/43-ms.md` | Modify | 6 | re-quote the two changed screens |

**Gate coverage — what the checker script does and does not prove.** Every fenced
block below that carries file content opens with

    ```go file=gui/composer_state.go mode=fragment
    ```go file=gui/composer_state_hook.go mode=whole

`mode=whole` means the block IS the file; `mode=fragment` means it must appear
VERBATIM inside it, indentation included. Markdown takes only the FIRST word of an
info string as the language, so highlighting is unaffected.
`scripts/h5-plan-blocks-vs-tree.sh` parses those headers and checks every block
against the gated tree — whole blocks by `diff`, fragments by exact substring —
and prints its own blind spots. A block with NO header is a command, an
illustration, or (Task 6) text destined for a file outside the fork tree, and
nothing checks it. The script says so on every run; read its tail rather than this
paragraph.

---


### Task 1: Per-digest hash provenance (spec §2, F-480)

**Files:**
- Modify: `gui/composer_state.go`, `gui/composer_hash.go`, `gui/composer_shape.go`, `gui/composer_hashlock.go`, `gui/composer_copy.go`
- Modify (tests): `gui/composer_copy_test.go`, `gui/composer_hashlock_test.go`
- Create: `gui/composer_provenance_test.go`

**Interfaces:**
- Removed: `composerState.hashByPhrase bool`; `composerHashByPhraseSync(*composerState)` and its two call sites (`gui/composer_hash.go:237`, `gui/composer_shape.go:356` at `b9a9a30`).
- Produces: `composerState.phraseDigests map[[32]byte]struct{}`; `composerNotePhraseDigest(st *composerState, h [32]byte)`; `composerAnyPathByPhrase(st *composerState) bool`.
- Consumes: `md.PathList.Paths []md.SpendPath` with `Hash *[32]byte`; `composerEveryPathHashed(list md.PathList) bool` (`gui/composer_state.go:244`).

**Why this task is FIRST.** The six sites that name `hashByPhrase` at `b9a9a30`
include `gui/composer_hashlock_test.go:916`, an assertion inside a test Task 2
also edits, and `gui/composer_copy_test.go:144`, a row Task 2 also edits. Landing
the state change first means every later task edits a file that already compiles;
the reverse order does not build at any intermediate commit.

- [ ] **Step 1: The failing tests.** Create `gui/composer_provenance_test.go`:

```go file=gui/composer_provenance_test.go mode=whole
package gui

import (
	"strings"
	"testing"

	"seedhammer.com/md"
)

// ─── H5 §2 (F-480): provenance is per DIGEST, not one flag per policy ────────
//
// composerState.hashByPhrase was one bool for a whole composition: set at HOLD,
// cleared only when NO path carried a hash at all. Replace a phrase-set hash on
// path 1 with a payload row while path 2 keeps a hex hash and Done still named a
// phrase the composition no longer has -- the operator is told to back up an
// artifact that does not exist, at the one screen whose job is to say what
// spending needs.
//
// The replacement is a value SET of the digests this composition derived from a
// phrase, and a predicate that walks the CURRENT paths. Nothing deletes from the
// set, because a value set cannot go stale: a digest no path carries is simply
// never matched.

// composerFlowShapedState builds composerState EXACTLY as composerFlow does
// (gui/composer_flow.go:34) -- a struct literal that never mentions
// phraseDigests, so the map arrives nil.
//
// THE NIL IS THE POINT. An assignment into a nil map panics, and this is the
// one production construction site, so a helper that did not allocate would
// panic on the machine at the moment the operator holds to confirm a hash that
// gates funds. Every test in this package builds the state the same way.
func composerFlowShapedState(t *testing.T, paths int) *composerState {
	t.Helper()
	st := &composerState{reg: &seedRegistry{}, bound: composerBoundFrom(nil)}
	if st.phraseDigests != nil {
		t.Fatal("this helper exists to reproduce the NIL map composerFlow leaves; it is not nil")
	}
	st.list = md.PathList{Wrapper: md.ComposeWsh, Paths: make([]md.SpendPath, paths)}
	return st
}

// TestComposerPhraseRouteHoldsOnTheZeroValueState is fidelity C-1, executed.
//
// The whole phrase route is driven to HOLD on a state built the way production
// builds it. If composerNotePhraseDigest did not allocate, this panics inside
// the GUI goroutine at the assignment -- it does not merely fail.
//
// MUTATION: drop the nil check from composerNotePhraseDigest (assign straight
// into st.phraseDigests) -> `panic: assignment to entry in nil map`.
func TestComposerPhraseRouteHoldsOnTheZeroValueState(t *testing.T) {
	st := composerFlowShapedState(t, 1)
	var ret bool
	h := runComposerHashEdit(t, st, composerSessionWith(nil, nil), 0, &ret)
	h.mustReach("Type a hashlock phrase")
	h.tapRow(0, 3)
	h.mustReach("32-byte value")
	h.tapNav(Button3)
	h.mustReach("Hashlock phrase")
	typeOnPassphraseKeyboard(t, h, hashlockAnchorPhrase)
	h.tapNav(Button3)
	h.mustReach("Which method?")
	h.tapRow(1, 2) // sha256: instant
	h.mustReach("brainwallet")
	h.holdConfirm()
	h.mustReach("Write down this phrase")
	h.holdConfirm()
	h.mustReach("run ms hashlock with this phrase")

	want := hashlockMustHex(t, hashlockAnchorSHA_H)
	if _, ok := st.phraseDigests[want]; !ok {
		t.Fatalf("the anchor's digest is not in the phrase set (%d entries)", len(st.phraseDigests))
	}
	if !composerAnyPathByPhrase(st) {
		t.Error("a path carries a phrase-derived digest and the predicate says otherwise")
	}
}

// TestComposerAnyPathByPhraseIsPerDigest is the predicate itself, over the
// compositions the flag got wrong.
//
// MUTATION: report len(st.phraseDigests) > 0 without walking the paths ->
// "the phrase path was edited to a payload row" and "the phrase path was
// removed" both fail.
// MUTATION: compare p.Hash pointers instead of the digest VALUE (`for d := range
// st.phraseDigests { if p.Hash == &d }`) -> every positive row fails.
func TestComposerAnyPathByPhraseIsPerDigest(t *testing.T) {
	phrase := hashlockMustHex(t, hashlockAnchorSHA_H)
	other := hashlockMustHex(t, strings.Repeat("5a", 32))

	for _, tc := range []struct {
		name string
		set  [][32]byte
		hash []*[32]byte // one entry per path; nil means no hash
		want bool
	}{
		{"no paths at all", nil, nil, false},
		{"a nil set is read, not written", nil, []*[32]byte{&phrase}, false},
		{"one phrase path", [][32]byte{phrase}, []*[32]byte{&phrase}, true},
		{"the phrase path was edited to a payload row", [][32]byte{phrase}, []*[32]byte{&other}, false},
		{"the phrase path was removed", [][32]byte{phrase}, []*[32]byte{nil}, false},
		{"a mixed wallet: one phrase path, one other", [][32]byte{phrase}, []*[32]byte{&phrase, &other}, true},
		{"two paths share one phrase digest", [][32]byte{phrase}, []*[32]byte{&phrase, &phrase}, true},
		{"the same digest re-typed as 64 hex is still by phrase", [][32]byte{phrase}, []*[32]byte{&phrase}, true},
	} {
		t.Run(tc.name, func(t *testing.T) {
			st := composerFlowShapedState(t, len(tc.hash))
			for _, d := range tc.set {
				composerNotePhraseDigest(st, d)
			}
			for i, h := range tc.hash {
				st.list.Paths[i].Hash = h
			}
			if got := composerAnyPathByPhrase(st); got != tc.want {
				t.Errorf("composerAnyPathByPhrase = %v, want %v", got, tc.want)
			}
		})
	}
}

// TestComposerHashEditToAPayloadRowDropsThePhraseForm is F-480's own scenario,
// driven through the production edit rather than asserted on the predicate.
//
// Path 1's phrase-set hash is replaced by a `hash:` record from the payload
// while path 2 keeps a hex hash. Every path is still hashed, so §8h fires -- and
// under the old bool it fired in the PHRASE form, naming a phrase this
// composition no longer holds.
//
// MUTATION: report len(st.phraseDigests) > 0 in composerAnyPathByPhrase -> the
// banner is the phrase form and this fails.
func TestComposerHashEditToAPayloadRowDropsThePhraseForm(t *testing.T) {
	payloadDigest := strings.Repeat("ab", 32)
	st := composerFlowShapedState(t, 2)
	phrase := hashlockMustHex(t, hashlockAnchorSHA_H)
	hexed := hashlockMustHex(t, strings.Repeat("5a", 32))
	st.list.Paths[0].Hash = &phrase
	composerNotePhraseDigest(st, phrase)
	st.list.Paths[1].Hash = &hexed
	if !composerAnyPathByPhrase(st) {
		t.Fatal("the fixture must START in the phrase form for this test to mean anything")
	}

	var ret bool
	h := runComposerHashEdit(t, st, composerSessionWith([]string{"hash:" + payloadDigest}, nil), 0, &ret)
	h.mustReach("Which hash?")
	h.tapRow(0, 4) // the one payload row, above the phrase / hex / none rows
	h.mustReach("32-byte value")
	h.tapNav(Button3)
	h.waitDone()
	if !ret {
		t.Fatal("composerHashEdit returned false after a payload row")
	}
	if got := hashlockHashHex(st.list.Paths[0].Hash); got != payloadDigest {
		t.Fatalf("path 1 hash = %s, want the payload row %s", got, payloadDigest)
	}
	if !composerEveryPathHashed(st.list) {
		t.Fatal("this test needs a composition §8h's guard ACCEPTS")
	}
	if composerAnyPathByPhrase(st) {
		t.Error("no path carries a phrase-derived digest and the predicate still says one does")
	}
	if got := composerCopyHashEveryPathFor(st); got != composerCopyHashEveryPath() {
		t.Errorf("§8h names a phrase this composition no longer has:\n%q", got)
	}
}

// TestComposerMixedWalletBannerNamesEveryPhraseAndEveryPlate is journey I-3.
//
// On a wallet with one phrase path and one payload-row path BOTH backups are
// needed -- one per path -- and the shipped sentence offered a choice ("the
// phrase and its method, OR the preimage plate"), which is an undercount at the
// one screen the operator reads to learn what spending needs.
//
// MUTATION: restore "Back up the phrase and its method, or the preimage plate,
// separately." -> both assertions fail.
func TestComposerMixedWalletBannerNamesEveryPhraseAndEveryPlate(t *testing.T) {
	st := composerFlowShapedState(t, 2)
	phrase := hashlockMustHex(t, hashlockAnchorSHA_H)
	fromPlate := hashlockMustHex(t, strings.Repeat("ab", 32))
	st.list.Paths[0].Hash = &phrase
	composerNotePhraseDigest(st, phrase)
	st.list.Paths[1].Hash = &fromPlate

	if !composerEveryPathHashed(st.list) || !composerAnyPathByPhrase(st) {
		t.Fatal("this test needs a MIXED wallet that §8h's guard accepts")
	}
	body := composerCopyHashEveryPathFor(st)
	for _, want := range []string{"every phrase and its method", "every preimage plate"} {
		if !strings.Contains(body, want) {
			t.Errorf("§8h's phrase form does not carry %q:\n%q", want, body)
		}
	}
	if strings.Contains(body, "method, or the") {
		t.Errorf("§8h's phrase form still offers a CHOICE of backups:\n%q", body)
	}
}
```

- [ ] **Step 2: Re-aim the three existing tests that name the removed field.**

`TestHashlockReconcileScreenIsReachableOnAMixedPolicy` asserted the flag
directly (`gui/composer_hashlock_test.go:916` at `b9a9a30`):

```go file=gui/composer_hashlock_test.go mode=fragment
	// r0 tests I-4, now per digest (H5 §2): the set's real insertion, driven
	// through the route rather than built as a struct literal. MUTATION: delete
	// the composerNotePhraseDigest(st, h) call from hashlockPhraseRoute -> this
	// fails.
	if !composerAnyPathByPhrase(st) {
		t.Fatal("the phrase route did not record that this hash was set by phrase")
	}
```

The `No hash lock` row test set the flag and asserted the sync cleared it. Both
halves change — the setter becomes the helper, and the assertion becomes the
predicate, with the MUTATION that now bites in place of the deleted function's:

```go file=gui/composer_hashlock_test.go mode=fragment
		st.list.Paths[0].Hash = &preset
		composerNotePhraseDigest(st, preset)
```

```go file=gui/composer_hashlock_test.go mode=fragment
		// H5 §2: `No hash lock` needs NO bookkeeping. It sets p.Hash to nil and
		// composerAnyPathByPhrase reads p.Hash, so the provenance follows the
		// edit by construction -- which is what let composerHashByPhraseSync and
		// both its call sites be deleted.
		// MUTATION: make composerAnyPathByPhrase report len(st.phraseDigests) > 0
		// instead of walking the paths -> this fails.
		if composerAnyPathByPhrase(st) {
			t.Fatal("the phrase form survived the last hash being cleared")
		}
```

`TestRemovePathReSyncsHashByPhrase` is RENAMED and re-scoped: with provenance per
digest there is no sync call to delete, so the test drives the whole event
instead and requires §8h's plain form at the end.

```go file=gui/composer_hashlock_test.go mode=fragment
// Post-impl interruption M-1, re-aimed at the value set (H5 §2). "Remove path"
// was the event that made the composition-wide flag stale, and it needed a
// composerHashByPhraseSync call in composerPathEdit's Remove arm to stay
// honest. With provenance held per DIGEST there is no call to make: the splice
// removes the path, composerAnyPathByPhrase walks the paths that remain, and
// the digest left in the set is simply never matched again.
//
// So this test now drives the WHOLE event: remove the phrase-set path, give the
// survivor a hash typed as 64 hex, and require §8h's PLAIN form -- the exact
// composition F-480 says the flag got wrong in the other direction.
//
// MUTATION: make composerAnyPathByPhrase report len(st.phraseDigests) > 0
// instead of walking the paths -> the removed path's digest is still in the set,
// so the phrase form is drawn for a composition that has no phrase in it and
// this fails.
func TestRemovePathThenAHexHashDrawsThePlainBanner(t *testing.T) {
```

```go file=gui/composer_hashlock_test.go mode=fragment
		st.list.Paths[0].Hash = &d
		composerNotePhraseDigest(st, d) // path 1's hash came from a phrase; path 2 has none
```

```go file=gui/composer_hashlock_test.go mode=fragment
		if composerAnyPathByPhrase(st) {
			t.Fatal("the only phrase-set path was removed and the phrase form is still chosen")
		}
		// The survivor gets a hash the operator typed as 64 hex -- a DIFFERENT
		// digest, so nothing in the set matches it. Every path is hashed again,
		// so §8h fires, and it must be the plain form.
		hexed := hashlockMustHex(t, strings.Repeat("5a", 32))
		st.list.Paths[0].Hash = &hexed
		if !composerEveryPathHashed(st.list) {
			t.Fatal("this test needs a composition §8h's guard ACCEPTS")
		}
		if got := composerCopyHashEveryPathFor(st); got != composerCopyHashEveryPath() {
			t.Errorf("§8h drew the phrase form for a composition with no phrase-set hash:\n%q", got)
		}
	})
}
```

- [ ] **Step 3: The copy table's two §8h rows and the state the second needs.**
The `declared` count does not move — no `composerCopy*` function is added or
removed — but the `composerCopyHashEveryPathFor` row can no longer be driven by a
struct literal, because the predicate reads the PATHS.

```go file=gui/composer_copy_test.go mode=fragment
		{"composerCopyHashEveryPathPhrase", "H2-4.7", composerCopyHashEveryPathPhrase(),
			"HASH ON EVERY PATH Every way to spend this wallet needs a hashlock preimage. It is not on this device and not on these plates. Back up every phrase and its method, and every preimage plate, separately."},
		// H5 §2: the FOR row is driven through composerAnyPathByPhrase, so it
		// needs a state whose PATH carries a digest that is in the phrase set --
		// a bool literal no longer exists to set.
		{"composerCopyHashEveryPathFor", "H2-4.7", composerCopyHashEveryPathFor(composerStateByPhraseForCopyTable()),
			"HASH ON EVERY PATH Every way to spend this wallet needs a hashlock preimage. It is not on this device and not on these plates. Back up every phrase and its method, and every preimage plate, separately."},
	}
```

```go file=gui/composer_copy_test.go mode=fragment
// composerStateByPhraseForCopyTable is the smallest composition §8h's phrase
// form applies to: one path, whose hash is a digest the phrase set holds.
//
// It exists because H5 §2 replaced composerState.hashByPhrase with a value set
// plus a predicate over the CURRENT paths, so the table's row can no longer be
// driven by a struct literal. Building it here keeps composerCopyTable a table.
func composerStateByPhraseForCopyTable() *composerState {
	var d [32]byte
	for i := range d {
		d[i] = byte(i)
	}
	st := &composerState{list: md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{{Hash: &d}}}}
	composerNotePhraseDigest(st, d)
	return st
}
```

`composer_copy_test.go` gains one import for it:

```go file=gui/composer_copy_test.go mode=fragment
	"seedhammer.com/hashlock"
	"seedhammer.com/md"
)
```

- [ ] **Step 4: RED.**

Run: `go test -count=1 -run 'TestComposerPhraseRouteHoldsOnTheZeroValueState|TestComposerAnyPathByPhraseIsPerDigest|TestComposerHashEditToAPayloadRowDropsThePhraseForm|TestComposerMixedWalletBannerNamesEveryPhraseAndEveryPlate|TestRemovePathThenAHexHashDrawsThePlainBanner' ./gui/`

Measured:

```
gui/composer_provenance_test.go:35:8: st.phraseDigests undefined (type *composerState has no field or method phraseDigests)
gui/composer_provenance_test.go:70:17: st.phraseDigests undefined (type *composerState has no field or method phraseDigests)
gui/composer_provenance_test.go:71:80: st.phraseDigests undefined (type *composerState has no field or method phraseDigests)
gui/composer_provenance_test.go:73:6: undefined: composerAnyPathByPhrase
gui/composer_provenance_test.go:108:5: undefined: composerNotePhraseDigest
gui/composer_provenance_test.go:108:5: too many errors
FAIL	seedhammer.com/gui [build failed]
```

- [ ] **Step 5: The state.** Replace `composerState.hashByPhrase` (`gui/composer_state.go:35-38` at `b9a9a30`) with the set:

```go file=gui/composer_state.go mode=fragment
	// phraseDigests is the set of digests THIS COMPOSITION derived from a
	// phrase typed on the device (H5 §2, F-480). Done's §8h form names the
	// phrase and its method as the backup exactly when a path CURRENTLY carries
	// one of them (composerAnyPathByPhrase, composerCopyHashEveryPathFor).
	//
	// A SET OF VALUES, NOT A FLAG AND NOT AN INDEX. The predecessor was one
	// bool for the whole policy, so replacing a phrase-set hash with a payload
	// row while another path kept a hex hash left §8h naming a phrase the
	// composition no longer had. An index would be the C16 shape a second time
	// -- "Remove path" splices the slice, so an index is not an identity.
	//
	// NOTHING EVER DELETES FROM IT, and it cannot go stale for that reason: a
	// digest no path carries is simply never matched. Two paths sharing one
	// phrase digest are both by-phrase, and a path whose phrase digest is later
	// re-entered as 64 hex is STILL by-phrase -- the digest was derived here
	// once and the backup burden is unchanged.
	//
	// It is nil until the first HOLD; composerNotePhraseDigest allocates.
	phraseDigests map[[32]byte]struct{}
```

The two helpers go beside `composerEveryPathHashed`, ABOVE `composerPathLine`:

```go file=gui/composer_state.go mode=fragment
// composerNotePhraseDigest records that h was derived from a phrase typed on
// this device, in this composition (H5 §2).
//
// IT ALLOCATES, and that is why the insertion is a function rather than one
// line at the HOLD. composerState is built as a zero-value struct literal at
// its one production site (gui/composer_flow.go:34) and in every test in this
// package, so phraseDigests arrives nil -- and an assignment into a nil map
// panics. The panic would be on the machine, in the GUI goroutine, at the
// moment the operator holds to confirm a hash that gates funds.
func composerNotePhraseDigest(st *composerState, h [32]byte) {
	if st.phraseDigests == nil {
		st.phraseDigests = make(map[[32]byte]struct{})
	}
	st.phraseDigests[h] = struct{}{}
}

// composerAnyPathByPhrase is §8h's provenance condition (H5 §2, H2 §4.7): some
// path CURRENTLY in this composition carries a digest derived from a phrase.
//
// IT WALKS THE PATHS, which is the whole design. Every edit that changes a hash
// -- "Remove path", `No hash lock`, `Type 64 hex`, a payload row -- changes
// p.Hash, and this reads p.Hash, so none of them needs bookkeeping and the
// composerHashByPhraseSync that used to keep the old flag honest is gone with
// both of its call sites. A predicate that only asked whether the SET is
// non-empty would reproduce F-480 exactly.
//
// Reading a nil map is legal in Go, so the pre-first-HOLD state answers false
// without a special case.
func composerAnyPathByPhrase(st *composerState) bool {
	for _, p := range st.list.Paths {
		if p.Hash == nil {
			continue
		}
		if _, ok := st.phraseDigests[*p.Hash]; ok {
			return true
		}
	}
	return false
}
```

- [ ] **Step 6: HOLD notes the digest.** In `hashlockPhraseRoute` (`gui/composer_hashlock.go:68-70` at `b9a9a30`):

```go file=gui/composer_hashlock.go mode=fragment
				d := h
				st.list.Paths[idx].Hash = &d
				composerNotePhraseDigest(st, d)
```

`hashlockOtherPathLine`'s doc comment names the removed field and is corrected in the same edit:

```go file=gui/composer_hashlock.go mode=fragment
// It reads *p.Hash directly rather than the phrase set, so it is unaffected by
```

- [ ] **Step 7: Delete `composerHashByPhraseSync` and BOTH call sites.** In
`gui/composer_hash.go` the whole function (`:177-199`) goes, and
`composerHashEdit`'s doc comment records why nothing replaced it:

```go file=gui/composer_hash.go mode=fragment
// composerHashEdit sets or clears one path's hashlock.
//
// NO PROVENANCE BOOKKEEPING LIVES HERE, and its absence is H5 §2's fix rather
// than an omission. composerHashByPhraseSync used to run in the noneRow arm and
// in composerPathEdit's Remove arm to keep a composition-wide bool honest; with
// provenance held per digest (composerAnyPathByPhrase) every arm below simply
// writes p.Hash, and the predicate reads it.
```

```go file=gui/composer_hash.go mode=fragment
		case sel == rows.noneRow:
			st.list.Paths[idx].Hash = nil
			return true
```

And the Remove arm in `gui/composer_shape.go` (`:353-357` at `b9a9a30`) loses its
call and the comment that justified it:

```go file=gui/composer_shape.go mode=fragment
			composerApplyShapeEdit(st, func() {
				st.list.Paths = append(st.list.Paths[:idx], st.list.Paths[idx+1:]...)
			})
```

- [ ] **Step 8: The copy — the predicate, and §8h's sentence (spec §2 item 5).**

```go file=gui/composer_copy.go mode=fragment
// §8h, the phrase-route form (SPEC_hashlock_H2_device §4.7 as H5 §2 folds it).
// The reconciliation line lives in composerCopyHashlockReconcile instead; see
// there.
//
// "EVERY ... AND EVERY", NOT "THE ... OR THE" (H5 §2 item 5, journey I-3). This
// banner is drawn when EVERY path is hashed and at least one of those hashes
// came from a phrase -- which on a mixed wallet means one path needs the phrase
// and another needs a preimage plate, so BOTH backups are required, one per
// path. The shipped sentence offered a choice between them, and a choice is an
// undercount at the one screen whose job is to say what spending needs.
func composerCopyHashEveryPathPhrase() string {
	return "HASH ON EVERY PATH\n" +
		"Every way to spend this wallet needs a hashlock preimage. It is not on " +
		"this device and not on these plates. Back up every phrase and its " +
		"method, and every preimage plate, separately."
}

func composerCopyHashEveryPathFor(st *composerState) string {
	if composerAnyPathByPhrase(st) {
		return composerCopyHashEveryPathPhrase()
	}
	return composerCopyHashEveryPath()
}
```

- [ ] **Step 9: GREEN, then every mutation.**

Run: `go test -count=1 -run 'TestComposerPhraseRouteHoldsOnTheZeroValueState|TestComposerAnyPathByPhraseIsPerDigest|TestComposerHashEditToAPayloadRowDropsThePhraseForm|TestComposerMixedWalletBannerNamesEveryPhraseAndEveryPlate|TestRemovePathThenAHexHashDrawsThePlainBanner|TestComposerCopy|TestModals|TestHashlock' ./gui/`
Expected: `ok  	seedhammer.com/gui	38.162s` (measured).

| Mutation | Measured failure |
| --- | --- |
| `composerNotePhraseDigest` assigns without the nil check | `TestComposerPhraseRouteHoldsOnTheZeroValueState`: `panic: assignment to entry in nil map [recovered, repanicked]`, at `composer_provenance_test.go:66` through `sessionHarness.holdConfirm`. Not a failure — a PANIC, in the GUI goroutine, at the hold. |
| `composerAnyPathByPhrase` returns `len(st.phraseDigests) > 0` | four tests: `TestComposerHashEditDispatchesByRowLabel/none_row_clears…`: `the phrase form survived the last hash being cleared`; `TestRemovePathThenAHexHashDrawsThePlainBanner`: `the only phrase-set path was removed and the phrase form is still chosen`; `TestComposerAnyPathByPhraseIsPerDigest/the_phrase_path_was_edited_to_a_payload_row` and `…/the_phrase_path_was_removed`: `composerAnyPathByPhrase = true, want false`; `TestComposerHashEditToAPayloadRowDropsThePhraseForm`: `no path carries a phrase-derived digest and the predicate still says one does` **and** `§8h names a phrase this composition no longer has`. |
| delete `composerNotePhraseDigest(st, d)` from `hashlockPhraseRoute` | `TestHashlockReconcileScreenIsReachableOnAMixedPolicy`: `the phrase route did not record that this hash was set by phrase`; `TestComposerPhraseRouteHoldsOnTheZeroValueState`: `the anchor's digest is not in the phrase set (0 entries)` |
| restore `"Back up the phrase and its method, or the preimage plate, separately."` | `TestComposerCopyIsVerbatimFromTheSpec` on BOTH §8h rows, and `TestComposerMixedWalletBannerNamesEveryPhraseAndEveryPlate`: `does not carry "every phrase and its method"`, `does not carry "every preimage plate"`, `still offers a CHOICE of backups` |

- [ ] **Step 10: The whole `gui` package.**

Run: `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`
Measured: **1229 top-level tests, `partition verified exhaustive: 1229 == 1229`,
all 24 shards ok, 22 s wall.** `b9a9a30` has 1225 by the same count
(`go test ./gui/ -list '.*' | grep -cE '^(Test|Example|Fuzz)'`), so this task adds
the four tests of Step 1 and renames one.

- [ ] **Step 11: Commit.**

```bash
git add gui/composer_state.go gui/composer_hash.go gui/composer_shape.go gui/composer_hashlock.go gui/composer_copy.go gui/composer_provenance_test.go gui/composer_copy_test.go gui/composer_hashlock_test.go
git commit -s -m "composer: hash provenance per digest, not one flag per policy -- phraseDigests + composerAnyPathByPhrase; composerHashByPhraseSync and both call sites deleted; §8h names every phrase AND every plate (hashlock H5, F-480)"
```

---


### Task 2: The reconcile screen carries its operand (spec §1, F-487)

**Files:**
- Modify: `gui/composer_copy.go`, `gui/composer_hashlock.go`
- Modify (tests): `gui/composer_copy_test.go`, `gui/modal_fits_test.go`, `gui/composer_hashlock_test.go`

**Interfaces:**
- Changed: `composerCopyHashlockReconcile()` → `composerCopyHashlockReconcile(first8last8, method string, chars int) string`. Its ONE production call site is `hashlockPhraseRoute`'s post-HOLD `showError` (`gui/composer_hashlock.go:82` at `b9a9a30`); its test call sites are the copy table and the fit table.
- Consumes: `hashlockFirst8Last8(h [32]byte) string` (`gui/composer_hashlock.go:131`), `hashlockMethod.String()` (`:36-41`), `len(phrase)`.
- Unchanged signature, changed text: `composerCopyHashlockConfirm(first8last8, method string, chars int, relation, otherPath string) string`.

**The needle is kept deliberately.** The first sentence still contains
*"run ms hashlock with this phrase"* verbatim, because
`TestHashlockReconcileScreenIsReachableOnAMixedPolicy`
(`gui/composer_hashlock_test.go:882` at `b9a9a30`, whose needle is the
`h.mustReach` at `:909`) and the walk (`cmd/emu/walk_hashlock_phrase.js:318`)
both pump to it. Neither is edited by
this task, and both are run in Step 6.

- [ ] **Step 1: The tables (RED).** The copy table's reconcile row takes the
longest variant's arguments, and the confirm row carries the new write-down
sentence:

```go file=gui/composer_copy_test.go mode=fragment
		{"composerCopyHashlockReconcile", "H2-4.5", composerCopyHashlockReconcile("b867db87..edbc96cb", "hardened", 100),
			"hash  b867db87..edbc96cb method: hardened   chars: 100 " +
				"Before you fund this wallet, run ms hashlock with this phrase and method on the host and check the digest matches. " +
				"If they differ, do not fund this wallet: build it again."},
```

```go file=gui/composer_copy_test.go mode=fragment
				"Write down this phrase, the method and this digest now. They are not on this device and not on your plates. Without both, this path can never be spent. " +
				"One phrase per policy. Never use this phrase as a passphrase or a password anywhere else."},
```

The error-body fit row is measured at the longest content the body can carry:

```go file=gui/modal_fits_test.go mode=fragment
		{
			// The LONGEST variant: `hardened` is the longer method name and
			// `chars: 100` the widest count hashlock.PhraseMaxChars permits.
			"the hashlock reconciliation screen (H2 §4.5, H5 §1)",
			composerCopyHashlockReconcile("b867db87..edbc96cb", "hardened", 100),
		},
```

- [ ] **Step 2: The two new tests, and the write-down assertion the confirm-modal test gains.**

```go file=gui/composer_hashlock_test.go mode=fragment
			// H5 §1 item 2: the write-down instruction names the DIGEST too --
			// the operator is asked on the very next screen to compare it
			// against the host, and until now nothing told them to record it.
			// MUTATION: restore "Write down this phrase and the method now." ->
			// this fails.
			if !strings.Contains(normalizeDrawn(body), normalizeDrawn("phrase, the method and this digest")) {
				t.Errorf("the confirm modal's write-down line does not name the digest: %q", normalizeDrawn(body))
			}
```

```go file=gui/composer_hashlock_test.go mode=fragment
// H5 §1 (F-487): the reconcile screen carries what it asks the operator to
// compare.
//
// The screen says "check the digest matches" -- and the digest had just left
// the panel with the confirm modal, so the check was asked for at the one moment
// the operand was not on screen. It now repeats the token, the method and the
// character count, spelled exactly as the confirm modal spells them, and adds
// what to do when they differ.
//
// HARDENED, deliberately: `3cf5d421..b70a4c12` is a token no other row of this
// file produces, so a screen that echoed the SHA-256 trial's frame could not
// pass, and `method: hardened` is the longer of the two method words.
//
// MUTATIONS:
//   - return the old one-sentence body -> fails at the token.
//   - drop the mismatch sentence -> fails at "If they differ".
//   - pass m.String() where len(phrase) is wanted, or vice versa -> the
//     method/chars line assertion fails.
func TestHashlockReconcileScreenCarriesTheDigestMethodAndChars(t *testing.T) {
	st := composerStateWithPaths(t, 1)
	var ret bool
	h := runComposerHashEdit(t, st, composerSessionWith(nil, nil), 0, &ret)
	h.mustReach("Type a hashlock phrase")
	h.tapRow(0, 3)
	h.mustReach("32-byte value")
	h.tapNav(Button3)
	h.mustReach("Hashlock phrase")
	typeOnPassphraseKeyboard(t, h, hashlockAnchorPhrase)
	h.tapNav(Button3)
	h.mustReach("Which method?")
	h.tapRow(0, 2) // Hardened: 28 characters, so no §4.3a warning
	h.mustReach("Write down this phrase")
	h.holdConfirm()
	frame := h.mustReach("run ms hashlock with this phrase")
	for _, want := range []string{
		"hash  3cf5d421..b70a4c12",
		"method: hardened   chars: 28",
		"If they differ, do not fund this wallet: build it again.",
	} {
		if !strings.Contains(normalizeDrawn(frame), normalizeDrawn(want)) {
			t.Errorf("the reconcile screen does not carry %q.\nFrame: %q", want, normalizeDrawn(frame))
		}
	}
}

// The reconcile screen's first two lines are spelled the way the confirm modal
// spells them (H5 §1 item 1), so the operator compares two screens that read
// alike rather than two that merely mean alike.
//
// A GATE RATHER THAN A COMMENT: the two bodies are separate string literals in
// composer_copy.go -- deliberately, because every operator-facing body is a
// composerCopy* function the AST scan counts and the fit table measures, and a
// shared un-scanned helper would smuggle one past both -- so nothing but this
// keeps them from drifting apart.
//
// MUTATION: change the reconcile body's separator to a single space
// ("method: %s chars: %d") -> this fails.
func TestHashlockReconcileHeaderIsSpelledLikeTheConfirmModal(t *testing.T) {
	const tok, method, chars = "b867db87..edbc96cb", "sha256", 28
	rec := composerCopyHashlockReconcile(tok, method, chars)
	con := composerCopyHashlockConfirm(tok, method, chars, "", "")
	head := "hash  " + tok + "\nmethod: " + method + "   chars: 28\n"
	if !strings.HasPrefix(rec, head) {
		t.Errorf("the reconcile body does not open with the shared header:\n got: %q\nwant prefix: %q", rec, head)
	}
	if !strings.HasPrefix(con, head) {
		t.Errorf("the confirm body does not open with the shared header:\n got: %q\nwant prefix: %q", con, head)
	}
}
```

- [ ] **Step 3: RED.**

Run: `go test -count=1 -run 'TestHashlockReconcileScreenCarriesTheDigestMethodAndChars|TestHashlockReconcileHeaderIsSpelledLikeTheConfirmModal' ./gui/`

Measured:

```
gui/composer_hashlock_test.go:992:39: too many arguments in call to composerCopyHashlockReconcile
	have (string, string, number)
	want ()
gui/modal_fits_test.go:344:34: too many arguments in call to composerCopyHashlockReconcile
	have (string, string, number)
	want ()
FAIL	seedhammer.com/gui [build failed]
```

- [ ] **Step 4: The copy.** The confirm modal's write-down sentence, and the
headroom claim its neighbour got wrong (spec §6 records: the fork comment claimed
186; the confirm body's measured headroom is 107):

```go file=gui/composer_copy.go mode=fragment
// THE HEADROOM NUMBER, CORRECTED (H5 §6 records; tests M-1 = journey N-1). The
// comment on composerCopyHashlockReconcile used to claim this body's measured
// headroom was 186; it is 107, and it was 107 before H5 touched it. The number
// that is true is logged by TestConfirmScreensThisBlockTouchesAreDrawnInFull on
// every run, which is why no literal is asserted here -- headroom is a LINE
// budget, not a character budget (modal_fits_test.go), so H5 §1's longer
// write-down sentence adds no line and does not move it.
```

```go file=gui/composer_copy.go mode=fragment
		"Write down this phrase, the method and this digest now. They are not on " +
		"this device and not on your plates. Without both, this path can never be spent.\n" +
```

Then the reconcile body itself, replacing the one-sentence version:

```go file=gui/composer_copy.go mode=fragment
// §4.5's reconciliation screen, drawn right after HOLD for every phrase-set
// hash.
//
// §4.5's drop-order step 2 says to move this line into the phrase-route §8h at
// Done, and the build gate did -- but §8h is guarded by composerEveryPathHashed
// (composer_state.go at the fork baseline c4a64fc), so on the ordinary
// wallet with one keyed path and one
// hashlocked path it was drawn NOWHERE (r0 adversarial I-1 = fidelity I-2 =
// journey I-3, all three tracing the same loss). Its own screen after HOLD is
// reachable for every policy that has a phrase-set hash.
//
// IT CARRIES THE OPERAND IT ASKS ABOUT (H5 §1, F-487). "Check the digest
// matches" was asked one frame AFTER the confirm modal took the digest off the
// panel, so the operator was told to compare against something no longer on
// screen. The token, the method and the character count come back here, spelled
// exactly as the confirm modal spells them
// (TestHashlockReconcileHeaderIsSpelledLikeTheConfirmModal), and `chars: <n>` is
// H2 §4.5's reconciliation field arriving at the moment of reconciliation --
// it is the one signal that shows a stray space against the host card's
// phrase_chars.
//
// AND IT SAYS WHAT A MISMATCH MEANS. A divergence found here is a path that
// could never have been spent; the remedy is to build the policy again, before
// it is funded, and not to fund it and hope.
//
// Measured on errorScreenBody at sh2DisplaySize, longest variant (`hardened`,
// `chars: 100`): see the row in TestModalsThisBlockTouchesAreDrawnInFull.
func composerCopyHashlockReconcile(first8last8, method string, chars int) string {
	return "hash  " + first8last8 + "\n" +
		fmt.Sprintf("method: %s   chars: %d", method, chars) + "\n" +
		"Before you fund this wallet, run ms hashlock with this phrase and " +
		"method on the host and check the digest matches. If they differ, do " +
		"not fund this wallet: build it again."
}
```

- [ ] **Step 5: The call site.** `hashlockPhraseRoute` passes the three values it already holds:

```go file=gui/composer_hashlock.go mode=fragment
				showError(ctx, th, "Hash lock",
					composerCopyHashlockReconcile(hashlockFirst8Last8(h), m.String(), len(phrase)))
```

- [ ] **Step 6: GREEN, the measured fits, then every mutation.**

Run: `go test -count=1 -run 'TestHashlockReconcile|TestHashlockPhraseRouteSetsTheCorpusDigest|TestComposerCopy|TestModals|TestConfirmScreens' ./gui/`
Expected: `ok  	seedhammer.com/gui	8.827s` (measured). The fit gates log their
numbers; measured on this tree at `sh2DisplaySize`:

```
modal_fits_test.go:352: the hashlock reconciliation screen (H2 §4.5, H5 §1): 186 chars drawn in full, headroom 339 chars (margin 80)
modal_fits_test.go:352: HASH ON EVERY PATH, phrase-route form (H2 §4.7): 165 chars drawn in full, headroom 378 chars (margin 80)
modal_fits_test.go:395: the hashlock confirm modal, longest variant (H2 §4.5): 347 chars drawn in full, headroom 107 chars (margin 80)
```

Those are spec §1.1's 186/339, spec §2.5's 165/378 and spec §1.2's 347/107 —
reproduced independently here, not copied from the spec.

| Mutation | Measured failure |
| --- | --- |
| return the old one-sentence reconcile body | `TestHashlockReconcileScreenCarriesTheDigestMethodAndChars`: `does not carry "hash  3cf5d421..b70a4c12"`, `does not carry "method: hardened   chars: 28"`, `does not carry "If they differ, …"`; and `TestHashlockReconcileHeaderIsSpelledLikeTheConfirmModal`: `the reconcile body does not open with the shared header` |
| drop the mismatch sentence only | `TestHashlockReconcileScreenCarriesTheDigestMethodAndChars`: `does not carry "If they differ, do not fund this wallet: build it again."`; `TestComposerCopyIsVerbatimFromTheSpec`: `composerCopyHashlockReconcile (SPEC §H2-4.5) does not match the spec.` |
| restore `"Write down this phrase and the method now."` | `TestHashlockPhraseRouteSetsTheCorpusDigest`, BOTH cases: `the confirm modal's write-down line does not name the digest: "hash3cf5d421..b70a4c12method:hardenedchars:28writedownthisphraseandthemethodnow.…"`; and `TestComposerCopyIsVerbatimFromTheSpec` on the confirm row |
| `"method: %s chars: %d"` — one space instead of three | `TestHashlockReconcileHeaderIsSpelledLikeTheConfirmModal`: `the reconcile body does not open with the shared header`. **`TestHashlockReconcileScreenCarriesTheDigestMethodAndChars` stays GREEN under it** — measured — because `normalizeDrawn` strips whitespace before comparing, so no frame assertion in this package can see a spacing change. That is exactly why the header-equality test exists as a separate gate. |

- [ ] **Step 7: The whole `gui` package.**

Run: `/scratch/code/shibboleth/mnemonic-engrave/scripts/gui-shard-test.sh ./gui/ 24`
Measured: **1231 top-level tests, `partition verified exhaustive: 1231 == 1231`,
all 24 shards ok, 32 s wall** — Task 1's 1229 plus this task's two.

- [ ] **Step 8: Commit.**

```bash
git add gui/composer_copy.go gui/composer_hashlock.go gui/composer_copy_test.go gui/modal_fits_test.go gui/composer_hashlock_test.go
git commit -s -m "composer: the reconcile screen carries the digest, the method and the char count it asks the operator to compare, and says what a mismatch means; the write-down line names the digest (hashlock H5, F-487)"
```

---


### Task 3: The phrase screen's lead inside the page band (spec §3, F-484)

**Files:**
- Modify: `gui/composer_paged.go`, `gui/passphrase_keyboard.go`, `gui/composer_hashlock.go`
- Create: `gui/composer_hashlock_geometry_test.go`

**Interfaces:**
- Produces: `composerTextBand(dims image.Point) (left, width int)` in `gui/composer_paged.go`; `hashlockPhraseLead(ctx *Context, th *Colors, dims image.Point, top int) (op.Op, image.Point)` in `gui/composer_hashlock.go`; package-level `const ppReadoutGap = 8` in `gui/passphrase_keyboard.go` (was a function-local `const readoutGap` at `:448`).
- Consumes: `widget.Labelw(&ctx.B, ctx.Styles.lead, width int, col, text) (op.Op, image.Point)`; `assets.NavBtnPrimary.Bounds().Size()`; `inkUnderNavOps(t, dims, []op.Op) (image.Rectangle, image.Point, bool)` and `navButtonRects(dims image.Point) []image.Rectangle` (`gui/composer_paged_geometry_test.go:42,74`); `PassphraseKeyboard.MaxHeight`, `.page`, `.size [4]image.Point`.

**Which branch of §3.3 was taken: NEITHER — the copy is unchanged.** §3.3 fires
only if the lead exceeds two lines in the narrower band. Measured on the wired
tree at `sh2DisplaySize`:

```
composer_hashlock_geometry_test.go:68: band left=8 width=411; lead (407,44) = 2 line(s) of 23 px
```

Two lines at 411 px, as it was at 464 px, so the fallback copy is NOT used and
H2 §4.2 is not folded. The line count is asserted by the gate, not by this
paragraph.

- [ ] **Step 1: Factor the band out of `composerPageLines`.** The arithmetic
already existed inline (`gui/composer_paged.go:88-91` at `b9a9a30`); a second
screen needs it, and a copy would be a second answer to "where does text stop".

```go file=gui/composer_paged.go mode=fragment
// composerTextBand is the ONE horizontal band composer text wraps inside: the
// panel, less the navigation column at the right edge and the same 8 px margin
// on the left (W-3, and see composerPageLines below for what centring on the
// whole panel cost).
//
// FACTORED OUT BECAUSE A SECOND SCREEN NEEDED IT (H5 §3, F-484). The hashlock
// phrase screen wrapped its lead at `dims.X - 2*8` and centred it on the panel,
// so 152 px of its ink landed inside the Back button's rectangle -- W-3's defect
// on a screen W-3's fix did not reach. A copy of the arithmetic there would be a
// second answer to "where does text stop", and the two would drift; a shared
// function cannot.
//
// It returns (left, width) rather than (left, right) because every caller wraps
// to the width and then centres inside it.
func composerTextBand(dims image.Point) (left, width int) {
	const bandMargin = 8
	left = bandMargin
	right := dims.X - assets.NavBtnPrimary.Bounds().Size().X - bandMargin
	return left, right - left
}
```

`composerPageLines` then uses it, and derives `bandRight` from it rather than recomputing:

```go file=gui/composer_paged.go mode=fragment
	bandLeft, lineWidth := composerTextBand(dims)
	bandRight := bandLeft + lineWidth
```

- [ ] **Step 2: Hoist `readoutGap` to package level** so §3.2(b)'s gate divides by
the SAME number `PassphraseKeyboard.Layout` divides by, rather than a copy of it:

```go file=gui/passphrase_keyboard.go mode=fragment
// ppReadoutGap separates the masked readout from the key grid. See its use in
// Layout: the budget the readout is clamped to is MaxHeight - grid - this.
const ppReadoutGap = 8
```

```go file=gui/passphrase_keyboard.go mode=fragment
	// ppReadoutGap, package level since H5 §3: the phrase screen's geometry gate
	// measures the readout budget with the SAME number Layout divides by, rather
	// than a copy of it.
	// Clamp the readout to the height that actually exists, keeping the TAIL:
	// the tail is what was just typed, and it is the end of a passphrase that
	// an operator is checking as they enter it. Dropping the head is safe only
	// because the n/100 counter -- which this clamp is what keeps visible --
	// reports the true length, and the confirm screen shows the value whole.
	if k.MaxHeight > 0 {
		avail := k.MaxHeight - k.size[k.page].Y - ppReadoutGap
```

```go file=gui/passphrase_keyboard.go mode=fragment
	gridY := readoutSz.Y + ppReadoutGap
```

- [ ] **Step 3: The lead.** A named function, so the gate rasterises what
production draws rather than its own arithmetic:

```go file=gui/composer_hashlock.go mode=fragment
// hashlockPhraseLead lays out the phrase screen's lead INSIDE the composer's
// text band, positioned at y = top (H5 §3, F-484).
//
// IT USED TO WRAP AT `dims.X - 2*8` AND CENTRE ON THE WHOLE PANEL, which is
// exactly the layout W-3 removed from composerPageLines: the lead's band
// overlaps the Back button's row, so 152 px of its ink was drawn inside that
// button's rectangle. No glyph or chip was lost -- the ink was in the button's
// empty margin -- but the margin is what keeps a glyph from sitting flush
// against a control it is not part of, and that margin was spent.
//
// A SEPARATE FUNCTION SO THE GATE MEASURES WHAT PRODUCTION DRAWS. The geometry
// test rasterises the op this returns; a test that re-derived the layout would
// pass on its own arithmetic rather than on the screen's
// (composer_paged_geometry_test.go's own split makes the same point).
func hashlockPhraseLead(ctx *Context, th *Colors, dims image.Point, top int) (op.Op, image.Point) {
	left, width := composerTextBand(dims)
	lbl, sz := widget.Labelw(&ctx.B, ctx.Styles.lead, width, th.Text,
		composerCopyHashlockPhraseLead())
	return lbl.Offset(image.Pt(left+(width-sz.X)/2, top)), sz
}
```

The call site in `hashlockPhraseFlow` replaces the panel-wide `Labelw` and the
`leadBand.N(leadSz)` offset (`gui/composer_hashlock.go:169-172` at `b9a9a30`):

```go file=gui/composer_hashlock.go mode=fragment
		leadOp, leadSz := hashlockPhraseLead(ctx, th, dims, content.Min.Y)
		_, content = content.CutTop(leadSz.Y)
```

- [ ] **Step 4: The geometry gate.** Create `gui/composer_hashlock_geometry_test.go`:

```go file=gui/composer_hashlock_geometry_test.go mode=whole
package gui

import (
	"image"
	"testing"

	"seedhammer.com/gui/layout"
	"seedhammer.com/gui/op"
	"seedhammer.com/gui/widget"
)

// ─── H5 §3 (F-484): the phrase screen's lead stays inside the page band ──────
//
// W-3 is the whole argument, one screen further on. composerPageLines wrapped at
// `dims.X - 2*8` and centred on the WHOLE panel while the navigation column sits
// at `dims.X - NavBtnPrimary.width`, so text was drawn under a button and the
// operator lost its tail. hashlockPhraseFlow's lead was still laid out that way:
// measured 152 px of its ink inside the Back button's rectangle. Nothing was
// LOST -- the ink fell in the button's empty margin, not on its chip or glyph --
// which is precisely why no text assertion and no screenshot review found it.
//
// A GEOMETRY TEST, for W-3's own reason: op.Drawer.ExtractText collects a
// glyph's rune wherever it lands, under a button included, so every existing
// assertion about this screen passes either way.

// hashlockPhraseLeadTop is the y the production flow lays the lead at: the panel
// less the title band (hashlockPhraseFlow's `screen.CutTop(leadingSize)`).
func hashlockPhraseLeadTop(dims image.Point) int {
	screen := layout.Rectangle{Max: dims}
	_, content := screen.CutTop(leadingSize)
	return content.Min.Y
}

// TestHashlockPhraseLeadIsDrawnInsideTheBand is §3.2 (a) and (c).
//
// MUTATION: restore the panel-wide layout in hashlockPhraseLead
// (`widget.Labelw(..., dims.X-2*8, ...)` centred on the panel) -> (a) fails,
// naming the Back button and the pixel it was hit at.
// MUTATION: composerTextBand returning `dims.X` for the width -> (a) fails.
func TestHashlockPhraseLeadIsDrawnInsideTheBand(t *testing.T) {
	p := newPlatform()
	p.display = sh2DisplaySize
	ctx := NewContext(p)
	dims := sh2DisplaySize

	leadOp, leadSz := hashlockPhraseLead(ctx, &descriptorTheme, dims, hashlockPhraseLeadTop(dims))

	// (a) no lead ink inside any navigation button rectangle. Only the lead is
	// rasterised, so any ink found there is lead ink by construction -- the
	// buttons are not drawn into this buffer.
	if nav, at, hit := inkUnderNavOps(t, dims, []op.Op{leadOp}); hit {
		t.Errorf("the phrase screen's lead is drawn UNDER a navigation button.\n"+
			"  button %v received ink at %v\n"+
			"The operator cannot read what a button covers, and ExtractText collects "+
			"the runes anyway -- which is why every text assertion on this screen "+
			"passed while 152 px of the lead sat inside Back (F-484, W-3).", nav, at)
	}

	// (c) at most two lines. The single-line height is MEASURED at the same
	// style and band width rather than hardcoded, so this stays true if the
	// face or the band changes.
	left, width := composerTextBand(dims)
	_, one := widget.Labelw(&ctx.B, ctx.Styles.lead, width, descriptorTheme.Text, "X")
	if one.Y <= 0 {
		t.Fatalf("a one-line label measured %v; this test cannot count lines", one)
	}
	lines := (leadSz.Y + one.Y - 1) / one.Y
	t.Logf("band left=%d width=%d; lead %v = %d line(s) of %d px", left, width, leadSz, lines, one.Y)
	if lines > 2 {
		t.Errorf("the lead wraps to %d lines in the %d px band, over §3.2(c)'s two. "+
			"§3.3's fallback copy applies: \"This screen does the hashing. Use a phrase "+
			"you have never used anywhere else.\", with H2 §4.2 folded to it.", lines, width)
	}
}

// TestHashlockPhraseLeadGeometryProbeCanSeeInk is the mutation proof for the
// scanner, at THIS screen's band.
//
// composer_paged_geometry_test.go proves inkUnderNavOps can see ink; this proves
// the same for the y the lead is actually drawn at, which is the row the Back
// button occupies. Without it, a gate above that found nothing would be
// indistinguishable from a gate looking at the wrong band.
func TestHashlockPhraseLeadGeometryProbeCanSeeInk(t *testing.T) {
	p := newPlatform()
	p.display = sh2DisplaySize
	ctx := NewContext(p)
	dims := sh2DisplaySize
	navs := navButtonRects(dims)

	lbl, sz := widget.Label(&ctx.B, ctx.Styles.lead, descriptorTheme.Text, "XXXX")
	under := navs[0].Min.Add(image.Pt(6, 6))
	if _, at, hit := inkUnderNavOps(t, dims, []op.Op{lbl.Offset(under)}); !hit {
		t.Fatalf("the scanner found no ink for a %v label drawn at %v, inside button %v -- "+
			"so §3.2(a) above is looking at nothing", sz, under, navs[0])
	} else {
		t.Logf("scanner sees ink at %v", at)
	}
	// The lead's own top y, at the LEFT margin, must not be reported: this is
	// the negative control that says the scanner reads the button rectangles
	// rather than the whole row the lead sits on.
	if _, _, hit := inkUnderNavOps(t, dims, []op.Op{lbl.Offset(image.Pt(8, hashlockPhraseLeadTop(dims)))}); hit {
		t.Error("the scanner reports ink under a button for a label at the left margin")
	}
}

// TestHashlockPhraseScreenKeepsTheReadoutBudget is §3.2 (b): F-481 must not
// regress.
//
// The lead's height decides how much of the panel is left for the keyboard, and
// the keyboard's readout is what is clamped away when that runs short -- an 8 px
// CutBottom once left the budget at 11 px, one line needs 19, and every typed
// character vanished while the `show` key stayed live. A NARROWER band can make
// the lead taller, so the screen that fixes F-484 is exactly the screen that
// could re-break F-481.
//
// Measured on the LIVE screen: kbd.MaxHeight is what hashlockPhraseFlow set on
// the frame, and kbd.size[page] is the grid the keyboard built, so this is the
// budget PassphraseKeyboard.Layout actually divided by, not a re-derivation.
//
// MUTATION: restore `content, _ = content.CutBottom(8)` in hashlockPhraseFlow ->
// the budget drops below one line and this fails.
func TestHashlockPhraseScreenKeepsTheReadoutBudget(t *testing.T) {
	st := composerStateWithPaths(t, 1)
	var ret bool
	h := runComposerHashEdit(t, st, composerSessionWith(nil, nil), 0, &ret)
	h.mustReach("Type a hashlock phrase")
	h.tapRow(0, 3)
	h.mustReach("32-byte value")
	h.tapNav(Button3)
	h.mustReach("Hashlock phrase")
	typeOnPassphraseKeyboard(t, h, "abc")
	h.mustReach("3/100")

	kbd, ok := hashlockKbdFor[h]
	if !ok {
		t.Fatal("no *PassphraseKeyboard was registered: this test measured nothing")
	}
	if kbd.MaxHeight <= 0 {
		t.Fatalf("the phrase screen left the keyboard UNBOUNDED (MaxHeight=%d); the "+
			"readout is free to grow over the counter and the title", kbd.MaxHeight)
	}
	grid := kbd.size[kbd.page]
	budget := kbd.MaxHeight - grid.Y - ppReadoutGap
	_, one := widget.Labelw(&h.ctx.B, h.ctx.Styles.word, grid.X, descriptorTheme.Text, "*")
	t.Logf("MaxHeight=%d grid=%v gap=%d -> readout budget %d px; one line is %d px",
		kbd.MaxHeight, grid, ppReadoutGap, budget, one.Y)
	if budget < one.Y {
		t.Errorf("the readout budget is %d px and one line needs %d: PassphraseKeyboard.Layout "+
			"clamps every rune away, so nothing is masked, nothing is revealed, and the "+
			"`show` key is a dead control (F-481)", budget, one.Y)
	}
}
```

- [ ] **Step 5: RED, GREEN, and the mutations.**

The pre-fix state IS the first mutation, so RED and the mutation table are the
same evidence. With the panel-wide layout restored in `hashlockPhraseLead`:

```
--- FAIL: TestHashlockPhraseLeadIsDrawnInsideTheBand (0.00s)
    composer_hashlock_geometry_test.go:52: the phrase screen's lead is drawn UNDER a navigation button.
          button (427,44)-(480,97) received ink at (431,52)
        The operator cannot read what a button covers, and ExtractText collects the runes anyway -- which is why every text assertion on this screen passed while 152 px of the lead sat inside Back (F-484, W-3).
    composer_hashlock_geometry_test.go:68: band left=8 width=411; lead (440,44) = 2 line(s) of 23 px
```

Note the lead's measured width under the mutation: 440 px against the band's 411.

Run: `go test -count=1 -run 'TestHashlockPhraseLead|TestHashlockPhraseScreen|TestComposerPaged|TestPassphraseKeyboard|TestTextKeyboard|TestComposerPickTouch|TestFreetext' ./gui/`
Expected: `ok  	seedhammer.com/gui	1.110s` (measured).

| Mutation | Measured failure |
| --- | --- |
| `hashlockPhraseLead` wraps at `dims.X-2*8` and centres on the panel | as quoted above: ink at `(431,52)` inside button `(427,44)-(480,97)`, the lead measuring 440 px wide |
| `composerTextBand`'s `right` drops the nav column (`dims.X - bandMargin`) | `TestHashlockPhraseLeadIsDrawnInsideTheBand`: the lead is drawn under a button — **and** `TestComposerPagedLinesNeverDrawUnderTheNavButtons` on `keyed stub` pages 0 and 1 and `keyless stub` page 0, which is W-3's own gate confirming the shared function is the one both screens use |
| restore `content, _ = content.CutBottom(8)` in `hashlockPhraseFlow` (F-481's original defect) | `TestHashlockPhraseScreenKeepsTheReadoutBudget`: `MaxHeight=201 grid=(340,182) gap=8 -> readout budget 11 px; one line is 19 px` then `the readout budget is 11 px and one line needs 19: … the `show` key is a dead control (F-481)`; and `TestHashlockPhraseScreenDrawsTheMaskedReadout`: `the phrase screen drew 0 asterisks for 10 typed characters` |

Unmutated, §3.2(b) measures `MaxHeight=209 grid=(340,182) gap=8 -> readout budget
19 px; one line is 19 px`. **The budget equals one line exactly** — the narrower
band did not change it (the lead is two lines at both widths, 44 px tall either
way), and there is no slack. Any future edit that adds a pixel above the keyboard
turns this gate red, which is the intent.

- [ ] **Step 6: Commit.**

```bash
git add gui/composer_paged.go gui/passphrase_keyboard.go gui/composer_hashlock.go gui/composer_hashlock_geometry_test.go
git commit -s -m "composer: the hashlock phrase lead wraps inside the page band, not under Back -- composerTextBand shared with composerPageLines, with a raster geometry gate and F-481's readout budget asserted (hashlock H5, F-484)"
```

---

### Task 4: The unlock refusal says what to do next (spec §5, F-488)

**Files:**
- Modify: `gui/unlock_kdf.go`, `gui/unlock_preimage_test.go`

**Interfaces:**
- Changed text, unchanged signature: `unlockNotPermittedBody(e *seal.RecordNotPermittedError) string` (`gui/unlock_kdf.go:390-393` at `b9a9a30`).
- Consumes: `seal.RecordNotPermittedError{Index int, Class seal.Classification, Section, Preimage bool}`; `unlockRecordNoun(e)` (`gui/unlock_kdf.go:404-424`), whose longest arm is `"not a format this machine reads"`.
- The body has NO `composerCopy*` row, because it is not composer copy: its fit gate is the `assertModalBodyFits(t, tc.name, errorScreenBody, body)` call inside `TestUnlockNotPermittedBodyNamesTheRecordAndTheKind` (`gui/unlock_preimage_test.go:139` at `b9a9a30`), and this task adds the longest-noun row to that table. `gui/s6b_p7_modal_fit_sweep_test.go` covers the codex32-too-long refusal, a different body (`gui/unlock_kdf.go:502` at `b9a9a30`; that sweep's row LABEL still reads `unlock_kdf.go:448`, which is stale and not this stage's to fix), and is not touched.

- [ ] **Step 1: The tests (RED).** The flow-level assertions:

```go file=gui/unlock_preimage_test.go mode=fragment
	// H5 §5 (F-488): the refusal says what to do next. Naming the record and
	// the kind leaves the operator holding an intact payload and no route.
	// MUTATION: drop the new sentence -> this fails.
	if !uiContains(got, "Remove that record") {
		t.Errorf("the screen must say what to do next; got %q", got)
	}
	// MUTATION: drop "(records count from 0)" -> this fails. The index is
	// 0-based (seal/record.go:69) and the device said so nowhere; once the
	// number is an instruction to DELETE, a 1-based reading deletes the record
	// above the plate -- in this fixture's own blob, the seed at record 0.
	if !uiContains(got, "records count from 0") {
		t.Errorf("the screen must say the index is 0-based; got %q", got)
	}
```

The table's three existing rows gain the sentence, and a fourth row is added for
the fit measurement at the longest noun and a two-digit index:

```go file=gui/unlock_preimage_test.go mode=fragment
			[]string{"Record 1", "hashlock preimage", "not a seed", "Nothing was opened",
				"Remove that record (records count from 0) on the host and seal the payload again."},
```

```go file=gui/unlock_preimage_test.go mode=fragment
		{
			// H5 §5's fit row: the LONGEST noun this body can carry
			// ("not a format this machine reads") at a two-digit index, so
			// assertModalBodyFits measures the widest arm rather than the
			// first one.
			"the longest noun at a two-digit index",
			&seal.RecordNotPermittedError{Index: 13, Class: seal.ClassUnknown, Section: seal.SectionEncrypted},
			[]string{"Record 13", "not a format this machine reads",
				"Remove that record (records count from 0) on the host and seal the payload again."},
			[]string{"hashlock preimage"},
		},
```

Run: `go test -count=1 -run 'TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable|TestUnlockNotPermittedBodyNamesTheRecordAndTheKind' ./gui/`

Measured RED:

```
--- FAIL: TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable (0.18s)
    unlock_preimage_test.go:69: the screen must say what to do next; got "Record1isahashlockpreimage,notaseed.Thispayloadcannotbeunlockedhere.Nothingwasopened.SealedPayload"
    unlock_preimage_test.go:76: the screen must say the index is 0-based; got "Record1isahashlockpreimage,notaseed.…"
--- FAIL: TestUnlockNotPermittedBodyNamesTheRecordAndTheKind (0.37s)
    … body "Record 13 is not a format this machine reads. This payload cannot be unlocked here. Nothing was opened." does not carry "Remove that record (records count from 0) on the host and seal the payload again."
```

- [ ] **Step 2: The body.**

```go file=gui/unlock_kdf.go mode=fragment
// AND IT SAYS WHAT TO DO NEXT (H5 §5, F-488). Naming the record and the kind
// left the operator holding an intact payload, a ~31 s derivation already spent,
// and no route: the machine cannot edit a sealed payload, so the only way
// forward is the host.
//
// "(RECORDS COUNT FROM 0)" IS LOAD-BEARING, not a parenthetical. The index is
// 0-based (seal/record.go:69) and the device said so nowhere, while `ms` says it
// throughout; the moment the number becomes an instruction to DELETE, a 1-based
// reading removes the record ABOVE the plate -- which in this package's own
// fixture (gui/unlock_preimage_test.go's blob, the plate at record 1) is a seed.
//
// The body is shared by every unlockRecordNoun arm below, so this reaches all of
// them; the fit is measured at the longest noun and a two-digit index in
// TestUnlockNotPermittedBodyNamesTheRecordAndTheKind.
func unlockNotPermittedBody(e *seal.RecordNotPermittedError) string {
	return fmt.Sprintf("Record %d is %s. This payload cannot be unlocked here. "+
		"Nothing was opened. Remove that record (records count from 0) on the "+
		"host and seal the payload again.",
		e.Index, unlockRecordNoun(e))
}
```

- [ ] **Step 3: GREEN, the measured fits, and the mutations.**

Run: `go test -count=1 -v -run 'TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable|TestUnlockNotPermittedBodyNamesTheRecordAndTheKind' ./gui/`
Measured:

```
--- PASS: TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable (0.10s)
    unlock_preimage_test.go:166: a preimage plate at record 1: 152 chars drawn in full, headroom 397 chars (margin 80)
    unlock_preimage_test.go:166: a preimage plate at record 0 -- records count from 0: 152 chars drawn in full, headroom 397 chars (margin 80)
    unlock_preimage_test.go:166: a codex32 secret in the public section: 140 chars drawn in full, headroom 418 chars (margin 80)
    unlock_preimage_test.go:166: the longest noun at a two-digit index: 153 chars drawn in full, headroom 397 chars (margin 80)
    unlock_preimage_test.go:166: a record this machine does not read at all: 152 chars drawn in full, headroom 397 chars (margin 80)
--- PASS: TestUnlockNotPermittedBodyNamesTheRecordAndTheKind (0.16s)
ok  	seedhammer.com/gui	0.272s
```

397 at the longest noun and a two-digit index, which is spec §5's number
reproduced here.

| Mutation | Measured failure |
| --- | --- |
| drop the whole new sentence | `TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable`: `the screen must say what to do next` and `the screen must say the index is 0-based`; `TestUnlockNotPermittedBodyNamesTheRecordAndTheKind`: all four rows `does not carry "Remove that record (records count from 0) …"` |
| drop `"(records count from 0)"` only | `TestUnlockNamesARefusedPreimageInsteadOfCallingItUnreadable`: `the screen must say the index is 0-based; got "…Removethatrecordonthehostandsealthepayloadagain.SealedPayload"`; and all four table rows |

- [ ] **Step 4: Commit.**

```bash
git add gui/unlock_kdf.go gui/unlock_preimage_test.go
git commit -s -m "unlock: the record refusal says what to do next -- remove that record (records count from 0) on the host and seal again; 0-based stated because the number is now an instruction to delete (hashlock H5, F-488)"
```

---


### Task 5: The walk proves hold order and stored-versus-displayed (spec §4, F-485)

**Files:**
- Create: `gui/composer_state_hook.go`, `gui/composer_state_hook_tinygo.go`, `gui/composer_state_hook_test.go`, `cmd/emu/composer_js.go`
- Modify: `gui/composer_flow.go`, `gui/tinygo_split_test.go`, `cmd/emu/platform.go`, `cmd/emu/walk_hashlock_phrase.js`, `cmd/emu/needle_test.go`

**Interfaces:**
- Produces (`!tinygo` only): `var composerStateHook func() []*[32]byte`; `setComposerStateHook(st *composerState)`; `clearComposerStateHook()`; **exported** `ComposerPathHashes() []*[32]byte`.
- Produces (both builds): `composerFlowExit(st *composerState)` in the untagged `gui/composer_flow.go`, which runs `st.reg.scrub()` and `clearComposerStateHook()`.
- Produces (`js` only): `installComposerAPI()`, publishing `window.shComposerPathHashes()`.
- Consumes: `gui.ComposerPathHashes` from `cmd/emu`; `encoding/hex.EncodeToString`; `syscall/js.FuncOf`; the existing `installScreenAPI`/`installWalkAPI` call site (`cmd/emu/platform.go:93-96`).
- Walk consumes: `window.shComposerPathHashes()`, alongside the existing `shScreen`, `shTargets`, `shTap`, `shPress`, `shRelease`, `shSysw`.

**The seam is the package's FOURTH `//go:build` pair** (after `plate_hook`,
`frame_hook`, `engraved_hook`) and the FIRST that is not an interface hook, which
`gui/tinygo_split_test.go` must be told about — Step 5.

- [ ] **Step 1: The seam.** Create `gui/composer_state_hook.go`:

```go file=gui/composer_state_hook.go mode=whole
//go:build !tinygo

// The composition-state seam, and the FOURTH build-tagged pair in this package.
// plate_hook.go states the general argument for the split, frame_hook.go the
// measurement discipline around it, and engraved_hook.go what a hook may
// announce; read those first, because everything they say applies here.
//
// WHAT IT IS FOR (H5 §4, F-485). cmd/emu's walk of the hashlock phrase route
// asserted the tokens the SCREEN drew and nothing about what the composition
// STORED, so two defects passed it: a hash assigned before the hold-to-confirm,
// and a stored digest that differs from the displayed one. Both are caught by
// the gui tests in CI; neither was caught by the gate the stage closes on, and a
// walk that cannot see the difference between "the screen says d" and "the
// policy holds d" is asserting the weaker of the two claims at the moment funds
// depend on the stronger.
//
// composerState is a LOCAL of composerFlow (gui/composer_flow.go:34) with no
// path out of this package, which is why a hook is needed at all: there is no
// accessor to add, no field to export, and giving the state a package-level home
// to make it readable would be a far larger change than the seam.
//
// WHAT A WALK MAY DO WITH IT, stated once and normatively: READ, to assert that
// what the screen shows equals what is stored. It never DRIVES through this
// hook. The driving primitives are window.shTap and its siblings (cmd/emu/walk_js.go),
// which inject the events a finger would; anything that let a walk reach past a
// screen would make the walk prove less than the operator's own hands do, which
// is the opposite of the point.
//
// WHY `!tinygo` AND NOT AN EXPORTED ACCESSOR. The same rule frame_hook.go
// applies: the consumer is JavaScript on a page, outside anything Go can wipe,
// so the firmware must not merely decline to use this, it must not carry it.
// What travels here is a set of 32-byte digests -- public values, and by H2 §4's
// design the preimage never leaves the stack -- but the rule is structural
// rather than a judgement about this payload, and a structural rule that is
// relaxed once for a value that seemed harmless is not a rule.
//
// WHAT IT COSTS, MEASURED: see composer_state_hook_tinygo.go.
package gui

// composerStateHook reports each spend path's hash for the composition that is
// running NOW, in path order, nil where a path carries none.
//
// nil except while composerFlow is running: it is installed at the top of the
// flow and cleared when the flow returns, so a consumer that calls it from the
// start screen gets nil rather than the last composition's digests. A stale
// answer is worse than no answer here -- a walk asserting "path 1 holds no hash
// yet" would pass on a previous run's cleared state.
var composerStateHook func() []*[32]byte

// setComposerStateHook installs read access to st for the composition's
// lifetime. Paired with clearComposerStateHook by composerFlow's defer, on the
// same construction the seed scrub uses there, so every exit -- a Back, a
// refusal, a ctx.Done unwind, a panic -- clears it.
//
// The closure COPIES each digest rather than handing out st's pointers: the
// caller is JavaScript, the state is live, and a *[32]byte into an md.SpendPath
// would let a consumer write the policy this hook exists to observe.
func setComposerStateHook(st *composerState) {
	composerStateHook = func() []*[32]byte {
		out := make([]*[32]byte, len(st.list.Paths))
		for i, p := range st.list.Paths {
			if p.Hash == nil {
				continue
			}
			d := *p.Hash
			out[i] = &d
		}
		return out
	}
}

func clearComposerStateHook() {
	composerStateHook = nil
}

// ComposerPathHashes is the consumer's entry point: each path's hash for the
// running composition, or nil when none is running.
//
// Exported because cmd/emu is a different package; it exists only in this
// build-tagged file, so the firmware has nothing to export.
func ComposerPathHashes() []*[32]byte {
	if composerStateHook == nil {
		return nil
	}
	return composerStateHook()
}
```

And its twin, `gui/composer_state_hook_tinygo.go`. **Its numbers are Step 7's,
measured; do not paste them from another hook's file.**

```go file=gui/composer_state_hook_tinygo.go mode=whole
//go:build tinygo

package gui

// setComposerStateHook and clearComposerStateHook do nothing on the machine,
// and composerStateHook and ComposerPathHashes do not exist here.
//
// See composer_state_hook.go for why the firmware carries none of them: what
// they would hand over is the composition the operator is building, and the
// only consumer for it lives in a browser. A variable the image does not
// contain cannot be assigned by accident.
//
// WHAT IT COSTS, MEASURED: nothing. Not one byte. Built at the production
// settings (-target pico-plus2 -stack-size 16kb -gc precise -opt 2
// -scheduler tasks) on the H5 gate tree, against the SAME tree with this file
// and composerFlow's hook lines deleted -- so the number is the hook's own
// share and not a delta inherited from frame_hook's measurement, which is a
// different call in a different place and was measured on a different day:
//
//	with the hook, one defer                1,599,164 B flash / 62,856 B RAM
//	the hook deleted from the tinygo view   1,599,164 B flash / 62,856 B RAM
//
// AND THE ZERO IS NOT AN ARTEFACT OF A BUILD THAT IGNORED THE EDIT. Giving this
// stub a body the compiler cannot drop -- one println -- moves the image to
// 1,599,388 B, +224 B. Edits here reach the image; this one costs nothing.
//
// WHAT IT COST BEFORE THE SHAPE WAS FIXED: 112 B. composerFlow first cleared the
// hook through a SECOND `defer clearComposerStateHook()` beside the seed
// scrub's own defer, and that measured 1,599,276 B -- TinyGo elides the empty
// call and not the defer record around it. Folding both into the one deferred
// composerFlowExit call the flow already had is what makes it free. Measured,
// because a guess about the compiler in plate_hook_tinygo.go's first version
// was wrong, and this file exists to say what the number IS.
//
// The numbers are recorded in IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md
// Task 5 as well, with the fork baseline they are a delta against.
func setComposerStateHook(*composerState) {}

func clearComposerStateHook() {}
```

- [ ] **Step 2: One defer, not two.** `composerFlow` installs the hook and leaves
through the deferred call it already had. Writing a second
`defer clearComposerStateHook()` beside `defer st.reg.scrub()` compiles and works
and costs **112 B of firmware flash** (Step 7) — TinyGo elides the empty stub's
CALL but not the defer record around it.

```go file=gui/composer_flow.go mode=fragment
// composerFlowExit is everything one composition must undo, in one deferred
// call: the seed scrub that has always run here, and H5 §4's composition-state
// hook.
//
// ONE DEFER, DELIBERATELY, and measured: a second `defer clearComposerStateHook()`
// costs 112 B of firmware flash against this shape's 0, because TinyGo removes
// the empty stub's CALL but not the defer bookkeeping around it. Both numbers
// are in IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md Task 5.
func composerFlowExit(st *composerState) {
	st.reg.scrub()
	clearComposerStateHook()
}
```

```go file=gui/composer_flow.go mode=fragment
	st := &composerState{reg: &seedRegistry{}, bound: composerBoundFrom(ctx.sysw)}
	// THE COMPOSITION-STATE SEAM IS INSTALLED HERE TOO (H5 §4, F-485), and it
	// leaves through the SAME defer the scrub does -- composerFlowExit -- rather
	// than through a second one. Two reasons, and the second is why it is
	// written this way: every exit below is covered without an implementer
	// remembering to add a clear to a new return, and on the machine the shape
	// costs nothing. A `defer clearComposerStateHook()` beside the scrub's own
	// defer measured +112 B of flash even though the tinygo stub is empty --
	// TinyGo elides the empty CALL and not the defer record around it. One
	// defer, as there has always been, is 0.
	setComposerStateHook(st)
	defer composerFlowExit(st)
```

- [ ] **Step 3: The seam's own gate.** The walk cannot run in CI, so the
properties it depends on are gated in `gui`. Create `gui/composer_state_hook_test.go`:

```go file=gui/composer_state_hook_test.go mode=whole
package gui

import (
	"testing"
	"testing/synctest"

	"seedhammer.com/md"
)

// ─── H5 §4 (F-485): the composition-state seam ──────────────────────────────
//
// cmd/emu's walk is the gate this seam exists for, and a walk cannot run in CI.
// So the properties it depends on are gated HERE: the hook is installed while a
// composition runs and nil otherwise, it reports p.Hash per path, and what it
// hands back cannot be written through.

// TestComposerStateHookIsInstalledOnlyWhileAFlowRuns is the lifetime property.
//
// A hook left installed after composerFlow returns would answer with the LAST
// composition's digests, and a walk asserting "path 1 holds no hash yet" would
// pass on a previous run's cleared state -- the stale-answer failure that is
// worse than no answer at all.
//
// MUTATION: delete `defer clearComposerStateHook()` from composerFlow -> the
// after-the-flow assertion fails.
// MUTATION: delete `setComposerStateHook(st)` from composerFlow -> the
// during-the-flow assertion fails (ComposerPathHashes returns nil).
func TestComposerStateHookIsInstalledOnlyWhileAFlowRuns(t *testing.T) {
	synctest.Test(t, func(t *testing.T) {
		if got := ComposerPathHashes(); got != nil {
			t.Fatalf("the hook is installed before any composition ran: %v", got)
		}
		p := newPlatform()
		p.display = sh2DisplaySize
		ctx := NewContext(p)
		ctx.sysw = composerSessionWith(nil, nil)

		done := false
		frame, quit := runUI(ctx, func() {
			composerFlow(ctx, &descriptorTheme)
			done = true
		})
		defer quit()

		if got, ok := pumpUntil(frame, "Which script?", 24); !ok {
			t.Fatalf("the wrapper picker never drew.\nLast frame: %q", got)
		}
		// Inside the flow: a composition with no paths yet is an EMPTY slice,
		// which is not the same answer as "no composition is running".
		hashes := ComposerPathHashes()
		if hashes == nil {
			t.Fatal("the hook is not installed while composerFlow is running")
		}
		if len(hashes) != 0 {
			t.Fatalf("a composition with no paths reports %d hash(es)", len(hashes))
		}

		// Back out of the wrapper picker: composerFlow returns and the deferred
		// clear runs.
		click(&ctx.Router, Button1)
		for i := 0; i < 64 && !done; i++ {
			if _, ok := frame(); !ok {
				break
			}
		}
		if !done {
			t.Fatal("composerFlow never returned, so the clear was never reached")
		}
		if got := ComposerPathHashes(); got != nil {
			t.Fatalf("the hook survived the composition it was installed for: %v", got)
		}
	})
}

// TestComposerStateHookReportsEachPathAndHandsOutCopies is the read contract.
//
// The copy half is the one that matters: the consumer is JavaScript on a page,
// and a *[32]byte into an md.SpendPath would let a walk WRITE the policy it
// exists to observe -- a reading seam that can drive is not a reading seam.
//
// MUTATION: return st's own pointers (`out[i] = p.Hash`) -> the write-through
// assertion fails.
// MUTATION: report only the paths that carry a hash (skip the nil entries
// instead of leaving a hole) -> the index alignment assertion fails, and the
// walk's "path 0 holds nothing yet" read would silently become "some path".
func TestComposerStateHookReportsEachPathAndHandsOutCopies(t *testing.T) {
	var d [32]byte
	for i := range d {
		d[i] = byte(i)
	}
	st := &composerState{list: md.PathList{Wrapper: md.ComposeWsh, Paths: []md.SpendPath{
		{}, {Hash: &d},
	}}}
	setComposerStateHook(st)
	t.Cleanup(clearComposerStateHook)

	got := ComposerPathHashes()
	if len(got) != 2 {
		t.Fatalf("the hook reports %d entries for a 2-path composition", len(got))
	}
	if got[0] != nil {
		t.Errorf("path 1 carries no hash and the hook reports %x", *got[0])
	}
	if got[1] == nil || *got[1] != d {
		t.Fatalf("path 2's hash is %v, want %x", got[1], d)
	}
	// Write through the reported pointer; the policy must not move.
	//
	// `want` is a SNAPSHOT, and it is load-bearing: st.list.Paths[1].Hash is
	// &d, so a hook that handed out st's own pointer would have this write
	// change `d` as well, and comparing the policy against `d` would compare a
	// variable with itself and pass. Measured -- the mutation below was GREEN
	// against `!= d` and is RED against `!= want`.
	want := d
	got[1][0] ^= 0xff
	if *st.list.Paths[1].Hash != want {
		t.Errorf("writing through the hook's pointer changed the POLICY: %x, want %x",
			*st.list.Paths[1].Hash, want)
	}
}
```

- [ ] **Step 4: The `js` glue.** Create `cmd/emu/composer_js.go`:

```go file=cmd/emu/composer_js.go mode=whole
//go:build js

package main

import (
	"encoding/hex"
	"syscall/js"

	"seedhammer.com/gui"
)

// installComposerAPI exposes the running composition's stored path hashes to the
// page as window.shComposerPathHashes.
//
//	shComposerPathHashes()   [ "<64 hex>" | null, ... ]  one entry per spend
//	                         path, in path order, null where a path carries no
//	                         hash; null (not an array) when no composition is
//	                         running.
//
// WHY A WALK NEEDS IT (H5 §4, F-485). Every other reading primitive here reports
// what was DRAWN. That is the right default -- a walk is evidence about the
// screen an operator sees -- but the hashlock phrase route ends by writing a
// digest into the policy, and "the confirm modal displayed b867db87..edbc96cb"
// and "the policy now holds b867db87..." are different claims. Two defects lived
// in the gap: a hash assigned BEFORE the hold-to-confirm (so Back after reading
// the digest left it set), and a stored digest that differs from the displayed
// one. Both are caught by CI's gui tests; the walk that closes the stage saw
// neither.
//
// READING ONLY, AND THAT IS NOT A CONVENTION. gui.ComposerPathHashes hands back
// COPIES of the digests (gui/composer_state_hook.go), so there is nothing here
// to write through. Driving stays with shTap and its siblings, which inject the
// events a finger would.
//
// FULL 64 HEX, not the first8..last8 the screens draw: the point of the call is
// to compare what is stored against what was shown, and comparing an
// abbreviation against an abbreviation would accept 2^192 wrong digests.
func installComposerAPI() {
	js.Global().Set("shComposerPathHashes", js.FuncOf(func(js.Value, []js.Value) any {
		hashes := gui.ComposerPathHashes()
		if hashes == nil {
			// No composition is running. Distinguishable from a composition
			// with no paths, which is an empty ARRAY -- a walk that could not
			// tell those apart would read "the flow is not running" as "no path
			// holds a hash" and pass on a screen it never reached.
			return nil
		}
		out := make([]any, 0, len(hashes))
		for _, h := range hashes {
			if h == nil {
				out = append(out, nil)
				continue
			}
			out = append(out, hex.EncodeToString(h[:]))
		}
		return out
	}))
}
```

Installed beside the other three APIs (`cmd/emu/platform.go:93-96` at `b9a9a30`):

```go file=cmd/emu/platform.go mode=fragment
	installWalkAPI(p)
	installScreenAPI(p.screen)
	installComposerAPI()
```

- [ ] **Step 5: Tell the split gate this pair is not an interface hook.**
`TestBuildTaggedHooksAreAbsentFromTheFirmwareImage` discovers `//go:build` pairs
from the tree and requires each host file to declare an exported interface. This
one declares a variable and a function instead, so the gate fails with
`composer_state_hook.go declares no exported interface`. Its own comment says what
to do: *"If a //go:build pair is ever legitimately not an interface hook, this is a
deliberate edit and not a stale list: the check has to be told, out loud."* Told —
and NOT exempted, because a named pair still goes through the "used outside its
owning file" scan and its stub is still required to export nothing:

```go file=gui/tinygo_split_test.go mode=fragment
// nonInterfaceHookPairs names every //go:build pair in gui whose host file
// carries no exported INTERFACE, with the reason. It is the "say so out loud"
// the per-pair check below demands, and it is deliberately a map keyed by the
// host filename so a pair that stops being an exception fails loudly when the
// entry is left behind.
//
// A pair named here is NOT exempt from the scan: its exported functions and
// variables go into the same owner map, and its stub is required to export
// nothing at all. Only the "declares an exported interface" shape is waived.
var nonInterfaceHookPairs = map[string]string{
	"composer_state_hook.go": "H5 §4 (F-485): the composition-state seam is a package " +
		"variable plus an exported reader, not an interface a Platform implements -- what it " +
		"reports is a LOCAL of composerFlow, so there is no object for cmd/emu to implement " +
		"anything on. The property this test protects is unchanged and is checked below: " +
		"ComposerPathHashes may be named in no other gui file, and composer_state_hook_tinygo.go " +
		"exports nothing.",
}
```

```go file=gui/tinygo_split_test.go mode=fragment
		if found == 0 {
			why, told := nonInterfaceHookPairs[p.host]
			if !told {
				t.Errorf("%s declares no exported interface, so nothing about it is checked below -- "+
					"if this pair is not an interface hook, say so here rather than leaving the "+
					"scan silently vacuous", p.host)
				continue
			}
			t.Logf("%s: %s", p.host, why)
			// TOLD IS NOT EXCUSED. A pair that carries no interface still has a
			// host-only surface the firmware must not contain, so its exported
			// declarations go into the same owner map and through the same
			// "used outside its owning file" scan below. Without this the
			// exemption would be a hole: naming a pair here would remove it from
			// every check rather than from one.
			for _, decl := range f.Decls {
				switch d := decl.(type) {
				case *ast.FuncDecl:
					if d.Recv == nil && d.Name.IsExported() {
						owner[d.Name.Name] = p.host
						found++
					}
				case *ast.GenDecl:
					for _, spec := range d.Specs {
						vs, ok := spec.(*ast.ValueSpec)
						if !ok {
							continue
						}
						for _, n := range vs.Names {
							if n.IsExported() {
								owner[n.Name] = p.host
								found++
							}
						}
					}
				}
			}
			if found == 0 {
				t.Errorf("%s is named in nonInterfaceHookPairs but exports nothing at all, so the "+
					"scan below still checks nothing for it", p.host)
			}
			// And the stub may export nothing whatever its shape: the rule the
			// interface check states ("a tinygo-tagged file is firmware") is
			// about the image, not about the Go kind of the declaration.
			sf, err := parser.ParseFile(fset, p.stub, nil, 0)
			if err != nil {
				t.Fatalf("parsing %s: %v", p.stub, err)
			}
			for _, decl := range sf.Decls {
				switch d := decl.(type) {
				case *ast.FuncDecl:
					if d.Recv == nil && d.Name.IsExported() {
						t.Errorf("%s exports %s -- that file IS the firmware, so the host-only "+
							"surface of this pair is in the image", p.stub, d.Name.Name)
					}
				case *ast.GenDecl:
					for _, spec := range d.Specs {
						vs, ok := spec.(*ast.ValueSpec)
						if !ok {
							continue
						}
						for _, n := range vs.Names {
							if n.IsExported() {
								t.Errorf("%s exports %s -- that file IS the firmware", p.stub, n.Name)
							}
						}
					}
				}
			}
		}
```

- [ ] **Step 6: The `ok`-shape guard, which is RED at `b9a9a30`.**

`TestWalkOkContainsNoDriverSuppliedPlateCount` (`cmd/emu/needle_test.go`) reads
`ok` with `okExprRe = ^\s*ok:.*?\n  \};`, an object-literal property. Two walks
ASSIGN instead, so the guard reports INCONCLUSIVE — a `t.Errorf` — for both, and
`CGO_ENABLED=0 go test ./...` has been failing since `45f3d4c`. Measured on the
PRISTINE fork checkout, before any H5 edit:

```
$ cd /scratch/code/shibboleth/seedhammer && CGO_ENABLED=0 go test -count=1 ./cmd/emu/
--- FAIL: TestWalkOkContainsNoDriverSuppliedPlateCount (0.00s)
    needle_test.go:525: INCONCLUSIVE: walk_h0_preimage.js has no `ok:` property this test can read, so nothing was checked for it — the walk's return shape changed and this guard did not
    needle_test.go:525: INCONCLUSIVE: walk_hashlock_phrase.js has no `ok:` property this test can read, so nothing was checked for it — the walk's return shape changed and this guard did not
    needle_test.go:563: 6 walk script(s) checked; no driver-supplied plate count in any `ok`
FAIL
FAIL	seedhammer.com/cmd/emu	1.094s
```

Spec §4.4 requires this walk's `ok` to be SET rather than recomputed, so the
guard has to read the assignment shape whatever else H5 does. Teach it both, and
treat a bare boolean right-hand side as the STRONGEST form of the property rather
than as an exemption from it:

```go file=cmd/emu/needle_test.go mode=fragment
// Blind spot, stated: this reads the `ok` expression textually. A driver that
// computed `ok` into a variable first, or that shipped a helper named something
// else, would slip past. It costs one grep and catches the shape that has now
// occurred twice.
//
// TWO SHAPES, BECAUSE THIS TEST WAS RED AT FORK MAIN b9a9a30 AND HAD BEEN SINCE
// H0 (found by the H5 plan's build gate, 2026-09-05). Only the object-literal
// property `ok: <expr>,` was readable, so every walk that instead ASSIGNS --
// `out.ok = <expr>;`, which walk_h0_preimage.js has done since 45f3d4c and
// walk_hashlock_phrase.js since e1bf137 -- reported INCONCLUSIVE, and
// INCONCLUSIVE here is a t.Errorf. CI runs `go test ./...`, so the package has
// been failing for two stages while the guard's own doc claimed it covered
// "BOTH walk scripts".
//
// The assignment regex captures the right-hand side EXACTLY, anchored on the
// `.ok =` it is looking for, so unlike the property span it cannot grab a
// neighbouring literal -- which is why the census/verdict floor below is
// required of the property shape and not of this one.
var (
	okPropRe   = regexp.MustCompile(`(?ms)^\s*ok:.*?\n  \};`)
	okAssignRe = regexp.MustCompile(`(?ms)^\s*\w+\.ok\s*=\s*(.*?);\s*$`)
	// A bare boolean right-hand side: `out.ok = true;`. This is the STRONGEST
	// form of the property under test, not an exemption from it -- an `ok` that
	// is SET after the last assertion contains no term at all, so it cannot
	// contain one the driver supplied, and there is nothing left for the
	// `plates` check to find. H5 §4.4 requires exactly this of the hashlock
	// walk, and a guard that called the strongest shape INCONCLUSIVE would push
	// the next author back to a recomputation.
	okSetRe = regexp.MustCompile(`^(true|false)$`)
)
```

```go file=cmd/emu/needle_test.go mode=fragment
		src := string(b)
		// The ASSIGNMENT shape first: its span is exact, so it needs no floor.
		if m := okAssignRe.FindStringSubmatch(src); m != nil {
			rhs := strings.TrimSpace(m[1])
			checked++
			if okSetRe.MatchString(rhs) {
				t.Logf("%s sets `ok` to %s after its last assertion, so it restates nothing "+
					"(H5 §4.4)", f, rhs)
				continue
			}
			if strings.Contains(rhs, "plates") {
				t.Errorf("%s's `ok` contains `plates`, which the CALLER supplies (I-1/F-170):\n%s\n"+
					"A walk cannot derive, so a caller-supplied count in `ok` is content the walk "+
					"never observed — a run that cut N WRONG strings is green.", f, rhs)
			}
			continue
		}
		expr := okPropRe.FindString(src)
		if expr == "" {
			t.Errorf("INCONCLUSIVE: %s has neither an `ok:` property nor an `x.ok =` assignment "+
				"this test can read, so nothing was checked for it — the walk's return shape "+
				"changed and this guard did not", f)
			continue
		}
```

- [ ] **Step 7: The walk.** `cmd/emu/walk_hashlock_phrase.js`, whole:

```javascript file=cmd/emu/walk_hashlock_phrase.js mode=whole
// H2 acceptance walk (IMPLEMENTATION_PLAN_hashlock_H2_device.md Task 5 Step 1):
// a hashlock phrase typed on the machine derives the digest ms hashlock derives
// on the host.
//
//   const w = await import("./walk_hashlock_phrase.js");
//   await w.run();
//
// NOT loaded by index.html: this drives the machine. It composes a policy and
// assigns a hash on the last trial, and a page that starts driving because
// somebody opened it is a trap.
//
// THE ORACLE IS THE CORPUS, NEVER THIS FILE'S ARITHMETIC. Every expected value
// below is a CONSTANT copied from ms-codec 0.8.0's own vectors, vendored at
// hashlock/testdata/hashlock-v0.8.json and pinned by
// hashlock/testdata/hashlock-v0.8.provenance.json (sha256
// a46c197a...11d30, mnemonic-secret cd0a60f). Nothing here recomputes a
// digest -- a walk that derived its own expectation would agree with a wrong
// device. The corpus is not fetchable from this page (it is outside the served
// cmd/emu directory), so each row names the corpus field it was copied from.
//
// Four trials, all through ONE `Which hash?` screen: Back at the phrase screen
// drops the phrase and returns there (spec §4.6), so the next trial starts
// clean without leaving the composer.
//
//   1. typed     "correct horse battery staple", SHA-256  -> derivation[0].sha256_h
//   2. control   "correct horse battery stapl"  (one char short), SHA-256
//                -> must NOT show trial 1's digest. This is what makes the
//                   walk falsifiable: without it, a screen that ignored the
//                   typed bytes entirely would still "pass" three times.
//   3. mixed     "Correct Horse Battery Staple", SHA-256  -> the mixed-case
//                   row's sha256_h. A screen that lowercased, trimmed or
//                   otherwise normalised the phrase (spec §2 forbids all of
//                   it) shows trial 1's digest here instead.
//   4. hardened  "correct horse battery staple", Hardened -> derivation[0]
//                   .hardened_h, after the countdown; then HOLD, so the digest
//                   is actually assigned and the reconciliation screen (§4.5)
//                   is reached.
//
// WHAT THE SCREEN SAYS IS NOT WHAT THE POLICY HOLDS (H5 §4, F-485). Trials 1-4
// above assert DISPLAYED tokens, and until this revision that was the whole
// walk -- so two defects passed it: a hash assigned BEFORE the hold-to-confirm
// (Back after reading the digest would have left it set), and a stored digest
// that differs from the displayed one. Both are red in CI's gui tests; neither
// was visible to the gate the stage closes on.
//
// So trial 4 also reads window.shComposerPathHashes(), the composition-state
// seam (gui/composer_state_hook.go, cmd/emu/composer_js.go):
//
//   * with the confirm modal UP and before the hold, the edited path's hash is
//     `null` -- the assignment has not happened yet. The read is pinned to that
//     frame rather than taken "some time before the hold", because a read taken
//     earlier passes trivially and proves nothing about the ORDER.
//   * after the hold, it is the corpus's FULL 64-hex hardened digest, and its
//     first8..last8 is the token the confirm modal displayed. Full hex on both
//     sides: comparing one abbreviation against another would accept 2^192
//     wrong digests.
//   * the reconciliation screen carries that same token AND the same
//     `chars: <n>` the confirm modal carried (§1.5).
//
// IT ONLY EVER READS. Driving stays with shTap/shPress/shRelease, which inject
// the events a finger would; a walk that reached past a screen would prove less
// than the operator's own hands do.
//
// ROW PICKING STAYS BY INDEX, and F-485's note about it is answered rather than
// deferred. chooseRow(i, expect, label) taps the i-th rectangle shTargets
// reports and then ASSERTS WHERE IT LANDED, so a moved row fails at the landing
// assertion with the screen it reached. A label pick is not available: shTargets
// returns bare rectangles because frameTargets drops the tag on purpose
// (cmd/emu/screen.go), so picking by label would need a second gui seam for no
// safety the landing assertion does not already give.
//
// Helpers are inlined from walk_h0_preimage.js / shots_composer.js (they are
// not exported there).
//
// THE KEYBOARD GRID WAS PROBED, NOT DERIVED (2026-09-05, this emulator build).
// window.shTargets() is no help here: it hit-tests only the CENTRE COLUMN, and
// on the 10-key `qwertyuiop` row x=240 lands in the 8 px gap BETWEEN two keys,
// so that row is missing from its output entirely. The phrase screen has no
// usable readout either -- hashlockPhraseFlow gives the keyboard a MaxHeight
// that leaves no room for one, so PassphraseKeyboard.Layout clamps it away and
// `show` reveals nothing to read a character back from.
//
// So the grid was measured by TAPPING: x swept in 4 px steps across each row
// with the n/100 counter as the oracle (a tap that increments it hit a key; one
// that does not fell in a gap). The bands come out as
//
//   y=152  qwertyuiop  centres  86 120 154 188 222 256 290 324 358 392
//   y=198  asdfghjkl   centres 103 137 171 205 239 273 307 341 375
//   y=244  zxcvbnm     centres 137 171 205 239 273 307 341
//
// -- a uniform 34 px pitch (26 px of it live, 8 px dead) with every row centred
// on x=239, i.e. key j of an n-key row at 239 - 17(n-1) + 34j. The page-cycle
// and reveal keys were found the same way, tapping along y=290.
//
// None of that is what makes the mapping trustworthy: the DIGEST is. One
// mistyped character changes sha256 completely, so trial 1 landing on
// b867db87..edbc96cb is a 28-character proof that every press hit the key this
// file believes it did -- and trial 2 proves the assertion can fail.

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const squash = (s) => String(s).replace(/\s+/g, "");
const BACK = [453, 70];
const CONFIRM = [453, 249];
const CAROUSEL_NEXT = [455, 160];

// ── the corpus constants (hashlock/testdata/hashlock-v0.8.json) ──────────────
const ANCHOR = "correct horse battery staple";        // derivation[0].phrase
const ANCHOR_SHA_H = "b867db87..edbc96cb";            // derivation[0].sha256_h, first8..last8
const ANCHOR_HARD_H = "3cf5d421..b70a4c12";           // derivation[0].hardened_h, first8..last8
// The same row's hardened_h WHOLE, for the stored-versus-displayed comparison.
// Copied from the corpus, never recomputed here.
const ANCHOR_HARD_FULL =
  "3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12";
const MIXED = "Correct Horse Battery Staple";         // the mixed-case derivation row's phrase
const MIXED_SHA_H = "95d44470..2297a7ff";             // that row's sha256_h, first8..last8
const CONTROL = "correct horse battery stapl";        // NOT a corpus row: one character short

// ── the keyboard ────────────────────────────────────────────────────────────
const PP_PAGES = [
  ["qwertyuiop", "asdfghjkl", "zxcvbnm"],
  ["QWERTYUIOP", "ASDFGHJKL", "ZXCVBNM"],
];
const PP_ROW_Y = [152, 198, 244];
const PP_FN_Y = 290;
const PP_PAGE_CYCLE = [130, PP_FN_Y];   // "ABC" / "?123" / "#+=" / "abc"
const PP_SPACE = [216, PP_FN_Y];        // page 0's `space` key (shTargets: x=177 w=78)
const PP_PITCH = 34;
const PP_NPAGES = 4;                    // lower, upper, symbols, symbols2

/** Where key j of an n-key row sits: rows are centred on x=239 at a 34px pitch. */
const ppKeyX = (n, j) => 239 - 17 * (n - 1) + PP_PITCH * j;

/** [page, x, y] for a character this walk types, or null if it is not on a letter page. */
function ppKeyPoint(ch) {
  for (let p = 0; p < PP_PAGES.length; p++) {
    for (let r = 0; r < PP_PAGES[p].length; r++) {
      const j = PP_PAGES[p][r].indexOf(ch);
      if (j >= 0) return [p, ppKeyX(PP_PAGES[p][r].length, j), PP_ROW_Y[r]];
    }
  }
  if (ch === " ") return [0, PP_SPACE[0], PP_SPACE[1]];
  throw new Error(`no key for ${JSON.stringify(ch)} on the pages this walk drives`);
}

const tap = async ([x, y], settle = 250) => { window.shTap(x, y); await sleep(settle); };

async function waitFor(needle, timeoutMs = 20000) {
  const want = squash(needle);
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    const text = window.shScreen();
    if (squash(text).includes(want)) return text;
    if (Date.now() >= deadline) {
      throw new Error(`waitFor(${JSON.stringify(needle)}) timed out after ${timeoutMs}ms; screen reads ${JSON.stringify(text)}`);
    }
    await sleep(50);
  }
}

function must(text, needle, why) {
  if (!squash(text).includes(squash(needle))) {
    throw new Error(`${why}: the screen does not carry ${JSON.stringify(needle)}.\nScreen: ${JSON.stringify(text)}`);
  }
}

/** The first of `needles` to appear -- for a screen that may be gone before it is polled. */
async function raceFor(needles, timeoutMs = 20000) {
  const deadline = Date.now() + timeoutMs;
  for (;;) {
    const text = squash(window.shScreen());
    for (const n of needles) if (text.includes(squash(n))) return n;
    if (Date.now() >= deadline) {
      throw new Error(`none of ${JSON.stringify(needles)} appeared within ${timeoutMs}ms; screen reads ${JSON.stringify(window.shScreen())}`);
    }
    await sleep(50);
  }
}

function mustNot(text, needle, why) {
  if (squash(text).includes(squash(needle))) {
    throw new Error(`${why}: the screen carries ${JSON.stringify(needle)} and must not.\nScreen: ${JSON.stringify(text)}`);
  }
}

async function goTo(program, max = 14) {
  const want = squash(program);
  for (let i = 0; i < max; i++) {
    if (squash(window.shScreen()).startsWith(want)) return i;
    await tap(CAROUSEL_NEXT, 200);
  }
  throw new Error(`goTo(${program}) never arrived; screen reads ${JSON.stringify(window.shScreen())}`);
}

/**
 * Select row `i` of the frame on screen NOW and take it.
 *
 * The coordinates are READ from window.shTargets(), not derived: it reports the
 * hit regions op.Drawer recorded, so this taps where a finger would land and
 * cannot tap a row that is not drawn. `expect` is not politeness -- a tap on
 * the wrong row picks a different parameter and the flow carries on happily.
 */
async function chooseRow(i, expect, label, settle = 350) {
  if (typeof window.shTargets !== "function") {
    throw new Error("shTargets is missing -- STALE emu.wasm. The browser caches it and a " +
      "cache-buster on index.html does not help; serve on a FRESH port.");
  }
  const targets = window.shTargets();
  if (i >= targets.length) {
    throw new Error(`choosing ${label}: the frame offers ${targets.length} tappable row(s), so row ${i} ` +
      `cannot be reached BY TOUCH.\nScreen: ${JSON.stringify(window.shScreen())}`);
  }
  await tap([targets[i].cx, targets[i].cy], 300);
  await tap(CONFIRM, settle);
  if (expect === null) return;
  try {
    await waitFor(expect);
  } catch (e) {
    throw new Error(`choosing ${label} (row ${i} of ${targets.length}) did not land on ${JSON.stringify(expect)}: ${e.message}`);
  }
}

/**
 * The hold-to-confirm gesture, with an explicit RELEASE.
 *
 * The release is load-bearing, for the same reason the gui harness's own
 * holdConfirm documents: the event router tracks ONE pointer contact, so a
 * second hold with no release in between routes to the FIRST screen's defunct
 * clickable and never leaves 0%. This walk holds three screens in sequence.
 */
async function hold([x, y], ms = 1800) {
  window.shPress(x, y);
  await sleep(ms);
  window.shRelease(x, y);
  await sleep(400);
}

/** Type `s` on the passphrase keyboard, cycling pages by touch as needed. */
async function typePhrase(s) {
  let page = 0;
  for (const ch of s) {
    const [want, x, y] = ppKeyPoint(ch);
    for (let n = 0; page !== want; n++) {
      if (n > PP_NPAGES) throw new Error(`the keyboard never reached page ${want}`);
      await tap(PP_PAGE_CYCLE, 120);
      page = (page + 1) % PP_NPAGES;
    }
    await tap([x, y], 80);
  }
  await waitFor(`${s.length}/100`);
}

/**
 * One trial, from `Which hash?` and back to it.
 *
 * method is "sha256" or "hardened". Returns {modal, firstFrame}: the confirm
 * modal's text, and which of the derivation screen / the modal appeared first.
 * The caller asserts on it; nothing here knows what a right answer looks like.
 *
 * firstFrame is a RACE and not an assertion, deliberately. The hardened
 * countdown is ~10 s of real PBKDF2 on the SH2 and effectively instant in
 * wasm, so on this emulator the confirm modal is usually already up by the
 * time the next poll runs. Demanding "Deriving" here would make the walk fail
 * on a device that is merely fast -- a timing assertion dressed as a
 * behavioural one. `TestHashlockDeriveKeepsAwakeUnderTheScreensaver` is what
 * gates that screen, in CI, on a clock the test controls.
 */
async function trial(phrase, method) {
  await waitFor("Type a hashlock phrase");
  await chooseRow(0, "32-byte value", "Type a hashlock phrase");   // the §8i rule modal
  await tap(CONFIRM, 500);
  await waitFor("Hashlock phrase");
  await typePhrase(phrase);
  await tap(CONFIRM, 500);                                          // OK
  await waitFor("Which method?");
  if (method === "sha256") {
    await chooseRow(1, "brainwallet", "SHA-256");                   // §4.3b, always warns
    await hold(CONFIRM);
  } else {
    await chooseRow(0, null, "Hardened (about 10 s)");              // 28 chars: no §4.3a modal
  }
  const firstFrame = await raceFor(["Deriving", "Write down this phrase"], 60000);
  const modal = await waitFor("Write down this phrase", 60000);     // the countdown is ~10 s on the SH2
  must(modal, "method: " + method, "the confirm modal's method line");
  must(modal, "chars: " + phrase.length, "the confirm modal's char count");
  return { modal, firstFrame };
}

/**
 * The composition's STORED path hashes, as 64-hex or null, in path order.
 *
 * Throws rather than returning undefined when the seam is missing: an emulator
 * built before H5 has no shComposerPathHashes, and a walk that silently skipped
 * the stored-versus-displayed assertions would report the same PASS as one that
 * ran them.
 */
function pathHashes(where) {
  if (typeof window.shComposerPathHashes !== "function") {
    throw new Error("shComposerPathHashes is missing -- STALE emu.wasm. The browser caches it " +
      "and a cache-buster on index.html does not help; serve on a FRESH port.");
  }
  const h = window.shComposerPathHashes();
  if (h === null) {
    throw new Error(`${where}: no composition is running, so there is nothing stored to compare ` +
      `against. The walk is not where it thinks it is.\nScreen: ${JSON.stringify(window.shScreen())}`);
  }
  return h;
}

/** first8..last8 of a 64-hex digest -- the abbreviation gui.hashlockFirst8Last8 draws. */
const short8 = (hex64) => `${hex64.slice(0, 8)}..${hex64.slice(-8)}`;

/** Back out of the confirm modal to `Which hash?`, dropping the phrase (§4.6). */
async function backToWhichHash() {
  await tap(BACK, 400);                       // confirm  -> method pick
  await waitFor("Which method?");
  await tap(BACK, 400);                       // method   -> phrase screen
  await waitFor("Hashlock phrase");
  await tap(BACK, 400);                       // phrase   -> Which hash?
  await waitFor("Type a hashlock phrase");
}

export async function run() {
  for (const fn of ["shScreen", "shTargets", "shTap", "shPress", "shRelease", "shSysw",
                    "shComposerPathHashes"]) {
    if (typeof window[fn] !== "function") {
      throw new Error(`${fn} missing -- stale or wrong emu.wasm; rebuild from the hashlock-h2 branch and serve on a FRESH port`);
    }
  }
  const out = { typed: null, control: null, mixed: null, hardened: null, ok: false };

  // An empty region: `Which hash?` then holds no payload rows, so the phrase
  // row is row 0 and the lead is the one this stage added.
  window.shSysw("none");
  await waitFor("Load it?");
  await tap(BACK, 500);                                    // SKIP
  await waitFor("SeedHammer");
  await goTo("Wallet Policy");
  await tap(CONFIRM, 500);
  await waitFor("Build a new policy");
  await chooseRow(1, "Which script?", "Build a new policy");
  await chooseRow(1, "Start from?", "Segwit (wsh)");       // a key-less path is wsh-only
  await chooseRow(0, "Add a spend path", "Build my own paths");
  await chooseRow(0, "What can spend on this path?", "Add a spend path");
  await chooseRow(1, "EXPERIMENTAL", "A hash, no keys");
  await hold(CONFIRM);                                     // §8a key-less consent
  const which = await waitFor("Type a hashlock phrase");
  must(which, "No hash record in the payload", "the no-payload lead (§4.1)");
  must(which, "ms hashlock on the host", "the no-payload lead names the host route");

  // ── 1. the anchor row, SHA-256 ────────────────────────────────────────────
  const { modal: typed } = await trial(ANCHOR, "sha256");
  must(typed, ANCHOR_SHA_H, "the anchor phrase's sha256 digest (corpus derivation[0].sha256_h)");
  must(typed, "One phrase per policy", "the confirm modal's reuse line");
  out.typed = squash(typed).slice(0, 220);
  await backToWhichHash();

  // ── 2. the negative control: one character short ──────────────────────────
  const { modal: control } = await trial(CONTROL, "sha256");
  mustNot(control, "b867db87", "the CONTROL phrase produced the anchor's digest -- the screen is not " +
    "reading the typed bytes, and every positive row above is worthless");
  out.control = squash(control).slice(0, 220);
  await backToWhichHash();

  // ── 3. the mixed-case row: nothing normalises the phrase (spec §2) ────────
  const { modal: mixed } = await trial(MIXED, "sha256");
  must(mixed, MIXED_SHA_H, "the mixed-case row's sha256 digest (corpus)");
  mustNot(mixed, "b867db87", "the mixed-case phrase produced the LOWERCASE row's digest -- the phrase " +
    "was case-folded somewhere, which spec §2 forbids");
  out.mixed = squash(mixed).slice(0, 220);
  await backToWhichHash();

  // ── 4. hardened, then HOLD: the digest is assigned and §4.5's ────────────
  //      reconciliation screen is reached.
  const { modal: hardened, firstFrame } = await trial(ANCHOR, "hardened");
  must(hardened, ANCHOR_HARD_H, "the anchor phrase's hardened digest (corpus derivation[0].hardened_h)");
  mustNot(hardened, "b867db87", "hardened produced the SHA-256 digest -- the method pick did nothing");
  out.hardened = squash(hardened).slice(0, 220);
  out.hardenedFirstFrame = firstFrame;

  // ── the ORDER assertion, pinned to the confirm-modal frame ───────────────
  // The modal is up and the hold has not happened. Nothing may be stored yet:
  // a route that assigned at derivation time would leave the digest set even
  // when the operator reads it and presses Back.
  const before = pathHashes("with the confirm modal up, before the hold");
  if (before.length !== 1) {
    throw new Error(`the composition has ${before.length} path(s), want exactly 1 -- the walk built ` +
      `a different policy than it thinks.\nStored: ${JSON.stringify(before)}`);
  }
  if (before[0] !== null) {
    throw new Error("the path ALREADY holds a hash while the confirm modal is up: the digest is " +
      `assigned before the hold, so Back after reading it would leave it set (F-485).\n` +
      `Stored: ${JSON.stringify(before[0])}`);
  }
  out.storedBeforeHold = before[0];

  await hold(CONFIRM);

  // ── stored versus displayed, in FULL hex ─────────────────────────────────
  const after = pathHashes("after the hold");
  if (after[0] !== ANCHOR_HARD_FULL) {
    throw new Error("the STORED digest is not the corpus's hardened digest for this phrase.\n" +
      `  stored:   ${JSON.stringify(after[0])}\n  corpus:   ${ANCHOR_HARD_FULL}`);
  }
  if (short8(after[0]) !== ANCHOR_HARD_H) {
    throw new Error("the stored digest does not abbreviate to the token the confirm modal drew " +
      `(${ANCHOR_HARD_H}): the screen showed one digest and the policy holds another.\n` +
      `  stored: ${after[0]} -> ${short8(after[0])}`);
  }
  out.stored = after[0];

  const reconcile = await waitFor("run ms hashlock with this phrase", 20000);
  must(reconcile, "check the digest matches", "the reconciliation screen (§4.5)");
  // §1.5: the screen that asks for the comparison carries the operands.
  must(reconcile, ANCHOR_HARD_H, "the reconciliation screen repeats the confirm modal's token");
  must(reconcile, "chars: " + ANCHOR.length, "the reconciliation screen repeats the confirm modal's char count");
  must(reconcile, "If they differ", "the reconciliation screen says what a mismatch means");
  out.reconcile = squash(reconcile).slice(0, 200);
  await tap(CONFIRM, 500);
  const list = await waitFor("Spend paths", 20000);
  must(list, "hash", "the path row after the hash was assigned");
  out.pathRow = squash(list).slice(0, 200);

  // ok is SET, never recomputed (§4.4). Every assertion above throws, so
  // reaching this line is the whole of the result; restating four of them here
  // -- as this walk used to -- reports a subset of what already passed and
  // silently omits the rest, including both stored-versus-displayed checks.
  out.ok = true;
  return out;
}
```

- [ ] **Step 8: GREEN, and the mutations the gui half can run.**

Run: `go test -count=1 -run 'TestComposerStateHook|TestBuildTaggedHooks' ./gui/ && go test -count=1 ./cmd/emu/`
Expected: both `ok`. Measured, the `ok`-guard now reports what it could not read before:

```
needle_test.go:554: walk_hashlock_phrase.js sets `ok` to true after its last assertion, so it restates nothing (H5 §4.4)
needle_test.go:606: 8 walk script(s) checked; no driver-supplied plate count in any `ok`
--- PASS: TestWalkOkContainsNoDriverSuppliedPlateCount (0.00s)
```

Eight scripts checked where six were before: `walk_h0_preimage.js` and
`walk_hashlock_phrase.js` are the two the guard had been silently skipping.

| Mutation | Measured failure |
| --- | --- |
| delete `defer composerFlowExit(st)`'s `clearComposerStateHook()` (or the defer) | `TestComposerStateHookIsInstalledOnlyWhileAFlowRuns`: `the hook survived the composition it was installed for: []` |
| delete `setComposerStateHook(st)` from `composerFlow` | the same test: `the hook is not installed while composerFlow is running` |
| the hook hands out `st`'s own pointers (`out[i] = p.Hash`) | `TestComposerStateHookReportsEachPathAndHandsOutCopies`: `writing through the hook's pointer changed the POLICY: ff01…1e1f, want 0001…1e1f`. **This mutation was GREEN against a first draft of the test** that compared the policy against `d` — the same variable `st.list.Paths[1].Hash` points at — so the test now snapshots `want := d` first. A false PASS caught by running the mutation, and the reason the snapshot has a comment. |
| the hook skips paths with no hash instead of leaving a hole | the same test: `the hook reports 1 entries for a 2-path composition` |
| `composer_state_hook_tinygo.go` exports anything | `TestBuildTaggedHooksAreAbsentFromTheFirmwareImage`: `composer_state_hook_tinygo.go exports ComposerPathHashesOnDevice -- that file IS the firmware, so the host-only surface of this pair is in the image` |
| `ComposerPathHashes` named in another `gui` file | the same test: `composer_flow.go uses ComposerPathHashes in code but is not composer_state_hook.go` |

- [ ] **Step 9: Firmware size, with the hook and without it.**

```bash
export PATH=/nix/var/nix/profiles/default/bin:$PATH
nix develop -c tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller
```

Measured on this tree and on the pristine fork checkout, same command, same day:

| build | code | data | bss | flash | ram |
| --- | --- | --- | --- | --- | --- |
| fork main `b9a9a30` (baseline) | 1,565,552 | 31,852 | 31,004 | **1,597,404** | **62,856** |
| H5, this tree | 1,567,312 | 31,852 | 31,004 | **1,599,164** | **62,856** |
| H5 with the hook deleted from the tinygo view (`composer_state_hook_tinygo.go` removed, `composerFlow`'s hook lines removed) | 1,567,312 | 31,852 | 31,004 | **1,599,164** | **62,856** |
| H5 with a SECOND `defer clearComposerStateHook()` instead of `composerFlowExit` — **the shape NOT used** | 1,567,424 | 31,852 | 31,004 | 1,599,276 | 62,856 |
| positive control: the tinygo stub given one `println` | 1,567,536 | 31,852 | 31,004 | 1,599,388 | 62,856 |

**The hook's share is 0 B of flash and 0 B of RAM** — spec §4.1's assertion,
measured rather than inherited from `frame_hook`'s number. **And the zero is not
an artefact of a build that ignored the edit:** the positive control moves the
image by +224 B, so edits to that stub do reach it. The 112 B row is why the
single-defer shape exists at all; both numbers are recorded in
`composer_state_hook_tinygo.go` as well, where the next reader will look.

Whole stage against the baseline: **+1,760 B flash (+0.11%), +0 B RAM** for all
five follow-ups. No numeric ceiling is asserted, because neither the spec nor this
plan sets one; the acceptance is the delta against the named baseline.

- [ ] **Step 10: Build the emulator.**

```bash
./cmd/emu/build.sh
```
Measured: `built emu.wasm (10873113 bytes)`. `GOOS=js GOARCH=wasm go vet ./cmd/emu/` is clean.

- [ ] **Step 11: Commit.**

```bash
git add gui/composer_state_hook.go gui/composer_state_hook_tinygo.go gui/composer_state_hook_test.go gui/composer_flow.go gui/tinygo_split_test.go cmd/emu/composer_js.go cmd/emu/platform.go cmd/emu/walk_hashlock_phrase.js cmd/emu/needle_test.go
git commit -s -m "emu: the hashlock walk proves hold ORDER and stored-versus-displayed -- a !tinygo composition-state seam (0 firmware bytes), shComposerPathHashes, ok set not recomputed; and the ok-shape guard reads the assignment form it has been silently skipping since 45f3d4c (hashlock H5, F-485)"
```

- [ ] **Step 12: The three walk runs (spec §4.5) — the CONTROLLER runs these.**

Serve a FRESH port (the browser caches `emu.wasm` and a cache-buster on
`index.html` does not help), drive with playwright, and record each run WITH the
assertion that failed. The two mutations are exact one-line edits to
`gui/composer_hashlock.go`, each rebuilt with `./cmd/emu/build.sh`:

| run | edit | must |
| --- | --- | --- |
| (a) unmutated | — | PASS, `ok: true` |
| (b) the assignment moved before the confirm | replace `\t\t\th := hashlock.Digest(&x)` with `\t\t\th := hashlock.Digest(&x); st.list.Paths[idx].Hash = &h` | FAIL at §4.2's pre-hold read: `the path ALREADY holds a hash while the confirm modal is up` |
| (c) the stored hash perturbed by one byte | replace `\t\t\t\td := h` with `\t\t\t\td := h; d[0] ^= 1` | FAIL at the stored-versus-displayed assertion — `the STORED digest is not the corpus's hardened digest for this phrase` — and at NO earlier one: the confirm modal still draws `3cf5d421..b70a4c12` from the unperturbed `h`, and the pre-hold `null` read still passes |

Run (c) is what makes the stored-versus-displayed assertion falsifiable; without
it that assertion has never been shown able to fail (journey I-7).

---


### Task 6: Records (spec §1.4, §2.5, §6, §7)

**Files (none in the fork):**
- Modify: engrave `design/SPEC_hashlock_H2_device.md`, engrave `design/FOLLOWUPS.md`
- Modify: toolkit `docs/manual/src/40-cli-reference/43-ms.md`

**Interfaces:** none. Every block in this task is prose destined for a file
OUTSIDE the fork tree, so none carries a `file=` header and
`scripts/h5-plan-blocks-vs-tree.sh` checks none of it — it says so in its own
output. The device-side gate for the two quoted strings is
`TestComposerCopyIsVerbatimFromTheSpec`, which diffs the shipped functions against
the transcription in `composerCopyTable()` (Tasks 1 and 2); the H2 spec document is
updated by hand ALONGSIDE that table, and the two are not the same artifact. Say so
plainly rather than implying the test reads the spec file.

- [ ] **Step 1: H2 spec §4.5 — the write-down sentence and the reconcile clause.**
In `design/SPEC_hashlock_H2_device.md`, the lines block at `:264-266` (engrave
`e03d8e7`) reads *"Write down this phrase and the method now. They are / not on
this device and not on your plates. Without / both, this path can never be
spent."* Replace the first sentence with item 2's text (the second and third are
unchanged), and replace the block's closing reconciliation lines (`:272-274`,
*"Before you fund this wallet, run ms hashlock with this / phrase and method on
the host and check the digest / matches."*) with §1.1's body — which is now a
SEPARATE screen carrying its own `hash` and `method:` lines, so it moves out of
the confirm modal's line list and into the post-HOLD paragraph beneath it:

```
Write down this phrase, the method and this digest
now. They are not on this device and not on your
plates. Without both, this path can never be spent.
```

```
hash  <first8>..<last8>
method: <m>   chars: <n>
Before you fund this wallet, run ms hashlock with this
phrase and method on the host and check the digest
matches. If they differ, do not fund this wallet:
build it again.
```

**Do not touch §4.5's reuse block in the same edit.** Its lines `:267-271` still
carry the pre-drop-order wording ("One phrase per policy. Spending any path of a
wsh wallet publishes this digest…") while the shipped body carries the two-sentence
form §4.5's own drop order prescribes. That drift is REAL, PRE-EXISTING and
outside H5's five follow-ups — file it (Step 3) rather than fixing it here, so
`git diff` on this commit is H5's change and nothing else.

- [ ] **Step 2: H2 spec §4.7 — the phrase form's last sentence only.** The
blockquote at `:337-341` ends *"Back up the phrase and its method, or the /
preimage plate, separately."*; it becomes *"Back up every phrase and its method,
and every preimage plate, separately."* Nothing else in §4.7 changes — §1.4 is
explicit that §4.7 receives only §2.5's sentence.

- [ ] **Step 3: engrave `design/FOLLOWUPS.md` — five closures, and one new entry.**
Close F-480, F-484, F-485, F-487 and F-488 in the file's own heading convention
(`### F-NNN — ~~slug~~ **CLOSED 2026-09-05** — …`, as F-475 at `:15765` does),
each naming the commit that closed it and the gate that proves it. All five were
filed with owning phase **the next device code cycle**, which IS this leg, so
none is overdue and none is deferred — say so in the closure line, because the
burndown rule is checked by grep.

File TWO new follow-ups in the same commit. The first, owning phase **H2 spec
hygiene**: the §4.5 reuse-block drift Step 1 declines to fix, with the shipped
two-sentence text quoted so the fix is a transcription and not a re-decision. The
second, owning phase **the `me`/sysw manual chapter**: spec §5's journey M-5 note,
which has no section to land in (Step 4 measures that), quoting what it asks for.

- [ ] **Step 4: toolkit `docs/manual/src/40-cli-reference/43-ms.md` — re-quote both screens.**
The manual quotes device copy this leg changed, in two places (toolkit `46b40bb`):
`:482-483` inside the `#### Confirm, and reconcile against the host` code block,
and `:501-502`, the reconciliation blockquote. First:

```text
Write down this phrase, the method and this digest now. They are not on this
device and not on your plates. Without both, this path can never be spent.
```

Then the blockquote, which now carries the digest lines the screen repeats:

```text
> hash  3cf5d421..b70a4c12
> method: hardened   chars: 28
> Before you fund this wallet, run ms hashlock with this phrase and method on
> the host and check the digest matches. If they differ, do not fund this
> wallet: build it again.
```

The sentence beneath it — *"That is the reconciliation… its first and last eight
characters are what the confirm screen showed"* — is now also true of the
reconcile screen itself, and reads correctly either way; leave it.

**Spec §5's documentation-only item (journey M-5) has no target that exists, so
it is FILED, not invented.** §5 asks that "the manual's unlock section" say the
re-sealed payload has a new passphrase. There is no such section: the toolkit
manual's `docs/manual/src/40-cli-reference/` holds `41-mnemonic.md`, `42-md.md`,
`43-ms.md` and `44-mk-cli.md`, and a grep of the whole `docs/manual/src/` tree for
the refusal's own words — `Nothing was opened`, `cannot be unlocked here`, `not a
seed` — returns nothing at toolkit `46b40bb`. Writing an unlock chapter is a
documentation deliverable of its own, several screens wide, and adding a
free-floating sentence about re-sealing to the `ms hashlock` chapter would put it
where nobody reading about unlocking would find it. File it (Step 3) with owning
phase **the `me`/sysw manual chapter**, quoting §5's requirement, and say in the
follow-up that the device text already carries the instruction.

Run the manual's own gate from `docs/manual/`:

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit/docs/manual && make lint
```

This is a SEPARATE repo and therefore a separate commit and a separate push; the
toolkit's push ritual is that repo's, not the fork's.

- [ ] **Step 5: engrave commit.**

```bash
git add design/SPEC_hashlock_H2_device.md design/FOLLOWUPS.md
git commit -m "records: H5 folds H2 §4.5's write-down line and post-HOLD reconcile body and §4.7's phrase form; F-480/F-484/F-485/F-487/F-488 CLOSED at their owning phase; §4.5 reuse-block drift filed (hashlock H5)"
```

- [ ] **Step 6: Acceptance (spec §7).** All three walk runs of Task 5 Step 12
recorded with the failing assertion named; the H2 spec folds and the manual
re-quote landed; ONE opus adversarial execution review over the whole diff
(brief `design/agent-briefs/hashlock-H5-post-impl-brief.md`, report
`design/agent-reports/hashlock-H5-post-impl.md`) GREEN before merge; merge to fork
`main` with `--no-ff`; a signed image built (`sh2-flash -b`). The device walk (H2
§8) stays ASSUMED at the operator's word until they run it.

---

## Build gate

Every block above was hand-wired into `/scratch/code/shibboleth/.tmp/h5-gate` (a
`git ls-files` copy of fork main `b9a9a30`) **task by task, in the order written**,
with a build and a test run at every boundary and the whole `gui` package sharded
after Tasks 1, 2 and 5. Nothing was wired ahead of its task, which is the defect
the H2 gate could not see.

**The whole tree, at the end (Go 1.26.7, `/scratch/code/shibboleth/.toolchain/go`):**

Spec §6's "four packages" first, run at the Task 5 boundary:

```
$ go test -count=1 ./hashlock/ ./codex32/ ./sysw/ ./seal/
ok  	seedhammer.com/hashlock	0.232s
ok  	seedhammer.com/codex32	0.003s
ok  	seedhammer.com/sysw	0.040s
ok  	seedhammer.com/seal	12.330s
```

Then everything else:

```
=== gofmt (baseline lists the same five) ===
gui/transaction.go
gui/transaction_golden_test.go
gui/transaction_txrecord_test.go
mt/mt.go
mt/mt_test.go
=== go vet ./gui/ ./cmd/emu/ ===
gui/freetext_sizeproof_golden_test.go:111:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)
gui/transaction_golden_test.go:104:13: testing.ArtifactDir requires go1.26 or later (file is go1.25)
=== GOOS=js GOARCH=wasm go vet ./cmd/emu/ ===
(clean)
=== ./cmd/emu/build.sh ===
built emu.wasm (10873113 bytes); serve this directory and open index.html
=== gui shards ===
    1236 top-level tests
    partition verified exhaustive: 1236 == 1236
=== wall: 31s ===
RESULT: ok -- all 1236 tests ran across 24 shards
```

Both `gofmt` and `go vet` findings are PRE-EXISTING and reproduce on the pristine
fork checkout at `b9a9a30` — verified, not assumed. `b9a9a30`'s `gui` package has
1225 top-level tests by the shard script's own count; this plan adds 11.

`CGO_ENABLED=0 go test -timeout 20m ./...` — CI's exact command — is GREEN on the
wired tree: **55 packages `ok`, exit code 0**, no `FAIL` line. On the pristine
`b9a9a30` checkout the same command FAILS, for the reason Task 5 Step 6 records
and quotes.

**The checker.** `scripts/h5-plan-blocks-vs-tree.sh` (a thin wrapper over the
already-parameterised `scripts/h2-plan-blocks-vs-tree.sh`, which takes plan and
tree as arguments) compares every headed block above against that tree:

```
plan: design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md
tree: /scratch/code/shibboleth/.tmp/h5-gate

PASS IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:155  whole          gui/composer_provenance_test.go               (195 lines, identical)
PASS IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:358  fragment       gui/composer_hashlock_test.go                 (7 lines, verbatim substring)
PASS IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:372  fragment       gui/composer_hashlock_test.go                 (2 lines, verbatim substring)
  ... 48 more PASS lines ...
51 blocks checked, 0 FAIL

NOT COVERED by this script:
  * 23 fenced blocks carry no file= header (bash recipes, illustrative
    snippets); nothing here runs or checks them:
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:484  ``` (no info string)
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:655  ```bash
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:799  ``` (no info string)
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:880  ``` (no info string)
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:904  ```bash
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:926  ``` (no info string)
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:1190  ``` (no info string)
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:1217  ```bash
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:1278  ``` (no info string)
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:1316  ``` (no info string)
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:1337  ```bash
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:1842  ``` (no info string)
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:2356  ``` (no info string)
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:2376  ```bash
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:2404  ```bash
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:2411  ```bash
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:2461  ``` (no info string)
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:2467  ``` (no info string)
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:2506  ```text
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:2513  ```text
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:2527  ```bash
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:2536  ```bash
      IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md:2561  ``` (no info string)
  * every PROSE claim: expected test names, mutation outcomes, headroom and
    firmware numbers, spec references, file:line citations.
  * whether the tree is GREEN -- this compares TEXT only; `go test` and the
    gate report are what say the text works.
  * files the plan modifies without carrying a block for them.
```

**What the gate did NOT cover, beyond the checker's own list:** the three
emulator walk runs (Task 5 Step 12) — the walk was written, parsed and loaded as
an ES module, and `cmd/emu` builds, but no browser drove it; the toolkit `make
lint` of Task 6 Step 4; and every prose claim in Task 6, whose files are not in
the fork tree.

---

## Self-review

- **Every number here was measured on the wired tree, not copied from the spec.**
  186/339, 165/378, 347/107, 397, the 19 px readout budget, the 2-line lead, the
  five firmware sizes, and every test count. Where the spec and the measurement
  agree it is because two independent runs produced the same number.
- **The mutation for every test in this plan was RUN**, and the quoted failure is
  what the run printed. One of them (Task 5's pointer-copy row) came back GREEN
  first time and exposed a false PASS in a test written minutes earlier; the test
  was fixed and the mutation re-run.
- **One pre-existing RED is fixed here and one pre-existing DRIFT is not.** The
  `cmd/emu` guard is fixed because spec §4.4 changes the very shape it cannot
  read; H2 §4.5's reuse block is left alone and filed, because it belongs to no
  H5 follow-up and mixing it in would cost a reviewer the one diff that matters.
- **The riskiest thing in this plan is the seam**, because it is new surface in a
  package whose whole discipline is about what the firmware may contain. It is
  gated four ways: the split test (told about the pair, and still scanning it),
  the lifetime and copy tests, the 0-byte size measurement, and a positive control
  proving that measurement can move.
- **What a reviewer should attack first:** whether `composerAnyPathByPhrase`
  walking the paths on every §8h draw is the right shape when a composition can
  hold `md.ComposeMaxPaths` paths (it is O(paths) on a screen draw, and the map
  lookup is by value on a 32-byte key); whether the reconcile screen repeating the
  digest gives an operator a false sense of having checked it against the host;
  and whether Task 5's `ok`-guard change widens a gate that was narrow on purpose.
