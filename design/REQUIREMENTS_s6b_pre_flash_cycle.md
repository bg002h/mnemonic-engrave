# S6b — REQUIREMENTS CAPTURE: the single pre-flash cycle (compressed)

**Status:** requirements capture only. **Not a spec, not a plan, not gated.** No
code may be written against this. Its job is to make the operator's directives
outlive the conversation they were given in, and to record what has already been
measured so a later spec does not re-derive it.

**Owning phase: S6b — the SINGLE pre-flash cycle.** Operator directive
2026-08-16: **"compress"**. There were three software cycles queued between here
and the hardware flash; there are now two.

| was | now |
| --- | --- |
| S6a — single-sig truth (mid-review, round 3) | **unchanged — closes as-is** |
| S6b — F-199 + F-204 (verify tail) | **merged** |
| S6c — passphrase plate + plate marking | **merged** |
| → flash | → flash |

**S6a is deliberately NOT compressed into, and that is the whole reason the rest
can be.** It is at review round 3 with a funds Critical folded four times;
adding scope now would restart its gate and is the surest way for it never to
close. It ships the F-198 Critical on its own.

**Everything else becomes one cycle: one spec, one R0, one implementer, one
whole-diff review.** The merged scope is F-199, F-204 and R1–R7 below.

**Why merging these is sound rather than merely faster.** They are one surface.
F-199 and F-204 are both verify-tail copy/control decisions, and S6a has already
restructured that tail — so the code is open. The plate work is the only part
that changes engraved output, and it *wants* the hardware flash immediately
after, which is exactly where it now sits.

**What merging costs, stated plainly:** F-199's own follow-up says it "needs a
decision, not a reflex", so this cycle **opens with a decision pass** before any
code — the same shape that produced the C-1 verify-tail ruling. A cycle that
begins by answering its open question is compressed; one that begins by coding
around it is just rushed.

**Why not compressed into the flash itself:** the plate marking changes what the
machine cuts, so it must be GREEN before the flash rather than validated by it.
The flash then proves it on real steel in one pass.

---

## 1. OPERATOR DIRECTIVES — verbatim, 2026-08-16

Recorded word for word, because a paraphrase of a decision is a decision lost.

1. > "Keep it somewhere separate" is the right approach. But we can offer to
   > engrave it and we should mark the associated keys and descriptor plates as
   > associated within a password…we need to verify the keys and descriptor do
   > include passphrase.

   — followed by the correction: **"associated WITH a password"**.

2. > Title is the perfect place for passphrase required notice

3. > SeedFP and combined FP are also perfect for title or footer

4. > Password, if user chooses to engrave it, should be on its own plate, with
   > associated key-id and wallet policy id in title or footer. QR code should
   > include only password

### What those resolve to

| # | requirement | status |
| --- | --- | --- |
| R1 | The restore document keeps saying to keep the passphrase somewhere separate | **already shipped** — `buildPassphraseInventoryLines` |
| R2 | The device **offers** to engrave the passphrase | new flow work |
| R3 | mk1 and md1 plates are marked as requiring a passphrase, in the **title** | new plate work |
| R4 | Seed FP and combined FP appear in **title or footer** | new plate work |
| R5 | An engraved passphrase is on **its own plate** | **already true** — `engravePassphraseFlow` cuts a dedicated plate |
| R6 | That plate carries the associated **key-id** and **wallet policy id** in title or footer | new plate work |
| R7 | Its **QR contains only the password** | **already true** — pin it with a test |

---

## 2. MEASURED FACTS — do not re-derive these in the spec

Every item read out of the fork at `main` = `b8a23bf`.

### 2.1 The keys and descriptor DO bind the passphrase — proven by bytes

The operator asked for this to be verified. It was, by deriving the same
mnemonic twice and comparing output, not by reading the call graph:

| artifact | bare seed | same seed + `"abandon about"` | |
| --- | --- | --- | --- |
| `ms1` | `ms10entrsqqq…cj9sxraq34v7f` | `ms10entrsqqq…cj9sxraq34v7f` | **identical** |
| master fp | `73c5da0a` | `fc60c6df` | **differs** |
| `mk1` | `mk1qph25epq…` | `mk1qpl36ypq…` | **differs** |
| `md1` | `md1fgdxlpqp…` | `md1faxr8pqp…` | **differs** |

So `ms1` encodes the **words only**; `mk1` and `md1` are passphrase-bound
through `deriveAccountXpub` (`gui/singlesig_derive.go:10`), and `ms1` is built
from mnemonic entropy alone (`:87`, `codex32.EncodeMS1(entropy)`).

