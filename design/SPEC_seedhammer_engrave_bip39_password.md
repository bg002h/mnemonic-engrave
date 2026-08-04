# SPEC — Engrave BIP-39 Password (SeedHammer II fork)

**Status:** **AMENDED 2026-08-03, pending re-gate.** The original spec reached
R0 GREEN (0C/0I) over three rounds, but Phase A Task 4 then hit a construction
panic none of them found (`timeConstantPath` requires single-run glyphs). §3.5.0
amends the design with per-run quantization and an accepted timing disclosure.
**R0 round 0 on the amendment: NOT GREEN (2C/4I) — all findings folded
2026-08-03, pending re-review.** Phase A Tasks 1-3 are complete and unaffected.

**Original status:** R0 GATE GREEN (0C/0I) as of 2026-08-03 — implementation may begin.
Three rounds: fable (2C/6I) → opus (0C/4I) → sonnet verification (GREEN). This
feature is **risk-set** under `CLAUDE.md` clause (b): it handles secret material
that guards funds, so the post-implementation adversarial execution review over
the whole diff is **mandatory and non-deferrable**.

**Repo:** `bg002h/seedhammer` (fork). Tooling/design docs live in
`bg002h/mnemonic-engrave`.

**Date:** 2026-08-03

---

## 1. Purpose

Add a top-level program that engraves a **BIP-39 passphrase** onto a metal plate,
optionally alongside two user-supplied fingerprints that record which seed the
passphrase belongs to.

A BIP-39 passphrase is the one part of a seed backup that today's SeedHammer
cannot preserve. A user with a passphrase-protected wallet has a durable metal
backup of their mnemonic and a paper or memorised copy of the passphrase — and
losing the passphrase loses the funds just as completely as losing the seed.

### 1.1 Fields

| Field | Required | Source | Max |
|---|---|---|---|
| Password/Passphrase | **yes** | typed on device | 100 chars |
| Seed Fingerprint | no | **typed by user** | 8 hex chars |
| Expected Combined Fingerprint | no | **typed by user** | 8 hex chars |
| Include QR code | no (default off) | user choice | — |

---

## 2. Decisions and their rationale

Each of these was settled during brainstorming; the rationale is recorded because
the *reasons* constrain future changes more than the choices do.

### D1 — Fingerprints are typed, never computed. They are claims, not proofs.

The device does **not** ask for the mnemonic and does **not** verify either
fingerprint. Consequence: the seed never enters device memory in this flow, and
there is no 24-word entry step.

The cost is real and must be surfaced to the user at every step: **nothing is
verified.** A mistyped Expected Combined Fingerprint is *worse than a blank
field*, because a future reader will trust it.

**A key fact that forces this design:** the Combined Fingerprint **cannot** be
derived from the Seed Fingerprint plus the password. BIP-39 places the passphrase
in the PBKDF2 *salt* —

```
seed = PBKDF2-HMAC-SHA512(password = mnemonic sentence,
                          salt     = "mnemonic" + passphrase,
                          2048 iterations, 64 bytes)
```

— so changing the passphrase yields an entirely unrelated seed and master key.
There is no algebraic path between the two fingerprints. Two independent walls
each make it impossible: (a) a fingerprint is only the first 4 bytes of
`RIPEMD160(SHA256(master pubkey))` (`bip32/bip32.go:38`), which is lossy beyond
recovery; and (b) the passphrase acts on the mnemonic words, upstream of the
seed, so even a full master key would not help. Computing it requires the
mnemonic itself.

This is also precisely *why* the Combined Fingerprint is worth engraving: it is a
commitment to the (seed, passphrase) **pair** that nobody can produce without
holding both.

### D2 — The second field is named "Expected Combined Fingerprint".

Not "Combined Fingerprint". The word **Expected** carries the caveat into the
field name, where it survives being read off a plate years after the UI warning
is forgotten. Both the UI and the plate state that it is a user-typed reminder,
**not** a confirmation.

### D3 — Charset: all 95 printable ASCII. Non-ASCII refused loudly.

BIP-39 permits **any** string as a passphrase — there is no symbol whitelist in
the spec, and `é`, `日本語` and emoji are all valid. The ASCII restriction is
**this device's limitation**, and the UI must say so rather than implying the
input is invalid.

The primary justification is **not** the font. It is conformance:

> `bip39.MnemonicSeed` (`bip39/bip39.go:217-226`) performs **no NFKD
> normalization** — it feeds raw passphrase bytes into PBKDF2, and `unicode/norm`
> appears nowhere in the tree. BIP-39 requires NFKD normalization of both the
> mnemonic sentence and the passphrase. For ASCII this is a no-op, so all current
> behavior is conformant. For non-ASCII it is not: `é` as U+00E9 versus
> `e`+U+0301 would derive **different seeds** here than a conformant wallet
> derives from the same typed characters.

Accepting a non-ASCII passphrase would therefore risk engraving a passphrase that
this device and the user's other wallet disagree about. **ASCII is the boundary
of provable conformance.** Filed separately as
`bip39-passphrase-nfkd-normalization` (see `FOLLOWUPS.md`), which carries a
**mandatory Rust-primary check** because `bip39` is a listed Go port and
normalization is normative behavior.

### D4 — Extend the engraving font. Do not encode, do not restrict.

The alternatives considered were (i) restricting the passphrase to the currently
engraveable characters and (ii) engraving a case-insensitive *encoding* of the
passphrase bytes. Both were rejected: (i) rejects a large fraction of real
passphrases, and (ii) makes the plate non-human-readable and requires a decoder
at recovery time.

### D5 — Font metrics unchanged; accept a 1-unit descender.

`font/constant` is a single-stroke centerline font: em box 9 units, baseline at
y=8, cap height 6 (`A` spans y2→y8), default advance 6. That leaves **1 unit
below the baseline** for the descenders of `g j p q y`.

Raising the em box to 10 for a proper 2-unit descender would rescale **every
existing glyph**, because `engrave.String` scales by `Metrics.Height`
(`engrave/engrave.go:1375-1377`). That changes the geometry of every plate the machine
produces — including seed backups already in the field — and trips the
geometry-golden drift-guard in the `me-preview` sidecar. Not worth better
letterforms.

Keeping the metrics confines the entire change to codepoints that previously had
no glyph, which makes existing output **provably** unaffected: no string the
device can engrave today contains a lowercase letter or any of the 17 added
symbols.

At the passphrase plate's font size a 1-unit descender is ~0.67 mm against a
0.3 mm stroke — a little over two stroke widths. Shallow but legible. Marked
revisitable.

### D6 — New plate type, not a flag on `SeedString`.

