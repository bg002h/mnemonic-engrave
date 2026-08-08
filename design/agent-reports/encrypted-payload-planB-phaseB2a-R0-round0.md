# Phase B2a plan — R0 round 0 (independent architect review, opus)

Artifact: `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2a.md` at commit `dbbd373`.
Fork under review: `/scratch/code/shibboleth/seedhammer` @ `78949e7`.
Persisted verbatim, before any fold. HTML entities from the transport layer
(`&gt;`, `&amp;`, `&lt;`) have been restored to their literal characters; nothing else is altered.

---

VERDICT: 1 Critical / 4 Important / 6 Minor / 3 Nit

---

## [C1] The wipe does not fire on a cancelled or failed engrave — §10.2.2's explicit bullet — and the stated reason it cannot is false

**WHERE:** Plan §"The one place this plan interprets the spec" (lines 159–190) and Task 6b (`unlockSecretPlate`, `unlockEngraveCodex32`, `unlockEngraveMnemonic`). Fork: `gui/gui.go:2644-2707`, `gui/engraver.go:17,64,95,153,170`.

**CLAIM:**
> "**This plan reads §10.2.2's 'wiped as its plate leaves the screen — by any route' literally: the wipe fires when `Engrave` RETURNS**"
> "The stricter reading — wipe the moment the job reports failure — makes the retry prompt a lie, **because the record it offers to re-cut is already zeroed**. Removing the prompt instead means editing `EngraveScreen` […]"

**DEFECT:** Two things.

1. **The premise is false.** `NewEngraveScreen(ctx, plate)` builds `newEngraverJob(ctx.Platform, plate.Spline, plate.Conf, 0)` (`gui/engraver.go:64`), and the retry path is `Status()` → `e.Start()` → `runEngraving`, which iterates `e.spline` (`gui/engraver.go:170`). **The record bytes are never read again after `toPlate`/`engraveSeed` returns.** Zeroing `rec` before `Engrave` is called does not make the retry prompt a lie — the retry re-cuts from the spline. There is no reason to hold the record across the engrave at all, and no `EngraveScreen` edit is required.

2. **The consequence is broader than the failure case the plan names.** §10.2.2 states, as a bullet and not as diagram furniture: *"**A cancelled or failed engrave wipes the record too.** Aborting mid-plate to re-seat shifted steel is the machine's most ordinary recovery […] Re-cutting needs a fresh unlock; that is the price and it is deliberate."* Under the plan, Back-while-running does **not** return from `Engrave`: `gui/gui.go:2651-2656` calls `s.job.Stop()` and stays in `frames`, the state settles to `engraveStopped`, and the screen reads *"Engraving paused.\nHold button to resume."* (`gui/gui.go:2777-2779`). The operator can **resume without a fresh unlock**, with the decrypted seed still resident, on the exact "abort mid-plate" path §10.2.2 singles out. Only a *second* Back returns and fires the defer. So a cancelled engrave does not wipe, and Task 9.6 as written ("Cancel a secret plate mid-cut, then confirm re-cutting requires the twelve words again") will observe the opposite of §10.2.2's stated price.

The brief instructs that where the plan and the SPEC disagree the SPEC wins. B2a ships with no §10.2.4 backstop, so this window has nothing behind it.

**CONSEQUENCE:** On the shipped B2a build, a decrypted `ms1` or bare mnemonic stays in SRAM for the whole ~21-minute engrave, indefinitely on the paused screen after a mid-cut abort, and indefinitely on `engraveFailed`'s retry prompt — with `debug enable: 1` making SWD SRAM reads live (§2.2 item 9). F-81 as filed understates it: it names only the failure path, not the abort path the SPEC calls "the machine's most ordinary recovery". A `SecretsResident()` that stays true for the entire engrave also forces B2b to special-case what the residency key was supposed to answer.

**FIX:** In `unlockEngraveCodex32` and `unlockEngraveMnemonic`, wipe the moment the plate exists — before `Engrave` is entered:

```go
	plate, err := toPlate(plan, params)
	if err != nil { … }
	// §10.2.2: the plate carries the geometry; the record is no longer needed.
	// engraveJob holds plate.Spline (gui/engraver.go:64,170), so a retry or a
	// resume re-cuts from the spline, not from these bytes.
	clear(rec)
	NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)
```

