# REPORT — H6 spec author: the measurements, the rulings applied, and what would not go in

Companion to `design/SPEC_hashlock_H6_preimage_plates.md` (DRAFT, R0 pending).
Written from `design/BRAINSTORM_hashlock_H6_preimage_plates.md` under the
operator's Group A and Group B rulings of 2026-09-05, with
`design/agent-reports/hashlock-H6-brainstorm-journey-questions.md` Q1-Q14 applied
where not contradicted.

Revisions: engrave `07a92e9b`, seedhammer fork main `fb0dd04`, mnemonic-secret
`504ff46`. All three repos are untouched; nothing is committed. Measurements were
taken on a scratch copy of the fork at `/scratch/code/shibboleth/.tmp/h6-gate`
with a throwaway `gui/h6_measure_test.go`; the raw capture is
`/scratch/code/shibboleth/.tmp/h6-measure.txt`.

**Machine checks run on the spec:** 168 `file:line` citations resolved against
the three repos at the revisions above (0 out of range); 36 symbol-at-range spot
checks (0 mismatches, after 7 corrections listed in §3).

---

## §1. The measurements

Every number in the spec comes from this section. Nothing was carried over from
the brainstorm without re-measuring.

### §1.1 Modal bodies — `assertModalBodyFits`, `errorScreenBody` / `confirmWarningBody` at `sh2DisplaySize`

| body | renderer | drawn | headroom | margin 80 |
| --- | --- | --- | --- | --- |
| §8h cut-in-this-set, plain form | showError | 153 | 378 | PASS |
| §8h cut-in-this-set, phrase form | showError | 159 | 378 | PASS |
| abort: preimage arm alone | showError | 288 | 244 | PASS |
| **abort: preimage arm + seed arm (LONGEST)** | showError | **411** | **121** | PASS |
| not-on-any-path notice | showError | 107 | 455 | PASS |
| id-is-not-hash refusal | showError | 165 | 397 | PASS |
| ms1-in-free-text warning (A8) | showError | 184 | 378 | PASS |
| **ms1-in-free-text warning (A8), as shipped** | confirm | **204** | **302** | PASS |
| QR toggle warning | confirm | 126 | 378 | PASS |

**The abort arm was found by measurement, not by drafting.** Four longer wordings
were measured and rejected because headroom is a LINE budget and each cost a
line in the combined body:

| draft | drawn (with seed arm) | headroom |
| --- | --- | --- |
| "...leaving the composer or losing power destroys it, and no plate already cut can recover it..." | 488 | **39** |
| "...leaving the composer or losing power destroys it..." | 455 | **79** |
| "...The phrase is held only by this composition and dies with it..." | 428 | **79** |
| "...The phrase dies with this composition, and no plate already cut carries it..." | 441 | **79** |
| **"NO PREIMAGE PLATE WAS CUT. The phrase dies with this composition. Do not fund this wallet."** | **411** | **121** |

Three of the four sat at 79 — one character under the margin. The chosen wording
is the longest of the set that clears it.

### §1.2 Pick rows — `composerPageLines` band at `sh2DisplaySize`

Band width 411 px; every row below drew at 23 px = **one line**, including the
widest shipped control:

`hash 10  b867db87..edbc96cb`, `Type a hashlock phrase`, `Type 64 hex`,
`No hash lock`, `phrase record 1 (derive to see the digest)`,
`phrase record 10 (derive to see the digest)`,
`preimage 1  b867db87..edbc96cb`, `preimage 10  b867db87..edbc96cb`.

### §1.3 Plate geometry — `constant.Font`, `sh2.Params()`

`constant.Font`: `W` advance **600**, `Metrics{Ascent: 800, Height: 900}`. The
advance at 3.0 mm is therefore exactly **12,800 units = 2.0000 mm**.

| rung | chars/line (79 mm) | lines/plate | advance | 64 mm band holds |
| --- | --- | --- | --- | --- |
| 6.0 mm | 19 | 13 | 4.000 mm | 16 chars |
| 5.0 mm | 23 | 15 | 3.435 mm | — |
| 4.4 mm | 26 | 17 | 2.933 mm | 21 chars |
| 3.8 mm | 31 | 20 | 2.533 mm | 25 chars |
| 3.4 mm | 34 | 23 | 2.267 mm | 28 chars |
| 3.0 mm | 39 | 26 | 2.000 mm | **32 chars** |

String lengths: `HASHLOCK PREIMAGE` 17, `HASHLOCK PHRASE` 15, `NOT A SEED` 10,
`PREIMAGE REQUIRED` 17 — all inside `MaxTitleLen = 18`. The hardened method line
is **73** characters; `mk1 stub (template): 1a2b3c4d` is 29;
`hash  b867db87..edbc96cb` is 24; `path 2   hash  b867db87..edbc96cb` is **33**.

