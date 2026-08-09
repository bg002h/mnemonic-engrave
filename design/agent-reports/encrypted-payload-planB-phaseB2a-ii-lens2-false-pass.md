# B2a-ii whole-diff review — LENS 2: TESTS THAT CANNOT FAIL

Reviewer: independent adversarial agent (opus), 2026-08-08.
Scope: `feat/encrypted-payload-b2a-ii`, `421dca8..HEAD` (10 commits), read against
`SPEC_encrypted_payload_delivery.md` (normative) and
`IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2a_ii.md`.
Lens: discredit the green suite. Every claim below is **mutation-proven** in a
private copy (`/tmp/.../lens2fp`), never argued from reading.

**Method.** 58 mutants applied one at a time to a private copy, each with the
substitution asserted to match exactly once before running and restored from a
file copy afterwards; the whole `./gui/ ./seal/ ./bip39/` suite run per mutant
(≈17 s). **38 killed, 20 survived.** For the three highest-value survivors I
also wrote the missing assertion, confirmed it passes on the real code, and
confirmed it kills the mutant — so each fix below is verified, not proposed.

Nothing here says the shipped code is wrong. In every case the production code
is **correct** and the suite **cannot tell**. That is the finding.

---

## Critical

None. No mutation produced a wrong engraved plate, a disclosed seed, or an
unmet §10 guarantee in the code as written.

---

## Important

### I1 — the `errUnlockCancelled` route has no test at all: a cancelled unlock can reach the plate list, and a dead Back button on the 31-second KDF screen ships green

`gui/unlock_kdf.go:424` `unlockSealedFlow`'s own doc says a false return "MUST NOT
fall through to the plate list … §6.4's incomplete-backup-believed-complete, the
worst available outcome". Two mutants on that exact route survive the whole suite:

```
MUTANT [G21 cancel falls through to the plate list]        ***SURVIVED***
  case errors.Is(err, errUnlockCancelled): return false  ->  return true
MUTANT [H1 Back during the ~31 s KDF does nothing]        ***SURVIVED***
  if backBtn.Clicked(ctx) { return nil, false }  ->  if backBtn.Clicked(ctx) && false {
```

Under G21 the operator taps Back during the derivation, `p.Secret` is never
populated, and `unlockPayloadFlow` proceeds through `clear(blob)`, an empty
secret session, and straight into `unlockPlateListFlow(unlockPlates(p))` — which
for vector D is five legitimate `mk1`/`md1` plates. The operator engraves the
public half of a sealed payload, sees a complete-looking plate list, and stores
an incomplete backup believing it complete.

`TestUnlockCancelNeverReachesThePlateList` does not cover this. It cancels at
**word entry**, which returns through `unlockPassphraseFlow`'s `ok == false`
(`return false` directly) and never produces `errUnlockCancelled`. The only
producer of that error is `unlockDerive` returning `!ok` — i.e. Back on the
progress screen, or `ctx.Done` — and no test in the diff taps Button1 on that
screen. H1 shows the same blind spot from the other side: the only escape from a
~31 s wait can be deleted and the suite stays green.

**Fix.** In `TestUnlockDerivesWithARealProgressScreen` (or a sibling), after the
first progress frame, `h.tapNav(Button1)`, then assert (a) `*h.done`, and (b) no
`plateLabel(labels[j], j)` ever appears — the same negative
`TestUnlockCancelNeverReachesThePlateList` already writes, pointed at the other
cancel route.

### I2 — "the count must come from the header and never from a constant" is asserted against a vector whose count *is* the plausible constant

`gui/unlock_kdf_test.go:394`:

```go
if len(c.iterations) != 1 || c.iterations[0] != int(v.Iterations) {
    t.Errorf("the KDF ran at %v iterations; the header says %d -- the count must "+
        "come from the header and never from a constant", c.iterations, v.Iterations)
}
```

Every vector used by a gui test is at `iterations = 100000`, which is §6.2's
**floor** and the most likely hardcoded value. Measured:

```
MUTANT [G16 iteration count read as a CONSTANT, not from the header]  ***SURVIVED***
  newDeriver(pass, h.Salt[:], int(h.Iterations))  ->  newDeriver(pass, h.Salt[:], 100000)
```

