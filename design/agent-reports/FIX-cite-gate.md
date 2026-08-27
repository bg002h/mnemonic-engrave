# FIX-cite-gate — three defects in `scripts/plan-cite-check.sh`

Worktree: `/scratch/code/shibboleth/_work/citegate/mnemonic-engrave`, branch
`fix/cite-gate`, based on `6c24e62`. All measurements below were taken by
running the actual script (`bash scripts/plan-cite-check.sh ...`), never by
reading it and reasoning about what it "should" do.

## F-286 — leading dot stripped, `.github/workflows/...` reported false DANGLING

**Root cause.** The grep pattern `(\./)?[A-Za-z0-9_][A-Za-z0-9_./-]*\.(...)`
requires the first captured character to be `[A-Za-z0-9_]`. For a citation
starting `.github/workflows/x.yml:N`, POSIX ERE cannot match the leading `.`
against that class or against the `(\./)?` group (which only matches the
literal two-character `./`), so `grep -oE` silently starts the match one
character later, at `g`. The captured citation is `github/workflows/x.yml:N`
— the dot is gone from the STRING the script resolves, not from the file on
disk — so the downstream lookup searches every root for a top-level `github/`
directory that does not exist, and reports a real file DANGLING.

**Measured before** (control doc, real file `.github/workflows/release.yml`,
465 lines):
```
DANGLING  github/workflows/release.yml:1                        (no such file under any root)
```

**Fix.** Added an optional single leading dot to the pattern, tried after the
existing `(\./)?` group:
```
grep -oE '(\./)?\.?[A-Za-z0-9_][A-Za-z0-9_./-]*\.(go|rs|sh|md|toml|yml|yaml|tsv):[0-9]+([,-][0-9]+)*'
```

**Measured after:**
```
ok  .github/workflows/release.yml:1                       name: release
```

