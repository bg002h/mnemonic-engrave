# SPEC — S6b, the single pre-flash cycle

**Status: DRAFT, ungated.** No code may be written against it until it passes R0
at 0 Critical / 0 Important.

**This is a clean rewrite**, replacing a version folded four times. Each fold had
left its superseded text visible for auditability, and five of the nineteen
findings in the comprehension review were artifacts of that style rather than of
the design — including one that reintroduced a Critical verbatim inside the
commentary about fixing it. **This document therefore contains no correction
blocks and no superseded text.** Every sentence is live. The history is in git:
`git log -p design/SPEC_s6b_pre_flash_cycle.md`, plus five reviews committed
verbatim under `design/agent-reports/`.

**Authority.** Decisions are not re-opened here; this turns them into normative
behaviour and gates. **Cross-document references are always qualified by
document** — a bare `§2.4` was previously ambiguous across three files.

| document | carries |
| --- | --- |
| `REQUIREMENTS_s6b_pre_flash_cycle.md` §1 | operator directives R1–R7 |
| `REQUIREMENTS_s6b_pre_flash_cycle.md` §2 | measured facts |
| `REQUIREMENTS_s6b_pre_flash_cycle.md` §2bis | rulings **R-A … R-M** |
| `PROPOSAL_s6b_q1_q3_q6.md` | Q1/Q3/Q6, approved |
| `SPIKE_s6b_q2_results.md` | executable measurements |

Code facts read from fork `bg002h/seedhammer` `main` = `b1479a1`.

---

## 0. SCOPE

**In:** F-199, F-204, F-206, F-192, F-208, single-sig plate marking
(R2/R3/R4/R6/R7), R-M's multisig `provedInnocent` arm, and **the restore
document's passphrase paragraph** (§6).

**Out, explicitly:**

- **Multisig marking** and F-205 → phase `key & password custody refinement`
  (**R-B**). Not closed here, even incidentally — see §1.3.
- **Watch-only marking** (**R-A**).
- **Restoring `fadeClip`'s clip mask** (**R-E**).
- Any change to `md1`/`mk1`/`ms1` **wire format**. This is plate layout and text
  only, so the Rust-primary rule is not triggered — re-verify at implementation,
  because it is what keeps S6b fork-native.

---

## 1. PLATE MARKING (R2, R3, R4)

### 1.1 `backup.Text` gains optional `Title` and `Footer`

Per **R-F**. `backup.Text` today carries `Paragraphs`, `Font`, `FontSize` and
renders no title or footer row (`backup/backup.go:33-41`, `:350-446`).

1. `Text` gains two optional string fields, rendered through the layout helpers
   `EngraveText` already shares with `Fitted` — `textLayout`
   (`backup/wrap.go:224`), `qrPlaceAt` (`backup/wrap.go:196`), `WrapText`.
2. **The title is plate row 0; the footer is the last plate row.** Both are
   screw-hole rows. This is normative and load-bearing: §1.2's 25-character
   budget is the inset span for *those rows only*. A footer placed directly
   after the paragraph text would sit mid-plate, outside the hole inset and
   inside the QR keep-out band (`qrPlaceAt` sets `Top = anchorY +
   holeLines*fontSize`), where nothing in the render layer would refuse it.
3. **An empty `Title` renders no row and consumes no vertical budget.** Likewise
   `Footer`. Normative, not an optimisation: it is what makes R-A and R-B
   enforceable at the call site and R-G's no-churn assertion possible.
4. **`validateMdmk` learns nothing about flows.** The caller decides.

### 1.2 What the marking says

| slot | content | when |
| --- | --- | --- |
| **Title** | `PASSWORD REQUIRED` (17) | iff the set contains a seed **and** was derived with a BIP-39 passphrase |
| **Footer** | `COMB FP: <8 hex, grouped>` (18) | iff the set contains a seed **and** was derived with a passphrase |
| **Footer** | `SEED FP: <8 hex, grouped>` (18) | iff the set contains a seed and was **not** derived with a passphrase |

Both rows read off **R-A**'s predicate — the set contains a seed — so a
passphrase-derived **watch-only** engrave (reachable: `gui/singlesig.go:97-103`
offers `"Watch-only (keys)"` with a passphrase already entered) is unmarked.

