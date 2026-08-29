# RECON — S2 fork seam: classify + display a `Descriptor` sysw record

Read-only recon of `/scratch/code/shibboleth/seedhammer`, branch `main` @ `a5e29b4`.
Medium breadth. Prepared for S2 planning of the descriptor-input cycle
(teaching the SeedHammer II device to classify and DISPLAY a `Descriptor` sysw
record).

All citations are `file:line` against the measured tree at `a5e29b4`.

---

## 1. Record classes in the fork

**Where classes are defined.** `sysw/record.go`, not `wire.go`. `wire.go` is
the CONTAINER-level format (header, magic, section length caps) and carries
no per-record class byte at all.

`sysw/record.go:24-44`:
```go
type Class int

const (
	ClassUnknown Class = iota
	ClassMnemonic
	ClassCodex32Secret
	ClassPassphrase
	ClassFreeText
	ClassDescriptor
	ClassMDMK
	ClassAddress
	// ClassMt is one chunk of an mt1 signed-transaction set. NOT secret: the
	// record exists to be engraved in cleartext, so flash holds nothing the
	// steel will not. An UNCONFIRMED one still reads as secret through
	// MTUnconfirmed, exactly as ClassMDMK does through MDMKUnconfirmed.
	ClassMt
	// ClassTx is a tx:-prefixed raw signed transaction, for the QR engraving
	// path. Classification already proved it parses; no confirmation walk
	// exists for it. Same secrecy reasoning as ClassMt.
	ClassTx
)
```

**`ClassDescriptor` already exists as a constant** (`sysw/record.go:32`). S2
does **not** need to add the constant — it needs to add the *classification
arm* that produces it. `sysw/record.go:92-96` states this explicitly:

```go
// Classify places a record.
//
// DESCRIPTOR AND ADDRESS ARE DELIBERATELY ABSENT, matching the Rust primary:
// classifying them needs a descriptor parser and an address decoder. An
// unclassifiable record is ClassUnknown and the caller fails closed.
func Classify(record string) Class {
```

**How a class round-trips the wire: it does not.** There is no class byte on
the wire at all. `sysw/wire.go` defines only the container header shape
(`MAGIC`, `Version`, `HeaderLen`, `SaltLen`, `IVLen`, `TagLen`,
`MaxSectionLen`, etc. — `sysw/wire.go:18-90`); nothing there encodes a
per-record type tag. Records are plain LF-separated UTF-8 strings inside the
`Public`/`Secret` sections, split at `sysw/open.go:67-75`:

```go
func splitRecords(b []byte) ([]string, error) {
	if !utf8.Valid(b) {
		return nil, ErrNotUTF8
	}
	if len(b) == 0 {
		return nil, nil
	}
	return strings.Split(string(b), "\n"), nil
}
```

`Class` is a purely **runtime-derived** value, computed once at load time by
calling `sysw.Classify(r)` per record string
(`gui/sysw_session.go:109-115`, quoted in full under Q2/Q4 below). A `sysw`
record's "class in bytes" is nothing — the classifier re-derives it from the
string's syntax every time a payload is loaded. `sysw/wire.go`'s recent touch
(comment at `sysw/wire.go:39-64`) was about raising `MaxSectionLen` to match
the Rust primary's section-length cap, unrelated to record classes.

---

## 2. Classification

**`classifyConstellation`'s arms**, `sysw/classify.go:34-58`:

```go
func classifyConstellation(record string) Class {
	// Rust reaches these through seal::record::validate_record, which trims
	// first. ...
	record = strings.TrimSpace(record)

	if isStrictMnemonic(record) {
		return ClassMnemonic
	}
	if isStrictMs1(record) {
		return ClassCodex32Secret
	}
	if codex32.ValidMD(record) || codex32.ValidMK(record) {
		return ClassMDMK
	}
	// Strict, like the rest of this function: exact BCH validity, no
	// correction, consistent case (codex32's engine), and mt.ParseHeader must
	// read a header or the string is not a chunk of anything.
	if codex32.ValidMT(record) {
		if _, err := mt.ParseHeader(record); err == nil {
			return ClassMt
		}
	}
	return ClassUnknown
}
```

