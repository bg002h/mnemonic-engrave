# PLAN: exporting "our reasonably complex wallet" to Bitcoin Core

Recon/planning report, 2026-08-22. Question: what must the m* constellation emit
so Bitcoin Core loads the four-tier degrading vault, (a) watch-only, (b) hot.

Method note: every Core claim below was tested against real `bitcoind` binaries
on this machine — v25.0 (the locally installed Bitcoin Satellite build, which
reports "Bitcoin Core version v25.0.0"), plus official bitcoincore.org release
binaries v26.0, v27.0, v28.0, v29.0 and v31.1 (current latest), each run as an
offline mainnet node (`-networkactive=0`) in a scratch datadir. "[verified:
empirical vN]" means the exact command was run against that binary this session.
A mid-flight correction landed from the coordinator: `mnemonic export-wallet`
(mnemonic-toolkit, v0.97.0) already exists with a default `bitcoin-core` format;
section 4 is re-aimed at extending it, and its behavior was re-verified here by
running it, not taken from the sibling report.

---

## 1. VERDICT

**The wallet as designed — with the keyless tier 4 — cannot be loaded into
Bitcoin Core AT ALL, in either wrapper, at any released version up to and
including v31.1.** Not watch-only, not hot. This is not a version-support gap;
it is a deliberate Core sanity rule with no opt-out.

| artifact | Core watch? | Core solve? | evidence |
| --- | --- | --- | --- |
| full 4-tier `tr(...)` | **NO, every version** | NO | `importdescriptors`/`getdescriptorinfo` refuse: `and_v(v:after(1383520),sha256(4743d7…954)) is not sane: witnesses without signature exist` [verified: empirical v26 (single-path), v29, v31.1 (multipath); v25 refuses earlier because tr-miniscript is unsupported there] |
| full 4-tier `wsh(...)` | **NO, every version** | NO | same error, whole `or_i` quoted [verified: empirical v25 (single-path), v29, v31.1 (multipath)] |
| the same wallets minus tier 4 (a DIFFERENT wallet — see warning below) | wsh: v24+ single-path, v29+ multipath. tr: v26+ single-path, v29+ multipath | wsh: v25+. tr: v26+ | matrix + citations in §2 [verified: empirical + release notes] |
| per-address `addr(bc1…)` imports of the real wallet's addresses | **YES, works today** | never | `importdescriptors` of `addr(bc1puvyd9zxz6uvz0y0ehq7r5qz6h30txl4mgr8fxl3dqjp6xzpsy0qsgpgyny)` succeeded on v29 [verified: empirical] |

