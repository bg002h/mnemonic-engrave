# Composer S3 plan -- Task A10 fold verification (targeted, sonnet)

**Question:** does the A10 fold (`0051be733de0d4a8b0060a73e810bf00dba8ca0c`, base `9081222`) apply
the four notes in `design/agent-briefs/composer-S3-A10-fold-notes.md` correctly against the facts,
and did nothing outside Task A10 move? Fold author's report checked against: `design/agent-reports/composer-S3-plan-A10-fold-report.md` (commit `c677c6d`).

No sub-agents spawned. Nothing edited. Nothing committed by this reviewer. No `.jsonl` file read.

---

## Item 1 -- vector names, wrappers, parameters -- VERIFIED

```
$ ls /scratch/code/shibboleth/wt-composer-s0b/crates/md-codec/tests/vectors/ | grep preset | wc -l
30
$ ls .../vectors/ | grep preset | sed -E 's/keyed_compose_preset_([a-z0-9_]+)\..*/\1/' | sort -u
decaying_multisig
hashlock_gated
kofn_recovery
plain_multisig
simple_timelocked_inheritance
tiered_recovery
```
30 files, six archetype names, matches A10's Files bullet verbatim (snake_case, "30 files").

Wrapper per `.template`'s first characters:
```
decaying_multisig:                  wsh(or_i(and_v(v:multi(2,...
hashlock_gated:                     wsh(or_i(and_v(v:pkh(...
kofn_recovery:                      tr(50929b74c1a04954...
plain_multisig:                     wsh(sortedmulti(2,...
simple_timelocked_inheritance:      wsh(or_i(pkh(...
tiered_recovery:                    wsh(or_d(multi(2,...
```
Five `wsh(`, `kofn_recovery` alone is `tr(` -- matches the brief and matches A10's pinned-pairs
table (`tr` x kofn-recovery is the sole `tr` pin; the other five are `wsh`).

Parameters, `crates/md-codec/tests/compose_support.rs:308-330` in the S0b worktree, cross-checked
against A10's defaults paragraph:
- `plain_multisig(Wrapper::Wsh, 2, 3)` -> A10 says "2-of-3". Match.
- `simple_timelocked_inheritance(Wrapper::Wsh, 26280)` -> A10 says `older(26280)`. Match.
- `kofn_recovery(Wrapper::Tr, 2, 3, 26280)` -> A10 says "2-of-3 with `older(26280)`". Match (lowering: `multi_a(2,...)` over 3 keys, `older(26280)`).
- `tiered_recovery(Wrapper::Wsh, 2, 2, 1, 2, 26280)` -> A10 says "2-of-2 then 1-of-2 with `older(26280)`". Match (lowering: `multi(2,@0,@1)` then `multi(1,@2,@3)` under `older(26280)`).
- `hashlock_gated(Wrapper::Wsh, H, 26280)` -> A10 says `older(26280)` and points at the `.template` file for the digest, "never typed from memory". Confirmed the plan text carries no literal digest (grepped the current A10 body; no 64-hex-char string appears in Step 3's defaults paragraph).
- `decaying_multisig(Wrapper::Wsh, 2, 2, 1, 1, 13140, 26280, 1_000_000)` -> A10 says "2-of-2 at `older(13140)`, then 1-of-1 at `older(26280)`, then one key at `after(1000000)`". Match (lowering: `multi(2,@0,@1)` under `older(13140)`, then `pkh(@2)` under `older(26280)`, then `pkh(@3)` under `after(1000000)` -- single-key legs read correctly as "1-of-1"/"one key").

Every parameter and digest reference A10 now quotes matches the S0b worktree byte-for-byte.

## Item 2 -- fork numbers -- VERIFIED

```
$ sed -n '16p' /scratch/code/shibboleth/seedhammer/scripts/vendor-compose-vectors.sh
mapfile -t files < <(cd "$VEC" && ls | grep -E '^(keyed_)?compose_' | sort)
```
Glob is `^(keyed_)?compose_`, which covers `keyed_compose_preset_*`. Matches A10's claim that no
script SELECTION change is needed.

```
$ awk '/var composeVectorNames/,/^}/' md/compose_vectors_pin_test.go | grep -o '"[^"]*"' | wc -l
26
$ grep -n 126 md/compose_vectors_pin_test.go
82:  // 22 keyed vectors carry five files, 4 unkeyed carry four: 126.
83:  if len(p.Files) != 126 {
84:    t.Fatalf("pin lists %d files, want 126", len(p.Files))
```
26 names today, asserts 126 files, and the array is 22 `keyed_*` + 4 non-`keyed_*` (counted
directly from the printed array). Adding six new `keyed_compose_preset_*` names: 26 -> 32 names;
28 keyed x 5 + 4 unkeyed x 4 = 156. A10's Files bullet states exactly "26 -> 32" and
"28 keyed x 5 + 4 unkeyed x 4 = **156**" -- matches.

```
$ ls /scratch/code/shibboleth/seedhammer/md/testdata/vectors/ | wc -l
277
```
A10's Step 1 Expected line states "`md/testdata/vectors` goes from 277 files to 307" (277 + 30
new files) -- 277 measured directly, matches; 307 is the stated arithmetic prediction, correctly
labelled as an Expected line rather than a present-tense fact (S0b has not merged yet, so these
files are not vendored into the fork today).