**Why the label changes with the passphrase.** `COMB FP` names a *combination*.
On a bare-seed set no combination exists, and a reader years later would look for
a second factor that was never used. The value is the same master fingerprint
either way; only the honest label differs.

**`PASSWORD REQUIRED` is the operator's wording** (`REQUIREMENTS` §2.4): it
matches what the device already shows on screen (`gui/gui.go:1997`,
`gui/passphrase_flow.go:645`), which matters more on steel read years later than
consistency with the source's vocabulary.

**Why the footer carries the combined fingerprint.** `mk1` carries exactly one
per-key identifier, `origin_fingerprint` — *"Master-key fingerprint … Verbatim
from BIP 380 origin notation"* — and a passphrase **changes the master key**, so
on a passphrase-derived wallet that value is the combined fingerprint
(`REQUIREMENTS` §2.1 measured `73c5da0a` bare vs `fc60c6df` combined). **That is
the whole mechanism R4 exploits:** restoring the words alone yields a fingerprint
that does not match what the plates encode, so a wrong-wallet restore
self-diagnoses instead of failing silently.

**The budget is 28 characters**, measured layout-based during P2 at
`plateFontSizeUR` = 3.8 mm. `SPIKE_s6b_q2_results.md` §3c reported **25** by raw
string width — and **said so, and said raw width under-reports**. It does: the
true bound is 28, so 25 was conservative rather than wrong, exactly as that
caveat predicted.

Every string above fits with ≥ 10 to spare. The merged two-fingerprint form (30)
still does **not** fit and is not used. **The gate asserts the measured 28, not
the estimate** — see `backup/engravetext_test.go`.

**Marking applies to `cardMK1` and `cardMD1` plates only. A `cardMS1` plate is
never marked.** In full mode the set includes an ms1 secret share —
`singleSigEngraveCards` prepends `bundleCard{kind: cardMS1, …}` when `full`
(`gui/singlesig_engrave.go:20-28`) — and its string goes through the same
`validateMdmk`. R3 and R4 name mk1 and md1 only; `ms1` is
passphrase-**independent**, so a combined fingerprint is the one line on that
plate that would not describe what the plate encodes; and it would tie a
**secret** share to a specific wallet on an artifact whose design posture
(`gui/singlesig_engrave.go:17-19`) is that it never leaves owner-held steel.

**Mechanism:** `bundlePlate` (`gui/bundle_flow.go:346-353`) gains
`kind bundleCardKind`, populated `kind: c.kind` in `bundlePlatePlan`
(`:358-373`, which already iterates the `bundleCard`); `bundleEngrave` applies
the title/footer only when `p.kind != cardMS1`. One field; the fact is already
one level up and simply not copied through.

**The marking renders on all three engraving variants.** `validateMdmk`
(`gui/gui.go:2288-2320`) offers `TEXT + QR`, `TEXT ONLY` and `QR ONLY`, and the
operator chooses. At 67–111 characters all three fit, so `QR ONLY` — a paragraph
with no text at all — is reachable and selectable. Title and footer are plate
rows, not paragraph content, and render in every variant.

### 1.3 The marking is CONDITIONED one frame above `validateMdmk`

`validateMdmk`'s own call sites cannot evaluate §1.2's conditions:
`gui/gui.go:2344` sits in `mdmkFlow(ctx, th, s mdmkText)` — a bare string from a
scan, and whether it was passphrase-derived **is not decidable from the string**;
`gui/bundle_flow.go:407` sits in `bundleEngrave(ctx, th, title string, cards
[]bundleCard)`, and `bundleCard` carries no seed or derivation record.

**`bundleEngrave` grows two string parameters, passed through to `validateMdmk`.
`gui/singlesig.go:177` is the only caller that passes non-empty values**,
computed there from `passphrase != ""` and `full`, both in scope (`:97`, `:103`,
`:107`).

**Go has no default parameters** — this codebase says so at the function under
discussion (`gui/bundle_flow.go:395-397`). So every other caller passes `""`,
`""` explicitly. **Two tests assert the call text as a source string and must be
updated in the same commit:** `gui/multisig_verify_report_test.go:940` and
`:942`.

