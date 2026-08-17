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
| **Title** | `PASSWORD REQUIRED` (17 chars) | iff the set was derived **with** a BIP-39 passphrase |
| **Footer** | the fingerprint(s) the plates actually encode | iff the set contains a seed (R-A) |

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

`validateMdmk` is a **four**-call-site chokepoint:

| call site | flow | marked this cycle? |
| --- | --- | --- |
| `gui/gui.go:2344` | md1/mk1 engrave | per §1.2 |
| `gui/bundle_flow.go:407` | bundle engrave | per §1.2 |
| `gui/unlock_platelist.go:222` | unlock engrave | **no** |
| `gui/derive_xpub.go:494` | `deriveXpubFlow` | **NO — this is F-205's flow** |

**Marking placed unconditionally inside `validateMdmk` would close part of F-205
as a side effect, crossing the R-B phase boundary without anyone deciding to.**
Per §1.1.3 the condition lives at the call site, which makes the boundary
structural rather than a promise.

**GATE 1.3:** a test asserting `deriveXpubFlow`'s plates are **unmarked**.

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

**NORMATIVE:**

```
preloaded    "POLICY <8 hex, grouped>  DERIVED, NOT TYPED"   36 chars -> fits
standalone   "FINGERPRINTS TYPED, NOT VERIFIED"              32 chars -> unchanged
```

This consumes **no new band line**. The measured budget is **42 characters wide
× 2 lines per band** (`SPIKE §3b`), and the worst case — both fingerprints plus
a passphrase containing a space — already fills **2 top and 2 bottom**. A third
line neither errors nor clips: it **silently cuts into the 3 mm outer margin**.

**GATE 2.3 — an assertion, not a comment.** `"FINGERPRINTS TYPED, NOT VERIFIED"`
(32) **plus** a policy id would be **50 against a 42-character band** — rendered
off both plate edges with no refusal. The design is safe only because the typed
footer and the policy id **never co-occur**, and nothing in `band`
(`backup/passphrase.go:228-235`) enforces it.

### 2.4 The identifiers

- **"wallet policy id"** = `md.WalletPolicyIDStub` — top 4 bytes → 8 hex,
  grouped `XXXX XXXX`.
- **"key-id"** = the **master fingerprint**, already on the plate as
  `SeedFP`/`CombinedFP` (`backup/passphrase.go:176-180`). `mk1` defines no other
  per-key identifier; `mk.Header.ChunkSetID` is a chunk-**reassembly** id and is
  **not** an identity.

**`backup` takes no dependency on `md`.** The plate's fields are pre-formatted
hex strings (`backup/passphrase.go:23-29`); the caller computes the stub and
passes hex, exactly as it already does for fingerprints. The standalone path has
no descriptor, passes `""`, and renders no line.

**Two constraints from the `mk` research:**

1. **`origin_fingerprint` is OPTIONAL** — omitted in the privacy-preserving mode
   (header bit 2 = 0). **An `mk1` may carry no fingerprint at all.** Under R-D
   the plate may not then assert a binding that does not exist. **GATE 2.4a:**
   what the marking does in that case must be specified and tested.
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

**GATE 3.1:** a test asserting the other three do **not** loop. A test that only
proves `:753` loops would pass against the forbidden fix.

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
   draws, because `fadeClip` clips nothing (R-E).
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

**NORMATIVE:** the predicate is defined against **actual visibility**, and the
implementation carries a comment **naming R-E**, because the predicate changes
when the mask is restored. An un-named safety argument here becomes exactly the
stale comment this project has been bitten by.

**GATE 5.1:** a test that the two agree — `maxScroll > 0` on a screen where
nothing is actually hidden is a **finding**, not a rounding error.

---

## 6. GATES, COLLECTED

| # | gate |
| --- | --- |
| 1.1 | with an empty title, `backup/testdata/text-{0,1,2}-shards-1.bin` **do not move** (R-G) |
| 1.2 | the offered **variant set** is pinned for a representative `md1` and `mk1`, title empty and set |
| 1.3 | `deriveXpubFlow`'s plates are unmarked |
| 2.2 | `syswSourceName`'s **rendered string** for the new source |
| 2.3 | the typed footer and a policy id never co-occur |
| 2.4a | behaviour when `mk1` carries no `origin_fingerprint` |
| 2.4b | the policy-id label is true of the template form too |
| 2.5 | the passphrase QR contains the passphrase and nothing else |
| 3.1 | the three non-correctable `verifyRefused` sites do **not** loop |
| 3.2 | both F-204 arms — passphrase entered and not |
| 3.3 | the ms1 clause at 1/1, **1 seed / 2 legs**, and 2/2 |
| 4 | the sweep states its own coverage |
| 5.1 | the arrow predicate agrees with actual visibility |
| — | **`me` CLI untouched**; `mk1`/`md1`/`ms1` byte-identical (§0) |

**Measurement still owed before this spec may close:** the **`Text` title/footer
horizontal budget** on an `md1`/`mk1` plate. `SPIKE §3b` measured **42 chars**
for the *passphrase band* — a different face and inset. §2.4 establishes that
**nothing in the render layer bounds a title**, so an over-long device-generated
title is neither rejected nor truncated: it is engraved, into a screw-hole band.

**Golden policy (R-G):** marked states get **new** golden files; the frozen
sixteen keep meaning what they meant; `backup/testdata/passphrase-*.bin` (4)
will legitimately move under §2.3 and are re-recorded **in the same commit**;
**never a bare `go test ./... -update`** — it rewrites the frozen sixteen.

---

## 7. WHAT THIS SPEC DOES NOT SETTLE

- The `Text` title/footer horizontal budget (above) — **a gate that has never
  been run, so this spec may not close on it**.
- Whether `PASSWORD REQUIRED` and the footer fingerprints **fit horizontally**
  once that budget is known.
- The exact wording of §3.2's no-passphrase arm and §2.4b's label.