and the same after `engraveSeed` in `unlockEngraveMnemonic` (`clear(rec)`; `m` is already `defer clear`ed). The defer in `unlockSecretPlate` stays and becomes idempotent. Delete F-81 and the "one place this plan interprets the spec" section, or rewrite it to record that no interpretation was needed. Task 9.6 then observes what it claims.

---

## [I1] `defer clear(blob)` pins the 64 KB region for the whole flow, defeating Task 2/F-79 in exactly the configuration Task 9.5 tests

**WHERE:** Plan §2c and §5d.

**CLAIM:**
> §2c: "`defer clear(blob)`"
> §5d: "`clear(blob)` / `blob = nil` […] Zero and drop it BEFORE the session, so the engrave that follows does not run with up to 16 KB of dead region on the heap. **The deferred clear at the top of this function is then a no-op on a nil slice.**"

**DEFECT:** Deferred call arguments are evaluated and stored when the `defer` statement executes. `defer clear(blob)` captures the slice header — including the pointer to the 65,536-byte array (`seal/wire.go:50` `RegionLen = 65_536`; `XIPReader.Read` allocates `clampRegion(RegionLen)`). A later `blob = nil` rebinds the local only; the defer record still holds the array, so the GC cannot collect it. Verified by execution under the repo's own toolchain (go1.26.3): with `b := []byte{1,2,3}; defer clear(b); b = nil`, the original array reads `[0 0 0]` after return — the deferred call reached it, which is only possible if the reference was retained.

So the deferred clear is not "a no-op on a nil slice"; it is a live 64 KB root held across `unlockSecretSession` and `unlockPlateListFlow`.

**CONSEQUENCE:** F-79's whole objective — "payload-present **plus** a running engrave is the one configuration hardware has never exercised … If that combination exhausts the heap the failure is an out-of-memory *during* an engrave" — is not achieved on the sealed path. Task 9.5 would be run against an implementation that still holds ~14% of free heap through the cut, and a green result would be read as F-79 closed.

**FIX:** Make the deferred call read the variable rather than capture the value:

```go
	// F-79: the closure reads `blob` at EXIT, so the `blob = nil` on the sealed
	// path actually releases the region. `defer clear(blob)` would capture the
	// slice header here and pin the array for the whole flow.
	defer func() { clear(blob) }()
```

and drop the "no-op on a nil slice" sentence from §5d.

---

## [I2] No KDF instrument exists on the `gui` side, yet three tables name a "KDF-counter test" as the sole killer for two mandatory §11.3 mutants

**WHERE:** Plan Global Constraints ("B2a-specific"), §5c `unlockAttemptOnce` doc, Task 4.2, Task 5.3, Task 5.6 mutation table, Task 8 mutation table.

**CLAIM:**
> Global Constraints: "`Opener.KDF` (`seal/open.go:24`) is the sanctioned seam; **Task 3 adds the equivalent seam for the chunked path.**"
> Task 5.6: "checksum gate moved after `unlockDerive` | **the KDF-counter assertion**, not the return value — both orders return the same error"
> Task 8: "BIP-39 checksum check removed | Task 5's KDF-counter test"; "KDF run before the checksum gate | Task 5's KDF-counter test"

**DEFECT:** Task 3 adds no seam. `seal/pbkdf2.go` as written exports `NewDeriver`, `Step`, `Done`, `Total`, `Key`, `Wipe` and nothing injectable. `unlockDerive` calls `seal.NewDeriver` directly, and `unlockAttemptOnce` builds `var o seal.Opener` (zero `KDF`) and calls `UnlockWithKey`, which never touches `Opener.KDF`. **B2a moves the device path off `Opener.Unlock` entirely, so the one existing instrument — `countingKDF` / `Opener.KDF` (`seal/open_test.go:14-20`) — is no longer in the path under test.** §5e then explicitly rejects stubbing ("rather than stubbing the KDF — which keeps the real deriver in the path under test"), so the plan simultaneously requires a counter and forbids the mechanism for one.

§11.2 requires "BIP-39 checksum rejection happens without invoking the KDF" and §11.3 makes both mutants mandatory; Task 8.2 makes a surviving mutant blocking.

