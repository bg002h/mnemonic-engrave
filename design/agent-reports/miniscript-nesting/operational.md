# Lens: OPERATIONAL — what deep nesting costs the operator and the signing stack

**Subject:** the "pathological example" 4-tier degrading vault
`/scratch/code/shibboleth/mnemonic-toolkit/.examples-build/degrade2.desc` (checksum `#4ld0crxa`, 1868 bytes).

**Date:** 2026-08-11. **Slug:** `operational`.

## Provenance of every number below

| repo | HEAD | tool | version |
| --- | --- | --- | --- |
| `rust-miniscript-fork` | `2092faa` | — | — |
| `descriptor-mnemonic` | `5a0a4f41` | `md` | 0.13.0 (md-codec 0.42.0) |
| `mnemonic-toolkit` | `c14b1e21` | `mnemonic` | 0.97.0 |
| `mnemonic-key` | `018aca0` | `mk` | 0.12.1 (vendors md-codec **0.34.0**) |
| `mnemonic-engrave` | `9d36220` | `me` | 0.5.1 |

Bitcoin Core on this box: **`Bitcoin Satellite v0.2.4 / Bitcoin Core v25.0.0`**
(`bitcoind --version`). Every Core result below was produced against a
locally-started node — regtest at first, then a **mainnet-configured**
`maxconnections=0` node, because the regtest node rejects mainnet xpubs for an
unrelated reason and that masked the real answer.

**A correction I had to make mid-run, recorded because it changes how to read
this report:** two of my Core probes returned *empty* output and I nearly wrote
them up as "Core accepts it". They were empty because a shell variable holding
the `bitcoin-cli …` invocation failed to expand, not because Core was happy.
Re-run with the command spelled out, both turned into real errors. Nothing in
the tables below is an absence-of-output result.

Structural claims about which keys can spend which tier are read off the
descriptor text *and* corroborated by `compare-cost`'s machine enumeration; I
did not hand-count anything a tool could count.

---

## Findings

### O1 — BLOCKING (operational). The printed recovery instruction is false.

`mnemonic bundle --network mainnet --descriptor-file .examples-build/degrade2.desc`
prints, on stderr, the operator's checklist. Two of its lines:

```
# Threshold: 3 of 11
# Recovery: any 3 of 11 signing keys + md1 (template card).
```

Neither is true of this wallet, in either direction.

`compare-cost` enumerates every satisfying condition. Measured output (full
table in O6) contains exactly **8 distinct minimal key-sets**:

| tier | key-set | extra requirements | # keys |
| --- | --- | --- | --- |
| 1 | `{@0,@1,@2}` | preimage + absolute **height** ≥ 1 000 000 | 3 |
| 2 | `{@3,@4}`, `{@3,@5}`, `{@4,@5}` | preimage + absolute **time** ≥ 1 893 456 000 | 2 |
| 3 | `{@6,@7}` | relative **65 535 blocks** | 2 |
| 4 | `{@8}`, `{@9}`, `{@10}` | relative **time** (BIP-68 flag, 4 255 898) | 1 |

- **Not sufficient.** There are `C(11,3) = 165` three-key subsets. Exactly **one**
  of them (`{@0,@1,@2}`) appears in the enumeration, and even that one needs the
  sha256 preimage *and* the absolute height. So the sentence is wrong for
  164 of 165 readings of "any 3 of 11".
- **Not necessary.** `{@8}` alone spends tier 4 once the relative timelock
  matures. One key, not three.

Root cause, measured to source: `crates/mnemonic-toolkit/src/format.rs:350`
hardcodes the sentence `"# Recovery: any {} of {} signing keys + md1 (template
card).\n"` from `(t, input.n)`. `t` comes from `extract_multisig_threshold`
(`crates/mnemonic-toolkit/src/cmd/bundle.rs:1217`), whose recursive arm is

```rust
Body::Children(children) => children.iter().find_map(extract_multisig_threshold),
```

