# B2a-ii review — LENS 1: §10.2.2's secret lifecycle

Branch `feat/encrypted-payload-b2a-ii` @ `3db3bfe`, base `421dca8`.
Scope: `gui/unlock_session.go`, `gui/unlock_session_test.go`, `gui/unlock_flow.go`,
`seal/session.go`, plus everything they call. Nothing else was reviewed.

All mutation and fix experiments were run in a throwaway copy
(`/tmp/lens1-probe`, deleted). The worktree was not written to.

```
VERDICT: 1 Critical / 1 Important / 1 Minor / 0 Nit
```

---

## C1 — the mnemonic arm zeroes `rec` before `Engrave` and `m` only after it

**WHERE** `gui/unlock_session.go:166-203` (specifically `:175` `defer clear(m)`
vs `:201` `clear(rec)`).

**DEFECT** `unlockEngraveMnemonic` holds **two** wipeable copies of the seed:
`rec` (seal's `[]byte`) and `m` (`bip39.Mnemonic` = `[]Word` = `[]int`, a full
independent copy returned by `bip39.Parse`, `bip39/bip39.go:257-275`). It clears
`rec` before entering `Engrave` — correct, and for the reason the file argues at
length at `:146-154`. It clears `m` with a **`defer`**, which fires only when
`unlockEngraveMnemonic` *returns*, i.e. after `Engrave` returns. That is exactly
the placement `:148-154` says is unsafe, applied to a copy that is just as
complete and just as wipeable.

The file's own comment at `:200` — "`m` is zeroed by this function's own defer;
`rec` is seal's buffer" — reads as though the defer discharges the obligation the
two lines above it say must be discharged *before* `Engrave`.

**EVIDENCE** Not read from the comment — measured. I added a review-only probe
that fires immediately before `NewEngraveScreen(...).Engrave(...)` and snapshots
`m`, then drove vector A (one `ClassMnemonic` secret) to the engrave screen:

```
m at Engrave entry: len=24, non-zero words=24,
  words=bacon bacon bacon ... (all 24)
seal record zeroed at Engrave entry: true
p.SecretsResident() at Engrave entry: false
```

So at the instant the ~21-minute cut begins: seal's buffer is zero,
`SecretsResident()` reports **false**, and the complete 24-word seed is sitting
in a heap slice.

`Engrave` (`gui/gui.go:2661-2725`) returns on exactly three routes — `break
frames` from Back while the job is **not** `engraveRunning`, `return true` from
select on `engraveDone`, and `ctx.Done`. Back while **running** calls
`s.job.Stop()` and continues the loop (`:2668-2674`); the screen then sits in
`engraveStopped` ("Engraving paused. Hold button to resume.") until the operator
presses Back a second time. `m` is live for all of it.

I confirmed nothing in the built plate aliases `m`: `Plate` is
`{Duration uint64, Spline bspline.Curve, Conf engrave.StepperConfig}`
(`gui/gui.go:488-513`) and `toPlate` (`:3016-3030`) returns only those three;
`engraveSeed` (`:516-538`) copies out to `words []string` and a QR before
planning.

**CONSEQUENCE** §10.2.2's stated threat is explicit: "§2.2 item 9 makes that
live, because `debug enable: 1` (measured, §3) lets SWD read SRAM with no
passphrase", and the section claims cutting secrets first "collapses that to the
first *N* plates". It does not. A payload-sourced seed remains in SRAM for the
whole cut, and indefinitely on the paused/failed/completed engrave screen — the
unattended abort-mid-plate state §10.2.2 singles out as the machine's most
ordinary recovery. An operator who aborts to re-seat shifted steel and walks away
leaves a spendable seed readable over SWD.

Worse for B2b: `seal/session.go:29-41`'s `SecretsResident()` scans only
`p.Secret[i].Record`, so §10.2.4's timer condition reports **false** while the
seed is live — the idle wipe would not fire, and could not reach `m` if it did.
`unlockPayloadFlow`'s `defer p.Wipe()` (`gui/unlock_flow.go:85`) cannot reach it
either. Nothing in the system can zero `m` except this function returning.

