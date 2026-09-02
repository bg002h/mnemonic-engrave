# Recon — key-origin convention for the composer's seed-derived `tr()` slots

**Question (single):** what key-origin (derivation path) convention should the composer
declare for slots it derives FROM A SEED in a `tr()` multi-path wallet policy, and is
there any standard or de-facto convention for taproot multisig / miniscript key origins
at all?

**Agent:** recon, opus tier. Dispatched from `BRAINSTORM_wallet_policy_composer.md` §2 C27
item (1). Read-only; this file is the only thing written.

**Heads measured against:** mnemonic-engrave `83aad9d`; fork `169073c`; md 0.14.0
(`/home/bcg/.cargo/bin/md`); ms (installed binary, `/home/bcg/.cargo/bin/ms`) + repo
`mnemonic-secret`; mnemonic-toolkit working tree. External heads pinned inline.

**Marking:** every claim is tagged **MEASURED** (I ran it here), **SOURCED** (quoted from
authoritative text with file:line or URL), or **UNVERIFIED**.

---

## 1. Recommendation (one paragraph)

**Adopt BIP-48 with a new script_type `3'` — `m/48'/coin'/account'/3'` — for the
composer's seed-derived `tr()` slots, and keep wsh on `m/48'/coin'/account'/2'`.**
There is **no standard** for taproot multisig origins to violate: BIP-48 registers only
`1'` and `2'` and mentions taproot zero times, BIP-86 is titled *Key Derivation for
**Single Key** P2TR Outputs*, and the BIPs PR that proposed exactly `3'` for taproot was
**closed unmerged** in 2024 with Sparrow's author stating on the thread *"We don't have
any derivation path standards here I'm aware of"* (all SOURCED, §4). In that vacuum `3'`
is the strongest candidate on evidence rather than taste: it is the only value with a
**shipping hardware signer that exports it** (Coldcard Edge, `bip48_3`, marked
`is_ms=True`), the only one with a **written spec diff** (the closed PR's own text), and
the only one **already implemented inside this constellation** (`mnemonic-toolkit`'s
`bip48-tr-multi-a`); and because the script_type sits at level 4, a seed's `tr` key and
its `wsh` key are structurally disjoint at the same account — the exact hazard BIP-87 §Account
names as *"key reuse - across ECDSA and Schnorr signatures, across different script types"*
(SOURCED, bip-0087 l.106). Every consumer I could measure **accepts** it and none warns:
BIP-388 constrains origins not at all (its own taproot vector uses `48'/0'/0'/**100'**`),
Ledger requires only that an origin be *present and self-consistent*, Nunchuk labels it
`"custom"` without rejecting, and `md` 0.14.0 encodes and round-trips it byte-clean
(MEASURED). **Two things this recommendation costs, both of which are operator decisions,
not mine:** (a) `ms derive` cannot emit it today and its doc comment at `derive.rs:102-105`
argues *against* a `bip48-p2tr` on the ground that it *"would derive to a path no other
wallet looks at"* — a sentence this recon **measurably falsifies** (Coldcard Edge exports
it; Liana's own test corpus consumes a Coldcard export carrying `"p2tr_deriv":
"m/48h/1h/0h/3h"`); and (b) it diverges from the operator's `bg002h-tr` = `m/270028'/…/0'`
ruling of 2026-08-18, whose stated justification — *"md requires depth 4 for any multisig
script context"* — is **also now stale** (md admits depth `{3,4}` since 2026-08-19, and I
encoded a depth-3 `m/87'/0'/0'` tr policy byte-clean). **Second-best, and the choice if the
priority is "a registered BIP number" over "a signer that exports it": BIP-87
`m/87'/coin'/account'`** — the only *Complete* BIP purpose-built for multisig, shipped by
Nunchuk for taproot, and already the **default** in `mnemonic-toolkit`. **wsh should stay
on `48'/…/2'`: yes** — it is the single most-agreed path in the field (BIP-48's own
"recommended default", plus Sparrow, Liana, Coldcard, Nunchuk and the shipped Multisig
Build all emit it).

---

## 2. Findings table