### §1.4 The vertical budget — 85 − 2 × innerMargin = **65 mm**

Worst-case bodies (header `path 2` + `hash <first8>..<last8>` +
`mk1 stub (template): <8 hex>`, blanks, the 73-character method line, a
100-character phrase; or the same header and the 75-character ms1 string):

| form | rung | text | gap + QR | total | verdict |
| --- | --- | --- | --- | --- | --- |
| phrase + QR, scale 2 | **3.0 mm** | 30.0 | 2 + 31.80 | **63.80** | **FITS, 1.20 mm spare** |
| phrase + QR, scale 2 | 3.4 mm | 37.4 | 2 + 31.80 | 71.20 | OVER |
| phrase + QR, scale 2 | 3.8 mm | 45.6 | 2 + 31.80 | 79.40 | OVER |
| phrase + QR, scale 3 | 3.0 mm | 30.0 | 2 + 47.70 | 79.70 | OVER |
| phrase, no QR | 4.4 mm | 57.2 | — | 57.2 | FITS |
| phrase, no QR | 5.0 mm | 80.0 | — | 80.0 | OVER |
| string, no QR | 6.0 mm | 60.0 | — | 60.0 | FITS |

**The tightest number in the spec is 1.20 mm.** The worst-case phrase-plus-QR
plate fits at exactly one rung and one QR scale. §11.4's fit gate must measure
the real render, and §6.5 item 3 pins the method line's length with its own
mutation: a 79th character would make it three lines at 3.0 mm and put the plate
3.0 mm over budget.

### §1.5 The QR — ECC-L byte mode, the fork's own encoder

| payload | bytes | modules | version | `ConstantQR` today |
| --- | --- | --- | --- | --- |
| §8.6 text, hardened, 100-char phrase (**worst case**) | 194 | **53** | **9** | refuses |
| §8.6 text, sha256, 100-char phrase | 135 | 45 | 7 | refuses |
| §8.6 text, hardened, 28-char anchor | 122 | 41 | 6 | refuses |
| §8.6 text, sha256, 1-char phrase | 36 | 29 | 3 | accepts |

Thresholds: ≥1 → 21 (v1), ≥18 → 25, ≥33 → 29, ≥54 → 33, ≥79 → 37 (v5, today's
cap = 106 bytes), ≥107 → 41 (v6), ≥135 → 45 (v7), ≥155 → 49 (v8), **≥193 → 53
(v9)**, ≥231 → 57 (v10).

Physical sizes at `StrokeWidth = 1920`, `Millimeter = 6400`: 53 modules is
**31.80 mm** at scale 2 and **47.70 mm** at scale 3.

### §1.6 Alignment-pattern centres — derived from the encoder's own bitmap

Found by testing the 5×5 ring shape at every candidate centre, not read from
memory or from a standard:

| version | dim | rings | centres |
| --- | --- | --- | --- |
| 2 | 25 | 1 | (18,18) |
| 3 | 29 | 1 | (22,22) |
| 4 | 33 | 1 | (26,26) |
| 5 | 37 | 1 | (30,30) |
| 6 | 41 | 1 | (34,34) |
| 7 | 45 | 6 | (22,6) (6,22) (22,22) (38,22) (22,38) (38,38) |
| 8 | 49 | 6 | (24,6) (6,24) (24,24) (42,24) (24,42) (42,42) |
| 9 | 53 | 6 | (26,6) (6,26) (26,26) (46,26) (26,46) (46,46) |

v2-v6 all sit at `(dim−9, dim−9)` once the top-left conversion (`centre − 2`) is
applied — which is the formula `bitmapForQRStatic` already carries, so **v6 costs
one entry in an existing case arm**. v7 and up need the six-ring table. §11.3
keeps this derivation as a test so the table cannot be transcribed wrong.

---

## §2. What I could not make consistent — deviations, each with its measurement

Six of these are corrections to a premise in the rulings; the spec applies the
ruling's INTENT and states the correction where it lands.

### D1 — A6's warning premise does not hold for the `sysw` container

Ruled: *"`me` WARNS when it seals such a payload that the device cannot unlock it
(the F-474 arm refuses a preimage in a sealed section)."*

Measured: `sysw.Open` (`sysw/open.go:36-73`) runs no admission at all — it
derives the key, opens the AEAD and splits records — and `syswSession.load`
(`gui/sysw_session.go:79-110`) appends `p.Public` **and** `p.Secret` into
`s.records`. A sealed `me sysw pack` payload's preimage is therefore fully
reachable after unlocking. The F-474 arm (`gui/unlock_kdf.go:415-420`) hangs off
`unlockSealedFlow` (`gui/unlock_flow.go:98`) over `seal.Payload` — the frozen
**Sealed Payload** container, whose `AdmitSection` (`seal/open.go:149`) does
refuse a preimage plate, and whose host half refuses it at pack time already
(`crates/me-cli/src/seal/record.rs:130-136`).