— a **first-match depth-first walk**. It returns tier 1's `k = 3` and pairs it
with `n = 11`, the count of *all* key slots in the descriptor. For any
multi-branch policy that pairing describes no branch that exists. The
function's doc comment (`:1213-1216`) only promises `None` for pure single-sig;
it makes no claim about multi-branch shapes, so this is an unguarded
extrapolation rather than a violated contract — but the *output* is a
funds-relevant instruction printed with no hedge.

**Affects:** the whole wallet; the operator-facing checklist for every tier.

---

### O2 — BLOCKING (operational, recovery). The sha256 preimage is not in the backup, and nothing says so.

Tiers 1 and 2 are both gated on
`sha256(a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad)`.
The plates carry the **hash**. The **preimage** is a secret that the m-format
bundle does not model, does not encode, and never mentions.

Measured — occurrences of `preimage|sha256|timelock|older|after` (case-insensitive)
across the whole bundle emission:

```
bundle stdout (57 strings): 0
bundle stderr (checklist):  0
```

So an operator who engraves all 58 plates, verifies them, and stores them
correctly has still not backed up a value that two of the four tiers require.
If the preimage is lost, tiers 1 and 2 are permanently unspendable; the funds
survive only through tier 3 (after 65 535 blocks, ≈ 455 days at 10 min/block) or
tier 4 (after the BIP-68 relative time). That is a real, silent, multi-year
degradation of the recovery envelope, and it is invisible at backup time.

This is the single most recovery-relevant thing I found: the backup looks
complete and is not.

**Affects:** tiers 1 and 2.

---

### O3 — IMPORTANT (operational). Real backup cost is 58 plates, not 25.

The brief's figure — 3 md1 + 22 mk1 = 25 — is reproducible, but only by a
pipeline that does not work end-to-end. Measured, both ways:

| path | md1 | mk1 | public | `me bundle` verdict |
| --- | --- | --- | --- | --- |
| brief's path: `md encode --force-chunked` on the **keyless** template + `mk encode --policy-id-stub <manual>` | 3 | 22 | 25 | **exit 4 — REFUSED** |
| real path: `mnemonic bundle --descriptor-file` (default `--md1-form=policy`) | 24 | 33 | 57 | exit 0 → **58 plates** |
| `mnemonic bundle … --md1-form=template` | 4 | 33 | 37 | exit 0 → **38 plates** |

The refusal on the brief's path:

```
me: md1 set 0x3079d is incomplete/inconsistent:
    non-canonical wrapper requires explicit origin for @0, but none provided
```

and upstream of it, `md encode` itself warns that the keyless card
"cannot be reliably restored on its own"; `md decode` on that 3-chunk set exits
**4** (VERIFY-ME) with `origin: «unspecified — supply on restore»`.

The real path's `me bundle` output:

```
me: backup needs 58 plates (57 public + ms1 on device):
```

At this repo's own measured engraving rate — **~21 minutes for a full plate**
(`design/FOLLOWUPS.md:600`) — 58 plates is ≈ **20.3 hours** of machine time. The
brief's 25 would have been ≈ 8.75 h. The estimate is off by a factor of 2.3 in
plates and ~11.5 hours in wall-clock.

**Affects:** the whole backup; planning, plate stock, and machine scheduling.

---

### O4 — IMPORTANT (tooling, recovery). Key cards cannot be bound to a chunked policy via `mk`.

The `policy_id_stub` is what tells a recoverer *which* key cards belong to
*which* wallet. `mk encode --from-md1` derives it. For this wallet it cannot:

```
$ mk encode --xpub <@0> --origin-fingerprint 73c5da0a --origin-path "m/84'/0'/0'" --from-md1 <chunk>
error: md1 input rejected: wire-format version mismatch: got 9, expected 4
```

Measured with chunk 1 alone, and with all three chunks passed as repeated
`--from-md1` — same error every time.

Source: `derive_stub_from_md1` (`mnemonic-key/crates/mk-cli/src/cmd/mod.rs:73`)
calls `md_codec::decode_md1_string(md1_str)` on **one** string. A chunked md1's
header carries version 9; md-codec's header parser accepts only 4
(`vendor/md-codec/src/header.rs:43`, error text at
`vendor/md-codec/src/error.rs:33`). Any policy large enough to chunk — and this
one *must* chunk, at 182 data symbols against the codex32 regular-code cap of 80
— cannot get its real stub from the `mk` CLI.

