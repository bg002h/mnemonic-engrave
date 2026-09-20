# R3 — did the r2 fold land, and is the F-630 plan IMPLEMENTABLE as written? (opus)

**Reviewer:** independent of the fold's author.
**Artifact:** `design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md` at mnemonic-engrave
`692d86fa` (verified byte-identical to the working tree: `git diff 692d86fa HEAD --
design/IMPLEMENTATION_PLAN_F630_xpub_header_sync.md` is empty).
**Trees:** fork `/scratch/code/shibboleth/seedhammer` main `95716e97`; primary
`/scratch/code/shibboleth/descriptor-mnemonic` main `b2c5d693`. Go 1.26.7 at
`/scratch/code/shibboleth/.toolchain/go`.
**Method:** everything below was EXECUTED, not read. The token scan was built with
`go/scanner` and run on the tree. T1's D1 gate, T2's `vectorRecordFor` + record pins,
and T3's widened re-vendor (the real `scripts/vendor-compose-vectors.sh`, widened and
run against `b2c5d693`) were implemented in a throwaway `git worktree` of the fork at
`95716e97` and the affected tests actually run. The worktree was removed with
`git worktree remove --force`; `git status --short` is empty in the fork and in
descriptor-mnemonic, and `git worktree list` no longer carries it.

**Counts: 2 Critical, 5 Important, 5 Minor, 2 Nit.**

---

# A. Did the r2 fold land?

## Q1 — evasion by a plain string literal: **FIXED**

D5b replaces the call-shape match with a token scan:

> "So the gate is a scan for the substring `conformance.json` in the **STRING
> TOKENS** of any `*_test.go` outside the loader's own file, failing on any
> occurrence. Not an AST call-shape match and not a raw text grep: `go/scanner`
> in its default mode yields string tokens and skips comments, which is exactly
> the discrimination needed."

Executed. A `go/scanner` implementation (mode 0, `token.STRING`, `strconv.Unquote`,
`strings.Contains`) over `gui/*_test.go` + `md/*_test.go` at `95716e97` finds every
spelling r2 used to defeat the old rule: both halves of `name+".conformance.json"`
(`gui/composer_policy_address_test.go:48`), bare literals
(`md/compose_stubs_test.go:15`), a glob pattern
(`gui/policy_address_test.go:125`) and a `strings.TrimSuffix` argument
(`gui/policy_address_test.go:164`). The r2 evasion is closed, and the plan names
its own residual (`".conformance" + ".json"`) rather than pretending otherwise.

The fix does introduce a NEW hole of the same class at file granularity — "outside
the loader's own file". Filed as I-3 under B, not charged against this verdict.

## Q2 — false-PASS vacuity: **PARTIAL**

D5c adds the three ordered assertions:

> "The gate therefore asserts three things in order: that it examined a plausible
> number of files (the precedent is `gui/tinygo_split_test.go:105`, `if len(files)
> < 20 { t.Fatalf("INCONCLUSIVE: …") }`); that a synthetic known-bad string IS
> flagged by the same matcher; and only then that the real tree is clean."

The *specific* vacuity r2 reproduced — a `*ast.BasicLit`-only matcher that never
fires — is FIXED, because a token scan has no call-shape to miss and the synthetic
probe would catch a matcher that could not match.

It is PARTIAL because the false-PASS *class* survives, reproduced below (I-4): the
self-test exercises the **matcher**, never the **walk that feeds it**, and one
aggregate file count cannot see a whole missing directory. `gui/` has 244
`*_test.go`; `md/` has 36. Measured, with the md glob mistyped to a plausible
wrong relative root:

```
correct:          examined 281 test files ... 16 unrouted conformance.json string token(s)
md glob broken:   examined 245 test files ... 10 unrouted conformance.json string token(s)
```

245 and 281 are both "plausible"; the precedent threshold is 20. The synthetic
known-bad still passes in both runs. Six real `md/` occurrences go unexamined and
nothing in D5c's three assertions can say so.