**CONSEQUENCE:** The implementer reaches Task 5.3 with a required assertion and no mechanism. The likely outcome is a return-value assertion on `errUnlockChecksum` — which the plan's own table says cannot distinguish the mutant ("both orders return the same error") — i.e. a false PASS over exactly the defect §11.2 warns about, shipped with Task 8 reported green.

**FIX:** Add the seam in Task 3 or Task 5, in the sanctioned in-file style already used by `unlockEngraveHook` and `unlockSecretHook`. Smallest version, in `gui/unlock_kdf.go`:

```go
// newDeriver is the KDF seam. §11.2/§11.3 require "no KDF ran" to be asserted by
// INSTRUMENTATION, not by return value — both orders of the checksum gate return
// the same error. UnlockWithKey bypasses Opener.KDF, so this is the only seam
// left on the device path. nil-free: production is seal.NewDeriver.
var newDeriver = seal.NewDeriver
```

and have `unlockDerive` call `newDeriver(...)`. Then state in Task 5.3 that the counter is a test-local swap of `newDeriver`. Apply the same treatment to Task 8's "passphrase prompted when `ct_len == 0` … asserted by instrumenting the entry point" row, which has no named entry-point instrument either.

---

## [I3] `sealForTest` is unreachable from `package gui`, and no vector carries `md1`/`mk1` in **both** sections — so Task 7.1's `(sealed)` fixture cannot be built as specified

**WHERE:** Plan §5e and Task 7.1 / Task 7.4.

**CLAIM:**
> §5e: "gui tests build low-iteration blobs with `sealForTest` (`seal/open_test.go:44`) rather than stubbing the KDF"
> Task 7.1: "a `sealForTest` payload with cards in **both** sections (the `(sealed)` suffix appears and the two `mk1 1/2` entries are distinguishable)"
> Task 7.4: "drop the `(sealed)` suffix → the both-sections test fails"

**DEFECT:** `sealForTest` is declared `func sealForTest(...)` — unexported — in `seal/open_test.go`, which is `package seal`. Test files are not part of the importable package; `gui` tests cannot call it under any import. `gui`'s only blob source today is `payloadReaderFor`/`sealVectorBlob` over `seal/testdata/vectors.json`.

And no vector supplies the fixture: measured from the vector file — A(pub 0/sec 1), B(0/1), C(0/6), D(pub 5/sec 1), E(pub 5/sec 0), F(0/15), G(pub 12/sec 3), with C's and F's encrypted cards sitting on `pub_len == 0` payloads and D's and G's encrypted records being `ms1`/mnemonic only. **`mixed = pub && enc` in `unlockPlates` is therefore false for every vector**, so the `(sealed)` suffix — the one branch that keeps a public `mk1 1/2` distinguishable from an encrypted `mk1 1/2` — has no reachable test.

**CONSEQUENCE:** Task 7's central correctness claim ("Rendered without this the list shows the same label twice and the operator cannot tell which plate they are about to cut") ships untested, and Task 7.4's named mutant cannot be run. The implementer discovers this only after writing Task 7's tests.

**FIX:** Pick one and name it in §5e: (a) move `sealForTest` into a non-`_test.go` file in `seal` behind an exported test-only name (e.g. `seal.SealForTest`), or (b) add a `gui`-local sealer in `gui/unlock_program_test.go` built on `seal.Header.Encode` + `seal.DeriveKey` + `crypto/cipher`, or (c) add an eighth vector with cards in both sections — but (c) is a Rust-primary change and does not belong in B2a. Drop the "low-iteration" motivation while you are there: the host derives 100,000 PBKDF2-HMAC-SHA256 iterations in tens of milliseconds; the ~31 s figure is device-only.

---

## [I4] `TestUnreadableEncryptedCardDoesNotReject` cannot fail for the property it is named for, and the §1.5 mutant it is named to kill survives the entire test set

**WHERE:** Plan §1e (the third test) and §1.5 mutation table.

**CLAIM:**
> Test comment: "A BCH-valid md1 that `ParseChunkHeader` refuses is what §6.3 documents as constructible; **here it is enough that** a mixed section with a record whose card cannot be read still ADMITS, with zero labels on that record."
> §1.5: "the grouping error returned instead of discarded | `TestUnreadableEncryptedCardDoesNotReject`"
> §1.5: "subset filter widened to every record (drop the `ClassMDMK` continue) | `TestUnreadableEncryptedCardDoesNotReject` — `cardKey` fails closed, so the whole section starts rejecting"

