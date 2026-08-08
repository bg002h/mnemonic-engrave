# Phase B2a plan — R0 round 1 (independent fold review, opus)

Artifact: `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2a.md` at commit `7e52a1e`.
Scope: **did the fold fix each of round 0's 14 findings, and did it introduce a new defect?** Not a fresh audit.
Primary object: `git diff 80d4c33..7e52a1e`.
Fork under review: `/scratch/code/shibboleth/seedhammer` @ `78949e7`.
Persisted verbatim, before any fold. HTML entities from the transport layer have been restored to their literal characters; nothing else is altered.

---

VERDICT: 0 Critical / 2 Important / 4 Minor / 3 Nit

| finding | fixed? | note |
| --- | --- | --- |
| C1 | **YES** | `clear(rec)` lands on **both** paths after plate construction. Verified in the fork: `gui/gui.go:2633` passes `plate.Spline`/`plate.Conf` into `newEngraverJob`, `gui/engraver.go:170` iterates `e.spline`, and neither path aliases `rec` (`string(rec)` and `bip39.Parse(rec)` both copy), so the wipe cannot break the plate, a retry or a resume. The quoted `gui/gui.go:2651-2656` block matches the fork byte-for-byte. `unlockSecretPlate`'s defer is still registered before any return; zeroed records never reach `unlockPlates`, which filters `Class != ClassMDMK`. No new hole. |
| I1 | **YES** | `defer func() { clear(blob) }()` is the correct closure form; the "no-op on a nil slice" sentence is gone; the new `seal/record.go:207` non-aliasing citation is exact. |
| I2 | **PARTIAL** | `newDeriver` is real and genuinely in the path (`unlockDerive`), and kills both §11.3 mutants. But the plan text I2 quoted is unchanged (→ M1) and `unlockPassphraseHook` is scheduled by nothing (→ **I2 new**). |
| I3 | **YES** | **Executed**: `sealBlobForTest` round-trips through the real `Opener.Inspect` + `Unlock`, sealed and unsealed, and produces the both-sections shape no vector can. One stale reference left (→ M3). |
| I4 | **YES** | Every measured value in the new §1e comment reproduces exactly, including `"md1t7yjcvgk6xetg"` (16 bytes) and both error strings. |
| M1 | **YES** | `end` stays with its guard; `nPub`/`split` go. |
| M2 | **YES** | All three rows repointed, and the two new killers check out (widened `IsSecret` → vector F yields 15 offers not 3; `unlockPlates` filters on `ClassMDMK`, not `IsSecret`). |
| M3 | **YES** | All three consequences scheduled in Task 7.2; `unlock_platelist.go:173-181` and `unlock_platelist_test.go:126` both resolve to exactly what is described. |
| M4 | **YES** | `showNotice(ctx, th, title, msg)` exists at `gui/slip39_polish.go:44` and blocks until dismissed via `showModal`'s loop. Title stays `""`, so `"Word %d of %d"` survives; the notice says passphrase, not seed. |
| M5 | **PARTIAL** | `NoEdit` added, zero value editable, split flagged loudly. The rationale and the only named test are wrong (→ **I1 new**). |
| M6 | **YES** | `d.done = 0`; post-`Wipe` `Key() == nil` asserted. |
| N1 | **YES** | Frame budget stated (→ N2 on the alternative). |
| N2 | **YES** | Multiply-before-divide. |
| N3 | **YES** | `runUnlock` keeps `[]byte` + `t.TempDir()` + `seal.FileReader` (exists, `seal/read_host.go:21`). |

---

## [I1] §6c's `NoEdit` test cannot fail, and the fix rests on a false claim about how the editor is reached

**WHERE:** Plan §6b comment and §6c. Fork: `gui/gui.go:2299`, `:2330-2333`, `:2467`; `gui/widget.go:70`; `gui/event.go:140-162`, `:296-330`; `cmd/controller/platform_sh2.go:398-417`.

