# Phase 2 whole-diff review (fable) — the copy inventory, and what a single-context read finds

- **Reviewer:** fable-tier whole-Phase-2 review, 2026-08-11 (post-release, deliberately)
- **Diff:** `seedhammer` `git diff 78949e7 93ee004` — 58 commits, 76 files, +13,563/-352, read as one artifact. Rust (`me`) read where it bears on the device.
- **Trees read at:** fork `823499c` (host suite green), `me` `4d5ef3f`.
- **Machine checks I ran myself:** `CGO_ENABLED=0 go test ./seal/ ./bip39/ ./backup/ ./engrave/ ./gui/ ./gui/op/ ./stepper/` → all `ok` (go 1.26.3). I did not re-run the TinyGo device build or the release scan (settled green in the brief).
- **My prior involvement, corrected for:** I designed §10.2.4 (`CONSULT_b2b_idle_timer_design.md`) and reviewed the spec's crypto core. That design shipped F-106 (fixed) and F-103 (open). §5 of this report reviews it adversarially.

## Verdict in one paragraph

**No new Critical. No new Important.** The whole-diff read confirms the B2b wipe-inventory audit's central measured result — *every copy reachable through a live reference after the §10.2.4 wipe is zeroed* — and finds no seam where a later phase made an earlier phase's guarantee unreachable. The residue that survives the wipe is exactly the accepted, filed, shipped-by-decision class (F-83/F-88/F-90/F-94/F-104 unwipeable garbage; F-103's silently-skippable timer; F-109's un-enumerated ~35 K). What a single-context read adds beyond those reports is three things: (1) a completed **entry-to-resting-place inventory** with two members no report has named — the `stepper.Driver` motion words and the SH2 LCD DMA chunk buffers, both F-83-class geometry copies; (2) a **positive** compositional finding — the wipe provably *cannot* fire mid-cut, so it can never strand resume geometry, which makes one bullet of F-110 describe an unreachable path; (3) confirmation that **F-103 defeats row 4 (the passphrase wipe) exactly as it defeats row 1**, so F-105's "CLOSED on hardware" holds only on a machine whose panel is quiet. All findings below are Minor or record-accuracy. I recommend closures for F-89, F-91, F-96, F-99, F-100, F-105, F-106, F-107, F-108, F-111 (already marked closed — I concur with evidence) and flag F-110's imprecise bullet.

---

## 1. THE COPY INVENTORY — every resting place of secret-derived bytes on the Sealed Payload path

Secret material enters at exactly one place: `XIPReader.Read()` copies the flash region into `blob` (`seal/read_tinygo.go:60`). From there it fans out. "Wipe reaches it?" is answered for the **§10.2.4 idle wipe / normal-exit** path. Legend: **Z** = zeroed on every exit and pinned by a test; **Z\*** = zeroed, wipe unpinned (deletable green); **G** = unwipeable garbage, dropped-not-zeroed, F-83/F-88 accepted class; **—** = not applicable.

### 1a. The envelope and the derived key

| # | Buffer / field | Where | Holds | Wipe reaches? |
|---|---|---|---|---|
| 1 | `blob` — region bytes (header ‖ public ‖ ct ‖ tag) | `gui/unlock_flow.go:38,58,110` | ciphertext + AAD, **not** plaintext | **Z** — `clear(blob); blob=nil` before the session (`:110`); deferred closure for other exits (`:58`, correctly a closure not `defer clear` — F-79). Not seed-equivalent. |
| 2 | `plaintext` — gcm.Open's whole decrypted container | `seal/unlock_key.go:81-91` | **every** secret in one array | **Z** — `defer clear(plaintext)` (`:88`), pinned FL via `unlockPlaintextHook`. Runs at unlock, before any timer window. |
| 3 | `pass` — §8.1 normalised passphrase | `gui/unlock_kdf.go:362-363` | passphrase | **Z** — `defer clear(pass)`; cap fixed 128 so `append` never orphans. |
| 4 | `key` — derived 32-byte AES key | `gui/unlock_kdf.go:364-369` | key | **Z** — `defer clear(key)`, pinned RL via `unlockKeyHook`. |
| 5 | `Deriver.u`, `Deriver.acc` | `seal/pbkdf2.go:38-53` | key-equivalent accumulators | **Z** — `defer d.Wipe()`; `dead` one-way so post-Wipe `Key()`=nil. |
| 6 | `Deriver.mac` ipad/opad (`crypto/hmac`) | `seal/pbkdf2.go:60-84` | passphrase-recoverable-by-XOR until 1st Step, key-equivalent for life | **G** — unexported stdlib; `Wipe` cannot reach. Documented precisely in-code. |
| 7 | AES round keys + GCM productTable | `seal/crypto.go:84-92` (`Open`) | key-equivalent | **G** — stdlib internals; unreachable when `Open` returns. Key-, not seed-, equivalent. |

