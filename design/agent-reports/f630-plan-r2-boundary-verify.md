# R2 verification — does D5a's boundary close the class? (sonnet)

**Reviewer:** sonnet, independent of the fold's author.
**Artifact:** `design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md` at mnemonic-engrave
`3b771bde` (confirmed byte-identical to the working tree — `git show
3b771bde:...| diff -` empty), decision D5a and task T2.
**Trees:** fork `/scratch/code/shibboleth/seedhammer` main `95716e97`; primary
`/scratch/code/shibboleth/descriptor-mnemonic` main `b2c5d693`. Go 1.26.7 at
`/scratch/code/shibboleth/.toolchain/go`.
**Method:** D5a's boundary was IMPLEMENTED FOR REAL in a throwaway `git
worktree` of the fork at `95716e97` — `loadVectorRecord` added (mirroring
`loadVectorChunks`), all seven `gui/` sites that read a `keyed_*.conformance.json`
record routed through it (the three split sites D5a names, plus the four
"non-intersecting" readers, since the plan states no exception list), and a
structural gate test written as literally as the plan's prose describes (a
`go/ast` scan of `gui/*_test.go` for `os.ReadFile(filepath.Join(...))` calls
naming `"vectors"` and a literal containing `"conformance.json"`, excluding the
loader's own file). T2's pins (three F-529 `.conformance.json` + the wsh phrase
card) were then copied into `md/testdata/forkbuilt/`, and every affected test
was actually run, not reasoned about. The worktree was removed with `git
worktree remove --force`; `git status --short` is empty in the fork, the
primary, and mnemonic-engrave.

**Counts: 0 Critical, 3 Important, 0 Minor, 0 Nit.**

---

## Answer to the ONE question

**Does D5a's boundary close the class, or does it move again?** It narrows the
class substantially — the three named split sites, once routed, are real
protection — but it does not close it. The class moves twice: (1) into the
scanner's own pattern-matching logic, where the most natural first-draft
implementation of the plan's prose silently fails to see the *exact* call
shape every existing site uses, and (2) sideways into `gui/`'s sibling package
`md/`, which the boundary's own text claims is covered and is not. Both are
demonstrated by execution below, not inferred from the prose.

---

## Q1 — Can the structural gate be defeated? YES (Important)

A plain string-literal read — no `filepath.Join` call at all — evades the
described scan entirely. This is not a contrived spelling: it is the *existing*
idiom for testdata paths in this same package (`gui/template_engrave_test.go:207`,
`gui/composer_selfcheck_test.go:269` both use bare string literals rather than
`filepath.Join`).

Reproduced: with the fully-routed, gate-passing tree in the worktree, a fourth
test was added —

```go
func TestEvasionProbeDirectStringLiteral(t *testing.T) {
	const vector = "keyed_wsh_timelock_hashlock"
	raw, err := os.ReadFile("../md/testdata/vectors/" + vector + ".conformance.json")
	...
	chunks := loadVectorChunks(t, vector)  // the pin-preferring CARD loader
	...
}
```

— compiles clean and the structural gate still **PASSES**:

```
$ go test ./gui/ -run 'TestNoDirectVectorRecordReadOutsideLoader' -v
--- PASS: TestNoDirectVectorRecordReadOutsideLoader (0.06s)
```

This is a live fourth occurrence of the exact C-1/NEW-1 shape (pin-preferring
card, directly-read vendored record) that the scan exists to prevent, invisible
to it. `os.DirFS`/`fs.ReadFile` and a package-level `const` holding the literal
path are the same evasion by construction — none of them is a `filepath.Join`
call, which is the only shape the plan's prose describes scanning for.

## Q2 — Can it report a FALSE PASS? YES, reproduced (Important)

Two distinct vacuity traps, one of them hit on the first honest attempt to
build the gate:

**A false pass on the gate's own motivating examples.** The first, most
natural implementation of "scans for a direct `filepath.Join(.., "vectors",
….conformance.json")` read" matches a call's arguments only as bare string
literals (`*ast.BasicLit`). Every actual call site in this codebase — including
the three D5a names — writes the argument as `name+".conformance.json"`, a
`BinaryExpr` concatenation, not a literal. Built and run exactly that way, the
gate examined 20+ `gui/*_test.go` files (non-vacuous by file count) and still
reported **PASS** on a tree where a real, unrouted violation of the identical
shape as the original C-1 finding was present — the scan's pattern never fires
on anything, ever, and nothing catches that. Only after widening the matcher to
also unwrap `BinaryExpr` concatenation did reintroducing a direct read (in
`composer_policy_address_test.go`) get caught:

```
# BasicLit-only matcher, violation present: PASS (silent false pass)
# same matcher widened to handle x+".conformance.json": FAIL, catches it
    f630_boundary_gate_test.go:98: direct keyed_*.conformance.json read(s)
    outside loadVectorRecord: [composer_policy_address_test.go:48:26]
```

**The plan requires no self-check.** Nothing in D5a or T2 requires the gate to
assert it can positively detect a known violation (a mutation-style self-test),
only that it runs (implicitly — the plan doesn't even require the `len(files)
== 0` non-vacuity guard that this package's own precedent,
`gui/tinygo_split_test.go:105` — `if len(files) < 20 { t.Fatal(...) }` —
already applies to a comparable source scan). To be non-vacuous the gate must
assert both that it examined a nonzero, plausible number of files **and** that
it flags a synthetic known-bad fixture (or the historical three sites, if
regressed) — a file count alone does not prove the pattern can match anything.

## Q3 — Does routing every reader through the loader break a currently-green test? NO (no finding, confirmed by execution)

All four non-intersecting readers were routed through `loadVectorRecord` and
run twice: once before T2's pins existed (baseline), once after copying the
three F-529 `.conformance.json` records into `md/testdata/forkbuilt/` (T2
state). Both runs are green:

```
--- PASS: TestEveryKeyedVectorReachesAnAddress          (vectorAddress, :261)
--- PASS: TestTaprootScriptPathMatchesRust
--- PASS: TestWshWitnessScriptHashesToRustsAddress
--- PASS: TestDeviceDerivesTheComposerTimelockHashlockPolicy   (keyed_compose_wsh_timelock_hashlock)
--- PASS: TestSeatedTemplateDerivesTheVectorAddress            (keyed_tr_with_leaf)
--- PASS: TestFingerprintDisambiguatesTwoMastersAtOnePath      (seat_same_origin_two_masters)
--- PASS: TestSwappingFingerprintsSeatsTheOtherWallet          (seat_same_origin_two_masters)
```

Mechanism confirmed rather than assumed: `ls md/testdata/forkbuilt/` shows the
only `.conformance.json` pins T2 adds are `keyed_tr_multi_a`,
`keyed_tr_sortedmulti_a`, `keyed_wsh_timelock_hashlock` — none of the four
readers' vector names (`keyed_tr_with_leaf`, `keyed_wsh_thresh`,
`seat_same_origin_two_masters`, `keyed_compose_wsh_timelock_hashlock`) matches
any of them, so `loadVectorRecord` falls through to the vendored copy for all
four, exactly as before routing. This holds for the plan's whole lifecycle, not
just at T2: `seat_same_origin_two_masters` exists nowhere in
descriptor-mnemonic (`find ... -iname '*seat_same_origin_two_masters*'` empty)
and doesn't match T3's `^(keyed_|compose_)` vendor pattern, so T3 never touches
it either; the other three names do match T3's pattern but have no forkbuilt
pin either, so card and record continue moving together through the vendored
tier after the re-vendor too.

## Q4 — Is `md/` in or out? OUT — and the plan's own text claiming otherwise is wrong (Important)

The structural gate as specified globs `gui/*_test.go` only; a Go test in
package `gui` cannot reach `md/*_test.go` (a different package, different
working directory) without an explicit different glob, which the plan doesn't
add. So `md/` is structurally outside the boundary — confirmed by construction,
not measurement.

The plan's own sentence claims otherwise: *"and `md/conformance_keyed_test.go:44`,
whose record and card both come from the vendored tier so they move together...
They are routed through the loader anyway, because the structural gate admits
no exceptions"* (lines ~203–206). This is false for that site: nothing routes
it through any loader (`loadVectorRecord` is a `gui`-package helper; `md`'s
reader uses `loadPhraseChunks`, untouched), and no md-side structural gate
exists anywhere in the plan.

It happens to be safe **today**, for a reason unrelated to any gate — verified
by grep across every `.conformance.json` reader in `md/`:
`compose_pkh_emit_test.go`, `compose_stubs_test.go`, and
`conformance_keyed_test.go` all pair the record with `vectorPath`/
`loadPhraseChunks`, neither of which is pin-preferring, so record and card
always move together by construction. The pin-preferring `vectorChunksFor` is
used only by `duplicate_keys_test.go` and `policy_shape_test.go` — neither of
which *also* reads a raw `.conformance.json` for comparison (`policy_shape_test.go`'s
`shapeFromVector` compares against hardcoded Go literals in the test table, not
a record file) — so there is no split anywhere in `md/` today. But nothing
prevents one from being introduced: a future `md/` test pairing
`vectorChunksFor` with a direct `.conformance.json` read for one of the F-529
three would be the C-1 shape recurring in the sibling package, and the `gui/`-
scoped gate would not see it.

## Q5 — Does T2's ordering still work? YES (no finding, confirmed by execution)

Verified by execution, not by inspection alone:

- The structural gate is a pure Go-source-shape check, unaffected by corpus
  data — it passed identically before and after T2's pins were added, with no
  re-vendor (T3) run at all.
- Every behavioral test in T2's stated gate (`go test ./md/ ./gui/ -run
  'Duplicate|Pinned|ReachesAnAddress|TaprootScriptPathMatchesRust|WshWitnessScriptHashesToRustsAddress|VectorRecord'
  -v`) passes green at T2, pre-T3: `TestEveryKeyedVectorReachesAnAddress`,
  `TestTaprootScriptPathMatchesRust`, `TestWshWitnessScriptHashesToRustsAddress`,
  and `TestDuplicateKeySlotOnTheCorpusKeyReuseVectors` all PASS with the pins
  in place and the vendored corpus untouched — because for the three F-529
  names both halves come from the same frozen pre-revendor forkbuilt pin, and
  for every other vector both halves still come from the same (stale but
  mutually consistent) vendored tier.
- T1's new gate (`TestKeyedConformanceDescriptorHeadersAgreeWithTheirOrigins`,
  which *would* be red on the stale corpus) does not match any term in T2's
  `-run` filter by substring, so it is correctly excluded and cannot spuriously
  break T2's gate. T2 does not depend on T3.

---

## What was not re-derived

Per the brief: the three split sites and line numbers, the four
non-intersecting readers and their vectors, the forkbuilt inventory (5
`.md1.txt`, 0 `.conformance.json` pre-T2), the 14 r0 findings being fixed, and
the fork's header-emission correctness were all taken as settled and not
re-measured, except where confirming them was a byproduct of executing Q3
(forkbuilt listing, `seat_same_origin_two_masters` provenance).

## Scope note

Per the brief's rules of evidence, one concrete spelling was required and
given for Q1; the plain-string-literal evasion and the BasicLit-only
false-pass were both reproduced by execution in the throwaway worktree, not
reasoned from the plan's prose. No tracked file was modified; both the fork
and mnemonic-engrave verify clean.