## Q4 — `md/` outside the boundary: **FIXED (at the decision level)**

D5d retracts the false sentence verbatim and widens the scope:

> "The first draft scoped the scan to `gui/*_test.go` and then claimed
> `md/conformance_keyed_test.go:44` was 'routed through the loader anyway'. That
> was false: `loadVectorRecord` is a `gui` helper, `md`'s reader uses
> `loadPhraseChunks`, and no md-side gate existed. … The scan therefore covers
> `md/*_test.go` as well — a test in `gui` reads `../md/*_test.go` as plain file
> I/O".

Executed and confirmed: a test in package `gui` globbing `../md/*_test.go` reads all
36 files and reports their occurrences with correct positions. D5e adds the md-side
loader: "`md/` also needs its own pin-preferring record loader (`vectorRecordFor`,
mirroring `vectorChunksFor`)".

FIXED as a decision. The *task* half did not follow it — T2 routes only `gui/` while
the gate it adds scans `md/`. Filed as I-1 under B.

---

# B. Is the plan implementable exactly as written?

Walked T1 → T2 → T3 → T4 as an implementer who may not invent anything.

## C-1 (Critical) — at T3, the gate CANNOT reach 46 of 46: T2 freezes three records that fail D1 by construction

T2: "copy the current `.conformance.json` of **all three** F-529 vectors to
`md/testdata/forkbuilt/`."
T1: "Use the **pin-preferring** loader … for the card and, once D5 lands, the
pin-preferring record too."
T3: "*T1's gate must now be GREEN at 46 of 46.*"

The three files T2 freezes are three of the 44 that T1's gate fails on. Their
`chains[].descriptor` carries the master xpub under a four-component origin — the
exact 0.44.0 defect D1 exists to detect:

```
md/testdata/vectors/keyed_tr_multi_a.conformance.json (today, = what T2 pins)
  tr([73c5da0a/48'/0'/0'/2']xpub661MyMwAqRbcGQnC8zMGwRc4EXYJCgXrxx9kXtw1RXqu4TcW26Px…
  base58-decoded: depth 0, parent fp 00000000, child 0   →  D1 wants depth 4, child 2'
  keys[] holds xpub6DkFAXWQ2dHxq… / xpub6DzhyrnFFYQ1H…   →  D2a's 65-byte compare also fails
```

Reproduced end to end in the worktree: T2's pins added, T3's widened re-vendor run
for real (246 files from `b2c5d693`), a D1-only gate reading the record through the
pin-preferring `vectorRecordFor`:

```
f630_d1_gate_test.go:67: F630 D1: 46 vectors, PASS 43, FAIL 3
--- FAIL: TestF630D1HeadersAgreeWithTheirOrigins
    keyed_tr_multi_a          chain 0/1: depth 0 (want 4), child 0 (want 2147483650)  [6 violations]
    keyed_tr_sortedmulti_a    chain 0/1: depth 0 (want 4), child 0 (want 2147483650)  [6 violations]
    keyed_wsh_timelock_hashlock chain 0/1: depth 0 (want 4), child 0 (want 2147483650) [10 violations]
```

43, not 46. And there is no correct-header source for these three anywhere: the
pin exists precisely because `b2c5d693` replaced the reuse-BEARING policy with a
reuse-FREE one (`wallet_descriptor_template_id` f15f2969… → 8c1c0566…, slot 2 moved
from `48'/0'/0'/2'` to `48'/0'/1'/2'`), so the primary has no corrected record of the
policy being frozen. An implementer following the plan reaches T3, sees three hard
failures, and the plan's only guidance is T1's warning that tuning the gate to exempt
those three is the one thing not to do.

**At T3, an implementer must decide whether the frozen forkbuilt records are inside
or outside D1's scope — and if inside, where a correct-header rendering of a policy
the primary no longer ships comes from. The plan does not say.** (Confirmatory
counts: the stale vendored corpus is PASS 2 / FAIL 44 — `keyed_tr_keyonly`,
`keyed_wpkh` — so T1's own acceptance is correct; `b2c5d693`'s corpus is PASS 46 /
FAIL 0, so the vendored half of T3 does reach green. Only the pinned half cannot.)

