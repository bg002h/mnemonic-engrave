# R0 round 3 — fold verification, LENS: DID THE FOLD FIX THE FINDINGS

**Artifact under review:** fold commit `0df8db0` of `design/SPEC_wallet_policy_composer.md`, responding to `design/agent-reports/composer-spec-R0-r2-adversarial.md` (BEFORE = `99463ac`, AFTER = `0df8db0` = working tree).
**Reviewer:** independent mechanical fold-verification agent, read-only. No repo file modified but this one.
**Scope:** ONLY whether the fold fixed C-1, I-1..I-7, M-1..M-5, N-1, and whether it introduced a new defect. The design itself, the operator rulings (brainstorm §2), and the controller defaults in brainstorm §3.12 items 1-20 are NOT re-litigated.

## VERDICT: 12 FIXED / 2 PARTIAL / 0 NOT FIXED / 0 DECLINED-BY-DEFAULT — 2 propagation regressions found, 0 new substantive defects

---

## 1. Per-finding table

| id | title (one line) | fold's response (quoted) | verdict |
| --- | --- | --- | --- |
| C-1 | unseated-slot origin collides with a seated slot's declared origin; one card silently fills both | §4f: "`account'` = the LOWEST account not already declared by any slot of this template … **Invariant** … no two slots of a produced template declare the same origin unless BOTH declare a fingerprint and those fingerprints differ." §12.6: "a named negative vector runs the asymmetric one-card case … against `seatKeyCards` and asserts it is never produced." §7e: self-check now holds "§4f's pairwise-distinguishability invariant … on the DECODED md1." | **FIXED** — re-derived the C-1 construction against the new rule: seated origins declare accounts `1',2',3'`; the unseated slot's lowest-free account is `0'`, not `3'`, so the collision cannot arise; a seated pair deliberately declaring one origin is caught separately by §8v |
| I-1 | §7c's re-show enumeration ("path or wrapper change") omits lock operand and hash digest, which also move the template id | §7c: "the wrapper, the path list, every lock operand and every hash digest all enter it … so the screen re-appears after EVERY edit made on the shape screen." §7d: discard narrowed to "any change that moves slot NUMBERING (the wrapper, the path count, or a path's key count)"; "a lock or hash edit moves no slot, keeps assignments, and re-shows the stub screen." | **FIXED** — re-show and discard are now two separate, correctly-scoped rules; §8j's body ("Slot numbers change with the shape") is true whenever §8j now fires, since it fires only on numbering changes |
| I-2 | §7f/§8p give contradictory rules for the partially-seated template (cards or no cards; both stubs or unsatisfiable; form A offered or impossible) | §7f new paragraph: "A PARTIALLY seated composition … offers no form A either; its form B is the keyless template … plus one card per SEATED slot carrying the TEMPLATE stub only, and the screen says the policy id does not exist until every slot is seated." §12.6: "both stubs when a keyed policy exists, the template stub otherwise." | **FIXED** — matches the minimal fix verbatim in substance; §7g gained the corresponding row |
| I-3 | §6a/§12.8's "door count is the only/reduced-by-one signal" is false for malformed `hash:`/`now:`, which change no count | §6a: "a malformed `key:` reduces the door's 'Keys loaded: N'; a malformed `hash:` or `now:` changes no count, so the door also carries 'N payload records were not understood'." §12.8 rewritten per class. §8r gained the "N … were not understood" line. | **FIXED** |
| I-4 | §8n's second-`now:` refusal fires on the host's own auto-appended record, names the wrong index, no remedy | §6a: "`me sysw pack` auto-appends `now:` ONLY when the operator's records contain none … two OPERATOR-supplied `now:` records are a host refusal with a remedy (§8n: 'Remove one.')." §8n copy gained "Remove one." | **PARTIAL** — the index math and the remedy are fixed (see re-derivation below), but see Regressions §2: a second, unqualified restatement of the OLD unconditional-append rule survives 12 lines later in the same §6a, contradicting the fix |
| I-5 | §6b's bound line withdraws the "cannot tell the time" disclaimer exactly when a stale `now:` makes it most needed | §6b: "the echo ADDS the pack date and never withdraws the disclaimer: 'This device cannot tell the time. The payload says it was packed on <pack date>, which may be long ago. Nothing here has checked that this is in the future.'" | **FIXED** — copy in §8c matches verbatim |
| I-6 | §6a's `key:` body rule reads as an integrity check but the account/interior components are unverifiable (F-217) | §6a: "**What a `key:` record's origin proves:** the xpub's depth and its last component are checked … the account and every interior component are declarations this device cannot verify (F-217) … the mapping review prints each slot's origin verbatim beside its fingerprint with the note that the device cannot confirm the key was derived there." §7g gained a DOCUMENTATION row. | **FIXED** |
| I-7 | §7e's self-check scope claim ("cannot reach steel as a reviewed wallet") is not delivered — origins/fingerprints/use-site are outside it | §7e: self-check now holds "the decoded shape, the slot assignment, every slot's origin and fingerprint (against the mapping review), the fixed use-site, and §4f's pairwise-distinguishability invariant … so a builder defect in the shape, the seating, the origins, the fingerprints or the use-site cannot reach steel … (what stays outside the check: the key bytes themselves, which the addresses cover)." | **FIXED** — matches the minimal fix's own proposed extension and residual-naming exactly |
| M-1 | §8t states a technical impossibility that is false (pre-2009 dates DO encode) | §8t: "This build will not write a date before 2009 as a time lock." | **FIXED** — exact wording match |
| M-2 | §8g's satisfiability sentence is false when the shared seed's slot count is below the path's threshold | §8g header + second body: fires "the first body when the shared seed's slots in that path reach the threshold, the second otherwise"; second body: "One person holds 2 of the 3 signatures this path needs." | **FIXED** |
| M-3 | §7a/§8r's "Keys loaded: N" undercounts a seed by treating it as one key | §8r gained "Keys loaded: 4, plus 1 seed." and "A seed is loaded. It can fill any number of slots." | **PARTIAL** — see Regressions §1: §7a's own normative door rule (line 378) was never updated and still reads "'Keys loaded: N' when a payload holds keys or seeds," with no rule distinguishing when each new §8r variant fires |
| M-4 | §8p's C5 cause line misdiagnoses a plain shortfall (fires on the repo's own fixture set) | §7d: "no cause is guessed (the C5 lesson is taught at the shape step by §8k)"; the C5 cause blockquote is removed from §8p. | **FIXED** — this is the report's own second offered option ("drop it and let line 1 stand"), applied as-is, not a divergent default |
| M-5 | §8s's "Path N key i of n" is undefined once taproot extracts the internal key (operator's list index vs. emitted leaf index) | §7d: "'Path N' in every seating and mapping prompt is the OPERATOR's listed path index, never an emitted leaf index." | **FIXED** — exact one-sentence fix as proposed |
| N-1 | §7d's re-mint can push a card into a third chunk; §7f's census doesn't count card chunks | §7f: "it counts CARD chunks too: appending stubs can push a card into a third chunk (`mk/encode.go:26-29`)." | **FIXED** |