**The consequence R4 exploits:** restoring the words alone yields a *different*
fingerprint from the one the key and descriptor plates encode. That mismatch is
already detectable on steel today — nothing merely says to look, or what it
means. R3+R4 turn a silent wrong-wallet restore into a self-diagnosing one.

### 2.2 No codec change is required, so the Rust-primary rule is NOT triggered

All marking is plate **layout and text**. The `mk1`/`md1`/`ms1` strings stay
byte-identical, so nothing here leads the primary Rust implementation. This must
be re-confirmed by the spec, since it is what keeps S6c a fork-native cycle.

### 2.3 There are FOUR different plate-text mechanisms, with different budgets

This is the spec's first design call, and it must precede any wording, because
the length budget differs per mechanism.

> **The "length rule" column of row 1 was CORRECTED 2026-08-17** — see §2.4. It
> read *"`MaxTitleLen = 18`, SILENT truncation via `TitleString`"*, and
> `TitleString` has no production callers; title and footer are engraved
> **verbatim**, with the 18 cap enforced only as a **UI-layer hard reject** on
> keyboard-entered text.

| mechanism | carries | length rule | used by |
| --- | --- | --- | --- |
| `Fitted.Title` / `.Footer` (`backup/fit.go:117-121`) | title at plate row 0, footer at the last row, at the screw-hole rows | **`MaxTitleLen = 18`, engraved VERBATIM.** Cap enforced only at the UI (`gui/freetext_flow.go:1174`, hard reject) on **keyboard-entered** text; the `backup` layer bounds nothing | free-text plate, preview |
| `Seed.Title` / `SeedString.Title` (`backup/backup.go:17,27`) | a title | rendered `strings.ToUpper` at `:223`, `:311` | codex32 / seed-share plates |
| the passphrase plate's own `topLines` / `bottomLines` banding | arbitrary bands | **no 18-char cap** — its footer is 32 chars | `backup/passphrase.go` |
| `Text.Paragraphs` | paragraphs only — **no title, no footer** | n/a | **`mk1` and `md1`**, via `validateMdmk` (`gui/gui.go`) |

**mk1 and md1 use the only mechanism with no title or footer at all.** R3 and R4
therefore require either giving `Text` a title/footer band or routing those
plates through a mechanism that has one. That choice is S6c's first gate.

### 2.4 The 18-character cap is real — but truncation is NOT how it bites

> **CORRECTED 2026-08-17** by `design/agent-reports/s6b-plate-mechanism-facts.md`,
> re-verified by the controller. **The original text of this section was wrong,
> and it was wrong in the direction that matters** — it described a hazard that
> cannot occur and missed the one that can. It is corrected in place; the
> superseded claims are named below so a reader who saw them can recognise them.
>
> **WRONG: "truncation is silent, via `TitleString`."** `TitleString` has **zero
> production callers anywhere in the repository** — 13 hits, every one a
> definition, a doc comment, or a test. The code says so outright:
> `backup/freetext.go:14` — *"f.Title and f.Footer are engraved VERBATIM —
> never through TitleString"*. Titles are **never truncated**, anywhere.
>
> **WRONG: `COMB FP: FC60 C6DF` (18, "fits EXACTLY AT THE CAP").** That string
> does not exist in the source. The emitted line is `"EXPECTED COMB FP: "` +
> grouped fingerprint = **27 characters** (`backup/passphrase.go:180`), and it
> sits in the passphrase plate's band, which has no cap at all.
>
> **RIGHT, and it is a different hazard.** The 18 limit is enforced in exactly
> one place — the **UI input layer**, as a **hard reject** that names the limit
> and the overrun (`gui/freetext_flow.go:1174`). That check guards
> **keyboard-entered free text only**. Nothing in the `backup` package bounds
> `Fitted.Title`/`Fitted.Footer` at all: `fitBlocksAt`, `FitSized` and
> `EngraveFitted` never compare a title's length to anything, so an over-long
> title reaching the render layer is **neither rejected nor truncated — it is
> engraved**, and its ink can run into a screw-hole band.
>
> **So for a DEVICE-GENERATED title — which is exactly what R3 and R4 are —
> there is no check at any layer.** The one existing precedent proves the point:
> `gui/slip39_polish.go:492` builds a title with `fmt.Sprintf` and justifies its
> length in a **comment** — `// max "32767 #16/16" = 12 <= MaxTitleLen 18` — a
> human-verified bound, not a checked one.
>
> **§2.4's conclusion therefore SURVIVES for a better reason:** the gate must be
> **a test asserting the budget**, because nothing else in the system will. What
> changes is the failure mode it protects against — not a silently shortened
> string, but ink in a screw hole.

