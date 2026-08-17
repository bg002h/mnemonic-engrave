# cycle-prep recon — 2026-08-17 — F-199, F-204, F-206, REQUIREMENTS_s6b_pre_flash_cycle

**Origin/master SHA at recon time:** `5fd0b74` (registry repo, `mnemonic-engrave`)
**Source repo SHA:** fork `bg002h/seedhammer` `main` = `b1479a1b38f6b045d27443764c858906e4e6e122`
**Local branch:** `master`
**Sync state:** up-to-date — 0 ahead / 0 behind, in **both** repos
**Untracked:** none, in both repos

**Placement note.** `cycle-prep` prescribes `cycle-prep-recon-<slug>.md` in the repo
root. This repo's `CLAUDE.md` governs design-artifact placement (`design/`, prefix
`RECON_`) and is the more specific rule, so the file lands here instead. Same
content, same format.

Slugs verified: `F-199`, `F-204`, `F-206`, plus §2 of
`REQUIREMENTS_s6b_pre_flash_cycle.md`. **Drift was expected and found, but it is
confined to line-number pointers — every substantive claim verified TRUE.**

**A note on why these slugs cite another repo.** All three follow-ups are
`#seedhammer`. The registry lives in `mnemonic-engrave`; the cited Go code lives in
the fork. Citations were therefore resolved against fork `main`, **not** this
repo's `master`. A recon that checked only this repo would have reported every
`gui/*.go` citation as missing.

**Evidence lives in two persisted agent reports**, committed verbatim in `e5859fd`
*before* this synthesis was written:

- `design/agent-reports/cycle-prep-s6b-followup-citations.md`
- `design/agent-reports/cycle-prep-s6b-requirements-facts.md`

This document does not restate their per-citation ACCURATE lists. Transcription is
where misquotes come from; the reports are the artifact. Reproduced below are only
the **exceptions** and the facts the controller measured independently.

---

## Per-slug verification

### F-199 — `verifyRefused` dead-ends on a CORRECTABLE readback

- **WHAT:** `gui/multisig_verify.go` shows a screen naming exactly what the operator
  should re-present, then returns `verifyRefused` — a verdict no engrave caller
  re-offers on. Reachable *before any seed is typed*. Needs a **per-site** decision,
  because `verifyRefused` also carries programmer-error refusals that must not loop.
- **Citations:** 8 checked — **6 ACCURATE, 1 DRIFTED, 1 STRUCTURALLY-WRONG.**
  - `gui/multisig_verify.go:698-702` → **DRIFTED-by-54**, now **`:752`**. String
    byte-exact. Controller-confirmed.
  - `gui/multisig_verify.go:82` "pre-existing in `main` from `b2c3231`" →
    **STRUCTURALLY-WRONG.** At `b2c3231` line 82 is **blank**; the string is at
    **`:64`**. Controller-confirmed with `git show b2c3231:… | sed -n '82p' | cat -A`
    → `$` (empty).
  - Also at `b2c3231` the wording was **singular** — `the operator key card (mk1).`
    — where F-199 quotes `card(s) (mk1).`, which is the *current* string. So the
    quoted bytes are today's, not the provenance commit's.
  - Everything else — `verifyRefused` new in `9f93362` (confirmed absent in
    `9f93362^` by `git log -S`), neither engrave caller re-offering
    (`gui/multisig_build.go:460`, `gui/multisig.go:345`), B3's `correctable` local scoped to
    the seed-entry/ms1-entry breaks, and the restore-doc string at
    `gui/multisig_build_census.go:69` — **ACCURATE**.
- **Controller-measured, and load-bearing for the design:** `verifyRefused` has
  **4** return sites, not 3.

  | line | trigger | correctable? |
  | --- | --- | --- |
  | 717 | `len(expectedSlots) == 0` | **no** — programmer error, must not loop |
  | 727 | `len(engravedMd1) == 0` | **no** — programmer error, must not loop |
  | **753** | `extractReadbackMd1AndMk1s` fails on gathered cards | **YES — F-199's site** |
  | 854 | `verifyFreshSlots` → `ferr != nil` | **no** — see below |

  `:854` looked like it might be operator-correctable, since it fires *after* seed
  and passphrase entry. It is not: `verifyFreshSlots`
  (`gui/multisig_verify.go:324-336`) has exactly **one** error return —
  `errVerifyNoExpectedSlots` on `len(expected) == 0` — and `expected` does not
  change inside the function, so `:854` is a defensive re-check of `:717`'s
  condition.

  **This confirms F-199's central claim quantitatively:** 3 of 4 sites must never
  loop. Widening the verdict would make all three loop, which is why the follow-up
  says the obvious move is the wrong one.