The unlock still succeeds (the key is right for D), and the assertion compares
`100000 != 100000` and passes. This is §11.3's mandatory row — *"iteration count
read as a constant | §11.4 vector B — nothing else sees it"* — and §11.2's
*"Vector B is not optional — it is the only test that catches a hardcoded
iteration count."* `grep '"B"' gui/*_test.go` returns **nothing**: vector B
(`iterations = 100001`) is used only in `seal/`, and the gui's `unlockDerive` is
a **separate call site** from `seal.Unlock`'s. The gui-side header read is
therefore unpinned, and §11.3's named killer is absent from the package that
needs it.

Real consequence of the mutant shipping: §7.1's default is **300,000**, so every
real payload would fail its tag ~10 s in and the device would report *"Wrong
passphrase, or this payload has been altered"* — teaching the operator to read a
false tamper alarm as normal, which is precisely the signal §2.2 item 4 exists
to raise.

**Fix, verified.** One gui unlock of vector B:

```
--- PASS: TestProbeVectorBIterationsComeFromTheHeader   (on the real code)
MUTANT [G16-recheck ...]: killed by TestProbeVectorBIterationsComeFromTheHeader
```

### I3 — §11.2's "the derived key and the passphrase buffer MUST read as zeroed" has no assertion anywhere; six wipe deletions ship green

§11.2: *"After leaving a bundle session by **each** exit … the plaintext record
buffer, the derived key, and the passphrase buffer MUST read as zeroed. Asserted
**on the buffers themselves**."* The suite asserts the **record** buffers only
(`unlockEngraveHook`, `unlockSecretHook`), on **one** of the four exits. Every
other buffer named by §10.2 step 10 is unwatched:

```
MUTANT [H2 unlockAttemptOnce: derived key never zeroed]                ***SURVIVED***
MUTANT [H3 unlockAttemptOnce: passphrase buffer never zeroed]          ***SURVIVED***
MUTANT [H4 unlockDerive: Deriver never wiped (defer d.Wipe removed)]   ***SURVIVED***
MUTANT [H5 unlockSealedFlow: the typed passphrase []Word never zeroed] ***SURVIVED***
MUTANT [H6 seal.UnlockWithKey: decrypted plaintext never zeroed]       ***SURVIVED***
MUTANT [H11 unlockPassphraseFlow: partial entry not zeroed on exit]    ***SURVIVED***
```

H6 is the worst of the set: `defer clear(plaintext)` in `seal/unlock_key.go:456`
is what removes the **decrypted record container** — every `ms1` and every bare
mnemonic in the payload — from the heap after `AdmitSection` has copied out of
it. Deleting it leaves a full plaintext copy of the seed live for the rest of the
power cycle, reachable by neither `p.Wipe()` nor `SecretsResident()`, which is
the same class of gap lens 1 raised as C1 and I1. Nothing notices.

**Fix, verified for the cheapest one — and it needs no production change**,
because `newDeriver` is already the seam and is handed `pass` itself:

```
--- PASS: TestProbePassphraseBufferIsZeroedAfterTheAttempt   (on the real code)
MUTANT [H3-recheck ...]: killed by TestProbePassphraseBufferIsZeroedAfterTheAttempt
```

The same seam pins H4 for free (`d.Key()` returns nil after `Wipe`, per
`seal/pbkdf2.go:128-138`). H2 and H5 need one hook each, in the sanctioned
in-file style already used three times in this diff. H6 needs a seam in `seal`.

### I4 — `SeedScreen{NoEdit: true}` is tested as a widget and unpinned at its only production call site

`TestSeedScreenNoEditClosesBOTHRoutes` is a good test of the *widget* — it drives
both the nav-slot and the Button2 route in both directions, and it kills both the
handler-guard and the layout-guard mutants (G22, G23 both died). What no test
covers is that `unlockEngraveMnemonic` actually **sets** the flag:

```
MUTANT [H16 SeedScreen.NoEdit never set on the payload seed]  ***SURVIVED***
  ss := &SeedScreen{NoEdit: true}  ->  ss := &SeedScreen{}
```

The harm is the one `gui/unlock_session.go:2300`'s own comment states: with the
flag clear the operator can edit a word of an **authoritative payload seed**;
`inputWordsFlow` mutates `m` in place, and `masterFingerprintFor(m, …)` and
`engraveSeed(params, m, mfp)` then both read the edited value — so the steel
plate is internally self-consistent, carries a matching fingerprint, and does
not restore the payload's wallet. Nothing on the plate or the screen says so.

