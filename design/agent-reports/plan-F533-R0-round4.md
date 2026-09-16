# R0 round 4 — PLAN_F533_bip388_reuse_predicate.md (mechanical fold check)

**Verdict: 0 Critical / 0 Important — GREEN.**

Read-only at engrave `a8aaf66d` (the round-4 plan commit), fork `e4ab97d`
(matches the plan's stated baseline; tree clean). Scope was exactly the two
questions in the brief: did the fold close round 3's findings, and did the
round-4 edits introduce anything new. Nothing was edited; the only write is
this file.

## 1. Per-finding disposition of round 3

| # | round 3 finding | disposition |
| --- | --- | --- |
| **I1** | mutation-set instruction yields 9, prose says 8, one member (`compose_tr_thirty_two_slots`) cannot redden | **Addressed.** The set is now defined as the enumeration **intersected with `ok/agrees`**, `compose_tr_thirty_two_slots` is named as excluded, and the text states 8 with no competing figure anywhere. Independently reverified against `design/policy-corpus-baseline.json` (engrave repo): `compose_tr_thirty_two_slots` → `{"device": "source"}`, confirming it is correctly outside the reddening set. The gate is now satisfiable by a correct implementation. |
| **M1** | `ok/refused` prints as absent, not `0` | **Addressed**, verbatim: table cell now reads "absent — the bucket VANISHES from `counts` rather than printing `0`". |
| **M2** | six-places list omits `md/duplicate_keys.go:80-89`; a second omission (`md/duplicate_keys_test.go:50-56`) was named as "worth folding in" | **Partially.** The primary omission is fixed — item 7 added, and lines 80-89 verified (`cat -n`) to hold exactly the quoted doc comment. The secondary item was not added; `md/duplicate_keys_test.go:50-56` still exists and still reads "counting the whole taptree at once instead of per leaf leaves BOTH tr rows green" (verified), which the widening falsifies and which is named nowhere in the plan. Round 3 flagged this as optional ("worth folding into"), not required, so non-blocking. |
| **M3** | gates named only as "the fit and golden gates"; no line citations | **Addressed**, plus more. Now cites `gui/composer_copy_test.go:112` and `:114` (verified: exactly the two `DuplicateRefusedByCore`/`DuplicateFewerKeys` rows) and names `gui/modal_fits_test.go` (verified: 3 hand-written `Duplicate*` rows at 321/326/331, matching round 3's count, though without the specific 319-332 range). Two files were added beyond round 3's ask — `gui/duplicate_seat_address_test.go` and `gui/composer_flow_test.go`. `composer_flow_test.go` checks out: `TestDuplicateWarningNamesTheRightHarm` (793-810) is a genuine 2-row hardcoded kind→copy gate that would stay green for a third kind. `duplicate_seat_address_test.go` is weaker than claimed — see new finding below. Also newly added: a concrete measurement instruction citing the real "96 chars, 6 lines... page one holds 7... 122 chars landed on 8 only because of where the words break" comment, verified verbatim at `gui/composer_copy.go:439-443`. |
| **M4** | F-529 still an "or"; no durable fixture | **Addressed.** Now reads "**Decision, not an option:** pin both vectors locally BEFORE implementing, in the same commit as the predicate." This removes the "or" and, by pinning the vectors themselves, addresses the durability concern (a different mechanism than the suggested `trBody` Go fixture, which round 3 offered as a suggestion, not a requirement). |
| **M5** | "17 taproot `ok/agrees`" is a name-grep artifact; real figure is 18 | **Addressed**, verbatim: "my '17 taproot `ok/agrees`' was a name-grep artifact and the real figure is 18," plus the root-cause fix — "Determine taproot-ness from the DECODED WRAPPER, never from the vector name." |
| **N1** | `gui/policy_address.go:77` should be `:76` | **Addressed.** Now cites `:76`; verified via `cat -n` that line 76 is the `DuplicateKeySlotChunks` call and line 77 is `return nil, false`. |

## 2. New findings from the round-4 edits

No Critical or Important. One new Minor, worth recording:

**New Minor — `gui/duplicate_seat_address_test.go` does not hand-enumerate
kinds the way the plan implies.** The plan lists it alongside
`composer_copy_test.go` and `modal_fits_test.go` as a gate that "would stay
green" and needs "the new kind added." Verified by reading the file: its two
relevant tests (`TestRepeatedSeatRefusalStillWarns`,
`TestKeylessRepeatedSeatTemplateIsNotSilent`) compute `want :=
composerCopyDuplicateKeys(slot, kind)` **dynamically** from whatever kind the
fixture produces, then check that string propagates to other screens — they
do not pin a hardcoded kind→string mapping the way `modal_fits_test.go` and
`composer_copy_test.go`'s tables do. A copy fallthrough bug would not make
this file fail, since `want` is derived from the same function under test.
Non-blocking: the instruction to "add the new kind to each" is harmlessly
over-inclusive here, not wrong-direction — worst case an implementer finds
nothing to add and moves on.

Everything else checked and holds: fork HEAD is `e4ab97d` exactly as the plan
states; every new file:line citation resolves and says what's claimed
(`md/duplicate_keys.go:80-89`, `gui/composer_copy_test.go:112`/`:114`,
`gui/modal_fits_test.go`, `gui/composer_flow_test.go`, `gui/policy_address.go:76`,
`gui/composer_copy.go:439-443`); the status header is still stale ("DRAFT
round 2") — same as round 3 found and explicitly did not count as a finding,
so treated the same way here (Nit, non-blocking).

## 3. Verdict

**GREEN — 0 Critical / 0 Important.** Round 3's blocking item (I1) is closed
by a satisfiable, independently-reverified mutation set. All four Minors and
the Nit are addressed or explicitly non-blocking per round 3's own framing.
The round-4 edits introduced no new Critical/Important and only one new
Minor (an over-broad file citation in the copy-gate list). The design has
been settled since round 1; this fold closes the loop.
