# Composer lowering rules — Bitcoin/miniscript expert review

Date: 2026-09-01. Reviewer: dispatched expert agent (read-only). Subject: the FIXED lowering rules for the SeedHammer II wallet-policy composer (spend-path list → BIP-388 template), as tabled in the brief; brainstorm `design/BRAINSTORM_wallet_policy_composer.md` §2 (C16/C17), §3.7, §3.8.

Evidence base: a scratch probe crate against the rust-miniscript fork (`/scratch/code/shibboleth/rust-miniscript-fork`, `miniscript` 13.0.0, HEAD `2092faaf`) running `from_str_insane` / `sanity_check` / `lift` / `script_size` / `satisfy` (real ECDSA/Schnorr signatures, real preimage) on 158 cases; `md` 0.14.0 (`~/.cargo/bin/md`); BIP-341/342/368(68)/379/386/387/388 fetched from `bitcoin/bips` master; Bitcoin Core `src/script/descriptor.cpp`, `src/script/interpreter.cpp`, `src/policy/policy.h` (master); Liana `liana/src/descriptors/{mod,analysis}.rs` (master); Ledger `doc/wallet.md`. Line numbers below refer to those fetched copies. Nothing in the repos was modified.

## 1. Verdict

The rules are type-correct and produce sane (safe, non-malleable, in-limits, no timelock mixing), lift-equivalent-to-the-compiler scripts for every KEYED shape the grammar admits, including the worst case (8 paths × 9-of-9 + hashlock + lock, both contexts), and the conjunct order `and_v(v:KEYS, and_v(v:sha256, LOCK))` is byte-optimal (any order with LOCK last is byte-identical; LOCK anywhere else costs +1). Four rules need to change. (1) **Critical:** the table never says how `@i` placeholders are numbered, and the internal-key extraction moves a later path's key to the front of the text; BIP-388 requires first-appearance numbering and `md encode` silently *renumbers* to it — a path-order numbering would engrave a template whose decoded placeholders are a permutation of what the operator was shown (wrong wallet on restore). (2) **Important:** `or_d(pkh(P1), rest)` is dominated — dissatisfying `pkh` pushes P1's pubkey, so every non-P1 spend costs +34 WU *and* reveals the very key `pkh` was chosen to hide; `or_d` should be reserved for a bare `multi` head and a single-key head should take `or_i`. (3) **Important:** the keyless row is wrong for taproot — `md encode` and rust-miniscript's `Descriptor::from_str` refuse keyless tap leaves outright while admitting them in `wsh`; a lock-only keyless path is admitted by the grammar, encodes, and is *anyone-can-spend after N* (not "bearer access"); and an all-keyless policy is unencodable. (4) **Important:** a raw-hex NUMS internal key is not a valid BIP-388 template (non-KP key), Liana refuses it, and a constant `H` announces "no key path" on every spend contrary to BIP-341's recommendation. Everything else (or_d/or_i typing, cross-path timelock mixing, right-spine depth, multi limits, keyless admission API) checks out as proposed, with measurements below.

## 2. Findings

### C1 — Placeholder numbering is unspecified; internal-key extraction produces out-of-order `@i`, which md silently renumbers  (Critical)

**Rule hit:** tr `internal key` row ("the single unlocked one-key path if exactly one exists (it is then NOT also a leaf)") + "one slot appears in exactly one path". The table never states how `@i` indices are assigned.

**Counterexample (MEASURED):** paths P1 = 2-of-2 (slots 0,1), P2 = 1-of-1 unlocked (slot 2), P3 = slot 3 + `older(100)`. Numbering slots in listed order gives the internal key `@2`, or with P1=2-of-2, P2=locked single, P3=unlocked single: `@3`. Both spellings encode to the SAME string and decode to the renumbered form:

```
$ md encode "tr(@3/<0;1>/*,{multi_a(2,@0/<0;1>/*,@1/<0;1>/*),and_v(v:pk(@2/<0;1>/*),older(100))})"
md1yrpqqxq3zjqgtye54hqqqqqxghgelnnqjntcet
$ md encode "tr(@0/<0;1>/*,{multi_a(2,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pk(@3/<0;1>/*),older(100))})"
md1yrpqqxq3zjqgtye54hqqqqqxghgelnnqjntcet
$ md decode md1yrpqqxq3zjqgtye54hqqqqqxghgelnnqjntcet
tr(@0/<0;1>/*,{multi_a(2,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pk(@3/<0;1>/*),older(100))})
```
(Also `tr(@1/<0;1>/*,{pk(@0/<0;1>/*),pk(@2/<0;1>/*)})` and `tr(@0/<0;1>/*,{pk(@1/<0;1>/*),pk(@2/<0;1>/*)})` both → `md1yzpqqxq3zjj2ssmae672yp47l4`.) The wire carries placeholders only by position of first appearance; a numbering that disagrees with the text order is not representable and is rewritten without any warning.

