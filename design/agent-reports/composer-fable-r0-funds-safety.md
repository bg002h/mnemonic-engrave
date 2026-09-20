# Composer fable review r0 — Lens 1: FUNDS SAFETY and CORRECTNESS of policies constructed on the SH2

**Verdict: 1C / 3I / 2M / 1N.** Across 70 shapes driven through the production composer functions at fork tip `f5b068faf3049ccf97603dfbaa3709f12893df97`, every ADMITTED policy's engraved md1 decodes on the host (`md 0.16.2`) to a script whose addresses equal the device's consent screen (68/68, 272 addresses) and whose spend conditions Bitcoin Core enforces exactly as the screens state (84 distinct funded branch tests on Core v31.1, 39 on Core v25.0, 0 failures once my own MTP anchor was fixed) — EXCEPT that (C-1) two key-less paths compose on the device into a MALLEABLE script the Rust primary refuses and no Core imports; (I-1) one key-less path makes the whole wallet un-importable in Core with nothing on screen saying so; (I-2) the same KEY at two slots is NOT refused when its second copy is a re-serialised xpub (the form `md descriptor` itself prints), and one signer then spends the "2-of-3"; (I-3) the consent screen never names the threshold k of a locked or hashed multi-key path (two of the six presets), and the self-check compares only n there.

Reviewer: fable-5.1, lens 1 of 4. Read-only on every repo; nothing committed. Worktree `/scratch/code/shibboleth/.tmp/fable-funds` (removed at the end). Evidence files, drivers and the three harness test files are preserved OUTSIDE the repos under `/scratch/code/shibboleth/.tmp/fable-funds-work/` (`cases.json`, `host.json`, `core_addrs_v31.json`, `core_addrs_v25.json`, `spend_v31*.log`, `spend_v25.log`, `cases.json.dupkey.json`, `matrix.md`, `harness/*.go`, `hostleg.py`, `coreleg.py`).

Method actually executed: (1) a Go test in package `gui` seats the three demo seeds (`abandon…about`, `zoo…wrong`, `beef…`) plus `legal winner…yellow` through `composerSeedDerive` (the production §4f account-ordinal rule), emits template+keyed chunks through `composerArtifactsFor`, runs `composerSelfCheck`, `composerListedPaths` + `composerConsentLinesFor` (the consent screen) and `composerMappingLines` (the mapping review) for each shape and dumps JSON; (2) `md decode` / `md descriptor` / `md address` on the keyed chunks; (3) Bitcoin Core regtest on `-rpcport=18543 -port=18544` — v31.1 (nix) for everything, then v25.0 (`/usr/local/bin`) on the same ports — `getdescriptorinfo` + `deriveaddresses` on the chain-collapsed descriptors and `importdescriptors` of the multipath form, then funding and PSBT spends per branch with per-branch private-key wallets (`createpsbt` with explicit `sequence`/`locktime`, `PSBT_IN_SHA256` preimages injected by hand, `walletprocesspsbt`, `finalizepsbt`, `testmempoolaccept`). Regtest refuses mainnet xpubs, so keys were re-versioned xpub→tpub / xprv→tprv by origin match and addresses compared by witness program / hash160 (network-independent).

---

## Findings

### C-1 (Critical) — Two key-less paths compose into a malleable script; the Rust primary refuses it, Core cannot import it, the device cuts it

**Input** (wsh, admitted by every gate on the way to steel): path 1 `1 key`; path 2 `keyless, sha256(cc6a74520f526a6135a4eae180547ae73648254ad1ae90bad93520402b0a123d)`; path 3 `keyless, sha256(5a23c169bc7e6f2569a3d74f58118576f005118603473a7069e22ab544484a3d) + older(5)`. (The keyless-first ordering `[keyless, keyless+older(5), 1 key]` behaves identically — matrix rows 62/63.)

**Observed on the device path** (production functions, harness `TestFableFundsHarness`, case `wsh-two-keyless`): `md.ValidatePathList` → nil; `composerArtifactsFor` → 4 keyed chunks; `composerSelfCheck` → nil; consent screen:

```
Script: Segwit (wsh)
Path 1: 1 key
Path 2: KEY-LESS (EXPERIMENTAL)
  hash sha256 cc6a7452..2b0a123d
Path 3: KEY-LESS (EXPERIMENTAL)
  5 blocks (about 0.0 days)
  hash sha256 5a23c169..44484a3d
Policy-ID: 96a316e021299589a66d74a963361cd9
mk1 stub (policy): 96a316e0
… Receive 0: bc1q… (four addresses shown)
```

**Observed on the host**: `md decode` → `wsh(or_i(pkh(@0/<0;1>/*),or_i(sha256(cc6a…123d),and_v(v:sha256(5a23…4a3d),older(5)))))`; `md descriptor` and `md address` render it without complaint.

**The Rust primary refuses the same list**:
```
$ md compose --wrapper wsh --experimental --path 1of1 --path keyless,sha256=cc6a…123d --path keyless,sha256=5a23…4a3d,older=5
warning: EXPERIMENTAL: path 2 has no key (bearer access to whoever holds the preimage)
warning: EXPERIMENTAL: path 3 has no key (bearer access to whoever holds the preimage)
md: composed a template that `md encode` refuses, so nothing was emitted:
  template parse error: miniscript parse failed even with --experimental: Miniscript is malleable (--experimental relaxes ONLY the signature rule; malleability, resource limits, repeated keys and timelock mixing still apply)
```
`md encode --experimental '<the template above>'` → rc=1, same message.

**Bitcoin Core refuses it** (both versions, `getdescriptorinfo` and `importdescriptors`, keys re-versioned to regtest): v31.1 and v25.0: `or_i(sha256(cc6a…),and_v(v:sha256(5a23…),older(5))) is not sane: malleable witnesses exist`.

**Why (structural, not a corner)**: §5 chains paths as `or_i(P, R)` unless the head is a bare multi; `or_i(X,Z)` is non-malleable only if `X.s || Z.s` (rust-miniscript `malleability.rs`, Core `miniscript.h`), and a key-less path has no `s`. With two key-less paths anywhere in the list, the innermost `or_i` that contains both has two sigless sides, so EVERY list with ≥2 key-less paths is malleable under this lowering. The primary catches it only because `md compose` re-parses its own output through `md encode` after lowering; the Go port deliberately "emits no text" (md/compose.go:1-22) and has no equivalent post-lowering check.

