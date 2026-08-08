# B2a-i §3d — scoped review of the `[SPLIT-NEW]` `DeriveKey` fold (opus)

Artifact: `design/IMPLEMENTATION_PLAN_encrypted_payload_deviceB_phaseB2a_i.md` §3d at commit `70be337`.
Scope: that section and its consequences only. Tasks 1, 2 and the rest of Task 3 closed GREEN over three prior R0 rounds and were out of scope.
Fork under review: `/scratch/code/shibboleth/seedhammer` @ `78949e7`.
Persisted verbatim, before any fold. HTML entities from the transport layer have been restored; nothing else is altered.

---

## VERDICT: 0 Critical / 3 Important / 2 Minor / 1 Nit

The fold itself is arithmetically and lifetime-correct — `d.Step(iterations)` always completes, the returned slice is never the one wiped, and no path in `DeriveKey` can return a short, wrong-length, or zero key. All three Importants are on §3d's **arguments**, not its code: two guarantees it claims are narrower than stated, and one doc comment it invalidates without repairing.

---

## [I1] §3d's fail-closed claim is true for `DeriveKey` but false as a property of `Key()` — and `Key()` is now the sole carrier

**WHERE:** plan §3d bullet 1, lines 1096–1102; the code it refers to is §3b's `Key()`/`Wipe()`, plan lines 845–865

**CLAIM:** "The `Deriver` cannot fail: it clamps `iterations < 1` and its key length is fixed at `sha256.Size` by a compile-time assertion. `Key()` still returns `nil` on an incomplete derivation, which preserves the property `crypto.go:47-52` argues for — an all-zero key 'is a VALID AES key and hides the fault.'"

**DEFECT:** `Key()`'s guard is `if d.done < d.total`. It is safe only because `NewDeriver` clamps `total >= 1`; it is **not** a property of `Key()`. At `total == 0` the guard inverts. Compiled and run against §3b's exact code:

```
zero-value: Step(1000)=true  Key()=0000…0000 (nil? false, len 32)
new(Deriver): Step(0)=true   Key() nil? false  len 32  allzero? true
```

A zero-value `Deriver` **reports the derivation complete** and hands out a non-nil, 32-byte, **all-zero AES key** — precisely the outcome `crypto.go:47-52` exists to forbid. `Deriver`, `Step` and `Key` are all exported, and §3b's own `Wipe` doc anticipates the exposure: *"this is a public seam and B2b will hold one of these across a timer."* A `Deriver` held as a **value field** in a B2a-ii/B2b session struct is a zero value until assigned. (Separately: `var z Deriver; z.Wipe()` panics on the nil `mac` — a brick, against the file's own stated rule.)

Before §3d, `DeriveKey` carried its own explicit fail-closed `nil` return. §3d removes it and promotes `Key()`'s guard to the only such guard in `seal`, then asserts it general. It is not.

**CONSEQUENCE:** Not reachable from `DeriveKey` — the wrapper always goes through `NewDeriver`, so nothing ships wrong today. But the one guarantee §3d hands the safety argument to has a hole, and the next phase is the one §3b names as the consumer.

**FIX:** One line in §3b's `Key()`, verified to fix the zero value without touching the happy path or post-`Wipe` behaviour:

```go
if d.total == 0 || d.done < d.total {
    return nil
}
```

and restate §3d bullet 1 as "`Key()` returns `nil` unless the derivation ran to completion" rather than "on an incomplete derivation".

---

## [I2] "The vectors are the authority" overstates what six vectors cover — and the oracle §3d gives up is recoverable at zero firmware cost

**WHERE:** plan §3d bullet 2, lines 1103–1106

**CLAIM:** "**The stdlib is no longer an independent cross-check** of the chunked loop. That is acceptable because the authority was never the stdlib: it is the six `derived_key_hex` literals in `testdata/vectors.json`, produced by the Rust implementation, which `TestDeriverReproducesEveryVectorKey` asserts directly."

**DEFECT:** Two problems, one measured and one free.