No finding required grading DECLINED-BY-DEFAULT: every place the fold diverged from the report's exact suggested wording (I-4's remedy text, M-4's chosen alternative) picked an option the report itself offered as sufficient, and brainstorm §3.12 items 17-20 record all four defaults taken, matching the spec text found.

---

## 2. Regressions (incomplete propagation — the facts were fixed in one place and left stale in another)

Both found by grepping the AFTER text for the exact phrasing each finding attacked, beyond the three phrasings the fold's own gate already swept to 0 hits.

**(a) The `now:` auto-append rule is stated twice in §6a with different values.** The corrected rule appears at line 294-296:

> "`me sysw pack` auto-appends `now:` ONLY when the operator's records contain none, so an operator-supplied `now:` wins silently and pins a deliberate bound"

Twelve lines later, in the SAME section's "Why `now:` is a lower bound" paragraph (line 308-309, byte-identical to `99463ac` — confirmed unchanged by the fold), the pre-fix rule survives untouched:

> "`me sysw pack` appends `now:` as the LAST record by default; `--no-now` omits it so a fixture's pack output stays a pure function of its inputs (§10 item 2)."

This is the exact unconditional-append behavior I-4 attacked, restated without the "only when none present" qualifier, in the same section as its own fix. §10 item 2 WAS updated correctly ("appended last ONLY when the operator's records hold none"), so the document now has two normative statements of this rule that disagree, one of them the specific sentence I-4 was raised against. This is why I-4 is graded PARTIAL rather than FIXED — an implementer reading only the "Why" paragraph would reproduce the exact defect.

