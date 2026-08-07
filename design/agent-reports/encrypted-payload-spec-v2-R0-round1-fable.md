# R0 v2 round 1 (fable) — cryptographic core only, `SPEC…` @ 86c0445

Scope deliberately narrow: the construction. Structure/consistency/vector
arithmetic/test quality were covered concurrently by opus.

Verdict: **2 Critical / 3 Important / 1 Minor / 1 Nit — GATE BLOCKED.**

## CRITICAL 1 — the downgrade is REAL, and §6.6 is designed to be blind to it

§6.1a promises "altering one byte of a public record fails the tag exactly as
altering ciphertext does." Unmet: the adversary does not alter a record under the
tag — **they delete the tag.** `ct_len = 0` + zeroed crypto fields satisfies every
§6.2 unencrypted rule, and §6.6's hash is *deliberately* invariant under exactly
that transformation ("independent of … whether anything is encrypted at all";
D and E MUST agree). The one integrity value the operator holds cannot see the
one transformation that destroys integrity.

Gain is twofold and both halves are real:
- **Seed suppression.** The `ms1` plate silently ceases to exist. Where the blob
  is the seed's only copy, the operator engraves five public cards and believes
  the backup complete — §6.4's own "worst available outcome", achieved by an
  attacker rather than a parser bug.
- **It is the enabling move for CRITICAL 2.** Against a sealed payload,
  substitution is 2⁻¹²⁸. The downgrade carries the payload into the regime where
  the only barrier is the 64-bit hash.

**Fix — domain-separate the hash by shape:**
```
sealed = 0x01 if ct_len > 0 else 0x00
digest = SHA-256( "MNEMBLOB/pub/v1" ‖ 0x00 ‖ sealed ‖ record_count(u8) ‖ input )
```
`me hash` takes `--sealed`/`--plaintext`. Kills the strip cryptographically
rather than by operator vigilance, and binds record_count so a removed record is
visible. Supersedes the D≡E pin — that mutant needs a new killer (two vectors
sharing records AND shape but differing in salt/IV/iterations must agree).

## CRITICAL 2 — 64 bits is grindable for ~$60k–$250k; the cost model is wrong

§6.6 assumes one **child derivation** per candidate. It is one to two **SHA-256
compressions**. The attacker fixes one xpub and grinds fields not bound to it:
- **origin path + parent fingerprint** in each `mk1` — arbitrary indices, arbitrary
  4-byte fingerprint; descriptors never verify origin metadata against the key.
  Unbounded free bits at zero EC cost.
- **record order** (5! = 120) — and ordering the grindable record last enables
  SHA-256 **midstate reuse**, cutting ~7 compressions to ~2.
- **record case** (see IMPORTANT 4) and slack in the `md1` policy encoding
  (`ValidMD` has no upper length bracket).

2⁶⁴ trials at ~2.1×10¹⁰ compressions/s/GPU ≈ 2.4×10⁵ GPU-hours ≈ **$60k–$250k**,
weeks on ~1,000 GPUs. Inside budget for a seed backup whose machine the attacker
has already handled. §6.6's own rejection of 32 bits — "it would look like
verification while being defeatable" — applies verbatim at 64 for a valuable
target.

**Fix: display 128 bits**, 8 groups of 4 hex. Same transcription effort the spec
already asks for the 12-word passphrase (itself 128 bits). 96 bits is the minimum
signable. Delete "out of reach for this threat model".

## IMPORTANT 3 — §6.3's central claim is FALSE (verified by execution)

`ValidMD`/`ValidMK` (`codex32/mdmk.go:124,136`) are **pure BCH verifiers** — HRP
and checksum only, never decoding the payload. The checksum is publicly
computable (the fork ships `MDChecksumSymbols`, `AssembleMD1`). So arbitrary
bytes wrap into a record that classifies as `mdmkText` and is admitted to the
PUBLIC section:

```
32 bytes of entropy → md1qqqsyqcyq5rqwzqfpg9scrgwpugpzysnzs23v9ccrydpk8qarc0sdmjzeptm5fdk0
  ValidMD = true  → scanner.Scan(...) = gui.mdmkText
  codex32.New = "invalid checksum"  → NOT a secret, so §6.3's rule never fires
  MDDataSymbols round-trips the entropy exactly
```

`mdmkFlow` (`gui/gui.go:2024`) engraves `mdmkText` **verbatim**; `md.Decode` is
only on the optional Inspect branch. §12 item 6 already states the correct weaker
position ("a mislabelling detector, not a policy gate") — the spec contradicts
itself and an implementer will build to the strong form.

**Fix:** correct §6.3/§10.2 step 2 to the weaker true claim, AND make it normative
that a public-section record must additionally **decode** (`md.Decode`, the mk1
decode path) — the decoders already exist and are already invoked on Inspect.

## IMPORTANT 4 — record case is unconstrained, so the hash is not well-defined

§6.4 constrains CR, space, hyphen, empties, count, length — **not case**.
Validators accept consistent uppercase (`engine.setCase`, `checksum.go:132`):

```
md1qqqsyqcyq5rq...  ValidMD=true  hash8 = 464d1ba1a4d6d306
MD1QQQSYQCYQ5RQ...  ValidMD=true  hash8 = d79dfde4db4c8cea
```

Not hypothetical: the device's own keyboard path emits **uppercase**
(`gui/codex32_input_test.go:62`). So `me hash` from the operator's cards can
legitimately mismatch the device — teaching them that mismatches are normal,
which disarms the single control both Criticals rest on.

**Fix:** §6.4 — records MUST be all-lowercase on the wire; seal-time and
device-side refusal. (Lowercasing only inside §6.6 leaves two byte-different
blobs engraving differently.)

## IMPORTANT 5 — §6.4/§6.5's trust boundary is wrong (also found by opus)

The public section IS parsed pre-authentication at §10.2 step 2, always, and
never authenticated when `ct_len == 0`. §6.5's table has it backwards for half
the payload, licensing a naive `bytes.Split` on attacker-controlled bytes.

## MINOR — the hash is shown before the tag is checked

Sealed payload: hash at step 3, tag at step 8, ~31 s apart. A tampered public
section surfaces as "wrong passphrase or damaged payload", so the operator
retypes rather than suspecting tampering — losing the one signal §2.2 item 4
exists to raise.

## NIT — `me seal` should print the whole line (hash + count + shape), not digits

## VERIFIED SOUND — do not re-open

AAD construction correct: `[0, 52+pub_len)` covers `pub_len` (offset 44) **and**
`ct_len` (offset 48), so the cleartext/ciphertext boundary is authenticated and
cannot be moved. Magic, version, reserved, kdf_id, aead_id, iterations, salt, iv
all inside the AAD — no parameter downgrade *within* a sealed payload. No way to
make the device parse a different number of public bytes than the sealer
authenticated. Cross-payload splicing is a non-question and stronger than assumed:
§8 forbids a user-supplied passphrase and §9 generates a fresh one per seal, so
**no two payloads ever share a passphrase**. §7.2's structural nonce-uniqueness
argument is correct. §6.2's all-zero rule correctly closes the downgrade-*staging*
channel it was written for. GCM's lack of key commitment is not exploitable here.
PBKDF2 dkLen=32 single block, 16-byte salt, 12-byte RBG IV, `crypto/subtle`
comparison all correct. Truncated SHA-256 is the right primitive and the
truncation introduces no weakness — **the only defect in §6.6 is the width**, and
the only defect in the AAD story is that §6.1a's guarantee is escapable by
deleting the tag rather than by defeating it.
