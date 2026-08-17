# SPEC — S6b, the single pre-flash cycle

**Status: DRAFT, ungated.** It has not passed R0. **No code may be written
against it** until it converges to 0 Critical / 0 Important.

**Authority.** Every decision here is already ruled on. This document does not
re-open them; it turns them into normative behaviour and gates.

| source | what it fixes |
| --- | --- |
| `REQUIREMENTS_s6b_pre_flash_cycle.md` §1 | the four operator directives (R1–R7) |
| `REQUIREMENTS_s6b_pre_flash_cycle.md` §2 | measured facts (§2.3/§2.4 **corrected** 2026-08-17) |
| `REQUIREMENTS_s6b_pre_flash_cycle.md` §2bis | rulings **R-A … R-I** |
| `PROPOSAL_s6b_q1_q3_q6.md` | Q1/Q3/Q6, approved |
| `SPIKE_s6b_q2_results.md` | the executable measurements |

Code facts are read from fork `main` = `b1479a1`.

---

## 0. SCOPE

**In:** F-199, F-204, F-206, F-192, F-208, and **single-sig** plate marking
(R2/R3/R4/R6/R7).

**Out, explicitly:**

- **Multisig marking** and F-205 → phase `key & password custody refinement`
  (R-B). This cycle must not close them, even incidentally — see §1.3.
- **Watch-only marking** (R-A).
- **Restoring `fadeClip`'s clip mask** (R-E).
- Any change to `md1`/`mk1`/`ms1` **wire format**. §2.2 holds: this is plate
  layout and text only, so the Rust-primary rule is not triggered. **The spec
  re-asserts this and it must be re-verified at implementation**, because it is
  what keeps S6b fork-native.

---

## 1. PLATE MARKING (R2, R3, R4)

### 1.1 `backup.Text` gains optional `Title` and `Footer`

Per **R-F**. `backup.Text` today carries `Paragraphs`, `Font`, `FontSize` and
renders **no** title or footer row (`backup/backup.go:33-41`, `:350-446`).

**NORMATIVE:**

1. `Text` gains two optional string fields, rendered through the layout helpers
   `EngraveText` **already shares** with `Fitted` — `textLayout`
   (`backup/wrap.go:224`), `qrPlaceAt` (`backup/wrap.go:196`), `WrapText`.
2. **An empty `Title` renders no row and consumes no vertical budget.** An empty
   `Footer` likewise. This is normative, not an optimisation: it is what makes
   R-A and R-B enforceable at the call site and R-G's no-churn assertion
   possible.
3. **`validateMdmk` learns nothing about flows.** The *caller* decides whether a
   plate is marked, by supplying a string or not.

### 1.2 What the marking says

| slot | content | when |
| --- | --- | --- |
| **Title** | `PASSWORD REQUIRED` (17 chars) | iff the set **contains a seed** *and* was derived **with** a BIP-39 passphrase |
| **Footer** | `COMB FP: <8 hex, grouped>` (18 chars) | iff the set **contains a seed** (R-A) |

**Both rows read off R-A's single predicate.** *(Corrected by R0 round 1, I2:
the Title row previously carried only the passphrase condition, so a
passphrase-derived **watch-only** engrave — reachable, `gui/singlesig.go:97-103`
offers `"Watch-only (keys)"` with the passphrase already entered — would have
been titled `PASSWORD REQUIRED` in contradiction of R-A. The title would have
been true; the spec would have re-opened a decision the operator closed.)*

**MARKING APPLIES TO `cardMK1` AND `cardMD1` PLATES ONLY. A `cardMS1` plate is
never marked.** *(R0 round 1, I3.)* In full mode the set includes an **ms1
secret share** — `singleSigEngraveCards` prepends
`bundleCard{kind: cardMS1, …}` when `full` (`gui/singlesig_engrave.go:20-28`) —
and its string goes through the **same** `validateMdmk`, so a condition phrased
over *"the set"* marks it by default.

Three reasons it must not, and no directive covers it: R3 and R4 both name
**mk1 and md1** plates only; §2.1 measured that `ms1` is passphrase-**
independent**, so a combined fingerprint is the one line on that plate that does
**not** describe what the plate encodes; and it would tie a **secret** share to a
specific wallet on an artifact whose whole design posture
(`gui/singlesig_engrave.go:17-19`) is that it never leaves owner-held steel.

Add `cardMS1`-is-unmarked to GATE 1.3's assertion list.

### 1.2a MEASURED — the title/footer budget on a `Text` plate is 25 characters

This was §7's outstanding gate. **Run 2026-08-17**, not inferred:

- a title/footer must sit inside `[innerMargin, plateSize - innerMargin]` =
  `[64000, 480000]` = **416000 device units** (the bound
  `TestTitleCapFitsAtEveryRung` uses);
- `plateFontSizeUR`, what every `md1`/`mk1` `Text` caller constructs, is
  **3.8 mm** — *not* the free-text ladder's tight 6.0 mm rung;
- at 3.8 mm, **25 characters** fit the span.

| candidate | chars | units | fits |
| --- | --- | --- | --- |
| `PASSWORD REQUIRED` | 17 | 275621 | **yes** (66% of span) |
| `SEED FP: 73C5 DA0A` | 18 | 291834 | yes |
| `COMB FP: FC60 C6DF` | 18 | 291834 | yes |
| `SEED 73C5 DA0A  COMB FC60 C6DF` | 30 | 486390 | **NO** |
| `EXPECTED COMB FP: FC60 C6DF` | 27 | — | **NO** (> 25 chars) |

**Consequence for R4, and it narrows the directive.** R4 asks for *"Seed FP and
combined FP … in title or footer"*. **Both do not fit.** The title is taken by
`PASSWORD REQUIRED`, one footer row holds **one** 18-character fingerprint, and
the merged 30-character form — which *does* fit the passphrase band's 42-char
budget — **overruns a `Text` title/footer**.

**NORMATIVE: the footer carries the COMBINED fingerprint.** That is the one the
plates actually encode (§1.2), so it is the value a words-only restore will
**fail to reproduce** — which is the entire diagnostic R4 exists to enable. The
seed fingerprint remains available on the passphrase plate
(`backup/passphrase.go:176-180`), where the band budget accommodates both.

**Method caveat, stated so a reviewer can retest it properly.** This measured
**raw string width against the inset span**. `TestTitleCapFitsAtEveryRung`
instead drives the real layout (`layAt`, `lay.holeChars * lay.charWidth`), and
the two do not agree at the 6.0 mm rung — raw width admits only 16 characters
there where the shipped cap is 18. The discrepancy does not touch the 3.8 mm
result used here, and the numbers above carry ≥ 7 characters of headroom, but
**the implementation's gate must be the layout-based form, not this one.**

`PASSWORD REQUIRED` is the operator's decided wording (§2.4): 17 characters, and
it matches the words the device already shows on screen (`gui/gui.go:1997`,
`gui/passphrase_flow.go:645`), which matters more on an artifact read years
later than internal consistency with the source's vocabulary.

**Why the footer carries the fingerprint, and which one.** The `mk` research
settled that `mk1` carries exactly one per-key identifier —
`origin_fingerprint`, *"Master-key fingerprint … Verbatim from BIP 380 origin
notation"* — and that a passphrase **changes the master key**, so on a
passphrase-derived wallet that value is the **combined** fingerprint, not the
bare seed's. §2.1 measured the difference: `73c5da0a` bare vs `fc60c6df`
combined.

**That is the whole mechanism R4 exploits:** restoring the words alone yields a
fingerprint that does **not** match what the key and descriptor plates encode.
The footer's job is to make that mismatch legible instead of silent.

### 1.3 The marking must be CONDITIONED, not merely located

> **CORRECTED by R0 round 1, findings C3 and I1.** This section previously
> located the condition at the **`validateMdmk` call sites** and marked two of
> them "per §1.2". **Neither has the passphrase or the seed in scope:**
> `gui/gui.go:2344` sits in `mdmkFlow(ctx, th, s mdmkText)` — a bare string,
> reached from a scan, and §2.1's measurement is precisely that
> passphrase-derivation **is not decidable from the string**;
> `gui/bundle_flow.go:407` sits in
> `bundleEngrave(ctx, th, title string, cards []bundleCard)`, and `bundleCard`
> carries no seed or derivation record.
>
> Implemented literally, both conditions evaluate to "unknown", an implementer
> resolves that to false, and **R3/R4 never fire on any path — while every gate
> in §6 passes green.** That is the cycle's central deliverable shipping as a
> no-op, and it is the class this constellation has paid for before.
>
> **And `gui/bundle_flow.go:407` is one site serving FOUR flows** — measured:
> `gui/singlesig.go:177` (`"Engrave Single-Sig"`), `gui/multisig.go:291`
> (`"Engrave Multisig"`), `gui/multisig_build.go:402` (`"Build Policy"`),
> `gui/bundle_flow.go:39` (`"Engrave Bundle"`). Marking there marks **multisig
> plates**, crossing the R-B boundary this section exists to make structural.

