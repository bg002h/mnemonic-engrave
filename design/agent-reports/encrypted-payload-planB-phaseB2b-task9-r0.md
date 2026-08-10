# B2b Task 9 — R0 review: arming the wipe during passphrase entry

- **Reviewer:** independent R0 agent (Opus 5), 2026-08-09
- **Artifact:** `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2b.md` **Task 9** at `a950c66`
- **Code:** `/scratch/code/shibboleth/seedhammer-b2b`, branch `b2b` at `a73191a` — **read only, untouched** (`git status` clean; all measurement ran in a scratch copy)
- **Also read:** `design/agent-reports/encrypted-payload-planB-phaseB2b-wipe-inventory.md`, `design/SPEC_encrypted_payload_delivery.md` §10.2.4, `design/HARDWARE_RESULT_2026-08-09_phaseB2b.md` §8.1a
- **Method:** every claim below that a tool can check was **run**, not read. Four checks with positive controls, two of them mutation-killed. Verbatim output in §5.

---

## VERDICT — **1 Critical, 3 Important**

**Task 9 as written would produce a wrong fix.** Its §"The change" — *"Install the `wipeGuard` at the top of the sealed flow … and uninstall it on the same defer discipline"* — arms §10.2.4's timer across the **key derivation**, and that is not a UX wrinkle. Measured: an armed derivation is **parked by its own wipe warning** and then wiped, so **34.6 % of §6.2's legal iteration range becomes permanently un-openable on the device**. Task 9 raises this as open question 2 and calls it "probably the right answer" for the *disarm* — it is not "probably", it is forced, and the plan's prescriptive §"The change" contradicts it.

Question 1's premise is also false — the complete-but-unsubmitted **park does not exist** — which makes step 9.3's second test unwritable as specified.

The core insight is right and the fix is genuinely small. It is just **one seam to the left** of where the plan puts it.

| # | Sev | Finding |
|---|---|---|
| C1 | **Critical** | Arming across the KDF wipes any derivation ≳ 1,343,284 iterations, deterministically. §"The change" must not span `unlockAttemptOnce`. |
| I1 | **Important** | The warning reads "This machine still holds **decrypted seed material**" — false at passphrase entry. Task 9 has no step for it. |
| I2 | **Important** | Step 9.3's "complete-unsubmitted entry" test pins an unreachable state; writing it requires fabricating one, i.e. a false PASS by construction. |
| I3 | **Important** | Step 9.5 sends the operator into the open re-entry Critical on a one-trip budget, with no precondition and no split of whose failure is whose. |

---

## 1. The complete-but-unsubmitted path — **the state does not exist**

**Answer: there is no such park, and `m` is zeroed on the route that actually exists — by `unlockSealedFlow`'s unconditional `clear(m)` at `gui/unlock_kdf.go:384`, not by the `!m.Valid()` branch and not by the partial-entry `clear(m)`.**

### The trace

A wipe can only be raised from Run's event loop, which runs **only inside `ctx.Frame`**: `Context.FrameCallback` has exactly one call site, `Context.Frame` (`gui/gui.go:85`), and Run's inner loop lives in the range body over `it`, which resumes only when the flow yields there (`gui/run_flow.go:112-230`). So every stretch of frame-free straight-line code is atomic against the wipe — the same argument `gui/wipe_guard.go:12-14` already makes for the session bracket's edges.

Between the twelfth word and the KDF there is no frame:

| step | file:line | draws a frame? |
|---|---|---|
| `mnemonic[selected] = w`; `selected++`; `selected == len(mnemonic)` → `return` | `gui/gui.go:733-738` | no |
| `isMnemonicComplete(m)` → true; `m.Valid()` → true; `return m, true` | `gui/unlock_kdf.go:132-145` | no |
| `unlockAttemptOnce`: gate, `passphraseBytes(m)`, `defer clear(pass)` | `gui/unlock_kdf.go:327-332` | no |
| `unlockDerive`: `newDeriver(...)`, `defer d.Wipe()`, `backBtn.Clicked`, `d.Step(500)` | `gui/unlock_kdf.go:210-238` | no |
| **first `ctx.Frame`** | `gui/unlock_kdf.go:304` | **yes — already the KDF screen** |

**Measured (Check A).** A real vector-D entry draws **263 frames**. Frames parked with `isMnemonicComplete(m) == true` **before** the KDF: **0**. After: **200** — the KDF's own progress frames, with `m` still live. Both positive controls fire (the KDF ran; a complete mnemonic *was* observed), so "never before" is not vacuous.

