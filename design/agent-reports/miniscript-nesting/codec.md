# Lens: our own codec's limits on nesting — `md` codec, toolkit gate, Go port

**Subject:** the "pathological example" 4-tier degrading vault
(`/scratch/code/shibboleth/mnemonic-toolkit/.examples-build/degrade2.desc`,
11 real xpubs, `or_i` three deep, 2× `sha256`, 4 timelocks across all four
BIP-65/BIP-68 units).

**Scope:** OUR bounds, not Bitcoin's. Consensus and standardness are another
lens's job; nothing below is a consensus or relay claim.

**Method:** every number here was produced by running something. A temporary
`cargo` example against `md-codec` and a temporary Go test against
`seedhammer.com/md` were used to bypass the `md` CLI's key gate; both were
deleted and both repos verified clean (`git status --porcelain` empty).

---

## Bottom line

Nesting is not the problem. The wallet sits at **depth 6 of a 128 cap** and
**26 AST nodes against no node cap at all**. Three other things bite, in
descending order of how much they actually matter:

1. **The Go/firmware `classifyPolicy` bound** — not a size or depth limit, a
   *shape* limit. The SeedHammer II decodes all 24 cards, verifies every
   checksum, prints all 11 key rows — and then refuses to say what the policy
   is or derive an address, because `wsh(or_i(...))` is `PolicyComplex`. This is
   the real ceiling for this wallet and it is deliberate.
2. **24 md1 cards / 25 plates.** Measured. 37.5 % of the 64-chunk cap, but the
   operational cost is the story, not the headroom.
3. **`md encode` refuses these keys outright** on a CLI heuristic that the wire
   format does not even care about (BIP-32 depth 4 required; the keys are
   depth 3, and depth is not on the wire).

`build-descriptor`'s refusal is real and I found it, but it is an
**over-refusal by ~6000×**: the "always-previewable envelope" estimates 73 728
spending conditions and the actual preview is **12 rows**.

---

## 1. Where the wallet sits against every codec bound

Measured unless the cell says "n/a". `md-codec` v0.42.0,
`/scratch/code/shibboleth/descriptor-mnemonic`.

| Bound | Value | Source | This wallet | Margin |
|---|---|---|---|---|
| Decode recursion depth | `MAX_DECODE_DEPTH = 128` | `crates/md-codec/src/tree.rs:185` | max node depth **6** | 4.7 % used |
| AST node count | *none* | — | **26** nodes | n/a |
| Key universe `n` | `1..=32` (5-bit) | `crates/md-codec/src/origin_path.rs:111` | **11** | 34 % |
| `key_index_width` | `⌈log₂ n⌉` | `crates/md-codec/src/encode.rs:37-41` | **4** bits | — |
| `multi` k and n | `1..=32` each (5-bit) | `crates/md-codec/src/tree.rs:117-124` | max **3-of-3** | 9 % |
| `thresh` k and n | `1..=32` | `crates/md-codec/src/tree.rs:92-99` | not used | — |
| Origin-path depth | `MAX_PATH_COMPONENTS = 15` | `crates/md-codec/src/origin_path.rs:43` | **3** (`84'/0'/i'`) | 20 % |
| Path component value | LP4-ext varint, max `2^29-1` | `crates/md-codec/src/varint.rs:12-14,30` | 84 / 0 / 0..3 | — |
| Use-site alternatives | `2..=9` | `crates/md-codec/src/use_site_path.rs:43,45` | **2** (`<0;1>`) | at the *minimum* |
| Single md1 string | 80 data symbols / 400 bits | `crates/md-codec/src/codex32.rs:25` | **1481** symbols | **18.5× OVER** |
| Chunk sizing budget | 320 bits/chunk | `crates/md-codec/src/chunk.rs:224` | — | — |
| Chunk count | `≤ 64` | `crates/md-codec/src/chunk.rs:255` | **24** | 37.5 % |
| Effective payload ceiling | 64 × 320 b = 2560 B | derived from the two above | **926 B** | 36 % |
| Top-level wrapper allow-list | `{Sh,Wsh,Wpkh,Pkh,Tr}` | `crates/md-codec/src/decode.rs:91-106` | `Wsh` | passes |
| Taptree forbidden leaves | `is_forbidden_leaf_tag` | `crates/md-codec/src/validate.rs:203-208` | never walked | n/a |
| Timelock value | raw `u32`, **no range check** | `crates/md-codec/src/tree.rs:160-162,293-296` | all four fine | — |