## C-2 (Critical) — routing `md/conformance_keyed_test.go`'s record reds the existing cross-language gate at T3

D5a: "**Every read of a `keyed_*.conformance.json` record goes through
`loadVectorRecord`, and a structural test enforces it.**"
D5d, on the md readers: "all pair the record with the non-pin-preferring
`vectorPath`/`loadPhraseChunks`, so both halves move together".

The boundary destroys the very property D5d relies on. `md/conformance_keyed_test.go`
globs `keyed_*` — which includes all three F-529 names — and its card is
`loadPhraseChunks` (vendored, not pin-preferring). Route the RECORD through
`vectorRecordFor` and the pinned (frozen, pre-re-vendor) record is now compared
against a re-vendored card: the C-1 split shape, inverted, in the sibling package.

Reproduced. Worktree at real T2+T3 state, record routed, card untouched:

```
$ go test ./md/ -run TestKeyedConformanceAgreesWithRust -v
conformance_keyed_test.go:96:  keyed_tr_multi_a: wallet_policy_id
      go:   fe4d264c6e40999b8695329adfe599d9
      rust: 1f26f9b7cdb8e745c898bb93f1134ba7
conformance_keyed_test.go:112: keyed_tr_multi_a: wallet_descriptor_template_id
      go:   8c1c05666abdf6b29df9c2056c0cb49e
      rust: f15f29695b582f0e3a7b5be6e54f3908
… identical failures for keyed_tr_sortedmulti_a and keyed_wsh_timelock_hashlock
--- FAIL: TestKeyedConformanceAgreesWithRust
```

(The "go:" values are D6's recorded "the primary moved here" ids — the card is the
re-vendored one, the record is the pin.) Baseline before the change: PASS.

The remedy is one line the plan never names — switching that test's card to the
pin-preferring `vectorChunksFor`. Verified:

```
$ # chunks := loadPhraseChunks(t, name)  ->  chunks := vectorChunksFor(t, name)
$ go test ./md/ -run TestKeyedConformanceAgreesWithRust
ok  	seedhammer.com/md	0.012s
```

T1 prescribes the pin-preferring card only for the NEW test it adds. **At T2, an
implementer must decide whether routing a record read also obliges moving that
test's CARD to the pin-preferring loader, and the plan does not say** — and the
plan's own T3 rule ("If T3 turns any `./gui/` or `./md/` test red for a reason
other than a stale expected-value, that is a finding, not a fixup — … it stops the
plan") means the honest implementer halts here. Same mechanism is latent in
`md/compose_pkh_emit_test.go:37` and `md/compose_stubs_test.go:15`, which also pair
a (newly routed) record with `loadPhraseChunks`; they are inert only because no
`keyed_compose_*` name has a forkbuilt pin today.

## I-1 (Important) — T2 routes only `gui/` but gates `md/`, so T2 cannot go green

T2: "Add `loadVectorRecord` and route **every** `keyed_*.conformance.json` read in
`gui/` through it … then add the structural test per D5b/D5c/D5d: a substring scan
over `gui/*_test.go` **and** `md/*_test.go`".

The md-side loader appears only in D5e's prose and in no task step. Executed: with
every `gui/` read routed, `md/` still holds six occurrences, five of them outside the
one exempt file, and the gate T2 itself adds fails on them:

```
../md/compose_pkh_emit_test.go:37:43
../md/compose_stubs_test.go:15:43
../md/compose_vectors_pin_test.go:158:41      (the D5e exemption)
../md/conformance_keyed_test.go:44:67
../md/conformance_keyed_test.go:49:11
../md/conformance_keyed_test.go:54:48
```

**At T2, an implementer must decide whether "route every read in `gui/`" also means
"and in `md/`", and the plan's task text says `gui/` while its gate says both.**

