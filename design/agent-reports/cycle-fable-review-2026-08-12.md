# Cycle review — systemwide payloads, stages 7–13 (pre-flash gate)

- **Date:** 2026-08-12
- **Reviewer:** Claude Fable 5, dispatched as the single whole-diff reviewer
- **Diff:** `seedhammer` `b14662a..85aa4cc` (10 commits, 46 files) +
  `mnemonic-engrave` `cb44252..22db348` (17 commits, 15 files)
- **Method:** Question 1 answered by EXECUTION — scratch Go tests drove the gui
  flows through the touch harness against region images produced by the real
  `me` binary (rebuilt at `22db348` first; the checked-in binary predated
  stage 7). Scratch file deleted; both trees verified clean at their original
  HEADs (`git status` empty, HEADs `22db348` / `85aa4cc`).
- **Verdict: 0 Critical, 0 Important, 6 Minor.** The journeys close. Nothing
  found that is unsafe to flash. Question 2 lists the decisions that are the
  operator's, with recommendations.

---

## QUESTION 1 — the operator CAN load a payload. Walked, not read.

Five walks, all end-to-end from `me`'s actual output, all passing. Every
function named below was watched in execution, not inferred. Where a claim
matters the walk carried a teeth check (an assertion that provably CAN fail).

### Walk 1 — plaintext, pack → boot → digest → two programs (J-A)

| step | command / function watched |
| --- | --- |
| 1. host pack | `me sysw pack --region --no-passphrase 'text:48656c6c6f20737465656c' '<12-word mnemonic>'` → **exit 0, 65536 bytes**, stderr digest `6fdf 89f2 eb72 0bb8 c125 ae8a 8153 f1b9` (`run_sysw` → `SyswCmd::Pack`, `crates/me-cli/src/main.rs:772`) |
| 2. boot offer | `uiFlow` (`gui/gui.go:1765`) → `syswLoadFlow(ctx, th, SyswReader(), true)` (`gui/sysw_load.go:25`) → `Probe()` → ChoiceScreen "A systemwide payload is present. Load it?" → LOAD |
| 3. read + parse | `sysw.FileReader.Read` (device: `sysw.XIPReader.Read`, `sysw/read_tinygo.go`) → `sysw.ParseHeader` → `sysw.Identity` → `sysw.Open` (plaintext: returns before any KDF) |
| 4. digest shown | `sysw.PublicDataHash` + `sysw.FormatHash` → `confirmReviewScreen` — **the rendered digest matched `me`'s stderr byte-for-byte**, and a corrupted digest deliberately did NOT match (the assertion has teeth) |
| 5. flags | `syswLoadWarnings` rendered F1 ("A SECRET is stored unencrypted in flash") → `syswHasFlag(flagSecretInPlaintext)` → KEEP/UNLOAD offer → KEEP |
| 6. session | `syswSession.load` (`gui/sysw_session.go:78`) — `ctx.sysw` assigned, `compared == true`. **The F-144 join exists and runs.** |
| 7. program A (seam) | `seedEntryFlow` (`gui/derive_xpub.go:88`) → `syswSeedPicker` → FROM PAYLOAD → `syswSession.take(ClassMnemonic)` → `bip39.ParseMnemonic` → `syswSourceAccept` ("Source: the systemwide payload") → **returned the packed mnemonic verbatim** (string-compared) |
| 8. program B (carousel program) | `engraveTextFlow` → `engraveTextFlowFrom` (`gui/freetext_flow.go:1485`) → `syswOffer(ClassFreeText)` → `sysw.DecodeBody` → text pre-filled → **"Hello steel" rendered on the text screen** after walking the QR/face/size steps — pre-fill, not bypass |

### Walk 2 — sealed, `pub_len == 0`, passphrase on the device keyboard (J-B)

`me sysw pack --region --passphrase-words 5 '<mnemonic>'` → 65536 bytes;
stderr printed the generated passphrase (`basic bread river forget liquid`)
and `digest: none — this payload has no public section`.

Device: `syswLoadFlow` → `inputWordsFlow` (checksumGate false, terminator
true; `gui/gui.go`) — all five words typed on the word keyboard; `done`
**touch-tapped on the drawn Button2 nav slot** (the control stage 9 found
unpressable is now pressable); count confirmation "5 words — unlock?" named
the FILLED count; UNLOCK → `sysw.Open` → `seal.DeriveKey` (iterations from
the header, bounded — below) → `compared = true` via the open (route 1,
§12.2 D1). **Asserted across every frame: no digest screen ever appeared**
(`[digest-shown]` at `pub_len == 0`). Session `sealed`, not `weak` (5 words ≥
cliff). The seam then delivered the packed mnemonic verbatim.

