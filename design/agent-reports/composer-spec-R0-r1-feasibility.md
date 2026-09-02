# R0 round 1 — SPEC_wallet_policy_composer.md, IMPLEMENTATION-FEASIBILITY lens

**Artifact:** `design/SPEC_wallet_policy_composer.md` at `bc1c07c` (identical at
working HEAD `2b8aeba` — `git diff bc1c07c..HEAD -- design/SPEC_wallet_policy_composer.md`
is empty).
**Question:** can it be implemented as written against the code that exists,
without contradicting an admission rule, API contract, wire invariant or test?
**Method:** everything below was RUN unless marked SOURCED or UNVERIFIED.

Heads measured against:

| repo | head |
| --- | --- |
| fork `bg002h/seedhammer` | `169073c` |
| `descriptor-mnemonic` | `3b0944fb` (spec cites `790fc224`; one commit ahead, a FOLLOWUPS entry only) |
| `mnemonic-key` | `2bb763d` |
| `mnemonic-secret` | `5f37b43` |
| `mnemonic-engrave` | `2b8aeba` |

Binaries by PATH (the `md` NAME is an alias for `mkdir -p` in this shell —
every invocation below used `~/.cargo/bin/md`): md 0.14.0, mk 0.13.0,
ms 0.16.0, me 0.7.0. Go 1.25.10 and TinyGo 0.41.1 were found in the Nix store
and both were used; nothing here is read-only inference where a run was possible.

Result: **1 Critical / 4 Important / 6 Minor / 2 Nit.**

---

## CRITICAL

### C-1 — §4b/§4e admit policies the md1 wire cannot encode: the 32-slot cap is policy-wide, not per-fragment

§4b sets the grammar at "1 to 8 spend paths" (§4 opening) with "`KEYS` |
k-of-n over FRESH slots, n in 1..=9 … every slot appears in exactly one path".
That permits **72 slots**. The md1 wire caps the descriptor's TOTAL placeholder
count at 32, in both languages, structurally:

```
$ ~/.cargo/bin/md encode --in b36.txt        # 4 paths x 9 keys, or_i chain
md: codec error: key count 36 out of range; require 1 ≤ n ≤ 32

$ ~/.cargo/bin/md encode --in big.txt        # 8 paths x 9 keys
md: codec error: key count 72 out of range; require 1 ≤ n ≤ 32
```

Sources:

- Rust — `crates/md-codec/src/error.rs:57-59`:
  `/// Key count `n` out of range. Per SPEC v0.30 §4: `1 ≤ n ≤ 32`.` →
  `KeyCountOutOfRange`, raised from `crates/md-codec/src/origin_path.rs:105`.
- Go — `md/md.go:215-221`, `readPathDecl`: `raw, err := r.read(5)` then
  `n := uint8(raw) + 1`. `n` is a **5-bit wire field**: 1..32 is not a policy,
  it is the field width. A 33rd slot has nowhere to go.

§4b's bounds cell cites `md1 32 per fragment (crates/md-codec/src/tree.rs:92-120)`.
That citation is TRUE and is a **different limit** — read it:
`tree.rs:92-120` bounds `*k` and `indices.len()` per `Body::MultiKeys` /
`Body::Variable` node. Nothing in §4b, §4e or §12 states the policy-wide one.

Consequences as written:

- §4e's structural-refusal table has no total-slot row, so the composer would
  let an operator build a 4x9 shape, show the §7c stub screen, and only fail at
  the emit in §7f — after the operator has written a template id down.
- §12 item 1 crosses `path count ∈ {…, 4, 8}` with key-set variation; every cell
  above 32 slots is unbuildable, so the acceptance cannot be constructed for
  part of its own stated product.

**Minimal spec change.** (a) Add to §4b's bounds table a policy row: *the
policy's TOTAL slot count is 1..=32*, cited to `crates/md-codec/src/error.rs:57-59`
and `md/md.go:215-221` (5-bit `path_decl.n`). (b) Add a §4e refusal, enforced at
the picker: "This wallet already has 32 key slots." (c) Bound §12 item 1's
8-path and 4-path cells to ≤ 32 slots.

(For completeness, the per-fragment limits §4b already cites verify:
`rust-miniscript-fork/src/miniscript/limits.rs:35` `MAX_PUBKEYS_PER_MULTISIG = 20`,
`:38` `MAX_PUBKEYS_IN_CHECKSIGADD = 999`. `multi(2, …32 keys)` is refused by
miniscript at "maximum size is 20" before md's own cap is reached, so the
per-path n ≤ 9 bound is not the problem — the sum is.)

---

## IMPORTANT

### I-1 — §9 item 1 names the single-string serialiser; every consumer of the composed md1 rejects its output

