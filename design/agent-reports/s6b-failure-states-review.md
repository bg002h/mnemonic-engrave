# S6b failure-states review — is it still correct when something goes WRONG, or is interrupted?

Second adversarial review, 2026-08-18. Scope: the 16 commits `b1479a1..HEAD` on
`s6b-pre-flash` (`/scratch/code/shibboleth/wt-s6b`). Lens: unhappy paths only —
Back, decline, abort, interruption, half-written state, and what the permanent
restore document says afterwards. The truth lens (first whole-diff review, RED
2C/2I, all four folded) was NOT re-run. All file:line references are to the
worktree at HEAD (`c333e97`).

## 1. Findings table

| id | severity | failure state | one line |
| --- | --- | --- | --- |
| F1 | **Important** | Passphrase-plate engrave aborted mid-cut, or completed and rejected at the accept screen | Passphrase-bearing steel exists that no screen says to destroy, and the restore document then states "nothing this device engraves carries a passphrase" |
| F2 | **Important** | Single-sig verify fails or its readback cannot be accounted for | Both adverse arms dead-end: the new F-204 copy prescribes a retry, no caller re-offers, and the only route back to the comparator re-cuts the entire plate set — the exact class F-199 fixed on multisig in this same diff |
| F3 | **Important** | Engrave used a passphrase; verify re-type omits it (or re-picks a different wallet type) | Guaranteed comparator FAIL on correct plates, and the `else` arm's copy says "Check the engraved plates." — a false lead the caller holds the fact to avoid |
| M1 | Minor | Verify skipped or failed, then the passphrase-plate offer runs anyway | Comment at `gui/singlesig.go:211` claims "a plate is offered only for a set already known good"; no such gate exists (behavior is fine; the stated invariant is false) |
| M2 | Nit | Wallet passphrase >100 chars / non-ASCII, operator never wanted a plate | C2's refusal modal ("Passphrase Plate: Too long…") fires before any offer is shown, about a plate never asked for; escapable, truthful, just unrequested |

Verdict at bottom. No Critical: none of these loses funds or strands the
operator mid-flow; every refusal and abort I traced is escapable.

## 2. Finding blocks

### F1 (Important) — aborted or rejected passphrase steel is unaccounted, and the document denies it

**Sequence.** Single-sig run with a passphrase → main set cut → verify offer →
passphrase-plate offer, **Engrave** → acceptance screen OK → entry OK → QR →
confirm OK → hold-to-start, the machine cuts. Then either: (a) Back twice
mid-cut (first Back stops the job, `gui/gui.go:3124-3130`; second exits) — a
partial plate, possibly with the full passphrase already cut, since the
passphrase rows lead; or (b) let it finish and press **Back at the accept
screen instead of the checkmark** — `EngraveScreen.Engrave` returns `true` only
on accept (`gui/gui.go:3132-3141`), so a COMPLETE passphrase plate exists and
the flow treats it as not cut. Either way `engravePassphraseFlowPreloaded`
returns to the confirm screen (`gui/passphrase_flow.go:886-891`), the operator
backs out, and the function returns `passphrasePlateNotCut`
(`gui/passphrase_flow.go:895`).

**What happens next.** Two things, both wrong:

1. **No destroy instruction, anywhere.** The main-set abort earned a whole
   modal whose own rationale is that a secret-bearing plate "must be DESTROYED,
   not binned" (`gui/bundle_flow.go:596-601`, doctrine at `:530-540`). The
   passphrase plate carries the passphrase VERBATIM and revealed — and its
   abort path shows nothing at all. A grep for DESTROY in production gui hits
   only `bundle_flow.go`.
