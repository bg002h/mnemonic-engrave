# Coordinator-compat: the Core tapscript-miniscript boundary release, measured

Closes the item "The Core boundary release is unmeasured" in
`design/DESIGN_coordinator_compatibility.md` (section "Open, and blocking a spec").
Measured 2026-09-23 against official bitcoincore.org release binaries (x86_64-linux-gnu).

## Finding

- **Boundary release: Bitcoin Core 26.0.** Every release from 24.2 through 25.2 refuses the
  representative `tr` + tapscript-miniscript descriptor with `Miniscript expressions can only be
  used in wsh`. Every release from 26.0 through 31.1 imports it, and receive/change 0..2 equal md's
  addresses. No release exists between 25.2 and 26.0 (bitcoincore.org lists 25.0, 25.1, 25.2, then 26.0).
- **F-449 kind-1 (Liana-unspendable internal key) on a release Core:** the first release that
  imports it is **26.0**. On every release from 26.0 through 31.1, receive 0..2 and change 0..2
  equal Liana v15.0's (commit `4684d5cb`). This replaces the v30.99 dev-build-only evidence. It
  refuses on 24.2 through 25.2 for the same miniscript reason, because its recovery leaf is
  `and_v(v:pk(..),older(26280))`.
- **The "one capability" claim holds across releases.** All 15 discriminating `tr` shapes flip
  together, at the same release: 0 of 15 accept on 24.2, 25.0 and 25.2, and 15 of 15 accept with md's
  chain-0 addresses on 26.0 and later. The `wsh` control is accepted with md's addresses on all 10 releases.

For the Core RuleSet, the span for "policy puts miniscript under `tr`" is **>= 26.0**: refused on
<= 25.2, accepted on 26.0 through 31.1.

## Table

Binary sha256 is the sha256 of the release tarball `bitcoin-<v>-x86_64-linux-gnu.tar.gz`, and each
one was checked with `sha256sum -c SHA256SUMS`. Verification = tarball sha256 matches
`SHA256SUMS`, and `SHA256SUMS.asc` was checked with gpg against the builder keys in
`bitcoin-core/guix.sigs` (`builder-keys/*.gpg`, 38 keys, clone HEAD
`6caff04f125e2c5c203fa8b343110a7c6cf06092`, imported into a throwaway GNUPGHOME). The count is
GOODSIG signatures, and "exp" counts EXPKEYSIG (a valid signature by a key that has since expired).
There were **zero BADSIG and zero REVKEYSIG on any release**. The keys were not cross-checked
against any web of trust, so they are trusted as published in guix.sigs.

| release | tarball sha256 | verification | representative verdict | addresses match md (recv+chg 0..2, derive + wallet) | kind-1 verdict (addresses == Liana v15.0) |
|---|---|---|---|---|---|
| 24.2 | `7540d6e34c311e355af2fd76e5eee853b76c291978d6b5ebb555c7877e9de38d` | sha256 OK; 10 GOODSIG, 1 exp | REFUSE: `Miniscript expressions can only be used in wsh` | n/a | REFUSE (same message) |
| 25.0 | `33930d432593e49d58a9bff4c30078823e9af5d98594d2935862788ce8a20aec` | sha256 OK; 11 GOODSIG, 2 exp | REFUSE (same) | n/a | REFUSE (same) |
| 25.2 | `8d8c387e597e0edfc256f0bbace1dac3ad1ebf4a3c06da3e2975fda333817dea` | sha256 OK; 6 GOODSIG, 1 exp | REFUSE (same) | n/a | REFUSE (same) |
| **26.0** | `23e5ab226d9e01ffaadef5ffabe8868d0db23db952b90b0593652993680bb8ab` | sha256 OK; 9 GOODSIG | **ACCEPT** | yes | **ACCEPT, match** |
| 26.2 | `77c63bec845b318c07f3a7660c579f63da18b58d25699ab8b8df6034e8ed55c0` | sha256 OK; 9 GOODSIG, 1 exp | ACCEPT | yes | ACCEPT, match |
| 27.2 | `acc223af46c178064c132b235392476f66d486453ddbd6bca6f1f8411547da78` | sha256 OK; 9 GOODSIG, 2 exp | ACCEPT | yes | ACCEPT, match |
| 28.4 | `661c02443433c006a52bd3e688c1bc14a37fb95b964f40dfe7b1dc4ffcac2926` | sha256 OK; 11 GOODSIG | ACCEPT | yes | ACCEPT, match |
| 29.4 | `e15bff6f6d21a315c4af25d2e8ae933a22bd51e924e0e90ab0474e1e11516331` | sha256 OK; 13 GOODSIG | ACCEPT | yes | ACCEPT, match |
| 30.3 | `4753cd70416c629a2090503e8545a3c05e94a0f5037f8fb97b4c35c6e1afee0e` | sha256 OK; 11 GOODSIG | ACCEPT | yes | ACCEPT, match |
| 31.1 (current release) | `b80d9c3e04da78fb6f0569685673418cf686fadba9042d926d13fb87ff503f9e` | sha256 OK; 11 GOODSIG | ACCEPT | yes | ACCEPT, match |

