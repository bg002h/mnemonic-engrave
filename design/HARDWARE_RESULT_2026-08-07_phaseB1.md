# Hardware result — Plan B Phase B1 on the SeedHammer II, 2026-08-07

**Closes F-73.** The XIP read at the NORMATIVE `0x10E00000` is verified on real
silicon, on a part where the region actually exists.

| | |
| --- | --- |
| device | SeedHammer II, chipid `0x77c483b745abf55c`, RP2350**B** QFN80 rev A4, secure boot 1 |
| firmware | `seedhammerii-v0.0.0-g8778bb1.signed.uf2`, signed with `~/.sh2/sh2-boot-key.pem` |
| branch | `feat/encrypted-payload-phaseB1` @ `8778bb1` |
| payload | vector G public section — 12 records, UNSEALED |

---

## What was previously unverified, and why this is different

Phase A's on-silicon test ran at `0x10300000` on a **4 MB** Pico 2. `0x10E00000`
is 14 MB in, so the region **does not exist** on that board: `picotool` refuses
the load, and an XIP read there aliases to `0x10200000`. `cmd/sealread`'s
"no payload at 0x10e00000 — CLEAN state" was therefore a correct-*looking*
answer from the wrong address (continuity §3).

This run is on a 16 MB RP2350B where 14 MB is in range.

## Results

### 1. Write and read back at the normative address — no firmware involved

`picotool load --verify` of the payload, then `picotool save -r 0x10E00000
0x10E00040` read straight back off the device:

```
first 16 : 4d 4e 45 4d 42 4c 4f 42 01 00 00 00 00 00 00 00
magic    : MNEMBLOB   version=1  kdf=0  aead=0  reserved=0  iterations=0
pub_len  : 1125       ct_len=0   -> unsealed
```

`pub_len` is independently right and was not taken from the tool: vector G's 12
records are 1114 characters plus 11 LF separators = **1125**. The `kdf`, `aead`
and `iterations` fields are all zero, which is §6.2's rule for an unsealed
payload — they *must* be zero when nothing is encrypted.

**On the 4 MB Pico this step is impossible**, so it is new information rather
than a repeat.

### 2. §10.1 negative path — region erased

`picotool erase -r 0x10E00000 0x10E10000` reported "Erased 65536 bytes", exactly
`seal.RegionLen`. On boot:

- the **Sealed Payload entry was ABSENT**
- the pager showed **8 dots**
- **Engrave Bundle was in slot 5**, unmoved
- **every other program remained reachable**

The slot-5 observation is the hardware confirmation of B1's one deliberate
departure from a prior artifact. The Phase A plan carried "insert the program
*before* `bip85Derive`, not appended"; B1 appended it and moved the compile-time
guard, arguing that a mid-enum insert would force the carousel to skip an
interior index and mis-fill `layoutMainPager`'s dot. Had B1 inserted mid-enum,
every program after the insertion point would have shifted and Engrave Bundle
would have moved. It did not.

"Every other program reachable" was the regression with no prior test coverage —
Task 1 converted four compile-time constants into runtime values.

### 3. §10.1 positive path, and §6.6 across host and device

With the payload loaded:

- entry **PRESENT**, pager at **9 dots**
- **`fc10 4898 39dc 6da3 8f56 575d 45f7 655b`** — byte-identical to the host
- **"12 records"**, **UNSEALED**
- the §10.2.3 unauthenticated-payload warning appeared and required confirmation
- the plate list rendered, and the `|` separator **is visible on glass**

Both host paths produced that digest independently — `me seal` when writing the
payload, and `me hash --unsealed` re-deriving it from the records alone — so a
device mismatch would have been real signal rather than a tooling artifact.

**This is the first end-to-end confirmation that §6.6 computes identically on the
host and on RP2350 silicon.** It also proves the firmware's XIP read returns the
right 1125 bytes: an aliased or erased read cannot produce a matching digest.

### 4. Present → absent — the entry tracks region state, not a cached menu

The earlier negative path (result 2) ran before any payload had been loaded, so
it could not distinguish "reads the region correctly" from "never found anything
to show". This one erases a payload that was **known present on the previous
boot**.

Verified at the host before the device booted:

```
BEFORE : 4d 4e 45 4d 42 4c 4f 42 …   MNEMBLOB present
AFTER  : ff ff ff ff ff ff ff ff …   all 0xFF, erased
```

which is §6.1 literally — "Erased flash reads 0xFF, which is not [MNEMBLOB]".
Only the 64 KB payload region was cleared; B1's firmware was untouched.

On the next boot: **entry gone, 8 dots, Engrave Bundle still in slot 5.** So the
menu entry reflects the region on each start rather than any cached or sticky
state, and §10.1's probe-once-at-startup is doing what it claims.

## A transcription scare worth recording

The digest was first reported from the screen as `… 6da**e** …` against the
host's `… 6da**3** …` — a **one-character** difference, later confirmed as a
typo in transcription, not on the device.

It was worth stopping for, and the reasoning that resolved it is reusable:
**a genuine hash mismatch cannot differ by one character.** SHA-256 avalanches,
so different inputs disagree in about half their output bits; agreeing in 31 of
32 hex digits has probability around 2⁻¹²⁴. So a near-miss is never "almost the
same data" — it is a rendering fault or a reading fault, and those are the only
two hypotheses worth spending time on.

That mattered here because a real mismatch is exactly the §2.2 item 4 signal this
screen exists to raise: a tampered payload, or a host/device disagreement.

**Incidental gap found while chasing it:** `uiContains` (`gui/gui_test.go:516`)
compares **extracted text, not pixels**, so no test in this suite can catch a
mis-drawn glyph. That is how the missing `·` (F-78) survived — it was found by
measuring width, not by rendering. Folded into F-78.

## The RP2350B PBKDF2 rate — deliberately NOT measured here

§7.1's 9,715 it/s is from an RP2350**A** (Pico 2); this machine is a **B**. The
plan was to close that with `cmd/kdfbench` on the same trip. **Skipped, operator
decision 2026-08-07.**

`cmd/kdfbench`'s own header (`cmd/kdfbench/main.go:11-21`) says to run it on a
Pico 2 / Pico Plus 2 and **NOT** the SeedHammer II, and argues the measurement
does not need a B at all: PBKDF2 here is compute-bound with a working set of a
few hundred bytes, so it lives in cache, and the A/B differences are package, pin
count and flash banking — none of which touch that loop. Same dual Cortex-M33,
same 150 MHz.

So running it on the SH2 would overwrite B1 and require a reflash to confirm a
number that should be identical, and running it on a Pico 2 adds nothing, because
**9,715 it/s already came from a Pico 2**. Continuity §5 was carrying this as an
open measurement without noticing the tool had already argued it unnecessary.

**It is still owed before release; only the method changed.** SPEC §7.1 is
amended to confirm it **in situ during Phase B2**, by timing the real unlock KDF
on the machine — which is the stronger measurement anyway, since it covers what
the operator actually experiences (~31 s at a progress screen) in the real call
path rather than a benchmark's idealised loop.

## Nothing else is open on this hardware

All four checks in this document passed. Task 7 is complete.
