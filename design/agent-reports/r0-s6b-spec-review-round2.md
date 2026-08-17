# R0 architect review — `SPEC_s6b_pre_flash_cycle.md`, round 2 (fold verification)

**Artifact:** `design/SPEC_s6b_pre_flash_cycle.md`, as folded in commits `6008fda`,
`7c48490`, `2a345d8`, `9ef6584` (on top of `6008fda`'s parent `07ce72e`, which
persisted round 1's report).
**Source under review:** fork `bg002h/seedhammer`, `main` = `b1479a1b38f6b045d27443764c858906e4e6e122`
(re-verified: `git rev-parse HEAD` matches, tree clean).
**Scope, per brief:** (1) did the fold close each round-1 finding; (2) did the
fold or the new material (§2.3d+R-J, R-K, §3.2a+R-M) introduce a new defect.
Not a fresh audit; round-1 CLOSED findings and R-A…R-M rulings themselves are
not relitigated.

---

## 1. Round-1 findings — fold status

| id | status | evidence |
| --- | --- | --- |
| **C1** | **CLOSED, contingent on N1** | §2.3 no longer asserts derivation as a bare claim; superseded by R-J, which requires the device to actually derive both fingerprints, making `"DERIVED, NOT TYPED"` true *if the mechanism runs*. The "quieter half" (footer/policy-id silently dropping together) is now GATE 2.3c. Text-level defect fixed; see **N1** for a gap in the mechanism that makes it true. |
| **C2** | **CLOSED** | §2.4 now names `md.FormAwareStubChunks`; confirmed it exists at `md/template_id.go:122` (form-aware wrapper at `:112`). GATE 2.4b now asserts value equality against the mk1's own `policy_id_stub`, on both forms, exactly as prescribed. |
| **C3** | **CLOSED** | §1.3 rewritten: condition moved to `gui/singlesig.go:177`, table lists all real call sites. Verified against source: `validateMdmk`'s 4 call sites are *exactly* `gui/gui.go:2344`, `gui/bundle_flow.go:407`, `gui/derive_xpub.go:494`, `gui/unlock_platelist.go:222` (`grep -n "validateMdmk("`), and `bundleEngrave`'s 4 callers are *exactly* `gui/singlesig.go:177`, `gui/multisig.go:291`, `gui/multisig_build.go:402`, `gui/bundle_flow.go:39` — both sets match the spec's table with no gaps or extras. GATE 1.3 widened to name every unmarked site. |
| **I1** | **CLOSED** | Subsumed by C3's fix; the widened GATE 1.3 now names `"Engrave Multisig"` and `"Build Policy"` explicitly as unmarked. |
| **I2** | **CLOSED** | §1.2's Title row now reads "iff the set contains a seed *and* was derived with a BIP-39 passphrase" — both rows read off R-A's predicate, matching the prescribed fix verbatim. |
| **I3** | **PARTIAL — see N2** | The wording fix landed ("MARKING APPLIES TO cardMK1 AND cardMD1 PLATES ONLY... never marked", added to GATE 1.3). But the mechanism C3's fold introduced (`bundleEngrave`-level title/footer, applied uniformly per §1.3's own NORMATIVE text) has no way to except a card by kind at the point it matters — see N2. The requirement is stated; the plumbing to satisfy it is not, and what exists today cannot satisfy it unmodified. |
| **I4** | **CLOSED** | §5 NORMATIVE 3 gained the chip-bounds/clear-text-rows requirement; new GATE 5.3 (pixel-level, one representative modal) added to §6. Matches the prescribed two-part fix exactly. |
| **I5** | **CLOSED** | §5.1 now states the predicate as an expression (`bodysz.Y > bodyClip.Dy()`), names the F-95 residual, requires an R-E-naming comment. GATE 5.1 (must be green) and GATE 5.1b (R-E divergence probe, failures expected) are split in both §5.1's body and the §6 table. Matches the prescribed fix exactly. |
| **M1** | **CLOSED** | GATE 3.1 text now specifies `:717`/`:727` behavioural, `:854` by source assertion, "the gate names which arm is which." Confirmed `gui/singlesig_truth_test.go` carries a "source assertion" idiom (`:2308`, "so a source assertion can be aimed at ONE comment") as cited. |
| **M2** | **CLOSED** | GATE 2.4a replaced with "assert the three construction sites... set the fingerprint," matching the prescribed fix; old unreachable-case gate withdrawn and said so in §7. |
| **M3** | **CLOSED** | Recorded in §7 as an action item for the implementation commit — the only thing a spec document can do with a fork-side stale comment. |
| **M4** | **CLOSED** | Recorded in §3.2a's body and §7 as an open, non-gating scope choice, correctly kept separate from R-M's *mandatory* multisig fix. |

**11 of 12 fully closed; 1 (I3) partially closed** — the wording landed, the
mechanism to enforce it did not, and the gap is now load-bearing because C3's
fold is what created the incompatibility (see N2).

---

## 2. New findings

### N1 — Critical — R2's "offer to engrave a passphrase plate" has no located call site, and §2.3d's lazy derivation depends on where it lands