**Where**: `md/compose.go:449-476` (`ValidatePathList` — no rule bounds key-less paths); `gui/composer_shape.go:327` (`composerAddPath` fires §8a per key-less path, no cap) and `gui/composer_shape.go:539` (the Done arm's only gate is `md.ValidatePathList`); `gui/composer_flow.go:280` (`composerArtifactsFor` emits without a sanity parse). Spec: §4b/§4e list no refusal for a second key-less path; §5b promises "for every composable list the emitted template … passes `sanity_check`" — unmet.

**Consequence**: a plate whose wallet the primary would never mint and no Core imports (keyed path included). The consensus script IS what the screens say — I spent a single-key-less sibling shape with a hand-built witness (I-1 below) — so funds are recoverable by hand-crafting witnesses, not by any wallet; and the operator passed two §8a confirms that say "bearer access", never "no wallet will import this". Rated Critical as an unmet guarantee (§5b) and a normative divergence from the Rust primary (Rust-primary rule); the controller may downgrade to Important on the ground that the script matches the screens.

**Remedy, one sentence**: refuse a second key-less path in `ValidatePathList` (Rust and Go, with a vector), which is exactly the set the primary's post-lowering parse rejects.

**RED test** (`harness/zz_fable_funds_red_test.go`, `TestFableRedTwoKeylessPathsAreRefused`):
```
zz_fable_funds_red_test.go:25: ValidatePathList admitted [1 key, keyless, keyless]; md compose --experimental refuses it as malleable
--- FAIL: TestFableRedTwoKeylessPathsAreRefused (0.00s)
```

### I-1 (Important) — One key-less path (the designed C16 EXPERIMENTAL shape) makes the WHOLE wallet un-importable in Bitcoin Core, keyed path included; no screen says so

**Input**: wsh; path 1 `1 key`; path 2 `keyless, sha256(cc6a…123d) + older(5)` (also `keyless, sha256(H)` alone; also `[keyless+older(5), 2-of-2]`).

**Observed**: device composes, self-checks and shows consent with addresses (matrix rows 57-59). Host `md descriptor` renders `wsh(or_i(pkh([73c5da0a/48'/0'/0'/2']xpub…/<0;1>/*),and_v(v:sha256(cc6a…),older(5))))`. Core v31.1 AND v25.0, `getdescriptorinfo` with tpub, and again with the tprv of @0 present: `… is not sane: witnesses without signature exist`. `importdescriptors` cannot even be reached (no checksum is issued). The Rust primary ADMITS this shape (`md compose --experimental` rc=0; `md encode --experimental` rc=0 with the bearer-access warning), so this is an import consequence the spec left unverified (§13 item 4: "Import tests … are import tests, not emit tests") — and the operator-facing copy (§8a) says only "This path needs no signature. Whoever knows the preimage of its hash can spend it", while §8f, for the comparable NUMS case, does name which wallets import the form.

**Consensus check (so the plate is not a lie)**: funded the device's receive-0 address on regtest and spent it with a HAND-BUILT witness `[preimage, <empty>, witnessScript]` using the witness script the device's own emitter produces (`md.EmitWitnessScriptChunks`, sha256 == the address's program):
```
[PASS] wsh-keyless-hash-older  P2 preimage only (hand witness), no key, seq=5 tip=h+0            -> allowed=False non-BIP68-final
[PASS] wsh-keyless-hash-older  P2 WRONG preimage (hand witness)                                   -> allowed=False
[PASS] wsh-keyless-hash-older  P2 preimage only (hand witness), seq=4 too small, tip=h+4          -> allowed=False mempool-script-verify-flag-failed (Locktime requirement not satisfied)
[PASS] wsh-keyless-hash-older  P2 preimage only (hand witness), seq=5, tip=h+4                    -> allowed=True
   sent + confirmed: 202512aa43c7075447eee04e1d220566e010692a0ebb144c1060aa259ea33fe2 conf= 1
[PASS] wsh-keyless-hash-only   P2 preimage only (hand witness), no key, seq=4294967293 tip=h+0    -> allowed=True
[PASS] wsh-keyless-hash-only   P2 WRONG preimage (hand witness)                                   -> allowed=False (…false/empty top stack element)
   sent + confirmed: 6b731ed084151a960c6f767ef1983a04cc1bb92d9ac4b54aaae4c89c6ab21f95 conf= 1
```
So the script does what §8a says (preimage alone spends; wrong preimage cannot; the lock holds) — but the KEYED path's owner cannot spend through Core at all.

**Where**: copy `gui/composer_copy.go` §8a body (`composerCopyKeylessPath`) and the consent (`gui/composer_consent.go:91-96` prints `KEY-LESS (EXPERIMENTAL)` and nothing about import). Expected: the §8a/consent copy names the consequence the way §8f does ("Bitcoin Core will not import a wallet with this path; spending needs a tool that builds the witness"), or the spec's C16 admission is revisited. Not a wrong script; a missing case on the promise side.

### I-2 (Important) — The same KEY at two slots is not refused when the second copy is a re-serialised xpub; one signer then spends the "2-of-3"

**Input**: wsh, one path `2-of-3` (sortedmulti). Slot @0 seated from seed `abandon…about` (account 0', xpub `xpub6DkFAXWQ2dHxq2vatrt9qyA3bXYU4ToWQwCHbf5XB2mSTexcHZCeKS1VZYcPoBd5X8yVcbXFHJR9R8UCVpt82VX1VhR28mCyxUFL4r6KFrf`); slot @1 seated from a `key:` record `[73c5da0a/48'/0'/1'/2']xpub6DXuQW1Q2JpZxsEnFKrPvDuiRMmQgU4fzHU1wsvM5EqgGAWRJ3cmwbtS8u1HQjrEHg3YFb7XGnFovPydJ8qpaGNNd2hSEPoheWd27EABdGH` — the SAME chain code and point as @0, re-serialised with parent fingerprint `00000000` (depth 4, child 2'); this is byte-for-byte the xpub string `md descriptor` prints for that slot (F-611: "EMITTED XPUBS CARRY A ZERO PARENT FINGERPRINT"), so an operator who copies an xpub off `md descriptor` output into a `key:` record, or onto an mk1 card, produces exactly this input. `sysw.ParseKeyRecord` admits it (depth 4, last component 2'). Slot @2 from seed `zoo…wrong`.

**Observed** (harness `TestFableSameKeyTwoSlotsReserialized`): `composerInvariantViolation` → false (different declared accounts); `composerDuplicateXpub` → false (strings differ); `composerArtifactsFor` → keyed md1 (6 chunks, in `cases.json.dupkey.json`); `composerSelfCheck` → nil; consent:
```
Script: Segwit (wsh)
Path 1: 2-of-3
Policy-ID: 1d664b0ab57dd9b8e0edabbb4445fe1d
Receive 0: bc1qgjtj7jvx7va7chdpryr50kdp3vhxn5dntq9eld5dgx56pwlm6qcsf65972 …
```
Mapping review: `@0: 73c5da0a m/48'/0'/0'/2'`, `@1: 73c5da0a m/48'/0'/1'/2'`, `@2: 3f635a63 …` plus the §8g body "SAME SEED, SAME PATH — Slots @0 and @1 are the same seed. This path's 2-of-3 can be satisfied by one person." — a warning about the PERMITTED same-seed-two-accounts case, on a wallet that is actually the UNSUPPORTED same-key case.

Host: `md descriptor` prints `wsh(sortedmulti(2,[73c5da0a/48'/0'/0'/2']xpub6DXuQW…BdGH/<0;1>/*,[73c5da0a/48'/0'/1'/2']xpub6DXuQW…BdGH/<0;1>/*,[3f635a63/…]xpub…/<0;1>/*))` (the same xpub twice), `md address` derives; but the Rust primary's `md decompose` of that descriptor REFUSES: `md: decompose: the same extended key is used at 2 positions — xpub6DXuQW1Q…27EABdGH … BIP 388 requires that "the public keys … must be pairwise distinct"`.

Core v31.1: `getdescriptorinfo` OK; `deriveaddresses` == device (payload-equal); with ONE tprv (the key at @0 and @1) imported: `walletprocesspsbt` + `finalizepsbt` complete, `testmempoolaccept allowed=True`; the witness is `[<>, sig, sig, script]` with sizes `[0, 71, 71, 105]` and `sig1 == sig2: True` — the 2-of-3 is spent by one key signing twice.

**Expected**: refusal at the mapping review (§7d "Two slots resolving to the same xpub → REFUSE", operator ruling 2026-09-19 "Same key at two slots is refused as UNSUPPORTED"). **Where**: `gui/composer_review.go:89-98` compares `a.xpub` STRINGS (`seen[a.xpub]`), while the artifact binds the 65-byte chain-code‖point (`decodeXpubBytes`, `gui/composer_flow.go:290-300`); the same key in two serialisations is one key on the wire and two strings on the screen. Consequence: a refusal that does not refuse; the plate carries a BIP-388-forbidden reuse under the §8g label. (The threshold consequence — one person spends — IS what §8g says, so this is not "fewer parties than shown"; it is the UNSUPPORTED shape reaching steel labelled as the supported one.) Remedy, one sentence: compare the decoded 65 bytes, not the string. Out of lens but adjacent: the host's `md descriptor`/`md address` render what `md decompose` refuses (0.44.x moved the validators off the decode path).

**RED test** (`TestFableRedSameKeyReserializedIsRefused`):
```
zz_fable_funds_red_test.go:54: composerDuplicateXpub did not refuse the same key at @0 and @1 (xpub strings "xpub6DkFAXW…KFrf" vs "xpub6DXuQW1…BdGH")
--- FAIL: TestFableRedSameKeyReserializedIsRefused (0.01s)
```

### I-3 (Important) — The consent screen never names the threshold of a locked or hashed multi-key path; the self-check compares only the key count there

**Input**: the `tiered-recovery` preset unedited (`[2-of-2], [1-of-2 + older]`, here older(5)); also `decaying-multisig` (`[2-of-2 + older], …`); also `wsh-32-slots` (`9-of-9 + older(5)` and `1-of-9 + older(6)`), `wsh-multi-hash-older-all` (`2-of-3 + sha256 + older(5)`), under both wrappers.

**Observed consent** (production `composerConsentLinesFor`):
```
wsh-tiered-recovery:   Path 1: 2-of-2 / Path 2: 2 key(s), custom / 5 blocks (about 0.0 days)
wsh-decaying:          Path 1: 2 key(s), custom / 3 blocks … / Path 2: 1 key / 6 blocks … / Path 3: 1 key / Block 150 …
wsh-32-slots:          Path 2: 9 key(s), custom / 5 blocks …   Path 3: 9 key(s), custom / 6 blocks …   (9-of-9 and 1-of-9 print identically)
wsh-multi-hash-older-all: Path 1: 3 key(s), custom / 5 blocks … / hash sha256 cc6a7452..2b0a123d
```
The path-list row does say `Path 2: 1-of-2 + 5 blocks` (`composerPathLine`), but the consent — "the screens are the promise", and the surface §7e says "MUST name, per path in listed order: its k-of-n or single key" — prints `N key(s), custom` for every multi-key path that carries a lock or a hash, because `md.PolicyShape.Branch.K/N` are set only for a PLAIN threshold (`md/policy_shape.go:247-262`, `plainMulti` looks through wrappers but not through `and_v`). `composerSelfCheck` then falls back to comparing the key COUNT alone (`gui/composer_selfcheck.go:104-111`), so a k that differed between the operator's shape and the artifact would pass the self-check and be invisible on the consent.

**Measured today**: the SCRIPT's k is correct in every executed case (Core v31 + v25: tiered `P2 @2 alone` spends after 5 blocks and `P1 @0 alone` cannot; 32-slots `P1 four of nine` refused / `five of nine` accepted, `P2 nine of nine` accepted; `P4 three of five` accepted) — so this is a promise-side silence (spec departure from §7e) rather than a wrong script; the risk it leaves open is an edited preset whose k was mis-tapped (or a future lowering defect in k) reaching steel with the review saying nothing. **Where**: `gui/composer_consent.go:99-104`; `md/policy_shape.go:247-262`; `gui/composer_selfcheck.go:104-111`. Remedy, one sentence: have `collect`/`branchOf` record the (k, n) of the single `multi*` node a branch contains, and print and self-check it.

**RED test** (`TestFableRedConsentNamesThresholdOfLockedMulti`):
```
zz_fable_funds_red_test.go:75: consent does not name the threshold of the locked 1-of-2 path (tiered-recovery preset):
        Path 1: 2-of-2
        Path 2: 2 key(s), custom
          5 blocks (about 0.0 days)
--- FAIL: TestFableRedConsentNamesThresholdOfLockedMulti (0.00s)
```

### M-1 (Minor) — Dates 2009-01-01 and 2009-01-02 are refused with the CEILING body, not §8t

`composerDateBandEcho("20090102")` → `"This build writes dates up to 2038-01-19. For a later time, use a block height instead."`; `"19851105"` → the correct §8t floor body. Cause: `gui/composer_lock.go:327` tests `y < 2009`, but the floor is 2009-01-03, so the two January dates fall through to the ceiling copy. The refusal still refuses (no operand reaches the artifact; `md.Lock.Check` also refuses), so copy only.

### M-2 (Minor / documentation) — A testnet `tpub` in a `key:` record is admitted and shown as a mainnet wallet

`[73c5da0a/48'/1'/0'/2']tpubDFH9dgz…EheQ` (coin 1', tpub version) parses (`sysw/composer_records.go:395` accepts any extended-key version), seats, self-checks, and the consent shows `bc1q…` mainnet addresses; the re-minted mk1 card carries the tpub string under `Network: "mainnet"` (`gui/composer_cards.go:36-40`). The wire is chain-agnostic key material, so nothing is wrong on steel; the device is mainnet-only by design (D1). Worth a refusal or a note at seating; not funds-affecting.

### N-1 (Nit) — Echo grammar and an unreachable echo

`"1 blocks (about 0.0 days)"`; and a 3-unit `older` echoes `"0 days = 3 units of 512 s (0.0 days)"` — unreachable from the pad (minimum 1 day = 169 units), seen only because the harness fed units directly.

---

## What held (executed, not read)

- **Addresses**: device consent (production `policyAddressAt`) == `md address` for receive 0..1 and change 0..1 on all 68 composable shapes (272 addresses); == Core v31.1 `deriveaddresses` on the 63 Core-importable shapes (payload-equal across bc1/bcrt1 and 3…/2…); == Core v25.0 on the 38 shapes v25 can parse (all 28 wsh incl. 32 slots and 8 paths, sh(wsh), sh, `tr(NUMS, sortedmulti_a)`, `tr(K, sortedmulti_a)`). The multipath `<0;1>` form imported into a Core v31 descriptor wallet on 63/63.
- **Lock semantics (Core is the oracle)**: `older(n)` blocks unspendable at tip h+n−2, spendable at h+n−1 (mempool), with `seq=n−1` refused by the signer; time-type `older(0x400000+3)` unspendable while tip MTP = coin MTP+1535 s and spendable at +1536 s (mocktime-anchored), and `seq=3` (wrong type) cannot satisfy it; `after(150)` refused at tip 149, accepted at tip 150, `locktime=149` refused by the signer; `after(499999999)` is a HEIGHT (never final on regtest); `after(500000000)` and `after(1230940800)` are TIMES (final now); `after(2147483647)` non-final; `older(1)`, `older(65535)`, units 1 and 65535 import and derive (long delays not simulated).
- **k-of-n**: 2-of-3 with one key refused / two accepted (wsh, sh(wsh), sh, tr `multi_a`); 2-of-2 with one key refused; 5-of-9 with four refused / five accepted; 9-of-9 and 3-of-5 accepted; same seed at two accounts satisfies 2-of-3 alone and does NOT satisfy 3-of-3 alone — exactly what the two §8g bodies say (both fire, under both wrappers).
- **Hash paths**: keyed+hash needs key AND the right preimage (no preimage / wrong preimage refused, PSBT `PSBT_IN_SHA256` path), under wsh (Core v25 and v31) and tr (v31).
- **Taproot**: extracted internal key spends alone with a 1-item witness (`tr-inheritance-older5`, `tr-multi-then-single` where the key path is the operator's Path 2 and the consent says `Key-path (Path 2)`); every leaf on the right spine reachable, including the deepest of 8 leaves (`tr-eight-paths` P8, `tr-32-slots` L4); NUMS internal key is `50929b74…03ac0` = sha256(uncompressed G) = BIP-341's H (computed), consent shows §8f for every NUMS shape and never for an extracted key.
- **Timelock mixing (C11)**: `wsh-four-paths-mixed-locks` (blocks, 512-s units, height, time in four or_i branches) and its tr twin import into Core v31 (sanity incl. `CheckTimeLocksMix`) and derive the device's addresses.
- **Key order**: `multi` (unsorted) key order == slot order == screen; `@0+@2` spend.
- **Digit pad** (§12 item 7): every edge behaves — `0`, `00000`, `65536`, `389`, `500000000`, `20380120`, `20270231`, `20230229`, `00000000` refused; `1`, `65535`, `388` (=65475 units), `499999999`, `20090103`, `20380119` (=2147472000), `20240229` accepted; `md.Lock.Check` refuses units=0 and >65535; no masked or zero operand reaches the artifact.
- **Refusals that refuse**: lock-only key-less path → `md: compose: a path with neither keys nor a hash is not a spend path: path 2`; key-less under tr → `… not expressible under tr: path 2` (both at `ValidatePathList`, which is the Done arm's gate).
- **§8h**: every route to consent passes the path list's Done arm (`gui/composer_shape.go:539-549`), where the all-hash body is shown after `ValidatePathList` (read, and the `wsh-all-hash` shape's consent carries the §8i kind line).

---

## What I ran (commands and the numbers they produced)

Toolchain: Go 1.26.7; `md 0.16.2`, `mnemonic 0.103.1`, `me 0.10.0`; Bitcoin Core v31.1.0 (`nix shell nixpkgs#bitcoind`, `/nix/store/h014588m81wq2q1pw5s81b8hv610ziy3-bitcoind-31.1/bin`) and v25.0.0 (`/usr/local/bin/bitcoind`, "Bitcoin Satellite v0.2.4" banner), each on `-rpcport=18543 -port=18544`, private datadirs under `/scratch/code/shibboleth/.tmp/fable-funds-work/`, stopped at the end.

1. `git -C /scratch/code/shibboleth/seedhammer worktree add --detach /scratch/code/shibboleth/.tmp/fable-funds f5b068faf3049ccf97603dfbaa3709f12893df97`
2. `CGO_ENABLED=0 FABLE_OUT=…/cases.json go test -run '^TestFable' -v ./gui/` → `wrote 70 cases`; `TestFableSameKeyTwoSlotsReserialized` PASS (observational, output above); `TestFableDigitPadEdges` PASS (observational, 33 values); `TestFableTpubKeyRecord` PASS (observational). 70 shapes: 68 composable, 2 refused as designed.
3. `python3 hostleg.py` → 68/68 `match=True`, 0 needed `--experimental` on `md descriptor`.
4. `CORETAG=v31 python3 coreleg.py <v31 bin> …/core31 addrs` → `v31 match: 63 multipath import ok: 63 of 68`; the 5 non-matches are the key-less shapes (3 × "witnesses without signature exist", 2 × "malleable witnesses exist").
5. `coreleg.py … spend` (three runs, `SPEND_FROM` to resume) → `grep -c '^\[PASS\]'` over `spend_v31*.log` = **118** PASS lines, **1** FAIL line (`wsh-older-units-3 seq=TYPE|3, MTP +1535s -> allowed=True` — my un-anchored first attempt; the re-run with MTP anchored at the coin's height gives `coin+1535s -> allowed=False`, `coin+1536s -> allowed=True`); `sort -u` of PASS lines = **84** distinct tests. Two hand-built key-less spends `sendrawtransaction` + mined: `202512aa…3fe2`, `6b731ed0…1f95`.
6. Dup-key on v31: `getdescriptorinfo` OK, `deriveaddresses` payload-equal to the device, `importdescriptors` (one tprv) `success: True`, spend `allowed=True`, witness `[0, 71, 71, 105]`, `sig1==sig2 True`.
7. `MULTIPATH=0 CORETAG=v25 python3 coreleg.py /usr/local/bin …/core25 addrs` → `v25 match: 38`, `{'tapscript-miniscript': 25, 'keyless-unsafe': 3, 'malleable': 2}`.
8. `CORETAG=v25 python3 coreleg.py /usr/local/bin …/core25 spend` → **39** PASS / **0** FAIL (wsh sections; stops at the first tr shape with "Miniscript expressions can only be used in wsh", as expected on v25).
9. `md compose --wrapper wsh --experimental --path …` for the five key-less lists → rc 0 for one key-less path, rc 1 ("Miniscript is malleable") for two; `md encode --experimental` on the decoded templates → same split; `md decompose` on the dup-key concrete descriptor → rc 1 (pairwise-distinct refusal).
10. `CGO_ENABLED=0 go test -run '^TestFableRed' -v ./gui/` → 3 FAIL (the counterexamples as tests; output quoted under each finding).
11. `python3 -c` sha256 of uncompressed G = `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0` (== the NUMS constant).

Tests added (all in the removed worktree; copies in `…/fable-funds-work/harness/`): `gui/zz_fable_funds_harness_test.go` (generator, env-gated), `gui/zz_fable_funds_extra_test.go` (observational), `gui/zz_fable_funds_red_test.go` (three assertions, all RED on this tree — the RED output above is their proof they can fail; each passes only when the finding is fixed).

## What I could not verify (scoped)

- The live SCREENS for C-1 and I-2: I drove the production functions the flow calls (`composerAddPath`'s only gate is the per-path §8a confirm; the Done arm's only gate is `ValidatePathList`; seating → `composerMappingReview` → `composerDuplicateXpub`/`composerInvariantViolation` → `composerArtifactsFor` → `composerSelfCheck` → `composerConsentLinesFor`), not the emulator walk; the cited lines are the whole set of gates between the path list and consent. `cmd/emu` is not scriptable.
- Sparrow / Nunchuk / Liana imports of any shape (not on this box). Core is the only oracle used.
- Spends on Core v25 for tr shapes (v25 has no tapscript miniscript); tr spends are v31-only. The plain `tr(…,sortedmulti_a)` shapes were address-checked on v25 but not spent there.
- The full delay of `older(65535)`, `older(0x400000+65535)`, and `after(2147483647)` — import, addresses and current non-finality only.
- Form B restore (`seatKeyCards` on the re-minted cards reproducing the keyed addresses) and the physical plates; §12 item 6 covers the former.
- Passphrase-bearing seeds (all four seeds used without a passphrase).

## Out of lens (short)

- `md descriptor --network regtest` still renders mainnet `xpub` version bytes ("Network for xpub validation" only), so its output cannot be pasted into a regtest/testnet Core without re-versioning (lens 3 / host CLI).
- `md descriptor` / `md address` (0.16.2, md-codec 0.44.2) render a duplicate-key wallet that `md decompose` refuses — the decode path no longer runs the mint-time validators (known 0.44.x change); the host is now inconsistent with itself on the dup-key artifact.
- `md descriptor` renders the two-key-less (malleable) template from chunks although `md encode` refuses the same text — same root.
- Core v31 `importdescriptors` of a multipath descriptor marks the change half usable only with an explicit `address_type` on `getrawchangeaddress`; not a device matter.

---

## Coverage matrix (generated from `cases.json`, `host.json`, `core_addrs_v31.json`, `core_addrs_v25.json`, `spend_v31*.log`, `spend_v25.log` by the script in `matrix.md`'s generator; nothing hand-counted)

Columns: screen promise = the consent's per-path lines as the device prints them (condensed with " / "); descriptor = `md decode` of the device's keyless template (the keyed form binds the same tree); device==md = consent addresses vs `md address` (4 per row); Core columns = `deriveaddresses` on the chain-collapsed concrete descriptor, payload-equal to the device; spend column = distinct funded branch tests that PASSED (v31 / v25) with their labels. "-" = not spend-tested (address-checked only).

| # | shape | wrapper | paths (as built) | screen promise (consent lines, condensed) | descriptor template (md decode of the device template) | device==md addrs | Core v31 addrs | Core v25 addrs | branches spend-tested (v31 / v25, distinct PASS) |
|---|---|---|---|---|---|---|---|---|---|
| 1 | wsh-plain-2of3 | wsh | 2-of-3 | Path 1: 2-of-3 | `wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))` | Y | Y | Y | - |
| 2 | wsh-plain-2of3-unsorted | wsh | 2-of-3(unsorted) | Path 1: 2-of-3 / UNSORTED (EXPERIMENTAL) | `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))` | Y | Y | Y | 1 / 0: @0+@2 sign |
| 3 | wsh-single | wsh | 1-of-1 | Path 1: 1 key | `wsh(pkh(@0/<0;1>/*))` | Y | Y | Y | - |
| 4 | wsh-inheritance-older5 | wsh | 1-of-1; 1-of-1+0/5 | Path 1: 1 key / Path 2: 1 key / 5 blocks (about 0.0 days) | `wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(5))))` | Y | Y | Y | 5 / 5: P1 @0 signs, no lock; P2 @1 signs, seq=4 (too small), tip=h+4; P2 @1 signs, seq=5, tip=h+0 (too early); P2 @1 signs, seq=5, tip=h+4; change/1: P1 @0 signs |
| 5 | wsh-kofn-recovery | wsh | 2-of-3; 1-of-1+0/5 | Path 1: 2-of-3 / Path 2: 1 key / 5 blocks (about 0.0 days) | `wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pkh(@3/<0;1>/*),older(5))))` | Y | Y | Y | 4 / 4: P1 one of three keys (insufficient); P1 two of three keys; P2 @3 seq=5 tip=h+0; P2 @3 seq=5 tip=h+4 |
| 6 | wsh-tiered-recovery | wsh | 2-of-2; 1-of-2+0/5 | Path 1: 2-of-2 / Path 2: 2 key(s), custom / 5 blocks (about 0.0 days) | `wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*),and_v(v:multi(1,@2/<0;1>/*,@3/<0;1>/*),older(5))))` | Y | Y | Y | 5 / 5: P1 @0 alone (insufficient); P1 @0+@1; P2 @2 alone seq=5 tip=h+0; P2 @2 alone seq=5 tip=h+4; P2 @3 alone seq=5 tip=h+4 |
| 7 | wsh-hashlock-gated | wsh | 1-of-1+sha256; 1-of-1+0/5 | Path 1: 1 key / hash sha256 cc6a7452..2b0a123d / Path 2: 1 key / 5 blocks (about 0.0 days) | `wsh(or_i(and_v(v:pkh(@0/<0;1>/*),sha256(cc6a74520f526a6135a4eae180547ae73648254ad1ae90bad93520402b0a123d)),and_v(v:pkh(@1/<0;1>/*),older(5))))` | Y | Y | Y | 5 / 5: P1 @0 + preimage; P1 @0 no preimage; P1 @0 wrong preimage; P2 @1 seq=5 tip=h+0; P2 @1 seq=5 tip=h+4 |
| 8 | wsh-decaying | wsh | 2-of-2+0/3; 1-of-1+0/6; 1-of-1+2/150 | Path 1: 2 key(s), custom / 3 blocks (about 0.0 days) / Path 2: 1 key / 6 blocks (about 0.0 days) / Path 3: 1 key / Block 150 / This device cannot tell the time. Nothing here has checked that this is in the future. | `wsh(or_i(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*),older(3)),or_i(and_v(v:pkh(@2/<0;1>/*),older(6)),and_v(v:pkh(@3/<0;1>/*),after(150)))))` | Y | Y | Y | 6 / 5: T1 @0+@1 seq=3 tip=h+0; T1 @0+@1 seq=3 tip=h+2; T2 @2 seq=6 tip=h+2; T2 @2 seq=6 tip=h+5; T3 @3 locktime=150 tip=147; T3 @3 locktime=150 tip=266 |
| 9 | wsh-older-units-1 | wsh | 1-of-1; 1-of-1+1/1 | Path 1: 1 key / Path 2: 1 key / 0 days = 1 units of 512 s (0.0 days) | `wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(4194305))))` | Y | Y | Y | - |
| 10 | wsh-older-units-3 | wsh | 1-of-1; 1-of-1+1/3 | Path 1: 1 key / Path 2: 1 key / 0 days = 3 units of 512 s (0.0 days) | `wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(4194307))))` | Y | Y | Y | 5 / 4: P2 @1 seq=3 (blocks, wrong type); P2 @1 seq=TYPE|3, MTP +1537s; P2 @1 seq=TYPE|3, MTP = coin+1535s; P2 @1 seq=TYPE|3, MTP = coin+1536s; P2 @1 seq=TYPE|3, no time passed |
| 11 | wsh-older-units-65535 | wsh | 1-of-1; 1-of-1+1/65535 | Path 1: 1 key / Path 2: 1 key / 388 days = 65535 units of 512 s (388.4 days) | `wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(4259839))))` | Y | Y | Y | - |
| 12 | wsh-older-blocks-1 | wsh | 1-of-1; 1-of-1+0/1 | Path 1: 1 key / Path 2: 1 key / 1 blocks (about 0.0 days) | `wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(1))))` | Y | Y | Y | - |
| 13 | wsh-older-blocks-65535 | wsh | 1-of-1; 1-of-1+0/65535 | Path 1: 1 key / Path 2: 1 key / 65535 blocks (about 455.1 days) | `wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(65535))))` | Y | Y | Y | - |
| 14 | wsh-after-height-499999999 | wsh | 1-of-1; 1-of-1+2/499999999 | Path 1: 1 key / Path 2: 1 key / Block 499999999 / This device cannot tell the time. Nothing here has checked that this is in the future. | `wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),after(499999999))))` | Y | Y | Y | 2 / 2: P1 @0 no lock; P2 @1 locktime=499999999 |
| 15 | wsh-after-time-500000000 | wsh | 1-of-1; 1-of-1+3/500000000 | Path 1: 1 key / Path 2: 1 key / 1985-11-05 00:00 UTC / This device cannot tell the time. Nothing here has checked that this is in the future. | `wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),after(500000000))))` | Y | Y | Y | 2 / 2: P1 @0 no lock; P2 @1 locktime=500000000 |
| 16 | wsh-after-time-2147483647 | wsh | 1-of-1; 1-of-1+3/2147483647 | Path 1: 1 key / Path 2: 1 key / 2038-01-19 00:00 UTC / This device cannot tell the time. Nothing here has checked that this is in the future. | `wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),after(2147483647))))` | Y | Y | Y | 2 / 2: P1 @0 no lock; P2 @1 locktime=2147483647 |
| 17 | wsh-after-time-2009-01-03 | wsh | 1-of-1; 1-of-1+3/1230940800 | Path 1: 1 key / Path 2: 1 key / 2009-01-03 00:00 UTC / This device cannot tell the time. Nothing here has checked that this is in the future. | `wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),after(1230940800))))` | Y | Y | Y | 2 / 2: P1 @0 no lock; P2 @1 locktime=1230940800 |
| 18 | wsh-after-height-150 | wsh | 1-of-1; 1-of-1+2/150 | Path 1: 1 key / Path 2: 1 key / Block 150 / This device cannot tell the time. Nothing here has checked that this is in the future. | `wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),after(150))))` | Y | Y | Y | 5 / 3: P2 @1 locktime=149 (too small) tip=149; P2 @1 locktime=149 (too small) tip=267; P2 @1 locktime=150 tip=149; P2 @1 locktime=150 tip=150; P2 @1 locktime=150 tip=267 |
| 19 | wsh-multi-then-single | wsh | 2-of-3; 1-of-1 | Path 1: 2-of-3 / Path 2: 1 key | `wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),pkh(@3/<0;1>/*)))` | Y | Y | Y | - |
| 20 | wsh-single-single-multi-older | wsh | 1-of-1; 1-of-1; 2-of-2+0/5 | Path 1: 1 key / Path 2: 1 key / Path 3: 2 key(s), custom / 5 blocks (about 0.0 days) | `wsh(or_i(pkh(@0/<0;1>/*),or_i(pkh(@1/<0;1>/*),and_v(v:multi(2,@2/<0;1>/*,@3/<0;1>/*),older(5)))))` | Y | Y | Y | - |
| 21 | wsh-hash-single-then-multi | wsh | 1-of-1+sha256; 2-of-2 | Path 1: 1 key / hash sha256 cc6a7452..2b0a123d / Path 2: 2-of-2 | `wsh(or_i(and_v(v:pkh(@0/<0;1>/*),sha256(cc6a74520f526a6135a4eae180547ae73648254ad1ae90bad93520402b0a123d)),multi(2,@1/<0;1>/*,@2/<0;1>/*)))` | Y | Y | Y | - |
| 22 | wsh-same-seed-2of3-threshold | wsh | 2-of-3 | Path 1: 2-of-3 | `wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))` | Y | Y | Y | 1 / 0: seed0 accounts 0+1 alone satisfy 2-of-3 |
| 23 | wsh-same-seed-3of3-below | wsh | 3-of-3 | Path 1: 3-of-3 | `wsh(sortedmulti(3,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))` | Y | Y | Y | 1 / 0: seed0 accounts 0+1 alone do NOT satisfy 3-of-3 |
| 24 | wsh-same-seed-two-paths | wsh | 1-of-1; 1-of-1+0/5 | Path 1: 1 key / Path 2: 1 key / 5 blocks (about 0.0 days) | `wsh(or_i(pkh(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(5))))` | Y | Y | Y | - |
| 25 | wsh-multi-hash-older-all | wsh | 2-of-3+sha256+0/5; 1-of-1+sha256+2/150 | Path 1: 3 key(s), custom / 5 blocks (about 0.0 days) / hash sha256 cc6a7452..2b0a123d / Path 2: 1 key / Block 150 / This device cannot tell the time. Nothing here has checked that this is in the future. / hash sha256 5a23c169..44484a3d | `wsh(or_i(and_v(v:multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:sha256(cc6a74520f526a6135a4eae180547ae73648254ad1ae90bad93520402b0a123d),older(5))),and_v(v:pkh(@3/<0;1>/*),and_v(v:sha256(5a23c169bc7e6f2569a3d74f58118576f005118603473a7069e22ab544484a3d),after(150)))))` | Y | Y | Y | - |
| 26 | wsh-four-paths-mixed-locks | wsh | 2-of-2; 1-of-1+0/5; 1-of-1+1/3; 1-of-1+2/150; 1-of-1+3/500000000 | Path 1: 2-of-2 / Path 2: 1 key / 5 blocks (about 0.0 days) / Path 3: 1 key / 0 days = 3 units of 512 s (0.0 days) / Path 4: 1 key / Block 150 / This device cannot tell the time. Nothing here has checked that this is in the future. / Path 5: 1 key / 1985-11-05 00:00 UTC / This device cannot tell the time. Nothing here has checked that this is in the future. | `wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*),or_i(and_v(v:pkh(@2/<0;1>/*),older(5)),or_i(and_v(v:pkh(@3/<0;1>/*),older(4194307)),or_i(and_v(v:pkh(@4/<0;1>/*),after(150)),and_v(v:pkh(@5/<0;1>/*),after(500000000)))))))` | Y | Y | Y | - |
| 27 | wsh-32-slots | wsh | 5-of-9; 9-of-9+0/5; 1-of-9+0/6; 3-of-5+2/150 | Path 1: 5-of-9 / Path 2: 9 key(s), custom / 5 blocks (about 0.0 days) / Path 3: 9 key(s), custom / 6 blocks (about 0.0 days) / Path 4: 5 key(s), custom / Block 150 / This device cannot tell the time. Nothing here has checked that this is in the future. | `wsh(or_d(multi(5,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*,@3/<0;1>/*,@4/<0;1>/*,@5/<0;1>/*,@6/<0;1>/*,@7/<0;1>/*,@8/<0;1>/*),or_i(and_v(v:multi(9,@9/<0;1>/*,@10/<0;1>/*,@11/<0;1>/*,@12/<0;1>/*,@13/<0;1>/*,@14/<0;1>/*,@15/<0;1>/*,@16/<0;1>/*,@17/<0;1>/*),older(5)),or_i(and_v(v:multi(1,@18/<0;1>/*,@19/<0;1>/*,@20/<0;1>/*,@21/<0;1>/*,@22/<0;1>/*,@23/<0;1>/*,@24/<0;1>/*,@25/<0;1>/*,@26/<0;1>/*),older(6)),and_v(v:multi(3,@27/<0;1>/*,@28/<0;1>/*,@29/<0;1>/*,@30/<0;1>/*,@31/<0;1>/*),after(150))))))` | Y | Y | Y | 4 / 0: P1 five of nine; P1 four of nine (insufficient); P2 nine of nine seq=5 tip=h+4; P4 three of five locktime 150 |
| 28 | wsh-eight-paths | wsh | 1-of-1; 1-of-1+0/1; 1-of-1+0/2; 1-of-1+0/3; 1-of-1+0/4; 1-of-1+0/5; 1-of-1+0/6; 1-of-1+0/7 | Path 1: 1 key / Path 2: 1 key / 1 blocks (about 0.0 days) / Path 3: 1 key / 2 blocks (about 0.0 days) / Path 4: 1 key / 3 blocks (about 0.0 days) / Path 5: 1 key / 4 blocks (about 0.0 days) / Path 6: 1 key / 5 blocks (about 0.0 days) / Path 7: 1 key / 6 blocks (about 0.0 days) / Path 8: 1 key / 7 blocks (about 0.0 days) | `wsh(or_i(pkh(@0/<0;1>/*),or_i(and_v(v:pkh(@1/<0;1>/*),older(1)),or_i(and_v(v:pkh(@2/<0;1>/*),older(2)),or_i(and_v(v:pkh(@3/<0;1>/*),older(3)),or_i(and_v(v:pkh(@4/<0;1>/*),older(4)),or_i(and_v(v:pkh(@5/<0;1>/*),older(5)),or_i(and_v(v:pkh(@6/<0;1>/*),older(6)),and_v(v:pkh(@7/<0;1>/*),older(7))))))))))` | Y | Y | Y | 2 / 0: P8 @7 seq=6 (too small); P8 @7 seq=7 tip=h+6 |
| 29 | tr-plain-2of3 | tr | 2-of-3 | Path 1: 2-of-3 / KEY PATH: NONE (NUMS)
Spends use the script paths only. Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable xpub instead (see F-449). | `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))` | Y | Y | Y | - |
| 30 | tr-plain-2of3-unsorted | tr | 2-of-3(unsorted) | Path 1: 2-of-3 / UNSORTED (EXPERIMENTAL) / KEY PATH: NONE (NUMS)
Spends use the script paths only. Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable xpub instead (see F-449). | `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))` | Y | Y | Y | - |
| 31 | tr-single | tr | 1-of-1 | Key-path (Path 1): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*)` | Y | Y | Y | - |
| 32 | tr-inheritance-older5 | tr | 1-of-1; 1-of-1+0/5 | Path 2: 1 key / 5 blocks (about 0.0 days) / Key-path (Path 1): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),older(5)))` | Y | Y | no tapscript miniscript | 3 / 0: IK @0 key-path spend; leaf @1 seq=5 tip=h+0; leaf @1 seq=5 tip=h+4 |
| 33 | tr-kofn-recovery | tr | 2-of-3; 1-of-1+0/5 | Path 1: 2-of-3 / Path 2: 1 key / 5 blocks (about 0.0 days) / KEY PATH: NONE (NUMS)
Spends use the script paths only. Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable xpub instead (see F-449). | `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:pk(@3/<0;1>/*),older(5))})` | Y | Y | no tapscript miniscript | 4 / 0: L1 one key (insufficient); L1 two keys; L2 @3 seq=5 tip=h+0; L2 @3 seq=5 tip=h+4 |
| 34 | tr-tiered-recovery | tr | 2-of-2; 1-of-2+0/5 | Path 1: 2-of-2 / Path 2: 2 key(s), custom / 5 blocks (about 0.0 days) / KEY PATH: NONE (NUMS)
Spends use the script paths only. Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable xpub instead (see F-449). | `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{multi_a(2,@0/<0;1>/*,@1/<0;1>/*),and_v(v:multi_a(1,@2/<0;1>/*,@3/<0;1>/*),older(5))})` | Y | Y | no tapscript miniscript | - |
| 35 | tr-hashlock-gated | tr | 1-of-1+sha256; 1-of-1+0/5 | Path 1: 1 key / hash sha256 cc6a7452..2b0a123d / Path 2: 1 key / 5 blocks (about 0.0 days) / KEY PATH: NONE (NUMS)
Spends use the script paths only. Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable xpub instead (see F-449). | `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{and_v(v:pk(@0/<0;1>/*),sha256(cc6a74520f526a6135a4eae180547ae73648254ad1ae90bad93520402b0a123d)),and_v(v:pk(@1/<0;1>/*),older(5))})` | Y | Y | no tapscript miniscript | 4 / 0: L1 @0 + preimage; L1 @0 no preimage; L2 @1 seq=5 tip=h+0; L2 @1 seq=5 tip=h+4 |
| 36 | tr-decaying | tr | 2-of-2+0/3; 1-of-1+0/6; 1-of-1+2/150 | Path 1: 2 key(s), custom / 3 blocks (about 0.0 days) / Path 2: 1 key / 6 blocks (about 0.0 days) / Path 3: 1 key / Block 150 / This device cannot tell the time. Nothing here has checked that this is in the future. / KEY PATH: NONE (NUMS)
Spends use the script paths only. Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable xpub instead (see F-449). | `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{and_v(v:multi_a(2,@0/<0;1>/*,@1/<0;1>/*),older(3)),{and_v(v:pk(@2/<0;1>/*),older(6)),and_v(v:pk(@3/<0;1>/*),after(150))}})` | Y | Y | no tapscript miniscript | 4 / 0: T1 @0+@1 seq=3 tip=h+2; T2 @2 seq=6 tip=h+2; T2 @2 seq=6 tip=h+5; T3 @3 locktime=150 |
| 37 | tr-older-units-1 | tr | 1-of-1; 1-of-1+1/1 | Path 2: 1 key / 0 days = 1 units of 512 s (0.0 days) / Key-path (Path 1): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),older(4194305)))` | Y | Y | no tapscript miniscript | - |
| 38 | tr-older-units-3 | tr | 1-of-1; 1-of-1+1/3 | Path 2: 1 key / 0 days = 3 units of 512 s (0.0 days) / Key-path (Path 1): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),older(4194307)))` | Y | Y | no tapscript miniscript | - |
| 39 | tr-older-units-65535 | tr | 1-of-1; 1-of-1+1/65535 | Path 2: 1 key / 388 days = 65535 units of 512 s (388.4 days) / Key-path (Path 1): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),older(4259839)))` | Y | Y | no tapscript miniscript | - |
| 40 | tr-older-blocks-1 | tr | 1-of-1; 1-of-1+0/1 | Path 2: 1 key / 1 blocks (about 0.0 days) / Key-path (Path 1): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),older(1)))` | Y | Y | no tapscript miniscript | - |
| 41 | tr-older-blocks-65535 | tr | 1-of-1; 1-of-1+0/65535 | Path 2: 1 key / 65535 blocks (about 455.1 days) / Key-path (Path 1): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),older(65535)))` | Y | Y | no tapscript miniscript | - |
| 42 | tr-after-height-499999999 | tr | 1-of-1; 1-of-1+2/499999999 | Path 2: 1 key / Block 499999999 / This device cannot tell the time. Nothing here has checked that this is in the future. / Key-path (Path 1): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),after(499999999)))` | Y | Y | no tapscript miniscript | - |
| 43 | tr-after-time-500000000 | tr | 1-of-1; 1-of-1+3/500000000 | Path 2: 1 key / 1985-11-05 00:00 UTC / This device cannot tell the time. Nothing here has checked that this is in the future. / Key-path (Path 1): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),after(500000000)))` | Y | Y | no tapscript miniscript | - |
| 44 | tr-after-time-2147483647 | tr | 1-of-1; 1-of-1+3/2147483647 | Path 2: 1 key / 2038-01-19 00:00 UTC / This device cannot tell the time. Nothing here has checked that this is in the future. / Key-path (Path 1): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),after(2147483647)))` | Y | Y | no tapscript miniscript | - |
| 45 | tr-after-time-2009-01-03 | tr | 1-of-1; 1-of-1+3/1230940800 | Path 2: 1 key / 2009-01-03 00:00 UTC / This device cannot tell the time. Nothing here has checked that this is in the future. / Key-path (Path 1): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),after(1230940800)))` | Y | Y | no tapscript miniscript | - |
| 46 | tr-after-height-150 | tr | 1-of-1; 1-of-1+2/150 | Path 2: 1 key / Block 150 / This device cannot tell the time. Nothing here has checked that this is in the future. / Key-path (Path 1): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),after(150)))` | Y | Y | no tapscript miniscript | - |
| 47 | tr-multi-then-single | tr | 2-of-3; 1-of-1 | Path 1: 2-of-3 / Key-path (Path 2): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*,sortedmulti_a(2,@1/<0;1>/*,@2/<0;1>/*,@3/<0;1>/*))` | Y | Y | Y | 3 / 0: Path 1 @1+@3; Path 1 one key (insufficient); Path 2 (@0) key-path alone |
| 48 | tr-single-single-multi-older | tr | 1-of-1; 1-of-1; 2-of-2+0/5 | Path 2: 1 key / Path 3: 2 key(s), custom / 5 blocks (about 0.0 days) / Key-path (Path 1): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*,{pk(@1/<0;1>/*),and_v(v:multi_a(2,@2/<0;1>/*,@3/<0;1>/*),older(5))})` | Y | Y | no tapscript miniscript | - |
| 49 | tr-hash-single-then-multi | tr | 1-of-1+sha256; 2-of-2 | Path 1: 1 key / hash sha256 cc6a7452..2b0a123d / Path 2: 2-of-2 / KEY PATH: NONE (NUMS)
Spends use the script paths only. Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable xpub instead (see F-449). | `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{and_v(v:pk(@0/<0;1>/*),sha256(cc6a74520f526a6135a4eae180547ae73648254ad1ae90bad93520402b0a123d)),multi_a(2,@1/<0;1>/*,@2/<0;1>/*)})` | Y | Y | no tapscript miniscript | - |
| 50 | tr-same-seed-2of3-threshold | tr | 2-of-3 | Path 1: 2-of-3 / KEY PATH: NONE (NUMS)
Spends use the script paths only. Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable xpub instead (see F-449). | `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))` | Y | Y | Y | - |
| 51 | tr-same-seed-3of3-below | tr | 3-of-3 | Path 1: 3-of-3 / KEY PATH: NONE (NUMS)
Spends use the script paths only. Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable xpub instead (see F-449). | `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,sortedmulti_a(3,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))` | Y | Y | Y | - |
| 52 | tr-same-seed-two-paths | tr | 1-of-1; 1-of-1+0/5 | Path 2: 1 key / 5 blocks (about 0.0 days) / Key-path (Path 1): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),older(5)))` | Y | Y | no tapscript miniscript | - |
| 53 | tr-multi-hash-older-all | tr | 2-of-3+sha256+0/5; 1-of-1+sha256+2/150 | Path 1: 3 key(s), custom / 5 blocks (about 0.0 days) / hash sha256 cc6a7452..2b0a123d / Path 2: 1 key / Block 150 / This device cannot tell the time. Nothing here has checked that this is in the future. / hash sha256 5a23c169..44484a3d / KEY PATH: NONE (NUMS)
Spends use the script paths only. Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable xpub instead (see F-449). | `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{and_v(v:multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),and_v(v:sha256(cc6a74520f526a6135a4eae180547ae73648254ad1ae90bad93520402b0a123d),older(5))),and_v(v:pk(@3/<0;1>/*),and_v(v:sha256(5a23c169bc7e6f2569a3d74f58118576f005118603473a7069e22ab544484a3d),after(150)))})` | Y | Y | no tapscript miniscript | - |
| 54 | tr-four-paths-mixed-locks | tr | 2-of-2; 1-of-1+0/5; 1-of-1+1/3; 1-of-1+2/150; 1-of-1+3/500000000 | Path 1: 2-of-2 / Path 2: 1 key / 5 blocks (about 0.0 days) / Path 3: 1 key / 0 days = 3 units of 512 s (0.0 days) / Path 4: 1 key / Block 150 / This device cannot tell the time. Nothing here has checked that this is in the future. / Path 5: 1 key / 1985-11-05 00:00 UTC / This device cannot tell the time. Nothing here has checked that this is in the future. / KEY PATH: NONE (NUMS)
Spends use the script paths only. Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable xpub instead (see F-449). | `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{multi_a(2,@0/<0;1>/*,@1/<0;1>/*),{and_v(v:pk(@2/<0;1>/*),older(5)),{and_v(v:pk(@3/<0;1>/*),older(4194307)),{and_v(v:pk(@4/<0;1>/*),after(150)),and_v(v:pk(@5/<0;1>/*),after(500000000))}}}})` | Y | Y | no tapscript miniscript | - |
| 55 | tr-32-slots | tr | 5-of-9; 9-of-9+0/5; 1-of-9+0/6; 3-of-5+2/150 | Path 1: 5-of-9 / Path 2: 9 key(s), custom / 5 blocks (about 0.0 days) / Path 3: 9 key(s), custom / 6 blocks (about 0.0 days) / Path 4: 5 key(s), custom / Block 150 / This device cannot tell the time. Nothing here has checked that this is in the future. / KEY PATH: NONE (NUMS)
Spends use the script paths only. Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable xpub instead (see F-449). | `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{multi_a(5,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*,@3/<0;1>/*,@4/<0;1>/*,@5/<0;1>/*,@6/<0;1>/*,@7/<0;1>/*,@8/<0;1>/*),{and_v(v:multi_a(9,@9/<0;1>/*,@10/<0;1>/*,@11/<0;1>/*,@12/<0;1>/*,@13/<0;1>/*,@14/<0;1>/*,@15/<0;1>/*,@16/<0;1>/*,@17/<0;1>/*),older(5)),{and_v(v:multi_a(1,@18/<0;1>/*,@19/<0;1>/*,@20/<0;1>/*,@21/<0;1>/*,@22/<0;1>/*,@23/<0;1>/*,@24/<0;1>/*,@25/<0;1>/*,@26/<0;1>/*),older(6)),and_v(v:multi_a(3,@27/<0;1>/*,@28/<0;1>/*,@29/<0;1>/*,@30/<0;1>/*,@31/<0;1>/*),after(150))}}})` | Y | Y | no tapscript miniscript | 2 / 0: L1 five of nine; L4 (deepest) three of five locktime 150 |
| 56 | tr-eight-paths | tr | 1-of-1; 1-of-1+0/1; 1-of-1+0/2; 1-of-1+0/3; 1-of-1+0/4; 1-of-1+0/5; 1-of-1+0/6; 1-of-1+0/7 | Path 2: 1 key / 1 blocks (about 0.0 days) / Path 3: 1 key / 2 blocks (about 0.0 days) / Path 4: 1 key / 3 blocks (about 0.0 days) / Path 5: 1 key / 4 blocks (about 0.0 days) / Path 6: 1 key / 5 blocks (about 0.0 days) / Path 7: 1 key / 6 blocks (about 0.0 days) / Path 8: 1 key / 7 blocks (about 0.0 days) / Key-path (Path 1): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*,{and_v(v:pk(@1/<0;1>/*),older(1)),{and_v(v:pk(@2/<0;1>/*),older(2)),{and_v(v:pk(@3/<0;1>/*),older(3)),{and_v(v:pk(@4/<0;1>/*),older(4)),{and_v(v:pk(@5/<0;1>/*),older(5)),{and_v(v:pk(@6/<0;1>/*),older(6)),and_v(v:pk(@7/<0;1>/*),older(7))}}}}}})` | Y | Y | no tapscript miniscript | 2 / 0: P8 @7 seq=6 (too small); P8 @7 seq=7 tip=h+6 |
| 57 | wsh-keyless-hash-only | wsh | 1-of-1; keyless+sha256 | Path 1: 1 key / Path 2: KEY-LESS (EXPERIMENTAL) / hash sha256 cc6a7452..2b0a123d | `wsh(or_i(pkh(@0/<0;1>/*),sha256(cc6a74520f526a6135a4eae180547ae73648254ad1ae90bad93520402b0a123d)))` | Y | REFUSED: witnesses without signature exist | REFUSED: witnesses without signature exist | 2 / 0: P2 WRONG preimage (hand witness); P2 preimage only (hand witness), no key, seq=4294967293 tip=h+0 |
| 58 | wsh-keyless-hash-older | wsh | 1-of-1; keyless+sha256+0/5 | Path 1: 1 key / Path 2: KEY-LESS (EXPERIMENTAL) / 5 blocks (about 0.0 days) / hash sha256 cc6a7452..2b0a123d | `wsh(or_i(pkh(@0/<0;1>/*),and_v(v:sha256(cc6a74520f526a6135a4eae180547ae73648254ad1ae90bad93520402b0a123d),older(5))))` | Y | REFUSED: witnesses without signature exist | REFUSED: witnesses without signature exist | 4 / 0: P2 WRONG preimage (hand witness); P2 preimage only (hand witness), no key, seq=5 tip=h+0; P2 preimage only (hand witness), seq=4 too small, tip=h+4; P2 preimage only (hand witness), seq=5, tip=h+4 |
| 59 | wsh-keyless-first | wsh | keyless+sha256+0/5; 2-of-2 | Path 1: KEY-LESS (EXPERIMENTAL) / 5 blocks (about 0.0 days) / hash sha256 cc6a7452..2b0a123d / Path 2: 2-of-2 | `wsh(or_i(and_v(v:sha256(cc6a74520f526a6135a4eae180547ae73648254ad1ae90bad93520402b0a123d),older(5)),multi(2,@0/<0;1>/*,@1/<0;1>/*)))` | Y | REFUSED: witnesses without signature exist | REFUSED: witnesses without signature exist | - |
| 60 | wsh-two-keyless | wsh | 1-of-1; keyless+sha256; keyless+sha256+0/5 | Path 1: 1 key / Path 2: KEY-LESS (EXPERIMENTAL) / hash sha256 cc6a7452..2b0a123d / Path 3: KEY-LESS (EXPERIMENTAL) / 5 blocks (about 0.0 days) / hash sha256 5a23c169..44484a3d | `wsh(or_i(pkh(@0/<0;1>/*),or_i(sha256(cc6a74520f526a6135a4eae180547ae73648254ad1ae90bad93520402b0a123d),and_v(v:sha256(5a23c169bc7e6f2569a3d74f58118576f005118603473a7069e22ab544484a3d),older(5)))))` | Y | REFUSED: malleable witnesses exist | REFUSED: malleable witnesses exist | - |
| 61 | wsh-two-keyless-first | wsh | keyless+sha256; keyless+sha256+0/5; 1-of-1 | Path 1: KEY-LESS (EXPERIMENTAL) / hash sha256 cc6a7452..2b0a123d / Path 2: KEY-LESS (EXPERIMENTAL) / 5 blocks (about 0.0 days) / hash sha256 5a23c169..44484a3d / Path 3: 1 key | `wsh(or_i(sha256(cc6a74520f526a6135a4eae180547ae73648254ad1ae90bad93520402b0a123d),or_i(and_v(v:sha256(5a23c169bc7e6f2569a3d74f58118576f005118603473a7069e22ab544484a3d),older(5)),pkh(@0/<0;1>/*))))` | Y | REFUSED: malleable witnesses exist | REFUSED: malleable witnesses exist | - |
| 62 | wsh-keyless-lock-only | wsh | 1-of-1; keyless+0/5 | REFUSED at Done: `md: compose: a path with neither keys nor a hash is not a spend path: path 2` | - | - | - | - | - |
| 63 | wsh-all-hash | wsh | 1-of-1+sha256; 2-of-2+sha256 | Path 1: 1 key / hash sha256 cc6a7452..2b0a123d / Path 2: 2 key(s), custom / hash sha256 5a23c169..44484a3d | `wsh(or_i(and_v(v:pkh(@0/<0;1>/*),sha256(cc6a74520f526a6135a4eae180547ae73648254ad1ae90bad93520402b0a123d)),and_v(v:multi(2,@1/<0;1>/*,@2/<0;1>/*),sha256(5a23c169bc7e6f2569a3d74f58118576f005118603473a7069e22ab544484a3d))))` | Y | Y | Y | - |
| 64 | sh-wsh-2of3 | sh-wsh | 2-of-3 | Path 1: 2-of-3 | `sh(wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*)))` | Y | Y | Y | 2 / 0: @1+@2 sign; one key (insufficient) |
| 65 | sh-2of3 | sh | 2-of-3 | Path 1: 2-of-3 | `sh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))` | Y | Y | Y | 2 / 0: @1+@2 sign; one key (insufficient) |
| 66 | sh-wsh-2of2 | sh-wsh | 2-of-2 | Path 1: 2-of-2 | `sh(wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*)))` | Y | Y | Y | - |
| 67 | tr-keyless-refused | tr | 1-of-1; keyless+sha256 | REFUSED at Done: `md: compose: a key-less path is not expressible under tr: path 2` | - | - | - | - | - |
| 68 | tr-hash-on-single-nums | tr | 1-of-1+sha256; 1-of-1+0/5 | Path 1: 1 key / hash sha256 cc6a7452..2b0a123d / Path 2: 1 key / 5 blocks (about 0.0 days) / KEY PATH: NONE (NUMS)
Spends use the script paths only. Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable xpub instead (see F-449). | `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{and_v(v:pk(@0/<0;1>/*),sha256(cc6a74520f526a6135a4eae180547ae73648254ad1ae90bad93520402b0a123d)),and_v(v:pk(@1/<0;1>/*),older(5))})` | Y | Y | no tapscript miniscript | - |
| 69 | tr-three-leaves-nums | tr | 2-of-2; 1-of-1+0/5; 1-of-1+2/150 | Path 1: 2-of-2 / Path 2: 1 key / 5 blocks (about 0.0 days) / Path 3: 1 key / Block 150 / This device cannot tell the time. Nothing here has checked that this is in the future. / KEY PATH: NONE (NUMS)
Spends use the script paths only. Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable xpub instead (see F-449). | `tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{multi_a(2,@0/<0;1>/*,@1/<0;1>/*),{and_v(v:pk(@2/<0;1>/*),older(5)),and_v(v:pk(@3/<0;1>/*),after(150))}})` | Y | Y | no tapscript miniscript | - |
| 70 | tr-nine-leaves-multi | tr | 1-of-1; 2-of-2; 1-of-2+0/2; 2-of-3+0/3; 1-of-1+0/4; 1-of-1+0/5; 1-of-1+0/6; 1-of-1+0/7 | Path 2: 2-of-2 / Path 3: 2 key(s), custom / 2 blocks (about 0.0 days) / Path 4: 3 key(s), custom / 3 blocks (about 0.0 days) / Path 5: 1 key / 4 blocks (about 0.0 days) / Path 6: 1 key / 5 blocks (about 0.0 days) / Path 7: 1 key / 6 blocks (about 0.0 days) / Path 8: 1 key / 7 blocks (about 0.0 days) / Key-path (Path 1): A KEY CAN SPEND ALONE | `tr(@0/<0;1>/*,{multi_a(2,@1/<0;1>/*,@2/<0;1>/*),{and_v(v:multi_a(1,@3/<0;1>/*,@4/<0;1>/*),older(2)),{and_v(v:multi_a(2,@5/<0;1>/*,@6/<0;1>/*,@7/<0;1>/*),older(3)),{and_v(v:pk(@8/<0;1>/*),older(4)),{and_v(v:pk(@9/<0;1>/*),older(5)),{and_v(v:pk(@10/<0;1>/*),older(6)),and_v(v:pk(@11/<0;1>/*),older(7))}}}}}})` | Y | Y | no tapscript miniscript | - |