2. **The restore document then affirmatively denies the steel.** notCut selects
   the shipped lines (`gui/multisig_build_census.go:310-317`): *"It is not on
   these plates and cannot be recovered from them: **nothing this device
   engraves carries a passphrase**."* In world (a)/(b) this run's own device
   just engraved exactly that. The first review's M5 froze this sentence by
   ruling (§6/6a) as "true of 'these plates'" — on the DECLINED path it is. The
   spec's supporting claim, *"decline … or abort mid-engrave, and in both cases
   the shipped sentence stays true"* (`SPEC_s6b_pre_flash_cycle.md:567-569`),
   is false for the abort half: only one of those two cases is guaranteed to
   leave no steel. R-D ("all things said must be true",
   `REQUIREMENTS_s6b_pre_flash_cycle.md:319-326`) is the governing ruling this
   violates on the failure path — so flagging it is within this brief's
   ruling boundary.

A sub-case worth naming: cut plate #1, reject it as ugly at the accept screen,
loop back through confirm, cut plate #2, accept. The doc then says "a separate
passphrase plate" (singular, `gui/multisig_build_census.go:304-309`) while two
exist. Same root: unaccepted steel is invisible to everything downstream.

**Why it is wrong.** A permanent document read years later, alone, understates
how many engraved copies of the spending passphrase exist, and on the
all-rejected path states categorically that none can exist. The operator
holds the counter-evidence at that moment, which is what keeps this Important
rather than Critical — but the device's own precedent (`bundleAbortWarning`)
is that "the operator was there" does not excuse silence about secret steel,
and that warning fires even when nothing was cut yet.

**Smallest fix.** One local bool in `engravePassphraseFlowPreloaded`
("an engrave attempt started", set just before `NewEngraveScreen(...).Engrave`
at `gui/passphrase_flow.go:886`); on any notCut exit with it set, one
dismissible `showError` in `bundleAbortWarningText`'s mold: the plate is not
counted anywhere, destroy (not bin) any steel that was cut. No new state
machine, no doc-wording change, honors §6/6a's cut-not-offered condition.
(Alternative — softening the categorical clause on the notCut arm — touches a
ruling-frozen sentence and needs the operator; the warning does not.)

### F2 (Important) — the single-sig verify tail dead-ends on both adverse arms, the class this cycle was built to fix