Measured — the six keyed vectors are not six independent points. Parsed from `seal/testdata/vectors.json`:

```
name  iterations  saltbytes  passlen
A     100000      16         59
B     100001      16         59
C     100000      16         59
D     100000      16         59
F     100000      16         59
G     100000      16         59
```

That is **two** iteration counts out of the 1.9M §6.2 admits, **one** salt length, and **one** passphrase length. `TestDeriverIsStepSizeIndependent` sweeps step sizes but only at vector A's single iteration count. And 59 bytes is **under SHA-256's 64-byte HMAC block size**, so no vector exercises the key-hashing path a longer §8.1-normalised passphrase would take. The vectors pin the function at one point in input space, six times.

That is fine against *today's* `Deriver`, which has no input-dependent branching. It is not fine as the answer to "what catches a future edit". An edit that introduces input-dependence — a batched inner loop keyed on `total`, an unroll when `total % k == 0`, a re-key of the `mac` — passes all six vectors and the step-size sweep. A differential catches every one.

Free — the section presents the loss as the price of the fold. It is not. Test files are never linked into the firmware (`tinygo build` on the gui main package does not compile `_test.go`), so a differential can live in `seal/pbkdf2_test.go` at **zero flash cost**, and the oracle is already in the module graph: `bip39/bip39.go:19` and `slip39/feistel.go:7` import `golang.org/x/crypto/pbkdf2`, which at v0.52.0 is a wrapper calling `crypto/pbkdf2.Key` (read in the module cache). Stdlib `crypto/pbkdf2` bottoms out in `crypto/internal/fips140/pbkdf2` — a genuinely separate implementation of exactly the layer §3b hand-rolled (block index, U-chain, XOR accumulator, iteration counting), sharing only the SHA-256 compression function.

**CONSEQUENCE:** `seal` keeps one PBKDF2 (correct) but also drops the only check that covers the input space the vectors do not — for no saving. A future `Deriver` edit that is correct at (100000, 16B salt, 59B passphrase) and wrong elsewhere reaches an operator as an indistinguishable tag mismatch ~31 s in.

**FIX:** Keep the oracle in the test file. Add to §3c a table or `testing/quick` differential asserting `NewDeriver(...)` + `Step` equals `pbkdf2.Key(sha256.New, ...)` over random iteration counts in [1, 5000] plus the §6.2 bounds, salt lengths {0, 1, 16, 64, 65}, and passphrase lengths {0, 59, 64, 65, 200}. Then rewrite bullet 2 to say the stdlib leaves *production* code and stays as a *test* oracle — which is the accurate and stronger claim.

---

## [I3] `pbkdf2.go`'s doc comment claims a `DeriveKey` equality test that does not exist, and §3d makes it impossible

**WHERE:** plan §3b, lines 756 and 768–770; §3d never amends it (steps 3.5–3.7, lines 1130–1142)

**CLAIM:** line 756 — "DeriveKey stays as the one-shot form and is what the vectors pin." line 770 — "pbkdf2_test.go asserts BOTH that this reproduces them and that it equals **DeriveKey iteration-for-iteration**."

**DEFECT:** No such assertion exists. Grepping §3c's whole `pbkdf2_test.go` block (plan lines 868–1031) for `DeriveKey` returns four hits, **all of them comment prose** — no call. The file's five tests (`TestDeriverReproducesEveryVectorKey`, `IsStepSizeIndependent`, `HonoursTheIterationCount`, `WithholdsAnIncompleteKey`, `WipeLeavesTheReturnedKeyIntact`) compare `Deriver` to the vector literals and to itself, never to `DeriveKey`. So the comment is false as written, and after §3d it becomes **unfixable** — such a test would compare the wrapper to the thing it wraps, a tautology.

Line 756 compounds it: after the fold `DeriveKey` does not "stay as the one-shot form", it *becomes* the chunked loop. §3d's own new comment carries the residue — "The vectors pin the pair" (line 1076), when there is no pair.