Four arms, in order:
1. `isStrictMnemonic(record)` (`sysw/classify.go:64-92`) — exact-case BIP-39,
   word-count in `{12,15,18,21,24}`, closest-word lookup round-tripped
   case-sensitively → `ClassMnemonic`.
2. `isStrictMs1(record)` (`sysw/classify.go:97-106`) — `ms1` HRP prefix, ≤
   `MaxEngraveableMs1Len` (90 chars, `sysw/classify.go:11-15`), and
   `codex32.New` succeeds → `ClassCodex32Secret`.
3. `codex32.ValidMD(record) || codex32.ValidMK(record)` → `ClassMDMK`.
4. `codex32.ValidMT(record)` AND `mt.ParseHeader(record)` succeeds →
   `ClassMt`.
5. Fall-through → `ClassUnknown`.

**No descriptor arm exists yet.** This is exactly S2's gap.

**Where it's called from.** `classifyConstellation` is unexported and has
exactly one call site, the tail of the exported `Classify`:
`sysw/record.go:121`, inside:

```go
// Classify places a record.
//
// DESCRIPTOR AND ADDRESS ARE DELIBERATELY ABSENT, ...
func Classify(record string) Class {
	if strings.HasPrefix(record, PassPrefix) { ... }
	if strings.HasPrefix(record, TextPrefix) { ... }
	if strings.HasPrefix(record, TxPrefix) { ... }
	return classifyConstellation(record)   // sysw/record.go:121
}
```

`Classify` itself is called from exactly one non-test site in the whole fork:
`gui/sysw_session.go:111`, inside `(*syswSession).load`:

```go
for i, r := range all {
	s.records = append(s.records, syswRecord{
		class:       sysw.Classify(r),
		body:        r,
		unconfirmed: unconfirmed[i],
	})
}
```
(`gui/sysw_session.go:109-115`)

**What happens to `ClassUnknown` records downstream: neither refused nor
displayed — they go inert.** Loading a payload never fails because of a
`ClassUnknown` record; `load` (`gui/sysw_session.go:80-116`) appends every
record regardless of its class. But `take`/`takeAll`/`has`
(`gui/sysw_session.go:118-192`) only ever match a caller-supplied `want`
class, and no call site anywhere in `gui/` ever passes `sysw.ClassUnknown` as
`want` (confirmed by grep across `gui/*.go`; the only place `ClassUnknown`
appears at all is `sysw/record.go` itself and `seal/`). So an unclassifiable
record sits in `syswSession.records` forever, never surfaced by
`syswOffer`, never counted in `syswLoadWarnings`
(`gui/sysw_load.go:261-293`, which iterates `s.records` but only emits lines
for `flagSecretInPlaintext`/`flagWeakPassphrase`, both gated by
`c.IsSecret()`, and `ClassUnknown.IsSecret()` is false —
`sysw/record.go:53-55`), and never shown raw anywhere. It is silently
unreachable rather than either refused-at-load or displayed-as-unknown. A
descriptor record today (pre-S2) is exactly this case: `ClassUnknown`, dead
weight in the session.

---

## 3. Display routing

**There is no single class→screen switch statement.** Routing happens in two
layers that S2's Descriptor path would need to bridge:

### Layer A — admission table (which *program* may consume which class)

`gui/sysw_admit.go:32-52`, the `admitted` map. `ClassDescriptor` is **already
present** here, anticipating S2:

```go
var admitted = map[syswProgram]map[sysw.Class]bool{
	progBackupWallet: {sysw.ClassMnemonic: true, sysw.ClassCodex32Secret: true},
	progPassword:     {sysw.ClassPassphrase: true},
	progText:         {sysw.ClassFreeText: true},
	progXpub:         {sysw.ClassMnemonic: true, sysw.ClassCodex32Secret: true, sysw.ClassPassphrase: true},
	progBundle:       {sysw.ClassDescriptor: true, sysw.ClassMDMK: true},
	progSingleSig:    {sysw.ClassMnemonic: true, sysw.ClassCodex32Secret: true, sysw.ClassPassphrase: true, sysw.ClassMDMK: true},
	progMultisig:     {sysw.ClassMnemonic: true, sysw.ClassCodex32Secret: true, sysw.ClassPassphrase: true, sysw.ClassDescriptor: true, sysw.ClassMDMK: true},
	progWalletPolicy: {sysw.ClassDescriptor: true, sysw.ClassMDMK: true},
	progBip85:        {sysw.ClassMnemonic: true, sysw.ClassCodex32Secret: true, sysw.ClassPassphrase: true},
	progTransaction: {sysw.ClassMt: true, sysw.ClassTx: true},
}
```
(`gui/sysw_admit.go:32-52`, `ClassDescriptor` at lines 37 and 39)

**But this entry is currently dead code.** `sysw.Classify` never returns
`ClassDescriptor` (per Q2), so no record can ever have `r.class ==
ClassDescriptor`, so these three admission cells can never fire. Confirmed
against the admission oracle test `gui/sysw_admit_oracle_test.go`: its
`syswConsumers` table (e.g. `gui/sysw_admit_oracle_test.go:66-69`, the
`wallet_policy.go` / `walletPolicyFlow` entry) names only `ClassMDMK` for
`progWalletPolicy`, and `bundleFlow` (`gui/bundle_flow.go:20-49`) offers only
`sysw.ClassMDMK` (`gui/bundle_flow.go:24`:
`syswOffer(ctx, th, sysw.ClassMDMK, "First card from where?")`), never
`sysw.ClassDescriptor`. So the admission table names the eventual target
before the classifier or any consumption call site exists.

### Layer B — object-type switch for a *scanned* value (the analogous, already-built pattern)

`gui/scan.go`'s NFC scanner classifier (a **separate, independent**
classifier from `sysw.Classify` — the "scan door") already recognizes a
descriptor and produces a `*bip380.Descriptor` directly:

`gui/scan.go:87-88`:
```go
} else if d, err := nonstandard.OutputDescriptor(buf); err == nil {
	return d, nil
```

That `any` value then reaches `engraveObjectFlow`'s type switch,
`gui/gui.go:2494-2522`:

```go
func engraveObjectFlow(ctx *Context, th *Colors, obj any) bool {
	switch scan := obj.(type) {
	case bip39.Mnemonic:
		backupWalletFlow(ctx, th, scan)
	case slip39words.Share:
		return engraveSLIP39(ctx, th, scan)
	case codex32.String:
		return engraveCodex32(ctx, th, scan)
	case *bip380.Descriptor:
		descriptorFlow(ctx, th, scan)
	case mdmkText:
		mdmkFlow(ctx, th, scan)
	case mtText:
		engraveTransactionFlowSeeded(ctx, th, string(scan))
	case freeTextScan:
		engraveTextFlowFrom(ctx, th, string(scan), srcNFC)
	case passScan:
		engravePassphraseFlowFrom(ctx, th, scan, srcNFC)
	default:
		return false
	}
	return true
}
```
(`gui/gui.go:2494-2522`; the `*bip380.Descriptor` arm is line 2502-2503)

### The representative screen: `descriptorFlow` / `DescriptorScreen`

`descriptorFlow` (`gui/gui.go:2727-2741`) is the pattern S2's sysw-sourced
Descriptor display would follow:

```go
func descriptorFlow(ctx *Context, th *Colors, desc *bip380.Descriptor) {
	ds := &DescriptorScreen{
		Descriptor: desc,
	}
	for {
		plate, ok := ds.Confirm(ctx, th)
		if !ok {
			break
		}
		completed := NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)
		if completed {
			return
		}
	}
}
```

`DescriptorScreen` (`gui/gui.go:3070-3189`) has `Confirm` (button-driven
loop: Back/Addresses/Engrave, addresses button gated by
`address.Supported(s.Descriptor)`, `gui/gui.go:3092,3098-3108`) and `Draw`
(`gui/gui.go:3146-3189`, renders Title/Type/Script rows via `richText`,
titled `"Engrave Descriptor"`). Engraving itself goes through
`validateDescriptor` (`gui/gui.go:693-741`), which builds `TEXT+QR` /
`TEXT ONLY` / `QR ONLY` `Plate` variants from `desc.Encode()` /
`desc.EncodeNoChecksum()`.

