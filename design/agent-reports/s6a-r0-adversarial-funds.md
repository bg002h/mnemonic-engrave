# S6a R0 — independent adversarial review (funds-safety lens)

**Artifact:** `design/IMPLEMENTATION_PLAN_s6a_singlesig_truth.md`
**Code under review:** `/scratch/code/shibboleth/seedhammer` @ `main` = `b8a23bf`
**Question asked:** after this plan is fully and correctly implemented, can an
operator still end up with a backup that is incomplete AND vouches for itself?
**Answer:** yes — one Critical route, on the very lines the plan edits.

---

## VERDICT: RED — 1 Critical, 4 Important

---

### C-1 — a single-sig verify that FAILS still falls through to the restore document

**Where:** `gui/singlesig.go:129-136` (the tail the plan gates at `:127` and then
leaves alone); plan §3 (F-197's scope), §4.5, §1.5's own table.

**The defect.** The plan gates the *engrave* on `bundleEngraveDone` and stops
there. The screen immediately after the engrave is the verify offer, and
`singleSigVerifyFlow` (`gui/singlesig_verify.go:65`) **returns `void`**. Every one
of its failure exits is a dismiss-only `showError` followed by a bare `return`:

- `:89` re-derive failed
- `:97` template rebuild failed
- `:116` "Need one key card (mk1) and one descriptor (md1) read back."
- `:129` / `:137` the hand-typed ms1 is not a valid secret share
- `:145` **"The read-back bundle does NOT match the seed. Check the engraved plates."**

In all five cases control returns to `gui/singlesig.go:134`, and the next
statement is `restoreDocFlow(...)` at `:136` — which under §4.2 now prints
`"This backup is 3 plates: ... If any of them is missing, this backup is
incomplete."`, the descriptor, the first receive and change addresses, and (per
§4.4) `"Seed: this set CONTAINS the seed ... treat that plate as the secret
itself."`

So the device says *your plates do not match your seed*, and the very next screen
is the durable artifact certifying those plates as a complete backup. The plan's
change makes this strictly worse, because before §4.2 the single-sig restore
document carried no inventory and therefore made no completeness claim at all
(§1.1). §4.2 is what turns a silent document into a vouching one — while the
verdict it is contradicting stays unread.

**This is not a novel design question; the codebase already ruled on it.**
`gui/multisig_verify.go:64-73` states the identical defect verbatim, as the
reason `multisigVerifyResult` exists:

> The flow returned void, so neither caller could tell a clean pass from an
> incomplete or a failed one, and the screen after every outcome was the same:
> the verify offer fell through to the restore document.

S5 fixed that for both multisig callers (`gui/multisig.go:329-342`,
`gui/multisig_build.go:445-458`: a failed or incomplete verify re-offers under
`multisigVerifyRetryLead`, `"Not every plate is verified. Try again?"`, with
`VERIFY AGAIN` / `CONTINUE`). `singleSigVerifyFlow` was never given a result
type and its caller was never given the loop. This is precisely the plan's own
thesis — *the single-sig path never learned what the multisig paths were taught*
— applied to an item the plan's §3 inventory does not contain. §1.5's table even
records `gui/singlesig.go:127` as the caller with a tail at `:130` **and** `:136`,
so the tail was seen; only half of it was gated.

**The harm.** The one mechanism on the device whose entire purpose is to catch a
mis-engraved plate set is overridden, one screen later, by the artifact that
outlives everybody. The operator files the plates and the document together. The
discovery point is recovery, years later, by someone who was not the operator,
holding a set that the machine knew was wrong and a page that says it is
complete. Funds are permanently unrecoverable.

**Suggested remedy (UNVERIFIED — I did not resolve the signature change against
every caller).** Give `singleSigVerifyFlow` a result the caller can read and gate
`restoreDocFlow` on it, mirroring `multisigVerifyResult` rather than inventing a
second shape. Note while doing so that the multisig paths are **not** clean
either: `if !ok || sel != 0 { break }` means an operator who taps `CONTINUE` after
a failed verify also reaches the restore document, so
`gui/multisig_build.go:439`'s claim *"Only verifyComplete falls through to the
restore document"* is false as written (see M-6). Matching multisig exactly
therefore matches a partially-open door; the plan should decide which door it
wants, not inherit one.

---

### I-1 — §4.4's presence arm is keyed on PATH capacity, so a one-seed build prints the plural over a single plate

**Where:** plan §4.4 (`buildSeedInventoryLines(cards []bundleCard, capacity
seedCapacity)`), §4.3's routing table, §3.1.2.

**The defect.** §4.3 wires `gui/multisig_build.go:479` to `seedCapacityMany`
**unconditionally** — capacity is declared a property of the path. §4.4 then uses
that same `capacity` to select the *presence* wording:

> `seedCapacityMany` → "Seed: this set CONTAINS seeds. Each plate marked 'ms1
> secret share' in the inventory above is a seed backup -- treat each one as the
> secret itself."

§3.1.2's justification is sound for the thing it was written about — the
seed-handling ruling describes what the *device* can hold, and two identical
builds must not print different documents. But the presence arm is a statement
about **what is on the plates**, which is a fact of the run, not of the path.

A build in which the operator holds one slot is the ordinary case
(`gui/multisig_build.go` asks *"Do you hold another slot?"*, and the default
answer ends the loop). `buildEngraveTail` then emits exactly one ms1
(`gui/multisig_build_tail.go:117-120` dedupes on the ms1 string), and
`numberedLabel("ms1 secret share", 0, 1)` returns the bare, **unnumbered** label
(`gui/multisig_engrave.go:63-68`). So the document lists

    ms1 secret share: 1 plate (secret seed backup)

and four lines later asserts the set contains *seeds*, and that *each* plate
marked `ms1 secret share` is a seed backup.

**The harm.** This is the self-vouching defect run backwards, and the codebase
has already named that failure mode as fatal: `buildPassphraseInventoryLines`'
own comment (`gui/multisig_build_census.go:110-114`) says silence "leaves the
reader unable to distinguish a complete backup from one whose operator forgot ...
and that is the state in which people give up on a recovery that would have
worked." A reader counting one seed plate against a document that says *seeds*
and *each plate* concludes a plate is missing from a set that is complete, and
stops. The plural is not a stylistic wobble; on this document it is a claim about
how much steel should exist.

**Suggested remedy (UNVERIFIED).** The presence arm has `cards` in hand and the
count is already derivable there — `bundleSetCarriesASecret` is a boolean over
the same slice. Whatever discriminant is chosen, it must be the run's ms1 count,
and the capacity parameter must stay bound to the §4.3 ruling where §3.1.2's
argument actually holds.

---

### I-2 — §4.3's one-seed ruling ends "the plates are the secret", which is false on every watch-only run and now sits four lines under a new sentence that says the opposite

**Where:** plan §4.3 (the `seedCapacityOne` string), §4.4 (absence arm +
placement), §3.1.6.

**The defect.** The plan authors this sentence fresh for the one-seed arm:

> "... and on a full build the words are also on the plates as they are cut. **Do
> not leave a mid-build machine unattended: the plates are the secret.** Power the
> device off when you are done."

The preceding clause is correctly conditioned (`on a full build`). The clause
that follows is not conditioned at all. On a watch-only run no plate in the set
carries seed material — which is exactly what §4.4's new absence arm asserts, in
the *same document*, in the position §4.4 puts it (item 2 of 4, with the ruling at
item 4):

> "Seed: this set contains NO seed. ... **no plate in this set holds it.**"

The document therefore states both that no plate holds the seed and that the
plates are the secret, roughly a screen apart, on every watch-only run of every
path that reaches this function.

**§3.1.6 is where this slipped.** It declares the seed-handling ruling "true in
both modes" and backs it with a measurement — `gui/singlesig.go:41` obtains the
mnemonic and `:90` derives unconditionally, so the device does hold a seed in
watch-only. That measurement is correct and settles the *device-memory* half of
the sentence. It does not touch the *plates* half, and the assumption is written
as though it covers both. This is the §1.3 landmine class exactly: the plan
audited `"this build can hold several"` in this string and did not audit
`"the plates are the secret"` in the same string.

**The harm.** A reader of a watch-only document is told the plates are the
secret. The optimistic reading — these plates hold what is needed to spend — is a
watch-only set vouching for a spending capability it does not have, which is the
defect this cycle exists to kill. The pessimistic reading sends someone to
destroy or over-secure public plates. Either way the durable artifact contradicts
itself, and the reader has no way to know which half is the true one.

**Note on scope.** The `seedCapacityMany` arm carries the same clause and is
byte-identical to shipped text — but the adjacency is new on both arms, because
§4.4's absence line is new on both. The clause the *plan itself writes* is the
one-seed one, which is why this is a finding against the plan rather than against
S5.

---

### I-3 — "this set CONTAINS the seed ... treat that plate as the secret itself" reads as sufficiency on every multisig path

**Where:** plan §4.4 (both presence arms), §4.3's routing (`gui/multisig.go:362`
→ `seedCapacityOne`; `gui/multisig_build.go:479` → `seedCapacityMany`).