The workaround exists and is correct: `mnemonic bundle` computes the stub itself
via `bundle_binding_stub` (`mnemonic-toolkit/src/cmd/bundle.rs:1237`), which is
form-aware and roots on `WalletPolicyId`. But an operator driving `mk` directly
gets pushed toward `--policy-id-stub <hex>`, which accepts any 8 hex characters
with no validation — I encoded 22 real key plates against the literal stub
`deadbeef` and nothing objected. Fabricated binding on a key card is a
recovery-time trap: the card looks bound and points nowhere.

**Adjacent, worth its own look:** `mnemonic-key` vendors **md-codec 0.34.0**
while the primary `descriptor-mnemonic` is at **0.42.0** — eight minor versions
of drift in the crate that defines the wire format the stub is derived from.

**Affects:** all 33 key plates.

---

### O5 — IMPORTANT (operational). The cheaper form is also the safer one, and it is not the default.

`--md1-form=template` costs **20 fewer plates** (38 vs 58, ≈ 7 hours) *and*
prints the one warning this wallet most needs. Measured stderr, template form
only:

```
warning: this is an ORDER-DEPENDENT template with 11 distinct cosigner slots — there are
         11! = 39916800 possible key→slot assignments and only one assignment reproduces this wallet.
         this is a GENERAL POLICY: a wrong assignment changes each key's SPENDING ROLE
         (timelock branch, threshold membership), not just the address.
         record the wallet-id above and/or a known receive address to complete safely.
```

The **default** (`--md1-form=policy`) prints none of it. `emit_template_order_warning`
has exactly one caller — `bundle.rs:1158` — inside the template branch
(`grep -rn emit_template_order_warning crates/ --include="*.rs"` returns the
definition and that single call site).

That is backwards for this shape. Slot order matters enormously here: swapping
`@0` and `@8` moves a key from "one of three signers on the 3-of-3 preimage
tier" to "unilateral spender after a relative timelock". The default path
engraves 20 extra plates and withholds the sentence that says so.

Two smaller things in the same area:

- The `--md1-form` help text says template form "REQUIRES a canonical single-sig
  shape (bip44/84/86); multisig / non-canonical / bip49-nested-segwit are
  refused". Measured: it **succeeded**, exit 0, on this non-canonical 11-key
  multisig. Doc/behaviour mismatch — and since the behaviour is the better one,
  the doc is what should move.
- The `# Recovery: any 3 of 11 …` line from O1 is printed in **both** forms.

**Affects:** the whole wallet; the default operator path.

---

### O6 — IMPORTANT (tooling). `compare-cost` refuses this descriptor by default, and takes 3m25s when allowed.

Default run:

```
error: compare-cost: spending conditions exceed --max-conditions cap (73728 > 4096);
       raise the cap or simplify the policy
```

`73 728 = 9 × 2^13` — 3 absolute-timelock states × 3 relative-timelock states ×
`2^(11 signers + 2 preimages)`. The default cap is `4096`
(`crates/mnemonic-toolkit/src/cmd/compare_cost.rs:39`,
`default_value_t = 4096`). This descriptor is **18× over**.

With `--max-conditions 100000`, measured timing: **204.66 s user, 3:25.45 wall.**

Full measured table (feerate 1.0, so sats == vB):