This is the exact failure this repo's standard names: a doc comment in a cryptographic file describing verification that was never written. It matters more given I2 — the comment promises the differential oracle I2 shows is absent, so a maintainer checking "is the chunked loop cross-checked?" reads yes and stops.

**CONSEQUENCE:** Inherited from §3b, but §3d is where it must be fixed: §3d is what makes the claimed test impossible, and §3d ships the file that contains it.

**FIX:** In §3d, add an explicit edit to `seal/pbkdf2.go`'s type comment: drop "and it equals DeriveKey iteration-for-iteration" (replace with whatever I2's differential actually asserts), and change line 756 to "DeriveKey is the one-shot wrapper over this type (crypto.go)". In §3d's own comment, replace "The vectors pin the pair" with "The vectors pin it" — there is one implementation now, which is the point.

---

## [M1] Step 3.5's verification grep returns hits on a correct fold

**WHERE:** plan step 3.5, lines 1130–1133; `seal/crypto.go:18-20`

**CLAIM:** "Confirm `crypto/pbkdf2` no longer appears in `seal`'s imports (`grep -rn 'crypto/pbkdf2' seal/`)."

**DEFECT:** Run against the real file today, four lines match; only one is the import. The other three are the file-level comment:

```
seal/crypto.go:18:// crypto/pbkdf2 and crypto/sha256 are already linked and already CALLED —
seal/crypto.go:19:// golang.org/x/crypto/pbkdf2, which bip39 and slip39 import, is a thin wrapper
seal/crypto.go:20:// over crypto/pbkdf2. crypto/aes and crypto/cipher are ABSENT from today's
```