`backup.SeedString` calls `strings.ToUpper` on its content
(`backup/backup.go:114`). Adding a `preserveCase bool` would mean a single wrong
argument uppercases a passphrase (destroying it) or mixed-cases a seed plate.
A separate `backup.Passphrase` type makes that mistake **unrepresentable**.

### D7 — Menu position: second of seven.

Inserted directly after `backupWallet`. This preserves `bip85Derive` as the last
navigable program, so the compile-time guard at `gui/gui.go:164`
(`var _ [1]struct{} = [qaProgram - bip85Derive]struct{}{}`) stays satisfied, the
wrap bound `m.prog > bip85Derive` stays correct, and `npage = int(bip85Derive)+1`
becomes 7 automatically. Appending after `bip85Derive` would have required
rewriting all five lockstep sites.

### D8 — QR is opt-in, shares the plate, and reflows the text smaller.

Rejected: a second QR-only plate (that means **two complete copies of the
secret** to store and protect — strictly worse), and shrinking QR modules to
0.6 mm (risks the QR not scanning, which defeats its only purpose).

---

## 3. Character set and validation

### 3.1 Measured font coverage (before this work)

Probed directly against `font/constant`'s compiled face:

```
SUPPORTED (52): space  # ' ( ) * , - . /  0-9  :  @  A-Z  [ ] { }
MISSING   (43): ! " $ % & + ; < = > ? \ ^ _ `  a-z  | ~
```

Composition: 26 uppercase, 0 lowercase, 10 digits, 15 symbols, and **space —
which is present as a blank advance**, not absent.

### 3.2 Glyphs to author: 44

- **26 lowercase** `a–z`
- **17 symbols** `! " $ % & + ; < = > ? \ ^ _ \` | ~`
- **1 visible-space mark** (see §3.3)

### 3.2.1 Confusable glyphs must be disambiguated — a hard requirement

Everything this machine engraves today carries redundancy: BIP-39 words are
wordlist-checked, and `md1`/`mk1` strings are bech32, a charset **designed** to
exclude ambiguous characters. **This plate has none.** It is case-sensitive free
text where a single misread character silently opens a different wallet, with no
checksum to catch it.

Adding lowercase creates collisions that the current uppercase-only font never
had. Two distinct classes:

**Cross-character collisions** — at minimum `l`/`1`/`I`, `0`/`O`/`o`,
`'`/`` ` ``, `;`/`:`, `,`/`.`, `8`/`B`, `5`/`S`, `2`/`Z`, `9`/`g`, `u`/`v`,
`rn`/`m`.

**Case-only collisions — the class this feature actually creates:**
`C/c O/o S/s U/u V/v W/w X/x Z/z K/k`. These letters have the *same stroke path*
in upper and lower case, distinguished **by size alone** (cap height 6 units vs
x-height ~4). D5 fixes the metrics, so they are same-shape-by-construction unless
the author deliberately differentiates them. On a case-sensitive plate with no
checksum, `C` read as `c` is a different wallet. This class needs the most
attention precisely because it is the least visible on screen.

Every pair above must be **visually distinct at engraved size** — slashed or
dotted zero, serifed `I`, flagged `1`, based `l`, and so on; the exact
conventions are the font author's choice. The enumerated pairs are checked by eye
as part of the O1 hardware inspection, not merely on screen.

### 3.3 The visible-space mark

A space is legal and common in a BIP-39 passphrase, and the existing keyboard
already emits one. On metal a space is **invisible**: one space and two spaces
look identical, and leading/trailing spaces are undetectable — yet `"hunter2 "`
is a different wallet from `"hunter2"`.

Every space in the passphrase is therefore engraved as a **visible mark**, making
runs countable and edge spaces obvious.

**The mark must NOT be placed at `0x20`.** `backup.TitleString`
(`backup/backup.go:49-61`) accepts any rune the face can decode, and space
decodes today — so plate *titles* may already contain spaces. Making `0x20`
visible would silently change the appearance of existing plate types. Instead:

- The glyph is authored at an **unused control codepoint** in `0x00–0x1F`
  (`Face.Index` is `[unicode.MaxASCII]`, so these slots exist and are free).
- `engravePassphrase` translates `' ' → <mark>` immediately before layout.
- A validated ASCII passphrase can never itself contain a control character, so
  there is no collision.

**Unstated prerequisite:** `cmd/vectorfont`'s `mapChar`
(`cmd/vectorfont/main.go:704-771`) maps single-character and named SVG ids only,
and cannot currently address a control codepoint. The generator needs a small
extension (a named id — e.g. `space_mark` — mapping to the chosen slot). The
binary font format already supports it; only the generator's name table does not.

### 3.4 Validation

A single exported entry point is the **only** path from user input to the
engraver:

```go
func ValidatePassphrase(s string) error   // typed errors, see below
func ValidateFingerprint(s string) error  // "" ok; else exactly 8 hex digits
```

`ValidatePassphrase` rejects, with a distinct error for each so the UI can
explain precisely:

| Condition | Message intent |
|---|---|
| empty | "A passphrase is required." |
| > 100 characters | "Too long for one plate (max 100)." |
| any rune > `0x7E` or < `0x20` | "This device can only engrave ASCII. BIP-39 allows other characters; we cannot engrave them, and cannot guarantee a matching seed for them." |
| ASCII the face cannot decode | **must be unreachable** after the font work — asserted, not assumed |
| ASCII not in `constantAlphabet` | **must be unreachable** after §3.5 — asserted, not assumed |

**This gate is load-bearing, and it is not the only check on the path.** Eight
are known — six charset checks and two timing checks: Every one must be satisfied or the device panics:

| # | Check | Site | Panic | When |
|---|---|---|---|---|
| 1 | `face.Decode(r)` — **metadata path only**; the passphrase's decode gate is `paddedString`'s `panic("unreachable")` | `engrave.go:1386` | `:1388` | engrave |
| 2 | `ConstantStringer` alphabet lookup | `engrave.go:1305` | `:1309` | engrave |
| 3 | uniform advance | `engrave.go:1240` | `:1241` | construction |
| 4 | alphabet in ascending codepoint order | `engrave.go:1232` | `:1233` | construction |
| 5 | every alphabet rune decodes in the face | `engrave.go:1237` | `:1238` | construction |
| 6 | each glyph is ONE continuous engrave run | `engrave.go:1180` | `:1181` | construction |
| 7 | a padded block is fully consumed before the next | `engrave.go:1076` | `:1077` | **engrave** |
| 8 | a block's actual duration ≤ its pad target | `engrave.go:1073` | `:1074` | **engrave** |

Check 2 is **independent of the face** — a binary search over the alphabet, not a
`Decode` call — so extending the font alone does **not** make the engrave path
safe (§3.5). Checks 3–5 fire at **construction**, meaning a badly-formed alphabet
takes down whichever `ConstantStringer` is being built; see §3.5.1.1 for the
ordering and atomicity constraints that follow.

Three earlier drafts of this spec understated this table — first claiming
`ValidatePassphrase` was the sole gate, then three checks, then five. All were
**false**. Check 6 (`timeConstantPath`'s "broken path") was found only when Phase
A Task 4 tried to build the alphabet for real, having survived three spec review
rounds and a plan review; §3.5.0 amends the design around it.

**Checks 7-8 fire at ENGRAVE time, not construction** — so they survive a green
test suite and crash the device mid-plate. §3.5.0's zero-run case is an instance
of check 7. Related engrave-time panics in the same routine: `delay during
spline` (`:1052-1054`), `unaligned delay` (`:1098-1100`), and `paddedString`'s
`unclamped spline` (`:1322`).

**This table lists eight KNOWN checks. It does not claim to be exhaustive.** Three
successive drafts asserted completeness and three were wrong, all in the same
direction — construction-time invariants that never fire during normal operation
and are therefore invisible until an alphabet exercises them. Treat any new panic
found in this path as expected rather than surprising, and add it here.

`ValidateFingerprint` accepts the empty string (the field is optional). Otherwise
it accepts 8 hex digits **with optional internal whitespace**, and normalises to
a canonical form: whitespace stripped, uppercased, exactly 8 characters.

**The canonical stripped form is the only stored and compared value; the 4-and-4
grouping (§4.3) is presentation only.** Stating this explicitly because a split
between stored and rendered form is exactly where an off-by-one or a
double-normalisation bug hides. Entry, storage, plate rendering and any future
comparison all agree on the canonical 8 characters.

---

## 3.5 Engraving primitives — constant-time, mandatory

A BIP-39 passphrase is secret material. `engrave.ConstantStringer` and
`engrave.ConstantQR` exist precisely so that **engraving timing does not leak
secret content**; every secret this machine engraves today goes through them.
This feature does the same. Neither `engrave.String` nor `engrave.QR` (the
content-timing-dependent variants, used today only for public data) may be used
for the passphrase or its QR.

That is a requirement with prerequisites, both of which are **in scope for this
feature**:

### 3.5.0 Multi-run glyphs — per-run quantization (AMENDMENT, 2026-08-03)

**Discovered during Phase A Task 4, after this spec reached R0 GREEN.** There is
a **fourth** construction-time panic, which §3.4's table (which claims five) and
three review rounds all missed:

```go
case engraving && !engrave:
    panic("broken path")        // engrave/engrave.go:1181, in timeConstantPath