### So what *is* the walk-away window with a complete passphrase?

**The KDF progress screen** — 40.2 s at the default, up to **4:28** at §6.2's ceiling. That is the state question 1 is really about, and it is covered:

- `m` → `clear(m)`, `gui/unlock_kdf.go:384`, unconditional, before the switch.
- `pass` (§8.1 normalised bytes) → `defer clear(pass)`, `:331`.
- `key` → never assigned on this route (`unlockDerive` returns `nil`); `defer clear(key)` at `:337` covers the other one.
- `Deriver.u`/`acc` → `defer d.Wipe()`, `:214`.

**Measured (Check C).** A `ctx.Done` unwind raised mid-derivation leaves the typed `[]Word` **all zero** and the normalised passphrase bytes **all zero**, read off the same backing arrays the flow used. **Mutation-killed**: turning `:384`'s `clear(m)` into `_ = m` fails it with the twelve words verbatim.

**Measured (Check D).** The partial-entry route Task 9 already claims works, does work: six words typed, `ctx.Done`, buffer all zero. **Mutation-killed** at `:133`.

### One residue Task 9 must not claim away

`m` is zeroed; the **in-progress word** is not. `kbd.Fragment` and `wordLabel` (`gui/gui.go:673, 706-720`) are Go strings rebuilt per keystroke and no wipe reaches them — F-104 §1c owns this, phase B2c. 9.3 may assert "the typed **word buffer** reads zero", never "the typed words are gone".

**Consequence for the plan → I2.** Step 9.3's *"for **both** a partial and a complete-unsubmitted entry"* cannot be honoured. An implementer who tries will pre-fill `m` and drive `unlockPassphraseFlow` directly, pinning a state the product cannot enter — a test that passes for a reason unrelated to the shipped behaviour. **Replace it with the mid-derivation wipe**, which is the real complete-passphrase window; Checks C and A in §5 are that test, already written and already mutation-killed.

---

## 2. Arming across the KDF — **it must disarm. Measured, not argued.**

**Recommendation: the timer MUST NOT run during a derivation.** Not because a wipe there is annoying — because it is **unsurvivable**, and it takes a third of §6.2's legal range with it.

### Why it is worse than the plan says

Task 9 predicts "the wipe warning mid-derivation". It misses what the warning branch *does*. `gui/run_flow.go:201-221`: when armed and idle, Run draws the warning and **`continue`s the inner loop** — it never returns control to the flow's blocked `ctx.Frame`. **The derivation is frozen for the whole 30 s window**, so it can never finish inside it. Once the warning starts, the wipe at 3:30 is not a risk, it is a certainty. This is F-93's park, re-created by the fix that was supposed to be small — one task after Task 5 closed it.

And it cannot be fixed by letting `KeepAwake` win: `TestRunKeepAwakeCannotPostponeAnArmedWipe` (`gui/run_flow_test.go:697`) exists specifically to kill the `(ctx.keepAwake && !armed)` → `(ctx.keepAwake)` mutant, and its doc calls `&& !armed` *"normative, not caution"*. The only conforming route is `armed() == false` during derivation.

### Measured (Check B)

The Task 5 test `TestRunKeepAwakeDuringDerivationDoesNotParkUnderTheScreensaver`, changed by **one line** — `ctx.wipe = &wipeGuard{}`, exactly Task 9's proposal:

```
sessions=2 derivedOK=false keyLen=0 frames=211 warningFrames=30
```

Thirty warning frames — `WIPING SECRET DATA`, drawn with **nothing decrypted** — then the wipe, then a fresh session. Control (Check B2, guard absent, same derivation): `derivedOK=true`.

### The hardware numbers

`design/HARDWARE_RESULT_2026-08-09_phaseB2b.md` §8.1a: 300,000 iterations = **40.2 s ± 1.0** wall, corroborated by the device's own on-screen estimate (40 s). Wall rate **7,463 it/s**. `idleTimeout` is wall time (`gui/gui.go:2949`), so:

| | |
|---|---|
| iterations whose derivation reaches 3:00 | **1,343,284** (180 s × 7,463) — band 1.31 M – 1.38 M over the ±1.0 s reading |
| §6.2 legal range (`seal/wire.go:36-37`) | 100,000 – 2,000,000 |
| **share of the legal range wiped, deterministically, every attempt** | **34.6 %** (32.8 – 36.3 % over the band) |
| ceiling (2,000,000) wall time | 268 s = **4:28** |

