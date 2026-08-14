# Review: is `TestBuildTaggedHooksAreAbsentFromTheFirmwareImage` a real structural guard?

**Reviewer:** independent sonnet review, mutation-tested
**Scope:** `gui/tinygo_split_test.go` (new) and `gui/plate_hook_test.go` (the deleted
structural test it replaced), diff `10286e4..HEAD` (5 commits) in
`/scratch/code/shibboleth/seedhammer`, branch `main`.
**Isolation note:** two other reviewers were working the same commits concurrently
in the primary checkout. All mutations, builds and test runs in this review were
done exclusively in a dedicated worktree at
`/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/3985bd41-08d3-42b8-a967-1493b588d215/scratchpad/wt-guard`
(HEAD `740888d`), per a mid-task correction from the coordinator. The primary
checkout was never written or edited by this review — only `Read` and
`git show <rev>:<path>` were used against it. Its dirty state, observed twice
during this review with two *different* sets of modified files
(`cmd/emu/nfc.go`/`nfc_js.go`/`nfc_test.go`/`platform.go`, then later
`cmd/emu/engraved.go`/`platform.go`/`toolpath_js.go`/`gui/engraved_hook.go`/
`gui/gui.go`/`gui/unlock_session.go`), is the other concurrent reviewer's activity,
not this review's. The worktree is confirmed clean at the end of this report
(`git status --short` and `git diff --stat` both empty, HEAD unchanged at
`740888d`).

## Verdict

**It looks like a structural guard and mostly behaves like one, but it has one
clean, silently-passing bypass that reaches the real firmware image, plus one
undisclosed structural blind spot beyond the one the test's own doc admits to.**
The bypass is proven end-to-end: guard test PASS, full `gui` package suite PASS,
and the actual `tinygo build` with CI's exact flags succeeds and links a live,
reachable call site carrying the sensitive interface into `./cmd/controller`.

## 1. Mutations that survive (the guard does NOT catch them)

### Finding A — CRITICAL: a second, real hook interface declared inside the `_tinygo.go` stub file is completely invisible

**File:line (root cause):** `gui/tinygo_split_test.go:118` —
```go
f, err := parser.ParseFile(fset, p.host, nil, 0)
```
Only `p.host` is ever parsed for exported interface declarations. `p.stub` (the
`_tinygo.go` file — the one file that IS compiled into the firmware) is **never
parsed for interfaces at all.** The `found == 0` guard at line 145-149 only checks
that the *host* declares at least one exported interface; it says nothing about
what the *stub* declares.

**Escape scenario:** pair a host file that declares any decoy exported interface
(satisfying `found > 0`, so the "this pair carries no interface to protect" error
never fires) with a stub file (`*_tinygo.go`, correctly `//go:build tinygo`) that
declares the *real* sensitive interface and a live type assertion against it. The
stub's own build-constraint line is correct (`//go:build tinygo`), so the
constraint check at lines 89-103 passes too — that check only verifies the tag
line's *text*, never the file's *content*. Since the real interface's name is
never captured into `owner`, the final identifier scan (lines 157-174) has
nothing to look for and never inspects the stub file's declarations.

**Evidence — reproduced live, with a real call site, against the actual CI toolchain:**

Two new files in `gui/` (both cleanly reverted afterward):

```go
// gui/sneaky_hook.go
//go:build !tinygo
package gui
type Placeholder interface{ Foo() }   // decoy: satisfies found>0
func notifySneaky(Platform, string) {}
```

```go
// gui/sneaky_hook_tinygo.go
//go:build tinygo
package gui
type SneakyAware interface{ Sneak(secret string) }   // the real hook — lives IN the tinygo build
func notifySneaky(pl Platform, secret string) {
	if sa, ok := pl.(SneakyAware); ok {
		sa.Sneak(secret)
	}
}
```

Plus one line added to `gui/run_flow.go`, right next to the real `notifyFrame`
call in the draw loop (`gui/run_flow.go:264`), so the sensitive call is genuinely
*live and reachable* on every drawn frame, not dead code:
```go
notifyFrame(pl, content)
notifySneaky(pl, "mutation-test-live-call")
```

Results, all from inside the isolated worktree:
- `go test ./gui/ -run TestBuildTaggedHooks -count=1 -v` → **PASS**
  (`--- PASS: TestBuildTaggedHooksAreAbsentFromTheFirmwareImage`)