- **Action for the spec:** correct `:82` → `:64` and `:698-702` → `:752`; state the
  provenance as *"pre-existing in control flow; the message text was pluralized
  after `b2c3231`"* rather than "unchanged". Carry the 4-site table into the spec so
  the per-site decision is made against a census, not a recollection. Cite fork SHA
  `b1479a1`.

### F-204 — a FAILED single-sig verify sends the operator to doubt the PLATES

- **WHAT:** `gui/singlesig_verify.go` tells a failed verify to "Check the engraved
  plates"; the multisig sibling rules the other way. A mistyped passphrase at verify
  derives a different wallet, so the screen can send an operator to destroy
  **correct** plates.
- **Citations:** 4 checked — **3 ACCURATE, 1 DRIFTED, 0 structural errors.**
  - `gui/singlesig_verify.go:145` → **DRIFTED-by-37**, now **`:182`**.
    Controller-confirmed. Context confirms it is genuinely the FAILED path
    (`"Verify Failed"`, `rec.adverse = true` set immediately above).
  - `multisigVerifyNoSlotBody` and the quoted "Check the passphrase before you doubt
    the plates" — **ACCURATE**; the function is at `:157` and the string spans
    `:164-165`. The cited range `151-165` is **loosely drawn** (it starts in the doc
    comment and stops mid-case) but contains no wrong line.
  - The SPEC §7.4 re-typed-seed claim — **ACCURATE**, at
    `design/SPEC_systemwide_payloads.md:1201-1208`.
- **Action for the spec:** correct `:145` → `:182`; tighten the range to the
  function bounds. Cite fork SHA `b1479a1`.

### F-206 — the pass line's ms1 clause stays singular on a multi-seed verify

- **WHAT:** clause **B** is a fixed singular string; a full multisig verify over two
  seeds still says "the ms1 secret" / "this seed". **Under-claims, so non-gating.**
- **Citations:** 4 checked — **4 ACCURATE, byte-exact. Clean.** The fixed string at
  `gui/verify_status.go:155`; §4.7c clause B at
  `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md:973`; the device's correctly-pluralized
  screen at `gui/multisig_verify.go:1134-1135`; `passRecord.legs` at
  `gui/verify_status.go:66`.
- **Confirmed mechanism:** `buildVerifyPassLine` (`gui/verify_status.go:211-231`)
  appends the clause whenever `p.full` is true — **unconditionally on `p.legs`**.
  The field needed for the fix already exists, so the fix is a plural rule over an
  existing recorded fact. **No new field, so NG1 does not reopen.**
- **Action for the spec:** none. Cite as-is against fork SHA `b1479a1`.

### REQUIREMENTS_s6b_pre_flash_cycle.md §2 — "MEASURED FACTS, do not re-derive"

- **WHAT:** the section a spec is about to inherit as givens. Written against fork
  `b8a23bf`; fork is now `b1479a1` (S6a merged: 21 files, +3209/−73).
- **Result: 19 ACCURATE, 0 DRIFTED, 1 STRUCTURALLY-WRONG.**
  - `deriveAccountXpub` at `gui/singlesig_derive.go:10` → **STRUCTURALLY-WRONG.**
    That line is an import (`"seedhammer.com/bip32"`). The function is at
    **`gui/derive.go:19`**. Controller-confirmed. **This was already wrong at the
    doc's own cited SHA — it is not S6a drift.**
  - **Not load-bearing.** The claim the pointer supports is independently true: the
    signature is
    `deriveAccountXpub(m bip39.Mnemonic, passphrase string, net *chaincfg.Params, path bip32.Path)`
    — passphrase-bound by parameter — and the `ms1` counter-citation
    (`gui/singlesig_derive.go:87`, `codex32.EncodeMS1(entropy)`) is **correct**. Only
    the pointer is wrong, not the fact.
