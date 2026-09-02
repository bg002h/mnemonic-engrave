# Recon: does Bitcoin Core dedupe/refuse/warn on a descriptor with the same master fingerprint twice at two paths?

**Repo:** github.com/bitcoin/bitcoin, branch `master`, pinned at commit
`dc0395c5858a1d55239b82a834e5075cf2069219` (fetched via
`raw.githubusercontent.com/bitcoin/bitcoin/master/...` on 2026-09-01; all
`file:line` citations below are against that commit).

**Bottom line: NO.** Core never dedupes, refuses, or warns on two distinct
xpubs from one seed (same 4-byte fingerprint, different derivation paths, and
therefore different actual pubkeys) appearing in one descriptor's key list.
Every uniqueness check in the codebase that exists at all is keyed on the
**derived public key**, not the origin fingerprint, and every key **lookup**
in signing/PSBT code is keyed on the pubkey (or its hash160/`CKeyID`), never
on the fingerprint alone. The example descriptor in the prompt (fingerprint
`73c5da0a` at two different paths) is accepted by `descriptor.cpp` with zero
warning, in both the legacy `sortedmulti()` path and, separately, in
miniscript.

---

## 1. `descriptor.cpp` — is there ANY uniqueness check across the key vector?

**SOURCED.** There is exactly one duplicate-key check in the whole file, and
it applies **only** to miniscript expressions (i.e. only reached from the
"process miniscript expressions" fallback block, never from the legacy
`multi`/`sortedmulti`/`multi_a`/`sortedmulti_a` direct-parse block that runs
earlier in the same function).

