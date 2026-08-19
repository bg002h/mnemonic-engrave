# Recon ruling — does anything have to land in Rust FIRST before the Go fork grows on-device miniscript rendering + address derivation?

**Agent:** recon / design-level ruling. Read-only except this file.
**Date:** 2026-08-18.
**Pins:** Rust primary `descriptor-mnemonic` @ `89ab0f62` (`main`), md-codec v0.42.0. Go fork `seedhammer` @ working tree. Rust workspace suite **GREEN at the time of measurement** (`cargo test --workspace` → 30 `test result: ok` lines, `0 failed` on every one), so every defect below is a live defect in a passing tree, not a broken checkout.

---

## Ruling

**Rust work is required first — the reference is NOT complete enough to port against.** Address *derivation* in Rust is genuinely broad and I confirmed it by running it (arbitrary `wsh(<miniscript>)`, unsorted `multi`, `sh(multi)`, `thresh`, `andor`, hash-locks, `tr(key)`, `tr(NUMS,multi_a)`, multi-leaf and depth-3 tap-trees all derive real addresses), so the *derivation* half of the doc claim in `derive.rs:3-8` survives execution with one named exception (`sortedmulti_a`). But the *rendering* half does not: `md-codec`'s template renderer emits, for two ordinary wrapper shapes, a string that **rust-miniscript's own parser rejects** (`vj:` is rendered as `v:j:`, `vdv:` as `v:d:v:`), and it emits `or_i(0,X)` / `or_i(X,0)` where every rust-miniscript-based wallet on the planet displays `l:X` / `u:X` — so "the exact text a device should show" is currently *wrong* for those shapes rather than merely undefined, and a Go device that faithfully ports today's `render.rs` would faithfully port a defect onto an engraved plate. Compounding it, and this is the decisive practical blocker: **there is no machine-readable conformance-vector export at all.** `md vectors --out` emits 60 files across 15 vectors containing **zero xpubs, zero addresses, zero wallet ids, zero script hex**; `md address --json` emits addresses and nothing else; `md inspect --json` emits ids and nothing else; and no CLI surface exposes the canonical descriptor STRING path (`to_miniscript_descriptor_multipath`) at all. The Go fork's existing conformance goldens were therefore hand-transcribed from `md inspect` at a **stale commit that its own README admits was never re-executed** — which is exactly the seam a rendering/derivation port must not be built on. Note finally that only `md/md.go` is bound by the Rust-primary rule: `bip380/bip380.go` and `address/address.go` are **upstream SeedHammer code** (both present in `upstream/main`), hence exempt clause (b), so most of the "Go narrowness" in the brief is *not* a Rust-first item.

---

## Ordered work list

### Rust first (blocking — these define semantics the Go port must reproduce)