**The defect.** R2 ("The device **offers** to engrave the passphrase" —
`REQUIREMENTS §1`, row R2, "new flow work") requires a **new** step: nothing in
`engraveSingleSigFlow` today offers to run the passphrase-plate program.
Verified: `engravePassphraseFlowFrom` (`gui/passphrase_flow.go:617`) has
exactly **two** production callers — `engravePassphraseFlow`
(`gui/passphrase_flow.go:606`, the menu-driven `srcTyped` entry) and
`gui/gui.go:2270` (NFC scan, `srcNFC`) — neither is chained from single-sig
engrave. `gui/singlesig.go` (224 lines, read in full) ends at its
`restoreDocFlow` call with no offer anywhere in between. **Every** citation of
`singlesig.go` anywhere in the spec (`grep -n "singlesig\.go:"`) names
*existing* lines (`:97-103`, `:107`, `:118-139`, `:177`) — none of them is an
insertion point for the new offer.

This matters because of *where the mnemonic is*. §2.3d (as revised by R-K)
requires the bare-seed fingerprint to be derived **lazily, only when the
operator elects to engrave a passphrase plate** — i.e. at whatever point that
new offer fires. Verified: `mnemonic` is **not** scrubbed at `:107` despite
the comment there ("consumed for the LAST time here") — the actual scrub is a
`defer` registered at `gui/singlesig.go:40-46`, which fires when
`engraveSingleSigFlow` **returns**, after `bundleEngrave`, the verify offer,
and `restoreDocFlow`. So:

- if the new offer is chained **inside** `engraveSingleSigFlow`, before it
  returns, `mnemonic` is provably still alive and the lazy derivation works;
- if the offer is a **separate, later program invocation** (plausible: this
  codebase already carries session-cache patterns for "don't make the
  operator re-type it" elsewhere — `syswSource`/payload offers, R-C's own
  framing) reached from the main menu after the single-sig run has already
  returned, `mnemonic` is already zeroed and the lazy derivation cannot
  produce a correct bare-seed fingerprint at all.

**The spec never says which**, and — unlike §1.3's careful, source-verified
table for the marking condition — no location is cited anywhere for R2's
offer or for where §2.3d's "elects" decision is made.

**The reachable case.** Every single-sig engrave with a passphrase, since R2
is unconditional (it is a directive, not gated on anything).