Secondary results from the same runs:

| release | 15 discriminating tr shapes (chain 0, getdescriptorinfo + deriveaddresses == md) | wsh control `preset-kofn-recovery-wsh` | multipath `<0;1>` in getdescriptorinfo |
|---|---|---|---|
| 24.2, 25.0, 25.2 | 0 / 15. Twelve refuse with `Miniscript expressions can only be used in wsh`, and three with an `'and_v(v:multi_a(...` parse failure | ACCEPT, addresses == md | refused: `Key path value '<0;1>' is not a valid uint32` |
| 26.0, 26.2, 27.2, 28.4 | 15 / 15 | ACCEPT, addresses == md | refused (same message) |
| 29.4, 30.3, 31.1 | 15 / 15 | ACCEPT, addresses == md | ok |

Multipath is a **separate** capability with its own boundary, which falls between 28.4 and 29.4 and
was not bracketed tighter than that. It does not affect the verdict above, because all wallet imports
used the per-chain `/0/*` and `/1/*` descriptors. It does matter for any consent text that tells a
Core 26 to 28 user to paste a `<0;1>` descriptor: those releases refuse the multipath string even
though they accept the policy.

## Representative descriptor and why

`preset-kofn-recovery-tr` from `design/evidence/composer-fable-r0/fable-liana-shapes.json` is one
of the 15 discriminating shapes, with NUMS internal key `50929b74...` and the leaves
`multi_a(2,@0,@1,@2)` and `and_v(v:pk(@3),older(26280))`. Its expectations are that record's
`md_addr` (md's own receive and change 0..2). I picked it because its template is identical to the
F-449 kind-1 case except for the internal key. The pair therefore isolates "tapscript miniscript"
from "which unspendable key". Running all 15 shapes as well confirms the single-capability reading
directly, without relying on the choice of representative.

Kind-1 = `CONTROL-accept-kofn-recovery-flat`, line 2 of
`design/evidence/f449-stage2/liana-live-gate-expected.jsonl` (line 1 records `liana_tag: v15.0`,
`liana_commit: 4684d5cb0c75471ae40f43dffc78333ac74afb38`). The receive descriptor is its
`receive_desc`, and the change descriptor is its `liana_desc` with `<0;1>` replaced by `1`. The
expectations are its `receive` and `change` arrays. I used this evidence rather than a fresh
`md compose --preset kofn-recovery --unspendable liana` run, because `md compose` (md 0.19.0) emits
a key-less template (`tr(UNSPENDABLE(liana),...)`), and filling in keys would have been a
second, unrecorded construction.

## Network (the regtest trap)

I did not use regtest. Every descriptor here carries **mainnet** xpubs with `bc1p` expectations,
and a regtest node rejects mainnet version bytes, which would have been a false refusal. Each node
is an **offline mainnet** node (`-connect=0 -listen=0 -dnsseed=0`, throwaway datadir), and
`getblockchaininfo.chain == "main"` is recorded in each result JSON. This is the same setup the
F-449 stage-4 script used. Two checks show that the refusals on 24.2 to 25.2 come from the
miniscript capability and not from the network. First, the error text is the miniscript refusal,
not a key-parse error. Second, the `wsh` control uses the same mainnet xpubs, and those same nodes
accept it with md's addresses.

## Exact commands

```sh
# fetch + verify (per release v in 24.2 25.0 25.2 26.0 26.2 27.2 28.4 29.4 30.3 31.1)
R=/scratch/code/shibboleth/.tmp/core-releases; mkdir -p $R/$v; cd $R/$v
curl -sfO https://bitcoincore.org/bin/bitcoin-core-$v/SHA256SUMS
curl -sfO https://bitcoincore.org/bin/bitcoin-core-$v/SHA256SUMS.asc
curl -sfO https://bitcoincore.org/bin/bitcoin-core-$v/bitcoin-$v-x86_64-linux-gnu.tar.gz
sha256sum -c --ignore-missing SHA256SUMS
git clone --depth 1 https://github.com/bitcoin-core/guix.sigs $R/guix.sigs     # HEAD 6caff04f
export GNUPGHOME=$R/gnupg; gpg --import $R/guix.sigs/builder-keys/*.gpg
gpg --verify --status-fd 1 SHA256SUMS.asc SHA256SUMS | grep -E 'GOODSIG|BADSIG|EXPKEYSIG|REVKEYSIG'
tar xzf bitcoin-$v-x86_64-linux-gnu.tar.gz

# probe (from the mnemonic-engrave root; the ten releases run in parallel on distinct rpc ports)
python3 design/evidence/coord-compat-core-boundary/probe.py $R/$v <rpcport> \
  > design/evidence/coord-compat-core-boundary/core-$v.json
```

Per descriptor, `probe.py` calls `getdescriptorinfo` on the multipath form and on each chain, then
`deriveaddresses <desc> [0,2]` on each chain. It then runs
`createwallet disable_private_keys=true blank=true descriptors=true`, and `importdescriptors` with
chain 0 (active, `internal:false`) and chain 1 (active, `internal:true`), both `range [0,5]`,
`timestamp now`. Finally it calls `getnewaddress "" bech32m` and `getrawchangeaddress bech32m`
three times each. "Addresses match" requires **all four** lists (derive recv, derive chg, wallet
recv, wallet chg) to equal the expectation exactly. Raw per-release results, including the full
error text and address lists, are in `design/evidence/coord-compat-core-boundary/core-<v>.json`.
Binaries and datadirs live in `/scratch/code/shibboleth/.tmp/core-releases/`. Nothing was
installed system-wide, and nothing is committed.

## Not measured

- **Point releases between the ones listed:** 24.0, 24.0.1, 24.1, 25.1, 26.1, 27.0, 27.1, 28.0 to 28.3,
  29.0 to 29.3, 30.2 and 31.0. The boundary itself is tight, because 25.2 is the last release before
  26.0. That later point releases behave monotonically is inferred from the major-series
  measurements, not measured.
- **Release candidates, and 32.0**, which exists only as `test.rc2` on bitcoincore.org and is not a release.
- **Any release before 24.2.** It is not needed for the boundary (24.2 already refuses).
- **Spending.** Only import and address derivation were tested. No PSBT, signing, or
  `walletprocesspsbt` was run on any release, so "imports" does not mean "can spend the
  miniscript leaf". That remains a separate question for the RuleSet's wording.
- **Multipath import via `importdescriptors`:** multipath support was probed with `getdescriptorinfo`
  only, and its boundary is bracketed only to between 28.4 and 29.4.
- **The local "Core 25.0"** used in `IMPORTABILITY_composer_shapes.md` identifies itself as
  `Bitcoin Satellite v0.2.4`, not Core. It was not re-run. Its recorded refusals do agree with the
  genuine 25.0 and 25.2 release binaries: the same `Miniscript expressions can only be used in wsh`
  message, and the same three `multi_a` parse failures.
- **Platforms other than x86_64-linux-gnu, and the Core GUI** (only `bitcoind`/RPC was used).
- **The signing-key trust anchor.** The guix.sigs keys were used as published and not checked
  against a web of trust.