## I-2 (Important) — nine of the sixteen occurrences are not record reads at all, and the plan gives them no disposition

D5b: "failing on any occurrence." D5e: "the boundary admits exactly one exemption".

Executed on the untouched tree, the gate flags 16 (the plan's count is exactly
right — see the Verified column below). Classified:

| class | n | sites |
| --- | --- | --- |
| record READS — routable | 6 | `gui/composer_policy_address_test.go:48`, `gui/key_card_seating_test.go:40,243`, `gui/policy_address_test.go:261`, `md/compose_pkh_emit_test.go:37`, `md/compose_stubs_test.go:15` |
| the D5e exemption | 1 | `md/compose_vectors_pin_test.go:158` |
| corpus-enumeration GLOBS | 4 | `gui/policy_address_test.go:125`, `gui/taproot_script_path_test.go:30`, `gui/wsh_script_emit_test.go:31`, `md/conformance_keyed_test.go:44` |
| `strings.TrimSuffix` name derivation | 4 | `gui/policy_address_test.go:164`, `gui/taproot_script_path_test.go:40`, `gui/wsh_script_emit_test.go:40`, `md/conformance_keyed_test.go:54` |
| a `t.Fatal` MESSAGE | 1 | `md/conformance_keyed_test.go:49` — `"no keyed_*.conformance.json vendored — the cross-language gate is checking NOTHING"` |

Routing eliminates 6. Nine remain, none of which is a read, and every one of them
still fails the gate. **At T2, an implementer must decide the fate of nine
occurrences the plan never mentions** — move the enumeration into the loader's file,
reword a diagnostic so it stops naming the file it is about, or add exemption
markers the plan says may not exist ("the boundary admits exactly one exemption").
Each choice is invention, and they produce materially different code.

## I-3 (Important) — "outside the loader's own file" is file-granular, and the plan's own phrasing puts the loader inside a split site

D5: "a `loadVectorRecord(t, name)` helper **mirroring `loadVectorChunks`**".
`loadVectorChunks` is defined at `gui/taproot_script_path_test.go:136` — the same
file as D5a's second split site (glob `:30`, card `loadVectorChunks` at `:55`).

Reproduced, by pointing the gate's loader-file exclusion at that file:

```
loader in a new file:                    16 unrouted string token(s)
loader beside loadVectorChunks:          14 unrouted string token(s)
   (taproot_script_path_test.go:30 and :40 vanish; the file is never examined)
```

The most natural reading of "mirroring `loadVectorChunks`" — put it next to the
thing it mirrors — makes the gate structurally blind to one of the three sites it
was built for. **At T2, an implementer must decide where `loadVectorRecord` lives,
and the plan's exclusion rule makes that choice load-bearing without saying so.**
A per-function or per-line exclusion would not have this property.

## I-4 (Important) — D5c's self-test proves the matcher, never the walk

Covered under A/Q2 with the 281-vs-245 reproduction. Restated as an implementability
gap: **at T2, an implementer must decide what "a plausible number of files" means,
and one aggregate threshold satisfies it with an entire package missing.** `gui/`
alone supplies 244 of the 281 files (87%), so any threshold an implementer would
plausibly pick — the precedent's 20, or 200, or 240 — is met by `gui/` alone. The
assertion that would close it (a per-directory count, or a synthetic known-bad
placed as a FILE inside each globbed directory so the walk itself is exercised) is
not what D5c asks for.

## I-5 (Important) — T3 names two edits; the directory-scan half of the pin test keeps covering only the old tier

T3: "update `md/compose_vectors_pin_test.go:103` (36 hardcoded names in
`composeVectorNames`) and `:110-111` (the literal `176`) to the widened set."

`isComposeVectorFile` (`md/compose_vectors_pin_test.go:79-84`) still returns false
for anything not prefixed `compose_` / `keyed_compose_`, and it is what drives the
directory scan — the half the file's own comment says exists because "a
compose-named file that reached the tree without an entry in the pin is the 'copied
in without a name' case (tests-lens C-1), and the pin alone cannot see it."

