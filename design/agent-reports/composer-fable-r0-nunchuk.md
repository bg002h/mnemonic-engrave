# Composer fable review r0 — Lens 2: USING the SH2-composed wallets in Nunchuk

**Verdict: 0 Critical / 2 Important / 3 Minor / 2 Nit.** Nunchuk Desktop 2.1.1
imports 20 of the 30 composer shapes measured (every `wsh`, `sh`, `sh-wsh` and
every `tr` with a real internal key) through its "Recover via BSMS/descriptors"
route and then derives byte-identical addresses to `md 0.16.2`, to the device
(`policyprobe` at fork tip `f5b068fa`) and to Bitcoin Core 31.1 on both chains
(6/6 addresses per wallet) with every spend path enumerated; it REFUSES every
NUMS-keyed `tr` policy the composer emits (7/7, so §8f's "Bitcoin Core and
Nunchuk import this form" is FALSE for Nunchuk), every `wsh` policy that mixes
time-based and height-based locks across paths ("Timelock mixing"), and the two
experimental key-less hash paths — all of it measured by RUN against libnunchuk
`a7cfb498` (the commit desktop tag `2.1.1` pins) built here with its vendored
Bitcoin Core `57b47c47` (v28.99).

Reviewer: fable, independent. Read-only on every repo; nothing committed. No
sub-agents. The operator's live Nunchuk state was never touched: I did NOT run
the AppImage at all — I only `--appimage-extract`ed a COPY of it under
`/scratch/code/shibboleth/.tmp/fable-nunchuk-app/` to read its embedded strings.

## Findings

### I-1 (Important) — Every NUMS-keyed `tr` policy is unimportable in Nunchuk, and §8f says the opposite

**Inputs.** All seven composer shapes whose taproot internal key falls back to
NUMS (no bare single-key path): `plain-2of3-tr`, `preset-kofn-recovery-tr`,
`preset-tiered-recovery-tr`, `preset-hashlock-gated-tr`,
`preset-decaying-multisig-tr`, `hashlock-gated-tr-hash160`,
`same-seed-two-paths-tr`. Composed at the fork tip through the same calls
`gui/composer_flow.go:280-320` makes, decoded by `md 0.16.2 descriptor`, e.g.
(`preset-kofn-recovery-tr`, the §7 "kofn-recovery" preset under tr):

```
tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,{multi_a(2,[73c5da0a/48'/0'/0'/3']xpub6DXuQW1Q2JpZyweiMewTZuMPvjG8hKhV2qoF6wL9VFxsMBExtbfqAAoR4oMG4GyxFzVdfas1v2eAdfLxyjc4Ceo5B6w6zTpf7F2BuXCJ52i/<0;1>/*,[3f635a63/48'/0'/0'/3']xpub6DXuQW1Q2Jpa1hNtFUcghdx7Q8kTDsqo7b54YAqZBNCH8EuSvmNSAKbAvkZ4HspgftJ1aqSMeFiZ4sr2QNEGm9geaEre3zDwiJD7C5gx5VH/<0;1>/*,[66d455ea/48'/0'/0'/3']xpub6DXuQW1Q2JpZyteDRGW1pD34uhumfnZJfTsmjDkgd4xcq3L5XX2KUE1n4rmvcDT3RmdchfhbD9DkvSyVhUMBjUYi691iFszgKtf4Bfqe2nL/<0;1>/*),and_v(v:pk([73c5da0a/48'/0'/1'/3']xpub6DXuQW1Q2JpZzLV9igdwdnmCSoaPVd4ZNZnvfgUsGvQ8AbNAhEmfBEMCMHctwZBuxWK8HkjqUW5F72MCSJCFfisVwRY62Kb1FuDZ66nNQe1/<0;1>/*),older(26280))})#xnta28tv
```

**Observed (RUN, libnunchuk a7cfb498).** `nunchuk::Utils::ParseWalletDescriptor`
— the exact function the desktop calls (`ifaces/qUtils.cpp:206-218` at tag
2.1.1) — throws `code=-1017 what='Could not parse descriptor'` for all seven,
in both the multipath and the `--chain 0` spellings (14/14 refused). Two
distinct mechanisms:

1. `plain-2of3-tr` renders `tr(NUMS,sortedmulti_a(2,K,K,K))`. libnunchuk
   routes every `tr()` with a script tree through its miniscript template
   validator, and `sortedmulti_a` is a descriptor function, not a miniscript
   fragment: `ParseDescriptors` returns
   `error='invalid miniscript: sortedmulti_a(2,[73c5da0a/48'/0'/0'/3']xpub…'`
   (`src/descriptor.cpp:557-560` → `Utils::IsValidMiniscriptTemplate`,
   `src/nunchukutils.cpp:1273-1281`). Nunchuk's own taproot multisig is
   musig-based (`descriptor.cpp:166-214`); it has no `sortedmulti_a` wallet.
2. The other six pass the template validator (`IsValidTapscriptTemplate=true`)
   and fail the round-trip check: `ParseDescriptors` re-renders the parsed
   wallet across four `DescriptorPath` forms and requires a string match
   (`src/descriptor.cpp:663-686`). A `tr` wallet with `keypath_m == 0` is
   stamped `WalletTemplate::DISABLE_KEY_PATH` (`src/dto/wallet.cpp:66-80`) and
   `Wallet::get_descriptor` then renders the key path as
   `GetUnspendableXpub(signers_) + "/<0;1>/*"` (`wallet.cpp:214-217`,
   `descriptor.cpp:724-747`) — an xpub, never the raw 32-byte `H` — so nothing
   matches and the result is `error='Failed to verify wallet descriptor'`.

**Expected.** §8f (spec l.711-714) is shown on the device whenever a tr policy
falls back to NUMS: *"KEY PATH: NONE (NUMS) … Bitcoin Core and Nunchuk import
this form. Liana and BIP-388 signers need an unspendable xpub instead (see
F-449)."* §5c (l.251) repeats it: *"The raw `H` spelling is valid in Bitcoin
Core and imported by Nunchuk."* Core: TRUE (v25 `getdescriptorinfo` ok on the
chain-0 form of `plain-2of3-tr`; v31.1 `getdescriptorinfo` + `importdescriptors`
ok on all seven multipath forms). Nunchuk: FALSE, 0 of 7.

