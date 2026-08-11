# Seed-residue group — implementation report (F-94, F-87, F-104 item 2)

- **Agent:** implementer, seed-residue group, 2026-08-11
- **Worktree:** `/scratch/code/shibboleth/sh-wt-residue`, branch `seed-residue-pins`, from `main` @ `5831335`
- **Commits (not merged, not pushed):**
  - `23242fa` — `gui: pin the BIP-39 seed and BIP-32 master key wipes (F-94)`
  - `110eb1e` — `gui: pin unlockEngraveMnemonic's third early return (F-87)`
  - `74e1cf4` — `bip39: zero the discarded entropy copies (F-104 item 2)`
- **Files changed on the branch** (`git diff --stat 5831335..HEAD`):

```
 bip39/bip39.go                              |  52 ++++-
 bip39/entropy_residue_test.go               | 236 +++++++++++++++++++++++
 gui/gui.go                                  |  58 +++++-
 gui/master_key_residue_test.go              | 285 ++++++++++++++++++++++++++++
 gui/unlock_mnemonic_fingerprint_leg_test.go |  73 +++++++
 5 files changed, 699 insertions(+), 5 deletions(-)
```

Nothing outside the allowlist (`gui/gui.go`, `bip39/*.go`, new `_test.go`) was
modified. `git diff 5831335..HEAD -- gui/unlock_session.go` is **empty** — it is
mutated transiently during the F-87 mutation runs and restored byte-identically,
verified by the harness and again by `git status --porcelain` (clean).

---

## 1. What was pinned

### F-94 — the two `Z*` rows of the whole-diff inventory (#19, #20)

Three wipes, all correct, all deletable with the suite green before this branch:

| # | Wipe | Where | Now pinned by |
|---|---|---|---|
| 1 | `defer wipeBytes(seed)` — the 64-byte BIP-39 seed | `gui/gui.go` `deriveMasterKey` | `TestDeriveMasterKeyZeroesTheBIP39Seed` |
| 2 | `defer mk.Zero()` — the BIP-32 master private key | `gui/gui.go` `masterFingerprintFor` | `TestMasterFingerprintForZeroesTheMasterPrivateKey` |
| 3 | `mk.Zero()` — the validity probe's discarded key | `gui/gui.go` `SeedScreen.Confirm` | `TestSeedScreenProbeZeroesTheDiscardedMasterKey` |

Two test-only seams in `gui/gui.go`, `nil` in production, in the file's own
sanctioned in-file style (`bip85PkeyHook`, `unlockMnemonicParsedHook`):

- `deriveSeedHook func([]byte)` — fires immediately **after** `defer
  wipeBytes(seed)` is registered. This is verbatim the seam F-94's own entry
  specifies; the claim in `gui/unlock_session.go`'s inventory comment that
  pinning "cannot be [done] without unsafe" is now retired in practice as well
  as on paper.
- `deriveMasterKeyHook func(*hdkeychain.ExtendedKey)` — fires on the success
  path. The master key is `deriveMasterKey`'s **return value**, so the wipe
  belongs to each caller; there is exactly one derivation site, so one seam
  covers both callers.

`deriveMasterKey`'s `return mk, err == nil` became an explicit error return so
the hook only ever sees a live key. `hdkeychain.NewMaster` returns a nil key
with its error, so the value returned on that path is unchanged.

### F-87 — the one remaining early-return leg

The entry was already **narrowed**: two of `unlockEngraveMnemonic`'s three early
returns are pinned in `gui/unlock_session_test.go`; the residue is the
`masterFingerprintFor`-error leg. That leg fires only when
`hdkeychain.NewMaster` rejects the derived key — 1 in 2^127 — and no mnemonic,
`Platform`, network or password argument can force it, so it was reachable by
nothing.

`masterFingerprintFailHook func() bool` in `gui/gui.go` is the seam.
`TestUnlockEngraveMnemonicZeroesMOnFingerprintError` drives the real flow
(SeedScreen confirm → forced fingerprint failure → error modal → return) and
asserts every word of `bip39.Parse`'s `[]Word` copy is zero, using the existing
`unlockMnemonicParsedHook` / `watchMnemonicParsed` / `assertMnemonicZeroed`
helpers.

**This is the one hook in `gui.go` that changes behaviour instead of observing
it, so the direction was chosen deliberately: it fails CLOSED.** Non-nil, the
machine refuses to derive a fingerprint and shows an error. It cannot cause a
different fingerprint or a different seed to be engraved, which a seam that
substituted the seed or the return value could. I considered and rejected two
alternatives that would have been observation-only but unsafe in exactly that
way: a `func([]byte) []byte` seed hook (could substitute the seed → wrong plate)
and length-manipulating the seed to trip `ErrInvalidSeedLen` (same).