**The defect.** §4.4 is explicit that sufficiency must not be claimed:

> Deliberately **not** claimed on the presence arm: that the seed plates *alone*
> suffice to spend (true on single-sig, false on a k-of-n multisig set)

The text does not deliver that intent. `"Seed: this set CONTAINS **the** seed"` —
definite article, no possessive — placed directly beneath `"If any of them is
missing, this backup is incomplete."` answers the one question the document
exists to answer ("is this everything?") with *yes*. On a 2-of-3 supplied policy
the set contains one cosigner's seed and the answer is *no*: two more signatures
are required and nothing on the page says where they live. The same applies to the
`seedCapacityMany` arm on a build where the operator holds fewer slots than the
threshold.

The codebase already has the right word and the plan drops it:
`oneSeedPassphraseFact` labels the fact `"your seed"`
(`gui/multisig_build_census.go:198`), and the passphrase lines it feeds read
`"Needs a passphrase: your seed"`. `"your seed"` is true in all six modes;
`"the seed"` is unambiguous in exactly one of them (single-sig full).

**Mitigating, stated so the fold does not over-correct:** on the `expandOK`
branch the multisig document does carry `"Type: <script> 2-of-3 multisig
(sorted)"` (`gui/multisig_restore.go:63-64` → `desc4Display` → `policyLine`,
`gui/md1_inspect.go:69-76`). A reader who parses that line has the threshold. A
reader who does not — the non-technical heir this document is written for, per
`gui/multisig_build_census.go:110-114` — has a sentence telling them the secret
is in the pile.

