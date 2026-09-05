# R0 round 0 — fidelity + design lens on `IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md`

**Reviewer:** independent fidelity + design (opus tier), round 0 of the R0 gate.
**Plan:** `design/IMPLEMENTATION_PLAN_hashlock_H5_device_polish.md` at engrave master `0c2b13e`.
**Spec:** `design/SPEC_hashlock_H5_device_polish.md` (R0 GREEN at engrave `e03d8e7`).
**Fork:** `/scratch/code/shibboleth/seedhammer` main `b9a9a30`. Author's gated tree
`/scratch/code/shibboleth/.tmp/h5-gate` (read, never modified). Execution copies:
`.tmp/h5-fidelity` (gate tree), `.tmp/h5-pristine` (`git ls-files` copy of `b9a9a30`),
`.tmp/h5-t1` / `.tmp/h5-t12` (Task-1-only and Tasks-1-2 reconstructions),
`.tmp/h5-nohook{,D}` / `.tmp/h5-println` / `.tmp/h5-secondefer` (firmware controls),
`.tmp/hm` / `.tmp/mutb` / `.tmp/mutc` (mutation runs). Nothing committed.

**Counts: 0 Critical / 3 Important / 4 Minor / 4 Nit.**

---

## Findings

### I-1 — the walk's "stored versus displayed" assertion is a tautology, and Task 5 Step 12's run (c) never exercises it (spec §4.5(c) / journey I-7 unmet)

**Plan section:** Task 5 Step 7 (the walk, `mode=whole`) and Task 5 Step 12 (the three runs).

Spec §4.5(c) is normative and its whole purpose is stated in it: *"the stored hash
perturbed after assignment (one byte) -- must FAIL on the stored-equals-displayed
assertion and on no earlier one (journey I-7: without (c) that assertion is never shown
able to fail)."*

The plan's walk writes, after the hold:

```javascript
  const after = pathHashes("after the hold");
  if (after[0] !== ANCHOR_HARD_FULL) {
    throw new Error("the STORED digest is not the corpus's hardened digest for this phrase.\n" + …);
  }
  if (short8(after[0]) !== ANCHOR_HARD_H) {
    throw new Error("the stored digest does not abbreviate to the token the confirm modal drew " + …);
  }
```

Both operands of the second assertion are file constants, and they are the *same*
constant:

```
$ python3 …  # constants read out of cmd/emu/walk_hashlock_phrase.js
HARD_FULL    = 3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
short8(FULL) = 3cf5d421..b70a4c12
ANCHOR_HARD_H= 3cf5d421..b70a4c12
EQUAL?        True
```

So once `after[0] === ANCHOR_HARD_FULL` has passed, `short8(after[0]) === ANCHOR_HARD_H`
holds by construction. **The second assertion cannot fail — ever, under any device
behaviour.** It also never reads the token the screen actually drew: the modal text is in
`hardened`, and the walk compares the stored value against a constant instead.

Consequently the plan's own run-(c) row is wrong about which assertion fires. Recipe (c)
is `d := h` → `d := h; d[0] ^= 1`; the perturbed digest trips the **corpus** assertion
first, and the plan's table nevertheless labels that outcome *"FAIL at the
stored-versus-displayed assertion"* while quoting the corpus message
(`the STORED digest is not the corpus's hardened digest for this phrase`). The recipe
compiles and is a valid one-line edit — verified:

```
$ cp -a .tmp/h5-fidelity .tmp/mutc && sed -i 's/\t\t\t\td := h$/\t\t\t\td := h; d[0] ^= 1/' …
$ GOOS=js GOARCH=wasm go build ./cmd/emu/     # build exit=0
```

— but there is no single-byte device mutation that reaches the stored-equals-displayed
assertion, so journey I-7's requirement stays unmet by exactly the run that was added to
meet it, and an implementer who follows Task 5 Step 12 literally will record run (c)
against the wrong assertion name.

