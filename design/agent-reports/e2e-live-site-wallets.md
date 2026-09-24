# E2E: wallets composed on the LIVE SH2 demo, checked against the wallets they target

Date 2026-09-23. Author: e2e agent. Nothing was committed and no product code changed.
Raw evidence is in `design/evidence/e2e-live-site-wallets/` (`live/` and `regtest/`).

## Verdict

- **Device = md, byte for byte, on all 8 live wallets.** That covers 4 addresses per wallet, the Template-ID, the Policy-ID, and the decoded template, which equals `md compose --preset` for all four preset × kind pairs.
- **Bitcoin Core 29.4 and 31.1 release binaries import all 8 wallets** (mainnet, offline). The wallet-derived receive/change addresses equal the device's.
- **Liana v15.0 accepts both Liana-key wallets and refuses all four NUMS wallets, as the copy says, but only when the operator seats distinct signers.** Its addresses equal the device's. The simplest seating a demo visitor can do (seat the one payload seed into every slot) produces wallets **Liana refuses, even with the Liana key**, because one signer appears twice in a path. The device does warn about this on the Key-mapping screen ("Liana will refuse it"). The consent screen still prints "Liana (as of v15.0) and Bitcoin Core import this form" a few screens later (D-1).
- **Spend test (regtest, Core 31.1, same shapes, keys we control):**
  - The primary path spends and confirms on all 4 shapes.
  - Recovery is rejected with `non-BIP68-final` immediately and again one block before maturity. It is accepted and confirmed at the first block allowed, on all 4.
  - The key path has no private key in any wallet. The internal key point is BIP-341's H, both as a raw NUMS key and as a Liana xpub.
- One tool discrepancy outside the device: `md descriptor --network regtest` writes leaf keys as mainnet `xpub` next to a `tpub` internal key, and Core regtest refuses the result (D-2).

## Table