**The harm.** An heir concludes the steel is self-sufficient, stops looking for
the other cosigners' material, and the recovery that would have worked is
abandoned. It is the same harm §4.4's own "deliberately not claimed" paragraph
was written to prevent.

**Suggested remedy (UNVERIFIED).** The one-word change to `your seed` /
`your seeds` closes the definite-article reading; whether the k-of-n case also
needs a positive sentence ("this is one cosigner's share") is a design call this
review does not make.

---

### I-4 — §5's "existing tests that must be updated" list is incomplete, and the not-weakened rule is scoped so narrowly that the only end-to-end proof of full mode can be deleted in good faith

**Where:** plan §5 (final paragraph), §4.6 (F-202), §1.7 (prior-art table).

**The defect.** §5 closes with:

> **Existing tests that must be updated, not weakened:** the six
> `buildPlateInventoryLines` call sites in §4.3. Any assertion deleted rather than
> re-parameterised is a blocking finding.

That reads as the complete list. It is not. §4.6 inserts a
`confirmReviewScreen(ctx, th, "Plates To Cut", ...)` between the wallet-policy
picker and the engrave. Both existing single-sig flow tests drive that stretch by
button sequence and then wait on the engrave title:

- `gui/singlesig_flow_test.go:83-86` — `click(Button3)` on "Engrave wallet
  policy", then `pumpUntil(frame, "Card 1 of 3", 64)`
- `gui/singlesig_flow_test.go:122-125` — same shape, `"Card 1 of 2"`

After F-202 the flow parks on the census and neither `pumpUntil` ever sees its
needle, so both `t.Fatalf`. These are the two tests §1.7 names as the harness the
implementer should mirror, so they are the first thing the implementer touches
and the plan says nothing about them.

