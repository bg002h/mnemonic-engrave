# S6a R1 — mechanical fold-check (coverage + factual-claim verification)

**Question asked:** did the fold (`b54f7ee..HEAD` on
`design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`) actually address every
finding from the two R0 reports, and are the factual claims it makes true?
**Not a fresh audit; not a design review.** Scope discipline honored: no new
defects hunted in the Go code, no re-opening of C-1's remedy shape or cycle
scope, no prose/markdown review.

Go tree checked: `/scratch/code/shibboleth/seedhammer` @ `b8a23bf` (HEAD at
time of check) — the same SHA both R0 reports cite. Confirmed with
`git rev-parse HEAD`.

---

## VERDICT: RED — 0 Critical, 1 Important

One false factual claim survived the fold and was repeated in the fold's own
commit message: §4.7 (and §3.2) assert that `gui/multisig.go:323` contains
the comment *"Only verifyComplete falls through to the restore document"* —
it does not. That exact sentence exists **only** at
`gui/multisig_build.go:439`, which the plan never names anywhere. This
inherits Important severity under the brief's own rule ("a false factual
claim in the plan is Important at minimum"), and it is also a **MISSED**
disposition for R0's M-6, whose actual defect (the false comment at
`multisig_build.go:439`) is left uncorrected by anything the plan directs.

Everything else checked — 1 Critical, 8 Important-or-higher-severity
dispositions, 6 Minors, 2 Nits across both reports, plus 7 factual claims —
is ADDRESSED and TRUE. Both gates ran clean and match the fold's commit
message exactly.

---

## PART 1 — COVERAGE