1. **R1 — Fix the `v:` wrapper-chain rendering break.** `render.rs` gives `Tag::Verify` its own arm (`render.rs:150-163`) that pushes `"v:"` and recurses into `render_node`, instead of letting `Verify` participate in `render_wrapper_chain` (`render.rs:358`), whose dispatch (`render.rs:217`) covers only `Check | Swap | Alt | DupIf | NonZero | ZeroNotEqual`. Result: a `v:` immediately above another wrapper emits a second `:`, producing a string rust-miniscript refuses to parse. **Land with a test vector** for at least `vj:` and `vdv:`. Critical: this is a normative rendering-semantics change and the Go device would show the broken string on a confirm screen.
2. **R2 — Decide and pin the `l:` / `u:` normalization.** Rust today renders `or_i(0,X)` / `or_i(X,0)`; rust-miniscript's canonical `Display` renders `l:X` / `u:X`. Both re-parse, so this is not a break, but it is a *divergence in the text the operator compares against their coordinator*. Pick one, write it into the spec, and pin it with a vector. (`t:X` = `and_v(X,1)` is in the same family; `Tag::True` exists at `tag.rs:88` so the shape is representable, but I did not get a well-typed `t:` case through the parser — see Open.)
3. **R3 — Ship a machine-readable conformance-vector export.** This is the single most likely "must land in Rust first" item and it is confirmed. Needed: one command emitting, per vector, `{template string, per-@N xpubs + fingerprints, canonical descriptor string, scriptPubKey hex, addresses[chain][0..N], WalletDescriptorTemplateId, WalletPolicyId, Md1EncodingId, md1 chunk strings}`. Extend `test_vectors::MANIFEST` (`test_vectors.rs:68-117`) so its 15 entries carry keys — 13 of 15 currently have `keys: &[]` — and add the derived fields to `md vectors --out`. Without this the Go side has nothing to conform *to*.
4. **R4 — Add `--path` to `md address` and `md verify`.** Both lack it (verified against `md address --help`, `md verify --help`, and `md gui-schema`); `md encode` has it. Consequence: every non-canonical shape — i.e. exactly the miniscript/taptree shapes this cycle is about — is unreachable through the `--template` route and can only be exercised by round-tripping through an md1 phrase. This blocks R3 from being generated cleanly.
5. **R5 — Close or formally fence `sortedmulti_a`.** It is a first-class wire tag (`tag.rs:109`, code `0x09`), it renders (`render.rs:109`, and the frozen KAT `render_template_snapshot.rs:58-61` asserts it), but it **cannot be encoded by the CLI** and **cannot be derived** (`to_miniscript.rs:581-586`). Already filed in the Rust repo as `sortedmulti-a-derive-gap-fenced` (`design/FOLLOWUPS.md:17`) — but it must be *decided* before the Go device is asked what to display for such a card, because today Rust can render one and cannot price one.

### Go after (only once R1–R4 land)

6. Port the corrected rendering semantics into a Go `Descriptor → template string` renderer. **The fork has none today** — `md/` contains `template_strip.go` and `template_guard.go` but no renderer; the only string-shaped surface is `md.Template` (`md/md.go:1205-1226`), a summary struct (`N, Root, Policy, K, M, Keys, Renderable, InnerWsh, InnerWpkh`), not a renderer.
7. Widen `classifyPolicy` (`md/md.go:1265`) beyond the shapes it admits. **This is the Rust-bound file** (`md/md.go` does not exist in `upstream/main`).
8. Add the missing unsorted-`multi` arm and any further arms to `scriptForTemplate` (`gui/md1_expand.go:82-121`). Rust derives `wsh(multi(...))` and `sh(multi(...))` correctly today (run below), so this is exempt clause (a) — convergence, not leading — and does **not** need a Rust change.
9. Widen `bip380/bip380.go:300-340` / `address/address.go:95-155` only as fork-native work (exempt clause (b), upstream files), gated by the R3 vectors.

**No Rust-first requirement exists for items 8 and 9** — but items 1–4 are hard prerequisites for 6 and 7.

---

## Evidence: rendering completeness

`render.rs` covers 35 of the 36 tags in `tag.rs:15-89` directly; the 36th (`Tag::TapTree`) is handled by `render_tap_node` (`render.rs:516-541`), so the `other =>` fallback (`render.rs:285-288`) is structurally unreachable. Coverage is therefore *total* — the problem is fidelity, not gaps.