**The harm.** `Card 1 of 3` versus `Card 1 of 2` is the **only executing
assertion in the tree that full mode puts the ms1 seed plate on steel and
watch-only does not.** The correct repair is one extra confirm press. A plausible
wrong repair — relaxing the needle to `"Card 1 of"`, or dropping the assertion to
get green — silently retires that proof, and §5's not-weakened rule does not
reach it because it is scoped to the six census call sites. Per the brief's own
standard: the sentence permits a wrong implementation.

Machine-checked: the six `buildPlateInventoryLines` call sites §4.3 lists are
exactly right (`grep -rn buildPlateInventoryLines gui/` → the two production
sites plus `multisig_build_prose_test.go:369,424,425` and
`multisig_build_perseed_passphrase_test.go:134,246,304`). It is the *other*
tests the list omits.

---

## Minors and Nits (recorded, do not gate)

### M-1 — the absence arm's address claim contradicts the line above it on a non-bip380 supplied policy
`"these plates can rebuild the wallet's addresses"` (§4.4) is appended to a
multisig supply document that, for a policy `expandedToDescriptor` cannot render,
already says `"Addresses unavailable for this policy shape."`
(`gui/multisig_restore.go:26-31`). Reachable: `allSlotsHaveXpub`
(`gui/multisig.go:115`) admits a full-keyed miniscript policy that is not
bip380-expressible. The claim is defensible about the *plates* with an off-device
toolkit; it is a visible contradiction on the page.

### M-2 — `restoreDocFlow`'s two error returns drop the entire inventory after a completed engrave
`gui/singlesig_restore.go:122` and `:127` `showError` and `return` **before**
`restoreDocScreen`. Under §4.2 the inventory rides in as `extra` and is appended
only on the success path, so on either error the operator gets no plate count, no
seed statement and — the half F-198 is Critical for — no passphrase statement,
after a set that was fully cut. Low reachability (all four `md.ScriptKind` values
map, and the xpub is device-derived), and `multisigRestoreDocFlow:103` has the
same shape today. Worth one line in the plan naming it rather than discovering it.

### M-3 — the template-only single-sig branch is untouched and its inventory mislabels the plate
On the opt-in template path (`gui/singlesig.go:101-122`) the engraved md1 is
keyless, but `singleSigEngraveCards` hard-codes
`summary: "wallet policy descriptor"` (`gui/singlesig_engrave.go:41`), so §4.2's
inventory calls a template plate the wallet policy. Separately: the build path
**skips** the restore document entirely for a template
(`gui/multisig_build.go:464`) while single-sig prints one built from the live
`xpub` — a full descriptor that is not on the plates. Neither loses funds (the
mk1 is in the set, and `templateWarningLines` already states the recovery
dependency), but the plan's §3 inventory does not mention the template branch at
all.

### M-4 — `"Verify OK — The engraved bundle matches the seed."` on a passphrase run
`gui/singlesig_verify.go:148`. True and incomplete, on the most vouching sentence
in the flow. The truthful label precedes it (§4.1) and the truthful document
follows it (§4.2), so the operator is bracketed by corrections — which is why
this is Minor and not Important. Named because §0 rule 2 says *every* screen.

### M-5 — FILE, not fix
`gui/singlesig_verify.go:145` tells a failed verify to "Check the engraved
plates" where the multisig sibling explicitly rules the passphrase must be
suspected first (`multisigVerifyNoSlotBody`, `gui/multisig_verify.go:151-165`:
"Check the passphrase before you doubt the plates"). A mistyped passphrase at
verify sends the operator to destroy correct steel.

### M-6 — FILE, not fix (same class as §1.5, one file over)
`gui/multisig_build.go:439` — "Only verifyComplete falls through to the restore
document" — is false: `if !ok || sel != 0 { break }` means `CONTINUE` and a
refusal fall through too. `gui/multisig.go:323` carries the same sentence and
then contradicts it in its own next clause. This is exactly the inherited-
assertion class §1.5 is correcting in `bundle_flow.go:535`, and it is load-bearing
for C-1's remedy.

### N-1 — FILE, not fix
`backupWalletFlow` (`gui/gui.go:2419-2432`) lets the operator engrave a
**passphrase-derived** master fingerprint onto a seed plate that carries only the
words, and says nothing about the missing factor; `deriveXpubFlow`
(`gui/derive_xpub.go:344-354`) mints a passphrase-bound mk1 with the same silence.
Neither produces a document, so neither is the C-1 class, but both are the F-198
class without the vouching half.