| finding | severity | disposition | evidence |
| --- | --- | --- | --- |
| **C-1** (verify-that-failed still reaches restore doc) | Critical | **ADDRESSED** | New §4.7 (plan L546-609): document always renders, exactly one of 4 status lines (VERIFIED / NOT VERIFIED / DID NOT COMPLETE / DISAGREED), single-sig gets a result type mirroring `multisigVerifyResult`. Text matches `design/agent-reports/s6a-c1-verify-tail-decision.md` verbatim. |
| **I-1** (presence arm keyed on path capacity, not run) | Important | **ADDRESSED** | §4.4 (L441-524): `buildSeedInventoryLines(cards []bundleCard)` — capacity param dropped; discriminant is ms1 card count in `cards`. Rationale block explicitly cites the one-slot-multisig-build failure mode R0 named. |
| **I-2** ("the plates are the secret" false on watch-only) | Important | **ADDRESSED** | §4.3 (L358-440) rewritten: ruling now has two independent axes — path capacity (subject clause) and `bundleSetCarriesASecret(cards)` (whether the "plates are the secret" pair appears at all). §3.1 item 6 (L299-315 pre-fold numbering) rewrites the assumption as "wrong," not "held." |
| **I-3** ("the seed" reads as sufficiency) | Important | **ADDRESSED** | §4.4 presence arms now read "this set contains YOUR seed" / "YOUR seeds," with an explicit numbered rationale (item 1, L497-503) citing `oneSeedPassphraseFact`'s `"your seed"` label as precedent. |
| **I-4** (missing test list; deletable `Card 1 of N` proof) | Important | **ADDRESSED** | New §5.1 (L638-681): names all three walks, states the `Card 1 of 3`/`Card 1 of 2` distinction is **not weakenable** ("a blocking finding, not a style note"), gives the exact repair shape mirroring the in-tree positive control. |
| **exec-I-1** (same three walks, executability lens) | Important | **ADDRESSED** | Same §5.1 — both reviewers' findings converge on one list; table with file:line and break-point for each walk. |
| **M-1** (address claim contradicts "unavailable" line) | Minor | **ADDRESSED** | §4.4 point 3 (L518-524): presence-arm wording changed from "these plates can rebuild the wallet's addresses" to "it records the wallet" — no address claim, so no contradiction with `gui/multisig_restore.go:26-31`'s "Addresses unavailable for this policy shape." |
| **M-2** (`restoreDocFlow` error returns drop inventory) | Minor | **ADDRESSED** | §8 known-limitations item 6 (L~835-845): named in full, with an explicit instruction to the implementer ("if the implementer can hoist the inventory ahead of the descriptor build cheaply, they should; if it needs restructuring, it is filed, not folded"). Matches R0's own ask ("worth one line naming it"). |
| **M-3** (template branch mislabels plate, untouched by §4) | Minor | **ADDRESSED** | §8 item 7: named, both consequences (mislabeled template plate, restore doc built for a template where build path skips it) stated. |
| **M-4** ("Verify OK" incomplete on passphrase run) | Minor | **ADDRESSED** | §8 item 8: named, left alone deliberately (bracketed by §4.1's label and §4.2's document), with the exact ASCII two-string quote — and a note that this exact quote itself was the em-dash misquote the glyph gate caught on the first fold pass. |
| **M-5** (verify-fail sends to doubt plates, not passphrase) | Minor | **DEFERRED, verified matching** | Filed as `F-204` (`design/FOLLOWUPS.md:7193`), owning phase "S6b — with F-199, before the hardware flash." Content matches R0 M-5 (`multisigVerifyNoSlotBody` contrast, `singlesig_verify.go:145`) with no material drift. |
| **M-6** (false "only verifyComplete falls through" comment) | Minor (original); **effectively MISSED** | **WEAKENED / MISSED** — see Part 2 claim #2 and Findings below | Declared in-scope (not filed) at §3.2 and §4.7, but the plan's remedy names only `gui/multisig.go:323`, which does not carry the quoted sentence. `gui/multisig_build.go:439`, which does, is never named anywhere in the plan (`grep` confirms zero hits). The commit message repeats the same error and its own "Minors folded" list omits M-6 entirely. |
| **N-1** (`backupWalletFlow`/`deriveXpubFlow` passphrase-silent) | Minor/Nit | **DEFERRED, verified matching** | Filed as `F-205` (`design/FOLLOWUPS.md:7214`), owning phase "none yet — needs a scoping decision; NOT gating the hardware flash." Content matches R0 N-1 (`gui/gui.go:2419-2432`, `gui/derive_xpub.go:344-354`) with no material drift. |
| exec report **T2** minor (pager + engraver machinery unstated) | Minor | **ADDRESSED** | §5.2 (L682-716) bullet 1: names `s5PageForNeedle`, `newEngraver()`, `sh2DisplaySize`, `s5EngraveOnePlate` explicitly. |
| exec report **T3** minor (which half does the mutation drive) | Minor | **ADDRESSED** | §5.2 bullet 2: states assert the document half directly via `buildPassphraseInventoryLines`, per prior art. |
| exec report **T4** minor (unit level proves nothing about the document) | Minor | **ADDRESSED** | §5.2 bullet 3: stated explicitly as a deliberate design choice, not an oversight. |
| exec report **T5** minor (must press through census; needle pair) | Minor | **ADDRESSED** | §5.2 bullet 4: "T5 must now press through the census (§5.1b)," plus the corrected needle pair (`"Verify the engraved plates?"` / `"Descriptor:"`) and confirms no engraver needed. |
| exec report **T7** minor (absence arm uncovered elsewhere) | Minor | **ADDRESSED** | §5.2 bullet 5: states it is T7's alone and must be built on purpose. |
| exec report **T8** minor (no positive half) | Minor | **ADDRESSED** | §5.2 bullet 6: "T8 needs a POSITIVE half" — must also assert the corrected comment names all three tail-carrying callers. |

**Supplementary (found while checking, not in the required list, both Nit-severity, non-gating):**
- Executability report's own **N-1** ("§4.3 header says 'all 8' and enumerates 9") is still present post-fold: §4.3's "Call sites (all 8, measured)" (plan L427) now lists 3 production sites + 6 test sites = 9, still labeled "8." Unaddressed, Nit, does not gate.
- Executability report's own **N-2** (T6's wording doesn't pin census placement) — T6's row (plan L629) is unchanged ("reaches `Plates To Cut` before the engrave picker"), still doesn't rule out relocating the census earlier. Mitigated in practice by §5.1(b)'s explicit repair recipe naming the exact insertion point, but the T6 row itself wasn't tightened. Nit, does not gate.

---

## PART 2 — FACTUAL CLAIMS