### 1b. The typed passphrase (row 4 subject)

| # | Buffer | Where | Wipe reaches? |
|---|---|---|---|
| 8 | typed `m` (`[]Word`, one per attempt) | `gui/unlock_kdf.go:156-177,416` | **Z** — `clear(m)` per attempt + partial/back exits; pinned RL via `unlockPassphraseWordsHook`. |
| 9 | rendered passphrase glyphs in `ctx.B` | `gui/unlock_kdf.go:143` | **Z** — `ctx.B.Scrub()` in `unlockPassphraseFlow`'s defer (F-107). |
| 10 | `Keyboard.Fragment`, `wordLabel`, per-keystroke concatenations | `gui/gui.go:671-744,993` | **G** — Go strings, orphaned per keystroke in typing order. F-104 item 4. |
| 11 | `LastWordCandidates` / `Valid()` `math/big` nat + `entBytes` over the 11-word prefix, up to 2,048× | `bip39/bip39.go:135-197` | **G** — `entBytes` is a zeroable `[]byte` (cheap fix); big.Int internals are not. F-104 item 2. |

### 1c. The decrypted records (§10.2.2 primary subject)

| # | Buffer | Where | Wipe reaches? |
|---|---|---|---|
| 12 | `p.Secret[i].Record` — each secret record | `seal/open.go:34` | **Z** — per-record `WipeSecretAt` at plate build + on Cut/Skip/cancel; backstop `p.Wipe()`; RL-pinned both vectors. |
| 13 | stale `p.Secret` from a previous unlock | `seal/unlock_key.go:109-112` | **Z** — explicit clear before reassign. |
| 14 | partial `out` on an admission/§10.2.1a failure | `seal/record.go:255,284,539` | **Z** — `wipe(out)`; call-sites not regression-testable (stated in-code). |
| 15 | `Classify`'s `s := string(b)` for every ms1 record | `seal/record.go:211` | **G** — Go string; the ClassMnemonic branch takes `[]byte` and never stringifies (deliberate). F-88. |

### 1d. The mnemonic engrave arm (`unlockEngraveMnemonic`, vector A)

