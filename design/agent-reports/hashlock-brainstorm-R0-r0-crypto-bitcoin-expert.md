# R0 round 0 — hashlock-phrase brainstorm, cryptography + Bitcoin programmer lens

**Date:** 2026-09-03
**Model:** opus (single agent, read-only)
**Artifact reviewed:** `design/BRAINSTORM_hashlock_phrase.md` at mnemonic-engrave commit `72081c5`
(`git log -1 --format=%h -- design/BRAINSTORM_hashlock_phrase.md` → `72081c5`; repo HEAD is the same commit, tree clean)
**Brief:** `design/agent-briefs/hashlock-brainstorm-R0-crypto-review-brief.md`

**Other files read (mnemonic-engrave):** `design/SPEC_wallet_policy_composer.md` §4a/§4b/§6c/§8h/§8i/§14; `design/FOLLOWUPS.md` F-132, F-465, F-466, F-467, F-468; `design/S4_journey_walk_2026-09-02.md` §W-5; `design/journeys/derive-hashvault-keys.sh:55-80`; `crates/me-cli/src/seal/crypto.rs`, `crates/me-cli/src/seal/wire.rs`.
**mnemonic-secret (`7fc1e58`):** `MIGRATION.md`; `design/SPEC_ms_v0_2_kofn.md:20-40`; `crates/ms-codec/src/consts.rs`, `src/envelope.rs`, `src/payload.rs`; `crates/ms-cli/src/cmd/{decode,combine,encode,verify,derive,inspect,payload_lang,repair,split}.rs`.
**seedhammer fork (`70008da`):** `codex32/mspayload.go`, `gui/composer_hash.go`, `gui/passphrase_keyboard.go`, `passphrase/passphrase.go`.
**Crates:** `miniscript-12.3.6`, `bitcoin-0.32.9`, `pbkdf2-0.12.2` (paths under `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`).
**Remote:** BIP-39, BIP-93, BIP-141, BIP-174, BIP-341; RFC 8018; LedgerHQ/app-bitcoin-new `src/handler/lib/policy.c`; hashcat v6.2.6 RTX 4090 benchmark gist.

---

## C:1 I:6 M:6 N:2

---

## Findings

### C-1 — `--method sha256` plus a method-independent 20-character floor produces a guessable preimage on an anyone-can-spend path

**Claim.** The record applies one warning floor to both derivation methods, but the two methods differ by a factor of 124,060 in guessing cost, so a phrase the record declares warning-free is recoverable in under a minute under `--method sha256` — and on a C16 keyless `wsh` path that is an anyone-can-spend output.