**DEFECT:** The test's section is a **single BIP-39 mnemonic** — no `ClassMDMK` record at all. `labelEncryptedCards` therefore returns at `if len(strs) == 0 { return }` and `groupRecords` is never called. The test asserts a path it does not reach.

Consequently the mutant "**the grouping error returned instead of discarded**" — the one that would turn a *label* failure into an **admission** change, which §1a reason 3 calls the binding reason for the whole design and which the Rust-primary rule puts in Rust first — **survives every test in §1e and §1f**: vectors C and F group cleanly (no error), and this test never reaches `groupRecords`. Verified by construction: the seal package's existing `smuggledMD1` fixture does *not* help, because `md.ParseChunkHeader(smuggledMD1)` **succeeds** (measured: `{Version:0 Chunked:false ChunkSetID:0 …}`, `err=<nil>`), so `cardKey` returns cleanly on it.

(The second row is separately mislabelled: with the subset filter widened, the error is still *discarded*, so nothing "starts rejecting" — that mutant is actually killed by `TestEncryptedSectionCardsAreLabelled`. See M2.)

**CONSEQUENCE:** Task 1 gates B2a, and its normative claim — "no grouping failure rejects a payload" — is unpinned. A later edit that propagates the error rejects a legitimate payload whose encrypted section contains one unparseable-header card; the operator's own backup becomes un-engravable and the failure reads as "payload unreadable", i.e. as tampering. Task 8.2 would report this mutant as blocking only if the fixture existed.

**FIX:** Use a record that is `ValidMD` but whose chunk header refuses. Measured in this fork: `codex32.AssembleMD1(make([]byte, 0))` classifies as `ClassMDMK`, and `md.ParseChunkHeader` returns `md: bit stream truncated`, so `cardKey` fails with `ErrUndecodableCardSet`. Replace the test's fixture with a mixed section — that record plus one real `md1` from vector C — and assert (a) `AdmitSection(..., SectionEncrypted)` returns **no error**, (b) every record is admitted, and (c) all label fields are zero. That single case kills both §1.5 rows.

---

## [M1] The `Unlock` tail fragment removes the local it then keeps a guard on

**WHERE:** Plan §5a, immediately after the `UnlockWithKey` block.

**CLAIM:** "with the now-unused `nPub`, `end` and `split` locals removed from `Unlock` (they moved into `UnlockWithKey`); **the `len(blob) < end` guard** and the `if !h.Sealed() { return nil }` early return stay in `Unlock`"

**DEFECT:** Self-contradictory: removing `end` makes `len(blob) < end` undefined. `nPub` and `split` do become unused; `end` does not.

**CONSEQUENCE:** A compile error on the one fragment the gate does not cover, plus a moment's doubt about whether the guard was meant to move.

**FIX:** "with the now-unused `nPub` and `split` locals removed; `end`, its `len(blob) < end` guard and the `if !h.Sealed() { return nil }` early return stay (the guard is now redundant with `UnlockWithKey`'s, deliberately — `Unlock` is still a public entry point)."

---

## [M2] Three mutation-table rows name a killer that does not kill

**WHERE:** Plan §1.5 and Task 6.4.

**CLAIM / DEFECT:**