**CLAIM:**
> §6c: "Note `editBtn` carries `AltButton: Center`, so on a touch-only SH2 a **centre tap** opens the editor — this is not a hard-to-reach affordance."
> §6c: "**Test:** with `NoEdit` set, tapping the centre of the seed screen does not reach word entry; with it clear, it still does (the existing scan-path behaviour, which must not regress)."
> §6c: "and where `editBtn` is added to the nav slots, skip it when `s.NoEdit`."

**DEFECT:** A centre tap does not open the editor, with or without `NoEdit`.

- SH2 production emits **only** `gui.PointerEvent` (`platform_sh2.go:398-417`); there is no `gui.Button` producer in it. `Center` is a `Button`, reachable only via `ButtonFilter`, and the only source in the fork is `cmd/controller/debug_sh2.go:70` (serial Return).
- Pointer events reach a `Clickable` solely by hit-test against a drawn `op.Input` region (`event.go:146` requires `e.Tag == f.tag`; `e.Tag` comes from `d.Hit(pe.Pos)` at `:311`). `editBtn`'s only `op.Input` region is the nav slot at the right edge, middle row.
- The centre of the seed screen is the word list, which registers `op.Input(&ctx.B, &s.words[i])` (`gui.go:2467`). Tapping there sets `s.selected` — it never reaches `inputWordsFlow`.

So the test's negative half is **vacuously true today**, with no change at all, and its positive control ("with it clear, it still does") **cannot pass**. Separately, `Filter.matches` gates `buttonEvent` on button identity alone with no bounds (`event.go:155-159`), so dropping the nav slot leaves `if editBtn.Clicked(ctx)` (`gui.go:2330`) still consuming `ButtonFilter(Button2)` and `ButtonFilter(Center)` — the guard does not cover the button route the plan itself names as the threat.

**CONSEQUENCE:** §6c names exactly one test for the M5 fix, and it passes whether or not `NoEdit` is implemented, wired, or honoured — the plan's own "Do not ship without it" has no assertion behind it. §6c is also the one section the build gate explicitly does not cover, so nothing else catches it. An implementer who takes "skip the nav slot" literally ships a `NoEdit` that any `Button2`/`Center` event walks straight past.

**FIX:** Two lines. Replace the rationale with what is true — the edit affordance is the **nav button** (`Button2`, right edge middle slot, or a tap on it) — and make the guard the click handler, not the layout: `if !s.NoEdit && editBtn.Clicked(ctx) { … }` (keep the nav-slot skip so the icon disappears too). Restate the test as: with `NoEdit` set, neither a tap on the edit nav slot nor `press(&ctx.Router, Button2)` reaches word entry; with it clear, both still do.

---

## [I2] `unlockPassphraseHook` is dead — no step writes its test, and Task 8's `ct_len == 0` row still names no instrument

**WHERE:** Plan §5c (`gui/unlock_kdf.go`), Task 5.3, Task 8's mutation table (line 2323).

**CLAIM:**
> §5c: "`unlockPassphraseHook` fires when the word-entry screen is ENTERED, and exists for one required negative: §11.2's 'Vector E reaches the plate list with the keyboard flow NEVER ENTERED — asserted by instrumenting the prompt entry point, not by return value.'"
> Task 8: "| passphrase prompted when `ct_len == 0` | vector E reaches the plate list with the word entry **never entered**, asserted by instrumenting the entry point, not by return value |"

**DEFECT:** Round 0's I2 fix required both halves: "Apply the same treatment to Task 8's … row, **which has no named entry-point instrument either**." The fold added the hook and stopped. Grepped over the whole folded plan, `unlockPassphraseHook` appears at exactly three places — its declaration, its doc comment, and its call site (`:1398`, `:1405`, `:1416-1417`). **No task step writes a test that uses it**, and Task 8's row is unchanged: it still describes a test rather than naming one, and names no instrument. Task 5.3's step list is "checksum gate with a KDF counter; the retry loop keeping the hash on screen; cancel never reaching the plate list" — the vector-E negative is not among them.

**CONSEQUENCE:** Task 8's own preamble says "**every one must name the test that kills it** — a mutant with no named killer is a gap in the suite, not a passing result", and 8.2 makes a surviving mutant blocking. B2a therefore reaches its final gate with a mandatory §11.3 row backed by nothing, and the hook ships as unused production-visible state. This is the same shape as the two defects it was created to prevent: an instrument that exists but is not in any test's path.

