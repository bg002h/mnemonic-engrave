# New feature ideas — filed 2026-08-22, to brainstorm after a context clear

> **SUPERSEDED IN PART, 2026-08-22.** The operator ruled *"keyless path is not
> reasonable"* and **tier 4 of the RCW is now keyed** —
> `after(1383520) AND sha256(H3) AND pk(@6)`, a seventh seed. The fixture,
> `derive-rcw-keys.sh` and the journey inputs are updated. So idea 1's premise
> ("the RCW's tier 4 has no key") is **closed by decision, not by this feature**,
> and F-229 is resolved: tier 4 got a key. Idea 1's KDF question survives as
> *how* `@6` should be derived; idea 2 is unaffected. Two numbers below are now
> stale and are marked inline.

Two features, both arising from the reasonably-complex-wallet cycle. **Neither is
specced or ruled.** This file exists so the reasoning survives a clear; the
brainstorm has not happened yet.

---

## 1. Reduced-entropy / passphrase-derived key generation — probably in `ms`

### Why

The RCW's tier 4 **had no key** (keyed 2026-08-22 — see the banner), and that one
fact closed every third-party route: Bitcoin Core (no version through v31.1), Nunchuk (same vendored
`NeedsSignature()` check), Sparrow (no miniscript engine at all). Measured, not
argued — see `PLAN_wallet_file_export.md` §1a.

Keying tier 4 fixes all of it. But the point of that tier was **recovery that
does not depend on holding a key**, so a normal seventh seed defeats its
purpose. The operator's framing: *make long-dormant funds easily spendable*.

**The idea:** derive `@6` from a **memorable passphrase through a KDF with a
deliberate work factor**, rather than from stored entropy.

### Why a KDF beats "just use a short key"

A deliberately low-entropy key hands an attacker a free speedup, and the cost is
fixed forever while hardware only improves — you would be choosing a bit-length
against an attacker in 2040.

A KDF makes the attacker pay **per guess**. You pay the work factor once, at
recovery. That asymmetry is the whole mechanism.

### The salt rule — the part that must not be got wrong

- **NEVER random.** A salt you must store separately is exactly as fatal to lose
  as the key. That reinvents the problem.
- **NEVER empty.** That is BIP-39's mistake (salt = the constant `"mnemonic"` ‖
  passphrase, 2048 iterations) and is why precomputed attacks on weak mnemonics
  are practical. One table would break every wallet using the scheme.
- **Derive it from ENGRAVED material.** Proposed:

      salt = wallet-descriptor-template-id ‖ fingerprints of @0…@5

  Every component is already on the plates *and* in any exported wallet file;
  none is secret; all exist **before `@6` does**, so there is no circularity.

  **Verified 2026-08-22:** the `wallet-descriptor-template-id` is **key-stable
  and path-stable** — the same value for the tr form whether keyless, keyed, at
  account 0 or account 8. That is what makes it usable as a salt input.

  **Stale value, corrected.** This paragraph originally cited `68a1a888…`, the
  template-id of the *keyless, single-hash* four-tier shape. Two rulings on
  2026-08-22 moved it — keying tier 4 (which adds a placeholder) and
  double-hashing the passphrases (which changes three literals). Both are
  **shape** changes, so the id moves. Current values, measured with `md 0.13.0`:

      tr   a00772edbdbb41fb4acb450672c5e5cb
      wsh  6c635eac0f5a772d80c2eb7a43872bc8

  Key-stability means the id does not move when `@N` is *bound to an xpub*. It
  does move when you add an `@N`, or change what the policy commits to. Anything
  citing a template-id should say which shape it measured.

- **Zero extra forgetting risk**, which is the argument that settles it: if you
  hold no plate and no wallet file, you have no policy, no other keys, and no
  way to spend anyway. The salt is available in exactly the cases where recovery
  is possible at all.

- **Caveat to spec explicitly:** two wallets with the same *shape* share a
  template-id, so the six fingerprints do the real per-wallet uniqueness work.
  The rule is **"salt from wallet-unique engraved material"**, not "salt from
  the template-id" — which reads pedantic until someone reuses the pattern on a
  single-signer wallet where `@6` is the only key and the salt degenerates.

### Forgetting the iteration count is nearly harmless

PBKDF2 is `U₁ ⊕ U₂ ⊕ … ⊕ U_c`, computed incrementally, so you can check the
target fingerprint **at every count as you go**. Recovering an unknown `c` costs
the same as computing the largest `c` you are willing to try.

Two consequences: the count is **not a secret** (an attacker gets the same free
sweep), and it is **not really forgettable** in a damaging way. It is a cost
multiplier, and belongs on the plate as convenience rather than protection.

### How a recoverer knows it worked