```
Condition                                               | wsh vB | tr vB |  Δ vB
--------------------------------------------------------+--------+-------+-------
key[0] + key[1] + key[2] + preimage(h0) + after(height) |    105 |   232 |  +127
key[0] + key[1] + key[2] + preimage(h1) + after(height) |    105 |   232 |  +127
key[3] + key[4] + preimage(h0) + after(time)            |     87 |   216 |  +129
key[3] + key[4] + preimage(h1) + after(time)            |     87 |   216 |  +129
key[3] + key[5] + preimage(h0) + after(time)            |     87 |   216 |  +129
key[3] + key[5] + preimage(h1) + after(time)            |     87 |   216 |  +129
key[4] + key[5] + preimage(h0) + after(time)            |     87 |   216 |  +129
key[4] + key[5] + preimage(h1) + after(time)            |     87 |   216 |  +129
key[6] + key[7] + older(blocks)                         |     79 |   208 |  +129
key[8] + older(512s)                                    |     61 |   192 |  +131
key[9] + older(512s)                                    |     61 |   192 |  +131
key[10] + older(512s)                                   |     61 |   192 |  +131
```

Operator reading: **per-tier input cost ranges 61 → 105 vB**, a 1.7× spread.
Tier 1 (the "everything is fine" tier) is the most expensive to spend; tier 4
(the last-resort single-key tier) is the cheapest. That ordering is benign — the
degraded tiers get cheaper, not dearer — which is the right way round for a
distressed recovery. The tool's own caveat applies: *"per-condition vbytes are
rounded individually; absolute numbers may differ by ±1 from real-tx
accounting, Δ values are correct."* The `tr` column is a hypothetical rewrap,
not this wallet.

Two sub-issues:

1. **Duplicate-hash inflation.** Both `sha256(...)` fragments are *textually
   identical* (`a84dce40…`), but the enumerator treats them as independent
   preimages `h0`/`h1`. That doubles the enumeration (`2^13` rather than `2^12`)
   and emits **12 printed rows for 8 distinct conditions** — 4 rows are exact
   duplicates. Deduplicating identical hashes would halve the cost and remove
   the confusing duplicate rows, though it would not by itself bring 36 864
   under 4096.
2. **A documented invariant this descriptor breaks.**
   `crates/mnemonic-toolkit/src/descriptor_builder/gate.rs:33` declares
   `pub const DEFAULT_PREVIEW_CAP: usize = 4096;` with the rationale
   *"matches `compare-cost`'s default `--max-conditions` … so a policy that
   passes this gate also renders in the Phase-3 cost preview without tripping
   `ConditionsTooMany`."* This descriptor is a live counterexample to that
   coupling. Whether it would actually pass the builder gate is worth a separate
   check — but the "always-previewable envelope" promise in that module's
   header comment does not hold at this size.

**Affects:** all four tiers (the whole enumeration); the cost-preview tooling.

---

### O7 — IMPORTANT (tooling, recovery). No erasure coding: one lost md1 plate loses the policy.

Measured. Full 24-chunk set decodes, exit 0. Remove one chunk:

```
$ md decode <23 of the 24 chunks>
md: codec error: chunk set incomplete: got 23 chunks, expected 24
EXIT=1
```

Per-chunk BCH corrects **t = 4 substitution errors**, with a detection radius of
`2t = 8` (`descriptor-mnemonic/crates/md-codec/src/error.rs:445-455`). So the
md1 set tolerates up to 4 wrong *characters* on any given plate but **zero
missing plates**, across 24 of them. The failure modes are asymmetric in a way
worth stating plainly to an operator: a scratched plate is usually survivable, a
lost plate is not.

There is no redundancy across the chunk set (the `word-card` subcommand offers an
optional RAID, but that is a different card format and is not in this path).

**Affects:** the 24-plate md1 set — i.e. the entire wallet policy.

---

### O8 — MINOR (tooling/operational). Core v25 cannot parse the canonical multipath form; the error is misleading.

Measured on the mainnet-configured node:

| input | result |
| --- | --- |
| descriptor as stored, `/<0;1>/*` | `error code: -5` — **`A function is needed within P2WSH`** |
| split single-path receive, `/0/*` | parses; `issolvable: true`; checksum **`nxmy5wta`** |
| `deriveaddresses` on the split form | `bc1q4g7564xxd9hj68hqwu5e558cqafhsklerkr0asfzqp6puq74veesrp6qss`, `bc1qx3wvfzk569qtpf28qfagk70z865r82ugqg6j5j0wcw6drajsj5dq7kpfaw`, `bc1q9hesadvdtpnpmkvqsallk9ev6wvr7rl6xlqstpsxjt5z7m9nuxaqx7ugk7` |