```

`NewConstantStringer` runs `timeConstantPath` over every glyph in its alphabet,
and that routine **asserts each glyph is ONE continuous engrave run** — pen down
once, never lifted. It was invisible until now because `constantAlphabet` is
digits-plus-uppercase and contains no multi-part glyph. The passphrase alphabet
contains **13**: `:` `#` `*` (pre-existing) and `i` `j` `x` `!` `"` `$` `%` `;`
`=` `?` (added in Phase A).

**Redrawing cannot fully solve it.** A glyph collapses to one stroke *iff its
parts form a connected figure* — retracing an existing edge is free, but
travelling between disjoint parts engraves a visible connector. So:

| | Glyphs | |
|---|---|---|
| Reducible | `x` `#` `*` `$` | parts intersect; retraceable, as uppercase `X` already is |
| **Resolved** | `%` | redrawn 2026-08-03: full-height slash with the top-left box joined at its top edge; bottom box detached. **3 parts -> 2**, validated visually. Max k = 2 holds. |
| **Irreducible** | `=` `"` | parallel disjoint bars, no shared point |
| **Irreducible** | `i` `j` `!` `:` `;` `?` | the detached dot *is* the glyph — join it and `i` becomes `l` |

Eight glyphs cannot be single-stroke at any price.

#### The rule

**Each engrave RUN is padded to `runeDuration`, exactly as each glyph is today.**
`runeDuration` becomes the maximum duration of any single *run* across the
alphabet; a k-part glyph emits k padded runs and costs k units.

This reuses the existing padding machinery rather than inventing a second timing
model — it is a smaller change than padding per-glyph totals would be.

#### Required code changes (normative)

The amendment is **not** a drop-in reuse of the existing machinery. Four changes
are required, three of which are invariants:

- **(i)** `timeConstantPath` (`engrave.go:1170-1189`) returns one `constantPlan`
  **per run**; `constantRune.Info` (`:786`) becomes a slice.
- **(ii)** `newConstantStringer`'s bounds accumulation (`:1247-1250`) must cover
  **every run's** start and end, not the first start and last end —
  `startEndDist` and `center` bound every padded move (`:1294-1296`).
- **(iii)** `paddedString` must split the spline at run boundaries. The boundary
  is encoded as a `Line`-flag flip **inside a shared clamped control-point
  triple** (measured on `:`: the run-1 end triple is flagged T,T,F, the run-2
  start triple F,F,T). The existing `for range 3 { spline.Next() }` skip
  (`:1320-1324`) does **not** generalise. §3.5.1.1's "no glyph may start at
  (0,0)" now applies to **every run's** start.
- **(iv)** `dot.X` advances once per **glyph**, not per run (`:1336`). Getting
  this wrong breaks §4's "position implies index" and draws the plate wrong.

- **(v)** `maxDur` (`:1255`) must iterate the per-run slice, not a single plan.

Each run's `Delay` denominator must equal that run's flush duration exactly, or
`timeScaler` panics `unaligned delay` (`:1099`) or `scale already in effect`
(`:1077`).

**The intra-glyph, inter-run move MUST be padded to `advDur`** — the same target
as an inter-glyph move. This is normative, not an implementation detail: pad it
to anything else and the leak upgrades from a per-row *count* to the per-row
*positions* of the eight multi-run glyphs, which is materially worse than what
§3.5.0 accepts. §7(b) pins the observable property; this pins the mechanism.

**Reduce k before quantizing.** The four reducible glyphs above (`x # * $`) MUST be redrawn
as single strokes. `#` and `*` are 4 parts today, so an un-redrawn `#` would cost
**4 units**. After the redraw, max k = 2 and the worst case is bounded at `2L`.