**FIX:** Add to Task 5.3: "write the vector-E negative — `unlockPassphraseHook` set, run the unsealed flow to the plate list, assert the hook never fired," and repoint Task 8's row to it by name.

---

## [M1] Global Constraints still says Task 3 adds the KDF seam, and its 31-second premise now contradicts the fold's own §5e correction

**WHERE:** Plan Global Constraints / "B2a-specific" (lines 136-139) vs. §5e (lines 1705-1709).

**CLAIM:**
> Global Constraints: "`Opener.KDF` (`seal/open.go:24`) is the sanctioned seam; **Task 3 adds the equivalent seam for the chunked path**. The whole `gui` suite is ~12 s today (measured); one real 100,000-iteration derivation would nearly triple it."
> §5e (new): "the 'low-iteration blob' motivation was wrong: the *host* derives 100,000 iterations in tens of milliseconds — the ~31 s figure is device-only."

**DEFECT:** This bullet is the text round-0 I2 quoted verbatim as its false CLAIM, and the fold left it untouched. Task 3 adds no seam; §5c (Task 5) does, and it is `newDeriver`, not an `Opener.KDF` equivalent. The second sentence is now measurably false by the fold's own §5e text — both cannot be true.

**CONSEQUENCE:** The plan contradicts itself on a bullet labelled binding. An implementer at Task 3 looks for a seam that is two tasks away, and the "must not gain 31 s per test" constraint argues against the real-floor fixture §5e mandates.

**FIX:** Rewrite the bullet: the chunked path's seam is `newDeriver` in §5c; the host derives 100,000 iterations in tens of milliseconds, so the gui cost is the ~200 frames `kdfStepIterations` implies, not the KDF.

---

## [M2] "`SecretsResident()` goes false as the cut starts" is false for every multi-secret payload, and contradicts §10.2.2's own cost paragraph

**WHERE:** Plan C1 section; §5b `SecretsResident`. SPEC §10.2.2 "What this costs".

**CLAIM:**
> "Residency collapses from 'the whole ~21-minute cut, plus indefinitely on a paused or failed screen' to '**the few milliseconds** between decrypt and plate construction, plus however long the Cut/Skip choice is on screen'."
> "`SecretsResident()` **now goes false as the cut starts**, so §10.2.4's residency key means what it says instead of staying true for the entire engrave."

**DEFECT:** `SecretsResident` (§5b) scans **every** secret record, and `unlockSecretSession` wipes them one at a time as each is offered. Vectors F and G each carry **three** `ms1` records (measured from `seal/testdata/vectors.json`). During plate 1's cut, records 2 and 3 are untouched, so `SecretsResident()` is `true` and residency is ~21 min × (N−1) — which is precisely what SPEC §10.2.2 states: "Cutting the secrets first collapses that to the first *N* plates: ~21 minutes for single-sig, **~63 for a 2-of-3**." Both new sentences are true only for a single-secret payload, and §10.2.2 makes a point of plurality being load-bearing.

**CONSEQUENCE:** B2b is told the residency key goes false as a cut starts and will design the §10.2.4 timer against that; it does not, on exactly the multisig payloads the SPEC calls out.

**FIX:** "`SecretsResident()` goes false as the **last** secret's cut starts; earlier cuts still hold the not-yet-offered records, per §10.2.2's cost paragraph. What the early wipe removes is the *cutting* record's residency — one plate's worth, not the session's."

---

## [M3] Task 7.1 still calls for `sealForTest`, the symbol I3 established is unreachable from `package gui`

**WHERE:** Plan Task 7.1 (line 2271).

**CLAIM:** "a `sealForTest` payload with cards in **both** sections (the `(sealed)` suffix appears…)"

**DEFECT:** The fold added `sealBlobForTest` and rewrote §5e, but left the task step that consumes it naming the unreachable symbol. Task 7.1 is where the implementer actually builds this fixture.