| # | Buffer | Where | Wipe reaches? |
|---|---|---|---|
| 16 | `rec` (seal's buffer) | `gui/unlock_session.go:325` | **Z** — `clear(rec)` before Engrave. |
| 17 | `m` — `bip39.Parse`'s `[]Word` copy | `gui/unlock_session.go:279,326` | **Z** — `clear(m)` at plate build **and** `defer clear(m)` for the unwind (F-89); RL-pinned (deleting the defer fails the audit test). |
| 18 | rendered seed words in `ctx.B` (`op.Glyph`→args) | `gui/unlock_session.go:104` | **Z** — `ctx.B.Scrub()` in `unlockSecretSession`'s defer (F-107), + run_flow.go:326 on wipe. RL-pinned (deleting Scrub → 2,031 non-zero args). |
| 19 | 64-byte BIP-39 seed | `gui/gui.go:246-251` (`deriveMasterKey`) | **Z\*** — `defer wipeBytes(seed)`; **unpinned, deletable green (F-94).** |
| 20 | BIP-32 master private key | `gui/gui.go:563-580`, SeedScreen probe | **Z\*** — `defer mk.Zero()`; **unpinned (F-94).** |
| 21 | `sentence []byte` — plaintext mnemonic, + its `append`-growth orphans | `bip39/bip39.go:217-226` (`MnemonicSeed`) | **G** — local of another package; made twice per engrave. F-88 row 1. |
| 22 | `x/crypto/pbkdf2` HMAC-SHA512 ipad/opad (sentence < 128-byte block) | `bip39/bip39.go:225` | **G** — plaintext mnemonic XOR-recoverable. F-104 item 1. |
| 23 | `splitMnemonic` big.Int nat + `entBytes` (full entropy) | `bip39/bip39.go:177-197` via `Valid()` | **G** — created by the classifier on **every** unlock. F-104 item 2. |
| 24 | `seedqr.QR(m)` digit string, `qr.Code.Bitmap`, ConstantQR `modules` | `gui/gui.go:540`, `seedqr/`, `engrave/engrave.go:418-` | **G** — content-encoding. F-88 row 2. |
| 25 | `engraveSeed`'s `words []string` — selection+order is the seed | `gui/gui.go:544-547` | **G** — captured by `frontSideSeed`'s closure, read *during* the cut; `clear(words)` would cut a corrupt plate. F-88 row 3, remedy retracted. |

### 1e. The ms1 engrave arm (`unlockEngraveCodex32`, six of seven vectors)

| # | Buffer | Where | Wipe reaches? |
|---|---|---|---|
| 26 | `rec` (seal's buffer) | `gui/unlock_session.go:219` | **Z** — `clear(rec)` before Engrave. |
| 27 | `string(rec)`, `codex32.String.s`, `id`, `s.String()`, `SeedString.Seed` (all alias one allocation) | `gui/unlock_session.go:187-199` | **G** — Go strings. F-90 item 1. |
| 28 | `strings.ToUpper(plate.Seed)` — one per ranging of the spline closure (≥2×) | `backup/backup.go:126,163` | **G** — F-104 item 3. |
| 29 | ms1 QR — `qr.Encode(ToUpper(share))` bitmap + ConstantQR modules | `backup/backup.go:127-137` | **G** — F-104 item 3. |

### 1f. The rendered plate as geometry (F-83 accepted class)

| # | Buffer | Where | Wipe reaches? |
|---|---|---|---|
| 30 | `plate.Spline` — closure over the plaintext, re-read per knot | `engrave/engrave.go` capturing `frontSideSeed`/`engraveSeedString` | **G** — F-83, accepted; live only during the cut. |
| 31 | `knotBuf` — materialised knots inside `planEngraving` | `engrave/engrave.go:1030-1050` | **Z** — `defer clear(knotBuf[:cap])` on every iterator exit (F-108); toolpath byte-identical across 5 plates. |
| 32 | `SafePointer.history` — resume knots | `engrave/engrave.go:1683-`, `ClearHistory` | **Z** on terminal exit via `releaseResumeState`; **G** on the engraveStopping double-Back (F-110, accepted — see §4). |
| 33 | `splineResumer.catchup` — per-restart copy of history | `gui/engraver.go:258-268` | **Z** — `defer clear(c)` in `Knot`. |
| 34 | **`stepper.Driver.buf` (128 words) + `bezier.Interpolator` position + `bspline.Segment`** | `stepper/stepper.go:16-19,41-78`, local in `runEngraving` | **G — NOT NAMED IN ANY REPORT.** The most-processed form of the seed geometry: the PIO step words tracing the QR modules. A local dropped as garbage when `runEngraving` returns; never zeroed. Same accepted register as `plate.Spline` (F-83) — live only during the cut, in the goroutine — so it changes no risk posture, but it belongs on F-83's roster. See §3 finding M1. |
| 35 | **SH2 LCD DMA chunk buffers `display.buffers[2][][2]byte`** | `cmd/controller/platform_sh2.go:61-62,646-680` | **G — refines the audit's abstract "display framebuffer".** Hold at most ~1/6 screen of the rendered seed as RGB565, reused chunk-by-chunk and overwritten on the next `Dirty`/`NextChunk` cycle (the 3:00 warning full-repaint, or the next screen). Physical-display class. See §3 finding M2. |

### 1g. Examined, not seed-bearing

- `a.warnBuf` (run_flow.go:29): only ever holds `wipeWarningOp` output — public warning text (sole writer verified). Reset each frame; the 228 KB accumulation is fixed and `warnBufHook`-pinned.
- Persistent `op.Drawer d`: cleared frame-by-frame by `Draw` (op.go:315) and by `Release()` on the wipe path; stale glyph refs bounded to one frame. F-109 §1d class.
- `EventRouter.events`/`evts`: tap coordinates, not typed content.
- `unlockPlates` / `labelEncryptedCards`: alias `p.Secret`/stringify the ClassMDMK subset only; secrets never converted.

**Inventory headline.** 35 distinct resting places. 18 are zeroed-and-pinned (**Z**); 2 are zeroed-but-unpinned (**Z\***, F-94); 15 are unwipeable garbage (**G**). Every **G** is live only while a plate is cut (F-83's own window) or is an unreachable stdlib/string internal — none is reachable through a live handle after the wipe, which the B2b audit measured directly. **Two members (#34, #35) appear in no prior inventory.** Neither shifts the accepted risk posture, because both are F-83-class geometry confined to the cut.

---

## 2. Cross-phase composition — what only a whole-diff read shows

**No invariant established in B2a was silently violated by B2b, and no B2a test guarantee was made unreachable.** Specifics I checked and cleared:

- **`RecordsResident` (B2a-ii) vs the timer key (B2b).** B2a-ii renamed `SecretsResident`→`RecordsResident` and narrowed its contract precisely *because* it reads false while string copies live. B2b keys the timer on the **session bracket** (`wipe_guard.go`), never on the predicate — F-90 item 2 dissolved, not fixed. The predicate still ships and its tests still bind. Verified: no `RecordsResident` call gates arming (`grep`: only `armed()` gates, and it reads `g.job.Status()`).
- **The multi-secret unwind sweep.** Vector F (3 ms1): parking on secret 1 and firing the wipe unwinds through `unlockSecretSession`'s `for _, i := range at` loop, where each remaining `unlockSecretPlate`'s `cs.Choose` returns `(0,false)` immediately under `ctx.Done` and the deferred `WipeSecretAt` runs. All 3 records zeroed — RL test `vectorF` captures and asserts exactly 3. The loop's lack of a `ctx.Done` check is safe by this degeneration, as the consult claimed.
- **The passphrase bracket closes before the KDF (row 5).** `unlockPassphraseFlow` returns `m` and its `defer` restores `ctx.wipe=prev(nil)` *before* `unlockAttemptOnce`→`unlockDerive` runs, so `ctx.wipe==nil` for the whole derivation and no wipe can fire mid-KDF (which would be unsurvivable — Run parks the warning branch). This is F-105's fix and it composes correctly with the secret-session bracket that arms *after* unlock. The two brackets never nest (save/restore makes it structural).
- **`ctx.B.Scrub()` double-call on the wipe path.** `unlockSecretSession`/`unlockPassphraseFlow` defers scrub `ctx.B`, and `run_flow.go:326` scrubs again after the unwind. Idempotent; both follow the last `draw()`. No ordering hazard — the FrameCallback fix (`if ctx.Done { return }`) means no frame is drawn during the unwind, so nothing races the scrub.
- **The FrameCallback Critical fix (Task 3).** `run_flow.go:143` — `if ctx.Done { return }` before `yield`, replacing the `ctx.Done = ctx.Done || !yield(o)` form that would clobber a Done set from *inside* the call. This is load-bearing for the wipe persisting; the comment's history (four rounds read it wrong) is accurate. The consult §1.2's two Frame-after-Done panic sites are made unreachable by this fix, and the `wiping` discard guard was correctly *removed* as dead (the comment at `:182-192` documents the measurement).

---

## 3. Findings

### M1 (Minor — inventory debt) — `stepper.Driver` motion state is an un-named F-83-class copy
`stepper.Driver.buf` (128 words), `bezier.Interpolator` position, and `bspline.Segment` (`stepper/stepper.go:16-78`) hold the seed geometry as PIO step words during a cut — the final, most-processed form of the QR the operator's seed encodes. `runEngraving` (`gui/engraver.go:199`) creates the `Driver` as a goroutine-local; it is never zeroed and drops as garbage on return. **Scenario:** an SWD probe during a secret cut (§2.2 item 9's attacker) reads these words and reconstructs the QR modules → the seed — the same recovery F-83 already concedes from `plate.Spline`. No F-number lists it. **Not a new exposure** (identical window and attacker to F-83), but F-83's roster is incomplete without it. `F-114` already notes `d.pos` exists as a *position source*; it does not note the buffer as *residue*. Suggest: add a row to F-83.

### M2 (Minor — inventory precision) — the SH2 LCD DMA buffers hold rendered-seed pixels
`display.buffers[2][][2]byte` (`platform_sh2.go:61`) hold rendered RGB565 pixels a chunk at a time. The wipe-inventory §1d dismissed "the display framebuffer" abstractly for the wipe path (the 3:00 warning repaints it). On a **normal exit** (read words, press Back) the pixels persist in the DMA buffers until the next screen's `Dirty`/`NextChunk` overwrites them chunk-by-chunk — a few frames. Bounded to ~1/6 screen at any instant (six chunks). Physical-display class, same register as the panel itself showing the seed. No action beyond recording it; the panel is inherently showing the operator their seed.

### M3 (Minor — record accuracy) — F-110 item 1 lists an unreachable wipe path
F-110 item 1 says `SafePointer.history` zeroing is skipped by "Engrave returning on `ctx.Done` (§10.2.4 firing mid-cut)". **§10.2.4 cannot fire mid-cut:** `wipeGuard.armed()` returns false for both `engraveRunning` and `engraveStopping` (`wipe_guard.go:53-57`), and in production `ctx.Done` is set *only* by the wipe (consult §1.1), which requires `armed`. So when `Engrave` returns on `ctx.Done`, the job is never running/stopping — it is either never-started (hold-to-start: no history) or terminal (plate-done: `releaseResumeState` runs and `ClearHistory` fires). The genuine skip is only the **operator double-Back in `engraveStopping`**, which is not the wipe. This is a **positive** compositional fact: the wipe provably cannot strand resume geometry. Suggest correcting F-110's bullet to name only the double-Back path; the residency conclusion is unaffected (that residue is still real and still accepted).

### M4 (Minor — cross-phase, operator-facing) — F-103 disables row 4 too; F-105 "CLOSED" is conditional
F-105 (passphrase wipe) is marked CLOSED on hardware (reading 3, warning 3:00 / wipe 3:30). But F-103 — the idle clock refreshing on raw `len(evts)>0` (`run_flow.go:251`, still shipped, still open) — defeats **every** idle-driven behaviour, row 4 included: a panel asserting spurious events (film, moisture) keeps the machine non-idle, so the passphrase wipe never fires either. F-105's reading was taken on a machine whose panel was quiet (film peeled). This is inside §2.2 item 16's umbrella ("the wipe can silently never run") and is not a new defect, but the F-105 entry's "CLOSED — hardware" reads as unconditional and is not: **F-105's guarantee is only real once F-103 is fixed.** Worth a one-line cross-reference on F-105, not a re-opening.

### N1 (Nit) — the warning title says "WIPING SECRET DATA" on the passphrase screen
`wipeWarningOp` hardcodes the title "WIPING SECRET DATA" (`wipe_warning.go:70`) for both subjects; only the body sentence varies by `subject`. On row 4 nothing is decrypted. The operator ruled the in-flight passphrase seed-equivalent, so "secret data" is defensible and the body sentence is correct and honest ("a partly typed passphrase"). Leave as-is or soften the title; no funds consequence.

---

## 4. Adversarial self-review of the §10.2.4 design (my consult)

The brief asked what else my design got wrong beyond F-106 (fixed) and F-103 (open). Reading the shipped implementation against the consult:

- **F-106 (6:00 doubling) — genuinely fixed and hardware-confirmed.** The `syncArmed`-before-block call (`run_flow.go:95-124,219`) processes a flow-installed arm edge on the frame it happens rather than at the next wakeup (which was the idle deadline, restarting the window). Readings 1 and 2 (3:00/3:30, and the `engraveStopping` park via `pl.Wakeup`) close both shapes. The `armed != a.armed` idempotence guard is load-bearing (dropping it fails 12 tests) and the two-call structure can only fire the wipe *earlier*, never later — I verified this reasoning against the code.
- **F-103 (spurious touch) — still my design's error, correctly filed.** Keying "idle" on raw event presence rather than *effective* input is the root cause. It is unfixed by explicit decision (§2.2 item 16). The right fix (key on router-consumed input) is itself a normative change to §10.2.4's "any touch resets it" and needs the R0 loop — correctly scoped that way in F-103's option 2.
- **What hardware has not yet exposed but the code is correct on:** the warning is drawn into `a.warnBuf`, never `ctx.B` (which holds the parked frame), and `wipeWarningOp` paints a full-screen background behind the text (op.Layer order verified), so the warning *replaces* the parked seed screen — the privacy blanking and the "no seed word under the warning" property are structural, not incidental. I could not find a **negative** test asserting no seed word appears in the composited warning frame (the consult §5.4 called for one); the property holds by construction (the warning op references none of the seed's ops), so this is a test-coverage gap, not a defect. Recording it so it is not mistaken for pinned.
- **The one design choice I'd still defend:** keying the timer on the session *bracket* rather than any buffer predicate. The whole-diff read confirms this is what makes the aborted-cut-mid-plate case safe and what dissolves F-90 item 2. It survives.

---

## 5. Suggested follow-up closures (operator decides — I did NOT edit FOLLOWUPS.md)

Items already marked CLOSED that my whole-diff read independently confirms with evidence (concur, no action): **F-89** (unwind via `ctx.Done`, killed by removing `ctx.Done=true` from the armed branch), **F-105** (bracket at `unlock_kdf.go:135-137`, hardware reading 3 — with the M4 caveat), **F-106** (both shapes, readings 1–2), **F-107** (`ctx.B.Scrub()` both brackets, RL-pinned, 2,031-arg mutation), **F-108** (knotBuf/history/catchup clears, toolpath byte-identical, reading 4), **F-110 `catchup` half** and **F-111** (subsumed by `planEngraving`'s defer).

Genuinely resolvable by the whole-diff vantage, proposed for closure:

1. **F-110 item 1 — reword, do not close.** Its "§10.2.4 firing mid-cut" bullet is unreachable (M3). The double-Back residue remains open and accepted. Suggest editing the bullet, not closing the entry.
2. **F-105 — add the F-103 cross-reference (M4).** "CLOSED — hardware" is conditional on a quiet panel; the entry should say so.
3. **F-83 — add the `stepper.Driver` row (M1).** Completes the geometry roster.

Items I explicitly **do not** propose touching: F-88, F-90, F-94, F-104, F-109 all still bind and are correctly scheduled to post-merge hardening — the audit's deferral analysis holds and my read found nothing reachable-after-wipe among them.

---

## 6. What I could NOT determine, and why

- **F-109's exact 81 objects.** I cannot enumerate them by object from a whole-diff read. `MemStats` counts, it does not name; and the un-enumerated ~35 K is dominated by **one-time infrastructure** whose identity I can characterise with high confidence but not prove per-object: `cipher.NewGCM`'s productTable (~4 KB) + AES round keys, `op.Buffer` backing arrays grown to seed-screen size (~12 KB, now content-zeroed on outgrow), font faces, grouping maps, `math/big` nats, HMAC states, the codex32/bip39 parse allocations. It **plateaus** (cycles 2/3 byte-identical) which is consistent with caches, not a per-wipe leak. The B2b audit measured that *nothing reachable through a live handle survives the wipe*, which answers the operator's real question ("is it reachable secret data the wipe should have caught?") in the negative — but "all 81 are provably harmless" requires the `runtime.SetFinalizer` census F-109 itself specifies, which is host-runnable but was not run here and is not a whole-diff-read task. **F-109 remains genuinely open; I narrowed its class, I did not close it.**
- **TinyGo GC fidelity.** Every measurement (mine and the audit's) ran host Go under `synctest`. TinyGo's conservative collector may retain or copy any **G**-class item longer than host Go. This is the firmware's standing caveat and I inherit it; the device readings (F-106/107/108 hardware) exercised timing and geometry, not residency.
- **Whether `stepper.Driver.buf` (#34) is collected promptly on TinyGo after a cut.** It is a dropped local on host; on TinyGo it lingers until allocation reuse, same as `plate.Spline`. Unmeasurable without an on-device finalizer probe.
- **The AES/GCM/HMAC stdlib internals (#6, #7, #22).** Reachability and content established by reading go1.26.3 sources (the `pbkdf2.go:60-84` caveat matches fips140/hmac), not by measurement — `unsafe`/reflection into stdlib internals would be needed to observe them, and I did not.

**Positive control for the negative claims:** the B2b audit's `TestWipeZeroesEveryPinnedBufferAtRunLevel` drives the real wipe and *fails* when any pinned wipe is deleted (2,031 non-zero args without `Scrub`; a surviving `[]Word` without the F-89 defer). So "everything reachable is zeroed" is a claim whose instrument demonstrably bites — it is not an empty search. Where I assert a copy is unreachable garbage (**G**), the control is that the same test holds a live handle to every reachable secret buffer and finds it zero, so a reachable survivor would have been caught; the **G** items are the complement it cannot hold a handle to.