**Mutation test** (must stay DANGLING — proves the fix didn't loosen the gate):
```
DANGLING  .github/workflows/does-not-exist.yml:1                (no such file under any root)
DANGLING  .github/workflows/release.yml:99999                   (file has 465 lines)
```
Both still fail correctly after the fix.

## F-296 — `mnemonic-gui` not in `ROOTS`

**Collision check, done independently** (not taken on the author's word).
Enumerated every file with a tracked extension (`go rs sh md toml yml yaml
tsv`) under `mnemonic-gui` (276 files) and diffed the full relative-path set
against each of the 7 existing roots. Overlap found: `Cargo.toml`, `CLAUDE.md`,
`README.md`, `CHANGELOG.md` — and a second check confirmed each of those four
already existed under 5-7 of the *other* roots before `mnemonic-gui` was
added, i.e. they were already `AMBIGUOUS`; a 6th-8th hit changes no citation's
classification. No previously-unique path collides. `mnemonic-gui` added
after `mnemonic-secret`, before the `mnemonic-transaction` root (whose
ordering-sensitive comment is unaffected — ambiguity detection loops over
every root regardless of position).

**Measured before** (real file `mnemonic-gui/src/lib.rs`, 21 lines):
```
DANGLING  mnemonic-gui/src/lib.rs:1                             (no such file under any root)
```

**Measured after:**
```
ok  mnemonic-gui/src/lib.rs:1                             //! mnemonic-gui — library crate root.
```

**Mutation test:**
```
DANGLING  mnemonic-gui/src/does_not_exist.rs:1                  (no such file under any root)
DANGLING  mnemonic-gui/src/lib.rs:99999                         (file has 21 lines)
```
Both still fail correctly.

**Side effect found by the corpus regression sweep (F-297, see below).**
Adding any new root risks turning an already-broken BARE citation elsewhere
in the corpus from a loud DANGLING into a silent wrong-file `ok`, if the new
root happens to carry a same-named file at its own top level. Found live in
`design/CONTINUITY_2026-08-07.md:86`. Not a reason to withhold this root —
see F-297 for the full analysis and why it doesn't touch the three assigned
plans.

## `.tsv` not a tracked extension

**Measured before**, using a real, pre-existing file
(`mnemonic-toolkit/cycle-b-audit-queue.tsv`, 827 lines): a control doc
containing a valid, in-range `.tsv` citation, a past-EOF `.tsv` citation, and
a `.tsv` citation to a nonexistent file — **none of the three appeared in the
script's output at all.** Total citation count for a 10-line control doc was
7, not 10. Confirmed: **silently skipped, not reported dangling** — the more
serious of the two possible failure modes, exactly as the fix brief
predicted, because a citation nobody sees is a citation nobody ever checks or
fixes.

**Fix.** Added `tsv` to the extension alternation (see F-286's diff above —
same line).

**Measured after:** all 3 `.tsv` lines now appear; total count for the same
control doc rose from 7 to 10.
```
ok  mnemonic-toolkit/cycle-b-audit-queue.tsv:1            file	line	token	class_hint	fastpath	suggested_anchor
DANGLING  mnemonic-toolkit/cycle-b-audit-queue.tsv:99999        (file has 827 lines)
DANGLING  mnemonic-toolkit/does-not-exist.tsv:1                 (no such file under any root)
```
Positive resolves; both negatives still fail correctly.

## Combined mutation-test run (the actual gate, not a description of it)

Control doc: 10 citations (3 per defect + 1 sanity baseline
`crates/me-cli/Cargo.toml:1`), 6 of which are deliberate negatives that must
never resolve.

Before fix: `1 / 7` resolved (the 3 `.tsv` lines invisible; both other
positives DANGLING). Exit 1.

After fix: `4 / 10` resolved — **every intended positive is `ok`, every
intended negative is still `DANGLING`**, and 3 previously-invisible citations
are now visible and checked (not silently skipped). Exit 1 (expected — the
control doc contains 6 deliberate negatives).

## The three real plans — before/after, run individually

The brief named `design/IMPLEMENTATION_PLAN_P{1,2,3}_*.md`; the worktree
actually holds four P-prefixed plans (P0, two P1s, P2 — no P3 file exists
yet). Ran all four, individually, with the saved pre-fix script and the
fixed script:

| Plan | exit before | resolved before | exit after | resolved after |
|---|---|---|---|---|
| `IMPLEMENTATION_PLAN_P0_mnemonic_io_lib.md` | 0 | 19/19, dangling 0, amb 0 | 0 | 19/19, dangling 0, amb 0 |
| `IMPLEMENTATION_PLAN_P1_me_container.md` | 1 | 98/109, dangling 8, amb 3 | 1 | 98/109, dangling 8, amb 3 |
| `IMPLEMENTATION_PLAN_P1_mt_adopts.md` | 0 | 41/41, dangling 0, amb 0 | 0 | 41/41, dangling 0, amb 0 |
| `IMPLEMENTATION_PLAN_P2_ms_adopts.md` | 0 | 29/29, dangling 0, amb 0 | 0 | 29/29, dangling 0, amb 0 |

**Byte-identical before and after for all four** — none of these plans
currently contain a `.github/...`, `mnemonic-gui/...`, or `.tsv` citation, so
the three fixes are neutral on this exact set. `P1_me_container`'s 8
dangling / 3 ambiguous are pre-existing and unrelated: 8 citations into the
vendored `bitcoin-0.32.9` crate source (not a tracked root — out of this
gate's scope by design) and 3 `AMBIGUOUS design/SPEC_mt_v0_1.md` hits (the
known cross-repo duplicate the `mnemonic-transaction`-root comment already
documents). Not touched by this fix.

## Full `design/*.md` corpus sweep (231 docs) — done as extra diligence

Ran the saved pre-fix script and the fixed script over every `design/*.md`
doc (not just the four plans) and diffed the DANGLING/AMBIGUOUS lines.
Combined totals: before `3535/4627 resolved, dangling 1085, ambiguous 7`;
after `3537/4627 resolved, dangling 1082, ambiguous 8` — same 4627 total
citations (corpus content unchanged before this diff was taken), resolved
+2, dangling -3, ambiguous +1. Reconciled line-by-line against the 6-line
diff below: test.yml (+1 resolved, -1 dangling), ci.yml (-1 dangling, +1
ambiguous, resolved unchanged), FOLLOWUPS.md:59 (+1 resolved, -1 dangling);
README.md, src/lib.rs:503 and FOLLOWUPS.md:1762 contribute zero net count
change (bucket-internal reason-text changes only). Sums to exactly the
measured deltas. All 6 explained:

1. **`.github/workflows/test.yml:29`** (cited twice, at lines 35 and 179 of
   `IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md`, but the two
   occurrences are identical strings and dedupe to one counted citation via
   the script's own `sort -u`): DANGLING → `ok`, resolving against the
   fork's real `.github/workflows/test.yml` (135 lines) — a genuine F-286
   fix, not a side effect.
2. **`.github/workflows/ci.yml:16`** (`design/FOLLOWUPS.md:12398`): DANGLING
   → `AMBIGUOUS (exists under 3 roots)` — `ci.yml` genuinely exists in
   `descriptor-mnemonic`, `mnemonic-key`, and `mnemonic-transaction`. F-286
   was previously *hiding* this real ambiguity behind a wrong DANGLING
   message; now it correctly tells the author to qualify the path. An
   improvement, not a regression.
3. **`README.md:269`**: `AMBIGUOUS (7 roots)` → `AMBIGUOUS (8 roots)` —
   `mnemonic-gui` adds an 8th hit to an already-ambiguous, already-flagged
   citation. No classification change.
4. **`src/lib.rs:503`** (cited twice in `design/FOLLOWUPS.md`, at lines 4387
   and 4414 of the `MAX_RECURSION_DEPTH` entry, deduped to one counted
   citation): DANGLING (no file) → DANGLING (file has 21 lines) —
   `mnemonic-gui/src/lib.rs` is now a real, bare-resolvable file,
   but line 503 is still out of its 21-line range. **Still correctly
   flagged**, but sits on a landmine — see F-297.
5. **`FOLLOWUPS.md:1762`** (`design/DESIGN_b2b_residency_zeroing.md:850`):
   same shape as #4 — DANGLING (no file) → DANGLING (`mnemonic-gui/FOLLOWUPS.md`
   has 1174 lines, 1762 is still past EOF). Still correct today; same
   landmine class.
6. **`FOLLOWUPS.md:59`** (`design/CONTINUITY_2026-08-07.md:86`): DANGLING →
   **`ok`, against `mnemonic-gui/FOLLOWUPS.md:59`** — the one live instance
   of the landmine. Filed as **F-297** (below), not silently fixed: the
   citation was already incomplete/stale before this session (missing its
   `design/` prefix, and even with the prefix restored it points at
   unrelated content in *this* repo's own `design/FOLLOWUPS.md` — an F-279
   content-drift problem, not a script bug). Documented in the script's own
   "NOT covered" block and in FOLLOWUPS.md; not patched, because the
   available "fixes" are either historical-content archaeology (outside this
   task) or a resolution-algorithm design change (explicitly out of scope
   per the fix brief — "do not build the anchor feature" / "don't build
   anything clever").

No other lines changed across 231 docs; nothing else newly dangled, nothing
else newly resolved.

## Quiet on non-defects

The control doc and all four target plans produce exactly the expected
`ok`/`DANGLING` classification with no extraneous findings. The one new
`AMBIGUOUS`/false-`ok` interaction found (F-297) is real and reported, not
suppressed.

## "NOT covered" block

Updated both the header comment and the runtime footer:
- Strengthened the F-279 (interpretation) bullet with the measured P1
  citation-drift numbers, so it reads as a measured fact rather than a
  disclaimer.
- Added a bullet stating the tracked-extension list explicitly and noting
  that an untracked extension is invisible, not dangling — the same failure
  class `.tsv` had, now general rather than closed by this one fix.
- Added a new bullet for the F-297 class: `AMBIGUOUS` only fires on a
  citation-time 2+-root collision, so a new root can silently absorb an
  already-broken bare citation from elsewhere in the corpus.
- Runtime footer (`─── NOT covered: ...`, printed on every run) now states
  the F-279 measurement and the untracked-extension gap directly, not just
  in the header comment a reader may not open.

## Follow-up filed

**F-297** in `design/FOLLOWUPS.md` (next free number per the dispatch brief):
the general class behind item 6 above — a new `ROOTS` entry can silently
convert an already-incomplete bare citation elsewhere in the corpus from a
correctly-negative DANGLING into a wrong-file `ok`, because the `AMBIGUOUS`
check only fires on multi-root collisions, not on a new single root
coincidentally matching a bare, unqualified, already-broken citation. Records
the one live instance and the two dormant ones, and offers two undeveloped
options (require qualification for generic top-level names, generalizing the
existing repo-qualified-prefix mechanism to single-hit cases; or accept the
risk and only document it) for an architect to decide later — not built here.

The F-297 entry itself was written, then re-checked against the gate: its
first draft **quoted the three broken example citations in gate-matchable
`path:N` form**, which caused the gate to pick them up as new citations of
its own (the same self-referential trap F-286's entry explicitly calls out
and avoids). Rewrote the illustrative examples to break colon-digit adjacency
("`FOLLOWUPS.md` line 59" instead of `` `FOLLOWUPS.md:59` ``) and qualified
the two real, intentional example citations
(`mnemonic-engrave/design/FOLLOWUPS.md:59`,
`mnemonic-engrave/design/FOLLOWUPS.md:4387`) that were colliding with the
pre-existing, unrelated 5-root `design/FOLLOWUPS.md` ambiguity. Re-ran the
gate on `design/FOLLOWUPS.md` alone after the rewrite: all 5 remaining
citations in the new entry resolve `ok`, none ambiguous, none dangling.

## What this does NOT fix (stop-and-report, not silently patched)

The F-297 live instance (`design/CONTINUITY_2026-08-07.md:86`) is not
corrected, because the "fix" available to a script-only task — restoring the
`design/` prefix — does not make the citation *correct*, only differently
wrong (it would point at a different, unrelated `design/FOLLOWUPS.md` entry
in this repo, per the F-279 content-drift blind spot). Fixing it for real
needs someone to identify which followup entry the 2026-08-07 estimate
correction actually meant. This is exactly the kind of thing this task's
brief says to stop and report rather than patch over.

## Final state

- `git diff --stat`: `design/FOLLOWUPS.md` (+65/-0), `scripts/plan-cite-check.sh`
  (+56/-10) — the only two files touched.
- `bash -n scripts/plan-cite-check.sh`: syntax OK.
- Full `design/*.md` corpus with the finished script: `3542/4632 resolved,
  dangling 1082, ambiguous 8`, exit 1 — expected and pre-existing; this
  corpus was never claimed clean (documented in the script's own CI-note:
  "a 12-doc sample ... found 13 dangling citations across 7 of them").
