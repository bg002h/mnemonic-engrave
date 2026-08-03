# SPEC — Engrave BIP-39 Password (SeedHammer II fork)

**Status:** draft, pending R0 architect gate (0C/0I) — no implementation may begin
before GREEN. This feature is **risk-set** under `CLAUDE.md` clause (b): it
handles secret material that guards funds.

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
had. At minimum: `l`/`1`/`I`, `0`/`O`/`o`, `'`/`` ` ``, `;`/`:`, `,`/`.`,
`8`/`B`, `5`/`S`, `2`/`Z`, `9`/`g`, `u`/`v`, `rn`/`m`.

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

**This gate is load-bearing, and there are THREE independent charset checks on
the path, not one.** All three must be satisfied or the device panics mid-flow,
potentially mid-plate:

| # | Check | Site | Panic |
|---|---|---|---|
| 1 | `face.Decode(r)` | `engrave/engrave.go:1363` | `engrave.go:1365` |
| 2 | `ConstantStringer` alphabet lookup | `engrave/engrave.go:1282` | `engrave.go:1286` |
| 3 | uniform-advance assertion | `engrave/engrave.go:1217` | `engrave.go:1218` |

Check 2 is **independent of the face** — it is a binary search over
`constantAlphabet`, not a `Decode` call. Extending the font alone therefore does
**not** make the engrave path safe; see §3.5. An earlier draft of this spec
claimed `ValidatePassphrase` was the only thing between input and a panic. That
was **false**, and is corrected here.

`ValidateFingerprint` accepts the empty string (the field is optional), otherwise
requires exactly 8 hex digits, case-insensitive, normalised to uppercase for
engraving.

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

### 3.5.1 `constantAlphabet` must be extended

```go
const constantAlphabet = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ"  // engrave.go:750
```

36 characters — uppercase and digits only. Every lowercase letter and every new
symbol would panic at `engrave.go:1286` *even after the font work*, because that
check does not consult the face. **Essentially every real passphrase would crash
the machine.**

Extend it to all 95 printable ASCII plus the visible-space mark.

**Prerequisite already satisfied:** `ConstantStringer` panics on a variable-width
font (`engrave.go:1218`), but `cmd/vectorfont` emits a single uniform advance for
every glyph (`cmd/vectorfont/main.go:425-427`), so the extended alphabet stays
fixed-width by construction. This also underwrites §4's "position implies index"
property.

**Cost to be measured, not assumed:** `ConstantStringer` pads every rune to the
duration of the longest one. Widening the alphabet from 36 to 96 glyphs can only
raise that maximum, slowing *all* constant-time engraving — including seed
plates. The implementation must measure the change in `runeDuration` and report
it; if the regression is material, the fix is a **separate** `ConstantStringer`
instance for the passphrase alphabet rather than widening the shared one.

### 3.5.2 `ConstantQR` must reach version 6

```go
if dim > 33 { return nil, fmt.Errorf("engrave: constant QR size too large: %d", dim) }
```

`bitmapForQRStatic` supports versions 1–4 only (dims 21/25/29/33)
(`engrave/engrave.go:406-413`). A 100-character passphrase needs **37 modules**,
so the QR layout in §4.2 **cannot be engraved at all** by the constant-time
primitive as it stands. Measured boundary: ≤78 bytes → 33 modules (works),
≥79 bytes → 37 modules (hard error).

Extend `bitmapForQRStatic` to versions 5 and 6 (dims 37 and 41).

Falling back to `engrave.QR` is **rejected**: it would engrave a secret with
content-dependent timing, silently dropping a property that seeds get. If the
extension proves infeasible, the correct retreat is to **cap the passphrase at 78
characters when QR is enabled** — a user-visible limit, not a silent security
downgrade.

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

```
<mark> = SPACE                     <- legend, see below
SEED FP:          A1B2C3D4
EXPECTED COMB FP: 5E6F7A8B
FINGERPRINTS TYPED, NOT VERIFIED
```

**Placement (mandatory):** metadata goes in the 10 mm margin bands, **not** the
usable area — matching existing practice, where the master fingerprint and title
are placed in exactly those bands (`backup/backup.go:123-130, 153-161`).
Fingerprints and legend in the top band, footer in the bottom band, horizontally
centred to clear the corner screw holes. The §4.1/§4.2 height budgets account for
the text block and QR **only**; putting metadata in the usable area overflows the
plate at full length (60 mm text + ~9 mm metadata + `metaMargin` > 65 mm usable).

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
3. Expected Combined Fingerprint  (optional, skippable)
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
  lands on a corner screw hole, and blocks do not overlap. The partial case
  (text only) is what let the metadata overflow go unnoticed.
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
| O3 | Final visible-space mark shape (must not resemble any real glyph) | font authoring |
| O4 | Exact legend + footer wording, once measured at 3 mm in the margin bands | layout |
| O5 | Measure `ConstantStringer.runeDuration` regression from widening `constantAlphabet` 36 → 96; if material, use a separate instance rather than widening the shared one (§3.5.1) | implementation |
| O6 | Confirm `bitmapForQRStatic` extension to versions 5–6 is feasible; if not, cap the passphrase at 78 chars when QR is enabled — **never** silently fall back to non-constant-time `engrave.QR` (§3.5.2) | implementation |

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