Why 18 is the number: row 0 and the last row are **screw-hole rows**, and the
cap is what keeps their ink clear of the holes — *"measured at every rung by
TestTitleCapFitsAtEveryRung"* (`backup/freetext.go:14-19`).

**The superseded description of `TitleString` follows, for reference only. It is
dead code as far as production is concerned.** It stops at exactly
`MaxTitleLen = 18` and **also silently drops any rune the face cannot decode**.

    19  PASSPHRASE REQUIRED   -> TRUNCATES to 'PASSPHRASE REQUIRE'
    17  PASSPHRASE NEEDED        fits
    16  NEEDS PASSPHRASE         fits
    18  SEED FP: 73C5 DA0A       fits EXACTLY AT THE CAP
    18  COMB FP: FC60 C6DF       fits EXACTLY AT THE CAP
    27  EXPECTED COMB FP: FC60 C6DF   band only

**"PASSPHRASE REQUIRED" is 19 characters** and would engrave as
`PASSPHRASE REQUIRE`, permanently, with no error.

### DECIDED — the title text is `PASSWORD REQUIRED`

Operator directive 2026-08-16, after being shown the measurement. **17
characters, fits with 1 to spare**, and it is not a coinage invented for the
plate: the device **already says "BIP-39 Password" to the operator** at
`gui/gui.go:1997` and `gui/passphrase_flow.go:645`. The steel therefore matches
the screen the operator already knows, which is worth more on an artifact read
years later than internal consistency with the word the *source code* uses.

**The budget still needs a TEST, not a correct string.** `PASSWORD REQUIRED` has
one character of headroom and both fingerprint forms sit *exactly* on the cap.
Every one of them is a single edit away from silent truncation onto steel, and
`TitleString` reports nothing when it truncates. The gate is an assertion on the
budget — for every title this cycle introduces, not just today's wording.

**So the length budget is a first-class gate in S6c, not a wording detail** — a
test must assert the budget, not merely the current string.

Fingerprints are grouped by the house helper: `passphrase.GroupFingerprint`
splits at 4 (`73C5DA0A` → `73C5 DA0A`), so every count above is of the grouped
form.

### 2.5 The passphrase plate already does most of R5, R6, R7

`backup/passphrase.go`:

- **R5 — its own plate:** yes. `engravePassphraseFlow` (`gui/passphrase_flow.go:605`)
  is a separate top-level program cutting a dedicated plate.
- **R7 — QR is password-only:** **yes, already.** `:86` is
  `qr.Encode(plate.Passphrase, qr.L)` — the passphrase and nothing else. The
  work is to **pin this with a test**, so no future edit folds metadata in.
- **R6 — identifiers in title/footer:** partially. It carries `SeedFP` and
  `CombinedFP` as `topLines` (`:176-180`) plus a footer (`:156`). It does **not**
  carry a key-id or a wallet-policy id.

**One sentence on that plate cannot be reused, and the asymmetry is the point.**
Its footer reads `FINGERPRINTS TYPED, NOT VERIFIED` — true there, because the
operator typed them. On `mk1`/`md1` the device **derived** those fingerprints, so
those plates may legitimately vouch for their own. Copying the string across
would understate what the device knows.

### 2.6 Which identifier fits, for R6

- `md.WalletPolicyId` (`md/walletpolicyid.go:30`) is **16 bytes → 32 hex** —
  far past any title budget.
- `md.WalletPolicyIDStub` (`:106`) is **4 bytes → 8 hex**, groups as
  `XXXX XXXX` exactly like a fingerprint, and is **already the binding `mk1`
  carries** (T6a spec: the bundle's mk1 stub = `WalletPolicyIDStub(md1)`,
  non-zero and policy-bound).

The stub is the candidate that fits. `mk.Header.ChunkSetID` (`mk/mk.go:50`, a
20-bit chunk-set id) is the other identifier in play and the spec must decide
which of the two is the "key-id" the operator means.

---

## 2bis. OPERATOR RULINGS — 2026-08-17, verbatim

Given in the decision pass, after §3's questions were put. Recorded word for word;
a paraphrase of a decision is a decision lost.

### R-A — watch-only sets are NOT marked (answers §3 Q4)

> "no"

**So the marking predicate is "the set contains a seed."** That is a clean,
testable condition rather than a per-flow list.

