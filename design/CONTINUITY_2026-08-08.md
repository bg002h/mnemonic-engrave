# Continuity — 2026-08-08

Supersedes `CONTINUITY_2026-08-07b.md` for the **encrypted payload delivery**
feature. That doc handed off "Phase B not started". Since then **Phase B1 was
planned, gated, implemented, verified on the SeedHammer II, merged and pushed**,
**F-73 and F-74 are closed**, and **B2 has been split into B2a and B2b**.

Its §3 operational traps are superseded where they concern F-73; the rest stands.

---

## 1. STATE

```
mnemonic-engrave  master  777960b   pushed, in sync
seedhammer        main    78949e7   pushed, in sync, 47 pkgs ok + 2 sanctioned
```

Both trees clean. Nothing is waiting to be merged or pushed.

| | state |
| --- | --- |
| **Plan A** — host `me seal` / `me hash` | shipped |
| **Plan B Phase A** — device headless core | shipped |
| **Plan B Phase B1** — the **unsealed** path with UI | **shipped, hardware-verified** |
| **Plan B Phase B2a** — unlock + the secret session | **NEXT.** Needs plan + R0 gate |
| **Plan B Phase B2b** — residency wipe + residue | after B2a |
| T1–T7 constellation-terminal roadmap | complete (see `RECON_seedhammer_constellation_terminal.md`) |

## 2. WHAT B1 SHIPPED

§10.2 steps 1–4 plus the plate list and engrave: XIP detection at GUI start, a
**conditionally-shown** *Sealed Payload* menu entry, `Inspect`, the §6.6 hash
screen, §10.2.3's unauthenticated warning (only when `ct_len == 0`), a paged
plate list, engrave one public record.

**No secret is ever resident in B1** — it derives no key and decrypts nothing.
That is why §10.2.2 and §10.2.4 were *out of scope* rather than half-built, and
why B1 was cheap to review.

A **sealed** payload stops at a terminal screen. It must never fall through to
the plate list: `p.Public` on a sealed payload is a legitimate record set, and
engraving it while dropping the encrypted half is §6.4's
incomplete-backup-believed-complete.

## 3. THE B2 SPLIT — decided 2026-08-08

**B2a — unlock and the secret session.** §10.2 steps 5–9 **plus** §10.2.2.

- 12-word BIP-39 entry; validate the checksum on the **§8.1-normalised** form
  **before** the ~31 s KDF. Phase A's API allows this; nothing enforces it.
- PBKDF2 with a progress indicator that says how long it will take, or the
  operator concludes the machine has hung.
- AES-256-GCM open over `AAD = header ‖ public section`. **Fail closed.** The
  message must offer BOTH readings — wrong passphrase *or* altered payload — and
  keep the §6.6 hash on screen through the retry loop (§10.2 step 8).
- §10.2.2: **every secret record offered FIRST, consecutively**, each wiped as
  its plate leaves the screen **by any route** — Cut, Skip, Cancel, error, a
  failed engrave. A cancelled engrave wipes too; re-cutting needs a fresh
  unlock, and that is deliberate.
- `Payload.Wipe()` exists for this. `AdmittedRecord.Record` is `[]byte`
  precisely so it can be zeroed — a Go string cannot be.

**B2b — the residency wipe and residue.** §10.2.4's idle timer, F-80's B2-owned
items, F-76.

**Why the seam is here.** A B2a that decrypted but did not offer and wipe secrets
would leave seed material resident with no lifecycle — strictly worse than not
decrypting. So unlock and session ship together. §10.2.4 is separable because the
spec itself calls it a backstop: *"the controls carrying real weight are physical
custody, Lock being one tap away, and the secrets being gone within the first N
plates."*

> **THE FEATURE IS NOT OPERATOR-COMPLETE UNTIL B2b.** Do not tag a release after
> B2a. §10.2.4 is a backstop, not an optional extra.

## 4. WHAT B2a MUST CLEAR