| Shape | Rust render site | Verdict |
| --- | --- | --- |
| unsorted `multi` | `render.rs:106` → `render_multi` | **Correct.** Round-trip stable, re-parses, derives. |
| `sortedmulti` | `render.rs:107` | **Correct.** Stable + re-parses. |
| `multi_a` | `render.rs:108` | **Correct.** Stable + re-parses + derives. |
| `sortedmulti_a` | `render.rs:109` | **Renders but cannot derive** — see `to_miniscript.rs:581-586`. Also cannot be produced by `md encode`. |
| `a:` `s:` `c:` `d:` `j:` `n:` chain | `render.rs:217` → `render_wrapper_chain` (`render.rs:358-435`) | **Correct**, collapses to `snj:` form. KAT at `render_template_snapshot.rs:100-104`. |
| `v:` above another wrapper | `render.rs:150-163` — a **separate arm**, not in the chain collapser | **BROKEN.** Emits `v:j:` / `v:d:v:`; rust-miniscript rejects both. |
| `l:` / `u:` | no tag; encoded as `OrI`+`False` and rendered via `render.rs:192` | **Diverges** from rust-miniscript Display (`or_i(0,X)` vs `l:X`). Re-parses, so not a break. |
| `c:pk_k(K)` | `render.rs:409-431` collapse | **Correct** — matches miniscript's `pk(K)` sugar. |
| key expression `@i/<0;1>/*` | `render_key` (`render.rs:543-572`) | **Correct**, incl. per-`@N` overrides (`render.rs:549-551`). |
| taptree braces `tr(K,{A,B})` | `render_tap_node` (`render.rs:516-541`) | **Correct at every depth I tested (1, 2, 3).** Notably **avoids** the upstream rust-miniscript ≥3-leaf `Display` bug documented at `design/FOLLOWUPS.md:1959-1960`, because it emits explicit binary braces rather than delegating to miniscript's formatter. |

**The frozen KAT does not catch the `v:` bug.** `render_template_snapshot.rs` has 14 entries; its wrapper-chain case is `snj:and_v(...)` (`render_template_snapshot.rs:100-104`) — `s`, `n`, `j` are all inside the collapser's dispatch set. **No corpus entry places `v:` immediately above another wrapper**, which is precisely the defective path. The test passes:

```
$ cargo test -p md-codec --test render_template_snapshot
test renderer_matches_frozen_md_cli_0_11_2_snapshot ... ok
test result: ok. 1 passed; 0 failed; ...
```

### The break, proven both directions

md-codec side (`md encode` → `md decode` → `md encode`):

```
=== wsh(or_d(pk(@0/<0;1>/*),and_v(vj:pk(@1/<0;1>/*),older(9))))
  decode emits : wsh(or_d(pk(@0/<0;1>/*),and_v(v:j:pk(@1/<0;1>/*),older(9))))
  TEXT STABLE  : NO
  re-encode    : FAIL -> md: template parse error: miniscript parse failed: separator ':' occurred multiple times (second time at position 135)

=== wsh(or_d(pk(@0/<0;1>/*),and_v(vdv:older(9),pk(@1/<0;1>/*))))
  decode emits : wsh(or_d(pk(@0/<0;1>/*),and_v(v:d:v:older(9),pk(@1/<0;1>/*))))
  TEXT STABLE  : NO
  re-encode    : FAIL -> md: template parse error: miniscript parse failed: separator ':' occurred multiple times (second time at position 135)
```

rust-miniscript v13 side, same ASTs, confirming md's output is the wrong string and not a parser quirk (scratch crate, `Miniscript::<DescriptorPublicKey, Segwitv0>::from_str` then `Display`):

```
A vj: chain
  input   : and_v(vj:pk([00000000/48'/0'/0'/2']xpub6DkFA…/0/*),older(9))
  Display : and_v(vj:pk([00000000/48'/0'/0'/2']xpub6DkFA…/0/*),older(9))

B v:j: split
  input   : and_v(v:j:pk([00000000/48'/0'/0'/2']xpub6DkFA…/0/*),older(9))
  PARSE ERR: separator ':' occurred multiple times (second time at position 10)

C vdv: chain
  input   : and_v(vdv:older(9),pk([00000000/48'/0'/0'/2']xpub6DkFA…/0/*))
  Display : and_v(vdv:older(9),pk([00000000/48'/0'/0'/2']xpub6DkFA…/0/*))
```

So rust-miniscript's canonical text for that AST is `vj:` — which md-codec renders as `v:j:`, which rust-miniscript then refuses.

### The `l:` / `u:` divergence, same method