**Evidence.**
- §4.2 states the floor once, for both methods: *"Under 20 characters the tool warns on stderr that anyone holding the template can guess it, and proceeds."* §4.2's method paragraph is the only place methods are distinguished, and it says nothing about the floor.
- §5's justification is stated only for the hardened method: *"20-character warning floor | ~40 bits of English is years per GPU at 100,000 iterations"*.
- `SPEC_wallet_policy_composer.md:128` admits the keyless path: *"`HASH` with or without `LOCK`, no `KEYS`: **wsh only, EXPERIMENTAL, confirm-to-proceed** (C16)"*. Nothing but the preimage gates it.
- Measured guessing rates, one RTX 4090, hashcat v6.2.6 (https://gist.github.com/Chick3nman/32e662a5bb63bc4f51b847bb422222fd): `-m 10900` PBKDF2-HMAC-SHA256 `"Speed.#1.........: 8865.7 kH/s"` at `"[Iterations: 999]"`; `-m 1400` SHA2-256 `"Speed.#1.........: 21975.5 MH/s"`.

```
$ python3 - <<'EOF'
pbkdf2_999 = 8865.7e3; sha256 = 21975.5e6
g_hardened = pbkdf2_999 * 999 / 100_000     # guesses/s at c=100,000
g_sha256   = sha256 / 2                     # H = sha256(sha256(phrase)) = 2 hashes/guess
print("hardened  guesses/s: %.3g" % g_hardened)
print("sha256    guesses/s: %.3g" % g_sha256)
print("ratio: %.0f x" % (g_sha256/g_hardened))
for bits in (36, 40, 44, 50, 56, 64):
    for name, r in (("hardened", g_hardened), ("sha256", g_sha256)):
        s = 2**bits / r / 2
        print(f"  {bits} bits {name:9s}: {s:11.4g} s = {s/86400:9.4g} days = {s/31557600:9.4g} yr")
EOF
hardened  guesses/s: 8.86e+04
sha256    guesses/s: 1.1e+10
ratio: 124060 x
  40 bits hardened :   6.207e+06 s =     71.84 days =    0.1967 yr
  40 bits sha256   :       50.03 s = 0.0005791 days = 1.585e-06 yr
  56 bits hardened :   4.068e+11 s = 4.708e+06 days = 1.289e+04 yr
  56 bits sha256   :   3.279e+06 s =     37.95 days =    0.1039 yr
```

**Counterexample or construction.** The record's own measured example. `S4_journey_walk_2026-09-02.md:227-232` records the operator's 2026-09-03 host measurement producing `sha256 hashlock (hash:) — b867db87..edbc96cb`. Recomputing it identifies the phrase and the method:

```
$ python3 - <<'EOF'
import hashlib
p = b"correct horse battery staple"
X = hashlib.sha256(p).digest()
print("phrase       :", p.decode(), f"({len(p)} chars)")
print("X = sha256(p):", X.hex())
print("H = sha256(X):", hashlib.sha256(X).hexdigest())
EOF
phrase       : correct horse battery staple (28 chars)
X = sha256(p): c4bbcb1fbec99d65bf59d85c8cb62ee2db963f0fe106f483d9afa73bd4e39a8a
H = sha256(X): b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb
```

`b867db87..edbc96cb` matches W-5 exactly. Two things follow.

1. The phrase is **28 characters — above the floor — so no warning fires at all** under §4.2's rule.
2. Under `--method sha256`, X is not merely guessable; it is the single most-published private key in Bitcoin's history. Derived on this machine:

```
$ python3  # secp256k1 + base58 in-line, full script in "Sources consulted"
sha256('correct horse battery staple') = c4bbcb1fbec99d65bf59d85c8cb62ee2db963f0fe106f483d9afa73bd4e39a8a
P2PKH address (uncompressed)          = 1JwSSubhmg6iPtRjtyqhUYYH7bZg3Lfy1T
```

`1JwSSubhmg6iPtRjtyqhUYYH7bZg3Lfy1T` is the canonical brainwallet address, swept since 2012. **`--method sha256` maps a phrase into exactly the brainwallet key-space**, against which continuously-running, permanently-funded grinding infrastructure has existed for fourteen years. Whoever holds the template plate (H) — or, after any spend, everyone (see I-5a) — grinds candidate phrases at 1.1e10/s. A funded keyless `wsh` path built from this phrase is taken by the first grinder that reaches it, with no key, no cosigner and no timelock.

Even away from the dictionary: a 20-character English phrase at ~2 bits/character is ~40 bits and falls in **50 seconds expected** under `--method sha256`.

**Remedy (non-authoritative).** Make the floor method-dependent and derive it from the number, not from prose. Under `--method sha256`, either refuse below a computed floor or require the phrase to come from a generator (the design already ships `--random`, which is strictly better here), and make that method's warning name the class: *"this is the brainwallet construction; anyone holding the digest tests 10^10 phrases per second."* L5 gives the operator the choice of method; it does not oblige the tool to present both as equally safe at the same phrase length.

---

### I-1 — §5's "years per GPU" is wrong by an order of magnitude, and the fixed salt makes the grind amortised across every hashlock ever made

**Claim.** The 20-character floor rests on a number that is off by ~10x even for the hardened method, and the fixed salt — a consequence of L4, not a challenge to it — converts a per-target grind into a one-time global precomputation, which is exactly what RFC 8018 says a salt is for.

**Evidence.**
- §5: *"~40 bits of English is years per GPU at 100,000 iterations"*. Measured above: 40 bits is **71.8 days expected on one RTX 4090**, ~7 days on a ten-GPU rig. Not years.
- Entropy assumption stated: **2 bits per character** for a chosen memorable English phrase. This is generous to the design — NIST SP 800-63 Appendix A's older model gives 4 + 7×2 + 12×1.5 = 36 bits at 20 characters, and Shannon's 1951 estimate for English prose is 0.6–1.3 bits/character. At 36 bits the hardened figure is 4.5 days.
- RFC 8018 §4.1 on why salts exist: *"It is difficult for an opponent to precompute all the keys, or even the most likely keys, corresponding to a dictionary of passwords."* §5 fixes the salt to `"ms-hashlock-v1"` for every user of the format, forever.
- Contrast within this constellation: `me`'s sealed payload uses a **random** 16-byte salt (`crates/me-cli/src/seal/wire.rs:13` `pub const SALT_LEN: usize = 16;`), so each sealed payload costs its own grind.

**Counterexample or construction.** One RTX 4090 precomputes PBKDF2(candidate, `"ms-hashlock-v1"`, 100000, 32) → sha256 → H for the top 2^32 phrase candidates in `2**32 / 8.86e4 = 48,470 s ≈ 13.5 hours`. That single table then breaks **every** ms1 hashlock in existence by lookup, at zero marginal cost per target, forever. Under a per-target random salt the same attacker pays 13.5 hours *per hashlock*. The record's floor is set against the per-target cost.

**Remedy.** Restate the floor from the computed number, in bits rather than characters, with a recommended generator (N diceware words, or `--random`). §5's veto row already permits "any fixed ASCII string" for the salt, so an optional `--salt` — printed verbatim on the method line, still hand-copyable, still one line in the external backup — would restore per-operator separation without touching L4's primitive or iteration count. This is a report of a ruling's consequence, not a request to reopen L4.

---

### I-2 — a wrong-length `0x03` payload has no refusal path, and the obvious implementation panics

**Claim.** §4.3 says *"Any other length under `0x03` is refused with the existing payload-length error"*, but with `Payload::Preimage([u8; 32])` the existing error cannot fire, and the dispatch site the record extends indexes the payload without a length guard.

**Evidence.**
- `crates/ms-codec/src/envelope.rs:208-209` (the `MNEM_PREFIX` arm the `0x03` arm would be modelled on) indexes unguarded: `let language = data[1];` / `let entropy = data[2..].to_vec();`.
- `crates/ms-codec/src/payload.rs:66-93` — `validate()` is the only producer of `Error::PayloadLengthMismatch`, and both of its arms hard-code `expected: VALID_ENTR_LENGTHS` and `tag: *Tag::ENTR.as_bytes()`. A `[u8; 32]` variant is structurally length-correct, so `validate()` can never fire for it.
- The crate carries a recorded incident of this exact class at `envelope.rs:196-201`: *"WITHOUT this, a valid-checksum but non-standard-length Entr share set recovered via `combine_shares` flowed unvalidated to the CLI's `from_entropy_in`, which panicked (audit I9, exit 101)."*

**Counterexample or construction.** Reachable `0x03` payload lengths, computed against BIP-93's bracket (*"a payload which is a sequence of up to 74 bech32 characters"*) and the crate's own 48-character short-codex32 floor (`envelope.rs:160-163`):

```
$ python3 -c "print('payload bytes reachable: %d .. %d' % (26*5//8, 74*5//8))"
payload bytes reachable: 16 .. 46
```

Any valid-checksum ms1 whose payload is `0x03` followed by 15–31 or 33–45 bytes is constructible — a mistyped-then-repaired plate, or a crafted string. Feed it to `ms decode`: `data[1..33]` on a 20-byte payload panics (index out of range, exit 101). If instead the implementer reuses the "existing payload-length error" as §4.3 instructs, a 21-byte `0x03` payload prints *"expected one of 16/20/24/28/32, got 20"* — a refusal that names a legal length as illegal, for a tag (`entr`) that is not the kind being refused.

**Remedy.** Check `data.len() == 33` in `dispatch_payload` **before** constructing the variant, returning a new `Error::PreimageLengthMismatch { got }` (or widening `PayloadLengthMismatch`'s `expected`/`tag` fields). Prefer `<[u8; 32]>::try_from(&data[1..])` over slice indexing in any case. Pin all of 16, 32, 34 and 46 payload bytes as vector rows.

---

### I-3 — `#[non_exhaustive]` removes the compiler's help at the four sites that most need it, and the designed refusal sits downstream of the panic

**Claim.** §4.3 cites `non_exhaustive` as a benefit — *"the enum is `non_exhaustive`, so downstream matches keep compiling"* — but ms-cli's four `_ => unreachable!` arms are exactly what "keep compiling" means here: the new variant is absorbed silently and panics at runtime, and two of the verbs the record says will *refuse* the kind reach the panic first.

**Evidence.** Four live sites in ms-cli, each a `_ =>` catch-all over `ms_codec::Payload`:
- `crates/ms-cli/src/cmd/combine.rs:166` — `_ => unreachable!("combine_shares returned an unknown Payload variant")`
- `crates/ms-cli/src/cmd/decode.rs:107` and `:112` — `_ => unreachable!("ms-codec decode returned unknown Payload variant")`
- `crates/ms-cli/src/cmd/payload_lang.rs:60-61` — `// ms_codec::Payload is #[non_exhaustive]; guard against future variants.` / `_ => unreachable!("ms-codec decode returned unknown Payload variant")`

And the two callers that route straight into it, with no refusal in between:
- `crates/ms-cli/src/cmd/verify.rs:98-104` — `Ok((_tag, payload)) => crate::cmd::payload_lang::payload_entropy_and_language(payload, …)`
- `crates/ms-cli/src/cmd/derive.rs:433-435` — the same call.

§4.2 says *"`derive` and `verify` refuse it with the executable remedy `ms hashlock <ms1>`"*, and §4.1 H1 lists `combine` among the verbs gaining arms — but the record does not say **where** the refusal is placed, and the shipped control flow consumes the decoded `Payload` before any kind check could run.

**Counterexample or construction.** `ms verify <preimage-ms1> --phrase …` → `ms_codec::decode` returns `Ok((tag, Payload::Preimage(..)))` → `payload_lang.rs:61` → panic, exit 101, instead of the designed refusal. `ms combine <k shares of a preimage>` → `combine.rs:166` → the same. Neither produces a compile error when ms-codec 0.8.0 is pinned, because ms-cli is a downstream crate and the enum is `non_exhaustive`.

**Remedy.** Enumerate the four sites as an explicit checklist in the H1 plan (they are the complete set today: `grep -rn '_ => unreachable' crates/ms-cli/src`), convert each to a typed refusal, place the `derive`/`verify` refusal immediately on the `Ok((tag, payload))` arm before the helper call, and add one test per site. The MIGRATION 0.7→0.8 section should tell every downstream reader to do the same sweep, because the compiler will not.

---

### I-4 — `0x03` at 33 payload bytes destroys the property that an ms1's length identifies its kind

**Claim.** §4.3 says the preimage is *"the same as entr-32; the prefix byte alone tells them apart, exactly as `0x02` mnem is told apart today"* — but mnem is *also* told apart by length, and the preimage is not. It is the first kind that collides with entr on length **and** on the visible id **and** on the first payload character.

**Evidence.** Derived from `crates/ms-codec/src/consts.rs:33` (`VALID_STR_LENGTHS`) and `:43` (`VALID_MNEM_STR_LENGTHS`):

```
$ python3 - <<'EOF'
import math
ENTR=[16,20,24,28,32]
sl=lambda b: 9 + math.ceil(b*8/5) + 13
entr={sl(n+1) for n in ENTR}; mnem={sl(n+2) for n in ENTR}; pre={sl(33)}
print("entr :", sorted(entr)); print("mnem :", sorted(mnem)); print("preim:", sorted(pre))
print("entr n mnem :", sorted(entr&mnem))
print("entr n preim:", sorted(entr&pre))
EOF
entr : [50, 56, 62, 69, 75]
mnem : [51, 58, 64, 70, 77]
preim: [75]
entr n mnem : []
entr n preim: [75]
```

- Today every ms1 length identifies its kind. After `0x03`, 75 does not.
- Both carry the id `entr`: `crates/ms-cli/src/cmd/encode.rs:200` `ms_codec::encode(Tag::ENTR, &payload)`, and §4.3 keeps it (*"Singles keep the legacy `entr` id, as mnem does"*).
- Both begin with the same character after the share index: 0x00, 0x02 and 0x03 all have `00000` as their top five bits, and bech32 index 0 is `q`. So an entr-32 seed plate and a preimage plate both read `ms10entrsq…` and are both 75 characters.

**Counterexample or construction.** An operator with a shelf of ms1 plates — some 24-word seed backups (entr-32, 75 chars) and one hashlock preimage (75 chars) — cannot tell them apart by eye, by length, by id, or by the leading characters. Only `ms inspect` resolves it. The consequence is asymmetric: a seed plate under a 2-of-3 needs two siblings to spend, while the preimage plate on a C22 keyless `wsh` path **spends alone** (§5 already records this for `--json`: *"a preimage on a keyless path can spend alone"*). A plate-handling procedure that files "the ms1 plates" as one class gives a bearer instrument the protection of a share.

*Not a defect, and worth recording as the mitigating half:* the two strings are at least nine characters apart, so no misread or repair can silently convert one into the other. BIP-93 on the short checksum: *"guarantees detection of any error affecting at most 8 characters"*, and it *"can correct up to 4 character substitutions"* — while `ms repair` performs *"full BCH(93,80,8) correction up to t=4"* (`crates/ms-cli/src/cmd/repair.rs:4-5`). A one-symbol prefix change forces a checksum change; the codeword distance dwarfs the correction radius.

**Remedy.** State "length no longer implies kind" explicitly in MIGRATION.md, the ms spec and the manual, as a first-class consequence rather than a footnote to "as mnem does". Require the engraving card and any plate template to carry the kind in human text next to the string. Consider the operator question below on giving the preimage single its own id.

---

### I-5 — §3.7's threat model misses when H becomes public, and misses cross-protocol phrase reuse

**Claim.** Section 3.7 is right as far as it goes but starts the clock too late and scopes reuse too narrowly. Both gaps change the copy the record commits to in §5.

**Evidence and construction, part (a) — H leaks at the first spend of *any* `wsh` path, not the first spend of the hash path.**

BIP-141, P2WSH: *"The witness must consist of an input stack to feed to the script, followed by a serialized script (witnessScript). The witnessScript (≤ 10,000 bytes) is popped off the initial witness stack."* The whole script — every branch — is published, including `OP_SHA256 <H> OP_EQUAL`.

BIP-341, script path: only *"the second-to-last stack element"* (the executed leaf) and the control block are revealed; Taproot commits *"only the actually executed part of the script to the blockchain, as opposed to all possible ways a script can be executed."*

So: a `wsh` policy with path A = 2-of-3 keys and path B = keyless `sha256(H)` (§4b C16). The operator spends routinely on path A. That transaction publishes H to the world. The grind against the phrase begins then — years before anyone touches the hash path — and it targets a branch that needs no key. §3.7 currently reads as though exposure begins when the hash path is spent. The same policy as `tr` leaks nothing until the hash leaf is used.

**Evidence and construction, part (b) — a spent X is a permanent public verification oracle for the *phrase*.**

§3.7 covers reuse across policies. It does not cover reuse across protocols. Once X is on-chain, anyone can test candidate phrases against X directly — 8.9e4/s hardened, 1.1e10/s sha256 — and a hit yields the plaintext phrase, not just a spend. If that text was also used as a BIP-39 passphrase, as an `me` sealed-payload passphrase, or as an account password, all of them fall with it. The reverse direction is safe (see "Confirmed sound", Q1), which makes this the *only* leak path — and it is the one the record does not name.

**Remedy.** Extend §3.7 to three consequences and carry (a) and (b) into the host card line and the device confirm modal: *"Spending any path of a wsh wallet publishes this digest. Never use this phrase as a passphrase or a password anywhere else — a spend publishes the preimage, and anyone can then test guesses at the phrase itself."*

---

### I-6 — 64 hex characters pasted into the phrase slot derives a different preimage, silently

**Claim.** The design gives `--hex` and `--hashlock-phrase*` the same shaped input and no signal when the operator confuses them, and the confusion produces a valid-looking `hash:` record whose preimage is not the one they hold.

**Evidence.** §4.2 lists `--hex HEX` (*"an existing X, exactly 32 bytes (64 hex characters)"*) and `--hashlock-phrase TEXT` as sibling sources; the phrase rule admits any printable ASCII including hex digits; the warning fires only *"Under 20 characters"*, and 64 ≥ 20. §4.2's `--json` and card both print `preimage_hex`, so hex is the form the operator most often has in hand — L8 even standardises the phrasing *"32 bytes (64 hex characters)"* across every refusal and help line, which trains the operator on the hex spelling.

**Counterexample or construction.** An operator holds `preimage_hex = c4bbcb1f…d4e39a8a` from an earlier card and wants to reprint the `hash:` record. They run `ms hashlock --hashlock-phrase-stdin` (the flag they remember) and paste the 64 hex characters. No warning fires. The tool derives `X' = PBKDF2("c4bbcb1f…", "ms-hashlock-v1", 100000, 32) ≠ X`, prints `hash:<H'>`, and the operator packs and engraves H'. The preimage they backed up does not open that policy; the preimage that does exists only on a card they did not expect to need.

**Remedy.** When a phrase is exactly 64 characters and every one of them is a hex digit, refuse (or warn at the loudest level) naming `--hex` as the executable remedy. The same check is worth applying on the device's phrase screen.

---

### M-1 — `Payload::Preimage([u8; 32])` is unscrubbable under the codec's own caller-wrap contract

**Class: secret-handling — non-gating by the 2026-08-27 ruling.**

**Claim.** The fixed-size array that makes the 32-byte rule structural also makes the secret impossible to scrub through the mechanism the crate documents.

**Evidence.** `crates/ms-codec/src/payload.rs:19-27`: *"the `Vec<u8>` inside `Payload::Entr` is NOT zeroize-wrapped … Callers MUST wrap the byte buffer at the use site (e.g., `let bytes = Zeroizing::new((*p.as_bytes()).to_vec());`)"*. That recipe **copies**: for `Entr` the heap allocation is at least reachable and the copy is the only extra; for an inline `[u8; 32]` the original 32 bytes remain in the `Payload` value itself, which has no `Drop`, and are memcpy'd afresh on every move of the enum (return from `decode`, match by value, pass by value), leaving stack copies nothing can reach.

**Counterexample or construction.** `let (_tag, p) = ms_codec::decode(&ms1)?;` followed by the documented `Zeroizing::new(p.as_bytes().to_vec())` leaves at least two unscrubbed copies of the preimage: the one inside `p`, and the one the return-by-value memcpy left behind.

**Remedy.** `Payload::Preimage(Zeroizing<[u8; 32]>)`, or a newtype implementing `Drop + Zeroize`, so the length rule stays structural and the scrub is not delegated to a contract that cannot be honoured. Filed as a follow-up per the ruling; it does not gate.

---

### M-2 — the host card prints no character count, so an invisible space is undetectable on the host but visible on the device

**Claim.** Every invisible byte the phrase rule admits fails closed except one: the space. The device counts characters on screen (`n/100`); the host does not print the count it already computes.

**Evidence.** §4.2: *"bytes used exactly as typed (no trimming, case folding or normalisation)"*; the card's listed contents include the digest, the operand, the ms1, the hex, the method line, §8i, F-132, the source kind and the short-phrase warning — but not the character count. `--json` does carry `phrase_chars`. §3.6 records the device's *"n/100 counter"*.

**Counterexample or construction.** A phrase file authored in an editor that leaves a trailing space: `ms hashlock --in phrase.txt` strips exactly one LF/CRLF, keeps the space, derives `X₁`. Later the operator types the phrase on the SH2 or into a spend-time tool without it, gets `X₂ ≠ X₁`, and has no signal on either side about which is right. Everything else invisible is refused: LF (0x0A), CR (0x0D), tab (0x09), a UTF-8 BOM (0xEF 0xBB 0xBF) and every non-ASCII byte all fall outside 0x20..0x7E.

**Remedy.** Print `phrase_chars` on the card, next to the method line. It is already computed for `--json`, and it is the one signal that makes a stray space visible on the host.

---

### M-3 — the `hardened` default silently diverges from the only recipe this project has documented

**Claim.** Every existing artefact in this repo teaches the sha256 recipe; the new tool's no-flag default is the other one, and the two produce different digests for one phrase with no diagnostic.

**Evidence.** `SPEC_wallet_policy_composer.md:700-703` (§8i, shipped device copy): *"A passphrase must be hashed to 32 bytes first, then hashed again."* `S4_journey_walk_2026-09-02.md:227-229` (measured 2026-09-03): *"`X = sha256(passphrase)` is the 32-byte preimage, `H = sha256(X)` the digest"*. `FOLLOWUPS.md:15557` (F-465) records the same two `sha256sum` calls. §5 makes the tool default to `hardened`.

**Counterexample or construction.** Recomputed for one phrase:

```
$ python3 - <<'EOF'
import hashlib
p=b"correct horse battery staple"
Xs=hashlib.sha256(p).digest()
Xk=hashlib.pbkdf2_hmac('sha256',p,b"ms-hashlock-v1",100000,32)
print("H (--method sha256)  :", hashlib.sha256(Xs).hexdigest())
print("H (default, hardened):", hashlib.sha256(Xk).hexdigest())
EOF
H (--method sha256)  : b867db875479bcc0287352cdaa4a1755689b8338777d0915e9acd9f6edbc96cb
H (default, hardened): 3cf5d421caf2a9c8eb9de1d400866ea7d475e6ba978861bb0167a37cb70a4c12
```

An operator reproducing last session's `hash:` record with `ms hashlock` and no `--method` gets `3cf5d421…`, not the `b867db87..edbc96cb` on their device.

**Remedy.** Name `--method sha256` as the pre-tool recipe in MIGRATION.md, the ms manual chapter and F-465's `Which hash?` screen hint. The default itself is right (see "Confirmed sound", Q2) — the migration note is what is missing. No funds are at stake today: nothing derived by the old recipe is funded, and F-467's hashlocks must be regenerated regardless.

---

### M-4 — "if unsure later, try both" is durable advice that will outlive its own precondition

**Claim.** The fallback is sound for exactly two methods and becomes wrong the moment a third exists or the salt changes — and it will be written on paper next to a phrase and never revisited.

**Evidence.** §4.2's card copy: *"write this next to your phrase; it is on no plate; if unsure later, try both"*. §5's own salt row: *"changing it after any vector ships is a new method"* — i.e. the method count is expected to be able to grow.

**Remedy.** Keep the full method line (already required) as the primary record and word the fallback so it cannot go stale: *"if the method line is lost, try each method that shipped with the version named on this card."* Compare `[[comments-outlive-their-conditions]]`.

---

### M-5 — `--random`'s card states only the reassuring half

**Claim.** §4.2 gives `--random` a one-sided advisory, and the omitted half is the loss path.

**Evidence.** §4.2: *"`--random`: 32 bytes from the OS CSPRNG (the one shares use). No phrase exists, so nothing can be guessed; the card says so."*

**Counterexample or construction.** With `--random` there is no memorable fallback and no derivable second copy: X exists only on the `--out` ms1 plate and on the card. Losing the plate makes every path gated by that hash permanently unspendable — a strictly worse loss profile than a phrase, which is the trade the operator is buying. F-132's required card line covers "not in the seed backup"; it does not cover "and there is nothing to remember".

**Remedy.** Pair the reassurance with its cost in the same breath: *"No phrase exists, so nothing can be guessed — and nothing can be remembered. This plate is the only copy."*

---

### M-6 — the device's phrase cap is inherited from a plate-legibility constant, and the two caps are independently editable

**Claim.** The hashlock phrase's 100-character limit on the device comes from a constant justified for a different purpose, and nothing binds it to the Rust side.

**Evidence.** `passphrase/passphrase.go:12-13`: *"// MaxLen is a plate-capacity limit chosen for legibility, not a BIP-39 rule."* / `const MaxLen = 100`. §3.6 records the composer reusing `passphrase.ValidatePassphrase`. A hashlock phrase is never engraved (L7: the device *"never stores, shows, engraves or sources a preimage"*), so the legibility rationale does not apply to it. The Rust-primary rule binds the *derivation*; it does not bind a validator constant in a second repo.

**Counterexample or construction.** A later cycle lowers `passphrase.MaxLen` to 80 for plate legibility. Every existing 81–100-character hashlock phrase becomes un-enterable on the SH2, and the digest can no longer be reproduced on the device that is supposed to be able to reproduce it.

**Remedy.** A dedicated constant on each side (`HASHLOCK_PHRASE_MAX_CHARS`), pinned by a lockstep vector row: a 100-character phrase derives byte-identically on both sides, a 101-character phrase is refused on both.

---

### N-1 — §4.2's verb list omits `split` and `combine`, which §4.1 includes

§4.1 H1: *"ms-cli 0.18.0 adds `ms hashlock` plus the kind's arms in decode, inspect, split, combine and refusals in derive and verify."* §4.2's closing paragraph lists only decode, inspect, derive, verify and encode. Per I-3, `combine` is the one that panics if the omission propagates into the plan.

### N-2 — §3.5 names two coordinators; neither name is verified in the record, and one is doubtful

§3.5: *"the coordinator (Liana, Sparrow) places the 32-byte X in the PSBT's sha256-preimage field"*. I could not confirm either. Liana's descriptor policy analysis (`liana/src/descriptors/analysis.rs`) validates a "primary path + timelocked recovery paths" shape whose leaves are checked by `is_single_key_or_multisig()`, which admits `SemanticPolicy::Key` and key-only `Thresh` — no hash surface appears in Liana's own wallet model, though the file imposes no fragment restriction on a foreign descriptor. For Sparrow I found BIP-174 preimage-field support in the ecosystem but no documented preimage-entry UI. **The design does not depend on the two names, but a reader will.** Suggested wording: *"a coordinator able to populate `PSBT_IN_SHA256`"*, with the examples verified before the sentence ships.

---

## Confirmed sound

**Q1 — the hardened construction.** Sound as a KDF. `dkLen = 32 = hLen`, so RFC 8018 §5.2's `"l = CEIL (dkLen / hLen)"` gives `l = 1`: one block, `DK = T_1` exactly, no multi-block truncation and no cost-amplification asymmetry; `dkLen` is trivially within `"at most (2^32 - 1) * hLen"`. The salt is 14 octets, above RFC 8018 §4.1's `"It should be at least eight octets (64 bits) long."` (its *fixedness* is I-1, not its length.)

**Domain separation from BIP-39: total, and the role swap is the reason.** BIP-39: *"we use the PBKDF2 function with a mnemonic sentence (in UTF-8 NFKD) used as the password and the string "mnemonic" + passphrase (again in UTF-8 NFKD) used as the salt. The iteration count is set to 2048 and HMAC-SHA512 is used as the pseudo-random function. The length of the derived key is 512 bits (= 64 bytes)."* Different PRF (SHA-512 vs SHA-256), different `c` (2048 vs 100,000), different `dkLen` (64 vs 32), and BIP-39 puts the user-chosen secret in the **salt** while this puts it in the **password**. An operator reusing one text in both places produces two unrelated outputs; neither derivation reveals anything about the other.

**Domain separation from `me`'s sealed payload: sound, but resting entirely on the salt.** Both are PBKDF2-HMAC-SHA256 at `dkLen = 32` (`crates/me-cli/src/seal/crypto.rs:33-36`, `pbkdf2::pbkdf2_hmac::<Sha256>`). The separator is that `me`'s salt is 16 random bytes (`seal/wire.rs:13`) and this one is the 14-byte ASCII `"ms-hashlock-v1"` — different lengths, so the PBKDF2 messages `S || INT(i)` can never coincide, and the outputs are independent. Worth noting the iteration counts do **not** separate them: `seal/wire.rs:17` `MIN_ITERATIONS = 100_000` is the same count. If the salt scheme ever changes, that is the load-bearing check.

**No cross-use leak in the pre-spend direction.** Given X, nothing about a BIP-39 seed or a sealed-payload key derived from the same text is learnable, and vice versa. The only leak is the post-spend oracle, which is I-5(b).

**Iteration count against the L3 budget: internally consistent.** §3.4's measured 9,715 it/s gives 10.3 s on the SH2 and 103 s at one tenth. Memory-hard alternatives are correctly excluded on 520 KB of RAM.

**HMAC key pre-hashing is a non-issue here.** HMAC-SHA256 replaces keys longer than the 64-byte block with their SHA-256, so phrases of 65–100 characters are keyed by `SHA-256(phrase)`. The colliding partner would be a 32-byte raw-binary key, which the printable-ASCII rule makes unreachable, so no two admissible phrases collide. (Corollary worth stating in the spec: no entropy is gained past 64 characters.)

**Guesses-per-second, stated so the floor rests on a number:** 8.86e4/s per RTX 4090 hardened, 1.1e10/s sha256, at an assumed **2 bits of entropy per character** of chosen memorable English. See C-1 and I-1.

**Q2 — the two methods cannot produce related outputs.** PBKDF2's first inner call is `HMAC-SHA256(key = phrase, msg = salt || 0x00000001)`, which keys on the phrase rather than hashing it, and 99,999 further iterations follow; `SHA-256(phrase)` appears nowhere in the hardened chain. Measured for one phrase, the two X differ in **127 of 256 bits** — the expected 128 for independent outputs.

**`hardened` is the right default.** It is the method whose failure mode is recoverable ("try both" resolves a 1-bit ambiguity) and whose guessing cost is 124,060× higher; the bip48 precedent in §5 (permissive on input, expressive on output) applies, and L3's external-backup requirement is served by the method line either way. The two caveats are C-1 (the floor must not be shared) and M-3 (the migration note).

**"Try both" is sound as a recovery procedure** for exactly two methods, given the operator still has H to compare against — see M-4 for its expiry.

**Q3(a) — the compiled script, two independent implementations.** `miniscript-12.3.6/src/miniscript/astelem.rs:369-374`:
```rust
Terminal::Sha256(ref h) => builder
    .push_opcode(opcodes::all::OP_SIZE)
    .push_int(32)
    .push_opcode(opcodes::all::OP_EQUALVERIFY)
    .push_opcode(opcodes::all::OP_SHA256)
    .push_slice(Pk::to_sha256(h).to_byte_array())
    .push_opcode(opcodes::all::OP_EQUAL),
```
and its own round-trip test at `mod.rs:1096-1099`: `"OP_SIZE OP_PUSHBYTES_1 20 OP_EQUALVERIFY OP_SHA256 OP_PUSHBYTES_32 e3b0c442… OP_EQUAL"` (`20` = 0x20 = 32). Independently, the Ledger Bitcoin app compiles the identical sequence (`app-bitcoin-new/src/handler/lib/policy.c`, `commands_sha256[]`: `OP_SIZE`, push 1, 32, `OP_EQUALVERIFY`, `OP_SHA256`, `CMD_CODE_PUSH_HASH32`, `OP_EQUAL`), and lists `TOKEN_SHA256` in `fragment_whitelist_wsh`. The record's §3.2/§6c statement of the script is exact. Stronger still: rust-miniscript's satisfier type is `pub type Preimage32 = [u8; 32]` (`satisfy.rs:27`) — a non-32-byte preimage cannot even be *expressed*.

**Q3(b) — single SHA-256, in both contexts.** `push_opcode(OP_SHA256)`, not `OP_HASH256` (the `Terminal::Hash256` arm at `astelem.rs:376-381` is the double, and the composer does not compose it: `SPEC_wallet_policy_composer.md:126` — *"`hash256`/`ripemd160`/`hash160` stay decodable, not composable"*). The `encode` implementation is generic over the script context, so a tapscript leaf emits the same bytes; BIP-342 does not redefine `OP_SHA256`. Ledger whitelists `TOKEN_SHA256` for tapscript too.

Also confirmed: **L3's ripemd160 reasoning is right on both counts.** `Terminal::Ripemd160` (`astelem.rs:383-388`) still emits `OP_SIZE 32 OP_EQUALVERIFY`, so *every* miniscript hash fragment demands a 32-byte X; and RIPEMD-160 outputs 20 bytes, so it cannot be X's derivation.

**Q3(c) — the PSBT field, with a caveat the record should carry.** BIP-174: `PSBT_IN_SHA256 = 0x0b`, keydata `<32-byte hash>` = *"The resulting hash of the preimage"*, valuedata `<bytes preimage>` = *"The hash preimage, encoded as a byte vector, which must equal the key when run through the `SHA256` algorithm"*. rust-bitcoin models it as `pub sha256_preimages: BTreeMap<sha256::Hash, Vec<u8>>` (`bitcoin-0.32.9/src/psbt/map/input.rs:105`). **BIP-174 imposes no length constraint on the preimage** — so a coordinator will happily carry F-467's 40-byte phrase in a well-formed PSBT, and the failure surfaces only at script execution. That is worth one sentence in the spec: the PSBT layer is not a guard.

I found no signer that derives a preimage from a phrase; Ledger consumes the fragment but has no preimage-derivation surface, and the coordinator finalises. §3.5's substance stands. Its two example names do not (N-2).

**Q3(d) — X is public after a spend, in every script type that can carry a hash.** `wsh`: X is a witness stack element (BIP-141). `tr` leaf: X is a witness stack element under script-path spending (BIP-341). `sh(wsh)`: **moot** — `SPEC_wallet_policy_composer.md:117` admits `sh(wsh)` *"ONLY a single path that is an unlocked, unhashed key set"*, so the composer can never place a hash there. Worth saying so explicitly rather than listing three types.

**Q4 — older readers fail safely, traced.** Rust: `crates/ms-codec/src/envelope.rs:215-217`, the `other =>` arm of `dispatch_payload` → `Err(Error::ReservedPrefixViolation { got: other })`. Go, on every flashed SH2 before H2: `codex32/mspayload.go:56-57`, the `default:` arm of `DecodeMS1` → `errMSBadPrefix` (`"codex32: not an m-format secret payload"`). Both are refusals; neither can produce a seed. A `0x03` string reaching `me`'s classifier is 75 ≤ `MAX_ENGRAVEABLE_MS1_LEN` 90 and would be offered at seed entry, then refused at decode — a dead end, not a wrong result.

**Q4 — the share axis is untouched and cannot be confused.** MIGRATION.md invariant 2: *"A reader MUST dispatch on the **threshold char first**, before interpreting the payload byte… `threshold ∈ '2'..'9'` → this is one share of a K-of-N set"*, surfacing `Error::IsShareNotSingleString`. So a share of a preimage can never enter prefix dispatch, and MIGRATION.md invariant 1's rule — *"a distributed share's first payload byte is a Lagrange-interpolation output, NOT a stable prefix"* — is not violated by adding a kind. A recovered `0x03` secret-at-S re-enters through `dispatch_payload`, which is exactly the header-gate-free tail the crate factored out for this purpose.

**Q4 — no reader can confuse a preimage string with an entr string by misreading.** BIP-93's short checksum *"guarantees detection of any error affecting at most 8 characters"* and corrects *"up to 4 character substitutions"*; `ms repair` performs correction *"up to t=4"* and demotes any applied substitution to an unverified candidate (exit 4, `cmd/repair.rs:17-26`). The two codewords are ≥9 characters apart. The confusion that *is* real is human and structural, not wire-level — I-4.

**Q4 — the `[u8; 32]` API.** Sound for making the length rule structural; unsound against the zeroize contract (M-1) and it removes the refusal path the record promised (I-2).

**Q5 — the phrase rule itself is sound, and host and device admit exactly the same set.** The device keyboard covers precisely the 95 printable-ASCII characters, verified:
```
$ python3  # union of ppPageLower/Upper/Symbols/Symbols2 + space, gui/passphrase_keyboard.go:19-23
keyboard count: 95
missing from keyboard: []
extra: []
```
so nothing typeable on the host is untypeable on the device or vice versa. Byte-exactness (no trimming, no case folding, no normalisation) is the right rule and is *stronger* than BIP-39's NFKD, which the fork already documents as unnecessary on ASCII (`passphrase/passphrase.go:4-8`). Every invisible byte except the space fails closed. The three gaps are I-6 (64 hex), M-2 (spaces) and M-6 (the coupled cap).

**Q6 — `--random`'s entropy source is sound.** `crates/ms-codec/src/shares.rs:43` `getrandom::fill(&mut raw).expect("getrandom::fill must not fail")`, `getrandom = "0.3"` (`Cargo.toml:18`) — the OS CSPRNG, and the `.expect` fails closed: a panic can never emit a weak preimage. `--random` is correctly described as the strongest form (M-5 is about the copy, not the source).

**Q6 — §3.7 confirmed as far as it goes.** Both stated consequences are correct: X in the witness is permanent and global, and a phrase reused across two policies hands the second away with the first spend. Sharpened by I-5.

**Q6 — C22 bearer-plate framing confirmed.** §5 already records *"a preimage on a keyless path can spend alone"*; I-4 adds that the plate is visually indistinguishable from an entr-32 seed plate.

**Q6 — the 10-second on-device derivation with the phrase in RAM** is a **secret-handling** matter and is non-gating by the 2026-08-27 ruling. Recorded, not a finding: the composer's single `defer st.reg.scrub()` at the flow top (`gui/composer_sources.go:222-230`) plus L7's explicit scrub of X is the right shape; the phrase itself lives in the keyboard's `Fragment` for the duration of entry.

**Q6 — F-132's wording is correct.** *"Spending either requires revealing the 32-byte preimage `X` where `H = sha256(X)`"* and its worked `X = sha256("opensessame")` are exactly the `--method sha256` construction. Nothing in F-132 needs correcting; §4.2 is right to put its line on the card.

**Q7 — F-467 CONFIRMED: the three hashvault hashlocks are unspendable.**
```
$ python3 - <<'EOF'
import hashlib
for p in ["correct horse battery staple vault alpha",
          "seven bridges over a quiet river bravo",
          "the last plate rings twice charlie"]:
    b=p.encode(); print(f"len={len(b):3d}  H={hashlib.sha256(b).hexdigest()}  {p!r}")
EOF
len= 40  H=ede0b28a805feca09a77c48f82babc1249c79aa840de94fa4a85bf55a453c813  'correct horse battery staple vault alpha'
len= 38  H=9ce09a8df1d1f8bc8892441a9eb30d0a18bc7db81bb0fb3f54c6d9064e7b76ad  'seven bridges over a quiet river bravo'
len= 34  H=4743d7c47df21d29e3ed3dfec5d0c0a884ccc2708637dddf771c36d214056954  'the last plate rings twice charlie'
```
`design/journeys/derive-hashvault-keys.sh:67-77` sets `h=$(printf '%s' "${PRE[$i]}" | sha256sum …)` and uses it as the policy's H, so the only known preimage of each H is the phrase itself at 40/38/34 bytes. The script is `OP_SIZE <32> OP_EQUALVERIFY OP_SHA256 <H> OP_EQUAL` (astelem.rs:369-374, cited above), so a 40-byte witness element fails at `OP_EQUALVERIFY` before `OP_SHA256` runs. Producing a *32-byte* preimage of those H requires a SHA-256 second-preimage attack (2^256). **Unspendable, exactly as filed.** Two additions for the fix: the failure is invisible in PSBT (BIP-174 does not bound the preimage length, Q3(c)) and rust-miniscript cannot even represent the satisfaction (`Preimage32 = [u8; 32]`), so the journey's transcript would have printed success right up to broadcast.

**Q8 — everything else I would refuse to sign off on is C-1, I-1 through I-6.** The wire format, the kind's placement in the codec, the share axis, the script fact and the KDF primitive are all correct.

---

## Questions for the operator

1. **Should `--method sha256` refuse below an entropy floor rather than warn?** L5 gives the operator the choice of method; it does not settle whether the tool must present both as safe at the same phrase length. C-1's remedy needs one of: a method-dependent floor, a refusal, or a requirement that `--method sha256` phrases come from a generator.
2. **Should the floor be stated in bits with a recommended generator, rather than in characters?** Characters are what the device counts and what an operator can check; bits are what an attacker spends. A recommendation ("five diceware words, or `--random`") converts the warning from a scold into an action.
3. **Is a `--salt` flag worth the extra line in the external backup?** §5's veto row already permits any fixed ASCII string. It would defeat I-1's global precomputation at the cost of one more thing L3 requires the operator to write down. This is a trade only the operator can price.
4. **Should the preimage single get its own 4-char id instead of `entr`?** It would break I-4's 75-character/`entr`/`q` collision at the cost of `RESERVED_ID_BLOCKLIST` churn and a departure from the "singles keep the legacy id" precedent. If the answer is no, the mitigation is entirely in the card and plate copy.
5. **Does the device leg (§4.4) want the 64-hex refusal of I-6?** The device's `Which hash?` already offers `Type 64 hex` as a separate row, so an operator on the phrase screen typing 64 hex characters has visibly picked the wrong row — the same check may be cheaper there than on the host.

---

## Sources consulted

**mnemonic-engrave (`72081c5`)**
- `design/BRAINSTORM_hashlock_phrase.md` — all sections (the artifact)
- `design/SPEC_wallet_policy_composer.md:117` (§4a `sh(wsh)`), `:126,128,129` (§4b `HASH`, keyless path, policy), `:380-387` (§6c), `:691-696` (§8h), `:698-703` (§8i), `:1042-1060` (§14)
- `design/FOLLOWUPS.md:4298-4322` (F-132), `:15555-15561` (F-465), `:15563-15570` (F-466), `:15572-15574` (F-467), `:15576-15578` (F-468)
- `design/S4_journey_walk_2026-09-02.md:221-243` (W-5)
- `design/journeys/derive-hashvault-keys.sh:55-80`
- `crates/me-cli/src/seal/crypto.rs:1-3,33-36`; `crates/me-cli/src/seal/wire.rs:13,17-18`

**mnemonic-secret (`7fc1e58`)**
- `MIGRATION.md:15-23` (v0.1→v0.2 invariants 1-4)
- `design/SPEC_ms_v0_2_kofn.md:20-27` (decode dispatch, bounds, registry table)
- `crates/ms-codec/src/consts.rs:17,33,39,43,63,72`
- `crates/ms-codec/src/envelope.rs:160-163,180-220` (`dispatch_payload`), `:196-201` (audit I9 comment), `:208-209`, `:215-217`, `:231-250` (`payload_wire_bytes`)
- `crates/ms-codec/src/payload.rs:19-27` (caller-wrap contract), `:66-93` (`validate`)
- `crates/ms-codec/src/shares.rs:37-43`; `crates/ms-codec/Cargo.toml:18`
- `crates/ms-cli/src/cmd/combine.rs:157-166`; `decode.rs:87-112`; `encode.rs:200`; `payload_lang.rs:35-62`; `verify.rs:88-104`; `derive.rs:429-435`; `inspect.rs:110-135`; `repair.rs:1-49`; `split.rs:108,125-126`

**seedhammer fork (`70008da`)**
- `codex32/mspayload.go:1-60` (`DecodeMS1`, `:51`, `:56-57` default arms)
- `gui/composer_hash.go:1-100`
- `gui/passphrase_keyboard.go:19-23` (the four pages), `:33-40`
- `passphrase/passphrase.go:4-8,12-13,23-38`

**Crates (`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`)**
- `miniscript-12.3.6/src/miniscript/astelem.rs:369-395`; `src/miniscript/satisfy.rs:27`; `src/miniscript/mod.rs:1096-1099`
- `bitcoin-0.32.9/src/psbt/map/input.rs:105,452`
- `pbkdf2-0.12.2`, `bip39-2.2.2` (presence/API only)

**Standards and remote sources (quoted verbatim in the findings)**
- BIP-174 — https://github.com/bitcoin/bips/blob/master/bip-0174.mediawiki — `PSBT_IN_SHA256 = 0x0b`; *"The hash preimage, encoded as a byte vector, which must equal the key when run through the `SHA256` algorithm"*
- BIP-39 — https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki — *"…the string "mnemonic" + passphrase … used as the salt. The iteration count is set to 2048 and HMAC-SHA512 is used as the pseudo-random function. The length of the derived key is 512 bits (= 64 bytes)."*
- BIP-93 — https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki — short checksum *"guarantees detection of any error affecting at most 8 characters"*; *"can correct up to 4 character substitutions"*; *"a payload which is a sequence of up to 74 bech32 characters"*
- BIP-141 — https://github.com/bitcoin/bips/blob/master/bip-0141.mediawiki — *"The witness must consist of an input stack to feed to the script, followed by a serialized script (witnessScript). The witnessScript (≤ 10,000 bytes) is popped off the initial witness stack."*
- BIP-341 — https://github.com/bitcoin/bips/blob/master/bip-0341.mediawiki — *"only the actually executed part of the script to the blockchain, as opposed to all possible ways a script can be executed"*
- RFC 8018 — https://www.rfc-editor.org/rfc/rfc8018.txt — §5.2 *"l = CEIL (dkLen / hLen)"*, *"dkLen, intended length in octets of the derived key, a positive integer, at most (2^32 - 1) * hLen"*; §4.1 *"It is difficult for an opponent to precompute all the keys, or even the most likely keys, corresponding to a dictionary of passwords."*, *"It should be at least eight octets (64 bits) long."*
- LedgerHQ/app-bitcoin-new — https://github.com/LedgerHQ/app-bitcoin-new/blob/develop/src/handler/lib/policy.c — `commands_sha256[]`, `fragment_whitelist_wsh[]` containing `TOKEN_SHA256`
- hashcat v6.2.6, single RTX 4090 — https://gist.github.com/Chick3nman/32e662a5bb63bc4f51b847bb422222fd — `-m 10900` *"Speed.#1.........: 8865.7 kH/s"* at *"[Iterations: 999]"*; `-m 1400` *"Speed.#1.........: 21975.5 MH/s"*

**Computations run on this machine (all reproduced inline above)**
- `openssl kdf -keylen 32 -kdfopt digest:SHA256 -kdfopt "pass:correct horse battery staple" -kdfopt salt:ms-hashlock-v1 -kdfopt iter:100000 PBKDF2` → `c3e97525442520da4cffd5f57aae3f6273990017f2e0fa30c056e32172e22016`, byte-identical to Python's `hashlib.pbkdf2_hmac` — the hardened derivation reproduces in both tools, as §4.3's vector plan requires.
- The secp256k1 + base58 script deriving `1JwSSubhmg6iPtRjtyqhUYYH7bZg3Lfy1T` from `sha256("correct horse battery staple")`: pure-Python scalar multiplication over secp256k1 (`p = 2**256-2**32-977`, standard G), uncompressed SEC pubkey → `ripemd160(sha256(·))` → version byte 0x00 → base58check.