**The fingerprint is the checksum.** The plate carries `@6`'s origin as
`[<fp>/270028'/0'/8'/0']`. Re-derive, compare. A match means right passphrase,
right salt, right count, right key — and a typo anywhere fails *before*
broadcasting rather than after.

### What does not exist yet

`ms derive --passphrase` is **BIP-39's** passphrase applied to a mnemonic — it
needs a seed phrase too. Tier 4 wants passphrase → key with no mnemonic. Sketch:

    ms derive --from-passphrase - --kdf pbkdf2 --iterations 300000 \
              --salt-from-plate <md1…> --template bg002h-tr --account 8

Precedent to copy: **`me seal`** already does PBKDF2 with a tunable count and
states its cost in **measured seconds on the real device** (300,000 = 30.9 s on
RP2350, from 9,715 iters/sec). State the work factor in device-seconds, not
iterations.

### Open questions

1. Which KDF — PBKDF2 (precedent, incremental recovery of `c`) or a
   memory-hard one (Argon2id, far better per-guess cost, but loses the
   incremental-`c` property and adds a dependency)?
2. Work factor measured against what — the RP2350, a laptop, or a stated
   attacker budget?
3. Where do the parameters live in the backup format? They are **recovery
   instructions**, not secrets, and a parameter you cannot recover is as fatal
   as a lost key. New TLV, or a text plate?
4. Does the journey rehearse recovery from the passphrase alone? It should —
   `can-a-user-do-the-thing`.
5. Rust-primary: this is `mnemonic-secret`, and any wire-format change is
   Rust-first with vectors before the Go port.

---

## 2. `mt` — mnemonic transaction, a new star in the constellation

### Why

**The constellation cannot encode a transaction at all.** Verified 2026-08-22:
every PSBT mention across all five repos is a *comment*; there is no type, no
parser, no serialiser, and the fork has none either. The formats are all static
wallet material — `md1` a descriptor, `mk1` a key card, `ms1` a secret.

So the spend path is unimplemented everywhere in reach. For the RCW specifically
it is worse: the software that *would* construct a spend is the software that
**refuses to load the wallet**.

### The idea

**Store presigned transactions** — with or without a locktime — so that
catastrophic recovery is possible at a **user-specified absolute block height**,
without anyone needing to reconstruct the wallet, hold a key, or run software
that understands the policy.

### The tensions to face in the brainstorm — these are the design, not caveats

- **A presigned transaction is a BEARER INSTRUMENT.** Whoever holds it can
  broadcast it once its locktime passes. That is the *same failure class* as the
  RCW's keyless tier 4, and the reason this feature needs its threat model
  written before its format.
- **It pins the destination at signing time.** A decade-old presigned tx pays to
  an address whose keys may themselves be lost.
- **It pins the fee at signing time.** A fee rate chosen in 2026 may be
  unbroadcastable in 2040. RBF/CPFP interact badly with a fixed signature.
- **It is invalidated by any other spend of its inputs.** One ordinary
  transaction from the wallet silently voids every presigned tx that spent those
  UTXOs — and nothing on a plate would say so.
- **Size.** A taproot script-path spend revealing a deep taptree is far larger
  than a descriptor. Plate-count and chunking need measuring before designing.
- **It cannot be produced today**, because nothing in the constellation signs.
  Signing lives in third-party software, which for this wallet refuses it.

### Interlock worth recording

Keying tier 4 → the wallet imports into Core → **hot export gains a consumer** →
**F-230's stated trigger is met** (*"a named wallet whose descriptor-with-keys is
measured to import"*). These three are not independent; deciding tier 4 moves
the other two.

### Open questions

1. Does `mt` **produce** transactions or only **encode** ones produced
   elsewhere? Encoding-only is a much smaller, safer surface.
2. Bearer-instrument handling: sealed payload only? Never engraved? Or engraved
   with the same "this plate is bearer access" warning the preimages carry?
3. What invalidates a stored tx, and how does a holder find out?
4. Is a presigned tx the right instrument at all, versus simply keying tier 4 and
   letting the holder construct a spend when needed?

---

## Related follow-ups

- **F-227** — keyless template seating (closed)
- **F-228** — the English→policy route is closed for keyless wallets:
  `--from-policy` is not in the default build, and `--experimental` does not
  reach the compiler
- **F-229** — whether tier 4 gets a key — **RESOLVED 2026-08-22: it does.**
  Operator ruling, *"keyless path is not reasonable."* Side effect worth
  recording: stock `rust-miniscript` refused the keyless **tr** form outright
  (*"All spend paths must require a signature"*) while accepting the keyless
  **wsh** form, because `Descriptor::from_str` only sanity-checks `Tr`. Both
  forms are accepted now, and `md encode` no longer needs `--experimental`.
- **F-230** — hot export NOT NOW, with a two-part trigger