The miniscript itself is fine on v25 — I confirmed separately that `or_i`,
`and_v`, `v:`, `older`, `sha256` and `multi`-inside-miniscript all parse with hex
keys. It is purely the `<0;1>` multipath syntax that v25 does not accept
(multipath support is a later-version question; v25.0.0 is what is installed
here).

The good news: `mnemonic export-wallet --format bitcoin-core` **already** emits
the split receive/change pair, and its checksums match Core's exactly —
`nxmy5wta` (receive, `internal: false`) and `krflz458` (change, `internal: true`).
The supported path exists and is correct.

The operational hazard is the error text. An operator who pastes the descriptor
straight off the plates (or out of `export-wallet --format descriptor`, which
emits the multipath form) into a v25 node is told *"A function is needed within
P2WSH"* — which reads as "your script is malformed", not "split the multipath
first". Under recovery pressure that is a plausible dead end.

**Affects:** the whole wallet, at wallet-restore time.

---

### O9 — MINOR (tooling). Encoder default output is rejected by the engraver.

`md` and `mk` both default to `--group-size 5`, emitting space-separated groups
for human transcription. `me bundle` refuses exactly that:

```
me: invalid input string: non-canonical md1: interior separator ' ' at byte 5 —
    md1 must contain no '-' and no interior whitespace (the converter engraves the
    string verbatim and the checksum does not cover stripped separators)
```

The refusal is *correct* and well-reasoned (the checksum does not cover stripped
separators, so accepting them would let a transcription error through). But the
default on one side of the seam is invalid on the other, and the operator has to
know to pass `--group-size 0` at every encoder. Measured both ways: 25/25 and
57/57 strings accepted once unbroken.

**Affects:** every plate, every run.

---

### O10 — NOTE (informational). Hardware-signer expressibility: no first-class match, and nothing in the pipeline checks.

Repo-recorded evidence (`descriptor-mnemonic/design/FOLLOWUPS.md:783` and `:792`)
says Ledger's vanadium Bitcoin app (`apps/bitcoin/common/src/bip388/cleartext.rs`)
admits four timelocked-multisig shapes as first-class BIP-388 wallet-policy
variants, with these ranges:

| variant | shape | range |
| --- | --- | --- |
| `RelativeHeightlockMultiSig` | `and_v(v:multi_a(…), older(n))` | `n < 65536` |
| `RelativeTimelockMultiSig` | `older(n)` | `4194305 ≤ n < 4259840` |
| `AbsoluteHeightlockMultiSig` | `after(n)` | `n < 500000000` |
| `AbsoluteTimelockMultiSig` | `after(n)` | `n ≥ 500000000` |

Every one of this descriptor's four timelocks lands *inside* a recognised range
— `older(65535) < 65536`; `older(4255898) ∈ [4194305, 4259840)`;
`after(1000000) < 500000000`; `after(1893456000) ≥ 500000000`. That is a nice
coincidence and nothing more: all four variants are a **single flat leaf** of the
form `and_v(v:multi_a(…), <timelock>)`, whereas this wallet is a **three-deep
`or_i` nest** of `multi` (not `multi_a`) with sha256 preimages inside two
branches. It matches none of them and would be handled, if at all, as a general
policy.

**Provenance caveat, stated because it changes how much weight this deserves:**
that is the repo's *record* of an external codebase, dated 2026-04-28. Vanadium
is not on disk here and I did not verify it against Ledger source. Treat the
ranges as a lead to confirm, not as an established fact.

Separately: `descriptor-mnemonic/design/MD_SCOPE_DECISION_2026-04-28.md:5`
records that MD's scope was deliberately narrowed to "encoding only", with
signer-compatibility moved out and `validate_tap_leaf_subset` "no longer invoked
by default". That is a defensible layering decision, but its consequence here is
concrete: **nothing anywhere in this pipeline tells the operator whether any
signer can sign this policy.** The plates will engrave, the wallet will restore
watch-only, and the first discovery of a signing problem would be at spend time.

