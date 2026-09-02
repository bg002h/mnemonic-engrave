# Recon — does a coordinator accept ONE master fingerprint at TWO hardened accounts?

Agent: composer recon (same-fingerprint / two-accounts import). Date: 2026-09-01.
Question: do Bitcoin Core, Sparrow, Nunchuk and Liana **accept** a wallet whose
cosigner list carries the same master fingerprint at two different hardened
accounts (one person, two keys, one seed) — or do they dedupe / refuse / warn
keyed on the fingerprint?

## Verdict

**Nobody dedupes on fingerprint, and only Liana refuses on it — deliberately, and
only within a single spending path.** Every implementation examined keys key
identity on the **key material** (Core: the derived `CPubKey`; Nunchuk: the full
`[fp/path]xpub/<0;1>/*` descriptor-key string; Sparrow: the xpub; Liana:
`(Xpub, DerivPaths)`), so two distinct xpubs from one master are two distinct
cosigners everywhere. Core **imported W2 and W3 with `"success": true`** and the
resulting PSBT carried **three separate `bip32_derivs`, two of them sharing
`73c5da0a` at different paths** — nothing collapsed, which is exactly what a
signer needs in order to sign twice. The one real refusal is Liana's
`DuplicateOriginSamePath`, which fires when one fingerprint appears **twice inside
one spending path** (W3's shape) and explicitly does *not* fire when it appears
once in the primary and once in the recovery path (W1/W2's shape — Liana accepted
W2). W1 and W3 are refused by Liana for **shape** reasons, not fingerprint ones:
proved by re-running both with all-distinct fingerprints, which changed nothing.
Two findings beyond the question asked: **Nunchuk's per-transaction signing
progress map is fingerprint-keyed** (`std::map<std::string,bool>`), so two keys of
one seed collapse to one row in the "who has signed" view even though the wallet
holds both signers; and **the depth-0 re-serialisation is a real risk on Ledger
specifically** — Ledger decides "is this key mine" by `memcmp` of the *whole*
serialized xpub against one it re-derives at the declared path, and a depth-0
record cannot equal a depth-4 one. Sparrow accepts the wallet silently with no
warning at all, which for W3 means a "3-cosigner" 2-of-3 that is really 1-of-2.

## Table

| Wallet | W1 `tr` two-path | W2 `wsh` two-path | W3 `wsh sortedmulti` single path |
| --- | --- | --- | --- |
| **Bitcoin Core** | **UNVERIFIED locally** — local node is v25.0.0, which has no tapscript miniscript (`"Miniscript expressions can only be used in wsh"`) and no BIP-389 multipath (`"Multi: Key path value '<0;1>' is not a valid uint32"`). Tapscript miniscript landed in **v26.0** (PR #27255) and W1's shape matches Core's own test vectors — so ACCEPT is expected on ≥26 but was not run. No fingerprint objection exists in either path. | **ACCEPT (MEASURED)** — `getdescriptorinfo` `issolvable:true`; `importdescriptors` → `"success": true`; derives `bc1qjmhvhdj70u2xwarv4ssznk0ftfl3tzdghuvcu8uy9t5kuwrg6zps4megn2` | **ACCEPT (MEASURED)** — same; derives `bc1q2sz6vvu6k7y9gtc6kfgfe0p6xkhmvmdlu97eecjkykpdktvps08scdjgr5` |
| **Sparrow** | **REFUSE — no miniscript (MEASURED, sibling agent)** — `tr(NUMS,{multi_a…})` rejected at parse: *"Cannot determine the multisig threshold"* (the `multi\(` regex does not match `multi_a(`). Issue #1700 open; maintainer: *"supporting any miniscript expression is not a goal."* | **REFUSE — no miniscript (MEASURED, sibling agent)** — parses without error but silently mis-collapses to a flat sortedmulti, then throws `IllegalStateException` on address derivation. **Silent mis-parse, not a clean refusal.** | **ACCEPT, no warning (MEASURED, sibling agent)** — 3 distinct `Keystore`s, `Wallet.isValid()` → `true`, `checkWallet()` throws nothing |
| **Nunchuk** | **ACCEPT w.r.t. fingerprints (SOURCED)** — `ParseTrDescriptor` keys `signers_map` on the key string; miniscript-template support not runnable here → overall admission UNVERIFIED | **ACCEPT w.r.t. fingerprints (SOURCED)** — `ParseWshDescriptor` miniscript path, same keying; overall admission UNVERIFIED | **ACCEPT (SOURCED)** — `ParseSortedMultiDescriptor` pushes into a `std::vector<SingleSigner>` with **no dedup of any kind**; `n = parts.size()-1 = 3` |
| **Liana** | **REFUSE (MEASURED)** — `IncompatibleDesc`. **Not a fingerprint refusal**: re-run with all fingerprints distinct → identical refusal | **ACCEPT (MEASURED)** — primary `Multi(2,…)`, recovery `[26280]`; the shared `73c5da0a` sits in *different* spending paths | **REFUSE (MEASURED)** — `IncompatibleDesc` (a bare sortedmulti has no recovery path). **Not a fingerprint refusal**: all-distinct fingerprints → identical refusal |
| **Ledger** (BIP-388 registration) | no fingerprint rule (SOURCED) | no fingerprint rule (SOURCED) | no fingerprint rule (SOURCED) |

Ledger, all three: a fingerprint match is explicitly treated as a possible false
positive and then confirmed by re-deriving the pubkey; V2 policies permit
`n_internal_keys > 1` (only V1 required exactly 1), so one master at two slots is a
*supported* case. Registration is gated on script support and on the **depth-0 xpub
risk in §6**, never on repeated fingerprints.
Liana's fingerprint refusal, isolated (MEASURED):

| control | result |
| --- | --- |
| C1 — W2 with the same fp **twice inside one `multi()`** | **REFUSE**: `Key '[73c5da0a/48'/0'/3'/2']xpub…' is derived from the same origin as another key present in the same spending path. It is not possible to use a signer more than once within a single spending path.` |
| C2 — W2 with all fingerprints distinct (baseline) | ACCEPT |
| C3 — true duplicate xpub across paths | REFUSE: `Miniscript error: 'Miniscript contains repeated pubkeys or pubkeyhashes'.` |

## Answers to points 1–6

### 1. Core — `importdescriptors` acceptance; fingerprint-keyed dedup? — MEASURED + SOURCED

**MEASURED (local `bitcoind` v25.0.0, `/Satoshi:25.0.0/`, mainnet datadir, no peers):**
W2 and W3 both returned `{"success": true}` for the receive and change descriptors,
both wallets then produced addresses and retained 2 descriptors each. No warning
field, no fingerprint complaint.

**MEASURED — there is no fingerprint-keyed dedup, and the key-keyed one is real.**
Controls run on the same node:

- `wsh(sortedmulti(2,A,B,A))` with **A literally repeated** → **ACCEPTED**,
  `issolvable:true`. Same for `tr(NUMS,multi_a(2,A,B,A))`. So `multi`/`sortedmulti`/
  `multi_a` carry **no uniqueness check at all**.
- `wsh(or_d(multi(2,A,B,C),and_v(v:pkh(A),older(26280))))` → **REFUSED**:
  `is not sane: contains duplicate public keys`. This is miniscript's sanity check.
- The same descriptor with the duplicate given a **different declared origin**
  (`[deadbeef/9'/9'/9'/9']`, same xpub bytes) → **still REFUSED, identical message**.
  So the check is keyed on **key material and ignores the origin entirely** — which
  is the mirror image of the question: different keys with the same fingerprint pass,
  the same key with different fingerprints does not.

**MEASURED — PSBT origin handling does not collapse.** On regtest (mainnet xpubs
converted to tpubs, key material preserved — the resulting witness program matches
the mainnet W3 address), the W3 watch-only wallet was funded and asked for a PSBT.
`decodepsbt` input 0:

```
pubkey=0299e0abe1239f349e6dc352...  fp=73c5da0a  path=m/48'/0'/2'/2'/0/0
pubkey=02cdc49e39ddebe2a8b82f8c...  fp=73c5da0a  path=m/48'/0'/1'/2'/0/0
pubkey=03dc1953c2756c7c58d4f48c...  fp=1b2c3d4e  path=m/48'/0'/0'/2'/0/0
```

Three entries; the two sharing `73c5da0a` are preserved at **distinct paths**.

**SOURCED (Core master, sibling agent `composer-recon-core-same-fingerprint.md`):**
the miniscript uniqueness check is `CheckDuplicateKey`, keyed on the derived
`CPubKey` via `KeyParser::KeyCompare` (`src/script/descriptor.cpp:2250`), never the
fingerprint. `PSBTInput::hd_keypaths` is `std::map<CPubKey, KeyOriginInfo>`
(`src/psbt.h:293`) and every `SigningProvider::GetKey`/`GetKeyOrigin` lookup is by
`CKeyID`, never by fingerprint alone. `getdescriptorinfo` has no `warnings` field;
`importdescriptors`' warnings cover only unsafe `older()` locktimes. Tapscript
miniscript landed in **v26.0** (PR #27255).

### 2. Sparrow — MEASURED (sibling agent), see `composer-recon-sparrow-same-fingerprint.md`

Keystores are identified by **nothing fingerprint-shaped**: `Keystore`/`Persistable`
have **no `equals()`**, and `Wallet.keystores` is a **`List`, not a `Set`**, so no
silent collapse is even possible. The only duplicate check,
`containsDuplicateExtendedKeys()`, compares **xpubs**, so it cannot catch a shared
fingerprint. A grep of both the drongo and sparrow trees found **zero**
fingerprint-uniqueness checks. Running real Java against drongo HEAD (`d666943`)
with cosigners A and C sharing one master fingerprint produced **3 distinct
`Keystore` objects**, `Wallet.isValid()` → `true`, `checkWallet()` threw nothing.
Sparrow's "Apply" button is gated purely on `isValid()`, so **W3 ships silently**
as a nominal 3-cosigner wallet whose real security is 1-of-2.

Miniscript: **not supported** (issue #1700 open). drongo's `Miniscript.java` is a
59-line regex heuristic, not an AST engine. W1 is rejected at parse time; **W2
parses without error, mis-collapses to a flat sortedmulti, and only fails later at
address derivation** — a silent mis-parse rather than a refusal, which is worse
than a clean rejection.

### 3. Nunchuk — SOURCED (C++, not built here)

`signers_map` is **`std::map<std::string, SingleSigner>` keyed on the full
descriptor-key string** — `[fp/path]xpub/<0;1>/*`, not the fingerprint
(`descriptor.cpp:527`, `:611`). Two different xpubs are two different map keys, so
W1/W2 keep both slots. `ParseSignerString` (`descriptor.cpp:405`) captures the xfp
via `SIGNER_REGEX` (`descriptor.cpp:394`) but only stores it as a field of
`SingleSigner`; identity is never derived from it.

W3 goes through `ParseSortedMultiDescriptor` (`descriptor.cpp:429–453`), which
`signers.push_back(...)` in a loop with **no dedup and no set**, and computes
`n = parts.size() - 1`. Two signers sharing `73c5da0a` therefore both survive as a
2-of-3. **Nothing collapses.**

The only set-based dedup in the file is `std::set<std::string> signerStr`
(`descriptor.cpp:485–492`) in `ParseMusigWallet`, and it is keyed on the **full key
string** `parts[i]`, not the fingerprint.

**Extra finding, beyond the question — a fingerprint-keyed map that does collapse.**
`class Transaction` exposes `std::map<std::string, bool> const& get_signers() const;`
(`include/nunchuk.h:1174`), and libnunchuk's own example drives it as
`signers.find(xfp)` where `xfp = device.get_master_fingerprint()`
(`miniscriptwallet.cpp:304–305`). So the **per-transaction "who has signed" view is
keyed on master fingerprint** and shows one row for two keys of one seed, even
though the wallet correctly holds both signers. `get_signed()` returns a
`std::vector<SingleSigner>`, so the underlying signature set is fine — this is a
progress/UI-level collapse, not a funds defect, but it is exactly the "surprising
behaviour" Liana cites as its reason for refusing.

**Caveat (UNVERIFIED):** whether W1/W2 pass `Utils::IsValidMiniscriptTemplate` was
not run (no build). Separately, `ParseDescriptors` re-serialises the parsed wallet
and requires it to **string-match the input** across four `DescriptorPath` forms,
else `"Failed to verify wallet descriptor"` — an acceptance risk driven by
formatting/ordering that is independent of fingerprints.

### 4. Liana — MEASURED, against repo master

Built `liana` **15.0.0** from `github.com/wizardsardine/liana` HEAD
`d8abe6c76cbc5cd62d497c7c00d5f2b950c5e3d7` (2026-09-01) and ran
`LianaDescriptor::from_str` on the real descriptors. Results in the tables above.

**Liana dedupes on BOTH, at different scopes** (`liana/src/descriptors/analysis.rs`):

- **Globally, by key:** `DescKeyChecker.keys_set` is
  `HashSet<(bip32::Xpub, descriptor::DerivPaths)>` (`analysis.rs:84`); a hit raises
  `DuplicateKey` (`analysis.rs:111`).
- **Per spending path, by fingerprint:** `check()` returns the **origin
  fingerprint** (`analysis.rs:103–106`, and its doc comment: *"This returns the
  origin fingerprint for this xpub, to make it possible for the caller to check the
  same signer is never used twice in the same spending path."*). The caller builds
  `origin_fingerprints: HashSet` **per path** and raises
  `DuplicateOriginSamePath` on a hit (`analysis.rs:524–532`). The in-code rationale:
  *"If any two keys share the same origin, they are from the same signer. We
  restrict using a signer more than once within a single spending path as it can
  lead to surprising behaviour"* — citing
  `https://github.com/wizardsardine/liana/pull/706#issuecomment-1744705808`.

So Liana is the **only** implementation here that refuses on fingerprint, it does so
**by explicit design**, and its scope is **within one spending path**. W1/W2 place
the two `73c5da0a` keys in *different* paths and are therefore outside the rule —
confirmed by W2 being accepted.

**Both W1 and W3 refusals are shape, not fingerprint (MEASURED).** Re-running each
with all fingerprints distinct produced the identical `IncompatibleDesc`. Replacing
W1's raw NUMS internal key with an origin-bearing xpub also did not help — Liana's
taproot model expects the primary path at the internal key with recovery leaves in
the tree, not a `multi_a` primary sitting in a leaf.

### 5. Ledger app-bitcoin-new — SOURCED

**No rule about repeated fingerprints in the key vector.** The distinctness check
BIP-388 requires is `count_distinct_keys_info(&policy_map.parsed) != n_keys`
(`src/handler/register_wallet.c:266`), and its implementation
(`src/handler/lib/policy.c:1770–1796`) returns `max(key_index)+1` over the
template's **placeholder indices** — a template/vector arity check with no bearing
on key material or origins.

The load-bearing logic is the internal/external classification
(`register_wallet.c:126–145`). The comment is explicit that a fingerprint match is
**only a hint**:

```
// if there is key origin information and the fingerprint matches, we make sure it's not
// a false positive (it could be wrong info, or a collision).
```

It then re-derives the pubkey at the declared path and `memcmp`s. Two accounts of
one seed both match the fingerprint and both re-derive successfully, so both become
`PUBKEY_TYPE_INTERNAL` and `n_internal_keys` becomes 2 — which V2 permits;
**only V1 required exactly one** (`register_wallet.c:154–158`). One master at two
slots is therefore a *designed-for* case on Ledger.

### 6. Consequence for the composer's copy

Two separate things the device could say, and they are not equally earned.

**(a) The same fingerprint at two slots — worth a WARNING, not a refusal.** Four of
five implementations accept it and Ledger supports it by design, so refusing would
be wrong. But the warning has a real, non-obvious trigger, and it should be
**scoped to a single spending path**:

- Two slots with one fingerprint **inside one spending path** (W3's shape) is the
  case that actually costs something: Liana **refuses** it outright, Nunchuk's
  signing-progress map collapses it to one row, and Sparrow ships it silently as a
  wallet whose advertised 2-of-3 is really 1-of-2. That last one is the funds-shaped
  risk and the strongest reason to warn: **the threshold overstates the security**.
  Suggested wording: *"Slots @0 and @2 are the same seed. This wallet's 2-of-3 can
  be satisfied by one person. Liana will refuse it."*
- Two slots with one fingerprint **across different paths** (W1/W2's shape) is the
  ruling-C5 normal case and is accepted everywhere that supports the script at all,
  Liana included. It warrants at most an informational line, not a warning.

**(b) The depth-0 re-serialisation is a distinct, Ledger-specific risk — and it is
NOT what any host objected to.** Measured: no wallet rejected the depth mismatch.
Core imported W2/W3 and derived addresses; Liana accepted W2 and its own parse shows
the contradiction plainly — `Xpub { depth: 0, parent_fingerprint: 00000000,
child_number: Normal { index: 0 } }` carrying `origin: Some((73c5da0a,
48'/0'/1'/2'))`, i.e. a 4-step origin on a depth-0 record. So for the four
coordinators the answer to the brief's question is: **the depth mismatch is not what
they object to; nothing objects to it.**

Ledger is the exception and it is worth flagging in the composer's copy.
`register_wallet.c:142` does `memcmp(&key_info.ext_pubkey, &pubkey_derived,
sizeof(pubkey_derived))` over the **whole** serialized record — `wallet.c` stores it
as `out->ext_pubkey = ext_pubkey_check.serialized_extended_pubkey;` after
base58-decoding 82 bytes (78 + 4 checksum), i.e. the full BIP-32 record, whose
layout includes depth, parent fingerprint and child number. A depth-0 xpub cannot
memcmp-equal a device-derived depth-4 one, so the operator's own keys would be
classified `PUBKEY_TYPE_EXTERNAL`; if no slot matches,
`n_internal_keys < 1` refuses registration outright with
`EC_REGISTER_WALLET_POLICY_HAS_NO_INTERNAL_KEY` (`register_wallet.c:148–152`).

**Marked UNVERIFIED and recommended as the next thing to check.** I could not fetch
the `serialized_extended_pubkey_t` definition (not at `src/common/bip32.h` on
`develop`) and there is no Ledger hardware or Speculos here, so the final step —
"the struct includes depth, therefore the memcmp fails" — is inference from the
82-byte decode and the BIP-32 record layout, not a measurement. It is cheap to
settle on Speculos and it would change whether md's depth-0 re-serialisation is
harmless or blocks Ledger registration entirely.

## The exact descriptors used

Generated with `md 0.14.0` (`~/.cargo/bin/md`, verified by path), keys from
`design/journeys/inputs-walletpolicy/key{0..3}.xpub`. W1 reproduced the brainstorm
§3.4 checksum `#72a8pans` exactly.

```
W1 (#72a8pans)
tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{multi_a(2,[73c5da0a/48'/0'/1'/2']xpub661MyMwAqRbcG5161axqKvt7Kx7XBe4pWwNMvgbwdffMtvzPnXA85ToGs3EtpEVAAYf9PggopL6xt7ySJw5Kc7ELWVcwopEjYXVHaHy6tFz/<0;1>/*,[1b2c3d4e/48'/0'/0'/2']xpub661MyMwAqRbcGQnC8zMGwRc4EXYJCgXrxx9kXtw1RXqu4TcW26PxHqssgp6sU4N5CR5o9QcUZPG31fzPeUHEVPkFit1WpVZTmqZLzpvZG2s/<0;1>/*,[5f6a7b8c/48'/0'/3'/2']xpub661MyMwAqRbcGeL2SEbsrRH3CCkYtRJF4Gjk9V8LmqyAyS1Gppz67wgGKMPHQowxQzxTq69yHZnxyg9XWBXXNLj5AK8asVFDATmZ7JECRrT/<0;1>/*),and_v(v:pk([73c5da0a/48'/0'/2'/2']xpub661MyMwAqRbcGS5QF7iox7vX4iVsg1dj9epBz8dhpHDRoJTCNYChFEZhZ26Fda2Bz3GyUAUPNGRn1zhwMZktL49GBdUAc1cPCMPNm3cGkqg/<0;1>/*),older(26280))})#72a8pans

W2 (#ss00t074)
wsh(or_d(multi(2,[73c5da0a/48'/0'/1'/2']xpub661MyMwAqRbcG5161axqKvt7Kx7XBe4pWwNMvgbwdffMtvzPnXA85ToGs3EtpEVAAYf9PggopL6xt7ySJw5Kc7ELWVcwopEjYXVHaHy6tFz/<0;1>/*,[1b2c3d4e/48'/0'/0'/2']xpub661MyMwAqRbcGQnC8zMGwRc4EXYJCgXrxx9kXtw1RXqu4TcW26PxHqssgp6sU4N5CR5o9QcUZPG31fzPeUHEVPkFit1WpVZTmqZLzpvZG2s/<0;1>/*,[5f6a7b8c/48'/0'/3'/2']xpub661MyMwAqRbcGeL2SEbsrRH3CCkYtRJF4Gjk9V8LmqyAyS1Gppz67wgGKMPHQowxQzxTq69yHZnxyg9XWBXXNLj5AK8asVFDATmZ7JECRrT/<0;1>/*),and_v(v:pkh([73c5da0a/48'/0'/2'/2']xpub661MyMwAqRbcGS5QF7iox7vX4iVsg1dj9epBz8dhpHDRoJTCNYChFEZhZ26Fda2Bz3GyUAUPNGRn1zhwMZktL49GBdUAc1cPCMPNm3cGkqg/<0;1>/*),older(26280))))#ss00t074

W3 (#vnfl7h4l)
wsh(sortedmulti(2,[73c5da0a/48'/0'/1'/2']xpub661MyMwAqRbcG5161axqKvt7Kx7XBe4pWwNMvgbwdffMtvzPnXA85ToGs3EtpEVAAYf9PggopL6xt7ySJw5Kc7ELWVcwopEjYXVHaHy6tFz/<0;1>/*,[1b2c3d4e/48'/0'/0'/2']xpub661MyMwAqRbcGQnC8zMGwRc4EXYJCgXrxx9kXtw1RXqu4TcW26PxHqssgp6sU4N5CR5o9QcUZPG31fzPeUHEVPkFit1WpVZTmqZLzpvZG2s/<0;1>/*,[73c5da0a/48'/0'/2'/2']xpub661MyMwAqRbcGS5QF7iox7vX4iVsg1dj9epBz8dhpHDRoJTCNYChFEZhZ26Fda2Bz3GyUAUPNGRn1zhwMZktL49GBdUAc1cPCMPNm3cGkqg/<0;1>/*))#vnfl7h4l
```

Single-chain variants (`--chain 0` / `--chain 1`) were generated for Core, which is
v25.0.0 and predates BIP-389: `w1c0 #dy643qug`, `w2c0 #g7tqcjhe`, `w3c0 #y6wdsq0v`.

## What I ran

- **`md 0.14.0`** — generated W1/W2/W3 and their single-chain variants. W1's
  checksum reproduced brainstorm §3.4's `#72a8pans`.
- **`bitcoind` v25.0.0** (reported as Bitcoin Satellite v0.2.4 wrapping Core
  v25.0.0 — a fork, noted as a caveat on the "Core" label), two throwaway nodes on
  free ports in the scratchpad, both stopped afterwards:
  - mainnet, `connect=0`, height 0 — `getdescriptorinfo`, `deriveaddresses`,
    `createwallet` (descriptor, watch-only), `importdescriptors` for W2/W3 plus
    seven controls (T1–T7).
  - regtest, 102 blocks — funded the W3 watch-only wallet and ran
    `walletcreatefundedpsbt` + `decodepsbt` to read `bip32_derivs`. Mainnet xpubs
    were converted to tpubs with a scratch base58 script; the resulting witness
    program matches the mainnet W3 address, confirming key material was preserved.
- **Liana 15.0.0** — `git clone --depth 1` of `wizardsardine/liana` at HEAD
  `d8abe6c`, built `liana/examples/fpprobe.rs` (a scratch example added inside the
  clone so it used Liana's own `Cargo.lock`) calling `LianaDescriptor::from_str`,
  and ran it on W1/W2/W3 plus five controls (C1–C3, W1a/W1b, W3b). A first attempt
  via crates.io `liana = "5"` failed to build on a dependency-version conflict and
  was abandoned in favour of the repo lockfile.
- **libnunchuk** — source reading only (C++, not built):
  `descriptor.cpp`, `descriptor_test.cpp`, `miniscriptwallet.cpp`, `compiler.cpp`,
  plus `include/nunchuk.h` fetched from master.
- **Ledger app-bitcoin-new** — source reading on `develop`:
  `src/handler/register_wallet.c`, `src/handler/lib/policy.c`, `src/common/wallet.c`.
- **Delegated and measured by sibling agents** (their reports are the primary
  record; I did not re-run their work):
  - `design/agent-reports/composer-recon-sparrow-same-fingerprint.md` — Sparrow/drongo,
    real Java executed against drongo HEAD `d666943`.
  - `design/agent-reports/composer-recon-core-same-fingerprint.md` — Bitcoin Core
    **master** source, with URL + file:line for each claim.

Not run, and marked as such above: Core ≥26 (W1's tapscript miniscript), any Ledger
device or Speculos, any libnunchuk build, and Sparrow's GUI.
