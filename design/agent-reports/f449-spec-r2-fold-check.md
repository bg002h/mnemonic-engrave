# F-449 spec r2 — fold verification (mechanical)

**Verdict: 0 findings unaddressed / 1 new defect found.**

Reviewer: sonnet, mechanical fold-check only. No design merits reviewed.
Diff verified: `git diff 946fb24d..a1136d97 -- design/SPEC_liana_unspendable_internal_key.md`
(188 lines). Baseline confirmed live, both clean trees, exactly matching the
spec's `**Baseline:**` line:

```
$ git -C descriptor-mnemonic log -1 --format=%H  → 6cbd49d80a657e4068e1c509f815330437752a6b (spec cites 6cbd49d8)
$ git -C seedhammer log -1 --format=%H           → 7b6f2fbbdb909ef812ccc8c90768280c5fa7a4d4  (spec cites 7b6f2fb)
```

`md 0.17.0` installed, matching both r1 reports' stated version.

---

## Checklist

### new-design (opus, 1C/4I/2M — persist commit `d0e4ac71` confirms opus authorship)

| id | verdict | note |
| --- | --- | --- |
| C1 | ADDRESSED | New §7a: `gui/policy_address.go:132-158`'s two-state branch reproduced verbatim; three required-work items (3-state `EmitTapLeavesChunks`, third address branch, fail-closed refusal) scheduled at stages 3/4/4 respectively, both in §7a's own text and in §9's content columns |
| I1 | ADDRESSED | New §4a per-surface table + prose: `md decompose` recomputes (real keys), template grammar accepts the marker (no key binding), `md encode` on a literal xpub refuses — citing the exact `substitute_synthetic` function and its synthetic-key mechanism |
| I2 | ADDRESSED | §0 narrows the trigger to `composerLianaOutsideModelClass(...) == ""`, names the 5-fires/2-matter split and the `plain-multisig`/`sortedmulti_a` clash with §6 row 1 explicitly, same predicate I2 named |
| I3 | ADDRESSED | §9 stage 2 gate now reads "§8.2 **and §8.8**, the live `harnesses/liana` install run"; new owning-stage table also lists item 8 at stage 2 |
| I4 | ADDRESSED | New §4a paragraph: match §2 → kind 1/no slot; otherwise → today's unchanged annotated-slot behavior. Both classes I4 named (libnunchuk PR-1746, a real spendable origin-less key) are named explicitly |
| M1 | ADDRESSED | §6 row 1 rewritten: reachability reasoned from `md encode` (not just the device composer) with the RUN claim reproduced by the prior reviewer; the chain-code reason replaced with the correct index-independent-hash-vs-derived-script-sort explanation, citing `address/taproot_script_path.go:261-270` (verified below) |
| M2 | ADDRESSED | §9 stage 1b's gate narrowed from bare "§8" to explicit "§8 vectors 1, 2, 5, 6, 7, 9 — every leg runnable in Rust alone"; items 3 (device) and 4 (Go) removed from 1b's claim |

### fold-check (sonnet, own prior report)

| item | verdict | note |
| --- | --- | --- |
| opus N1 (Nit, previously unaddressed) | **NOW ADDRESSED** | §6's last row rewritten from "every `Body::Tr` at `kind = 0`" to "the descriptor's root `Tag::Tr` (there is at most one, `decode.rs:97-104`) is at `kind = 0` — or which has no `tr` at all", closing the vacuous-quantification gap N1 flagged |
| New defect 1 (58/8+ blast-radius figure unreproducible) | ADDRESSED | Replaced with a printed command (`grep -rn "isNums\|IsNUMS\|KeyPathNUMS\|NUMS" md/*.go gui/*.go \| wc -l`) and figures 87/49/136 across 32 files — reproduced exactly, see below |
| New defect 2 (§8 item 8 unscheduled) | ADDRESSED | Same fix as new-design I3 above: stage 2's gate now names §8.8 |

Both of fold-check's new defects and its one unaddressed Nit are now closed. Nothing outstanding from either r1 report.

---

## New defects introduced by this fold

### 1. File:line citations — all six new citations verified accurate, zero errors

Every NEW `file:line` citation added in r2 (6 total, found by diffing `+` lines against `\.(rs|go):[0-9]`):

| citation | claim | verified |
| --- | --- | --- |
| `decode.rs:97-104` | supports "root `Tag::Tr` (there is at most one)" | Exact — lines 97-104 are the root-tag `matches!` guard (`Tag::Sh\|Wsh\|Wpkh\|Pkh\|Tr`), a single `tree.tag` field, one root per descriptor |
| `md-cli/src/parse/template.rs:1047-1084` | `substitute_synthetic`'s exact span | Exact — `grep -n "fn substitute_synthetic\|^}"` gives `1047:fn substitute_synthetic` … `1084:}` |
| `gui/policy_address.go:132-158` | the two internal-key branches | Exact — line 132 `EmitTapLeavesChunks` call, 153-158 the `isNUMS`/else branches, matches spec prose verbatim |
| `md/tapleaves.go:188`, `:204` | `EmitTapLeavesChunks` signature and its two-state return | Exact — `188:func EmitTapLeavesChunks(... isNUMS bool ...)`, `204:return b.keyIndex, b.isNums, out, nil` |
| `address/taproot_script_path.go:261-270` | `MultiALeafScript` sorts serialized derived x-only keys | Exact — function body at those lines sorts `ser` (schnorr-serialized derived keys) when `sorted` is set |
| `gui/composer_consent.go:261` | `composerLianaOutsideModelClass` called there | Exact — `261: if class := composerLianaOutsideModelClass(tpl.Root, shape); class != ""` |