**NORMATIVE — the condition lives ONE FRAME UP, where the facts are.**

`bundleEngrave` grows two optional strings, passed through to `validateMdmk`.
**`gui/singlesig.go:177` is the only caller that passes non-empty values**,
computed there from `passphrase != ""` and `full`, both in scope (`:97`, `:103`,
`:107`). Every other caller passes nothing and is byte-unchanged.

This keeps §1.1.2's optionality argument intact, keeps R-A and R-B structural
rather than promised, and still teaches `validateMdmk` nothing about flows.

| call path | marked this cycle? |
| --- | --- |
| **`gui/singlesig.go:177`** → `bundleEngrave` → `validateMdmk` | **yes, per §1.2** |
| `gui/multisig.go:291` → `bundleEngrave` | **no** — R-B |
| `gui/multisig_build.go:402` → `bundleEngrave` | **no** — R-B |
| `gui/bundle_flow.go:39` → `bundleEngrave` | **no** — scanned, no derivation |
| `gui/gui.go:2344` `mdmkFlow` | **no** — a bare string; not decidable |
| `gui/unlock_platelist.go:222` | **no** |
| `gui/derive_xpub.go:494` | **NO — F-205's flow, R-B** |

**GATE 1.3 (widened):** **only** the single-sig engrave marks. Assert that
`deriveXpubFlow`, `"Engrave Multisig"`, `"Build Policy"`, `"Engrave Bundle"` and
`mdmkFlow` all produce **unmarked** plates. *(The former gate named only
`deriveXpubFlow` and would have stayed green while every multisig plate got
marked.)*

---

## 2. THE PASSPHRASE PLATE (R2, R5, R6, R7)

### 2.1 R2 — the program runs preloaded

Per **R-C**. The device does **not** build a new offer flow. It runs the existing
dedicated passphrase-plate program with the passphrase already in hand, via
`engravePassphraseFlowFrom(ctx, th, body []byte, src syswSource)`
(`gui/passphrase_flow.go:617`), which already takes a body **and a provenance**.

**Re-typing is not the safer alternative**, and the spec records why so it is not
"corrected" later: the preloaded passphrase is the one the device **actually
derived with**, so the plate records the passphrase belonging to the wallet that
was engraved. Re-typing introduces a second chance to disagree, not a check.

**§7.4 does not bind.** §7.4 forbids the session cache answering a
**verification** prompt, because verify would then compare the engrave source
against itself. R-C preloads an **engrave** input. The shapes rhyme; the spec
states the distinction rather than leaving it implicit.

### 2.2 A new `syswSource` value is REQUIRED

Per **R-C.1**, forced by **R-D** + **R-C.3**.

`syswSourceAccept` (`gui/sysw_source.go:113`) **runs** for the preloaded
passphrase (R-C.3) and prints `"Source: " + syswSourceName(src)` (`:127`).
`syswSourceName` (`:9`) resolves `srcTyped` through its `default:` arm to
**`"the keyboard"`**. A preloaded passphrase did not come from the keyboard, so
reusing `srcTyped` makes that screen state a falsehood.

**NORMATIVE:**

1. A new `syswSource` value is added for "carried from this session's own
   derivation".
2. **It ships with an explicit `case` in `syswSourceName`.** The `default:` arm
   returns `"the keyboard"`, so a new value added without its case becomes a
   **printed falsehood** with no compile error and no failing test — the F-198
   class in miniature.
3. **GATE 2.2:** a test pinning the **rendered string**, not the enum value.
4. The `"Source: …"` line appears automatically: `syswFlags`
   (`gui/sysw_admit.go`) raises `flagSource` on `src != srcTyped`. Typing is the
   unremarkable baseline; a carried-over passphrase is not, and now says so.

### 2.3 The footer becomes conditional, and carries the policy id

Per **R-D** and **R-H**.

`passphraseFooter = "FINGERPRINTS TYPED, NOT VERIFIED"`
(`backup/passphrase.go:156`) asserts a provenance that did not occur on the
preloaded path, where the device **derived** the fingerprints.

> **CORRECTED by R0 round 1, finding C1 — the earlier normative text was an
> affirmative falsehood, authored here and caught by review.** It read
> `"POLICY … DERIVED, NOT TYPED"`, on the stated ground that the device
> *derived* the fingerprints on the preloaded path. **It does not.** R-C
> preloads the **passphrase bytes only**; both fingerprints are still typed by
> the operator through `fingerprintEntryFlow`
> (`gui/passphrase_flow.go:665-678`), and the code says so outright at
> `gui/passphrase_flow.go:628-629`: *"The payload PRE-FILLS the passphrase; the
> operator still walks the fingerprint fields and the confirm screen."*
>
> So `"FINGERPRINTS TYPED, NOT VERIFIED"` is **true on the preloaded path too**,
> and the replacement would have been strictly worse than the string it
> replaced — the F-198 class, on the cycle built to close it.