## Item 3 -- OFFER kept, PINNING narrowed to six pairs -- VERIFIED

Current A10 text (`sed -n '4796,4908p'`) keeps the OFFER sentence verbatim in substance:
> "Which presets are OFFERED depends on the wrapper (§4d), and S0b changed nothing there: all six
> under `wsh` and `tr` ...; under `sh`/`sh(wsh)`, plain k-of-n alone..."

and narrows only the PINNING via the table:
| pairs | check |
|---|---|
| `wsh` x plain-multisig, simple-timelocked-inheritance, tiered-recovery, hashlock-gated, decaying-multisig | PINNED |
| `tr` x kofn-recovery | PINNED |
| `tr` x the other five, and `wsh` x kofn-recovery | STRUCTURAL, named UNPINNED |

Enumerating both test bodies as specified: PINNED sub-tests are `wsh/plain-multisig`,
`wsh/simple-timelocked-inheritance`, `wsh/tiered-recovery`, `wsh/hashlock-gated`,
`wsh/decaying-multisig`, `tr/kofn-recovery` (5 wsh + 1 tr, matching the exported vector set
exactly). STRUCTURAL sub-tests are `tr/plain-multisig`, `tr/simple-timelocked-inheritance`,
`tr/tiered-recovery`, `tr/hashlock-gated`, `tr/decaying-multisig`, `wsh/kofn-recovery` (the
complementary 5 tr + 1 wsh). All twelve offered pairs appear exactly once across the two sets --
no overlap, no gap.

Step 4's Expected line states a concrete count and naming: "three top-level PASS; ... six
sub-tests, one per pinned pair, named wsh/plain-multisig, ...; under the structural test six
more, one per unpinned pair; `ok seedhammer.com/gui`."

The plan states explicitly, twice (§4d block and Step 2's test spec), that a `tr` vector's chunks
are never compared against a `wsh` preset's: "The wrapper is half of an assertion's identity
here... a table iterated at one wrapper against vectors exported at another is a test that passes
by comparing the wrong things" -- and the sub-test naming scheme (`<wrapper>/<preset>`, looked up
"for ITS OWN wrapper") makes that structurally true, not just stated.

## Item 4 -- BLOCKED on S0b, named path, owner+time, Step 1 builds by path -- VERIFIED