**What S2 needs is the bridge from a sysw-sourced Descriptor record body to
this existing screen** — i.e. a consumer that calls
`ctx.sysw.take(sysw.ClassDescriptor)` (or `takeAll`, for a program that may
hold more than one), feeds the returned string through
`nonstandard.OutputDescriptor([]byte(body))`, and hands the resulting
`*bip380.Descriptor` to `descriptorFlow`/`DescriptorScreen` — exactly as
`gui/scan.go:87-88` → `engraveObjectFlow` already does for the NFC-scan
route.

### How `mt` hooked its class in — the exact touch points a NEW class must hit

Cross-referencing `sysw.ClassMt`'s footprint gives the checklist. Every one
of these is a place S2's `ClassDescriptor` arm would also need to touch:

1. **Constant** — `sysw/record.go:39` (`ClassMt` in the `Class` enum; already
   done for Descriptor at line 32).
2. **Classifier arm** — `sysw/classify.go:52-56` (`codex32.ValidMT` +
   `mt.ParseHeader`); **this is the arm S2 must add**, analogous but using
   `nonstandard.OutputDescriptor` (see seal's already-built version, Q1/Q5
   below) instead of `mt.ParseHeader`.
3. **Admission table** — `gui/sysw_admit.go:51` (`progTransaction:
   {sysw.ClassMt: true, sysw.ClassTx: true}`); Descriptor's admission cells
   already exist (`gui/sysw_admit.go:37,39,45`).
4. **Human-readable name** — `txClassName`'s switch,
   `gui/transaction.go:276-296`, has a `case sysw.ClassMt:` returning
   `"mt1 chunk"` at line 292-293; the `case sysw.ClassDescriptor:` arm
   returning `"descriptor"` **already exists too**, at
   `gui/transaction.go:286-287`.
5. **Consumption call site** — `gui/transaction.go:408`:
   `mtRecs, ok := ctx.sysw.takeAll(sysw.ClassMt)`, inside
   `payloadTransactions`, which groups mt1 chunks by `chunk_set_id`
   (`gui/transaction.go:396-430`+) and turns them into `txCandidate` values a
   picker screen shows (`transactionChoiceRow`, `gui/transaction.go:392-`).
   **This is the piece with no Descriptor analogue yet** — no `gui/*.go` file
   calls `ctx.sysw.take(sysw.ClassDescriptor)` or
   `ctx.sysw.takeAll(sysw.ClassDescriptor)` anywhere (confirmed by grep; the
   only non-test, non-admission-table appearances of `ClassDescriptor` are
   the constant declaration and the `txClassName` naming arm).
6. **Oracle-test registration** — `gui/sysw_admit_oracle_test.go`'s
   `syswConsumers` table lists real call sites per program
   (e.g. `wallet_policy.go` / `walletPolicyFlow` at lines 66-69); a
   Descriptor consumer must be added there too once it exists, or
   `TestEverySyswConsumptionSiteNamesAnAdmittedClass` has nothing to check
   against it.

---

## 4. The seam test

`nonstandard/descriptor_seam_test.go` and
`nonstandard/testdata/descriptor_seam_vectors.json`.

**Does it read `sysw_class` today?** Yes, but only to **count** the column,
not to assert values — it explicitly SKIPS the assertion, naming S2 by
number. `TestDescriptorSeamSyswClass`, `nonstandard/descriptor_seam_test.go:371-391`:

```go
// TestDescriptorSeamSyswClass is S2's arm. `sysw.Classify` has no descriptor
// case today (sysw/classify.go:34 -- measured, ClassUnknown for all 39 probed
// descriptor inputs), so the column states what it will answer once §5.2's arm
// lands, and this test SKIPS with that reason rather than asserting a value
// nothing can produce. It still counts the column, so the rows cannot vanish
// while the assertion is parked.
func TestDescriptorSeamSyswClass(t *testing.T) {
	rows := loadSeamVectors(t)
	var n int
	for _, v := range rows {
		if v.SyswClass != "" {
			n++
		}
	}
	if n != wantSyswClass {
		t.Errorf("sysw_class population: %d, want %d", n, wantSyswClass)
	}
	t.Skipf("S2 (F-418): sysw.Classify has no descriptor arm yet, so the %d sysw_class "+
		"rows cannot be asserted. Un-skip when §5.2's arm lands -- importing sysw here "+
		"is why this file is package nonstandard_test.", n)
}
```