| Wallet / BIP | What it does for taproot multisig origins | Citation | Mark |
| --- | --- | --- | --- |
| **BIP-48** | Registers **`1'` (p2sh-p2wsh) and `2'` (p2wsh) ONLY.** *"Currently the only script types covered by this BIP are Native Segwit (p2wsh) and Nested Segwit (p2sh-p2wsh)."* Zero mentions of taproot/p2tr/schnorr. No `3'`. | `bip-0048.mediawiki` l.103-104, l.107, l.110, l.112; `grep -ci 'taproot\|p2tr\|schnorr'` = **0** | SOURCED + MEASURED |
| **BIP-86** | *"Key Derivation for **Single Key** P2TR Outputs"*. `m/86'/coin'/account'`. **Zero** mentions of multisig/multi_a. Scope is explicitly the taproot **internal key** of a single-key output. | `bip-0086.mediawiki` l.4, l.14-15, l.41-46; `grep -ci 'multisig\|multi-sig\|multi_a'` = **0** | SOURCED + MEASURED |
| **BIP-87** | `m/87'/coin'/account'`, **Status: Complete**, script-**agnostic** by design. Documents `48'`'s level-4 as *"where P2SH-P2WSH=1, P2WSH=2, **Future_Script=3**, etc"*. Says increment `account'` per wallet, which *"prevents key reuse - across ECDSA and **Schnorr** signatures, across different script types"*. **"Reference Implementation: None at the moment."** Its own examples are `wsh(sortedmulti(...))` — no `tr()` example. | `bip-0087.mediawiki` l.6, l.49, l.68, l.77, l.106, l.132, l.256 | SOURCED |
| **BIPs PR #1473** — *"BIP-48 - define P2SH, P2TR derivation paths"* | Proposed literally `3'`: *"The following path represents Taproot (p2tr) mainnet, account 0: `3'`: Taproot (p2tr) `m/48'/0'/0'/3'`"*. **CLOSED unmerged 2024-05-14.** craigraw (Sparrow): *"Wrt P2TR: Are we talking about script-based multisig, or more generally? **We don't have any derivation path standards here I'm aware of**"*. murchandamus closed it citing BIP-48's deployed status and lack of Champion endorsement. | `gh api repos/bitcoin/bips/pulls/1473` (+ `/files`, `/issues/1473/comments`) | SOURCED |
| **Coldcard (Edge / new_edge firmware)** | **Exports `bip48_3` = `m/48h/{ct}h/{acc}h/3h`, `AF_P2TR`, `is_ms=True` (multisig).** Also exports `bip86` for single-sig tr. Master branch has **neither** — no taproot at all. | `Coldcard/firmware@new_edge:shared/export.py:411,414` (head `2a3c9df2`); `@edge:shared/export.py:369,372` (head `158c8a77`); master `shared/export.py:389-390` has only `bip48_1`/`bip48_2` | SOURCED |
| **Coldcard, cross-checked by a third party** | Liana ships a real Coldcard export fixture: `{ "p2tr_deriv": "m/48h/1h/0h/3h", "p2tr_key_exp": "[C658B283/48h/1h/0h/3h]tpub…" }` — i.e. another project's test corpus consumes Coldcard's `3'` taproot multisig key. | `wizardsardine/liana:liana-gui/test_assets/ccxp-C658B283.json` | SOURCED |
| **Nunchuk (libnunchuk)** | **Implements BIP-87 for taproot multisig.** `case AddressType::TAPROOT: // Taproot Musig BIP87 Wallets: m/87h/ch/zh` under `WalletType::MULTI_SIG` **and** `WalletType::MINISCRIPT` (same switch). `GetBip32Path` is what Nunchuk actually asks a Ledger/Trezor/Tapsigner/HWI device for. A `48h/…/3h` key is labelled `"custom"` — **not rejected**. | `nunchuk-io/libnunchuk:src/utils/bip32.hpp:83-84,103-105` (TAPROOT→`m/87h/…`), l.135-139 (`3h`→`"custom"`); usages in `src/nunchukimpl.cpp`, `src/utils/trezor.cpp`, `src/utils/ledger/ledger_session.cpp` | SOURCED |
| **Sparrow (drongo)** | **Has no taproot multisig at all.** `P2TR.getAllowedPolicyTypes()` returns `List.of(SINGLE_HD, SINGLE_SP)` — MULTI_HD absent. `PolicyType` has only three members and MULTI_HD's default script type is P2WSH. Defaults: `P2WSH("…","m/48'/0'/0'/2'")`, `P2TR("…","m/86'/0'/0'")`. Taproot input construction throws `UnsupportedOperationException`. | `sparrowwallet/drongo@d6669436:…/protocol/ScriptType.java:968, 1086-1088, 1090, 1196, 1201, 1212-1214`; `…/policy/PolicyType.java:8` | SOURCED |
| **Liana** | **Requests `m/48'/{0\|1}'/{account}'/2'` unconditionally — no taproot branch.** `derivation_path()` is hard-coded to purpose 48 / script_type 2; it feeds both the hardware path (`get_extended_pubkey`) and the hot signer. `support_taproot` is used only to *refuse a device*, never to change the path. Its `tr(…multi_a…)` test corpus is full of `48'/1'/0'/2'` origins. | `wizardsardine/liana@master:liana-gui/src/installer/step/descriptor/editor/key.rs:1509-1523, 1525-1535, 519-528, 1130` | SOURCED |
| **Ledger app-bitcoin-new** | **No origin convention constraint for multisig/miniscript.** Key origin is **compulsory** and, when the fingerprint matches, must actually re-derive the xpub. "Standard" (registration-free) wallets are single-key only — `tr()` **with a tree returns -1** from `get_bip44_purpose`, so every taproot multisig must be explicitly registered, whatever its origins. No path-shape warning. | `LedgerHQ/app-bitcoin-new@develop:src/handler/register_wallet.c:112-146`; `src/handler/lib/policy.c:1386-1435` (`TOKEN_TR` + non-null tree → `-1`), `:1437-1520`; `doc/wallet.md` l.13-14, l.43-57 | SOURCED |
| **BIP-388 (wallet policies)** | **Imposes NO origin constraint.** Key origin is *"Optionally"* present with *"zero or more"* path elements. Its own **taproot** vector uses `tr([6738736c/48'/0'/0'/**100'**]xpub…/<0;1>/*,{sortedmulti_a(…)})`, and a wsh vector mixes `48'/0'/0'/100'` with `44'/0'/0'/100'`. Only *recommends* hardened distinctness: *"It is strongly recommended to avoid key reuse across accounts… This specification does not mandate hardened derivation."* | `bip-0388.mediawiki` l.68-70, l.180-187, l.282-283, l.292-293 | SOURCED |
| **mnemonic-toolkit (ours)** | Already ships **both**: `MultisigPathFamily{Bip48, Bip87}` with **`Bip87` as `#[default]`**; BIP-48 script_type `3'` is named **`bip48-tr-multi-a`** and is one of the three paths `xpub-search` sweeps. CLI: `--multisig-path-family {bip48,bip87}` (default `bip87`). | `crates/mnemonic-toolkit/src/parse.rs:62-103`; `src/cmd/xpub_search/candidate_paths.rs:8-10, 81-86, 199-203`; `crates/mnemonic-toolkit/README.md:195` | MEASURED |
| **mnemonic-secret `ms` (ours)** | `ms derive --template` offers `bip44\|bip49\|bip84\|bip86\|bip48-p2wsh\|bip48-p2sh-p2wsh\|bip48` — **no `bip48-p2tr`, no `bip87`**. Repo adds `bg002h-tr` = `m/270028'/coin'/account'/**0'**` and `bg002h-wsh` = `…/1'`. Its doc argues against a `bip48-p2tr` because *"BIP-48 registers no Taproot script type, and inventing one would derive to a path no other wallet looks at."* | `ms derive --help` (installed binary); `mnemonic-secret/crates/ms-cli/src/cmd/derive.rs:102-105, 123-143, 146-176` | MEASURED + SOURCED |
| **`md` codec (ours)** | **Applies no taproot path convention and refuses none.** `md encode --path` takes named `bip44\|48\|49\|84\|86` (purpose only — a keyless card stores literally `m/86`), hex `0xNN`, or a literal `m/…`. All four candidate origins encode **and round-trip byte-clean** in a `tr(@0,multi_a(2,@1,@2))` template. Depth admission is `{3,4}` in either script context since 2026-08-19, so BIP-87's depth-3 accounts are legal on the wire. | `md encode --help`; MEASURED round-trips below; `descriptor-mnemonic/crates/md-cli/src/parse/keys.rs:12-22, 91-121` | MEASURED |
| **seedhammer fork (shipped Multisig Build)** | wsh: `multisigSharedOrigin()` = `m/48'/0'/0'/2'`, *"the LOCKED shared origin for OriginShared mode"*; in the divergent mode each held slot takes its own hardened **account** by ordinal among the slots one master fills. No taproot build path exists yet. | `seedhammer@169073c:gui/multisig_build.go:1356-1362`, l.593-601 | MEASURED |

