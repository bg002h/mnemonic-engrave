# PLAN: exporting "our reasonably complex wallet" to Nunchuk

Agent report, 2026-08-22. Planning only — no code was written or modified.

Evidence base: libnunchuk source at commit `d6019019e959b8824a7b914b26e3165e88fd74dc`
(clone tip 2026-08-21 — one day old) [verified: git log of shallow clone]; Bitcoin Core
v25.0 `getdescriptorinfo` on the running mainnet node (Bitcoin Satellite v0.2.4 fork of
Core v25.0.0) [verified: `bitcoin-cli getnetworkinfo`]; `mnemonic export-wallet` and `md`
binaries installed on this box, executed today; nunchuk.io blog pages fetched 2026-08-22.

**A brief correction landed mid-flight:** the original brief's settled fact "no export
surface exists anywhere in ms/md/mk/me today" omitted the fifth repo. `mnemonic
export-wallet` (mnemonic-toolkit, since v0.97.0) emits 11 watch-only formats including
`bsms` and `descriptor`. This report is written against that command as the thing being
extended, per the correction. Nunchuk is not among its 11 formats [verified: `mnemonic
export-wallet --help`].

---

## VERDICT

**Nunchuk cannot import this wallet, in either wrapping.** The single defeating feature
is **tier 4 — the keyless `sha256(H3)`-only spend path**. Every libnunchuk import and
creation surface requires the vendored Bitcoin Core miniscript `IsSane()`, whose
definition includes `NeedsSignature()`:

```cpp
bool NeedsSignature() const { return GetType() << "s"_mst; }              // miniscript.h:1630
bool IsSane() const { return IsValidTopLevel() && IsSaneSubexpression() && NeedsSignature(); }  // miniscript.h:1645
```
[verified: src/miniscript/miniscript.h in libnunchuk @ d6019019]

- **wsh: NO.** The `or_i` chain only has the `s` (signed) property if *every* branch has
  it; tier 4 has none, so the whole script fails `NeedsSignature()`.
  Empirically confirmed with Bitcoin Core v25 (identical vendored checker), see the
  minimal-pair experiment below [verified: executed `getdescriptorinfo`].
- **tr: NO, twice over.** (1) The tier-4 tapleaf fails the same per-leaf sanity gate.
  (2) Independently, Nunchuk cannot represent a **fixed raw NUMS internal key** at all —
  its taproot miniscript wallets always re-render the internal key as a *ranged
  unspendable xpub* derived per address index, which yields different taproot tweaks and
  therefore different addresses than our `tr(50929b74…,…)` wallet. So even a
  tier-4-repaired tr variant is not this wallet in Nunchuk. Details below
  [verified: source reading; the address consequence is [unverified: reasoning] from
  BIP-341 tweak mechanics].
- **Hot wallet: NO**, by inheritance — Nunchuk's hot-key machinery (software signers
  from seed words or master xprv, custom derivation paths, sha256-preimage injection at
  spend time) all exists and would suffice, but there is no wallet to attach the keys to.

**However: the export side is already done.** For any Nunchuk-*representable* wallet
(all spend paths signed, wsh — or taproot in Nunchuk's own internal-key convention),
`mnemonic export-wallet --format descriptor` and `--format bsms` already emit exactly the
file shapes Nunchuk imports. No new emitter is needed; what is missing is not in the
constellation, it is in the wallet. A well-evidenced "impossible" is the answer here.

---

## 1. What Nunchuk accepts on import (file formats)

All Nunchuk apps (desktop/iOS/Android) sit on libnunchuk. The file-import entry point is
`Nunchuk::ImportWalletDescriptor(file_path, name, description)` → whole file contents →
`Utils::ParseWalletDescriptor(descs)` [verified: src/nunchukimpl.cpp:459-463]. That
function tries, in order [verified: src/nunchukutils.cpp:414-449]:

1. **BSMS 1.0 record** (`ParseBSMSRecord`, src/utils/bsms.hpp:49-79). Exactly four lines:
   - line 1: literally `BSMS 1.0`
   - line 2: the output descriptor (checksum tolerated; multipath `<0;1>` accepted)
   - line 3: literally `/0/*,/1/*` **or** `No path restrictions` (nothing else parses)
   - line 4: the first external address — Nunchuk re-derives and refuses on mismatch.
2. **Bare output descriptor** (`ParseDescriptors`, src/descriptor.cpp:663-686). First
   line of the file, `#checksum` stripped. Recognized prefixes: `tr(`, `wsh(`,
   `wsh(sortedmulti(`, `sh(wsh(sortedmulti(`, `sh(sortedmulti(`, `sh(wpkh(`, `wpkh(`,
   `pkh(` [verified: PREFIX_MATCHER + ParseOutputDescriptors, descriptor.cpp:620-657].
3. **JSON** `{"descriptor": "...", "label": "..."}` (`ParseJSONDescriptors`,
   descriptor.cpp:688-707).
4. **Multisig config files** (Unchained / Coldcard-style key-value text) — plain
   multisig only, cannot express miniscript [verified: fallback in nunchukutils.cpp:437].

Key syntax requirements: every xpub MUST carry a key origin `[xfp/path]` — the parser
regex demands it and the error message says so explicitly ("Note that key origin is
required for XPUB") [verified: SIGNER_REGEX + ParseSignerString, descriptor.cpp:394,
405-420]. Hardened markers `'` and `h` are both fine — the comparison normalizes `'`→`h`
and lowercases [verified: GetDescriptorWithoutChecksum, descriptor.cpp:422-428].
Child-path forms accepted are the four Nunchuk itself renders: `/0/*`, `/<0;1>/*`, `/*`,
`/**` — the import re-renders the parsed wallet and requires the input line to equal one
of those four renderings byte-for-byte after normalization ("Failed to verify wallet
descriptor" otherwise) [verified: DESCRIPTOR_PATHS loop, descriptor.cpp:658-686].

Nunchuk's own docs confirm the user-facing surface: wallet configuration files "either
in Output Descriptors or BSMS format", raw-descriptor files where "the wallet descriptor
is the entire file content", multipath `<0;1>/*` syntax, and a paste-in "Custom Script"
path for raw miniscript [verified: nunchuk.io/blog/miniscript101 and
/blog/miniscript-wallet-recovery, fetched 2026-08-22]. File extension is not interpreted
by libnunchuk (it reads the path it is given); what the mobile/desktop file pickers
filter on was not checked [unverified: UI repos not audited].

Miniscript support level: real. sha256/hash256/ripemd160/hash160 fragments parse
[verified: miniscript.h:1953-1980], `after`/`older` both supported, taproot script trees
supported, and preimages are injected into the PSBT at spend time via a first-class
`RevealPreimage(wallet_id, tx_id, hash, preimage)` API that validates the preimage
against all four hash types [verified: NunchukImpl::RevealPreimage, nunchukimpl.cpp].

## 2. Why this wallet cannot be represented — the crux

Every route to a live Nunchuk wallet funnels through the same validator,
`Utils::IsValidMiniscriptTemplate`:

```cpp
bool isValid = node && node->IsValidTopLevel() && node->IsSane() && !node->IsNotSatisfiable();
```
[verified: src/nunchukutils.cpp:1382-1390]

Call sites covering all surfaces [all verified by grep + read at d6019019]:
- wsh descriptor import: `ParseWshDescriptor` (descriptor.cpp:594-616) — whole script;
- tr descriptor import: `ParseTrDescriptor` → `IsValidTapscriptTemplate` — **per tapleaf**
  (descriptor.cpp:556; nunchukutils.cpp:1392-1435);
- BSMS import: line 2 → `ParseDescriptors` → same two functions (bsms.hpp:63);
- JSON import: → `ParseDescriptors` → same;
- paste-in creation: `CreateMiniscriptWallet` gates every input class through
  `IsValidMiniscriptTemplate` / `IsValidTapscriptTemplate`, and its policy-compiler path
  checks `IsSane()` on the compilation result (compiler.cpp:968);
- `MiniscriptTemplateToMiniscript` re-checks `IsSane()` (nunchukutils.cpp:1451).

Tier 4 is `and_v(v:after(1383520),sha256(H3))` — no key, so no `s` property, so
`NeedsSignature()` is false: as a tapleaf it is insane on its own, and in wsh it strips
the `s` property from the whole `or_i` chain (`or_i` is `s` only if both arms are).
The property algebra is [unverified: reasoning from the miniscript type system], but the
conclusion is machine-checked twice:

**Minimal-pair experiment, run today** against Bitcoin Core v25 `getdescriptorinfo`
(same miniscript code libnunchuk vendors; concrete keys = the six hashvault-journey
xpubs; `/0/*` child paths because v25 lacks multipath — structure is what is judged):

| descriptor | verdict |
| --- | --- |
| our wsh, keyless tier 4 | **rejected**: `is not sane: witnesses without signature exist` |
| identical except tier 4 = `and_v(v:after(1383520),and_v(v:sha256(H3),pk(<key>)))` | **accepted**, checksum `yyrg9eh6`, `issolvable: true` |

[verified: both commands executed 2026-08-22; full outputs in session transcript]

So the hashlocks, both timelock flavours, the 3/2/1 thresholds, the six keys, the
`or_i` nesting, and the script size are all fine — **the keyless branch alone defeats
it**. rust-miniscript agrees from the other side: `md encode` on the tr policy refuses
with exactly `All spend paths must require a signature` [verified: executed today;
message text originates in miniscript-12/13 `analyzable.rs` (`SiglessBranch`)].

### The tr form's second, independent defeater

Nunchuk recognizes a raw `50929b74…03ac0` internal key on parse — that exact constant is
its `H_POINT` [verified: descriptor.h:28 — identical to our NUMS pin]. But a taproot
miniscript wallet whose keypath is unspendable is stored as `DISABLE_KEY_PATH`
[verified: Wallet miniscript ctor, dto/wallet.cpp:66-81] and **re-rendered** with
`GetUnspendableXpub(signers)` — an xpub whose root pubkey is `02||H_POINT` and whose
chaincode is SHA256 over the sorted signer pubkeys, then child-derived per address
(`…/<0;1>/*`) [verified: dto/wallet.cpp:189-230; descriptor.cpp:726-753]. Consequences:

1. Our exact descriptor (raw fixed H_POINT) fails the import round-trip equality — all
   four renderings carry the xpub form, so `ParseDescriptors` returns "Failed to verify
   wallet descriptor" and the import throws "Could not parse descriptor"
   [verified: source reading of the compare loop; not executed — building libnunchuk was
   out of budget, flagged in open questions].
2. No Nunchuk-representable taproot wallet has a *fixed* internal key across indices:
   the derived child of the unspendable xpub differs per index, the BIP-341 tweak
   differs, and every address differs from our fixture's (`bc1puvyd9…`)
   [unverified: reasoning from BIP-341; the rendering facts feeding it are verified].

So for tr the answer is categorical: not this wallet, not a repaired variant, not at
these addresses.

### Checks that do NOT bite (worth recording so nobody re-derives them)

- **Timelock mixing:** Nunchuk's `MiniscriptTimeline` throws "Timelock mixing" if
  height-based and time-based locks coexist anywhere in one wallet — stricter than
  Core's per-path rule — but all three of our locks are height-based (32768 has bit 22
  clear; 1173520 and 1383520 < 500000000), so it passes
  [verified: src/miniscript/timeline.cpp:31-38 + lock values].
- **Satisfiability:** `IsNotSatisfiable()` is `!GetStackSize()` — structural, and hash
  preimages count as available [verified: miniscript.h:1550].
- **Size/resource limits:** enforced via `IsSaneSubexpression()`; the keyed control
  passed them with room (whole descriptor 1.4 kB, script well under standardness caps)
  [verified: control accepted by Core].

## 3. The exact file format, with a worked example

Moot for this wallet (verdict NO), but load-bearing for any representable neighbour and
for the "is the constellation ready" question. The file Nunchuk wants is **one line,
the whole file, concrete keys with origins, multipath `<0;1>`, checksum optional**:

```
wsh(or_i(and_v(v:sha256(ede0…c813),multi(3,[39ec1b6e/270028'/0'/0'/1']xpub6E6…Kt88/<0;1>/*,…)),or_i(…,and_v(v:after(1383520),and_v(v:sha256(4743…6954),pk(<KEY7>/<0;1>/*))))))#<checksum>
```

(that is: the keyed-tier-4 *control* shape — labeled clearly: it is a DIFFERENT wallet,
shown only to pin the format; the full accepted control descriptor with checksum
`yyrg9eh6` is in the transcript.)

`mnemonic export-wallet` already emits both Nunchuk-accepted shapes today, verified by
running it on a concrete wsh descriptor:

- `--format descriptor` → exactly one stdout line, `[fp/270028'/0'/0'/1']xpub…/<0;1>/*`
  keys, `#checksum`, the watch-only note on stderr [verified: executed today].
- `--format bsms` → exactly Nunchuk's four-line record: `BSMS 1.0` / descriptor /
  `/0/*,/1/*` / first address [verified: executed today — the line-3 token is
  byte-identical to one of the two strings bsms.hpp:66-69 accepts].

Byte-equality of the toolkit's descriptor line against Nunchuk's re-rendering (the
round-trip verify) is [unverified: reasoning — both sides emit canonical miniscript with
`[xfp/path]xpub/<0;1>/*` keys and the comparison normalizes case and hardened markers,
but no import into a running Nunchuk was executed; listed as the top open question].

Notably, the toolkit accepted the **keyless** wsh form (`--format descriptor` emitted
our real wsh wallet with checksum `#tf2w8zn8`) — the toolkit does not run the
requires-sig sanity rule on wsh. Useful for Core/Sparrow-class targets; wasted on
Nunchuk, which re-checks and refuses. The **tr form cannot even be emitted** today:
`export-wallet --descriptor tr(…)` fails with `All spend paths must require a signature`
(rust-miniscript tapleaf analysis at parse) [verified: executed today].

## 4. Hot wallet

Nunchuk has no combined descriptor+secrets import file. Hot = watch-only wallet import
**plus** per-key software signers, matched by master fingerprint + derivation path:

- `CreateSoftwareSigner(name, mnemonic, passphrase, …)` — BIP-39 words, or
- `CreateSoftwareSignerFromMasterXprv(name, master_xprv, …)` — master xprv
  [verified: include/nunchuk.h:1909-1919], with arbitrary hardened paths available via
  `GetSignerFromMasterSigner(mastersigner_id, path)` [verified: nunchuk.h:~1608], so the
  nonstandard `m/270028'/…` origin is not a blocker for software keys.

So the secret material is **the six seed mnemonics themselves (or six master xprvs),
entered one signer at a time** — the constellation's existing seed artifacts, nothing
new to emit. Spending tiers 1/2/4 additionally requires presenting H1/H2/H3 preimages at
transaction time (`RevealPreimage`); those live in the journeys inputs, not in any
import file. Security consequence, in one sentence: entering all six seeds makes every
key tier hot in one app on one machine — the four-tier degradation story collapses to
"whoever owns that device owns tiers 1-3 immediately", and the H3 preimage becomes the
only cold secret left. All of this is moot for this wallet: the wallet import it hangs
off is refused.

## 5. What the constellation must compute that it does not compute today

Against `mnemonic export-wallet` as the extension point:

1. **Nothing, to reach Nunchuk for representable wallets.** `--format bsms` and
   `--format descriptor` are already Nunchuk's two import shapes. A dedicated `nunchuk`
   format would add nothing — recommend explicitly NOT adding one, and at most
   documenting "Nunchuk: use `--format bsms` (or `descriptor`)" once the open question
   below is closed by one live import test.
2. **Nothing can reach Nunchuk for THIS wallet.** Not an emitter gap; the wallet's
   tier-4 keyless path is outside Nunchuk's (and Core's) sanity envelope, and the tr
   internal-key convention is additionally unrepresentable.
3. If tr-form *emission* (for other targets) ever matters: `export-wallet --descriptor`
   currently cannot parse the tr fixture at all (sigless-branch refusal at parse) — an
   `--experimental`-style relaxation would be a toolkit change. File it only if a
   consumer for the tr export exists; Nunchuk is not that consumer.

## 6. Nunchuk-side limits that bite (or don't)

- Keyless spend paths: **fatal**, every surface (the crux above).
- Fixed raw NUMS internal key in tr: **fatal** for import round-trip and for address
  equality [verified/reasoning split as in §2].
- sha256 preimage branches: supported, including at spend time (`RevealPreimage`)
  [verified: source]; whether every app UI exposes the reveal flow was not checked
  [unverified].
- Wallet-wide height-vs-time timelock mixing: refused (stricter than Core); ours is
  all-height, unaffected [verified: timeline.cpp].
- Key origins: mandatory on every xpub [verified: parser].
- Descriptor length: no Nunchuk-specific cap found beyond Core's miniscript resource
  limits; our 1.2-1.4 kB descriptors are far inside them [verified: control accepted].
- Watch-only display of unsignable policies: import does not require owned keys
  (signers become remote/airgap entries), so a watch-only import of a *sane* wallet
  displays fine [verified: import path constructs SingleSigners from the descriptor
  alone; UI rendering itself unverified].

## 7. Open questions

1. **No live end-to-end import was executed.** The rejection verdicts rest on
   libnunchuk source at d6019019 plus Core-v25 execution of the identical vendored
   checker; the acceptance claim for toolkit-emitted files (round-trip byte-equality,
   §3) is reasoning only. One test with an actual Nunchuk build (or a compiled
   libnunchuk harness calling `Utils::ParseWalletDescriptor` on the two fixture
   descriptors and one toolkit `bsms` file) would convert both to verified. Building
   libnunchuk (embedded Bitcoin Core, ~9 contrib deps) was out of budget this pass.
2. **Path discrepancy in the brief.** The dispatch brief says tr at
   `m/270028'/0'/8'/0'` and wsh at `m/270028'/0'/9'/1'`; the on-disk fixture policies
   and README say `m/270028'/0'/0'/0'` (tr) and `m/270028'/0'/0'/1'` (wsh)
   [verified: read both]. This report used the on-disk fixture. Does not change any
   verdict (paths are opaque to every check involved), but someone should reconcile.
3. **Fixture README staleness (side observation):** it prescribes
   `md encode --experimental`, but the installed `md` has no such flag on `encode`, and
   the wsh policy encodes keyless *without* any flag while tr refuses with no bypass
   [verified: executed today]. Worth a follow-up in the fixture's own repo hygiene, not
   here.
4. **Minimum Nunchuk app versions** carrying miniscript support: announced 2025
   [verified: bitcoinmagazine.com coverage + nunchuk.io blog]; exact version numbers per
   platform not confirmed.
5. **QR (BC-UR) and group-wallet import paths** were not audited to the same depth as
   file import; every wallet-construction route found does pass through the same
   `IsSane()` gate, and address derivation ultimately uses the embedded Core descriptor
   parser, so a bypass that yields a *usable* wallet is [unverified: reasoning]
   implausible but not proven.
