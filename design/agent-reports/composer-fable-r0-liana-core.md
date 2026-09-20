# Composer fable review r0 — Lens 5: IMPORTABILITY into Liana and Bitcoin Core as working wallets

**Verdict: 0 Critical / 3 Important / 4 Minor / 3 Nit.** Every shape the composer can produce imports into
Bitcoin Core 31.1 as a working wallet (54 of 56; the two key-less shapes are refused) with receive AND change
addresses byte-equal to `md 0.17.0` and every spend path — primary, immature/matured `older`, sha256 hashlock
via a hand-spliced `PSBT_IN_SHA256`, taproot key path, NUMS script paths — spent through the wallet's own
`walletcreatefundedpsbt → walletprocesspsbt → finalizepsbt → sendrawtransaction`; Core 25.0 imports the
`wsh`/`sh`/`sh(wsh)`/tree-less-`tr` subset (39 of 54) and spends the `wsh` journeys; Liana 8.0 imports
**17 of 56** (three of the five non-plain presets under `wsh`, one under `tr`, plus `older`-only variants),
with addresses equal to `md` and every primary and recovery path spent through `lianad` + Liana's own signer —
but it refuses the demo payload's own wallet and every other shape in nine classes of which the device names
three, and it silently re-reads one composable three-path policy as "2 of 4 keys" so its GUI can never spend
the single-key path the device promised; four spec sentences about Core/Liana are false or stale as written.

Reviewer: fable-tier, independent, lens 5 (requested after lenses 1-4 closed). Fork tip reviewed
`f5b068faf3049ccf97603dfbaa3709f12893df97` (detached worktree `.tmp/fable-liana`, removed at the end);
host `md 0.17.0` (md-codec 0.45.0) as shape generator and address oracle; Liana `v8.0` (`9d2fb742`) built
from source; Core 31.1.0 (nix) and 25.0.0 (`/usr/local/bin`, "Bitcoin Satellite v0.2.4"). Read-only on every
repo; nothing committed; no sub-agents; no `.jsonl` transcript read; nothing under `~/.liana*` or `/tmp`.
Evidence files: `/scratch/code/shibboleth/.tmp/fable-liana-*` (scripts, JSON/JSONL results, logs, the
harness crate, Liana source/target, per-wallet lianad dirs).


## Findings

### I-1 (Important) — Liana imports 17 of 56 composable shapes; the device names Liana on three of the nine refusal classes, and the silent six include the demo wallet's own class and the `after`-dated inheritance an operator is most likely to build

**Inputs.** All 56 shapes (30 device-composed at fork tip `f5b068fa`, decoded by `md 0.17.0`; 26 composed by
`md compose` 0.17.0 and seated with the demo seeds), `md descriptor` multipath spelling, through
`LianaDescriptor::from_str` of the `liana` crate at tag `v8.0` (`9d2fb742`), which is the exact call the GUI's
"Import the wallet" screen makes (`gui/src/installer/step/descriptor/mod.rs:47`).

**Observed (RUN, harness `parse`).** 17 ACCEPT / 39 REFUSE. The 39 refusals fall into nine classes. The
device's copy at the tip says "Liana" in exactly three strings (`gui/composer_copy.go:151,161,169` = §8f NUMS
and the two §8g same-seed bodies); the spec's folded §8a (l.673) adds the key-less class. Those are the rows
marked ✓ — everything else is silent:

| class | shapes | Liana error (verbatim, shortened) | where in Liana | device says |
|---|---|---|---|---|
| sole unlocked multi-key path, any wrapper (`sortedmulti`), or any policy with no lock at all | plain-2of3-{wsh,sh,shwsh,tr,UNSORTED}, single-{wsh,tr}, same-seed-two-accounts-wsh, nine-of-nine-wsh, X04, X05 | `Descriptor is not compatible with a Liana spending policy.` / `A Liana policy requires at least one recovery path.` | `analysis.rs:554-558`, `:472-474`, `:583` | nothing |
| `sh`, `sh(wsh)` (only composable as plain multisig anyway) | plain-2of3-sh, -shwsh | `IncompatibleDesc` | `:586-587` | nothing |
| any hash, keyed or key-less, any kind, any wrapper | preset-hashlock-gated-{wsh,tr}, hash160/ripemd160/hash256-gated-wsh, hashlock-gated-tr-hash160, keyless-hash-{path,older-path}-wsh, X15, X20 | `IncompatibleDesc` | `:186-199`, `:212-257` (only `Key`/`Threshold`/`Older` are matched) | ✓ key-less only (§8a as folded in the spec; the tip copy `composerCopyKeylessPath` does not name Liana yet); nothing for the eight KEYED hash shapes, the `hashlock-gated` preset included |
| `after(...)` in any path | after-time-wsh, mixed-lock-bases-{wsh,tr}, X07 | `IncompatibleDesc` (or `InsaneTimelock` when the same policy also carries units) | `:212-257` | nothing (§8w names Nunchuk and Core only) |
| `older` in 512-second units | older-units-wsh, X06, mixed-lock-bases-* | `Timelock value '4194404' isn't valid or safe to use` | `csv_check :139-145` | nothing |
| NUMS internal key (raw `H`) | preset-kofn/tiered/hashlock/decaying-tr, hashlock-gated-tr-hash160, same-seed-two-paths-tr, X19, X20, plain-2of3-tr | `IncompatibleDesc` (plain-2of3-tr: rust-miniscript `unexpected «sortedmulti_a(4 args) while parsing Miniscript»` first) | `:568-569` | ✓ (§8f) |
| no unlocked path (every path timelocked) | preset-decaying-multisig-{wsh,tr}, X17, X21 | `IncompatibleDesc` | `:633` | nothing |
| two recovery paths with the same `older` | X03 | `IncompatibleDesc` | `:624-626` | nothing |
| same fingerprint twice inside one multi path | X11 | `Key '[73c5da0a/48'/0'/1'/2']xpub…' is derived from the same origin as another key present in the same spending path…` | `:506-515` | ✓ (§8g) |
| a second unlocked path when the first is a single key, or a second unlocked MULTI path | X05, X25, X26 | `IncompatibleDesc` | `:611-616` | nothing |

**Expected.** By this lens's brief a composable wallet a coordinator cannot import with no notice on the
device is Important. The spec already KNOWS two of the silent classes (§13 item 4: "Liana's import
refuses any `after` or hashlock path regardless of head ... F-449's acceptance wallet must be
`older`-only") and the device names Liana in §8f and §8g as if it were a supported target, yet an operator
who composes "me now, my heir after 2027-01-01" (`after`, the most natural inheritance clock) or the
`hashlock-gated` preset with a KEYED hash path is told nothing and finds out at "Failed to read the
descriptor" — Liana shows no reason (`step/descriptor/mod.rs:47` drops the error). The demo payload's own
wallet is in the first, silent class.

**One-sentence remedy (not authoritative).** One consent notice, in §8w's style, when a policy is outside
Liana's model, stating the class (no recovery path / hash / `after` / time units / no unlocked path /
duplicate lock / second unlocked path), so the operator picks the `wsh` `older`-only shape before cutting
steel; the classes above are the complete list for v8.0.

### I-2 (Important) — Liana imports `[2-of-3, 1 key, 1 key after N]` as a DIFFERENT policy ("2 of 4 keys"), and its GUI can never spend the single-key path the device promised

**Inputs.** X24: `md compose --wrapper wsh --path 2of3 --path 1of1 --path 1of1,older=100`, seated with four
distinct seeds (demo 0,1,2 in the 2-of-3; the fourth seed `b8688df1` = `legal winner … yellow` alone; demo 0
at account 4 as the heir). The composer admits it (no EXPERIMENTAL mark), lowers it as
`wsh(or_d(multi(2,A,B,C),or_i(pkh(D),and_v(v:pkh(E),older(100)))))`
(`md compose --json`: `experimental: []`, template
`wsh(or_d(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*),or_i(pkh(@3/<0;1>/*),and_v(v:pkh(@4/<0;1>/*),older(100)))))`);
the device's own port admits and composes the same list (`md.ValidatePathList` + `md.Compose` at `f5b068fa`,
RUN in a scratch test, `TestLens5X24Admitted`: X24 `slots=5 experimental=[]`, X25 `slots=5 experimental=[]`, PASS, file removed). Three paths: 2-of-3, D alone, E after 100 blocks.

**Observed (RUN).** `LianaDescriptor::from_str` → ACCEPT with `primary = Multi(2, [A, B, C, D])`,
`recovery = {100: Single(E)}` (harness output: `primary multi 2 [4 keys]`). Mechanism: the semantic policy
normalises to `thresh(1, thresh(2,A,B,C), pk(D), thresh(2, pk(E), older(100)))`; `from_multipath_descriptor`
takes `thresh(2,A,B,C)` as the primary path and then folds the bare `pk(D)` into it with
`PathInfo::with_added_key` (`analysis.rs:611-613`, `:261-269`), which appends the key **without changing the
threshold**. Addresses are unaffected (Liana keeps the descriptor as given; receive/change 0..2 == md ==
Core 31.1). Through `lianad` 8.0 on regtest: `createspend` → sign with seed `b8688df1` ONLY → `updatespend` →
`broadcastspend` → **accepted, in Core's mempool, witness `[71, 33, 1, 0, 0, 0, 164]`** (the `pkh(D)` branch) —
the daemon's finalizer (`commands/mod.rs:707`, rust-miniscript `finalize_mut`) satisfies the real script. But
Liana's signature accounting for that PSBT is `primary: threshold 2, sigs_count 1` (`partial_spend_info`),
and the GUI offers **Broadcast** only when `path_ready()` is `Some` (`gui/src/app/view/psbt.rs:378-393`),
i.e. `sigs_count >= threshold` on some path (`gui/src/daemon/model.rs:190-199`) — so the GUI shows the spend
as "1 more signature from …" with a **Sign** button and never a Broadcast button. The reverse order,
`[1of1, 2of3, older]` (X25) and the `tr` form (X26), are refused (`IncompatibleDesc`, `:615`), so only this
order is silently re-read.