**Why this is Important and not Critical.** Nunchuk refuses the device's
artifact; it never imports it with different addresses, and Core imports it, so
funds are reachable. It is a real unmet claim on the device's own copy and a
wallet the operator cannot import. **But note the Critical-shaped trap one step
away:** the only NUMS spelling Nunchuk accepts is the unspendable-xpub form the
same §8f paragraph points Liana users at. I constructed it exactly as libnunchuk
does (chain code = sha256 of the sorted, de-duplicated compressed account
pubkeys; key = `02‖H`) and fed it back: `preset-kofn-recovery-tr` is then
ACCEPTED as `MINISCRIPT/TAPROOT/DISABLE_KEY_PATH 0-of-4` — and derives
`bc1pm0udr8a92du44f098j49pv8cnjq63nm68tudld57fqth9ee9udyqa2j0wx …` where the
device, `md` and Core derive `bc1pac935qvs2pj56zc7a0ruawaz2493rpenejg6qzt95znhvtv9pcnq84yvl5 …`
(`same-seed-two-paths-tr`: `bc1ppzqzlp…` vs `bc1p7tsex3…`). The xpub form
derives per-index children of `H`, the raw form does not; they are different
wallets by construction. So F-449 must never be described as "the same wallet
for Nunchuk", and the SH2 copy must not send a Nunchuk user there.

**What must change on the SH2 side (one sentence).** §8f's copy: drop Nunchuk
from the "import this form" sentence and say plainly that Nunchuk cannot import
a NUMS policy (and that plain k-of-n under tr is never Nunchuk-importable), so
an operator who wants Nunchuk chooses `wsh` or a tr policy with a bare
single-key path.

### I-2 (Important) — Nunchuk refuses any `wsh` policy that mixes time-based and height-based locks across paths

**Inputs.** `mixed-lock-bases-wsh`: paths `[1-of-1]`, `[1-of-1, older 100 ×
512 s]`, `[1-of-1, after height 1,000,000]` — every lock inside §4c's ranges,
admitted by `md.ValidatePathList`, composed, decoded:

```
wsh(or_i(pkh([73c5da0a/48'/0'/0'/2']xpub6DXuQW1Q2JpZxsEnFKrPvDuiRMmQgU4fzHU1wsvM5EqgGAWRJ3cmwbtS8u1HQjrEHg3YFb7XGnFovPydJ8qpaGNNd2hSEPoheWd27EABdGH/<0;1>/*),or_i(and_v(v:pkh([3f635a63/48'/0'/0'/2']xpub6DXuQW1Q2JpZwZhyeFyRwoVcxxRQUvWjfWf5X5tre7aRCMTYwNR1DnwZAehowmtGsB2oEka2aWofzRgVnexutt2KVBZfRcPtuxS6JYwywD5/<0;1>/*),older(4194404)),and_v(v:pkh([66d455ea/48'/0'/0'/2']xpub6DXuQW1Q2JpZw2pTr5epQqR2dceAT9UAeRtrbqNAh24Mrg99k6spPvgiaDoCdEmvzqcka6r5Yfpa4asbrxqr6PbrJ6LVphjMPiZ1iJKrJPm/<0;1>/*),after(1000000)))))#h5upaq8n
```

