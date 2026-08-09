# B2a-ii whole-diff review — LENS 3: the cryptographic and headless path

Reviewer: independent adversarial pass (Opus 5, 1M).
Diff: `feat/encrypted-payload-b2a-ii`, `421dca8..HEAD` (10 commits) in
`/scratch/code/shibboleth/seedhammer-wt-b2aii`.
Normative: `design/SPEC_encrypted_payload_delivery.md` (wins over plan and code).
Scope: `seal/unlock_key.go`, `seal/session.go`, `seal/open.go`, `seal/crypto.go`,
`seal/pbkdf2.go`, and `gui`'s use of them (`gui/unlock_kdf.go`,
`gui/unlock_flow.go`, `gui/unlock_session.go`, `gui/unlock_plates.go`).

**Verdict: 0 Critical / 1 Important / 2 Minor / 2 Nit.**

Nothing here says a wrong plaintext can be produced, a tag can be bypassed, or a
key can be derived from the wrong input. The Important finding is a **measured
suite gap** on §10.2 step 10 / §11.2 that the phase's own mutation record
(commit `3db3bfe`) reports as covered, plus a cheap, verified fix.

Everything below was executed, not read off a comment. Mutants were applied to a
private copy (`cp -a` to `/tmp`, deleted after); the review worktree was never
written and `git status --porcelain` is empty there.

---

## Important

### I1 — Every §10.2-step-10 wipe on the crypto path survives mutation; §11.2 requires the opposite, and the Task 8 record says they are covered

**Where:** `gui/unlock_kdf.go:166` (`defer d.Wipe()`), `:221`
(`defer clear(pass)`), `:227` (`defer clear(key)`); `seal/unlock_key.go:51`
(`defer clear(plaintext)`). Mutation record: commit `3db3bfe`.

**Defect.** §11.2 is explicit and normative:

> **Wipe on every exit path.** After leaving a bundle session by **each** exit —
> Lock, Back, an error path, and `ctx.Done` — the plaintext record buffer, **the
> derived key, and the passphrase buffer** MUST read as zeroed. Asserted **on the
> buffers themselves** … never on a return value.