**Stated consequence, so it is chosen rather than discovered:** a watch-only set
that *was* passphrase-derived carries no passphrase mark. That is tolerable
because a watch-only set **already disclaims the thing that would make the
omission dangerous** — F-195 shipped `"Seed: this set contains NO seed. It is
watch-only"` (`gui/multisig_build_census.go:208`), so the set never claims to
restore a wallet in the first place. F-198's defect was an artifact *vouching*
for restorability it did not have; a watch-only set makes no such claim.

### R-B — the multisig marking moves to a new phase (answers §3 Q5)

> "let's make further refinement of verifying user has keys and passwords for
> any or all keys a separate polish phase"

Phase created: **`key & password custody refinement`** (see `FOLLOWUPS.md`,
"Phases used in this file"). It takes §3 Q5 **and F-205**. It does **not** gate
the hardware flash.

**S6b's marking is therefore SINGLE-SIG ONLY**, and that constrains a design
call §3 Q1 does not mention: `validateMdmk` has four call sites, one of which is
`gui/derive_xpub.go:494` — F-205's flow, now owned by the new phase. Marking
placed unconditionally at that chokepoint would cross the phase boundary by
accident. **The marking must be conditioned, not merely located.**

### R-C — R2 is the existing passphrase program, preloaded

> "passphrase program gets run with passphrase preloaded"

So R2 is **not** a new offer flow. The device runs the existing dedicated
passphrase-plate program with the passphrase already in hand, rather than asking
the operator to re-type it. This is a reuse ruling, and the machinery is already
shaped for it: `engravePassphraseFlowFrom(ctx, th, body []byte, src syswSource)`
(`gui/passphrase_flow.go:617`) already takes a body **and a provenance**.

**Why re-typing is not the safer option here, stated because the instinct says
otherwise:** the preloaded passphrase is the one the device *actually derived
with*, so the plate records the passphrase belonging to the wallet that was
engraved. If the operator mistyped at entry, they derived a different wallet —
and the plate correctly records the passphrase for *that* wallet. Re-typing
would introduce a second chance to disagree, not a check.

**Three questions R-C opens, which the spec must answer:**

1. **Which `syswSource`?** The enum is `srcTyped` / `srcNFC` / `srcPayload`
   (`gui/sysw_admit.go:55-58`). None means "carried from this session's own
   derivation". Reuse `srcTyped`, or add a value?
2. **Does `passphraseFooter` still tell the truth?** It is the fixed string
   `"FINGERPRINTS TYPED, NOT VERIFIED"` (`backup/passphrase.go:156`). In the
   preloaded case the device **derived** those fingerprints itself, so the
   footer asserts a provenance that did not happen. §2.5 already flags this
   asymmetry in the opposite direction. It under-claims rather than
   over-claims — the F-206 direction, so likely not gating — but it is an
   affirmative false statement about provenance, not a mere omission.
3. **Does the acceptance screen run?** `syswSourceAccept`
   (`gui/sysw_source.go:113`) is the screen shown when a record *enters* a flow,
   and it names the source. Preloading enters a record without the operator
   presenting it.

**§7.4 adjacency, to be stated and dismissed in the spec rather than left
hanging:** §7.4 forbids the session cache answering a *verification* prompt,
because verify would then compare the engrave source against itself. R-C
preloads an *engrave* input, not a verification input, so §7.4 does not bind —
but the spec must say so explicitly, because the shapes rhyme.

### R-D — the governing principle, and it decides two of R-C's three questions

Operator, 2026-08-17, answering R-C question 2:

> "all things said must be true"

**This is a general rule, not a footer fix**, and it is recorded at this level
because it settles cases nobody has enumerated yet. Applied to R-C:

**R-C.3 — the acceptance screen RUNS.** Operator: *"yes."*
`syswSourceAccept` (`gui/sysw_source.go:113`) is shown when the preloaded
passphrase enters the program.

**R-C.1 — therefore a NEW `syswSource` value is required.** Not a free choice:
it is forced by R-D plus R-C.3. `syswSourceName` (`gui/sysw_source.go:9`)
resolves `srcTyped` through its `default:` arm to **`"the keyboard"`**, and
`syswSourceAccept` prints `"Source: " + syswSourceName(src)`
(`gui/sysw_source.go:127`). A preloaded passphrase did **not** come from the
keyboard, so reusing `srcTyped` makes the screen state a falsehood — exactly
what R-D forbids.

Two mechanical consequences the spec must carry:

- **The `default:` arm is a trap of the F-198 class.** A new enum value added
  without an explicit `case` in `syswSourceName` falls through to
  **`"the keyboard"`** — a missed edit becomes a *printed falsehood*, with no
  compile error and no test failure. The new case and the new value must land
  together, and a test must pin the rendered string, not the enum.