**NORMATIVE:**

```
preloaded    "POLICY <8 hex, grouped>  FPS TYPED"   27 chars -> fits (42 budget)
standalone   "FINGERPRINTS TYPED, NOT VERIFIED"     32 chars -> unchanged
```

The preloaded footer states the policy binding **and** the fingerprints'
true provenance. 27 characters leaves 15 of headroom; the denser
`"POLICY 1A2B 3C4D  FPS TYPED, NOT VERIFIED"` is 41 against 42 and is
**rejected for having one character of headroom** — §2.4's whole lesson is that
a budget with no slack is one edit from silent overrun.

**GATE 2.3b:** the footer must not claim derivation while
`fingerprintEntryFlow` is on the path. Assert the rendered string, not the
intent.

### 2.3a OPEN — should the device preload the fingerprints too?

**Not folded, because it grows R-C's scope and that is the operator's call.**

R-C's stated purpose is that the operator should not re-type what the device
already holds. The device *does* already hold the fingerprints: `masterFP`
comes back from `deriveSingleSigBundle` (`gui/singlesig.go:107`), and the
bare-seed value is one more call to the same derivation with an empty
passphrase.

**If preloaded** — and `ppStepSeedFP`/`ppStepCombinedFP` skipped on that path —
the fingerprints become **device-derived**, `"DERIVED, NOT TYPED"` becomes
**true**, and the plate carries *stronger* fingerprints than a typed one, with
two fewer keyboard steps for the operator.

**Until that is ruled on, the normative text above stands**, because it is the
only version that is true of the code as it exists.

This consumes **no new band line**. The measured budget is **42 characters wide
× 2 lines per band** (`SPIKE §3b`), and the worst case — both fingerprints plus
a passphrase containing a space — already fills **2 top and 2 bottom**. A third
line neither errors nor clips: it **silently cuts into the 3 mm outer margin**.

**GATE 2.3 — an assertion, not a comment.** `"FINGERPRINTS TYPED, NOT VERIFIED"`
(32) **plus** a policy id would be **50 against a 42-character band** — rendered
off both plate edges with no refusal. The design is safe only because the typed
footer and the policy id **never co-occur**, and nothing in `band`
(`backup/passphrase.go:228-235`) enforces it.

**GATE 2.3c — the footer is CONDITIONAL ON A FINGERPRINT, and R-H's deliverable
rides on it.** `backup/passphrase.go:185-187` appends the footer only when
`plate.SeedFP != "" || plate.CombinedFP != ""`. So an implementation that
"solves" a wording problem by leaving the fingerprint fields empty **silently
drops the policy id with it**, on the very path R-H was written for. Assert that
the policy id renders when it is supposed to — a test on presence, not only on
content. *(R0 C1, second half.)*

### 2.4 The identifiers

- **"wallet policy id"** = **`md.FormAwareStubChunks`** — top 4 bytes → 8 hex,
  grouped `XXXX XXXX`. The caller holds the md1 **chunk strings**, so the
  `Chunks` form is the one that applies.

  > **CORRECTED by R0 round 1, finding C2.** This section previously named
  > `md.WalletPolicyIDStub`, which is **not form-aware** — it is the *keyed*
  > branch, reached through `md.FormAwareStub` (`md/template_id.go:112-118`),
  > which routes to `WalletDescriptorTemplateIdStub` for a keyless template.
  > **Every** production stub site in the fork calls the form-aware form:
  > `gui/singlesig_derive.go:68`, `gui/multisig_derive.go:43`,
  > `gui/multisig_build.go:673`, `gui/template_engrave.go:29`,
  > `bundle/verify.go:118`.
  >
  > **The reachable case is `"Template-only md1"`** (`gui/singlesig.go:118-139`),
  > an offered choice on the very flow this cycle marks. The mk1 that run cuts
  > carries a stub derived from `WalletDescriptorTemplateId`; the spec as
  > written would have engraved the top 4 bytes of `WalletPolicyId` — **a
  > different four bytes**, binding the passphrase plate to an identifier
  > appearing on none of the plates beside it.
- **"key-id"** = the **master fingerprint**, already on the plate as
  `SeedFP`/`CombinedFP` (`backup/passphrase.go:176-180`). `mk1` defines no other
  per-key identifier; `mk.Header.ChunkSetID` is a chunk-**reassembly** id and is
  **not** an identity.