§3d removes the import but says nothing about the comment, so a correct fold leaves the stated check reporting **3 hits**. Separately the comment goes stale in place: it justifies imports the file no longer has, and its size accounting omits `crypto/hmac`, which §3b newly imports. (The comment's substance survives — `crypto/pbkdf2` stays linked via `bip39`/`slip39`, and `crypto/hmac` is already imported by `bip85/bip85.go:7`, `nonstandard/parse.go:7` and `slip39/combine.go:4`, so §3d removes nothing from the firmware image. It is placement, not fact, that breaks.)

**CONSEQUENCE:** Either a false alarm at the gate, or an operator who deletes the size-accounting comment to make the grep clean.

**FIX:** Rewrite `crypto.go:18-22` in §3d to say `seal` now derives via `seal/pbkdf2.go` and to keep the AES-GCM ~1.6 KB accounting, adding `crypto/hmac` to the already-linked list. Change step 3.5's check to `grep -n '"crypto/pbkdf2"' seal/*.go` (quoted, so it matches imports only) and state the expected result: no output.

---

## [M2] After the fold, `crypto_test.go`'s and `pbkdf2_test.go`'s vector tests are the same assertion

**WHERE:** plan step 3.5, lines 1130–1132; `seal/crypto_test.go:20,46`

**CLAIM:** "**every existing vector and crypto test must pass UNCHANGED**; they are what proves the wrapper did not fork the primitive."

**DEFECT:** True at the instant of the fold, and only then. `TestDeriveKeyMatchesTheVectors` and `TestDeriverReproducesEveryVectorKey` then drive identical code over identical inputs against identical literals; likewise `TestIterationCountChangesTheKey` and `TestDeriverHonoursTheIterationCount`. The independent-vector-check count halves without any test being deleted — the shape that inflates a coverage story.

**CONSEQUENCE:** Recorded only. Both pairs still pin the (single) implementation; nothing goes unchecked. But a future reader counting green tests over-reads the assurance.

**FIX:** One sentence in §3d: after the fold, `crypto_test.go`'s `DeriveKey` tests exercise the `Deriver` through the wrapper and are no longer independent of `pbkdf2_test.go`; they are retained as the wrapper's own call-path check.

---

## [N1] `[]byte(passphrase)` in the wrapper makes exactly the unwipeable copy `NewDeriver`'s doc warns about

**WHERE:** plan §3d, line 1080

**DEFECT:** `NewDeriver`'s doc argues `passphrase []byte` is deliberate — "it is the caller's buffer and the caller can zero it, which Unlock's string parameter makes impossible." The wrapper's `[]byte(passphrase)` allocates a fresh heap copy that is neither the caller's buffer nor ever zeroed. Not a regression: the old body handed a `string` to `pbkdf2.Key`, which does `[]byte(password)` internally, so the copy existed before. Worth one line so B2a-ii routes the real unlock through `Deriver` with a caller-owned `[]byte` rather than reaching for `DeriveKey`.

**FIX:** Add to the wrapper comment: "Note the `[]byte` conversion is an unwipeable copy — same as the old body's `string` argument. A caller that wants a zeroable passphrase must use `NewDeriver` directly."

---

## What I checked and found sound

- **`d.Step(iterations)` always completes, for every `int`.** `NewDeriver` sets `done = 1`, `total = max(iterations, 1)`. For `iterations >= 1`, `total - done = iterations - 1` remaining and `n = iterations` — always at least one to spare. For `iterations <= 0`, `total = 1 == done`, so the loop is skipped and `Step` returns `true` immediately. Ran the wrapper at `iterations ∈ {-1, 0, 1, 2, 3}`: all return 32 non-zero bytes, none `nil`, none short. No overflow — `i` peaks at `n-1`. §6.2's 2,000,000 upper bound is nowhere near a boundary.
- **Lifetime and wipe ordering are safe twice over.** Go evaluates the return expression into the (unnamed) result before deferred calls run, so `d.Wipe()` cannot reach it — and `Key()` returns `append([]byte(nil), d.acc[:]...)`, a fresh allocation, so even a reordering would be safe. Verified: after `Wipe`, the handed-out key is intact and non-zero while `d.acc`/`d.u` are zeroed.
- **`Wipe()` resetting `done = 0` is correct because `total` survives it** — `0 < total` holds for any `total >= 1`, so post-`Wipe` `Key()` returns `nil`. The `iterations < 1` clamp is doing double duty here; that is the load-bearing detail I1 is about.
- **The compile-time key-length assertion works in both directions.** Compiled `var _ [0]struct{} = [KeyLen - sha256.Size]struct{}{}` at `KeyLen` 16 / 32 / 64: `invalid array length … -16` / clean / `cannot use [32]struct{} as [0]struct{}`. §3d's claim that key length cannot go wrong is exact, and `Key()` is therefore either `nil` or exactly `KeyLen` bytes — never short.
- **No partial result is observable.** `DeriveKey` is synchronous, `d` is local and never handed to another goroutine, and the sole production caller (`seal/open.go:205-209`, via the `o.KDF` seam) takes the return value and `defer clear(key)`s it. `nil` propagates to `aes.NewCipher`, which fails, and `Open` fails closed — the old contract preserved.
- **The one-implementation argument is right.** Two PBKDF2s in one package that must agree forever, whose divergence is a wrong key indistinguishable from a wrong passphrase, is the correct thing to eliminate. My I2 is about *where* the second one should live (test file), not *whether* production should have one.
- **Firmware footprint: unchanged, and the section correctly claims no saving.** `crypto/pbkdf2` stays linked via `bip39/bip39.go:19` and `slip39/feistel.go:7` → `golang.org/x/crypto/pbkdf2@v0.52.0`, which is a documented thin wrapper calling `crypto/pbkdf2.Key`. `crypto/hmac` was already imported by three packages. §3d removes entries from one file's import list and nothing from the image.
- **F-82 / the Rust-primary rule: the plan's claim holds.** The fold changes no wire format, no identity or stub algorithm, no validation and no admission. `DeriveKey`'s output is byte-identical to the stdlib call it replaces — already measured at 8 iteration counts and 6 vectors — so the host and device still agree on every input the wire format admits. This is an implementation detail, not normative behaviour, and does not require a Rust-first landing.