- **The source line appears automatically, which is the desired behaviour.**
  `syswFlags` (`gui/sysw_admit.go`) raises `flagSource` on `src != srcTyped`,
  so any non-typed provenance is announced. Typing is the unremarkable
  baseline; a carried-over passphrase is not, and now says so.

**R-C.2 — `passphraseFooter` must become conditional.** The fixed string
`"FINGERPRINTS TYPED, NOT VERIFIED"` (`backup/passphrase.go:156`) asserts a
provenance that did not occur on the preloaded path, where the device **derived**
those fingerprints itself. Under R-D that is not tolerable merely because it
under-claims.

**Measured, so the spec does not have to re-derive it:**

```
32  FINGERPRINTS TYPED, NOT VERIFIED     <- the existing string
35  FINGERPRINTS DERIVED BY THIS DEVICE
31  FINGERPRINTS DERIVED, NOT TYPED
22  DERIVED BY THIS DEVICE
```

**The budget is NOT known to be 32.** §2.3 records only that this mechanism has
no 18-char cap and that *this* footer happens to be 32 characters — that is the
string's length, not a measured limit. **The real limit must be measured before
a replacement is chosen**, and it belongs in the same spike as §3 Q2.

This also resolves the asymmetry §2.5 flagged: the passphrase plate's footer was
correct *because the operator typed the fingerprints there*. On the preloaded
path the device derived them, which puts that plate in the same category as
`mk1`/`md1` — plates that may legitimately vouch for their own fingerprints.

### R-E — `fadeClip`'s clip mask is NOT restored in S6b

Operator, 2026-08-17, accepting the controller's recommendation.

**What stays broken on purpose.** `fadeClip` (`gui/gui.go:763`) remains the
no-op stub `return o.Offset(image.Pt(0, 0))`, with the real alpha mask
(`gui/gui.go:768-777`) left commented out.

**Why.** Restoring it would *start* enforcing a clip that nothing enforces
today, silently deleting text that currently draws — F-95 measured that exact
case, a 19-pixel window holding *"the encrypted part has been REMOVED. Do not
continue."* Doing that to safety screens immediately before an irreversible
flash trades a latent problem for a live one, and it buys nothing the arrows do
not buy better: an arrow is a clearer cue than a gradient.

### THE CONSEQUENCE R-E FORCES ON F-208, stated because it complicates the same recommendation

**With the mask stubbed, `maxScroll > 0` is NOT a sound visibility predicate,
so the arrow cannot simply be wired to it.**

`maxScroll = bodysz.Y - (bodyClip.Dy() - 2*scrollFadeDist)` (`gui/gui.go:409`)
reserves **32 px** of fade margin that is not being rendered as fade, and the
body is not clipped to `bodyClip` at all — F-95 measured it drawing to y=317
against `bodyClip.Max.Y = 314`, inside a 320-px panel. So content can satisfy
`maxScroll > 0` while being **entirely visible**, and an arrow keyed to it would
appear with nothing below the fold. Under **R-D** that is a false statement by
the UI, in the other direction.

**So the spec must define the arrow's predicate against what is ACTUALLY
VISIBLE** — the panel, not `bodyClip` — for as long as the mask stays stubbed,
and must state the predicate it chose. Two consequences:

1. The predicate is **coupled to R-E** and changes when the mask is restored.
   Whatever S6b writes must be revisited by the honest-geometry work, and should
   say so in a comment naming R-E, or it becomes a stale safety argument of the
   kind this project has been bitten by.
2. **S6b owes a test that the two agree**: if `maxScroll > 0` on a screen where
   nothing is actually hidden, that is a **finding**, not a rounding error. This
   is the cheapest possible guard on the divergence R-E chooses to leave in
   place.

**Restoring the mask is filed with the honest-geometry work, after F-192 closes
— not in S6b.**

### R-F — Q1: optional `Title`/`Footer` on `backup.Text`

Operator, 2026-08-17: *"I accept your recommendations."* Full argument and
citations in `design/PROPOSAL_s6b_q1_q3_q6.md`.

`mk1`/`md1` keep their current mechanism (`backup.Text`), which gains
**optional** `Title`/`Footer` rendered through the layout helpers it **already
shares** with the free-text plate (`textLayout`, `qrPlaceAt`, `WrapText`, all in
`backup/wrap.go`). Rejected: the passphrase band (a private closure with no
wrap, no QR-narrowing, no screw-hole clamp — and these plates carry a QR), and
migrating to `Fitted` (one row per string vs wrapped paragraphs).

