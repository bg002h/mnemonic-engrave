# Comprehension-lens review — IMPLEMENTATION_PLAN_multisig_build_repair.md

Lens: the human at the panel — what does the operator believe after each screen,
is it true, and can they perform any check the screen asks of them. Reviewed
against the plan, `SPEC_multisig_build_repair.md`, and the REAL screen strings at
`/scratch/code/shibboleth/seedhammer` @ `a10d007` (`gui/multisig_build.go`,
`gui/bundle_flow.go`, `gui/multisig_restore.go`, `gui/multisig_engrave.go`,
`gui/derive_xpub.go`, `gui/md1_inspect.go`). Overlap with the adversarial (A1)
and failure-state (F1/F2/M1) reports checked and excluded; findings below are
the residue of that class, found systematically.

**Verdict: 2 Critical, 5 Important, 2 Minor — the flow's gates are sound, but
three of its screens still teach a belief that is false at restore time, and the
plan's own two remedial fixes (the review digest, the recovery procedure) each
land in a form the operator cannot actually use.**

## Ranked table

| # | id | screen | plan section | severity | one line |
| --- | --- | --- | --- | --- | --- |
| 1 | CH-1 | Engrave Mode + restore doc | S5, S3 | **Critical** | "Full (seed + keys)" and the restore doc are silent about the BIP-39 passphrase — a required factor absent from the backup and unmentioned by it (F-132's device sibling) |
| 2 | CH-2 | seed↔key gate FAIL screen | S4 | **Critical** | the only visible route past a gate failure is reassigning the slot to `payloadKey` — which silences the check instead of fixing the mismatch, and nothing warns against it |
| 3 | CH-3 | slot-assignment + its review | S4 | Important | `payloadKey`/`derived`/`both` is spec language; a wrong pick either silently skips the check the operator asked for, or fires a false alarm on an honest build |
| 4 | CH-4 | policy review + EXPERIMENTAL | S5 | Important | the "unambiguous digest" arm and the existing "match your coordinator" stub line name comparisons no operator artifact can perform; the warning "names S6", a plan stage |
| 5 | CH-5 | abort warning / restore doc | S5 item 7 | Important | the interruption-recovery procedure lives on a screen an interrupted operator can never reach, while the abort screen still says "discard the engraved plate(s) and start the bundle over" |
| 6 | CH-6 | seed entry + Passphrase prompt | S4 | Important | with several seeds, nothing on the entry or passphrase screens names which slot/seed they bind to; a swapped pairing mints keys nobody can re-derive, and `derived` slots have no gate to catch it |
| 7 | CH-7 | the engrave tail | S5 | Important | the operator is never told the plate count before the first cut; the restore doc never enumerates the full set, so a missing plate is invisible years later |
| 8 | CH-8 | S1 cosigner review | S1 item 6 | Minor | two payload cards render as two identical "mk1 key" rows — a tally wearing the name of a review; the per-slot key display arrives only at S5's policy review |
| 9 | CH-9 | (restore doc, again) | S4 walk-away | Minor | the non-wiping disclosure is directed to "the restore doc", the wrong surface for a RAM-residency fact |

---

## CH-1 (Critical) — "Full (seed + keys)" is a false claim when a passphrase was used, and nothing anywhere says so

**Screen / plan section.** The mode choice (`gui/multisig_build.go:151`,
`"Full (seed + keys)"` / `"Watch-only (keys)"`) and the restore doc
(`gui/multisig_restore.go:41-44`: `Descriptor:` + chunked text + `First
receive:` + `First change:` — nothing else). The plan touches both surfaces: S5
generalises full mode to every master's ms1, S3 and S5 item 7 both edit the
restore doc. Spec §4.1 states the fact itself: *"The ms1 backup carries entropy
only, never the passphrase, so a wrong binding is invisible in every engraved
artifact"* — and uses it only to justify per-seed prompting. No requirement in
spec or plan ever surfaces it to the operator.

**What a reasonable person concludes.** The operator chose "Add passphrase" and
then "Full (seed + keys)". The label says the backup is full. The restore doc —
which is the template for whatever the operator writes down; it is display-only,
so its omissions propagate verbatim to the paper record — shows a descriptor and
two addresses and says nothing about a passphrase. The conclusion, for the
operator years later ("did this wallet have a passphrase? which?") and for any
future reader restoring from the ms1 plate, is that the steel suffices. It does
not: the ms1 restores a mnemonic whose derivations do not match the descriptor,
and nothing explains why or says a missing factor exists.

**Cost, and when they find out.** The operator's legs are underivable at restore
time — possibly by an heir, possibly after the passphrase is unrecoverable. In a
k-of-n this can be total loss (multi-slot self via one passphrased seed loses
every held leg at once — exactly the Trace B shape S5 builds). Discovery is at
spend time, years later, by someone who was not at the panel. This is F-132's
recorded shape ("the defect is silence; the backup should say the factor is not
in it") landed on the plan's own new surface, and project precedent already
ruled that disclosure is the fix.

**Bounded fix.** Two lines of screen text, one plan sentence in S5: (a) when any
supplied seed carried a passphrase, the restore doc MUST state it — "Keys derive
from seed + BIP-39 passphrase. The passphrase is NOT in this backup." — naming
which seed(s) if there are several; (b) the full-mode confirmation states "the
passphrase is not engraved" before the tail starts. A test asserts the line
appears iff a passphrase was set (both directions — the iff matters, or every
no-passphrase wallet sends its reader hunting for a passphrase that never
existed, F-131's inverse arm).

## CH-2 (Critical) — the gate's failure screen leaves exactly one visible route forward, and it is the one that removes the check

**Screen / plan section.** S4's seed↔key gate failure ("FAIL LOUDLY, naming the
slot"; spec §4.3: "states which slot, that the key and seed disagree, and that
nothing was engraved"). The plan specifies the alarm completely and the next
step not at all.

**What a reasonable person concludes.** The most likely honest causes of a
`both`-slot failure are, in order: a mistyped passphrase; a passphrase that
differs from the one the card was minted under (a card packed months ago); a
wrong account in the assignment. The screen names none of them — it says the key
and seed disagree at @1. The operator *knows* the card is theirs (they packed
it), so a persistent failure reads as the device being wrong. The one
on-device action that makes the failure go away is re-entering the assignment
and switching @1 from `both` to `payloadKey` — "just use my card directly".
The review screen then shows @1 as a payload key, the operator confirms, and the
engrave proceeds. Nothing anywhere states that this move does not resolve the
disagreement, it only stops measuring it.

**Cost, and when they find out.** The engraved wallet contains a key at @1 that
provably does not derive from the operator's seed — say, the old-passphrase key.
The operator believes they hold that slot; they hold nothing at it. The
full-mode ms1 restores a seed that cannot sign for the wallet it is filed with.
Discovery at spend or recovery, years later. This is precisely the class this
lens exists for: both founding findings were "the screen before the irreversible
act says the wrong thing"; this one is "the screens around the failure jointly
present the wrong thing as the only exit".

**Bounded fix.** Plan text in S4, three sentences on the failure screen: (1)
name the likely causes in order — "check the passphrase you entered, then the
account assignment"; (2) name the terminal route — "if both are right, this card
was not made from this seed: rebuild the payload on the host" (the same
phase-1-language rule S1 test 7 already enforces for under-supply); (3) state
plainly: "changing this slot to 'payload key' does not fix this — it only skips
the check." Optionally, the strong form: after a `both` failure, reassigning
that same record to `payloadKey` within the same flow requires an explicit
confirm that repeats sentence (3). A test asserts the three elements are
present, alongside the existing `TestGateNeverPrintsSeedOrPassphrase`.

## CH-3 (Important) — the slot-source model reaches the screen in spec language, and both misreadings have teeth

**Screen / plan section.** S4's assignment step and its review screen
("`payloadKey(record)` / `derived(seedID, account)` / `both(seedID, account,
record)`, with a review screen the operator confirms before assembly"). No
operator-facing wording is specified anywhere in the plan.

**What a reasonable person concludes.** "Both" is not a thing a slot obviously
is. The two natural misreadings, for someone who has not read §4.3: (a) the
operator whose own card is on the payload picks `payloadKey` — it *is* a payload
key — believing the device still "verifies key can be derived from seed" (the
requirement is theirs, quoted in the spec; they know the device does this). The
gate never fires and no screen says which slots were checked. (b) The operator
picks `both` for a cosigner's card because their seed is *also* on the payload
("both are present") — the gate then correctly fails on an honest build, the
device announces that a key and seed disagree, and the operator concludes the
payload is corrupt or a cosigner's card is bad. A false alarm here is also what
teaches operators that gate failures are noise — feeding CH-2.

**Cost, and when they find out.** (a) an unchecked own-key slot — the exposure
§4.3 exists to close, silently reopened per-build by a menu choice; discovered
never, or at CH-2's endpoint. (b) an aborted honest build and an operator
mistrusting the correct machinery; discovered immediately, cost is confidence
and time.

**Bounded fix.** The plan should specify screen language and one rule: sources
are named by what they mean, not what they are — "Cosigner card (from payload)"
/ "My seed, account N" / "My card — check it against my seed"; the `both`
label states the check it triggers; and the review screen marks each checked
slot after the gate passes ("@1 checked against seed ✓" or a summary line
"1 slot verified against your seed"), so what-was-checked is visible rather
than inferred. One paragraph in S4; the strings land with S4's review screen.

## CH-4 (Important) — the review's "digest" arm and the warning's referents name comparisons the operator cannot perform

**Screen / plan section.** S5: "(1) show the per-slot keys, **or an unambiguous
digest of them**, on the review screen; (2) rewrite the warning to demand a key
or descriptor comparison against an independent source… The external-coordinator
restore at S6 is the real backstop, and the warning should name it as such."
Plus the existing review line the plan keeps (`gui/multisig_build.go:530`):
"Fingerprint choice changes the policy id — **match your coordinator**."

**What a reasonable person concludes / can they do the check.** Walk it with
what the operator actually holds at the panel: cosigner xpubs communicated out
of band (what `mk verify --xpub` prints host-side), maybe a coordinator app on
another device. A device-invented digest of the slot keys has **no counterpart
in any of those** — no external tool prints it — so the "digest" arm rebuilds
the exact defect this lens was created from: a screen teaching a check that
cannot check, one abstraction level up from fingerprints. Likewise the stub
line: no coordinator displays an md1 WalletPolicyId, so "match your coordinator"
instructs a comparison whose other side does not exist (the id IS comparable —
against `mk verify --policy-id-stub` on the host, not a coordinator). And "name
S6 as the backstop" is plan-speak: S6 is a development stage; the operator's
backstop is an action — *their* external-coordinator restore before funding.

**Cost, and when they find out.** The rewritten warning is the last screen
before steel. If its demanded check is unperformable, the competent operator
does what the fingerprint version already trained: holds the button. The cosigner
set goes to steel unverified; discovery only if A1's attack was live, at fund
loss.

**Bounded fix.** Rule the arm in the plan: the review shows **the keys, in the
form the operator's counterpart artifacts use** — the base58 account xpub as
carried on the mk1 card and printed by `mk verify --xpub` (note: capture them at
assembly; post-encode the md1 holds only 65-byte chaincode‖pubkey, F-130, and
the base58 form is unrecoverable). If a digest is ever shown, the plan must name
the exact host command that prints the identical value, or it may not ship.
Reword the stub line's referent ("match the id your host toolkit computes", not
"your coordinator"). The warning names the performable sequence in operator
words: "after engraving, restore this descriptor in your own coordinator;
compare each cosigner key against what that cosigner gave you; compare the First
receive address with the restore document; only then fund."

## CH-5 (Important) — the recovery procedure is filed where an interrupted operator can never read it, and the abort screen still contradicts it

**Screen / plan section.** S5 item 7 rules that re-running mints byte-identical
plates and says "put the recovery procedure in the restore doc, or an
interrupted operator has no route out." But the restore doc is **step 11 of a
completed flow** (`gui/multisig_build.go:185-192`) — a power loss mid-tail
reboots to the main menu and never renders it; a deliberate abort exits through
`bundleAbortWarning` (`gui/bundle_flow.go:353-356`), whose text the plan keeps
for public plates: *"A partial bundle can't be used — discard the engraved
plate(s) and start the bundle over."*

**What a reasonable person concludes.** Aborting at plate 5 of 9: the plates
already cut are waste; start from zero. Both halves are false — the plates are
byte-identical to what a re-run mints (the property S5 test 7 pins), and only
the missing plates need cutting.

**Cost, and when they find out.** Hours of re-engraving and binned good steel on
every interruption of a 6–9-plate tail; found out never (the operator has no
reason to doubt the screen). Worst case, the "discard" instruction is applied to
a correct plate from the set's completed head. The fold that fixed F2 (DESTROY
for secret plates) fixed the *dangerous* arm and left the *wasteful* arm's text
asserting the exact opposite of the property the same stage proves.

**Bounded fix.** Move the recovery statement to the surface the interrupted
operator actually sees: the abort warning — "Plates already engraved are still
correct. Re-run this build with the same seed(s) and payload; the device cuts
identical plates — continue from where you stopped and keep what is cut." The
restore doc may carry it too, but as a copy, not the home. The DESTROY ruling
for secret plates is unchanged. One sentence relocated in S5; the abort text is
already cards-derived so the change stays local.

## CH-6 (Important) — with several seeds, no screen says which seed it is collecting or which seed a passphrase binds to

**Screen / plan section.** S4's implementation ("Per-seed passphrase (§4.1),
asked at that seed's entry"). The current screens the multi-seed loop will
reuse: `Input Seed / Where from?` (`gui/derive_xpub.go:187`), `Input Seed /
Choose number of words`, and `Passphrase / Add a BIP-39 passphrase?`
(`gui/multisig_build.go:81`) — none carries any slot or seed identifier, because
until now there was only ever one seed.

**What a reasonable person concludes.** Entering seeds A then B for slots
@0/@2, with a passphrase on one: the second identical "Input Seed" screen is
"the next one", and the operator's model of which is which lives entirely in
their head across four to eight screens. Spec §4.1 itself names the stakes of a
wrong pairing: keys "the operator can only re-derive with a pairing they never
chose", invisible in every engraved artifact — and for `derived` slots there is
no card, so §4.3's gate structurally cannot catch it. The spec worried about a
flow-global passphrase; a mis-sequenced per-seed one produces the same object.

**Cost, and when they find out.** A leg (or several — multi-slot self
concentrates them) derived from a pairing nobody can reproduce; discovered at
restore. Likelihood is the ordinary confusion of N identical screens in
sequence, not an invented fool.

**Bounded fix.** One plan sentence in S4: every seed-entry and passphrase screen
in the multi-seed loop names its binding in the title or lead — "Seed 2 (for
slot @2) — where from?", "Passphrase for seed 2?" — and the slot review echoes
it ("@2 my seed 2, account 0"). The scrub-registry test already tracks seeds by
`seedID`; the same identity reaching the screen title is the whole fix.

## CH-7 (Important) — nobody tells the operator how many plates the tail will cut, and the restore doc never enumerates the set

**Screen / plan section.** S5's tail (Trace B: "6–9 plates over hours") and the
restore doc. The plan computes exactly this census — "the script computes how
many md1 chunks, mk1s and ms1s the inputs REQUIRE" (§3) — but only for the
gate, never for the human.

**What a reasonable person concludes.** Standing at the machine with a stack of
blanks, the operator learns the plate count one plate at a time ("Card 2 of 4 |
Plate 1 of 2"). Nothing before the first cut says the build needs 9 plates.
Starting with 7 blanks guarantees a mid-set interruption — the exact state CH-5
handles — chosen blind. Years later, a reader holding the box has no way to know
the set is complete: the restore doc lists a descriptor and two addresses, not
"this wallet's backup is 2 seed plates + 3 key plates + 1 descriptor of 3
plates". A quietly missing ms1(B) plate — separated, not never-cut; C2's test
guards the cutting, not the drawer — is invisible until k cannot be met.

**Cost, and when they find out.** Pre-cut: wasted hours and an avoidable abort,
found at plate N. Post-hoc: a backup whose incompleteness surfaces at restore —
F-132's silence-shape again, at set granularity.

**Bounded fix.** Derive the census from the input tuple (the code the walk
script already needs) and show it twice: a line on the pre-engrave confirmation
— "This will engrave 9 plates: 2 seed (secret, cut last), 6 key, 1 descriptor.
Have 9 blanks ready." — and an inventory block on the restore doc ("Backup set:
…"), which the operator's hand-copy then preserves. One deliverable sentence in
S5; the counting function is shared with the walk gate.

## CH-8 (Minor) — the S1 cosigner review's rows are indistinguishable, so it reviews nothing

S1 item 6 renames the gather "a review of what the payload supplied" and test 5
asserts @N ordering — but every mk1 card renders as the same
`"mk1 key ✓ / account key card"` label (`gui/multisig_engrave.go`,
`bundleReviewFlow`). Two cosigners produce two identical rows; the operator
"approves" an assignment they cannot see. The keys arrive on-screen only at
S5's policy review, four screens later and one stage later in the schedule. A1
already carries the adversarial half and its fix names S1; recorded here so the
comprehension half survives if A1's fold lands only at S5: the S1 screen spec
should state what a row shows (slot @N + the key identifier of CH-4's ruled
form). Until then it is a tally titled review.

## CH-9 (Minor) — the non-wiping disclosure is aimed at the wrong document

S4's walk-away ruling ends "…or an explicit recorded decision that the build
flow is non-wiping…, **stated in the restore doc**." A restore-doc reader years
later has no use for a RAM-residency fact; the person who needs it is the
operator mid-build, deciding whether to walk away. If the non-wiping arm is
chosen, the disclosure belongs on the build flow's own surface (or the operator
manual), with the restore doc at most echoing it. One phrase to redirect.

---

## Well handled — probed, and right

- **Under-supply refusals speak phase-1 language** (S1 test 7): the refusal
  names the only route that exists (rewrite the payload on the host) instead of
  the retired "scan a card". This is the exact template CH-2's fix asks the gate
  failure to follow.
- **Public plates first, DESTROY for cut secrets** (S5): both halves of F2's
  fix are ruled, cards-derived, and scoped so no other flow's call site moves.
- **The gate's false-positive is tested against**
  (`TestGateIgnoresUnassignedCosigners`): a gate that cried wolf on the normal
  seed-plus-cosigner-cards payload would train operators to page past gates.
  The plan names this "the false-positive that would make the feature unusable"
  — the right comprehension instinct, made a test.
- **The legitimate multi-account shape proceeds with a notice, not a refusal**
  (S4 test 5) — the operator building the wallet §4.1 exists for is not told
  their wallet is an error.
- **`TestGateNeverPrintsSeedOrPassphrase`, mutation-checked** — the loud
  failure cannot leak what it is protecting.
- **Named refusals over fall-throughs** (depth-0 card, S5 test 6; duplicate
  keys, S2 test 4) — the operator gets a reason, not "Couldn't assemble".
- **The EXPERIMENTAL rewrite's core sentence** — "a matching fingerprint is not
  verification" — kills the taught non-check; CH-4 is about completing the
  replacement, not about the diagnosis.
- **No inert controls** (§5.1 ruling; the conditional pager precedent in
  `confirmReviewScreen`) — the plan already reasons in terms of what a control
  teaches, on a device whose other buttons cut steel.
- **The restore doc shows first receive/change addresses** — the one artifact
  that makes the external-coordinator comparison genuinely performable; CH-4's
  fix leans on it rather than inventing anything.

## Severity summary

Critical: CH-1, CH-2. Important: CH-3, CH-4, CH-5, CH-6, CH-7. Minor: CH-8,
CH-9. All are plan-text fixes — screen-content requirements and rulings, no new
machinery beyond a shared census function CH-7 notes the walk script needs
anyway.