`wantSyswClass = 4` (`nonstandard/descriptor_seam_test.go:74`) — 4 rows in
the vector file currently carry `"sysw_class": "Descriptor"`
(`nonstandard/testdata/descriptor_seam_vectors.json:181,199,216,232`, on
rows `formats-happy/bluewallet-sh-fixture`,
`formats-happy/bip380-sortedmulti-multipath`,
`formats-happy/json-label-descriptor`, `promotion/01-bare-xpub`).

The JSON file's own header comment spells out the column's meaning
(`nonstandard/testdata/descriptor_seam_vectors.json:53-58`):
```
"  sysw_class     -- the device CLASSIFIER's answer (a different predicate from",
"                    the scan door, S2.3). Asserted by the Go test only once",
"                    S5.2's arm lands; until then those rows are SKIPPED with a",
"                    named reason. `sysw.Classify` returns ClassUnknown for",
"                    every descriptor today -- measured, 39 of 39.",
```

**What `device_probe` drives.** It marks rows whose input would *crash* the
Go parser/encoder rather than error cleanly, so the test harness knows which
call to skip. Handled in `TestDescriptorSeamDeviceColumn`,
`nonstandard/descriptor_seam_test.go:125-140`:

```go
switch v.DeviceProbe {
case "panic:parse":
	panicParse++
	if v.DeviceAdmits != nil {
		t.Errorf("%s: a panic:parse row must OMIT device_admits -- the predicate "+
			"cannot be evaluated, so either boolean is a false claim", v.Name)
	}
	// Deliberately NOT parsed. nonstandard/parse.go:136-149 checks only
	// len(fp) > 4 before binary.BigEndian.Uint32(fp[:]).
	continue
case "panic:encode":
	panicEncode++
case "":
default:
	t.Errorf("%s: unknown device_probe %q", v.Name, v.DeviceProbe)
}
```

`"panic:parse"` rows are **never fed** to `nonstandard.OutputDescriptor` at
all (the test `continue`s past them). `"panic:encode"` rows are parsed but
`Descriptor.Encode()` is never called on them. The JSON header documents
both (`nonstandard/testdata/descriptor_seam_vectors.json:59-64`):
```
"  device_probe   -- the row's input PANICS the device, and at which site.",
"                    \"panic:parse\" (S4.2 defect 4): the Go test must NOT feed",
"                    the input to OutputDescriptor. \"panic:encode\" (S4.2 defects",
"                    1-2): parse ACCEPTS and Descriptor.Encode() panics, so the",
"                    Go test may parse but must NOT call Encode. A panic would",
"                    CRASH the suite rather than fail it -- a false-signal shape.",
```
Populations pinned: `wantPanicParse = 1`, `wantPanicEncode = 2`
(`nonstandard/descriptor_seam_test.go:75-76`).

---

## 5. F-426's site

`bip380/bip380.go`, inside `ParseExtendedKey`.

**Version-constant declarations** — `bip380/bip380.go:433-441`:
```go
const (
	xpubVer = "0488b21e"
	zpubVer = "04b24746"
	ypubVer = "049d7cb2"
	YpubVer = "0295b43f"
	ZpubVer = "02aa7ed3"

	tpubVer = "043587cf"
)
```

**The CLASSIFICATION switch** (maps SLIP-132 versions to script types) —
`bip380/bip380.go:442-455`:
```go
version := hex.EncodeToString(xpub.Version())
var script Script
switch version {
case xpubVer, tpubVer:
	script = P2PKH
case zpubVer:
	script = P2WPKH
case YpubVer:
	script = P2SH_P2WSH
case ZpubVer:
	script = P2WSH
default:
	return 0, nil, fmt.Errorf("hdkey: unsupported version: %s", version)
}
```
`ypubVer` has **no case here** — it falls to `default` and returns
`"hdkey: unsupported version: 049d7cb2"`.