The composite property *is* still proven transitively (`must(hardened, ANCHOR_HARD_H, …)`
pins displayed to the constant; the corpus check pins stored to the constant), so this is
not a hole in what the walk establishes today. It is a gate that cannot fail, presented as
the gate the spec asked for.

**SUGGESTION.** Make the assertion read the frame. `trial()` already returns the confirm
modal's text; parse the drawn token out of it (`squash(modal)` contains `hash3cf5d421..b70a4c12`,
so a `/hash([0-9a-f]{8}\.\.[0-9a-f]{8})/` match on the squashed text is enough), keep it in
`out.displayed`, and compare `short8(after[0])` against **that** — placed *before* the
corpus check, so recipe (c) fails there and the corpus check remains as the oracle. Then
Task 5 Step 12's run-(c) row names the assertion that actually fires. If the plan prefers
to keep the current order, it must instead correct the table to say run (c) fails at the
corpus assertion and record that spec §4.5(c) is not satisfied — but that reopens
journey I-7.

---

### I-2 — the rewritten `ok`-shape guard reads the FIRST `x.ok =` in a walk, so a caller-supplied plate count in the LAST one passes, and is logged as proof that it does not

**Plan section:** Task 5 Step 6 (`cmd/emu/needle_test.go`).

The plan's new regex and its branch:

```go
	okAssignRe = regexp.MustCompile(`(?ms)^\s*\w+\.ok\s*=\s*(.*?);\s*$`)
…
		if m := okAssignRe.FindStringSubmatch(src); m != nil {
			rhs := strings.TrimSpace(m[1])
			checked++
			if okSetRe.MatchString(rhs) { t.Logf(…"restates nothing (H5 §4.4)"…); continue }
			if strings.Contains(rhs, "plates") { t.Errorf(…) }
			continue
		}
```

`FindStringSubmatch` returns the **first** match; a walk's verdict is its **last**
assignment. The plan's comment asserts the opposite — *"The assignment regex captures the
right-hand side EXACTLY, anchored on the `.ok =` it is looking for, so unlike the property
span it cannot grab a neighbouring literal"* — and the branch also skips the census/verdict
floor that guards the property shape against precisely this class of mis-span.

Counterexample, run on the gate tree (`cmd/emu/walk_zzz_probe.js`, since removed):

```javascript
export async function run() {
  const out = { plates: null, ok: false };
  out.ok = false;
  out.plates = window.shPlates();
  out.ok = out.plates === 3;
  return out;
}
```

```
$ go test -count=1 -v -run TestWalkOkContainsNoDriverSuppliedPlateCount ./cmd/emu/
    needle_test.go:554: walk_hashlock_phrase.js sets `ok` to true after its last assertion, so it restates nothing (H5 §4.4)
    needle_test.go:554: walk_zzz_probe.js sets `ok` to false after its last assertion, so it restates nothing (H5 §4.4)
    needle_test.go:606: 9 walk script(s) checked; no driver-supplied plate count in any `ok`
--- PASS: TestWalkOkContainsNoDriverSuppliedPlateCount (0.01s)
```

The guard passes the exact defect it exists to catch (I-1/F-170), reports it as checked,
and emits a log line stating the opposite of the truth. That is worse than the RED it
replaced, because the RED at least said INCONCLUSIVE. It is also precisely the widening the
plan's own self-review flags for attack ("whether Task 5's `ok`-guard change widens a gate
that was narrow on purpose") — it does.

**SUGGESTION.** Take every match, not the first: `for _, m := range
okAssignRe.FindAllStringSubmatch(src, -1)`, run the `plates` check on each right-hand side,
and only take the "sets it after the last assertion" exit when **every** match is a bare
boolean (or when the last one is and no earlier one names `plates`). Add a fixture walk in
`cmd/emu/testdata` shaped like the counterexample above with a MUTATION comment, so the
guard's own new blind spot has a test.

---

### I-3 — two production doc comments are silently reattached to the new helpers the plan inserts

**Plan section:** Task 5 Step 2 (`gui/composer_flow.go`) and Task 3 Step 1 (`gui/composer_paged.go`).