- **F-77 — GATING.** The encrypted section routinely carries `mk1`/`md1` cards
  (12 of vector F's 15 secret records are cards), and pass 3 computes grouping
  for the **public section only** (`seal/record.go:186`). §10.2.2's secret plate
  labels are unimplementable until it is extended over the encrypted `ClassMDMK`
  subset. **Reuse `groupCards`/`cardKey` — do NOT re-derive classification in
  `gui`**, which is the divergence `Opener.Inspect`'s doc comment exists to
  prevent.
- **F-79** — 64 KB retained for the GUI's lifetime when at most 16,450 bytes can
  be meaningful (~14% of free heap). Fix before an operator sees the feature.
  Payload-present *plus* a running engrave is the one config hardware never
  exercised.
- **SPEC §7.1's RP2350B KDF rate** — owed before release, and now scheduled to be
  measured **in situ during B2** by timing the real unlock. `cmd/kdfbench` was
  deliberately NOT run (it says to run on a Pico, and argues the rate transfers).

## 5. CARRY FORWARD — verified, do not rediscover

- **§10.2.4's timer does not exist in a flow-visible form.** `idleTimeout`
  (`gui/gui.go:2801`) drives the *screensaver* from `Run`'s frame loop and is
  invisible to flows. Chosen approach: a last-input timestamp on `Context`, with
  the engrave screen pausing the timer by simply not consulting it.
  `AppendEvents` (`cmd/controller/platform_sh2.go:368`) appends only on touch or
  stdin, so `a.idle.start` is a true last-physical-input time.
- **Timer semantics, operator decision 2026-08-07:** warning at **3:00**, wipe at
  **3:30**, **paused while engraving** (§10.2.4 row 2).
- `seedEntryFlow` (`gui/derive_xpub.go:82`) returns `bip39.Mnemonic` = `[]Word` =
  `[]int` — scrubbed by **manual zeroing**, NOT `wipeBytes`, which takes `[]byte`.
- `layoutNavigation` indexes a fixed `[3]int` (`gui/gui.go:1857`). §10.3 as
  amended: the plate list is **Back / Page / OK, and Back IS Lock**.
- `uiContains` (`gui/gui_test.go:516`) compares **extracted text, not pixels** —
  no screen test can catch a mis-drawn glyph.

## 6. HOW TO WORK IN THIS REPO

- **Push `master` via `ci/staging`** so the required check is SATISFIED, not
  bypassed. See `CLAUDE.md`. The tell: a push printing "Bypassed rule violations"
  means the staging step was missed.
- **Run both gates before any plan reaches a reviewer**:
  `scripts/plan-cite-gate.sh` (resolves `file:line` and `pkg.Symbol`, prints the
  line) and `scripts/plan-build-gate-go.sh` (tier 1 type-checks whole-file Go
  blocks and GATES; tier 2 parse-checks fragments, informational).
- **Write plan code blocks as COMPLETE files where possible.** On the B1 plan
  tier 1 had nothing to check because every whole-file block was elided with
  `...`. A plan that elides everything gets no type checking.
- **Ask before a second round of review** (user directive 2026-08-07). Report,
  fold, gate, commit — then stop and ask.
- Persist each review verbatim in its own commit; the fold lands in a second with
  the gate output in its message.

## 7. THE LESSONS THIS CYCLE TAUGHT

**Folds author most defects — and specifically the part nobody asked for.**
R0 rounds 1 and 2 each found a defect authored by the previous fold, both times
in context *volunteered beyond the finding*, not in the fixes. Round 1's claimed
a data flow that did not exist; round 2's claimed encrypted-section records
"can be `ms1` or a bare mnemonic, neither of which is a card at all" — which the
spec, `permitted()` and two vectors all contradict. **Treat every factual claim
in volunteered context as a new assertion needing its own verification.**

**Tests that cannot fail.** Both whole-diff Importants were this. `"SEALED"` is a
substring of `"UNSEALED"`, so the sealed assertion passed on an unsealed screen —
and it was **one-directional**, catching the opposite mutant, which is exactly
why it read as correct. And `sel = start` was unfalsifiable because the test
asserted *some* screen appeared, never *which record*. **Mutate in both
directions; a one-directional kill is not a kill.**

**Overclaiming commit messages, three more.** `gofmt -l <path> && echo "clean"`
always prints "clean" — `gofmt -l` reports by **printing** and exits 0 either
way. Test the output: `out=$(gofmt -l …); [ -z "$out" ]`.

**A near-miss digest is never almost-the-same data.** SHA-256 avalanches, so
agreeing in 31 of 32 hex digits has probability ~2⁻¹²⁴. On a near-miss suspect
transcription or rendering — never the algorithm.

## 8. WHAT COMES AFTER B2

The T1–T7 roadmap is complete and B2 finishes the last feature cycle, so the
project shifts from building to shipping.

1. **Tag a release.** `mnemonic-engrave` is at `v0.4.0`, the fork at `v1.4.3`.
   The §7.1 KDF number closes inside B2, so B2b completing means release-ready.
2. **F-65 — back up the SH2 boot signing key**, using the feature itself. The key
   is a 256-bit secp256k1 scalar and BIP-39 at 256 bits is 256 + 8 checksum =
   **exactly 24 words**, so it encodes losslessly as an already-admitted secret
   record. Zero spec change. Calibrate: slots 2 and 3 are free, so losing it
   costs a spare OTP slot, **not funds**.
3. **F-66 — arbitrary plain text over the sealed path.** Own gated cycle;
   subsumes F-65. Real hazard: a naive raw-text record kind reopens the
   `command: lock-boot` → `LockBoot()` hole that §10.2.1 exists to close. That
   was R0 round 1's first Critical and must not be undone.
4. **Residue and other tracks** — a font cycle (F-78's invisible `·` in four
   shipped screens, plus a rasterising check so tests can see rendering at all),
   F-71/F-75/F-80, and the untouched GUI/engraving track (F-58, F-61, F-62,
   F-63, F-64).
5. Optional: upstreaming to `seedhammer/seedhammer` (branch off `upstream/main`,
   signed + DCO, small focused PRs).

## 9. OPEN FOLLOW-UPS

**Gating B2a:** F-77. **In B2a:** F-79. **In B2b:** F-80's two B2 items, F-76.
**After the cycle ships:** F-65, F-66.
**Ownerless residue:** F-71, F-75, F-78, the rest of F-80.
**Other tracks:** F-58, F-60, F-61, F-62, F-63, F-64. **Historical:** F-72.
**CLOSED:** F-67…F-70, F-73, F-74.

*(Record defect: F-73 and F-74 are marked CLOSED in their headings but still sit
above the `## Resolved` marker, unlike F-67–F-70 which were moved. Fold the move
into the next commit that touches `FOLLOWUPS.md`.)*