**The NORMALISATION switch** (converts SLIP-132 versions back to `xpub`) —
`bip380/bip380.go:456-462`:
```go
// Now we have a derivation path, normalize the version bytes to xpub.
switch version {
case zpubVer, ypubVer, YpubVer, ZpubVer:
	xpub.SetNet(&chaincfg.MainNetParams)
case tpubVer:
	xpub.SetNet(&chaincfg.TestNet3Params)
}
```
`ypubVer` **is** listed here (`bip380/bip380.go:458`, in the
`zpubVer, ypubVer, YpubVer, ZpubVer` case) — but this switch runs
unconditionally after the classification switch already returned an error
for `ypub`, so it is dead for that version today: `ParseExtendedKey` returns
its error at line 454 before this switch is ever reached for a `ypub` input.

This confirms the FOLLOWUPS.md F-426 claim verbatim
(`design/FOLLOWUPS.md:14713-14722`, from the mnemonic-engrave repo): `ypub`
is declared (constant exists) and would-be-normalisable (appears in the
normalisation switch's case list) but is **unclassified** — absent from the
classification switch, hitting `default` and erroring. One case-arm
(`ypubVer` → `P2SH_P2WPKH`) is what F-426 calls for.

---

## 6. Plate/QR packing

**Current layout: one plate per record string**, and it is entirely
**fork/device-side** (GUI/engraving code in `seedhammer/gui/`), not
host-side — there is no Rust/`me` counterpart for this layout decision; the
host's job (per F-423's own scope note, from
`design/FOLLOWUPS.md`) is only to produce the record strings that get
packed, not to lay out plates.

`bundlePlatePlan`, `gui/bundle_flow.go:384-402`:
```go
// bundlePlatePlan flattens the verified cards into a per-plate engrave plan, in
// card-then-plate order. Every plate carries a gathered string UNMODIFIED (I-4)
// -- md1 + mk1 alike, no re-encode. A standalone md1 card yields exactly 1 plate.
func bundlePlatePlan(cards []bundleCard) []bundlePlate {
	var plan []bundlePlate
	for ci, c := range cards {
		for pi, s := range c.strings {
			plan = append(plan, bundlePlate{
				cardIdx:    ci + 1,
				cardTotal:  len(cards),
				plateIdx:   pi + 1,
				plateTotal: len(c.strings),
				str:        s,
				label:      c.label,
				kind:       c.kind,
			})
		}
	}
	return plan
}
```
The inner `for pi, s := range c.strings` is the "one plate per string" rule
— `plateTotal == len(c.strings)`, pinned by
`TestBundlePlanSingleMD1OnePlate` (`gui/bundle_engrave_test.go:47`) and by
`TestBundlePlanVerbatim` (`gui/bundle_engrave_test.go:9-45`, asserting
`len(plan) == sum(len(c.strings))`).

The same one-string-one-plate shape appears in the single-record engraving
paths: `validateMdmk` (`gui/gui.go:2543-2593`) builds exactly one `Plate` per
chosen variant (`TEXT+QR`/`TEXT ONLY`/`QR ONLY`) from one string `s`, and
`validateDescriptor` (`gui/gui.go:693-741`) does the same for one
`*bip380.Descriptor`'s encoded string.