---

## 3. Answers to points 1-6

### 1. BIP-48 — which script_type values, and does it say anything about taproot?

**SOURCED. `1'` and `2'` only. It does not define `0'`. It says nothing about taproot.**

> `bip-0048.mediawiki:103-104` — *"Currently the only script types covered by this BIP are
> Native Segwit (p2wsh) and Nested Segwit (p2sh-p2wsh)."*
>
> `:107` — *"`1'`: Nested Segwit (p2sh-p2wsh) `m/48'/0'/0'/1'`"*
> `:110` — *"`2'`: Native Segwit (p2wsh) `m/48'/0'/0'/2'`"*
> `:112` — *"The recommended default for wallets is pay to witness script hash `m/48'/0'/0'/2'`."*
> `:100-101` — *"This level splits the key space into two separate `script_type`(s). To
> provide forward compatibility for future script types this specification can be easily
> extended."*

MEASURED: `grep -ci 'taproot\|p2tr\|schnorr' bip-0048.mediawiki` → **0**.
MEASURED: no `3'` appears anywhere in the file.

Note the *extension clause* at l.100-101 — BIP-48 explicitly anticipates new script types
being appended. That is the hook PR #1473 tried to use, and the hook `3'` would occupy.
**A `0'` value is not defined anywhere in current BIP-48**; PR #1473 proposed it for
bare P2SH and it was rejected on the merits (craigraw: *"P2SH is at this stage a legacy
script type … Like it or not, BIP45 (`m/45'`) is the standard"*). This matters here
because the operator's `bg002h-tr` uses level-4 `0'`, which — under BIP-48's numbering —
is the value the community declined to assign to bare P2SH. `bg002h` sits at purpose
`270028'`, so there is no actual collision; but the *number* is not an unclaimed one.

### 2. BIP-86 — does it say anything about multisig / multi_a?

**SOURCED. No. Not one word.**

> `bip-0086.mediawiki:4` — Title: *"Key Derivation for **Single Key** P2TR Outputs"*
> `:14-15` — *"a derivation scheme for HD wallets whose keys are involved in **single key**
> P2TR ([BIP 341]) outputs **as the Taproot internal key**."*
> `:23-24` — *"With the usage of **single key** P2TR transactions, it is useful to have a
> common derivation scheme so that HD wallets that only have a backup of the HD seed can be
> likely to recover **single key** Taproot outputs."*

MEASURED: `grep -ci 'multisig\|multi-sig\|multi_a' bip-0086.mediawiki` → **0**.

Its address-derivation section (l.51-62) *tweaks the key with an unspendable script path*,
which is the opposite of a multisig taptree. Using `86'` for cosigner keys would also make a
tr-multisig slot key **identical** to the operator's own BIP-86 single-sig account key at
the same index — a real key-reuse collision, and one a BIP-86-scanning recovery tool would
walk straight into. **Reject `86'` for multisig slots.**

### 3. BIP-87 — what it proposes, its status, who implements it, and is it used for taproot multisig?

**SOURCED, with one implementation confirmed.**

- **Proposes** `m / 87' / coin_type' / account' / change / address_index` (l.68, l.77), a
  **script-agnostic** multisig hierarchy: *"We should not be mixing keys and scripts in the
  same layer"* (l.63). Descriptor carries the script; the path does not.
- **Status: Complete** (l.6). **"Reference Implementation: None at the moment."** (l.256).
- **Its own critique of BIP-48** is the most useful sentence for this question (l.49):
  > *"vendors decided to insert `script_type'` into the derivation path (where
  > P2SH-P2WSH=1, P2WSH=2, **Future_Script=3**, etc). As described previously, this is
  > unnecessary, as the descriptor sets the script."*

  So even BIP-87 — the BIP arguing *against* the script_type level — reads `3'` as the next
  slot in the sequence. That is the closest thing to a written statement that `3'` is the
  natural next value.
