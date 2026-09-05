# R0 round 0 — fidelity + design lens on `SPEC_hashlock_H6_preimage_plates.md`

**Artifact:** `design/SPEC_hashlock_H6_preimage_plates.md` at engrave master `a0f832d0`
(reviewed against the working tree at `3ad25cbd`, which carries the same spec bytes).
**Ground:** fork `fb0dd04`, engrave master, mnemonic-secret `504ff46`.
**Question asked:** implemented literally, does the spec produce plates that carry exactly
what they claim, through host and device paths that refuse everything H0 refused, with
every fork/host claim true and every number a measurement?

**Method.** Every measurable claim was re-run in a detached fork worktree at `fb0dd04`
(`/scratch/code/shibboleth/.tmp/h6-lens-fidelity`, Go 1.26.7, removed after the run).
The QR ceiling and alignment table were re-derived from the fork's own encoder; the
§6.5 geometry table was recomputed from `backup.CharsPerLine` / `LinesPerPlate` /
`fixedCharWidth`; every §8 headroom pair was re-measured through
`modalHeadroom`/`bodyDrawnFully`; §7.2 was implemented literally and run.

**What holds.** A large fraction of the spec is exactly right and was verified, not
assumed. Stated here so a fold does not re-derive it:

- **§7.1's ceiling table and ECC-L thresholds are exact.** Measured with
  `qr.Encode(..., qr.L)`: the 194-byte worst case is **53 modules = v9**; sha256/100 is
  135 B → 45 (v7); hardened/28 is 122 B → 41 (v6); sha256/1 is 36 B → 29 (v3). Thresholds
  ≥1→21, ≥18→25, ≥33→29, ≥54→33, ≥79→37, ≥107→41, ≥135→45, ≥155→49, **≥193→53**, ≥231→57 —
  every row as written. The correction of the ruling's "v7 = 53 modules" to **v9** is right.
- **§7.2's alignment table is exact, every row.** Re-derived by testing the 5×5 ring shape
  at every non-position-marker centre on the encoder's own bitmap: v2 (18,18); v3 (22,22);
  v4 (26,26); v5 (30,30); **v6 (34,34)**, single, and `dim−9 = 32` ✓; **v7** (22,6) (6,22)
  (22,22) (38,22) (22,38) (38,38); **v8** (24,6) (6,24) (24,24) (42,24) (24,42) (42,42);
  **v9** (26,6) (6,26) (26,26) (46,26) (26,46) (46,46). Six rings each, three dropped for
  the position markers, exactly as stated.
- **§6.5's geometry table reproduces**, one cell excepted (M-3): chars/line 19/23/26/31/34/39
  and lines/plate 13/15/17/20/23/26 at 6.0/5.0/4.4/3.8/3.4/3.0 mm; `constant.Font` W advance
  600 against `Metrics{Ascent:800, Height:900}`; 12,800 units = 2.0000 mm at 3.0 mm. The
  worst-case row arithmetic checks: phrase body 10 rows = 30.0 mm at 3.0 mm, 11 = 37.4 at
  3.4, 13 = 57.2 at 4.4, 16 = 80.0 at 5.0; string body 10 rows = 60.0 mm at 6.0 mm. QR at
  53 modules is 31.80 mm at scale 2 and 47.70 mm at scale 3. 65 mm is the correct body
  budget: at total = 65 mm exactly, `passphraseLayoutFor`'s centring puts row 0 at y = 10
  and the last row's bottom at 75, and `lineLayout.at`'s screw-hole predicate
  (`y < innerMargin`, `baseY+(i+1)*fontSize > plateHeight-innerMargin`) is false at both
  bounds. The 64 mm band ceiling holds 32 / 25 / 16 characters at 3.0 / 3.8 / 6.0 mm ✓.