```
=== wsh(and_v(v:pk(@0/<0;1>/*),l:older(144)))
  decode emits : wsh(and_v(v:pk(@0/<0;1>/*),or_i(0,older(144))))
  TEXT STABLE  : NO
  re-encode    : OK

=== wsh(and_v(v:pk(@0/<0;1>/*),u:older(144)))
  decode emits : wsh(and_v(v:pk(@0/<0;1>/*),or_i(older(144),0)))
  TEXT STABLE  : NO
  re-encode    : OK

=== wsh(thresh(2,pk(@0/<0;1>/*),s:pk(@1/<0;1>/*),sln:older(144)))
  decode emits : wsh(thresh(2,pk(@0/<0;1>/*),s:pk(@1/<0;1>/*),s:or_i(0,n:older(144))))
  TEXT STABLE  : NO
  re-encode    : OK
```

rust-miniscript re-normalizes the other way — feeding it `or_i(0,older(144))` prints `l:older(144)`:

```
E or_i(0,X)
  input   : and_v(v:pk(…),or_i(0,older(144)))
  Display : and_v(v:pk(…),l:older(144))
F u: sugar
  input   : and_v(v:pk(…),u:older(144))
  Display : and_v(v:pk(…),u:older(144))
```

Shapes that ARE text-stable and re-parse (control set): `wsh(sortedmulti(…))`, `wsh(multi(…))`, `wsh(and_v(v:pk,older))`, `wsh(or_d(pk,and_v(v:pkh,older)))`, and all three tap-trees below. `wsh(c:pk_k(@0/…))` → `wsh(pk(@0/…))` is an intended sugar collapse and re-parses.

---

## Evidence: derivation, as RUN

Built at `89ab0f62`: `cargo build` → `Finished dev profile`. Keys are depth-4 `m/48'/0'/N'/2'` and depth-3 `m/86'/0'/0'` xpubs derived from the BIP-39 `abandon…about` seed; my derivation was validated by reproducing `MS0` byte-for-byte against the repo's own known-good constant at `crates/md-cli/src/parse/keys.rs::XPUB_DEPTH4`.

`md address --template` requires depth 3 for single-sig and depth 4 for multisig (`crates/md-cli/src/parse/keys.rs:75`).

**Direct `--template` route (canonical shapes only):**

```
### A1 wsh(sortedmulti 2-of-3)
bc1q2sz6vvu6k7y9gtc6kfgfe0p6xkhmvmdlu97eecjkykpdktvps08scdjgr5
bc1qg8fpeqrl9uf3w5vawye5s235xylqhyxd7gjs4hq78crn6sm5w3asar4472

### A2 wsh(multi UNSORTED 2-of-3)
bc1q8z8kvwnpeqy79hfkggrtfm26hkgq2tu708a86tcwtm5gy5wrc85s99fe24
bc1qg8fpeqrl9uf3w5vawye5s235xylqhyxd7gjs4hq78crn6sm5w3asar4472

### A5 tr(key) depth3
bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr

### A3 wsh(or_d(pk(@0),and_v(v:pkh(@1),older(1000))))
md: codec error: non-canonical wrapper requires explicit origin for @0, but none provided

### A4 wsh(thresh(2,pk,s:pk,snl:older))
md: codec error: non-canonical wrapper requires explicit origin for @0, but none provided
```

A3/A4 are **not** derivation failures — they are the R4 CLI gap: `md address` has no `--path` with which to supply the explicit origin the codec demands. Routed through `md encode --path … | md address <phrases>` the same shapes derive:

```
### B1 wsh(or_d(pk,and_v(v:pkh,older(1000)))) — address from phrases
bc1qm2erg9d35wccfxyld3a79jtvt39vaqtqam9nqut88ydcc68wp74ssghsz6
bc1qqvar6kmqp8h2677qz8vs9aplxkggknr3wqleyz6elfgzztmau6eqnjakds
### B1b decode → template
wsh(or_d(pk(@0/<0;1>/*),and_v(v:pkh(@1/<0;1>/*),older(1000))))
```

**Full shape battery** (`md encode --path "48'/0'/0'/2'" … --force-chunked` → `md decode` + `md address --count 1`):

