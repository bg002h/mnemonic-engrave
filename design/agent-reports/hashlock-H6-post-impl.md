# H6 — independent adversarial execution review, whole diff, three trees

**Written as this agent's final action. Verbatim; nothing here has been read or filtered by the controller.**

| tree | tip | base | what was reviewed |
| --- | --- | --- | --- |
| fork `seedhammer` | `e089a53917f92c44624fa72617e0d4a67a7f7050` | main `fb0dd04` | 57 files, +7,301 / −112 |
| `me` `mnemonic-engrave` | `8cf2a7f98e5f5bfa535200b260a514cdae33f087` | master `f82b8c11` | 12 files, +1,197 / −29 |
| engrave records | `h6-records` `32a46860` | master `5e7211d0` | specs, FOLLOWUPS, acceptance |
| toolkit manual | `h6-manual` `4fc30009` | master `6cb55bb8` | `docs/manual/src/40-cli-reference/43-ms.md` |

Detached review worktrees: `/scratch/code/shibboleth/.tmp/seedhammer-h6-review` and
`/scratch/code/shibboleth/me-worktrees/h6-review`, both left **clean at their tips**
(`git status --short` empty) and every mutation reverted with `git checkout <file>`.
Go 1.26.7 at `/scratch/code/shibboleth/.toolchain/go`; `CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/h6-review-target`.

**THE ONE QUESTION, answered.** I could not construct an input — a typed phrase, a
`phrase:` record, an ms1 string or a packed payload — for which the device cuts a plate
whose text or QR fails to yield the same 32-byte preimage the host derives, nor one that
reaches a plate without the `NOT A SEED` band, nor one that reaches a seed layout. The
host↔device identity holds byte for byte over 22 (phrase, method) pairs including every
edge the brief names, and over the whole ms1 string path. **One Important remains**, and
it is a route-availability defect rather than a fidelity one: a whitespace-padded preimage
plate record that `me` packs and both classifiers call `ClassPreimage` is dropped by
`hashlockPlatesRecords`, so the door offers the Hashlock plates route and the flow then
tells the operator the payload holds nothing to cut.

---

## Findings

### I-1 — a whitespace-padded preimage plate is admitted, classified and packed, and then the Hashlock plates flow refuses it: the door offers a route it cannot take

**Spec violated:** §5.2 — *"A door row that names a route it cannot take is the F-437
defect the door exists to remove"*, and the shipped comment on
`composerCopyHashlockPlatesEmpty` (`gui/composer_hashlock_plates.go`), which asserts the
state is *"Unreachable from the door, whose predicate is the same question."* It is
reachable.

**The mechanism.** Three predicates answer the same question three ways:

| function | how it reads the record | trims? |
| --- | --- | --- |
| `sysw.Classify` → `classifyConstellation` (`sysw/classify.go`) | `record = strings.TrimSpace(record)` before `isPreimagePlateRecord` | **yes** |
| `composerPayloadPreimages` (`gui/composer_hash.go`, `Which hash?` band 2) | `codex32.New(strings.TrimSpace(r.body))` | **yes** |
| `hashlockPlatesRecords` (`gui/composer_hashlock_plates.go`, the §5.2 flow) | `codex32.New(r.body)` → `continue` on error | **no** |

`composerDoorHasPreimage` and `composerDoorCounts` are class-based, so they trim; the
flow's own list is not.

**Reproduction, end to end, with the real binaries.**

```sh
$ printf 'hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12\nms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c \n' > recs.txt
$ me sysw pack --pack-preimage --no-passphrase --in recs.txt --out ws.bin
me: WARNING — this payload carries a hashlock PREIMAGE. ...
sealing:  NOT SEALED — you passed --no-passphrase, and this payload HOLDS
      SECRET MATERIAL (record 1 (hashlock preimage plate)). ...
```

The trailing space survives `split_record_stream` into the container (`\r` does not — a
CRLF file is handled). Read back with the FIRMWARE's own `sysw.Open`:

```
payload record 0 class=11 "hash:3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12"
payload record 1 class=12 "now:31373838363936363933"
payload record 2 class=13 "ms10hashsqw46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46h2at4w46kzv2ncy60u7z9c "
```

`class=13` is `ClassPreimage`. Driving the three device predicates on that record
(temporary test in `gui/`, since removed):

```
bare            class=13 doorOffersRoute=true doorCount=1 hashlockPlatesRows=1 whichHashBand2=1
trailing space  class=13 doorOffersRoute=true doorCount=1 hashlockPlatesRows=0 whichHashBand2=1
leading space   class=13 doorOffersRoute=true doorCount=1 hashlockPlatesRows=0 whichHashBand2=1
trailing CR     class=13 doorOffersRoute=true doorCount=1 hashlockPlatesRows=0 whichHashBand2=1
```

