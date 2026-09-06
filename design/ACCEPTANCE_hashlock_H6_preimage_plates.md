# ACCEPTANCE — Hashlock H6: preimage plates

**Written 2026-09-06 by H6 Task 13 Step 4** (`IMPLEMENTATION_PLAN_hashlock_H6_preimage_plates.md`).
This is a TRANSCRIPTION of `SPEC_hashlock_H6_preimage_plates.md` §12 into the
form the operator walks, plus one thing the spec's list does not carry: which
items a machine has already proved, and which need a hand, a plate, a phone or
the flashed device. The spec keeps its authority as the statement of *what must
be true*; this page says *who can prove it and how*.

**Nothing here is closed by writing it down.** Every row below is UNWALKED until
the operator walks it on the flashed device. What the tree already proves is
recorded in the "already proved" column so that budget goes to the rows nothing
can prove without hardware.

**The SH2 has no camera.** Nothing on the device can read a plate it cut, so
item 1's read-back is a phone and the host, and no acceptance may be written
that assumes otherwise (spec §12, closing paragraph).

---

## The twelve items

| # | what must be true | already proved, and by what | still needs |
| --- | --- | --- | --- |
| **0** | **`ms-codec` 0.9.0 is PUBLISHED and `me` depends on it** — the phrase rule, `looks_like_ms1` and §8.6's `qr_text` are in the codec, `ms-cli` delegates with its refusal sentences unchanged, the CHANGELOG records the corpus sha, `crates/me-cli/Cargo.toml` reads `ms-codec = "0.9"`, and no `[patch.crates-io]` remains in either workspace | mnemonic-secret `990df82` released 0.9.0 to crates.io; H6 Task 2 Step 0 bumps `me` | a grep for `[patch.crates-io]` in both workspaces at the merged tips, before the fork is flashed |
| **1** | the operator types the anchor phrase, reaches Done, accepts a preimage plate in the phrase form with a QR, and the plate is cut; a **phone scan** of the QR returns §8.6's text byte for byte, and `ms hashlock --hashlock-phrase-stdin` over the phrase on that plate reproduces the digest the composer showed | the screens up to the cut: the H6 walk arm (`cmd/emu/walk_hashlock_phrase.js`, Task 12) accepts a plate at §5.3's pick screen and asserts §8.3's census row carries the same `first8..last8` the confirm modal drew. The plate BYTES: §11.4's goldens (`backup/testdata/hashlock-*.bin`) | **the machine, a blank, and a phone.** No emulator can prove a phone reads steel |
| **2** | the same composition's md1 plates carry `PREIMAGE REQUIRED` | §10.3's mark, gated in `gui` (Task 9); the mutation that suppressed it reds the mk1 key-card row | a cut plate, read by eye |
| **3** | `ms hashlock --out X.txt` **plus its `hash:` record**, then `me sysw pack --pack-preimage --no-passphrase --in <records file>`, a tap, the Hashlock plates flow, and a cut plate whose ms1 string round-trips through `ms hashlock --in`. `--in` is REQUIRED, not stylistic (§3.6) | the host half: `tests/sysw_pack_preimage.rs` (Task 3); the device half: `gui/composer_hashlock_plates_test.go` (Task 10) | **NFC hardware and the flashed device.** The tap is the one step no test in either repo performs |
| **3a** | a device-derived preimage cut in the **STRING** form — the default (decision 1) on the composer-native path: type the anchor phrase, reach Done, pick `preimage string`, and the cut plate's 75 characters round-trip through `ms hashlock --in` to the digest the composer showed | the encoder (`codex32.EncodeMS1Preimage`, Task 5) and the layout (Task 6); the H6 walk arm ACCEPTS exactly this form and the census reports it as `preimage string` | a cut plate, and `ms hashlock --in` on the host |
| **4** | a `phrase:` record in the same payload appears on `Which hash?` as `phrase record 1 (derive to see the digest)`, derives on pick, and its digest matches the host's | Task 8b's band 2 and Task 10's derive-on-pick, with the once-guard mutation ("the second pick took 11.6 ms: the KDF ran again") | a payload actually carrying one — **and nothing in the constellation emits a `phrase:` record yet (F-495)**, so this item's input is hand-assembled hex until that lands |
| **5** | a kind-`0x03` string under an id outside `{entr, hash}` is refused on the host with §8.1.2, **one refusal and not three**; the same payload under `entr` is refused with the shipped `TagKindMismatch` text; both are inert on the device | `tests/sysw_pack_preimage.rs` (Task 3) and the device classifier (Task 7) | nothing beyond a re-run at the merged tip |
| **5a** | a payload holding a preimage and no `hash:` record draws §8.2.3's NOTE, and a payload holding `hash:` records none of which match draws its WARNING | Task 3's warning rows | a payload of each shape, on the device |
| **6** | a preimage plate presented to any seed flow is still refused (H0's walk) | `cmd/emu/walk_h0_preimage.js`, shipped since H0; `codex32.IsPreimage` unchanged by this stage | a re-run of H0's walk at the H6 tip |
| **7** | a payload holding only a `phrase:` record, opened at the **Password program**, draws §8.8's notice | Task 11 | the device. **This is the one refusal in the stage whose correctness is otherwise unobservable from outside the code** |
| **8** | **THE QR IS A GATE** — see the section below | nothing. §11.3's gates are all bytes and toolpath | **a test plate and a phone scan, before the QR toggle ships** |

---

## Item 8 is a GATE, not an assumption

**It is the only row on this page that can stop a feature shipping, and it fails
OPEN by design** — which is exactly why its cost decides whether it is run at
all, and why the cost is stated here in full rather than left to be discovered.

**What must be cut:** one test plate at the WORST case — a 100-character
hardened phrase, v9, **53 modules, scale 2, 0.6 mm modules** — and scanned with
a phone.

**Why it cannot be assumed:** 53 modules at scale 2 has no precedent in this
tree on either axis. Every gate §11.3 carries is bytes and toolpath; none of
them is an optical claim, and no test in either repo can make one.

**The verdict if it fails:** the QR toggle **does not ship** and the phrase form
is text-only. The plate is still complete, because §6.4 makes the TEXT
authoritative — the QR is a convenience over a plate that already carries
everything.

**BUDGET ONE ATTEMPT PER SESSION.** MEASURED by `engrave.TimePlan` at production
`internal/sh2.Params()` and logged on every run by
`TestHashlockPlateCutDurationsAreBounded`:

| what | duration |
| --- | --- |
| the worst-case plate, whole | **43m31s** |
| the same plate without the QR | 14m39s |
| **the QR alone, at scale 2** | **32m12s** |

**The single-character test-plate pattern does NOT apply here**, on both halves.
`~21 min` is the md1 plate's figure, not this plate's; and a constant-time QR is
INDIVISIBLE by construction, because `ConstantQRCmd.Engrave` runs `for range
nmod` and pads every move to `maxDur` — which is the whole reason the toolpath is
content-independent. The only reduction available is cutting the QR alone onto a
blank: 32m12s against 43m31s, a 26% saving and not a 900x one.

A gate budgeted at two seconds and costing half an hour gets deferred — and
deferring this one ships an untested 53-module QR carrying a spend secret.

---

## What stands in for the device until the operator walks it

Spec §12's closing paragraph, transcribed: **until the operator walks it,
§11.7's emulator arm and §11.4's goldens are the acceptance.** They are not a
substitute for it. What they cover and what they cannot:

- **the emulator arm** (Task 12) drives the real firmware GUI in a browser and
  asserts SCREENS: it types the phrase, holds, adds a keyed path, reaches Done,
  accepts a plate at §5.3's pick screen, and asserts §8.3's census row carries
  the digest the confirm modal drew. It ran four times — unmutated, against two
  mutations that each failed it, and unmutated again after the revert. It
  engraves nothing.
- **§11.4's goldens** assert the plate BYTES that would be cut. They say nothing
  about steel, about a phone, or about whether a 0.6 mm module survives a burr.

The gap between the two is item 8, item 1's read-back and item 3's tap — and it
is a hand, a blank and a phone wide.