**Expected.** The device promised "D alone spends"; Liana displays a wallet in which D alone cannot (GUI) and
counts D as a fourth member of the 2-of-3. Not a wrong address, so not Critical by the brief's rule; a real
mis-import of a composable wallet with nothing on the device, Important. (Whether Liana's reading is a Liana
bug is Liana's business; the SH2 side can only refuse or warn on the shape: a multi-key unlocked path followed
by a single-key unlocked path.)

### I-3 (Important) — §13 item 4 claims every fully seated shape "imports into Core v25/v31.1"; on v25 every `tr` policy with a tapscript-miniscript leaf is refused

**Inputs.** The 54 non-key-less shapes, re-versioned to `tpub`, through the WALLET journey on the local
Core (`/usr/local/bin/bitcoind`, Bitcoin Satellite v0.2.4 = Core v25.0.0): `getdescriptorinfo` of
`md descriptor --chain 0` and `--chain 1` (v25 has no BIP-389), `createwallet` (descriptors, no private keys),
`importdescriptors` (chain 1 `internal: true`), `getnewaddress`/`getrawchangeaddress` ×3.

**Observed (RUN).** 39 of 54 import (`listdescriptors` = 2, receive+change 0..2 == md for 39 of 39); 17 refused at `getdescriptorinfo`: {'no tapscript miniscript ("Miniscript expressions can only be used in wsh")': 12, 'no tapscript miniscript (a `multi_a` leaf: "… is not a valid descriptor function")': 3, 'key-less ("witnesses without signature exist")': 2}. The refused ones are every `tr` policy with a tapscript-miniscript leaf; the two tree-less `tr(NUMS, sortedmulti_a(...))` / `tr(K)` shapes import. Spend leg on v25 (S1–S4, `wsh`): 13 attempts, 7 SENT, 3 `non-BIP68-final`, 3 incomplete, 0 other.

**Expected.** The sentence at `SPEC_wallet_policy_composer.md:1219-1222` reads as "v25 AND v31.1" (on v25 15 of the 17 `tr` shapes are refused, all with "Miniscript expressions can only be used in wsh"; the 2 tree-less ones import); lens 1
already recorded 25 v25 refusals for "tapscript-miniscript", and the same document says at l.1228 that the
local build "lacks BIP-389 multipath and tapscript miniscript". A false spec sentence is Important by the
brief. One-sentence fix: "imports into Core v31.1 (and into v25 for `wsh`/`sh`/`sh(wsh)` and tree-less `tr`)".


### M-1 (Minor, runbook) — Core's `getdescriptorinfo` hands back only the `/0/*` half of a multipath descriptor in its `descriptor` field

RUN, Core 31.1: `getdescriptorinfo "tr([73c5da0a/48'/0'/0'/3']tpub…/<0;1>/*)"` → `"descriptor":
"tr(…/0/*)#6s6haeww"`, `"checksum": "mku7779e"`, `"multipath_expansion": [ …/0/*, …/1/* ]`. Importing the
`descriptor` field gives a wallet with ONE descriptor (`internal: false`), `getrawchangeaddress` → "This wallet
has no available keys" — a receive-only wallet that will send change to an address it can then only find by
rescanning after re-import. The right string is `<md descriptor output>` (it already carries the multipath
checksum) or `<descriptor>#<checksum field>`. Liana's own `doc/RECOVER.md` (l.39-57) documents the v25 split
but not this v26+ trap. Not an SH2 defect; belongs in the demo/runbook text that tells the operator to use
`getdescriptorinfo`.

### M-2 (Minor, runbook) — `getnewaddress` / `getrawchangeaddress` without an address type fail on every `tr`, `sh` and `sh(wsh)` wallet

RUN, Core 31.1: 17 `tr`, 1 `sh`, 1 `sh(wsh)` wallets → `error code: -12 … No bech32 addresses available.`
(the wallet's default type is bech32); the 35 `wsh` wallets answer untyped. The operator must pass
`bech32m` / `legacy` / `p2sh-segwit`. Documentation only.

### M-3 (Minor, spec) — §13 item 2 (l.1203) describes "md's depth-0 xpubs"; `md 0.17.0` renders depth 4

Decoded from the demo wallet's `md descriptor` output: `xpub6DXuQW1Q2JpZxsEn…` = version `0488b21e`, depth
**4**, parent fingerprint `00000000`, child `0x80000002`; the signer's own export of the same key is
`xpub6DkFAXWQ2dHxq2va…` = depth 4, parent `1cf29716`, child `0x80000002`. The acceptance claim holds (Core 54/54,
Liana 17/17 in both spellings, same addresses) but the "depth-0" wording is md 0.16.2's; the sentence should
say "zero-parent-fingerprint xpubs (F-611)".

### M-4 (Minor, spec) — §13 item 4 (l.1224-1226) gives Liana's constraint as "no `after`, no hashlock"; the measured constraint is narrower

Liana also refuses `older` in 512-second units (`InsaneTimelock(4194404)`, `csv_check :139-145`), any policy
with no unlocked path, two recovery paths sharing a lock, a second unlocked multi-key path, and every
`sortedmulti` head (§5's sole-path lowering) — so "F-449's acceptance wallet must be `older`-only" should read
"`older`-only, in blocks, ≤ 65535, one unlocked path, distinct locks". The full list is in I-1.

### N-1 (Nit) — F-449 cites Liana line numbers from a different revision

`analysis.rs:596-599` (claimed: the raw-hex refusal) is the `Threshold(1, subs)` check at v8.0; the raw-key
refusal is `:568-569`. `analysis.rs:404-445` (the recipe) is `:398-430` at v8.0. F-449 was written from Liana
master on 2026-09-01; the operator's binary is 8.0. Cite the tag with the lines.

### N-2 (Nit) — §4c l.137 cites Core `script.h:48` for `LOCKTIME_THRESHOLD`

At the only Core source on this box (28.99, vendored under `fable-nunchuk-lib/contrib/bitcoin`) the line is
47; the spec pins no Core version for the citation.

### N-3 (Nit) — §8a/§13 attribute Core's string "witnesses without signature exist" to all three coordinators

Liana's refusal of a key-less path is `Descriptor is not compatible with a Liana spending policy.` (2/2,
RUN); the GUI shows "Failed to read the descriptor". The quoted string is Core's alone (and libnunchuk's,
which embeds Core). Wording only.


## Liana v8.0 — what the importer requires (SOURCED, `liana` crate at tag `v8.0` = `9d2fb742`, file `src/descriptors/analysis.rs`)

`LianaDescriptor::from_str` (`src/descriptors/mod.rs:111-139`) parses with rust-miniscript 11
(`Descriptor::<DescriptorPublicKey>::from_str`, `:117`) then calls
`LianaPolicy::from_multipath_descriptor` (`analysis.rs:548-635`):

