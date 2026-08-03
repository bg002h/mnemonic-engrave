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
`RIPEMD160(SHA256(master pubkey))` (`bip32/bip32.go:37`), which is lossy beyond
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
(`engrave/engrave.go:1370`). That changes the geometry of every plate the machine
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

### 3.3 The visible-space mark

A space is legal and common in a BIP-39 passphrase, and the existing keyboard
already emits one. On metal a space is **invisible**: one space and two spaces
look identical, and leading/trailing spaces are undetectable — yet `"hunter2 "`
is a different wallet from `"hunter2"`.

Every space in the passphrase is therefore engraved as a **visible mark**, making
runs countable and edge spaces obvious.

**The mark must NOT be placed at `0x20`.** `backup.TitleString`
(`backup/backup.go:41-53`) accepts any rune the face can decode, and space
decodes today — so plate *titles* may already contain spaces. Making `0x20`
visible would silently change the appearance of existing plate types. Instead:

- The glyph is authored at an **unused control codepoint** in `0x00–0x1F`
  (`Face.Index` is `[unicode.MaxASCII]`, so these slots exist and are free).
- `engravePassphrase` translates `' ' → <mark>` immediately before layout.
- A validated ASCII passphrase can never itself contain a control character, so
  there is no collision.

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

**This gate is load-bearing.** `engrave.String` **panics** on an unknown rune
(`engrave/engrave.go:1365`), so an unvalidated string reaching the engraver
crashes the device mid-flow. The last row above exists so that a font regression
surfaces as a clean refusal rather than a panic.

`ValidateFingerprint` accepts the empty string (the field is optional), otherwise
requires exactly 8 hex digits, case-insensitive, normalised to uppercase for
engraving.

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

For a 100-character passphrase the QR is **37 modules at ECC-L** (41 at M),
i.e. **33.3 mm** at the standard `qrScale = 3`. Text and QR cannot both be
full size (40 + 33.3 = 73 mm > 65 mm usable), so the text reflows:

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

### 4.3 Metadata

Both fingerprint lines are **omitted entirely when blank** — an empty label is
worse than no label. Rendered at `plateSmallFontSize` (3 mm):

```
SEED FP:          A1B2C3D4
EXPECTED COMB FP: 5E6F7A8B
FINGERPRINTS TYPED, NOT VERIFIED
```

The footer is engraved whenever **either** fingerprint is present. It is the only
warning a future reader will ever see.

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

```
1. Passphrase entry  (required)
     - PassphraseKeyboard, masked with reveal toggle
     - live character counter (n/100)
     - refuses to advance while empty
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

### 5.2 Touch

**Every interactive element must be reachable by touch.** SeedHammer II has no
directional buttons; its only production input is the `ft6x36` capacitive panel
emitting `PointerEvent`s. A screen wired only to `ButtonFilter(...)` is dead on
real hardware — this exact defect shipped in the StartScreen pager and was fixed
in `86e0da9`. Bind interactive elements to `Clickable` (which routes both
`ButtonFilter(c.Button)` and `PointerFilter(c)`) and register an `op.Input` hit
area for each.

### 5.3 Secret handling

The passphrase is secret material and follows the codebase's existing scrubbing
discipline: wiped on flow exit and on abort, never logged, never written to any
persistent store. It is **not** sent over NFC (consistent with the constellation
rule that secrets are hand-typed on the air-gapped device, never transmitted).

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

- **Font:** a coverage test asserting all 95 printable ASCII decode successfully
  after the font work — the same probe that produced §3.1, promoted to a
  permanent regression test. Plus a golden for the visible-space mark.
- **Validation:** table-driven over empty / 100 / 101 chars / non-ASCII /
  control characters / every printable ASCII character individually.
- **No-panic guarantee:** a test that feeds every string accepted by
  `ValidatePassphrase` through the engrave path and asserts it does not panic.
  This is the pairing that makes §3.4 meaningful.
- **Case preservation:** assert `backup.Passphrase` output preserves case — the
  regression that `SeedString`'s `ToUpper` would introduce if the types were
  merged.
- **Space fidelity:** leading, trailing, interior, and repeated spaces each
  produce the corresponding count of visible marks.
- **Layout:** measurement tests asserting both layouts fit within the plate for a
  100-character passphrase, with and without QR.
- **Touch:** flow navigation driven by `PointerEvent` via the `runUITouch`/`tap`
  harness added in `gui/start_screen_touch_test.go` — **not** by synthesized
  button events, which no production path emits.
- **Existing-output invariance:** goldens for current plate types must be
  byte-identical after the font change.

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
| O1 | Hardware legibility check of lowercase at 4.5 mm and 6 mm em | before feature is called done |
| O2 | Confirm `program` enum values are not persisted | implementation |
| O3 | Final visible-space mark shape (must not resemble any real glyph) | font authoring |
| O4 | Exact plate footer wording, once measured at 3 mm | layout |