- **Key-reuse clause (l.104-106)**, which bears directly on the composer's C5:
  > *"It is crucial that this level is increased for each new wallet joined or
  > private/public keys created … This prevents key reuse - across ECDSA and Schnorr
  > signatures, across different script types, and in between the same wallet types."*

  Read carefully: because BIP-87 has **no** script_type level, preventing tr/wsh key reuse
  is entirely the **account bookkeeping's** job. Under BIP-48-with-`3'` the level-4 value
  does it structurally. For a composer that will emit both a `tr` and a `wsh` form of the
  same wallet from the same seeds (the reference fixture does exactly this), that is a
  meaningful difference.
- **Who implements it:** **Nunchuk** — `libnunchuk:src/utils/bip32.hpp:103-105` returns
  `m/87h/ch/zh` for `AddressType::TAPROOT` under `WalletType::MULTI_SIG` *and*
  `WalletType::MINISCRIPT`, and `GetBip32Path` is the function Nunchuk calls before asking
  a Ledger / Trezor / Tapsigner / HWI device for an xpub (`nunchukimpl.cpp`,
  `utils/trezor.cpp`, `utils/ledger/ledger_session.cpp`). **Our own `mnemonic-toolkit`**
  also implements it and makes it the **default** family (`parse.rs:67-68`, README l.195).
  **Sparrow: no** (`ScriptType.java` has no `87'`; P2WSH default is `m/48'/0'/0'/2'`).
  **Coldcard: no** (`87` appears nowhere in its export path list). **Specter: UNVERIFIED**
  — I did not reach Specter's source in this pass.