```
### C1 tr(NUMS,multi_a)                      ADDR bc1pz5s2f3a6ndm7nfelva73yxcfg0ylhes60tf25lw3a26ug2v5m6fqljvsm8
### C2 tr(NUMS,{multi_a,and_v(v:pk,older)})  ADDR bc1p9kcphqlpq5y9vvcttpcxjxz7pkd2nx933x3jg3eanzsp66pwlgqqjr57dh
### C3 tr(key,{pk,pk})                       ADDR bc1pcyudk5taqajtukqglj9pp73y5mupy05eqeluzr540cngv9ttnxuqj34udt
### C4 DEPTH-2 taptree {{pk,pk},{pk,pk}}     ADDR bc1p8ayhwuau6zds098nwh326ukp38wyzy7x6thjeram7w6kmzgmy9vqryng9k
### C5 tr(NUMS,sortedmulti_a)
      ENCODE FAILED: md: template parse error: miniscript parse failed: unrecognized name 'xpub6DXuQW1FgeHbgDHmM5…'
### C6 tr(NUMS,sortedmulti)
      ENCODE FAILED: md: template parse error: miniscript parse failed: unrecognized name 'xpub6DXuQW1FgeHbgJuoY…'
### C7 wsh(multi UNSORTED)                   ADDR bc1q8z8kvwnpeqy79hfkggrtfm26hkgq2tu708a86tcwtm5gy5wrc85s99fe24
### C8 sh(multi UNSORTED) legacy             ADDR 3BQnjCvvT44kb1bXCKrt3vveLqq2bv253u
### C9 sh(sortedmulti)                       ADDR 331ruadTuNWk7UvjA1wKgMSndirtjjuvTG
### C10 wsh(thresh(2,pk,s:pk,sln:older))     ADDR bc1qjnyc35f5qwjztjq09tqq5essyr6z6sjzfs75g8ru2c7zwf48xxys3kd5am
      (render diverges — see above)
### C11 wsh(andor)                           ADDR bc1qpru88wxsrv5tfmfcgzm8hcqwyt49mvwmk0zt3wswhvup337fdgjsshqtp4
### C13 wsh(and_v(v:pk,or_d(sha256,and_v(v:hash160,older))))
                                             ADDR bc1q6539p0ghmczcep4zqdrpj5uvf4tj4jn7ddyu2qz93h6gqyserwpqk4y5zc
```

The C5/C6 "unrecognized name '<xpub>'" errors are the md-cli template parser substituting synthetic xpubs for `@i` and handing the result to rust-miniscript, which has no `sortedmulti`/`sortedmulti_a` fragment in a tapleaf position. So neither shape can be *produced* by the CLI at all.

**Tap-tree depth battery** (all three text-stable, re-parse, and derive):

```
=== tr(NUMS,{pk(@0),{pk(@1),pk(@2)}})            stable YES  reparse OK  addr bc1p5c5j3vqht90rdgd0k7epwlvmtgz9pnfm03nv8z7e3svd2c5s27tqrhn05e
=== tr(NUMS,{{pk(@0),pk(@1)},{pk(@2),pk(@3)}})   stable YES  reparse OK  addr bc1p8ayhwuau6zds098nwh326ukp38wyzy7x6thjeram7w6kmzgmy9vqryng9k
=== tr(NUMS,{{{pk(@0),pk(@1)},pk(@2)},pk(@3)})   stable YES  reparse OK  addr bc1pshujlfq0ftpp59lqs7fxfayhdl5skh4q8ykzczs45eynwu597ycqmc463g
```

**`sortedmulti_a` derivation, proven with a control.** Because `md encode` refuses the shape, I decoded the frozen KAT wire string (`render_template_snapshot.rs:60`), injected valid xpubs so `MissingPubkey` could not mask the real error, and called `derive_address` directly (scratch crate against `md-codec` by path):

```
template rendered : tr(50929b74…03ac0,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))
SORTEDMULTI_A DERIVE ERROR: address derivation failed: Tag::SortedMultiA must be a tap-leaf root child; rust-miniscript v13 has no Terminal::SortedMultiA fragment
control template  : tr(50929b74…03ac0,multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))
MULTI_A ADDRESS (control): Address<NetworkUnchecked>(bc1pzeaaknl303m726xl83nxsqd5flqd9aa5slw37tteswfuvvgy55sq46sjtf)
```