| # | claim | TRUE / FALSE | evidence |
| --- | --- | --- | --- |
| 1 | §3.2/§4.7: multisig retry loop breaks on `!ok \|\| sel != 0`, then reaches `multisigRestoreDocFlow` unconditionally | **TRUE** | `gui/multisig.go:325-327` and `gui/multisig_build.go:449-451`: both loops break exactly on `if !ok \|\| sel != 0`. `multisigRestoreDocFlow` calls at `gui/multisig.go:361` and `gui/multisig_build.go:478` are not gated on the verify result `res` in either file. |
| 2 | §4.7: `gui/multisig.go:323` contains a comment asserting "Only verifyComplete falls through to the restore document" | **FALSE** | `grep -rn "falls through to the restore document" gui/*.go` → **one** hit, `gui/multisig_build.go:439`. `gui/multisig.go:321-323` reads: *"...RE-OFFERS itself. Only verifyComplete falls through; a refusal or an abandon does not loop, because neither is a state the operator can change by trying again with the same inputs."* — a different sentence, at an adjacent but not-identical location, that never mentions "the restore document." The plan's `plan-cite-check.sh` output itself surfaces this: it resolves `gui/multisig.go:323` and prints the real line ("// neither is a state the operator can change...") right next to the citation — the mismatch is visible in the gate's own output, but citation-checking is declared out of scope for quote-verification ("NOT covered: interpretation"). |
| 3 | §4.4: `numberedLabel` leaves the ms1 label unnumbered when there is exactly one | **TRUE** | `gui/multisig_engrave.go:63-68`: `if n <= 1 { return base }`. The plan's cited comment ("a one-leg build reads exactly as it always did") is at `gui/multisig_engrave.go:30-31`, adjacent to the `multisigEngraveCardsMulti` call site the plan cites at `:37`. |
| 4 | §4.4: `oneSeedPassphraseFact` uses the label `"your seed"` | **TRUE** | `gui/multisig_build_census.go:198`: `return []seedPassphraseFact{{Label: "your seed", Uses: uses}}`. |
| 5 | §4.3: `seedCapacityMany` + seed-on-plates assembly reproduces the shipped string byte for byte | **TRUE** | Programmatically assembled the plan's stated formula (`base := "Seed handling: ... " + subject_many + " stays in device memory until the build ends"` + the seed-bearing suffix) and diffed against the literal string built by `gui/multisig_build_census.go:86-90` (`fmt`-concatenated in Python from the same source lines). `EQUAL: True`, zero-diff, character for character. |
| 6 | §5.1: exactly four tests drive `engraveSingleSigFlow`, and three break | **TRUE** | `grep -rn "engraveSingleSigFlow(" --include="*_test.go" gui/` → exactly 4 call sites: `singlesig_flow_test.go:55,95,141` and `template_engrave_test.go:83`, inside `TestEngraveSingleSigFlowFull`, `TestEngraveSingleSigFlowWatchOnly`, `TestEngraveSingleSigFlowSeedScrubbed`, `TestEngraveSingleSigFlowTemplate` respectively. All three breaking-walk citations in §5.1's table (`:82`/`:83`, `:121`/`:122`, `:128`/`:129` for `click(Button3)` / `pumpUntil("Card 1 of N")`) match `grep -n` exactly, line for line. `TestEngraveSingleSigFlowSeedScrubbed` aborts before the engrave (not disputed — not one of the three named). |
| 7 | §8.8: `gui/singlesig_verify.go:148` quoted as `showNotice(ctx, th, "Verify OK", "The engraved bundle matches the seed.")` | **TRUE** | `sed -n '148p' gui/singlesig_verify.go` → `showNotice(ctx, th, "Verify OK", "The engraved bundle matches the seed.")`, character for character, two separate ASCII arguments as the plan states (no em dash). |

---

## PART 3 — GATES

```
$ export PATH="/nix/var/nix/profiles/default/bin:$PATH"
$ ./scripts/plan-cite-check.sh design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md ; echo "exit=$?"
...
─── citations resolved: 76 / 76 ; dangling: 0
─── NOT covered: interpretation, absence-claims, code-block compilation.
exit=0
```