- **Eight of the ten §8 headroom pairs reproduce to the character**: §8.3 107/455; §8.4
  LONGEST 411/121; §8.5 126/378; §9 204/302 and 184/378; §10.1 153/378 and 159/378; and
  the four rejected §8.4 drafts (my reconstructions gave 490/**39**, 457/**79** against the
  spec's 488/39, 455/79 — headroom identical).
- **§5.1's row measurement is exact.** `composerTextBand(480,320)` = band 411 px; every new
  row draws on ONE 23 px line: `preimage 10  b867db87..edbc96cb` 276 px,
  `phrase record 10 (derive to see the digest)` 336 px, `phrase 10  …` 253 px.
- **§3.4's D1 correction is right.** `sysw.Open` (`sysw/open.go:36-73`) runs no admission,
  and `syswSession.load` (`gui/sysw_session.go:79-110`) appends `p.Public` **and**
  `p.Secret` and classifies each — so a sealed `sysw` payload's preimage IS reachable after
  unlocking. `seal.Classify`'s codex32 arm (`seal/record.go:260`) and `permitted`/
  `AdmitSection` do refuse a preimage plate, so the two clauses of §8.2.4 are each true of
  the container they name. F1's text at `gui/sysw_load.go:283` is verbatim.
- **§4.2/§4.3's H0 reasoning is sound.** Keeping `isStrictMs1`'s wide
  `!codex32.IsPreimage(c)` while adding a NARROW admission predicate preserves the shipped
  property the `IsPreimage` header argues for (a mistagged real preimage must not fall
  into the seed class); a kind-0x03 single under a foreign id ends at `ClassUnknown`, inert,
  as §8.1.2 says.
- **Small facts that check out:** the corpus provenance pin is byte-exact (commit
  `c05074f1d45970ca416785dfa9d9a812aaa21dbd`, sha256
  `5b3960cad7f924f6f1e7f19ef49599814733cee4874d0f5eb48c28af4cd8b312`, 47 rows);
  the ms1 preimage string in `ms`'s `hashlock-v0.8.json` is **75 characters** with id
  `hash`; `hashlock.Salt/Iterations/PreimageLen` are at `:21,:24,:27`; `ValidatePhrase`'s
  order is exactly the order §3.1 states; the `now:` parser cuts on the FIRST comma
  (`strings.Cut`); the 96 B second-defer measurement is recorded verbatim at
  `gui/composer_flow.go:53-57`; `MaxTitleLen = 18` and the band literals are 17/15/10;
  `"PREIMAGE REQUIRED"` is 17 characters, matching `"PASSWORD REQUIRED"`; `bundlePlateMark`
  refuses to mark a `cardMS1`; ~10 s per hardened derive at 9,715 it/s is the shipped figure.

---

## Findings

### C-1 — §7.2 omits `constantTimeQRModules`, so every newly admitted version FAILS to build a QR at all; and §7.2 item 4's test cannot detect it

**Section:** §7.2 items 1–4, §11.3.

§7.2 enumerates the change as: add `41` to `bitmapForQRStatic`'s case arm, add the six-ring
table for 45/49/53, move `ConstantQR`'s bound to `dim > 53`, plus a test and goldens. It
never mentions `constantTimeQRModules` (`engrave/engrave.go:349-374`), which tabulates
21/25/29/33/37 and returns **0** for everything else ("Not supported, return a low number
to force error"). `nmod` is the budget `ConstantQR` checks (`if len(modules) > nmod`) and
the loop count `ConstantQRCmd.Engrave` runs (`for range nmod`).

**Counterexample.** I applied §7.2 items 1–3 to `engrave/engrave.go` literally — nothing
else — and ran `ConstantQR` on the spec's own §8.6 texts:

```
constantTimeQRModules(41) = 0   (45)=0   (49)=0   (53)=0
v6/hardened28    dim=41 ConstantQR err=too many dims 41 QR modules for constant time engraving n: 745 waste: 14
v7/sha256-100    dim=45 ConstantQR err=too many dims 45 QR modules for constant time engraving n: 881 waste: 41
v9/hardened100   dim=53 ConstantQR err=too many dims 53 QR modules for constant time engraving n: 1201 waste: 44
```

So the headline deliverable — a phrase-form plate carrying the QR — cannot be produced at
any of the four versions the raise admits.

**And the budget is content-dependent, so it cannot simply be invented.** Over 4,000 random
§8.6 texts (phrase 1..100 chars, both methods), `len(modules)` spread was:

```
dim=41 (v6)  min= 700 max= 785
dim=45 (v7)  min= 827 max= 917
dim=49 (v8)  min=1025 max=1156
dim=53 (v9)  min=1209 max=1281
```

A budget between min and max makes the plate cuttable for some phrases and refused for
others at the same QR version — an operator-visible, content-dependent failure on the funds
path, which is the very class `ConstantQR` exists to remove. The shipped v5 value was
derived by fuzzing (`engrave/engrave.go:359-368`: 18.5M executions, 32 min, +20 buffer with
its trend argument). The spec provides neither values nor a derivation method.

**§7.2 item 4's stated deliverable cannot catch this.** It asks for a test that "the emitted
move count is a function of `dim` ALONE — identical for two different payloads of the same
version". `Engrave` loops `for range nmod` and pads every move to `maxDur` via
`DelayMove(…, maxDur, …)`, so the emitted count is a function of `dim` alone **for any
value of `nmod`, correct or not**. The test passes on a budget of 0, on a budget of 700,
and on a correct one. The property that actually needs proving is that `nmod` upper-bounds
`len(modules)` over ALL payloads at that version.

**SUGGESTION.** Add a fifth normative item to §7.2: `constantTimeQRModules` gains arms for
41/45/49/53, each value derived by the same fuzzing protocol the v5 comment records, with
the observed maximum, the sample count and the buffer stated in the code comment as v5's
is; and state the trend check (ratio of observed max to `dim²`) so an under-converged fuzz
is visible. Then restate §11.3's constant-time row as two rows: (a) `len(modules) <= nmod`
for N fuzzed payloads at each of v6..v9 — the row that can actually fail — and (b) the
move-count-is-a-function-of-dim row, labelled as the regression guard it is.

---

### C-2 — §6.4 pins `engrave.ConstantQR` and §6.5 pins scale 2; `engraveModule` PANICS on scale 2, and the whole fit table rests on it

**Section:** §6.4 bullet 3, §6.5 normative consequence 1, §11.4.

§6.5 consequence 1: *"The QR scale is 2, not the passphrase plate's 3. At 53 modules,
scale 3 is 47.70 mm and does not fit at any rung; scale 2 is 31.80 mm."* §6.4: *"It is built
with `engrave.ConstantQR` … never `engrave.QR`."* Those two are incompatible in the fork as
it stands. `ConstantQRCmd.Engrave` renders each module through `engraveModule`
(`engrave/engrave.go:688-710`), whose `switch scale` has **only `case 3` and `case 4`** and
a `default: panic("unsupported module scale")`.

**Counterexample** (budget temporarily stubbed so a v9 command could be built at all):

```
dim=53 cmd built ok
scale=2 PANIC: unsupported module scale
scale=3 emitted 51 cmds
```

The cited precedent does not transfer: `freeTextQRScale = 2` (`backup/fit.go:16-19`) belongs
to the free-text plate, which draws through `engrave.QR` (`engrave/engrave.go:277-317`) — a
raster path that never calls `engraveModule`. Every ConstantQR caller in the tree uses
scale 3 (`passphraseQRScale = 3`).

This is load-bearing, not incidental: §6.5's only FITS verdict for phrase+QR
(63.80 mm at 3.0 mm, 1.20 mm spare) exists solely because scale 2 is 31.80 mm; at scale 3 the
worst case is 79.70 mm and the plate does not exist at any rung. §11.4's gate ("MUTATION:
set the QR scale to 3 → it reds") therefore protects a configuration the encoder cannot
emit.

**SUGGESTION.** Make the scale-2 module shape a normative item of §7 alongside the version
raise: state the `case 2` path `engraveModule` gains, state that it emits a **constant**
number of `Line` commands (the constant-time argument is per-module command count plus
`DelayMove`'s `maxDur` padding, and a new arm re-earns it), and add a golden at scale 2 to
§11.3's list. If scale 2 turns out to cut illegibly at 0.6 mm modules on steel, the honest
alternative is to say the phrase form's QR is out of scope at this stage rather than to
keep a fit table that rests on an unreachable scale.

---

### C-3 — §3 never says where `--pack-preimage` enters classification, and BOTH readings break something funds-relevant

**Section:** §3.1, §3.2, §3.4 item 1, §4.2, §11.1.

§3.2 gives the clap arg and says *"Without it, `me sysw pack` keeps refusing by index through
`admit_check` (`crates/me-cli/src/sysw/mod.rs:463-470`) and `unknown_reason`'s preimage arm
(`:209-211`)"*. §3.1 and §4.2 describe classification unconditionally (`phrase:` "joins
`key:`, `hash:`, `now:` as a RESERVED prefix"; the ms1 arm "learns the preimage class through
a NEW predicate"). Those cannot both be true, and the fork/host code makes each reading
fail differently.

**Reading (a), admission-gated** (the only reading under which §3.2's refusal survives — the
mechanism already exists: `Admission { allow_unsigned_inputs }`, `classify_with(record, adm)`,
`admit_check(records, adm)`):

1. `decide_sealing` (`crates/me-cli/src/main.rs:2353-2410`) filters with
   **`sysw::classify(r)`** — the strict, `Admission::default()` classifier (`mod.rs:246-248`)
   — and is called at `main.rs:1696` **without** the `admission` value that exists at
   `main.rs:1478`. A preimage under `--pack-preimage` would classify `Unknown` there,
   `is_secret()` false, and `decide_sealing` takes its `secret.is_empty()` branch and prints:

   > `sealing:  NOT SEALED — no record in this payload is secret material, so there is nothing to encrypt. The container is cleartext: anyone holding the file can read it.`

   That is a false statement on stderr and a **cleartext payload holding bearer material**,
   which is the exact opposite of §3.4 item 1's normative "SEALS by default". It also
   silently disables §8.2.4, whose condition (SEALED and holds a preimage) never arises.
2. The vendored corpus round-trip breaks. `crates/me-cli/tests/sysw_composer_records.rs`
   generates every fixture row with `classify(c.record)` (strict), and the fork's
   `TestComposerRecordsClassifyExactlyAsTheHost` asserts `Classify(row.Record)` equals the
   row's class. With classification admission-gated on the host and unconditional on the
   device (the device has no admission flag — `syswSession.load` calls `sysw.Classify`), the
   new rows say `Unknown` in the corpus and `ClassPreimage`/`ClassPhrase` on the device, and
   the lockstep gate reds. §3.1's "regenerated by its own test and re-vendored" cannot hold.

**Reading (b), unconditional** (the plain reading of §3.1/§4.2): `admit_check` never returns
`Unclassifiable` for a preimage plate, so `unknown_reason`'s `U::PreimagePlate` arm becomes
unreachable, `--pack-preimage` admits nothing that was refused, and §8.1.1's added sentence
("Re-run with --pack-preimage if that is what you intend") never prints. The operator's
decision that admission be **explicit** is then unimplemented, and §11.1's row
("without it both are refused by index with §8.1.1's text") cannot pass.

**SUGGESTION.** Make §3.2 normative about the mechanism: `Admission` gains
`pack_preimage: bool`; `classify_with` answers `Preimage`/`Phrase` only under it and
`Unknown` otherwise; and — the part that must be stated because it is a *second* call site —
`decide_sealing` takes `admission` and uses `classify_with`, so the sealing verdict is
computed under the same admission the pack used. Add a §11.1 row for exactly that:
`--pack-preimage` alone (no `--no-passphrase`) SEALS and the sealing line names the class;
MUTATION: pass `Admission::default()` to `decide_sealing` → the row reports NOT SEALED. And
state how the corpus is generated for the two new classes (either generate the H6 rows under
the permissive `Admission`, or record both answers per row) so the vendored lockstep test
still has one truth to compare against.

---

### C-4 — §5.3 puts the per-plate form/QR pick, the decline and the `show` toggle on `confirmReviewScreen`, which has no free input

**Section:** §5.3 opening and items 3, 5, 7; §12 acceptance item 1.

§5.3 pins the review to *"the census screen `confirmReviewScreen(ctx, th, "Plates To Cut",
composerCensusLines(...))` (`:389-390`, `gui/multisig_build.go:1895`)"* and then requires,
**on the same screen**: (3) form and QR chosen per plate, (5) a plate declined, (7) a phrase
masked with a `show` toggle held to reveal.

`confirmReviewScreen` (`gui/multisig_build.go:1895-1957`) consumes every input the device
has: `backBtn = Button1`, `pageBtn = Button2`, `contBtn = Button3 + Center`. Its body lines
are `widget.Labelw` ops offset into the frame — **not** `Clickable`s, so they are not touch
targets. On the machine the only production input is the ft6x36 panel
(`gui/composer_paged.go`'s W-2 note: "SeedHammer II has no directional buttons"), and W-2's
own measurement was 205 taps that moved nothing on a screen whose rows were not targets.

So there is no button and no touch target left for three distinct per-row interactions. An
implementer would have to invent a screen the spec does not describe, and §12's acceptance
item 1 ("reaches Done, **accepts** a preimage plate in the phrase form **with a QR**") is
not reachable through the screen §5.3 names.

**SUGGESTION.** Split the review in two, and say so: the census stays a
`confirmReviewScreen` (read-and-confirm) carrying §8.3's rows, and the per-plate decisions
move to a `composerPickScreen`-shaped step *before* it — that screen already has row-tap
targets, a cursor, Button3-takes and Button1-declines, and it is the shape the composer's
other per-item decisions use. State which screen owns each of items 3, 5 and 7, and state
the `show` toggle's control explicitly (the phrase keyboard's held-reveal is a keyboard
affordance, not a pick-screen one).

---

### I-1 — §8.4 attaches the abort clause to `bundleAbortWarningText`, where its stated condition can never hold

**Section:** §5.4, §8.4, §11.5.

§8.4: *"`bundleAbortWarningText` (`gui/bundle_flow.go:780-792`) gains a third clause …
**Fires when the run ends before every accepted preimage plate is cut.**"* But §5.4 cuts
every accepted preimage plate in its own loop **before** calling `bundleEngrave`, and
`bundleAbortWarningText` is reached only from `bundleAbortWarning` at
`gui/bundle_flow.go:625` and `:640` — both inside `bundleEngrave`. By construction, at every
site where that text can be built, every accepted preimage plate has already been cut. The
clause is dead.

The window §5.4 says the arm covers ("a blank runs out mid-plate") is the *other* path —
`NewEngraveScreen(ctx, pl).Engrave(...)` returning false, which §5.4 routes to
`composerAbortNoPreimage(ctx, th, st)`. §8 gives that function no body: §8.4's text is
attached elsewhere, so the one screen that must stop a funding decision has no specified copy.

Two mechanical consequences the spec does not address: `bundleAbortWarningText(p bundlePlate,
secret bool)` has no channel for the new fact, and the shipped comment at
`gui/bundle_flow.go:610-616` **prohibits** a variadic tail on this call chain by name
("it would leave order and arity unchecked on the value deciding whether a plate says
PASSWORD REQUIRED"). §11.5's row ("the abort arm fires when the run ends before a preimage
plate is cut, and not when every one was") therefore has no implementation to test.

**SUGGESTION.** Move §8.4's clause to `composerAbortNoPreimage` and say so — it is the
composer's own screen, it already knows how many plates were accepted and how many were cut,
and it needs no signature change anywhere in `bundle_flow.go`. Keep the measured
headroom pair for the LONGEST variant only if the combined body is genuinely reachable
there; if `composerAbortNoPreimage` shows the preimage clause alone, re-measure it alone
(see I-6) and delete the "+ seed clause" arithmetic.

---

### I-2 — §11.4's band-budget mutation cannot fail: no locator row exceeds the band cap

**Section:** §6.3, §11.4 bullet 3.

§11.4: *"MUTATION: move `mk1 stub (template): <8 hex>` into a band → the geometry gate fails
at 3.0 mm (29 characters against the 32-character cap is inside, but the 33-character `path`
row is not, and the gate measures the longest)."*

Measured: the band cap at 3.0 mm (`plateSmallFontSize`) is `F(64)/advance = 409600/12800 =
32` characters. §6.3's locator row set is `path <n>` (6), `hash  <first8>..<last8>` (24),
`mk1 stub (policy): <8 hex>` (26), `mk1 stub (template): <8 hex>` (29),
`matches hash <i> in the payload` (30 at a two-digit `<i>`). **The longest is 30, and every
one fits.** The "33-character `path` row" the mutation relies on is `path 2   hash
b867db87..edbc96cb` — a concatenation that appears nowhere in §6.3's row list, which puts
`path <n>` and `hash …` on separate rows. So the named mutation passes the gate.

The same straw row is what §6.3 uses to justify the design: *"They are body rows because they
do not fit a band."* Two of the three cited rows do fit a band at 3.0 mm. The real
constraint is the **line count**: `passphraseLayoutFor`'s own comment records that a band
offers `innerMargin - outerMargin = 7 mm` and holds at most two 3 mm lines, and §6.2 already
spends the top band on the title and the bottom band on the space legend plus `NOT A SEED` —
so there is no room for up to four locator rows regardless of width.

**SUGGESTION.** Restate §6.3's justification as the line-count one (with the 7 mm / two-line
measurement, which is in the tree), and rewrite §11.4's mutation to one that can fail — e.g.
"MUTATION: draw any locator row at `l.topY`/`l.bottomY` → the ink-in-band assertion fires",
which tests placement rather than width and does not depend on a row nobody proposes.

---

### I-3 — §10.2's suppression guards a screen a payload phrase cannot reach; §11.5's mutation cannot fail

**Section:** §10.2, §11.5.

§10.2 makes the reconcile screen conditional: *"The screen is drawn only when the digest's
provenance is `hashlockFromPhrase`."* `composerCopyHashlockReconcile` has exactly **one**
call site in the whole tree — `gui/composer_hashlock.go:83`, inside `hashlockPhraseRoute`,
immediately after the device-typed phrase's confirm. A payload `phrase:` record is picked
from `Which hash?`'s new band (§5.1 row group 3) and never enters `hashlockPhraseRoute`, so
the screen is already unreachable for it. The guard changes no behaviour.

§11.5's row — *"MUTATION: key §10.2 on `phraseDigests` instead of provenance → the payload
row draws it"* — therefore cannot fail either: at that call site, no keying draws the screen
for a payload row, because the payload row does not execute that code.

**SUGGESTION.** Either delete §10.2 and §11.5's row (recording that the screen is
provenance-correct by construction, and *why*, so the next reader does not re-open it), or —
if the intent is that a *payload* phrase should also get a reconcile-style screen and merely
a different one — say that instead, and move the guard to whatever new shared site §5.1's
lazy-derive arm introduces.

---

### I-4 — §10.1's third §8h arm asserts a plate decision the operator has not yet made

**Section:** §10.1, §5.3 item 5.

§10.1's new arm reads *"**This run cuts a preimage plate for each one.** Store those plates
apart from these"*, chosen *"when EVERY hashed path's digest has material in `hashlockHeld`
that this run will cut"*.

`composerCopyHashEveryPathFor` is called from exactly one site,
`gui/composer_shape.go:442-443`, inside `composerShapeFlow` — which `composerFlow` runs at
line 75, whereas `composerEngraveStep` (where §5.3's per-plate accept/decline happens) runs
at line 125. So the banner is drawn several steps *before* the plate set exists: before the
form pick, before the census, and before the operator can decline a plate — which §5.3 item
5 explicitly permits ("Declining does not abort the run"). An operator who reads "this run
cuts a preimage plate for each one" and then declines one is left with a backup instruction
that was false, on the screen whose whole job is to say what spending needs.

This is the defect class H5 §1.2 removed from the confirm modal, and §8.4 cites it correctly
in its own reasoning ("Saying it is gone would be false on the screen whose job is to stop a
funding decision") — the same standard is not applied here.

**SUGGESTION.** Phrase the third arm in the present tense of what is *held*, not the future
tense of what will be cut — e.g. "This composition holds the preimage for each one and can
cut a plate for it at Done" — or move the arm to a screen after the census, where the
decision is final. Either way, add a §11.5 row that declines a plate and asserts the §8h
body does not claim it was cut.

---

### I-5 — §4.3's statement about the host's `preimage_plate` is false, and §11.1's "any other id" row fails for the most likely id

**Section:** §4.3, §8.1.2, §11.1 bullet 3.

§4.3: *"the host's `preimage_plate` (`crates/me-cli/src/seal/record.rs:287-320`) does not
[consult the id] either — it tests `unshared && len(data)==33 && data[0]==0x03`."*

It does consult the id. `preimage_plate` opens with

```rust
// An id/kind MISMATCH is diagnosed separately (ruling L24), never as a plate.
if id_kind_mismatch(s) { return false; }
```

and `id_kind_mismatch` is true when `ms_codec::decode` returns `TagKindMismatch`, which
`crates/ms-codec/src/decode.rs:86-96` raises for `x == TAG_ENTR || x == TAG_HASH` when the
tag does not name the payload's kind. So a kind-0x03 single under the id **`entr`** is
already refused today, by `U::TagKindMismatch` (`main.rs:2799-2804`), whose shipped text
already cites *"SPEC_ms_hashlock §1 rule 2"* — the same citation §8.1.2 proposes to add in a
second, near-identical body. Only ids outside `{entr, hash}` reach `U::PreimagePlate` (they
decode to `UnknownTag`, `id_kind_mismatch` is false, and the fallback shape test answers
true).

§11.1's row — *"the same payload under any other id is refused with §8.1.2"* — fails for
`entr`, which is the first "other id" an implementer or a reviewer is likely to pick, since
it is the only other tag the codec knows.

**SUGGESTION.** Correct §4.3's sentence to "the host's `preimage_plate` consults the id only
to route an `entr`/`hash` mismatch to `TagKindMismatch`; for every other id it tests
`unshared && len==33 && data[0]==0x03`". Then either scope §8.1.2 to ids outside
`{entr, hash}` and say so in the row ("MUTATION: use `entr` → the row now expects the
shipped TagKindMismatch text"), or drop §8.1.2 entirely and extend the shipped
`TagKindMismatch` body by one sentence — §9's own rule ("two near-identical bodies is how
one of them goes stale") argues for the second.

---

### I-6 — §8.1.2's body cannot render on a device modal at all, and two of the ten stated measurements do not reproduce

**Section:** §8 preamble, §8.1.2, §8.4, §11.5 last bullet; and the spec header's guarantee
that "Every number in this spec is a measurement".

§8 opens **"ASCII only."** Five of its own bodies are not: §8.1.1, §8.2.1, §8.2.3 and §8.2.4
carry an em dash (`—`) and §8.1.2 carries `§`.

That is not cosmetic on this device. `font/bitmap/bitmap.go:33` sets
`indexLen = unicode.MaxASCII` and `glyphFor` rejects `int(r) >= indexLen`, so — as
`gui/font_coverage_test.go`'s header records — **an unrenderable rune blanks the ENTIRE body
of its frame**, and `ExtractText` still reports the text as present. Controlled measurement,
same body and same length, only the marked glyph differing:

```
section-sign   the modal drew only 5004 ink pixels (floor 6000)
ascii-S        drawnFully=true drew=153 want=153
em-dash        the modal drew only 5004 ink pixels (floor 6000)
```

§8.1.2 states *"The wording measured **165 characters drawn, headroom 397** on
`errorScreenBody` so that it can be surfaced later without a re-measure."* That measurement
cannot have been taken on the quoted body: with `§` present the frame falls below the raster
floor, and with `§` replaced by ASCII the quoted body measures **242 drawn / headroom 320**.
The stated 397 does reproduce — on the body's second sentence alone (159 drawn / 397), not
on the wording as quoted.

§8.4 states *"**288 drawn / headroom 244** alone"*. Measured, the preimage clause alone is
**75 drawn / headroom 476**; the clause prepended to the non-secret shipped abort text is
317/219; the shipped secret text alone is 336/201. Nothing I could construct gives 288/244.
The companion figure (411 drawn / headroom 121 for the LONGEST variant) reproduces exactly.

Neither error changes a fit verdict — both bodies clear the 80-character margin — but §11.5
requires *"Every body §8 adds is in `modal_fits_test.go`'s table and passes
`assertModalBodyFits`"*, and §8.1.2's body as written would RED at the raster floor rather
than pass.

**SUGGESTION.** Replace every em dash and `§` in §8 with ASCII (`--`, "section"), re-measure
§8.1.2 and §8.4's stand-alone figure and paste the values, and add one line to §8's preamble
recording *why* ASCII-only is a hard rule here (the blanking mechanism at
`font/bitmap/bitmap.go:33`, with the 5004-vs-full-frame measurement), so the next author does
not treat it as house style. Host-only lines may keep an em dash if the spec says which
bodies are host-only and therefore exempt — but then §8.1.2 must stop claiming a device
measurement.

---

### I-7 — §5.2's Hashlock plates flow must print a digest it has no way to compute for a `phrase:` record

**Section:** §5.2, §2.3.

§5.2: the flow *"lists the payload's preimage and phrase records"*, *"builds no composition
and reads nothing from `composerState`"*, and its locator header *"prints the digest
**always**"*. §2.3 reinforces that it *"holds nothing"*.

For a `ClassPreimage` record the digest is cheap (`DecodeMS1Preimage` then `hashlock.Digest`).
For a `ClassPhrase` record there is no digest in the record — §3.1's wire form carries
`<method>,<phrase>` and nothing else — so the digest requires a full derivation: ~10 s of
PBKDF2 for `hardened` at the fork's measured 9,715 it/s. §5.1 specifies lazy derivation
behind the `Deriving` countdown for the **composer** path and enters the result into
`hashlockHeld`; §5.2 specifies neither the derivation nor anywhere to keep it, while
requiring the digest unconditionally and, for the QR form, the phrase itself.

As written the flow either cannot draw its locator for a phrase record, or re-derives on
every screen that shows the digest — a 10 s stall per redraw, on a device where the operator
can page and go Back.

**SUGGESTION.** Say in §5.2 that a `phrase:` record derives once on PICK, behind the same
`Deriving` countdown §5.1 uses, and name where the result lives for the duration of the flow
(a local, scrubbed on return, is enough and keeps §2.3's "holds nothing in `composerState`"
true). Add a §11.5 row asserting the locator's digest equals the host's for a `phrase:`
record, and one asserting the derivation runs once per pick rather than per frame.

---

### M-1 — §2.2 item 1 describes `composerState`'s construction wrongly

`composerState` is **not** "built as a zero-value struct literal" at
`gui/composer_flow.go:48`; it is
`st := &composerState{reg: &seedRegistry{}, bound: composerBoundFrom(ctx.sysw)}`. The
conclusion the spec draws (the new map arrives nil, so the insertion site must allocate) is
correct, and the spec at least fixes the line number the shipped comment gets wrong
(`composerNotePhraseDigest` says `gui/composer_flow.go:34`).
**SUGGESTION.** Say "built with two fields set and every other field zero, so `hashlockHeld`
arrives nil", and file the stale `:34` in `composer_state.go:275` as a nit for the same fold.

### M-2 — §6.3's width justification is measurably wrong (see I-2)

Carried as a Minor separately from I-2 because the *conclusion* (locator rows are body rows)
is right; only the stated reason is false. **SUGGESTION.** Fold with I-2.

### M-3 — §6.5's 5.0 mm advance cell reads 3.435 mm; measured 3.3333 mm

`fixedCharWidth(constant.Font, F(5.0)) = 600 * 32000 / 900 = 21333` units = **3.3333 mm**.
The error is inert — the chars/line figure of 23 is right either way (505600/21333 = 23.7,
505600/21984 = 23.0) — but the header guarantees every number is a measurement.
**SUGGESTION.** Correct the cell to 3.333 mm.

### M-4 — the wrap rule is unstated, and the 6.0 mm string verdict depends on it

§6.5 computes every row count as `ceil(len/charsPerLine)`, i.e. a hard **character** wrap
(which is what `passphraseLayoutFor` does: `l.rows = (len(glyphs)+rowLen-1)/rowLen`). Under a
word wrap the string form's header at 6.0 mm becomes 6 rows instead of 5
(`mk1 stub (template): 1a2b3c4d` breaks into three), the body becomes 11 rows = **66.0 mm**,
and the plate no longer fits at 6.0 mm — the auto-fit would drop to 5.0 mm and every golden
would differ. A character wrap is in any case forced (a 100-character phrase or a 75-character
ms1 string can be a single token), but §6 never says so.
**SUGGESTION.** One sentence in §6.1 or §6.5: the body wraps at exactly `charsPerLine`
characters, never on word boundaries, because the phrase and the ms1 string are single
tokens; and note that this splits an 8-hex stub mid-token at the narrow rungs.

### M-5 — citation drift, five places

`backup/freetext_test.go:75` for `TestTitleCapFitsAtEveryRung` → the test is at **:109**
(:75 is a comment naming it); `gui/composer_engrave.go:40-61` for *"no id yet"* → the string
is at **:80**; `gui/composer_door.go:93-97` for `composerDoorHasConsumablePolicy` → it is at
**:84-90** (:93-97 is `composerDoorFlow`'s doc comment); `gui/composer_flow.go:388` for "the
ms1 secret cards keep their place at the head of `cards`" → the append is at **:381** (:388
is a comment); `backup/freetext.go:11-19` for `TitleString` upper-casing and truncating →
`TitleString` is at **backup/backup.go:111** (freetext.go:11-19 is the comment that describes
it). Everything else in §14 that I checked resolved exactly, including all twenty me-cli
citations.
**SUGGESTION.** Re-run `scripts/plan-staleness-check.sh` over §14 at fold time and fix these
five.

### M-6 — §3.3's orphan-warning placement and the warnings' order against the passphrase ceremony

§3.3 item 3 says the orphan check *"lives with `pack_with`
(`crates/me-cli/src/sysw/mod.rs:335`) beside the 'at most one `now:`' rule"*. The
`now:` rule is enforced in `split` (`mod.rs:~477`) and, for ordering reasons, a second time in
`main.rs:1597`; `pack_with` is the entry point above them. Separately, §3.3 never orders the
four warnings against the passphrase ceremony, which `main.rs:1474-1478` and `:1580-1588`
treat as normative (F-246: nothing that could refuse or alarm may print *after* the operator
is told to write a passphrase down) — while §8.2.4's wording ("the device needs **the
passphrase above**") requires that one to print *after* it.
**SUGGESTION.** State the site as `main.rs`'s pack arm alongside the existing `now:` check
(before the ceremony) for warnings 1–3, and after the sealing line for warning 4, and say so
in §3.3 so the F-246 ordering is a stated rule rather than an accident.

### M-7 — §8.3's census rows are never measured against `confirmReviewScreen`'s band

§8 measures ten bodies and none of §8.3's seven census rows. Measured at
`sh2DisplaySize`, `confirmReviewScreen` wraps at `dims.X - 2*8 = 464` px and centres at
`(480-w)/2`, while the nav column starts at 427 px — so any row wider than 374 px has its
right edge under the buttons. All seven new rows exceed it (389–459 px), as does the shipped
completeness line (459 px), so this is a pre-existing property of that screen rather than
something H6 introduces — but it is the W-3 class the composer's own paged screens were
rebuilt to remove, and the spec adds seven rows to it without a measurement.
**SUGGESTION.** Either measure the rows and record that the overlap is pre-existing and
accepted, or route the census block through the `composerPageLines` band (411 px), which is
the surface that already solves this.

### N-1 — "the four band literals" (§8.7, §11.4): there are **three** distinct literals
(`HASHLOCK PREIMAGE`, `HASHLOCK PHRASE`, `NOT A SEED`) in four table cells. §6.2's own
"MEASURED lengths: 17, 15, 10" gives three numbers.

### N-2 — the H0 guard inventory (§4.3) omits two live `IsPreimage` call sites:
`gui/unlock_session.go:197` (`unlockEngraveCodex32`'s "this is the one call on the sealed path
that cuts metal") and `seal/record.go:260` (`seal.Classify`'s codex32 arm). Both are covered
in spirit by "`seal.Classify` and `seal.AdmitSection` are not touched", but a grep-based
re-check at plan time will find them and should not have to re-decide they are safe.

### N-3 — §5.3 item 2 appends the preimage block "by `composerCensusLines`", whose signature is
`(params engrave.Params, cards []bundleCard)` and which therefore cannot see the plates or
`hashlockHeld`. The needed parameter is not stated.

---

## Counts

**4 Critical · 7 Important · 7 Minor · 3 Nit**

Critical: C-1 (the QR raise omits the module budget and its test cannot detect it), C-2
(scale 2 panics under `ConstantQR`, and the fit table rests on it), C-3 (`--pack-preimage`
has no stated place in classification; one reading ships bearer material in cleartext, the
other makes the flag inert), C-4 (the per-plate review has no input budget on the screen it
is specified on).

The spec is unusually well measured where it measures — §7.1, §7.2's alignment table, §6.5's
geometry and eight of ten §8 headroom pairs all reproduce exactly, and §3.4's D1 correction
is right. Every Critical is a **step nobody ran**: the raise was never built, the scale was
never engraved, the flag was never traced past `admit_check`, and the review screen was never
asked what button it had left.
