# Composer fable review r0 — Lens 3: restore from steel and Go↔Rust artifact fidelity

Reviewer: fable-tier, independent. Worktree slug `steel` (detached at fork
`f5b068faf3049ccf97603dfbaa3709f12893df97`, removed at the end). Host tools:
`md 0.16.2` (descriptor-mnemonic `922778ad`), `mnemonic 0.103.1`, `ms 0.19.0`,
`mk 0.13.0`, `me 0.10.0`. Bitcoin Core `v25.0.0` (`/usr/local/bin`) and
`v31.1.0` (nix, for tapscript miniscript), private regtest on 18743/18744,
stopped afterwards. Evidence directory (outside every repo):
`/scratch/code/shibboleth/.tmp/steel-out/` — `matrix.json` (the device's
output per shape), `restore-results*.json` (every host check with its
observed detail), `restore.py`, `spend.py`, `b58.py`, `mut/` (the mutation
run). Read-only on all four repos; nothing committed.

## Verdict

**0 C / 3 I / 3 M / 2 N.** Yes for every fully seated shape the grammar
admits: from the strings alone, `md` reconstructs the same descriptor, the
same template/policy ids and stubs, and the same receive/change addresses the
device showed, and Bitcoin Core derives the same scriptPubKeys (25 shapes;
22 keyed; 0 disagreements); the Go port and Rust 0.44.2 agree byte-for-byte
in both directions (25/25 templates, 22/22 keyed re-mints, and the device's
keyed md1 from host-rendered keys is byte-identical to `mnemonic bundle`'s).
The three Importants are at the edges of the restore journey, not in the
artifacts: a keyless (EXPERIMENTAL) path restores in `md` but Bitcoin Core
v25 and v31 refuse the wallet outright and the consent copy does not say so;
a PARTIAL template completed on the host loses the host-seated slot's
fingerprint, so the completed descriptor carries no origin for that key; and
the device's own restore document says "Addresses unavailable for this policy
shape" for 13 of the 22 keyed shapes whose addresses the consent screen had
just displayed.

## Findings

### I-1 — A keyless (EXPERIMENTAL) path restores in `md` but Bitcoin Core refuses the wallet; the operator is never told

**Inputs.** Case `C04`: wrapper `wsh`, paths `[1-of-1]`, `[keyless,
sha256(H), after=900000]` (`md compose` spelling `1of1 |
keyless,after=900000,sha256=facee1786b8c3e6f98104fffd02507c4f92f1df9f7e02d753279a6e38f024019`),
slot @0 seated from `key:` record `[73c5da0a/48'/0'/0'/2']xpub6DkFAXW…`.
Device cut: template 2 chunks, keyed 4 chunks, one 2-chunk card; self-check
`ok`; consent screen shows four addresses (`bc1qgcge6wquqra7ytyqyjxcc6zne0g9lh7tkyaruun0dd2ktwvfpa3qszfvqq` first).

**Observed.** `md descriptor`, `md address` and `mnemonic restore --md1`
all reconstruct it (descriptor
`wsh(or_i(pkh([73c5da0a/48'/0'/0'/2']xpub…/<0;1>/*),and_v(v:sha256(facee…4019),after(900000))))#hvs8kp2f`,
addresses equal to the device's). Bitcoin Core `getdescriptorinfo` on the
same descriptor (keys re-versioned to `tpub`, single chain):

- v25.0: `error code: -5 … is not sane: witnesses without signature exist`
- v31.1: `error code: -5 … or_i(pkh([73c5da0a/48'/0'/0'/2']tpubDDuXvjq…/0/*),and_v(v:sha256(facee1786…4019),after(900000))) is not sane: witnesses without signature exist`

Core cannot import it even watch-only; Liana refuses any hashlock path (spec
§13 item 4); Nunchuk uses Core's library verbatim (§13 item 3,
`design/agent-reports/PLAN_export_nunchuk.md:135` records the same refusal).
The constellation already knew: `design/policy-differential-seed1.md:54`
classes exactly this refusal as expected for anything that needed
`--experimental`.

**Expected, and why.** The operator's only warning is §8a
(`gui/composer_copy.go:76-80`, spec lines 656-661): "This path needs no
signature. Whoever knows the preimage of its hash can spend it. If that
preimage is ever engraved, the plate is bearer access." Nothing at the
confirm, at consent (§8i restated) or on the restore document says that the
wallet cannot be watched or spent by Bitcoin Core, Liana or Nunchuk — i.e.
that once funded, its only spending route is `md`-family tooling that does not
sign. Spec §13 item 4 (line 1164) defers import tests to the journey; the
journey has not run them (§13 item 5 measured recon only). The wrong outcome
(funding a wallet no reference software accepts, discovered at restore) is
worse than telling the operator nothing. Copy sites: `gui/composer_copy.go:76`
(§8a) and the consent restatement in `gui/composer_consent.go:234-237`; spec
§8a, §13 items 4-5.

### I-2 — A PARTIAL template completed on the host loses the host-seated slot's fingerprint; the completed descriptor carries no origin for that key

**Inputs.** Case `C13`: `wsh`, `[2-of-3 sorted]`; @0 `key:`
`[73c5da0a/48'/0'/0'/2']xpub6DkFAXW…`, @1 `[3f635a63/48'/0'/0'/2']xpub6FHZCoN…`,
@2 left unseated (§8p). Device cut: template (1 chunk) declaring
`path_decl` divergent `[48'/0'/0'/2', 48'/0'/0'/2', 48'/0'/1'/2']`,
fingerprints `[73c5da0a, 3f635a63, —]`; stub screen line `Slot @2 expects a
key at m/48h/0h/1h/2h`; two cards carrying the template stub `b02b4403` only.
Host completion, as §7f's journey implies: `ms derive --template bip48-p2wsh
--account 1` of a third seed → `mk encode --xpub … --origin-fingerprint
28645006 --origin-path m/48'/0'/1'/2' --policy-id-stub b02b4403` (3-chunk
card), then `md descriptor <template> --from-mk1 <2 device cards> <host card>`.

**Observed.** Seating succeeds (`3 card(s) SHAPE-CONFIRMED`, `composed wallet
id 6a7f307f`), and the descriptor is

```
wsh(sortedmulti(2,[73c5da0a/48'/0'/0'/2']xpub6DXuQW1Q2JpZxsE…/<0;1>/*,[3f635a63/48'/0'/0'/2']xpub6DXuQW1Q2JpZwZh…/<0;1>/*,xpub6DXuQW1Q2JpZwtNiGZ3gCp3GBFPZRd48NFedS6wPPkhFabkDxE7YiDG5VHpqTSCKudYSPavk3bVNC2eDLkZHiek2LSBDaRNAteAb8rAhfyD/<0;1>/*))#rhhs56l8
```

— the third key has NO `[fp/path]` at all. `--emit md1` + `md inspect --json`
on the completed card: `tlv.fingerprints = [[0,"73c5da0a"],[1,"3f635a63"]]`
(no entry for @2 although the card declared `28645006`), `path_decl` still
carries `48'/0'/1'/2'` for @2, `wallet_policy_id 6a7f307f80ce265388e98ff27ad7eb97`.
Same on `C22` (tr with the INTERNAL key unseated: completed descriptor
`tr(xpub6DXuQW1Q2Jpa1eq…/<0;1>/*,and_v(v:multi_a(2,[3f635a63/…]…,[66d455ea/…]…),older(26280)))` — an
internal key with no origin) and `C24`.

**Expected, and why.** Spec §5 "declarations" (line 218): "EVERY slot
declares an origin (§4f) and, when seated, the master fingerprint of the
seated key." The card seated on the host carries both; the completed wallet
should carry `[28645006/48'/0'/1'/2']` for @2 exactly as a full device seating
of the same three keys does (every fully seated case in the matrix carries a
fingerprint on every slot). A BIP-380 key with no origin cannot be attributed
to its signer by a coordinator or an HWI, and the completed wallet's id
differs from the one the device would mint for the same keys. Mechanism, both
sides by design: the device declares no fingerprint for an unseated slot
(`md/compose.go` `resolveOrigins`, §4f) and the host seating engine "never
overwrites a fingerprint-free declaration" — `crates/md-cli/src/seat/compose.rs:8-11`
(module doc) and `:70-83` (`compose` fills only `tlv.pubkeys`); the renderer
then drops the whole origin when the fingerprint is absent,
`crates/md-codec/src/to_miniscript.rs:148` (`let origin = e.fingerprint.map(…)`).
Addresses are unaffected. One sentence on remedy: either the host carries a
seated card's fingerprint into a fingerprint-free slot, or §7f/§8p tell the
operator that a slot seated later on the host will have no origin in the
restored descriptor.

### I-3 — The device's restore document says "Addresses unavailable for this policy shape" for 13 of the 22 keyed shapes, right after the consent screen displayed them

(On-device restore document, not steel — filed here because it is the
document the restorer is told to keep; re-rate if the controller scopes it
out.)

**Inputs.** Any composed shape that is not a flat `sortedmulti`; e.g. `C02`
`wsh(or_d(multi(2,@0,@1,@2),and_v(v:pkh(@3),older(26280))))`, all four slots
seated from `key:` records.

**Observed** (`matrix.json`, per case: `consent_lines`, `device_addr_ok`,
`device_restore_lines`). Consent: `device_addr_ok=true`, four addresses on
screen. Restore document (`multisigRestoreLines`): `Wallet policy (read-only):`
… `Addresses unavailable for this policy shape.` — for C02, C03, C04, C05,
C06, C07, C10, C11, C12, C14, C20, C21, C25 (13 shapes). The nine flat
sortedmulti shapes (C01, C08, C09, C15, C16, C17, C18, C19, C23) get
`Descriptor:` + first receive/change, equal to the consent addresses.

**Expected, and why.** `gui/composer_flow.go:517-538` (`composerRestoreDoc`,
F-544: "an operator who BUILT a wallet here walked away with steel and no
document") hands the keyed policy to `multisigRestoreDocFlow`, whose
`multisigRestoreLines` (`gui/multisig_restore.go:23-41`) only knows the flat
`expandedToDescriptor` route and prints the sentence at `:41`; the router the
consent screen uses, `policyAddressAt` (`gui/wallet_policy.go:378`), derives
every one of these shapes (`complexAddressSource`). The document therefore
states as fact something the same device disproved one screen earlier; a
restorer reading it in five years is told not to try. The sentence is worse
than silence.

### M-1 — Device restore-doc descriptor spells hardened as `h`, the host as `'`; the two carry different (both valid) BIP-380 checksums

C01: device `…[73c5da0a/48h/0h/0h/2h]xpub6DXuQW1Q2JpZxsE…))#c6ptw3rr`, host
`md descriptor` `…[73c5da0a/48'/0'/0'/2']xpub6DXuQW1Q2JpZxsE…))#k9z7pr9l`.
Both checksums verify for their own spelling (recomputed in Python from the
BIP-380 algorithm; also C08 `gvmrn7e9`/`tesu3hzh`, C09 `tnh3lq80`/`ztqktxhv`).
The xpub STRINGS are identical on both sides (depth-4 header from the origin,
zero parent fingerprint — the 0.44.0 header class agrees), so only the
checksum an operator compares by eye differs. `gui/md1_expand.go:129-147`
renders through `bip380.Key`; documentation-level.

### M-2 — A `tpub` `key:` record is admitted silently and cut into a mainnet policy; the host renders it as an `xpub` under a coin-type-1 origin without a word

C17: @1 record `[3f635a63/48'/1'/0'/2']tpubDFPtPArj4Gz…` (testnet version
bytes). `sysw.ParseKeyRecord` accepts it (`sysw/composer_records.go:376-411`
checks depth/child only), the consent screen shows mainnet addresses, the
re-minted card carries the `tpub` (`mnemonic inspect`: `xpub: tpubDFPt…`),
`md descriptor` renders `[3f635a63/48'/1'/0'/2']xpub6DXuQW1Q2JpZw6UK…` and
`md descriptor --from-mk1` reports `WALLET-CONFIRMED` — no warning anywhere.
Addresses agree on all sides and the key is spendable by whoever derives at
that origin, so this is a documentation divergence under §4f's "mainnet-only
by construction", not a wrong result.

### M-3 — `mnemonic restore --md1 … --from ms1=<seed plate> --account 0,1` cross-checks only one of the two slots the seed filled

C11 (seed `73c5da0a` at @0 account 0 and @3 account 1, plus two `key:`
records): output `cosigner @0: 73c5da0a [48'/0'/0'/2'] ← your seed (verified)`
… `cosigner @3: 73c5da0a [48'/0'/1'/2'] from md1 (not independently
verified)`, stderr `PARTIAL: cross-checked 1/4 cosigners`. `ms derive
--template bip48-p2wsh --account 1` on the same plate reproduces @3's card
xpub exactly, so the material is on the plates; the toolkit's keyed-md1
cross-check stops at one position. Host-side (mnemonic-toolkit 0.103.1).

### N-1 — `mnemonic inspect` (text and `--json`) reports `policy_id_stub_count` only; the stub values the stub screen tells the operator to stamp and compare are readable only through `mk decode`

Device card C01 @0: `mnemonic inspect` → `policy_id_stub_count: 2`; `mk decode`
→ `policy_id_stubs: b02b4403, 9db5d8b6` (the template and policy stubs the
screen printed). Toolkit surface.

### N-2 — `md descriptor --network regtest` still renders mainnet `xpub` keys

Core regtest refuses the output (`Multi: key 'xpub6DXuQW…' is not valid`);
every Core check below re-versioned the keys by hand (`b58.py`). The device
is mainnet-only so the operator journey is unaffected; a test/regtest journey
is.

## Restore matrix

Shapes were produced through the composer's own functions (the ones
`composerFlow` calls between the mapping review and the engraver:
`composerArtifactsFor`, `composerSelfCheck`, `composerStubLines`,
`composerConsentLinesFor`, `composerMintCards`, `composerSecretCards`,
`composerPreimagePlates` + `composerBuildHashlockPlate`, `composerCensusLines`,
`policyAddressAt`, `multisigRestoreLines`), with `key:` records parsed by
`sysw.ParseKeyRecord`, mk1 sources by `mk.Decode`, seeds by
`seedRegistry.add` + `composerSeedDerive`. Seeds: BIP-39 "abandon…about",
"zoo…wrong", "beef×12", "letter advice…above" — test vectors only.

Columns: descriptor= is `md descriptor <template> --from-mk1 <device cards>`
== `md descriptor <keyed>`; ids= is device Template-ID / Policy-ID (and
stubs) == `md inspect --json`; addr= is `md address` receive+change [0,1] ==
device, then Core `deriveaddresses` scriptPubKey == device (both chains, [0,1]);
tmpl bytes= is `md compose --wrapper W --path … --json` →
`md encode --force-chunked` == the device's unseated `md.Compose` chunks;
re-mint bytes= is `md descriptor <template> --from-mk1 <cards> --emit md1` ==
the device's keyed md1.

| case | form | md1 tmpl/keyed chunks | mk1 cards (chunks) | ms1 | descriptor= | ids= (T/P) | addr= md / Core | tmpl bytes= | re-mint bytes= | seed / preimage |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| C01 wsh 2-of-3 sorted | A+B | 1/6 | 3 (2,2,2) | 0 | Y | Y/Y | YY / Y/Y (v25.0) | Y | Y | - |
| C02 wsh 2-of-3 then 1 key older | A+B | 1/8 | 4 (2,2,2,2) | 0 | Y | Y/Y | YY / Y/Y (v25.0) | Y | Y | - |
| C03 wsh 1 key+sha256 / 1 key older(units) | A+B+preimage | 2/5 | 2 (2,2) | 0 | Y | Y/Y | YY / Y/Y (v25.0) | Y | Y | preimage Y, phrase Y; SPENT on regtest |
| C04 wsh 1 key / keyless sha256 after(height) | A+B+preimage | 2/4 | 1 (2) | 0 | Y | Y/Y | YY / REFUSED (v25.0, v31.1) — I-1 | Y | Y | preimage Y, phrase Y |
| C05 tr internal key then 2-of-3 older | A+B | 1/8 | 4 (3,3,3,3) | 0 | Y | Y/Y | YY / Y/Y (v31.1) | Y | Y | - |
| C06 tr NUMS sortedmulti_a | A+B | 1/6 | 3 (3,3,3) | 0 | Y | Y/Y | YY / Y/Y (v25.0) | Y | Y | - |
| C07 tr NUMS 3 leaves depth 2 | A+B | 2/8 | 4 (3,3,3,3) | 0 | Y | Y/Y | YY / Y/Y (v31.1) | Y | Y | - |
| C08 sh(wsh) 2-of-2 sorted | A+B | 1/4 | 2 (2,2) | 0 | Y | Y/Y | YY / Y/Y (v25.0) | Y | Y | - |
| C09 sh 2-of-2 sorted | A+B | 1/4 | 2 (2,2) | 0 | Y | Y/Y | YY / Y/Y (v25.0) | Y | Y | - |
| C10 tr internal key then hash160+older | A+B+preimage | 2/5 | 2 (3,3) | 0 | Y | Y/Y | YY / Y/Y (v31.1) | Y | Y | preimage Y, phrase Y |
| C11 wsh seed fills @0 and @3 (two paths) | A+B+ms1 (Full) | 2/8 | 4 (2,2,2,3) | 1 | Y | Y/Y | YY / Y/Y (v25.0) | Y | Y | seed Y (both accounts) |
| C12 wsh 2-of-2 unsorted, B then A | A+B | 1/4 | 2 (2,2) | 0 | Y | Y/Y | YY / Y/Y (v25.0) | Y | Y | - |
| C13 wsh 2-of-3, @2 unseated | B partial | 1/– | 2 (2,2) | 0 | completed on host: Y, but I-2 | Y/– | – | Y | – | - |
| C14 wsh older(units)/after(height)/after(time) | A+B | 2/7 | 3 (2,2,2) | 0 | Y | Y/Y | YY / Y/Y (v25.0) | Y | Y | - |
| C15 wsh same seed twice in one path | A+B+ms1 | 1/4 | 2 (2,3) | 1 | Y | Y/Y | YY / Y/Y (v25.0) | Y | Y | seed Y |
| C16 wsh 2-of-3 from host `mnemonic bundle` mk1 cards, re-minted | A+B | 1/6 | 3 (2,2,2) | 0 | Y | Y/Y | YY / Y/Y (v25.0) | Y | Y | - |
| C17 wsh 2-of-2, @1 a tpub record | A+B | 1/4 | 2 (2,2) | 0 | Y (M-2) | Y/Y | YY / Y/Y (v25.0) | Y | Y | - |
| C18 wsh 2-of-3 from host-rendered (`md descriptor`) key records | A+B | 1/6 | 3 (2,2,2) | 0 | Y | Y/Y | YY / Y/Y (v25.0) | Y | Y; also == `mnemonic bundle` md1 | - |
| C19 wsh 2-of-3 three seeds, Full | A+B+3×ms1 | 1/6 | 3 (2,2,2) | 3 | Y | Y/Y | YY / Y/Y (v25.0) | Y | Y | seed Y ×3 |
| C20 tr internal key + 2 leaves depth 1 | A+B | 1/6 | 3 (3,3,3) | 0 | Y | Y/Y | YY / Y/Y (v31.1) | Y | Y | - |
| C21 wsh 1 key+sha256 (hardened phrase) / 1 key older | A+B+preimage | 2/5 | 2 (2,2) | 0 | Y | Y/Y | YY / Y/Y (v25.0) | Y | Y | preimage Y, phrase(hardened) Y |
| C22 tr, INTERNAL key unseated | B partial | 1/– | 2 (3,3) | 0 | completed on host: Y, but I-2 | Y/– | – | Y | – | - |
| C23 wsh 2-of-3, same fp at two accounts | A+B | 1/6 | 3 (2,3,2) | 0 | Y | Y/Y | YY / Y/Y (v25.0) | Y | Y | - |
| C24 wsh 2-of-3 partial, seated accounts 1 and 0, @1 → lowest free = 2 | B partial | 1/– | 2 (3,2) | 0 | completed on host: Y, but I-2 | Y/– | – | Y | – | - |
| C25 tr 2-of-2 sorted then 1 key+hash256+older | A+B+preimage | 2/7 | 3 (3,3,3) | 0 | Y | Y/Y | YY / Y/Y (v31.1) | Y | Y | preimage Y, phrase Y |

Notes on the hunt list, measured:

- **Partial seating and §4f defaults.** The device's lowest-free-account
  choice is visible on the template (`md decode`: C13 `@2: m/48'/0'/1'/2'`,
  C24 `@1: m/48'/0'/2'/2'` with accounts 1 and 0 taken, C22 internal key
  `@0: m/48'/0'/1'/3'`) and matches the stub screen line to the character;
  a host card minted at exactly that origin seats (`3 SHAPE-CONFIRMED`). What
  the completion loses is I-2. `md compose` has no declared-origin input, so
  host-vs-device agreement on defaults WITH seated slots present is verified
  only through the Go port's pinned 66bdf2f4 vectors, not a CLI run; the
  all-unseated defaults are byte-verified 25/25.
- **Third chunk from stub appending.** Every card re-minted with a
  non-standard-table path (account ≠ 0, or script 3') is 3 chunks (C05, C06,
  C07, C10, C11 @3, C15 @1, C20, C22, C23 @1, C24 @0, C25). The census counts
  PLATES through `bundlePlatePlan` (a 3-chunk card still fits one plate:
  `mk1 key @0: 1 plate (m/48'/0'/0'/3')`), so "counts card chunks" is true
  at plate granularity. Restore reads 3-chunk cards in any order: C05 with
  every card's chunks reversed, and with all 12 chunks shuffled (seed 3), gives
  the same descriptor; the keyed md1's 8 chunks reversed likewise.
- **Key order.** C12 (`multi(2,@0,@1)` seated B-then-A): host descriptor and
  addresses equal the device's; the stub screen's slot lines carry the order.
  `sortedmulti` shapes are order-free on both sides (Rust comparison form
  sorts, `mk1 key @N` labels stay the emitted index).
- **NUMS.** The artifact carries the `is_nums` flag, both sides recompute
  `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`
  (host renders it; device `address.NUMSInternalKey()`); C06/C07/C25
  scriptPubKeys equal on Core v25/v31.
- **Hashlock.** The preimage plate carries the 32-byte preimage as an ms1
  `hash` string (C03 `ms10hashsq0kjhnmz200rwnyl0n09x0pajctydpc6akr3v0rxnx4k60dxjzy2ctpfs0aagvm59p`),
  locator `path 1 / hash sha256 799a82ad..51098bd3 / mk1 stub (policy):
  5138807a`; the phrase plate carries phrase + method line + QR text.
  `ms hashlock --in <plate> --kind sha256 --json` → the device's digest and
  preimage; `printf phrase | ms hashlock --hashlock-phrase-stdin --method
  hardened|sha256 --kind K` → the digest (C03, C04, C10 hash160, C21
  hardened, C25 hash256). **Spent on regtest (C03, Core v25.0):** wallet
  imported from `md descriptor` with @0's `tprv` at the card's origin, funded
  at `bcrt1qcal6twjuxefxe5t8y89l3954tnxvsx9emlmjrpr7y5hmzvxtaq4sy4nv6g` (the
  device's `bc1qcal6twj…7y094a`, same witness program), PSBT built by Core,
  `PSBT_IN_SHA256` (0x0b) key/value = plate digest/preimage inserted,
  `walletprocesspsbt` complete, `finalizepsbt` complete, txid
  `802ecb9c6b2386c644e8bb2a0c3b2cfb4b414d284c33db512c0eaa290d2a5a78`, 1
  confirmation, 5 witness items with the preimage among them.
- **Network.** See M-2. The mk1 wire carries no network field; the xpub
  version bytes on the card are the only signal and nothing reads them.
- **Seed plate.** Full mode cuts one ms1 per seed
  (`ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f` for "abandon…");
  `ms derive --in <plate> --template bip48-p2wsh --account N` reproduces every
  card xpub that seed filled (C11 accounts 0 and 1, C15 accounts 0 and 1, C19
  three seeds). The ms1 carries entropy only; a BIP-39 passphrase (none used
  here) would be a factor absent from the set, which the Full label names.
- **Ids on screen vs Rust.** 25/25 `Template-ID` and stub, 22/22 `Policy-ID`
  and stub, byte-equal to `md inspect --json` (`wallet_descriptor_template_id`,
  `wallet_policy_id`); the keyed card's template id equals the template's.

## Direction table (66bdf2f4..922778ad, measured)

| changed behaviour | device-minted → host (Rust 0.44.2) | host-minted → device (Go port at f5b068f) |
| --- | --- | --- |
| 0.44.0: rendered xpub header takes depth/child from the origin | `md descriptor` on 22 device keyed md1s renders depth-4 `xpub6D…` keys under their origins; the STRING equals the device's own restore-doc rendering (C01, C08, C09, C15, C16, C17, C18, C19, C23) — only `h`/`'` differs (M-1) | `key:` records built from `md descriptor` output (zero parent fp, depth 4): `ParseKeyRecord` accepts; the device's keyed md1 (C18) is byte-identical to `mnemonic bundle`'s own md1 (6 chunks) and its policy id `9db5d8b6d3a0bcbfd1998679da280f6e` equals the host's. A depth-0 `xpub661…` under a 4-component origin (the pre-0.44 rendering) is refused with `ErrKeyRecord`; so is `[00000000/m]xpub661…` |
| 0.44.1: all-zero fingerprint is ABSENT | the composer emits `00000000` only if a record declares it (none here) | host `mnemonic bundle` WIF-slot md1 (`[00000000/m]` ×2): `md.ExpandWalletPolicyChunks` decodes it, `WalletPolicyId 7fdbedfd27735326c9fee8fbf1aaf648` = Rust's, device derives `bc1qh49h5na…`; seating its two cards into a 2-of-2 is refused `md: compose: two slots declare the same origin without two distinct fingerprints … slots @0 and @1` and `composerInvariantViolation=true` (§8v) — the port treats `[0,0,0,0]` as an identity, which here fails closed |
| 0.44.2: mint-time validators off the decode path | Rust decodes all 25 device templates and 22 keyed cards (0 refusals) | the port never ran them on decode: `Reassemble` → `computeEncodingID` → `encodePayload` runs only `validatePlaceholderUsage` / `validateMultipathConsistency` / `validateTapScriptTree` (`md/encode.go:380-390`); `ErrOriginKeyContradiction` lives in `md/encode_multisig.go:275`, the Multisig Build mint path. No divergence class |
| compose: `HashLock` with four kinds | C10 hash160 and C25 hash256: Rust decodes, addresses equal, Core v31 equal | `md compose --path …,hash160=<40 hex>` / `hash256=<64 hex>` + `md encode --force-chunked` → bytes equal to Go `md.Compose` (C10, C25) |
| compose: lowering otherwise unchanged | 25/25 `md compose`+`md encode --force-chunked` byte-equal to the device's unseated `md.Compose(list).Chunks()` (incl. `--experimental` for C04) | 22/22 `md descriptor <device template> --from-mk1 <device cards> --emit md1` byte-equal to the device's keyed md1 |

## What I ran

- Worktree: `git -C /scratch/code/shibboleth/seedhammer worktree add --detach /scratch/code/shibboleth/.tmp/fable-steel f5b068faf3049ccf97603dfbaa3709f12893df97`; added `gui/zz_steel_restore_test.go` (a DRIVER: it asserts nothing, it walks the flow's functions for 25 cases and writes `matrix.json`, `direction-go.json`); removed with `git worktree remove --force` at the end. `CGO_ENABLED=0 go test ./gui/ -run '^TestZZSteel' -v -count=1` → `ok seedhammer.com/gui 0.178s`, 25 cases, 0 build errors (`error` field empty in every row).
- Host: `python3 restore.py` (explicit argv lists; secrets via `--in`/stdin). v25 run over 25 cases: **463 checks, 360 substantive, 360 passed, 0 failed**; 103 were harness-shaped and excluded after inspection — (a) "change[0] present in `mnemonic restore`" (it prints only `first recv`, 22), (b) "stubs present in `mnemonic inspect --json`" (it prints a count, N-1, 66), (c) Core v25 on tapscript-miniscript shapes (re-run on v31 below, 12), (d) my partial-origin regex (3). v31 run over C01, C03, C04, C05, C06, C07, C10, C20, C25: 185 checks; every Core scriptPubKey comparison passed except C04 (I-1).
- Core: `bitcoind -regtest -datadir=… -rpcport=18743 -port=18744 -networkactive=0` v25.0.0, then v31.1.0 (`/nix/store/h014588m81wq2q1pw5s81b8hv610ziy3-bitcoind-31.1/bin`); `getdescriptorinfo` + `deriveaddresses "[0,1]"` per chain; `validateaddress` scriptPubKey vs `mnemonic decode-address --json script_pubkey` of the device's mainnet address. Keys re-versioned `xpub→tpub` by `b58.py` (N-2).
- Spend: `python3 spend.py C03` as described above (txid `802ecb9c…`).
- Chunk order: `md descriptor <C05 template> --from-mk1 <12 chunks reversed | shuffled>` and `md descriptor <C05 keyed 8 chunks reversed>` → identical descriptors.
- Checksums: BIP-380 checksum recomputed in Python for the device and host descriptors of C01/C08/C09 — all six valid for their own spelling.
- **Mutation (proof the checks can fail):** with C01's `policy_id` set to zeros, `receive[0]` replaced, one character of keyed chunk 0 changed, and C03's `hash_digest` set to `f…f`, `restore.py C01 C03` went RED on: `rust decodes device keyed md1` (`BCH checksum verification failed`), `md address == device receive/change`, `mnemonic restore --md1 succeeds`, `template+cards descriptor == keyed descriptor`, `rust re-minted keyed md1 == device keyed md1 (bytes)`, `ms hashlock <plate> digest == device digest`, `ms hashlock from phrase plate == device digest` — 10 RED on C01, 2 on C03 (`mut/restore-results-mutated-C01_C03.json`); matrix restored afterwards.
- Go decode of host artifacts: `TestZZSteelHostMintedDecode` (WIF md1, `direction-go.json`), `TestZZSteelDepth0KeyRecord`.

## What I could not verify

- **Screens were not driven.** Artifacts came from the flow's functions, not from the GUI harness walking `composerEngraveStep`; the strings handed to `bundleEngrave` are the `bundleCard.strings` those functions build, and `composerEngraveStep` adds nothing to them, but a harness walk was not run. Plate legibility is out of lens.
- **Declared-origin defaults on the host** (C13/C22/C24 with seated slots present): `md compose` cannot take declared origins, so the "same defaults" question is answered by the device's pinned vectors, not by a Rust CLI run.
- **On-chain spends** were done for one shape (C03 wsh sha256, Core v25). hash256/ripemd160/hash160, taproot leaves and the keyed-then-timelock paths were verified by scriptPubKey only.
- `mnemonic verify-bundle` was not exercised (no ms1+mk1+md1 bundle shape maps onto a composed multisig).
- Multisig Build / Load Payload seating of the host's WIF cards (origin `m`, zero fingerprint) into a template was not measured — out of lens; see the display divergence below.
- The consent screen's addresses were taken through `policyAddressAt` (the function `composerConsentLinesFor` calls); the consent lines in `matrix.json` carry the same addresses (checked on C17 and C04).

## Out of lens

- **`mnemonic bundle` 0.103.1 mk1 cards** carry the SAME stub three times (`mk decode`: `policy_id_stubs: 9db5d8b6, 9db5d8b6, 9db5d8b6`) and a chunk-set id `mk decode` warns "was not derived from its content, which computes 79aec". The device's re-mint (C16) keeps the triplicate verbatim and appends the template stub (4 stubs) and its `mk.Encode` re-derives the id (no warning) — consistent with `mk.AppendStubs`'s stated rule, but the host input is odd.
- **WIF-slot md1 origin display**: Rust `md decode` reports `@0: [00000000/m]`; the port's `ExpandWalletPolicyChunks` reports `m/48h/0h/0h/2h` (the canonical-fill in the display accessor `md/walletpolicyid.go` documents as a known divergence from Rust's `expand_per_at_n`); `Template.Keys[].OriginPath` says `m`. Ids agree. Whether `slotMatchesCard` seats a path-`m` card against the filled origin was not measured.
- `mnemonic restore --md1` help says depth-≥2 taproot is refused; it restored C07 (depth 2) with addresses — stale help text.
- The mnemonic-engrave checkout showed `demo/sh2/CLI_SPINE.md` and `demo/sh2/src/index.html` modified at the end of this session; I did not write to either.