- **Is it used for taproot multisig anywhere?** **Yes, by Nunchuk** (SOURCED, above) — and,
  as far as I could measure, only by Nunchuk among the wallets surveyed. BIP-87's own text
  contains one taproot-family mention (`grep -ci 'taproot|p2tr|schnorr'` = 1, MEASURED: the
  word "Schnorr" at l.106) and prints **no** `tr()` example.

### 4. De-facto practice, wallet by wallet

| Wallet | Taproot multisig supported? | Path it uses / requests | Evidence |
| --- | --- | --- | --- |
| **Sparrow** | **No.** `P2TR` allows only `SINGLE_HD, SINGLE_SP`; multisig (`MULTI_HD`) defaults to `P2WSH`; taproot input construction throws `UnsupportedOperationException("Constructing Taproot inputs is not yet supported")`. | n/a. Its P2TR **single-sig** default is `m/86'/0'/0'`; its P2WSH default is `m/48'/0'/0'/2'`. | `drongo:ScriptType.java:1090, 1212-1214, 1196, 1201, 968`; `PolicyType.java:8` |
| **Nunchuk** | **Yes** (MULTI_SIG and MINISCRIPT). | **`m/87h/{coin}h/{account}h`** — BIP-87. Comment: *"Taproot Musig BIP87 Wallets"*. | `libnunchuk:src/utils/bip32.hpp:103-105` |
| **Liana** | **Yes** (`tr(…,{…multi_a…})` descriptors). | **`m/48'/{coin}'/{account}'/2'`** — hard-coded, **no taproot branch**. Same function for hardware and hot signer. | `liana-gui/src/installer/step/descriptor/editor/key.rs:1509-1523, 1527-1535, 519-528` |
| **Ledger app-bitcoin-new** | **Yes**, via BIP-388 registration. | **No convention imposed.** Origin **required**; if the fingerprint matches the device the origin must re-derive the xpub. `tr()` *with a tree* is never "standard" → always needs registration. | `register_wallet.c:112-146`; `policy.c:1411-1419, 1443-1447, 1498-1510`; `doc/wallet.md:13-14, 43-57` |
| **Coldcard** | **Yes on `edge`/`new_edge` only** (miniscript wallets); master has no taproot. | **`m/48h/{coin}h/{acct}h/3h`**, exported as `bip48_3`, `AF_P2TR`, `is_ms=True`. | `firmware@new_edge:shared/export.py:414`; `@edge:shared/export.py:372` |
| **BIP-388 itself** | — | **No origin constraint whatsoever**; its own tr vector uses `48'/0'/0'/100'`, and it mixes `44'` and `48'` origins inside one wsh policy. | `bip-0388.mediawiki:180-187, 292-293, 282-283` |

**Coldcard's `3'` is confirmed by an independent consumer**, not just by its own source:
Liana carries `liana-gui/test_assets/ccxp-C658B283.json` with
`"p2tr_deriv": "m/48h/1h/0h/3h"` and a matching `p2tr_key_exp`. So the `3'` convention has
crossed at least one project boundary in practice.

**Non-adoption is worth stating honestly.** A GitHub-wide code search for the literal
`"48h/0h/0h/3h"` returned **one** unrelated repo (`w-s-bitcoin/entropylab`), and
`"48'/0'/0'/3'"` returned **only our own `mnemonic-toolkit`** (MEASURED). Adoption of `3'`
outside Coldcard is thin — this is a de-facto convention with one strong implementer, not a
widely-deployed one. It is still the *broadest* taproot-multisig convention available,
because the alternatives are: one implementer each (Nunchuk/`87'`, Liana/`2'`) and zero for
everything else.

### 5. Does anything REJECT or WARN on `tr()` with `48'/…/2'` origins, or on a custom purpose like `270028'`?

**MEASURED / SOURCED: nothing I could find rejects or warns on either.**

- **BIP-388:** no. Origin is optional and unconstrained (l.180-187); its own taproot vector
  uses `48'/0'/0'/100'` (l.292-293), which is neither `2'` nor `3'` nor any registered
  value. If BIP-388 tolerates `100'` under `tr()`, it tolerates `2'`, `3'`, `87'` and
  `270028'`.
- **Ledger:** no path-shape check for multisig. It requires an origin to be **present**
  (`register_wallet.c:112-116`) and, when the fingerprint matches, to actually derive the
  xpub (`:128-146`) — a *self-consistency* check, not a convention check. `tr()` with a tree
  can never be a "default wallet" (`policy.c:1411-1419`), so it always goes through
  registration and user approval; the path never affects that outcome.