**CONSEQUENCE:** A compile error at the exact step I3 predicted, then a hunt back through §5e. Fail-loud and cheap, hence Minor — but it is the finding's own consumer.

**FIX:** `sealForTest` → `sealBlobForTest` at line 2271.

---

## [M4] §6a's shipping comment still states the pre-C1 price, without the resume carve-out

**WHERE:** Plan §6b's whole-file block, `gui/unlock_session.go` header comment.

**CLAIM:** "aborting mid-plate to re-seat shifted steel is the machine's most ordinary recovery, and keying on completion would leave the seed resident in a state nothing guards. **Re-cutting then needs a fresh unlock — twelve words and a ~31 s KDF.**"

**DEFECT:** The C1 section now establishes the opposite for the case this sentence names: a plate aborted mid-cut **resumes without a fresh unlock**, because the job holds the spline. Only leaving the engrave screen costs twelve words. The comment is the text that ships into the fork and is what a B2b reader will find; the correction lives only in the plan.

**CONSEQUENCE:** Exactly the stale-comment class §1d spends a task fixing and M3(c) fixes for `unlockEngraveFlow` — a shipped comment telling the next phase the opposite of what B2a decided.

**FIX:** Append one clause: "…a fresh unlock, once the engrave screen is left. A plate merely *paused* resumes from the spline — the record was zeroed before the plate reached the screen, so there is nothing left to re-protect."

---

## [N1] The C1 section misquotes `NewEngraveScreen`'s call

`gui/gui.go:2633` is `newEngraverJob(ctx.Platform, plate.Spline, plate.Conf, 0)`. The plan writes `…, opts)`. Round 0 quoted it correctly; the fold changed it to the parameter name. Cosmetic, but the surrounding paragraph is the C1 argument's evidence.

## [N2] Task 5.3's "better" alternative cannot reach a successful unlock

"have the counting `newDeriver` return a deriver over a small iteration count" yields a key that disagrees with the header's iteration count, so `UnlockWithKey` returns `ErrAuthentication` and the flow can never reach the plate list. Harmless for the three tests 5.3 names (all want a failure or a count), but the plan does not confine it, and Task 7's `(sealed)` test needs a successful unlock. Fails loudly, hence a Nit — one clause ("only for tests that do not need the unlock to succeed") closes it.

## [N3] §5e assigns the fixture round-trip to Task 5.1; Task 5.1's checklist omits it

The blockquote says "Task 5.1 asserts it round-trips through `seal.Opener.Inspect` + `UnlockWithKey` before any test depends on it", but 5.1's step list is `UnlockWithKey`/`session.go` only. (I ran that round-trip myself and it passes — see below — so this is bookkeeping, not risk.)

---

## What I executed, beyond the stated gate

Against a scratch copy of the fork at `78949e7` with go1.26.3:

- **`sealBlobForTest` verbatim from §5e, sealed**: `Inspect` accepts, `Unlock` decrypts, 3 secret records returned, first classified `codex32 secret`, plaintext byte-identical. Public `mk1`×2 + encrypted `ms1`/`mk1`/`md1` — i.e. the both-sections `(sealed)` shape I3 said no vector can supply is producible. 528-byte blob.
- **`sealBlobForTest` unsealed**: `Inspect` accepts, `Sealed()` false, 257-byte blob.
- **§1e's measured claims**: `codex32.AssembleMD1(make([]byte,0))` = `"md1t7yjcvgk6xetg"`, 16 bytes; `ValidMD` true; `Classify` = `ClassMDMK`; `ParseChunkHeader` = `md: bit stream truncated`; `cardKey` = `…record 0: md: bit stream truncated`. Every value in the comment is exact.
- **Vector metadata** re-measured from `vectors.json`: A(0/1) B(0/1) C(0/6) D(5/1) E(5/0) F(0/15) G(12/3); C/F have `pub_len == 0`, D/G have `ms1`-only secrets. Every §5e claim holds.

**Bottom line:** the C1 rewrite — the largest and most dangerous change — is correct, and I found no hole it opens. The two Importants are both places where the fold did half of a finding's fix: M5's guard has no test that can fail, and I2's second instrument was created but never scheduled.