- **Controller-remeasured, since the whole cycle's title budget rests on it:**

  ```
   19  PASSPHRASE REQUIRED
   17  PASSWORD REQUIRED
   18  SEED FP: 73C5 DA0A
   18  COMB FP: FC60 C6DF
   17  PASSPHRASE NEEDED
   16  NEEDS PASSPHRASE
   27  EXPECTED COMB FP: FC60 C6DF
  ```

  All 7 match the doc exactly. `PASSWORD REQUIRED` = **17**, one character under
  `MaxTitleLen = 18`; **both fingerprint forms sit exactly ON the cap.**
- **Action for the spec:** correct the one pointer. Otherwise §2 may be inherited as
  written — it survived a whole cycle's merge intact.

---

## Cross-cutting observations

1. **Every substantive claim across all four documents is TRUE. All three defects
   are stale line-number pointers.** This inverts the S6a lesson, where the records
   were the weak half. Worth recording because it is evidence about *which* record
   defects survive: prose claims here held; pointers rotted.

2. **Citation decay was confined to the previous cycle's diff, and that is
   predictive.** S6a touched only `gui/multisig_verify.go` (+80) and
   `gui/singlesig_verify.go` (+56) among cited files. Both F-199's and F-204's drift
   is in exactly those two files; §2, which barely cites them, drifted **zero**. A
   cheap future heuristic: `git diff <doc-SHA>..HEAD --stat -- <cited files>` bounds
   where drift can be *before* re-verifying everything.

3. **One structural error predates its own document.** The `gui/singlesig_derive.go:10`
   pointer was already wrong at `b8a23bf`, the SHA §2 cites as its source. A recon
   that only diffed old-SHA-vs-new would have missed it. **Re-verification must
   resolve against current source, not diff against the cited SHA.**

4. **F-199's owning-phase header disagrees with every other record.** Its own entry
   reads owning phase **`SPEC_multisig_build_repair.md` S6** — not S6b. Only F-204
   and F-206 declare S6b. F-199's S6b membership is asserted *externally* (F-204's
   body at `design/FOLLOWUPS.md:7211`, the continuity doc, the requirements doc) and never
   in its own header. Since S6a shipped as part of S6, the registry's own rule —
   *"an item whose owning phase has already passed is overdue, not deferred"* —
   reads F-199 as **overdue**. Fix the header when the spec lands.

5. **S6b is fork-native only, so none of cycle-prep's standard locksteps apply.**
   Measured: **0** hits for `MaxTitleLen`/`TitleString`/`PASSWORD REQUIRED` in
   `crates/`; **0** CLI-surface references across the three slug entries; no
   `docs/manual/src/40-cli-reference/`; **0** `schema_mirror` files in this repo.
   **No SemVer call for `me`, no manual mirror, no schema mirror.** §2.2's claim that
   the Rust-primary rule is untriggered is consistent with this: all marking is plate
   layout and text, and the `mk1`/`md1`/`ms1` strings stay byte-identical.

6. **The real lockstep is the goldens, and it is the one §3 already worries about
   (Q6).** Any title/footer band changes plate rendering, and a churned golden is
   only as good as its review.

7. **R3/R4 land on a four-call-site chokepoint that §3 never asks about.**
   `validateMdmk` (`gui/gui.go:2288`) is the single mechanism laying out md1/mk1
   plates — the one §2.3 identifies as having **no title and no footer**. Production
   call sites, measured:

   | call site | flow |
   | --- | --- |
   | `gui/bundle_flow.go:407` | bundle engrave |
   | `gui/gui.go:2344` | md1/mk1 engrave |
   | `gui/unlock_platelist.go:222` | unlock engrave |
   | **`gui/derive_xpub.go:494`** | **`deriveXpubFlow` — this is F-205's flow** |

   §3 Q1 asks *which mechanism* mk1/md1 move to; it never asks *which call sites get
   marked*. Marking inside `validateMdmk` reaches all four uniformly and would close
   part of **F-205** incidentally; marking at a call site leaves three flows minting
   unmarked passphrase-bound plates. **The spec must decide this deliberately** —
   this is structurally the "found 1 of 4 sites" class the last cycle paid for.