Reproduced after performing T3 exactly as written (script widened, pin regenerated,
both literals updated, test green) and then smuggling two unpinned files in:

```
keyed_compose_smuggled.conformance.json  -> FAIL "in testdata/vectors but not in the provenance pin"
keyed_tr_smuggled.conformance.json       -> not flagged; test passes
```

So the 14 vectors T3 newly pins — the tier this whole cycle is about, including all
three F-529 names — gain sha256 coverage and gain no directory coverage. **At T3, an
implementer must decide whether widening the pin also widens `isComposeVectorFile`,
and the plan enumerates exactly two edits.**

---

## Minor

**M-1 — the scan's stated mode cannot see its own exemption marker.** D5b commits to
`go/scanner` "in its default mode … skips comments", and D5e's exemption is a
comment (`//go:vectortier vendored`). The gate needs a second read (raw text, or
`scanner.ScanComments`) to honour it. Implementable, but unstated. Separately, a
naive `strings.Contains(src, "//go:vectortier vendored")` matches the gate's OWN
source — observed: my implementation reported `honoured 1 marker` on a tree where
`md/compose_vectors_pin_test.go` carried none, the hit being the gate file itself.
(Verified separately that the marker is `gofmt`- and `go vet`-clean, so T4 is
unaffected: `gofmt -l md/compose_vectors_pin_test.go` empty, `go vet ./md/` exit 0.)

**M-2 — "reports" is not "asserts".** D5e: "the gate reports how many markers it
honoured so a second one cannot appear unnoticed." A `t.Logf` in a passing test is
invisible; only `if markers != 1 { t.Errorf }` delivers the stated property. An
implementer must choose, and the two readings differ materially.

**M-3 — `loadVectorRecord`'s return type is unspecified.** "mirroring
`loadVectorChunks`" implies raw bytes (`loadVectorChunks` returns `[]string`), but
the seven call sites unmarshal into four different anonymous struct shapes. `[]byte`
and a shared decoded type both work and produce very different diffs across seven
files.

**M-4 — T2's `-run` filter depends on a test name the plan never gives.** T2's gate
is `-run 'Duplicate|Pinned|ReachesAnAddress|TaprootScriptPathMatchesRust|WshWitnessScriptHashesToRustsAddress|VectorRecord'`.
The structural test is referred to only as "a structural test" / "the structural
test per D5b/D5c/D5d". A name not containing `VectorRecord` makes T2's gate report
`ok` without ever running the new gate — the "a filtered run that matches nothing
says ok" trap.

**M-5 — T3's two new numbers are not stated. They are 246 and 50.** Measured by
performing T3 for real: `scripts/vendor-compose-vectors.sh:16` widened to
`ls | grep -E '^(keyed_|compose_)' | grep -v '^compose_refusal_' | sort`, run against
`b2c5d693`:

```
vendored 246 files, 50 vectors, primary b2c5d6938432
compose_vectors.provenance.json: file_count 246  vectors 50
```

So `md/compose_vectors_pin_test.go:110` becomes `if len(p.Files) != 246`, and
`composeVectorNames` grows from 36 to 50 by adding exactly these 14:
`keyed_tr_depth2`, `keyed_tr_depth2_rightspine`, `keyed_tr_keyonly`,
`keyed_tr_multi_a`, `keyed_tr_pathological`, `keyed_tr_sortedmulti_a`,
`keyed_tr_with_leaf`, `keyed_wpkh`, `keyed_wsh_multi_2of3`, `keyed_wsh_or_b`,
`keyed_wsh_or_d_degrading`, `keyed_wsh_sortedmulti_2of3`, `keyed_wsh_thresh`,
`keyed_wsh_timelock_hashlock`. Arithmetic check: 176 + 14×5 = 246; 36 + 14 = 50. With
both edits applied, `TestComposeVectorsMatchTheirProvenancePin` and
`TestEveryKeyedComposeVectorHasAConformanceRecord` both PASS. Filed Minor rather than
Important only because the script prints both numbers at the moment they are needed
— but the plan's own standard is that an implementer must not guess, and the comment
at `:110-111` ("32 keyed vectors carry five files, 4 unkeyed carry four: 176") must
be rewritten too, which no edit instruction covers.