§9 item 1: *"emit keyless and keyed md1 through the existing serialiser
(`encodePayload`, `encodeMD1String`)"*.

`md/encode.go:461` `encodeMD1String` emits the **single-string** md1 form. Every
device-side API the composer's own sections then consume goes through
`md.Reassemble` (`md/chunk.go:207`), which is chunk-form only. Measured, on one
descriptor encoded both ways:

```
single        DecodeChunks err=md: wire version mismatch
single        FormAwareIdChunks err=md: wire version mismatch
single        PolicyShapeChunks err=md: wire version mismatch
single        Decode err=<nil> renderable=true

chunk-of-one  DecodeChunks err=<nil>
chunk-of-one  FormAwareIdChunks err=<nil> id=a235ee7574702e45f80089c07e73ed22 kind=Template-ID
chunk-of-one  PolicyShapeChunks err=<nil> shape={Complete:true … Branches:[{K:2 N:2 Keys:2 …}]}
chunk-of-one  Decode err=md: chunked md1 not supported
```

The rejected APIs are exactly §7e's (`PolicyShapeChunks`, `FormAwareIdChunks`,
`ExpandWalletPolicyChunks`), §7f's (`TemplateEngraveShapeGuardChunks`) and
§12 item 6's (`seatKeyCards`, which calls `FormAwareStubChunks` +
`ExpandWalletPolicyChunks` at `gui/key_card_seating.go:53-61`). A builder that
follows §9 item 1 literally produces a card the rest of the spec cannot read.

The shipped emit path is right there and is already chunk-form for a set of one:
`md/chunk.go:121` `split(d)` — `count = chunksNeeded; if count == 0 { count = 1 }`,
header written with `Chunked: true` unconditionally. `EncodeMultisig`
(`md/encode_multisig.go:112`) and `StripToTemplate` (`md/template_strip.go:27`)
both use it.

**Minimal spec change.** §9 item 1: name `split` (`md/chunk.go:121`) as the emit
path, and keep `encodeMD1String` only where it belongs — the single-string parity
assertions of the vector corpus (§12 item 1).

### I-2 — §12 item 1's "byte for byte" does not name the wire FORM, and the two forms differ

For one and the same descriptor:

```
$ ~/.cargo/bin/md encode "wsh(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))"
md15pfdsssjjtvyyw2sqrqscy9zsn0mkdw0fzr7
$ ~/.cargo/bin/md encode "…same…" --force-chunked
md1f4frpqq9q2tvyyy5jmpprj5qqcyxppgqaudyc7r5fys4a
```

Different strings. `md encode` defaults to single-string for short payloads; the
device (per I-1) only ever produces and only ever consumes the chunk form on the
composer's path. §12 item 1 says only *"the Go builder reproduces every template,
md1 and address byte for byte"*, which is satisfiable in two incompatible ways.

The mechanism to say which already exists and is already used: `Vector::force_chunked`
(`crates/md-codec/src/test_vectors.rs:26-29`) and the fork's split of the corpus
into `singleStringVectorNames` (string parity via `encodeMD1String`) versus
`byteParityVectorNames` (pre-chunk payload bytes) at `md/testdata_test.go:15-53`.

**Minimal spec change.** §12 item 1: state that the Go leg asserts the CHUNK form
(the artifact the device emits), and that any single-string assertion is a
separate, named parity leg on the pre-chunk payload bytes.

### I-3 — §6a assigns a payload-CARDINALITY rule to a per-record classifier, so the rule has no home and §12 item 8 cannot cover it

§6a: *"`now:` MUST match `^[0-9]{1,10}(,[0-9]{1,9})?$` … **at most ONE `now:`
record per payload, two or more is a refusal**."* — stated inside the paragraph
whose subject is `sysw.Classify`.

Both classifiers take exactly one record and cannot see a second:

- Rust `pub fn classify(record: &str) -> record::Class`
  (`crates/me-cli/src/sysw/mod.rs:207`, dispatcher at `:213 classify_with`).
- Go `func Classify(record string) Class` (`sysw/record.go:100`).

And §12 item 8's acceptance is per-record by construction: *"each `key:`, `hash:`,
`now:` record (valid and each §6a malformation) classifies identically on the
host and on the device"* — a two-record refusal is not a classification.

**Minimal spec change.** Name the enforcement site on each side — host
`sysw::pack_with` (`crates/me-cli/src/sysw/mod.rs:288`, which already walks the
whole record vector and already refuses an unclassifiable record by index) and
device `syswSession.load` (`gui/sysw_session.go:80`) — and move the "two `now:`
records" vector from §12 item 8 into §12 item 4, naming that site.