**Fix, verified.**

```
--- PASS: TestProbePayloadSeedScreenRefusesEditing   (on the real code)
MUTANT [H16-recheck ...]: killed by TestProbePayloadSeedScreenRefusesEditing
```

(reach `SeedScreen` through the real session, assert
`drawer().Hit(navSlotPoint(ctx, Button2))` is false, then `click(Button2)` and
assert word entry is never reached.)

### I5 — the Cut/Skip labels are never bound to their actions; swapping them ships green

Every session test selects a choice by **touch-target index** (`h.choose(0)`,
`h.choose(1)`), and nothing anywhere asserts which label sits at which index.
`grep -n 'Cut this plate' gui/*_test.go` returns nothing.

```
MUTANT [H27 secret plate choices reversed (Skip first)]  ***SURVIVED***
  Choices: []string{"Cut this plate", "Skip"}  ->  []string{"Skip", "Cut this plate"}
```

Under the mutant the handler is unchanged (`choice != 0` still means *do not
engrave*), so the button **labelled "Skip" cuts the seed plate** and the button
**labelled "Cut this plate" wipes the record and moves on**. Both directions are
harmful on a screen whose whole job is deciding whether seed material becomes
steel: the first engraves a seed the operator asked not to engrave; the second
wipes a secret record the operator asked to cut, and on the last secret the
session simply ends — §6.4's incomplete-backup-believed-complete again.

This is the "asserting that SOME screen appeared rather than WHICH" family,
displaced one level: the tests assert *the Nth target does X* rather than *the
target labelled Skip does X*.

**Fix.** Have `sessionHarness.choose` take a label, resolve it against the drawn
text, and fail if the label is not found — or, minimally, assert
`uiContains(content, "Cut this plate")` and pin the ordering once.

### I6 — the pass-3 fold (per-class secret numbering) landed with no test that can fail

`d0baf13` rewrote `unlockSecretSession` to count `seen`/`total` **per
Classification** rather than across all secrets. Reverting that fold survives:

```
MUTANT [G1 unlockSecretSession numbers ACROSS classes]  ***SURVIVED***
  seen[c]/total[c]  ->  n / len(at)
```

No canonical vector mixes the two secret classes — the fold's own comment says
so (`A 0/1, B 0/1, C 0/6, D 5/1, E 5/0, F 0/15, G 12/3`) — and
`TestUnlockSecretLabelNamesByClassification` tests `unlockSecretLabel(c, i, n)`
with hand-supplied `i, n`, so it exercises the formatter and never the counter
that feeds it. On a payload carrying one `ms1` **and** one bare mnemonic the
mutant renders `ms1 1/2` then `seed words 2/2`, telling the operator there are
two `ms1` cards and they are holding the second — the exact defect the fold
fixed. A fold is authorship; this one re-earned the gate and no test stands
behind it.

**Fix.** The shape is trivially constructible with the fixture already in the
diff — `sealBlobForTest(t, nil, []string{d.Secret[0], a.Secret[0]}, …)` gives one
`ms1` plus one 24-word mnemonic from the vector file — then assert the two
offered titles are `ms1` and `seed words`, unnumbered.

---

## Minor

### M1 — `TestUnlockRejectsAPartialPassphraseWithoutAKDF` names a mutant it cannot kill

Its failure message reads *"the isMnemonicComplete half of the gate is missing"*,
but:

```
MUTANT [G15 gate drops the isMnemonicComplete half]  ***SURVIVED***
  if !isMnemonicComplete(m) || !m.Valid()  ->  if !m.Valid()
```

`emptyBIP39Mnemonic` fills with `-1` (`gui/gui.go:625-631`) and
`Mnemonic.Valid()` ends `ChecksumWord(ent) == last` (`bip39/bip39.go:107-115`),
which can never equal `-1` — so `!m.Valid()` alone already rejects every partial
this flow can produce. The `isMnemonicComplete` half is real defence in depth
(it also keeps `Valid()`/`splitMnemonic` off `-1` input, where
`bytes.Repeat` with a negative count would panic — a brick on a device), but the
test's claim to pin it is false. Either assert the panic-safety property
directly, or drop the misattributed sentence so a future reader does not treat
the row as covered.