| rule | where | consequence for an SH2 policy |
|---|---|---|
| Only `wsh(<miniscript>)` and `tr(<xpub>,<tree>)`; `wsh(sortedmulti(...))` is `WshInner::SortedMulti`, not `Ms` | `:554-558`, `:586-587` | every `sh`, `sh(wsh)`, and every SOLE-path `wsh` policy (lowered as `sortedmulti`, §5 l.214) is `IncompatibleDesc` — the demo 2-of-3 included |
| `tr` must carry a tap tree and its internal key must be a MultiXPub; a raw x-only key (md's NUMS `H`) is not | `:564-569`, `:581-584` | every NUMS `tr` is refused (`IncompatibleDesc`; `plain-2of3-tr` already at rust-miniscript's parser, which has no `sortedmulti_a` leaf); `tr(K)` (single key, no tree) too |
| internal key equal to Liana's own unspendable recipe is dropped from the policy; any OTHER xpub becomes a spendable key of the policy | `:398-430` (recipe: pubkey = BIP-341 `H`, chain code = sha256 of the leaf xpubs' 33-byte pubkeys in tap-tree order, depth 0, parent 00000000, no origin), `:566-580` | only that exact xpub imports as "no key path"; the PR-1746 / libnunchuk recipe (sorted, de-duplicated) is a different xpub and lands in the policy as a key without origin → `InvalidKey` |
| semantic lift + `normalized()`; top must be `thresh(1, ...)` | `:559`, `:565`, `:589`, `:595-599` | `or_i`/`or_d`, `pkh`/`pk`, `multi`/`multi_a` differences are erased — the composer's lowering choices do not matter to Liana |
| exactly ONE unlocked path (single key or k-of-n); a second unlocked SINGLE key is merged into the first path's key set WITHOUT changing k; a second unlocked MULTI is refused | `:604-619` (`with_added_key`, `:261-269`) | `[2of3, 1of1, rec]` imports as "primary 2-of-4"; `[1of1, 2of3, rec]` is `IncompatibleDesc` |
| at least one recovery path | `:472-474` | any policy with no lock (plain multisig, single key) is `MissingRecoveryPath` — but SOLE-path policies never get that far (sortedmulti) |
| recovery path = `thresh(2, older(n), keys)` or the n-of-n normal form; `older` only, `1 <= n <= 65535`, blocks only | `:205-258`, `csv_check :139-145`, `:483-485` | `after(...)` → `IncompatibleDesc`; `older` in 512-s units (`0x400000+u`) → `InsaneTimelock(4194404)`; every hash (`sha256/hash256/hash160/ripemd160`, keyed or key-less) → `IncompatibleDesc`; a policy with NO unlocked path (decaying-multisig) → `IncompatibleDesc` at `:633` |
| two recovery paths with the same `older` value | `:624-626` | `IncompatibleDesc` |
| each key: MultiXPub with an origin, exactly two derivation paths, all unhardened, wildcard | `DescKeyChecker::check :101-129` | md's `[fp/path]xpub/<0;1>/*` passes; `md descriptor --chain 0` output (`/0/*`) is `InvalidKey`; the xpub HEADER bytes are never inspected (only `(xkey, derivation_paths)` equality for duplicates, `:106-111`) |
| the same master fingerprint twice inside ONE multi-key path | `:506-515` | `DuplicateOriginSamePath` (§8g's "Liana will refuse it" is TRUE); the same fingerprint in DIFFERENT paths is allowed |
| the policy must recompile through the miniscript compiler | `:526`, `:671-730` | none of the accepted shapes failed here |

The GUI (`liana-gui` at the same tag) imports through the installer flow `UserFlow::AddWallet`
(`gui/src/installer/mod.rs:137-149`): launcher "Add an existing Liana wallet" (`gui/src/launcher.rs:287`)
→ ChooseBackend / ImportRemoteWallet (both skipped when no Liana-Connect backend; on regtest/testnet the
backend step is skipped outright, `mod.rs:104-111`) → **ImportDescriptor** (`step/descriptor/mod.rs:28-72`)
→ RecoverMnemonic (skipped unless the hot signer's fingerprint is in the descriptor, `step/mnemonic.rs:45-49`)
→ RegisterDescriptor (skipped when no hardware wallet was used, `step/descriptor/mod.rs:234-236`)
→ SelectBitcoindType → DefineNode → Final. The ImportDescriptor screen is titled "Import the wallet" with a
single "Descriptor:" field (`installer/view/mod.rs:256-296`); it calls the SAME `LianaDescriptor::from_str`
(`step/descriptor/mod.rs:47`), then `all_xpubs_net_is(Bitcoin|Testnet)` (`:49-52`), and on ANY parse
error shows the static warning **"Failed to read the descriptor"** (`view/mod.rs:271`) with Next disabled
(`view/mod.rs:290-296`) — the verbatim reason (`IncompatibleDesc`, `InsaneTimelock`, ...) is discarded at
`:47` and never reaches the operator. So the harness below is the gold standard for WHY; the GUI only says no.


## The matrix

56 shapes: rows 1-30 are lens 2's device-composed chunk sets at fork tip `f5b068fa` (the SAME artifact the
device cuts, decoded by `md 0.17.0 descriptor`); rows 31-56 are `md compose` 0.17.0 shapes seated with the
demo seeds (+ one extra seed for the four-signer cases) through `md descriptor --template`. "Liana" is
`LianaDescriptor::from_str` on the multipath spelling (the policy Liana infers is in parentheses; "addrs==md"
compares Liana's receive+change 0..2 with `md address`). "Core … wallet import" is `createwallet` +
`importdescriptors` + `getnewaddress`/`getrawchangeaddress` ×3 vs `md address --chain 0|1` by scriptPubKey
(31.1: multipath; 25.0: `--chain 0` + `--chain 1` as two descriptors). "wallet spends" lists the S-journeys
run on that shape (all through `walletcreatefundedpsbt → walletprocesspsbt → finalizepsbt →
sendrawtransaction`; "refused" = `sendrawtransaction`'s own refusal). "lianad" is the daemon journey:
`getnewaddress` vs the Core watch-only wallet's strings, primary and each recovery path spent. Evidence
class is RUN for every cell; SOURCED citations are in the Liana section above.

| # | shape | wrapper | spelling (K = `[fp/path]xpub/<0;1>/*`) | Liana v8.0 `LianaDescriptor::from_str` (RUN) | Core 31.1 wallet import | Core 31.1 wallet spends | Core 25.0 wallet import | Core 25.0 wallet spends | lianad 8.0 on regtest (RUN) |
|---|---|---|---|---|---|---|---|---|---|
| 1 | single-tr | tr | `tr(K)` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 2 | single-wsh | wsh | `wsh(pkh(K))` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 3 | plain-2of3-shwsh | sh-wsh | `sh(wsh(sortedmulti(2,K,K,K)))` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 4 | plain-2of3-sh | sh | `sh(sortedmulti(2,K,K,K))` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 5 | plain-2of3-wsh-DEMO | wsh | `wsh(sortedmulti(2,K,K,K))` | REFUSE `IncompatibleDesc` | import Y; recv/chg == md at idx [1, 2, 3]/[0, 1, 2] (earlier payments on chain) | @0@1 → SENT<br>@0 only → incomplete | import Y; recv/chg 0..2 == md Y | @0@1 → SENT<br>@0 only → incomplete | - |
| 6 | plain-2of3-tr | tr | `tr(NUMS,sortedmulti_a(2,K,K,K))` | REFUSE `Miniscript: unexpected «sortedmulti_a(4 args) while parsing Miniscript»` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 7 | plain-2of3-wsh-UNSORTED | wsh | `wsh(multi(2,K,K,K))` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 8 | preset-simple-timelocked-inheritance-wsh | wsh | `wsh(or_i(pkh(K),and_v(v:pkh(K),older(26280))))` | ACCEPT (primary 1 key; rec older(26280) 1 key); addrs==md Y | import Y; recv/chg 0..2 == md Y | heir @1 seq=26280 tip=h → refused: error code: -26 error message: non-BIP68-final<br>heir @1 seq=26280 tip=h+26278 → None<br>heir @1 seq=26280 tip=h+26279 → SENT | import Y; recv/chg 0..2 == md Y | - | - |
| 9 | preset-kofn-recovery-wsh | wsh | `wsh(or_d(multi(2,K,K,K),and_v(v:pkh(K),older(26280))))` | ACCEPT (primary 2-of-3; rec older(26280) 1 key); addrs==md Y | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 10 | preset-tiered-recovery-wsh | wsh | `wsh(or_d(multi(2,K,K),and_v(v:multi(1,K,K),older(26280))))` | ACCEPT (primary 2-of-2; rec older(26280) 1-of-2); addrs==md Y | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 11 | preset-hashlock-gated-wsh | wsh | `wsh(or_i(and_v(v:pkh(K),sha256(<digest>)),and_v(v:pkh(K),older(26280))))` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 12 | preset-decaying-multisig-wsh | wsh | `wsh(or_i(and_v(v:multi(2,K,K),older(13140)),or_i(and_v(v:pkh(K),older(26280…` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 13 | preset-simple-timelocked-inheritance-tr | tr | `tr(K,and_v(v:pk(K),older(26280)))` | ACCEPT (primary 1 key; rec older(26280) 1 key); addrs==md Y | import Y; recv/chg 0..2 == md Y | - | REFUSED `Miniscript expressions can only be used in wsh` | - | - |
| 14 | preset-kofn-recovery-tr | tr | `tr(NUMS,{multi_a(2,K,K,K),and_v(v:pk(K),older(26280))})` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | REFUSED `Miniscript expressions can only be used in wsh` | - | - |
| 15 | preset-tiered-recovery-tr | tr | `tr(NUMS,{multi_a(2,K,K),and_v(v:multi_a(1,K,K),older(26280))})` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | REFUSED `… is not a valid descriptor function` (a `multi_a` leaf; v25 has no tapscript miniscript) | - | - |
| 16 | preset-hashlock-gated-tr | tr | `tr(NUMS,{and_v(v:pk(K),sha256(<digest>)),and_v(v:pk(K),older(26280))})` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | REFUSED `Miniscript expressions can only be used in wsh` | - | - |
| 17 | preset-decaying-multisig-tr | tr | `tr(NUMS,{and_v(v:multi_a(2,K,K),older(13140)),{and_v(v:pk(K),older(26280)),…` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | REFUSED `… is not a valid descriptor function` (a `multi_a` leaf; v25 has no tapscript miniscript) | - | - |
| 18 | keyless-hash-path-wsh | wsh | `wsh(or_i(pkh(K),sha256(<digest>)))` | REFUSE `IncompatibleDesc` | REFUSED `… is not sane: witnesses without signature exist` | - | REFUSED `… is not sane: witnesses without signature exist` | - | - |
| 19 | keyless-hash-older-path-wsh | wsh | `wsh(or_d(multi(2,K,K),and_v(v:sha256(<digest>),older(100))))` | REFUSE `IncompatibleDesc` | REFUSED `… is not sane: witnesses without signature exist` | - | REFUSED `… is not sane: witnesses without signature exist` | - | - |
| 20 | hash160-gated-wsh | wsh | `wsh(or_i(and_v(v:pkh(K),hash160(<digest>)),and_v(v:pkh(K),older(26280))))` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 21 | ripemd160-gated-wsh | wsh | `wsh(or_i(and_v(v:pkh(K),ripemd160(<digest>)),and_v(v:pkh(K),older(26280))))` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 22 | hash256-gated-wsh | wsh | `wsh(or_i(and_v(v:pkh(K),hash256(<digest>)),and_v(v:pkh(K),older(26280))))` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 23 | hashlock-gated-tr-hash160 | tr | `tr(NUMS,{and_v(v:pk(K),hash160(<digest>)),and_v(v:pk(K),older(26280))})` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | REFUSED `Miniscript expressions can only be used in wsh` | - | - |
| 24 | older-units-wsh | wsh | `wsh(or_i(pkh(K),and_v(v:pkh(K),older(4194404))))` | REFUSE `InsaneTimelock(4194404)` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 25 | after-time-wsh | wsh | `wsh(or_i(pkh(K),and_v(v:pkh(K),after(1700000000))))` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 26 | mixed-lock-bases-wsh | wsh | `wsh(or_i(pkh(K),or_i(and_v(v:pkh(K),older(4194404)),and_v(v:pkh(K),after(10…` | REFUSE `InsaneTimelock(4194404)` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 27 | mixed-lock-bases-tr | tr | `tr(K,{and_v(v:pk(K),older(4194404)),and_v(v:pk(K),after(1000000))})` | REFUSE `InsaneTimelock(4194404)` | import Y; recv/chg 0..2 == md Y | - | REFUSED `Miniscript expressions can only be used in wsh` | - | - |
| 28 | same-seed-two-accounts-wsh | wsh | `wsh(sortedmulti(2,K,K,K))` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 29 | same-seed-two-paths-tr | tr | `tr(NUMS,{multi_a(2,K,K,K),and_v(v:pk(K),older(26280))})` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | REFUSED `Miniscript expressions can only be used in wsh` | - | - |
| 30 | nine-of-nine-wsh | wsh | `wsh(sortedmulti(9,K,K,K,K,K,K,K,K,K))` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 31 | X01-wsh-1of1-2of3older | wsh | `wsh(or_i(pkh(K),and_v(v:multi(2,K,K,K),older(26280))))` | ACCEPT (primary 1 key; rec older(26280) 2-of-3); addrs==md Y | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 32 | X02-wsh-2of3-tworec | wsh | `wsh(or_d(multi(2,K,K,K),or_i(and_v(v:pkh(K),older(100)),and_v(v:pkh(K),olde…` | ACCEPT (primary 2-of-3; rec older(100) 1 key, rec older(200) 1 key); addrs==md Y | import Y; recv/chg == md at idx [5, 6, 7]/[1, 2, 3] (earlier payments on chain) | - | import Y; recv/chg 0..2 == md Y | - | recv/chg 0..2 == md True/True; primary SENT; recovery_100 SENT; recovery_200 SENT |
| 33 | X03-wsh-2of3-tworec-samelock | wsh | `wsh(or_d(multi(2,K,K,K),or_i(and_v(v:pkh(K),older(100)),and_v(v:pkh(K),olde…` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 34 | X04-wsh-2of3-plus-1of1-unlocked | wsh | `wsh(or_d(multi(2,K,K,K),pkh(K)))` | REFUSE `MissingRecoveryPath` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 35 | X05-wsh-1of1-plus-2of3-unlocked | wsh | `wsh(or_i(pkh(K),multi(2,K,K,K)))` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 36 | X06-wsh-1of1-older-units | wsh | `wsh(or_i(pkh(K),and_v(v:pkh(K),older(4194404))))` | REFUSE `InsaneTimelock(4194404)` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 37 | X07-wsh-1of1-after-height | wsh | `wsh(or_i(pkh(K),and_v(v:pkh(K),after(1000000))))` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 38 | X08-wsh-1of1-older-65535 | wsh | `wsh(or_i(pkh(K),and_v(v:pkh(K),older(65535))))` | ACCEPT (primary 1 key; rec older(65535) 1 key); addrs==md Y | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 39 | X09-tr-1of1-2of3older | tr | `tr(K,and_v(v:multi_a(2,K,K,K),older(26280)))` | ACCEPT (primary 1 key; rec older(26280) 2-of-3); addrs==md Y | import Y; recv/chg 0..2 == md Y | - | REFUSED `… is not a valid descriptor function` (a `multi_a` leaf; v25 has no tapscript miniscript) | - | - |
| 40 | X10-tr-1of1-tworec | tr | `tr(K,{and_v(v:pk(K),older(100)),and_v(v:pk(K),older(200))})` | ACCEPT (primary 1 key; rec older(100) 1 key, rec older(200) 1 key); addrs==md Y | import Y; recv/chg == md at idx [5, 6, 7]/[1, 2, 3] (earlier payments on chain) | - | REFUSED `Miniscript expressions can only be used in wsh` | - | recv/chg 0..2 == md True/True; primary SENT; recovery_100 SENT; recovery_200 SENT |
| 41 | X11-wsh-samefp-in-path | wsh | `wsh(or_d(multi(2,K,K,K),and_v(v:pkh(K),older(26280))))` | REFUSE `DuplicateOriginSamePath([73c5da0a/4…)` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 42 | X12-wsh-samefp-across-paths | wsh | `wsh(or_i(pkh(K),and_v(v:pkh(K),older(26280))))` | ACCEPT (primary 1 key; rec older(26280) 1 key); addrs==md Y | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 43 | X13-wsh-2of2-2of2older | wsh | `wsh(or_d(multi(2,K,K),and_v(v:multi(2,K,K),older(26280))))` | ACCEPT (primary 2-of-2; rec older(26280) 2-of-2); addrs==md Y | import Y; recv/chg == md at idx [4, 5, 6]/[1, 2, 3] (earlier payments on chain) | - | import Y; recv/chg 0..2 == md Y | - | recv/chg 0..2 == md True/True; primary SENT; recovery_26280 SENT on the matured run-1 coin (lianad DB: confirmed at block 27166); fresh coin inconclusive |
| 44 | X14-wsh-1of1-1of2older | wsh | `wsh(or_i(pkh(K),and_v(v:multi(1,K,K),older(26280))))` | ACCEPT (primary 1 key; rec older(26280) 1-of-2); addrs==md Y | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 45 | X15-wsh-hashlock-known | wsh | `wsh(or_i(and_v(v:pkh(K),sha256(<digest>)),and_v(v:pkh(K),older(5))))` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | @0 funded, no preimage → incomplete<br>@0 funded + PSBT_IN_SHA256 → SENT<br>recovery @1 seq=5 matured → SENT | import Y; recv/chg 0..2 == md Y | @0 funded, no preimage → incomplete<br>@0 funded + PSBT_IN_SHA256 → SENT<br>recovery @1 seq=5 matured → SENT | - |
| 46 | X16-wsh-kofn-older5 | wsh | `wsh(or_d(multi(2,K,K,K),and_v(v:pkh(K),older(5))))` | ACCEPT (primary 2-of-3; rec older(5) 1 key); addrs==md Y | import Y; recv/chg == md at idx [4, 5, 6]/[1, 2, 3] (earlier payments on chain) | primary @0@1 → SENT<br>recovery @3 no sequence (tip=h) → incomplete<br>recovery @3 seq=5 tip=h → refused: error code: -26 error message: non-BIP68-final<br>recovery @3 seq=5 tip=h+3 → refused: error code: -26 error message: non-BIP68-final<br>recovery @3 seq=5 tip=h+4 → SENT | import Y; recv/chg 0..2 == md Y | primary @0@1 → SENT<br>recovery @3 no sequence (tip=h) → incomplete<br>recovery @3 seq=5 tip=h → refused: error code: -26 error message: non-BIP68-final<br>recovery @3 seq=5 tip=h+3 → refused: error code: -26 error message: non-BIP68-final<br>recovery @3 seq=5 tip=h+4 → SENT | recv/chg 0..2 == md True/True; primary SENT; recovery_5 SENT |
| 47 | X17-wsh-decaying-small | wsh | `wsh(or_i(and_v(v:multi(2,K,K),older(3)),and_v(v:pkh(K),older(6))))` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | tier1 @0@1 seq=3 tip=h → refused: error code: -26 error message: non-BIP68-final<br>tier1 @0@1 seq=3 tip=h+2 → SENT<br>tier2 @2 seq=6 tip=h+5 → SENT | import Y; recv/chg 0..2 == md Y | tier1 @0@1 seq=3 tip=h → refused: error code: -26 error message: non-BIP68-final<br>tier1 @0@1 seq=3 tip=h+2 → SENT<br>tier2 @2 seq=6 tip=h+5 → SENT | - |
| 48 | X18-tr-inherit-older5 | tr | `tr(K,and_v(v:pk(K),older(5)))` | ACCEPT (primary 1 key; rec older(5) 1 key); addrs==md Y | import Y; recv/chg == md at idx [4, 5, 6]/[1, 2, 3] (earlier payments on chain) | key path @0 → SENT<br>script path @1 seq=5 → SENT | REFUSED `Miniscript expressions can only be used in wsh` | - | recv/chg 0..2 == md True/True; primary SENT; recovery_5 SENT |
| 49 | X19-tr-kofn-nums-older5 | tr | `tr(NUMS,{multi_a(2,K,K,K),and_v(v:pk(K),older(5))})` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | NUMS multi_a @0@1 → SENT<br>NUMS recovery @3 seq=5 → SENT | REFUSED `Miniscript expressions can only be used in wsh` | - | - |
| 50 | X20-tr-hashlock-known | tr | `tr(NUMS,{and_v(v:pk(K),sha256(<digest>)),and_v(v:pk(K),older(5))})` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | @0 no preimage → incomplete<br>@0 + PSBT_IN_SHA256 → SENT | REFUSED `Miniscript expressions can only be used in wsh` | - | - |
| 51 | X21-wsh-preset-decaying-default | wsh | `wsh(or_i(and_v(v:multi(2,K,K),older(13140)),or_i(and_v(v:pkh(K),older(26280…` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 52 | X22-wsh-1of1-2of3older-tr-origins | wsh | `wsh(or_i(pkh(K),and_v(v:multi(2,K,K,K),older(26280))))` | ACCEPT (primary 1 key; rec older(26280) 2-of-3); addrs==md Y | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 53 | X23-tr-1of1-1of1older-h-spelling | tr | `tr(K,and_v(v:pk(K),older(26280)))` | ACCEPT (primary 1 key; rec older(26280) 1 key); addrs==md Y | import Y; recv/chg 0..2 == md Y | - | REFUSED `Miniscript expressions can only be used in wsh` | - | - |
| 54 | X24-wsh-2of3-1of1-unlocked-plus-rec | wsh | `wsh(or_d(multi(2,K,K,K),or_i(pkh(K),and_v(v:pkh(K),older(100)))))` | ACCEPT (primary 2-of-4; rec older(100) 1 key); addrs==md Y | import Y; recv/chg == md at idx [5, 6, 7]/[5, 6, 7] (earlier payments on chain) | @3 alone → SENT | import Y; recv/chg 0..2 == md Y | - | recv/chg 0..2 == md True/True; primary SENT; recovery_100 SENT; seed3-alone: SENT; Liana sigs view {'signed': [['b8688df1', 1]], 'sigs_count': 1, 'threshold': 2} |
| 55 | X25-wsh-1of1-2of3-unlocked-plus-rec | wsh | `wsh(or_i(pkh(K),or_d(multi(2,K,K,K),and_v(v:pkh(K),older(100)))))` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | import Y; recv/chg 0..2 == md Y | - | - |
| 56 | X26-tr-2of3-1of1-unlocked-plus-rec | tr | `tr(K,{multi_a(2,K,K,K),and_v(v:pk(K),older(100))})` | REFUSE `IncompatibleDesc` | import Y; recv/chg 0..2 == md Y | - | REFUSED `Miniscript expressions can only be used in wsh` | - | - |


## Spec claims naming Liana or Bitcoin Core (`design/SPEC_wallet_policy_composer.md`, master; line numbers as of this review)

| line | sentence (condensed) | verdict | evidence |
|---|---|---|---|
| l.137 | `after(n)` height: "Core `script.h:48` `LOCKTIME_THRESHOLD`" | TRUE in substance; line is 47 at the only Core source on this box (28.99, `fable-nunchuk-lib/contrib/bitcoin/src/script/script.h:47` = `static const unsigned int LOCKTIME_THRESHOLD = 500000000;`); the spec pins no Core version | SOURCED |
| l.182 | "nothing measured refuses or warns on any origin (BIP-388, Ledger, Nunchuk, Liana, md ...)" | TRUE for Liana v8.0: X22 with every origin rewritten `/2'` → `/3'` under `wsh` → ACCEPT, same policy read (`DescKeyChecker` looks at origin PRESENCE and path hardness only, `analysis.rs:117-126`) | RUN |
| l.253 | raw NUMS "is valid in Bitcoin Core (v25 and v31.1, measured)" | TRUE: v31.1 wallet import + addresses for all 9 NUMS shapes; v25: the one tree-less NUMS shape (`plain-2of3-tr`, `tr(NUMS,sortedmulti_a(...))`) imports, the 8 with a tapscript-miniscript tree are REFUSED ("Miniscript expressions can only be used in wsh") — so "valid in Core v25" is only true of the tree-less form | RUN |
| l.258 | "BIP-388-strict registration and Liana refuse it too [the raw NUMS point]" | TRUE for Liana: 9/9 NUMS shapes refused — 8 with `IncompatibleDesc` at `analysis.rs:568-569` (`get_multi_xkey(desc.internal_key())` on an x-only raw key), and `plain-2of3-tr` one layer earlier in rust-miniscript's parser (`Miniscript error: 'unexpected «sortedmulti_a(4 args) while parsing Miniscript»'`, `mod.rs:117`): rust-miniscript 11 has no `sortedmulti_a` leaf, so a sole-path taproot multisig can never import into Liana under ANY internal-key spelling | RUN + SOURCED |
| l.258-260 | "The unspendable-xpub form Nunchuk and BIP-388 accept is a DIFFERENT wallet with different addresses (measured)" | TRUE for Liana's own recipe too: of the 8 NUMS trees rewritten with Liana's xpub, 4 ACCEPT (kofn-recovery-tr, tiered-recovery-tr, same-seed-two-paths-tr, X19) and derive addresses ≠ md/device (receive 0..2 differ, RUN); the other 4 stay refused for their other reason (3 carry a hash, decaying-multisig-tr has no unlocked path); `plain-2of3-tr` cannot be rewritten at all (rust-miniscript has no `sortedmulti_a` leaf) | RUN |
| l.673-674 (§8a) | "Bitcoin Core, Nunchuk and Liana all refuse it [a key-less path]" | TRUE: Core 31.1 and 25.0 `getdescriptorinfo` REFUSE both key-less shapes ("witnesses without signature exist"); Liana → `IncompatibleDesc` (2/2) | RUN |
| l.677-678 | "Liana refuses any hashlock path" | TRUE: 10/10 shapes carrying a hash (keyed or key-less, all four kinds, wsh and tr) → `IncompatibleDesc` | RUN |
| l.732 (§8f) | "Bitcoin Core imports this form [NUMS]" | TRUE on 31.1 (9/9 import, addresses == md, NUMS multi_a and older leaves SPENT through the wallet, S6); on v25 only the tree-less `sortedmulti_a` form parses | RUN |
| l.734-736 (§8f) | "Liana and BIP-388 signers need an unspendable xpub instead (see F-449), which is a different wallet with different addresses" | TRUE, with a precision the copy lacks: Liana needs ITS recipe (`analysis.rs:398-430`), not "an" unspendable xpub — any other xpub is treated as a spendable key and, having no origin, is `InvalidKey`; and even with it only `older`-only NUMS trees import | RUN + SOURCED |
| l.746, l.751 (§8g) | "Liana will refuse it [same seed twice in one path]" | TRUE: X11 (seed 0 at accounts 0 and 1 inside the 2-of-3 primary of kofn-recovery) → `DuplicateOriginSamePath` (`analysis.rs:506-515`); the same seed in DIFFERENT paths (X12) is ACCEPTED, which §8g does not claim otherwise | RUN |
| l.928 | `md.DuplicateKeySlot` "answers Bitcoin CORE's question by design and scopes to a single expression" | PARTLY: Core 31.1 refuses a duplicate key inside a MINISCRIPT expression (`wsh(or_d(multi(2,K,K),...))`, `or_d(pk(K),and_v(v:pkh(K),older(5)))` → "not sane: contains duplicate public keys") but ACCEPTS `wsh(multi(2,K,K))`, `tr(K1,multi_a(2,K,K))`, `tr(K,multi_a(2,K,K1))` and `tr(K1,{pk(K),pk(K)})` (`getdescriptorinfo` OK) — bare `multi`/`multi_a` are descriptor-level, not miniscript, and get no duplicate check; out of lens (funds-safety), recorded | RUN |
| l.979-980 (§8w) | mixed lock bases under wsh: "Bitcoin Core imports it. Taproot accepts both" | TRUE: `mixed-lock-bases-wsh` and `-tr` import into 31.1 with addresses == md; v25 imports the wsh one (the tr one is tapscript miniscript, refused on v25 for that reason, not for the locks) | RUN |
| l.985 | "Core v25/v31.1 import it [mixed lock bases]" | TRUE for wsh on both; see above | RUN |
| l.1203-1204 (§13.2) | "Ledger registration of md's depth-0 xpubs. Core, Liana and Sparrow accept them (measured)" | acceptance TRUE, description STALE: `md 0.17.0` renders depth **4**, parent fingerprint `00000000`, child `0x80000002` for `m/48'/0'/0'/2'` (decoded), not depth 0; Core 31.1 (54/54) and Liana (17/17, and the same 17 with the signer's real header) accept and derive the same addresses | RUN |
| l.1219-1222 (§13.4) | "every fully seated shape without a key-less path imports into Core v25/v31.1 with addresses equal to the consent screen" | v31.1 TRUE (54/54 through the wallet, receive AND change 0..2 == md); **v25 FALSE as written**: on v25 15 of the 17 `tr` shapes are refused, all with "Miniscript expressions can only be used in wsh"; the 2 tree-less ones import — v25 imports 39 of 54 (see the matrix) | RUN |
| l.1222-1224 | "A key-less path makes the WHOLE wallet un-importable in Core, Nunchuk and Liana ('witnesses without signature exist')" | TRUE for Core (both) and Liana (the Liana error text is `IncompatibleDesc`, not Core's string) | RUN |
| l.1224-1226 | "Liana's import refuses any `after` or hashlock path regardless of head ... so F-449's acceptance wallet must be `older`-only" | TRUE: X07/after-time-wsh (`after`) → `IncompatibleDesc`; hashes as above; ADD: `older` must also be in BLOCKS and ≤ 65535 (X06 `older(100u)` → `InsaneTimelock(4194404)`; X08 `older(65535)` ACCEPT) | RUN |
| l.1227-1229 (§13.5) | recon "verified against ... Liana master and drongo HEAD" | UNVERIFIED here (I measured tag `v8.0`, which is what the operator's `liana_8.0-1_amd64.deb` is); nothing measured contradicts it | — |
| F-449 (FOLLOWUPS l.15684ff) | "Liana's importer requires an xpub internal key and refuses the raw hex (`analysis.rs:596-599`, IncompatibleDesc)" | TRUE in substance; at v8.0 the refusing line is `analysis.rs:568-569` (`:596-599` is the `Threshold(1, subs)` check) — citation drift | SOURCED |
| F-449 | recipe (a) "Liana — pubkey H, chaincode = sha256 of the leaf xpubs' pubkeys in left-to-right order, not sorted, not deduplicated (`analysis.rs:404-445`)" | TRUE (rebuilt exactly so in the harness → ACCEPT); at v8.0 the function is `analysis.rs:398-430` | RUN + SOURCED |
| F-449 | "raw `H` and any xpub form derive DIFFERENT ADDRESSES for the same tree" | TRUE, measured with Liana's recipe: receive 0..2 differ on all four accepted rewrites | RUN |
| F-449 acceptance (4) | "imports into Liana with success AND derives byte-identical addresses on the device, in md, and in Liana" | this is the test that would close F-449; today it fails at step one for every NUMS shape (refused) and the xpub rewrite that Liana accepts fails the byte-identical half | RUN |


## Bitcoin Core — the wallet journey (RUN; Core 31.1.0 from nix `/nix/store/h014588m…-bitcoind-31.1`, and Core 25.0.0 = `/usr/local/bin/bitcoind` "Bitcoin Satellite v0.2.4" banner; regtest, `-rpcport=18943 -port=18944`, private datadirs under `.tmp/fable-liana-core{31,25}`, both stopped at the end)

Every shape, keys re-versioned `xpub`→`tpub` (and `tprv` for the slots a signing wallet holds; checksum
recomputed by `getdescriptorinfo`), through the RPCs an operator types: `createwallet` (descriptors, blank,
watch-only or with keys) → `importdescriptors` → `listdescriptors` → `getnewaddress`/`getrawchangeaddress`
→ fund → `walletcreatefundedpsbt` → `walletprocesspsbt` → `finalizepsbt` → `sendrawtransaction`. No hand-built
witness anywhere; the ONE thing built by hand is a `PSBT_IN_SHA256` record (BIP-174 key type 0x0b), because no
Core RPC can add a preimage to a PSBT.

### Import and addresses

- **Core 31.1, multipath spelling (`md descriptor` default):** 54 of 56 shapes import (`importdescriptors`
  success, `listdescriptors` = 2 per wallet: the `/0/*` half and the `/1/*` half with `internal: true`);
  the two key-less shapes are refused at `getdescriptorinfo` (`… is not sane: witnesses without signature
  exist` / `… malleable witnesses exist`). **Receive 0..2 AND change 0..2 equal `md 0.17.0`'s
  `md address --chain 0|1` by scriptPubKey for all 54** (the strings differ only by the `bc1`/`bcrt1` HRP;
  47 of 54 at indices 0..2 in the re-run; the other 7 (plain-2of3-wsh-DEMO, X02-wsh-2of3-tworec, X10-tr-1of1-tworec, X13-wsh-2of2-2of2older, X16-wsh-kofn-older5, X18-tr-inherit-older5, X24-wsh-2of3-1of1-unlocked-plus-rec) had already been paid on this regtest chain by the spend/lianad legs and Core started them at the next unused index — every one of their receive/change strings is md's address at that index (offsets recorded in the JSON); the first import run, before any payment, matched 54/54 at 0..2).
- **Core 25.0, single-chain spellings (`--chain 0` + `--chain 1`, the latter `internal: true`):**
  39 of 56 import, 17 refused (no tapscript miniscript ("Miniscript expressions can only be used in wsh") ×12, no tapscript miniscript (a `multi_a` leaf: "… is not a valid descriptor function") ×3, key-less ("witnesses without signature exist") ×2); receive+change 0..2 == md for 39 of 39 imported. The v25 `getnewaddress`/`getrawchangeaddress` default-type behaviour matches 31.1.
- **Three journey traps, none of them SH2 defects, all worth a runbook line:**
  1. `getdescriptorinfo` on a multipath descriptor returns, in its `descriptor` field, ONLY the `/0/*`
     expansion (with that half's checksum); the whole descriptor's checksum is the `checksum` field and
     both halves sit in `multipath_expansion`. An operator who copies `descriptor` into `importdescriptors`
     gets a receive-only wallet with no change chain (measured: 1 descriptor, `getrawchangeaddress` →
     "This wallet has no available keys"). Use `<descriptor>#<checksum>`.
  2. `getnewaddress` with no address type fails on every `tr` (17), `sh` (1) and `sh(wsh)` (1) wallet
     ("No bech32 addresses available") because Core's default type is bech32; the operator must say
     `bech32m` / `legacy` / `p2sh-segwit`. `wsh` wallets (35) work untyped.
  3. Core's wallet does not refuse an immature `older` spend — `walletprocesspsbt` signs it and
     `finalizepsbt` completes; the refusal comes from `sendrawtransaction` (`non-BIP68-final`) and would come
     from the network on mainnet. The wallet does refuse to SIGN a recovery path when the input's
     `sequence` is not set (it then looks for the primary path's signatures: "missing signatures" of the
     other keys).

### Spends through the wallet (Core 31.1; `walletcreatefundedpsbt` funded EVERY attempt, including the hash-only and timelock-only wallets)

| # | wallet (keys held) | attempt | walletprocesspsbt | finalizepsbt | sendrawtransaction | witness element sizes |
|---|---|---|---|---|---|---|
| S1 | demo 2-of-3 wsh, @0 @1 | default sequence | complete | complete | SENT | `[0, 71, 71, 105]` |
| S1 | change output of that spend | — | — | — | change went to `/1/*` index 0 = `md address --chain 1` index 0 (`bcrt1q3gy9f…8fy5tz` vs `bc1q3gy9f…accayh`, same program) | — |
| S1b | demo 2-of-3 wsh, @0 only | default | incomplete | `missing.signatures` (two pkh) | — | — |
| S2 | X16 kofn older(5), @0 @1 | primary | complete | complete | SENT | `[0, 71, 70, 135]` |
| S2 | X16, heir @3 only | no sequence | incomplete ("missing signatures" of the 2-of-3) | — | — | — |
| S2 | X16, heir @3 only | `sequence=5`, coin height h, tip h | complete | complete | **refused `non-BIP68-final`** | `[71, 33, 0, 0, 0, 135]` |
| S2 | same | tip h+3 | complete | complete | **refused `non-BIP68-final`** | same |
| S2 | same | tip h+4 | complete | complete | SENT | same |
| S3 | X15 hashlock (sha256 of a known 32-byte preimage) + older(5), @0 only | no preimage | incomplete (`missing.signatures` = the OTHER key: Core moved to the older branch) | — | — | — |
| S3 | same | `PSBT_IN_SHA256` spliced by hand | complete | complete | SENT | `[32, 71, 33, 1, 94]` (preimage, sig, key, `1` = or_i branch, script) |
| S3 | X15, heir @1 only | `sequence=5`, matured | complete | complete | SENT | `[71, 33, 0, 94]` |
| S4 | X17 decaying (2-of-2 older(3) / @2 older(6)), @0 @1 | `sequence=3`, tip h | complete | complete | **refused `non-BIP68-final`** | `[0, 71, 71, 1, 103]` |
| S4 | same | tip h+2 | complete | complete | SENT | same |
| S4 | X17, @2 only | `sequence=6`, tip h+5 | complete | complete | SENT | `[71, 33, 0, 103]` |
| S5 | X18 `tr(K0, and_v(v:pk(K1),older(5)))`, @0 (= internal key) | default | complete | complete | SENT | `[64]` — **key path** |
| S5 | X18, @1 only | `sequence=5`, matured | complete | complete | SENT | `[64, 36, 33]` — sig, leaf script, 33-byte control block (single leaf) |
| S6 | X19 NUMS `tr(H,{multi_a(2,K0,K1,K2), and_v(v:pk(K3),older(5))})`, @0 @1 | default | complete | complete | SENT | `[0, 64, 64, 104, 65]` — multi_a leaf (104) + 65-byte control block (two-leaf tree); the wallet never had a key path to try (`getaddressinfo` desc: `tr([7c461e5d]50929b74…`) |
| S6 | X19, @3 only | `sequence=5`, matured | complete | complete | SENT | `[64, 36, 65]` |
| S7 | X20 NUMS `tr(H,{and_v(v:pk(K0),sha256(H)), and_v(v:pk(K1),older(5))})`, @0 | no preimage | incomplete (`next: updater`) | — | — | — |
| S7 | same | `PSBT_IN_SHA256` spliced | complete | complete | SENT | `[32, 64, 73, 65]` |
| S8 | X24-wsh-2of3-1of1-unlocked-plus-rec @3 alone | second unlocked path, single key | complete | complete | SENT | `[71, 33, 1, 0, 0, 0, 164]` |
| S9 | preset-simple-timelocked-inheritance-wsh heir @1 seq=26280 tip=h | unedited preset, immature | complete | complete | **refused `non-BIP68-final`** | `[71, 33, 0, 57]` |
| S9 | preset-simple-timelocked-inheritance-wsh heir @1 seq=26280 tip=h+26279 | unedited preset, matured | complete | complete | SENT | `[71, 33, 0, 57]` |

(S9 mined 26279 blocks in 662.4 s.)


## Operator runbook — the demo payload's wallet in Liana 8.0 (the operator is present; nothing below touches `~/.liana`)

**Facts first.** The demo payload's wallet (`demo/sh2/build-payload.sh`: the three demo seeds at
`m/48'/0'/0'/2'`, composed as the plain 2-of-3 under `wsh`) is **not a Liana wallet and cannot be made one
by any spelling**: it lowers to `wsh(sortedmulti(2,...))` (§5 l.214), which Liana's importer refuses before
it even looks for a recovery path (`analysis.rs:554-558`, `WshInner::SortedMulti` is not `Ms`), and a
Liana policy needs a timelocked recovery path anyway (`:472-474`). Measured: `LianaDescriptor::from_str`
→ `Descriptor is not compatible with a Liana spending policy.` In the GUI this surfaces only as
**"Failed to read the descriptor"** under the field, Next greyed out.

So the click-through has two halves: (1) see the refusal on the demo wallet, (2) import the nearest wallet
the same three seeds DO make importable — the **kofn-recovery** preset under `wsh` (2-of-3 of the demo
seeds, heir = demo seed 0 at account 1, 26280 blocks), which the device composes and cuts as-is.

### 1. The demo 2-of-3 (expect refusal)

Descriptor (`md descriptor` on the six keyed md1 chunks the device cuts as form A, or on the template chunk
`--from-mk1` the three cards — byte-identical, lens 2 measured):

```
wsh(sortedmulti(2,[73c5da0a/48'/0'/0'/2']xpub6DXuQW1Q2JpZxsEnFKrPvDuiRMmQgU4fzHU1wsvM5EqgGAWRJ3cmwbtS8u1HQjrEHg3YFb7XGnFovPydJ8qpaGNNd2hSEPoheWd27EABdGH/<0;1>/*,[3f635a63/48'/0'/0'/2']xpub6DXuQW1Q2JpZwZhyeFyRwoVcxxRQUvWjfWf5X5tre7aRCMTYwNR1DnwZAehowmtGsB2oEka2aWofzRgVnexutt2KVBZfRcPtuxS6JYwywD5/<0;1>/*,[66d455ea/48'/0'/0'/2']xpub6DXuQW1Q2JpZw2pTr5epQqR2dceAT9UAeRtrbqNAh24Mrg99k6spPvgiaDoCdEmvzqcka6r5Yfpa4asbrxqr6PbrJ6LVphjMPiZ1iJKrJPm/<0;1>/*))#k9z7pr9l
```

Route: start Liana → pick the network (the launcher lists Bitcoin / Signet / Testnet / Regtest,
`gui/src/launcher.rs:213-216`) → **"Add an existing Liana wallet"** (`launcher.rs:287`) → on mainnet the
backend question comes first (choose your own node, not Liana Connect) → **"Import the wallet"** screen,
field **"Descriptor:"** → paste the line above → observe: the field turns invalid with
**"Failed to read the descriptor"** and **Next** stays disabled (`installer/view/mod.rs:266-296`). That is
the whole of Liana's feedback; the reason (`IncompatibleDesc`) is discarded at `step/descriptor/mod.rs:47`.
STOP here for the demo wallet — there is no setting that changes the answer.

### 2. The nearest importable wallet from the same seeds: `preset-kofn-recovery-wsh`

On the device: Wallet Policy > Build a new policy > preset **kofn-recovery**, 2-of-3, older 26280 blocks,
wrapper wsh; seat the three demo seeds in path 1 and demo seed 0 (account 1) as the heir; engrave form A
(8 md1 chunks) or form B (2 template chunks + 9 mk1 cards). `md descriptor` on either gives:

```
wsh(or_d(multi(2,[73c5da0a/48'/0'/0'/2']xpub6DXuQW1Q2JpZxsEnFKrPvDuiRMmQgU4fzHU1wsvM5EqgGAWRJ3cmwbtS8u1HQjrEHg3YFb7XGnFovPydJ8qpaGNNd2hSEPoheWd27EABdGH/<0;1>/*,[3f635a63/48'/0'/0'/2']xpub6DXuQW1Q2JpZwZhyeFyRwoVcxxRQUvWjfWf5X5tre7aRCMTYwNR1DnwZAehowmtGsB2oEka2aWofzRgVnexutt2KVBZfRcPtuxS6JYwywD5/<0;1>/*,[66d455ea/48'/0'/0'/2']xpub6DXuQW1Q2JpZw2pTr5epQqR2dceAT9UAeRtrbqNAh24Mrg99k6spPvgiaDoCdEmvzqcka6r5Yfpa4asbrxqr6PbrJ6LVphjMPiZ1iJKrJPm/<0;1>/*),and_v(v:pkh([73c5da0a/48'/0'/1'/2']xpub6DXuQW1Q2JpZxXTg7vTxJjBmWnLdfRbdYGgdLfbHHNf96dtK4UNwjDoqK89JkuyKFoctVsBrXj6jnqxfxbdugyrTQeJsDiUyRCYxgiPSpgC/<0;1>/*),older(26280))))#82sjmrzj
```

1. Same route: network → "Add an existing Liana wallet" → "Import the wallet" → paste → the field stays
   valid and **Next** lights up. (Network check: on mainnet the xpubs must be `xpub`, elsewhere `tpub` —
   `step/descriptor/mod.rs:49-52`; this line is mainnet.)
2. "Recover mnemonic" is skipped unless the app's own hot signer is in the descriptor (it is not);
   "Register descriptor" is skipped because no signing device was used; then the node steps: choose
   "your own node", enter the bitcoind RPC address and cookie/auth; Finish.
3. **Expect the wallet to show as: primary path 2 of 3 keys `73c5da0a`, `3f635a63`, `66d455ea`; one
   recovery path after 26280 blocks with key `73c5da0a`** (Liana identifies keys by fingerprint, so the
   heir shows as the SAME signer as the first primary key — it is: seed 0 at account 1).
4. **Receive addresses 0, 1, 2 — must match exactly** (Liana v8.0 = `md 0.17.0` = Core 31.1 wallet,
   measured):
   - `bc1qwjzval06strhmknc74ysnhaee8h2wqrmafrn7fytwk6uvjnkx2aqtzjlm5`
   - `bc1qy40f2j3ldwkpqfe2hvv97kedrxl6hphkxnxkfuxhpe9vn06psphsyq876x`
   - `bc1qtwruevfx2jcmx2p43q45r87pwpdxsj036mtex9x4ypkudlwh8dtqcxca5v`

   **Change addresses 0, 1, 2** (Liana spends to these; `Settings > Addresses` or `listaddresses`):
   - `bc1quz0uma4vuzvtuqzsye99uq4agxlruj477e7f7m0hfsu2qvjdylss2rawn9`
   - `bc1qqmztlp4fxqvgqqf3678yln7peglatsatqhu948enu9dshftlt32quz3qz6`
   - `bc1qqxs6yvg7fkyqpr2jsqktr3d9d4vk7jrwnugqrq0p7vdjrw9tpsysx59gm0`

   Any other string at index 0 = STOP, record it.
5. The on-screen note on the import page is right: with your own Bitcoin Core node, do **Settings >
   Node > rescan** from the wallet's birth height or Liana will not see earlier coins.

### 3. Two more from the same seeds, if time allows

- `preset-simple-timelocked-inheritance-wsh` (seed 0 now, seed 1 after 26280 blocks) → ACCEPT; receive 0
  `bc1q47ngr890w3lp4xcw3qmgudpgq7v50kktac0l060lcrtvzg8qr3zs3vcu3r`, change 0
  `bc1qvjmcl02wpnvqztvhvcrt6hcnstw2s7ll5yfte48a0400fz7fmgaswsyd0k`.
- `preset-simple-timelocked-inheritance-tr` (the only preset that imports under `tr`: the key path IS
  seed 0) → ACCEPT as a Taproot wallet; receive 0
  `bc1pacagu2lz8mxwfrtzcslp5d4knl6c38u029txzwr98dxpjefxte4s90xlrw`, change 0
  `bc1pn20fzmrjeezkpa0gznusmp3wlgtrduq93ha0yq2d8zhhm8xz889qgfh7qy` (the same two strings lens 2 got from
  Nunchuk).
- The expected refusals, for calibration: `preset-hashlock-gated-wsh` (hash), `preset-decaying-multisig-wsh`
  (no unlocked path), every `tr` preset other than the first (NUMS) — all "Failed to read the descriptor".

### 4. Presets that survive unedited, per wrapper (measured, `LianaDescriptor::from_str`)

| preset | wsh | tr | why not |
|---|---|---|---|
| plain-multisig | ✗ | ✗ | `sortedmulti` / no recovery path (wsh); NUMS (tr) |
| simple-timelocked-inheritance | ✓ | ✓ | — |
| kofn-recovery | ✓ | ✗ | NUMS internal key |
| tiered-recovery | ✓ | ✗ | NUMS internal key |
| hashlock-gated | ✗ | ✗ | hash |
| decaying-multisig | ✗ | ✗ | no unlocked path (Liana needs one), and NUMS under tr |

Four of the twelve preset×wrapper combinations; three of the five non-plain presets under `wsh`, one under `tr`.


## What I ran (every number pasted from the evidence files under `/scratch/code/shibboleth/.tmp/fable-liana-*`; nothing hand-counted)

- **Liana source and build.** `git clone https://github.com/wizardsardine/liana` →
  `/scratch/code/shibboleth/.tmp/fable-liana-src`, `git checkout v8.0` (= `9d2fb742 Merge #1450: Update liana v8`,
  crate `liana` 8.0.0, `miniscript = "11.0"`). `cargo build --locked --bin lianad --bin liana-cli`
  with `CARGO_TARGET_DIR=/scratch/code/shibboleth/.tmp/fable-liana-target` (rustc 1.98.0); `lianad --version`
  → `8.0.0`. Harness crate `/scratch/code/shibboleth/.tmp/fable-liana-harness` (`src/main.rs`, Liana's own
  `Cargo.lock` copied in, so every dependency version is Liana's): `parse` (= `LianaDescriptor::from_str`,
  then `policy()`, `receive_descriptor().derive(i).address(net)`), `unspendable` (rebuilds
  `analysis.rs:398-430`), `sign` (`HotSigner::from_str` + `sign_psbt`), `spendinfo` (`partial_spend_info`,
  what the GUI's signature rows read).
- **Keyring.** `cmd/fableliana/main.go` in a detached worktree of the fork at `f5b068fa` (kept as
  `.tmp/fable-liana-keyring-gen.go`; the worktree was removed mid-review by the controller and recreated,
  and the regenerated keyring is byte-identical to the first, `cmp` clean): the three demo seeds plus
  `legal winner … yellow` (fingerprint `b8688df1`), accounts 0..5, BIP-48 types 1..3 → 72 entries.
- **Shapes.** `fable-liana-shapes.py`: lens 2's 30 device-composed chunk sets (`fable-nunchuk-shapes.jsonl`,
  composed at `f5b068fa` through the device's own calls) decoded with `md 0.17.0` (`md descriptor`,
  `--chain 0`, `--chain 1`, `md decode`, `md address --chain {0,1} --count 3 --json`), plus 26 shapes from
  `md compose --wrapper … --path …/--preset … --json` seated through `md descriptor --template
  <template_with_origins> --key @i=[fp/path]xpub`: **56 rows, 56 with a descriptor, 0 compose errors**
  (the one preset that needed parameters, X21, was re-run with `2of2,1of1,older1=13140,older2=26280,after=1000000`).
- **Liana parser.** `fable-liana-parse.py` → `fable-liana-parse-in.jsonl` / `-out.jsonl`: 56 shapes ×
  {md, real-xpub, h-spelling, regtest-tpub, chain0 (device rows only), origins-edited (X22),
  Liana-unspendable-xpub (9 NUMS rows)} = **289 inputs, 73 ACCEPT / 216
  REFUSE; md spelling 17 ACCEPT of 56; real-xpub / h-spelling / regtest-tpub give the SAME verdict as md
  for 56/56 each and the same addresses for the 17 accepted; chain0 0 ACCEPT of 30**. Table:
  `fable-liana-matrix-liana.md` (`fable-liana-analyze.py`). Proof the harness can fail: the same shape
  flips ACCEPT→REFUSE on one token (`/<0;1>/*` → `/0/*` = `InvalidKey`; `older(100)` → `older(100u)` =
  `InsaneTimelock`), and X24 vs X25 differ only in path order.
- **Core 31.1.** `fable-liana-core.py <cli> v31 import|spend|spend2` (log `fable-liana-core-v31.log`,
  results `fable-liana-core-v31.json`): import leg over 56 shapes → **54 import, 2 refused (key-less)**,
  `listdescriptors` = 2 for 54/54, receive+change payload-equal to md for **54/54** (first run, before any payment: 54/54 at
  indices 0..2 once the untyped-probe offset on `wsh` wallets was accounted for; the re-run after the spend
  legs is described in the Core section). Spend leg: **22 wallet spend attempts (S1–S9): 14 SENT, 4 refused `non-BIP68-final` (the immature `older` attempts, by design), 4 incomplete (`walletprocesspsbt` could not satisfy: one key of 2-of-3, recovery without `sequence`, hash without preimage), 0 other**.
- **lianad 8.0.** `fable-liana-lianad.py` (`fable-liana-lianad.json`, `.log`, per-wallet dirs under
  `.tmp/fable-liana-lianad/<shape>/` with `config.toml` and `lianad.log`): X16, X18, X02, X10, X24, X13 —
  `getnewaddress` ×3, `listaddresses`, fund from the regtest miner, `createspend` → HotSigner → `updatespend`
  → `broadcastspend` → Core mempool; `createrecovery <addr> 1 <older>` at tip h (immature), h+n−2, h+n−1,
  signed with the recovery seed(s) → broadcast. X16-wsh-kofn-older5: receive 0..2 == md [True, True, True], change 0..2 == md [True, True, True] (by scriptPubKey), primary SENT; recovery_5 SENT. X18-tr-inherit-older5: receive 0..2 == md [True, True, True], change 0..2 == md [True, True, True] (by scriptPubKey), primary SENT; recovery_5 SENT. X02-wsh-2of3-tworec: receive 0..2 == md [True, True, True], change 0..2 == md [True, True, True] (by scriptPubKey), primary SENT; recovery_100 SENT; recovery_200 SENT. X10-tr-1of1-tworec: receive 0..2 == md [True, True, True], change 0..2 == md [True, True, True] (by scriptPubKey), primary SENT; recovery_100 SENT; recovery_200 SENT. X13-wsh-2of2-2of2older: receive 0..2 == md [True, True, True], change 0..2 == md [True, True, True] (by scriptPubKey), primary SENT; recovery_26280 SENT on the matured run-1 coin (DB: confirmed at 27166). X24-wsh-2of3-1of1-unlocked-plus-rec: receive 0..2 == md [True, True, True], change 0..2 == md [True, True, True] (by scriptPubKey), primary SENT; recovery_100 SENT.
- **Core 25.0.** `fable-liana-run-v25.sh`: stop 31.1, start `/usr/local/bin/bitcoind` on the same ports with a
  fresh datadir, `fable-liana-core.py /usr/local/bin/bitcoin-cli v25 import` then `spend` (S1–S4, the `wsh`
  journeys), stop. Ran: `done v25 spend`.
- Hygiene: nothing written under `$HOME` (Liana's default `~/.lianad` never used: every `config.toml` sets
  `data_dir` under `.tmp`); `DISPLAY` never used; no `.jsonl` transcript read (the only `.jsonl` files read are
  lens 2's shape list and my own harness I/O); nothing built under `/tmp`; every repo checkout read-only;
  the fork worktree `.tmp/fable-liana` removed at the end; both regtest nodes stopped.

## What I could not verify

- **The Liana GUI itself was never run** (no window, no `~/.liana`). Every "Liana" verdict is the `liana`
  crate's `LianaDescriptor::from_str` in-process — the same function the GUI's import step calls
  (`step/descriptor/mod.rs:47`) and the same crate `lianad` links; the screens, labels and the Broadcast
  gating are read from `liana-gui` source at the same tag, not seen. The runbook exists so the operator
  closes that gap by eye.
- **Hardware signers in Liana** (Ledger/Coldcard registration of md's zero-parent-fingerprint xpubs) — not
  on this box; Liana's own hot signer signed every spend here, and it derives from the origin, not from the
  xpub header, so nothing here speaks to Ledger's `memcmp` (§13 item 2 stays UNVERIFIED for Ledger).
- **Liana on mainnet-shaped descriptors** — the network check (`all_xpubs_net_is`) was read, not run; all
  lianad runs were regtest with `tpub`.
- **Core v25 spends of `tr` shapes** — impossible on v25 (no tapscript miniscript); the v25 spend leg covers
  the four `wsh` journeys only.
- **The unedited 26280-block presets through lianad** — X13 (2-of-2 + 2-of-2 after 26280) the 2-of-2 primary was SENT through lianad (witness `[0, 71, 71, 149]`); `createrecovery … 26280` then succeeded and `broadcastspend` returned ok at the FIRST attempt (`createrecovery_immature_tip=h: ok`, `broadcast_immature: ok`) — not because the fresh coin was mature but because lianad re-used the bitcoind watch-only wallet still loaded from the crashed first attempt, whose coin (funded ~26,500 blocks earlier) had matured; so the unedited 26280-block preset recovery DOES go through `createrecovery` + HotSigner + `broadcastspend`, but the fresh coin's own timeline is inconclusive: the chunked mining ended at tip 49,484, ~3,900 blocks short of h+26279, and its three attempts stayed "No coin currently spendable through this timelocked recovery path." (lianad restarts in that run: 0). lianad's own database settles the question (read-only `sqlite3` on `regtest/lianad.sqlite3`, table `coins`): the coin `4f629785…` (1 BTC, block 728) and the change coin `c2de345a…` (block 727) are both `spend_txid = d81b6179…`, `spend_block_height = 27166` — the 26280-block recovery sweep CONFIRMED (728 + 26280 = 27008 ≤ 27166); the run-2 primary spend `7c0f1027…` confirmed at 27164; the fresh coin `696a7838…` (block 27165) is unspent, as it should be.
- **`lianad` under a 26,000-block regtest jump** crashed once with `thread 'Bitcoin Network poller' has
  overflowed its stack` (debug build, `poller::looper` "Chain tip changed while we were updating our state.
  Starting over." recursing); mining in 500-block chunks with a restart on death was the workaround. A Liana
  robustness note, not an SH2 matter; recorded in "Out of lens".

## Out of lens

- **The fork-tip copy still carries the pre-fold §8f sentence**: `gui/composer_copy.go:150-151` at
  `f5b068fa` says "Bitcoin Core and Nunchuk import this form. Liana and BIP-388 signers need an unspendable
  xpub instead (see F-449)." while the spec's §8f (l.732-736) has lens 2's fold. Lens 2's finding; the fold is
  in flight (`781dc7cd`).
- **Core's duplicate-key question is narrower than §8v l.928 says.** Core 31.1 `getdescriptorinfo` accepts
  `wsh(multi(2,K,K))`, `tr(K1,multi_a(2,K,K))`, `tr(K,multi_a(2,K,K1))` and `tr(K1,{pk(K),pk(K)})` (bare
  `multi`/`multi_a` are descriptor functions with no duplicate check) and refuses only duplicates INSIDE a
  miniscript expression (`wsh(or_d(multi(2,K,K),…))`, `or_d(pk(K),and_v(v:pkh(K),older(5)))` → "not sane:
  contains duplicate public keys"). Funds-safety lens.
- **Core's `getaddressinfo` renders the raw NUMS key as `tr([7c461e5d]50929b74…`** — an origin fingerprint
  it invents for a raw key (hash160 of the key). Harmless; noted in case a reviewer diffs descriptor text.
- **`md compose` assigns `account'` = slot index** for every slot (`@3/48'/0'/3'/2'`), the device assigns
  by ordinal among the slots one master fills (`[73c5da0a/48'/0'/1'/2']` for the same preset, §4f) — both
  valid, different wallets; matters only when someone expects `md compose` + seating to reproduce a
  device-composed wallet byte for byte.
- **lianad poller stack overflow** on a 26k-block jump (above).