`src/script/descriptor.cpp:2676-2708` (the `ParseScript` miniscript branch):
```
        const auto script_ctx{ctx == ParseScriptContext::P2WSH ? miniscript::MiniscriptContext::P2WSH : miniscript::MiniscriptContext::TAPSCRIPT};
        KeyParser parser(/*out = */&out, /* in = */nullptr, /* ctx = */script_ctx, key_exp_index);
        auto node = miniscript::FromString(std::string(expr.begin(), expr.end()), parser);
        ...
            if (!node->IsSane() || node->IsNotSatisfiable()) {
                ...
                    } else if (!insane_node->CheckDuplicateKey()) {
                        error += ": contains duplicate public keys";
```
(url: https://github.com/bitcoin/bitcoin/blob/dc0395c5858a1d55239b82a834e5075cf2069219/src/script/descriptor.cpp#L2706-L2707)

Line 2687-2689 gates this to `wsh`/`tr` only:
```
            if (ctx != ParseScriptContext::P2WSH && ctx != ParseScriptContext::P2TR) {
                error = "Miniscript expressions can only be used in wsh or tr.";
                return {};
```

**Legacy `multi`/`sortedmulti`/`multi_a`/`sortedmulti_a` (the user's exact
example, `wsh(sortedmulti(...))`) never reach this check at all.** The whole
handling block, `src/script/descriptor.cpp:2380-2469`, was read in full
(command: `sed -n '2380,2469p'`) — it validates threshold bounds, key count
vs. `MAX_PUBKEYS_PER_MULTISIG`/`MAX_PUBKEYS_PER_MULTI_A`, script size, and
multipath-length consistency, and contains **no** duplicate/uniqueness check
of any kind (no `std::set`, no pubkey comparison, no fingerprint comparison).
This is a **grep-confirmed negative**: `grep -n -i "duplicate\|std::set\|std::unique\|seen\."` over the whole 3092-line file (command run, full output captured) returns only unrelated `std::unique_ptr` hits plus the single `CheckDuplicateKey()` call at 2706 discussed above — nothing else in the file mentions duplicates or uses a set/seen-map to dedupe keys.

**What the miniscript check is keyed on:** the **derived public key bytes**,
not the fingerprint. `KeyParser::KeyCompare`, `src/script/descriptor.cpp:2250-2262`:
```
    bool KeyCompare(const Key& a, const Key& b) const {
        // Deriving a hardened step needs the private key, so use the provider that was filled
        // while parsing, or the one we are inferring from, rather than an empty one.
        const SigningProvider& provider{m_out ? *m_out : (m_in ? *m_in : DUMMY_SIGNING_PROVIDER)};
        const PubkeyProvider& key_a{*m_keys.at(a).at(0)};
        const PubkeyProvider& key_b{*m_keys.at(b).at(0)};
        FlatSigningProvider out_a, out_b;
        const std::optional<CPubKey> pub_a{key_a.GetPubKey(0, provider, out_a)};
        const std::optional<CPubKey> pub_b{key_b.GetPubKey(0, provider, out_b)};
        if (pub_a && pub_b) return *pub_a < *pub_b;
        // Keys that cannot be derived sort before the ones that can, and are compared by their
        // expression so that two different keys are not taken for duplicates.
```
It compares `CPubKey` values obtained from `key_a.GetPubKey(0, provider, out_a)` — the actual derived compressed/x-only pubkey — never `KeyOriginInfo::fingerprint`. `GetFingerprint`/`fingerprint` does not appear anywhere in `KeyCompare` or in the `DuplicateKeyCheck` machinery (see §2). Two xpubs from one seed at different paths derive to different pubkeys, so this comparator treats them as distinct — no dedup, no error — regardless of matching fingerprint.

**Scope note (relevant to §6):** `ParseScript` is called once per leaf/subscript (confirmed at the `tr()` parsing site, `src/script/descriptor.cpp:2553`: `subscripts.emplace_back(ParseScript(key_exp_index, sarg, ParseScriptContext::P2TR, out, error));`), so even where `CheckDuplicateKey()` does run, it only ever inspects one miniscript expression tree (one `wsh()` body, or one tapscript leaf) — never across leaves of one `tr()` tree, and never across a whole descriptor's full key list.

## 2. `miniscript.h`/`.cpp` — `CheckDuplicateKey`/`has_duplicate_keys`: what identity function?

**SOURCED.** `src/script/miniscript.h:1692-1693`:
```
    //! Check whether there is no duplicate key across this fragment and all its sub-fragments.
    bool CheckDuplicateKey() const { return has_duplicate_keys && !*has_duplicate_keys; }
```
Computed by `DuplicateKeyCheck`, `src/script/miniscript.h:1500-1552` (read in
full). The comparator is injected from the caller's context, not fixed:
```
            bool operator()(const Key& a, const Key& b) const { return ctx_ptr->KeyCompare(a, b); }
```
(`src/script/miniscript.h:1507`) — it builds a `std::set<Key, Comp>` per node
and unions children's sets, flagging duplicates when the merged set shrinks.
For descriptors, `Ctx::KeyCompare` is `KeyParser::KeyCompare` from §1 above —
**pubkey-value comparison, not fingerprint, and not key index**. (`Key` here
is `uint32_t`, an index into `DescriptorImpl::m_pubkey_args`, per
`src/script/descriptor.cpp:2233`: `using Key = uint32_t;` — the index itself
is never the identity used for comparison; `KeyCompare` dereferences it to a
`PubkeyProvider` and derives the actual `CPubKey`.)

`src/script/miniscript.cpp` (432 lines, fetched in full) contains **no**
occurrence of "duplicate" (grep run, zero hits) — all of the duplicate-key
logic lives in the header, as shown above.

**Empirical confirmation from Core's own test suite** that this check fires
on identical derived pubkeys (same xpub, same path) and produces exactly the
`descriptor.cpp:2707` error text — `src/test/descriptor_tests.cpp:1120`:
```
CheckUnparsable("wsh(and_v(v:pk(xprv9s21ZrQH143K31xYSDQpPDxsXRTUcvj2iNHm5NUtrGiGG5e2DtALGdso3pGz6ssrdK4PFmM8NSpSBHNqPqm55Qn3LqFtT2emdEXVYsCzC2U/2147483647'/0),pk(xprv9s21ZrQH143K31xYSDQpPDxsXRTUcvj2iNHm5NUtrGiGG5e2DtALGdso3pGz6ssrdK4PFmM8NSpSBHNqPqm55Qn3LqFtT2emdEXVYsCzC2U/2147483647'/0)))", ...,
"and_v(v:pk(xpub661MyMwAqRbcFW31YEwpkMuc5THy2PSt5bDMsktWQcFF8syAmRUapSCGu8ED9W6oDMSgv6Zz8idoc4a6mr8BDzTJY47LJhkJ8UB7WEGuduB/2147483647'/0),pk(xpub661MyMwAqRbcFW31YEwpkMuc5THy2PSt5bDMsktWQcFF8syAmRUapSCGu8ED9W6oDMSgv6Zz8idoc4a6mr8BDzTJY47LJhkJ8UB7WEGuduB/2147483647'/0)) is not sane: contains duplicate public keys");
```
Note both `pk()` calls here are the **same xpub at the same path** — an
exact-pubkey collision, the only case this check is designed to catch. A
same-fingerprint/different-path pair (this recon's actual question) derives
different pubkeys and would not trigger this.

## 3. `scriptpubkeyman.cpp` / `wallet/rpc/backup.cpp` — per-wallet fingerprint check?

**SOURCED negative.** `grep -n -i "fingerprint"` over the full
`src/wallet/scriptpubkeyman.cpp` (1691 lines) returns three hits, all benign
metadata population, not comparison/dedup:
- `src/wallet/scriptpubkeyman.cpp:424`: `info.fingerprint = meta.key_origin.fingerprint;`
- `src/wallet/scriptpubkeyman.cpp:426-427`: `} else { // Single pubkeys get the master fingerprint of themselves` / `info.fingerprint = keyID.fingerprint();`
- `src/wallet/scriptpubkeyman.cpp:597`: `std::string origin_str = has_info ? "[" + HexStr(info.fingerprint) + FormatHDKeypath(info.path) + "]" : "";` (display-string formatting only)

None of these compare one fingerprint against another or reject/dedupe
anything.

`grep -n -i "fingerprint\|warning"` over the full `src/wallet/rpc/backup.cpp`
(653 lines) shows the `importdescriptors` warnings path
(`src/wallet/rpc/backup.cpp:149,187,248-249,258,296,312`) collects warnings
from `parsed_desc->Warnings()` (line 248-249: `for (const auto& w : parsed_desc->Warnings()) { warnings.push_back(w); }`), plus a hardcoded "Not all private keys provided..." (line 258) and "Unknown output type, cannot set descriptor to active." (line 296). **No fingerprint mention anywhere in this file.**

`Descriptor::Warnings()` itself (`src/script/descriptor.cpp:1077-1081`,
populated only at `src/script/descriptor.cpp:1703,1705`) emits exactly two
message shapes, both about `older()` relative-locktime unsafety
(`"time-based relative locktime: older(%u) > (65535 * 512) seconds is unsafe"`
and `"height-based relative locktime: older(%u) > 65535 blocks is unsafe"`) —
nothing about duplicate/repeated fingerprints.

## 4. PSBT / `sign.cpp` — is any key ever looked up by fingerprint alone? (load-bearing)

**SOURCED — no, keys are always looked up by the actual pubkey / its hash,
never by fingerprint alone.**

- PSBT's own on-disk BIP-32-derivation map is keyed by the **pubkey**, not
  the fingerprint: `src/psbt.h:293`: `std::map<CPubKey, KeyOriginInfo> hd_keypaths;`
  (deserializer confirms this too, `src/psbt.h:167-183`,
  `DeserializeHDKeypaths(..., std::map<CPubKey, KeyOriginInfo>& hd_keypaths)`
  → `hd_keypaths.emplace(pubkey, std::move(keypath));` at line 183). The
  fingerprint lives only as a *value* field inside `KeyOriginInfo`, never as
  a map key.
- `SigningProvider`'s core virtual interface, `src/script/signingprovider.h:174,176`:
  ```
  virtual bool GetKey(const CKeyID &address, CKey& key) const { return false; }
  virtual bool GetKeyOrigin(const CKeyID& keyid, KeyOriginInfo& info) const { return false; }
  ```
  Both are keyed by `CKeyID` (= `Hash160(pubkey)`), not by the 4-byte BIP-32
  fingerprint. `FlatSigningProvider::origins` (`src/script/signingprovider.h:238`):
  `std::map<CKeyID, std::pair<CPubKey, KeyOriginInfo>> origins;` — again keyed
  by the pubkey hash, with `KeyOriginInfo` (which contains the fingerprint)
  only as the stored value.
- The actual signing call sites resolve by the pubkey extracted from the
  script being satisfied, never by fingerprint:
  - `src/script/sign.cpp:54-59` (`MutableTransactionSignatureCreator::CreateSig`):
    `if (!provider.GetKey(address, key))` where `address` is a `CKeyID`
    derived from the script's embedded pubkey.
  - `src/script/sign.cpp:120-124` (Taproot script-path signing):
    `if (!provider.GetKey(part_pubkey.GetID(), key)) return {};` — again
    `part_pubkey.GetID()`, the actual (x-only) pubkey's id, not a fingerprint.
  - `SignPSBTInput` (`src/psbt.cpp:650-756`, read in full) does not itself
    look anything up by fingerprint; it delegates entirely to
    `ProduceSignature(provider, creator, utxo.scriptPubKey, sigdata)`
    (`src/psbt.cpp:723,726`), which walks the actual `scriptPubKey`/witness
    script/tapscript and resolves each embedded pubkey via the `GetKey`/
    `CreateSig` chain above.

The only fingerprint *comparison* found anywhere in `sign.cpp` is unrelated
to key lookup — it's a MuSig2 aggregate-key sanity check,
`src/script/sign.cpp:316`: `if (agg_info.fingerprint != agg_pub.GetID().fingerprint()) { continue; }`, which guards a specific BIP-32-tweak-of-an-aggregate-key code path and still isn't used to *find* which private key to sign with.

**Conclusion for the load-bearing question:** Core's whole signing/PSBT
pipeline is pubkey-identified end to end (`CPubKey` in the PSBT map,
`CKeyID` = hash of the pubkey in every `SigningProvider` lookup). Two
accounts of one seed sharing a fingerprint but sitting at different paths
produce different pubkeys/`CKeyID`s and are therefore fully unambiguous to
every lookup found — this is not a partial mitigation, it's structural: no
code path keys any map or lookup by the bare 4-byte fingerprint.

## 5. Does `getdescriptorinfo`/`importdescriptors` warn on repeated fingerprints?

**SOURCED negative.** `getdescriptorinfo` lives in `src/rpc/output_script.cpp`
(not `wallet/rpc/backup.cpp` as the prompt guessed — confirmed by grepping
several candidate files; found at `src/rpc/output_script.cpp:166-215`). Its
full `RPCResult` field list (lines 174-182) is: `descriptor`,
`multipath_expansion`, `checksum`, `isrange`, `issolvable`,
`hasprivatekeys` — **there is no `warnings` field in `getdescriptorinfo`'s
result schema at all**, so it structurally cannot emit any warning, fingerprint-related or otherwise.

`importdescriptors` (`src/wallet/rpc/backup.cpp`, RPC body around
lines 149-312) does have a `warnings` array, sourced only from
`Descriptor::Warnings()` (the two `older()`-locktime messages, per §3) plus
the two hardcoded strings quoted in §3. Confirmed by exhaustive grep of the
warning-string literals in the file — none mention "fingerprint" or
"duplicate".

## 6. W1/W2 miniscript descriptors — accepted on master?

**W1** `tr(50929b74...,{multi_a(2,A,B,C),and_v(v:pk(D),older(26280))})` and
**W2** `wsh(or_d(multi(2,A,B,C),and_v(v:pkh(D),older(26280))))`.

**`multi()` inside miniscript combinators (not top-level-only):** the old
"multi is only a top-level wsh fragment" restriction the prompt asks about is
**not accurate for master**. `Fragment::MULTI` is a real, nestable miniscript
fragment (`src/script/miniscript.h:239`: `MULTI, //!< [k] [key_n]* [n] OP_CHECKMULTISIG (only available within P2WSH context)`), and the **text parser itself** recognizes `"multi("` and `"multi_a("` as tokens inside its combinator grammar alongside `and_v`/`or_d`/`thresh`/etc.
(`src/script/miniscript.h:2037,2039`: `} else if (Const("multi(", in)) {` /
`} else if (Const("multi_a(", in)) {`, in the same else-if chain as
`and_v(`/`or_d(` etc. at lines 2061-2079). The only restriction is
**context**, not nesting position: `MULTI` is rejected in Tapscript context
and `MULTI_A` is rejected outside Tapscript (enforced in the script-decode
direction at `src/script/miniscript.h:2380` `if (IsTapscript(ctx.MsContext())) return {};` for `MULTI`, and `:2398` `if (!IsTapscript(ctx.MsContext())) return {};` for `MULTI_A`).

Bitcoin Core's own miniscript test suite has this exact shape as a
`TESTMODE_VALID` vector — `src/test/miniscript_tests.cpp:568`:
```
Test("or_d(multi(1,02f9308a...),or_b(multi(3,022f01e5...,032fa210...,03d01115...),su:after(500000)))", "512102f9308a...", "?", TESTMODE_VALID | TESTMODE_NONMAL | TESTMODE_TAPSCRIPT_INVALID, ...);
```
— `or_d(multi(...), ...)` nested inside `wsh`-style (non-tapscript) context
is `TESTMODE_VALID`, and is explicitly `TESTMODE_TAPSCRIPT_INVALID` (matching
the `IsTapscript` guard above — consistent with W2 correctly using `wsh()`,
not `tr()`, for its `multi()`). `src/test/descriptor_tests.cpp:902-903`
additionally has a `CheckMultipath` test of the identical
`or_d(X, and_v(v:pkh(Y), older(N)))` skeleton (with `pk()` in place of
`multi()`) parsing successfully through the real descriptor parser. Given
these two independent confirmations of every structural piece of W2
(`or_d` + `multi()` nested + `and_v(v:pkh(...),older(...))`, all in `wsh()`),
**W2 is SOURCED-CONFIDENT to be accepted**, though I did not hand-execute
`IsSane()`'s full type-derivation for this exact 4-key instantiation (that
would require either running the real parser or manually deriving
miniscript's type-system judgements — out of scope for a source-reading
recon; flag this residual as **UNVERIFIED-BY-EXECUTION**, mitigated by the
two matching test vectors above).

**`tr()` with a script tree combining `multi_a()` and another miniscript
leaf, on master:** confirmed directly and almost structurally identically —
`src/test/descriptor_tests.cpp:1157-1158`:
```
Check("tr(a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd,{and_v(and_v(v:hash256(...),v:pk(...)),older(42)),multi_a(2,adf586a32ad4b0674a86022b000348b681b4c97a811f67eefe4a6e066e55080c,KztMyyi1pXUtuZfJSB7JzVdmJMAz7wfGVFoSRUR5CVZxXxULXuGR)})", ..., MISSING_PRIVKEYS | XONLY_KEYS | SIGNABLE, ...);
```
— a `tr()` two-leaf script tree with one leaf being a bare `multi_a(2,...)`
and the other an `and_v(...,older(42))` miniscript leaf, marked `SIGNABLE`
(accepted and satisfiable) at `descriptor_tests.cpp:1158`. This is the same
shape as W1 (just `pk`/`hash256` swapped for `pk(D)`, and `older(26280)`
instead of `older(42)`). **W1 is SOURCED-CONFIDENT to be accepted.**

**Version tapscript-miniscript landed:** v26.0, PR #27255 "MiniTapscript:
port Miniscript to Tapscript" (merged 2023-10-08, confirmed via GitHub API:
`"merged": true, "merged_at": "2023-10-08T16:10:32Z"`, base milestone "26.0").
Release notes, `doc/release-notes.md` at tag `v26.0`, line 83:
```
- [Miniscript](https://bitcoin.sipa.be/miniscript/) expressions can now be used in Taproot descriptors for all RPCs working with descriptors. (#27255)
```
(url: https://raw.githubusercontent.com/bitcoin/bitcoin/v26.0/doc/release-notes.md)
Since the task states a local v25.0.0 `bitcoind` exists: **that binary
predates tapscript-miniscript support** (v25 < v26) and would not be a valid
oracle for W1's `tr()` miniscript leaf — this is why W1/W2 validity was
determined from master source/tests rather than by executing against the
local v25 node, per the task's explicit scope ("your job is the SOURCE on
master").

---

## Summary of search coverage (for the negatives above)

| Claim | Files searched (full) | Patterns |
|---|---|---|
| No dedup in legacy multi/sortedmulti/multi_a/sortedmulti_a parse | `src/script/descriptor.cpp` (3092 lines, full read of the block 2380-2469 plus whole-file grep) | `duplicate`, `Duplicate`, `std::set`, `std::unique`, `seen\.` |
| No fingerprint-keyed check in wallet | `src/wallet/scriptpubkeyman.cpp` (1691 lines, full) | `fingerprint` (case-insensitive) |
| No fingerprint warning in import/describe RPCs | `src/wallet/rpc/backup.cpp` (653 lines, full), `src/rpc/output_script.cpp` (full, getdescriptorinfo body read lines 166-215) | `fingerprint`, `warning`/`Warning`(-string literals) |
| No fingerprint-alone lookup in signing | `src/script/sign.cpp` (1091 lines, full), `src/psbt.cpp` (898 lines, `SignPSBTInput` read in full 650-756), `src/psbt.h` (1690 lines, hd_keypaths/KeyOriginInfo serialization sections), `src/script/signingprovider.h` (full) | `fingerprint`, `GetKey(`, `GetKeyOrigin(`, `SignPSBTInput`, `HidingSigningProvider`, `FillPSBT` (latter two: not found as named symbols in these files — signing dispatch goes through `ProduceSignature`/`GetKey` instead, confirmed by reading `SignPSBTInput`'s body) |
| `miniscript.cpp` has no separate duplicate logic | `src/script/miniscript.cpp` (432 lines, full) | `duplicate` (case-insensitive) — zero hits |

A negative here means: within the named file(s), the named grep pattern set
was run against the complete fetched file content (not a truncated excerpt),
and the only hits are quoted or listed above as non-matches (e.g. stray
`std::unique_ptr` tokens).
