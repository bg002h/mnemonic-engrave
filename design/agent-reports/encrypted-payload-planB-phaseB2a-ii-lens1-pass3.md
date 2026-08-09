# B2a-ii whole-diff review — lens 1, pass 3

**Lens:** §10.2.2's secret lifecycle and the scrubbing of seed-equivalent material.
**Scope:** the whole lifecycle at the current tip (`991bee8`), with all three folds in.
**Method:** read `gui/unlock_session.go` in full plus every callee; traced the routes
prior passes did not (mixed secret classes, mid-session parse failure, `ctx.Done`
between secrets, the loop's index stability, the plate-list handoff, `SecretsResident`'s
edge cases); machine-measured four claims in a private copy of the worktree
(`cp -a` to `/tmp`, deleted after; the worktree was not written to).

```
VERDICT: 0 Critical / 0 Important / 3 Minor / 1 Nit
```

The three folds did not break the ordering and did not leave anything half-done.
The lifecycle is correct on every route I could construct. What follows is real
but does not gate.

---

## M1 — `clear(m)` reaches only the LAST of six arrays; `bip39.Parse` orphans the rest

**WHERE** `gui/unlock_session.go:180-199` (the caveat block, row `ZEROED m`),
`gui/unlock_session.go:209,249`; root cause `bip39/bip39.go:194-210` (`Parse`);
same shape at `seal/record.go:143-148` (`Classify`) and `gui/unlock_kdf.go:267`.

**DEFECT** `bip39.Parse` builds its result with `var m Mnemonic` followed by one
`append` per word. Every growth step abandons a backing array holding the seed's
first *k* words. `clear(m)` — and `defer clear(m)`, and `Classify`'s `clear(m)` —
reach only the final array. The abandoned ones are unreachable from any package
and are never zeroed. The caveat block asserts `ZEROED m — bip39.Parse's []Word,
cleared beside it` without qualification, and F-88 does not cover this: F-88 lists
`sentence []byte`, the QR bitmap and `words []string`, all in the `LIVE` rows.
`m` is in the `ZEROED` rows.

**EVIDENCE** measured, not derived — replayed `Parse`'s exact append sequence and
recorded each distinct backing array by data pointer:

```
12 words: final cap=16, orphaned arrays hold [1 2 4 8] words  (largest = 8 of 12)
24 words: final cap=32, orphaned arrays hold [1 2 4 8 16] words (largest = 16 of 24)
```

and against the real function: `bip39.Parse` on a 24-word mnemonic returns
`len=24 cap=32`, confirming the same five growth steps ran.

**CONSEQUENCE** After a payload-sourced mnemonic is engraved, SRAM holds an
unwipeable array with the seed's **first 8 of 12 words** (or 16 of 24) as
`bip39.Word` indices, for as long as TinyGo's GC leaves it. §2.2 item 9's threat
is physical access plus SWD reading SRAM, which is measured-live on this device.
Eight of twelve words leaves ~40 bits of entropy — inside brute-force range. It
is bounded by the fact that `sentence []byte` (the *full* plaintext mnemonic,
F-88) is already accepted as resident on the same path, so this widens a hole
rather than opening one; that is why it is Minor and not Important. But the
`ZEROED` row is the third time this inventory has claimed more than it delivers
(first `m` only, then "full inventory", now "cleared" for a partial clear).

**FIX** One line, and it makes the row true: in `bip39.Parse`, `m := make(Mnemonic,
0, 24)` instead of `var m Mnemonic`. The 24 cap is already the enforced maximum
two lines below (`if len(m) == 24 { return nil, ... }`), so no growth can occur and
no orphan is created. It fixes `Classify` and `unlockSealedFlow`'s passphrase clear
at the same time. If instead the decision is to leave `bip39` alone, the row must
be rewritten to `ZEROED (final array only) m — bip39.Parse orphans up to 5 partial
copies, largest holding 8 of 12 words; F-88`.

---

## M2 — `unlockSecretLabel` numbers across ALL secrets while naming per class

**WHERE** `gui/unlock_session.go:52-64` and its one call site, `:76`.

**DEFECT** `unlockSecretSession` passes `n` = the plate's position among *all*
secrets and `len(at)` = the count of *all* secrets, but `unlockSecretLabel`
prefixes the **class** name. When one payload carries both an `ms1` and a bare
mnemonic, the denominator counts a set the name does not describe.

**EVIDENCE** The call site is `unlockSecretLabel(p.Secret[i].Class, n, len(at))`
with `n, i := range at` and `at` holding every `IsSecret` index. The rendered
strings are pinned by `TestUnlockSecretLabelNamesByClassification`
(`gui/unlock_session_test.go:609-626`): `{ClassCodex32Secret, 0, 3} = "ms1 1/3"`,
`{ClassMnemonic, 1, 2} = "seed words 2/2"`. So a payload of one `ms1` + one
mnemonic renders **`ms1 1/2`** then **`seed words 2/2`**.