`me seal --iterations 1500000` is a conforming payload. Under Task 9 as written the device **can never open it** — not "sometimes", not "if the operator walks away": the KDF produces no events, so the clock runs from the last keystroke every single time. That is an unmet §6.2 guarantee, hence **Critical**.

### Why disarming is safe

A derivation is a **machine-driven operation in progress**, the same category §10.2.4 row 2 already exempts — and a strictly weaker exemption: bounded by §6.2's ceiling at ≤ 4:28 and **self-terminating**, where an engrave is ~21 min and needs the operator to leave the screen. The window also closes into a covered state by construction: on success `unlockSecretSession` arms (`gui/unlock_session.go:82-84`) and Run's arm-edge resets the clock (`gui/run_flow.go:153-171`), giving a fresh 3:00 from the derivation's end; on failure `clear(m)` at `:384` has already run, so the retry screen holds nothing wipeable.

Against that: wiping mid-KDF costs twelve words of typing **and** up to 4½ minutes of derivation, protects material for at most those 4½ minutes, and — as measured — makes a third of the legal parameter space unusable.

---

## 3. What a wipe with nothing decrypted should do

**Answer: exactly what every other §10.2.4 wipe does — and it already does it.** No new behaviour is needed; only the spec's *scope* is wrong.

Traced: `ctx.Done` → `inputWordsFlow` returns (`gui/gui.go:713`) → `unlockPassphraseFlow` `clear(m)` + `return nil, false` (`:132-135`) → `unlockSealedFlow` returns false (`:376-380`) → `unlockPayloadFlow` returns, running `defer p.Wipe()` (`gui/unlock_flow.go:85`) and the `clear(blob)` **closure** (`:58`) → Run's session loop scrubs `ctx.B` (`gui/run_flow.go:245`) and restarts at the main menu. **Nothing in flash is touched** — the region is only ever `Read()`, never written — **and no attempt is consumed**: there is no attempt counter, and the retry loop is per-session. Reopening costs the passphrase and the KDF, the price §10.2.2 already charges.

One behaviour worth naming so it is not later mistaken for a bug: during the warning window `a.idle.active` gates `ctx.Router.Events` (`gui/run_flow.go:174`), so the touch that dismisses the warning is **swallowed rather than typed**. On the keyboard that is the correct outcome, and it is the existing screensaver-dismissal behaviour (`:162-169`).

### The §10.2.4 amendment, drafted

Structure preserved: the two rows are **appended, not inserted**, so every existing "row 1" / "row 2" / "the third row" reference — in this spec, in `gui/wipe_guard.go:36`, in `gui/unlock_session.go` and in the plan — stays correct.

**(a) Amend row 3's condition** (it would otherwise contradict the new row 4):

> | **no** secret record resident **and no passphrase in flight** | **none** | Public data only. Nothing to protect. |

**(b) Append two rows:**

> | **4. a passphrase is being typed** — §10.2 step 5's keyboard, before any unlock | **3 min, 30 s warning** | An in-flight passphrase derives the key that opens everything, so it is **seed-equivalent** (operator ruling 2026-08-09). Twelve words on a touch keyboard is the longest manual step in the flow and the likeliest place to be interrupted — with the sealed blob in flash beside them. |
> | **5. a key derivation is running** — §10.2 step 7 | **paused** | Row 2's rule, same reason: an operation the machine is performing is not idleness, and the operator cannot shorten it. Strictly weaker than row 2 — bounded by §6.2's ceiling (≤ 4:28 at the measured 7,463 it/s wall) and self-terminating, where a cut is ~21 min and needs the operator. |

**(c) Append the prose:**