- `go test ./gui/... -count=1` (whole package tree, 62.8s) → **all PASS** — confirms
  this is a genuinely quiet mutation, not one that trips some other test.
- `tinygo build -size short -o /dev/null -target pico-plus2 -stack-size 16kb
  -gc precise -opt 2 -scheduler tasks ./cmd/controller` (the exact command and
  flags `.github/workflows/test.yml`'s `tinygo-device-build` job runs) → **EXIT 0**,
  `1311132 code | 31176 data | 30732 bss | 1342308 flash | 61908 ram`. `SneakyAware`,
  its type assertion, and the live call site are compiled and linked into the real
  firmware image with the real toolchain.

Removing the decoy `Placeholder` interface (leaving `sneaky_hook.go` with no
exported interface) makes the guard correctly fail at `tinygo_split_test.go:146`
("declares no exported interface... if this pair is not an interface hook, say so
here"). So the `found == 0` rail exists and works — it is just trivially defeated
by any unrelated decoy interface in the host file, because it was never designed
to check that the SAME interfaces reachable from the host correspond to what the
stub actually declares.

This is exactly the class of mutation the review brief names ("a second interface
in a stub file") and is precisely the property the test's own header comment says
must hold "by CONSTRUCTION rather than by argument" — disproven by construction.

### Finding B — IMPORTANT: the scan is not recursive; a hook pair moved into any `gui/` subpackage is entirely invisible, and the doc overclaims coverage

**File:line:** `gui/tinygo_split_test.go:52` — `os.ReadDir(".")`, not
`filepath.WalkDir`.

The header comment claims (lines 41-42): *"This discovers every `//go:build` pair
instead, so hook number three is covered by existing without anybody remembering
to add it here."* That claim is only true within the immediate `gui/` directory.
`gui/` has real subpackages that are compiled into the firmware
(`gui/op`, `gui/saver`, `gui/text`, `gui/widget`, `gui/assets`, `gui/layout` —
all appear in the real `tinygo build -size` output captured above). A
`foo_hook.go` / `foo_hook_tinygo.go` pair placed in any of them, declaring and
using a brand-new exported interface entirely within that subpackage, would never
be enumerated: `pairs` is built purely from `os.ReadDir(".")`'s single-level
listing, so pairing, constraint-checking and interface-extraction all silently
skip every subdirectory.

**Evidence:** created `gui/subhook/sub_hook.go` (`!tinygo`) and
`gui/subhook/sub_hook_tinygo.go` (`tinygo`) declaring `SubAware`; guard still
**PASS**. (Both files removed and worktree confirmed clean afterward.) The pass
here is unsurprising given the code (`os.ReadDir` cannot see into
subdirectories) — the evidence is offered mainly to confirm the reasoning matches
observed behavior, not as a novel discovery requiring a live-build proof the way
Finding A did.

This is weaker than Finding A as an attack (relocating an *existing* hook's whole
implementation into a new subpackage is a large, visible change, not a quiet
one-file edit) but it is a real, **undisclosed** limitation: the doc's "discovers
every pair" is stated without qualification, and nowhere does the test say its
search root is non-recursive.

### Mutations that were tried and correctly FAIL (not bypasses — reported for completeness)

- **Interface embedded in another interface**, in an untagged file (`gui/gui.go`):
  ```go
  type embedTestAware interface { FrameAware }
  ```
  Correctly caught: `gui.go uses FrameAware in code but is not frame_hook.go`.
  `ast.Inspect` visits the `*ast.Ident` inside the embedded field regardless of
  embedding syntax, so this is not an escape.
- **Legacy `// +build` constraint with no `//go:build` line**, on a brand-new pair
  (`gui/legacy_hook.go` / `gui/legacy_hook_tinygo.go`): the actual Go toolchain
  still honors the legacy-only form for real build-constraint purposes (confirmed
  with `go build -tags tinygo` correctly excluding/including the files), but
  `buildConstraint()` only recognizes a `//go:build ` prefix, so it returns `""`
  for both files and the guard correctly fails loudly (`legacy_hook.go carries
  "", want "//go:build !tinygo"`). This is over-strict in a safe direction, not a
  bypass.
- Type alias / defined-type wrapping an owner interface name necessarily still
  contains the identifier token and would be caught the same way embedding is;
  not separately mutation-tested since the mechanism is identical to the embed
  case above.
- A hook added to `gui/preview.go` is not an escape by itself: `preview.go`
  carries its own `//go:build !tinygo` and so never enters the tinygo build
  regardless of what this guard checks. It only becomes dangerous if paired with
  a *second*, untagged or tinygo-tagged reference elsewhere — and any such
  reference, if it names the interface identifier directly, is caught the same
  way the embed case is. (Not exhaustively mutation-tested; flagged as reasoned,
  not proven, since it collapses into already-tested cases.)

## 2. Is the floor real? Can it be satisfied vacuously?

**Floors, verified by code inspection and mutation:**
- `len(files) < 20` → `t.Fatalf` (INCONCLUSIVE). Current `gui/` has **67**
  non-test `.go` files — 3.3x the floor, not fragile.
- `len(pairs) < 2` → `t.Fatalf` (INCONCLUSIVE). Current `gui/` has **3** real
  pairs (`plate_hook`, `frame_hook`, `engraved_hook`) — 1.5x the floor.
- `found == 0` on a pair's host → `t.Errorf` (loud failure, not silent).

**No, these floors cannot be satisfied vacuously to produce a false PASS.** Every
one of them, when under-satisfied, calls `t.Fatalf` or `t.Errorf` — a hard,
visible test failure, not a silent skip or a vacuously-true assertion. Padding
`gui/` with junk files to clear the 20-file floor, or adding a decoy pair to clear
the 2-pair floor, does not defeat any actual check; it just satisfies a sanity
check whose only job is "did this scan land in the right directory." The one place
a floor *is* defeated is `found == 0` (Finding A above) — but that is defeated by
supplying a **real, unrelated, exported interface**, not by any vacuous or empty
construct, and it does not touch the two `Fatalf` floors at all.

## 3. Does the AST scan do what the comment claims?

- **"It actually walks every non-test file"** — **true**, verified by reading the
  code: `files` is built once (lines 56-63) by filtering `os.ReadDir(".")` for
  non-dir, `.go`, non-`_test.go` entries, and the exact same slice is reused
  unmodified for the final identifier scan (`for _, name := range files` at line
  157). There is no second, narrower filter applied later.
- **"Mode 0 ... to avoid matching identifiers in comments"** (lines 110-114) —
  **the stated mechanism is wrong, though the outcome it describes is correct.**
  Verified with an isolated Go program: parsing the same source with `mode = 0`
  and `mode = parser.ParseComments` produces **identical** results for whether an
  `*ast.Ident` matching a comment-only mention is found — neither mode ever
  produces one, because comment text is stripped by the scanner and never becomes
  an `*ast.Ident` node under *any* parser mode. `parser.ParseComments` only
  controls whether `*ast.CommentGroup` nodes are attached to the tree (for
  `.Doc`/`.Comment` fields); it does not turn comment text into identifiers
  either way. So mode 0 is not doing the load-bearing work the comment credits it
  with — the actual reason comments are safe is that Go's lexer discards them
  before any identifier token exists, independent of parser mode. This does not
  weaken the guard (the behavior is correct regardless of mode), but the
  explanation is factually wrong and could mislead a future maintainer into
  thinking a mode change is safety-relevant here. **Minor.**

## 4. Is the admitted blind spot the only one?

**No.** The doc (lines 44-50) admits exactly two things: (a) pairing is
filename-suffix-based (`_tinygo.go`), so a `foo_host.go`/`foo_device.go`-style
split would be invisible; (b) a `!tinygo` file with no stub at all (like
`preview.go`) is excluded wholesale rather than hooked.

Findings A and B above are **not** covered by either disclosure:
- Finding A (interface declared directly in the stub file, host carries a decoy)
  has nothing to do with filename convention — the pairing succeeds correctly by
  the documented convention; the gap is that only the host side of a correctly-
  paired split is ever inspected for interface declarations.
- Finding B (non-recursive search root) is a directory-locality assumption never
  stated anywhere in the doc, and the doc's "discovers every pair" line
  overclaims relative to it.

A gate that hides a blind spot is worse than no gate; here the test discloses two
real ones and has (at least) two more it does not.

## `gui/plate_hook_test.go` — was anything the old structural test pinned silently lost?

**No — checked against `git show 10286e4:gui/plate_hook_test.go` and confirmed
verbatim.**

- `TestPlateHookFiresOncePerJobWithTheWholeSpline` (the functional/behavioral
  test) is retained **byte-for-byte identical** — diffed line-by-line against the
  old version, zero differences in the test body itself. Only the file's import
  block shrank (removing `go/ast`, `go/parser`, `go/token`, `os`, `path/filepath`,
  `strings`), which is exactly the expected consequence of removing the deleted
  test and the `buildConstraint` helper that only it and the deleted test used.
- `TestPlateHookIsAbsentFromTheFirmwareBuild` (the structural half) was deleted
  and replaced with a pointer comment (new `plate_hook_test.go:133-138`) that
  correctly states it moved to `tinygo_split_test.go`, and why (keyed to
  `frame_hook.go` becoming the second hook, avoiding a hand-maintained
  file/identifier list). The property is not lost — it is superseded by a more
  general version, consistent with the stated intent.
- `buildConstraint()` was not duplicated or orphaned: `grep` confirms it now
  exists exactly once, in `gui/tinygo_split_test.go:179`, and nowhere else in
  `gui/*.go`.

## Summary of findings

| Severity | Finding | Location |
|---|---|---|
| **Critical** | Real hook interface declared and used entirely inside a `_tinygo.go` stub file, paired with a host carrying only a decoy exported interface, is completely invisible — proven with a live call site that compiles, links, and passes the whole `gui` suite AND a real `tinygo build` with CI's exact flags | `gui/tinygo_split_test.go:118` (only `p.host` is ever parsed for interfaces; stub content is never inspected) |
| **Important** | Scan root is `os.ReadDir(".")`, not recursive; a hook pair relocated into any `gui/` subpackage (`gui/op`, `gui/saver`, etc. — all real, firmware-compiled subpackages) is entirely invisible to pairing, and the header comment's "discovers every `//go:build` pair" overclaims this | `gui/tinygo_split_test.go:52` |
| **Minor** | "Mode 0 ... to avoid matching identifiers in comments" attributes correct behavior to the wrong mechanism; verified empirically that parser mode has no effect on whether comment text can produce `*ast.Ident` matches (it never can, under any mode) | `gui/tinygo_split_test.go:110-114` |
| No finding | Floors (`<20` files, `<2` pairs, `found==0`) cannot be satisfied vacuously — every one fails loudly (`Fatalf`/`Errorf`), never silently | `gui/tinygo_split_test.go:64-67, 84-87, 145-149` |
| No finding | `plate_hook_test.go`'s deleted structural test: functional test retained verbatim, structural test's replacement pointer comment is accurate, `buildConstraint` cleanly relocated with no duplication | `gui/plate_hook_test.go:133-138` |

**Critical and Important both block per project policy.** Finding A in
particular directly falsifies the guard's own stated design goal ("true by
CONSTRUCTION rather than by argument") with a reproducible, live, CI-flag-exact
build proof — this is not a theoretical gap.

## Suggested direction (not a mandate — out of scope to design the fix here)

Finding A's root cause is a one-line asymmetry: line 118 parses only `p.host`.
The natural fix is to also parse `p.stub` for exported interface/type
declarations and require it declare none beyond what a legitimate no-op stub
needs — or, more simply, treat *any* exported interface found in a `_tinygo.go`
file as an automatic failure, since a stub file legitimately has no reason to
export a new interface type at all. Finding B's natural fix is
`filepath.WalkDir` from the package root instead of `os.ReadDir(".")`, with the
usual test/vendor/subpackage-`_test.go` exclusions preserved.

## Worktree cleanliness (confirmed)

```
$ cd /tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/3985bd41-08d3-42b8-a967-1493b588d215/scratchpad/wt-guard
$ git status --short   # (empty)
$ git diff --stat      # (empty)
$ git log --oneline -1
740888d emu: shToolpath.strings() -- the census of what was actually engraved
```

All mutation files (`gui/sneaky_hook.go`, `gui/sneaky_hook_tinygo.go`,
`gui/subhook/`, `gui/legacy_hook.go`, `gui/legacy_hook_tinygo.go`) were removed
and `gui/run_flow.go`, `gui/gui.go` were `git checkout`-restored after each
mutation. The primary checkout at `/scratch/code/shibboleth/seedhammer` was never
written to by this review.