No test and **no vector** exercises this. Measured from the fixture header
(`gui/seal_fixture_test.go:19-20`): A(0/1), B(0/1), C(0/6), D(5/1), E(5/0),
F(0/15), G(12/3) — F's three secrets are all `ms1`, A's single secret is a
mnemonic, and nothing mixes the two.

**CONSEQUENCE** For vector F, `ms1 1/3` correctly reads "cosigner share 1 of 3".
Mixed, `ms1 1/2` reads the same way and is wrong — the operator looks for a
second `ms1` share that does not exist, or (worse direction) reads
`seed words 2/2` as "2 of 2 seed-word plates" when there is one. §6.3 makes the
mixed shape legal: the encrypted section "may carry anything — `ms1`, `mk1`,
`md1`, a BIP-39 mnemonic". This is a labelling defect, not a scrubbing one —
every route still wipes.

**FIX** Number within the class: build the per-class count and per-class ordinal
when collecting `at`, so the denominator matches the noun. Add a fixture with one
`ms1` + one mnemonic and assert both labels.

---

## M3 — the caveat presents `defer clear(m)` as sufficient for the early returns; for B2b it is not

**WHERE** `gui/unlock_session.go:246-247` ("The defer stays: it covers the three
early returns above"), read against `:238-244` and `seal/session.go:29-41`.
**Owning phase: B2b.**

**DEFECT** On the three early returns in `unlockEngraveMnemonic` — `Confirm`
cancelled (`:216`), `masterFingerprintFor` error (`:223`), `engraveSeed` error
(`:229`) — `m` holds the full seed and is cleared only when the function
*returns*. That is fine today because nothing outside the flow acts on residency.
It stops being fine the moment B2b lands §10.2.4's timer: `SecretsResident()`
scans `p.Secret` and `Payload.Wipe()` loops `p.Secret`/`p.Public` — neither
reaches a local. This is verbatim the reasoning the C1 fold wrote three lines
above to explain why a defer was *not* sufficient for the engrave path; the
comment then presents the same defer as sufficient for the error paths.

**EVIDENCE** `seal/session.go:29-41` and `seal/open.go:56-63` — both iterate
`p.Secret` only. `gui/unlock_session.go:241-244` states the consequence explicitly
for the engrave path.

**CONSEQUENCE** If B2b's idle wipe calls `p.Wipe()` **in place** (rather than
unwinding the flow), it zeroes `rec`, `SecretsResident()` goes false, the timer
disarms — and `m` still holds the seed, on a screen the operator has walked away
from. That is exactly the state C1 removed, reintroduced through a different door.
Reachability of the two error branches is low (`engraveSeed` fails only on a
plate-fit failure; `masterFingerprintFor` cannot fail post-`Confirm`, per
`gui/gui.go:2166`), but the `Confirm`-cancelled branch is an ordinary operator
action.

**FIX** Record it as a constraint B2b must satisfy, in the code rather than in a
follow-up: §10.2.4's wipe MUST unwind the flow (so every `defer clear(...)` fires),
never merely zero `p.Secret` where it stands. One sentence in the caveat block and
one line in FOLLOWUPS with owning phase B2b.

---

## N1 — the deferred hook indexes `p.Secret[i]` unguarded, three lines after `WipeSecretAt` fails closed on the same index

**WHERE** `gui/unlock_session.go:87-92`; `seal/session.go:48-53`.

**DEFECT** `WipeSecretAt` bounds-checks and no-ops out of range, with the stated
rationale "on a device a panic is a brick". The very next statement,
`unlockSecretHook("wiped", i, p.Secret[i].Record)`, indexes the same slice with
the same `i` and no check. Unreachable in production (`unlockSecretHook` is nil,
and `i` comes from a loop over validated indices), so this is a Nit — but the two
lines disagree about what a bad `i` means, and the guard's justification is
undercut by its own caller.

**FIX** Move the hook call inside the bounds check, or drop the check.

---

## What I checked and found sound

**The folds.**
- `clear(m)` sits after the last read of `m` on every path. Traced: `Parse` →
  `Confirm(m)` → `masterFingerprintFor(m)` → `engraveSeed(params, m, mfp)` →
  `clear(rec); clear(m)` → hook → `Engrave`. `engraveSeed` (`gui/gui.go:521-543`)
  copies eagerly — `qr.Encode(string(seedqr.QR(m)))` and
  `words[i] = bip39.LabelFor(w)` — so `plate` does not alias `m` and the early
  clear cannot corrupt the cut.