**Why it is wrong.** This is the C3 pattern exactly: a specified behaviour
("derive lazily... only if the operator elects") with no located mechanism.
GATE 2.3d ("the ~31 s derivation does not run when no passphrase plate is
engraved") only tests the negative case; it is green whether or not the
positive case is even wired, and green whether the positive case is wired
correctly or wired to read a dead mnemonic. If R2's offer is never
implemented (because its home was never specified), R2 — and by extension
R-J's whole `DERIVED, NOT TYPED` deliverable §2.3 depends on — ships as a
no-op, silently, under a full green §6.

**Smallest fix.** Cite the insertion point with the same rigor §1.3 used for
the marking condition: state that the new offer is chained **inside**
`engraveSingleSigFlow`, after `bundleEngrave` returns (or wherever chosen) and
**before** the function returns, so `mnemonic` is in scope by construction,
and add a GATE asserting that fact (e.g. a hook/test proving the lazy
derivation observes the *same* mnemonic bytes the engrave used, not a
zeroed buffer).

**Escalation.** Whether R-C's "preloaded" is meant to be same-session-chained
or a session-cache spanning separate program invocations is a UX/threat-model
question the requirements doc does not disambiguate ("passphrase program gets
run with passphrase preloaded" is silent on timing). This determination
should go to a design-level reviewer or back to the operator, not be resolved
by a verification pass.

---

### N2 — Important — I3's `cardMS1`-exclusion has no mechanism under C3's fold, and reopens I3's exact reachable case

**The defect.** §1.2 requires "`cardMS1` plates are never marked," and §1.3's
NORMATIVE fix (closing C3) is: *"`bundleEngrave` grows two optional strings,
passed through to `validateMdmk`... `gui/singlesig.go:177` is the only caller
that passes non-empty values"* — i.e. the title/footer are **function-call-scoped**:
one `bundleEngrave` call, one title/footer, applied (per this text) to
`validateMdmk` for every plate that call produces.

**Verified against source.** `bundleEngrave`'s loop is
`for _, p := range plan { validateMdmk(ctx.Platform, p.str)... }`
(`gui/bundle_flow.go:404-407`), where `plan` is `[]bundlePlate` built by
`bundlePlatePlan` (`gui/bundle_flow.go:358-373`). The `bundlePlate` struct
(`gui/bundle_flow.go:346-353`) carries `cardIdx, cardTotal, plateIdx,
plateTotal, str, label` — **no `kind` field**. `bundlePlatePlan`'s literal
construction (`gui/bundle_flow.go:361-369`) does not copy `c.kind` from the
source `bundleCard` (`gui/bundle.go:33-38`, which *does* have `kind
bundleCardKind`) into the `bundlePlate` it emits. So by the time
`bundleEngrave`'s loop reaches the point where title/footer would be threaded
into `validateMdmk`, **there is no way to tell a `cardMS1` plate apart from a
`cardMK1`/`cardMD1` plate in the same call.**

**The reachable case.** `singleSigEngraveCards(b, full)` prepends a `cardMS1`
card when `full` (`gui/singlesig_engrave.go:20-28`, re-verified in round 1)
— i.e. **exactly** the full-mode, passphrase-derived engrave that I3 named.
`gui/singlesig.go:177`'s single `bundleEngrave` call carries `cardMK1`,
`cardMD1`, and `cardMS1` together in that mode. Passing one title/footer pair
"through to `validateMdmk`" for that whole call, as §1.3 specifies, marks
all three.

**Why it is wrong.** This is I3's original defect surviving the fold that was
supposed to close it — not a new class of risk, so it keeps I3's original
severity (round 1 flagged it Important, with the *judgement* of severity —
not reachability — as the open question). What changed is that C3's fold
introduced the specific mechanism, and that mechanism is now demonstrably
incompatible with I3's requirement as literally specified, rather than merely
unaddressed.

**Smallest fix.** Add `kind bundleCardKind` to `bundlePlate`; set it from
`c.kind` in `bundlePlatePlan`'s loop; condition the two-string pass-through in
`bundleEngrave` on `p.kind != cardMS1`. One field, one assignment, one
condition — no new abstraction.

---

### N3 — Important — GATE 2.3d and GATE 3.2a are missing from the §6 "GATES, COLLECTED" table

**The defect.** Both new gates are defined inline, in prose, in the sections
that introduce them: `GATE 2.3d (revised)` (§2.3d) and `GATE 3.2a` (§3.2a).
Neither appears as a row in §6's table. Confirmed by extracting every table
row between `## 6. GATES, COLLECTED` and `## 7.`
(`awk '/^## 6\./,/^## 7\./' ... | grep '^|'`): 19 rows, spanning 1.1 through
"— (me CLI untouched)" — no `2.3d`, no `3.2a`.

**Why it is wrong.** §6 is the document's own authoritative checklist — it is
what "GATE 1.2a is still owed at implementation" (§6, own text) points back
to, and it is the natural place an implementer works from to know what must be
tested before the cycle closes. Two gates covering genuinely new, previously
un-reviewed requirements — R-K's convenience/lazy-derivation boundary and
R-M's truthfulness fix on shipped multisig copy — are invisible to anyone
using that table as the checklist, which is exactly its stated purpose.

**Smallest fix.** Two rows, copied from the inline gate text:

```
| 2.3d ▲ | preloaded path has no fingerprint-entry step; the ~31 s KDF does not run when no passphrase plate is engraved |
| 3.2a ▲ | the provedInnocent replacement passes §4's fit gate; string contains no claim that a passphrase is required |
```

---

## 3. The new material, section by section

### §2.3d + R-J — the device preloads the fingerprints, lazily under R-K

**Defective — see N1.** The KDF-cost analysis is sound and correctly cited
(masterFP free at `:107`; bare-seed FP costs a second ~31 s derivation,
`gui/gui.go:825`, `:1653`, `gui/unlock_platelist.go:175` — all in the
already-settled measured-facts list). What's missing is the mechanism: where
the "elect to engrave a passphrase plate" decision is made, and whether the
mnemonic is in scope there. R-K's premise that R-J's `:107` constraint "rested
on the scrub point being inviolable" is itself imprecise — the actual scrub is
a function-return `defer`, not a `:107` event — but this doesn't change the
finding's substance: lifting a constraint that was never quite where it was
said to be doesn't specify where the new, later point actually is.

### R-K — the threat model

**Sound.** Internally consistent: correctly scoped away from the
sealed-payload program (never touched, no contrary text found anywhere in the
spec), explicitly does not relax R-D ("all things said must be true" stays
binding — §2.3d's GATE 2.3b/2.3c are untouched by R-K and still enforce
truthfulness). No section of the spec was found to apply R-K's
convenience-over-security license somewhere R-K didn't authorize it (checked
§3's verify tail, which re-derives with a re-typed seed independent of R-K).
Its only defect surfaces one layer down, in §2.3d's translation of "the
constraint is lifted" into an actual mechanism (N1) — the ruling itself is
sound.

### §3.2a + R-M — the multisig `provedInnocent` replacement

**Sound.** The adopted wording in the spec is a byte-for-byte match against
`REQUIREMENTS_s6b_pre_flash_cycle.md`'s "ADOPTED WORDING" block — verbatim,
no drift. The arm it replaces (`gui/multisig_verify.go:157-170`,
`provedInnocent` case) was re-read directly from source and matches both
documents' quotes exactly. GATE 3.2a is well-formed (subject only to N3's
omission from the collected table). The scope note ("copy only... does not
cross R-B") holds — no control-flow or marking change is introduced by this
section.

---

## 4. Escalation

**N1** (where R2's offer lives, and whether "preloaded" spans one session or
crosses program invocations) needs a design-level or operator call — it is
not resolvable by checking the spec against source, because the source has no
existing analogue to check against; S6b is what's supposed to create it. Route
to an architect-tier reviewer or back to the operator before folding.

---

## Verdict

`RED 1C/2I`