Measured payload sizes (temporary `cargo run -p md-codec --example …`):

```
AST nodes (incl. wsh root)  = 26
max node depth (root = 0)   = 6      (MAX_DECODE_DEPTH = 128, headroom 122)

template-only, 11 divergent 3-component origins:
  payload 1205 bits / 151 bytes / 241 codex32 symbols  -> 4 chunks
  chunk::reassemble round-trip = OK, equal = true

wallet-policy (11 xpubs + 11 fingerprints in TLV):
  payload 7405 bits / 926 bytes / 1481 codex32 symbols -> 24 chunks
  chunk string length = 86 chars
  chunk::reassemble round-trip = OK, equal = true
```

The 64-chunk cap **is** reachable, just not by this wallet:

```
n-sweep, wsh(multi(1,@0..@n-1)), 3-component divergent origins, wallet-policy:
  n=11:  6607 bits -> 21 chunks
  n=24: 14411 bits -> 46 chunks
  n=32: 19201 bits -> 61 chunks          <- just under
n=32 with 15-component (MAX_PATH_COMPONENTS) divergent origins:
  36608 bits / 4576 bytes -> ERR "encoding requires 115 chunks; max is 64 per spec §9.8"
```

So `n ≤ 32` and the 64-chunk cap are roughly co-located for realistic paths, and
the chunk cap becomes the binding one as soon as origin paths get deep. This
wallet is nowhere near either.

---

## 2. Findings

### F1 — `md encode` refuses this wallet's keys outright (TOOLING, blocking)

```
$ md encode "<template>" --key @0=xpub6CatWdiZiodmU… …
md: --key @0: expected depth 4 for this script context, got 3
```

`ctx_for_template` (`crates/md-cli/src/parse/template.rs:2202-2216`) is a string
test on the template head: anything that is not `wpkh(`/`pkh(`/`sh(wpkh(` and not
a bare key-path `tr(` is `ScriptCtx::MultiSig`, and `parse_key` then requires
BIP-32 depth exactly 4 (`crates/md-cli/src/parse/keys.rs:66-76`). All 11 keys
here are BIP-84 *account* keys at depth 3 — measured by decoding the version/depth
prefix of three of them (`depth = 3` for `xpub6CatWdiZ…`, `xpub6DNfJehq…`,
`xpub6DBbzvud…`).

This is a **CLI heuristic, not a codec constraint**. The wire format does not
carry xpub depth at all: `parse_key` keeps only `bytes[13..78]` — chain code ‖
compressed pubkey (`keys.rs:84`). Proof: driving `md_codec` directly with the same
11 keys encodes, chunks, and round-trips cleanly (§1). The gate is a
BIP-48-shaped assumption (`wsh` ⇒ multisig ⇒ `48'/0'/0'/2'`) applied to a
miniscript body that is not BIP-48 anything.

**Affects:** every tier — it is a whole-descriptor refusal at intake.
**Workaround today:** none through `md encode`. `--path` only flattens to a
single *shared* path, and these 11 origins are divergent.

### F2 — `md encode` does not auto-chunk despite two places saying it does (TOOLING, minor)

```
$ md encode "<template>"
md: codec error: payload is 182 data symbols; the codex32 regular code caps
    single strings at 80 (use chunked encoding / --force-chunked)
$ md encode "<template>" --force-chunked
chunk-set-id: 0x3079d      (3 chunks)
```

Chunking only happens under `if args.force_chunked`
(`crates/md-cli/src/cmd/encode.rs:69` for `--json`, `:103` for text); the `else`
arm calls `encode_md1_string`, which hard-errors above 400 bits. Both
`chunk.rs:224`'s doc-comment ("A payload that fits ≤ 400 bits is emitted as ONE
string; only a payload exceeding the 400-bit single-string cap … is split") and
the CLI's own `--force-long-code` error text ("payloads >400 bits are chunked",
`cmd/encode.rs:43`) describe automatic behavior that does not exist.

Minor, not blocking: the error message names the flag, so it is recoverable and
loud. But it is a documentation/behavior mismatch in the exact path a large
policy takes.

### F3 — 24 md1 cards, 25 plates (OPERATIONAL)

`me bundle` on the 24-chunk wallet-policy set (measured, `me` v0.5.1):

```
"wallet_plates": 25,  "sets": [{ "kind":"md1", "chunk_set_id":"0x4b28c",
                                 "total": 24, "integrity":"set-verified" }]
me: backup needs 25 plates (24 public + ms1 on device)
```