> *(Amended 2026-08-09b.)* **The timer's subject is seed-equivalent material, not decrypted records.** As first written this section scoped the timer to resident *records*, which left the entry keyboard outside it — nothing has been decrypted there. The operator ruled that an in-flight passphrase **is** seed-equivalent: it derives the key that opens everything, and the machine holds it beside the sealed blob it opens. Rows 4 and 5 are that ruling. Row 4's wipe does exactly what every other row's does — `ctx.Done` unwinds the flow, every deferred `clear` runs (the typed `[]Word`, §8.1's normalised bytes, the derived key, the Deriver's accumulators), and `Run` restarts the UI at the main menu. **Nothing in flash is touched and no attempt is consumed**: the payload is exactly as it was, and reopening costs the passphrase and the KDF.
>
> **Row 5 is a bound, not a preference.** `Run` parks the flow for the whole 30 s warning (`gui/run_flow.go:201-221` draws and `continue`s without returning control), so a derivation that reaches 3:00 can never finish and the wipe becomes certain. At the measured wall rate that is every payload above **~1,343,284** iterations — **34.6 %** of §6.2's legal 100,000–2,000,000 range, permanently un-openable on the device. `ctx.KeepAwake()` must **not** be the remedy: "KeepAwake can never postpone an armed wipe" is normative (row 2's own guarantee) and is pinned by `TestRunKeepAwakeCannotPostponeAnArmedWipe`. Row 5 is implemented as the **passphrase bracket closing before `unlockAttemptOnce` is called**, not as a second flag on the guard.
>
> **The warning must name what is actually at risk.** Row 1's text — *"This machine still holds decrypted seed material"* — is **false** under row 4, and telling an operator that on a screen they know they have not unlocked is the fastest way to teach them the warning is furniture, the same reasoning §10.2 step 3 uses to refuse a constant hash. Two texts, one per subject; the countdown, the touch-to-keep affordance and the 3:00/3:30 schedule are unchanged.

---

## 4. Is the top of the sealed flow the right seam? — **No. Bracket the keyboard.**

Installing at `unlockSealedFlow`'s top arms five screens, and exactly one of them holds anything a wipe can reach:

| screen | site | resident & wipeable? | verdict |
|---|---|---|---|
| passphrase notice | `unlock_kdf.go:375` | nothing typed yet | pointless |
| **word keyboard** | `unlock_kdf.go:128` | **`m`, up to 11 words** | **the target** |
| "Not a valid passphrase" | `unlock_kdf.go:142` | `clear(m)` ran at `:141` | pointless |
| **KDF progress** | `unlock_kdf.go:304` | `m`, `pass`, Deriver | **C1 — must be excluded** |
| 4 × retry/terminal errors | `unlock_kdf.go:391-403` | `clear(m)` ran at `:384` | pointless, **and costly** |

The last row is not merely wasted: the `seal.ErrAuthentication` screen is the one carrying the **§6.6 public-data hash** the operator is instructed to compare against a written record (`unlockRetryBody`, `:351-361`). Wiping it at 3:30 destroys the comparison mid-comparison and charges twelve words plus a KDF to see the number again. Arming it protects nothing.

**Not affected either way:** the public-data-hash notice (`unlock_flow.go:92`) sits in `unlockPayloadFlow`, *before* `unlockSealedFlow`. Do **not** hoist the guard there to catch it — that would also arm `unlockPlatesOrNotice`, which holds public records only, contradicting §10.2.4 row 3 head-on.

### Recommended shape

```go
func unlockPassphraseFlow(ctx *Context, th *Colors) (bip39.Mnemonic, bool) {
	// §10.2.4 row 4: the typed passphrase is seed-equivalent, so the keyboard is
	// an armed walk-away state. The bracket CLOSES before unlockAttemptOnce runs,
	// and that IS row 5 -- see there for why a derivation must not be wiped.
	prev := ctx.wipe
	ctx.wipe = &wipeGuard{}
	defer func() { ctx.wipe = prev }()
	...
}
```

Four reasons this beats a `deriving` field on `wipeGuard`:

1. **`armed()` keeps one meaning** — "a secret is held and the machine is idle". No new normative concept in code; row 5 lives in the spec, where it belongs.
2. **F-93 survives untouched.** `ctx.wipe` is nil during `unlockDerive`, so `ctx.keepAwake && !armed` is true and Task 5's fix keeps working, verbatim. No new interaction between two features that already collided once.
3. **The KDF's exemption is expressed by the bracket's own boundary**, not by a flag whose lifetime someone must keep in sync with a loop in another file.
4. **It arms only the screen that needs it**, so it does not multiply encounters with the open post-wipe re-entry Critical (see I3) on screens that gain nothing.

Per-attempt install/uninstall is correct and free: each install is an `armed` false→true edge, and `gui/run_flow.go:153-171` resets `a.idle.start` on that edge, so every retry starts a fresh 3:00.

`prev := ctx.wipe` rather than `ctx.wipe = nil` — see M2.

---

## 5. Evidence