"demo" is the one payload seed (abandon…about, 73c5da0a) seated into every slot, which is the default path.
"distinct" is the payload seed plus typed seeds B (legal…yellow, b8688df1) and C (letter…above, 28645006). The seating is kofn @0=A @1=B @2=C @3=A(acct 1'), and tiered @0=A @1=B @2=A(1') @3=B(1').
All live wallets are tr with older(26280). kofn-recovery is `{multi_a(2,@0,@1,@2), and_v(v:pk(@3),older)}`. tiered-recovery is `{multi_a(2,@0,@1), and_v(v:multi_a(1,@2,@3),older)}`.

| preset | kind | seating | device = md (4 addrs, T-ID, P-ID) | Liana v15.0 | Core 29.4 import / addrs | Core 31.1 import / addrs |
|---|---|---|---|---|---|---|
| kofn | NUMS | demo | YES | REFUSE (generic "not compatible") | OK / = device | OK / = device |
| kofn | Liana | demo | YES | **REFUSE**: "derived from the same origin as another key present in the same spending path" | OK / = device | OK / = device |
| tiered | NUMS | demo | YES | REFUSE (generic) | OK / = device | OK / = device |
| tiered | Liana | demo | YES | **REFUSE** (same-origin, as above) | OK / = device | OK / = device |
| kofn | NUMS | distinct | YES | REFUSE (generic) | OK / = device | OK / = device |
| kofn | Liana | distinct | YES | **ACCEPT**, recv/chg 0-1 = device | OK / = device | OK / = device |
| tiered | NUMS | distinct | YES | REFUSE (generic) | OK / = device | OK / = device |
| tiered | Liana | distinct | YES | **ACCEPT**, recv/chg 0-1 = device | OK / = device | OK / = device |

Spend test (regtest, Core 31.1 release). Keys are A/B/C at m/48'/1'/k'/3', seated like the "distinct" rows. Descriptors come from `md compose` + `md descriptor` with leaf keys re-serialised as tpub/tprv (see D-2). The UTXO was funded at height 111 and older = 26280.

| shape | primary spend (wallet holds @0,@1 only) | recovery at tip 112 | recovery at tip 26389 (one short) | recovery at tip 26390 | key path |
|---|---|---|---|---|---|
| kofn / NUMS | `5596756759a0d58d287ddbacd92c68fa0bba0a7ecd5382ee790989d190392a5c`, multi_a leaf, 1 conf | rejected non-BIP68-final | rejected non-BIP68-final | accepted `6056e294370ebcca7ad5b3ae67d6f5518a89c77aef2babe2f882e6dccef9bbb9`, 1 conf | unspendable (below) |
| kofn / Liana | `52fa88f2d9e84755493e928254387527a6c338f041b9e6a8f197c226729e164a`, 1 conf | rejected | rejected | accepted `f6d9c4caf94856325612ee1137b03230c6d0b1f7ebe02b12e541a04eee845659`, 1 conf | unspendable |
| tiered / NUMS | `27781e07a546b81c057947ff299d9f31a2476c0c96ed39172c46c8d662f01f0b`, 1 conf | rejected | rejected | accepted `150cb7aa9fae4d9f826d05065707ba74cfee7ba2ece603c675c99dcb4be1eea8`, 1 conf | unspendable |
| tiered / Liana | `eabc04f586ccfca1cce046f9fe7e64bc854b2103cb7843e30b07b63b8bba0b74`, 1 conf | rejected | rejected | accepted `8e5102e7e65acc56e26bab0e5411c7788af3f36c9dfcadc3ad6bb19ba57b3028`, 1 conf | unspendable |

About the spend rows:

- The recovery wallet holds only the @3 key (kofn) or the @2 key (tiered), so only the recovery leaf is signable. Recovery txs are version 2 with nSequence 26280. The same signed hex was broadcast at each tip.
- Every spend's witness is a script-path witness: the stack ends with a leaf script plus a control block with prefix c0/c1. Every primary leaf contains OP_CHECKSIGADD and no CSV. Every recovery leaf contains `26280 OP_CHECKSEQUENCEVERIFY`.
- Liana v15.0 also ACCEPTS the two regtest Liana-key descriptors and REFUSES the two NUMS ones (`regtest/liana-regtest-out.jsonl`), so the spend-tested wallets are the ones Liana would hold.

**How the key path was established as unspendable (c).**

- BIP-341's H has x = sha256(uncompressed G) = `50929b74…3ac0`. We recomputed this in `keypath-check.txt`.
- The NUMS internal key is that exact x-only value. The Liana internal key is a depth-0 xpub whose point is `0250929b…3ac0`, which is H. Only its chain code (sha256 of the leaf keys) differs, so every derived internal key is H + t·G.
- The discrete log of H is unknown by construction, and so is the discrete log of H + t·G. `listdescriptors true` on every private wallet shows the internal key still public (no tprv).
- Every confirmed spend used the script path.
- This rests on construction plus observation. We did not attempt a key-path signature, which is impossible without a key.
- The live mainnet Liana-key descriptors carry the same H point (`live/live-keypath-check.txt`).

## Discrepancies

**D-1 (copy vs outcome, demo seating).** A visitor who seats the loaded payload seed into every slot, which is the only seed offered without typing, gets a "Liana key" wallet that Liana v15.0 refuses:

`Key '[73c5da0a/48'/0'/1'/3']xpub6DXu…' is derived from the same origin as another key present in the same spending path.`

The device is not silent about this. The Key-mapping page 2 reads "SAME SEED, SAME PATH … Slots @0, @1 and @2 are the same seed … Liana will refuse it." But the consent screen that follows still says "Liana (as of v15.0) and Bitcoin Core import this form", and so does the key-path chooser. `md descriptor` agrees with Liana ("Liana 8.0-15.0: refuses (one signer twice in a path)"). Evidence: `live/live-kofn-liana.json` (`mapping`, `consent`) and `live/liana-verdicts.txt`. This is a copy-accuracy / sequencing question, not a wrong address.

**D-2 (md, non-mainnet only).** `md descriptor --template … --key @i=[fp/48'/1'/k'/3']tpub… --network regtest` emits every leaf as `xpub6DXu…` (mainnet version bytes, parent fp 00000000) while the Liana internal key comes out as `tpubD6Nz…`. Core 31.1 regtest rejects this with `Multi: key 'xpub6DXuQW1Q2Jpa1DM9…' is not valid`.

- The key and chain code are correct. `regtest/mkdesc.py` asserts they are byte-equal before substituting the tpub.
- md's Liana key over those leaves equals Liana's own recipe (the probe rule held).
- This does not touch the device or mainnet. It does mean md's `--network` output for testnet/regtest is not directly importable.

**D-3 (registry gap, not a defect).** `md descriptor` prints "Bitcoin Core 29.4-31.1: unproven (not measured for this policy)" for all 8 descriptors. This run measured import OK plus matching addresses on 29.4 and 31.1 for all 8.

**Not discrepancies (checked):**

- The NUMS refusals carry Liana's generic message. They are attributable to the key: the Liana-key twin with identical leaves is accepted, and the probe rule held for every Liana-key probe.
- The gate's two controls behaved: the accept control was accepted and the stale-key control refused.
- Core warns "Range not given" and, on the regtest partial-key wallets, "Not all private keys provided". Both are expected.

## Exact commands

Live capture: Playwright on the **live page** `https://quantoshi.xyz/SH2/emu/index.html`, not a local copy.

- Helpers were injected via `add_script_tag`: `live/helpers.js` and `live/drive.py`.
- Steps are in `live/boot.js` + `live/full.tmpl.js` → `live/run-<wallet>.js`:
  1. Boot offer LOAD, then digest `55adb800…`, then KEEP.
  2. Wallet Policy → Build a new policy → Taproot (tr) → preset row 3 (kofn-recovery) or row 4 (tiered-recovery) → Done.
  3. Key path row 0 (NUMS) or row 1 (Liana key).
  4. Seat keys → Type a seed → FROM PAYLOAD → skip passphrase.
  5. Each slot takes seed N. Extra seeds are typed on the BIP-39 keyboard (TYPE IT, 12 words).
  6. Mapping → stub → Review (all pages read) → hold → "The policy itself" → Watch-only → engrave tail. The md1 strings are taken from `shToolpath.strings()`.
- The live emu files' sha256 equals the fork's local `cmd/emu/emu.wasm` (`live/live-site-sha256.txt`: emu.wasm `78de4fab…`).

```
python3 drive.py run-kofn-liana.js > live-kofn-liana.json        # x8
./pipeline.sh     # md decode --in F; md decode --json; md descriptor $(cat F); md address $(cat F) --count 2; md address … --change --count 2; diff vs device
md inspect $(cat F)                                               # ids.txt
./liana.sh        # harness copy built against .tmp/fable-liana-src-v15/liana (v15.0, 4684d5cb0c75471ae40f43dffc78333ac74afb38); `unspendable` probe rule, then `parse`
./core-mainnet.sh # bitcoind -chain=main -connect=0 -listen=0 -dnsseed=0 -fixedseeds=0; createwallet disable_private_keys blank; importdescriptors [{desc,timestamp:"now",active:true}]; getnewaddress "" bech32m x2; getrawchangeaddress bech32m x2; deriveaddresses desc [0,1]
# regtest
python3 bip32.py test m/48h/1h/<k>h/3h <mnemonic>                 # keys.txt; self-checked vs the device's own mainnet key for A at 48'/0'/0'/3'
md compose --wrapper tr --preset kofn-recovery,2of3,older=26280 [--unspendable liana] --json
md compose --wrapper tr --preset tiered-recovery,2of2,1of2,older=26280 [--unspendable liana] --json
md descriptor --template "<template>" --key @i=[fp/path]tpub… --network regtest
python3 mkdesc.py <md-descriptor> <private-origins>               # xpub->tpub/tprv (D-2)
python3 spend.py   # Core 31.1 regtest: walletcreatefundedpsbt (sequence 26280 for recovery), walletprocesspsbt, finalizepsbt, sendrawtransaction at tips 112 / 26389 / 26390
```

md 0.20.0. Core tarballs were re-checked against SHA256SUMS (OK). Liana checkout: tag v15.0, clean.

## Not tested

- Nunchuk (out of scope by brief).
- The "Template plus key cards" form (B), and Full mode. Only form A, watch-only, was engraved.
- Liana's own signer, and Liana-driven spends. Spends went through Core only; Liana was used for import/addresses.
- Spending on the demo-seated (same-seed) shapes. Spends used the distinct-signer seating.
- Mainnet funds.
- Core versions other than 29.4 and 31.1 (the brief asked for 29.x and 31.1).
- Recovery spends from a watch-only Core wallet plus an external signer.
- Keys typed with a BIP-39 passphrase. The payload's passphrase record was skipped.
- Other presets, and the wsh wrapper.