**(b) The door's key-count rule (§7a) was never updated to match the new §8r copy.** §7a (line 377-378, byte-identical to `99463ac`) still reads:

> "Beneath Build the door states the key state: 'Keys loaded: N' when a payload holds keys or seeds; 'No keys loaded. This builds a key-less template.' when it holds none or none is loaded"

This is the exact sentence M-3 attacked (a seed still described as producing "Keys loaded: N"). The fold added three new blockquote strings to §8r ("Keys loaded: 4, plus 1 seed.", "A seed is loaded. It can fill any number of slots.", "3 payload records were not understood.") but never amended §7a's normative WHEN-rule to say which of "Keys loaded: N" / "Keys loaded: N, plus 1 seed" / "A seed is loaded …" fires for which payload composition. An implementer has four candidate door strings and the same single trigger condition ("holds keys or seeds") that produced the bug. Graded PARTIAL for the same reason as (a): the copy inventory was fixed, the normative rule that dispatches it was not.

No other stale phrasing was found for the remaining ten findings (checked directly, beyond the three the fold's own sweep covered): the old slot-index unseated-account rule, the old "path or wrapper change" re-show enumeration, "the door's … only signal", and the old §8p C5 cause-line text all return 0 hits in the AFTER text.

**No cross-reference regressions found.** Every `§8v` reference (6 occurrences: §4f, §7d ×2, §7g, §9 item 7, §12 item 4) resolves to the one new `### 8v` subsection. §12 items 4/5/6/8 cross-references at lines 504, 573, 831 all point at content matching their citing context. §7g's new/changed rows (partially-seated engrave, the two split shape-edit rows, the §4f-invariant refusal, the `key:`-origin documentation row) have no orphaned old row left behind. No table cell disagrees with its governing prose beyond (a) and (b) above.

---

## 3. Consistency of the new invariants

**Unseated-slot origin rule** ("lowest account not already declared, ascending emitted slot index"): checked against every site the task named.
- §4f (normative statement): states the rule and the pairwise-distinguishability invariant.
- §7c: "The per-slot 'expects a key at' line is shown ONLY for slots that will stay unseated (§4f's unseated rule)" — defers to §4f, no restatement to disagree.
- §7f: "whose unseated slots take §4f's lowest-free accounts" — consistent wording.
- §12 item 3: "keyless-template engrave whose … slots carry distinct-account origins" — consistent (a weaker but non-contradictory restatement).
- §12 item 6: restates the full invariant, consistent.
- §13 ("What is NOT verified"): does not mention the unseated-slot rule at all — nothing to disagree with.
No disagreement found. The account-assignment rule for SEATED seed-derived slots (§4f's table: "by ordinal among the slots that master fills") is a distinct population from the unseated-slot rule and does not conflict with it.

**Discard-on-edit rule** ("numbering changes only: wrapper, path count, or a path's key count"): checked §7b, §7d, §7g, §8j, §12 item 4.
- §7d states the narrowed rule and the lock/hash carve-out.
- §7g has two rows, matching the split exactly (discard row + kept-and-reshow row).
- §8j's body ("Slot numbers change with the shape … cleared") is true in every state where §8j now fires (only numbering changes), so the fold's own claim ("§8j's body then stays true") holds under inspection.
- §12 item 4 exercises both branches by name ("a path-count edit AND a wrapper change … (discard), and a lock edit … (kept)").
- §7b does not mention the discard rule at all (it never did); no conflict.
Consistent throughout — this invariant propagated correctly, unlike the two in section 2.

---

## 4. Citation content — citations ADDED in this fold

Diffed the full `file:line` citation set between `99463ac` and `0df8db0`; exactly two are new.

**`gui/key_card_seating.go:117-140`** (used in §4f to support: "`slotMatchesCard` skips the fingerprint test when the template declares none"). Read at the fork worktree (`/scratch/code/shibboleth/seedhammer`):
```
117  }
118
119  // slotMatchesCard is layer 2's predicate: same origin, and same fingerprint
120  // when the template declares one.
...
128  func slotMatchesCard(slot md.ExpandedKey, c mk.Card) (bool, error) {
129-140  [path parsing + structural path-component comparison loop]
```
The doc comment (119-120) states the exact behavior cited ("same fingerprint when the template declares one," i.e., skipped otherwise) in prose, and the cited range does correctly identify the function. But the OPERATIVE code implementing the skip — `if slot.FingerprintPresent { … }` — sits at lines 141-151, immediately past the cited range's end (140). The citation supports the claim only via the header comment, not via the code it is nominally pointing at. This is a citation-precision defect, not a false citation (the R2 report itself cited the correct range, 141-148, for the same mechanism). Grade: Nit — true fact, imprecise line range.

**`mk/encode.go:26-29`** (used in §7f to support: "appending stubs can push a card into a third chunk"). Read at the fork worktree:
```
24  const (
25  	// CHUNKED_FRAGMENT_LONG_BYTES — the max bytes per chunk fragment of the
26  	// cross-chunk stream (mk-codec chunk.rs). The stream is always > 53 bytes for
27  	// a 1-stub card (~84 B), so a T4 card always splits into >= 2 chunks.
28  	chunkedFragmentBytes = 53
29  )
```
Exact match, line-for-line. This is the identical citation the R2 report itself used for N-1's underlying claim ("the cross-chunk fragment is 53 bytes … a 1-stub card is ~84 B → 2 chunks"), reused correctly. Supports the claim.

---

## 5. Copy check — §8v and the changed §8c/§8g/§8p/§8r/§8t bodies

Measured directly (line count, max line length excluding the `> ` marker, ASCII-only) on every body under these six headers in the AFTER text:

| body | lines | max chars | ASCII |
| --- | --- | --- | --- |
| §8c body 4 (NEW, "cannot tell the time … packed on …") | 4 | 47 | yes |
| §8c body 5 ("cannot tell the time … Nothing here has …", unchanged) | 2 | 50 | yes |
| §8g body 1 (unchanged, "can be satisfied by one person") | 4 | 46 | yes |
| §8g body 2 (NEW, "holds 2 of the 3 signatures") | 4 | 45 | yes |
| §8p (C5 line removed; 2 lines remain) | 2 | 26 | yes |
| §8r "Keys loaded: 4, plus 1 seed." | 1 | 28 | yes |
| §8r "A seed is loaded. It can fill any number of slots." | 1 | 50 | yes |
| §8r "3 payload records were not understood." | 1 | 38 | yes |
| §8t (changed) | 2 | 44 | yes |
| §8v (NEW) | 3 | 49 | yes |

All ten bodies are ≤ 4 lines and ≤ 50 characters per line, ASCII only. Consistent with the already-run glyph gate (103 strings / 0 undrawable); this is a targeted re-derivation on exactly the bodies this fold touched, not a re-run of the full gate.

---

## What I ran

- `git show 99463ac:design/SPEC_wallet_policy_composer.md`, `git show 0df8db0:…` (== working tree), `git diff 99463ac..0df8db0 -- design/SPEC_wallet_policy_composer.md` — read in full.
- `design/agent-reports/composer-spec-R0-r2-adversarial.md` — read in full (all 16 findings + "attacks tried" + "what I ran").
- `design/BRAINSTORM_wallet_policy_composer.md` section 3.12, items 17-20 — read, cross-checked against the fold text.
- Re-derived C-1's constructed input (accounts `1'/2'/3'` seated) against the new §4f rule by hand: lowest-free account for the unseated slot is `0'`, not `3'` — no collision.
- Citation diff: extracted every `` `path.go:N-M` `` / `` `path.rs:N-M` `` pattern from BEFORE and AFTER, diffed the sorted sets — exactly two new citations.
- Read `gui/key_card_seating.go` lines 117-151 and `mk/encode.go` lines 20-35 at the fork worktree (`/scratch/code/shibboleth/seedhammer`) against both new citations, with exact `awk`-verified line numbers.
- Grepped the AFTER text for the exact superseded phrasings of all 16 findings (slot-index unseated rule, "path or wrapper change," "door's … only signal," unconditional `now:` append, unqualified "Keys loaded: N … keys or seeds," old C5 cause-line text) — found two survivors (I-4, M-3) not covered by the fold's own three-phrase sweep.
- Grepped every `§8v` and `§12 item {4,5,6,8}` occurrence and checked each resolves to matching content.
- Programmatically extracted and measured every blockquote body under §8c, §8g, §8p, §8r, §8t, §8v (line count, max length, ASCII) via a small Python script over the AFTER text.

No repo file was modified except this report.