- `defer mk.Zero()` does not change any caller's result. `masterFingerprintFor`
  returns a `uint32`, and Go evaluates `return bip32.Fingerprint(pkey), nil`
  before running defers. Measured rather than argued: `masterFingerprintFor` on
  the `abandon…about` vector returns **73C5DA0A**, identical to the same
  derivation computed with no `Zero()` anywhere — and 73C5DA0A is the published
  BIP-39 master fingerprint for that mnemonic. All six call sites take only the
  `uint32`.
- `defer wipeBytes(seed)` does not corrupt derivation. Measured directly:
  derived a master key, took its fingerprint, `clear(seed)`, took it again —
  73C5DA0A both times. `hdkeychain.NewMaster` does not retain the slice.
- The I1 fold is complete across the tree: all three non-test `bip39.MnemonicSeed`
  call sites now scrub — `gui/gui.go:228`, `gui/derive.go:20`, `gui/bip85.go:76`.
- `defer p.Wipe()` is registered at `unlock_flow.go:85` when `p.Secret` is still
  empty, but `Wipe` is a pointer method reading `p.Secret` at call time, and
  `UnlockWithKey` assigns `p.Secret = admitted`. The backstop sees the records.

**The untraced routes.**
- **Mixed secret classes** — both arms of the `switch` at `:108-113` are present
  and each ends in the caller's defer; wiping is correct. Only the label is wrong (M2).
- **Parse failure mid-session** — `unlockEngraveMnemonic` `showError` → `return` →
  caller's defer wipes. Structurally near-unreachable: the record was admitted
  *because* `Classify` ran `bip39.Parse` on it, and `at` holds unique indices so
  no record is offered twice on an already-zeroed buffer.
- **`ctx.Done` between two secrets** — `ChoiceScreen.Choose` returns `(0, false)`
  on `!ctx.Done` loop exit (`gui/gui.go:1430-1481`), so each remaining secret is
  entered, immediately returns, and is wiped by the defer. `ctx.Done` does **not**
  strand a resident secret; it drains the queue.
- **Loop index stability** — nothing on the session path appends to, reslices, or
  reassigns `p.Secret`; the only writer is `UnlockWithKey`, which runs before the
  session and wipes the prior `p.Secret` before overwriting it (`seal/unlock_key.go:69-72`).
  Zero secrets → `at` empty → the loop is a no-op and the plate list is built normally.
- **The plate-list handoff** — `unlockPlates` (`gui/unlock_plates.go:76-81`) admits
  only `ClassMDMK` out of `p.Secret`, which `IsSecret` excludes, so a wiped record
  cannot reach the list. No aliasing either: `AdmitSection` copies every record
  (`seal/record.go:210`, `append([]byte(nil), r...)`), so each record owns its
  array and clearing one cannot touch another. `unlockEngraveFlow`'s
  `string(rec.Record)` — the one unwipeable conversion — is reachable only from
  the list, i.e. only for `ClassMDMK`.
- **The blob** — `defer func() { clear(blob) }()` is a closure, so `blob = nil` at
  `unlock_flow.go:111` releases the 65,536-byte region before the engrave; the
  array was already zeroed at `:110`. `UnlockWithKey` has `defer clear(plaintext)`;
  `Unlock` and `unlockAttemptOnce` both `defer clear(key)`; `unlockDerive` has
  `defer d.Wipe()` and `Key()` returns a copy; `passphraseBytes` preallocates
  cap 128 so it cannot regrow, and is `defer clear`ed.

**`SecretsResident` for B2b (item 5).** Its blind spot — a zero-length or
all-zero record reads as "not resident" — is **not reachable**. Measured:
`Classify` returns `unknown format` (`IsSecret = false`) for `{}`, `{0}`,
32 NULs, and `" "`, so the allow-list refuses every one of them before it can be
admitted as a secret. §10.2.4's timer condition is therefore sound on the shapes
the device can actually hold. Its *real* limits are already filed: it cannot see
`plate.Spline` (F-83, accepted) or the F-88 copies, and cannot see a local — which
is what M3 above is about.

**Tests.** I tried to discredit the wipe assertions and could not.
`TestSecretRecordIsZeroWHILETheEngraveScreenIsUp` reads the buffer at
`"Insert a blank plate"` and additionally asserts the deferred backstop has
*not* fired, so it cannot pass on a late wipe.
`TestMnemonicWordsAreZeroWhenThePlateReachesEngrave` snapshots `m` through a hook
placed between `clear(m)` and `Engrave`, and fails on `atEngrave == nil`, so it
cannot pass vacuously. `TestSecretSessionWipesEachBeforeTheNextIsOffered`
snapshots the whole set at each offer, which is the only reading that
distinguishes per-record wiping from a lump wipe. The gaps are coverage, not
false PASSes: no mixed-class vector (M2), and `TestMasterFingerprintPassphrase`
(`gui/gui_test.go:751`) asserts only `bare != pass` rather than a known value —
which is why I pinned 73C5DA0A by measurement above rather than leaning on the
green suite for the `mk.Zero()` fold.