The control rules out my harness: the identical path derives fine for `multi_a`.

**Net:** the `derive.rs:3-8` doc claim is TRUE for multi-leaf tap-trees, `tr(NUMS,…)`, `sh(multi)` and arbitrary `wsh(<miniscript>)` — verified by execution, not by reading — and FALSE for `sortedmulti_a`.

---

## Test-vector export

### What exists

- `md vectors --out <DIR>` — regenerates the canonical corpus. Ran it: **60 files, 15 vectors × 4** (`.template`, `.bytes.hex`, `.phrase.txt`, `.descriptor.json`).
- `md inspect` / `md inspect --json` — emits `template`, `n`, `wallet-policy-mode`, `md1-encoding-id`, `wallet-descriptor-template-id`, `wallet-policy-id`, `wallet-policy-id-fingerprint`.
- `md address --json` — emits `{addresses:[{address,chain,index}], network, schema}`.
- `md decode --json`, `md bytecode --json`, `md compile --json`, `md encode --json`, `md repair --json`.

### What does not exist

- **The vector corpus carries no keys and no derived data.** Machine-checked over the 60 emitted files: `grep -l xpub *.json` → `0`; `grep -il "address|wallet_id|walletid|policy_id" *.json` → `0`. Cause is upstream in the manifest: **13 of the 15 `MANIFEST` entries have `keys: &[]`** (`test_vectors.rs:68-117`); the two with data carry only `fingerprints`. So `.descriptor.json` is a pure AST dump (`n`, `path_decl`, `use_site_path`, `tree`).
- **No command emits a descriptor string + addresses + ids together.** `md address --json` has addresses but no ids and no descriptor string; `md inspect --json` has ids but no addresses. Producing a conformance row today requires correlating two commands by hand.
- **No script hex / scriptPubKey anywhere** in any output surface.
- **The canonical descriptor STRING path is unreachable from the CLI.** `to_miniscript_descriptor_multipath` — the function that would render the faithful `<0;1>` multipath descriptor via rust-miniscript's own `Display` — has **zero non-test callers**: its only references are the `pub use` at `lib.rs:69`, a doc mention at `validate.rs:123`, and four call sites in `crates/md-codec/tests/per_key_use_site_override.rs:387,409,446,481`. The CLI's only string producer is `render.rs` (via `crates/md-cli/src/cmd/decode.rs:55`) — i.e. the divergent one.
- **`--path` is missing from `md address` and `md verify`** (confirmed against both `--help` outputs and the `md gui-schema` flag dump), so the exact non-canonical shapes needing vectors cannot be driven from a template.

### How the fork's goldens were actually seeded — and why they cannot be trusted as a base

`seedhammer/md/testdata/vectors/README_multisig.md` states it verbatim:

> Rust CLI cross-check (descriptor-mnemonic @ c85cd49, md-codec v0.36.0; depth-4 xpub required for ScriptCtx::MultiSig, md-cli/src/parse/keys.rs:67-77). Not re-run against the current pin (S0 D8, descriptor-mnemonic @ `5a0a4f41`, md-codec v0.42.0): the encode_payload wire format the cross-check exercises is confirmed byte-identical 0.36.0 → 0.42.0 (see `../README.md`), so the conclusion stands, but the literal command outputs below were captured at c85cd49 and have not been re-executed

`seedhammer/md/template_id_test.go:78` is the same pattern for the template id — "Golden from md inspect (descriptor-mnemonic@54dd765, md-codec v0.37.0)", a hand-transcribed value.

And the goldens themselves carry no addresses. `md/testdata/vectors/multisig_wsh_full.meta.json` carries `script, k, n, origin_mode, shared_origin, cosigners[], payload_hex, wallet_policy_id, wallet_policy_id_stub` — no address, no descriptor string. `singlesig_pkh.meta.json` adds `md1_encoding_id`, still no address.