Related, and free while you are there: an unknown record is INERT on the device,
not fatal (`sysw/descriptor.go:46-48`: *"it stays in the session, is offered to
nobody, and reaches no screen"*), and a `now:` record is REFUSED by the host
today — measured:

```
$ ~/.cargo/bin/me sysw pack --in r3.txt --no-passphrase --out /dev/null
me: record 0 … is not a form this container can place: not a BIP-39 mnemonic,
    not an md1/mk1/ms1/mt1 string, and not a `text:`/`pass:`/`tx:` record.
```

so the host class must land before §12 item 2's journey can be packed, and old
firmware ignores the record rather than failing. Both directions are safe; the
spec should say so once.

### I-4 — §12 item 1's vector product is 28,800 cells (57,600 with the origin axis); it is not an acceptance that can be built

Counted, not estimated (`itertools.product` over the axes exactly as §12 item 1
lists them):

```
product (9 axes): 28800
with a 2-way origin axis: 57600
after the stated structural constraints: 1487
```

4 wrappers x 5 path counts x 3 heads x 4 internal-key states x 5 locks x 2 hash
x 2 key-set x 2 keyless x 3 fingerprint = 28,800. Even after removing every cell
the spec's own §4a/§4e/§5 rules forbid, 1,487 remain.

**Impossible cells (each from the spec's own rules, and each measured where a
tool could answer):**

| cell | why impossible |
| --- | --- |
| `sh` / `sh(wsh)` with path count ≠ 1 | §4a: "ONLY a single path" |
| `sh` / `sh(wsh)` with any lock, any hash, a keyless path, a single-key head, or n = 1 | §4a, §4e |
| `tr` with `internal key = none` | §5 tr row: the fallback is NUMS, never none |
| `wsh` / `sh` with `internal key ≠ none` | §5: "n/a" |
| `tr` with `keyless wsh path = present` | §4e; and md refuses it — measured: `md: template parse error: miniscript parse failed: All spend paths must require a signature` |
| path count = 1 with `keyless path = present` | §4b: at least one path must have KEYS |
| `head = locked` with `lock = none` | contradiction in terms |
| `key set = sorted` with path count ≥ 2, or any lock, or a hash | §5: sorted only for the SOLE unlocked, unhashed path/leaf |
| `head = single key` with the sorted/unsorted axis | vacuous for n = 1 |
| `head = single key` with `fingerprints = one seed at two slots in one path` | needs n ≥ 2 in that path |
| `fingerprints = one seed across two paths` with path count = 1 | needs two paths |
| `internal key = extracted not first-listed` with path count < 4 | the axis says "with ≥ 4 paths" |
| NUMS with an unlocked, unhashed single-key path present | §5 forces extraction |
| any cell whose total slot count > 32 | C-1 |

**Minimal spec change.** Replace the product with a **tagged-coverage**
requirement: every vector carries the set of §5 rows and §4c lock rows it
exercises; a test asserts each tag appears in **≥ 2** vectors; the vector count
is whatever a pairwise covering array over the legal axes needs (≈ 50-60 named
vectors, including the 5 lock encodings, the 3 fingerprint cases, the 4 wrappers
and the m ∈ {0,1,2,3,7} taptree spine shapes). That keeps §12 item 1's guarantee
("every §5 rule at least twice") and makes it countable by a script instead of by
a reviewer.

---

## MINOR

### M-1 — §6a cites a flag that does not exist: `mk encode --keys`

§6a defines the `key:` body as *"hex of the UTF-8 text `[fingerprint/path]xpub`
(BIP-380 key-origin notation, the same line `mk encode --keys` reads)"*.

`mk 0.13.0` has no `--keys` flag anywhere. `mk --help` lists
encode/decode/inspect/verify/vectors/gui-schema/repair/address/derive/gen-man;
`mk encode --help` lists `--xpub --origin-fingerprint --origin-path
--policy-id-stub --from-md1 --privacy-preserving --force-chunked
--force-long-code --chunk-set-id --group-size --separator --json`. Grepping
`--keys` across `mk encode|decode`, `md encode|decompose|descriptor` returns
nothing.

The phrase is inherited from md's own error text, which is itself stale —
measured:

```
md: decompose: key @0 is depth-inconsistent IN THE INPUT: … so `mk encode --keys`
    refuses such a record outright …
```

The real producer/consumer of that line form is `md decompose` ("one
origin-notated key line per slot", `md decompose --help`).

**Fix:** cite `md decompose`'s key-line output, and file a follow-up in
descriptor-mnemonic for the stale `mk encode --keys` string in md's decompose
error.

The rest of §6a's `key:` rule is correctly sourced: md's depth admission is
`{3, 4}` (`crates/md-cli/src/parse/keys.rs:130-136`, "expected an account-level
xpub at depth 3 or 4"), and the `depth == origin_path.len()` conjunct lives in
`parse_key_with_origin` (`keys.rs:579+`), exactly as §6a splits them.

### M-2 — `md compile` is behind a non-default feature; §10 item 1 should say `compose` is not

`crates/md-cli/Cargo.toml`: `default = ["json"]`, `cli-compiler = ["miniscript/compiler"]`;
`crates/md-cli/src/main.rs:967-971` — *"compile requires the cli-compiler feature;
rebuild with --features cli-compiler"*. §10 item 1 says `compose` sits "beside
`md compile`", which invites the same gate.

`md-codec` itself already depends on miniscript through a DEFAULT-on optional
feature (`default = ["derive"]`, `derive = ["dep:miniscript"]`), so
`ExtParams::top_unsafe()`, `lift()` and `sanity_check` are reachable there; the
lowering is search-free and needs none of them.

**Fix:** one sentence in §10 item 1 — `compose` is unconditional (not behind
`cli-compiler`); only §5b's compile-leg cross-check needs `cli-compiler`, which
CI already supplies (`.github/workflows/ci.yml:48` `cargo test --workspace
--all-targets --all-features`).

### M-3 — the vector corpus cannot express divergent origins through its `path` field

`Vector` (`crates/md-codec/src/test_vectors.rs:14-42`) carries
`path: Option<&'static str>` — a **shared** origin override only. §12 item 1
requires "origin per wrapper per §4f including unseated-slot origins", which are
divergent by construction.

They are expressible today, INLINE in `template`, and that route is measured to
work end to end:

```
$ ~/.cargo/bin/md inspect md15pfdsssjjtvyyw2sqrqscy9zsn0mkdw0fzr7
origins:
  @0: m/48'/0'/0'/2'
  @1: m/48'/0'/1'/2'
```

`make_path_decl` (`crates/md-cli/src/parse/template.rs:883-897`) emits
`PathDeclPaths::Divergent` when the inline origins differ.

**Fix:** say so in §10 item 1, so the implementer writes inline origins rather
than adding a field to a `#[non_exhaustive]` struct the fork also reads.

### M-4 — §7e's justification is overbroad: not every composable shape is non-renderable

§7e: `md1Summary` *"prints 'Complex policy - cannot display safely.' for every
shape this composer exists to author"*. Measured false for at least one
composable shape — a one-path `wsh(multi(2,…))`, which is exactly the §4a
`sh`/`sh(wsh)`/wsh migration case:

```
md.Decode("md15pfdsssjjtvyyw2sqrqscy9zsn0mkdw0fzr7") -> Renderable:true
md.Decode("md15zfdsssj6tvyywtfdssj5hqqxqu2gpp…")     -> Renderable:false   (tr NUMS, 2 leaves)
```

The design conclusion is untouched — §7e mandates the new consent surface
unconditionally — only the sentence is wrong. **Fix:** "for every multi-path or
taproot shape this composer exists to author".

### M-5 — §4a and §5 disagree on whether `sh(multi(...))` is composable

§4a admits `sh`/`sh(wsh)` for "a single path that is an unlocked, unhashed key
set with n ≥ 2 (**a `sortedmulti`**…)"; §5's unsorted row offers `multi` instead
of `sortedmulti` "where sorted was legal", excluding nothing. Measured: md
accepts it —

```
$ ~/.cargo/bin/md encode "sh(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))"
md15pfdsssjjtvyyw2sqrqccy9qtzfq8u453mqg
```

so §12 item 1's `sh x unsorted` cell is realisable but undecided by the spec.
**Fix:** one clause in §4a saying whether the §8b unsorted confirm is offered
under the legacy wrappers.

### M-6 — §6a's `now:` height field is wider than §4c's height band

§6a admits `^[0-9]{1,10}(,[0-9]{1,9})?$` — a height up to 999,999,999 — while
§4c's `after` height band is 1..=499,999,999. A `now:` whose height field is
above the band makes §6b's bound line refuse **every** composable absolute-height
lock, with copy ("That is before this payload was packed") that does not name the
real cause. **Fix:** bound the height field to 1..=499,999,999 in §6a's body
rules, alongside the seconds bound already there.

---

## NIT

### N-1 — §3's `older(0x400000)` row reconfirmed at md 0.14.0

```
$ ~/.cargo/bin/md encode "wsh(and_v(v:multi(2,@0/…,@1/…),older(4194304)))"
md1yppqqxpye5vzzhqqgqqqqrdk2nxeut4ezs        # exit 0
```

Still accepted, so §12 item 7's device-side gate is still the only thing that
refuses it. No change needed; recorded because the inventory row is load-bearing
for §4c and §10 item 4.

### N-2 — the flash/RAM figure is stale; headroom is ample either way

The brainstorm's "1.41 MB / 62 KB" is not what fork HEAD `169073c` builds.
Measured:

```
$ tinygo build -size short -target pico-plus2 -stack-size 16kb -gc precise \
      -opt 2 -scheduler tasks ./cmd/controller
   code    data     bss |   flash     ram
1472016   31636   30956 | 1503652   62592
```

1.50 MB flash / 61.1 KiB RAM. Nothing in the spec cites the number, so this is
context, not a defect.

---

## Answers to the nine points

**1. §4f unseated-slot origins — MEASURED, fully feasible.**
`md encode` mints a keyless template with DIVERGENT per-slot origins and no
fingerprints, for both wrappers:

```
$ md encode "wsh(multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*))"
md15pfdsssjjtvyyw2sqrqscy9zsn0mkdw0fzr7      # no contested-slot warning
$ md encode "tr(<NUMS>,{multi_a(2,@0/48'/0'/0'/3'/…,@1/48'/0'/1'/3'/…),
                        and_v(v:multi_a(1,@2/48'/0'/2'/3'/…),older(144))})"
md15zfdsssj6tvyywtfdssj5hqqxqu2gppznxjqqyuqqqqpyqtqgg0c5ze4g2d
```

`md inspect` reports `@0: m/48'/0'/0'/3'`, `@1: …/1'/3'`, `@2: …/2'/3'`. The
IDENTICAL-origin variant produces exactly the warning §4f exists to avoid
("this keyless template's slots cannot be told apart — @0, @1 all declare
m/48'/0'/0'/2'"), so distinct accounts by slot index is the right answer.

The fork decodes both (`md.Decode` → `Keys:[{Index:0 … OriginPath:"m/48h/0h/0h/3h"} …]`,
`Fingerprint:""` on every slot). F-166 is about a PATHLESS slot; divergent
declared paths are handled — `md/canonicalize.go:68-79` reorders the divergent
vector in lockstep, `md/walletpolicyid.go:180-187` resolves per-index.

`seatKeyCards` seats by origin alone when the template declares no fingerprint —
`gui/key_card_seating.go:139-150`: *"The fingerprint is checked only when the
TEMPLATE declares one … requiring the card to match an absent declaration would
refuse every card for a legal template."* Path comparison is structural
(`bip32.ParsePath`), so the `m/48h/…` vs `m/48'/…` spelling split does not bite.

**2. Both stubs on a minted card — MEASURED, no cost.**

```
$ mk encode --xpub <X> --origin-fingerprint 73c5da0a --origin-path "m/48'/0'/0'/2'" \
      --policy-id-stub aabbccdd --policy-id-stub 11223344
mk1qpqes0pqqsp24w7vm…        (2 chunks)
$ mk decode …
policy_id_stubs:     aabbccdd, 11223344
chunks:              2 (long)
```

Deterministic (re-run is byte-identical). Chunk count is **2 for one, two AND
three stubs** — the re-mint of §7d/§7f costs no extra chunk. Go agrees byte for
byte: `mk.Decode` → 2 stubs, `mk.Encode(card)` reproduces the Rust strings
exactly (`byte-identical to Rust: true`), and appending a third stub still gives
2 chunks. `mk/encode.go:39-45` `Encode`, bytecode layout
`header|stub_count|stubs(4*N)|[fp]|path|compact73` at `mk/encode.go:73-76`.

**3. §7e self-check + extended `PolicyShape` — MEASURED, feasible.**
Today `md.Branch` (`md/policy_shape.go:43-62`) carries `K, N, Keys, Timelock,
Hashlock, Depth` — presence FLAGS, no operand, no digest — and `collect`
(`:196-240`) folds `tagOrI/tagOrD/tagAndV/…` into ONE branch, so a multi-path
wsh script is one `Branch` (measured: `wsh(or_i(pkh,and_v(v:multi,older)))` →
`Branches=[{K:0 N:0 Keys:3 Timelock:true Hashlock:false Depth:0}]`), while a
taptree yields one per leaf.

The decoder DOES retain what §9 item 1 needs: `md/md.go:120-122` declares
`type hash256Body [32]byte`, `hash160Body [20]byte`, `timelockBody uint32`, and
they survive into the tree — dumped from a real card:

```
Tr isNums=false keyIndex=0 hasTree=true
  TapTree (2 children)
    PkK @1
    AndV (2 children)
      Verify (1 children)
        Sha256 digest=a1b2c3d4e5f60718293a4b5c6d7e8f90a1b2c3d4e5f60718293a4b5c6d7e8f90
      PkK @2
…  Older n=144
```

So carrying the operand and the digest onto `Branch` is a body type-assert, not
a decoder change.

`walletPolicyConsentLines`'s tail is ALREADY factored: `md.FormAwareIdChunks`
returns `(id, kind)` (`gui/wallet_policy.go:190-194`) and
`walletPolicyAddressLines(md1, tpl, keys)` is a standalone function
(`gui/wallet_policy.go:245`) that never returns empty and already emits
"Keyless template - no addresses." — §7e's D4 line, verbatim, for free.

**4. Go tree builder — MEASURED/SOURCED, feasible; `EncodeMultisig` is the template.**
`descriptor` is `{n uint8; pathDecl; useSite; tree node; tlv tlvSection}`
(`md/md.go:816-822`). `encodePayload` (`md/encode.go:374`) runs
`canonicalize(d)` ITSELF, then `validatePlaceholderUsage`,
`validateMultipathConsistency`, `validateTapScriptTree`, and the
`pathDecl.n == d.n` guard — so the builder does not run canonicalisation, it
must merely be consistent. `md/encode_multisig.go:112-175` shows exactly what to
populate for the keyed case (pathDecl Shared/Divergent, `useSitePath{hasMultipath,
multipath:[{0},{1}], wildcardHardened:false}`, `pubkeys` + optional `fps` TLVs);
the keyless case is the same with `pubPresent:false, fpPresent:false`
(`md/template_strip.go:33-38` — note that function clears fingerprints too, so
§7f form B's "keyless WITH fingerprints" must NOT go through `StripToTemplate`).

`md.WalletDescriptorTemplateId(d *descriptor)` and the policy id are computable
from a built descriptor BEFORE serialisation (`md/template_id.go:39, 96, 112`),
but they take the UNEXPORTED `*descriptor` — the builder must therefore live
inside package `md`, which §9 item 1 already implies.

Emit path: see I-1 — `split`, not `encodeMD1String`.

**5. Payload classes — MEASURED, additive, `seal/` untouched.**
Device: `sysw/record.go:14-21` declares `TextPrefix/PassPrefix/TxPrefix`;
`Classify` (`:100-124`) matches them ahead of `classifyConstellation`. Three more
arms with body validation is a local edit. `seal.Classify`
(`seal/record.go:194`) is a separate function over `[]byte` with its own
`cmdPrefix` and shares no prefix table — nothing frozen is touched.
Host: `Class` enum at `crates/me-cli/src/sysw/record.rs:45-64`, order at
`crates/me-cli/src/sysw/mod.rs:213-268` (tx → pass → text → mnemonic → mt →
seal validate → descriptor last). Prefixes go ahead of the sniffers, as §6a says.
`me sysw pack --in` reads whole lines at `crates/me-cli/src/main.rs:2362-2377`
(`read_records` → `split_record_stream`); the `now:` append belongs after the
`--as` substitution at `main.rs:1447` (`let recs = recs;`) and before
`--expect`/admission, so `--expect` and the refusal-by-index see it.
Admission today: `progWalletPolicy: {ClassDescriptor, ClassMDMK}`
(`gui/sysw_admit.go:53`), with the "NO seed class … least privilege" comment
§6a says this cycle reverses — both are one map entry and one comment.
See I-3 for the one rule that has no home.

**6. Lock range check, digit pad, `time` under TinyGo, headroom — MEASURED.**
There is NO numeric-entry widget: `gui/passphrase_keyboard.go:18-24` mixes digits
with punctuation on page 3 (`ppPageSymbols = "1234567890\n-/:;()&$@\"\n.,?!'+=_#"`),
and grepping the fork for `time.Date` / `time.Unix` outside tests returns
**nothing** — the six `time` importers in `gui/` all use `time.Now` for timeouts
only. So §9 item 3 is genuinely net-new, as §3's inventory says.

`time.Date(…, time.UTC).Unix()` **compiles for the real target under TinyGo** —
built and linked:

```
$ tinygo build -target pico2 -o tg.uf2 .   # program uses time.Date/UTC/.Unix()
rc=0 ; 39936-byte uf2
```

so §6b's date→Unix conversion needs no hand-rolled civil-date arithmetic, and
`time.Date`'s normalisation gives the impossible-date refusal (2027-02-31
normalises, so `d.Day() != 31` detects it) without extra code.

Every §4c/§6b arithmetic claim checks out:
2009-01-03 00:00 UTC = **1230940800**; 1985-11-05 00:00 UTC = **499996800**;
epoch 500000000 = **1985-11-05 00:53:20 UTC**; 2038-01-19 00:00 UTC =
**2147472000 ≤ 2147483647**; days 388 → `ceil(388*86400/512)` = **65475 ≤ 65535**
while 389 → **65644 > 65535**; 65535 blocks = **455.10 d**; 65535 units =
**388.36 d**.

Headroom at fork HEAD: flash **1,503,652 B**, RAM **62,592 B** (N-2).

**7. `md compose` in md-codec — MEASURED.**
md-codec already depends on miniscript: `default = ["derive"]`,
`derive = ["dep:miniscript"]`, workspace pin `miniscript 13.0.0` patched to
`rust-bitcoin/rust-miniscript` rev `ff4732e5f75aa555682343cb180fa72ee3e8e9d5`.
`cli-compiler` is on **md-cli only** and gates `md compile` alone
(`main.rs:952-971`), so `compose` need not sit behind it (M-2).
`ExtParams::new().top_unsafe()` exists at the pinned rev and is already used
(`crates/md-cli/src/parse/template.rs:2687`). §5b's carve-out is correct in an
interesting way: rust-miniscript's signature gate runs for `tr` ONLY (upstream
issue #734, quoted at `template.rs:2637-2641`), so a keyless **wsh** path passes
`Descriptor::from_str` with no flag —

```
$ md encode "wsh(or_i(multi(2,@0/…,@1/…),and_v(v:sha256(<H>),older(144))))"   # exit 0
$ md encode "tr(<NUMS>,{multi_a(2,…),and_v(v:sha256(<H>),older(144))})"
md: template parse error: miniscript parse failed: All spend paths must require a signature
```

— which is exactly §4e's split, measured. The corpus lives at
`crates/md-codec/src/test_vectors.rs` (`Vector` at `:14`, `MANIFEST` at `:70`),
is exported by `crates/md-cli/src/cmd/vectors.rs` (which already emits
`md1_encoding_id`, `wallet_descriptor_template_id`, `wallet_policy_id`, per-chain
`descriptor` and `addresses`), and is consumed by the fork at
`md/testdata_test.go`. A new family fits that shape, with the caveat in M-3.

**8. §12 item 1 product — MEASURED: 28,800 (57,600 with the origin axis).**
See I-4 for the count, the impossible-cell table and the pairwise/tagged-coverage
replacement.

**9. Everything else checked — the §9/§10 items are otherwise executable.**
Spot-verified, all TRUE:

- §9 item 2 is real and load-bearing: `grep -c tagAndOr md/script_emit.go` = 0,
  `grep -c tagPkH` = 0, and the decoder confirms `pkh(@0)` in a wsh script lands
  on `tagPkH` (dumped: `PkH @0` directly under `OrI`) — so today the device
  cannot emit Script for §5's wsh single-key head, hence cannot derive its
  address.
- §5's placeholder rule matches md's canonicalisation exactly — measured:
  `wsh(or_i(pkh(@1/…/1'/2'),and_v(v:multi(2,@0/…,@2/…),older(144))))` decodes back
  as `@0` = the FIRST-APPEARING key, carrying its origin (`@0: m/48'/0'/1'/2'`).
- Every other §5 lowering shape admits: `tr(K)` (m=0), `tr(K,P)` bare leaf (m=1),
  `{P1,P2}` (m=2), `{P1,{P2,P3}}` (m=3), the 8-leaf right spine (depth 7),
  `tr(NUMS,sortedmulti_a)`, `sortedmulti_a` under a REAL internal key,
  `or_d` over a bare multi head, `or_i(pkh(K),R)`, a bare `pkh` as the last arm,
  `wsh(pkh(@0))` as a sole single-key path, a keyless `sha256`-only path,
  `sh(sortedmulti)`, `sh(wsh(sortedmulti))`. `{P}` is refused as §5 says
  ("taptree branch must have 2 children, but found 1").
- `TemplateEngraveShapeGuardChunks` refuses only `sortedmulti` under a combinator
  (`md/template_guard.go:48-80`), which §5's lowering never produces — no clash
  with §7f.
- §7c/§9 item 6's label ambiguity is real: `gui/wallet_policy.go:194` prints the
  16-byte id labelled by kind, `gui/template_engrave.go:63-70` prints a 4-BYTE
  stub also labelled `Template-ID:`.
- §6a's motivating measurements hold: a bare xpub packs as a `pkh(xpub)` WALLET
  ("read as: a single extended key … descriptor: pkh(xpub…)"), and a
  `[fp/path]xpub` line is refused with the "not inferable" guidance.
- §9 item 8's "ONE site": `multisigScriptTypeComponent`
  (`gui/multisig_build_slots.go:125-130`) returns 1 for ShWsh else 2, over an
  `md.MultisigScript` with exactly three members (`md/encode_multisig.go:26-33`)
  — a taproot member plus a `3` arm is the whole change.
  §4f's account-by-ordinal rule is at `gui/multisig_build.go:594-601` and
  `:642-661`, as cited.
- §10 item 5: `ms derive --template` has no taproot value today
  (`bip44|bip49|bip84|bip86|bip48-p2wsh|bip48-p2sh-p2wsh|bip48`,
  `crates/ms-cli/src/cmd/derive.rs:108-116`); `script_type` returns
  `Some(2)|Some(1)|None` at `:164-167`, so `Bip48P2tr → Some(3)` is a two-line
  addition.
- §12 item 5's gates all exist: `assertModalBodyFits`
  (`gui/modal_fits_test.go:201`), `gui/raster_test.go`, and
  `scripts/{plan-glyph-check,plan-cite-check,plan-build-gate,plan-build-gate-go,
  spec-structure-check,plan-staleness-check}.sh`.
- §7f's plate machinery exists and is reusable — `planTransactionTextPlates`
  (greedy first-fit over equal-length strings, `gui/transaction.go:1145`) and
  `qrCeilingBytes` (`gui/transaction.go:1369`, which already MEASURES rather than
  reads a constant, matching §13 item 1's promise). Note the text packer packs
  WHOLE strings and cannot split one; §7f's census refusal is what covers a
  descriptor longer than a plate, which is consistent.
- §9 item 7's premise holds: `ChoiceScreen`'s layout
  (`gui/gui.go:1985-2026`) sums child heights with no scroll offset.
- §12 item 10's golden exists (`gui/testdata/t6b_multisig_full.md1.txt`, 6 chunks).
- Sizing datapoint for §13 item 1: a KEYED 2-slot wsh policy with a hash+lock
  recovery path is **5 chunks**; the keyed 3-slot tr with a locked leaf is
  **6 chunks**; both derive an address and round-trip through `md decode`
  byte-identically.

**Explicitly UNVERIFIED (and correctly listed in §13, or newly proposed for it):**
plate ceilings and the pick list's row capacity (§13-1); Ledger registration
(§13-2); Nunchuk UI (§13-3); Core/Liana/Nunchuk import of composed output
(§13-4); a Core re-run on the multipath forms (§13-5). Newly proposed for §13:
**no emulator run was performed here** — §12 items 2, 3, 5, 9 remain hypotheses
until the walk executes, and the "closure is lens-closure" rule applies: §12
item 2 is a gate that has never run.

---

## What I ran

- `git diff bc1c07c..HEAD -- design/SPEC_wallet_policy_composer.md` (empty).
- `md encode` on 30+ templates covering every §5 lowering shape, both wrappers,
  keyless/keyed, divergent/shared origins, with and without fingerprints;
  `md decode`, `md inspect`, `md address`, `md bytecode`, `md compile`,
  `md decompose`, `md descriptor` on the results.
- `md encode --in` on generated 32/33/36/72-slot policies (C-1).
- `mk encode` with 1, 2 and 3 `--policy-id-stub` values; `mk decode`;
  a byte-identity re-run for determinism.
- `ms derive --template bip48-p2wsh --account {0,1,2}`; `ms derive --help`.
- `me sysw pack --in` on a bare xpub, a `[fp/path]xpub` line and a `now:` record.
- Go 1.25.10 (`/nix/store/6rlw…-go-1.25.10/bin/go`): `go build ./md/`; a scratch
  module with `replace seedhammer.com => …` running `md.Decode`,
  `md.DecodeChunks`, `md.FormAwareIdChunks`, `md.PolicyShapeChunks`,
  `mk.Decode`/`mk.Encode`; and `go test -overlay=…` with an in-package probe in
  `md` that dumps decoded trees and `policyShape` output (no repo file written).
- TinyGo 0.41.1 (`/nix/store/ld75…-tinygo-0.41.1/bin/tinygo`):
  `tinygo build -target pico2` on a `time.Date`/`time.UTC`/`.Unix()` program;
  `tinygo build -size short -target pico-plus2 -stack-size 16kb -gc precise
  -opt 2 -scheduler tasks ./cmd/controller` for the size numbers.
- Python for the §4c/§6b date arithmetic and the §12 item 1 product count.
- Reads: `md/{md,encode,chunk,policy_shape,template_guard,template_strip,
  encode_multisig,script_emit,testdata_test}.go`,
  `gui/{key_card_seating,wallet_policy,sysw_admit,sysw_session,template_engrave,
  multisig_build_slots,transaction,passphrase_keyboard,gui}.go`,
  `sysw/{record,descriptor}.go`, `seal/record.go`,
  `crates/md-codec/src/{tag,tree,test_vectors,error}.rs`,
  `crates/md-cli/src/{main,cmd/encode,cmd/vectors,parse/template,parse/keys}.rs`,
  `crates/me-cli/src/sysw/{record,mod}.rs`, `crates/ms-cli/src/cmd/derive.rs`,
  both `Cargo.toml`s, `flake.nix`, `rust-miniscript-fork/src/miniscript/limits.rs`.

No repository file was modified other than this report.