**`backup` takes no dependency on `md`.** The plate's fields are pre-formatted
hex strings (`backup/passphrase.go:23-29`); the caller computes the stub and
passes hex, exactly as it already does for fingerprints. The standalone path has
no descriptor, passes `""`, and renders no line.

**Two constraints from the `mk` research:**

1. **`origin_fingerprint` is OPTIONAL in the format — but THIS DEVICE CANNOT
   EMIT ONE WITHOUT IT.** *(R0 round 1, M2, verified.)* The header flag is set
   iff `card.Fingerprint != ""` (`mk/encode.go:70-80`), and every device-side
   `mk.Card` sets it: `gui/singlesig_derive.go:77`, `gui/multisig_derive.go:51`,
   `gui/derive_xpub.go:369`, all `Fingerprint: fmt.Sprintf("%08x", masterFP)`.
   A fingerprint-less `mk1` can only arrive **from another tool**, over scanned
   paths that §1.3 marks **not at all**.

   **So the case is out of scope for S6b, and the former GATE 2.4a is
   withdrawn** — it gated behaviour this spec never specified, for a state the
   device cannot produce, and would have been recorded as covered without ever
   running. **GATE 2.4a (replacement):** assert the three construction sites
   above set the fingerprint, so the assumption is pinned rather than believed.
2. **The policy stub is FORM-AWARE** — `WalletPolicyId` for a keyed
   wallet-policy `md1`, but the key-stable `WalletDescriptorTemplateId` for a
   **keyless template** `md1`. A label reading "wallet policy id" is **false**
   on the template form. **GATE 2.4b:** choose a label true of both, or
   distinguish them.

### 2.5 R7 — the QR stays password-only

Already true: `qr.Encode(plate.Passphrase, qr.L)` (`backup/passphrase.go:86`).
**GATE 2.5:** pin it with a test, so no future edit folds metadata into the QR.

---

## 3. THE VERIFY TAIL (F-199, F-204, F-206)

### 3.1 F-199 — `verifyRefused` re-offers at ONE site only

`verifyRefused` has **four** return sites, and three must never loop:

| line | trigger | correctable |
| --- | --- | --- |
| 717 | `len(expectedSlots) == 0` | no — programmer error |
| 727 | `len(engravedMd1) == 0` | no — programmer error |
| **753** | `extractReadbackMd1AndMk1s` fails on gathered cards | **YES** |
| 854 | `verifyFreshSlots` → `ferr != nil` | no — `verifyFreshSlots` (`gui/multisig_verify.go:324-336`) has exactly one error return, `errVerifyNoExpectedSlots` on `len(expected) == 0`, and `expected` does not change inside it, so this is a defensive re-check of 717 |

**NORMATIVE:** only `:753` re-offers. **Widening the verdict is prohibited** —
it would make all four loop, including two programmer-error refusals.

**GATE 3.1 — and one arm cannot be tested behaviourally.** *(R0 round 1, M1.)*
`:717` and `:727` are reachable by calling `multisigVerifyFlow` with an empty
slice, so they get **behavioural non-loop assertions**. **`:854` is unreachable
in-process** — `expectedSlots` is a parameter never reassigned in the function,
and `:715-717` returns before `:851` can run — so it gets a **source
assertion** that it returns `verifyRefused`, the idiom
`gui/singlesig_truth_test.go` already uses. **The gate names which arm is which,
so "never executed" is visible rather than assumed.** A test that only proves
`:753` loops would pass against the forbidden fix.

### 3.2 F-204 — the failed verify stops blaming the plates

`gui/singlesig_verify.go:182` tells a failed verify *"Check the engraved
plates."* The multisig sibling rules the other way
(`multisigVerifyNoSlotBody`, `gui/multisig_verify.go:157`): *"Check the
passphrase before you doubt the plates."*

The asymmetry costs steel: verify requires the seed **re-typed** (§7.4), and one
wrong passphrase character derives an entirely different wallet — so a mistyped
passphrase is a common cause of a FAILED comparison on **correct** plates.

**NORMATIVE — the copy is CONDITIONAL, not a string swap.** The multisig wording
is only true when a passphrase was actually entered. `singleSigVerifyFlow` has
`passphrase` in scope at the failure site (`gui/singlesig_verify.go:108-112`),
so:

- **passphrase entered** → suspect the passphrase before the plates;
- **no passphrase** → the multisig wording would be a **false lead**; the copy
  must say something true of that case.