**What constrains packing more strings onto one plate.** `validateMdmk` /
`validateDescriptor` both go through `backup.EngraveText(params,
plate)` → `toPlate(plan, params)` (`gui/gui.go:2569-2570`,
`gui/gui.go:728-729`), where `params` is `engrave.Params`
(`engrave/engrave.go:38-44`):
```go
type Params struct {
	// The StrokeWidth measured in machine units.
	StrokeWidth int
	// A Millimeter measured in machine units.
	Millimeter int
	StepperConfig
}
```
So the fit check is driven by **plate dimensions + font/stroke-width
metrics** (via `engrave.Params.F`/`I` unit conversions,
`engrave/engrave.go:46-52`), not by QR version directly — `toPlate` returns
an error when a variant does not fit and the caller (`validateMdmk`,
`gui/gui.go:2571-2574`; `validateDescriptor`, `gui/gui.go:730-733`) simply
drops that variant from the offered choices. The QR side is a separate,
earlier constraint: `qr.Encode(s, qr.L)` (`gui/gui.go:2545`,
`gui/gui.go:695`) picks a QR **version** from the string length at a fixed
error-correction level (`L`); a longer string produces a bigger QR matrix,
which then also has to fit the plate via the same `toPlate` mm/stroke-width
check. F-423's own text (`design/FOLLOWUPS.md:14658-14675`) names the
concrete constraint the S2 batch is asked to measure rather than guess:
"the 2-stroke-width minimum feature rule (the engraving-font standing
rules)" against `engrave.Params` and the shipped font metrics, then pack
greedily "with per-string BCH integrity preserved."

**`me bundle` (host side).** Out of scope for this recon per the task's own
instruction, but noted per the ask: nothing in the fork's Go source computes
or receives a host-side plate layout — `bundleCard.strings` arrives already
split into BCH-chunked record strings (host produces the chunking via `md`/
`mk` chunk-set encoding), and the fork's `bundlePlatePlan` is the only place
that decides how those strings map onto physical plates. The layout F-423
targets is thus **fully device/fork-side**, exactly as its FOLLOWUPS scope
line states: *"Scope: fork-native GUI/engraving code (no Rust counterpart —
fork-native exemption applies)."*

---

## Appendix — file inventory touched by this recon

- `sysw/record.go` — `Class` enum, `Classify`, `IsSecret`, `DecodeBody`.
- `sysw/classify.go` — `classifyConstellation` and its strict-predicate
  helpers.
- `sysw/wire.go` — container-level constants (no per-record class).
- `sysw/open.go`, `sysw/header.go` — wire parse/split, confirming no class
  byte exists on the wire.
- `gui/sysw_admit.go` — the `admitted` (program × class) table and
  `syswFlags`.
- `gui/sysw_admit_oracle_test.go` — the consumer-site oracle that cross-checks
  `admitted` against real call sites.
- `gui/sysw_session.go` — `syswSession`, `syswRecord`, `load`, `take`,
  `takeAll`, `has`, `syswOffer`.
- `gui/sysw_source.go` — `syswSourceAccept`, `syswPassphraseFlow`.
- `gui/sysw_load.go` — `syswLoadFlow`, `syswLoadWarnings`, `syswHasFlag`.
- `gui/transaction.go` — `txClassName`, `payloadTransactions`, the `ClassMt`/
  `ClassTx` consumption pattern S2's Descriptor consumer would mirror.
- `gui/scan.go` — the independent NFC "scan door" classifier, already
  recognizing descriptors via `nonstandard.OutputDescriptor`.
- `gui/gui.go` — `engraveObjectFlow` (object-type switch),
  `descriptorFlow`/`DescriptorScreen` (the representative display screen),
  `validateDescriptor`, `validateMdmk`.
- `gui/wallet_policy.go`, `gui/bundle_flow.go` — current ClassMDMK-only
  consumers in the two programs whose admission table already lists
  `ClassDescriptor`.
- `seal/record.go` — the SEALED PAYLOAD container's own, **already-built**
  `ClassDescriptor` classifier (`Classify`, lines 194-225), using
  `nonstandard.OutputDescriptor(b)` — the likely model for `sysw`'s new arm.
- `nonstandard/descriptor_seam_test.go`,
  `nonstandard/testdata/descriptor_seam_vectors.json` — the cross-language
  seam vectors and `TestDescriptorSeamSyswClass`, S2's parked assertion.
- `bip380/bip380.go` — `ParseExtendedKey`'s version constants + normalisation
  + classification switches (F-426's site).
- `gui/bundle_flow.go`, `gui/bundle_engrave_test.go` — `bundlePlatePlan`
  (F-423's site) and its pinning tests.
- `engrave/engrave.go` — `Params` (plate dims / stroke width), the fit
  constraint `toPlate` checks against.