- **Liana:** it *emits* `48'/…/2'` under `tr()` itself, so it plainly does not warn on it.
- **Nunchuk:** `GetBip32Type` maps `48h/…/1h`→`bip48_1`, `48h/…/2h`→`bip48_2`, `87h/…`→
  `bip87`, and **everything else → `"custom"`** (`bip32.hpp:135-139`). A `48h/…/3h` or a
  `270028'` key is labelled `"custom"`. That is a **label**, not a refusal — I found no
  code path that rejects on it. (Whether the Android/iOS UI surfaces "custom" as a warning
  is **UNVERIFIED** — the apps are separate repos I did not fetch.)
- **Sparrow:** cannot be asked — it has no taproot multisig (§4).
- **Our own `md` 0.14.0:** no. **MEASURED**, all four encode and round-trip byte-identically
  in `tr(@0/<0;1>/*,multi_a(2,@1/<0;1>/*,@2/<0;1>/*))`:

  | `--path` | md1 | decode → origins |
  | --- | --- | --- |
  | `m/48'/0'/0'/2'` | (encodes) | `m/48'/0'/0'/2'` ×3 |
  | `m/48'/0'/0'/3'` | `md1yzfdsssjuqqczyszzcg5txj4zje277l` | `m/48'/0'/0'/3'` ×3 |
  | `m/87'/0'/0'` | `md1yz80tcggqpsyfqy9s9yfqn6z0ezcnj` | `m/87'/0'/0'` ×3 |
  | `m/270028'/0'/0'/0'` | `md1yzf74anyyyyyqqczyszzczecvchptgavp4` | `m/270028'/0'/0'/0'` ×3 |

  The only diagnostic emitted was the **unrelated** indistinguishable-slots warning (all
  three slots declaring one shared path), which fires identically for every path and is
  answered by `--fingerprint`.

**Therefore the choice is not constrained by any refusal.** It is constrained only by (a)
what a cosigner's own signer can *export*, and (b) whether the key collides with another
key the same seed already uses. Both favour `3'`; see §1.

### 6. What our own tooling can derive today

**MEASURED.**