### M2 — the four wipe folds in `gui.go` / `seal.Classify` are unpinned, and the comment excusing that is wrong

```
MUTANT [G27 seal.Classify no longer clears bip39.Parse's []Word (M1)]     ***SURVIVED***
MUTANT [G28 deriveMasterKey no longer wipes the 64-byte BIP-39 seed (I1)] ***SURVIVED***
MUTANT [G29 masterFingerprintFor no longer zeroes the master key (I1)]    ***SURVIVED***
MUTANT [G30 seed-entry validity probe discards the master key again (I1)] ***SURVIVED***
```

These are lens 1's I1 and M1 — the fixes are right, and every one can be deleted
with the suite green. `gui/unlock_session.go`'s inventory comment says: *"`m` is
pinned by a test; the seed and the master key are not, **and cannot be without
unsafe** — they are internal to functions that return neither."* That claim is
false by the same file's own precedent: `unlockMnemonicHook` pins `m` — also a
local, in a function that does not return it — with an ordinary package var. A
`var deriveSeedHook func([]byte)` called immediately after
`seed := bip39.MnemonicSeed(...)` lets the caller hold the slice and assert it
reads zero once `deriveMasterKey` has returned and its `defer wipeBytes(seed)`
has run. No `unsafe`. A comment that licenses a coverage gap on seed-equivalent
material should not overstate the obstacle.

### M3 — `unlockRetryBody`'s `pub_len == 0` branch is untested, and removing it prints exactly the furniture §10.2 step 3 forbids

```
MUTANT [H9 unlockRetryBody: pub_len==0 branch removed]  ***SURVIVED***
  if !p.HasHash  ->  if false
```

No test drives a **wrong passphrase against a payload with no public section**
(A, B, C, F all have `pub_len == 0`; the retry test uses D). Under the mutant a
wrong passphrase on vector C displays *"Public data hash (0 records, SEALED):
0000 0000 0000 0000 0000 0000 0000 0000"* — a constant shown on every
fully-encrypted payload, which §10.2 step 3 and §6.6 say "would teach the
operator it is furniture", on the one screen that exists to raise a tamper
signal.

### M4 — `TestUnlockFlowWipesEveryRecordOnExit` covers one exit of the four §11.2 names

The test is good and it does kill `defer p.Wipe()` (G8 died by it). But §11.2
requires **Lock, Back, an error path, and `ctx.Done`**, and only the Back exit
(which is Lock, per §10.3) is driven. The error and `ctx.Done` exits are
structurally covered by the `defer`, which is the right implementation — but the
suite has no way to notice if that structure is ever traded for explicit calls.

---

## Nit

- **N1** — `unlockPlate.idx` is unobservable: `MUTANT [H7 encrypted-section idx
  collapsed to 0] ***SURVIVED***`. It is only read by `plateLabel`'s
  `record %d` fallback, which `unlock_platelist.go:52-58` says is **reachable**
  for the encrypted section (any card of a failed grouping keeps `HRP 0`). Under
  the mutant every such entry renders `record 1` and the operator cannot tell two
  plates apart. No test constructs that shape.
- **N2** — `masterFingerprintFor(m, …, "")`'s bare-passphrase argument is
  unpinned: `MUTANT [H17 called with a passphrase "x"] ***SURVIVED***`. §8 says
  the twelve words are never seed entropy; a non-empty argument silently engraves
  a fingerprint that does not match the wallet those words restore.
- **N3** — `TestUnlockWithKeyReproducesUnlock`'s Unlock-vs-UnlockWithKey
  comparison is tautological after the refactor (`seal/open.go:141`
  `return o.UnlockWithKey(blob, p, key)`), so `viaKey.Secret[i].Record` can never
  differ from `viaUnlock.Secret[i].Record`. The real work is done by the
  `v.Secret[i]` comparison beside it; the tautological half reads as coverage it
  does not provide.
- **N4** — `TestUnlockWithKeyDoesNotRetainOrZeroTheKey` asserts only "does not
  zero". Nothing tests "does not retain", which is the half of the name that
  would matter.
- **N5** — `TestSecretSessionEngravesAMnemonic` reaches `SeedScreen` via
  `h.mustReach("1:")`. A two-character needle on a screen that draws a numbered
  24-word list is fragile as a landmark, though it is not load-bearing for any
  assertion.

---

## What I checked and found sound