### Walk 3 — wrong passphrase (no prior GUI test drove this)

Same image, last word typed wrong (`lizard`): "That passphrase did not open
this payload." — flow returned false, **no session created**. A wrong open
cannot hand a program anything.

### Walk 4 — J-I, both halves

`me sysw pack` over a lone chunk of a declared 3-chunk `md1` set (S-J record
3, real card): host warned `record 0: an md1/mk1 this tool could not decode;
the device will treat it as a SECRET`, **exit 0** (D6: warns, never refuses).
Device: `syswSession.load` → `sysw.MDMKUnconfirmed` → `syswLoadWarnings`
rendered the DISTINCT sentence ("An md1/mk1 the device could not confirm —
treated as a secret…"), not the plain-secret one.

### Walk 5 — UNLOAD, then RELOAD (§13 D10)

`syswPayloadMenu` (`gui/sysw_unload.go:34`) → "A payload is loaded." →
UNLOAD → `syswUnloadFlow` → `syswReloadCost` — the sealed/`pub_len==0` case
carried the emphatic wording ("You will need the PASSPHRASE, and nothing else
will do") → confirm → `ctx.sysw = nil` → notice says "still in flash", names
`me sysw wipe`, **never says "erase"** (asserted). Second `syswPayloadMenu`
call went straight to `syswLoadFlow`, which **demanded the passphrase again**
— `[compared]` re-earned, fresh KDF, fresh session. **SHA-256 of the region
file identical before and after the whole walk.**

**D10 verified three ways beyond the walk:** (1) `sysw.XIPReader`
(`sysw/read_tinygo.go`) maps XIP with `unsafe.Slice` and copies out — no
write; (2) whole-tree sweep of `cmd/ gui/ sysw/ seal/` for flash-write
primitives (`flash.*(write|erase)`, `EraseBlocks`, `WriteBlocks`,
`machine.Flash`) finds only the unload notice STRING; the diff adds no
`unsafe`/`uintptr` anywhere; (3) `TestNoErasePathExistsOnTheDevice`
(`gui/sysw_unload_test.go:423`) pins erase-identifiers and the `SyswEraser`
seam on every suite run.

**Answer: the journeys close.** J-A, J-B, J-I, J-D(device)/J-E all walked by
execution from `me`'s real artifacts through to program consumption. No
F-144-shaped gap found: `ctx.sysw` is assigned at `gui/sysw_load.go:172`,
read through `take`/`has`/`syswOffer`, and the records arrive in programs
byte-identical to what was packed.

---

## QUESTION 2 — the operator's decisions

### 2.1 D9 on real hardware — keep it as written

Confirmed by code: `cmd/controller/platform_sh2.go` sets
`p.feats |= gui.FeatureNFC` unconditionally in `Init()` (the ST25R3916 is on
every board), so `syswSeedPicker` (`gui/derive_xpub.go:161`) always builds ≥2
rows on the SeedHammer II and the picker always appears.

- **Option (a), keep D9 as written.** Cost on the SH2: one press of the
  already-highlighted default (TYPE IT is row 0) at the head of seed entry in
  four programs. But the picker there is NOT "a menu of one" — it offers a
  genuine choice (TYPE IT / SCAN), which is exactly what D9's own text gates
  on. The ruling's stated complaint ("cost a click to offer ONE option") does
  not arise on the SH2 as written.
- **Option (b), gate on the payload alone (the one-word change).** Cost:
  `scanSeedFlow`'s ONLY production caller is inside `syswSeedPicker`
  (`gui/derive_xpub.go:191` — measured). With no payload loaded the picker —
  and SCAN with it — vanishes, making **NFC seed entry unreachable in all
  four seam programs** on the machine that always has a reader. That deletes
  the working half of J-C to save one press, and silently contradicts §3.1
  and §2.1's emphasised NFC-for-everything.

**Recommendation: (a).** D9 as written is the letter AND the sensible
reading; the implementer's uncertainty was right to flag and wrong to worry.
If the extra press on the most-walked path still grates in use, the fix is a
NEW ruling that re-shapes seed entry (e.g. keyboard-first with SCAN behind a
menu row), not a re-gating — that trade should be made looking at it on the
panel, after this flash.

### 2.2 Stage 13b — correct the plan, and correct the CORRECTION's number too

- **The rewrite preserves both rules — verified three ways.** (1) Grep:
  `backupWalletFlow` (`gui/gui.go:2338`), `singlesig_verify.go:83`,
  `multisig_verify.go:63`, `slip39_polish.go:291` still call plain
  `passphraseFlow`; the five seam sites (`derive_xpub.go`, `bip85.go`,
  `singlesig.go`, `multisig.go`, `multisig_build.go`) call
  `syswPassphraseFlow`, which also hex-decodes the body
  (`gui/sysw_source.go:82` — the wallet-changing trap, fixed and tested).
  (2) `TestTheSeamPassphraseOfferReachesOnlyProgramsThatAdmitIt`
  (`gui/sysw_admit_oracle_test.go:185`) pins BOTH directions — five sites
  must call it, four named files must not, with the rule each would break.
  (3) My sealed walk consumed a payload record only through the seam.
- **Yes, correct the plan** — it is already a corrected-in-place document
  (two CORRECTED markers); this is the third. **But the brief's "it has ten"
  is also wrong, and the fold must not transcribe it.** Measured at the
  pre-stage-13 tree: `git grep -E '\bpassphraseFlow\(' 2f498b8 -- 'gui/*.go'`
  minus tests = **10 matches = 1 definition (`gui/gui.go:654`) + 9 call
  sites** (5 seam + backupWallet + 2 verifies + slip39). "Ten non-test
  callers" counted the definition. The same miscount ships in two comments —
  `gui/sysw_source.go:53` and `gui/sysw_admit_oracle_test.go:183` — and
  should be fixed in the same fold (comment-only; per the proportional rule
  it re-triggers no gate).
- Fold the plan's journey-map state column at the same time — six rows stale
  (J-B, J-C, J-D device, J-G, J-H 13a–c, J-I all say "open", all built), the
  third implementer running to flag it. A map that exists to make absent
  steps visible must not be wrong in the other direction.

### 2.3 Other operator decisions in this diff (nobody has ruled)

1. **Index bases differ between `me sysw pack` (argv indices) and `me sysw
   show` (public-section indices)** for unconfirmed-record reports; they
   diverge exactly on sealed payloads (stage 7/8 log §5). Recommend the cheap
   third option: each line names its basis ("record 3, as given" / "public
   record 1") — two string edits, no renumbering, kills the confusion.
2. **"1 words — unlock?"** (`gui/sysw_load.go:121`) — transcribed verbatim
   from the plan. Recommend ruling the pluralization trivial and fixing it;
   it is not a rule change.
3. **Stage-12 integration is pinned by AST only** (P9, marker 0): no test
   drives `backupWalletFlow` through a COMPLETED engrave into
   `plateVerifyFlow`. The harness has `testEngraver`, so a behavioural pin is
   feasible. Recommend filing with an owning phase (the journeys/simulator
   phase fits) rather than leaving it a log paragraph.
4. **Acceptance-screen asymmetry** (stage 11–13 log §13): Engrave Text /
   BIP-39 Password render an F3 acceptance screen for payload records; the
   13a/13b/13c `syswOffer` sites do not (the offer screen itself names the
   source the operator picks). I judge this CONSISTENT with §3.2 as D5 scoped
   it. Recommend recording that reading in §3.2's scoped note; no code
   change.
5. **`checksumGate` off even for §7's every-word verify** — the implementer's
   reasoning (a checksum-masked keyboard would hide the wrong last word of a
   mis-cut plate, the exact defect §7 hunts) is better than the plan's.
   Recommend the spec absorb it as a §7.2 note so it isn't re-litigated.
6. **Vectors JSON `mdmk_unconfirmed` field degrades silently for a stale
   consumer** (stage 7/8 log §6). Both current consumers recompute from the
   blob, and the file is test-fixture-only. Recommend: accept, no action.

---

## QUESTION 3 — is anything unsafe to flash? No.

- **Brick risk: none found.** The only `cmd/controller` change is
  `p.feats |= gui.FeatureNFC` after NFC init (cannot fail). No new code runs
  before display init. The boot addition is an OFFERED ChoiceScreen; with no
  payload, `Probe()` false is silent (walked + pinned by
  `TestSyswLoadFlowIsSilentWithoutAPayload`). A garbage region that happens
  to carry the magic costs one error screen, not a hang: `ParseHeader`
  bounds everything, including **iterations (100 000–2 000 000, enforced on
  BOTH sides — I executed `me`'s refusal at 50 and at 5 000 000)**, so no
  crafted header can demand an unbounded KDF. Firmware builds at 1 341 040 B
  flash — nowhere near the payload region at `0x10D00000` (13 MiB in). Per
  F-148, even a bad image returns to BOOTSEL on workstation power; what
  cannot be done remotely is *judging* the boot — flash freely, record the
  result as UNVERIFIED until someone is in the room.
- **No flash write, verified tree-wide** (Q1, walk 5): read-only XIP reader,
  no write primitive anywhere in `cmd/ gui/ sysw/ seal/`, no new
  `unsafe`/`uintptr` in the diff, region bytes hash-identical across
  load/unload/reload, and the AST guard runs on every suite. D10 holds.
- **No new secret persistence.** The session is RAM-only and dies at
  power-off (§3.2.1); unload writes nothing. Passphrase/record residue in RAM
  is the explicitly accepted non-wiping class (§6.2.2a) — unchanged by this
  diff.
- **Wrong-but-plausible values — each candidate walked or pinned:** device
  digest == `me`'s digest byte-for-byte (teeth-checked); `show` == `pack`
  digest (executed); wrong passphrase → error + NO session (walked); count
  confirmation names the FILLED count (walked); unconfirmed md1 named
  distinctly at load (walked, both halves); `pass:` bodies hex-decoded at all
  three consumption sites (the wallet-changing trap — fixed, tested,
  `sysw_cells_test.go`); plate verify cannot self-certify (Back →
  `provNotVerified`, pinned by mutation; §7.4 kept structural by test 16 and
  the typed-only seam).
- **Stated residual:** the feature has never run on hardware — F4/NFC paths,
  touch coordinates on the real panel, and KDF duration under the payload's
  own header are all harness-verified only. None of that is a flash risk; it
  is the reason the first hardware session should walk Q1's five steps once
  on the panel.

---

## Findings

| # | sev | where | what |
| --- | --- | --- | --- |
| 1 | Minor | plan stage 13b; `gui/sysw_source.go:53`; `gui/sysw_admit_oracle_test.go:183` | caller count wrong twice: plan says 4, log/comments/brief say 10; measured **9 call sites + 1 definition**. Fix all three in the plan fold |
| 2 | Minor | `design/IMPLEMENTATION_PLAN_systemwide_payloads.md` journey map | six state-column rows stale ("open" for built stages) — third log running |
| 3 | Minor | `gui/sysw_load.go:121` | "1 words — unlock?" for a one-word passphrase |
| 4 | Minor | `me sysw pack` vs `show` | unconfirmed-record index bases diverge on sealed payloads; label the basis in each line |
| 5 | Minor | `gui/plate_verify.go` ↔ `gui/gui.go:2324` | stage-12 integration pinned by AST only; no behavioural test through a completed engrave (P9 marker 0, implementer-recorded) |
| 6 | Minor | `font/bitmap/bitmap.go:33` (`indexLen = unicode.MaxASCII`) | em dashes in the new operator strings ("5 words — unlock?", the F1 sentences, the unload notice) have no glyph and render as nothing — observed in extracted frames ("5words unlock?"). Legible; pre-existing house pattern (`bip85.go:228` predates the diff); worth knowing before reading the panel |

No Critical. No Important. A clean result is a real result: the two
components I would have flagged — the undrawn `done` button and the undecoded
`pass:` body — were each found and fixed by their own implementers, with
tests that fail without the fix; I verified both by driving them.

## What this review did NOT do

Did not re-audit `seal/`, `gui/unlock_kdf.go`, `unlockPayload` (frozen), nor
re-derive the machine-verified gate results (Rust/Go/tinygo/wasm/clippy/
spec-check green, 38 mutants). Did not run hardware. Did not walk J-C in a
browser (`shNFC.present` ordering caveat stands as the stage-10 log recorded
it). Scratch tests deleted; both trees left byte-identical to how they were
found (`git status` clean, HEADs `22db348` / `85aa4cc`); `me` was rebuilt in
place at HEAD (build artifact only — the checked-in binary predated stage 7,
which reviewers after me should know before trusting `target/release/me`).