**GATE 3.2:** both arms tested. A single-arm test would ship a false lead to
every operator who used no passphrase.

**A THIRD ARM IS AVAILABLE, and the choice is recorded rather than inherited.**
*(R0 round 1, M4.)* The multisig sibling has **three** arms
(`gui/multisig_verify.go:157-170`), and the one this spec drops is the
strongest: *"That seed IS a cosigner of this policy, but not with the passphrase
you typed … **Your plates are fine.** Try again and skip the passphrase."*

On the single-sig path it is cheap: the flow already re-derives from a re-typed
seed (`deriveSingleSigBundle`, `gui/singlesig_verify.go:115`), so **one more
derivation with an empty passphrase**, compared against the same readback, tells
the device outright whether the plates are good — turning "suspect the
passphrase" from advice into a verdict.

Nothing false ships without it; the two-arm copy is true, only weaker. **It is
therefore an operator/spec choice, not a defect** — recorded here so it is
decided rather than lost.

### 3.3 F-206 — and its FILED REMEDY IS UNSOUND

**The defect is real.** `verifyStatusMS1Clause` (`gui/verify_status.go:155`) is
the fixed string *"The ms1 secret you typed matched this seed."*, appended
whenever `p.full` (`buildVerifyPassLine`, `gui/verify_status.go:211-231`) —
**unconditionally on `p.legs`**, so a two-seed multisig verify still says "the
ms1 secret" and "this seed", singular.

**The remedy F-206 prescribes does not work.** It proposes *"a plural rule over
`passRecord.legs` / seed count … no new field, so it does not reopen NG1."*
**`legs` is not a seed count.** It is `len(legs)` (`gui/multisig_verify.go:1059`)
— one leg per **filled slot** — and the code states plainly that *"one seed
fills several slots of a policy that puts it at several accounts"*
(`gui/multisig_verify.go:299-300`).

So on **one seed at two accounts**, pluralising on `legs` would print *"the ms1
secrets you typed"* when the operator typed **one**. That is a **new false
statement in the over-claiming direction** — precisely what **R-D** forbids and
what this design exists to prevent. `passRecord` carries no seed count
(`gui/multisig_verify.go:1057-1061`), so a true plural **would** need a new
field, contradicting F-206's own constraint.

**NORMATIVE — fix it count-free.** The device's own screen already solves this:
`multisigVerifyOKMessage` (`gui/multisig_verify.go:1134-1135`) says *"the ms1
you typed for each seed"* — true for any n ≥ 1, requiring no count. The document
clause adopts that shape.

**No new recorded fact, so NG1 stays closed** — via a different route than
F-206 prescribed. **GATE 3.3:** assert the clause is true at 1 seed / 1 leg,
1 seed / 2 legs, and 2 seeds / 2 legs. **The middle case is the one that kills
the filed remedy**, and a test suite without it would pass the unsound fix.

---

## 4. MODAL FIT (F-192)

The F-185 drawn-frame check exists and is **a one-line call**; the sweep never
happened. Only S5.C's screens are measured.

**NORMATIVE:** every long modal body is gated by the class check.