So the door lead reads *"1 preimage or phrase record loaded."*, the fourth route
`Hashlock plates` is offered, the operator takes it, and
`composerHashlockPlatesFlow` draws *"This payload holds no preimage or phrase record to
cut."* — the screen whose own comment says it cannot be reached this way.

**Why Important and not Critical.** No plate is cut wrongly and nothing is lost: `Which
hash?` band 2 trims, so the same record is still selectable in a composition, its digest
is correct, and the plate built from it is a re-encode of X through
`codex32.EncodeMS1Preimage` and therefore canonical. What fails is one route, on a
payload the host accepted without a murmur.

**The fix is one call.** `hashlockPlatesRecords` should read
`codex32.New(strings.TrimSpace(r.body))`, matching `composerPayloadPreimages` two files
away. The phrase branch needs nothing: a padded `phrase:` record can never classify
`ClassPhrase` on either side (the prefix test is untrimmed and the hex body test rejects
the space), which I checked rather than assumed.

---

### M-1 — a kind-`0x03` single under id `hash` with a 16-byte X is `Unknown` on the host and `ClassCodex32Secret` — a SEED class — on the device

**Pre-existing at `fb0dd04`, unchanged by H6, and unreachable through `me`.** Logged
because it is the one host↔device classification divergence in the whole probe set, and
because §4.3 names this exact string as the row that must not become a class.

```
hash-kind03-16byte   host=Unknown        device=Codex32Secret   <-- DIVERGENCE
  ms10hashsqvqqqqqqqqqqqqqqqqqqqqqqqqqqmv3lqlgkn6s5c
```

Host: `preimage_plate_admissible` fails the 33-byte conjunct → falls through → refused as
`Unclassifiable(0, PreimagePlate)` (verified by running `me sysw pack`). Device:
`IsPreimagePlate` is false (`IsPreimage` needs `len(d)==33`), so `isStrictMs1`'s
`!codex32.IsPreimage(c)` is satisfied and it answers `ClassCodex32Secret`. Measured at
the **baseline** `fb0dd04` with the same input: class `2` — identical, so H6 introduced
nothing. `me` will not pack it, so it cannot arrive in a payload this constellation
wrote.

**Verdict: log, do not fix in this stage.** Narrowing `isStrictMs1` is an H0 change with
its own inertness argument and its own corpus.

---

### M-2 — the shipped `--pack-preimage` help and the two acceptance docs are right; the acceptance doc's release SHA is right too — no finding here, recorded so it is not re-checked

Checked and TRUE, each by command rather than by reading:

- `crates/me-cli/Cargo.toml:53` reads `ms-codec = "0.9"`; `Cargo.lock` resolves
  `ms-codec 0.9.0` from `registry+https://github.com/rust-lang/crates.io-index`.
- `grep -rn "patch.crates-io"` in both workspaces: **none**.
- The acceptance doc names *"mnemonic-secret `990df82` released 0.9.0"*; the brief names
  `1a4f4aa8`. `git rev-parse ms-codec-v0.9.0^{commit}` = **`990df82e2ec…`**, and
  `RELEASE_PROCESS` publishes from the tag's tree, so the acceptance doc is the accurate
  one and the brief's SHA is the pre-publish fold.
- The fork's `hashlock/testdata/hashlock-v0.8.provenance.json` pins ms `ffdb77d4` +
  sha `4f1819cd…aba21d4`; that file at `ffdb77d4` AND at `1a4f4aa8` both hash to
  `4f1819cdd0862b101afd48d0478e8f0b218f933dd3da449915fa3c5eaaba21d4`, which is what
  `hashlock/hashlock_test.go:13` asserts.

*(Numbered as a finding slot so the counts below stay legible; it costs nothing and it is
the class of claim that decays.)*

---

### N-1 — §8.1.2's body diagnoses an UPPERCASE plate as a wrong-id record and offers the 1-in-256 seed-backup sentence

```
$ echo "MS10HASHSQW46H2AT4W46H2AT4W46H2AT4W46H2AT4W46H2AT4W46H2AT4W46KZV2NCY60U7Z9C" > upper.txt
$ me sysw pack --pack-preimage --no-passphrase --in upper.txt --out /dev/null
me: record 0 (records count from 0) is a kind-0x03 preimage payload whose 4-character id is not `hash`. ... If this string is a 33-byte seed backup that happens to begin 0x03, it is not a preimage: roughly 1 in 256 of them look like this.
```