**OPTIONAL IS NORMATIVE, not an implementation detail.** Empty title → no row →
no vertical budget consumed → **byte-identical to today**. This is the mechanism
by which R-A and R-B are honoured without teaching `validateMdmk` about flows:
the **caller** conditions the marking by supplying a string or not.

**Required gate (from the proposal, adopted with it):** a test pinning **which
engraving variants are offered** for a representative `md1` and `mk1`, with the
title empty and with it set. TEXT+QR is the tightest variant — fits to 268
characters, fails at 269 — so a band could silently change the offered set,
which is what `backup/backup.go:386-392` warns about in its own words.

### R-G — Q6: no golden churn is the ASSERTION, not the review burden

Operator, same ruling.

1. Because R-F's fields are optional, the unmarked path **must** stay
   byte-identical. So if `backup/testdata/text-{0,1,2}-shards-1.bin` move, that
   is a **finding — a defect in the optionality** — not a golden to refresh.
2. **Marked states get NEW golden files.** The frozen sixteen keep meaning what
   they meant.
3. `backup/testdata/passphrase-*.bin` (4) **will** legitimately move if Q3's
   policy-id line lands. Re-record them **in the same commit as the change that
   moved them**, which is the contract `sizeproof-{front,back}.bin` already
   documents.
4. **Never run a bare `go test ./... -update`** — it rewrites the frozen
   sixteen. Scope every regeneration with `-run`.

### R-H — the wallet-policy id rides IN the footer line, preloaded path only

Operator, 2026-08-17, choosing from the options the Q2 spike forced open. Full
measurements: `design/SPIKE_s6b_q2_results.md`.

**Why a ruling was needed at all:** the Q3 proposal assumed a band line was
available for the policy id. Execution showed there is none — the worst case
(both fingerprints + a passphrase containing a space) already fills **2 top and
2 bottom** lines, and a third line does not error or clip, it **silently cuts
into the 3 mm outer margin**.

**The ruling:**

```
preloaded path   "POLICY 1A2B 3C4D  DERIVED, NOT TYPED"    36 chars  -> fits (42 budget)
standalone path  "FINGERPRINTS TYPED, NOT VERIFIED"        32 chars  -> unchanged
```

**It consumes no new line.** The worst case stays 2+2, both bands stay legal,
nothing is displaced, and no font size changes. It is free because **R-C already
requires a new footer string on that path** — the existing one is false once the
device has derived the fingerprints — so the line is being rewritten anyway.

The split is structural rather than chosen: the standalone path has **no
descriptor**, hence no policy id to render, so its footer is untouched.

**REQUIRED ASSERTION, not a comment.** `"FINGERPRINTS TYPED, NOT VERIFIED"`
(32) plus a policy id would be **50 characters against a 42-character band** —
over budget and, per §3b, rendered off both plate edges **with no refusal**.
This design is safe only because the typed footer and the policy id **never
co-occur**. That must be a test, because nothing in `band` enforces it.

**Options rejected, with the reason, so they are not revisited from scratch:**

| option | verdict |
| --- | --- |
| render only when a slot is free | rejected — present unpredictably, keyed to whether the passphrase has a space |
| displace the `␟ = SPACE` legend | **rejected outright** — that legend prevents a mistyped passphrase and a wrong wallet; it outranks an identifier |
| merge the two fingerprint lines (`SEED 73C5 DA0A  COMB FC60 C6DF`, 30 chars, fits) | viable and held in reserve — but costs the word **EXPECTED**, which marks a check target rather than a fact |
| shrink the band font to fit three lines | rejected — 2.33 mm on steel read years later, against the project's own minimum-feature rule, and it churns every band |
| omit the policy id entirely | rejected — but note R6's **key-id** half is already satisfied: the `mk` research confirmed the master fingerprint *is* the key identifier, and it is already on the plate |

### R-I — the scroll arrows FLOAT over the body's edges

Operator, 2026-08-17, choosing from four measured layouts.

**Measured geometry this rests on:** panel 480×320; `leadingSize = 44`;
`NavBtnPrimary` 53×53 at right-gutter slots **y = 44 / 133 / 223**, x = 427–480;
body clip (6,44)–(423,314), i.e. **417 wide**; and `assets.ArrowDown` /
`assets.ArrowUp` are **15×9** — icons, not buttons.

**The ruling:** draw the arrows at the **top-centre and bottom-centre of the
body**, over the 16 px fade zone, each with a background chip for legibility and
an **enlarged invisible touch target**. Not in the nav gutter, not in a new
column.