**Sequence (arm 1, comparator).** Full single-sig run with a passphrase, set
cut, verify accepted → re-type seed → re-type passphrase with one wrong
character (F-204's own motivating slip, `SPEC_s6b_pre_flash_cycle.md:395-399`)
→ readback OK → comparator FAILS on correct plates
(`gui/singlesig_verify.go:178-201`). The operator sees this diff's new copy:
*"Check the passphrase before you doubt the plates: one wrong character derives
a different wallet."* (`:193-197`).

**Sequence (arm 2, readback accounting).** Same run; at readback, present the
wrong plates or forget the md1 → `singleSigReadbackCards` fails →
`rec.adverse = true`, *"Need one key card (mk1) and one descriptor (md1) read
back."* (`gui/singlesig_verify.go:143-155`) — a screen that names exactly what
to re-present, F-199's literal shape.

**What happens.** Both arms `return`. The caller is a one-shot `if`
(`gui/singlesig.go:205-207`) — the code states proudly that
`statusVerifiedOnRetry` is "unreachable from it by construction"
(`gui/singlesig.go:198-201`). There is no re-offer, no standalone verify
program (the program switch at `gui/gui.go:1985-2010` has none; the multisig
I-4 comment's "the program table has no standalone bundle verify" holds for
single-sig too), and re-reaching the comparator requires completing
`bundleEngrave` again — which cuts a full fresh set: no skip exists anywhere in
its tree, per its own documentation (`gui/bundle_flow.go:550-560`). The next
screens are the passphrase-plate offer and the restore document, which
permanently records *"A verification check ran and did not pass … Do NOT rely
on this backup until a full check passes"* (`gui/verify_status.go:147-151`)
over steel that is fine.

**Why it is wrong.** This diff itself states the doctrine, twice: *"a screen
that says try again and is followed by the restore document is worse than one
that says nothing"* (`gui/multisig_verify.go:966-971`), and F-199 exists
because a refusal *"names precisely what the operator should do"* with no
caller re-offering (FOLLOWUPS F-199). The same diff wires the multisig callers
to loop on `verifyIncomplete`/`verifyFailed` (`gui/multisig.go:340-353`,
`gui/multisig_build.go:452-467`) and leaves the flagship path one-shot — while
ADDING, via F-204's fold, the "try again" half of the shape on that very path.
The mechanism is pre-existing; the instruct-then-dead-end composition is
S6b's. The doc line is TRUE (conservative direction, G2-safe), which is why
this is Important, not Critical: the harm is a permanently damning document
over a good backup, curable only by re-cutting the whole set.

**Smallest fix.** Mirror the sibling: give `singleSigVerifyFlow` a
`bool` return ("operator can act on a retry": true at the two adverse arms,
false everywhere else) and wrap the offer at `gui/singlesig.go:205-207` in the
same escapable `for` + retry-lead loop both multisig callers already use. The
sticky `rec` already produces the honest `statusVerifiedOnRetry` on a clean
second pass; no new machinery.

### F3 (Important) — verify's no-passphrase arm blames the plates when the missing passphrase is the cause

**Sequence.** Engrave WITH a passphrase (it derived the set:
`gui/singlesig.go:71-80,115`) → verify accepted → re-type seed correctly → at
the verify's own "Add a BIP-39 passphrase?" prompt press **Skip**
(`gui/singlesig_verify.go:108-114` — plausibly a second person, or the belief
that checking needs only the seed) → readback OK → comparator FAILS,
guaranteed, on correct plates. Because the VERIFY-side `passphrase` is `""`,
the new conditional at `gui/singlesig_verify.go:193-199` takes the `else` arm:
*"The read-back bundle does NOT match the seed. Check the engraved plates."*
The same arm fires when the operator re-picks a different wallet type at
`singleSigPickFlow` (`:104-107`) than the engrave used.

**Why it is wrong.** The spec's reasoning for leaving that arm alone — *"no
passphrase → there is no passphrase to blame"* (`SPEC` §3.2) — is false in
exactly this reachable state: there IS a passphrase to blame, the omitted one.
F-204's own filed harm is verbatim what results: *"The screen then sends the
operator to destroy them"* / "the asymmetry costs steel" — now surviving only
on this arm. The multisig sibling hedges its analogous arm (*"The wallet may
have been built with a passphrase"*, `gui/multisig_verify.go:196-203`);
single-sig has stronger information available and uses none of it: the caller
knows `passphrase != ""` and the engrave's `purpose/script`
(`gui/singlesig.go:71,63`) and plumbs neither. Passing a hint bool does not
touch §7.4's independence rule — it is not derivation input, only failure
copy. Compounded by F2: with no retry, this false lead is the operator's last
screen before the damning document.

**Smallest fix.** Plumb one bool (`engraveUsedPassphrase`) into
`singleSigVerifyFlow`; when it is true and the verify-side `passphrase` is
empty, the failure copy says the set was engraved with a passphrase and none
was typed — retry with it before doubting the plates. (Lands naturally on
F2's retry loop.) Wording itself is the operator's to settle; the finding is
the false lead, not a phrasing preference.

### M1 (Minor) — "offered only for a set already known good" is a gate that does not exist

`gui/singlesig.go:210-212`. The passphrase-plate offer runs identically after
a skipped verify and after a FAILED one (nothing reads `rec` before
`singleSigPassphrasePlateOffer` at `:224`). The behavior is arguably right —
the plate's fingerprints derive from RAM truth, not from the suspect steel —
but the comment asserts an invariant a future reader may build on. One-line
comment fix. (Same class as the first review's M2, which is adjacent at
`:111`; both are folds of the "comments outlive their conditions" family.)

### M2 (Nit) — C2's refusal is unrequested

`gui/singlesig.go:346-355`: `ValidatePassphrase` runs before the offer is
shown, so an operator with a 150-char payload passphrase who would have
declined anyway sees "Passphrase Plate: Too long. At most 100 characters fit
on one plate." about a plate they never asked for, then the run continues
normally. Escapable, truthful, correctly ordered for safety (the check MUST
precede `engravePassphraseFlowPreloaded`'s 100-byte buffer). Recorded only.

## 3. Failure states traced and found SOUND

1. **The preloaded step machine's Back arithmetic** (`step -= 2` + uncond.
   `step++`, `gui/passphrase_flow.go:836-895`): QR→entry (1−2=−1,++→0),
   confirm→QR (2−2=0,++→1), engrave-build-error→confirm and
   engrave-backed-out→confirm (3−2=1,++→2), all land on real screens; the
   elided fingerprint steps cannot bounce because the type cannot name them.
   C1's refusal nets to stay-on-entry (`step--` then `step++`).
2. **C1's refusal** (edit ≠ body): reachable (the keyboard is editable),
   escapable (`showModal` dismiss → entry reloaded with the TRUE passphrase
   via `n = copy(secret, body)`; Back exits notCut), and records nothing.
   Buffer hygiene holds: an edited-longer residue past `n` is never engraved
   (`secret[:n]`) and the deferred `wipeBytes` covers the full backing array.
   PASSPROOF! typed as an edit is inert (`ppPassProofOffer` returns false on
   `load == nil`, `gui/passphrase_passproof.go:216-218`) and falls into the
   same refusal; a wallet passphrase that IS literally "PASSPROOF!" engraves
   correctly through the equality check.
3. **C2's refusal**: reachable both ways (keyboard passphrase length is
   unvalidated — `passphraseFlowTitled` returns `kbd.Fragment` raw,
   `gui/gui.go:840-846`; payload adds non-ASCII), escapable, and afterwards
   notCut + the shipped doc lines, which are true there.
4. **Offer declined / acceptance screen declined / ctx.Done at any step**:
   `passphrasePlateNotCut` on every such exit, including the `for !ctx.Done`
   fall-through; the scrub defer runs on all of them. The acceptance screen
   always has at least its Source line (`flagSource` is unconditional for
   non-typed sources, `gui/sysw_admit.go:114-116`).
5. **Power loss / walk-away mid-flow**: before the engrave, nothing cut and
   nothing recorded; mid-engrave, steel may exist but NO restore document is
   produced that run (the doc requires reaching `restoreDocFlow` in-process),
   so no false artifact — only F1's missing destroy guidance applies.
6. **Main set aborted with the new marking in place**: `bundleEngrave !=
   bundleEngraveDone` returns before the verify offer, the passphrase offer
   and the doc (`gui/singlesig.go:203`), with `bundleAbortWarning` as the last
   screen. Marking changes none of this; `bundlePlateMark` keys per-plate on
   kind, so a re-run after abort marks identically and ms1 stays unmarked.
7. **F-199's newly-looping path, driven through its failure modes**: `:753`
   readback failure → `verifyIncomplete` → both callers re-offer with an
   escapable loop (decline = CONTINUE or Back, `gui/multisig.go:340-353`,
   `gui/multisig_build.go:452-467`); repeated failure loops again; retried to
   a clean full pass → sticky `rec.adverse` + `pass` →
   `statusVerifiedOnRetry`, whose doc line is true; declined after failure →
   `statusCheckDidNotPass`, whose line explicitly covers "a plate could not
   be read or accounted for" (`gui/verify_status.go:147-151`).
8. **Verify skipped** (all three paths): `rec` stays zero →
   `statusNotFullyChecked`, the weakest true line; nothing downstream
   strengthens it (monotonic derivation, `gui/verify_status.go:100-124`).
9. **verifyRecord half-writes**: every early exit of `singleSigVerifyFlow`
   and `multisigVerifyFlow` writes neither bit unless steel was actually
   read/compared adversely; the pass is written only at the success
   fall-through. Interrupting anywhere lands in the zero cell.
10. **Scroll arrows under interruption**: per-direction predicates agree with
    the clamp at both extremes (at `scroll == maxScroll` the down predicate is
    `bodyClip.Max.Y − scrollFadeDist > dims.Y` = false; up hides exactly at
    0); the I1 fold clears BOTH `Pressed` and `repeat` when a direction hides
    (`gui/gui.go:435-446`), covering hold-to-max, swallowed releases, and
    modal dismissal mid-press; every `Warning` holder (`ErrorScreen`,
    `ConfirmWarningScreen`) is constructed fresh per showing, so scroll
    position cannot bleed between one refusal modal and the next.
11. **`Derived`/`PolicyID` half-written**: unset only together (typed path and
    preview pass `"", false`; preloaded passes both), and the footer selector
    keys on `Derived` alone, so no interrupted path can mint a DERIVED footer
    over typed values.
12. **The lazy bare-seed KDF**: `deriveAccountXpub` neither reads nor zeroes
    the mnemonic argument beyond deriving (`gui/derive.go:19-53`); the
    caller-scoped scrub defer fires only at flow exit, so the offer-time
    second KDF operates on live words; a KDF error is caught with its own
    escapable refusal and notCut. Power-cycling during the ~31 s synchronous
    wait loses nothing (nothing cut, nothing recorded); the frozen screen
    matches the shipped behavior of every other derive site.
13. **Watch-only + passphrase + plate cut**: both cut-branch doc lines are
    true of that world; declined stays on the shipped lines, true there.

## 4. States found unreachable, with evidence

1. **"Main set aborted but a passphrase plate already cut."** Unreachable: the
   offer is strictly downstream of `bundleEngraveDone`
   (`gui/singlesig.go:203-224`), and `engravePassphraseFlowPreloaded`'s only
   caller is `singleSigPassphrasePlateOffer` (`gui/singlesig.go:376`), whose
   only caller is `engraveSingleSigFlow` (`:224`). The reverse (main cut,
   passphrase plate not) is reachable and is F1's territory.
2. **The silent bounce through elided fingerprint steps** (GATE 2.1's failure
   mode): structurally impossible — `ppPreloadedStep` has no fingerprint
   cases to land on (`gui/passphrase_flow.go:747-753`).
3. **`verifyFreshSlots`' error arm looping** (`gui/multisig_verify.go:894`):
   confirmed unreachable in-process — its only error is
   `errVerifyNoExpectedSlots` on `len(expected)==0`, `expectedSlots` is never
   reassigned, and `:717` already refused that condition; correctly left
   `verifyRefused` (non-looping) as defense.
4. **`"POLICY   DERIVED"` malformed footer** (Derived with empty PolicyID):
   requires `md.FormAwareStubChunks(b.MD1)` (`md/template_id.go:122-128`) to
   fail on an md1 the same run already stub-derived and engraved. Already
   recorded as the first review's M4; nothing on a failure path reaches it —
   every abort/decline exits before `ppBuildPlate`, or passes the same `b`.
5. **`ppBuildPlate` failing on the preloaded engrave step**: the C2 guard
   enforces the same constraint the plate layout does (≤100 printable ASCII),
   so the "does not fit a plate" refusal is defensive; if it ever fired it
   returns to confirm and is escapable.
6. **An empty preloaded passphrase**: `pass == ""` returns notCut before the
   offer (`gui/singlesig.go:333-335`), and `ValidatePassphrase` refuses `""`
   independently.

## Verdict

**RED 0C/3I** — F1 (unaccounted passphrase steel + the document's categorical
denial, abort path), F2 (both single-sig verify adverse arms dead-end; F-199's
class shipped on the flagship path with F-204's fold adding the
"try again" half), F3 (the no-passphrase verify arm's false lead when the
engrave used a passphrase). All three are on failure paths only; every refusal
and abort traced is escapable, and no finding touches the happy-path claims
the first review closed. F2/F3 share one screen and one fix site; F1 is one
local bool and one modal. Minors M1/M2 recorded, non-gating.