**SOURCED:** BIP-388 line 199: "The key information vector should be ordered so that placeholder `@i` never appears for the first time before an occurrence of `@j` for some `j < i`; for example, the first placeholder is always `@0`, the next one is `@1`, etc." and line 306 lists `sh(multi(1,@1/**,@0/**))`: "Key placeholders out of order" under "Invalid policies".

**Consequence:** if the composer labels slots in path order and the operator seats keys by those labels, the engraved/decoded template assigns the keys to different placeholders (slot 0's key becomes the internal key above). A restore derives a different wallet: wrong result / unmet guarantee.

**Minimal change:** add a rule row: *"Placeholders are numbered by first appearance in the EMITTED template text (tr: internal key first, then leaves in spine order; wsh: `or_d`/`or_i` head first). Slot labels shown to the operator are these emitted indices, computed after lowering."* Add a vector where the internal key is not path 1. (Optionally assert in tests that `md encode(t)` decodes to byte-identical `t`.)

### I1 — `or_d` with a single-key (`pkh`) head is dominated: +34 WU on every other path AND leaks the head's pubkey  (Important)

**Rule hit:** wsh `paths combine` ("`or_d(P1, rest)` when P1 is a bare key set") together with `key set` ("one key: `pkh`").

**Measured witness (probe `sat`, ECDSA sig 72 B, keys 33 B; R = `and_v(v:pkh(K1),older(100))`):**

| form | script B | P1 spend items → bytes | R spend items → bytes |
| --- | --- | --- | --- |
| `or_d(pkh(K0),R)` (proposed) | 56 | `[72,33<031b84c5..>]` → 107 | `[72,33<024d4b6c..>,0,33<031b84c5..>]` → **142, reveals K0** |
| `or_i(pkh(K0),R)` | 56 | `[72,33<031b84c5..>,1]` → 109 | `[72,33<024d4b6c..>,0]` → 108 |
| `or_d(pk(K0),R)` (compiler, §3.7) | 66 | `[72]` → 73 | `[72,33<024d4b6c..>,0]` → 108 |
| `or_i(pk(K0),R)` | 66 | `[72,1]` → 75 | `[72,33<024d4b6c..>,0]` → 108 |

(probe lines L44–L51; `031b84c5..` is K0's pubkey.) The `or_d` dissatisfaction of `pkh(K0)` is `<empty sig> <pubkey K0>` — `DUP HASH160 <h> EQUALVERIFY` must still pass — so K0 is published whenever any other path spends. The C16 rationale for `pkh` ("hides the pubkeys of UNSPENT single-key branches") is void for a `pkh` at an `or_d` head, at a 34 WU premium. Same at every nesting level (`or_d(multi(1,..),or_d(pkh(K2),multi(2,..)))`, L11: spending the 2-of-2 reveals K2).

**All four are `Bsfm`/`Bdsem`, sanity OK, and lift-equal** (L52–L55, L153: `lift_equal=true`).

**Minimal change:** `or_d` only when the head is a bare MULTI-key set (`multi(k,…)`, n ≥ 2); a bare single-key head takes `or_i(pkh(P1), rest)`. (Alternative if fees on P1 spends matter more than P1-key privacy: `or_d(pk(P1), rest)` as the compiler does — 36 WU cheaper on P1 spends, 10 B larger script, K0 in cleartext in the script.) The keyless head is already `or_i`, correctly: `or_d(sha256(H),R)` is malleable (`Bf`, `nonmall=false`, L8) while `or_i(sha256(H),R)` is `Bdm` (L9).

### I2 — Keyless paths: refused in taproot by md and rust-miniscript, admitted in wsh — the "same" in the tr column is false  (Important)

**Rule hit:** `keyless path` row, tr column "same".

**MEASURED:**
```
$ md encode "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{older(200),pk(@0/<0;1>/*)})"
md: template parse error: miniscript parse failed: All spend paths must require a signature
$ md encode "tr(50929b74…3ac0,{pk(@0/<0;1>/*),{sha256(a84d…08ad),sha256(a84d…08ad)}})"
md: template parse error: miniscript parse failed: All spend paths must require a signature
$ md encode "wsh(or_i(sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad),and_v(v:pkh(@0/<0;1>/*),older(100))))"
md1yqpqqxpxtk5ymnjqjatj0sucqg70h4gdtkemje3rw4fp6rc6cckmmq5mngy26nxjmsqqqqryaduqs28k92vvf
```
Probe: `desc|tr(NUMS,{sha256(HH),pk(K0)})` → `DESC-PARSE-ERR All spend paths must require a signature`; `desc|wsh(or_i(sha256(HH),and_v(v:pkh(K0),older(100))))` → parses, `sanity=ERR All spend paths must require a signature`. **SOURCED:** rust-miniscript `src/descriptor/mod.rs:1138-1150` — `Descriptor::from_str` runs `sanity_check()` and per-leaf `ext_check(&ExtParams::sane())` ONLY for `Descriptor::Tr` ("FIXME preserve weird/broken behavior from 12.x", issue #734); `Wsh::from_tree` (`segwitv0.rs:189-197`) runs no sanity check.

**Consequence:** the EXPERIMENTAL keyless path can be authored in tr and then cannot be encoded; a device rule promising it is a rule the toolchain refuses.

**Minimal change:** keyless paths are **wsh-only** (refuse in tr with the reason), until md/rust-miniscript gain an insane tap-leaf parse (`Miniscript::<_,Tap>::from_str_insane` per leaf + `Tr::new`, which itself checks nothing, `tr/mod.rs:95-98`).

### I3 — Lock-only keyless path (`older(N)` / `after(T)` alone) is admitted by the grammar, missing from the table, encodes, and is anyone-can-spend  (Important)

**Rule hit:** grammar "keyless paths (hashlock and/or lock, no key)" vs `keyless path` row, which lists only `and_v(v:sha256(H), LOCK)` and `sha256(H)`.

**MEASURED:** `ms|wsh|older(100)` → `type=Bzfm sanity=ERR SiglessBranch / All spend paths must require a signature … script_size=3 max_sat_size=Ok(0)`; `desc|wsh(or_i(older(100),and_v(v:pkh(K0),older(200))))` → parses, `sanity=ERR All spend paths must require a signature`; and md accepts it:
```
$ md encode "wsh(or_i(and_v(v:pkh(@0/<0;1>/*),older(100)),older(200)))"
md1yqpqqxpx2v6twqqqqqv3cqqqqryq8985gvkcjypae
```
**Consequence:** after 200 blocks every UTXO of this wallet is spendable by anyone (empty witness for that branch). "Bearer access" (a secret preimage) does not describe it; nothing is borne. Worse than telling the user nothing → refusal class.

**Minimal change:** a path must carry a key set or a hashlock; a lock-only path is REFUSED ("anyone can spend after N"). If the operator insists on admitting it, the warning must say "PUBLIC after N", not "bearer".

### I4 — An all-keyless policy is unencodable  (Important)

**Rule hit:** `keyless path` row + wrapper rows (no "≥ 1 keyed path" precondition).

**MEASURED:**
```
$ md encode "wsh(older(100))"
md: template parse error: template contains no @i placeholders
$ md encode "tr(50929b74…3ac0,{sha256(a84d…08ad),sha256(a84d…08ad)})"
md: template parse error: template contains no @i placeholders
```
**SOURCED:** BIP-388 line 191: "A wallet policy must have at least one key placeholder and the corresponding key."

**Minimal change:** precondition on the list: at least one path has a key set (refuse otherwise, before lowering).

### I5 — Raw-hex NUMS internal key is not a BIP-388 template, Liana refuses it, and a constant `H` leaks "no key path" on every spend  (Important; remedy partly outside this review's scope)

**Rule hit:** tr `internal key` row ("otherwise the BIP-341 NUMS H-point 50929b…").

**SOURCED:**
- BIP-388 line 310 (Invalid policies): `sh(multi(1,@0/**,xpub6AHA9hZ…/<0;1>/*))`: "Expression with a non-KP key present". Lines 139/150-153: the `tr(KEY)`/`tr(KEY,TREE)` template's `KEY` is a `KP` "always followed by `/**` or `/<NUM;NUM>/*`". A raw 32-byte hex is not a KP.
- Liana `liana/src/descriptors/analysis.rs:596-599`: `let desc_int_xpub = get_multi_xkey(desc.internal_key()).ok_or(LianaPolicyError::IncompatibleDesc)?;` — any non-xpub internal key → `IncompatibleDesc`. Liana's own unspendable key (`analysis.rs:404-460`) is an xpub whose pubkey is `H` and whose chaincode is `sha256(concat of the leaf xpubs' pubkeys)`, derived `/<0;1>/*` (comment: "in a way which could eventually be standardized into wallet policies … See https://delvingbitcoin.org/t/unspendable-keys-in-descriptors/304/21"). A live Liana `tr(tpubD6NzVbkrYhZ4X6BRkDMxFyZxfUCQdjpK27dNgqwDqsQ2PUbMmjjPPFxfcTJiGEjeNz2zLbZ1PRmgCAzXn4pE6tEuQPScXyUbuAgdcec6pMN/<0;1>/*,{and_v(v:multi_a(2,…),older(36)),multi_a(2,…)})` imported into Bitcoin Core with `'success': True` (gist jdlcdl/c38e1b80).
- BIP-341 line 157: "In order to avoid leaking the information that key path spending is not possible it is recommended to pick a fresh integer r … and use H + rG as internal key."
- Bitcoin Core accepts a raw x-only internal key (BIP-386/387 examples, e.g. BIP-387 line 71 `tr(50929b74…3ac0,sortedmulti_a(…))`), so Core import is unaffected.

**Consequence:** (d) Ledger-class BIP-388 registration and Liana import refuse the template as emitted; (e) every script-path spend reveals the same well-known `H` in the control block — a wallet-class fingerprint BIP-341 says to avoid. Coordinators that only know the raw form still work (Core).

**Minimal change (documentation, in-scope):** the row states that `H` is the md1-local spelling, valid in Core, NOT a BIP-388 KP, and that a host exporting to a BIP-388 coordinator must substitute a NUMS xpub placeholder. **Better (design-scope, not proposed here as a wire change):** adopt Liana's deterministic NUMS-xpub as `@0` — it is BIP-388-valid, per-address distinct, and interoperable with the one coordinator (Liana) whose taproot import was inspected.

### M1 — "Internal key only if EXACTLY one unlocked single key" leaves ~101 WU and key-path privacy on the table when two exist  (Minor)

**MEASURED:** key-path spend `tr(K0)` `max_weight_to_satisfy=66`; a `pk` leaf at depth 1 costs `167` (`tr(NUMS,{pk(K0),pk(K0)})`, `tr(K0,{pk(K0),pk(K1)})`): +101 WU (~25 vB) and a visible script path. The compiler extracts the FIRST single key even when two exist: `comp|tap|or(pk(K0),or(pk(K1),and(pk(K2),older(65535))))` → `tr(031b84c5…,{pk(024d4b6c…),and_v(v:pk(02531fe6…),older(65535))})` (L158). **SOURCED:** BIP-341 line 157: "If one or more of the spending conditions consist of just a single key …, the most likely one should be made the internal key."

**Change:** "the FIRST-LISTED unlocked, unhashed single-key path (listed order is the frequency order the spine already assumes); otherwise NUMS". Still fixed and search-free.

### M2 — `older` range must be stated in the rule; rust-miniscript's `sanity_check` accepts values consensus masks to zero  (Minor — md's codec catches it today)

**MEASURED:** `ms|wsh|and_v(v:pkh(K0),older(65536))` → `type=Bnsfm sanity=OK` (L86); `older(4259840)` → `sanity=OK` (L90); `older(0)` → `PARSE-ERR(insane) relative locktimes in Miniscript have a minimum value of 1`; `older(2147483648)` → `locktime value 2147483648 is not a valid BIP68 relative locktime`. md refuses the masked cases with a precise message:
```
$ md encode "wsh(and_v(v:pkh(@0/<0;1>/*),older(65536)))"
md: codec error: older(65536) is not what consensus enforces: BIP-68 reads only the low 16 bits (and bit 22 for units), so this locks for 0 blocks, not 65536. A relative timelock cannot exceed 65535 blocks -- use an absolute after() height for longer delays
```
**SOURCED:** BIP-68 line 40 ("a mask of 0x0000ffff MUST be applied"), line 238 (bits 16–21 unused); Core `interpreter.cpp` `CheckSequence`: `nLockTimeMask = SEQUENCE_LOCKTIME_TYPE_FLAG | SEQUENCE_LOCKTIME_MASK; … nSequenceMasked = nSequence & nLockTimeMask`.

**Change:** the LOCK cell states the admitted ranges — `older`: 1..=65535 blocks, or 1..=65535 units encoded as 4194304+units; `after`: 1..=0x7fffffff (rust-miniscript: `absolute locktimes in Miniscript have a minimum value of 1` / `maximum value of 0x7fffffff`). The device emitter sits upstream of md and must not rely on md's codec error.

### M3 — `or_d` over `multi(k)` charges k+1 empty pushes on every non-head spend  (Minor, document)

**MEASURED** (R spends, L57–L62): `or_d(multi(2,…),R)` 110 vs `or_i` 108; `or_d(multi(9,…),R)` 116 vs `or_i` 107; head spends: `or_d` 147 vs `or_i` 149. Net: `or_d` saves 2 WU on P1 spends and costs k+1−1 WU on every other path. Correct as a default (the compiler and BIP-388's own example `wsh(or_d(pk(@0/**),and_v(v:multi(2,…),older(65535))))`, line 249, use it); state the trade-off so nobody "optimises" it later.

### N1 — Conjunct order/nesting is byte-identical; only the text differs  (Nit, confirms the rule)

**MEASURED** (wsh script/witness bytes, 2-of-3 + hash + older): keys-first 148/180; hash-first 148/180; left-nested `and_v(v:and_v(v:multi,sha256),older)` 148/180; lock-first 149/180; lock-middle 149/180. Tap: keys-first 147/164, hash-first 147/164, lock-first 148/164. `lift_equal=true` (L155). The rule's fixed textual form is what pins template IDs; keep it.

### N2 — Taptree text order at a node is irrelevant to the address; only depth is  (Nit, confirms)

**MEASURED:** `tr(NUMS,{pk(K0),{pk(K1),pk(K2)}})`, `tr(NUMS,{{pk(K1),pk(K2)},pk(K0)})`, `tr(NUMS,{pk(K0),{pk(K2),pk(K1)}})` → identical `bc1pfpxjcfa6s7mwckln9fu4w74f666fyehlqz06xvtkex5unknrgsrqctlfup`; `tr(NUMS,{{pk(K0),pk(K1)},pk(K2)})` → different. **SOURCED:** BIP-341 line 74 (children sorted lexicographically before hashing). The right spine is a canonical TEXT form (for md1/IDs), not a consensus choice.

### N3 — `wsh(pkh(@0))` for a lone unlocked single-key path  (Nit, grammar-adjacent)

`desc|wsh(pkh(K0))` sanity OK, `max_weight_to_satisfy=133`; `md encode "wsh(pkh(@0/<0;1>/*))"` → `md1yqpqqxpzcyzv0v4n9qw89t`. Valid, but `wpkh(@0/**)` is the BIP-388 default template (line 237) and is what every coordinator expects for single-sig. Wrapper choice is settled; noting only.

## 3. Corrected rules table

| rule | wsh | tr |
| --- | --- | --- |
| paths combine | listed order, recursive, last path stands alone: `or_d(P1, rest)` **iff P1 is an unlocked, unhashed MULTI-key set (`multi`, n ≥ 2)**; otherwise `or_i(P1, rest)` (single-key, locked, hashed or keyless head). Never `andor`, never `thresh` over paths | one leaf per path, right spine in listed order `{P1,{P2,{P3,P4}}}`; path 1 shallowest (P_k at depth min(k, n−1)) |
| inside a path | `and_v(v:KEYS, and_v(v:sha256(H), LOCK))`, dropping absent parts; keys, hash, lock last | same |
| key set | unlocked single-path: `sortedmulti`; locked/hashed multi-key: `multi`; one key: `pkh` | unlocked whole leaf: `sortedmulti_a`; locked/hashed: `multi_a`; one key: `pk` |
| LOCK values | `older`: 1..=65535 blocks, or 4194304+u for u in 1..=65535 (512-s units); `after`: 1..=2147483647 | same |
| internal key | n/a | the **first-listed** unlocked, unhashed one-key path if any (not also a leaf); otherwise NUMS `50929b…3ac0` — **md1-local spelling; not a BIP-388 KP; hosts exporting to BIP-388 coordinators substitute a NUMS xpub placeholder** (Liana convention) |
| **placeholder numbering** | **`@i` by first appearance in the EMITTED text; slot labels shown to the operator are the emitted indices** | **same; internal key is `@0` when extracted** |
| keyless path (EXPERIMENTAL) | `and_v(v:sha256(H), LOCK)` or `sha256(H)`; **lock-only paths REFUSED (anyone-can-spend)**; **policy must contain ≥ 1 keyed path** | **REFUSED in tr** (md/rust-miniscript reject keyless tap leaves) until an insane tap-leaf parse exists |

## 4. Points 1–10

1. **or_d left-arm typing — MEASURED.** `multi` is `Bndusem`, `pkh` = `c:pk_h` is `Bndusem`, `pk` is `Bondusem` (L16–L18): all satisfy or_d's `Bdu` (correctness.rs:386-393 `LeftNotDissatisfiable`/`LeftNotUnit`). `and_v(...)` is never `d` (correctness.rs:354 `dissatisfiable: false`); `or_d(and_v(v:pkh(K0),older(100)),multi(…))` → `typecheck: fragment «…» requires its left child be dissatisfiable` (L5), so `or_i` is forced for a locked head. Rule picks `or_i` where `or_d` is valid but WORSE: keyless `sha256(H)` head (`or_d` malleable, L8) — correct. Rule picks `or_d` where `or_i` is better: single-key `pkh` head — finding I1.
2. **Conjunct order — MEASURED.** Type-valid in both contexts: `and_v(v:multi,and_v(v:sha256,older))` `Bnsfm` (wsh), `Bsfm` (tap, L31); with `after` and time-`older` likewise (L35–L37). Top level `B`, `s`, `f`, `m` — `e` absent (and_v is never `d`), which top level does not need. No malleability difference vs the compiler's hash-first (`m` in all orders; `lift_equal=true`). Fee: byte-identical as long as LOCK is last (N1); LOCK first costs +1 (`v:older` needs an explicit VERIFY). Witness bytes identical (180 wsh / 164 tap) — only element order differs.
3. **Top-level safety — MEASURED.** Worst cases all `sanity=OK safe=true nonmall=true limits_ok=true mixed_tl=false`: 8 × 9-of-9 + hash + alternating older/after (height & time): `script_size=2843 max_sat_size=Ok(699) max_wit_elems=Ok(19)` (< 3600 / < 100, Core policy.h:54,60); with `or_d(multi(9…))` head: 2801 / 708 / 28; 8 × pkh + hash + lock: 571 / 148 / 11; 8 × 9-of-9 + lock (no hash): 2531 / 666 / 18. Tap 8 × 9-of-9 + hash + lock: leaves 350–353 B, depths 1..7, `cb_len` ≤ 257, `max_weight_to_satisfy=1243`. Ops: 8 × (CMS 1 + 9 keys + 4 hash ops + 1–2 lock) + 21 IF/ELSE/ENDIF ≈ 141 < 201 (rust-miniscript's `within_resource_limits` agrees).
4. **Timelock mixing — MEASURED + SOURCED.** Cross-path `older(h)` + `older(t)` + `after(h)` + `after(t)` in one `or_d`/`or_i` chain: `mixed_tl=false sanity=OK` (L74–L77); `combine_or` never sets `contains_combination` (extra_props.rs:50-54). Within one path it IS flagged: `and_v(v:pkh,and_v(v:older(100),older(4194404)))` → `HeightTimelockCombination / Contains a combination of heightlock and timelock` (L78, L79 for `after`, L81 tap) — excluded by the one-lock grammar. `older`+`after` in one path is NOT flagged by rust-miniscript (L80) — also excluded by grammar; nothing cross-path is flagged.
5. **Right spine — MEASURED + SOURCED.** Control block 33+32·depth: spine depths for 8 leaves 1,2,3,4,5,6,7,7 (L69) → P1 65 B vs balanced 129 B (−64 WU), P7/P8 257 B (+128 WU each). "Path 1 shallowest" is sound iff listed order is expected-frequency order (BIP-341 line 68 footnote: Huffman by probability). Depth limit 128 (BIP-341 line 68; rust-miniscript `TAPROOT_CONTROL_MAX_NODE_COUNT`, taptree.rs:44) vs max 7 here. Leaf ORDER at a node does not change the root (BIP-341 line 74; N2 measured). Repeated identical leaves: rust-miniscript accepts (`tr(NUMS,{pk(K0),pk(K0)})` `sanity=OK`, L105), BIP-341 does not forbid; the grammar's fresh-slot rule makes duplicates possible only for two identical keyless paths, which tr refuses anyway (I2).
6. **Internal-key extraction — MEASURED/SOURCED.** `tr(@0/<0;1>/*,{…})` is a valid BIP-388 template (BIP-388 lines 139, 291 `tr(@0/**,{sortedmulti_a(…),…})`); `md encode` accepts (`md1yrpqqxq3zjqgtye54hqqqqqxghgelnnqjntcet`). Core: `tr(KEY,TREE)` fully supported (descriptor.cpp:1560). Liana emits exactly this shape itself (`mod.rs:870` test: `tr([abcdef01]xpub6Eze…/<0;1>/*,and_v(v:pk([abcdef01]xpub688H…/<0;1>/*),older(52560)))`) and imports it (analysis.rs:600-606). Ledger: `tr(KP, TREE)` (wallet.md:30). Nunchuk: UNVERIFIED. Hazard: only the numbering trap (C1) and "exactly one" (M1); losing the unspendable-key-path property is the point — key-path spend is 66 WU vs 167 WU for a depth-1 leaf and looks like single-sig. Recommendation: keep extraction (first-listed), do NOT default to NUMS-always; fix NUMS spelling per I5.
7. **multi/multi_a limits — MEASURED + SOURCED.** `multi(20,…)` OK (685 B); `multi(1, 21 keys)` → `invalid threshold 1-of-21; maximum size is 20`; `multi_a(9)` tap OK 308 B; `multi_a` in wsh → `Multi a(CHECKSIGADD) only allowed post tapscript`; `multi` in tap → `Invalid use of Multi node in taproot context`. Tapscript has no CHECKSIGADD count limit; the per-input sigops budget is 50 + witness size, −50 per executed sig op with a non-empty signature (BIP-342 line 130) — always met since each Schnorr sig item weighs 65–66 WU. wsh `multi(k,n≤16)` counts n sigops per CHECKMULTISIG (whole witnessScript), 8×9 = 72 sigops = 288 sigop-cost per input vs `MAX_STANDARD_TX_SIGOPS_COST` 16000 (policy.h:44) — not a constraint. Legacy `sh(sortedmulti)`: 16 keys → `The Miniscript corresponding Script cannot be larger than 520 bytes, but got 547 bytes.` (L137) and Core `MAX_P2SH_SIGOPS{15}` (policy.h:42); n ≤ 9 fine (`sh(sortedmulti(9,…))` sanity OK, L135).
8. **Keyless admission — MEASURED + SOURCED.** Variant: `AnalysisError::SiglessBranch`, Display `All spend paths must require a signature` (analyzable.rs:225-227). API: `Descriptor::from_str` admits an insane `wsh` (sanity is a separate `sanity_check()` call) but refuses insane `tr` leaves (mod.rs:1138-1150); `Miniscript::from_str_insane` = `from_str_ext(&ExtParams::insane())` also waives malleability/limits/timelock/repeated-pk — too wide. Right tool: `Miniscript::from_str_ext(s, &ExtParams::new().top_unsafe())` (analyzable.rs:83-86) per context, which admits sigless scripts while still enforcing `m`, limits, no mixing, no repeated keys; for tr build leaves this way and assemble with `TapTree`/`Tr::new`. Script validity: `sha256(H)` alone = `SIZE <32> EQUALVERIFY SHA256 <H> EQUAL` 39 B, witness one 32-byte item (≤ 80 B, policy.h:56/58) — consensus-valid and standard in both contexts; `and_v(v:sha256(H),older(N))` 42 B; `older(N)` alone 3 B, empty witness (I3). Coordinators refuse: Core `is not sane: witnesses without signature exist` (descriptor.cpp:2691-2703); Liana `IncompatibleDesc`.
9. **Interop.** (a) `wsh(or_d(multi(2,A,B,C),and_v(v:pkh(D),older(N))))`: Core — SOURCED accepts (IsSane; probe `sanity=OK`), not executed; Liana — SOURCED accepts by shape (lift-based importer, analysis.rs:578-640; `pkh` lifts to `pk`), not executed; Sparrow — SOURCED refuses (no miniscript: issue #1700 open, 2.5.0 release notes silent); Nunchuk — SOURCED for wsh only: "For advanced users, you can paste a raw Miniscript policy or import one from a file. Nunchuk automatically analyzes the script, checks for correctness, and shows you the spending conditions in plain language." (nunchuk.io/blog/miniscript101), not executed; whether a custom `wsh` template in this exact spelling passes its "checks for correctness" is UNVERIFIED. (b) `tr(NUMS-hex,{multi_a(2,A,B,C),and_v(v:pk(D),older(N))})`: Core — SOURCED accepts raw x-only internal key (BIP-387 line 71 example); Liana — SOURCED REFUSES (`IncompatibleDesc`, analysis.rs:598-599: internal key must be a MultiXPub); Ledger/BIP-388 — SOURCED invalid (non-KP key, BIP-388:310); Sparrow — refuses; Nunchuk — UNVERIFIED. (c) keyed internal key: Core yes; Liana yes (its own form, mod.rs:870); Ledger — BIP-388-valid `tr(@0/**,{…})` (BIP-388:291), device behaviour UNVERIFIED; Nunchuk — UNVERIFIED. All non-Core claims are from source/docs, none executed.
10. **Anything else — MEASURED.** (i) `or_d` head dissat cost grows with k (M3). (ii) `older` masking (M2). (iii) The compiler cross-check confirms lift-equivalence of every proposed form tested (`lift_equal=true` L150, L153, L155; compiled `tr(NUMS,{multi_a(…),and_v(v:pk(…),older(100))})` L151 is byte-identical in shape to the rule's output). (iv) `pkh` in tap parses (`Bndusem`, 25 B, L147) but costs 99 vs 66 WU to satisfy — the rule's `pk` in tr is right. (v) No fee/privacy alternative found for the `and_v` row or the right spine beyond N1/N2.

## 5. What I ran

- Probe crate `/tmp/claude-1000/-scratch-code-shibboleth-mnemonic-engrave/702b37c9-e041-404f-8220-2456ff9c6bf3/scratchpad/probe/` (`Cargo.toml`, `src/main.rs`, `cases.txt` 158 cases, `cases2.txt` 4 cases; output `out.txt`), path-dep on the fork with `features=["compiler","std"]`: `Miniscript::<PublicKey,{Segwitv0,Tap}>::from_str_insane`, `.ty`, `sanity_check`, `requires_sig`, `is_non_malleable`, `within_resource_limits`, `has_mixed_timelocks`, `script_size`, `max_satisfaction_size`, `max_satisfaction_witness_elements`, `lift().normalized().sorted()`, `encode().to_asm_string()`, `satisfy(&Sat)` with a `Satisfier` producing real ECDSA (`sign_ecdsa`) / Schnorr (`sign_schnorr_no_aux_rand`) signatures and the real preimage `[7u8;32]`; `Descriptor::<DefiniteDescriptorKey>::from_str`, `sanity_check`, `max_weight_to_satisfy`, `address`, `Tr::leaves()` depths; `Concrete::compile::<Segwitv0>()` and `compile_tr(Some(02‖H))`. Keys: secp256k1 secret keys `[i+1;32]`, i = 0..80.
- `md 0.14.0`: `encode`/`decode` on 22 templates (placeholder order, pkh, keyless wsh/tr, lock-only, duplicate hash leaves, `older` 0/65535/65536/4259840/2147483647, `tr(@0)`, `wsh(multi|sortedmulti)`).
- Fetched: BIP-0068, -0341, -0342, -0379, -0383, -0386, -0387, -0388; Core `src/script/descriptor.cpp` (3092 lines), `src/script/interpreter.cpp` (`CheckSequence`), `src/policy/policy.h`; Liana `liana/src/descriptors/mod.rs` (2441 lines), `analysis.rs` (901 lines), `keys.rs`; Ledger `app-bitcoin-new/doc/wallet.md` (106 lines); gist jdlcdl/c38e1b80 (Liana tr import into Core); Sparrow issue #1700 and release 2.5.0 notes; Nunchuk "Miniscript 101" blog.
- rust-miniscript fork source read: `src/miniscript/analyzable.rs`, `types/correctness.rs` (or_d/or_i/and_v), `types/malleability.rs`, `types/extra_props.rs` (`combine_or`), `descriptor/mod.rs:1136-1150`, `descriptor/segwitv0.rs:185-215`, `descriptor/tr/mod.rs:95-148,340-400`, `descriptor/tr/taptree.rs`, `primitives/relative_locktime.rs:42`, `miniscript/limits.rs`.
- Not run: any coordinator binary (Core, Liana, Sparrow, Nunchuk, Ledger app) — all coordinator claims are source/doc-based and marked so.