Both fragments are inserted immediately after the closing line of an existing doc comment,
with no blank line, so Go merges them into one comment block that then documents the new
function and leaves the old one undocumented. Measured on the gated tree:

```
$ go doc -u seedhammer.com/gui composerFlow
func composerFlow(ctx *Context, th *Colors)
                                              ← no documentation at all

$ go doc -u seedhammer.com/gui composerFlowExit
func composerFlowExit(st *composerState)
    composerFlow is "Build a new policy" (SPEC_wallet_policy_composer.md §7),
    from the door to the plate, in §7's order.

    THIS FUNCTION IS THE JOIN, and its absence was the defect two R0
    lenses found independently. …
```

```
$ go doc -u seedhammer.com/gui composerPageLines
func composerPageLines(…) ([]op.Op, int, []image.Rectangle)
                                              ← no documentation at all

$ go doc -u seedhammer.com/gui composerTextBand
func composerTextBand(dims image.Point) (left, width int)
    composerPageLines lays out lines[start:] into the content box and returns
    the ops, HOW MANY were drawn, and each drawn row's TOUCH BAND.

    THE ONE MEASURE SITE. Every capacity number in SPEC §13 comes from this
    function, …
```

At `b9a9a30` `go doc composerFlow` prints the full comment; after the plan it prints
nothing. What is lost is not decoration: `composerFlow`'s comment is the written record of
the "plans list components and omit the call that joins them" Critical, and
`composerPageLines`'s is the "ONE MEASURE SITE" record behind every §13 capacity number.
Nothing in the plan's gate can see this — `gofmt` is clean, `go vet` reports only the two
pre-existing go1.26 findings, every test passes, and
`scripts/h5-plan-blocks-vs-tree.sh` matches fragments by substring, which is satisfied
either way. Two files carry it, so it is a systematic placement error, not a slip.

**SUGGESTION.** One blank line before each inserted comment block (and, for
`composer_paged.go`, put `composerTextBand` *above* `composerPageLines`' doc comment rather
than between it and its function). Say in the fragment header where the block goes, since
`mode=fragment` cannot express placement — or make these two `mode=whole` so the checker
sees the adjacency.

---

### M-1 — the "same digest re-typed as 64 hex" case is a byte-identical duplicate of "one phrase path"

**Plan section:** Task 1 Step 1, `TestComposerAnyPathByPhraseIsPerDigest`.

```go
		{"one phrase path", [][32]byte{phrase}, []*[32]byte{&phrase}, true},
…
		{"the same digest re-typed as 64 hex is still by phrase", [][32]byte{phrase}, []*[32]byte{&phrase}, true},
```

Same set, same hash slice, same expectation — the second row exercises nothing the first
does not, and nothing about *re-typing* happens in either. Spec §6 lists "same digest
re-typed as hex" as one of §2's cases; as written it is covered only by an alias. The
underlying property is safe (the predicate compares values, and the hex arm writes a new
`*[32]byte` to `p.Hash`), so this is a coverage record defect rather than a behaviour risk.

**SUGGESTION.** Drive it: take the state after a phrase HOLD, then run `composerHashEdit`
through the `Type 64 hex` row entering the same 64 hex, and assert
`composerAnyPathByPhrase(st)` is still true and §8h still draws the phrase form — which
also proves the hex arm replaces the pointer without disturbing the set. Or drop the row
and note in the test that a value set makes the case indistinguishable by construction.

### M-2 — spec §6 asks for the firmware delta "stated per change"; the plan states it per stage

**Plan section:** Task 5 Step 9.

Spec §6: *"firmware size with the delta stated per change and the hook's share asserted 0
bytes."* The plan gives five whole-image rows and one aggregate — *"Whole stage against
the baseline: +1,760 B flash (+0.11%), +0 B RAM for all five follow-ups"* — plus the hook's
own 0 B. The hook's share, which is the gate that matters, is measured and reproduces
exactly (see "Verified true" below). The per-follow-up breakdown is not there.

