# R0 round 5 — mechanical fold-vs-findings verification, `e1baf3d`

**Target:** `IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md` and
`SPEC_descriptor_input.md`, both at `e1baf3d` (fold of PLAN-r4, which also
carries a record correction to `13188e3`'s commit message).
**Question, as briefed:** proportional close — re-run r4's three named
commands and disposition its four Importants and eight Minor/Nit findings,
not a fresh audit.
**Reviewer:** independent context. Read-only throughout — `git status
--porcelain` confirmed nothing modified beyond this report; the plan/spec
diff `e1baf3d..HEAD` is empty (verified).

---

## Counts

| severity | r4 disposition this round |
| --- | :-: |
| **Critical** | 0 → **0** (none open) |
| **Important** | 4 → **0 open, all 4 CLOSED** |
| Minor | 6 → **0 FIXED, 6 OPEN** (untouched by this fold, none blocking) |
| Nit | 2 → **0 FIXED, 2 OPEN** (untouched, non-blocking) |

**GREEN — 0 Critical / 0 Important.**

---

## The four Importants

**NEW-I1 (tier/parenthetical contradiction) — CLOSED.** Read spec 1178–1201
directly. The tier rule (1178–1182) now reads: *"A wallet NO path admits
gets the PARTIAL block — including a conjunct-8 failure, RE-DECIDED per
PLAN-r3's I4 … so a compare prompt would PASS on an impossible wallet."*
The parenthetical (1195–1200) now reads: *"A conjunct-8-PASSING `multi`
input in the window is FULL-tier … a conjunct-8-FAILING `multi`, the
colliding-origin twin, is PARTIAL like every conjunct-8 failure — the
compare prompt would pass byte-identically on the impossible wallet
(PLAN-r4's NEW-I1, aligning this parenthetical with the rule above it)."*
Both statements now agree: clause 8's colliding-origin `wsh(multi(…))`
twin fails conjunct 8 → PARTIAL, closing the constructed failure (a
compare prompt passing on an impossible wallet before the permanent
refusal fires). No remaining reading gives FULL to a conjunct-8 failure.

**NEW-I2 ("34 rows" stale at plan L164) — CLOSED.**
```
$ grep -n "34 rows" design/IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md
(no output, exit 1)
```
P3.1 (plan L163–164) now reads *"36 rows"*, matching P2.4's *"all 36
rows"* and §6's 36 data rows. `plan-fold-sweep.sh` re-run with r4's exact
8 terms: `conjunct 7a`, `87`, `70`, `pair`, `b949d18`, `35 rows`, `34
rows`, `36 gate` → **7 gone**, **1 survivor** (`pair` at plan L48 — the §7
overlap pair, confirmed legitimate by r4 and re-read here; it is not the
superseded "impossible-wallet pair" phrasing, which is now "trio"
throughout).

**NEW-I3 (anchors block cites a nonexistent `src/encode.rs`) — CLOSED to
the fold's stated scope.**
```
$ ./scripts/plan-cite-gate.sh design/IMPLEMENTATION_PLAN_descriptor_input_S1_S3.md
  FAIL  crates/md-codec/src/encode.rs:118    no such file
  ok    <all 7 other citations>
  1 unresolvable citation(s) -- fix before review
```
Only **one** FAIL now (down from r4's two) — the gate does not recognize
the `:120` shorthand as a second citation, so this is the sole reported
FAIL, and it is the cross-repo one the fold commit message names as
"known … by design." Verified the line content directly:
```
$ sed -n '118p;120p' /scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/encode.rs
    crate::validate::validate_origin_key_consistency(d)?;
    crate::validate::validate_no_duplicate_key_slots(d)?;
```
Both are the two `validate_` calls the anchors block claims. Bonus check
(not required by the brief but cheap): `plan-staleness-check.sh` against
`descriptor-mnemonic` at `6864f377` now resolves **1** citation (up from
r4's 0), confirming the path correction also fixed the staleness gate's
own stated purpose for that repo, not just the cite gate.

**NEW-I4 (fold commit message was a byte-identical copy of an unrelated
commit's) — CLOSED.** `git log -1 --format=%B e1baf3d` opens with a
"RECORD CORRECTION" naming `13188e3`'s message as wrong and stating what
`13188e3` actually contains. Checked every clause against `git show
13188e3` (179-line diff, read in full):
- "ten renumber/recount sites (conjunct 8, 37/88/71, trio)" — ✓, `conjunct
  7a`→`conjunct 8` at all sites, `87`→`88`, `70`→`71`, `36`→`37` gate,
  "pair"→"trio".
- "the 6 key-identity row split in two" — ✓, one row for
  same-origin/different-xpub, a new row for same-key-twice.
- "the clause-8 multi twin scoped to md1" — ✓, §7 clause 8 gains "whose
  conjunct-8 refusal binds the `--as md1` path ONLY."
- "conjunct-8 failures re-decided PARTIAL" — ✓, the §5.4 tier rule change.
- "P1.1 refusing with 6's rows" — ✓, P1.1's `admit.rs` bullet.
- "the load-bearing anchors block" — ✓, new section added.
- "header tense/arithmetic fixes" — ✓, both plan and spec headers.
`git log -1 --format='%an <%ae>' e1baf3d` → `bg <goss.brian@gmail.com>`;
this is the in-place rewrite the r4 remedy proposed (master unpushed in
the window). The correction is truthful.

---

## r4's Minors and Nits — disposition

None of r4's six Minors or two Nits were touched by this fold; the diff
`13188e3..e1baf3d` is exactly three edits (P3.1's row count, the anchors
path, the §5.4 parenthetical). All eight are re-confirmed present,
verbatim, and **OPEN, non-blocking** — recorded here as P0 implementer
notes, not findings:

| # | status | where (re-checked) |
| --- | --- | --- |
| NEW-M1 | OPEN | spec L723: still singular *"§6's key-identity row"* in §4.7 conjunct 8 prose |
| NEW-M2 | OPEN | `design/FOLLOWUPS.md:14642,14645`: still *"conjunct 7a"*, twice |
| NEW-M3 | OPEN | spec L1636: clause 8 still says *"binds the `--as md1` path ONLY"*, `--as`-omitted case still unnamed |
| NEW-M4 | OPEN | spec L1389: duplicate-slot row text unchanged — still omits the "holder can produce two signatures" risk |
| NEW-M5 | OPEN | plan L12: still *"re-pinned at each phase gate to the spec's CURRENT tip"* — read/re-pin ordering unchanged |
| NEW-M6 | OPEN (pre-existing, explicitly out of this round's scope per r4) | `refusal_row` still appears exactly 2× in the spec (L1529, L1639) |
| N1 | OPEN | spec L1186 still an over-long line (colliding-lines/PARTIAL-block sentence) |
| N2 | OPEN | spec ~L1271–1272: closing paragraph still generalises the address-compare claim past PARTIAL-tier |

---

## Propagation sweep

Re-ran `plan-fold-sweep.sh` on the plan with r4's exact 8 terms: **7
gone, 1 survivor** (`pair`, confirmed legitimate — see NEW-I2 above).
Additionally: `grep -c "34 rows\|35 rows"` on both plan and spec → **0** in
each. `grep -n '`src/encode.rs'` (bare, unprefixed) on the plan → **0**
hits, confirming no stray unfixed anchor reference remains.

---

## Process note (non-blocking, outside the four-Important scope)

`git status --porcelain` shows `design/agent-reports/PLAN-descriptor-S1S3-r4.md`
as **untracked** — r4's report has not yet been committed, even though
`e1baf3d` (the fold responding to it) already exists on top of it. This
repeats the exact "persist before fold" ordering violation that r3's own
persist commit (`0d3e3a8`) explicitly owned. It does not affect plan
content and is not scored above, but the controller should commit r4's
report (and this r5 report) before treating the cycle as closed on the
record, not only in content.

---

## Verdict

**GREEN — 0 Critical / 0 Important.** All four of r4's Importants are
closed, verified by direct re-execution of the commands r4 used to find
them (`plan-cite-gate.sh`, `plan-fold-sweep.sh`, `plan-staleness-check.sh`,
`git log`/`git show`), plus direct reading of the composed text for NEW-I1.
The diff `13188e3..e1baf3d` is small (3 edits, 7 insertions / 4 deletions
across 2 files) and introduces nothing new or blocking.

**P0 may dispatch.** Implementer notes carried forward (all non-blocking):
the six Minors and two Nits above (M1–M6, N1–N2), still open and none of
them gate; and the process note that r4's and this r5 report still need a
persist commit before the record is complete.