No codec bound is violated (24 of 64), and `me`'s NDEF short-record limit is
untouched (an md1 chunk is 86 chars against a 255-byte cap,
`crates/me-cli/src/ndef.rs:32-34`). This is simply the number: **24 engraved
plates plus the ms1 seed plate**, each of which has to be NFC-pushed, cut, and
read back. Template-only would be 4 cards + ms1 — but a template-only md1 for a
non-canonical wrapper only PARTIAL-decodes (`md` warns: "without an explicit
origin … it cannot be reliably restored on its own"), so the 24-card
wallet-policy form is the one that actually restores.

### F4 — `build-descriptor` refuses, and the bound is ~6000× conservative (TOOLING)

Measured:

```
$ mnemonic build-descriptor --spec .examples-build/degrade2-spec.json
build-descriptor: refused — 1 diagnostic(s):
  [over_envelope] root: policy exceeds the always-previewable envelope
  (2^(11 keys + 2 hashes) × 9 timelock-states > cap 4096); use the raw
  `--descriptor` path for arbitrarily complex policies
```

**The bound, named:** `DEFAULT_PREVIEW_CAP = 4096`
(`crates/mnemonic-toolkit/src/descriptor_builder/gate.rs:33`), gate step 4,
`check_cap` (`gate.rs:574-605`). Its inputs:

- `distinct_keys` — `BTreeSet<DescriptorPublicKey>` over `for_each_key`,
  deduped (`gate.rs:607-614`) → **11**.
- `hash_and_timelock_counts` (`gate.rs:627-658`) → hash **LEAVES**, not distinct
  digests (the same `a84dce40…` digest in tiers 1 and 2 counts as **2**), and
  `n_abs × n_rel` where each factor is `1 + has_height + has_time`. This wallet
  has all four timelock kinds — absolute height (t1), absolute time (t2),
  relative blocks (t3), relative 512s (t4) — so `3 × 3 = 9`.
- `raw = 2^(11+2) × 9 = 73728 > 4096`.

**What it actually protects against:** `cost::enumerate_minimal_conditions`
(`crates/mnemonic-toolkit/src/cost/enumerate.rs:110-140`) is a literal
`for key_mask in 0..2^n_keys { for hash_mask in 0..2^n_hashes { for abs { for
rel {` nested loop with a satisfaction attempt per iteration. The same precheck
appears there (`enumerate.rs:110-127`), which is why `check_cap`'s doc insists
the two counts stay in lockstep. Measured with the cap lifted:

```
$ mnemonic compare-cost --descriptor "<degrade2.desc>" --max-conditions 73728
… 12 rows …
elapsed = 196.4 s
```

**So: the cap is protecting against a 3¼-minute hang in a cost *preview*, not
against an unsafe or unspendable descriptor.** And it is a crude bound — the
power-set estimate counts every key subset including the ~73 716 that no `or_i`
arm can satisfy. For a *tiered* vault, where the arms are disjoint by
construction, `2^(all keys)` is exactly the wrong model. The honest description
of this wallet is "12 spending conditions", not "73 728".

**Not overridable.** The five `--allow` tokens are `malleable`,
`mixed-timelock`, `repeated-keys`, `resource-limit`, `sigless-branch` (measured
from `--help`); `AllowSet` (`gate.rs:51-58`) has no field for `OverEnvelope`,
and `cmd/build_descriptor.rs:289,326` pass `DEFAULT_PREVIEW_CAP` as a constant
with no flag behind it. The only escape is the `compare-cost --descriptor
--max-conditions` path the message points at — which is a *cost* tool, not a
descriptor builder, so this wallet has no `build-descriptor` route at all.

Worth noting the gate is otherwise silent on this wallet: steps 1–3 (schema,
type-check, `sanity_check` + the mixed-timelock / sigless / malleable /
repeated-key localizers) all pass — the 3×3 timelock-state count is *not* a
`HeightTimelockCombination` finding, because the mixing is across disjoint
`or_i` arms, not within one satisfaction path.

### F5 — encoder has no depth guard; decoder does (TOOLING, latent)

`read_node_with_depth` refuses at `MAX_DECODE_DEPTH`
(`crates/md-codec/src/tree.rs:204-209`). `write_node` (`tree.rs:79-176`)
recurses with **no depth counter at all**. Measured:

```
probe or_i depth 128: encode OK (1572 bits),
                      decode = ERR decode recursion depth 128 exceeded maximum 128
```

That is precisely the "engrave-but-can't-restore gap" that the encode-side
`KGreaterThanN` mirrors at `tree.rs:100-108` and `tree.rs:125-133` were added to
close — the same class of defect, left open on the depth axis. Not reachable for
this wallet (depth 6) and not reachable through `md encode` (miniscript parses
first), but it **is** reachable through the `md_codec` library API, which both
`me` and the fork consume.

Severity here: minor, because nothing in this descriptor gets near it. As a
class it is important for library callers, and it is a five-line fix
(thread a depth counter through `write_node`, mirroring `read_node_with_depth`).

### F6 — Rust ↔ Go bounds are in exact lockstep (CLEAN, verified by execution)

Source-level, every bound matches:

| Bound | Rust | Go |
|---|---|---|
| Decode recursion depth 128 | `tree.rs:185` | `md/md.go:324` |
| Path components 15 | `origin_path.rs:43` | `md/md.go:171` |
| Alt count 2..9 | `use_site_path.rs:43,45` | `md/md.go:242`, `md/encode.go:153` |
| Chunk sizing 320 bits | `chunk.rs:224` | `md/chunk.go:39` |
| Chunk cap 64 | `chunk.rs:255` | `md/chunk.go:134` |
| Varint `l_high > 15` overflow | `varint.rs:30` | `md/encode.go:65` |
| Wire version 4 | `header.rs:27` | `md/md.go:302` |

And executed — the Rust-emitted cards fed to the Go port
(`go test ./md/` with a temporary in-package test, Go 1.26.3):

```
template-only: 4 chunks,  header chunked=true total=4  csid=0x9b9bc
  Reassemble OK n=11 ; DecodeChunks OK N=11
wallet-policy: 24 chunks, header chunked=true total=24 csid=0x4b28c
  Reassemble OK n=11 ; DecodeChunks OK N=11
  @0 fp="73c5da0a" origin="m/84h/0h/0h" usesite="<0;1>/*"
  …
  @10 fp="28645006" origin="m/84h/0h/2h" usesite="<0;1>/*"
Go maxDecodeDepth = 128 / maxPathComponents = 15 / maxAltCount = 9 /
   minAltCount = 2 / singleStringPayloadBitLimit = 320
```

The `csid=0x4b28c` agrees across all three implementations (Rust `split`, Go
`ParseChunkHeader`, and `me bundle`'s manifest). **No bound differs.**

### F7 — the Go port has one bound the Rust codec does NOT: `Renderable` (INFORMATIONAL, and the real ceiling)

`classifyPolicy` (`seedhammer/md/md.go:1266-1315`) recognizes exactly
single-sig, `multi`, `sortedmulti`, `multi_a`, `sortedmulti_a`, and a key-path-only
`tr`. Everything else falls through to `PolicyComplex`, and
`renderable := policy != PolicyComplex` (`md/md.go:1368`).

Measured on both card sets: `Policy=5 (PolicyComplex)`, **`Renderable=false`**.

Downstream on-device, all three consequences deliberate:

- `gui/md1_inspect.go:57-60` → the inspect screen prints
  `"Complex policy — cannot display safely."` plus `Keys: 11`, then the 11 key
  rows. No policy type, no k-of-N.
- `gui/md1_expand.go:83` → `scriptForTemplate` returns `!ok`, so there is
  **no `bip380.Descriptor`**: no on-device address derivation, no verify of a
  receive address against the engraved cards.
- `gui/template_engrave.go:65-80` → the consent screen degrades to the
  "honest-minimal" form: `"COMPLEX POLICY (advanced)" / "Cannot fully display
  on-device." / Script / Key slots / Template-ID / "VERIFY against your
  coordinator / toolkit BEFORE funding."`

This is the answer to the lens question. The codec's *size* and *depth* bounds
have plenty of headroom for this wallet; what it runs out of is **the firmware's
ability to say what it is**. The device will faithfully engrave 24 cards and
faithfully read them back, and will never once tell the operator that tier 3 is
2-of-2-after-65535-blocks.

### F8 — `design/CORPUS.md` §C6 is an open placeholder this wallet answers (INFORMATIONAL)

`descriptor-mnemonic/design/CORPUS.md:180-217`, "C6 — Pathological deeply-nested
miniscript (chunking forced)", concedes its own example does not work:

> "Wait: actually fits single string. Let me revise to genuinely force chunking."
> … **"Open: identify a realistic miniscript that genuinely needs chunking."**

This descriptor is that example, measured: **4 chunks template-only, 24 chunks
wallet-policy** — and it is a real wallet, not a synthetic 20-of-25. It also
exercises all four timelock units, a repeated hash digest across two arms, and
divergent per-`@N` origins in one shape. Worth landing in the corpus.

---

## 3. Risks checked that do NOT apply

Stated plainly, per the brief — a clean lens is a result.

- **`MAX_DECODE_DEPTH = 128`** — depth 6. Not close, and the probe confirms the
  wall is real and lands exactly at 128.
- **Node-count / expression-complexity cap** — there is none in the codec. The
  only implicit bound is the bit budget (64 chunks × 320 bits = 2560 B payload);
  this wallet uses 926 B.
- **`is_forbidden_leaf_tag`** (`validate.rs:203-208`) — reachable only under a
  `Tr` root (`encode.rs:107-111`, `decode.rs:129-133`). This is `wsh`; the
  taptree walker never runs. The tag list (`Wpkh|Tr|Wsh|Sh|Pkh|Multi|SortedMulti`)
  is irrelevant here even though `Multi` appears four times in the tree.
- **`MAX_PATH_COMPONENTS = 15`** — origins are 3 deep.
- **LP4-ext varint `2^29-1` ceiling** — components are 84 / 0 / 0..3. (It would
  bite a path like `m/84'/0'/999999999'`; 999 999 999 > 536 870 911. Not this
  wallet.)
- **`n ≤ 32`** — 11 keys, `key_index_width` 4 bits.
- **`multi` k/n ≤ 32** — largest is 3-of-3.
- **Multipath alt count 2..=9** — `<0;1>` is 2, the *minimum*; all 11 use-sites
  identical, so `validate_multipath_consistency` is trivially satisfied and no
  per-`@N` use-site override is emitted.
- **BIP-388 placeholder canonical ordering** (`validate.rs:17-37`) — `@0`…`@10`
  first-occur in ascending pre-order; passes without canonicalization
  reordering anything.
- **Top-level wrapper allow-list** (`decode.rs:91-106`) — `Wsh` is on it.
- **Timelock range validation** — the codec does none: `Body::Timelock(u32)` is
  written and read as a bare 32-bit field (`tree.rs:160-162`, `tree.rs:293-296`).
  `older(4255898)` with BIP-68 bit 22 set survives verbatim. Semantic timelock
  gates live in `mnemonic-toolkit` (`timelock_advisory.rs`), not in `md-codec`.
- **64-chunk cap** — 24 used. Verified reachable at n=32 with deep paths (115
  chunks, refused), so the cap is not decorative; this wallet just is not near it.
- **`me` NDEF short-record limit** (`crates/me-cli/src/ndef.rs:32-34`, 255 bytes)
  — an md1 chunk is 86 chars.
- **Go-port divergence** — none found, source and execution (F6).
- **`build-descriptor` steps 1–3** — schema, miniscript type-check, and
  `sanity_check` (including `HeightTimelockCombination`, the "wrong timelock
  loses money" guard) all pass. Only step 4 refuses.

---

## 4. Reproduction

```sh
# The refusal and its exact arithmetic
cd /scratch/code/shibboleth/mnemonic-toolkit
./target/release/mnemonic build-descriptor --spec .examples-build/degrade2-spec.json

# The bound is a preview-time bound, not a safety bound (~196 s, 12 rows)
D=$(tr -d '\n' < .examples-build/degrade2.desc)
./target/release/mnemonic compare-cost --descriptor "$D"                       # refuses, 73728 > 4096
./target/release/mnemonic compare-cost --descriptor "$D" --max-conditions 73728 # 12 rows

# The CLI key-depth refusal
cd /scratch/code/shibboleth/descriptor-mnemonic
./target/release/md encode "wsh(or_i(...))" --key @0=xpub6CatWdiZiodmU… …
#   md: --key @0: expected depth 4 for this script context, got 3

# Payload sizes / chunk counts / depth wall: temporary md-codec example,
#   crates/md-codec/examples/scratch_degrade2.rs (deleted; rebuild from §1 if needed)
# Go lockstep: temporary md/zz_scratch_degrade2_test.go (deleted),
#   PATH=/nix/store/33fw5m31lfcnk4ff2f0df7j2bxnh8lgk-go-1.26.3/bin:$PATH \
#     go test ./md/ -run TestScratchDegrade2 -v
```

Both temporary probe files were removed; `descriptor-mnemonic` and `seedhammer`
are `git status --porcelain`-clean.