**SUGGESTION.** Either add four more one-line builds (Tasks 1..4 applied cumulatively, one
row each) at the cost of ~4 tinygo builds, or fold spec §6 to say "the delta stated for the
stage and the hook's share asserted 0 bytes" and record why per-change is not worth the
builds for four copy-string edits.

### M-3 — `hashlockOtherPathLine`'s corrected comment still points at the deleted flag

**Plan section:** Task 1 Step 6.

The plan states *"`hashlockOtherPathLine`'s doc comment names the removed field and is
corrected in the same edit"* and supplies a one-line fragment. In the gated tree the result
reads:

```go
// It reads *p.Hash directly rather than the phrase set, so it is unaffected by
// that flag's own staleness, and it skips idx because …
```

"that flag" no longer exists; the sentence now refers to nothing. The correction stopped
one line short of the clause that made the original sentence true.

**SUGGESTION.** Extend the fragment by one line, e.g. *"…rather than the phrase set, so it
is unaffected by that set's own history, and it skips idx because…"*, or drop the clause.

### M-4 — spec §5's "the copy-table row updated" is unsatisfiable and nothing records a correction

**Plan section:** Task 4 "Interfaces" and Task 6.

The plan's departure is correct on the facts — verified: `unlockNotPermittedBody` is not a
`composerCopy*` function, `composerCopyTable()` covers only those (its AST scan requires
the `composerCopy` prefix), and the body's fit gate is
`assertModalBodyFits(t, tc.name, errorScreenBody, body)` at
`gui/unlock_preimage_test.go:166`. But spec §5 remains normative text asking for a row that
cannot exist, and Task 6 folds H2 §4.5 and §4.7 while recording nothing for this. A future
reader diffing spec §5 against the implementation sees one unmet item with the explanation
living only in a plan paragraph.

**SUGGESTION.** Add to Task 6 Step 3 (or a new step) a one-line H5-spec erratum — the same
shape §1.4/§2.5's H2 folds take — saying §5's "copy-table row" clause does not apply
because the body is not composer copy, and naming
`TestUnlockNotPermittedBodyNamesTheRecordAndTheKind`'s longest-noun row as what stands in
its place.

### N-1 — the walk's prelude still tells the reader to rebuild from the `hashlock-h2` branch

`cmd/emu/walk_hashlock_phrase.js:326` (gated tree):
`throw new Error(\`${fn} missing -- stale or wrong emu.wasm; rebuild from the hashlock-h2 branch …\`)`.
Task 5 Step 7 rewrites this file `mode=whole`, so the string is the plan's now, and the
branch is `hashlock-h5` (Global Constraints).

### N-2 — "THE KEYBOARD GRID WAS PROBED … (2026-09-05, this emulator build)" is inherited, not established this cycle

That block is unchanged from `b9a9a30`'s walk (it does not appear in
`diff -u seedhammer/cmd/emu/walk_hashlock_phrase.js .tmp/h5-gate/…`), yet the file is
presented `mode=whole` and the plan's own Build gate says *"no browser drove it"*. The
coordinates are almost certainly still right — Task 3 leaves the lead two lines at 44 px
and `MaxHeight=209` is unchanged — but "this emulator build" now claims a probe the cycle
did not run. Re-date it to the H2 probe, or re-probe during Task 5 Step 12's run (a).

### N-3 — the positive control's +224 B is not reproducible from the plan's description

Task 5 Step 9 records *"the tinygo stub given one `println`"* → 1,599,388 B. The plan does
not say what the `println` printed or which stub carried it; `println("hook")` in
`setComposerStateHook` measures 1,599,340 B (+176). The control's conclusion — that edits
to the stub reach the image — holds either way; only the number is unreproducible.

### N-4 — `composer_hashlock.go`'s HOLD comment keeps the `composer_state.go:239` citation the plan dropped elsewhere

Task 2 Step 4 deliberately changes the identical claim in `composer_copy.go` from
`(composer_state.go:239 at the fork baseline c4a64fc)` to `(composer_state.go at the fork
baseline c4a64fc)`, but the same citation at `gui/composer_hashlock.go:72` is left. One or
the other.