**Observed (RUN).** `Utils::IsValidMiniscriptTemplate` throws
`code=-1017 what='Timelock mixing'` (`src/miniscript/timeline.cpp:9-37`:
`MiniscriptTimeline` walks the whole script and `detect_timelock_mixing` throws
on the first lock whose base — TIME vs HEIGHT — differs from any earlier one,
regardless of relative/absolute and regardless of which `or` branch it sits
in). `ParseDescriptors` swallows the exception (`descriptor.cpp:683`), so the
app shows only `Could not parse descriptor`. Bitcoin Core imports the same
descriptor: v25 `deriveaddresses` matches `md`, v31.1 `importdescriptors` ok
(miniscript's own rule only forbids mixing inside one satisfaction). The same
three paths under `tr` (`mixed-lock-bases-tr`) are ACCEPTED, because the
tapscript route validates each leaf separately (`nunchukutils.cpp:1310-1330`).
The composer's own presets are safe: `preset-decaying-multisig-wsh` mixes
`older(blocks)` with `after(height)` — both HEIGHT — and imports.

**Expected.** §4c admits `older` in 512-second units and `after` as a Unix
time alongside height locks, and nothing in §8 warns that a coordinator will
refuse the combination under `wsh`. A composable wallet the operator cannot
import is Important by this lens's brief. The remedy is copy, not wire: a §8
notice on the review screen when a `wsh` policy carries both lock bases
("Nunchuk will refuse this policy; Bitcoin Core imports it"), in the style of
§8g's Liana line.

### M-1 (Minor) — the change-chain single-chain spelling is never importable

`md descriptor --chain 1` output (`…/1/*`) is refused for all 30 shapes
(`Failed to verify wallet descriptor`): `ParseDescriptors` compares only
against `EXTERNAL_ALL`, `EXTERNAL_INTERNAL`, `ANY`, `TEMPLATE`
(`descriptor.cpp:658-661`), never `INTERNAL_ALL`. The default multipath output
and `--chain 0` both import (20/20 each). Documentation only: the runbook says
"paste the default (multipath) form".

### M-2 (Minor) — experimental key-less hash paths import nowhere

`keyless-hash-path-wsh` (`wsh(or_i(pkh(K),sha256(<digest>)))`) and
`keyless-hash-older-path-wsh` are refused by Nunchuk
(`Invalid miniscript: …` from `IsSane()` in `nunchukutils.cpp:1276`) AND by
Core v25 and v31.1 (`getdescriptorinfo`/`importdescriptors` refuse the
sigless path). The composer already marks these `ExperimentalKeylessPath`
with the §8 warning; that warning should say no coordinator measured here
accepts the form.

### M-3 (Minor, UI-level UNVERIFIED) — the xpub header bytes: Nunchuk keys nothing on them at import, one string equality remains

The first-time question. `md 0.16.2` renders the demo wallet's key 0 as
`xpub6DXuQW1Q2JpZxsEnFKr…` = depth 4, parent fingerprint `00000000`, child
`80000002`; the device's card carries `xpub6DkFAXWQ2dHxq2vatrt…` = depth 4,
parent `1cf29716`, child `80000002`; same chain code and point (decoded
here). RUN: Nunchuk parses the zero-parent form, stores it verbatim as the
signer's xpub (`ParseSignerString` keeps `sm[3]`, `descriptor.cpp:405-420`),
and derives identical addresses (20/20 wallets, 6/6 addresses each) — parse
and derivation ignore depth/parent/child. SOURCED: on `CreateWallet` the
signer is linked to the user's OWN key by master fingerprint and path, not by
xpub bytes (`src/storage/storage.cpp:485-520` `save_true_signer`: if a master
signer with that xfp exists it `AddXPub`s the imported string under that path;
otherwise the key is stored as a hidden `import`/`UNKNOWN` remote signer). The
one xpub-string equality is `NunchukStorage::HasSigner(const SingleSigner&)`
(`storage.cpp:660-679`, `remote.get_xpub() == signer.get_xpub()`), which would
report an md-rendered key as "not one of yours" if the same key had earlier
been added as a REMOTE signer with its real xpub string. Cosmetic at worst;
which screens call it was not traced.

### N-1 (Nit) — the brief's "raw `H` hardened spelling" is a misreading; measured anyway

§5c's "raw `H` spelling" is the NUMS point written as raw hex (contrast: the
BIP-388/F-449 xpub form), not the hardened marker. Measured both readings.
Hardened marker: the vendored Core accepts only `'` and `h`
(`src/script/descriptor.cpp:1642` at `57b47c47`: `if (last == '\'' || last ==
'h')`); an uppercase-`H` spelling of the demo wallet is REFUSED by Nunchuk
(`Could not parse descriptor`, exception swallowed; not traced further) and
the `h` spelling is ACCEPTED. `md` never emits `H`, so nothing to change;
the spec sentence could say "the raw NUMS point" to stop the next reader
tripping.

### N-2 (Nit) — an experimental unsorted `multi` imports as a "miniscript" wallet

`plain-2of3-wsh-UNSORTED` (`wsh(multi(2,K,K,K))`) imports, addresses match,
but Nunchuk classifies it `MINISCRIPT 0-of-3` with one signing path rather
than `MULTI_SIG 2-of-3`, because only the `wsh(sortedmulti(` prefix takes the
multisig route (`descriptor.cpp:596-598`). Documentation for the
`ExperimentalUnsortedKeys` warning.

## The matrix

Every row: composed at fork tip `f5b068fa` by `md.ComposeWith` + `Bind` + mk1
minting through the device's own calls; decoded by `md 0.16.2`; form B
(template + cards, `md descriptor --from-mk1`) re-derives the SAME descriptor
as form A for 30/30; device addresses via `cmd/policyprobe` = `md address`
for 30/30. "Nunchuk 2.1.1 result" is `Utils::ParseWalletDescriptor` on the
default multipath spelling (the `--chain 0` spelling gives the same
accept/refuse for all 30; `--chain 1` refuses for all 30). Addresses are the
first three receive + first three change. Core column is Bitcoin Core 31.1
`importdescriptors` of the multipath form into a watch-only descriptor wallet.