**Budget constraint (I2).** The redraws convert pen-up moves into retraced
engraving, which is slower per unit distance and adds retraced length. Measured
`runeDuration` is **181080 ticks**, set by `8`. A single-stroke `$` must retrace
much of the S to reach its bar and could plausibly exceed that — becoming the new
`runeDuration` and inflating the cost of **all 96 glyphs**, since every run is
padded to it. **The redrawn glyphs MUST NOT become the longest single run.**
Re-measure `runeDuration` after the redraws and restate the worst case in
absolute time; `2L` is expressed in a unit the redraw can itself inflate.

#### Zero-run glyphs — the mechanism, not the outcome

**Corrected after R0 round 0 (C2). The earlier rule `k = max(runs, 1)` was
justified by a behaviour that does not exist, and implemented literally it
panics.**

`0x20` has advance 600 and an **empty spline** (`cmd/vectorfont/main.go:331-333`
sets `Index[' '] = Glyph{Advance: meta.Advance}`; Start = End = 0). It does **not**
"cost one unit today": `paddedString` would emit `Delay(0, runeDuration)`
(`engrave.go:1328`) followed by **no knots at all**, leaving `timeScaler` holding
`rem = runeDuration`, so the next `DelayMove` trips `Reset`'s `if s.rem > 0`
guard and panics **`scale already in effect`** (`:1076-1078`). The `denom == 0`
special case (`:1093-1095`) rescues a zero-*distance* move, which still emits
knots — not a zero-*knot* spline. This was measured, not reasoned.

Note the failure mode: that panic fires at **engrave** time, not construction.
Construction succeeds, §7's alphabet test passes, and the device crashes
mid-plate.

**The rule:** for a glyph with zero runs, emit a single
`DelayMove(conf, totalDur + runeDuration, pen, dot)` and **no** separate `Delay`,
so the move's own knots absorb the whole unit and the `denom == 0` path applies
correctly.

**Two assertions are required, because this path is currently unreachable and
will stay untested otherwise:**

1. A construction-time assertion that no alphabet rune has zero runs unless the
   `DelayMove` path above is taken.
2. `0x20` **must never reach the stringer.** §3.3 translates every space to the
   `0x1F` mark before layout, which is what masks this today — an invariant the
   earlier draft never named as load-bearing while simultaneously forbidding
   `0x20`'s removal from the alphabet. State it as load-bearing.

#### The accepted disclosure — stated at ROW granularity

**Corrected after R0 round 0 (C1). An earlier draft stated this per plate and was
wrong by roughly 10×; the user re-accepted against the statement below.**

The passphrase is **not** engraved as one padded call. §4 lays it out as
10-character rows through `stringColumn` (`backup/backup.go:268-276`), which
issues **one `ConstantStringer.String` call per row**. Each call is bookended by
blocks whose durations differ from the intra-row `advDur` — an unpadded `Move`
(`engrave.go:1292`), a `centerDur` opening block (`:1296-1297`), a `padDur`
closing block (`:1343`). **Rows are therefore separable in the tick stream.**

And every full row is exactly 10 characters — that is §4's "position implies
index" property, deliberately chosen. So `L_row` is **public by construction**.

What an observer actually recovers:

```
T_row = rowLen + n_row    ->    n_row = T_row - rowLen   EXACTLY, per row
```

`rowLen` is **10** in the no-QR layout (§4.1) and **20** in the QR layout (§4.2).
Do not hard-code 10: an earlier draft did, and it was wrong for one of the two
normative layouts.

- `n_row` — the number of two-run glyphs (`= " i j ! : ; ?`) in that decade —
  is disclosed **exactly**, for **every row**. A 100-character passphrase yields
  ten precise counts, not one aggregate.
- **`L` is disclosed EXACTLY** — not merely "to within the final row". The park
  position at the end of a `String` call is length-dependent
  (`mid2 := longest + shortest - 1; dot = Pt(mid2*advDist/2, baseline)`,
  `engrave.go:1340-1341`), the move *to* park is padded, but the **next**
  element's approach move is **not** (`:1292`). Its duration is therefore a
  function of the preceding row's character count — which for the final partial
  row hands over `L_last`, and with it `n_last`. This is consistent with §3.5.0's
  own note that the `String` path leaks `L` exactly.
- No side knowledge is required. The layout supplies what the earlier draft
  claimed an attacker would have to obtain independently.
- **The legend leaks one content bit.** §4.3 engraves the `<mark> = SPACE`
  legend *conditionally* — "whenever the passphrase contains a space". It is a
  large, positionally distinct, non-constant-time block, so its presence or
  absence tells a timing-only observer **whether the passphrase contains at
  least one space**.
- **With QR enabled, the QR discloses a length bracket and a charset class.**
  Module count varies (33 / 37) with byte length *and* with whether the
  passphrase falls inside QR's alphanumeric subset (§4.2). `ConstantQR` is
  constant-time *given* a version, but the version is selected from the content
  and engrave duration scales with it.

The last three items are **pre-existing** `String`/layout behaviour, not created
by per-run quantization, and none is larger in kind than what is accepted above.
They are enumerated because this section is titled "what an observer actually
recovers" and carries a standing instruction not to under-state.