Four checks, run in a **scratch copy** (`…/scratchpad/sh-b2b-check`, `go1.26.3`); the reviewed repo was not written to. Recommend lifting A–D into step 9.3 as the phase's own tests.

```
=== RUN   TestCheckA_NoParkedFrameHoldsACompletePassphrase
    frames=263  complete-before-KDF=0  complete-after-KDF=200  kdfStarted=true
--- PASS
=== RUN   TestCheckB_ArmedDerivationIsWipedMidKDF
    sessions=2 derivedOK=false keyLen=0 frames=211 warningFrames=30
--- PASS
=== RUN   TestCheckB2_UnarmedDerivationCompletes
    derivedOK=true
--- PASS
=== RUN   TestCheckC_DoneMidDerivationZeroesThePassphrase
--- PASS
=== RUN   TestCheckD_DoneDuringEntryZeroesThePartialPassphrase
    before the unwind: 6 words set, buffer = beef beef beef beef beef beef
--- PASS
```

Mutation evidence — the two zeroing checks are killable, not false PASSes:

| Mutant | Result |
|---|---|
| `unlock_kdf.go:133` `clear(m)` → `_ = m` (partial exit) | **FAIL** — "the PARTIAL typed passphrase SURVIVED the unwind: beef ×6" |
| `unlock_kdf.go:384` `clear(m)` → `_ = m` (post-attempt) | **FAIL** — "the typed passphrase SURVIVED a mid-derivation unwind: beef ×12" |

Both reverted; `./gui` green after revert.

**What these do not cover.** Check B's 200 s of derivation is *synctest bubble* time produced by a 1 s tick floor over 100,000 iterations — it proves the **mechanism** (warning parks the KDF, wipe follows), not a hardware duration. The iteration threshold in §2 comes separately from the stopwatch + on-screen readings in `HARDWARE_RESULT_2026-08-09_phaseB2b.md` §8.1a. Everything ran under host gc Go; the standing TinyGo caveat (F-92) applies unchanged.

---

## 6. Findings in Task 9 as written

### C1 — Critical. §"The change" arms the key derivation

`design/…phaseB2b.md:1415-1417` prescribes installing at the top of the sealed flow, which spans `unlockAttemptOnce`. Measured consequence: the warning parks the derivation and the wipe follows, deterministically, for every payload above ~1,343,284 iterations — **34.6 % of §6.2's legal range, permanently un-openable**. Task 9 files this under open question 2 ("probably the right answer … but it is a **normative** choice"), which understates it twice: it is not optional, and its cost is not a mid-derivation warning but an unopenable conforming payload.

**Fix:** replace §"The change" with the `unlockPassphraseFlow` bracket in §4, and state in Steps that the bracket **must close before `unlockAttemptOnce`**. Question 2 is settled: **disarm**, by bracket boundary, not by a flag and never by `KeepAwake`.

### I1 — Important. The warning tells the operator something false

`gui/wipe_warning.go:50-53` renders *"This machine still holds decrypted seed material and has been idle."* Under Task 9 that is drawn at passphrase entry, where nothing is decrypted — machine-observed 30 times in Check B. Task 9 has **no step** for it; 9.2 amends only the spec's scope.

**Fix:** add a step. `wipeWarningOp` already takes its buffer and styles explicitly, and `ctx.wipe` is in scope at the call site (`gui/run_flow.go:209`), so a field on `wipeGuard` naming the subject is enough. Suggested row-4 text: *"This machine holds a partly typed passphrase and has been idle. / It will be erased in %d seconds. / Touch the screen to keep it."* Keep the countdown, the schedule and the touch-to-keep affordance identical.

### I2 — Important. Step 9.3's second test cannot be written honestly

*"…for **both** a partial and a complete-unsubmitted entry"* — §1 measures that no frame is ever parked with a complete, unsubmitted mnemonic. Forcing the test means constructing a state the product cannot reach, which is a false PASS by construction — the precise failure this phase's own history keeps producing.

**Fix:** rewrite 9.3 as (a) a wipe during **partial** entry zeroes the word buffer (Check D), (b) a wipe during the **derivation** zeroes the word buffer *and* §8.1's normalised bytes (Check C), (c) a frame-walk proving the complete-unsubmitted park does not exist, so nobody re-opens the question (Check A), (d) an **armed-derivation** regression test asserting a derivation still completes with the passphrase bracket in place (Check B inverted — this is C1's mutation row). Add the F-104 caveat: `kbd.Fragment` is not covered by any of them.