Applied as: §3.4 records the correction, and §8.2.4's warning says what is true
of each container and claims nothing about the other —

> me: this payload is SEALED and holds a hashlock preimage, so the device needs
> the passphrase above before it can reach it. `me seal` — the Sealed Payload
> container — refuses a preimage plate outright; this one does not.

Also applied: A6's *"the existing no-seal flag produces the cleartext payload the
Hashlock plates flow reads"* is written in the spec as a convenience rather than
a requirement, because the flow reads the session either way.

### D2 — the abort wording is false at the screen it is drawn on, and cannot name a path

Ruled: *"the phrase for path N is not on this device and is now gone; do not fund
this wallet."*

Two problems, both measured. (1) An abort inside `bundleEngrave` returns
`bundleEngraveAborted`, so `composerEngraveStep` returns false and `composerFlow`
loops back to the shape with `composerState` intact
(`gui/composer_flow.go:47-131`) — the phrase is still held, and "is now gone" is
false on the one screen whose job is to stop a funding decision. That is the
defect class H5 §1.2 removed from the confirm modal when "and this digest" made a
claim false of the one item that WAS on the plates. (2)
`bundleAbortWarningText(p bundlePlate, secret bool)`
(`gui/bundle_flow.go:780-792`) has no path context — `bundlePlate` carries
`cardIdx`, `cardTotal`, `plateIdx`, `plateTotal`, `strs`, `label`, `kind` — so
"for path N" cannot be rendered there without a signature change that would touch
four other call sites.

Applied as §8.4: *"NO PREIMAGE PLATE WAS CUT. The phrase dies with this
composition. Do not fund this wallet."* — 411 drawn / headroom 121 in the longest
variant. Same instruction, true at the moment it is read.

### D3 — "v7 = 53 modules" is v9

Ruled: *"raise the constant-time QR encoder ... (v7 = 53 modules covers 194 B at
ECC-L)."* A QR's side is `4 × version + 17`, so v7 is 45 modules and 53 is
**v9** — measured directly from the encoder (§1.5, §1.6). The 194-byte worst case
is version 9. The spec's ceiling is `dim > 53` and its alignment table runs to v9;
§7.1 states the correction so the plan does not inherit the wrong number.

### D4 — "passphrase-plate stacking" implies scale 3, which does not fit

A3 rules the QR below the text, "passphrase-plate stacking". The passphrase plate
uses `passphraseQRScale = 3` (`backup/passphrase.go:66-67`). Measured: 53 modules at
scale 3 is 47.70 mm, and the worst case is 79.70 mm against a 65 mm budget —
OVER at every rung (§1.4). The spec keeps the STACKING and takes scale **2**,
which is the free-text plate's own scale (`freeTextQRScale = 2`,
`backup/fit.go:16-19`, "0.6mm modules against the 0.9mm every other plate uses").
Stated in §6.4 and §6.5 item 1 with the measurement.

### D5 — the locator cannot be a screw-hole band row

A5 fixes the locator; decision 4 fixes the path number. Measured: the 64 mm band
ceiling holds 32 characters at the 3.0 mm rung, and
`path 2   hash  b867db87..edbc96cb` is 33 (§1.3). The spec puts the title and
`NOT A SEED` in the bands (17 and 10 characters, inside `MaxTitleLen = 18`) and
the locator in the plate BODY, which wraps at the full 79 mm. §6.3 says so with
the numbers.

### D6 — two method spellings, deliberately