*(This section originally named a third, `gui/bundle_abort_prose_test.go:258`.
P2 checked it empirically rather than editing it because a document said to, and
found its substring-window check tolerates the new trailing arguments. Corrected
here so the next reader does not "fix" a test that was never broken.)*

**A variadic tail is prohibited**: it would leave order and arity unchecked on
the value that decides whether a plate says `PASSWORD REQUIRED`. **Shared state
on `Context` or a package variable is prohibited**: it would let the marking
reach `"Engrave Multisig"` and `"Build Policy"` and cross the R-B boundary this
section exists to make structural.

| call path | marked |
| --- | --- |
| **`gui/singlesig.go:177`** → `bundleEngrave` → `validateMdmk` | **yes, per §1.2** |
| `gui/multisig.go:291` → `bundleEngrave` | no — R-B |
| `gui/multisig_build.go:402` → `bundleEngrave` | no — R-B |
| `gui/bundle_flow.go:39` → `bundleEngrave` | no — scanned, no derivation |
| `gui/gui.go:2344` `mdmkFlow` | no — not decidable |
| `gui/unlock_platelist.go:222` | no |
| `gui/derive_xpub.go:494` | **no — F-205's flow, R-B** |

---

## 2. THE PASSPHRASE PLATE (R2, R5, R6, R7)

### 2.1 The program runs preloaded, and what that requires

Per **R-C**: the device runs the existing dedicated passphrase-plate program with
values already in hand rather than building a new offer flow. Per **R-J** it
preloads the **fingerprints** too, not only the passphrase.

**The existing entry point cannot carry them, and this is the mechanism change
the cycle owes.** Verified: `engravePassphraseFlowFrom` takes
`(ctx, th, body []byte, src syswSource)` (`gui/passphrase_flow.go:617`);
`ppBuildPlate` takes `(params, secret []byte, seedFP, combinedFP string, qr
bool)` (`:546`); `backup.Passphrase` is `{Passphrase, SeedFP, CombinedFP, QR,
Font}` (`backup/passphrase.go:23-31`) — no policy-id field. `seedFP` and
`combinedFP` are declared inside the function (`:645`) and written only by
`ppStepSeedFP`/`ppStepCombinedFP`.

**NORMATIVE:**

1. The preloaded entry carries **seed FP, combined FP and the policy-id hex as
   parameters**, and the plate type gains a policy-id field.
2. **The two fingerprint steps are ELIDED from the step sequence, not skipped
   inside it.** The step machine is an integer loop whose Back transition is
   `step -= 2` (`gui/passphrase_flow.go:656-706`); leaving the steps present and
   short-circuiting them lands Back from `ppStepQR` on `ppStepCombinedFP`. The
   sequence itself must be shorter on this path.
3. **No package-level variable and no field on `Context`.** Either would make a
   secret-adjacent value outlive one flow.

**Re-typing is not the safer alternative**, and the reason is recorded so it is
not "corrected" later: the preloaded passphrase is the one the device actually
derived with, so the plate records the passphrase belonging to the wallet that
was engraved. Re-typing adds a second chance to disagree, not a check.

**`SPEC_seedhammer_systemwide_payloads.md` §7.4 does not bind.** It forbids the
session cache answering a **verification** prompt, because verify would then
compare the engrave source against itself. R-C preloads an **engrave** input. The
shapes rhyme; the distinction is stated rather than left implicit.

### 2.2 A new `syswSource` value is required

Forced by **R-D** plus **R-C**: `syswSourceAccept` (`gui/sysw_source.go:113`)
runs for the preloaded passphrase and prints `"Source: " + syswSourceName(src)`
(`:127`). `syswSourceName` (`:9`) resolves `srcTyped` through its `default:` arm
to **`"the keyboard"`**. A preloaded passphrase did not come from the keyboard.

1. A new `syswSource` value is added for "carried from this session's own
   derivation".
2. **It ships with an explicit `case` in `syswSourceName`.** The `default:` arm
   returns `"the keyboard"`, so a value added without its case becomes a
   **printed falsehood** with no compile error and no failing test.
3. The `"Source: …"` line appears automatically: `syswFlags`
   (`gui/sysw_admit.go:91-109`) raises `flagSource` on `src != srcTyped`, and
   raises nothing else for a new value. Admission (`:47`) is keyed on program ×
   class, not source, so no admission table changes.