### I3 — Important. Step 9.5 spends a hardware trip inside a known-open Critical

9.5 reads *"type six words, walk away, confirm the wipe fires **and the machine returns usable**"*. The second half is the open post-wipe re-entry Critical (`HARDWARE_RESULT_2026-08-09_phaseB2b.md`, "the device HANGS re-entering the sealed payload after a wipe"). As written, a known failure will be observed and scored as Task 9's. The plan's own constraint — *"The operator gets ONE trip"* (line 1460) — makes that expensive. Task 9 also **raises the wipe rate** on screens that could never wipe before, so it increases exposure to that hang even under the narrow bracket.

**Fix:** state the precondition (Task 9's hardware step runs **after** the re-entry Critical closes, or explicitly splits its observations: "the wipe fires + the UI restarts" = Task 9; "re-entering the sealed payload works" = the other Critical), and fold the same ordering note into the phase's exit criteria.

---

## 7. Minor / Nit

- **M1 (Minor).** `clear(m)` at `gui/unlock_kdf.go:384` is straight-line, not deferred, so a panic inside `unlockAttemptOnce` unwinds past it with the passphrase live — while `unlockPayloadFlow`'s `defer p.Wipe()` and `clear(blob)` both survive that same unwind. `defer` cannot be used directly (it is inside a `for`); wrapping the attempt in a closure with `defer clear(m)` costs one indent. Owning phase: **B2b Task 9** (the line is being touched anyway).
- **M2 (Minor).** `unlockSecretSession` installs with `ctx.wipe = g; defer func() { ctx.wipe = nil }()` (`gui/unlock_session.go:83-84`). With a second installer, an accidental nesting silently uninstalls the outer guard. They cannot nest **today** — `unlockSealedFlow` returns before `unlockSecretSession` runs — but nothing enforces it. Save-and-restore (`prev := ctx.wipe` … `ctx.wipe = prev`) in **both** places makes the property structural. Owning phase: **B2b Task 9**.
- **M3 (Minor).** `design/FOLLOWUPS.md` F-105 carries the operator ruling in a banner and then, four paragraphs down, still says *"It is a **design boundary**, not a defect … That is why it is filed rather than treated as a regression"* and *"Worth deciding deliberately: is the passphrase in-flight seed-equivalent?"* — a stale record beside a settled decision, which is exactly the shape `unlockSecretLabel`'s godoc note warns about. Strike or mark those two paragraphs superseded. Owning phase: **B2b Task 9**.
- **M4 (Minor).** 9.3's phrasing "zeroes the typed words" overclaims. The `[]Word` buffer is zeroed; `kbd.Fragment` / `wordLabel` (`gui/gui.go:673, 706-720`) are Go strings no wipe reaches (F-104 §1c, phase B2c). Say "the typed word **buffer**".
- **N1 (Nit).** Task 9's §"The finding" claim that `armed()` "returns false for a nil receiver" is **correct** — `gui/wipe_guard.go:42-44`. Verified, no change; recorded so it is not re-derived.
- **N2 (Nit).** Worth one sentence in the amendment: during the warning, `a.idle.active` gates `ctx.Router.Events` (`gui/run_flow.go:174`), so the dismissing touch is swallowed rather than typed into the keyboard. Correct behaviour, pre-existing, but it looks like a dropped keystroke to anyone testing row 4 by hand.
- **N3 (Nit).** `design/…phaseB2b.md:1405` cites `inputWordsFlow`'s loop as "`gui/gui.go:671` onward". `:671` is the `func` line; the `for !ctx.Done` is `:713`. Cosmetic.

---

## 8. What to do next

1. Rewrite §"The change" to the `unlockPassphraseFlow` bracket (§4) and record question 2 as **settled: disarm during derivation** (C1).
2. Add the warning-copy step (I1) and rewrite 9.3 around Checks A–D (I2).
3. Fold the §10.2.4 amendment from §3 into 9.2 — rows appended, row 3's condition amended, three prose paragraphs.
4. Add 9.5's precondition and observation split (I3); fold M1–M4.
5. Re-dispatch scoped to *"did the fold fix C1/I1/I2/I3, and did it introduce a new defect"* — **sonnet is the right tier**; every remaining question here is mechanical, and the design question the tier would buy is the one this report just settled with measurements. State in the brief that §1's park analysis, §2's threshold arithmetic and the four checks are already machine-verified and must not be re-derived.