My own draft-report recommendation for Q2.4 was one spelling everywhere
(decision 2's). Ruling B says the record follows the siblings. The spec takes the
ruling and states the reason rather than leaving two spellings unexplained
(§3.1): the wire carries a method SELECTOR (`hardened` / `sha256`, the spelling
`hashlockMethod.String()` and `ms hashlock --method` already use) and the plate
carries the method DEFINITION (§8.6's full line), because a wire record is read
by a tool that knows the parameter set and a plate is read by a person who may
have neither the tool nor this firmware.

### D7 — `phrase:` sits one letter from `pass:`

The new prefix is `phrase:`, beside `key:`, `hash:`, `now:`, `text:`, `pass:`,
`tx:`. It reads close to `pass:`, and confusing a hashlock phrase with a BIP-39
passphrase is the hazard ruling L2 exists to prevent. Mitigated structurally
rather than by name: §4.1 admits `ClassPhrase` at `progWalletPolicy` alone and
makes the row comment normative that it is never admitted at `progPassword`, so
the device cannot offer one where the other belongs. `hlphrase:` and
`hashphrase:` were considered and dropped as less sibling-like. Flagged for R0.

### D8 — the 1.20 mm plate margin

§1.4. Not an inconsistency, but the number a reviewer should push on: the
worst-case plate fits at one rung and one scale with 1.9 % of the budget to
spare, and three of the spec's own strings (the method line, the stub label, the
digest row) sit on that budget. §11.4 requires the gate to measure the real
render and pins the method line with a mutation.

### D9 — the host's own preimage predicate has the id blind spot too

Reported here because A7 is written as a device-side rule and it is not.
`preimage_plate` (`crates/me-cli/src/seal/record.rs:287-320`) excludes an
id/kind MISMATCH (ruling L24's `entr` over `0x03`) but otherwise tests only
`unshared && len(data) == 33 && data[0] == 0x03` — so a `0x03` single under an id
that is not in the codec's accept set at all (`UnknownTag`) is called a preimage
plate on the host today. §4.3 narrows BOTH sides with a new admission predicate
(`codex32.IsPreimagePlate`, `preimage_plate_admissible`) and leaves the
DIAGNOSTIC predicate alone, so a mistagged plate is still named correctly when it
is refused. This closes the draft report's B8 for the H6 path.

### D10 — declined findings, recorded

- **B5 of the draft report** (give the string form a QR) is DECLINED by A2. Noted
  in §6.4: the string-form plate has no machine-readable copy by design. The
  measurement that motivated it stands — a 75-byte ms1 string is 33 modules,
  already inside today's `ConstantQR` bound — so the decline costs nothing to
  reverse later.
- **Journey Q3(b)/(c)** (keep the plate a `bundleCard`) is DECLINED by ruling B's
  "its own layout function": the plate bypasses `validateMdmkStrings`, which
  would otherwise offer `TEXT + QR` / `QR ONLY` with a QR of the ms1 string
  (`gui/gui.go:2634-2643`) — the thing decision 1 forbids.
- **Journey Q4(c)** (delete stale retained material) is DECLINED: §2.2 item 2
  keeps `phraseDigests`' no-deletion rule and REPORTS instead.

---

## §3. Corrections the machine checks caught in my own draft

Seven citations drifted between the brainstorm and `fb0dd04` or were wrong when
written; all were found by the range/symbol checker, not by reading:

| cited (WRONG — do not follow) | actual |
| --- | --- |
| `gui/composer_copy.go:461-466` (§8h phrase form) | `:520-525` |
| `gui/composer_copy.go:467-471` (the chooser) | `:527-532` |
| `gui/composer_copy.go:~488` (reconcile body) | `:484-490` |
| `gui/passphrase_keyboard.go:76` | `:80` |
| `gui/composer_stub.go:62-72` | `:56-72` |
| `engrave/engrave.go:378-380` | `:349` (`constantTimeQRModules`), `:377-379` (`constantTimeStartEnd`) |
| `sysw/record.go:111-140` | `:111-139` (the file is 139 lines) |
| `walk_hashlock_phrase.js:318` (H5's needle line) | `cmd/emu/walk_hashlock_phrase.js:458` |

The last one is worth naming separately: it was inherited from H5 §1.1, which
cited `:318` at `b9a9a30`. The needle string is unchanged; the line moved. A
citation copied forward from a prior spec is a citation nobody re-grepped.

---

## §4. Where the rulings landed

§15 of the spec carries the full map. In brief: A1 → §7 (with D3's correction);
A2 → §6.4; A3 → §6.4-§6.5 (with D4's scale change); A4 → §6.2; A5 → §6.3 (with
D5's band-to-body move); A6 → §3.4 (with D1's correction); A7 → §4.3 and §8.1.2
(and D9); A8 → §9 (fits as ruled, 204/302, no wording change). Group B: cut order
and abort → §5.4, §8.4 (with D2's wording); own layout and census → §6.1, §5.3;
retention → §2.2; unused material → §5.3.4; lazy derivation → §5.1; review →
§5.3; marking → §10.3; provenance-aware copy → §10.1-§10.2; one-program admission
→ §4.1; QR warning → §8.5; `SpaceMark` → §6.1; wire shape → §3.1; corpus → §11.2.
Journey Q1-Q14 map in §15's last row; Q13 (masking a phrase on screen) is §5.3
item 7.

## §5. What R0 should push on first

1. **The 1.20 mm plate margin** (§1.4, D8) and whether the header should lose a
   row to buy headroom.
2. **The `engrave` raise** (§7): the constant-time argument for v7-v9 is a
   normative deliverable with no prior art in this tree, and §11.3's "same move
   count for two payloads of one version" is the only thing that makes it a
   property rather than a hope.
3. **D1 and D2** — both are places where an operator-supplied sentence was false
   of the mechanism, and both should be re-read against the code rather than
   against this report.
4. **Whether `phrase:` should be renamed** (D7).