---

## Nit

**N-1 — `import "go/scanner"` does not compile in package `gui`.** `gui/scan.go:17`
already declares `scanner`:

```
gui/scan.go:17:6: scanner already declared through import of package scanner ("go/scanner")
	gui/f630_vector_boundary_test.go:4:2: other declaration of scanner
```

The import must be aliased. Caught by the compiler in seconds; noted only because
D5b prescribes `go/scanner` and D5d puts the gate in package `gui`.

**N-2 — a measurement trap in the list T3 must edit.** `composeVectorNames` really
does hold 36 names (the plan is right), but a naive
`grep -o '"[^"]*"'` over the declaration returns 37: a comment *inside* the composite
literal quotes the phrase `"a stale pin hides the next defect"`. Anyone re-counting
that list mechanically should count `ast` elements or trust the pin's `vectors`
field, not grep.

---

## Verified — plan claims that hold

| claim | verdict |
| --- | --- |
| D5b: 18 occurrences across 9 files, 2 in comments, 16 in code | **TRUE**, exact. `grep -c` = 18; `go/scanner` token scan = 16; the two comment lines are `md/compose_vectors_pin_test.go:151`, `md/conformance_keyed_test.go:12`; files = 9 |
| T1 acceptance: "44 of 46 fail, 2 pass" | **TRUE.** D1 over the stale corpus: PASS 2 (`keyed_tr_keyonly`, `keyed_wpkh`), FAIL 44 |
| T3: `b2c5d693`'s corpus satisfies D1 | **TRUE.** PASS 46 / FAIL 0 |
| T3: "0 files added or removed (247 both sides)" | **TRUE.** `^(keyed_\|compose_)` = 247 in both trees; the copy set is 246 (the 247th, `compose_refusal_keyless_cap.json`, is carried by its own pin, per D4) |
| D4: the script selects 177 against a pin of 176 | **TRUE.** 177 files / 37 names selected; pin holds 176 / 36 |
| D3's allowlisted Go paths `m/84h/0h/0h`, `m/86h/0h/0h` | **TRUE.** `canonicalOrigin`, `md/md.go:1103-1111` |
| T4: `gofmt -l .` five-file baseline | **TRUE** at `95716e97`: `gui/transaction.go`, `gui/transaction_golden_test.go`, `gui/transaction_txrecord_test.go`, `mt/mt.go`, `mt/mt_test.go` |
| Probe 1 on `md/compose_pkh_emit_test.go:37` and `md/compose_stubs_test.go:15` | **SAFE to route today.** Both pair the record with `loadPhraseChunks`; their vectors (`keyed_compose_wsh_*`, `keyed_compose_tr_nums_three_leaves`, `keyed_compose_wsh_sole_sortedmulti`) have no forkbuilt pin, so a pin-preferring record falls through. The occurrence that is NOT safe is `md/conformance_keyed_test.go:44` — see C-2 |
| Probe 4: is T1 implementable at T1? | **YES.** `vectorChunksFor` and the two tr `.md1.txt` pins already exist at `95716e97`, and the pins are asserted byte-identical to the vendored cards, so T1's card side is inert and its 44/2 acceptance holds with a plain vendored record read. The dependency runs the other way: **T2 breaks T1** (C-1), and T1's "once D5 lands, the pin-preferring record too" is an un-scheduled edit that belongs to T2 and appears in no T2 step |

## Out of scope, per the brief

Earlier rounds' findings were not re-verified. No tracked file in either repo was
modified; both trees and the worktree list verify clean.