No missed citation — the author's stated six matches the six found, and all six resolve true.

### 2. §9's new "exactly one owning stage" table contradicts itself: item 2 is assigned to two stages

**Location:** §9, the table headed *"Every §8 item has exactly one owning
stage"* (new in r2 — this whole block did not exist in r1).

```
| §8 item | owning stage |
| --- | --- |
| 1 recipe vectors, 2 descriptor equality, 5 identity, 6 structure pin, 7 fixpoint, 9 mutation | 1b |
| 2 (re-run with the CLI), **8 live Liana install** | 2 |
| 4 dispatch round trip — Go leg | 3 |
| 3 address equality — device leg | 4 |
```

Item 2 ("descriptor equality") appears in **both** row 1 (owning stage 1b) and
row 2 (owning stage 2, "re-run with the CLI"). This is the same numbered §8
item — §8 has only one item 2 — assigned two owning stages in a table whose
own header asserts "exactly one." Items 1, 3, 4, 5, 6, 7, 8, 9 each appear
exactly once; only item 2 is duplicated.

This is not a fresh gap: the main content/gate table above it already gates
item 2 twice (stage 1b's gate lists "2"; stage 2's gate is "§8.2 and §8.8"),
so the underlying double-verification (recipe-level equality at 1b, CLI-level
re-run at 2) is a real, consistent, pre-existing design choice, not a slip
introduced by this fold. What **is** new in r2 is the table's own title
claiming uniqueness that its own two rows — and the main table it summarizes
— falsify. The claim should either drop "exactly one," name item 2 as the one
deliberate exception, or the "(re-run with the CLI)" row should be reworded to
not restate item 2 as if newly owned by stage 2.

**Severity:** cosmetic/internal-consistency only — no ruling, gate, or
sequencing changes as a result; the underlying double-check itself is sound
and does not weaken any gate. Worth a one-line fix before the next round, does
not block.

### 3. Blast-radius command reproduces exactly

```
$ grep -rn "isNums\|IsNUMS\|KeyPathNUMS\|NUMS" md/*.go | wc -l   → 87   (23 files)
$ grep -rn "isNums\|IsNUMS\|KeyPathNUMS\|NUMS" gui/*.go | wc -l  → 49   (9 files)
$ grep -rn "isNums\|IsNUMS\|KeyPathNUMS\|NUMS" md/*.go gui/*.go | wc -l → 136
$ grep -rl "isNums\|IsNUMS\|KeyPathNUMS\|NUMS" md/*.go gui/*.go | wc -l → 32
```
Matches the spec's "87 in `md/*.go` and 49 in `gui/*.go`, 136 across 32
files" exactly (23 + 9 = 32).

---

## Consistency checks (no defects found)

- **§0's narrowed trigger vs §7a vs §9 stage 4 describe the same device
  work.** §0 names two pieces of device work (narrowed choice screen; device
  address derivation, "See §7a"). §7a's three numbered items map 1:1 onto §9:
  item 1 (3-state `EmitTapLeavesChunks`) → stage 3, matching stage 3's content
  ("§7a.1"); items 2 and 3 (third address branch, fail-closed refusal) → stage
  4, matching stage 4's content ("§7a.2's third address branch and §7a.3's
  refusal"). No mismatch.
- **§4a's per-surface table vs the prose beneath it vs §6's refusals.** The
  table's three rows (`md decompose` recomputes; template grammar accepts the
  marker; `md encode` refuses a literal xpub) are each expanded correctly in
  the following prose, including the exact `substitute_synthetic` mechanism
  for why `md encode` cannot recompute. §4's row 1 (rendering a kind-1
  descriptor *emits* a literal xpub) does not contradict §4a's refusal of a
  literal xpub as `md encode` *input* — one is the render direction, the other
  the parse direction, and the "opposite journey" text names `md decompose`,
  not `md encode`, as the correct re-entry path.
- **"One choice screen is the minimum" is fully retracted.** Only remaining
  occurrence of that phrase is in the new §7a, stated as what r1 *wrongly*
  asserted ("r1 asserted one choice screen was 'the minimum that makes §0
  true.' It was not"). `grep -n "one choice screen\|minimum that makes"`
  confirms no other occurrence.

---

## What was NOT re-derived (per the brief)

The RUN evidence already reproduced by the r1 new-design reviewer (the
sortedmulti_a `md encode` acceptance under kind-0 NUMS, the synthetic-vs-real
xpub mismatch derivation) was taken as settled and not re-run here — this was
a fold-vs-findings and new-citation check only.