The id IS `hash`, in the other case, so the first sentence is true only under the
case-sensitive reading §4.3 chose, and the last sentence is inapplicable. The remedy the
body gives — *"Re-encode it with `ms hashlock`"* — is the right one, which is why this is
a Nit and not a Minor. Both sides agree on the refusal (device: `ClassUnknown`), which is
the property that matters.

### N-2 — `cmd/emu/emu.wasm` does not rebuild to the size the plan, spec §11.6 and implementer F all record

Rebuilt from the reviewed tip with `sh ./cmd/emu/build.sh`, Go 1.26.7, the walk file
byte-identical (`sha256 dfb9e6d5cf5521349db0c116cf7426039e7ff6c177f86e269f92105ddc9bc581`):

```
built emu.wasm (11011082 bytes)
```

against the **11,011,084 B** that §11.6 and implementer F's four runs record. Two bytes,
in an artifact that is not the firmware and not shipped. Recorded only so the next reader
does not treat that figure as a reproducible pin.

---

## Host ↔ device digest identity, end to end

`ms_codec::hashlock::{qr_text, preimage_hardened, preimage_sha256, digest}` from the
PUBLISHED 0.9.0 crate against `hashlock.{QRText, MethodLine, PreimageHardened,
PreimageSHA256, Digest}` in the fork, over the corpus's seven `qr_text` phrases plus the
edge phrases the brief names. Both harnesses wrote hex to files and the comparison is
byte-for-byte on the hex.

| input | method | QR text equal | X equal | digest equal | QR bytes |
| --- | --- | --- | --- | --- | --- |
| `correct horse battery staple` | hardened / sha256 | ✔ / ✔ | ✔ / ✔ | ✔ / ✔ | 122 / 63 |
| 100 × `0` (the cap) | hardened / sha256 | ✔ / ✔ | ✔ / ✔ | ✔ / ✔ | **194** / 135 |
| `note: it is under the mat` (a `:`) | hardened / sha256 | ✔ / ✔ | ✔ / ✔ | ✔ / ✔ | 119 / 60 |
| `hunter2 ` (trailing space) | hardened / sha256 | ✔ / ✔ | ✔ / ✔ | ✔ / ✔ | 102 / 43 |
| `one, two, three` (commas) | hardened / sha256 | ✔ / ✔ | ✔ / ✔ | ✔ / ✔ | 109 / 50 |
| ` lead` (leading space) | hardened / sha256 | ✔ / ✔ | ✔ / ✔ | ✔ / ✔ | 99 / 40 |
| `a  b` (doubled space) | hardened / sha256 | ✔ / ✔ | ✔ / ✔ | ✔ / ✔ | 98 / 39 |
| 100 chars with `:` and a doubled space | hardened / sha256 | ✔ / ✔ | ✔ / ✔ | ✔ / ✔ | **194** / 135 |
| `x` (one character) | hardened / sha256 | ✔ / ✔ | ✔ / ✔ | ✔ / ✔ | 95 / 36 |
| `abc~` (the top of the printable range) | hardened / sha256 | ✔ / ✔ | ✔ / ✔ | ✔ / ✔ | 98 / 39 |
| `" "` (a single space) | hardened / sha256 | ✔ / ✔ | ✔ / ✔ | ✔ / ✔ | 95 / 36 |

**22 pairs, 0 mismatches.** A 64-hex phrase and an ms1-shaped phrase are refused by
`validate_phrase` and `hashlock.ValidatePhrase` alike, so they never reach a plate — I
checked that both sides refuse rather than assuming it (`hostvalid` column of the same
run; the corpus rows `phrase-64-hex` and `phrase-ms1-shaped` classify `Unknown` on both
sides).

**And the same identity through a REAL payload.** `me sysw pack --pack-preimage
--no-passphrase --in` over a `hash:` record, a plate string and three `phrase:` records,
opened by the firmware's own `sysw.Open`:

| record | device class | device digest | host said |
| --- | --- | --- | --- |
| `hash:3cf5d421…` | 11 `Hash` | — | — |
| the ms1 plate | 13 `Preimage` | `9a2db2e2…821af885` | `9a2db2e2..821af885` (orphan WARNING) |
| `phrase:hardened,correct horse battery staple` | 14 `Phrase` | `3cf5d421…b70a4c12` | no warning — matches the `hash:` record |
| `phrase:hardened, correct horse battery staple` (**space after the comma**) | 14 `Phrase` | `e2e4c1ed…81c69c51` | `e2e4c1ed..81c69c51` (phrase WARNING) |
| `phrase:sha256,correct horse battery staple` | 14 `Phrase` | `b867db87…edbc96cb` | `b867db87..edbc96cb` (phrase WARNING) |