| shape | SH2 forms cut | descriptor spelling (K = `[fp/path]xpub/<0;1>/*`) | Nunchuk route | Nunchuk 2.1.1 result | wallet as Nunchuk sees it | addresses match | spend paths in Nunchuk | Core | evidence |
|---|---|---|---|---|---|---|---|---|---|
| single-tr | A 3 md1; B 1 md1 + 3 mk1 | `tr(K)` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | SINGLE_SIG/TAPROOT 1-of-1 | YES (6/6 = md = device = Core 31.1) | 1-of-1 (native multisig) | Core 31.1 import: ok | RUN |
| single-wsh | A 3 md1; B 1 md1 + 2 mk1 | `wsh(pkh(K))` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MINISCRIPT/NATIVE_SEGWIT 0-of-1 | YES (6/6 = md = device = Core 31.1) | 1 signing path(s) | Core 31.1 import: ok | RUN |
| plain-2of3-shwsh | A 6 md1; B 1 md1 + 6 mk1 | `sh(wsh(sortedmulti(2,K,K,K)))` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MULTI_SIG/NESTED_SEGWIT 2-of-3 | YES (6/6 = md = device = Core 31.1) | 2-of-3 (native multisig) | Core 31.1 import: ok | RUN |
| plain-2of3-sh | A 6 md1; B 1 md1 + 6 mk1 | `sh(sortedmulti(2,K,K,K))` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MULTI_SIG/LEGACY 2-of-3 | YES (6/6 = md = device = Core 31.1) | 2-of-3 (native multisig) | Core 31.1 import: ok | RUN |
| plain-2of3-wsh-DEMO | A 6 md1; B 1 md1 + 6 mk1 | `wsh(sortedmulti(2,K,K,K))` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MULTI_SIG/NATIVE_SEGWIT 2-of-3 | YES (6/6 = md = device = Core 31.1) | 2-of-3 (native multisig) | Core 31.1 import: ok | RUN |
| plain-2of3-tr | A 6 md1; B 1 md1 + 9 mk1 | `tr(NUMS,sortedmulti_a(2,K,K,K))` | none | REFUSE: UI shows 'Could not parse descriptor'; internal: invalid miniscript: sortedmulti_a(2,[73c5da0a/48 | - | n/a | - | Core 31.1 import: ok | RUN |
| plain-2of3-wsh-UNSORTED | A 6 md1; B 1 md1 + 6 mk1 | `wsh(multi(2,K,K,K))` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MINISCRIPT/NATIVE_SEGWIT 0-of-3 | YES (6/6 = md = device = Core 31.1) | 1 signing path(s) | Core 31.1 import: ok | RUN |
| preset-simple-timelocked-inheritance-wsh | A 4 md1; B 1 md1 + 4 mk1 | `wsh(or_i(pkh(K),and_v(v:pkh(K),older(26280))))` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MINISCRIPT/NATIVE_SEGWIT 0-of-2 | YES (6/6 = md = device = Core 31.1) | 2 signing path(s) | Core 31.1 import: ok | RUN |
| preset-kofn-recovery-wsh | A 8 md1; B 2 md1 + 9 mk1 | `wsh(or_d(multi(2,K,K,K),and_v(v:pkh(K),older(26280))))` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MINISCRIPT/NATIVE_SEGWIT 0-of-4 | YES (6/6 = md = device = Core 31.1) | 2 signing path(s) | Core 31.1 import: ok | RUN |
| preset-tiered-recovery-wsh | A 8 md1; B 2 md1 + 9 mk1 | `wsh(or_d(multi(2,K,K),and_v(v:multi(1,K,K),older(26280))))` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MINISCRIPT/NATIVE_SEGWIT 0-of-4 | YES (6/6 = md = device = Core 31.1) | 2 signing path(s) | Core 31.1 import: ok | RUN |
| preset-hashlock-gated-wsh | A 5 md1; B 2 md1 + 4 mk1 | `wsh(or_i(and_v(v:pkh(K),sha256(<digest>)),and_v(v:pkh(K),older(26280))` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MINISCRIPT/NATIVE_SEGWIT 0-of-2 | YES (6/6 = md = device = Core 31.1) | 2 signing path(s) | Core 31.1 import: ok | RUN |
| preset-decaying-multisig-wsh | A 9 md1; B 2 md1 + 9 mk1 | `wsh(or_i(and_v(v:multi(2,K,K),older(13140)),or_i(and_v(v:pkh(K),older(` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MINISCRIPT/NATIVE_SEGWIT 0-of-4 | YES (6/6 = md = device = Core 31.1) | 3 signing path(s) | Core 31.1 import: ok | RUN |
| preset-simple-timelocked-inheritance-tr | A 4 md1; B 1 md1 + 6 mk1 | `tr(K,and_v(v:pk(K),older(26280)))` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MINISCRIPT/TAPROOT 1-of-2 | YES (6/6 = md = device = Core 31.1) | 1 signing path(s) | Core 31.1 import: ok | RUN |
| preset-kofn-recovery-tr | A 8 md1; B 2 md1 + 12 mk1 | `tr(NUMS,{multi_a(2,K,K,K),and_v(v:pk(K),older(26280))})` | none | REFUSE: UI shows 'Could not parse descriptor'; internal: Failed to verify wallet descriptor | - | n/a | - | Core 31.1 import: ok | RUN |
| preset-tiered-recovery-tr | A 8 md1; B 2 md1 + 12 mk1 | `tr(NUMS,{multi_a(2,K,K),and_v(v:multi_a(1,K,K),older(26280))})` | none | REFUSE: UI shows 'Could not parse descriptor'; internal: Failed to verify wallet descriptor | - | n/a | - | Core 31.1 import: ok | RUN |
| preset-hashlock-gated-tr | A 5 md1; B 2 md1 + 6 mk1 | `tr(NUMS,{and_v(v:pk(K),sha256(<digest>)),and_v(v:pk(K),older(26280))})` | none | REFUSE: UI shows 'Could not parse descriptor'; internal: Failed to verify wallet descriptor | - | n/a | - | Core 31.1 import: ok | RUN |
| preset-decaying-multisig-tr | A 9 md1; B 2 md1 + 12 mk1 | `tr(NUMS,{and_v(v:multi_a(2,K,K),older(13140)),{and_v(v:pk(K),older(262` | none | REFUSE: UI shows 'Could not parse descriptor'; internal: Failed to verify wallet descriptor | - | n/a | - | Core 31.1 import: ok | RUN |
| keyless-hash-path-wsh | A 3 md1; B 2 md1 + 2 mk1 | `wsh(or_i(pkh(K),sha256(<digest>)))` | none | REFUSE: UI shows 'Could not parse descriptor'; internal: Invalid miniscript: or_i(pkh([73c5da0a/48 | - | n/a | - | Core 31.1 import: REFUSED | RUN |
| keyless-hash-older-path-wsh | A 5 md1; B 2 md1 + 4 mk1 | `wsh(or_d(multi(2,K,K),and_v(v:sha256(<digest>),older(100))))` | none | REFUSE: UI shows 'Could not parse descriptor'; internal: Invalid miniscript: or_d(multi(2,[73c5da0a/48 | - | n/a | - | Core 31.1 import: REFUSED | RUN |
| hash160-gated-wsh | A 5 md1; B 2 md1 + 4 mk1 | `wsh(or_i(and_v(v:pkh(K),hash160(<digest>)),and_v(v:pkh(K),older(26280)` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MINISCRIPT/NATIVE_SEGWIT 0-of-2 | YES (6/6 = md = device = Core 31.1) | 2 signing path(s) | Core 31.1 import: ok | RUN |
| ripemd160-gated-wsh | A 5 md1; B 2 md1 + 4 mk1 | `wsh(or_i(and_v(v:pkh(K),ripemd160(<digest>)),and_v(v:pkh(K),older(2628` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MINISCRIPT/NATIVE_SEGWIT 0-of-2 | YES (6/6 = md = device = Core 31.1) | 2 signing path(s) | Core 31.1 import: ok | RUN |
| hash256-gated-wsh | A 5 md1; B 2 md1 + 4 mk1 | `wsh(or_i(and_v(v:pkh(K),hash256(<digest>)),and_v(v:pkh(K),older(26280)` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MINISCRIPT/NATIVE_SEGWIT 0-of-2 | YES (6/6 = md = device = Core 31.1) | 2 signing path(s) | Core 31.1 import: ok | RUN |
| hashlock-gated-tr-hash160 | A 5 md1; B 2 md1 + 6 mk1 | `tr(NUMS,{and_v(v:pk(K),hash160(<digest>)),and_v(v:pk(K),older(26280))}` | none | REFUSE: UI shows 'Could not parse descriptor'; internal: Failed to verify wallet descriptor | - | n/a | - | Core 31.1 import: ok | RUN |
| older-units-wsh | A 4 md1; B 1 md1 + 4 mk1 | `wsh(or_i(pkh(K),and_v(v:pkh(K),older(4194404))))` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MINISCRIPT/NATIVE_SEGWIT 0-of-2 | YES (6/6 = md = device = Core 31.1) | 2 signing path(s) | Core 31.1 import: ok | RUN |
| after-time-wsh | A 4 md1; B 1 md1 + 4 mk1 | `wsh(or_i(pkh(K),and_v(v:pkh(K),after(1700000000))))` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MINISCRIPT/NATIVE_SEGWIT 0-of-2 | YES (6/6 = md = device = Core 31.1) | 2 signing path(s) | Core 31.1 import: ok | RUN |
| mixed-lock-bases-wsh | A 6 md1; B 1 md1 + 6 mk1 | `wsh(or_i(pkh(K),or_i(and_v(v:pkh(K),older(4194404)),and_v(v:pkh(K),aft` | none | REFUSE: UI shows 'Could not parse descriptor'; internal: Timelock mixing (exception swallowed) | - | n/a | - | Core 31.1 import: ok | RUN |
| mixed-lock-bases-tr | A 6 md1; B 1 md1 + 9 mk1 | `tr(K,{and_v(v:pk(K),older(4194404)),and_v(v:pk(K),after(1000000))})` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MINISCRIPT/TAPROOT 1-of-3 | YES (6/6 = md = device = Core 31.1) | 2 signing path(s) | Core 31.1 import: ok | RUN |
| same-seed-two-accounts-wsh | A 6 md1; B 1 md1 + 7 mk1 | `wsh(sortedmulti(2,K,K,K))` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MULTI_SIG/NATIVE_SEGWIT 2-of-3 | YES (6/6 = md = device = Core 31.1) | 2-of-3 (native multisig) | Core 31.1 import: ok | RUN |
| same-seed-two-paths-tr | A 8 md1; B 2 md1 + 12 mk1 | `tr(NUMS,{multi_a(2,K,K,K),and_v(v:pk(K),older(26280))})` | none | REFUSE: UI shows 'Could not parse descriptor'; internal: Failed to verify wallet descriptor | - | n/a | - | Core 31.1 import: ok | RUN |
| nine-of-nine-wsh | A 18 md1; B 3 md1 + 24 mk1 | `wsh(sortedmulti(9,K,K,K,K,K,K,K,K,K))` | Recover via BSMS/descriptors (file) or QR text | ACCEPT | MULTI_SIG/NATIVE_SEGWIT 9-of-9 | YES (6/6 = md = device = Core 31.1) | 9-of-9 (native multisig) | Core 31.1 import: ok | RUN |

Key-record seeds: the three demo-payload seeds (`abandon…about`,
`zoo…wrong`, `beef×12`), slot *i* = seed *i mod 3* at account *i div 3*
unless the row's note says otherwise; `same-seed-two-accounts-wsh` seats seed
0 at accounts 0 and 1 (the recon's W3 shape); `same-seed-two-paths-tr` seats
seed 0 in both paths at accounts 1 and 2 (the recon's W1 shape).

## Spec claims

| claim | verdict | evidence |
|---|---|---|
| §5c l.251: the raw `H` spelling "is valid in Bitcoin Core" | TRUE | RUN — Core v25 `getdescriptorinfo` ok on `plain-2of3-tr` chain-0; Core 31.1 `getdescriptorinfo` + `importdescriptors` ok on all 7 NUMS multipath forms |
| §5c l.251: "… and imported by Nunchuk" | FALSE | RUN — 0/7 NUMS shapes accepted by `Utils::ParseWalletDescriptor` (I-1); `descriptor.cpp:663-686`, `wallet.cpp:66-80`, `wallet.cpp:214-217` |
| §8f l.711-714: "Bitcoin Core and Nunchuk import this form" | Core TRUE / Nunchuk FALSE | as above; plus the accepted xpub form is a different wallet (different addresses, RUN) |
| §13 item 3: Nunchuk UI treatment of `or_i` vs `or_d` | VERIFIED — identical | SOURCED `src/miniscript/util.cpp:145-156`: `OR_I`, `OR_B`, `OR_C`, `OR_D` all map to `ScriptNode::Type::OR` (with `or_i(0,X)`/`or_i(X,0)` collapsed to X); RUN: `preset-kofn-recovery-wsh` (`or_d`) and `preset-hashlock-gated-wsh` (`or_i`) both render as `or(…)` with 2 signing paths |
| §13 item 3: custom miniscript template imports | VERIFIED with limits | RUN — `wsh(<miniscript>)` with origin keys imports as `MINISCRIPT` (12 shapes); limits: no key-less path (M-2), no mixed lock bases (I-2), no `sortedmulti_a`, no NUMS (I-1) |
| §13 item 4: import tests of composed outputs into Nunchuk | DONE | this report, 30 shapes × 3 spellings |
| recon `composer-recon-same-fingerprint-two-accounts-import.md` §3: `ParseSortedMultiDescriptor` keeps two signers of one fingerprint, nothing collapses | TRUE (now RUN) | `same-seed-two-accounts-wsh` → `MULTI_SIG 2-of-3`, signers `73c5da0a m/48h/0h/0h/2h`, `73c5da0a m/48h/0h/1h/2h`, `3f635a63 m/48h/0h/0h/2h`; addresses match |
| recon §3 caveat: W1/W2 through `IsValidMiniscriptTemplate` unverified | W2 TRUE, W1 FALSE (now RUN) | `preset-kofn-recovery-wsh` is W2's shape (`or_d(multi(2,…),and_v(v:pkh(…),older(26280)))`) → ACCEPT, 4 signers incl. two `73c5da0a`; `same-seed-two-paths-tr` is W1's shape → REFUSE (NUMS, I-1) |
| recon §3 caveat: the four-form round-trip "acceptance risk driven by formatting" | Confirmed as the NUMS mechanism only | RUN — every non-NUMS shape round-trips; `md`'s miniscript text matches Core's `ToString` byte-for-byte after `'`→`h` normalisation |

## Operator runbook — the live check in Nunchuk Desktop 2.1.1

The wallet is the demo payload's own: 2-of-3, native segwit, the three demo
seeds at `m/48'/0'/0'/2'` (`demo/sh2/build-payload.sh`). Everything below is
what the device cuts for it at the tip, regenerated by the device's own code
paths.

Form A (keyed md1, 6 chunks, what "The policy itself" cuts):

```
md1fhs00zspqjtvyyy4qqxppcgsc97v883w6pf8a345cuek52h4ptc0za6p372zc9gwrh7hqaey5ncx636k07
md1fhs00zs2s9tjrg0g2z0agd4urfpzanhaq3lcdlz64mrqgdrha0m7umapumfj075dhzfzqlw6aue5272u2y
md1fhs00zskzfmadfj6e20ur0anz7jwkzae8effc6wcqy5dyvwc0l6d4jhfpw5nxf3dzx8yqpv00rhc3xjew2
md1fhs00zsulcta57626hyxde4ma6qcwgdd2ud5psf2sj7f9cmfvakhfseukrfwaeshgp0tqyeukl9ty0528e
md1fhs00z39q6qrgw2rv8qx4s5s3zn6yywkvjdrgd6t80m8855fr4rnvayq9n63euvhzukas4nh806veapd3t
md1fhs00z3wsp76nmgp8mfc796t857j9qn3emyqkylfwmqez4cnfr4kdyqnds8guskkd0am4asy4pl
```

Form B (keyless template WITH fingerprints, 1 chunk, plus one mk1 card per slot, 2 chunks each):

```
md1fvh00qqpqjtvyyy4qqxppcgsc97v883w6pf8a345cuek52h4qf4ygqqtmj7hts

# card @0  73c5da0a  m/48'/0'/0'/2'
mk1qp3uadpqqsptq26yqwwmtk9kw0za5zs9qjyty8su72t3dwaqcl9pvz58pmltjs9tjrg0g2z0agd4urfpzanhaq3lcdlz63lt6w6u96uevqvg
mk1qp3uadpp2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e20ur0anz7jwkzae8ef3uadqugywhdcdfu7e82j

# card @1  3f635a63  m/48'/0'/0'/2'
mk1qpvghgpqqsptq26yqwwmtk9k8a345cc9qjyty8hww8uv2wxnkqp9rfrrkrl7ndv46gt4yejvtg33ez0u9760d9dtjrxu6cwzp22yl7kr67zy
mk1qpvghgppwlwsxrjrt2hrdqvz25yhjfwx6t8d46vx09s6thwv96qt66sdqp589pkrvghgs6sgx856cjzmhjfw

# card @2  66d455ea  m/48'/0'/0'/2'
mk1qp4wpzpqqsptq26yqwwmtk9kvm29t6s9qjyty85r5le57q6kzjzy20gs36ejf5dphfvalvu7j3yw5wdn5sqk02883jutjgphc3y2e3vfcdtf
mk1qp4wpzppdm6q8m20dqyld8rchfv7n6g5zw88vszcna9mvry2hzdywke5szdkqarj4wpz2lsj95h5emdk9axp

```

Concrete descriptor (`md descriptor` on either form, byte-identical):

```
wsh(sortedmulti(2,[73c5da0a/48'/0'/0'/2']xpub6DXuQW1Q2JpZxsEnFKrPvDuiRMmQgU4fzHU1wsvM5EqgGAWRJ3cmwbtS8u1HQjrEHg3YFb7XGnFovPydJ8qpaGNNd2hSEPoheWd27EABdGH/<0;1>/*,[3f635a63/48'/0'/0'/2']xpub6DXuQW1Q2JpZwZhyeFyRwoVcxxRQUvWjfWf5X5tre7aRCMTYwNR1DnwZAehowmtGsB2oEka2aWofzRgVnexutt2KVBZfRcPtuxS6JYwywD5/<0;1>/*,[66d455ea/48'/0'/0'/2']xpub6DXuQW1Q2JpZw2pTr5epQqR2dceAT9UAeRtrbqNAh24Mrg99k6spPvgiaDoCdEmvzqcka6r5Yfpa4asbrxqr6PbrJ6LVphjMPiZ1iJKrJPm/<0;1>/*))#k9z7pr9l
```

Keyless template (`md decode` of the template chunk): `wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))`

1. **Host.** Put the concrete descriptor (the one line above, checksum
   included) into a plain text file, e.g. `~/sh2-demo-wallet.txt`. Either
   `md descriptor <the 6 keyed md1 chunks>` (form A) or
   `md descriptor <template chunk> --from-mk1 <the 6 mk1 chunks>` (form B)
   prints it; both print the same line (measured). `md` warns on stderr
   `note: stdout is watch-only — public keys only, cannot spend`; that is
   expected.
2. **Nunchuk.** Home → **Add wallet** (or the first-run screen) →
   **Recover existing wallet** → **Recover via BSMS/descriptors** → choose the
   file → give it a name → confirm. (Source: `Qml/Screens/LocalMode/Onboarding/QAddAWallet.qml:246-275`
   sets `recoverType = "recover-via-bsms-config-file"`;
   `Views/STATE_ID_SCR_ONBOARDING.cpp:101-108` → `ImportWalletDescriptor` →
   `nunchuk::Utils::ParseWalletDescriptor`.) The "Recover via QR code" option
   feeds the same parser for a single-frame text QR (`nunchukutils.cpp:487`).
3. **Expect** a wallet of type **Multisig 2/3, Native SegWit**, three keys
   listed with fingerprints **73c5da0a**, **3f635a63**, **66d455ea**, all at
   **m/48h/0h/0h/2h**. Keys that are not already yours appear as hidden
   `import` signers (`storage.cpp:509-516`); that is normal. Nunchuk's
   internal wallet id is `45m69s9l` (the checksum of its `/0/*` descriptor),
   visible in wallet info if shown.
4. **Receive addresses 0, 1, 2 — must match exactly:**
   - `bc1qe84j8r5nucqgke0s53w695a7n4whv268mrnsp2p75u57s0q8zh9qcg4xky`
   - `bc1qneumn7xm855e9axkyj7t7vfpgrz0zmswe4djm5fk5v0f7r3v4sfqe0p00y`
   - `bc1qzw63ms59c4q20j0m6e499ulplmgulcflddks34tdu32n6phn95rse8m2hn`

   **Change addresses 0, 1, 2:**
   - `bc1q3gy9fcmr3zfwujcrj7paehw627qye7kxemmny3k3w0ft9pt3pexsaccayh`
   - `bc1qhqvklrt6q87uqsjxu88te4xstuvf8g7gtj5flarchdu23tlg2w5sc7xvsp`
   - `bc1qemh77zk4fz2j3js02s3d29d5zyt94vrnh3rvt0ghhrx9vmp6c6dq2ysra5`

   These six are what the device (`policyprobe`), `md 0.16.2 address`, Core
   v25, Core 31.1 and libnunchuk a7cfb498 all produced. Any other string at
   index 0 = STOP, report the string.
5. **Second wallet, a taproot miniscript that Nunchuk accepts**
   (`preset-simple-timelocked-inheritance-tr`, internal key = slot 0):

   ```
   tr([73c5da0a/48'/0'/0'/3']xpub6DXuQW1Q2JpZyweiMewTZuMPvjG8hKhV2qoF6wL9VFxsMBExtbfqAAoR4oMG4GyxFzVdfas1v2eAdfLxyjc4Ceo5B6w6zTpf7F2BuXCJ52i/<0;1>/*,and_v(v:pk([3f635a63/48'/0'/0'/3']xpub6DXuQW1Q2Jpa1hNtFUcghdx7Q8kTDsqo7b54YAqZBNCH8EuSvmNSAKbAvkZ4HspgftJ1aqSMeFiZ4sr2QNEGm9geaEre3zDwiJD7C5gx5VH/<0;1>/*),older(26280)))#l7d8tzd8
   ```

   Expect **Miniscript / Taproot**, key path 1-of-1 (`73c5da0a`), one script
   path `and(pk(3f635a63), older(26280))`; receive 0 =
   `bc1pacagu2lz8mxwfrtzcslp5d4knl6c38u029txzwr98dxpjefxte4s90xlrw`, change 0
   = `bc1pn20fzmrjeezkpa0gznusmp3wlgtrduq93ha0yq2d8zhhm8xz889qgfh7qy`.
6. **Third, the expected refusal** (`preset-kofn-recovery-tr`, the NUMS form
   quoted in I-1): the app must show the toast **"Could not parse
   descriptor"**. If it imports, the desktop is not running the libnunchuk
   this report built — record the address it shows against
   `bc1pac935qvs2pj56zc7a0ruawaz2493rpenejg6qzt95znhvtv9pcnq84yvl5`.

## What I ran

- **libnunchuk resolution.** `git ls-tree 2.1.1 contrib/libnunchuk` in a
  blob-less clone of `nunchuk-io/nunchuk-desktop` → `a7cfb498f461b61f4453b56f2aa8efbbf225855c`
  (tag 2.1.1 = `398280183687`, 2025-12-29; the operator's AppImage is
  `nunchuk-linux-v2.1.1_be55150d…`, built 2025-12-29, and its embedded strings
  cite `contrib/libnunchuk/contrib/bitcoin/src/bip324.cpp`, `common/messages.cpp`
  — consistent with a v28-era Core). libnunchuk at that commit pins
  `contrib/bitcoin` = `57b47c47ef0bd36e1c32d709c62998c51dc76f34`
  (`CLIENT_VERSION 28.99.0`, "test: Test MuSig2 in the wallet", 2024-10-29):
  BIP-389 multipath present (`descriptor.cpp:1674 ParseKeyPath(... allow_multipath)`),
  tapscript miniscript present, hardened markers `'`/`h` only (`:1642`).
- **Build.** `/scratch/code/shibboleth/.tmp/fable-nunchuk-lib` at a7cfb498,
  submodules shallow; one scratch patch (`OPENSSL_USE_STATIC_LIBS OFF`, no
  static OpenSSL on this box); cmake 4.4.2 + ninja from nix, gcc 16.2.1,
  Boost **1.86** from nix (`boost186.dev`; system Boost 1.92 and nix 1.91 break
  Core 28.99's `txmempool.h` multi_index — first attempt's log kept), OpenSSL
  3.6.3, libevent 2.1.13. `ninja nunchuk`: **462 steps, 1229 s CPU, 76 s
  wall, 0 errors** → `build/libnunchuk.a` 26.9 MB. Harness
  `fable/harness.cpp` linked as `fableharness` (16.9 MB).
- **Harness stages per input** (`Utils::ParseWalletDescriptor` = the app's
  call; `ParseDescriptors` with its error string; `IsValidMiniscriptTemplate` /
  `IsValidTapscriptTemplate` called directly so swallowed exceptions surface;
  then `get_descriptor` × 4 forms, `get_miniscript`, `GetScriptNode`,
  `GetAllSigningPaths`, `check_valid`, `SanitizeSingleSigners`, and
  `CoreUtils::DeriveAddresses` 0..2 on `EXTERNAL_ALL` and `INTERNAL_ALL`
  through the embedded Core, `EmbeddedRpc::Init("main")`). Input 98 lines =
  30 shapes × {multipath, chain0, chain1} + 8 spelling variants. **Result: 43
  ACCEPT / 55 REFUSE; multipath 20/10; chain0 20/10; chain1 0/30.** Variants:
  `h`-spelling ACCEPT, `H`-spelling REFUSE, no-checksum ACCEPT, `/**` ACCEPT,
  keyless `@0` template REFUSE (both with and without fingerprints), NUMS
  `h`-spelling REFUSE. Second input file (3 lines): the unspendable-xpub NUMS
  rewrites — 1 REFUSE (`sortedmulti_a`), 2 ACCEPT with addresses ≠ device.
  Proof the harness can fail: the same wallet flips ACCEPT→REFUSE on one token
  (`/0/*`→`/1/*`, `'`→`H`), and the Timelock-mixing exception is printed
  verbatim by stage 1b while stage 1 shows `error=''`.
- **Shapes.** `cmd/fablenunchuk/main.go` (kept at
  `/scratch/code/shibboleth/.tmp/fable-nunchuk-gen.go`) in a detached
  worktree of the fork at `f5b068fa`: 30 shapes, all four wrappers, the five
  presets under `wsh` and `tr`, all four hash kinds, key-less hash paths,
  `older` units / `after` time / mixed bases, unsorted, 9-of-9, same-seed
  cases. Emits keyed chunks (form A), template chunks with fingerprints and
  one `mk.Encode(mk.AppendStubs(card, md.ComposerStubs(...)))` card per slot
  (form B), and the template-only chunks. 30/30 composed.
  `CGO_ENABLED=0 go build` with Go 1.26.7. Worktree removed afterwards
  (`git worktree remove --force`); the fork checkout is untouched.
- **Device leg.** `cmd/policyprobe` on the 30 keyed chunk sets, indices
  0..2: **30/30 ok**, stage errors 0.
- **Host leg** (`fable-nunchuk-host.py`, `md 0.16.2` by `~/.cargo/bin/md`):
  `md descriptor` multipath/chain0/chain1, `md decode` of template and
  template-only chunks, `md descriptor <template> --from-mk1 <cards>`,
  `md address --chain {0,1} --count 3 --json`. **formA == formB 30/30;
  device == md 30/30** (receive and change).
- **Bitcoin Core v25.0.0** (`/usr/local/bin/bitcoind`, mainnet-mode,
  `-connect=0`, height 0, rpcport 18643, private datadir under `.tmp`,
  stopped): `getdescriptorinfo` + `deriveaddresses [0,2]` on the chain-0
  forms: **21 == md; 9 refused** (7 tr-miniscript "Miniscript expressions can
  only be used in wsh" / tapscript leaves, 2 key-less hash paths).
- **Bitcoin Core 31.1.0** (`nix build nixpkgs#bitcoind`, same ports after
  v25 stopped, stopped afterwards): `getdescriptorinfo` on multipath **28 ok /
  2 refused** (key-less); `deriveaddresses` chain0+chain1 **28/28 == md**;
  `createwallet fable31` (descriptors, no private keys) +
  `importdescriptors` of every multipath form **28 ok / 2 refused**;
  `listdescriptors` = 56.
- **Desktop route.** `nunchuk-desktop` tag 2.1.1: `qUtils::ParseWalletDescriptor`,
  `OnBoardingModel::ImportWalletDescriptor`, `QAddAWallet.qml` labels
  (`STR_QML_037 "Recover via BSMS/descriptors"`, `STR_QML_038 "Recover via QR code"`).
- Nothing was written under `$HOME`; `DISPLAY=:0` never used; no `.jsonl`
  transcript read; nothing built under `/tmp`.

## What I could not verify

- **The Qt UI itself.** Nothing was clicked; the AppImage was never
  executed. Every "Nunchuk" cell is libnunchuk `a7cfb498` run in-process,
  which is what the desktop links (evidence: the tag's submodule pointer and
  the AppImage's embedded source paths), but the toast text, the tree
  rendering and the key list are inferred from `qUtils.cpp:206-218`,
  `MiniscriptWallet.cpp` and `storage.cpp`, not seen. The runbook exists so the
  operator closes that gap by eye.
- **Signing.** Whether a hardware signer added later links to an `import`
  signer and signs from these wallets (the recon's `get_signers()`-keyed-by-xfp
  progress view) was not exercised; no Nunchuk instance/DB was created.
- **Nunchuk mobile** (same libnunchuk, different pin) — not measured.
- **BSMS** route — not fed; the device emits no BSMS.
- **Why the uppercase-`H` spelling throws** inside `ParseDescriptors` — not
  traced (swallowed exception; `md` never emits it).
- **Exact AppImage ⇄ libnunchuk commit identity.** The desktop tag's
  submodule pointer is the best available evidence; the AppImage's md5 suffix
  does not resolve to a commit.

## Out of lens

- **Lowering (§5):** a sole plain path under `tr` renders `sortedmulti_a`,
  while the same k-of-n inside a multi-leaf tree renders `multi_a`
  (`preset-kofn-recovery-tr`). Both are BIP-388-legal; noting only that the
  first form is the one no Nunchuk route can ever take.
- **Brief wording:** the lens brief's "raw `H` hardened spelling" reads §5c's
  NUMS-point sentence as a hardened-marker claim (N-1).
- `md descriptor`'s stderr does not mention the zero parent fingerprint the
  `--help` text documents (F-611); harmless here since Nunchuk ignores the
  header bytes.
- The libnunchuk build tree (`/scratch/code/shibboleth/.tmp/fable-nunchuk-lib`,
  430M with submodules and build) and the harness are left in place as evidence;
  every other artifact of this review is under
  `/scratch/code/shibboleth/.tmp/fable-nunchuk-*`.