8. **The chokepoint and the marking target coincide exactly — a genuinely
   convenient fact.** The fork's own reviewed comment at `gui/engraved_hook.go:8-17`
   enumerates the paths that **bypass** `validateMdmk`: `engraveCodex32`
   (`gui/codex32_polish.go:218`) and `unlockEngraveCodex32`
   (`gui/unlock_session.go:186`). Both are **ms1** — and §2.1 establishes ms1 is
   words-only, *not* passphrase-bound, so those plates correctly need no passphrase
   marking. Every passphrase-bound artifact flows through the one chokepoint.

9. **A claim-counting nuance, resolved rather than left ambiguous.** F-199 says
   `verifyRefused` "also carries two programmer-error refusals". That is two
   *conditions*, spread over **three of four** return sites. The follow-up never
   claims a site count, so it is not wrong — but a spec that reads "two" as "two
   sites" would leave `:854` unclassified. Carry the census, not the sentence.

---

## Recommended brainstorm-session scope

**One cycle, as the requirements doc directs — and it opens with a decision pass,
not code.** Nothing found here argues for splitting or resequencing it.

**Ordering.** The dependency is strict: Q1 (which plate-text mechanism) gates the
length budget, which gates every wording decision, which gates the goldens.

1. **Decision pass — no code.** Answers, in order:
   - **F-199's per-site ruling**, against the 4-site census above. Only `:753` may
     loop.
   - **§3 Q1** — give `Text` a title/footer band, or route md1/mk1 through a
     mechanism that has one.
   - **The call-site question (obs. 7)** — mark inside `validateMdmk` or at call
     sites, and therefore **whether S6b closes part of F-205 or explicitly does
     not**. Decide it; do not let it happen.
   - **§3 Q3** — is the "key-id" `md.WalletPolicyIDStub` (4 bytes → 8 hex, groups as
     `XXXX XXXX`, already the binding mk1 carries) or `mk.Header.ChunkSetID`?
   - **§3 Q4/Q5** — watch-only sets, and the multisig paths.
2. **A throwaway executable spike, BEFORE the spec closes.** This is the S6a lesson
   applied: *has every specified output ever been produced?* The spike answers **§3
   Q2** — does a title/footer actually **fit** alongside the existing text+QR on an
   md1/mk1 plate at current sizes? It is unverified, measurable, and everything
   downstream assumes it. Engraving feature-size and bounding-box limits apply. It
   also produces the first real measurement of **golden churn** (Q6).
3. **Spec → R0 loop to 0C/0I** (opus for the design-level adversarial pass; sonnet
   for fold verification). Enumerate the lenses up front — correctness,
   failure-states, does-the-walk-run, comprehension, spec-coverage — rather than
   discovering them one exhausted round at a time.
4. **One implementer**, TDD, in a worktree.
5. **Mandatory whole-diff adversarial review**, then the hardware flash.

**Rough sizing (estimate, flagged as such).**

| work | shape | size |
| --- | --- | --- |
| F-204 | one screen's copy, plus its test | ~10-30 lines |
| F-206 | plural rule over the existing `passRecord.legs` | ~20-40 lines |
| F-199 | per-site control flow at `:753` only, plus a test that the other 3 do **not** loop | ~30-60 lines |
| R3/R4/R6 plate marking | title/footer band on the `Text` mechanism + wiring + goldens | **the bulk — unknown until the spike** |
| R7 | pin the password-only QR with a test | ~15 lines |

**SemVer:** none. Fork-native; no `me` surface touched (obs. 5).

**Mandatory locksteps:** the plate **goldens** (obs. 6). None of cycle-prep's
standard mirrors apply.

**Gate note.** `plan-glyph-check.sh` scanned **zero** strings on `REQUIREMENTS_s6b_*`
and still exited 0 — its heuristics reach only blockquotes and backtick spans ≥40
chars. **A 0-scanned pass is not a clean pass.** Every engraved string this cycle
introduces must be glyph-checked directly, and the length budget must be asserted by
a **test on the budget**, not on today's string: `PASSWORD REQUIRED` has one
character of headroom and both fingerprint forms sit exactly on the cap, where
`TitleString` truncates **silently**.

**This recon is input to a spec. It is not a spec, not gated, and not
implementable.** The R0 gate is unaffected and still mandatory.