### F-104 item 2 — the zeroable half

`entBytes` is zeroed at all three sites it is **discarded**. The `math/big` nat
beside it is untouched and stays filed, as instructed.

| Site | Buffer | Note |
|---|---|---|
| `splitMnemonic` | `raw` (was `entBytes`) — `big.Int.Bytes()`'s own copy | Orphaned by the padding `append`, which copies **out of** it into another array, so the returned slice can never alias it. Renamed so the two are distinguishable. |
| `Valid` | the discarded entropy | Highest-count site: `LastWordCandidates` calls it once per candidate word. |
| `FixChecksum` | the discarded entropy | Only the checksum **word** survives the call. |

`Entropy()` returns its buffer and is deliberately untouched.

One seam, `entropyResidueHook func(where string, ent []byte)`, same style and
same reason as the existing `parseWordsHook` in that file.

**Rust-primary rule:** no normative change. `splitMnemonic` returns byte-identical
padded entropy, `Valid` returns the identical bool, `FixChecksum` the identical
`Mnemonic`. Nothing about what `bip39` computes, accepts or rejects moved, so
this is memory hygiene and exempt. `TestSplitMnemonicPaddingIsUnchanged`
re-derives every corpus vector's entropy **and** checksum byte through the
rewritten padding, including vector 1 whose all-zero entropy makes
`big.Int.Bytes()` return an empty slice — the maximal-padding case a rewrite
breaks first.

---

## 2. Mutation table — every pin proved able to fail

Mutants were applied by a harness (`mutate.py` / `mutate_multi.py`, scratchpad)
that **refuses to run** unless the old text matched exactly once, the file
content actually changed, and the old text is absent afterwards — then restores
and asserts the restore is byte-identical. This is the guard against the false
`SURVIVED` row a non-matching `sed` produces. Each row's "APPLIED" byte-count
delta is the machine's own evidence the edit landed.

| # | Mutation | Target | Test(s) run | Applied | Result |
|---|---|---|---|---|---|
| M1 | delete `defer wipeBytes(seed)` | `gui.go` `deriveMasterKey` | `TestDeriveMasterKeyZeroesTheBIP39Seed` | 85648→85625 | **KILLED** — `byte 0 ... is still 0xc5` |
| M1b | delete the `deriveSeedHook` fire (vacuity) | `gui.go` | same | 85648→85594 | **KILLED** — `deriveSeedHook never fired` |
| M2 | delete `defer mk.Zero()` | `gui.go` `masterFingerprintFor` | `TestMasterFingerprintForZeroesTheMasterPrivateKey` | 85648→85631 | **KILLED** — chain code `7923408d…b70e` still live |
| M2b | delete the `deriveMasterKeyHook` fire (vacuity) | `gui.go` | both master-key tests | 85648→85586 | **KILLED** — `never fired` / `fired 0 times` |
| M3 | `mk.Zero()` → `_ = mk` at the validity probe | `gui.go` `SeedScreen.Confirm` | `TestSeedScreenProbeZeroesTheDiscardedMasterKey` | 85648→85645 | **KILLED** — chain code `7462b7a4…a8f3` still live |
| M4 | delete `defer clear(m)` | `unlock_session.go` `unlockEngraveMnemonic` | **the new F-87 test alone** | 15530→15514 | **KILLED** — 24 words still `138` |
| M5 | shared `defer` → explicit `clear(m)` at legs **1 and 3 only** | `unlock_session.go` | all three `…ZeroesMOn…` tests | 15530→15536 (3 edits) | **KILLED by the new test only.** The two pre-existing tests both **PASS** under it. |
| M6 | delete `clear(ent)` in `Valid` | `bip39.go` | `TestValidZeroes…`, `TestLastWordCandidates…` | 11425→11413 | **KILLED** — `Valid.ent` still `7f7f…7f` |
| M7 | delete `clear(raw)` in `splitMnemonic` | `bip39.go` | `TestValidZeroes…` | 11425→11413 | **KILLED** — `splitMnemonic.raw` still `7f7f…7f` |
| M8 | delete `clear(ent)` in `FixChecksum` | `bip39.go` | `TestFixChecksumZeroes…` | 11425→11256 | **KILLED** — `FixChecksum.ent` still `7f7f…7f` |
| M9 | **wrong fix:** `clear(entBytes)` (the RETURNED slice) instead of `clear(raw)` | `bip39.go` | whole `./bip39/` | 11425→11430 | **KILLED** — 3 tests, `bip39: invalid checksum` |
| M10 | delete all three `entropyResidueHook` fires (vacuity) | `bip39.go` | whole `./bip39/` | 11425→11189 (3 edits) | **KILLED** — `never handed over`, `fired 0 times, want 4096` |