```
$ ./scripts/plan-glyph-check.sh design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md ; echo "exit=$?"
═══ design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md

─── operator strings scanned: 41 ; undrawable: 0
─── NOT covered: prose-embedded strings, line-fit, the Go source itself.
exit=0
```

**Commit message claim ("76/76 citations, 0 dangling; 41 operator strings, 0
undrawable") CONFIRMED exactly** — both numbers match live gate output, both
exit 0. stdout/stderr were captured separately; stderr was empty for both
runs (no "Git tree is dirty" noise, no nix warnings surfaced in this
invocation).

**Glyph banned-set cross-check:** `scripts/plan-glyph-check.sh`'s
`BANNED = "—–·''""…"` (codepoints U+2014, U+2013, U+00B7, U+2018, U+2019,
U+201C, U+201D, U+2026) matches `gui/multisig_build_prose_test.go:394`'s
`strings.ContainsAny(ruling, "—–·''""…")` **exactly**, same 8 characters in
the same order. Confirmed by extracting both literals and comparing.

Note: the citation gate is a location-existence + line-print check, not a
quote-verification check (it says so itself: "NOT covered: interpretation").
That is precisely why Part 2 claim #2's misquote passed the gate at 76/76
while still being false — the gate proved the *line number* resolves, not
that the plan's quoted prose matches the line's actual content. This is a
declared, not hidden, blind spot of the gate, but it is exactly the kind of
gap this mechanical-verification pass exists to catch by hand.

---

## FINDINGS

### Important — §4.7/§3.2 misattribute the false "only verifyComplete falls through" comment to the wrong file, and the actual false comment is never named

**Severity:** Important (false factual claim, per brief's rule "a false
factual claim in the plan is Important at minimum"). Also constitutes a
**MISSED** disposition for R0's M-6.

**What's wrong:** §4.7 (plan L565-567, repeated at L605, and again in the
fold's own commit message) states: *"The comment at `gui/multisig.go:323`
asserting 'Only verifyComplete falls through to the restore document' is
false in its own file (R0 M-6)."* This is not what `gui/multisig.go:323`
says. The exact quoted sentence exists **only** at
`gui/multisig_build.go:439`. `gui/multisig.go`'s nearby comment (lines
321-323) says something related but different — it never mentions "the
restore document" at all, and its literal claim ("a refusal or an abandon
does not loop") is arguably true as written.

**Where this originated:** R0's own M-6 finding already contained the error
— it correctly quoted and cited `gui/multisig_build.go:439` as the false
comment's location, then separately (and incorrectly) asserted "`gui/multisig.go:323`
carries the same sentence." The fold inherited this without independently
verifying it (exactly the "review catches reasoning, execution catches
facts" failure mode: a wrong fact in a review propagates into the response
as a given).

**Consequence:** The plan's remedy — "`gui/multisig.go:323`'s false comment
is corrected in the same edit" — points an implementer at a location that
does not need this particular correction, and **never names
`gui/multisig_build.go:439` anywhere** (`grep` for `multisig_build.go:439`
or the quoted phrase across the whole plan returns exactly one hit: the
original, wrong, `gui/multisig.go:323` citation). An implementer following
the plan literally has no instruction to fix the comment that is actually
false. The fold's own "Minors folded" list in the commit message omits M-6
by name entirely, which is consistent with the substance never having been
correctly resolved.

**Suggested fix (not authoritative — for the next fold to decide):** correct
the citation to `gui/multisig_build.go:439` (the actual location of the
quoted false sentence), and separately decide whether `gui/multisig.go`'s
own nearby comment also needs tightening on its own (weaker, not obviously
false) terms.

---

## WHAT THIS ROUND DID NOT FIND

No Critical. No MISSED items among C-1, I-1..I-4, exec-I-1, or the funds
Minors/Nits M-1..M-5/N-1 — all confirmed ADDRESSED or correctly DEFERRED with
matching follow-up content. All 6 other checked factual claims (byte-for-byte
string reassembly, `numberedLabel`, `oneSeedPassphraseFact`, the four-test /
three-breaking-walk count with exact line numbers, and the verbatim
`showNotice` quote) verified TRUE against the live tree at the cited SHA. Both
build gates ran clean and match the fold commit's claimed numbers exactly,
and the glyph gate's banned character set matches the shipped test's set
character for character.