**FIX** One line. Keep the `defer` (it covers the Confirm-cancelled,
fingerprint-error and `engraveSeed`-error returns) and add an explicit clear
beside `clear(rec)`; `clear` is idempotent:

```go
	clear(rec)
	clear(m)
	NewEngraveScreen(ctx, plate).Engrave(ctx, &engraveTheme)
```

Machine-checked: with this applied, `go test ./gui/ ./seal/ ./bip39/ ./backup/`
is green (`ok gui 16.106s`, `ok seal 12.462s`, `ok bip39`, `ok backup`) — so the
plate does not alias `m`.

The test suite should gain the assertion that would have caught it. There is
currently **no** test that can observe `m`; the natural one mirrors
`TestSecretRecordIsZeroWHILETheEngraveScreenIsUp` using a probe on the same
`unlockSecretHook` pattern (e.g. a `"engraving"` stage fired with the `[]Word`
copy), so the guarantee is pinned for the arm that has two copies rather than one.

---

## I1 — the BIP-32 master key and the 64-byte BIP-39 seed derived from a payload seed are dropped unscrubbed, twice per plate

**WHERE** `gui/gui.go:227-236` (`deriveMasterKey`), `gui/gui.go:540-550`
(`masterFingerprintFor`), `gui/gui.go:2377` (`SeedScreen.Confirm`'s validity
check), reached from `gui/unlock_session.go:181` and `:188`.

**DEFECT** Cutting one payload-sourced mnemonic derives seed-equivalent material
twice and zeroes none of it:

- `bip39.MnemonicSeed(m, "")` (`bip39/bip39.go:217-226`) returns a fresh 64-byte
  PBKDF2 output — **the BIP-39 seed**. `deriveMasterKey` drops it.
- `hdkeychain.NewMaster(seed, net)` returns an `*ExtendedKey` holding the
  **master private key**. `masterFingerprintFor` drops it after `ECPubKey()`;
  `SeedScreen.Confirm:2377` discards it into `_`.

Both happen once inside `Confirm` and once inside `masterFingerprintFor`, so
four unscrubbed seed-equivalent objects per secret plate.

**EVIDENCE** This is a deviation from the repo's own established convention, in
the same package. `gui/derive.go:20-51` does the identical derivation with
`defer wipeBytes(seed)` and `k.Zero()` on master and every intermediate,
annotated with an R0-C1 note; `gui/bip85.go:94-110` does the same
(`k.Zero() // scrub master + each intermediate`, `defer pkey.Zero()`).
`masterFingerprintFor` has neither, and `git show 421dca8:gui/gui.go` confirms it
is byte-identical at the base — **pre-existing shared machinery** (5 callers:
`bip85.go:137`, `slip39_polish.go:432`, `seedxor_polish.go:66`, `gui.go:2154`,
`gui.go:2163`) that the new §10.2.2 path inherits.

The honesty gap is what makes it in-lens. `unlockEngraveCodex32:117-121` carries
an explicit HONEST CAVEAT naming its unwipeable copies. Its sibling
`unlockEngraveMnemonic` says only "`m` is zeroed by this function's own defer;
`rec` is seal's buffer" (`:200`), which reads as a complete inventory and is not
one — it omits the BIP-39 seed, the master key, `engraveSeed`'s
`words []string` (`gui.go:521-524`) and `string(seedqr.QR(m))` (`gui.go:517`).

**CONSEQUENCE** Same threat as C1 and strictly worse material: a 64-byte BIP-39
seed and a BIP-32 master private key are as good as the mnemonic for spending,
and unlike the `[]string`/`codex32.String` copies they are `[]byte` that *can* be
zeroed and are not. A reviewer or a future author reading `:200` would reasonably
conclude the only surviving copies are plate geometry, and would not go looking.

**FIX** Three lines in shared code plus one scrub at the discard site, and a
corrected caveat on `unlockEngraveMnemonic`:

```go
// deriveMasterKey
	seed := bip39.MnemonicSeed(m, password)
	defer wipeBytes(seed)
	mk, err := hdkeychain.NewMaster(seed, net)

// masterFingerprintFor
	defer mk.Zero()          // fingerprint is computed before defers run
	pkey, err := mk.ECPubKey()

// SeedScreen.Confirm:2377
	if mk, ok := deriveMasterKey(mnemonic, &chaincfg.MainNetParams, ""); !ok {
		showErr(...); continue
	} else {
		mk.Zero()
	}
```

Machine-checked: with C1's fix and all of the above applied,
`go test ./gui/ ./seal/ ./bip85/` is green (`ok gui 15.884s`, `ok seal 12.049s`).
`hdkeychain.NewMaster` does not retain the seed slice — `deriveAccountXpub`
already relies on that.

If the scrub is judged out of B2a-ii's scope because the function is shared, the
caveat correction is **not** optional: `:200` must name what it does not zero, to
the standard `unlockEngraveCodex32:117-121` already sets, and the scrub filed with
an owning phase.

---

## M1 — `seal.Classify` allocates and drops a wipeable `[]Word` copy of every mnemonic record

**WHERE** `seal/record.go:139-144` (pre-existing, not in the diff; reached from
`AdmitSection` for every encrypted record).

**DEFECT** `Classify` does `s := string(b)` — acknowledged in the comment at
`:136-138` as an unwipeable copy with a function-scoped lifetime — and then
`bip39.Parse(b)`, which allocates a full `Mnemonic` copy of the seed and discards
it into `_`. That one **is** wipeable and is not mentioned or wiped. It happens
once per admitted record, on the admission path for every sealed payload.

**EVIDENCE** `seal/record.go:143`: `if _, err := bip39.Parse(b); err == nil`.
`bip39.Parse` returns a heap `Mnemonic` (`bip39/bip39.go:257-275`); it also
allocates a `bytes.ToUpper` copy and a Go string per word inside the loop.

**CONSEQUENCE** Minor rather than blocking: it is one more SRAM copy on the same
SWD-readable heap, but it is short-lived, unreachable after admission, and
subject to the same TinyGo-GC caveat the whole design already carries. It is
listed because the answer to "which copies escape the wipe" is otherwise
incomplete.

**FIX** `if m, err := bip39.Parse(b); err == nil { clear(m); return ClassMnemonic }`
— and extend the `:136-138` comment to cover it, since that comment currently
accounts for the string copy only.

---

## What I checked and found sound

**Q1 — every route out of a secret plate.** Enumerated from source, not from the
brief. `unlockSecretPlate` (`unlock_session.go:78-106`):

| route | mechanism | record zeroed? |
| --- | --- | --- |
| Back from the Cut/Skip choice | `Choose` returns `(0,false)` on `cancelBtn` (`gui.go:1425-1426,1467`) | yes, deferred backstop |
| Skip | `choice != 0` → `return` | yes, backstop |
| `ctx.Done` during the choice | `for !ctx.Done` falls out → `(0,false)` | yes, backstop |
| Cut → error branch (bad codex32, no plate size, bad fingerprint) | `showError` then `return` | yes, backstop |
| Cut → engrave entered | `clear(rec)` **before** `Engrave` | yes, early — and independently by the backstop on return |
| Cut → cancelled engrave (Back while idle/stopped) | `break frames` (`gui.go:2670-2671`) | already zero + backstop |
| Cut → FAILED engrave, then Back | `engraveFailed` ≠ `engraveRunning` → `break frames` | already zero + backstop |
| Cut → Back **while running** | `Stop()`, keeps rendering; needs a 2nd Back | already zero (this is the case the early clear exists for) |
| Cut → completed, select | `return true` | already zero + backstop |
| `ctx.Done` during engrave | loop exits → `return false` | already zero + backstop |
| panic unwind | Go runs defers; `grep recover() gui/ cmd/` → no hits, so it unwinds to the top | yes, backstop |

For the **codex32** arm every one of these leaves no wipeable copy. For the
**mnemonic** arm every row in the "already zero" half is wrong about `m` — that
is C1.

**Q2 — backstop registration.** `defer func(){ p.WipeSecretAt(i); ... }()` is the
**first statement** of `unlockSecretPlate` (`:79-84`), before the `"offered"`
hook and before `ChoiceScreen` is constructed. Nothing can return ahead of it.