The hand-build error §8.2.3 exists for is caught, and the host's digest is the device's.
Both plate forms then lay out at production `sh2.Params()` for every one of them.

**Sealed container, §3.4's correction and §8.2.4's claim, both executed.**
`sysw.Open(sealed, "")` → `sysw: this payload is sealed and needs a passphrase`;
`sysw.Open(sealed, <the 12 words me printed>)` → `public=2 secret=4`, and **1 preimage +
3 phrase records reachable after unlocking**. §8.2.4's sentence is true of this container.

**The composer/plates flows agree with all of it on the walk** (below): the confirm modal,
the pick screen, §8.3's census row and `shComposerPathHashes()` all carry
`3cf5d421..b70a4c12`, and the stored full digest is
`3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12` = the corpus's
`derivation[0].hardened_h`.

---

## The preimage-plate string path

| input | host `classify` | device `Classify` | `IsPreimage` | `IsPreimagePlate` | verdict |
| --- | --- | --- | --- | --- | --- |
| `ms10hashsq…kzv2ncy60u7z9c` (canonical) | `Preimage` | `ClassPreimage` | true | true | admitted, cut |
| the same, UPPERCASE | `Unknown` | `ClassUnknown` | true | **false** | refused both sides |
| the same, last character flipped (bad checksum) | `Unknown` | `ClassUnknown` | — | — | refused both sides |
| kind-`0x03`, id `entr` | `Unknown` | `ClassUnknown` | true | **false** | shipped `TagKindMismatch` text |
| kind-`0x03`, id `test` | `Unknown` | `ClassUnknown` | true | **false** | §8.1.2, **one refusal, not three** |
| kind-`0x03`, id `hash`, 16-byte X | `Unknown` | `ClassCodex32Secret` | false | false | **M-1** |
| whitespace-padded canonical | `Preimage` | `ClassPreimage` | true | true | **I-1** |

`codex32.EncodeMS1Preimage` round-tripped **200/200 random X** through
`DecodeMS1Preimage(New(…))`, every output's `Split()` id `"hash"`, and it reproduces the
corpus `kind` row byte for byte. `ms hashlock --in` reads back the plate `--out` wrote
and prints the same `hash:` record (run on the real `ms` binary at `a994a99`).

**§12 item 3's own invocation and the manual's recipe were EXECUTED, not read:**

```sh
$ ms hashlock --hashlock-phrase-stdin --out preimage.txt < phrase.txt > records.txt
$ cat preimage.txt >> records.txt
$ me sysw pack --pack-preimage --no-passphrase --in records.txt --out payload.bin   # exit 0
```

`records.txt` is exactly two lines — `hash:…` from stdout, the plate from `--out` — so
the toolkit manual's block is a working command, and `--in` is required as §3.6 says
(`me sysw pack --pack-preimage --no-passphrase <ms1>` is refused by `argv_secret_guard`
with the new hashlock wording).

---

## The layouts

Measured on the reviewed tip at production `internal/sh2.Params()`, by running the code.

- **Every plate carries its band.** `l.topLines` is `HASHLOCK PREIMAGE` / `HASHLOCK
  PHRASE` unconditionally by form and `hashlockFooter` (`NOT A SEED`) is appended
  unconditionally; `backup.EngraveHashlock` is the ONLY producer and
  `composerHashlockPlateFor` its only caller, so there is no path to a hashlock plate
  that skips them. The band assertion in `TestHashlockBandsAndBodyRows` reads ENGRAVING
  knots (not text) and fails closed if a band draws nothing.
- **No ink leaves the plate.** Over four phrases × {QR, no QR} plus the string form,
  ink bounds are inside `[0, 544000]` (85 mm) on both axes; the worst case is
  `y=[23467, 516267]`.
- **The worst case fits at exactly one rung.** The 100-character hardened phrase with
  three locator rows and the v9 QR lays out (18,152 commands); the method line is **73
  characters**; growing it to 79 reds the gate with *"11 rows at 3.0mm need 427520 units
  against a budget of 416000"*.
- **A QR on the string form is refused**: `backup: a QR is legal only on the phrase form
  of a hashlock plate`.
- **The QR is below the text**: `l.envY = l.textY + l.blockH + gap`.
- **The constant-time argument holds at every raised version.** Over 800 fresh §8.6
  payloads I measured the emitted command count of `ConstantQRCmd.Engrave(..., scale 2)`
  per dimension: **exactly one distinct value per dim** — 3827 @29, 5075 @33, 6171 @37,
  7443 @41, 9313 @45, 10801 @49, 12401 @53. Content-independent.
