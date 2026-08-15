# The true file-touch matrix for S1–S6 — closing the completeness gap

Scope: one question — for each stage of `IMPLEMENTATION_PLAN_multisig_build_repair.md`,
what does it ACTUALLY touch, versus what the plan names? Measured against
`/scratch/code/shibboleth/seedhammer` at `0ae3756`, 2026-08-14. **S0 excluded by
brief.** Everything in `parallel-implementation-feasibility.md` is taken as
settled and is not re-derived here.

## VERDICT

**The concurrency ceiling is 1, and the completeness gap makes it worse, not
better.** The prior report found five stages sharing one function; the true sets
show five stages sharing one *function*, one *shared gatherer used by four other
flows*, one *AST-oracle test that fails the moment a consumption site moves*,
one *13 KB walk script*, and — for S1 — a `Context` field (`gui/gui.go:69`) whose
type must change under every flow in the package. The plan's named sets
understate the true sets by 3× on average and by infinity on S2, which names
**zero** file paths for a stage that must edit `gui/bundle_flow.go` — a file
whose one function `bundleGatherFlow` has **five** call sites across five flows
(`git grep -n "bundleGatherFlow(ctx"`, count 5). Not one pair in S1–S6 is
disjoint. S6 is the only stage that writes no fork file, and it is not a peer:
it flashes the union of S1–S5 onto one machine and its item 2 is S3's acceptance
criterion. Nothing here changes the prior verdict; it removes the last
qualification from it.

## METHOD

Symbols were extracted from each stage's plan section, then resolved:

```
git grep -n "\bSYMBOL\b" -- '*.go'         # definition + every reference
git grep -n "TYPED-ONLY" -- '*.go'          # 10 (unchanged from settled)
git grep -n "bundleGatherFlow(ctx" -- '*.go'
git grep -n "multisigEngraveCards(" -- '*.go'
grep -n "^func \|^type " gui/multisig_build.go
sed -n '<range>p' <file>                    # every quoted line read, not inferred
```

Named-set extraction from the plan, per stage line range:

```
sed -n "${a},${b}p" IMPLEMENTATION_PLAN_multisig_build_repair.md \
  | grep -oE '`?[a-z0-9_]+(/[a-z0-9_.]+)+\.(go|js|bin|sh|py|md)[^`]*' | sort -u
```

## MATRIX

`●` = must edit (proven by a command below). `○` = conditional on a ruling the
plan leaves open. Rows are files; columns are stages.

| file | S1 | S2 | S3 | S4 | S5 | S6 |
| --- | --- | --- | --- | --- | --- | --- |
| `gui/multisig_build.go` | ● | ● | ● | ● | ● | |
| `cmd/emu/walk_trace_a.js` | ● | ● | ● | ● | ● | |
| `gui/bundle_flow.go` | ● | ● | | ○ | ● | |
| `gui/multisig.go` | | ● | ● | | ● | |
| `gui/multisig_build_test.go` | ○ | ● | | ● | ● | |
| `gui/template_engrave_test.go` | | ● | | ● | ● | |
| `gui/multisig_build_flow_test.go` | ● | ● | | ● | ○ | |
| `gui/multisig_restore.go` | | | ● | ● | ● | |
| `gui/multisig_restore_test.go` | | ○ | ● | ○ | ● | |
| `gui/gui.go` | ● | | | ○ | | |
| `gui/sysw_session.go` | ● | | | | | |
| `gui/sysw_admit_oracle_test.go` | ● | | | | | |
| `gui/sysw_cells_test.go` | ● | | | | | |
| `gui/sysw_programs_test.go` | ○ | | | | | |
| `gui/multisig_verify.go` | | ● | | | ○ | |
| `gui/singlesig_verify.go` | | ● | | | | |
| `gui/raster_test.go` | | ○ | | | | |
| `gui/md1_inspect.go` | | | ● | | | |
| `gui/md1_inspect_test.go` | | | ● | | | |
| `gui/bundle.go` | | | ● | | | |
| `gui/bip85.go` | | | ● | | | |
| `gui/singlesig.go` | | | ● | | | |
| `cmd/emu/embed_confinement_test.go` | | | ● | | | |
| `gui/multisig_match.go` | | | | ● | | |
| `gui/multisig_engrave.go` | | | | | ● | |
| `gui/multisig_engrave_test.go` | | | | | ● | |
| `gui/bundle_engrave_test.go` | | | | | ○ | |
| `gui/wipe_guard.go` | | | | ○ | | |
| *(whole merged tree, flashed)* | | | | | | ● |

## PER-STAGE: NAMED vs ACTUAL

| stage | named | actual (●) | ratio |
| --- | --- | --- | --- |
| S1 | 4 | 8 | 2.0× |
| S2 | **0** | 9 | ∞ |
| S3 | 7 | 11 | 1.6× |
| S4 | 2 | 7 | 3.5× |
| S5 | 3 (all citations; 0 named as edit targets) | 10 | ∞ |
| S6 | 0 | whole tree | — |

**S1 — what the plan missed.** `ctx.syswBundleSeed` is `string`
(`gui/gui.go:69`), and `bundleGatherFlow` reads exactly one
(`gui/bundle_flow.go:104-106`); N cards through the same `offer()` forces a type
or sibling-field change in `gui/gui.go`. Two structural tests break outright:
`gui/sysw_admit_oracle_test.go:61-62` hard-codes
`{"multisig_build.go", "buildMultisigPolicyFlow"}` and its AST walk matches only
`syswOffer` idents and `.take` selectors — a `takeAll` selector matches neither,
so the site vanishes and the `sites < len(syswConsumers)` floor at `:168-172`
fires. `gui/sysw_cells_test.go:154-206` has a `"built policy"` subtest asserting
`"First card from where?"` then `"md1 descriptors: 1"` — the exact offer S1
deletes. Conditional: `gui/sysw_programs_test.go:52-62` asserts the literals
`"ctx.syswBundleSeed = body"`, `"seed := ctx.syswBundleSeed"` and
`"scr.g.offer(mdmkText(seed))"` — it survives a new-sibling-field design and
breaks under widening.

**S2 — the stage that names no files at all.** D-4's gather title is
`layoutTitle(..., "Engrave Bundle")` at `gui/bundle_flow.go:155`, inside the
gatherer shared by **five** call sites: `gui/bundle_flow.go:29`,
`gui/multisig.go:79`, `gui/multisig_build.go:57`, `gui/multisig_verify.go:76`,
`gui/singlesig_verify.go:110`. Making the title program-specific edits the
shared file and its four external callers. Two of `gui/multisig_build_flow_test.go`'s
own assertions on the build path (`:239`, `:249`) wait for `"Engrave Bundle"`.
The interim refusal and duplicate check land in `assembleBuildPolicy`, whose
callers include `gui/template_engrave_test.go:154`. Test 5's raster floor wants
`gui/raster_test.go`'s `runUITouchRaster`/`countInk`/`assertFrameHasBody`.

**S3 — the gate is already unsatisfiable inside its own stage.**
`grep -rn TYPED-ONLY --include='*.go'` returns **10**, one of them a comment in
`cmd/emu/embed_confinement_test.go:12`. Reaching 0 means editing a `cmd/emu`
test. `scriptName` has exactly four sites (`gui/md1_inspect.go:20,58`,
`gui/multisig_restore.go:51`, `gui/bundle.go:315`) and no test asserts its output
strings — verified, so the two new tests are additive into
`gui/md1_inspect_test.go` and `gui/multisig_restore_test.go`.

**S4 — the struct literals the plan never looked for.** `buildPolicyParams`
literals with `SelfSlot:` live at `gui/multisig_build_test.go:216,329,451,491`
and `gui/template_engrave_test.go:153`. The seed hook is pinned at
`gui/multisig_build_flow_test.go:202-203`. The plan's own text mandates "a set
inventory on the restore doc", which is `gui/multisig_restore.go`. The walk-away
bound, if ruled as an idle limit, reaches `gui/wipe_guard.go` and `gui/gui.go:81`.

**S5 — the widest undeclared leak.** `multisigEngraveCards(ms1 string, mk1, md1
[]string, full bool)` (`gui/multisig_engrave.go:11`) takes exactly **one** ms1
and **one** mk1 set. Trace B needs one mk1 per held slot and one ms1 per master.
That signature change reaches its other production caller `gui/multisig.go:163`
and its test at `gui/multisig_engrave_test.go:14,36`. The DESTROY wording is
`bundleAbortWarning` at `gui/bundle_flow.go:351-356` — the plan's "no other
flow's call site changes" is true of the *signature* and false of the *file*,
which S2 also edits. `multisigVerifyFlow(ctx, th, derived bundle.Bundle, full)`
(`gui/multisig_verify.go:49`) takes one bundle; several held slots produce
several.

## DISJOINT PAIRS

**None.** Every pair among S1–S5 collides on at least two files: all five edit
`gui/multisig_build.go` (606 lines, `buildMultisigPolicyFlow` at `:39-198`) and
all five extend `cmd/emu/walk_trace_a.js` (266 lines, the only script that
reaches a completed engrave). Beyond that: S1∩S2∩S5 on `gui/bundle_flow.go`;
S2∩S3∩S5 on `gui/multisig.go`; S2∩S4∩S5 on `gui/multisig_build_test.go` and
`gui/template_engrave_test.go`; S3∩S4∩S5 on `gui/multisig_restore.go`.

The only pair with an empty *file* intersection is **(S3, S6)** — S6 writes no
fork file. It is not usable: S6 item 2 ("Engrave and restore an `sh(wsh)`
multisig… Confirms S3 on the plate") is S3's own acceptance, S6 flashes a build
of the tree S3 is mutating, and there is one machine and one flash cycle. An
empty write set is not independence when the read set is the entire tree.

**Maximum concurrent stages: 1.**

## PLAN CITATIONS THAT DO NOT RESOLVE

Checked every path and line the plan cites for S1–S6. **All resolve** —
`gui/multisig_build.go:54,61,67,75-79,162-168,254-272,324,338-344,342,437-458,464-511`,
`gui/multisig_match.go:34`, `gui/md1_inspect.go:58`, `gui/multisig_restore.go:51`,
`gui/bundle.go:315`, `gui/multisig.go:163`, `gui/bundle_flow.go:100-103`,
`md/encode_multisig.go:13-21,104-106`, `mk/mk.go:136,286`,
`TestBip85DeriveFlow_ScrubsBothMnemonics` (`gui/bip85_test.go:212`). Two notes:

- **S3's stated count is stale but its gate line is the authority.** The prose
  says "all 9"; `git grep -n TYPED-ONLY --include='*.go'` returns **10**. Already
  settled; recorded so the matrix's `cmd/emu/embed_confinement_test.go` row has a
  citation.
- *(S0, noted only — out of brief.)* `design/journeys/shot_server.py` does not
  exist in the fork; it is in `mnemonic-engrave`, and §"Reference convention"
  says paths are in the fork unless stated.

## WHAT I DID NOT CHECK

- I did not compile, run `go test`, or run the emulator walk. Every `●` is
  derived from grep/read evidence about what a change must reach, not from a
  build that failed.
- I did not enter S0 at all, per brief; S0's own file set is not in this matrix,
  and stages that consume S0 artifacts (the payload blob, the harness shapes) are
  scored on their own edits only.
- `○` rows depend on a design choice the plan leaves open (field-widening vs
  sibling field; idle-limit vs recorded non-wiping ruling; where a new test file
  lands). I did not pick a design in order to force them to `●`.
- New test files each stage must create are not columns — their names are not
  fixed by the plan, and inventing them would inflate the overlap with files that
  do not exist yet. The true overlap is therefore a **lower bound**.
- I did not audit whether any stage's content is correct, only what it touches.