The record buffers are pinned (`TestUnlockFlowWipesEveryRecordOnExit`,
`gui/unlock_session_test.go`'s buffer assertions). **The derived key, the
passphrase buffer, the PBKDF2 state and the decrypted-section plaintext are
pinned by nothing.** Measured, four separate runs of
`CGO_ENABLED=0 go test ./gui/ ./seal/` under `nix develop`:

| mutation | result |
| --- | --- |
| `defer clear(key)` **and** `defer clear(pass)` deleted from `unlockAttemptOnce` | `ok gui 42.331s` / `ok seal 55.377s` — **SURVIVES** |
| `defer clear(plaintext)` deleted from `UnlockWithKey` | `ok gui 22.541s` / `ok seal 16.348s` — **SURVIVES** |
| `defer d.Wipe()` deleted from `unlockDerive` | `ok gui 18.115s` — **SURVIVES** |
| *(positive control)* `p.WipeSecretAt(i)` deleted from `unlockSecretPlate` | **FAIL** — 4 tests, incl. `TestSecretSessionSkipWipes`: `Skip left record 0 resident: "ms10entrsqq…"` |

The positive control is there because a green run proves nothing about the
runner. The runner kills real wipe mutants; it does not see these four.

**Why it matters beyond bookkeeping.** Commit `3db3bfe`'s table reports
`wipe omitted on the Back exit path | killed | the Back test (6.7) AND defer
p.Wipe() (6.9)` and closes with `30 mutants run, 29 KILLED, 1 SURVIVING`. Both
named killers reach **record** buffers only. So the phase's evidence overstates
coverage on precisely the class of buffer that is not a record — the passphrase
(12 words, the entire strength of §2.2a's security argument), the AES-256 key,
and the GCM output buffer holding *every* decrypted secret record in one
allocation that `p.Wipe()`, `WipeSecretAt` and `SecretsResident()` cannot reach.
That last one is the same shape as lens 1's C1: a second copy of the seed that
nothing in the session lifecycle can see. It is correctly wiped today; nothing
stops the next edit from moving or deleting the line.

**Fix — measured, and it needs no new seam for three of the four.** The existing
`newDeriver` seam already receives `pass` (the same backing array
`unlockAttemptOnce` clears) and returns the `*seal.Deriver`. I wrote the probe
and ran it both ways:

```go
newDeriver = func(passphrase, salt []byte, iterations int) *seal.Deriver {
        heldPass = passphrase
        heldD = seal.NewDeriver(passphrase, salt, iterations)
        return heldD
}
… after runUnlockAttempt(…) returns:
if !allZeroBytes(heldPass)   { … }   // kills `defer clear(pass)`
if k := heldD.Key(); k != nil { … }  // kills `defer d.Wipe()` (Wipe zeroes done)
```

- against the shipped code: `--- PASS` (0.10s);
- against the two mutants: `--- FAIL`, printing
  `the passphrase buffer survived unlockAttemptOnce: "beef beef beef beef beef beef beef beef beef beef beef beef"`
  and `the Deriver was not wiped; Key() still returns ac975af4…c7f60a06`.

The derived key needs one small hook — `var unlockKeyHook func([]byte)` fired in
`unlockAttemptOnce` after `UnlockWithKey` returns, in the sanctioned
`unlockSecretHook` / `unlockEngraveHook` in-file style — asserted zero once the
function has returned.

`defer clear(plaintext)` in `UnlockWithKey` genuinely is not observable through
the public API (the buffer is `gcm.Open`'s own allocation and no handle escapes).
That one should be **recorded as a surviving mutant with its reason**, exactly
the way the plan already does for `clear(blob)` / `blob = nil` (Task 5.6) — not
left inside a table that reads as full coverage.

---

## Minor

### M1 — `seal.Classify` makes an unwipeable `string` of every record, including `ms1` and bare mnemonics, and its comment says otherwise

**Where:** `seal/record.go:135-139`.

```go
// … Converting once here is a copy of the record, which is why Classify is only
// ever called on records the caller already holds — the copy's lifetime is this
// function, and the wipeable original stays in AdmittedRecord.Record.
s := string(b)
```

**Defect.** "the copy's lifetime is this function" is false. A Go string is a
heap allocation that outlives the frame until the GC collects it, is never
zeroed, and — this is the fork's own standing caveat — TinyGo's GC may copy or
retain. `Classify` is called from `AdmitSection(recs, SectionEncrypted)`, i.e.
from inside `UnlockWithKey`, once per decrypted record. A vector-F payload
therefore produces **three unwipeable copies of `ms1` seed material at the moment
of unlock**, before any plate is offered, and they persist after §10.2.2 has
wiped all three record buffers.

**Consequence.** §2.2 item 9's narrowing — *"the seed record is wiped as soon as
its plate is cut or skipped … After that RAM holds public records only"* — and
§10.2.2's "What this costs" both read as false for these copies. This is the same
family as F-83/F-88, but with a worse shape: F-83's `plate.Spline` and F-88's
`words []string` exist only while a plate is live, whereas these exist from
unlock to power-off for **every** secret record at once. It is not in the wipe
lens's found list nor in F-88's accepted list.

**Fix (partial, and free).** `bytes.HasPrefix`, `bip39.Parse` and
`nonstandard.OutputDescriptor` all take `[]byte`; only `codex32.New`,
`ValidMD`/`ValidMK` and `DecodeAddress` need the string. Move `s := string(b)`
down to just above the `codex32.New(s)` branch: a bare-mnemonic record (and a
`command:` record, and a descriptor) is then never stringified at all. The `ms1`
copy is unavoidable and is exactly the caveat `gui/ms1_decode.go:19-20` already
carries — say so in the comment instead of claiming the copy dies with the frame,
and add the residue to F-88 so the inventory is honest (this is D1's lesson
applied to `seal`).

### M2 — `SecretsResident()`'s comment claims the property B2b's timer will rest on, and the property is not true

**Where:** `seal/session.go:20-28`, especially `:25`.

> "…cancel a secret plate mid-cut, §10.2.2 wipes the record, and this goes false
> because the secret is **ACTUALLY GONE**, not because a button was pressed."

**Defect, two parts.**

1. The record is not cleared *on cancel*. `unlockEngraveCodex32` and
   `unlockEngraveMnemonic` clear it when the plate is **built**, before
   `Engrave` is ever called (`gui/unlock_session.go:184`, `:269`) — which is the
   right design and is exactly what lens 1's C1 fix moved. So the predicate goes
   false the instant the plate reaches the screen, for every route, not on
   cancellation.
2. At that instant the seed is *not* gone. It is live as `plate.Spline` (F-83,
   accepted: "IS the seed as geometry"), plus `codex32.String` /
   `backup.SeedString` on the `ms1` arm and F-88's four rows on the mnemonic arm.

So `SecretsResident() == false` means **"no `seal`-owned record buffer is
non-zero"**, not "no seed material is resident". §10.2.4's third row reads the
two as equivalent: *"no secret record resident | none | Public data only.
Nothing to protect."*

**Consequence (B2b, not B2a).** Nothing consumes the predicate yet, so this is
not a defect in this diff. But B2b's timer is specified to key on it, and if it
does, the state "last secret plate stopped or failed, operator walks away" has
**no timer at all** while the seed is on the screen as geometry — the machine's
most ordinary recovery, per §10.2.2's own words. Reword the comment to state what
the predicate actually measures, and record the gap against F-89/F-83 so B2b's
author does not inherit the claim as fact.

---

## Nit

### N1 — `Deriver.Wipe()` leaves a resurrectable Deriver that yields a complete-looking wrong key

`seal/pbkdf2.go:128-138` zeroes `u`, `acc` and `done` but leaves `total`. A
post-`Wipe` `Step(n)` therefore re-runs a full derivation from a zeroed `u`, and
`Key()` (`:113`) sees `done >= total` and hands back 32 bytes that are **not** the
right key — surfacing ~31 s later as a tag mismatch indistinguishable from a
wrong passphrase, which is the exact failure mode `Key()`'s own doc says it exists
to prevent. Unreachable in B2a (`unlockDerive` builds a fresh `Deriver` per
attempt and `Wipe` is deferred), but `Deriver`/`Step`/`Wipe`/`Key` are all
exported and the file's own comment says "B2b will hold one of these across a
timer". Mark the Deriver dead in `Wipe` (a flag that makes `Key()` return nil and
`Step` a terminating no-op). Note the obvious `d.total = 0` shortcut has a
wrinkle: `Step` then returns `true` immediately, so `unlockDerive` would take the
nil key — fail-closed via `aes.NewCipher`, but reported as "Payload unreadable".

### N2 — the passphrase itself, not merely a key-equivalent, is unreachable inside the HMAC

`seal/pbkdf2.go:47-52` says `hmac.New` folds the passphrase into an ipad/opad
pair that is "key-equivalent and not reachable to be zeroed". Precisely: Go's
`hmac` stores `ipad = key ⊕ 0x36` in an unexported field, so the **passphrase is
recoverable by XOR**, not merely equivalent to the derived key — and combined
with the ciphertext §2.2 item 2 concedes is published, that is the seed. It
outlives every wipe in the session. Also, `hmac.Reset()` does not clear the inner
SHA-256 block buffer, so the last `U_i` lingers (harmless alone — `acc` is the
XOR of all of them). No action beyond adding these to F-88's inventory; avoiding
them means hand-rolling HMAC, which this design correctly refuses.

---

## Checked and found sound — do not re-derive

Each of the seven brief questions, answered against executed code.

1. **Is `UnlockWithKey` a faithful split of `Unlock`?** Yes, line for line.
   `split = HeaderLen + PubLen`; `end = HeaderLen + PubLen + CtLen + TagLen`
   (unconditional `TagLen`, reached only after `ErrNotSealed` guards
   `!h.Sealed()`, so identical to `Unlock`'s conditional form); the same
   `len(blob) < end` / `ErrTooShort` guard; AAD `blob[:split]`, ciphertext-with-tag
   `blob[split:end]`, taken from the blob's own bytes per §6.1a; the same
   `describeRecordCount(err, p.nPub, nSec)`; the same cross-section
   `p.nPub + nSec > MaxRecords` check (`p.nPub` is never mutated between
   `Inspect` and here, so the old `nPub := p.nPub` capture was a no-op); the same
   `for _, r := range p.Secret { clear(r.Record) }` before the reassignment.
   `Unlock` keeps its own redundant bound check and its own `!h.Sealed() → nil`
   contract. `TestUnlockWithKeyReproducesUnlock` drives both routes over vectors
   A, B, C, D, F, G and compares records, classes and the hash.
2. **Fail-closed on tag mismatch.** `Open` returns `nil, ErrAuthentication` and
   there is no path that inspects a partial result; Go's `gcm.Open` `clear(out)`s
   its own buffer before returning `errOpen`. `UnlockWithKey` returns before
   touching `p`, so `Header`, `Public`, `Hash` and `HasHash` survive intact —
   pinned by `TestUnlockWithKeyFailsClosedOnAWrongKey` and, at the UI,
   `TestUnlockRetryKeepsTheHashOnScreen` (anchored on `", SEALED):"`, with
   `", UNSEALED):"` asserted absent). §10.2 step 8's retry loop holds.
3. **Key/passphrase lifetime across every exit** — the *code* is right on every
   path: `defer clear(pass)` and `defer clear(key)` are registered before
   anything can return, `defer d.Wipe()` before the first `Step`, and the `!ok`
   cancel path returns a nil key. `Deriver.Key()` returns a fresh copy so the
   deferred `Wipe` cannot zero the result out from under the caller. What is
   missing is the *assertion* — that is I1, not a behavioural defect.
4. **`IsSecret` vs §6.3.** `ClassCodex32Secret || ClassMnemonic`, matching
   §10.2.1's table and §6.3's "an xpub and a wallet policy leak privacy but do not
   spend coins". `Classify`'s branch order puts `codex32.New` **before**
   `ValidMD`/`ValidMK`, so an `ms1` can never be re-read as a card; `bip39.Parse`
   is second, so a mnemonic cannot be re-read as anything else.
   `SecretsResident` is `IsSecret`-filtered and byte-exact (see M2 for what the
   name does *not* mean); `WipeSecretAt` fails closed on `i < 0 || i >= len`.
   `unlockSecretSession` offers every `IsSecret` record and `unlockPlates`
   forwards every `ClassMDMK` one, so no admitted record is dropped or
   double-handled.
5. **§6.2 bounds and §6.4's cap on the decrypted section.** `ct_len <= 8191` is
   enforced pre-KDF by `ParseHeader`, and GCM plaintext length equals ciphertext
   length, so the decrypted section is bounded by the same constant. `SplitSection`
   applies the pre-split separator scan, `MaxRecords`, `MaxRecordLen` and the
   CR/empty-record rules to the plaintext identically to the public section, and
   the cross-section total is checked in the one place it is knowable.
6. **Does anything bypass a check `Unlock` used to perform?** The device path is
   `Inspect → unlockDerive(seal.NewDeriver) → UnlockWithKey`, so it skips exactly
   one thing: `NormalisePassphrase`. `passphraseBytes` produces §8.1's form
   directly, and `TestPassphraseBytesIsSection81sNormalisedForm` asserts equality
   with both the vector's own passphrase string **and**
   `seal.NormalisePassphrase(m.String())`, plus lowercase/single-space/trim and a
   `cap >= 107` no-regrow bound. The `Opener.KDF` seam leaving the device path is
   compensated by `newDeriver`, and `TestUnlockChecksumGateRunsNoKDF` asserts
   0 derivations on a checksum-invalid entry and exactly 1 at the **header's**
   iteration count on a valid one.
7. **Chunked KDF: partial consumption or Deriver reuse?** No. `unlockDerive`
   constructs a fresh `Deriver` per attempt (`unlockAttemptOnce` is called once
   per pass through `unlockSealedFlow`'s retry loop), returns `d.Key()` only when
   `Step` reports completion, and `Key()` independently returns nil while
   `done < total` or on the zero value. Cancel and `ctx.Done` both return
   `nil, false → errUnlockCancelled`, which cannot fall through to the plate list
   (`TestUnlockCancelNeverReachesThePlateList`).

Also checked and sound: `clear(blob)` operates on a heap copy, not on XIP —
`XIPReader.Read` copies out of flash (`seal/read_tinygo.go:56-58`), so the F-79
release cannot fault; the deferred wipe is a closure reading `blob` at exit, so
`blob = nil` really does release the 64 KB; `unlockPlates` copies the
`AdmittedRecord` *struct*, so `rec.Record` still aliases `p.Public/p.Secret` and
the flow-level `p.Wipe()` reaches it (pinned by
`TestUnlockFlowWipesEveryRecordOnExit`); the `gui/seal_fixture_test.go` sealer
composes `Header.Encode` + `DeriveKey` and is asserted against vector D's
`pubhash_sealed` and vector E's `pubhash_unsealed` before anything is built on
it; `unlockRetryBody` uses `len(p.Public)` and never the cross-section total, so
the displayed count matches the digest's input; a non-`ErrAuthentication` failure
after a successful tag check (over-long record set, disallowed classification)
ends the session rather than looping, and `ErrTooManyRecords` gets its own §6.4
message.

Deliberately **not** re-reported, per the brief: lens 1's C1/I1/M1/D1/D2 and its
pass-3 items; F-83, F-86, F-87, F-88, F-89; the surviving `clear(blob)` mutant;
"there is no idle timer" (B2b).

One theoretical note recorded without a finding: `UnlockWithKey` and `Unlock`
compute `end` in native `int`, so a caller that hand-built a `seal.Payload`
literal with a `PubLen`/`CtLen` near 2³² could drive a negative slice bound on a
32-bit target. Unreachable — every `Payload` in the tree comes from `Inspect`,
where `ParseHeader` has already bounded both to 8191 — and identical in the
`421dca8` baseline, so it is not this diff's.