Mechanism of the refusal: Core requires every miniscript spend path to require
a signature. `src/script/descriptor.cpp` (v29.0) reports `": contains duplicate
public keys"` / `"is not sane: witnesses without signature exist"` from the
miniscript sanity block at lines 2106–2107; the signature requirement is
`IsSaneTopLevel()`'s `NeedsSignature()` term. It applies per-taptree-leaf in
`tr()` and to the whole script in `wsh()`, at descriptor *parse* time — so it
also blocks `getdescriptorinfo`, `deriveaddresses` and `scantxoutset`
[verified: `scantxoutset` and `deriveaddresses` refused with the same error on
v29]. There is no RPC flag, wallet flag or startup option to waive it — none in
`help importdescriptors`/`help getdescriptorinfo` on v29 or v31.1 [verified:
help text; absence of a bypass in released versions]. v24 is expected to behave
the same (the sanity code shipped with wsh-miniscript support itself, PR #24148)
but was not run here [unverified: reasoning — v24 binary not tested].

**WARNING — the tiers-1-3 truncation is not a fallback.** Dropping tier 4
produces a different taptree / witness script, therefore different addresses.
It imports beautifully (all rows below) and it watches *nothing this vault
owns*. Do not report it as "Core support with a caveat"; it is a different
wallet. [verified: `getnewaddress` on the truncated wsh import returned
`bc1q2duhch83rd737mrcww7v20nqtrlaq6pvpq8dzlltstzjav75e33q22apl4`, ≠ the
fixture's `bc1qyd7k9t5y0pxsg558y7mypgekdf0y25awnkw9tlvtec59c4wu5eeqsagcgq`]

What Core loading means for the real wallet, then:

- **Watch-only: possible ONLY via `addr()` imports** of constellation-derived
  addresses (md derives them; Core cannot — its parser refuses the descriptor
  before address derivation). Core will track funds and see spends; it will
  never construct or solve a spend, and the address window must be topped up
  manually (no keypool for non-ranged imports).
- **Hot: not possible for this wallet, full stop.** Solving requires the
  descriptor to be imported; the descriptor cannot be imported.
- If the *design* ever adds a key requirement to tier 4 (e.g.
  `and_v(v:after(1383520),and_v(v:sha256(H3),pk(@recovery)))`), everything
  imports at v29+ in multipath form and Core can sign the key portions of every
  tier — the hot flow in §3.3 was proven end-to-end on the truncated variant.

## 2. Version matrix, empirically pinned

| capability | min version | how known |
| --- | --- | --- |
| miniscript in `wsh()`, watch-only import | **v24.0.1** | release notes: "The `wsh()` output descriptor was extended with Miniscript support. You can import Miniscript descriptors for P2WSH in a watchonly wallet to track coins, but you can't spend from them using the Bitcoin Core wallet yet." (#24148) [verified: release-notes-24.0.1.md]; v25 accepts our tiers-1-3 wsh single-path with `"issolvable": true` [verified: empirical v25] |
| miniscript in `wsh()`, signing | **v25.0** | release notes: "Descriptor wallets can now spend coins sent to P2WSH Miniscript descriptors. (#24149)"; "`finalizepsbt` is now able to finalize a transaction with inputs spending Miniscript-compatible P2WSH scripts. (#24149)" [verified: release-notes-25.0.md] |
| miniscript in `tr()` leaves (parse AND sign) | **v26.0** | release notes: "Miniscript expressions can now be used in Taproot descriptors for all RPCs working with descriptors. (#27255)"; "`finalizepsbt` is now able to finalize a PSBT with inputs spending Miniscript-compatible Taproot leaves. (#27255)" [verified: release-notes-26.0.md]; empirically: v25 rejects our tr leaf as "not a valid descriptor function", v26 accepts tiers-1-3 tr single-path [verified: empirical v25, v26] |
| multipath `<0;1>` — ANYWHERE in a descriptor | **v29.0** | empirically absent in v25, v26, v27.0, v28.0 (`Key path value '<0;1>' is not a valid uint32` even for plain `wpkh()`), present in v29.0 [verified: empirical, all five binaries]; PR #22838 "descriptors: Be able to specify change and receiving in a single descriptor string", merged 2024-08-28, milestone 29.0 [verified: GitHub API] — note the 29.0 release notes do NOT mention it, so do not expect users to know this boundary |
| `multi_a` in a taptree (plain, non-miniscript) | v24 | release notes (#24043) [verified: release-notes-24.0.1.md]; inside a miniscript leaf it needs v26 like everything else there [verified: empirical] |
| keyless spend path (tier 4) | **never (through v31.1)** | §1 [verified: empirical v25/v26/v29/v31.1] |
| PSBT hash-preimage fields (`sha256_preimages` etc.) | ≤ v25 | `help decodepsbt` lists ripemd160/sha256/hash160/hash256 preimage fields on the v25 node [verified: help text] |

Feature acceptance within the tiers-1-3 subset, all confirmed in one import
[verified: empirical v25 (wsh), v26 (tr), v29/v31.1 (both, multipath)]:
`sha256()` branches — yes, watchable and solvable-in-principle; mixed
`older(32768)` + `after(1173520)` in one descriptor — yes (miniscript's
timelock-mix rule binds within one satisfaction path, and each tier uses one
flavour); `multi()` under `or_i` in wsh — yes; `multi_a` inside a miniscript
taptree leaf — yes (v26+).

## 3. The exact RPC mechanics (worked, not sketched)

### 3.1 Watch-only recipe — and where the real wallet fails in it

```sh
# 1. wallet: no private keys, no auto-generated descriptors
bitcoin-cli -named createwallet wallet_name=rcw_watch disable_private_keys=true \
    blank=true descriptors=true

# 2. import — ONE request, the multipath descriptor with its checksum
bitcoin-cli -rpcwallet=rcw_watch importdescriptors '[
  {
    "desc": "wsh(or_i(and_v(v:sha256(ede0…),multi(3,[39ec1b6e/270028'"'"'/0'"'"'/9'"'"'/1'"'"']xpub…/<0;1>/*,…)),…))#<checksum>",
    "active": true,
    "range": [0, 999],
    "timestamp": 0
  }
]'
```

Field semantics, from `help importdescriptors` on v29 [verified: help text]:
`timestamp` (required) — unix time to rescan from; `0` = whole chain (right for
a wallet that may already have funds), `"now"` = skip rescan; `range` — required
when a ranged descriptor is `active`; `active` — makes it supply
`getnewaddress`; `internal` — must NOT be passed with a 2-element multipath
descriptor (`Cannot have multipath descriptor while also specifying 'internal'`
[verified: v29 source backup.cpp:1478]); the second multipath element is
imported as the internal (change) descriptor automatically [verified: empirical
— 2 descriptors stored, `internal: true` on the `/1/*` one, and
`getrawchangeaddress` works]; `label` — refused for ranged descriptors
(`Ranged descriptors should not have a label` [verified: empirical v29]).

Run against the REAL wallet, step 2 answers:
`success: false … is not sane: witnesses without signature exist`
[verified: empirical v29 and v31.1, both wrappers, checksummed input].

Three traps found by running it, all constellation-relevant:

1. **`importdescriptors` requires the checksum.** A bare descriptor gets
   `Missing checksum` [verified: empirical v29]. `getdescriptorinfo` normally
   supplies it — but it refuses this wallet's descriptor entirely, so for
   anything near the sanity boundary the constellation must compute BIP-380
   checksums itself. It already can: `md descriptor` output ends `#checksum`
   and Core confirms the value on the byte-identical string (`74d2zf4u` both
   sides) [verified: empirical], and `mnemonic export-wallet` emits
   checksummed descriptors [verified: output inspected].
2. **Never round-trip a multipath descriptor through `getdescriptorinfo`'s
   `descriptor` field** — that field silently collapses `<0;1>` to the first
   element, and importing it yields a wallet with NO change descriptor and a
   broken `getrawchangeaddress` [verified: empirical v29 AND v31.1 — the trap
   reproduces on current Core]. Append the checksum to the original string
   instead (the `checksum` field is computed over the input as written).
3. **The `addr()` fallback works today**: `getdescriptorinfo
   "addr(bc1puvyd…gyny)"` → checksum, then a non-ranged `importdescriptors`
   entry with `timestamp` and optional `label` → `success: true` [verified:
   empirical v29, using the fixture's real first receive address].

### 3.2 The watch-only export that exists today (`mnemonic export-wallet`)

Correction acknowledged: mnemonic-toolkit v0.97.0 ships `mnemonic
export-wallet`, default format `bitcoin-core`. Re-verified by running it against
the concrete rcw descriptors
(`/scratch/code/shibboleth/mnemonic-engrave/design/journeys/out/rcw/{tr,wsh}/descriptor.txt`):

- **wsh: exports, exit 0** — a 2-entry JSON array (receive `internal: false`,
  change `internal: true`, both `active: true`, `range: [0,999]`,
  `timestamp: 0`, checksummed, multipath pre-split into `/0/*` and `/1/*`)
  [verified: ran it, inspected JSON]. The array is shape-perfect: it is
  directly usable as the `importdescriptors` argument [verified: fed verbatim
  to v29 — Core parsed the request fine and rejected each entry only on the
  sanity rule]. The pre-split single-path form is also what makes the output
  valid all the way down to v24/v25, sidestepping the v29 multipath floor.
- **tr: refuses, exit 2** — `error: export-wallet --descriptor: All spend paths
  must require a signature` [verified: ran it]. The refusal happens at
  `MsDescriptor::<DescriptorPublicKey>::from_str` in
  `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/export_wallet.rs`
  (the parse at ~line 525) [verified: source read]. There is no
  `--experimental`/`--allow` flag on export-wallet [verified: full --help
  scanned, zero matches].
- So the asymmetry is real but **irrelevant to the outcome**: the wsh export
  that "succeeds" produces an artifact every Core release refuses. rust-
  miniscript happens to run the sigless-branch analysis on `tr()` parse and not
  on `wsh()` parse; Core runs it on both at import.
- Side observation: the rcw `descriptor.txt` keys (and `md descriptor` output)
  serialize account-level key material with zeroed BIP32 headers (depth 0,
  parent fingerprint 00000000, child 0 — `xpub661MyMwAqRbc…` prefix) behind a
  4-step hardened origin. The chaincode+pubkey bytes are identical to the
  properly-headed account xpubs in `keys.txt` [verified: base58-decoded and
  compared], and Core parses them without complaint (the import error is the
  sanity one, i.e. key parsing succeeded) [verified: empirical]. Other stacks
  may warn on the depth/origin mismatch [unverified: reasoning].

### 3.3 Hot wallet — what the descriptor must carry

No hot export exists anywhere in the constellation: `export-wallet` refuses
secret slot subkeys by SPEC ("watch-only by definition") [verified: --help
text], and `ms derive` emits only fingerprints/xpubs, no xprv [verified:
--help]. For the real 4-tier wallet a hot export would be pointless anyway
(§1). For any Core-loadable variant, the requirements were established
empirically on the truncated tr form:

1. **Per key: the ACCOUNT-LEVEL xprv with the origin prefix**, i.e. exactly the
   watch-only form with `xpub` swapped for the account `xprv`:
   `[39ec1b6e/270028'/0'/0'/0']xprvAcct…/<0;1>/*`. Import into a wallet created
   with `disable_private_keys=false, blank=true` → `success: true`,
   `getnewaddress`/`getrawchangeaddress` work, and the first three receive
   addresses are byte-identical to the watch-only import's [verified: empirical
   v29 — the address match also independently confirms the fixture seeds ↔
   xpubs correspondence].
2. **NOT the master xprv with the hardened path inside the descriptor.**
   `xprvMaster/270028'/0'/0'/0'/<0;1>/*` keys are REFUSED whenever two or more
   of them are siblings under a wrapper: `…is not sane: contains duplicate
   public keys` — a Core false positive. `PubkeyProvider::operator<`
   (descriptor.cpp v29, lines 173–185) compares keys by deriving pubkey 0 with
   a dummy SigningProvider; hardened steps make both derivations fail, both
   `CPubKey`s stay default-constructed, and every such pair compares equal
   [verified: source read + minimal repro — two distinct xprvs, same hardened
   path: bare `multi_a` passes, `and_v(v:older(100),multi_a(…))` fails;
   reproduces identically on v29.0 and v31.1]. Account-level xprvs dodge this
   because the remaining path is unhardened and publicly derivable.
3. Expect and whitelist the warning `Not all private keys provided…` on tr
   imports whose internal key is the NUMS point — that key has no private half
   by construction [verified: empirical v29].
4. Security consequence, two sentences: an account xprv in an export file is
   the spending key for every address of that account, and for this vault six
   of them in one artifact is the whole wallet minus the hash preimages;
   tier-1/2 hashlocks then gate nothing against an attacker holding the file
   plus preimage knowledge. Hot export, if ever built, must be an explicit
   secret-materializing surface like `mnemonic bundle`, not a flag on
   `export-wallet`.
5. Even hot, Core never stores hash preimages. Satisfying tier 1/2 requires an
   external party to inject the preimage into the PSBT
   (`sha256_preimages` fields exist in Core's PSBT model ≤ v25 [verified: help
   decodepsbt]); `finalizepsbt` claims miniscript finalization for wsh (v25,
   #24149) and taproot leaves (v26, #27255) [verified: release notes]. An
   actual funded hash-branch spend through a Core wallet was NOT exercised
   here [unverified — flagged as open question 3].

## 4. What must the constellation compute that it does not today

Re-aimed at extending `mnemonic export-wallet` (per the mid-flight correction),
in descending order of value:

1. **An `addr()`-list Bitcoin Core format** (e.g. `--format
   bitcoin-core-addresses`): N non-ranged `addr(…)#checksum` entries with
   `timestamp` and a labeling scheme, derived via the constellation's own
   address derivation (`md address` exists; the toolkit has
   `derive_address.rs`). This is the ONLY Core-loadable watch artifact for the
   real wallet [verified: the addr() import path works, §3.1]. It must state
   its own limits in the emitted file: no solving, fixed window, top-up
   required.
2. **A typed, symmetric refusal for Core-insane descriptors.** Today tr fails
   with a bare rust-miniscript message and wsh "succeeds" into an artifact Core
   rejects. The emitter should detect the sigless branch itself (the toolkit
   already has exactly this taxonomy: `CliAllow::SiglessBranch` /
   `DiagnosticKind::SiglessBranch` in `cmd/build_descriptor.rs:133` [verified:
   source]) and refuse BOTH wrappers for `--format bitcoin-core` with "Bitcoin
   Core (≤ 31.1) cannot import descriptors with a signatureless spend path;
   use bitcoin-core-addresses". An `--allow sigless-branch` waiver alone is a
   footgun here — it would emit artifacts that fail downstream.
3. **Version-floor metadata.** The `--bitcoin-core-version` knob accepts only
   24|25 and is currently dead in the emitter
   (`_bitcoin_core_version: u8`, underscore-unused, `wallet_export/
   bitcoin_core.rs:46` [verified: source]). Either wire it (v29+ could emit
   the native multipath single entry; ≤28 keeps the split form — the split
   form the emitter already produces is valid on every version, so this is
   polish) or drop it; and the docs should carry the v24/v25/v26/v29 floor
   table from §2, since the v29 multipath floor is absent from Core's own
   release notes.
4. **Hot export does not exist and stays a separate decision** (§3.3). If
   built: account-level xprvs only (the Core duplicate-key false positive with
   ≥2 hardened-path key expressions is present through v31.1), same JSON
   shape, `disable_private_keys=false` instructions, and the NUMS warning
   documented. `ms derive` would need an xprv-emitting (secret) counterpart —
   nothing emits xprvs anywhere today [verified: ms/mnemonic help surfaces].
5. Nothing to build for checksums — `md descriptor` and `export-wallet` both
   already compute Core-agreeing BIP-380 checksums [verified: §3.1]. Keep the
   "never round-trip through getdescriptorinfo" rule (§3.1 trap 2) in whatever
   operator doc accompanies the export.

## 5. Open questions (could not confirm)

1. **v24.0.1 behavior was not run** (no binary fetched). Claims about v24 rest
   on release notes and on the sanity code being part of #24148 itself
   [unverified: reasoning]. Irrelevant to the verdict — v25→v31.1 empirics
   bracket everything that matters.
2. **v27.0/v28.0 keyless rejection not individually exercised** (their parsers
   refuse the multipath form first; single-path keyless was tested on v25, v26,
   v29, v31.1 and refused identically). The v27/28 gap is bounded on both
   sides [unverified for those two binaries specifically: interpolation].
3. **No end-to-end spend of a hash-preimage branch from a Core hot wallet**
   (needs a funded regtest chain and an external preimage-injecting tool; not
   built this session). Which constellation tool owns preimage injection into
   the PSBT is an unanswered design question.
4. **No end-to-end timelock spend** (tier 2/3) — acceptance and solvability
   flags only.
5. **Whether upstream Core will ever allow insane-miniscript imports** — no
   flag exists through v31.1 [verified: absence]; future policy unknown.
6. The v25 node is the Bitcoin Satellite build reporting Core v25.0.0; its
   descriptor code is assumed unpatched from upstream v25 [unverified:
   reasoning — all its results were consistent with the official-binary
   neighbors].
7. Coordinator's sibling claim that concrete keyed descriptors live at
   `design/journeys/out/rcw/{tr,wsh}/descriptor.txt` — confirmed and used;
   note their accounts are `8'/0'` (tr) and `9'/1'` (wsh), while the
   `design/fixtures/reasonably-complex-wallet/` policies use account `0'`.
   Same shape, different instantiations; both were exercised (fixture policies
   against Core via mechanical key substitution, rcw descriptors against
   export-wallet + Core).

## Appendix: raw error strings for grepping

- `Missing checksum` — importdescriptors/deriveaddresses without `#…` (v29)
- `is not sane: witnesses without signature exist` — the tier-4 blocker (v25 wsh, v26 tr, v29/v31.1 both, scantxoutset v29)
- `is not sane: contains duplicate public keys` — the hardened-path-xprv false positive (v29, v31.1)
- `Key path value '<0;1>' is not a valid uint32` — pre-v29 multipath (v25–v28)
- `A function is needed within P2WSH` — pre-v29 wsh-miniscript-with-multipath parse failure (v25–v28)
- `'and_v(…multi_a(…)…)' is not a valid descriptor function` — pre-v26 tr miniscript (v25)
- `Cannot have multipath descriptor while also specifying 'internal'` (v29 source)
- `Ranged descriptors should not have a label` (v29)
- `error: export-wallet --descriptor: All spend paths must require a signature` — mnemonic-toolkit v0.97.0, tr form