---

## The four recorded departures — judged

| # | departure | judgement |
| --- | --- | --- |
| 1 | hook is 0 firmware bytes via one exit defer | **Right call, and reproduced exactly.** See below. Not really a departure — spec §4.1 asked for a measured 0, and it is measured. |
| 2 | a pre-existing RED walk-`ok` guard fixed inside Task 5 | **Right call.** The RED is real and reproduces verbatim on the pristine checkout; spec §4.4 changes the very shape the guard cannot read, and a red suite is itself blocking, so deferring it would leave H5 unable to close. The *fix* carries I-2. |
| 3 | no unlock manual section exists, so journey M-5 is filed not invented | **Right call, verified.** `docs/manual/src/40-cli-reference/` holds exactly `41-mnemonic.md`, `42-md.md`, `43-ms.md`, `44-mk-cli.md` at toolkit `46b40bb`, and `grep -rn "Nothing was opened\|cannot be unlocked here\|not a seed" docs/manual/src/` returns nothing. Writing a chapter is out of scope; the device text already carries the instruction. |
| 4 | no copy-table row for the unlock body | **Right call on the facts** (verified: the body is not a `composerCopy*` function and `composerCopyTable` covers only those; its fit gate is `assertModalBodyFits` at `unlock_preimage_test.go:166`), **but it owes the spec erratum in M-4.** |

---

## Verified true — do not re-derive

Machine-checked on my own copies; state these in any re-review brief.