These were suspected and are **verified kills** — do not re-open them:

| Property | Killed by |
| --- | --- |
| every secret offered, in order, none skipped (`at[:1]`, reversed order) | `TestSecretSessionOffersEverySecretFirstAndInOrder` |
| `IsSecret` widened to `ClassMDMK` | 5 tests across both packages |
| per-record wipe on Skip / Back / cancelled engrave | `TestSecretSession{Skip,Back}Wipes`, `…CancelledEngraveLeavesNothing` |
| the **early** `clear(rec)` on both arms, and `clear(m)` on the mnemonic arm | `TestSecretRecordIsZeroWHILETheEngraveScreenIsUp`, `TestMnemonicWordsAreZeroWhenThePlateReachesEngrave` (both read the buffer while the engrave screen is up — the only reading that discriminates early from deferred) |
| `defer p.Wipe()` at the flow level | `TestUnlockFlowWipesEveryRecordOnExit` |
| checksum gate **before** the KDF (both orders return the same error) | `TestUnlockChecksumGateRunsNoKDF`, instrumented, with a positive control |
| passphrase never prompted when `ct_len == 0` | `TestUnlockNeverPromptsWhenNothingIsEncrypted` (hook, not return value) + its positive control |
| retry body keeps the §6.6 hash and the **anchored** `, SEALED):` shape | `TestUnlockRetryKeepsTheHashOnScreen` — the `"SEALED"`-inside-`"UNSEALED"` trap is closed correctly here |
| `ErrTooManyRecords` distinguished from "unreadable" | `TestUnlockTooManyRecordsIsNotReportedAsAWrongPassphrase` |
| chunked KDF with a real advancing progress screen (exact frame count) | `TestUnlockDerivesWithARealProgressScreen` |
| `passphraseBytes` capacity (no orphaned regrow) | `TestPassphraseBytesIsSection81sNormalisedForm` |
| `(sealed)` set only when both sections carry cards; secrets never listed; entry order | `TestUnlockPlates*`, `TestPlateListShowsTheSealedSuffixOnDuplicateLabels` |
| `(cut)` on completion only, and relabelled live | `TestPlateListMarksCutAfterACompletedEngraveAndNotAfterACancelledOne` |
| Back slot renders `IconDiscard` and **not** `IconBack` (both directions, on pixels) | `TestPlateListBackIconIsDiscardNotBack` |
| `SeedScreen.NoEdit` closes the handler route **and** the layout route | `TestSeedScreenNoEditClosesBOTHRoutes` (widget level — see I4 for the wiring) |
| `WipeSecretAt` zeroes exactly one record; `SecretsResident` keyed on `IsSecret` | `seal/session_test.go`, both directions |
| `UnlockWithKey` refuses an unsealed payload, bound-checks, fails closed, wipes a previous unlock | `seal/unlock_key_test.go` |
| `bip39.Parse` never grows (I tried `cap 12`, which grows to exactly 24) | `TestParseNeverGrowsItsResult` — the 12-word arm pins `cap == 24` tightly and cannot be reached by any growth path |

The fixture is sound: `sealBlobForTest` composes `seal.Header.Encode` and
`seal.DeriveKey` rather than reimplementing the format, and
`TestSealBlobForTestAgreesWithTheNormativeVectors` checks it against vector D's
`pubhash_sealed` and E's `pubhash_unsealed` read from the vector file before
anything is built on it. `TestSealBlobForTestBuildsTheBothSectionsShape` proves
the label collision the `(sealed)` suffix exists for is real, rather than
assuming it — so this is **not** a fixture that never reaches its own code path.

---

## Summary

58 mutants, 38 killed, 20 survived. The suite is strong where it instruments
(hooks on buffers, the KDF counter, pixel comparison of the Back icon, exact
frame counts) and weak in three recurring shapes:

1. **A route with no test at all** — the KDF cancel (I1), the wrong-passphrase
   retry on a `pub_len == 0` payload (M3).
2. **A widget tested in isolation and unpinned at its call site** — `NoEdit`
   (I4), `unlockSecretLabel` vs its counter (I6).
3. **An expected value that coincides with the constant the mutant would use** —
   the iteration count (I2), and the whole derived-key/passphrase/plaintext wipe
   family that has no expected value at all (I3).

Three fixes are written and verified in the report above; the rest are one
assertion each.