**Affects:** all four tiers.

---

## Clean — risks checked that do NOT apply

These were checked against real tooling and came back clean. Recording them so
nobody re-derives them.

1. **BIP-388 expressibility — clean.** `export-wallet --format bip388` produces a
   well-formed policy: 11 placeholders, introduced in order `@0…@10`, each used
   exactly once, each with an identical `/**` suffix. The nesting depth is not a
   problem for the wallet-policy language.

2. **Wallet-policy round trip — clean, and byte-exact.** policy JSON →
   `export-wallet --format descriptor` reproduces the original **1868 bytes
   including the `#4ld0crxa` checksum**; `cmp -s` reports identical. Verified by
   byte comparison, not by eyeballing the checksum.

3. **BIP-388 distinct-key rule — clean.** All 11 xpubs are distinct, so the
   "decaying multisig must not reuse keys" rule (`descriptor-mnemonic/design/CORPUS.md:338-344`)
   is satisfied. Independently corroborated: Core v25 *does* raise
   `is not sane: contains duplicate public keys` — I triggered it accidentally
   with a deliberately duplicated key in a probe — and does **not** raise it for
   this descriptor.

4. **md1 self-sufficiency — clean, and better than expected.** The 24-chunk
   `--md1-form=policy` md1 carries everything needed for a watch-only restore, on
   its own: `md decode --json` shows 11 pubkey entries of **65 bytes each
   (32-byte chaincode + 33-byte pubkey)**, 11 fingerprints, and
   `path_decl.tag = Divergent`. Deriving addresses from the **plates alone**
   (`md address <24 chunks>`) reproduces Bitcoin Core's addresses character for
   character (the three `bc1q…` values in O8). The 33 mk1 plates carry no key or
   origin data that the md1 set does not already hold — they are per-cosigner
   distribution artifacts, not additional recovery information.

5. **Divergent per-key origins — clean.** The 11 keys use eight distinct account
   paths across three master fingerprints (`73c5da0a` accounts 0-3, `b8688df1`
   accounts 0-3, `28645006` accounts 0-2). md-codec encodes these natively as
   `Divergent`; nothing is flattened to a single shared path. (`md encode --path`
   *would* flatten them and be wrong for 8 of 11 keys — but the real bundle path
   never invokes it.)

6. **Address-derivation agreement — clean.** Two independent implementations —
   `md` reading the engraved plates, and Bitcoin Core reading the descriptor —
   produce identical addresses at indices 0, 1, 2.

---

## Two hygiene notes (not findings)

- **`.examples-build/degrade2.out` is stale** relative to `degrade2.desc`:
  different preimage hash (`68100fc148a239c4…` vs `a84dce40975727c3…`) and
  different checksum (`r3dwp4km` vs `4ld0crxa`). Anyone using the `.out` file as
  a reference is looking at a different wallet.
- **The conformance corpus does not reach this size.** `design/CORPUS.md`'s
  summary table tops out at 1 chunk for every named entry, with `C6`
  ("Chunking-forced") listed as `2+ / TBD`. This wallet is **24 chunks** — an
  order of magnitude beyond anything the corpus pins. The codec handled it
  correctly in every test I ran; the point is that nothing in the suite would
  have caught it if it hadn't.

---

## What I would fix first, if it were mine to rank

1. **O2** — back up the preimage, or refuse to call the bundle complete. A backup
   that omits a spending precondition for half the tiers, silently, is the
   failure mode this whole exercise exists to prevent.
2. **O1** — stop printing a k-of-n recovery sentence for policies that have no
   single k-of-n. For a multi-branch shape, either enumerate the branches or say
   "this policy has 8 spending conditions; see the descriptor" — anything but a
   confident wrong number.
3. **O5** — make the order-dependence warning fire in *both* md1 forms; it is
   currently attached to the branch that needs it less.
4. **O4** — teach `derive_stub_from_md1` to reassemble a chunk set, so key cards
   can be bound to a policy that chunks.
