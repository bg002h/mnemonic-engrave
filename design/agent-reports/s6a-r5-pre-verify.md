# S6a R5 — cheap pre-review verification pass

Scope: `git diff 25759af..HEAD -- design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
(one commit, 9238f6d, 149-line diff — the fold responding to
`design/agent-reports/s6a-r4-adversarial.md`, 1C/2I). Repo checked against:
`/scratch/code/shibboleth/seedhammer`, `main` @ `b8a23bf` (confirmed clean
`go build ./...`, exit 0, before reading).

## VERDICT: DIRTY — 0 false, 4 stale, 0 table/switch disagreements

The core risk — does the new §4.7d five-site split hold up, and does the
switch still reproduce its own enumerated table — is CLEAN. But the fold
renamed the sticky trigger from `verifyFailed` to `verifyMismatch` in §4.7a's
switch, in P2, and added two new enumerated-table rows for `failed`, and did
**not** propagate that rename into two rows of §5's test-plan table (T10,
T13b) or update the enumerated table's own row-count claim (now stated twice
as "ten rows", actually twelve). One of the two test-row misses (T10) is
serious: as literally worded it now asserts the exact opposite of P3, the
fold's own headline property.

## §4.7d — THE FIVE SITES

| site | plan says | code says | agree? |
| --- | --- | --- | --- |
| `gui/multisig_verify.go:719` | `!slices.Equal(readbackMd1, engravedMd1)` — different wallet, not a comparison | confirmed verbatim: `if !slices.Equal(readbackMd1, engravedMd1) { ... return verifyFailed }` | yes |
| `gui/multisig_verify.go:724` | readback would not decode, not a comparison | confirmed: `_, keys, err := md.ExpandWalletPolicyChunks(readbackMd1); if err != nil { ... return verifyFailed }` | yes |
| `gui/multisig_verify.go:897` | re-typed seed would not derive, not a comparison | confirmed: `b, derr := deriveMultisigLeg(...); if derr != nil { ... return verifyFailed }` | yes |
| `gui/multisig_verify.go:963` | `verifyMultisigLegsPartial` mismatch — a comparison | confirmed: `if _, err := verifyMultisigLegsPartial(legs, readbackMk1s, readbackMd1); err != nil { ... return verifyFailed }` | yes |
| `gui/multisig_verify.go:984` | `verifyMultisigLegs` mismatch — a comparison | confirmed: `if err := verifyMultisigLegs(legs, readbackMk1s, readbackMd1); err != nil { ... return verifyFailed }` | yes |

`grep -n "return verifyFailed" gui/multisig_verify.go` → exactly these 5 lines,
no more, no fewer. No other site in the codebase misclassified in either
direction: `singleSigVerifyFlow` (`gui/singlesig_verify.go:65`) was traced
end-to-end — 10 explicit `return` + 1 fall-through = 11 exits, matching the
plan's own count, of which exactly one (`verifySingleSig` failure) is a
comparison, and the function has no `verifyStatus`-style retry loop, matching
"Single-sig is unaffected."

Quoted comment at `gui/multisig_verify.go:42-48` (`multisigVerifyForeignPolicyBody`)
checked byte-for-byte: *"It says the one thing the operator can act on. A
generic 'Verify Failed' sends them to re-cut plates that are perfectly
good..."* — the plan's excerpt matches verbatim.

## §4.7a — SWITCH vs TABLE, ROW BY ROW

Executed the pseudocode at plan lines 705–718 by hand over all **twelve**
rows of the enumerated table (plan lines 818–829) — note this table grew from
10 to 12 rows in this fold (two new `failed` rows), which the surrounding
prose does not yet reflect (see STALE REFERENCES).

| # | sequence | table says | switch produces | agree? |
| --- | --- | --- | --- | --- |
| 1 | `S` (skip) | `NOT VERIFIED` | loop never runs, `status` stays `statusNotVerified` | yes |
| 2 | `complete` | `VERIFIED` | `sawDisagreement=false`; `res==complete` arm → `statusVerified` | yes |
| 3 | `incomplete` then stop | `DID NOT COMPLETE` | `sawDisagreement=false`; default arm | yes |
| 4 | `refused`/`abandoned` | `DID NOT COMPLETE` | same, default arm | yes |
| 5 | `incomplete` → `complete` | `VERIFIED` | iter2 `res==complete`, `sawDisagreement` still false → `statusVerified` | yes |
| 6 | `mismatch` then stop | `DISAGREED` | `res==verifyMismatch` sets sticky; `sawDisagreement` arm | yes |
| 7 | `mismatch` → `abandoned` | `DISAGREED` | sticky stays true across iter2; `sawDisagreement` arm | yes |
| 8 | `mismatch` → `incomplete` | `DISAGREED` | same pattern | yes |
| 9 | `mismatch` → `complete` | `VERIFIED on a repeat check` | `res==complete && sawDisagreement` both true → `statusVerifiedOnRetry` | yes |
| 10 | `incomplete` → `mismatch` → `complete` | `VERIFIED on a repeat check` | iter2 sets sticky, iter3 both-true arm | yes |
| 11 | `failed` then stop | `DID NOT COMPLETE` | `res==verifyFailed` does **not** set sticky (only `verifyMismatch` does); default arm | yes |
| 12 | `failed` → `complete` | `VERIFIED` (plain) | sticky still false at iter2; `res==complete` arm → `statusVerified`, not the retry arm | yes |

**12/12 agree — 0 disagreements.** Also confirmed independently: control flow
still causes both `failed` and `mismatch` to loop (see below), so rows 11–12
are reachable, not vacuous.

Also confirmed: `verifyStatus`'s 5 constants (§4.7c, unchanged by this fold)
are exactly the 5 values the switch can produce, so no row targets an
undefined status.

## CONTROL FLOW

`gui/multisig.go:337` and `gui/multisig_build.go:453` both currently read,
verbatim:

    if res != verifyIncomplete && res != verifyFailed { break }

matching the plan's "before" quote exactly. The fold's proposed replacement,

    if res != verifyIncomplete && res != verifyFailed && res != verifyMismatch { break }

is logically the same set of 5 physical failure sites (719/724/897/963/984)
continuing to loop — they're just now split across two named codes instead of
one, and the OR-of-three-negations still evaluates false (loop continues) for
every one of the 5 sites. Traced against both call sites; both are genuinely
inside the retry `for {}` loop described (`gui/multisig.go:330-342`,
`gui/multisig_build.go:444-459`), and `verifyAbandoned`/`verifyRefused`/
`verifyComplete` still break in both — unaffected by this change. "Preserves
current looping exactly" is TRUE.

## STALE REFERENCES

Grepped `verifyFailed`, `verifyMismatch`, "ten rows", "condemn", `DISAGREED`
across the whole file. Two real misses, both Important:

**1. T10 (`plan:1073`) — the fold's own headline property, P3, contradicted
by an un-renamed test row.** T10 reads: *"the stickiness. `failed` →
`abandoned` prints `DISAGREED`, not `DID NOT COMPLETE`."* Under the design
this very fold introduces, `failed` never sets `sawDisagreement` (only
`mismatch` does — §4.7d, confirmed above), so `failed` → `abandoned` produces
`sawDisagreement=false`, default arm, **`DID NOT COMPLETE`** — exactly the
row-11 case just added to §4.7a's own table, and exactly what P3 exists to
guarantee ("no sequence prints `DISAGREED` unless a comparison actually ran
and disagreed"). T10 as worded now asserts the opposite of P3 for the same
input. The plan's own prose (`plan:1112-1117`) calls T10 "the sharpest test
in this plan… If T10 does not fail against that [last-wins] implementation,
it is not testing anything" — as currently worded it tests the wrong
sequence. The sequence T10 evidently *means* to test (mismatch's stickiness
surviving an abandon) already exists as row 7 (`mismatch` → `abandoned` →
`DISAGREED`, plan:824); T10 needs `mismatch` substituted for `failed`, or it
needs deleting as now-redundant with row 7's coverage. This is a rename that
was applied to §4.7a's table, to P2, and to the new T15, but not propagated to
T10 — one code path fixed, the other silently left, the exact failure mode
this pre-verify pass exists to catch.

**2. T13b (`plan:1078`)** — still reads *"P2 — a disagreement is never lost.
Every sequence containing `verifyFailed` prints `DISAGREED`… never `DID NOT
COMPLETE`"* — but P2 itself was renamed in this same fold (`plan:792-796`,
diff hunk) to `verifyMismatch`. Literally worded, T13b now directly
contradicts T15 (`plan:1079`, new in this fold) for the identical set of
sequences: T13b says a `failed`-containing sequence must never print `DID NOT
COMPLETE`; T15 says it must never print anything else. Needs `verifyMismatch`
substituted for `verifyFailed`.

**3. The row-count claim, two locations, both now wrong.** `plan:832` ("these
**ten** rows are the complete image of it") and `plan:1077`'s T13a
description ("Table-driven over §4.7a's **ten** rows") both predate this
fold's two new `failed` rows. Counted mechanically:
`awk 'NR==816,829' plan.md | grep -c "^|"` → 14 lines (header + separator +
12 data rows) = **twelve** rows, not ten. Neither location was updated when
the two `failed` rows were added just above/below them in this same fold.
Does not change either property's correctness (P1's "clean pass" set and the
switch/table agreement both hold across all 12), but the stated count is
false in both places it appears.

**Not a finding, checked and clean:** greps for "on `verifyFailed`" as a
sticky-trigger description outside the four items above turned up nothing;
`plan:680/684` still describe `verifyFailed` as one of the two looping
verdicts — this is accurate, unchanged **current shipped code** being quoted
as historical motivation, not a claim about the new design, and matches
`gui/multisig.go:337`'s actual (pre-fold) text.

**Out of this fold's diff, flagged anyway (pre-existing, not introduced
here):** T12 (`plan:1075`) reads *"`incomplete` → `complete` prints the
repeat-check line, not `DID NOT COMPLETE` and not bare `VERIFIED`"* — this
contradicts row 5 of §4.7a's own table (`plan:822`: `incomplete` → `complete`
→ plain `VERIFIED`, false/complete/VERIFIED) and the prose immediately above
the table explaining why (`plan:733-738`, "closes R2 C-2… prints `VERIFIED`").
`git log -p` shows T12 was last rewritten in an earlier round (before
eb9df42/d78016e) and untouched by 25759af..HEAD; neither the R4 pre-verify
(`s6a-r4-pre-verify.md`) nor the R4 adversarial review mention T12 or T10.
Reporting since it's mechanically checkable and live in the current artifact,
but it is not something this fold created or was asked to fix.

## BUILD ORDER + I-1

**Build order.** The changed paragraph (`plan:971-983`) claims steps 1–4 and
8–9 leave the tree green, steps 5–7 land together. Checked against the
(unchanged-by-this-diff) step table: row 6's cell already reads "must
accompany step 5, not follow it" (`plan:992`) — matches the paragraph's own
citation of that cell. Row 8 ("independent; deliberately last") and row 9
("in its own commit") are consistent with "green and independently landable."
No cell contradicts the new header. (Sanity-checked in passing, since the
brief asked: step 2's content — `verifyStatus` + `buildVerifyStatusLines` +
T14 only — matches what §4.7b/§4.7c actually define; not part of this fold's
diff, unchanged, self-consistent.)

**I-1.** `grep -n "Re-verify\|re-verify"` over the whole plan → zero hits;
the word is gone. The replacement line (`plan:845`) —
`WARNING: a read-back check DISAGREED with these plates. Do NOT rely on this
backup: engrave a fresh set and check it before use.` — is the only
occurrence of that sentence in the file (no stale duplicate elsewhere) and
`LC_ALL=C grep -P '[^\x00-\x7F]'` over that exact line returns nothing: ASCII
clean.

## Notes on scope

- Did not review design quality, re-litigate the split/ONE-PIECE/status-line
  decisions, or audit the codebase for new defects.
- Did not re-check `file:line` citation existence (gate already ran, per the
  brief).
- `go build ./...` on the fork at `main`@`b8a23bf` was run once as a sanity
  check before reading any cited line (exit 0) — the code being cited is a
  real, currently-compiling baseline, not a broken WIP.