- **The budget bounds fresh content.** 6,000 fresh §8.6 payloads (both methods, phrase
  lengths 1..100, 30 reps each) — `ConstantQR` refused **none**, at dims
  `{29:540, 33:750, 37:1200, 41:1680, 45:630, 49:1140, 53:60}`.
- **`engrave.QR` — the content-dependent one — is unreachable from any hashlock path.**
  Its only non-test call sites are `backup/backup.go:462` (seed plates) and
  `backup/freetext.go:168`. `backup/hashlock.go` uses `engrave.ConstantQR` only.
- **Every character the phrase rule admits engraves.** Over `0x20..0x7e`, at one
  character and at the 100-character cap, with and without the QR: no panic, no refusal.
  The only glyph `constant.Font` draws nothing for is `0x20` — which is exactly why
  `passphraseGlyphs` substitutes `SpaceMark` (15 commands) and why the legend row is
  drawn when the phrase holds a space. So the plate's TEXT is sufficient to recover any
  admissible phrase.

---

## Mutations — 26 applied, run and reverted

Every one applied in my own worktree, run, quoted, then `git checkout`ed; both trees end
clean at their tips.

### Fork

| # | mutation | result |
| --- | --- | --- |
| A | `EncodeMS1Preimage` mints id `entr` | **RED** — `EncodeMS1Preimage = ms10entrsq… want the corpus ms10hashsq…`; `EncodeMS1Preimage emitted id "entr" for x=…`; plus two gui rows |
| B | `IsPreimagePlate` drops the id test | **RED** — `IsPreimagePlate(ms10entrsq…) = true: a kind-0x03 payload under id "entr" was admitted`; `IsPreimagePlate admitted the UPPERCASE spelling of a plate`; `sysw`: `Classify("ms10entrsqv0qqqqqqqq"…) = 13, want ClassUnknown` |
| C | the plate's `QRText` carries the SpaceMark glyphs | **RED** — `the QR text is "hashlock v1\nmethod: sha256\nphrase: correct\x1fhorse\x1fbattery\x1fstaple"` |
| D | `hashlockPlateLocator` omits the `hash` row | **RED**, 3 tests — `locator = [], want a "hash  e7e68d52..476e1407" row: a phrase-form plate with no locator at all is the worst artifact this stage can cut` |
| E | the plates flow builds the locator BEFORE `hashlockPlatesDerive` | **RED** — `the plate's locator = ["hash  00000000..00000000"], want a "hash  f22cc3f5..81f6e837" row` |
| F | `constantTimeQRModules(53)` `1379+20` → `1378` | **RED** — `constantTimeQRModules(53) = 1378, want 1399 (1379 observed + 20 buffer). … A different number needs its own campaign.` plus both v9 goldens |
| G | `engraveModule` loses `case 2` | **RED** — `panic: unsupported module scale`, in `engrave` and in `backup`'s v9 golden |
| H | the two phrase rows offered with no phrase held | **RED** — `a payload preimage record offers [preimage string phrase + method phrase + method + QR do not cut this preimage]; the two phrase forms have no phrase to engrave` |
| I | `bundlePlateMark` suppresses the marking on `cardMK1` | **RED** — `TestComposerPreimageMarkTitleMarksMd1AndMk1ButNeverMs1: bundlePlateMark(0) = "", want "PREIMAGE REQUIRED"` (whole-`gui` run) |
| J | `MethodLine` grows six characters | **RED** — the corpus lockstep fails on every hardened row, quoting corpus vs built |
| J2 | the fit gate with a 79-character method line | **RED** — `3.0mm: fits = false (11 rows, 427520 units against 416000)` |
| K | `composerFlowExit` loses `composerScrubHashlockHeld` | **RED**, 3 tests — `the phrase survived the flow-exit defer: "correct horse battery staple"`; `composerFlowExit does not call composerScrubHashlockHeld`; and the join guard names `[composerScrubHashlockHeld]` |
| L | `hashlockPayloadRoute` ignores the record's method | **RED** — `hash = &[60 245 …], want b867db875479…edbc96cb`; `the confirm modal does not name the RECORD's method` |
| M | `ParsePhraseRecord` cuts on the LAST comma | **RED** — `phrase-containing-a-comma: Classify(…) = 0, want 14 (host's answer)` |
| M8 | delete `progWalletPolicy`'s two admission cells | **RED at SIX sites** — `composerPayloadPreimages names ClassPreimage, which §3.3.2 REFUSES to program 7`, and the same for `composerPayloadPhrases`, `hashlockPlatesRecords` (×2) and `composerDoorCounts` (×2); `22 consumption sites reconciled` |
| N | `composerHoldHashlockMaterial` assigns without the nil check | **RED** — `panic: assignment to entry in nil map` |
| O | `composerPreimagePlates` ranges `hashlockHeld` directly | **RED** — `attempt 4: plate 0 is c83b49c4, want bddf2d8a -- the order is map order` |
| P | `hashlockPlatesDerive` re-derives on every pick | **RED** — `the second pick took 11.655392ms: the KDF ran again` |
| Q | `isPreimagePlateRecord` moved AFTER `isStrictMs1` | **GREEN — and it is an EQUIVALENT MUTANT, not a survivor.** `isStrictMs1`'s last line is `err == nil && !codex32.IsPreimage(c)`, so for a 33-byte `0x03` single it is already false and the order cannot decide anything. Recorded rather than reported: the ordering is defensive, and H0's inertness conjunct is the load-bearing guard |
| R | §8.8's notice removed | **RED** — `engravePassphraseFlowFrom draws no §8.8 notice: the operator … still meets the ordinary keyboard with no offer, no mention and no reason` |
| S | §9's predicate becomes `codex32.IsPreimage` | **RED**, 3 tests — `§9's warning did not fire at the entry screen`; the passphrase twin; and the once/re-arm test |

### `me` (Rust)

| # | mutation | result |
| --- | --- | --- |
| R1 | `preimage_plate_admissible` drops the 33-byte conjunct | **RED** — `left: Err(PreimageNotAdmitted(0, Preimage))` / `right: Err(Unclassifiable(0, PreimagePlate))` |
| R2 | the id compared with `eq_ignore_ascii_case` | **RED** — the same pair, on the UPPERCASE row |
| R3 | `Class::is_secret` drops `Preimage \| Phrase` | **RED**, 4 tests — `not sealed: me: WARNING …` and `the passphrase ceremony did not run`. This is the funds-relevant one: NOT SEALED ships bearer material in cleartext |

Each reproduces implementer B's own quoted failure exactly.

### The walk — four runs, each on its own fresh port

`cmd/emu/walk_hashlock_phrase.js`, `sha256 dfb9e6d5cf5521349db0c116cf7426039e7ff6c177f86e269f92105ddc9bc581`,
byte-identical across all four; driven through playwright, fire-and-forget with
`window.__done` / `window.__walk` / `window.__err` polled. **Both mutations are mine and
neither is one of implementer F's two.**

| run | port | tree | result |
| --- | --- | --- | --- |
| 1 | 8951 | unmutated | **`ok: true`** — `displayed = censusToken = 3cf5d421..b70a4c12`, `stored = 3cf5d421caf2…b70a4c12` (= corpus `derivation[0].hardened_h`), `censusPages: 2` |
| 2 | 8952 | **MUT W1** — `hashlockPhraseRoute` no longer calls `composerHoldHashlockMaterial` | **REJECTED** |
| 3 | 8953 | **MUT W2** — `composerPreimagePlatePick`'s lead names a digest with bit 0 flipped | **REJECTED** |
| 4 | 8954 | unmutated, after `git checkout` (`emu.wasm` back to run 1's 11,011,082 B and sha `3adcc7d2…`) | **`ok: true`**, same three tokens |

Run 2, verbatim from `window.__err`:

```
waitFor("Preimage plate") timed out after 30000ms; screen reads "PlatesToCutThisengraves1plate.md1template:1plate(key-lesswalletpolicy)Eachplatetakesminutestocut.Havethatmanyblanksreadybeforeyoustart:asetisonlyabackupwhenallofitexists.md1andmk1platescarryerrorcorrection.Aplaindescriptorplatecarriesonlyitschecksum,whichfindsamistakebutcannotfixone."
```

— §2.2's retention is what makes §5.3's plate offer exist at all: without the hold, the
run walks straight past the pick step to the census. Run 3, verbatim:

```
the pick screen offers a plate for a DIFFERENT digest than the confirm modal drew, so the operator would accept a plate for a preimage they never saw.
  confirm modal: 3cf5d421..b70a4c12
  pick screen:   3df5d421..b70a4c12
```

**The harness inputs exist on the hardware.** The walk drives only `shTap` / `shPress` /
`shRelease` over `shTargets()` and the three real buttons; nothing it presses is absent
from the SH2, and it reads state only through `shComposerPathHashes()` to compare screen
against store (H5 §4.1's doctrine). No camera is assumed anywhere.

---

## Flows and Back edges, traced against the spec

| edge | spec | shipped | verdict |
| --- | --- | --- | --- |
| door → `Hashlock plates` | §5.2, a LOOP member so Back lands on the door | `walletPolicyFlow` `continue`s after `composerHashlockPlatesFlow` | ✔ |
| plates list, Back | returns to the door | `composerPickScreen !ok` → `return` | ✔ |
| plates derive countdown, Back | back to the list, nothing derived | `!hashlockPlatesDerive` → `continue` | ✔ |
| plates form pick, Back | decline → back to the list | `hashlockPlateDecline` → `continue` | ✔ |
| §8.5 QR warning declined | back to the rows, nothing decided | inner `continue` | ✔ |
| plates engrave failed | this flow's own abort, **neither §8.4 arm** | `composerCopyHashlockPlatesNotCut`, and `TestHashlockPlatesAbortDrawsNeitherSection84Arm` gates it | ✔ |
| plates empty payload | refuse rather than draw an empty picker | `composerCopyHashlockPlatesEmpty` | ✔, but see **I-1** |
| pick step (A), Button1 | `do not cut` for the highlighted plate; backing out of the STEP is NOT offered | `!ok` → `hashlockPlateDecline`; no failure return on `composerPreimagePlateStep` | ✔ (the corrected §5.3 contract) |
| census (B), Button1 | returns false from `composerEngraveStep`, state intact | `composerReadScreen` → `return false` | ✔ |
| §5.4 abort arms | 8.4a at `cut == 0`, 8.4b at `cut > 0`, both after `bundleEngrave` too | a `cut` counter, incremented only after a successful `Engrave`; `!done && cut > 0` after `bundleEngrave` | ✔ both reachable |
| free text / passphrase §9 | warn, never refuse; declining stays on the screen | `syswWarnMS1Shaped` returns false → `continue` | ✔ |
| sealed payload | refusal stands | `sysw.Open(blob, "")` → `sysw: this payload is sealed and needs a passphrase` | ✔ |

---

## Records

| claim | checked how | verdict |
| --- | --- | --- |
| the fork's `record_class_vectors.provenance.json` names B's ACTUAL commit (D's was PROVISIONAL) | `commit 8cf2a7f9…` = B's tip, `file_commit 4d00fbbf…` = the commit that carries the file (`git log -1 --format=%H 8cf2a7f9 -- …` returns `4d00fbbf`) | ✔ |
| the vendored corpus is byte-identical on both sides | `diff -q` silent; both `sha256 3575ccb0e12d…c4abaf`; 68 rows; `FIXTURE_SHA256` at `tests/sysw_composer_records.rs:399` is the same literal | ✔ |
| `hashlock-v0.8.json` sha and the fork's `corpusSHA256` | `4f1819cdd086…aba21d4`, identical to ms at `ffdb77d4` and at `1a4f4aa8` | ✔ |
| `record_corpus_pre_s2.json` moves EXACTLY one row | the diff is one `"class": "Unknown"` → `"Preimage"` on `codex32_seam/preimage-plate-0x03`; the sibling `preimage-shape-entr-id` untouched; the move is recorded in the test's own doc comment | ✔ |
| the five falsified records (§0) | 1 & 2 rewritten in the fork's comments; 3 rewritten in `SPEC_wallet_policy_composer.md` §6c and the §14 row; 5 rewritten in `SPEC_hashlock_H2_device.md` §4.5 **and** the shipped string, and the two are byte-identical when the blockquote's line breaks are joined; 4 is F-501, the controller's | ✔ (4 open by design) |
| FOLLOWUPS SHAs are real | every 7–40-hex token in the diff resolves: `5e7211d0` (engrave), `b9a9a30` and `fb0dd04` (fork) | ✔ |
| F-491's claim that the four-sentence reuse block was NEVER shipped | `git show b9a9a30:gui/composer_copy.go \| sed -n '418,424p'` prints the two-sentence form | ✔ |
| F-498's claim (`composer_flow.go:34` is stale, the literal is at `:51`) | `git grep -n "composer_flow.go:34" e089a539` → three sites; the literal is at `:51` | ✔ (note the SPEC says `:48`; `:51` is the measurement at this tip) |
| the toolkit manual carries `--pack-preimage`, the `phrase:` wire form and both plate forms | read, and its shell recipe EXECUTED end to end (above) | ✔ |
| the acceptance doc flags §12 item 8 as a GATE that fails open, with the four measured durations | read; `TestHashlockPlateCutDurationsAreBounded` is the logger | ✔ |
| §8.9's "twenty `composerCopy*` bodies added" | `composerCopyTable`'s guard moves 53 → 73 in four documented steps (+1, +13, +4, +2 = 20) | ✔ |

### Every implementer-report count I could measure

| report | claim | measured |
| --- | --- | --- |
| B | `record_class_vectors.json` 68 rows, sha `3575ccb0…c4abaf` | **68**, `3575ccb0e12d12646c45dde583380199170cff815ea5e8d86d4d37d4a1c4abaf` ✔ |
| B | 12 tests in `tests/sysw_pack_preimage.rs`, all pass | 12 run, 12 pass ✔ |
| B | mutations A, B, C, E, I each RED with the quoted line | reproduced R1/R2/R3 verbatim ✔ |
| C | `engrave` 36, `codex32` 62, `hashlock` 10, `backup` 141 tests pass | **36 / 62 / 10 / 141** ✔ |
| D | one commit, 7 files, +339 / −11 | `git diff --stat 872ba06..aeb1a07` → **7 files, 339 insertions, 11 deletions** ✔ |
| E | 27 files, 4,536 insertions, 30 deletions | `git diff --stat aeb1a070..bdb66963` → **27 files changed, 4536 insertions(+), 30 deletions(-)** ✔ |
| E | 48 gui tests added | `git diff fb0dd04..HEAD -- gui/ \| grep -c "^+func Test"` → **48** ✔ |
| F | the census is 2 pages on the walk's fixture | my run 1 and run 4: `censusPages: 2` ✔ |
| F | `emu.wasm` 11,011,084 B | **11,011,082 B** — N-2 |
| controller | fork gofmt clean on every changed file | `gofmt -l $(git diff --name-only fb0dd04..HEAD -- '*.go')` → empty ✔ |
| controller | the tree is green | `go test ./gui/ ./sysw/ ./codex32/ ./backup/ ./hashlock/ ./engrave/` at the reverted tip → **all `ok`** (gui 163.9 s) ✔ |

**No false count found.**

---

## Deviations from the spec, with a verdict each

| # | deviation | verdict |
| --- | --- | --- |
| D1 | §5.3 item 6 says *"One per path that carries held material"*; `composerPreimagePlates` deduplicates by DIGEST, so two paths sharing a digest get ONE plate, and the census row names the FIRST path only | **ACCEPT.** Cutting two identical bearer plates for one secret is worse than naming one path, the code says so in a comment, and `TestComposerPreimagePlatesCountOneSharedDigestOnce` pins it. §5.3 item 6 is about the absence of a cap, not a mandate to duplicate |
| D2 | §11.5 asks that §8.3's census ROWS be in `modal_fits_test.go`'s table; only the stand-alone notice is. The rows are in `composerCopyTable` and gated by `TestComposerCensusRowsAreDrawnInsideTheBand` | **ACCEPT.** A census row is not a modal body; `assertModalBodyFits` would measure it on the wrong renderer. The band test is the correct instrument and it is present |
| D3 | a plate BUILD/fit failure aborts the whole run (no policy plates cut) and draws §8.4a/b. §5.4's pseudo-code covers only an `Engrave` failure | **ACCEPT.** Fail-closed, and `composerCopyPreimagePlateRefusal` names the remedy (*"choose a smaller form"*) before the arm |
| D4 | `composerNotePlateCut("preimage")` fires BEFORE the engrave, so the hook records a plate that may not be cut | **ACCEPT.** The hook exists only for §5.4's ORDER assertion and is nil in production |
| D5 | the acceptance doc names ms `990df82` where the dispatch brief names `1a4f4aa8` | **ACCEPT — the doc is right.** `ms-codec-v0.9.0` tags `990df82`, and the release publishes from the tag's tree |
| D6 | `Which hash?` band 2 trims the record body; the Hashlock plates flow does not | **REJECT — this is I-1** |

---

## Closing

| severity | count |
| --- | --- |
| **Critical** | **0** |
| **Important** | **1** (I-1) |
| Minor | 1 (M-1) + 1 informational slot (M-2, no defect) |
| Nit | 2 (N-1, N-2) |

**NOT GREEN** — one Important is open. Everything else the brief asked me to break held:
no host↔device preimage divergence, no hashlock plate without the band or on a seed
layout, no preimage-kind string placed as a seed, no refusal that fails to refuse and no
warning that refuses, and no test that cannot fail on a spec-normative guarantee — 25 of
26 mutations red on the guard they name, the 26th an equivalent mutant with its reason
recorded. I-1 is one `strings.TrimSpace` in `hashlockPlatesRecords`, and it wants a row
in `composer_hashlock_plates_test.go` asserting that whatever the door counts, the flow
can list.

Two acceptance gates remain outside code review and are unaffected by the above: §12 item
8 (the 53-module scale-2 QR must be cut and scanned before the QR toggle is relied on)
and §12 items 1/3 (the phone read-back and the NFC tap). The acceptance doc states both
honestly.