### Why this one, and it is a scheduling result more than an aesthetic one

**Body width is unchanged — 417 before, 417 after.** Wrap is what decides a
modal's capacity, and body width is what decides wrap. So:

> **R-I DECOUPLES F-192 FROM F-208.** The sequencing constraint recorded earlier
> in both follow-ups — *"the arrow layout must be decided before F-192's sweep
> sets its budgets, or every screen gets measured twice"* — **no longer binds.**
> The sweep and the arrows can proceed independently, and F-192's fit
> measurements stay valid whenever the arrows land.

The rejected options each failed on something measured, recorded so they are not
re-litigated:

| option | why not |
| --- | --- |
| free nav slots | `ErrorScreen` has **two** free slots (top, middle), `ConfirmWarningScreen` only **one** (middle) — the arrows cannot occupy the same position on both, and one screen cannot host both arrows |
| dedicated scroll gutter | costs ~25 px of 417 (~6%), re-opening the wrap calculation and making F-192 **strictly dependent** on it |
| single Down arrow | fits both screens, but leaves no way back up except wrap-around — surprising on a screen whose purpose is to stop a funds-losing mistake |

### Implementation constraints the spec must carry

1. **It must NOT go through `layoutNavigation`.** That function computes
   `idx := int(clk.Button - Button1)` and indexes a `[3]int`; `Up` and `Down`
   sort **before** `Button1` in the enum, so they index **negative**. This binds
   every layout option, not just this one.
2. **A larger hit area than the drawn icon is idiomatic here.** The nav button
   already separates the two — `op.Input(buf, t).Clip(...)` establishes the
   touch region independently of the mask that is drawn. A 15×9 icon needs a
   finger-sized target behind it.
3. **Scrolling itself is free.** A `Clickable` bound to `Up`/`Down` gets pointer
   routing (`gui/widget.go:70`) and press-and-hold auto-repeat
   (`gui/widget.go:48`) with no new machinery.
4. **The visibility predicate is R-E's problem, not this ruling's.** Arrows
   render **iff** content is actually below the fold — which, while `fadeClip`
   stays stubbed, is *not* `maxScroll > 0`. See R-E.
5. **The chip is not optional.** The arrows sit over the region where body text
   currently draws (`fadeClip` clips nothing), so without a background they can
   land on top of a glyph.

### R-J — the device preloads the FINGERPRINTS too, not just the passphrase

Operator, 2026-08-17, on R0 round 1's C1: **yes**.

**What C1 exposed.** R-C preloads the passphrase **bytes only**; both
fingerprints are still typed (`gui/passphrase_flow.go:665-678`, and the code
says so at `:628-629`). R-J closes that: the device supplies the fingerprints it
already computed, `ppStepSeedFP`/`ppStepCombinedFP` are skipped on the preloaded
path, and §2.3's footer may then truthfully read **`DERIVED, NOT TYPED`**.

This is more faithful to R-C's own purpose — *do not make the operator re-type
what the device already holds* — and the plate gets **stronger** fingerprints as
a side effect: derived beats typed-and-unchecked.

### R-J's two measured costs, stated because they are not free

**1. It costs one extra ~31 s KDF.** The engrave path derives **once**, with the
passphrase, at `gui/singlesig.go:107`. That yields `masterFP` — the **combined**
fingerprint — for free. The **bare-seed** fingerprint is a second derivation with
an empty passphrase, and a seed derivation on this device is a **~31 second
KDF** (`gui/gui.go:825`, `gui/gui.go:1653`, `gui/unlock_platelist.go:175`).

**2. It must happen AT `gui/singlesig.go:107`, not later — this is the binding
constraint.** That line's own comment reads *"The mnemonic is consumed for the
LAST time here."* Deriving the seed fingerprint afterwards would require keeping
the mnemonic alive past its scrub point, **weakening a security property to buy
a plate legend**. That trade is refused.

**NORMATIVE:** both derivations happen back-to-back at `gui/singlesig.go:107`,
while the mnemonic is legitimately alive; the scrub point does not move.

The ~31 s lands during engrave setup — not on a failure screen, and small
against a plate that takes roughly twenty minutes to cut.

### R-K — THE THREAT MODEL, stated. Sealed payload is the security program; the rest favour convenience

Operator, 2026-08-17, verbatim, answering the spec's refusal to extend the
mnemonic's lifetime:

> "yes, please weaken the security properties. The device is offline, can't
> write to flash, and the computerized portion is even somewhat
> disposable…sealed payload is our security program, the remaining programs are
> to favor convenience over security."

**This is far larger than S6b and is recorded at this level deliberately.** It
is the threat model the whole device has been reasoned about without ever being
written down, and its absence is why the spec reached for a refusal
(*"weakening a security property to buy a plate legend"*) that was never the
operator's policy.

**What it authorises.** In the **non-sealed-payload** programs, secret-lifetime
hardening is **not** a blocking concern, and convenience wins where the two
trade off. The premises are stated so they can be re-checked rather than
assumed: the device is **offline**, it **cannot write to flash**, and the
computerised portion is **disposable**.

**What it does NOT authorise, and the boundary is the whole point.** The
**sealed-payload** program remains the security program. Its properties are
**not** relaxed by this ruling. A change that touches sealed-payload secret
handling is out of scope for R-K and needs its own decision.

**Nor does it relax R-D.** Convenience over security is a ruling about
*secret-lifetime hardening*. It says nothing about truthfulness: *"all things
said must be true"* is unaffected, and a plate or screen may still not claim
what did not happen.

### R-K's consequence for R-J — the constraint is lifted, and a better design opens

R-J's *"must happen at `gui/singlesig.go:107`"* rested on the scrub point being
inviolable. **Under R-K it is not.** So the seed-fingerprint derivation may be
**deferred**, and should be:

**Derive the bare-seed fingerprint LAZILY — only if the operator chooses to
engrave a passphrase plate.** The ~31 s KDF is then paid only by runs that
actually want it, instead of by every single-sig engrave. That is strictly
better for the operator, and it is only available because R-K lifted the
constraint.

The combined fingerprint stays free — it falls out of the existing derivation at
`:107` regardless.

### R-L — the single-sig failed-verify copy is approved as specified

Operator, 2026-08-17: *"check passphrase is great advice."* §3.2's F-204 fix
stands, including its conditional second arm for the no-passphrase case.

---

## THE DECISION PASS IS CLOSED

Every question §3 posed is answered, except the one that was never a decision.

| §3 question | outcome |
| --- | --- |
| Q1 — which plate-text mechanism | **R-F** — optional `Title`/`Footer` on `backup.Text` |
| Q2 — does a title/footer FIT? | **not a decision — the gating SPIKE, still owed** |
| Q3 — which identifier is the "key-id" | **settled** — the master fingerprint; `mk1` defines no key-id concept |
| Q4 — watch-only sets | **R-A** — not marked |
| Q5 — multisig paths | **R-B** — moved to `key & password custody refinement` |
| Q6 — the goldens | **R-G** — no churn is the assertion |

Rulings not arising from §3: **R-C** (passphrase program preloaded), **R-D**
(*"all things said must be true"*), **R-E** (`fadeClip` stays stubbed).

**What S6b owes before a spec may close, in order:**

1. **The Q2 spike** — an executable measurement, not prose. It must answer:
   does a title/footer fit alongside text+QR on an `md1`/`mk1` plate at current
   sizes; what is the TEXT+QR threshold **with** a band present (today: 268/269
   without); what is the passphrase band's real footer budget (§2.4's 32 is a
   string length, not a measured limit); and does the band's third line problem
   (§2.5 / Q3) force `bottomLines`.
2. **The F-208 arrow layout** — before F-192's sweep sets its budgets, or every
   screen gets measured twice.

**The plan may not close while either has never been run.**

---

## 3. OPEN QUESTIONS FOR THE SPEC — not answered here

1. **Which plate-text mechanism** do `mk1`/`md1` move to (§2.3)? Everything else
   depends on it, including the length budget.
2. **Does a title/footer actually FIT** alongside the existing text+QR on an
   `mk1`/`md1` plate at current sizes? Unverified, and measurable. Engraving
   feature-size and bounding-box limits apply.
3. **Which identifier is the "key-id"** — the wallet-policy stub, or the mk1
   chunk-set id (§2.6)?
4. **Does marking apply to watch-only sets too?** They carry `mk1`+`md1` and no
   seed, and a watch-only build can still be passphrase-derived.
5. **Do the multisig paths get the same marking?** They have the same defect and
   more plates.
6. **What happens to the goldens?** Plate rendering has golden tests; any layout
   change churns them, and a churned golden is only as good as its review.

---

## 4. WHAT THIS IS NOT

- Not gated, not reviewed, and **not implementable**. It is input to a spec.
- It does **not** change S6a, which closes F-198's Critical independently.
- It does not decide sequencing against S6b (F-199 + F-204).
