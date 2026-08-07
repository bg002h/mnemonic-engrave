# R0 v2 round 2 (opus) — fold verification of the opus+fable round-1 folds

Artifact @ 381265d. Verdict: **1 Critical / 3 Important / 4 Minor / 2 Nit — BLOCKED.**

All crypto verified by execution: all five vectors reproduce byte-exactly and the
wire format did not move; both new §6.6 literals confirmed (D `a26ed22b…`,
E `70f3e35a…`) and **D ≠ E**; 128 bits confirmed sufficient (targeted preimage,
no birthday route; 2¹²⁸ ≈ 2.8×10¹⁷ GPU-years); the downgrade trace closes with no
residual path.

## CRITICAL — §11.1 still pinned the requirement the fix existed to destroy
`git diff 86c0445..381265d` has **no hunk in §11.1**. It still read "the same
public records yield the same hash … with or without an encrypted section
(pinned by vectors D and E, which MUST agree)" — contradicting §6.6 point 1,
§11.2's literals, §11.3's `sealed`-omitted row and §11.4's "MUST DIFFER".
Since §11.1 is the Rust-primary suite and the Go port binds to whatever Rust
lands, an implementer would have written `assert_eq!(hash(D), hash(E))`, whose
only satisfying construction omits `sealed` — reinstating fable's Critical with a
green suite.
**Folded:** replaced with the literal-value assertions and a correctly scoped
stability pin (same records AND same shape, differing salt/IV/iterations).

## IMPORTANT — `record_count` was ambiguous, and the wrong reading was the defined one
§6.6 wrote `record_count(u8)` unqualified; §6.4 defines `record_count` as the
total across both sections. §6.6 needs the **public** count. Vector D is exactly
where they diverge: 5 public of 6 total, giving `a26ed22b…` vs `c7e152ae…`
(verified). A host counting totals and a device counting public records disagree
on every mixed payload — an untampered blob showing a mismatch, which is the
"teach the operator that mismatches are normal" harm arriving by another door.
**Folded:** renamed `public_record_count` with an explicit NOT-§6.4's-cap note in
both sections.

## IMPORTANT — the DECODE requirement existed only in §6.3 prose
`md.Decode` appeared nowhere but §6.3. §10.2.1 — the normative table an
implementer builds the allow-list from — still said `mdmkText` "only (via
`ValidMD`/`ValidMK`)". §6.4, §9 and the whole test surface had nothing.
**Folded:** into §10.2.1's table, §6.4's constraint list, §9's `me seal`
validation, a §11.2 negative and a §11.3 mutant row.

## IMPORTANT — nothing discriminated plural from singular secrets
§10.2.2 is correctly plural, but every vector carried at most one secret record,
so a singular implementation passed the entire suite — C4's original failure
scenario, undetected.
**Folded:** vector F, a real 2-of-3 `wsh-sortedmulti` bundle — 15 records, three
`ms1` at indices 0–2 — with a §11.2 offer-order assertion and a §11.3 row.

## Minors / Nits folded
Two §11.3 rows named killers that do not kill: the `record_count` mutant passes
"a 4-record variant hashes differently" (LF-joined records are already injective
over the record list), and the subset row named tag-mismatch flips that fire on
the AAD regardless of what the hash covers — both re-pointed at the literal
assertion, plus a new E-shape hash-flip negative that actually discriminates.
Stale `§2.2 item 10` refs → item 11 after the insertion. Lowercase had no test →
added. `--plaintext` meant two different things across `me seal` and `me hash` →
the latter is now `--sealed|--unsealed`. The "16 hex digits alone is strictly
less" claim was false under the new construction. "§9 prints it always" →
"for every payload with a public section". Duplicated mode-0600 bullet removed.

## Controller note
The Critical was a fold-coverage failure, not a design error: §11.2/§11.3/§11.4
were edited for the hash change and §11.1 was missed. Worth recording as a
process lesson — when a normative value changes, grep for every section that
asserts it, not just the ones being rewritten.