---

## WHAT I CHECKED AND FOUND SOUND

- **The engrave call-site inventory is complete.** `grep -rn "bundleEngrave(ctx"`
  → exactly the four production sites §1.5 tables. `multiPlateEngrave`
  (`gui/derive_xpub.go:390`) is a fifth engrave path with its own abort warning
  and **no** post-engrave tail; `bip85.go:346`, `slip39_polish.go:507`,
  `freetext_flow.go:1604`, `passphrase_flow.go:699`, `gui.go:2393/2440/2463/2479`
  and the unlock paths all engrave without any completeness document. No other
  engrave-plus-document route exists.
- **§4.1's predicate is in scope and correct.** `passphrase` is bound at
  `gui/singlesig.go:64-74`, before the mode picker at `:77`, and is the *same*
  variable passed to `deriveSingleSigBundle` at `:90`. `passphrase != ""` therefore
  matches the derivation exactly, including the `("", true)` explicit-empty return
  from `syswPassphraseFlow` and the abort-at-keyboard case (operator picks "Add
  passphrase", backs out, derivation proceeds bare — and the new document
  correctly says "No BIP-39 passphrase was used").
- **`bundleSetCarriesASecret` is a faithful watch-only discriminant on all three
  paths.** Single-sig: `full` gates the `cardMS1` at `singlesig_engrave.go:23`.
  Build: `full` reaches `deriveMultisigLeg`, and a build with no held slot is
  refused outright (`errBuildNoHeldSlot`), so full mode always yields ≥1 ms1.
  Supply: one seed seam, one ms1. There is no "full build with no seed plate"
  state that would print the absence arm.
- **§3.1.5's title reuse does not break the needle gate.** `"Plates To Cut"` is
  not in `buildFlowNeedles` and not in `decoyNeedles` (`cmd/emu/needle_test.go`);
  the pinned census needle is `"Plate Count"` → `gui/multisig_build.go`, which
  stays single-site. No walk drives the single-sig flow (`grep` over
  `cmd/emu/walk_*.js` → no hits), so the census insertion breaks no walk.
- **The ASCII guard admits the new strings.** `multisig_build_prose_test.go:395`
  bans `—–·''""…`; the straight apostrophes in `'ms1 secret share'` are not in
  that set, and straight apostrophes already ship in operator strings
  (`derive_xpub.go:496`, `bundle_flow.go:488`), so the body face carries them.
- **§4.2's snippet type-checks against the real call site.**
  `deriveSingleSigBundle` returns `(bundle.Bundle, uint32, uint32, string, error)`
  at `gui/singlesig.go:90`, matching the proposed
  `restoreDocFlow(ctx, th, xpub, masterFP, parentFP, script, path, extra)`
  signature; `cards` is in scope from `:126`. §1.8's blast radius (one production
  call site, zero test call sites) is correct.
- **§3.1.7's prefix claim holds.** `numberedLabel` returns the bare
  `"ms1 secret share"` for n≤1 and `"ms1 secret share i of n"` otherwise, so "the
  plate marked 'ms1 secret share'" matches both.
- **§3.1.1's supply-path capacity claim is correct.** `supplyMultisigPolicyFlow`
  has exactly one `seedEntryFlow` seam (`gui/multisig.go:124`); the supply
  document really does move onto the one-seed arm, and its shipped
  "can hold several" really is false there today.
- **T5's vacuity risk is already handled.** §8.3 requires T5 to assert it saw
  `Bundle Incomplete` before asserting what came after, which also covers the new
  hazard F-202 introduces (a Back at the census returns without ever reaching
  `bundleEngrave`).
- **§8's declared blind spots are honest** — the paging-overflow gap (§8.2) and
  the unchecked-capacity-argument gap (§8.4) are both real and both named. I did
  not find an *undeclared* one beyond the findings above.

---

## What the next round should NOT re-ask

Line numbers (gated, 49/49). The Rust-primary check (§2, negative). §1's measured
facts, all of which I read for meaning and none of which I found contradicted.
The suite's green baseline. Whether the label change breaks a test that pins the
literal (§1.9 — it does not; the flow tests select by index, and it is the *census
screen*, not the label, that breaks them — see I-4).
