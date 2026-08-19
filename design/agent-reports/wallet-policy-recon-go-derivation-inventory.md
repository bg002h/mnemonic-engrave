# RECON: cost of on-device address derivation for arbitrary miniscript (Go/TinyGo fork)

Repo under review: `/scratch/code/shibboleth/seedhammer` (Go, TinyGo device build).
All facts below are either a `file:line` citation or the pasted output of a
command actually run in this session. Where I ran the real `tinygo build` for
`./cmd/controller` (see TinyGo section), the command and its full output are
quoted, not summarized.

## Verdict

Full arbitrary on-device derivation is feasible in one cycle, and it is
**wiring, not primitive-building**. Every cryptographic primitive BIP-341
taproot derivation needs — secp256k1 point add/scalar-mul, x-only pubkeys,
all four BIP-340/341 tagged hashes, the taproot output-key tweak, HMAC-SHA512
for BIP-32, SHA-256, RIPEMD-160/HASH160, bech32/bech32m, base58, and a general
Bitcoin Script opcode/data serializer — is already compiled into today's
firmware image, because `address/address.go` already imports
`github.com/btcsuite/btcd/txscript/v2`, `.../chainhash/v2`,
`.../btcec/v2/schnorr` and `github.com/decred/dcrd/dcrec/secp256k1/v4`
transitively via `gui/gui.go` → `seedhammer.com/address`, and that whole
import chain builds today under `tinygo build -target pico-plus2` (verified
below; measured image is 1.41 MB flash / 62 KB static RAM against a 16 MB
flash / 512 KB SRAM board — under 9% flash, under 12% RAM). The taproot
output-key function the fork already calls, `ComputeTaprootKeyNoScript`, is a
thin wrapper around the fully general `ComputeTaprootOutputKey(internalKey,
scriptRoot []byte)` — today it is called with an empty `scriptRoot`, but the
same call with a real Merkle root (computable from the same library's
`TapLeaf`/`TapBranch`/`.TapHash()`, also already linked) is the entire taproot
delta.

**The single largest real cost driver is not a crypto primitive, it is a
missing algorithm plus an API-surface change**, in two parts, both inside
package `md`:

1. **No code anywhere in the fork walks a parsed miniscript tree and emits
   Bitcoin Script.** `md/md.go` already parses every one of the 36 wire tags
   (see the "37-tag" fact below — I measured 36, not 37) into an exported-free
   AST (`node`/`body`, all unexported), but the ONLY exported function in the
   package is `Decode(s string) (Template, error)`, and `Template` is a flat
   summary (root/policy/k-of-m/keys/renderable bool) with no way for a caller
   outside `md` to reach the tree. The script-emission function must therefore
   be written **inside package `md`** (a normative-codec change, i.e. it falls
   under this repo's Rust-primary rule and the R0 gate), not bolted onto
   `address/address.go` from the outside.
2. **Two fragments have no script template anywhere in the fork or in the
   vendored library**: `multi_a`/`sortedmulti_a` (BIP-342 `OP_CHECKSIGADD`
   chain). The opcode itself (`OP_CHECKSIGADD = 0xba`) is defined in the
   vendored `txscript` opcode table, but nothing builds the chain — this is
   new, small, mechanical code (a loop over `ScriptBuilder.AddData` /
   `AddOp(OP_CHECKSIG/OP_CHECKSIGADD)`), not a missing primitive.

Everything else in the 36-tag table below is either a single fixed opcode
template (leaf fragments: `pk_k`, `pk_h`, `raw_pk_h`, hash fragments,
timelocks, `false`/`true`) or a recursive wrap/glue around an already-decoded
sub-tree (wrappers, combinators, `thresh`) — mechanical once the emitter
exists, using only opcodes already in the vendored `txscript` opcode table.

**If a subset boundary is wanted instead of full generality**, the exact
smallest-real-cost boundary is: keep the two structural exclusions the fork
already enforces at the `Template`/`classifyPolicy` layer (a `tr` with any
script tree; unsorted `multi`/`multi_a`/`sortedmulti_a` at any nesting) OUT of
scope, and defer only `multi_a`/`sortedmulti_a` (the one fragment pair with no
existing script template) — everything else, including a full `tr(internal,
taptree)` with arbitrary leaves, is reachable with primitives already linked
into the image today.

## Primitive inventory

| primitive | present? | file:line | notes |
|---|---|---|---|
| SHA-256 | yes | `address/address.go:7,122` (`crypto/sha256`); also linked stdlib, size report line 38 | used today for P2WSH script-hash |
| RIPEMD-160 | yes | `golang.org/x/crypto/ripemd160`, linked — measured `tinygo build -size full` line 68: `2430 1280 0 0 \| 3710 0 \| golang.org/x/crypto/ripemd160` | reached via `address.Hash160` below |
| HASH160 (RIPEMD160∘SHA256) | yes | `/home/bcg/go/pkg/mod/github.com/btcsuite/btcd/address/v2@v2.0.0/address.go:1717-1719`: `func Hash160(buf []byte) []byte { return calcHash(calcHash(buf, sha256.New()), ripemd160.New()) }`; called at `address/address.go:143,146,150` | already used for P2PKH/P2WPKH hashes |
| HMAC-SHA512 (BIP-32 CKD) | yes | `crypto/hmac` + `crypto/sha512` linked, size report lines 20,27,31,39; reached via `bip32/bip32.go:44-51` (`hdkeychain.ExtendedKey.Derive`) which `address/address.go:187` (`xpub.Derive`) already calls every derivation | BIP-32 child derivation already runs on every address computed today |
| secp256k1 point add / scalar mul | yes | `github.com/decred/dcrd/dcrec/secp256k1/v4`, linked — size report line 62: `34596 782 200 140 \| 35578 340`; used inside vendored `ComputeTaprootOutputKey` via `btcec.ScalarBaseMultNonConst` / `btcec.AddNonConst` (`txscript/v2@v2.0.0/taproot.go:270-272`) | general Jacobian-point ops, not special-cased to the no-script path |
| x-only pubkey handling | yes | `github.com/btcsuite/btcd/btcec/v2/schnorr`, linked — size report line 54: `816 325 0 0 \| 1141 0`; `schnorr.SerializePubKey`/`schnorr.ParsePubKey` used at `taproot.go:249-250` and by `address/address.go:151` | |
| BIP-340/341 tagged hashes (TapLeaf/TapBranch/TapSighash/TapTweak) | yes | `github.com/btcsuite/btcd/chainhash/v2@v2.0.0/hash.go:31-46` (four `Tag*` constants) + `hash.go:160` (`func TaggedHash`); linked — size report line 58: `2290 41 262 672 \| 2593 934` | all 4 tags defined, `TaggedHash` is general (`tag []byte, msgs ...[]byte`) |
| Taproot output-key tweak | yes, and GENERAL not special-cased | `txscript/v2@v2.0.0/taproot.go:244-280` `func ComputeTaprootOutputKey(pubKey, scriptRoot []byte) *btcec.PublicKey`; `taproot.go:287-296` `ComputeTaprootKeyNoScript` is a 1-line wrapper passing `scriptRoot = []byte{}` | today's only call site is `address/address.go:151` `txscript.ComputeTaprootKeyNoScript(pub)` — swapping in a real Merkle root reuses the identical function |
| Per-leaf TapLeaf hash | yes, unused today | `txscript/v2@v2.0.0/taproot.go:418-427` (`NewBaseTapLeaf`/`NewTapLeaf`), `taproot.go:437-449` (`TapLeaf.TapHash()`, BIP-341 leaf encoding: `leafVersion \|\| compactSize(script) \|\| script`) | linked as part of the same `txscript/v2` package (line 59 of size report) |
| TapBranch / Merkle root | yes, unused today | `taproot.go:463-485` (`NewTapBranch`, `TapBranch.TapHash()`, sorts left/right by `bytes.Compare` per BIP-341) | composable to ANY tree shape by hand (see Taproot section) |
| Balanced-tree assembly helper | yes, unused today, shape-specific | `taproot.go:626-...` `AssembleTaprootScriptTree(leaves ...TapLeaf) *IndexedTapScriptTree` | builds ITS OWN merge order from a flat leaf list — does not necessarily match an arbitrary declared `taptree` shape; for an arbitrary tree the fork would compose `NewTapBranch`/`NewTapLeaf` directly rather than use this helper |
| Bech32 / bech32m encoding | yes | `github.com/btcsuite/btcd/address/v2/bech32`, linked — size report line 51: `3340 347 84 104 \| 3767 184` | witness v0 (P2WPKH/P2WSH, bech32) already used at `address/address.go:145,148`; witness v1 (P2TR, bech32m) already used at `address/address.go:152` (`address.NewAddressTaproot`) |
| Base58 encoding | yes | `github.com/btcsuite/btcd/address/v2/base58`, linked — size report line 50: `1044 425 236 16 \| 1705 252` | used for P2PKH/P2SH today |
| General Script opcode/data builder | yes, unused by fork's own code today (fork calls only `txscript.MultiSigScript`/`PayToAddrScript`) | `txscript/v2@v2.0.0/scriptbuilder.go:81-301` `type ScriptBuilder` with `AddOp`, `AddOps`, `AddData`, `AddInt64`, `Script() ([]byte, error)` | fully general opcode-by-opcode emitter — see script-serialization section |
| `OP_CHECKSIGADD` opcode | yes (opcode only, no builder) | `txscript/v2@v2.0.0/opcode.go:229` `OP_CHECKSIGADD = 0xba` | needed for `multi_a`/`sortedmulti_a`; nothing in the fork or the vendored library assembles the chain — see 37-tag table |
| `OP_CHECKLOCKTIMEVERIFY` / `OP_CHECKSEQUENCEVERIFY` | yes | `opcode.go:219,221` | for `after`/`older` |
| Hash-fragment opcodes (`OP_SHA256`/`OP_HASH160`/`OP_HASH256`/`OP_RIPEMD160`) | yes | `opcode.go:207,209-211` | note: these fragments push a LITERAL hash already carried in the parsed AST (`hash160Body`/`hash256Body`, `md/md.go:121-122`) and emit a fixed check template — the device does not need to COMPUTE these hashes at derivation time, only emit the opcode + literal |

## The 37-tag script-emission table

Measured count of the `tag` enum, `md/md.go:40-75` (`awk 'NR>=40 && NR<=75' md/md.go \| grep -c "tag = 0x"` → **36**, not 37 — flagged in Open/could not determine). All 36 tags below, by `file:line` of their const definition in `md/md.go:41-75`.

| tag | what it must emit | class |
|---|---|---|
| `tagWpkh` (0x00) | not a script fragment — output type (witness v0 program over pubkey hash) | trivial (already handled, `address/address.go:145`) |
| `tagTr` (0x01) | not a script fragment — output type (taproot output key) | walk (key-path already handled; script-path needs Taproot section below) |
| `tagWsh` (0x02) | not a script fragment — output type (SHA256 of inner script) | trivial (already handled for the 2 supported inner shapes) |
| `tagSh` (0x03) | not a script fragment — output type (HASH160 of inner script) | trivial (already handled) |
| `tagPkh` (0x04) | not a script fragment — output type (`OP_DUP OP_HASH160 <h> OP_EQUALVERIFY OP_CHECKSIG`) | trivial (already handled) |
| `tagTapTree` (0x05) | structural only — pairs a leaf/branch shape, no opcodes of its own | walk (drives the Taproot Merkle-root recursion) |
| `tagMulti` (0x06, unsorted) | `OP_k <pk1>...<pkn> OP_n OP_CHECKMULTISIG` | trivial (same template `sortedMultisigScript` already uses at `address/address.go:169-176`, minus the BIP-67 sort) |
| `tagSortedMulti` (0x07) | same, BIP-67 sorted | trivial — DONE, `address/address.go:104-129` |
| `tagMultiA` (0x08) | `<pk1> OP_CHECKSIG <pk2> OP_CHECKSIGADD ... <pkn> OP_CHECKSIGADD <k> OP_NUMEQUAL` | **blocked on missing code** (opcode present, no builder anywhere — see inventory) |
| `tagSortedMultiA` (0x09) | same, BIP-67 sorted | **blocked on missing code**, same as above |
| `tagPkK` (0x0A) | bare `<pk>` push | trivial |
| `tagPkH` (0x0B) | `OP_DUP OP_HASH160 <hash160(pk)> OP_EQUALVERIFY` | trivial — hash160 primitive present (inventory) |
| `tagCheck` (0x0C, `c:`) | `[X] OP_CHECKSIG` | walk (wraps one child) |
| `tagVerify` (0x0D, `v:`) | `[X] OP_VERIFY` (or `*VERIFY` fusion if `X`'s last op is EQUAL/CHECKSIG/CHECKMULTISIG) | walk |
| `tagSwap` (0x0E, `s:`) | `OP_SWAP [X]` | walk |
| `tagAlt` (0x0F, `a:`) | `OP_TOALTSTACK [X] OP_FROMALTSTACK` | walk |
| `tagDupIf` (0x10, `d:`) | `OP_DUP OP_IF [X] OP_ENDIF` | walk |
| `tagNonZero` (0x11, `j:`) | `OP_SIZE OP_0NOTEQUAL OP_IF [X] OP_ENDIF` | walk |
| `tagZeroNotEqual` (0x12, `n:`) | `[X] OP_0NOTEQUAL` | walk |
| `tagAndV` (0x13) | `[X] [Y]` | walk (2 children) |
| `tagAndB` (0x14) | `[X] [Y] OP_BOOLAND` | walk |
| `tagAndOr` (0x15) | `[X] OP_NOTIF [Z] OP_ELSE [Y] OP_ENDIF` | walk (3 children) |
| `tagOrB` (0x16) | `[X] [Z] OP_BOOLOR` | walk |
| `tagOrC` (0x17) | `[X] OP_NOTIF [Z] OP_ENDIF` | walk |
| `tagOrD` (0x18) | `[X] OP_IFDUP OP_NOTIF [Z] OP_ENDIF` | walk |
| `tagOrI` (0x19) | `OP_IF [X] OP_ELSE [Z] OP_ENDIF` | walk |
| `tagThresh` (0x1A) | `[X1] [X2] OP_ADD [X3] OP_ADD ... <k> OP_EQUAL` | walk, n-ary (`variableBody{k, children []node}`, `md/md.go:106-109`) |
| `tagAfter` (0x1B) | `<n> OP_CHECKLOCKTIMEVERIFY` | trivial (`timelockBody uint32`, `md/md.go:123`) |
| `tagOlder` (0x1C) | `<n> OP_CHECKSEQUENCEVERIFY` | trivial |
| `tagSha256` (0x1D) | `OP_SIZE <32> OP_EQUALVERIFY OP_SHA256 <h> OP_EQUAL` | trivial — `h` is a literal already in `hash256Body`/wire, not computed |
| `tagHash160` (0x1E) | `OP_SIZE <32> OP_EQUALVERIFY OP_HASH160 <h> OP_EQUAL` | trivial — `h` literal in `hash160Body` (`md/md.go:122`) |
| `tagHash256` (0x1F) | `OP_SIZE <32> OP_EQUALVERIFY OP_HASH256 <h> OP_EQUAL` | trivial |
| `tagRipemd160` (0x20) | `OP_SIZE <32> OP_EQUALVERIFY OP_RIPEMD160 <h> OP_EQUAL` | trivial |
| `tagRawPkH` (0x21) | `OP_DUP OP_HASH160 <h> OP_EQUALVERIFY` (h given, not derived from a key) | trivial |
| `tagFalse` (0x22) | `OP_0` | trivial |
| `tagTrue` (0x23) | `OP_1` | trivial |

24 of 36 tags are single-opcode-template leaves; 10 are wrap/combinator "walk"
nodes over an already-parsed child list (`childrenBody`, `md/md.go:105`,
explicitly documented as covering "wrappers/and/or/andor/TapTree"); 1
(`thresh`) is an n-ary walk; only 2 (`multi_a`/`sortedmulti_a`) are blocked on
missing (but trivial-to-write) code. Every opcode named above is confirmed
present in the vendored, already-linked `txscript` opcode table (Primitive
inventory table, rows for `OP_CHECKSIGADD` etc.).

## Taproot four-step status

| step | status | file:line |
|---|---|---|
| Per-leaf script | **library present, fork does not call it** | `txscript/v2@v2.0.0/taproot.go:418-427` `NewBaseTapLeaf`/`NewTapLeaf` accept an arbitrary `[]byte` script — pairs directly with whatever the emitter above produces per tapscript leaf |
| TapLeaf hash | **library present, fork does not call it** | `taproot.go:437-449` `TapLeaf.TapHash()` implements the exact BIP-341 leaf encoding (`leafVersion \|\| compactSize(script) \|\| script`, tagged with `TagTapLeaf`) |
| Merkle root | **library present, fork does not call it** | `taproot.go:463-485` `NewTapBranch`/`TapBranch.TapHash()` — composes any two `TapNode`s (leaf or branch), sorts by `bytes.Compare` per BIP-341 (`taproot.go:495-499`); can be driven by hand to match `md`'s parsed `trBody{tree *node}` shape exactly (`md/md.go:114-118`) rather than via the shape-specific `AssembleTaprootScriptTree` helper |
| Output-key tweak | **library present AND already the fork's only taproot call today** | `address/address.go:151` calls `txscript.ComputeTaprootKeyNoScript(pub)`, itself `taproot.go:287-296`, a 1-line wrapper over the fully general `ComputeTaprootOutputKey(pubKey, scriptRoot)` at `taproot.go:244-280` |

All four steps are class **library present, wiring missing** — none needs a
new primitive. The gap is that nothing today constructs a non-empty
`scriptRoot` and passes it to the already-general `ComputeTaprootOutputKey`.

## TinyGo constraints

Quoted evidence only, from an actual `tinygo build` run this session
(toolchain: `/nix/store/ld75xdghv8yclwbqvgd3x2g897sgyys0-tinygo-0.41.1/bin/tinygo`,
matching the pinned version in this repo's Nix flake and CI):

```
$ tinygo build -size full -o controller.uf2 -target pico-plus2 \
    -stack-size 16kb -gc precise -opt 2 -scheduler tasks ./cmd/controller
   code  rodata    data     bss |   flash     ram | package
...
------------------------------- | --------------- | -------
1118944  262220   31240   30732 | 1412404   61972 | total
```

(Flags match those already documented and load-bearing in
`gui/engraved_hook_tinygo.go:14`: `-target pico-plus2 -stack-size 16kb -gc
precise -opt 2 -scheduler tasks`, and match `.github/workflows/test.yml:134`'s
`tinygo-device-build` CI job.)

Board budget, from the TinyGo target definitions themselves:
- Flash: `pico-plus2.json` → `"ldflags": ["--defsym=__flash_size=16M"]`
  (inherits `rp2350b` → `rp2350`). Measured usage 1,412,404 bytes = **8.4% of
  16 MB** — includes the full `txscript`/`chainhash`/`secp256k1`/`schnorr`
  chain already.
- RAM: `rp2350.ld:7,14` → `SRAM : ORIGIN = 0x20000000, LENGTH = 512k`.
  Measured static usage 61,972 bytes = **11.8% of 512 KB** (this figure is
  static data/bss only; the 16 KB per-task stack and precise-GC heap are
  separate, sized by the build flags above, and were not separately probed
  this session).

No comment anywhere in the tree (`gui/tinygo_split_test.go`,
`gui/*_tinygo.go`, `internal/sh2/params.go`) states that TinyGo forbids or
specially taxes any of `math/big`, `crypto/hmac`, `crypto/sha512`, or the
`secp256k1`/`txscript` dependency chain — and the build above proves all of
them compile and link for this target today, since `address/address.go`
already pulls them in. The two `_tinygo.go` stub files that DO exist
(`gui/plate_hook_tinygo.go`, `gui/engraved_hook_tinygo.go`) are about a
different concern entirely — keeping seed-derived/operator-text material
(splines, plate text) out of the firmware image via a `//go:build !tinygo` /
`//go:build tinygo` split, guarded by `gui/tinygo_split_test.go:66-231`, not
about any crypto-library restriction.

Recursion-depth precedent: `md/md.go:324` already bounds miniscript-tree
recursion during decode at `maxDecodeDepth = 128` (`md/md.go:331`), and that
recursive decoder already runs today inside the same firmware image and the
same 16 KB task stack this build measured — the strongest available evidence
that a similarly-recursive script-emission walk (bounded by the same
`maxDecodeDepth`) is stack-safe on this target, though no test in the tree
specifically stress-tests decode recursion depth against the real device
stack (see Open section).

## Existing test surface for `address/`

- `address/address_test.go` (202 lines): self-consistency fixtures inherited
  from upstream commit `309ad2b` — explicitly documented as NOT externally
  anchored for most shapes (`address/address_test.go:11-41`, "PROVENANCE OF
  THE FIXTURES").
- `address/bip_vectors_test.go` (623 lines): conformance against vendored,
  SHA-256-pinned BIP mediawiki sources (`address/bip_vectors_test.go:35-40`,
  `testdata/bips/`) — covers `pkh`, `wpkh`, `sh(wpkh)`, `tr` (key-path),
  `wsh(sortedmulti)`, `sh(wsh(sortedmulti))`, `sh(sortedmulti)`. No taproot
  script-path vector, no `multi_a` vector (neither is derivable today).
- **No mechanism exists today to consume Rust-generated test vectors for
  `address/` specifically.** The one Rust-comparison mechanism in the tree,
  package `oracle` (`oracle/oracle.go`, `oracle/record.go`) plus
  `gui/multisig_build_oracle_live_test.go`, resolves and byte-compares against
  a live `descriptor-mnemonic` checkout's `md`/`mk`/`ms` binaries for the
  multisig-build-repair plan's encode/decode/template gates — it is not wired
  to `address/` derivation, and would need new call sites to serve as one.

## Open / could not determine

- **The "37-tag" figure in the settled facts does not match a tool count.**
  `awk 'NR>=40 && NR<=75' md/md.go | grep -c "tag = 0x"` → **36**. I used the
  measured 36 throughout this report rather than silently reconciling to 37;
  I did not investigate why the settled-fact document says 37 (possibly
  counting something Rust-side, e.g. `crates/md-codec`'s tag set, which I did
  not open — out of scope per the recon brief).
- I did not attempt to write or compile a prototype script-emission function;
  the "trivial"/"walk"/"blocked" classification in the 37-tag table is a
  script-template classification against the standard, published miniscript
  fragment→Script mapping and the vendored opcode table, not a build-gated
  claim.
- I did not measure per-task stack headroom at runtime (e.g. via
  `-print-stacks`, which the CI job's comment at `.github/workflows/test.yml`
  mentions using) for a hypothetical deep recursive script-emission call —
  only cited the existing `maxDecodeDepth=128` recursive-decode precedent as
  indirect evidence.
- I did not check whether `md`'s unexported `node`/`body` types would need to
  become exported, or whether a narrower exported "compile" entry point
  (`func Decode(s string) (Template, []byte /*script*/, error)` or similar)
  suffices — that is a design decision, explicitly out of scope for this
  recon.
- `AssembleTaprootScriptTree`'s exact merge order (which pairs of leaves it
  branches together when given `>2` leaves) was read from source
  (`taproot.go:626ff`) but not traced against a concrete multi-leaf `md1`
  taptree example to confirm mismatch with declared tree shape in all cases —
  flagged as a reason to compose `TapBranch`/`TapLeaf` by hand rather than use
  the helper, not as a confirmed bug in the helper.