Heading: `### Task A10: the five presets (§4d) -- BLOCKED on F-453's Rust half (plan S0b), and the
only task that is`. Body names the mini-plan path:
```
$ ls -la design/IMPLEMENTATION_PLAN_composer_S0b_presets.md
-rw-r--r-- 1 bcg bcg 112417 Sep  2 07:41 design/IMPLEMENTATION_PLAN_composer_S0b_presets.md
```
File exists, matches the citation exactly. Branch named (`composer-s0b`) also confirmed:
```
$ git -C /scratch/code/shibboleth/wt-composer-s0b branch --show-current
composer-s0b
```
Owner and timing for the merge-commit fact: "The controller records it on the line below, on the
day S0b merges to descriptor-mnemonic `main`," followed by a blockquote placeholder line. Owner =
controller, time = day of merge. Matches the brief's "says who records the merge commit and when."

Step 1 builds `md` by path rather than trusting `PATH`:
```bash
git -C /scratch/code/shibboleth/descriptor-mnemonic rev-parse main
cargo build -q --manifest-path /scratch/code/shibboleth/descriptor-mnemonic/Cargo.toml -p md-cli
/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md compose --help 2>&1 | ...
```
with the explicit rationale "A bare `md` on `PATH` may be a stale install". Matches.

## Item 5 -- diff scope, no go fence, plan checks -- VERIFIED

```
$ git diff -U0 9081222..0051be7 -- design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md | grep -c '^@@'
18
```
18 hunks. New-side start lines (from the `@@ -a,b +c,d @@` headers): 4796, 4798, 4806, 4812,
4815, 4818, 4830, 4835, 4842, 4846, 4853, 4857, 4861, 4868, 4879, 4885, 4889, 4895 -- every one
inside A10 (heading at 4796, `### Task A11` heading at line 4909, confirmed by
`grep -n '^### Task A1[01]'`). Hunks outside 4796-4908: 0.

```
$ git diff 9081222..0051be7 -- ...md | grep -n '^+```'
71:+```
74:+```bash
```
Only a plain fence-close and a `bash` fence were added. The one occurrence of the literal string
"```go" in the diff is inside an added prose sentence ("Every other task's code is in a ```go
fence because it can be compiled today; this one cannot...") -- it is text describing other
tasks' convention, not a fence delimiter that was opened. No ```go fence was added inside A10.

```
$ ./scripts/plan-glyph-check.sh design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md
─── operator strings scanned: 306 ; undrawable: 0            exit=0
$ ./scripts/plan-table-check.sh design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md
─── table rows checked: 129 ; malformed: 0                   exit=0
$ ./scripts/plan-stepref-check.sh design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md
─── step numbers in prose: 0                                 exit=0
$ CITE_FORK_ROOT=/scratch/code/shibboleth/seedhammer ./scripts/plan-cite-check.sh design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md
─── citations resolved: 249 / 249 ; dangling: 0 ; ambiguous: 0            exit=0
```
All four gates reproduce the controller's stated figures exactly (306/0, 129/0, 0, 249/249, all
exit 0), run from `/scratch/code/shibboleth/mnemonic-engrave` as specified. Spot-checked several
`ok` citation lines against their printed content (`md/compose.go:288` DefaultOrigin,
`md/compose.go:42` `ScriptType()`, `md/compose_vectors_pin_test.go:39/57/82/84`,
`scripts/vendor-compose-vectors.sh:16/29`) -- each printed line's content matches what A10 claims
is there, not merely a resolved-but-stale line.

```
$ git show --stat 0051be7
 design/IMPLEMENTATION_PLAN_composer_S3_fork_gui.md | 78 +++++++++++++++++-----
 1 file changed, 60 insertions(+), 18 deletions(-)
```
Only this one file changed in the fold commit.

---

## Closing counts

5 / 5 items VERIFIED. 0 Critical, 0 Important, 0 Minor, 0 Nit.

No fact misquoted, no note misapplied, no hunk outside Task A10, no ```go fence added, all four
plan-check gates reproduce the controller's cited figures under independent re-execution.