**Do not re-derive the mechanism.** It is **not** a character budget: capacity
depends on how words **wrap** (588 normalised chars of short-word filler fit,
while F-185's real refusal was cut at ~500), so the check compares the **drawn
frame** to the source string and binary-searches the cut point.

**R-I decoupled this from F-208** — the arrows cost no body width, so this sweep
does not wait on them and its measurements stay valid.

**GATE 4:** the sweep's own coverage is stated — which modals were gated, and
which were judged not "long" and why. A sweep that silently skips is the defect
it is fixing.

---

## 5. SCROLL ARROWS (F-208)

Per **R-I**. Arrows draw at the **top-centre and bottom-centre of the body**,
over the 16 px fade zone, each with a **background chip** and an **enlarged
invisible touch target**.

Measured geometry: panel 480×320; nav slots 53×53 at y = 44/133/223, x =
427–480; body clip (6,44)–(423,314) = **417 wide**; `assets.ArrowDown`/`ArrowUp`
= **15×9**.

**NORMATIVE:**

1. **Not through `layoutNavigation`.** It computes
   `idx := int(clk.Button - Button1)` into a `[3]int`; `Up`/`Down` sort *before*
   `Button1` and index **negative**. This binds any layout.
2. **Body width is unchanged at 417.** If an implementation changes it, R-I's
   decoupling of F-192 is void and §4's measurements must be retaken.
3. **The chip is not optional** — the arrows sit where body text currently
   draws, because `fadeClip` clips nothing (R-E). **And because it is opaque,
   §5 must state the chip's drawn bounds, and the body's first and last drawn
   text rows must clear them.** The body already starts at
   `bodyClip.Min.Y + scrollFadeDist` (`gui/gui.go:416`) — *inside* the 16 px
   zone the arrows occupy — so this is a real constraint, not a formality.

   > **R0 round 1, I4 — §4's sweep is structurally blind to this.** The F-185
   > class check compares the drawn frame's **op tree** to the source string
   > (`bodyDrawnFully`, `gui/modal_fits_test.go:81-100`), and that file states
   > the seam itself at `:22-27`: *"ExtractText walks the op tree, so a body the
   > panel renders as nothing still 'appears' to uiContains."* **A glyph drawn
   > *under* an opaque chip is still in the op tree, and §4 returns green.**
   > That is F-185's own defect — a line of a safety modal the operator cannot
   > read and is not told exists — re-introduced beneath a gate certifying the
   > opposite. R-I's decoupling argument does not reach it: that argument is
   > about **wrap**, and occlusion is a different axis.

   **GATE 5.3:** one pixel-level assertion, on one representative long modal
   with an arrow showing — the top and bottom drawn rows are not overlapped by a
   chip. One screen proves the geometry; nothing more general is needed.
4. Scrolling needs no new machinery: a `Clickable` bound to `Up`/`Down` gets
   pointer routing (`gui/widget.go:70`) and press-and-hold auto-repeat
   (`gui/widget.go:48`).

### 5.1 The visibility predicate — the hard part

Arrows render **iff** content is actually below the fold. **While `fadeClip`
stays stubbed (R-E), that is NOT `maxScroll > 0`.**

`maxScroll = bodysz.Y - (bodyClip.Dy() - 2*scrollFadeDist)` (`gui/gui.go:409`)
reserves **32 px** of fade margin that is never drawn as fade, and the body is
not clipped to `bodyClip` at all — F-95 measured it drawing to y=317 against
`bodyClip.Max.Y = 314` in a 320 px panel. **Content can satisfy `maxScroll > 0`
while being entirely visible**, and an arrow keyed to it would appear with
nothing below the fold — a false statement by the UI in the other direction,
which R-D forbids just as firmly.

**NORMATIVE — the predicate is an EXPRESSION, stated here.** *(R0 round 1, I5:
this section previously required a predicate "defined against actual
visibility" and then never defined it, while §7 did not list it as unsettled —
so an implementer would read it as closed and reach for the one expression the
spec hands them, `maxScroll > 0`, which this section says is false.)*

```
show arrows  iff  bodysz.Y > bodyClip.Dy()
```

That drops the `2*scrollFadeDist` reservation — **32 px that is never drawn as
fade** — which is what makes `maxScroll > 0` over-report. **Residual, named
rather than hidden:** F-95 measured the body drawing to y=317 against
`bodyClip.Max.Y = 314` in a 320 px panel, so a few pixels of overdraw remain
outside this predicate's model while `fadeClip` is stubbed.

The implementation carries a comment **naming R-E**, because this expression
changes when the mask is restored. An un-named safety argument here becomes
exactly the stale comment this project has been bitten by.

**GATE 5.1 — the new predicate agrees with actual visibility. MUST BE GREEN.**

**GATE 5.1b — R-E's divergence probe, and it is EXPECTED TO FAIL.** `maxScroll`
vs actual visibility: each disagreement is a **finding to file**, not a gate to
weaken. *(These were one gate in two places with opposite expected outcomes — a
red result on the probe would have read as "the gate failed, weaken it", which
is how a false-PASS gate is born.)*

---

## 6. GATES, COLLECTED

Revised by R0 round 1. Changes marked **▲**.

| # | gate |
| --- | --- |
| 1.1 | with an empty title, `backup/testdata/text-{0,1,2}-shards-1.bin` **do not move** (R-G) |
| 1.2 | the offered **variant set** is pinned for a representative `md1` and `mk1`, title empty and set |
| **1.2a** ▲ | the title/footer budget assertion, in the **layout-based** form, for every title and footer this cycle introduces |
| **1.3** ▲ | **only** the single-sig engrave marks: `deriveXpubFlow`, `"Engrave Multisig"`, `"Build Policy"`, `"Engrave Bundle"`, `mdmkFlow` **and `cardMS1` plates** are all unmarked |
| 2.2 | `syswSourceName`'s **rendered string** for the new source |
| 2.3 | the typed footer and a policy id never co-occur |
| **2.3b** ▲ | the footer does not claim derivation while `fingerprintEntryFlow` is on the path |
| **2.3c** ▲ | the policy id **renders** — it is gated on a non-empty fingerprint, so it can vanish silently |
| **2.4a** ▲ | the three `mk.Card` sites set `Fingerprint` *(replaces the withdrawn unreachable-case gate)* |
| **2.4b** ▲ | the engraved policy id **equals the `policy_id_stub` the mk1 this run cut carries**, on **both** forms — a value equality, not a label check |
| 2.5 | the passphrase QR contains the passphrase and nothing else |
| **3.1** ▲ | `:717`/`:727` behavioural non-loop; **`:854` by source assertion** — it is unreachable in-process, and the gate says which arm is which |
| 3.2 | both F-204 arms — passphrase entered and not |
| 3.3 | the ms1 clause at 1/1, **1 seed / 2 legs**, and 2/2 |
| 4 | the sweep states its own coverage |
| **5.1** ▲ | the **new** predicate agrees with actual visibility — **must be green** |
| **5.1b** ▲ | R-E's `maxScroll` divergence probe — **failures expected, files findings, does not gate** |
| **5.3** ▲ | one pixel-level assertion: an arrow's chip does not overlap the top/bottom drawn text rows |
| — | **`me` CLI untouched**; `mk1`/`md1`/`ms1` byte-identical (§0) |

**A bound that is now PROVEN rather than observed** *(R0 round 1, closed blind
spot).* `SPIKE §2` flagged the maximum `md1`/`mk1` payload as unmeasured. It is
a hard, code-enforced constant: `ValidMD` rejects a data part over
`mdRegularMaxLen = 93` → **md1 ≤ 96 chars**; `ValidMK` admits only `[14,93]` and
`[96,108]` → **mk1 ≤ 111 chars** (`codex32/mdmk.go:49,54-57,137-143,152-160`).
So the spike's *"longest in-repo 111"* is the **absolute maximum**, and §1.1's
240-character title+footer budget carries better than 2× margin against a proven
bound. The spike's caveat is retired.

**~~Measurement still owed~~ — RUN 2026-08-17, see §1.2a.** The `Text`
title/footer budget is **25 characters** at `plateFontSizeUR` = 3.8 mm. Every
string this cycle puts there fits with ≥ 7 characters of headroom, and the
merged two-fingerprint form was **eliminated by measurement** rather than by
review.

**GATE 1.2a is still owed at implementation, and it is not this measurement.**
§2.4 establishes that **nothing in the render layer bounds a title**, so an
over-long device-generated title is neither rejected nor truncated — it is
engraved, into a screw-hole band. The gate is therefore an assertion **on the
budget**, in the layout-based form `TestTitleCapFitsAtEveryRung` uses, **for
every title and footer this cycle introduces** — not on today's strings, which
would pass while the next edit walks off the plate.

**Golden policy (R-G):** marked states get **new** golden files; the frozen
sixteen keep meaning what they meant; `backup/testdata/passphrase-*.bin` (4)
will legitimately move under §2.3 and are re-recorded **in the same commit**;
**never a bare `go test ./... -update`** — it rewrites the frozen sixteen.

---

## 7. WHAT THIS SPEC DOES NOT SETTLE

- ~~The `Text` title/footer horizontal budget~~ — **measured, §1.2a: 25 chars.**
- ~~Whether `PASSWORD REQUIRED` and the footer fingerprint fit~~ — **measured,
  they do**; and the merged two-fingerprint form was eliminated because it does
  not.
- The exact wording of §3.2's no-passphrase arm.
- **§2.3a — should the device preload the FINGERPRINTS as well as the
  passphrase?** It would make `"DERIVED, NOT TYPED"` true, give the plate
  stronger fingerprints, and remove two keyboard steps — but it grows R-C's
  scope, so it is the operator's call. Until then §2.3's truth-preserving footer
  stands.
- **§3.2's third arm** — the *"Your plates are fine"* verdict. Cheap on this
  path and strictly better copy; a scope choice, not a defect.
- **`gui/singlesig_derive.go:28`'s stale doc comment** still names
  `md.WalletPolicyIDStubChunks` where the code forty lines below calls
  `md.FormAwareStubChunks`. Pre-existing in the fork, and **the most likely
  origin of C2** — it should be corrected in the same commit that lands C2's
  fix, or it will mislead the implementer again. *(R0 round 1, M3.)*
- **The raw-width vs layout-based discrepancy at the 6.0 mm rung** (§1.2a's
  method caveat). It does not affect this cycle's 3.8 mm plates, and it is
  **not** newly introduced here — but a reviewer should know the two methods
  disagree before relying on either.