**This is accepted** (user decision, 2026-08-03, against this corrected
statement). The attack still requires physical proximity to an air-gapped device
the owner controls, timing resolution fine enough to segment rows, and yields a
per-decade count over 8 of 95 characters. Rejected alternatives: engraving the
whole passphrase through a single padded call (restores aggregate-only
observability but requires reworking §4's row layout), dropping the eight glyphs
(breaks D3), and uniform k-unit padding (fully constant, doubles engraving time).

**`PaddedString` loses a stronger guarantee (I4).** `paddedString` runs exactly
`longest` slots regardless of content, repeating runes to fill
(`engrave.go:1303-1338`), which today makes a `PaddedString` call's duration
independent of the string *including its length within `[shortest, longest]`* —
the property seed plates rely on and `TestConstantWords` asserts. Under per-run,
`T = Σ k(rune at slot)` depends on content and on which runes get repeated.
**`PaddedString` MUST NOT be called with `shortest != longest` on a multi-run
alphabet.** (Phrasing it as "do not use `PaddedString`" is self-defeating —
`String` *is* `PaddedString(yield, txt, n, n)`, `engrave.go:1272-1275`.) Per this
spec's own asserted-not-assumed doctrine this needs a **guard**, not a prose
prohibition: set a `hasMultiRun` flag at construction and panic in `PaddedString`
when `shortest != longest`. (This also means "today leaks `L` exactly" is true only of
the `String` path — on the `PaddedString` path today's scheme leaks nothing.)

**Do not restate this as an aggregate.** Any future edit that reintroduces
"an observer measures only `T`" is reintroducing C1.

#### Consequences for §3.4 (applied)

§3.4's table gained rows 6-8 — `timeConstantPath` / `engrave.go:1181` /
"broken path" / construction — and drop its claim of completeness. Two earlier
drafts of that table were wrong in the same direction; the count is now six
*known* checks, and the table should say so rather than assert exhaustiveness a
third time.

### 3.5.1 A SEPARATE passphrase alphabet and `ConstantStringer` instance

```go
const constantAlphabet = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"  // engrave.go:750
```

36 characters — uppercase and digits only. Every lowercase letter and every new
symbol would panic at `engrave.go:1286` *even after the font work*, because that
check does not consult the face. **Essentially every real passphrase would crash
the machine.**

**The shared `constantAlphabet` MUST NOT be widened.** Add a second alphabet
constant and construct a **separate `ConstantStringer` instance** for the
passphrase plate.

This is not a cost optimisation — it is what keeps D5's guarantee true.
`NewConstantStringer` derives **three** values from its alphabet, not one
(`engrave.go:1224-1235`): `runeDuration`, `startEndDist`
(`ManhattanDist(bounds.Min, bounds.Max)`), and `center` (their midpoint), where
`bounds` accumulates the path start/end of **every glyph in the alphabet**. All
52 current glyphs have control-Y in `[-600, 0]`, so `bounds.Max.Y` is exactly 0
today. D5 puts descenders **below the baseline**, so a single `g`, `j`, `p`, `q`
or `y` whose stroke begins or ends at the tail pushes `bounds.Max.Y` positive —
moving `center` and `startEndDist` for **every constant-time string the machine
engraves**. Those feed the data-independent start/park positions
(`engrave.go:1274`, `:1319`) and `padDur`/`advDur`/`centerDur` (`:1277-1279`),
and `golden.CompareBSpline` compares tick timings as well as control points
(`internal/golden/golden.go:72-75`).

So widening the shared alphabet would change the goldens for **every existing
plate type** — seed, SLIP-39, codex32 — making §7's byte-identical-goldens
requirement unsatisfiable and inviting a `-update` that discards the only
evidence D5's "existing output provably unaffected" rests on. The engraved
artwork would be unchanged; the *plan and the guard* would not be. A separate
instance avoids all of it at no cost.

### 3.5.1.1 Constraints on the passphrase alphabet

`NewConstantStringer` enforces three things at **construction** time, each a
panic, each affecting whichever instance is being built:

- **Ascending codepoint order** (`engrave.go:1208-1210`, `panic("unsorted
  alphabet")`) — the rune lookup is a `sort.Find` binary search
  (`engrave.go:1282`). The visible-space mark lives at a control codepoint
  (§3.3), so it sorts **first**, not appended at the end. The alphabet is a
  single ascending string: `<mark>` then `0x20`–`0x7E`.
- **Every alphabet rune must decode in the face** (`engrave.go:1213-1215`,
  `panic("unsupported rune")`). Therefore **the glyph authoring (§3.2) and the
  alphabet definition must land in the same commit.** Defining a 96-character
  alphabet before the 44 glyphs exist panics at construction — and because
  construction happens for seed plates too if the shared constant is touched,
  a mis-staged change could brick existing plate types.
- **Uniform advance** (`engrave.go:1216-1218`, `panic("variable width font")`).
  Already satisfied: `cmd/vectorfont` emits one advance for every glyph
  (`cmd/vectorfont/main.go:425-427`) — verified, all 52 current glyphs have
  advance 600. This also underwrites §4's "position implies index" property.

**Font-authoring trap (`engrave.go:1294-1296`):** `paddedString` uses
`inf.Start != (bezier.Point{})` as its sentinel for "this glyph has a leading
move segment". Every current glyph keeps X ∈ [100, 500], so the origin is never
hit. A new glyph whose first engraved point sits exactly at the origin — very
plausible for `_` — would take the wrong branch. **No new glyph may start at
(0,0).**

### 3.5.2 `ConstantQR` must reach version 6

```go
if dim > 33 { return nil, fmt.Errorf("engrave: constant QR size too large: %d", dim) }
```

The constant-time QR path supports versions 1–4 only (dims 21/25/29/33). A
100-character passphrase needs **37 modules**, so the QR layout in §4.2 **cannot
be engraved at all** as things stand. Measured boundary: ≤78 bytes → 33 modules
(works), ≥79 bytes → 37 modules (hard error). 78 is exact — it is QR v4-L's byte
capacity.

**Reaching version 6 requires THREE changes, not one:**

| # | Site | Change |
|---|---|---|
| 1 | `ConstantQR` guard, `engrave.go:406-413` | relax `dim > 33` |
| 2 | `bitmapForQRStatic`, `engrave.go:384-401` | add dims 37, 41 |
| 3 | `constantTimeQRModules`, `engrave.go:349-365` | add module maxima for 37, 41 |

**Site 3 is the hard one and is easy to miss.** It is a hardcoded switch that
returns `0` for any unlisted dim (`:363-364`), and `0` propagates: `ConstantQR`
reads it at `:430` and rejects at `:479`, and `ConstantQRCmd.Engrave` re-reads it
at `:641` to drive the constant-time loop `for range nmod` (`:649`). Relaxing the
guard and extending the bitmap **without** site 3 leaves every version-5/6 QR
still failing.

Nor is site 3 mechanical. Its values are documented in-repo as *"maximum numbers
found through fuzzing… Add a bit more to account for outliers not yet found"*
(`:350-352`), and each simultaneously sets the failure threshold **and** the
engraving duration for every QR of that size. Too small → content-dependent
engrave-time failures; too large → every QR of that size gets slower. **The
version-5/6 maxima must be derived by extending the existing fuzz corpus
(`engrave/testdata/fuzz`), not estimated.** Fail-closed behaviour is preserved
either way: `:479` errors rather than truncating.

*(Verified: v5/v6 dims are 37/41, each takes exactly one alignment marker at
`(dim-9, dim-9)`, so the `case 25, 29, 33:` line extends cleanly; `newBitmap`
panics only above width 64, so 41 is safe.)*

Falling back to `engrave.QR` is **rejected**: it would engrave a secret with
content-dependent timing, silently dropping a property that seeds get. If the
fuzz-derivation proves impractical, the correct retreat is to **cap the
passphrase at 78 characters when QR is enabled** — a user-visible limit, not a
silent security downgrade.

---

## 4. Plate layout

Plate is 85 × 85 mm; `innerMargin` 10 mm, `outerMargin` 3 mm; stroke 0.3 mm.
The passphrase is laid out one **10-character group per row**, reusing the
existing column machinery — position implies index, and there are **no
intra-row gaps** to be confused with the visible-space mark.

**Row count is `ceil(n / rowLen)`; the em size is fixed per layout mode and does
NOT scale with passphrase length.** A 20-character passphrase occupies 2 rows at
the same 6 mm em as a 100-character one, leaving the rest of the plate blank.
Scaling type to fill the plate would make every plate a different size and defeat
the point of choosing an em for legibility. The em values below are the *maximum*
case; they are pinned as constants during implementation and asserted by the
layout tests in §7, not recomputed per engraving.

### 4.1 Without QR (default)

- **10 rows × 10 characters**, em **≈ 6 mm**
- Lowercase x-height ≈ 2.67 mm ≈ **9 stroke widths** — comparable to what
  uppercase gets at today's 4.1 mm
- Block ≈ 40 mm wide × 60 mm tall, inside ~65 mm usable

### 4.2 With QR

QR size is **variable, bounded by 37 modules at ECC-L** — it is *not* a constant.
Measured against `kortschak-qr` v0.3.2: a 100-character byte-mode passphrase
gives 37 modules (41 at M), but a passphrase drawn from QR's alphanumeric subset
gives 33, as does any byte-mode passphrase ≤78 characters. Layout must therefore
**centre the QR within a reserved 37-module envelope** rather than assume a fixed
size, and tests must not assert exactly 37.

At the worst case, 37 modules is **33.3 mm** at the standard `qrScale = 3`. Text
and QR cannot both be full size (40 + 33.3 = 73 mm > 65 mm usable), so the text
reflows:

- **5 rows × 20 characters**, em **≈ 4.5 mm**, QR beneath
- x-height ≈ 2.0 mm ≈ **6.7 stroke widths**
- Total ≈ 22.5 + 33.3 = 56 mm tall

**ECC-L is pinned** (not M): the engraved text is the authoritative copy and the
QR is convenience, so 4 fewer modules is the better trade.

**QR contents:** the passphrase **exactly as entered** — same bytes, same case,
real spaces (`0x20`, *not* the visible mark, which is a rendering device only).
Nothing else: no fingerprints, no labels, no prefix. A scanner must yield a
string that can be pasted straight into a wallet's passphrase field. The QR is
therefore **secret material in machine-readable form**, and the confirm screen
says so before the user opts in.

### 4.3 Metadata, and the space legend

Rendered at `plateSmallFontSize` (3 mm). Both fingerprint lines are **omitted
entirely when blank** — an empty label is worse than no label.

Fingerprints are engraved **grouped 4-and-4** (`A1B2 C3D4`, not `A1B2C3D4`).
Eight unbroken hex digits invite a dropped or doubled character when transcribed;
the gap halves that. The separator is a **plain space, never the visible-space
mark** — the mark means "a literal space in the passphrase", and hex is `0-9A-F`
so a gap cannot be misread as a digit. The contrast reinforces the convention
rather than muddying it. An implementer must not "helpfully" apply the mark here.

**Placement (mandatory), with a per-band line budget:**

| Band | Contents | Lines |
|---|---|---|
| Top (10 mm) | `SEED FP:` , `EXPECTED COMB FP:` | ≤ 2 |
| Bottom (10 mm) | `<mark> = SPACE` legend, `FINGERPRINTS TYPED, NOT VERIFIED` | ≤ 2 |

```
        SEED FP:  A1B2 C3D4                 <- top band
EXPECTED COMB FP:  5E6F 7A8B

        [ passphrase text block, and QR if enabled ]

        <mark> = SPACE                       <- bottom band
   FINGERPRINTS TYPED, NOT VERIFIED
```

Metadata goes in the 10 mm margin bands, **not** the usable area — matching
existing practice, where the master fingerprint and title sit in exactly those
bands (`backup/backup.go:123-130, 153-161`). The §4.1/§4.2 height budgets cover
the text block and QR **only**.

**The two-line-per-band cap is normative, not stylistic.** A band offers
`innerMargin` 10 mm − `outerMargin` 3 mm = **7 mm** of engraveable height
(`backup.go:46-47`). Three 3 mm lines need 9 mm and run off the plate edge;
existing practice places exactly one line per band and already reaches y ≈ 2.7 mm.
Two lines (6 mm) fit with margin to spare. Splitting fingerprints to the top and
legend+footer to the bottom is what makes the worst case — both fingerprints
present **and** a space in the passphrase — fit at all.

**Width:** all lines are horizontally centred. The longest,
`FINGERPRINTS TYPED, NOT VERIFIED` (32 chars × 2.0 mm advance = 64 mm), spans
x ∈ [10.5, 74.5] and clears the 10 mm corner screw-hole bands by 0.5 mm. That
margin is thin enough to pin explicitly: **no metadata line may exceed 64 mm**,
asserted by the §7 worst-case fit test.

**The legend is required whenever the passphrase contains a space**, and is
engraved using the real mark glyph so the reader is matching shapes, not
descriptions.

Without it the mark is a **private convention documented nowhere the reader will
be**. Every other element of this plate is directly human-readable. Someone
inheriting a plate reading `correct⎵horse⎵battery⎵staple` — with a mark
deliberately shaped (O3) to resemble no real character — has no way to know
whether to type a space, a hyphen, or nothing. Each guess silently opens a
different wallet, and the QR that would settle it is opt-in and off by default.
Spaces are among the most common passphrase characters, so this is the ordinary
case, not an edge case.

The footer is engraved whenever **either** fingerprint is present. Together with
the legend, these are the only instructions a future reader will ever have.

### 4.4 Geometry is provisional until validated on metal

The em sizes above are derived, not measured. Lowercase at ~2 mm x-height is new
territory for this machine, and there is precedent for the concern:
`seedhammer-engrave-33word-font-legibility` is open for exactly this reason at
3.859 mm. **A real engraved plate must be inspected by eye before this feature is
called done.** If lowercase proves illegible at 4.5 mm (the QR layout), that
layout — not the whole feature — is what gets revisited.

---

## 5. Flow

New program `engravePassphrase`, reached from the main menu (position 2 of 7).

### 5.0 The keyboard must be extended first

`PassphraseKeyboard` (`gui/passphrase_keyboard.go:18-22`) currently offers
26 + 26 + 10 + **19 symbols** + space = 82 characters. Thirteen of the 95 that
D3 promises **cannot be typed at all**: `% * < > [ \ ] ^ \` { | } ~`.

Extending the font without extending the keyboard makes the charset guarantee
hollow at the input stage — a user whose passphrase contains `~` simply cannot
back it up. The symbol pages must cover all 32 ASCII symbols (a fourth page; the
existing `ppPageCycleLabel` cycle generalises).

```
1. Passphrase entry  (required)
     - PassphraseKeyboard, masked with reveal toggle
     - live character counter (n/100)
     - refuses to advance while empty
     - keyboard MUST be extended to all 32 symbols (see 5.0)
2. Seed Fingerprint  (optional, skippable)
     - warning: typed, not verified
     - displayed grouped 4-and-4, matching the plate
3. Expected Combined Fingerprint  (optional, skippable)
     - displayed grouped 4-and-4, matching the plate
     - warning: typed, not verified; a wrong passphrase yields a
       COMPLETELY DIFFERENT WALLET, not an error
4. QR code?  (default no)
5. Confirm  — full review screen, passphrase revealed for proof-reading
6. Engrave
```

### 5.1 Warnings

Steps 2, 3 and 5 each state that nothing is verified and that the user must
double-check — **especially the passphrase**, because an incorrect passphrase
does not fail, it silently opens a different wallet.

The confirm screen (step 5) shows the passphrase **revealed**, since a masked
readout cannot be proof-read, and this is the last moment before a permanent
plate.

**Revealing the text is not sufficient, because a space is as invisible on screen
as it is on metal.** §3.3's own argument — one space and two spaces look
identical, and `"hunter2 "` is a different wallet from `"hunter2"` — applies
verbatim to the last checkpoint before permanent engraving. A 100-character
string also wraps (`widget.Labelw` MaxWidth), which hides spaces adjacent to a
line break entirely. So the confirm screen must, in addition:

- **render spaces with the visible mark**, mirroring the plate convention, so
  what the user proof-reads matches what gets engraved; and
- **display derived counts** — e.g. `100 chars · 3 spaces · 1 trailing` —
  because a count is checkable against intent in a way that a wall of characters
  is not. Leading and trailing spaces are called out by name.

Without this, a user can fat-finger a trailing space, proof-read a revealed
string, see nothing wrong, and confirm a plate that faithfully engraves the wrong
wallet.

### 5.2 Touch

**Every interactive element must be reachable by touch.** SeedHammer II has no
directional buttons; its only production input is the `ft6x36` capacitive panel
emitting `PointerEvent`s. A screen wired only to `ButtonFilter(...)` is dead on
real hardware — this exact defect shipped in the StartScreen pager and was fixed
in `86e0da9`. Bind interactive elements to `Clickable` (which routes both
`ButtonFilter(c.Button)` and `PointerFilter(c)`) and register an `op.Input` hit
area for each.

### 5.3 Secret handling

The passphrase is secret material: never logged, never written to any persistent
store, and **not** sent over NFC (consistent with the constellation rule that
secrets are hand-typed on the air-gapped device, never transmitted).

**On wiping — an honest statement of what is achievable.**
`PassphraseKeyboard.Fragment` is a Go `string` grown by concatenation
(`gui/passphrase_keyboard.go:192`). Go strings are immutable, so every keystroke
leaves an unreachable heap copy of a prefix that **cannot be wiped**. The
codebase's `wipeBytes` / `k.Zero` discipline (`gui/derive.go`) applies to
`[]byte` and key material, not to strings. A blanket claim that the passphrase is
"wiped on flow exit" would be false.

The requirement is therefore: **accumulate the passphrase in a `[]byte`** that is
explicitly wiped on flow exit and on abort, and keep string conversions to the
minimum needed for rendering and encoding. Residual copies from the existing
string-based widget are accepted only to the extent they are unavoidable, and the
implementation must state where they remain. Mitigating context, not an excuse:
RAM is volatile, the device is air-gapped, and it powers down between uses.

---

## 6. Menu wiring

```go
const (
    backupWallet program = iota
    engravePassphrase          // NEW — position 2 of 7
    engraveXpub
    engraveBundle
    engraveSingleSig
    engraveMultisig
    bip85Derive
    qaProgram
)
```

Sites to update: `layoutMainPlates`' case list, `StartScreen.draw`'s title
switch, and the flow dispatch in the `startScreenAction` handler. The guard at
`gui.go:164` fails the build if the enum ordering invariant is broken.

**To verify during implementation:** that `program` values are not persisted
anywhere, since inserting shifts the numeric value of every later program.

---

## 7. Testing

- **Font coverage:** all 95 printable ASCII decode successfully after the font
  work — the §3.1 probe promoted to a permanent regression test. Plus a golden
  for the visible-space mark.
- **Three-way charset alignment** (§3.4/§3.5/§5.0): for **every** rune
  `ValidatePassphrase` accepts, assert it is (i) decodable by the face,
  (ii) present in `constantAlphabet`, and (iii) typeable on the keyboard. Any one
  of the three drifting produces a crash or a hollow guarantee; this test is what
  keeps them in lockstep.
- **No-panic guarantee:** feed every string accepted by `ValidatePassphrase`
  through the **real plate-layout entry point** — not `engrave.String` in
  isolation — and assert no panic. Must exercise `ConstantStringer` and, when QR
  is on, `ConstantQR`. Testing the wrong entry point is what made the earlier
  draft's guarantee false.
- **QR byte-exactness** — *the highest-leverage test here.* Encode → decode the
  QR and assert **byte-identity with the passphrase as entered**, across leading,
  trailing, interior and repeated spaces and all 95 characters. Separately assert
  the engraved **text** stream contains **zero `0x20` glyph indices**. Two
  variants of the secret are in flight — the raw string (QR, confirm) and the
  mark-translated one (engraver) — and swapping them either engraves invisible
  real spaces or QR-encodes the control-codepoint mark, which a scanner hands to
  a wallet as different bytes. Silent either way.
- **Worst-case layout fit:** 100 characters **+ QR + both fingerprints + legend +
  footer** simultaneously, asserting nothing exceeds the usable area, nothing
  lands on a corner screw hole, and blocks do not overlap. Specifically assert
  the §4.3 budgets: **≤ 2 lines per margin band** and **no metadata line wider
  than 64 mm**. The partial case (text only) is what let the metadata overflow go
  unnoticed the first time, and the three-lines-in-one-band error the second.
- **Fingerprint canonicalisation:** input with and without internal spaces, mixed
  case, produces an identical stored value; the engraved form is grouped 4-and-4;
  and the separator is a plain `0x20`, **not** the visible-space mark.
- **QR size variability:** assert the layout is correct for 33-, 37- and
  41-module codes, not just the 37 worst case.
- **Validation:** table-driven over empty / 100 / 101 chars / non-ASCII / control
  characters / every printable ASCII character individually.
- **Case preservation:** assert `backup.Passphrase` output preserves case — the
  regression `SeedString`'s `ToUpper` would introduce if the types were merged.
- **Space fidelity:** leading, trailing, interior and repeated spaces each
  produce the corresponding count of visible marks, and the legend is emitted
  whenever any space is present.
- **Confirm-screen space surfacing** (§5.1): assert the revealed text renders
  marks and that the derived counts are correct, including the trailing-space
  call-out.
- **Touch:** flow navigation driven by `PointerEvent` via the `runUITouch`/`tap`
  harness in `gui/start_screen_touch_test.go` — **not** synthesized button
  events, which no production path emits.
- **Existing-output invariance:** goldens for current plate types must be
  byte-identical after the font change, and `ConstantStringer`'s `runeDuration`
  regression (§3.5.1) must be measured and reported.
- **Per-run timing (§3.5.0) — the amendment's normative behaviour, currently
  unguarded:** (a) two passphrases of equal length with equal per-row multi-run
  counts must produce **identical** `ProfileSpline` output — the weakened form of
  `TestConstantWords` (`engrave/engrave_test.go:192-215`), which asserts
  `refProf.Equal(prof)` across all BIP-39 words and is exactly the assertion
  per-run quantization weakens; (b) per-run blocks are uniform, so no *position*
  leak arises within a row; (c) the disclosure bound itself — `T_row = rowLen +
  n_row`, for **both** `rowLen = 10` (§4.1) and `rowLen = 20` (§4.2) — holds for
  constructed worst cases.
- **Zero-run path (§3.5.0):** exercise the stringer **directly** with a zero-run
  glyph — the real plate path cannot reach it, because §3.3 strips `0x20` before
  layout, which is also why §7's existing no-panic bullet does not cover it.
  Assert **both** that it does not panic **and** that the slot costs exactly
  `advDur + runeDuration`, identical to a one-run slot. No-panic alone does not
  test the C2 mechanism's normative content.

---

## 8. Out of scope

- **Computing or verifying fingerprints.** A later cycle may add an *optional*
  "verify against seed" step that derives both fingerprints and compares. The
  design must not foreclose it; it is not built now.
- **Non-ASCII passphrases.** Blocked on `bip39-passphrase-nfkd-normalization`
  and its mandatory Rust-primary check.
- **Passphrases over 100 characters.**
- **Multi-plate passphrases.**

---

## 9. Open items

| # | Item | Owning phase |
|---|---|---|
| O1 | Hardware legibility check of lowercase at 4.5 mm and 6 mm em, **including every confusable pair enumerated in §3.2.1** | before feature is called done |
| O2 | Confirm `program` enum values are not persisted | implementation |
| O3 | ~~Final visible-space mark shape~~ **RESOLVED 2026-08-03.** Low bar at `y9` with a central spike to `y6`. Replaced an earlier form that was the *identical shape* to `u` (both an open-top box, differing only in height and depth) — the worst collision available, since misreading the mark as a letter silently yields a different wallet. User reviewed the rendered glyph in running text and in the legend line and confirmed the low position: it reads as a floor mark between characters, which is where transcription actually happens. It sits visibly below the uppercase legend text as a result; accepted. | closed |
| O4 | Exact legend + footer wording, once measured at 3 mm in the margin bands | layout |
| O5 | Confirm the separate passphrase `ConstantStringer` leaves existing goldens byte-identical (§3.5.1). If the shared alphabet is ever widened instead, `runeDuration`, `startEndDist` AND `center` must all be measured, and §7 must name which goldens change and why | implementation |
| O6 | Derive version-5/6 module maxima for `constantTimeQRModules` by extending the fuzz corpus (`engrave/testdata/fuzz`) — NOT by estimation — and confirm all three §3.5.2 sites change together. If impractical, cap the passphrase at 78 chars when QR is enabled; **never** fall back to non-constant-time `engrave.QR` | implementation |

**O2 note:** the R0 review already verified this — `m.prog` is runtime-only and
`gui/saver` persists nothing program-related. Expect it to close trivially.

---

## 10. Review history

- **R0 round 0** — fable architect, `design/agent-reports/seedhammer-engrave-bip39-password-spec-R0-round0.md`.
  **NOT GREEN (2C/6I).** All eight empirical claims CONFIRMED, three with cite
  drift (fixed). Findings folded: C1 (no on-plate legend for the space mark →
  §4.3), C2 (`ConstantStringer`'s face-independent alphabet check meant the
  spec's "single load-bearing gate" was false, and every real passphrase would
  panic → §3.4 corrected to three checks, new §3.5), I1 (`ConstantQR` caps at 33
  modules, cannot engrave the specified 37 → §3.5.2), I2 (metadata omitted from
  the height budget → §4.3 placement in margin bands), I3 (confirm screen cannot
  surface spaces → §5.1), I4 (keyboard cannot type 13 promised characters →
  §5.0), I5 (no QR byte-exactness test → §7), I6 (no confusable-glyph
  requirement → §3.2.1), M1 (Go strings cannot be wiped → §5.3 rewritten),
  M2 (QR size is variable → §4.2), M3 (generator cannot address a control
  codepoint → §3.3), N1 (cite drift → fixed).
- **R0 round 1** — opus architect, `design/agent-reports/seedhammer-engrave-bip39-password-spec-R0-round1.md`.
  **NOT GREEN (0C/4I).** Both round-0 Criticals verified closed. All four new
  findings were in material the round-0 fold itself added. Folded: I1 (reaching
  QR v6 needs three sites, not one — `constantTimeQRModules` returns 0 for
  unlisted dims and that 0 propagates into both the size check and the
  constant-time loop; its maxima are fuzz-derived and must stay so → §3.5.2),
  I2 (the margin-band relocation reproduced the overflow in the other direction:
  3 lines × 3 mm = 9 mm against a 7 mm band → §4.3 splits across both bands with
  a normative ≤2-lines and ≤64 mm budget), I3 (widening the shared
  `constantAlphabet` would move `center`/`startEndDist` for *every* constant-time
  string and break existing goldens, making §7 unsatisfiable — a **separate**
  instance is now the default, not the retreat → §3.5.1), I4 (the check table was
  again incomplete: **five** checks, two firing at construction, implying the mark
  sorts first and that glyphs+alphabet land in one commit → §3.4, §3.5.1.1),
  M1 (case-only confusables → §3.2.1), N1 (cite → fixed), N2 (no glyph may start
  at the origin → §3.5.1.1). The user's fingerprint 4-and-4 grouping was folded in
  the same pass.
- **R0 round 2** — sonnet verification, `design/agent-reports/seedhammer-engrave-bip39-password-spec-R0-round2.md`.
  **GREEN (0C/0I).** All seven round-1 fixes verified against source; three
  consistency checks pass. Two nits (a one-line cite drift and this history being
  a round behind) fixed inline. **Gate closed — implementation may begin.**