**Q3 — secrets first and consecutively, plural.** `unlockSecretSession:60-70`
collects **all** indices satisfying `seal.IsSecret` into `at` and loops the whole
slice; `unlock_flow.go:114-115` calls it immediately before
`unlockPlateListFlow(ctx, th, unlockPlates(p))`, so no public plate can precede a
secret. Vector F's three `ms1` among fifteen are covered by
`TestSecretSessionOffersEverySecretFirstAndInOrder`, which asserts the count
**exactly** (3) rather than "at least one" and states both mutants.

**Q4 — which copies escape.** Zeroed: `rec` (early + backstop), `plaintext` in
`UnlockWithKey` (`seal/unlock_key.go:49`), the KDF key (`seal/open.go:224`), the
partial `out` on an `AdmitSection` failure (`record.go:192` → `wipe`, `:442-446`),
any `p.Secret` a previous unlock left behind (`unlock_key.go:63-65`), the 64 KiB
blob (`unlock_flow.go:58,110`). Dropped but unwipeable and honestly documented:
`codex32.String` (`codex32/codex32.go:16-18` — it is literally `struct{ s string }`,
the whole secret), `backup.SeedString`, `plan`; the `HONEST CAVEAT` at
`unlock_session.go:117-121` names these. Dropped and **not** documented: the
mnemonic arm's derived key material (I1) and `Classify`'s `[]Word` (M1). Leaks by
construction and correctly so: the `Plate.Spline` geometry *is* the seed rendered
— unavoidable, since the machine must cut it.

**Q5 — never wipes a non-secret.** `unlockSecretSession` gates on
`seal.IsSecret`, which is `ClassCodex32Secret || ClassMnemonic` only
(`seal/session.go:16-18`); `WipeSecretAt` is called for no other index.
`unlockPlates:76-81` carries every `ClassMDMK` record out of `p.Secret` into the
list. `TestSecretsResidentIsFalseWhenTheSessionEnds:482-491` asserts all **12** of
vector F's encrypted cards survive. Sound.

**Q6 — can the wipe tests fail?** Yes. Verified by mutation, not by reading:

| mutant | result |
| --- | --- |
| drop `clear(rec)` in the **codex32** arm | `TestSecretRecordIsZeroWHILETheEngraveScreenIsUp` FAILS, printing the live `ms1…` record |
| drop `clear(rec)` in the **mnemonic** arm | `TestSecretSessionEngravesAMnemonic` FAILS, printing the 24 live words |
| drop `p.WipeSecretAt(i)` from the backstop (hook kept, so the "wiped" event still fires) | 4 tests FAIL: `…WipesEachBeforeTheNextIsOffered`, `…SkipWipes`, `…BackWipes`, `TestSecretsResidentIsFalseWhenTheSessionEnds` |

Note the first mutant leaves `TestSecretSessionCancelledEngraveLeavesNothing`
**passing** — precisely the false-PASS the test file predicts at `:397-398`, and
the reason `TestSecretRecordIsZeroWHILETheEngraveScreenIsUp` is the only test that
can pin the early wipe. Its guard at `:389-394` (assert the deferred backstop has
*not* yet fired) is what stops it degrading into an after-the-fact check. That
design is right; it just has no counterpart for `m`.

---

## OUT OF LENS

- `unlockSecretLabel` numbers across mixed classes: 2×`ms1` + 1×mnemonic renders
  `ms1 1/3`, `ms1 2/3`, `seed words 3/3` — `n` is the ordinal among *all* secrets,
  not within the class.
- `unlockPlates:80` sets `idx: i` from the position in `p.Secret` **including** the
  secret records, so the fallback label numbering for encrypted cards skips.
- The `showError` branches in both engrave arms hold `rec` resident while the error
  screen is up; no plate was built, so the record has no further use and could be
  wiped first. Bears on §10.2.4's timer, which does see it.
- `unlock_session.go:125-128` claims the `codex32.New` failure branch is unreachable
  behind §10.2.1's allow-list; not verified.