Two rows carry more weight than the rest:

- **M5** is the defect F-87 actually names — a "simplification" that replaces
  the shared `defer` with explicit `clear()` calls and misses one return. It is
  **invisible to the existing suite** (both pre-existing tests pass under it)
  and visible only to the new test. That is the measured justification for
  writing this test at all, given the plain `defer` deletion was already caught.
- **M9** is the only real hazard the F-104 fix introduces — zeroing the buffer
  that is handed back instead of the orphan. It dies loudly across three tests
  rather than silently returning corrupt entropy.

---

## 3. How each test establishes it is inspecting the right memory

This was the explicit risk in the brief, so it is answered per test rather than
in general.

**The `[]byte` pins (seed, `entBytes`).** The hooks hand over the slice
**value**, and a Go slice value carries the pointer to its backing array — so
the test's variable and the function's local name the *same* array, and the
wipe writes through it. There is no copy or reallocation anywhere on either
path: `bip39.MnemonicSeed` returns `pbkdf2.Key`'s single 64-byte result and
nothing appends to it; `big.Int.Bytes()` allocates once and nothing appends to
`raw`. The read after the call therefore observes the same allocation.

**The master-key pins.** `hdkeychain.ExtendedKey.Zero` zeroes `k.key`,
`k.pubKey`, `k.chainCode` and `k.parentFP` *in place*, and only then nils
`k.version` and `k.key`. `k.chainCode` is **not** nilled, so the exported
`ChainCode()` accessor keeps copying out of the same backing array before and
after — reading it twice through the captured `*ExtendedKey` is a genuine
before/after of one allocation. The private key bytes have no exported
accessor, so the tests additionally assert `IsPrivate()` flipped to `false`;
within `Zero` that assignment happens strictly after `zero(k.key)`, so it
cannot be observed unless the key bytes were wiped first. Both together mean
"`Zero` ran on this object", which is precisely the deletable thing.

**Positive controls (a buffer that is deliberately NOT wiped).**

| Control | What it proves |
|---|---|
| `TestDeriveSeedPinFailsWhenTheSeedIsNotWiped` | Same capture-then-read over an unwiped 64-byte seed; asserts the read comes back **non-zero** and equal to the published vector. If Go were handing the tests a fresh allocation on the second read — the exact false-success mode named in the brief — this test would see zeroes and fail. |
| `TestSplitMnemonicReturnedEntropyIsNotWiped` | The `bip39` half of the same argument: `splitMnemonic`'s return value is the caller's and nothing zeroes it, so the read-after-return must be non-zero. It also fails if `clear(raw)` ever starts reaching the returned buffer. |
| `assertAllZeroed`'s `sawLive` guard | Refuses to pass if *every* buffer handed over was already all-zero at hand-over time — i.e. an all-zero read can never be mistaken for a wipe of an empty buffer. |

**Anti-vacuity guards.** Every test fails if its hook never fired
(`deriveSeedHook never fired`, `deriveMasterKeyHook fired 0 times`,
`entropyResidueHook never fired`, `Valid.ent was never handed over`,
`masterFingerprintFailHook fired 0 times`). M1b, M2b and M10 exist to prove
those guards actually bite.

**Independent positive controls of content**, so a zero read is only meaningful
after the buffer is shown to have held the secret:
- BIP-39 English vector 1: `"abandon …about"` + `"TREZOR"` → seed
  `c55257c3…7463b04`. Recomputed on this tree before it was written into the
  test, not quoted from memory.
- This package's own corpus vector 2: entropy `7f7f…7f` ↔ `"legal winner thank
  year …"`. Chosen because it has no leading zero byte, so `big.Int.Bytes()`
  returns it unshortened and `splitMnemonic.raw` must equal it exactly.

**One measured count rather than a described one.**
`TestLastWordCandidatesLeavesNoEntropyResidue` asserts the hook fired
`2 * int(NumWords)` = **4096** times (2 buffers per `Valid`, one `Valid` per
candidate word), derived from `NumWords` at runtime rather than written down.
That is the first machine-checked version of F-104's "roughly 2,048×" claim.

---

## 4. Gate output (verbatim)

Baseline at the branch point `5831335`, before any edit:

```
$ nix develop … --command env CGO_ENABLED=0 go test ./...
EXIT=0   ok-packages=48   FAIL-lines=0
```

Final, at `74e1cf4`:

```
$ cd /scratch/code/shibboleth/sh-wt-residue
$ nix develop /scratch/code/shibboleth/seedhammer --command gofmt -l gui/ bip39/
gui/bip85_test.go
gui/md1_expand_fuzz_test.go
gui/multisig_build_test.go
gui/multisig_match.go
gui/multisig_testhelpers_test.go

$ nix develop /scratch/code/shibboleth/seedhammer --command env CGO_ENABLED=0 go test ./...
TEST_EXIT=0
total lines           66
^ok lines             48
non-ok lines          18   (all "[no test files]")
FAIL lines             0

ok  	seedhammer.com/bip39	(cached)
ok  	seedhammer.com/gui	28.863s
```

```
$ nix develop … --command tinygo build -o /tmp/residue.uf2 -target pico-plus2 \
      -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller
TINYGO_EXIT=0
-rw-r--r-- 1 bcg bcg 2632704 Aug 11 01:23 /tmp/residue.uf2
```

**On `gofmt -l`, which is not empty.** Those five files are unformatted **at
`HEAD` already** — I checked by extracting each from `git show HEAD:<path>` into
a temp dir and running `gofmt -l` there, which lists the same five. None is a
file this branch touches, and neither `gui/gui.go`, `bip39/bip39.go` nor any of
the three new test files appears in the list, i.e. everything written here is
`gofmt`-clean. I did not reformat them: they are outside this work's scope and
would bury the diff that matters. **Flagging it because the brief's gate did not
state an expected non-empty baseline** — worth a follow-up of its own.

Exit statuses were read from `$status` directly, never through a pipe into
`tail`/`head`.

---

## 5. Out of scope — what I did not do, and why

**Deliberately untouched, per the brief (the `G` class).** Go strings,
`math/big` nats (including the `ent` nat in `splitMnemonic`, which is the
unwipeable half of F-104 item 2), `x/crypto/pbkdf2` HMAC ipad/opad state, the
per-keystroke keyboard fragment orphans, `sentence []byte` inside
`bip39.MnemonicSeed`, the seedqr/QR bitmaps, `plate.Spline`, `stepper.Driver`
motion words, and the LCD DMA chunk buffers. **`engraveSeed`'s `words []string`
was not touched and no remedy was re-proposed** — the retraction is correct:
`frontSideSeed`'s closure reads that slice *during* the cut, so `clear(words)`
would cut a corrupt plate.

**Three things I judged out of scope and am reporting instead of acting on:**

1. **`gui/unlock_session.go`'s inventory comment is now STALE, and it is in a
   file I am not permitted to edit.** Lines 257–269 read: *"The 64-byte seed and
   the BIP-32 master key are NOT [pinned], and the reason is scheduling … A `var
   deriveSeedHook func([]byte)` fired beside `seed := bip39.MnemonicSeed(...)`
   would do the same for the seed, with no unsafe. It is not done here because
   … filed as F-94."* That seam now exists and both are pinned. Lines 243–245's
   `ZEROED` rows are still accurate but the "`m` and `rec` are pinned by tests"
   sentence undersells the new state. This is precisely the "comments outlive
   their conditions" shape, and it is a **licence for a coverage gap that no
   longer exists** — worth correcting in whichever agent owns that file, or in a
   controller-side fold. I did not edit it.

2. **A sibling residue in `LastWordCandidates` that F-104 does not name.**
   `m := make(Mnemonic, len(prefix)); copy(m, prefix)` (`bip39/bip39.go`) is a
   full copy of the operator's 11-word prefix, zeroable in one line
   (`defer clear(m)`), and left live. I did not fix it because F-104 item 2
   names `entBytes`, and this is a `[]Word`, not that buffer — expanding a
   Rust-primary ported package's diff beyond the filed item on my own initiative
   is the wrong default. It is genuinely weaker than the items I did fix (the
   caller holds an identical live copy in `prefix` for the duration of the
   screen), but it is *cheap* and *zeroable*, so it belongs on the ledger rather
   than in my head. **Suggested: file it against F-104 as item 2b.**

3. **F-87's entry text is now wrong in the other direction.** Its title still
   reads "nothing pins `unlockEngraveMnemonic`'s deferred wipe"; the entry's own
   NARROWED note already corrected that, and after this branch all three legs
   are pinned. The controller owns `FOLLOWUPS.md` — I did not edit it.

**Nothing was blocked.** No fix required touching `gui/run_flow.go`,
`gui/unlock_kdf.go`, `gui/unlock_flow.go`, `backup/backup.go` or the display
font, and no change to `bip39` altered what it computes, accepts or rejects, so
the Rust-primary escalation path was not triggered.

**Test material.** BIP-39 English vector 1 and this repo's own committed
`testVectors`/seal vector "A". No real seed material at any point.