### 2.3 The footer

```
preloaded    "POLICY <8 hex, grouped>  DERIVED"    25 chars
standalone   "FINGERPRINTS TYPED, NOT VERIFIED"    32 chars
```

**Band budget is 32 characters × 2 lines**, from spec 4.3 — *no metadata line
may exceed 64 mm* = 409600 device units — sized to clear the 10 mm corner
screw-hole bands by 0.5 mm each side, and enforced by `backup`'s pre-existing
`TestPassphraseBandBudget`.

**Not 42.** `SPIKE_s6b_q2_results.md` §3b's 42 is where a line runs off the
**plate edge** (544000 units), which is a looser and wrong bound for a metadata
line. R-H's literal string `POLICY 1A2B 3C4D  DERIVED, NOT TYPED` measures 36
chars / 460800 units — **8 mm over the cap, into the screw-hole zone** — and was
corrected during P3 by dropping `, NOT TYPED`. `DERIVED` alone is already a
positive true claim on this path, and the acceptance screen immediately before
it states the source in those words (§2.2).

**The standalone footer sits at exactly 409600 — the cap, with zero spare.** Any
edit to it overruns.

The preloaded form consumes **no new band line** — the worst case (both
fingerprints plus a spaced passphrase) already fills 2 top and 2 bottom, and a
third line does not error or clip: it silently cuts into the 3 mm outer margin.

**`DERIVED` is NOT operator-approved wording**, unlike R-M's arm. It is the
minimal truthful string that fits, chosen in implementation and flagged for
confirmation — see §8.

**What selects between them is a recorded PROVENANCE, not the policy id.**
`backup.Passphrase` gains a field stating whether the fingerprints were derived
or typed, **independent** of whether a policy id is present. Keying the footer on
`PolicyID != ""` would couple two unrelated facts, and any future path that
preloads fingerprints without a descriptor would then print `TYPED` over derived
values — a falsehood in the direction GATE 2.3b does not test.

**The `DERIVED` claim is true only if §2.1's elision ships with it.** They land
in the same change.

### 2.4 The identifiers

- **"wallet policy id"** = **`md.FormAwareStubChunks`** — top 4 bytes → 8 hex,
  grouped `XXXX XXXX`. **Not `md.WalletPolicyIDStub`**, which is the *keyed*
  branch reached through `md.FormAwareStub` (`md/template_id.go:112-118`); every
  production stub site uses the form-aware form. `"Template-only md1"` is an
  offered choice on this very flow (`gui/singlesig.go:118-139`), and the keyed
  function would engrave four bytes matching no card the run cut.
- **It is computed from the FINAL `b.MD1`, at the offer site.**
  `templateizeBundle` (`gui/template_engrave.go:24-38`) replaces the bundle and
  re-mints the stub, and `gui/singlesig.go:139` assigns `b = tb`. A value
  captured near `:107` would be the pre-template one.
- **"key-id"** = the **master fingerprint**, already on the plate as
  `SeedFP`/`CombinedFP` (`backup/passphrase.go:176-180`). `mk1` defines no other
  per-key identifier; `mk.Header.ChunkSetID` is a chunk-**reassembly** id, not an
  identity.
- **The label must be true of both forms.** The stub is form-aware: the keyed
  `WalletPolicyId` for a wallet-policy `md1`, the key-stable
  `WalletDescriptorTemplateId` for a keyless template. A label reading "wallet
  policy id" is false on the template form.

`backup` takes **no dependency on `md`** — the plate's fields are pre-formatted
hex strings and the caller passes hex, exactly as it already does for
fingerprints. The standalone path has no descriptor, passes `""`, renders no
line.

**`origin_fingerprint` is optional in the format but this device always sets
it** — the header flag is set iff `card.Fingerprint != ""` (`mk/encode.go:70-80`)
and all three device-side `mk.Card` sites set it (`gui/singlesig_derive.go:77`,
`gui/multisig_derive.go:51`, `gui/derive_xpub.go:369`). A fingerprint-less `mk1`
can only arrive from another tool, over scanned paths §1.3 does not mark.

### 2.5 The QR stays password-only

Already true: `qr.Encode(plate.Passphrase, qr.L)` (`backup/passphrase.go:86`).
Pinned by a test so no future edit folds metadata into it.