**Task order.** I reconstructed a Task-1-only tree and a Tasks-1-2 tree from `b9a9a30`
(reverting every later task's hunks in the four multi-task files) and ran the whole `gui`
package on each:

```
Task 1 alone : go vet clean (bar the 2 pre-existing go1.26 findings)
               RESULT: ok -- all 1229 tests ran across 24 shards
Tasks 1-2    : RESULT: ok -- all 1231 tests ran across 24 shards
```

Both match the plan's Step 10 / Step 7 numbers exactly. Task 1 first is necessary and
sufficient: `composerHashRows` depends only on the payload digests, so run (b)'s early
assignment does not move the walk's rows either.

**Whole tree.** `CGO_ENABLED=0 go test -count=1 -timeout 20m ./...` → **exit 0, 55
packages `ok`, no FAIL**. `gui` shards: **1236 tests, partition exhaustive**. `b9a9a30`
lists **1225** by the shard script's count, so +11 as claimed. `gofmt -l .` lists the same
five files on both trees. Four packages (`hashlock codex32 sysw seal`) `ok`.

**Pre-existing RED, verbatim on the pristine checkout:**

```
--- FAIL: TestWalkOkContainsNoDriverSuppliedPlateCount (0.00s)
    needle_test.go:525: INCONCLUSIVE: walk_h0_preimage.js has no `ok:` property this test can read, …
    needle_test.go:525: INCONCLUSIVE: walk_hashlock_phrase.js has no `ok:` property this test can read, …
    needle_test.go:563: 6 walk script(s) checked; …
```

**Fit numbers, reproduced independently:** reconcile **186 drawn / 339 headroom**; §8h
phrase form **165 / 378**; confirm modal longest variant **347 / 107**; unlock body at the
longest noun and a two-digit index **153 / 397**. All four match spec §1.1, §2.5, §1.2, §5.

**Geometry, reproduced:** `band left=8 width=411; lead (407,44) = 2 line(s) of 23 px` and
`MaxHeight=209 grid=(340,182) gap=8 -> readout budget 19 px; one line is 19 px`. §3.3's
fallback correctly does not fire.

**Firmware sizes, rebuilt with the plan's exact command:**

| build | flash | ram |
| --- | --- | --- |
| pristine `b9a9a30` | **1,597,404** | 62,856 |
| gate tree (H5) | **1,599,164** | 62,856 |
| hook removed, `composerFlowExit` collapsed back to `defer st.reg.scrub()` | **1,599,164** | 62,856 |
| second `defer clearComposerStateHook()` | **1,599,276** (+112) | 62,856 |

Every number matches the plan. **The hook's share is 0 B of flash and 0 B of RAM,
confirmed**, and the +112 B second-defer row is exact to the byte. (Keeping
`composerFlowExit` as a scrub-only wrapper while deleting the hook calls measures 1,599,196
— i.e. the counterfactual the plan measured is the strict one; worth naming in the plan for
the next reader, but the claim as stated is correct.)

**Corpus constants** in the walk are the corpus's:
`derivation[0].sha256_h = b867db87…edbc96cb`, `derivation[0].hardened_h = 3cf5d421…b70a4c12`,
mixed-case row `95d44470…2297a7ff`. `ANCHOR_HARD_FULL` is byte-exact.

**Copy strings** match spec §1.1, §1.2, §2.5 and §5 word for word (checked against the
`composer_copy.go` / `unlock_kdf.go` literals and the `composerCopyTable` transcriptions).

**`TestComposerCopyIsVerbatimFromTheSpec`'s mechanism:** it compares `r.got` against
`r.verbatim` inside `composerCopyTable()` — it does **not** read
`SPEC_hashlock_H2_device.md`. Task 6's statement of this is correct and is the more accurate
of the two; spec §1.4's *"diffs the code against `SPEC_hashlock_H2_device.md`"* is the loose
one. No action needed for the plan.

**H2-spec and manual citations are exact:** §4.5's write-down block is at `:264-266`,
its reconcile lines at `:272-274`, §4.7's blockquote at `:337-341`; the reuse-block drift at
`:267-271` is real and correctly declined + filed. Toolkit `43-ms.md:482` and `:501-502`
are the two places quoting changed copy, and a whole-tree grep finds no third. All five
follow-ups exist with owning phase **the next device code cycle**, so none is overdue;
F-475's closure convention at `FOLLOWUPS.md:15765` is as quoted.

**Mutations re-run, all reproducing the plan's quoted failures:** the nil-map panic
(`panic: assignment to entry in nil map`); `composerAnyPathByPhrase` → `len(set) > 0`
(the same four tests, the same subtests); the hook handing out `st`'s pointers; the hook
skipping nil paths; deleting `clearComposerStateHook()`; deleting `setComposerStateHook`;
the tinygo stub exporting a symbol; `ComposerPathHashes` named in another `gui` file; and
the three-space→one-space reconcile header — including the plan's subtle claim that
`TestHashlockReconcileScreenCarriesTheDigestMethodAndChars` stays **GREEN** under it while
`TestHashlockReconcileHeaderIsSpelledLikeTheConfirmModal` alone goes red. Confirmed.

Both Task 5 Step 12 mutation recipes are exact, unique one-line anchors
(`\t\t\th := hashlock.Digest(&x)` and `\t\t\t\td := h`, one occurrence each) and both
compile under `GOOS=js GOARCH=wasm go build ./cmd/emu/`. The walk parses as an ES module
(`node --check`).

**Design items raised in the plan's self-review, resolved:** `md.ComposeMaxPaths = 8`
(`md/compose.go:77`), so `composerAnyPathByPhrase`'s O(paths) walk with a 32-byte key is
eight map lookups on a screen draw — not a concern. No live reference to `hashByPhrase`
survives (only two historical mentions inside comments). `TestComposerCopyTableCoversEveryBody`'s
`declared != 53` literal does not move. The six other tests that drive `composerFlow` and
abandon it via `defer quit()` do **not** leak an installed hook — `iter.Pull`'s stop sets
`ctx.Done` and the flow unwinds through the defer — so there is no cross-test pollution
risk for `TestComposerStateHookIsInstalledOnlyWhileAFlowRuns`.

---

**Counts: 0 Critical / 3 Important / 4 Minor / 4 Nit.** The plan is unusually well
measured — every headroom, geometry and firmware number in it reproduced on an independent
tree, and every mutation I re-ran printed what it says it prints. All three Importants are
in Task 5: two are gates that cannot fail (I-1, I-2) and one is a mechanical placement error
that costs two long-form records (I-3).