- **`ms derive --help`** (installed binary `/home/bcg/.cargo/bin/ms`) — `--template` accepts
  exactly: `bip44`, `bip49`, `bip84`, `bip86`, `bip48-p2wsh` *(script_type 2', "the BIP's
  recommended default")*, `bip48-p2sh-p2wsh` *(1')*, `bip48` *(assumes 2' and says so)*.
  **No `bip48-p2tr`. No `bip87`.** The installed binary predates the repo: the working tree
  at `mnemonic-secret/crates/ms-cli/src/cmd/derive.rs:134,141` adds `bg002h-tr` =
  `m/270028'/coin'/account'/0'` and `bg002h-wsh` = `…/1'` (purpose from `:153`, script_type
  from `:172-173`).

  **The stale premise, stated plainly** (this is the single most actionable finding in this
  report). `derive.rs:102-105` reads:
  > *"There is no `bip48-p2tr`: BIP-48 registers no Taproot script type, and inventing one
  > would derive to a path **no other wallet looks at**. That is where permissiveness STOPS
  > — assuming a documented default is service, inventing an unregistered path is data
  > loss."*

  and `:126-133`:
  > *"BIP-48 registers no taproot script type, so a `tr()` multisig key has no standard
  > 4-level home -- and **md requires depth 4 for any multisig script context**."*

  **Both load-bearing clauses are now false.** (i) Coldcard Edge exports `m/48h/…/3h` for
  taproot multisig and Liana's test corpus consumes it, so `3'` is a path at least one other
  wallet *does* look at. (ii) `md` has not required depth 4 since 2026-08-19 —
  `md-cli/src/parse/keys.rs:12-14` (*"Since 2026-08-19 this no longer gates depth —
  `parse_key` admits depth 3 or 4 in either context"*) and `:91-121` (*"OPERATOR STOPGAP
  2026-08-19, widening a previous exact match … TOO STRICT — BIP-87 … a DEPTH-3 xpub in
  multisig, which the old rule rejected outright"*); and I encoded a depth-3
  `m/87'/0'/0'` `tr()` policy byte-clean above. The first clause was written 2026-08-18, one
  day before the depth rule changed. **This does not overturn the operator's `bg002h`
  ruling** — that is theirs to revisit — but the ruling's stated reasons no longer hold, and
  a composer built on them would inherit a premise nobody has re-checked.

- **`md encode --help`** — `--path <PATH>`: *"Override the inferred origin path with a
  single shared path (flattens Divergent mode to Shared). Accepts named
  (`bip44|48|49|84|86`), hex (`0xNN`), or literal (`m/...`) forms."* **No `bip87` named
  form; no taproot-specific behaviour of any kind.** MEASURED: the named forms are
  *purpose-only* — `md encode … --path 86` on a `tr()` template stores literally `m/86` as
  each slot's origin (decode confirms `@0: m/86`), not a 3-level BIP-86 account path. Any
  full path must be given literally. So **`md` applies no path convention to `tr()`
  templates**; whatever the composer decides, the composer must spell out.

- **`mnemonic-toolkit`** already has the answer implemented on both sides:
  `MultisigPathFamily::{Bip48, Bip87}` with `#[default] Bip87` (`parse.rs:62-69`),
  `default_origin_path` producing `m/87'/coin'/account'` or
  `m/48'/coin'/account'/script_type'` (`:86-103`), and the taproot script_type named
  `bip48-tr-multi-a` = `3'` in the xpub search (`candidate_paths.rs:8-10, 81-86`, asserted
  by a unit test at `:199-203`). CLI surface: `--multisig-path-family {bip48,bip87}`,
  default `bip87` (README l.195).

**The constellation is already internally inconsistent on this question, three ways:**
`mnemonic-toolkit` defaults to **BIP-87** and knows `48'/…/3'` as `bip48-tr-multi-a`;
`mnemonic-secret` rules for **`270028'/…/0'`** and argues *against* `48'/…/3'`; the
seedhammer fork ships **`48'/…/2'`** (wsh only). Whichever way the composer goes, one of
these needs a follow-up — and per the Rust-primary rule any change lands in Rust with
vectors before the Go port.

---

## 4. Interop consequences of each rejected alternative (from evidence)

| Option | Cost if chosen, stated from measured evidence |
| --- | --- |
| **A. Reuse `48'/…/2'` under `tr`** | Ships in Liana today, so it is the only convention a Liana-sourced cosigner can produce **without a custom path entry** (`key.rs:1509-1523` is hard-coded). But the path then *says p2wsh and means taproot*, and a seed's `tr` slot and `wsh` slot at the same account become **the same key** — the reuse BIP-87 l.106 names across "ECDSA and Schnorr signatures". The composer emits both forms of one wallet from one seed set, so this pushes a funds-relevant distinctness burden entirely onto account bookkeeping. Also: `md`'s `--path 48` shorthand and the fork's `multisigSharedOrigin()` would silently produce it, making the wrong thing the easy thing. |
| **B. New script_type `3'`** — **RECOMMENDED** | Cost: it is **not a registered value** — BIPs PR #1473 was closed, so no document blesses it, and "we follow BIP-48" would be ambiguous (murchandamus' stated reason for closing). Adoption outside Coldcard is thin (MEASURED: two GitHub hits total, one of them ours). Requires a new `ms derive` template and reverses `derive.rs:102-105`. |
| **C. BIP-86 `m/86'/coin'/account'`** | **Reject.** BIP-86 is single-key by title and scope (l.4, l.14-15), and a multisig slot key would be **identical** to the operator's own BIP-86 single-sig account key — a live collision a BIP-86 recovery scan would walk into. No surveyed wallet uses `86'` for a multisig slot. |
| **D. BIP-87 `m/87'/coin'/account'`** — **second choice** | Cheapest on paper: a **Complete** BIP, purpose-built for multisig, script-agnostic (so one path serves the `tr` and `wsh` forms), shipped by **Nunchuk** for taproot, already the **default** in `mnemonic-toolkit`, and legal on our wire since the depth-`{3,4}` widening. Cost: **no hardware signer surveyed exports it** — Coldcard, Sparrow and Liana have no `87'` path at all, so a cosigner on those must enter a custom derivation by hand; the BIP itself says *"Reference Implementation: None at the moment."*; and being script-agnostic, it puts the tr-vs-wsh distinctness back on account bookkeeping (its own l.104-106 acknowledges this and instructs incrementing `account'` per wallet). |
| **E. `bg002h` `m/270028'/coin'/account'/{0'\|1'}`** | Maximal collision-freedom (no BIP will ever claim purpose `270028'`) and it is the operator's 2026-08-18 ruling with the reference-wallet fixture built on it. Cost: **zero third-party tooling can produce it** — every cosigner on a Coldcard, Ledger, Sparrow, Nunchuk or Liana must type a custom path, and neither `ms` (installed) nor `md` will name it for them. Both of its stated justifications are now stale (§3.6). For a composer whose whole point is seating keys other people exported, this is the weakest interop position — though it remains defensible for the operator's *own* seed-derived keys, which is exactly what `ms derive` produces. |

**Boundary worth naming for the composer's UX (feeds the C8 journey row
*"a card's origin script type disagrees with the wrapper"*, currently classed
documentation/warning):** because **no** convention is standard, the composer must not
*refuse* a seated card whose origin is `48'/…/2'` in a `tr()` policy — Liana produces
exactly that shape and it is valid everywhere I measured. A **warning** naming both the
declared script type and the wrapper is the right class, and the warning text can now cite
concrete facts: `2'` means p2wsh, this policy is taproot, Coldcard writes `3'` here and
Nunchuk writes `87'`, and no BIP settles it.