**This is the headline.** The mechanism the Go side would need in order to conform on rendering + derivation does not exist, and the mechanism it used for the *previous* (narrower) surface is a stale manual transcription that its own README flags as un-re-executed. Building address derivation on that base means the first divergence is discovered on a plate.

---

## The two wallet ids

Both exist and both are CLI-reachable via `md inspect` (`crates/md-cli/src/cmd/inspect.rs:7-8,26-36`).

**`WalletDescriptorTemplateId`** (`identity.rs:55`, computed at `identity.rs:71-111`): canonicalizes placeholder indices on a clone (`identity.rs:78-79`), then hashes **only** `use_site_path` bits ‖ tree bits ‖ the `UseSitePathOverrides` TLV entry bits (`identity.rs:84-105`), `SHA-256(...)[0..16]`. Excludes header, origin-path-decl, `Fingerprints` TLV, HRP and BCH checksum — so it is invariant to origin/account changes **and to whether any keys are present at all**.

**`WalletPolicyId`** (`identity.rs:122`, computed at `identity.rs:186-285`): canonicalizes placeholders, hashes the placeholder-form tree bytes ‖ per-`@N` canonical records, where each record is `presence_byte ‖ varint(path_bit_len) ‖ path_bits ‖ varint(use_site_bit_len) ‖ use_site_bits ‖ fp? ‖ xpub?` and `presence_byte = (fp_present | (xpub_present << 1)) & 0b0000_0011` (`identity.rs:264`). `SHA-256(...)[0..16]`. So it is **presence- and value-significant on the xpub and fingerprint axes**.

### Worked example — they differ, run on `wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))`

```
############ A: KEYLESS TEMPLATE card (no --key) ############
wallet-policy-mode: false
md1-encoding-id:               7b859d0c14ec2c1005659821b6009d71
wallet-descriptor-template-id: b02b44037119e6b6fd1d82f61aa17e21
wallet-policy-id:              e78d105b83c3eeb9e40f93767be2c706

############ B: FULL POLICY card (3 xpubs) ############
wallet-policy-mode: true
md1-encoding-id:               ed2276f14d38fe58488907a50a89c7dd
wallet-descriptor-template-id: b02b44037119e6b6fd1d82f61aa17e21
wallet-policy-id:              5ecbf0a3e3dddb1f57a4bb7b9ccebe03

############ C: FULL POLICY, DIFFERENT key set (@2 swapped) ############
wallet-policy-mode: true
md1-encoding-id:               8573b9198d9ef762ada1bda64168648b
wallet-descriptor-template-id: b02b44037119e6b6fd1d82f61aa17e21
wallet-policy-id:              328c05436fea9ad36dfe816d4811b12c
```

**Plainly:** the `WalletDescriptorTemplateId` is byte-identical across all three (`b02b4403…`) — it is the *shape*. The `WalletPolicyId` differs in all three (`e78d105b…` / `5ecbf0a3…` / `328c0543…`) — keyless ≠ keyed, and one key set ≠ another. This is exactly the property the cycle design depends on, and it holds. A keyless template and its corresponding full policy **do** produce different `WalletPolicyId`s while sharing a `WalletDescriptorTemplateId`.

**Minor observation, not in scope but noted for the fold:** cards B and C print `wallet-policy-mode: true` yet still emit `note: stdout is a keyless descriptor template (no keys)`. The advisory contradicts the mode line on the same screen. Worth a look before this text is mirrored onto a device.

---

## Defect symmetry