| row | why the named killer does not kill |
| --- | --- |
| §1.5 "subset filter widened to every record … `TestUnreadableEncryptedCardDoesNotReject` — cardKey fails closed, **so the whole section starts rejecting**" | The error is discarded by design, so nothing rejects. Actually killed by `TestEncryptedSectionCardsAreLabelled` (vector C's `ms1` poisons the group, all labels come back zero). |
| 6.4 "`defer p.WipeSecretAt(i)` moved to after the `Engrave` call \| **the cancelled-engrave test**" | `Engrave` returns on cancel, so the moved statement still runs and the buffer is still zero. Actually killed by the Skip test and the Back test, which return before `Engrave`. |
| 6.4 "`IsSecret` widened to include `ClassMDMK` \| the 'no secret in the plate list' test — **vector F's twelve cards would vanish from it**" | `unlockPlates` filters on `r.Class != seal.ClassMDMK`, not on `IsSecret`; the twelve entries stay in the list (with zeroed `Record` and intact labels). Actually killed by the vector-F offer-order test, which sees 15 offers instead of 3. |

**CONSEQUENCE:** Task 8.1 requires the results pasted into the commit message. A row whose named killer stays green reads as a surviving mutant and costs a round of investigation, or worse gets waved through.

**FIX:** Repoint each row to the test that actually kills it, as above.

---

## [M3] Two `unlockPlateListFlow` call sites and one now-false comment are not scheduled

**WHERE:** Plan §7c and Task 7.2.

**CLAIM:** "`unlockPlateListFlow` takes `[]unlockPlate`"

**DEFECT:** Three consequences go unmentioned: (a) the **unsealed** call site `gui/unlock_flow.go:73` `unlockPlateListFlow(ctx, th, p.Public)` must become `unlockPlates(p)`; (b) `gui/unlock_platelist_test.go:126` calls it with `[]seal.AdmittedRecord`; (c) `unlockEngraveFlow`'s call-site comment (`gui/unlock_platelist.go:~180`) says the `string(rec.Record)` conversion is "HARMLESS HERE — B1 holds public data only — and **ACTIVELY WRONG in B2**, where the same call shape on a secret record makes an unwipeable copy" — but Task 7 deliberately routes *encrypted-section* `md1`/`mk1` through that exact call, which §6.3 says is correct. The comment now tells a B2b reader the opposite of what B2a decided, which is precisely the failure §1d spends a task fixing for `AdmittedRecord`.

**CONSEQUENCE:** (a) and (b) are compile errors; (c) is the stale-comment class the plan itself calls "not cosmetic".

**FIX:** Name all three in Task 7.2, and give (c) the same treatment as §1d: rewrite it to say the conversion is admissible for `md1`/`mk1` from either section per §6.3, and remains wrong for anything `seal.IsSecret` admits.

---

## [M4] `inputWordsFlow(…, "Passphrase")` removes the only progress indicator on the word-entry screen

**WHERE:** Plan §4a / §5c `unlockPassphraseFlow`.

**CLAIM:** "What it does reuse unmodified is `inputWordsFlow`, whose length is `len(mnemonic)` and whose `title` parameter is a documented additive seam (`gui/gui.go:762-764`: `""` renders `"Word %d of %d"`, non-empty replaces it)."

**DEFECT:** The reading of the seam is right, and that is the problem: `gui/gui.go:765-770` renders the title as an either/or. Passing `"Passphrase"` **replaces** `"Word 3 of 12"`, and that line is the only per-word progress on the screen (the rest is the keyboard, the current fragment and a match count). SPEC §8 says the device "reuses the existing 12-word seed-entry flow unmodified".

**CONSEQUENCE:** The operator types twelve words with no indication of how many remain, on the screen that gates a ~31 s KDF. It also breaks the existing negative assertion idiom `uiContains(content, "Word 1 of")` used by `TestSealedPayloadStopsAtATerminalScreen` (`gui/unlock_flow_test.go:236`).

**FIX:** Either pass `""` and set the screen's identity elsewhere, or widen the seam once — `layoutTitlef(ctx, dims.X, th.Text, "%s — word %d of %d", title, selected+1, len(mnemonic))` when `title != ""` — and say so in §4a. Note the knock-on to the §5e replacement test's anchor.

---

## [M5] `SeedScreen.Confirm` lets the operator edit the payload's mnemonic before it is engraved

**WHERE:** Plan Decision 5 and §6b `unlockEngraveMnemonic`. Fork: `gui/gui.go:2296-2412`.

**CLAIM:** "The plate produced here is the one `backupWalletFlow`'s Skip-passphrase path produces."

**DEFECT:** True, and it inherits `SeedScreen`'s **Edit** affordance: `editBtn := &Clickable{Button: Button2, AltButton: Center}` → `inputWordsFlow(ctx, th, mnemonic, s.selected, "")`, which writes `mnemonic[selected] = w` in place. On a touch-only SH2, `AltButton: Center` means a centre tap opens the editor. For a *typed* seed that is a typo fix; for a *payload-sourced* seed every edit is a corruption of authoritative data, and the flow then derives a matching fingerprint and engraves it, so the plate is internally self-consistent.

**CONSEQUENCE:** An operator can cut and store a seed plate that does not restore the payload's wallet, with nothing on the plate contradicting it. It takes deliberate typing to reach, which is why this is Minor and not Critical.

**FIX:** Either add an `Editable bool` to `SeedScreen` (default true, false here) and skip the `editBtn` slot when false, or state in §6b why edit is acceptable on a payload-sourced seed. Do not leave it unremarked.

---

## [M6] `Deriver.Wipe()` leaves `done == total`, so a post-`Wipe` `Key()` returns an all-zero 32-byte key

**WHERE:** Plan §3b, `Wipe` and `Key`.

**CLAIM:** "It returns nil while the derivation is incomplete: a partial accumulator is not a short key, it is the wrong key"

**DEFECT:** `Wipe` clears `u` and `acc` but leaves `done` and `total` untouched, so `Key()` still passes its `d.done < d.total` guard and returns `append([]byte(nil), d.acc[:]...)` — 32 zero bytes. `seal/crypto.go:47-52` states the governing rule for exactly this: *"An all-zero key would be worse — it is a VALID AES key and hides the fault."* Not reachable today (in `unlockDerive` the return value `d.Key()` is evaluated before the deferred `d.Wipe()` runs), but the type is now a public seam and B2b will hold one across a timer.

**CONSEQUENCE:** A future caller that queries `Key()` after `Wipe()` gets a valid AES-256 key of zeros and a tag mismatch indistinguishable from a wrong passphrase.

**FIX:** One line in `Wipe`: `d.done = 0` (and a line in `TestDeriverWipeLeavesTheReturnedKeyIntact` asserting `d.Key() == nil` after `Wipe`, which also pins it).

---

## [N1] The described gui tests must pump ~200+ frames through the chunked KDF

`unlockDerive` draws one frame per `kdfStepIterations = 500`; every vector that carries a key uses 100,000 or 100,001 iterations, so a test driving a full unlock needs ≥ 200 frames before the post-KDF screen exists. The repo's idiom is `pumpUntil(frame, want, 32)` (`gui/slip39_polish_test.go:329`). Fails loudly rather than falsely, hence a Nit — but worth one line in Task 5.3 so it is not diagnosed as a hang.

## [N2] `unlockKDFLead` truncates before it multiplies

`left := time.Duration(int64(elapsed) / int64(done) * int64(total-done))` divides to whole nanoseconds first. On the device the error is under a millisecond; on a fast host `elapsed/done` can round to 0 and the screen reads "About 0 seconds left." Cosmetic, and only in tests. `int64(elapsed) * int64(total-done) / int64(done)` overflows `int64` only past ~10^10 ns of elapsed time, so the reordering is safe.

## [N3] "one line per call site" understates the `runUnlock` conversion

§2d says converting `runUnlock` to a `seal.Reader` is "one line per call site" because `payloadReaderFor` produces one from a vector name. Two existing call sites pass blobs that no vector name can produce — `gui/unlock_flow_test.go:171-172` (`tc.mangle(sealVectorBlob(t, "E"))`) and `:195`. Simplest resolution: keep `runUnlock(t, blob []byte)` and have it write the bytes to `t.TempDir()` and hand `unlockPayloadFlow` a `seal.FileReader`; then it really is one edit.

---

## What I checked and found sound

- **`passphraseBytes` ≡ §8.1.** Executed against the real packages: for the canonical `beef`×12 passphrase, `passphraseBytes(m)` is byte-identical to `seal.NormalisePassphrase(m.String())` (`"beef beef … beef"`, 59 bytes), and the longest label in the shipped wordlist is 8 bytes, so the worst-case 12-word buffer is 107 — the fixed `cap` of 128 never regrows. The single highest-consequence claim in the plan holds.
- **`Deriver`'s decomposition on inputs other than the vectors.** `Step` past completion is a no-op (`d.done < d.total` guard); `Key()` before completion returns nil and `unlockDerive` only calls it after `Step` reports true; `d.mac.Write(d.u[:])` precedes `d.mac.Sum(d.u[:0])` so the in-place overwrite is ordered correctly under `crypto/hmac`'s two-phase `Sum`; `iterations < 1` is clamped rather than panicked; `Done()*100/Total()` peaks at 2×10⁸ under §6.2's cap and cannot overflow a 32-bit `int`. The only residue is M6.
- **§1f's test inversion is legitimate, not the anti-pattern.** `TestEncryptedRecordsCarryNoGrouping` (`seal/grouping_test.go:103`) says in its own comment that it pins "the trap Phase B2 inherits (F-77)" and exists to make a *deficiency* "a measured fact rather than a recollection". The invariant beside it, `TestGroupingRunsAfterTheAllowList`, is untouched, and `labelEncryptedCards` runs after the pass-1/pass-2 loop exactly as pass 3 does. The premise check (12 cards of 15) is correctly retained.
- **The secret/non-secret split.** `IsSecret = ClassCodex32Secret || ClassMnemonic` matches §6.3's table verbatim ("`md1` and `mk1` carry public data … The secret half of the constellation is `ms1`, and a raw BIP-39 mnemonic is equally secret") and §11.2's vector-F requirement (three `ms1` offered first, twelve `mk1`/`md1` as ordinary plates).
- **`labelEncryptedCards`' reuse of `labelCards`.** `labelCards` indexes `out[i]` over `g.perRecord`, so a subset slice of `len(strs)` is correctly labelled in its own coordinates and scattered back; `PlateTotal` comes from `len(g.groups[k])` and `CardIndex/CardTotal` from `g.keys`, all within the subset. No `§6.3` logic is re-derived.
- **`AdmitSection` copies each record** (`append([]byte(nil), r...)`), so `p.Public[i].Record` does **not** alias `blob` — §5d's `clear(blob)` before the plate list is safe and does not zero the public records.
- **The KDF frame loop's wakeup ordering.** `Run` reads `ctx.Wakeup` inside the `Frame` call and `ctx.Reset()` zeroes it afterwards, so `ctx.WakeupAt(time.Now())` placed *after* `ctx.Frame` correctly gives the next `AppendEvents` a past deadline; the loop does not block on input.
- **`ChoiceScreen` Skip is reachable on hardware.** `Draw` registers `op.Input(&ctx.B, &c.click).Clip(bg)` per choice (`gui/gui.go:1521`), so "Skip" is tappable on a device with no directional buttons. `Choose` returns `(0,false)` on both Cancel and `ctx.Done`, which the plan's `if !ok || choice != 0` handles.
- **§10.2 step 10 coverage.** Derived key (`defer clear(key)`), passphrase buffer (`defer clear(pass)`), mnemonic (`clear(m)`), PBKDF2 intermediates (`defer d.Wipe()`), whole payload (`defer p.Wipe()`) — every clause has an owner, with the hmac ipad/opad caveat stated honestly.
- **`UnlockWithKey` is a faithful split of `Unlock`.** Offsets, the `len(blob) < end` re-check, AAD `blob[:split]`, ciphertext-with-tag `blob[split:end]`, the cross-section `MaxRecords` check on `p.nPub + nSec`, and the wipe of a previous `p.Secret` all match `seal/open.go:176-243` line for line.
- **`unlockSealedFlow` never falls through to the plate list**, `ErrTooManyRecords` stays distinguishable from "unreadable" (§6.4), and `ErrRecordNotPermitted` lands in the `default:` "Payload unreadable" arm as §10.2.1 requires.
- **`uiContains` anchoring.** It lowercases and strips spaces from the needle only, against space-stripped extracted text; `", SEALED):"` → `",sealed):"` is genuinely not a substring of `",unsealed):"`, so Task 5.6's retry-body anchor does discriminate. The B1 `"SEALED"`/`"UNSEALED"` trap is avoided.
- **Citations.** Spot-checked 29 `file:line` references against the fork at `78949e7`, including every one in "Carried-forward citations that have DRIFTED"; all resolve **and say what the plan claims** (`gui/gui.go:2879` `idleTimeout`, `:983` `NewKeyboard`, `:2644/:2661/:2707` `Engrave`, `:1595` the dispatch case, `seal/record.go:214` the `SectionPublic` gate, `gui/slip39_polish.go:342` `wipeBytes`, `gui/unlock_platelist.go:50` the `plateLabel` fallback, `assets.IconDiscard`). The gate's blind spot was checked by hand where the claim is load-bearing and nothing further was found.
- **Vector metadata**, read from `seal/testdata/vectors.json`: A(0/1,100000) B(0/1,100001) C(0/6) D(5/1) E(5/0,unsealed) F(0/15) G(12/3) — consistent with every count the plan states.