### 2.6 Where the offer goes, and the lazy derivation

**No passphrase-plate offer exists today** — `gui/singlesig.go` contains zero
references to one.

**The offer is inserted in `engraveSingleSigFlow` (`gui/singlesig.go:38`),
between the verify offer (`:188-192`) and `restoreDocFlow` (`:221-223`).**
Everything it needs is in scope there: `passphrase` (`:97`, read at `:223`),
`masterFP` (`:107`), the final `b.MD1` (post-`:139`), and a live mnemonic — the
scrub is a `defer` registered at `:50-54` that fires at **return**.

After the verify, so a plate is offered only for a set known good. Before the
restore document, because the document must report whether a passphrase plate was
cut (§6).

**The bare-seed fingerprint derives LAZILY** — only when the operator elects to
engrave a passphrase plate — because it costs a **~31 s KDF**
(`gui/gui.go:825`, `:1653`, `gui/unlock_platelist.go:175`). The combined
fingerprint is free: it falls out of the existing derivation at `:107`.

Deferring it is sound because the mnemonic lives until return, and permitted
regardless by **R-K**: the device is offline, cannot write to flash, and is
disposable; **sealed payload is the security program, and the remaining programs
favour convenience over security.** Single-sig engrave is not the sealed-payload
program. **R-K does not relax R-D** — it concerns secret lifetime, not
truthfulness.

---

## 3. THE VERIFY TAIL (F-199, F-204, F-206, R-M)

### 3.1 F-199 — `verifyRefused` re-offers at ONE site

| line | trigger | correctable |
| --- | --- | --- |
| 717 | `len(expectedSlots) == 0` | no — programmer error |
| 727 | `len(engravedMd1) == 0` | no — programmer error |
| **753** | `extractReadbackMd1AndMk1s` fails on gathered cards | **yes** |
| 854 | `verifyFreshSlots` → `ferr != nil` | no — a defensive re-check of 717 |

Only `:753` re-offers. **Widening the verdict is prohibited** — it would make all
four loop, including two programmer-error refusals.

`:854` is **unreachable in-process**: `verifyFreshSlots`
(`gui/multisig_verify.go:324-336`) has one error return, on `len(expected) == 0`;
`expectedSlots` is never reassigned in `multisigVerifyFlow`; and `:715-717`
returns before `:851` can run.

### 3.2 F-204 — the failed single-sig verify stops blaming the plates

`gui/singlesig_verify.go:182` says *"Check the engraved plates."* The multisig
sibling rules the other way (`gui/multisig_verify.go:157`). The asymmetry costs
steel: verify requires the seed **re-typed**, and one wrong passphrase character
derives an entirely different wallet, so a mistyped passphrase is a common cause
of a FAILED comparison on **correct** plates.

**The copy is CONDITIONAL, not a string swap.** The multisig wording is true only
when a passphrase was entered, and `passphrase` is in scope at the failure site
(`gui/singlesig_verify.go:108-112`):

- **passphrase entered** → suspect the passphrase before the plates;
- **no passphrase** → that wording would be a false lead; the copy says something
  true of that case.

### 3.2a R-M — the multisig `provedInnocent` arm

`multisigVerifyNoSlotBody`'s `provedInnocent` arm
(`gui/multisig_verify.go:157-170`) currently ends *"Your plates are fine. Try
again and skip the passphrase."* **R-M** struck the skip advice and ruled that
the operator must be told this is not a passphrase-protected wallet.

```
These plates match this seed with NO passphrase. This is not a
passphrase-protected wallet. If you meant to use one, these plates are not that
wallet: try the password again. If you continue without a passphrase, these
plates are complete as they are.
```

**`"A passphrase will be necessary to use the key"` is FORBIDDEN here** — false
on this arm, which fires precisely because the plates match with **no**
passphrase.

Pre-measured: **251 characters**, every rune drawable, **no em dash** — the last
a hard constraint (`gui/multisig_build.go:735-739`: an em dash in a modal body
once meant *"the BODY DID NOT DRAW AT ALL"*).

**`multisigVerifyNoSlotBody`'s doc comment (`gui/multisig_verify.go:150-156`)
describes the arm being replaced and is updated in the same commit.**