---

## 5. What I ran / fetched

**Fetched (authoritative source text), into the session scratchpad:**
- `bitcoin/bips@master`: `bip-0048.mediawiki` (253 l.), `bip-0086.mediawiki` (126 l.),
  `bip-0087.mediawiki` (272 l.) — fetched fresh this session. `bip-0388.mediawiki`,
  `bip-0341/342/379/383/386/387` were already cached from earlier in the session.
- `gh api repos/bitcoin/bips/pulls/1473` + `/files` + `/issues/1473/comments` — the closed
  taproot-`3'` PR, its diff, and the full maintainer/wallet-author discussion.
- `Coldcard/firmware`: `shared/export.py` on `master`, `edge` (`158c8a77`) and `new_edge`
  (`2a3c9df2`); `shared/multisig.py` on both; `shared/miniscript.py`, `shared/chains.py`.
  Branch list via `api.github.com/repos/Coldcard/firmware/branches`.
- `sparrowwallet/drongo@d6669436`: `protocol/ScriptType.java` (1594 l.), `policy/PolicyType.java`.
- `nunchuk-io/libnunchuk`: `src/utils/bip32.hpp`; `src/descriptor.cpp` and the file tree
  were already cached. Usage sites via `gh search code`.
- `wizardsardine/liana@master`: `liana-gui/src/installer/step/descriptor/editor/key.rs`
  (1601 l.), `installer/step/share_xpubs.rs`, full repo tree; `liana_desc.rs` /
  `liana-analysis.rs` were already cached.
- `LedgerHQ/app-bitcoin-new@develop`: `src/handler/register_wallet.c`,
  `src/handler/lib/policy.c` (2126 l.), `src/common/wallet.c`; `doc/wallet.md` cached.
- `gh search code` adoption sweeps for `"48h/0h/0h/3h"`, `"48'/0'/0'/3'"`, `p2tr_deriv`,
  `multi_a` in Sparrow/drongo. Search health confirmed with two positive controls after an
  empty result, per the "empty output is not absence" rule.

**Ran locally (measurements):**
- `ms derive --help` (installed binary, resolved by path — `md` is shell-aliased to
  `mkdir -p` here, so both tools were invoked as `/home/bcg/.cargo/bin/…`).
- `md --version` → `md 0.14.0`; `md encode --help`.
- `md encode 'tr(@0/<0;1>/*,multi_a(2,@1/<0;1>/*,@2/<0;1>/*))' --path X` for
  X ∈ {`86`, `48`, `44`, `m/48'/0'/0'/2'`, `m/48'/0'/0'/3'`, `m/87'/0'/0'`,
  `m/270028'/0'/0'/0'`}, each followed by `md decode` — all round-trip byte-clean, no
  path-convention diagnostic.
- `grep -ci 'taproot|p2tr|schnorr'` and `grep -ci 'multisig|multi-sig|multi_a'` over
  BIP-48/86/87.
- Repo reads: `mnemonic-secret/crates/ms-cli/src/cmd/derive.rs`;
  `descriptor-mnemonic/crates/md-cli/src/parse/keys.rs`;
  `mnemonic-toolkit/crates/mnemonic-toolkit/src/parse.rs`,
  `src/cmd/xpub_search/candidate_paths.rs`, `crates/mnemonic-toolkit/README.md`;
  `seedhammer@169073c:gui/multisig_build.go` (citation re-verified, not taken on trust).

**Prior art I did not duplicate:** `design/agent-reports/recon-protocol-multisig-xpub-depth.md`
(2026-08-19, 25 KB) already established the BIP-48/86/87/388 depth landscape and drove the
`{3,4}` widening. This report cites it rather than re-deriving it, and adds what it did not
cover: the **closed BIPs PR #1473**, and the **shipping wallet behaviour** (Coldcard `3'`,
Nunchuk `87'`, Liana `2'`, Sparrow none).

**Not reached — UNVERIFIED:**
- **Specter Desktop** — point 3 asked whether it implements BIP-87; I did not fetch it.
- **Nunchuk's mobile UI**: whether the `"custom"` label from `GetBip32Type` is surfaced to
  the user as a warning.
- **Blockstream Jade, Keystone, Bitcoin Keeper, Foundation Passport** — not surveyed; a
  `3'`-vs-`87'` decision that must hold for those signers should check them first.
- **Coldcard's miniscript/descriptor import**: I confirmed `shared/miniscript.py`
  (`new_edge`, 1172 l.) contains no origin-path validation, but I did not exhaustively
  sweep the rest of the `new_edge` `shared/` tree for a non-standard-path warning.