| Go narrowness | File:line | Same in Rust? | Rust-first? |
| --- | --- | --- | --- |
| `classifyPolicy` returns `PolicyComplex` for any `tr`-with-tree | `md/md.go:1265`+ | **No.** Rust derives `tr(key,{…})`, `tr(NUMS,multi_a)`, and depth-1/2/3 tap-trees (C1–C4 above) and renders them text-stably. | **No** — Go-only, but blocked on R1/R3 for the *rendering* half. `md/md.go` IS the Rust-bound port (absent from `upstream/main`). |
| `classifyPolicy` returns `PolicyComplex` for any combinator | `md/md.go:1265`+ | **No.** Rust handles `or_d`, `and_v`, `andor`, `thresh`, hash-locks (C10–C13). | **No** — Go-only, same blocking. |
| `scriptForTemplate` has no unsorted-`multi` arm | `gui/md1_expand.go:82-121` (fall-through comment at 118-120) | **No.** Rust derives `wsh(multi(…))` → `bc1q8z8kvw…` and `sh(multi(…))` → `3BQnjCvv…`. | **No** — exempt clause (a), convergence to already-correct Rust. |
| `scriptForTemplate` has no `multi_a` / `sortedmulti_a` / taptree arm | same | **Mixed.** `multi_a` and tap-trees derive in Rust; **`sortedmulti_a` does not** (`to_miniscript.rs:581-586`, proven above). | **Yes, for `sortedmulti_a` only** — R5. The rest is Go-only. |
| `bip380` accepts only `sortedmulti` as an inner function; `tr(sortedmulti(…))` parses then fails at derivation | `bip380/bip380.go:300-340` | **Rust is narrower, and earlier.** `tr(NUMS,sortedmulti(…))` is refused at *template parse* (C6), never reaching derivation. So Rust does not carry this parse-then-fail-late asymmetry. | **No** — and `bip380/bip380.go` is **upstream SeedHammer** (`git log upstream/main -- bip380/bip380.go` → `ea4b65b`), hence exempt clause (b). |
| `address.go` derives only sortedmulti + singlesig | `address/address.go:95-155` | **No.** Rust derives far more. | **No** — `address/address.go` is **upstream** (`git log upstream/main -- address/address.go` → `3233c94`), exempt clause (b). |
| Go has **no** `Descriptor → template string` renderer at all | `md/` has `template_strip.go`, `template_guard.go`, `md.Template` (`md/md.go:1205-1226`) — no renderer | N/A — Rust has one, and it is **defective for `v:` chains** (R1) and **divergent for `l:`/`u:`** (R2). | **Yes.** Porting today's `render.rs` would port the defect. R1 + R2 must land first. |

Per the rule's mandatory clause — "whenever a defect is found in a Go port we MUST always check whether the same defect exists in the primary Rust implementation" — I ran that check for every row above. The one that came back *positive* is `sortedmulti_a`; the one that came back *worse in Rust* is the renderer.

---

## Open / could not determine

- **`t:X` (= `and_v(X,1)`) rendering.** `Tag::True` exists (`tag.rs:88`, code `0x23`) and `render.rs:220-223` emits `1`, so the shape is representable. I did not get a well-typed `t:` case through the md-cli parser (`t:older(144)` fails typecheck — `older` is type B and `and_v` needs V first — which is correct rust-miniscript behaviour, not an md defect). **I could not determine** whether md renders a genuine `t:`-shaped AST as `t:X` or as `and_v(X,1)`. Given the confirmed `l:`/`u:` desugaring, `and_v(X,1)` is the likely behaviour, but I am not asserting it — it should be measured when R2 is worked.
- **`md verify`'s behaviour on non-canonical shapes.** It has no `--path`, so I could not exercise it against any miniscript shape; my attempts produced payload-length mismatches attributable to the missing origin, not to a verify defect. Left unresolved deliberately rather than guessed.
- **Whether BIP-388 normatively requires `sortedmulti_a`.** The repo vendors BIP text at `seedhammer/address/testdata/bips/` but that set is `bip-0067/0084/0086/0143/0383` — **BIP-388 itself is not vendored anywhere in the three repos** (only prose references in design docs). I did not fetch external text, so I state the gap as "Rust can render a shape it cannot derive", which is defect enough on its own, and **do not** assert what BIP-388 mandates.
- **Whether the R2 `l:`/`u:` divergence should resolve toward miniscript's sugar or toward md's desugared form.** That is a design decision (screen width vs. cross-tool comparability), explicitly out of scope for this recon.
- **Go-side rendering breadth beyond the three cited call sites.** I verified the fork has no renderer and checked the three sites named in the brief; I did not sweep `gui/` exhaustively for other display paths that might already stringify a template.