Copy only — no control flow, no marking — so it does not cross R-B.

### 3.3 F-206 — the ms1 clause becomes count-free

`verifyStatusMS1Clause` (`gui/verify_status.go:155`) is the fixed singular *"The
ms1 secret you typed matched this seed."*, appended whenever `p.full`
(`buildVerifyPassLine`, `:211-231`) — unconditionally on `p.legs`.

**F-206's filed remedy is unsound and is not used.** It proposes pluralising over
`passRecord.legs`, but `legs` is `len(legs)` (`gui/multisig_verify.go:1059`) —
one leg per **filled slot** — and *"one seed fills several slots of a policy that
puts it at several accounts"* (`:299-300`). On one seed at two accounts that
would print *"the ms1 secrets you typed"* when the operator typed **one**: a new
falsehood in the over-claiming direction, which **R-D** forbids.

**Replacement clause:**

```
The ms1 you typed for each seed matched.
```

True at any seed count, requiring none — the shape the device's own screen
already uses (`gui/multisig_verify.go:1134-1135`). **No new recorded fact, so
NG1 stays closed.**

---

## 4. MODAL FIT (F-192)

Every long modal body is gated by the F-185 class check, which exists and is a
one-line call. Only S5.C's screens are measured today.

**Do not re-derive the mechanism.** It is not a character budget: capacity
depends on how words **wrap** (588 normalised chars of short-word filler fit,
while F-185's real refusal was cut at ~500), so the check compares the **drawn
frame** to the source string and binary-searches the cut point.

**R-I decoupled this from F-208** — the arrows cost no body width — so this sweep
does not wait on them and its measurements stay valid.

The sweep **states its own coverage**: which modals were gated, and which were
judged not "long" and why. A sweep that silently skips is the defect it fixes.

---

## 5. SCROLL ARROWS (F-208)

Per **R-I**: arrows draw at the **top-centre and bottom-centre of the body**,
over the 16 px fade zone, each with a background chip and an enlarged invisible
touch target. Geometry: panel 480×320; nav slots 53×53 at y = 44/133/223,
x = 427–480; body clip (6,44)–(423,314) = **417 wide**; arrows **15×9**.

1. **Not through `layoutNavigation`.** It computes
   `idx := int(clk.Button - Button1)` into a `[3]int`, and `Up`/`Down` sort
   before `Button1`, so they index **negative**.
2. **Body width stays 417.** If an implementation changes it, R-I's decoupling of
   §4 is void and §4's measurements must be retaken.
3. **The chip is opaque and mandatory**, so §5 states its drawn bounds and the
   body's first and last drawn text rows must clear them. The body starts at
   `bodyClip.Min.Y + scrollFadeDist` (`gui/gui.go:416`) — *inside* the zone the
   arrows occupy. **§4's sweep cannot catch this**: it compares the op tree
   (`bodyDrawnFully`, `gui/modal_fits_test.go:81-100`), and a glyph drawn *under*
   an opaque chip is still in the op tree (`:22-27`).
4. Scrolling needs no new machinery: a `Clickable` bound to `Up`/`Down` gets
   pointer routing (`gui/widget.go:70`) and press-and-hold auto-repeat (`:48`).

### 5.1 The visibility predicate

Arrows render **iff content is actually off the panel**. While `fadeClip` stays
stubbed (**R-E**), that is **not** `maxScroll > 0`, and it is **not**
`bodysz.Y > bodyClip.Dy()` either.

Measured: `leadingSize` = 44 (`gui/theme.go:43`), `boxMargin` = 6
(`gui/gui.go:400`), `scrollFadeDist` = 16 (`:761`), `dims.Y` = 320. The body's
drawn top is `bodyClip.Min.Y + scrollFadeDist` = **60** (`:416`, at
`scroll == 0`), and `bodyClip.Dy()` = **270**.

Content is off the panel iff `60 + bodysz.Y > 320`, i.e. **`bodysz.Y > 260`**.
A predicate of `bodysz.Y > bodyClip.Dy()` fires at 270 — short by
`scrollFadeDist - boxMargin` = **10 px**, a **false negative** that hides content
below the panel edge with no arrow. That is F-185's own harm.

**NORMATIVE:**

```
show arrows  iff  bodyClip.Min.Y + scrollFadeDist + bodysz.Y > dims.Y
```

Against the **panel**, which is what R-E required. The implementation carries a
comment naming R-E, because this changes when the mask is restored.

---

## 6. THE RESTORE DOCUMENT (R1)

**R1 is not "already shipped and untouched" — S6b makes its shipped text
false.** `buildPassphraseInventoryLines` (`gui/multisig_build_census.go:258-279`)
prints, when a passphrase was used:

> *"A BIP-39 passphrase WAS used. It is not on these plates and cannot be
> recovered from them: **nothing this device engraves carries a passphrase.**"*

`engraveSingleSigFlow` passes `oneSeedPassphraseFact(passphrase != "")`
(`gui/singlesig.go:223`), and §2.6's offer appears on **the same predicate**. So
on every run where S6b can cut a passphrase plate, the document printed minutes
later asserts the device engraves no passphrase.

**NORMATIVE:** that sentence is conditional on whether **this run cut a
passphrase plate** — not on the flow, since the multisig build path reaches the
same function. When one was cut, the document says so and says where it is.

**CUT, not OFFERED — and the mechanism does not exist yet.** The operator can
decline the offer, or abort mid-engrave, and in both cases the shipped sentence
stays true and must not change. But `engravePassphraseFlowFrom`
(`gui/passphrase_flow.go:617`) and `engravePassphraseFlow` (`:605`) **return
nothing**, so today the caller cannot tell a completed plate from a declined
offer.

**The passphrase flow returns a result, following the idiom already beside it**:
`bundleEngrave` returns `bundleEngraveResult` with `bundleEngraveDone` meaning
*"every plate in the plan was engraved"* (`gui/bundle_flow.go:404,443,455-456`).
§6's condition reads that result. Conditioning on the offer having been *shown*
would put the claim on runs that cut nothing — a falsehood in the opposite
direction, which R-D forbids equally.

### 6.1 The passphrase plate is NOT a member of the backup set

`buildPlateInventoryLines` (`gui/multisig_build_census.go:59-73`) prints
`"This backup is N plates:"` from `bundlePlatePlan(cards)`, then **"If any of
them is missing, this backup is incomplete."**

**The passphrase plate must not enter that count.** The nearest mechanism —
appending a `bundleCard` for it — would place it under that sentence, telling a
reader it travels **with** the set: the exact inverse of R1's *"Keep it somewhere
separate"* printed four lines below, and it would also flow into
`bundleSetCarriesASecret` (`gui/bundle_flow.go:492`).

**NORMATIVE:** the passphrase plate is named on a **separate line** that repeats
the separation instruction, and `len(plan)` is unchanged.

---

## 7. GATES

| # | gate |
| --- | --- |
| 1.1 | with an empty title, `backup/testdata/text-{0,1,2}-shards-1.bin` **do not move** (R-G) |
| 1.2 | the offered **variant set** is pinned for a representative `md1` and `mk1`, title empty and set; and the marking **renders in each offered variant**, `QR ONLY` included |
| 1.2a | the title/footer budget, asserted in the **layout-based** form (as `TestTitleCapFitsAtEveryRung` does), for every title and footer this cycle introduces — **on the budget, not on today's strings** |
| 1.2b | title is plate row 0 and footer the last plate row |
| 1.3 | **only** the single-sig engrave marks: `deriveXpubFlow`, `"Engrave Multisig"`, `"Build Policy"`, `"Engrave Bundle"`, `mdmkFlow` and **`cardMS1` plates** all unmarked |
| 2.1 | the preloaded path presents **no** fingerprint-entry step, and Back from `ppStepQR` lands on a real prior step |
| 2.2 | `syswSourceName`'s **rendered string** for the new value |
| 2.3 | the typed footer and a policy id never co-occur — 32 + a policy id is **50 against a 42-character band**, rendered off both plate edges with **no refusal** in `band` (`backup/passphrase.go:228-235`) |
| 2.3b | the footer does not claim derivation while `fingerprintEntryFlow` is on the path |
| 2.3c | the policy id **renders** — the footer is gated on a non-empty fingerprint (`backup/passphrase.go:185-187`), so it can vanish silently |
| 2.3d | the footer is selected by the recorded provenance, **not** by policy-id presence |
| 2.4a | the three `mk.Card` sites set `Fingerprint` |
| 2.4b | the engraved policy id **equals the `policy_id_stub` the mk1 this run cut carries**, on **both** forms — a value equality, not a label check |
| 2.4c | the stub is computed from the **post-`templateizeBundle`** `b.MD1` |
| 2.5 | the passphrase QR contains the passphrase and nothing else |
| 2.6 | the offer appears only when `passphrase != ""`; the ~31 s derivation does **not** run when no passphrase plate is engraved |
| 3.1 | **all three non-correctable arms by SOURCE assertion**, and the gate says which arm is which. `:717`/`:727` were specified as behavioural and are **not**, deliberately: whether a verdict re-offers is decided *only* by the callers' loop condition `res != verifyIncomplete && res != verifyFailed` — identical at `gui/multisig.go:346` and `gui/multisig_build.go:461`, with no retry construct at either refusal site. That is a **source fact**, so the assertion covers **both** callers uniformly where a walk samples one path, and costs ~119 s less against a `gui` package already at ~400 s of Go's 600 s ceiling. The test names what a source assertion cannot see |
| 3.2 | both F-204 arms — passphrase entered and not |
| 3.2a | R-M's body passes §4's class check, and asserts no claim that a passphrase is required |
| 3.3 | the ms1 clause at 1 seed/1 leg, **1 seed/2 legs**, and 2 seeds/2 legs. **`passRecord` cannot distinguish the last two** (`{full, legs, suppliedCosigners}`), so this is a **flow-level** assertion driving verify with two seeds — as a unit test it silently degrades to two cases and stops exercising the middle one, which is the case that kills the filed remedy |
| 4 | the sweep states its own coverage |
| 5.1 | the predicate agrees with actual visibility — **must be green** |
| 5.1b | R-E's `maxScroll` divergence probe — **failures expected, files findings, does not gate** |
| 5.3 | one pixel-level assertion: an arrow's chip does not overlap the top/bottom drawn text rows |
| 6 | the restore document does not claim the device engraves no passphrase on a run that cut a passphrase plate |
| 6a | the condition is **cut**, not **offered** — a declined offer and an aborted engrave both leave the shipped sentence unchanged |
| 6.1 | the passphrase plate is not inside `len(plan)` and not under *"If any of them is missing"* |
| — | **`me` CLI untouched**; `mk1`/`md1`/`ms1` byte-identical (§0) |

**Golden policy (R-G):** marked states get **new** golden files; the frozen
sixteen keep meaning what they meant; `backup/testdata/passphrase-*.bin` (4) will
legitimately move under §2.3 and are re-recorded **in the same commit**; **never
a bare `go test ./... -update`** — it rewrites the frozen sixteen.

---

## 8. WHAT THIS SPEC DOES NOT SETTLE

- The exact wording of §3.2's no-passphrase arm, and of §6's conditional clause.
- **§2.3's preloaded footer, `POLICY <8 hex>  DERIVED`.** R-H's literal string
  was measured 8 mm over the band cap during P3 and had to change; the
  replacement is the **minimal truthful string that fits**, not an operator
  ruling. `DERIVED` alone is a positive true claim on that path and the
  acceptance screen states the source in the same words (§2.2) — but the
  operator approved *"DERIVED, NOT TYPED"*, and this is not that. **Needs
  confirmation.**
- The label satisfying §2.4's both-forms requirement.
- **§3.2's third arm** — the multisig sibling's *"Your plates are fine"* verdict,
  adapted to single-sig. Cheap here (the flow already re-derives from a re-typed
  seed) and strictly better copy; a scope choice, not a defect.
- **`gui/singlesig_derive.go:28`'s stale doc comment** names
  `md.WalletPolicyIDStubChunks` where `:68` calls `md.FormAwareStubChunks`.
  Pre-existing, and the likely origin of the wrong-function defect §2.4 corrects
  — fix it in the same commit or it will mislead again.
- **R-H places the preloaded footer in `bottomLines` while the fingerprints it
  describes sit in `topLines`**, at the opposite edge of an 85 mm plate, so the
  isolated line reads as though the *policy* was derived. R-H is an operator
  ruling and is not re-opened — recorded so it is a choice rather than a
  discovery.
